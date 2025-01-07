use std::collections::{HashMap, VecDeque};

use chunk::{Chunk, ChunkCoord, CHUNK_SIZE};
use meshgen::generate_model;
use noise::{Fbm, NoiseFn};
use voxel::Blocks;

use super::{draw::{Drawable, Model}, mesh::Mesh};

pub mod chunk;
pub mod voxel;
pub mod meshgen;

pub trait Generator {
    fn generate(&self, _chunk: &mut Chunk) {}
}

struct NoiseSampler {
    noise: Fbm<noise::SuperSimplex>,
}

impl NoiseSampler {
    pub fn new(seed: u32) -> Self {
        Self {
            noise: Fbm::new(seed),
        }
    }

    pub fn sample(&self, chunk_coord: ChunkCoord, x: f64, y: f64, z: f64) -> f64 {
        self.noise.get([
            (chunk_coord.x * CHUNK_SIZE.0 as i32) as f64 + x,
            (chunk_coord.y * CHUNK_SIZE.1 as i32) as f64 + y,
            (chunk_coord.z * CHUNK_SIZE.2 as i32) as f64 + z,
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
        const SCALE: f64 = 0.02;
        for x in 0..CHUNK_SIZE.0 {
            for z in 0..CHUNK_SIZE.1 {
                for y in 0..CHUNK_SIZE.2 {
                    let v = self.sampler.sample(
                        chunk.coord,
                        x as f64 * SCALE,
                        y as f64 * SCALE,
                        z as f64 * SCALE,
                    );

                    if v < 0.3 {
                        chunk.set_voxel(x, y, z, Blocks::STONE.default_state());
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

    // To be run on a separate thread
    pub fn generate_chunk(&mut self, coord: ChunkCoord) where T: Generator {
        if self.chunks.contains_key(&coord) {
            return;
        }

        self.chunks.insert(coord, Chunk::new(coord));
        self.generator.generate(&mut self.chunks.get_mut(&coord).unwrap());

        self.chunks_to_generate.push_back(coord);
    }

    pub fn dequeue_meshgen(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bg_layout: &wgpu::BindGroupLayout,
    ) {
        if let Some(coord) = self.chunks_to_generate.pop_front() {
            let model = generate_model(device, bg_layout, &self.chunks[&coord]);

            if let Some(model) = model {
                self.models.insert(
                    coord,
                    model
                );
                self.models[&coord].update_buffer(queue);
            }
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
