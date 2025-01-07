use std::fmt::Display;

use super::voxel::{Blocks, Voxel};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct ChunkCoord {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl ChunkCoord {
    pub fn to_world(&self) -> cgmath::Vector3<f32> {
        cgmath::Vector3::new(
            self.x as f32 * CHUNK_SIZE.0 as f32,
            self.y as f32 * CHUNK_SIZE.1 as f32,
            self.z as f32 * CHUNK_SIZE.2 as f32,
        )
    }
}

impl Display for ChunkCoord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

pub const CHUNK_SIZE: (usize, usize, usize) = (20, 20, 20);
const CHUNK_SIZE_ITEMS: usize = CHUNK_SIZE.0 * CHUNK_SIZE.1 * CHUNK_SIZE.2;

#[derive(Clone, Debug)]
pub struct Chunk {
    pub chunk_data: [Voxel; CHUNK_SIZE_ITEMS], // NOTE: DO NOT store on stack
    pub coord: ChunkCoord,
}

impl Chunk {
    pub fn new(coord: ChunkCoord) -> Self {
        Self {
            coord,
            chunk_data: [Blocks::AIR.default_state(); CHUNK_SIZE_ITEMS],
        }
    }

    pub fn get_voxel(&self, x: usize, y: usize, z: usize) -> Option<Voxel> {
        if x >= CHUNK_SIZE.0 || y >= CHUNK_SIZE.1 || z >= CHUNK_SIZE.2 {
            return None;
        }

        Some(self.chunk_data[x + z * CHUNK_SIZE.0 + y * (CHUNK_SIZE.0 * CHUNK_SIZE.2)])
    }

    #[inline]
    pub fn set_voxel(&mut self, x: usize, y: usize, z: usize, voxel: Voxel) {
        if x >= CHUNK_SIZE.0 || y >= CHUNK_SIZE.1 || z >= CHUNK_SIZE.2 {
            return;
        }

        self.chunk_data[x + z * CHUNK_SIZE.0 + y * (CHUNK_SIZE.0 * CHUNK_SIZE.2)] = voxel;
    }
}
