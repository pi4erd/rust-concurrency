use std::collections::{HashMap, VecDeque};

use cgmath::{MetricSpace, Zero};
use chunk::{Chunk, ChunkCoord, CHUNK_SIZE};
use fastnoise_lite::FastNoiseLite;
use meshgen::generate_model;
use noise::{Fbm, NoiseFn};
use voxel::{Blocks, Voxel};

use super::{camera::Camera, debug::{DebugDrawer, ModelName}, draw::{Drawable, Model}, mesh::Mesh};

pub mod chunk;
pub mod voxel;
pub mod meshgen;

pub trait Generator {
    fn generate(&self, _chunk: &mut Chunk) {}
}

struct NoiseSampler {
    // noise: Fbm<noise::Simplex>,
    noise: FastNoiseLite,
}

impl NoiseSampler {
    pub fn new(seed: i32) -> Self {
        Self {
            noise: FastNoiseLite::with_seed(seed),
        }
    }

    pub fn sample(&self, chunk_coord: ChunkCoord, x: f32, y: f32, z: f32, scale: f32) -> f32 {
        self.noise.get_noise_3d(
            ((chunk_coord.x as f32 * CHUNK_SIZE.0 as f32) + x) * scale,
            ((chunk_coord.y as f32 * CHUNK_SIZE.1 as f32) + y) * scale,
            ((chunk_coord.z as f32 * CHUNK_SIZE.2 as f32) + z) * scale,
        )
    }
}

pub struct NoiseGenerator {
    sampler: NoiseSampler,
}

impl NoiseGenerator {
    pub fn new(seed: i32) -> Self {
        Self {
            sampler: NoiseSampler::new(seed),
        }
    }
}

impl Generator for NoiseGenerator {
    fn generate(&self, chunk: &mut Chunk) {
        const SCALE: f32 = 2.0;

        for x in 0..CHUNK_SIZE.0 {
            for z in 0..CHUNK_SIZE.2 {
                for y in 0..CHUNK_SIZE.1 {
                    let sample = self.sampler.sample(
                        chunk.coord,
                        x as f32, y as f32, z as f32,
                        SCALE
                    ) * 0.5 + 0.5;

                    let small_sample = self.sampler.sample(
                        chunk.coord,
                        x as f32, y as f32, z as f32,
                        SCALE * 4.0
                    ) * 0.5 + 0.5;

                    if small_sample < 0.3 {
                        continue;
                    }

                    if sample < 0.4 {
                        chunk.set_voxel(x, y, z, Blocks::STONE.default_state());
                    } else if sample < 0.5 {
                        chunk.set_voxel(x, y, z, Blocks::GRASS_BLOCK.default_state());
                    }
                }
            }
        }
    }
}

type Queue<T> = VecDeque<T>;

pub struct World<T> {
    generator: T,
    chunks: HashMap<ChunkCoord, Chunk>,
    models: HashMap<ChunkCoord, Model<Mesh>>,
    
    chunk_gen_queue: Queue<ChunkCoord>,
    meshgen_queue: Queue<ChunkCoord>,
}

impl<T> World<T> {
    pub fn new(generator: T) -> Self {
        Self {
            generator,
            chunks: HashMap::new(),
            models: HashMap::new(),
            chunk_gen_queue: Queue::new(),
            meshgen_queue: Queue::new(),
        }
    }

    pub fn get_voxel(&self, mut chunk: ChunkCoord, mut position: (i32, i32, i32)) -> Option<Voxel> {
        if position.0 < 0 || position.0 >= CHUNK_SIZE.0 as i32 {
            chunk.x += if position.0 < 0 { -1 } else { 1 };
            position.0 += if position.0 < 0 { CHUNK_SIZE.0 as i32 } else { -(CHUNK_SIZE.0 as i32) };
        }
        if position.1 < 0 || position.1 >= CHUNK_SIZE.1 as i32 {
            chunk.y += if position.1 < 0 { -1 } else { 1 };
            position.1 += if position.1 < 0 { CHUNK_SIZE.1 as i32 } else { -(CHUNK_SIZE.1 as i32) };
        }
        if position.2 < 0 || position.2 >= CHUNK_SIZE.2 as i32 {
            chunk.z += if position.2 < 0 { -1 } else { 1 };
            position.2 += if position.2 < 0 { CHUNK_SIZE.2 as i32 } else { -(CHUNK_SIZE.2 as i32) };
        }

        let chunk = &self.chunks.get(&chunk)?;
        chunk.get_voxel(
            position.0 as usize,
            position.1 as usize,
            position.2 as usize,
        )
    }

