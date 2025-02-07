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
    pub texture_ids: [usize; 6],
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
        texture_ids: [0; 6],
        default_state: Voxel { id: 0 },
    };

    pub const STONE: RegisteredBlock = RegisteredBlock {
        name: "Stone",
        transparent: false,
        solid: true,
        texture_ids: [1; 6],
        default_state: Voxel { id: 1 },
    };

    pub const GRASS_BLOCK: RegisteredBlock = RegisteredBlock {
        name: "Grass Block",
        transparent: false,
        solid: true,
        texture_ids: [3, 3, 2, 4, 3, 3],
        default_state: Voxel { id: 2 },
    };

    pub const DIRT_BLOCK: RegisteredBlock = RegisteredBlock {
        name: "Dirt",
        transparent: false,
        solid: true,
        texture_ids: [4; 6],
        default_state: Voxel { id: 3 },
    };

    pub const BLOCKS: &'static [RegisteredBlock] = &[
        Self::AIR,
        Self::STONE,
        Self::GRASS_BLOCK,
        Self::DIRT_BLOCK,
    ];
}
