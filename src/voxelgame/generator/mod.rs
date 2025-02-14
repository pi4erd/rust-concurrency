pub mod chunk;
pub mod voxel;
pub mod meshgen;

use std::{
    collections::{HashMap, HashSet, VecDeque}, hash::Hasher, sync::{mpsc::{self, Receiver, Sender}, Arc, Mutex}, thread::{self, JoinHandle}
};

use cgmath::{EuclideanSpace, MetricSpace};
use fastnoise_lite::FastNoiseLite;

use chunk::{Chunk, ChunkCoord, ChunkLocalCoord, WorldCoord, CHUNK_SIZE};
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
                let height_sample = 40.0 + self.sampler.sample(
                    ChunkCoord { x: chunk.coord.x, y: 0, z: chunk.coord.z},
                    x as f32,
                    0.0,
                    z as f32,
                    SCALE
                ) * 90.0;

                for y in 0..CHUNK_SIZE {
                    let wy = chunk.coord.y as i32 * CHUNK_SIZE as i32 + y as i32;

                    let coord = ChunkLocalCoord {
                        x, y, z,
                    };

                    if wy <= height_sample as i32 {
                        let cave_sample = self.sampler.sample(
                            chunk.coord,
                            x as f32, y as f32, z as f32,
                            SCALE * 4.0
                        );

                        let caviness = self.sampler.sample(
                            chunk.coord,
                            x as f32, y as f32, z as f32,
                            SCALE * 0.3
                        ) * 0.5 + 0.5;
    
                        if cave_sample * (1.0 - caviness) < -0.23 {
                            continue;
                        }

                        if wy - height_sample as i32 >= 0 {
                            chunk.set_voxel(coord, Blocks::GRASS_BLOCK.default_state());
                        } else if wy - height_sample as i32 >= -2 {
                            chunk.set_voxel(coord, Blocks::DIRT_BLOCK.default_state());
                        } else {
                            chunk.set_voxel(coord, Blocks::STONE.default_state());
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

    pub fn get_voxel(&self, position: WorldCoord) -> Option<Voxel> {
        let chunk_coord: ChunkCoord = position.into();
        let local_coord: ChunkLocalCoord = position.into();

        let chunk = &self.chunks.get(&chunk_coord)?;
        chunk.get_voxel(local_coord)
    }

    pub fn break_block(&mut self, position: WorldCoord) {
        let chunk_coord: ChunkCoord = position.into();
        let local_coord: ChunkLocalCoord = position.into();

        let chunk = self.chunks.get_mut(&chunk_coord);

        if let Some(chunk) = chunk {
            chunk.set_voxel(
                local_coord,
                Blocks::AIR.default_state(),
            );

            if let None = local_coord.left() {
                self.meshgen_queue.lock().unwrap().push_front(chunk.coord.left());
            }

            if let None = local_coord.right() {
                self.meshgen_queue.lock().unwrap().push_front(chunk.coord.right());
            }

            if let None = local_coord.up() {
                self.meshgen_queue.lock().unwrap().push_front(chunk.coord.up());
            }

            if let None = local_coord.down() {
                self.meshgen_queue.lock().unwrap().push_front(chunk.coord.down());
            }

            if let None = local_coord.front() {
                self.meshgen_queue.lock().unwrap().push_front(chunk.coord.front());
            }

            if let None = local_coord.back() {
                self.meshgen_queue.lock().unwrap().push_front(chunk.coord.back());
            }

            self.meshgen_queue.lock().unwrap().push_front(chunk.coord);

            log::info!("Broken block at {:?}", position);
        }
    }

    pub fn enqueue_chunks_around(&mut self, camera: &Camera, height: usize, distance: usize) {
        let height = height as i32;
        let distance = distance as i32;

        let world_coord: WorldCoord = camera.eye.to_vec().into();
        let center = ChunkCoord::from(world_coord);
        
        for i in -(height / 2)..=height / 2 {
            for j in -distance..=distance {
                for k in -distance..=distance {
                    let chunk = center + ChunkCoord {
                        x: k as i16,
                        y: i as i16,
                        z: j as i16,
                    };
                    
                    self.enqueue_chunk(chunk);
                }
            }
        }
    }

    pub fn enqueue_chunk(&mut self, chunk_coord: ChunkCoord) {
        if !self.sent_chunks.lock().unwrap().insert(chunk_coord) {
            return;
        }

        let mut queue = self.chunk_gen_queue.lock().unwrap();
        if queue.len() >= Self::MAX_QUEUE_SIZE {
            log::warn!("Queue limit reached!");
            queue.clear();
        }

        queue.push_back(chunk_coord);
    }

    pub fn chunks_enqueued_count(&self) -> usize {
        self.chunk_gen_queue.lock().unwrap().len()
    }

    pub fn meshgen_queue_count(&self) -> usize {
        self.meshgen_queue.lock().unwrap().len()
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

    pub fn ray_hit(&self, ray: Ray, mut debug: Option<&mut DebugDrawer>) -> Option<(WorldCoord, Voxel)> {
        const MAX_DISTANCE: f32 = 32.0;

        let mut distance = 0.0;

        while distance < MAX_DISTANCE {
            let point = ray.origin + ray.direction * distance;

            let world_coord: WorldCoord = point.to_vec().into();

            if let Some(debug) = debug.as_mut() {
                let world_coord_f32: cgmath::Vector3<f32> = world_coord.into();
                debug.append_mesh(
                    ModelName::Cube,
                    world_coord_f32 - cgmath::Vector3::new(0.05, 0.05, 0.05),
                    cgmath::Vector3::new(0.1, 0.1, 0.1),
                    cgmath::Vector4::new(1.0, 0.0, 1.0, 0.3),
                );
            }

            let result = self.get_voxel(world_coord);

            distance += 0.1;

            if let Some(voxel) = result {
                if !Blocks::BLOCKS[voxel.id as usize].solid {
                    continue;
                }

                return Some((
                    world_coord,
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

    // Returns a number of chunks drawn
    pub fn draw_distance(&self, render_pass: &mut wgpu::RenderPass, eye: cgmath::Vector3<f32>, max_chunks: usize) -> usize {
        let mut count = 0;
        for (coord, model) in self.models.iter() {
            let position: cgmath::Vector3<f32> = (*coord).into(); // rust being weird
            if position.distance(eye) > (max_chunks as f32 * CHUNK_SIZE as f32) {
                continue;
            }
            // log::debug!("Drawing chunk at {}", coord);
            model.draw(render_pass);
            count += 1;
        }
        count
    }

    pub fn append_debug(
        &self,
        debug: &mut DebugDrawer,
    ) {
        for (coord, _) in self.models.iter() {
            let position = (*coord).into(); // rust being weird again

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
