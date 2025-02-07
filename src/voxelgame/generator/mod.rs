pub mod chunk;
pub mod voxel;
pub mod meshgen;

use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::{mpsc::{self, Receiver, Sender}, Arc, Mutex},
    thread::{self, JoinHandle}
};

use cgmath::{EuclideanSpace, MetricSpace};
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
            ((chunk_coord.x as f32 * CHUNK_SIZE as f32) + x) * scale,
            ((chunk_coord.y as f32 * CHUNK_SIZE as f32) + y) * scale,
            ((chunk_coord.z as f32 * CHUNK_SIZE as f32) + z) * scale,
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
        const SCALE: f32 = 0.3;

        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let height_sample = 5.0 + self.sampler.sample(
                    ChunkCoord { x: chunk.coord.x, y: 0, z: chunk.coord.z},
                    x as f32,
                    0.0,
                    z as f32,
                    SCALE
                ) * 30.0;

                for y in 0..CHUNK_SIZE {
                    let wy = chunk.coord.y as i32 * CHUNK_SIZE as i32 + y as i32;

                    if wy <= height_sample as i32 {
                        let cave_sample = self.sampler.sample(
                            chunk.coord,
                            x as f32, y as f32, z as f32,
                            SCALE * 4.0
                        );

                        let caviness = self.sampler.sample(
                            chunk.coord,
                            x as f32, y as f32, z as f32,
                            SCALE * 2.0
                        ) * 0.5 + 0.5 + wy as f32 * 0.1;
    
                        if cave_sample * (1.0 - caviness) < -0.2 {
                            continue;
                        }

                        if wy - height_sample as i32 >= -1 {
                            chunk.set_voxel(x, y, z, Blocks::GRASS_BLOCK.default_state());
                        } else {
                            chunk.set_voxel(x, y, z, Blocks::STONE.default_state());
                        }
                    }
                }
            }
        }
    }
}

pub struct Ray {
    pub origin: cgmath::Point3<f32>,
    pub direction: cgmath::Vector3<f32>,
}

type Queue<T> = VecDeque<T>;

pub struct World<T> {
    generator: Arc<T>,
    chunks: HashMap<ChunkCoord, Box<Chunk>>,
    models: HashMap<ChunkCoord, Model<Mesh>>,
    
    chunk_gen_queue: Arc<Mutex<Queue<ChunkCoord>>>,
    meshgen_queue: Arc<Mutex<Queue<ChunkCoord>>>,
    world_gen_threads: Vec<JoinHandle<()>>,
    sent_chunks: Arc<Mutex<HashSet<ChunkCoord>>>,

    receiver: Receiver<Box<Chunk>>,
    sender: Sender<Box<Chunk>>,
}

impl<T> World<T> {
    pub const MAX_QUEUE_SIZE: usize = 4096;

    pub fn new(generator: T) -> Self {
        let (tx, rx) = mpsc::channel::<Box<Chunk>>();
        Self {
            generator: Arc::new(generator),
            chunks: HashMap::new(),
            models: HashMap::new(),
            chunk_gen_queue: Arc::new(Mutex::new(Queue::new())),
            
            meshgen_queue: Arc::new(Mutex::new(Queue::new())),
            world_gen_threads: Vec::new(),
            sent_chunks: Arc::new(Mutex::new(HashSet::new())),

            receiver: rx,
            sender: tx,
        }
    }

    pub fn reset(&mut self) {
        let mut sent = self.sent_chunks.lock().unwrap();
        let mut genqueue = self.chunk_gen_queue.lock().unwrap();
        let mut meshqueue = self.meshgen_queue.lock().unwrap();

        sent.clear();
        genqueue.clear();
        meshqueue.clear();

        self.chunks.clear();
        self.models.clear();
    }

