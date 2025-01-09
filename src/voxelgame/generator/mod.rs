use std::collections::{HashMap, VecDeque};

use cgmath::{MetricSpace, Zero};
use chunk::{Chunk, ChunkCoord, CHUNK_SIZE};
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
    noise: Fbm<noise::Simplex>,
}

impl NoiseSampler {
    pub fn new(seed: u32) -> Self {
        Self {
            noise: Fbm::new(seed),
        }
    }

    pub fn sample(&self, chunk_coord: ChunkCoord, x: f64, y: f64, z: f64, scale: f64) -> f64 {
        self.noise.get([
            ((chunk_coord.x as f64 * CHUNK_SIZE.0 as f64) + x) * scale,
            ((chunk_coord.y as f64 * CHUNK_SIZE.1 as f64) + y) * scale,
            ((chunk_coord.z as f64 * CHUNK_SIZE.2 as f64) + z) * scale,
        ])
    }
}

pub struct NoiseGenerator {
    sampler: NoiseSampler,
}

impl NoiseGenerator {
    pub fn new(seed: u32) -> Self {
        Self {
            sampler: NoiseSampler::new(seed),
        }
    }
}

impl Generator for NoiseGenerator {
    fn generate(&self, chunk: &mut Chunk) {
        const SCALE: f64 = 0.05;

        for x in 0..CHUNK_SIZE.0 {
            for z in 0..CHUNK_SIZE.2 {
                let sample = (self.sampler.sample(
                    chunk.coord,
                    x as f64, 0.0, z as f64,
                    SCALE
                ) / 2.0 + 0.5) * 30.0 + 10.0;

                for y in 0..sample as usize {
                    chunk.set_voxel(x, y, z, Blocks::STONE.default_state());
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
    chunks_to_generate: Queue<ChunkCoord>,
}

impl<T> World<T> {
    pub fn new(generator: T) -> Self {
        Self {
            generator,
            chunks: HashMap::new(),
            models: HashMap::new(),
            chunks_to_generate: Queue::new(),
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

    // To be run on a separate thread
    pub fn generate_chunk(&mut self, coord: ChunkCoord) where T: Generator {
        if self.chunks.contains_key(&coord) {
            return;
        }

        self.chunks.insert(coord, Chunk::new(coord));
        self.generator.generate(self.chunks.get_mut(&coord).unwrap());

        self.chunks_to_generate.push_back(coord);
    }

    pub fn dequeue_meshgen(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bg_layout: &wgpu::BindGroupLayout,
    ) {
        if let Some(coord) = self.chunks_to_generate.pop_front() {
            // TODO: Check if all neighbors present, otherwise not draw
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
        for (coord, _) in self.models.iter() {
            let position: cgmath::Vector3<f32> = coord.to_world();
            let point: cgmath::Point3<f32> = cgmath::Point3::new(
                position.x,
                position.y,
                position.z
            );

            if point.distance(observer.eye) > 80.0 {
                continue;
            }
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
}

impl<T: Generator> Drawable for World<T> {
    fn draw(&self, render_pass: &mut wgpu::RenderPass) {
        for (_coord, model) in self.models.iter() {
            // log::debug!("Drawing chunk at {}", coord);
            model.draw(render_pass);
        }
    }
}