    pub fn enqueue_chunks_around(&mut self, camera: &Camera, distance: usize) {
        let distance = distance as i32;
        let center = ChunkCoord::from_world(cgmath::Vector3::new(
            camera.eye.x,
            camera.eye.y,
            camera.eye.z,
        ));
        
        for i in -distance..=distance {
            for j in -distance..=distance {
                for k in -distance..=distance {
                    let chunk = center + ChunkCoord {
                        x: i,
                        y: j,
                        z: k,
                    };
                    self.enqueue_chunk_gen(chunk);
                }
            }
        }
    }

    // To be run on a separate thread
    pub fn generate_chunk(&mut self, coord: ChunkCoord) where T: Generator {
        self.chunks.insert(coord, Chunk::new(coord));
        self.generator.generate(self.chunks.get_mut(&coord).unwrap());

        self.meshgen_queue.push_back(coord);
    }

    pub fn dequeue_chunk_gen(&mut self) where T: Generator {
        if let Some(chunk) = self.chunk_gen_queue.pop_front() {
            self.generate_chunk(chunk);
        }
    }

    pub fn enqueue_chunk_gen(&mut self, coord: ChunkCoord) {
        if self.chunks.contains_key(&coord) || self.chunk_gen_queue.contains(&coord) {
            return;
        }

        self.chunk_gen_queue.push_back(coord);
    }

    pub fn dequeue_meshgen(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bg_layout: &wgpu::BindGroupLayout,
    ) {
        if let Some(coord) = self.meshgen_queue.pop_front() {
            // TODO: Check if all neighbors present, otherwise not draw
            if !(self.chunks.contains_key(&coord.left()) &&
                self.chunks.contains_key(&coord.right()) &&
                self.chunks.contains_key(&coord.front()) &&
                self.chunks.contains_key(&coord.back()) &&
                self.chunks.contains_key(&coord.up()) &&
                self.chunks.contains_key(&coord.down()))
            {
                self.meshgen_queue.push_back(coord);
                return; // put back and try next time
            }

            let model = generate_model(
                device,
                bg_layout,
                &self.chunks[&coord],
                self
            );

            if let Some(model) = model {
                self.models.insert(
                    coord,
                    model
                );
                self.models[&coord].update_buffer(queue);
            }
        }
    }

    pub fn append_debug(
        &self,
        debug: &mut DebugDrawer,
        observer: &Camera,
        model_bg_layout: &wgpu::BindGroupLayout,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        let chunk_coord = ChunkCoord::from_world(cgmath::Vector3::new(
            observer.eye.x,
            observer.eye.y,
            observer.eye.z,
        ));

        if !self.chunks.contains_key(&chunk_coord) {
            return;
        }

        let position = chunk_coord.to_world();

        let scale = cgmath::Vector3::new(
            CHUNK_SIZE.0 as f32,
            CHUNK_SIZE.1 as f32,
            CHUNK_SIZE.2 as f32,
        );

        debug.append_model(
            ModelName::Cube,
            device,
            queue,
            model_bg_layout,
            position,
            cgmath::Quaternion::zero(),
            scale,
            [1.0, 0.0, 1.0]
        );
    }
}

impl<T: Generator> Drawable for World<T> {
    fn draw(&self, render_pass: &mut wgpu::RenderPass) {
        for (_coord, model) in self.models.iter() {
            // log::debug!("Drawing chunk at {}", coord);
            model.draw(render_pass);
        }
    }
}
