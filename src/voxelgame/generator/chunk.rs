use std::{fmt::Display, ops::{Add, Neg}};

use super::voxel::{Blocks, Voxel};

pub const CHUNK_SIZE: usize = 20; // NOTE: size > 20 crashes debug build
const CHUNK_SIZE_ITEMS: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct ChunkCoord {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Neg for ChunkCoord {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl Add for ChunkCoord {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl ChunkCoord {
    pub fn to_world(&self) -> cgmath::Vector3<f32> {
        cgmath::Vector3::new(
            self.x as f32 * CHUNK_SIZE as f32,
            self.y as f32 * CHUNK_SIZE as f32,
            self.z as f32 * CHUNK_SIZE as f32,
        )
    }

    pub fn from_world(coord: cgmath::Vector3<f32>) -> Self {
        // FIXME: It's off by 1 sometimes and I can't be bothered to fix it now
        let x = coord.x as i32;
        let y = coord.y as i32;
        let z = coord.z as i32;

        Self {
            x: x / CHUNK_SIZE as i32 - if x < 0 { 1 } else { 0 },
            y: y / CHUNK_SIZE as i32 - if y < 0 { 1 } else { 0 },
            z: z / CHUNK_SIZE as i32 - if z < 0 { 1 } else { 0 },
        }
    }

    pub fn left(self) -> Self {
        Self {
            x: self.x - 1,
            y: self.y,
            z: self.z,
        }
    }

    pub fn right(self) -> Self {
        Self {
            x: self.x + 1,
            y: self.y,
            z: self.z,
        }
    }

    pub fn front(self) -> Self {
        Self {
            x: self.x,
            y: self.y,
            z: self.z - 1,
        }
    }

    pub fn back(self) -> Self {
        Self {
            x: self.x,
            y: self.y,
            z: self.z + 1,
        }
    }

    pub fn up(self) -> Self {
        Self {
            x: self.x,
            y: self.y + 1,
            z: self.z,
        }
    }

    pub fn down(self) -> Self {
        Self {
            x: self.x,
            y: self.y - 1,
            z: self.z,
        }
    }
}

impl Display for ChunkCoord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

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
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
            return None
        }

        self.chunk_data.get(x + y * CHUNK_SIZE + (z * CHUNK_SIZE * CHUNK_SIZE)).cloned()
    }

    #[inline]
    pub fn set_voxel(&mut self, x: usize, y: usize, z: usize, voxel: Voxel) {
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
            return;
        }

        self.chunk_data[x + y * CHUNK_SIZE + (z * CHUNK_SIZE * CHUNK_SIZE)] = voxel;
    }
}