    pub fn get_voxel(&self, mut chunk: ChunkCoord, mut position: (i32, i32, i32)) -> Option<Voxel> {
        if position.0 < 0 || position.0 >= CHUNK_SIZE as i32 {
            chunk.x += if position.0 < 0 { -1 } else { 1 };
            position.0 += if position.0 < 0 { CHUNK_SIZE as i32 } else { -(CHUNK_SIZE as i32) };
        }
        if position.1 < 0 || position.1 >= CHUNK_SIZE as i32 {
            chunk.y += if position.1 < 0 { -1 } else { 1 };
            position.1 += if position.1 < 0 { CHUNK_SIZE as i32 } else { -(CHUNK_SIZE as i32) };
        }
        if position.2 < 0 || position.2 >= CHUNK_SIZE as i32 {
            chunk.z += if position.2 < 0 { -1 } else { 1 };
            position.2 += if position.2 < 0 { CHUNK_SIZE as i32 } else { -(CHUNK_SIZE as i32) };
        }

        let chunk = &self.chunks.get(&chunk)?;
        chunk.get_voxel(
            position.0 as usize,
            position.1 as usize,
            position.2 as usize,
        )
    }

    pub fn break_block(&mut self, mut chunk: ChunkCoord, mut position: (i32, i32, i32)) {
        if position.0 < 0 || position.0 >= CHUNK_SIZE as i32 {
            chunk.x += if position.0 < 0 { -1 } else { 1 };
            position.0 += if position.0 < 0 { CHUNK_SIZE as i32 } else { -(CHUNK_SIZE as i32) };
        }
        if position.1 < 0 || position.1 >= CHUNK_SIZE as i32 {
            chunk.y += if position.1 < 0 { -1 } else { 1 };
            position.1 += if position.1 < 0 { CHUNK_SIZE as i32 } else { -(CHUNK_SIZE as i32) };
        }
        if position.2 < 0 || position.2 >= CHUNK_SIZE as i32 {
            chunk.z += if position.2 < 0 { -1 } else { 1 };
            position.2 += if position.2 < 0 { CHUNK_SIZE as i32 } else { -(CHUNK_SIZE as i32) };
        }

        let chunk = self.chunks.get_mut(&chunk);

        if let Some(chunk) = chunk {
            chunk.set_voxel(
                position.0 as usize,
                position.1 as usize,
                position.2 as usize,
                Blocks::AIR.default_state(),
            );

            self.meshgen_queue.lock().unwrap().push_front(chunk.coord);

            log::info!("Broken block at {:?}", position);
        }
    }


    pub fn enqueue_chunks_around(&mut self, camera: &Camera, distance: usize) {
        // TODO: Improve this function for more dynamic generation
        let distance = distance as i32;
        let center = ChunkCoord::from_world(cgmath::Vector3::new(
            camera.eye.x,
            camera.eye.y,
            camera.eye.z,
        ));
        
        let mut queue = self.chunk_gen_queue.lock().unwrap();
        let mut set = self.sent_chunks.lock().unwrap();
        for i in -distance..=distance {
            for j in -distance..=distance {
                for k in -distance..=distance {
                    let chunk = center + ChunkCoord {
                        x: k,
                        y: -i,
                        z: j,
                    };
                    
                    if self.chunks.contains_key(&chunk) || set.contains(&chunk) {
                        continue;
                    }

                    if queue.len() >= Self::MAX_QUEUE_SIZE {
                        log::warn!("Queue limit reached!");
                        queue.clear();
                    }

                    set.insert(chunk);
                    queue.push_back(chunk);
                }
            }
        }
    }

