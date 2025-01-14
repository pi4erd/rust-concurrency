pub mod chunk;
pub mod voxel;
pub mod meshgen;

use std::{
    collections::{HashMap, VecDeque},
    sync::{mpsc::{self, Receiver, Sender}, Arc, Mutex},
    thread::{self, JoinHandle}
};

use cgmath::Zero;
use fastnoise_lite::FastNoiseLite;

use chunk::{Chunk, ChunkCoord, CHUNK_SIZE};
use meshgen::generate_model;
use voxel::{Blocks, Voxel};

use super::{
    camera::Camera,
    debug::{DebugDrawer, ModelName},
    draw::{Drawable, Model},
    mesh::Mesh
};

pub trait Generator: Sync + Send {
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
                let sample = self.sampler.sample(
                    chunk.coord,
                    x as f32, 0.0, z as f32,
                    SCALE
                ) * 0.5 + 0.5;

                for y in 0..CHUNK_SIZE.1 {
                    let wy = chunk.coord.y as i32 * CHUNK_SIZE.1 as i32 + y as i32;
                    if wy < sample as i32 {
                        chunk.set_voxel(x, y, z, Blocks::STONE.default_state());
                    }

                    let flying_islands = self.sampler.sample(
                        chunk.coord,
                        x as f32, y as f32, z as f32,
                        SCALE * 0.8
                    );
                    let flying_islands = flying_islands * (wy as f32 / CHUNK_SIZE.1 as f32);

                    if flying_islands > 2.0 {
                        chunk.set_voxel(x, y, z, Blocks::STONE.default_state());
                    }
                }
            }
        }
    }
}

type Queue<T> = VecDeque<T>;

pub struct World<T> {
    generator: Arc<T>,
    chunks: HashMap<ChunkCoord, Chunk>,
    models: HashMap<ChunkCoord, Model<Mesh>>,
    
    chunk_gen_queue: Arc<Mutex<Queue<ChunkCoord>>>,
    meshgen_queue: Queue<ChunkCoord>,
    world_gen_threads: Vec<JoinHandle<()>>,

    receiver: Receiver<Chunk>,
    sender: Sender<Chunk>,
}

impl<T> World<T> {
    pub fn new(generator: T) -> Self {
        let (tx, rx) = mpsc::channel::<Chunk>();
        Self {
            generator: Arc::new(generator),
            chunks: HashMap::new(),
            models: HashMap::new(),
            chunk_gen_queue: Arc::new(Mutex::new(Queue::new())),
            meshgen_queue: Queue::new(),
            world_gen_threads: Vec::new(),

            receiver: rx,
            sender: tx,
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

    pub fn dispatch_threads(&mut self, n: usize) where T: 'static + Generator {
        for _ in 0..n {
            let tx = self.sender.clone();
            let chunk_gen_queue = self.chunk_gen_queue.clone();
            let generator = self.generator.clone();
            self.world_gen_threads.push(thread::spawn(move || {
                loop {
                    let chunk_to_generate: Option<ChunkCoord>;
                    {
                        chunk_to_generate = chunk_gen_queue.lock().unwrap().pop_front()
                    }
    
                    if let Some(chunk_to_generate) = chunk_to_generate {
                        let mut chunk = Chunk::new(chunk_to_generate);
                        generator.generate(&mut chunk);
    
                        tx.send(chunk).unwrap();
                    } else {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                }
            }));
        }
    }

    pub fn enqueue_chunk_gen(&mut self, coord: ChunkCoord) {
        if self.chunks.contains_key(&coord) || self.chunk_gen_queue.lock().unwrap().contains(&coord) {
            return;
        }

        self.chunk_gen_queue.lock().unwrap().push_back(coord);
    }

    pub fn receive_chunk(&mut self, millis: u64) {
        let recv = self.receiver.recv_timeout(std::time::Duration::from_millis(millis));
        if let Ok(chunk) = recv {
            let coord = chunk.coord;
            self.chunks.insert(coord, chunk);
            self.meshgen_queue.push_back(coord);
        }
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
