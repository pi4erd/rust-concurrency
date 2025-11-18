pub mod chunk;
pub mod meshgen;
pub mod voxel;

use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use cgmath::{EuclideanSpace, MetricSpace};
use fastnoise_lite::FastNoiseLite;

use chunk::{Chunk, ChunkCoord, ChunkLocalCoord, WorldCoord, CHUNK_SIZE};
use rand::Rng;
use voxel::{Blocks, Voxel};

use crate::voxelgame::{
    generator::{chunk::BlockOffsetCoord, meshgen::generate_mesh_lod},
    mesh::{MeshInfo, Vertex3d},
};

use super::{
    camera::Camera,
    debug::{DebugDrawer, ModelName},
    draw::{Drawable, Model},
    mesh::Mesh,
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

        let mut rng = rand::rng();

        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let height_sample = 40.0
                    + self.sampler.sample(
                        ChunkCoord {
                            x: chunk.coord.x,
                            y: 0,
                            z: chunk.coord.z,
                        },
                        x as f32,
                        0.0,
                        z as f32,
                        SCALE,
                    ) * 90.0;

                for y in 0..CHUNK_SIZE {
                    let wy = chunk.coord.y as i32 * CHUNK_SIZE as i32 + y as i32;

                    let coord = ChunkLocalCoord { x, y, z };

                    if wy <= height_sample as i32 {
                        let cave_sample = self.sampler.sample(
                            chunk.coord,
                            x as f32,
                            y as f32,
                            z as f32,
                            SCALE * 4.0,
                        );

                        let caviness = self.sampler.sample(
                            chunk.coord,
                            x as f32,
                            y as f32,
                            z as f32,
                            SCALE * 0.3,
                        ) * 0.5
                            + 0.5;

                        if cave_sample * (1.0 - caviness) < -0.23 {
                            continue;
                        }

                        if wy - height_sample as i32 >= 0 {
                            if rng.random_bool(0.0001) {
                                for k in 0..10 {
                                    chunk.set_voxel(
                                        ChunkLocalCoord {
                                            x: coord.x,
                                            y: coord.y + k,
                                            z: coord.z,
                                        },
                                        Blocks::LOG.default_state(),
                                    );
                                }
                            }
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

#[derive(Clone)]
pub struct WorldAccessor {
    pub chunks: Arc<Mutex<HashMap<ChunkCoord, Box<Chunk>>>>,
}

impl WorldAccessor {
    pub fn get_voxel(&self, coord: WorldCoord) -> Option<Voxel> {
        let chunk_coord: ChunkCoord = coord.into();
        let local_coord: ChunkLocalCoord = coord.into();

        let lock = &self.chunks.lock().unwrap();
        let chunk = &lock.get(&chunk_coord)?;
        chunk.get_voxel(local_coord)
    }
}

pub struct World<T> {
    generator: Arc<T>,
    chunks: Arc<Mutex<HashMap<ChunkCoord, Box<Chunk>>>>,
    world_accessor: WorldAccessor,
    models: HashMap<ChunkCoord, Model<Mesh>>,

    chunk_gen_queue: Arc<Mutex<Queue<ChunkCoord>>>,
    meshgen_queue: Arc<Mutex<Queue<ChunkCoord>>>,

    world_gen_threads: Vec<JoinHandle<()>>,
    meshgen_threads: Vec<JoinHandle<()>>,

    loaded_chunks: Arc<Mutex<HashSet<ChunkCoord>>>,
    meshed_chunks: Arc<Mutex<HashSet<ChunkCoord>>>,

    chunk_receiver: Receiver<Box<Chunk>>,
    chunk_sender: Sender<Box<Chunk>>,
    mesh_receiver: Receiver<(ChunkCoord, MeshInfo<Vertex3d>)>,
    mesh_sender: Sender<(ChunkCoord, MeshInfo<Vertex3d>)>,
}

#[allow(dead_code)]
impl<T> World<T> {
    pub fn new(generator: T) -> Self {
        let (ctx, crx) = mpsc::channel::<Box<Chunk>>();
        let (mtx, mrx) = mpsc::channel::<(ChunkCoord, MeshInfo<Vertex3d>)>();

        let chunks = Arc::new(Mutex::new(HashMap::new()));
        let world_accessor = WorldAccessor {
            chunks: chunks.clone(),
        };

        Self {
            generator: Arc::new(generator),
            chunks,
            world_accessor,

            models: HashMap::new(),
            chunk_gen_queue: Arc::new(Mutex::new(Queue::new())),

            meshgen_queue: Arc::new(Mutex::new(Queue::new())),

            world_gen_threads: Vec::new(),
            meshgen_threads: Vec::new(),

            loaded_chunks: Arc::new(Mutex::new(HashSet::new())),
            meshed_chunks: Arc::new(Mutex::new(HashSet::new())),

            chunk_receiver: crx,
            chunk_sender: ctx,
            mesh_receiver: mrx,
            mesh_sender: mtx,
        }
    }

    pub fn reset(&mut self) {
        let mut sent = self.loaded_chunks.lock().unwrap();
        let mut genqueue = self.chunk_gen_queue.lock().unwrap();
        let mut meshqueue = self.meshgen_queue.lock().unwrap();

        sent.clear();
        genqueue.clear();
        meshqueue.clear();

        self.chunks.lock().unwrap().clear();
        self.models.clear();
    }

    pub fn get_voxel(&self, position: WorldCoord) -> Option<Voxel> {
        let chunk_coord: ChunkCoord = position.into();
        let local_coord: ChunkLocalCoord = position.into();

        let lock = &self.chunks.lock().unwrap();
        let chunk = &lock.get(&chunk_coord)?;
        chunk.get_voxel(local_coord)
    }

    pub fn set_voxels_radius(&mut self, center: WorldCoord, radius: u32, block: Voxel) {
        let mut chunks_affected: HashSet<ChunkCoord> = HashSet::new();
        let radius = radius as i32;

        let mut lock = self.chunks.lock().unwrap();

        for i in -radius..radius {
            for j in -radius..radius {
                for k in -radius..radius {
                    let offset_float = (i as f32, j as f32, k as f32);
                    let dst_float = offset_float.0 * offset_float.0
                        + offset_float.1 * offset_float.1
                        + offset_float.2 * offset_float.2;

                    if dst_float > (radius * radius) as f32 {
                        continue;
                    }

                    let offset = BlockOffsetCoord { x: i, y: j, z: k };
                    let world_coord = center + offset;
                    let chunk_coord: ChunkCoord = world_coord.into();
                    let local_coord: ChunkLocalCoord = world_coord.into();
                    let chunk = lock.get_mut(&chunk_coord);

                    if let Some(chunk) = chunk {
                        chunk.set_voxel(local_coord, block);

                        if local_coord.left().is_none() && lock.contains_key(&chunk_coord.left()) {
                            chunks_affected.insert(chunk_coord.left());
                        }

                        if local_coord.right().is_none() && lock.contains_key(&chunk_coord.right()) {
                            chunks_affected.insert(chunk_coord.right());
                        }

                        if local_coord.up().is_none() && lock.contains_key(&chunk_coord.up()) {
                            chunks_affected.insert(chunk_coord.up());
                        }

                        if local_coord.down().is_none() && lock.contains_key(&chunk_coord.down()) {
                            chunks_affected.insert(chunk_coord.down());
                        }

                        if local_coord.front().is_none() && lock.contains_key(&chunk_coord.front()) {
                            chunks_affected.insert(chunk_coord.front());
                        }

                        if local_coord.back().is_none() && lock.contains_key(&chunk_coord.back()) {
                            chunks_affected.insert(chunk_coord.back());
                        }

                        chunks_affected.insert(chunk_coord);
                    }
                }
            }
        }

        let mut lock = self.meshgen_queue.lock().unwrap();
        for coord in chunks_affected {
            lock.push_front(coord);
        }
    }

    pub fn set_voxel(&mut self, position: WorldCoord, block: Voxel) {
        // TODO: Remove code duplication
        let chunk_coord: ChunkCoord = position.into();
        let local_coord: ChunkLocalCoord = position.into();

        let mut lock = self.chunks.lock().unwrap();

        let chunk = lock.get_mut(&chunk_coord);

        if let Some(chunk) = chunk {
            chunk.set_voxel(local_coord, block);

            if let None = local_coord.left() {
                self.meshgen_queue
                    .lock()
                    .unwrap()
                    .push_front(chunk.coord.left());
            }

            if let None = local_coord.right() {
                self.meshgen_queue
                    .lock()
                    .unwrap()
                    .push_front(chunk.coord.right());
            }

            if let None = local_coord.up() {
                self.meshgen_queue
                    .lock()
                    .unwrap()
                    .push_front(chunk.coord.up());
            }

            if let None = local_coord.down() {
                self.meshgen_queue
                    .lock()
                    .unwrap()
                    .push_front(chunk.coord.down());
            }

            if let None = local_coord.front() {
                self.meshgen_queue
                    .lock()
                    .unwrap()
                    .push_front(chunk.coord.front());
            }

            if let None = local_coord.back() {
                self.meshgen_queue
                    .lock()
                    .unwrap()
                    .push_front(chunk.coord.back());
            }

            self.meshgen_queue.lock().unwrap().push_front(chunk.coord);
        }
    }

    pub fn break_block(&mut self, position: WorldCoord) {
        self.set_voxel(position, Blocks::AIR.default_state());
    }

    pub fn enqueue_chunks_around(&mut self, camera: &Camera, height: usize, distance: usize) {
        let height = height as i32;
        let distance = distance as i32;

        let world_coord: WorldCoord = camera.eye.to_vec().into();
        let center = ChunkCoord::from(world_coord);

        for i in -(height / 2)..=height / 2 {
            for j in -distance..=distance {
                for k in -distance..=distance {
                    let chunk = center
                        + ChunkCoord {
                            x: k as i32,
                            y: i as i32,
                            z: j as i32,
                        };

                    self.enqueue_chunk(chunk);
                    self.enqueue_meshgen(chunk);
                }
            }
        }
    }

    pub fn enqueue_chunk(&mut self, chunk_coord: ChunkCoord) {
        if !self.loaded_chunks.lock().unwrap().insert(chunk_coord) {
            return;
        }

        self.chunk_gen_queue.lock().unwrap().push_back(chunk_coord);
    }

    pub fn enqueue_meshgen(&mut self, coord: ChunkCoord) {
        let lock = self.chunks.lock().unwrap();
        if !(lock.contains_key(&coord.left())
            && lock.contains_key(&coord.right())
            && lock.contains_key(&coord.front())
            && lock.contains_key(&coord.back())
            && lock.contains_key(&coord.up())
            && lock.contains_key(&coord.down()))
        {
            return;
        }

        if !self.meshed_chunks.lock().unwrap().insert(coord) {
            return;
        }

        log::debug!("Enqueued meshgen.");
        self.meshgen_queue.lock().unwrap().push_back(coord);
    }

    pub fn chunks_enqueued_count(&self) -> usize {
        self.chunk_gen_queue.lock().unwrap().len()
    }

    pub fn meshgen_queue_count(&self) -> usize {
        self.meshgen_queue.lock().unwrap().len()
    }

    pub fn dispatch_threads(&mut self, worldgen: usize, meshgen: usize)
    where
        T: 'static + Generator,
    {
        for _ in 0..worldgen {
            let tx = self.chunk_sender.clone();
            let chunk_gen_queue = self.chunk_gen_queue.clone();
            let generator = self.generator.clone();
            self.world_gen_threads.push(thread::spawn(move || loop {
                let chunk_to_generate: Option<ChunkCoord> =
                    chunk_gen_queue.lock().unwrap().pop_front();

                if let Some(chunk_to_generate) = chunk_to_generate {
                    let mut chunk = Box::new(Chunk::new(chunk_to_generate));
                    generator.generate(&mut chunk);

                    log::debug!("Generated chunk {}", chunk_to_generate);

                    tx.send(chunk).expect("Channel was closed");
                } else {
                    thread::sleep(Duration::from_millis(1));
                }
            }));
        }

        for _ in 0..meshgen {
            let mesh_gen_queue = self.meshgen_queue.clone();
            let world_accessor = self.world_accessor.clone();
            let tx = self.mesh_sender.clone();
            self.meshgen_threads.push(thread::spawn(move || loop {
                let coord: Option<ChunkCoord> = mesh_gen_queue.lock().unwrap().pop_front();

                if let Some(mesh_to_gen) = coord {
                    let chunk = world_accessor.chunks.lock().unwrap()[&mesh_to_gen].clone();
                    let mesh =
                        generate_mesh_lod(chunk, world_accessor.clone(), meshgen::LodLevel::_0);
                    log::debug!("Finished meshing {}!", mesh_to_gen);

                    if let Some(mesh) = mesh {
                        tx.send((mesh_to_gen, mesh)).unwrap();
                    }
                } else {
                    thread::sleep(Duration::from_millis(1));
                }
            }));
        }
    }

    pub fn receive_chunk(&mut self) {
        let recv_iterator = self.chunk_receiver.try_iter();
        let mut coords_to_mesh = Vec::new();

        for chunk in recv_iterator {
            let coord = chunk.coord;
            self.chunks.lock().unwrap().insert(coord, chunk);
            coords_to_mesh.push(coord);
        }

        for coord in coords_to_mesh {
            self.enqueue_meshgen(coord);
        }
    }

    pub fn ray_hit(
        &self,
        ray: Ray,
        mut debug: Option<&mut DebugDrawer>,
    ) -> Option<(WorldCoord, cgmath::Point3<f32>, Voxel)> {
        const MAX_DISTANCE: f32 = 32.0;

        let mut distance = 0.0;

        while distance < MAX_DISTANCE {
            let point = ray.origin + ray.direction * distance;

            let world_coord: WorldCoord = point.to_vec().into();

            if let Some(debug) = debug.as_mut() {
                debug.append_mesh(
                    ModelName::Cube,
                    point.to_vec() - cgmath::Vector3::new(0.05, 0.05, 0.05),
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

                return Some((world_coord, point, voxel));
            }
        }

        None
    }

    pub fn dequeue_meshgen(
        &mut self,
        limit: usize,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bg_layout: &wgpu::BindGroupLayout,
    ) {
        for (i, (coord, mesh)) in self.mesh_receiver.try_iter().enumerate() {
            log::debug!("Received mesh for chunk {}", coord);
            let mut model = Model::new(bg_layout, device, Mesh::from_info(device, mesh));
            model.position = coord.into();
            _ = self.models.insert(coord, model);
            self.models[&coord].update_buffer(queue);

            if i >= limit {
                break;
            }
        }
    }

    // Returns a number of chunks drawn
    pub fn draw_distance(
        &self,
        render_pass: &mut wgpu::RenderPass,
        eye: cgmath::Vector3<f32>,
        max_chunks: usize,
    ) -> usize {
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

    pub fn append_debug(&self, debug: &mut DebugDrawer) {
        for (coord, _) in self.models.iter() {
            let position = (*coord).into(); // rust being weird again

            let scale =
                cgmath::Vector3::new(CHUNK_SIZE as f32, CHUNK_SIZE as f32, CHUNK_SIZE as f32);

            debug.append_mesh(
                ModelName::Cube,
                position,
                scale,
                [1.0, 0.0, 1.0, 0.0].into(),
            );
        }

        let meshgen_queue_text = format!(
            "Meshgen Queue size: {}",
            self.meshgen_queue.lock().unwrap().len(),
        );
        let chunk_queue_text = format!(
            "Chunk Queue size: {}",
            self.chunk_gen_queue.lock().unwrap().len(),
        );
        debug.set_text("world.meshgen_queue_size", meshgen_queue_text);
        debug.set_text("world.worldgen_queue_size", chunk_queue_text);
    }
}