    pub fn dispatch_threads(&mut self, worldgen: usize) where T: 'static + Generator {
        for _ in 0..worldgen {
            let tx = self.sender.clone();
            let chunk_gen_queue = self.chunk_gen_queue.clone();
            let generator = self.generator.clone();
            self.world_gen_threads.push(thread::spawn(move || {
                loop {
                    let chunk_to_generate: Option<ChunkCoord> = {
                        let mut queue_lock = chunk_gen_queue.lock().unwrap();
                        queue_lock.pop_front()
                    };
    
                    if let Some(chunk_to_generate) = chunk_to_generate {
                        let mut chunk = Box::new(Chunk::new(chunk_to_generate));
                        generator.generate(&mut chunk);
    
                        log::info!("Generated chunk {}", chunk_to_generate);

                        // NOTE: Previous issue here is fixed by Box
                        tx.send(chunk).expect("Channel was closed");
                    }
                }
            }));
        }
    }

    pub fn receive_chunk(&mut self) {
        let recv = self.receiver.try_recv();
        if let Ok(chunk) = recv {
            let coord = chunk.coord;
            self.chunks.insert(coord, chunk);
            self.meshgen_queue.lock().unwrap().push_back(coord);
        }
    }

    pub fn ray_hit(&self, ray: Ray, mut debug: Option<&mut DebugDrawer>) -> Option<(cgmath::Vector3<i32>, Voxel)> {
        const MAX_DISTANCE: f32 = 32.0;

        let mut distance = 0.0;

        while distance < MAX_DISTANCE {
            let point = ray.origin + ray.direction * distance;

            let vec = point.to_vec();
            let chunk_coord = ChunkCoord::from_world(vec);
            let local_coord = vec - chunk_coord.to_world();

            let mut global_block_coord = cgmath::Vector3::new(
                vec.x as i32,
                vec.y as i32,
                vec.z as i32,
            );

            if chunk_coord.x < 0 {
                global_block_coord.x -= 1;
            }
            if chunk_coord.y < 0 {
                global_block_coord.y -= 1;
            }
            if chunk_coord.z < 0 {
                global_block_coord.z -= 1;
            }

            if let Some(debug) = debug.as_mut() {
                debug.append_mesh(
                    ModelName::Cube,
                    vec - cgmath::Vector3::new(0.05, 0.05, 0.05),
                    cgmath::Vector3::new(0.1, 0.1, 0.1),
                    cgmath::Vector4::new(1.0, 0.0, 1.0, 0.3),
                );
            }

            let result = self.get_voxel(
                chunk_coord,
                (
                    local_coord.x as i32,
                    local_coord.y as i32,
                    local_coord.z as i32,
                )
            );

            distance += 0.1;

            if let Some(voxel) = result {
                if !Blocks::BLOCKS[voxel.id as usize].solid {
                    continue;
                }

                return Some((
                    global_block_coord,
                    voxel
                ));
            }
        }

        None
    }

    pub fn dequeue_meshgen(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bg_layout: &wgpu::BindGroupLayout,
    ) {
        let coord: Option<ChunkCoord>;
        {
            coord = self.meshgen_queue.lock().unwrap().pop_front();
        }

        if let Some(coord) = coord {
            if !(self.chunks.contains_key(&coord.left()) &&
                self.chunks.contains_key(&coord.right()) &&
                self.chunks.contains_key(&coord.front()) &&
                self.chunks.contains_key(&coord.back()) &&
                self.chunks.contains_key(&coord.up()) &&
                self.chunks.contains_key(&coord.down()))
            {
                self.meshgen_queue.lock().unwrap().push_back(coord);
                return; // put back and try next time
            }

            let model = generate_model(
                device,
                bg_layout,
                &self.chunks[&coord],
                self
            );

            if let Some(model) = model {
                _ = self.models.insert(
                    coord,
                    model
                );
                self.models[&coord].update_buffer(queue);
            }
        }
    }

    pub fn draw_distance(&self, render_pass: &mut wgpu::RenderPass, eye: cgmath::Vector3<f32>, max_chunks: usize) {
        for (coord, model) in self.models.iter() {
            if coord.to_world().distance(eye) > (max_chunks as f32 * CHUNK_SIZE as f32) {
                continue;
            }
            // log::debug!("Drawing chunk at {}", coord);
            model.draw(render_pass);
        }
    }

    pub fn append_debug(
        &self,
        debug: &mut DebugDrawer,
    ) {
        for (coord, _) in self.models.iter() {
            let position = coord.to_world();

            let scale = cgmath::Vector3::new(
                CHUNK_SIZE as f32,
                CHUNK_SIZE as f32,
                CHUNK_SIZE as f32,
            );
            
            debug.append_mesh(
                ModelName::Cube,
                position,
                scale,
                [1.0, 0.0, 1.0, 0.0].into()
            );
        }
    }
}
