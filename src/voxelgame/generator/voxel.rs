type VoxelId = u8;

#[derive(Clone, Copy, Debug)]
pub struct Voxel {
    pub id: VoxelId,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct RegisteredBlock {
    pub name: &'static str,
    pub transparent: bool,
    pub solid: bool,
    pub texture_id: usize,
    default_state: Voxel,
}

impl RegisteredBlock {
    pub fn default_state(&self) -> Voxel {
        self.default_state
    }
}

pub struct Blocks;

impl Blocks {
    pub const AIR: RegisteredBlock = RegisteredBlock {
        name: "Air",
        transparent: true,
        solid: false,
        texture_id: 0,
        default_state: Voxel { id: 0 },
    };

    pub const STONE: RegisteredBlock = RegisteredBlock {
        name: "Stone",
        transparent: false,
        solid: true,
        texture_id: 1,
        default_state: Voxel { id: 1 },
    };

    pub const BLOCKS: &'static [RegisteredBlock] = &[Self::AIR, Self::STONE];
}
