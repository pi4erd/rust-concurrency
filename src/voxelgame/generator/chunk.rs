use std::{fmt::Display, ops::{Add, Neg, Sub}};

use super::voxel::{Blocks, Voxel};

pub const CHUNK_SIZE: usize = 32; // NOTE: size > 20 crashes debug build
const CHUNK_SIZE_ITEMS: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct WorldCoord {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[allow(dead_code)]
impl WorldCoord {
    pub fn from_chunk_and_local(chunk_coord: ChunkCoord, offset: BlockOffsetCoord) -> Self {
        let chunk_world_coord: WorldCoord = chunk_coord.into();

        chunk_world_coord + offset
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

impl Add<BlockOffsetCoord> for WorldCoord {
    type Output = WorldCoord;

    fn add(self, rhs: BlockOffsetCoord) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub<WorldCoord> for WorldCoord {
    type Output = BlockOffsetCoord;

    fn sub(self, rhs: WorldCoord) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Into<cgmath::Vector3<f32>> for WorldCoord {
    fn into(self) -> cgmath::Vector3<f32> {
        cgmath::Vector3::new(
            self.x as f32,
            self.y as f32,
            self.z as f32,
        )
    }
}

impl From<cgmath::Vector3<f32>> for WorldCoord {
    fn from(value: cgmath::Vector3<f32>) -> Self {
        Self {
            x: value.x.floor() as i32,
            y: value.y.floor() as i32,
            z: value.z.floor() as i32,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct BlockOffsetCoord {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockOffsetCoord {
    // Optional because can go outside of chunk
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
}

impl Into<ChunkLocalCoord> for BlockOffsetCoord {
    fn into(self) -> ChunkLocalCoord {
        ChunkLocalCoord {
            x: self.x as usize % CHUNK_SIZE,
            y: self.y as usize % CHUNK_SIZE,
            z: self.z as usize % CHUNK_SIZE,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct ChunkLocalCoord {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}

impl ChunkLocalCoord {
    // Optional because can go outside of chunk
    pub fn left(self) -> Option<Self> {
        if self.x == 0 { return None }

        return Some(Self {
            x: self.x - 1,
            y: self.y,
            z: self.z,
        });
    }

    pub fn right(self) -> Option<Self> {
        if self.x == CHUNK_SIZE - 1 { return None }

        return Some(Self {
            x: self.x + 1,
            y: self.y,
            z: self.z,
        });
    }

    pub fn up(self) -> Option<Self> {
        if self.y == CHUNK_SIZE - 1 { return None }

        return Some(Self {
            x: self.x,
            y: self.y + 1,
            z: self.z,
        });
    }

    pub fn down(self) -> Option<Self> {
        if self.y == 0 { return None }

        return Some(Self {
            x: self.x,
            y: self.y - 1,
            z: self.z,
        });
    }

    pub fn front(self) -> Option<Self> {
        if self.z == 0 { return None }

        return Some(Self {
            x: self.x,
            y: self.y,
            z: self.z - 1,
        });
    }

    pub fn back(self) -> Option<Self> {
        if self.z == CHUNK_SIZE - 1 { return None }

        return Some(Self {
            x: self.x,
            y: self.y,
            z: self.z + 1,
        });
    }
}

impl From<WorldCoord> for ChunkLocalCoord {
    fn from(value: WorldCoord) -> Self {
        let chunk_coord: ChunkCoord = value.into();
        let chunk_world_coord: WorldCoord = chunk_coord.into();

        // NOTE: offset should technically always be positive
        let offset = value - chunk_world_coord;

        assert!(offset.x >= 0 && offset.y >= 0 && offset.z >= 0);
        
        offset.into()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct ChunkCoord {
    pub x: i16,
    pub y: i16,
    pub z: i16,
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

impl From<WorldCoord> for ChunkCoord {
    fn from(mut coord: WorldCoord) -> Self {
        let neg_x = coord.x < 0;
        let neg_y = coord.y < 0;
        let neg_z = coord.z < 0;

        if neg_x {
            coord.x += 1;
        }

        if neg_y {
            coord.y += 1;
        }

        if neg_z {
            coord.z += 1;
        }

        let chunk_x = coord.x / CHUNK_SIZE as i32 - if neg_x { 1 } else { 0 };
        let chunk_y = coord.y / CHUNK_SIZE as i32 - if neg_y { 1 } else { 0 };
        let chunk_z = coord.z / CHUNK_SIZE as i32 - if neg_z { 1 } else { 0 };

        Self {
            x: chunk_x as i16,
            y: chunk_y as i16,
            z: chunk_z as i16,
        }
    }
}

impl Into<WorldCoord> for ChunkCoord {
    fn into(self) -> WorldCoord {
        WorldCoord {
            x: self.x as i32 * CHUNK_SIZE as i32,
            y: self.y as i32 * CHUNK_SIZE as i32,
            z: self.z as i32 * CHUNK_SIZE as i32,
        }
    }
}

impl Into<cgmath::Vector3<f32>> for ChunkCoord {
    fn into(self) -> cgmath::Vector3<f32> {
        let world_coord: WorldCoord = self.into();

        world_coord.into()
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

    #[inline(always)]
    fn translate_index(coord: ChunkLocalCoord) -> usize {
        coord.x + coord.y * CHUNK_SIZE + (coord.z * CHUNK_SIZE * CHUNK_SIZE)
    }

    pub fn get_voxel(&self, coord: ChunkLocalCoord) -> Option<Voxel> {
        if coord.x >= CHUNK_SIZE || coord.y >= CHUNK_SIZE || coord.z >= CHUNK_SIZE {
            return None
        }

        self.chunk_data.get(Self::translate_index(coord)).cloned()
    }

    #[inline]
    pub fn set_voxel(&mut self, coord: ChunkLocalCoord, voxel: Voxel) {
        if coord.x >= CHUNK_SIZE || coord.y >= CHUNK_SIZE || coord.z >= CHUNK_SIZE {
            return;
        }

        self.chunk_data[Self::translate_index(coord)] = voxel;
    }
}
