use crate::voxelgame::{
    draw::Model,
    mesh::{Mesh, MeshInfo, Vertex3d},
};

use super::{
    chunk::{BlockOffsetCoord, Chunk, WorldCoord, CHUNK_SIZE},
    voxel::{Blocks, Voxel},
    World,
};

pub const TEXTURE_COUNT: (usize, usize) = (32, 32);
pub const TEXTURE_UV_STEP: (f32, f32) =
    (1.0 / TEXTURE_COUNT.0 as f32, 1.0 / TEXTURE_COUNT.1 as f32);

#[derive(Clone, Copy, Debug)]
enum FaceOrientation {
    Left,
    Right,
    Top,
    Bottom,
    Front,
    Back,
}

impl FaceOrientation {
    fn to_texture_id(self) -> usize {
        match self {
            Self::Left => 0,
            Self::Right => 1,
            Self::Top => 2,
            Self::Bottom => 3,
            Self::Back => 4,
            Self::Front => 5,
        }
    }
}

const fn texture_offset(texture_id: usize) -> (f32, f32) {
    (
        (texture_id % TEXTURE_COUNT.0) as f32 * TEXTURE_UV_STEP.0,
        (texture_id / TEXTURE_COUNT.0) as f32 * TEXTURE_UV_STEP.1,
    )
}

fn face(
    texture_id: usize,
    offset: (usize, usize, usize),
    orientation: FaceOrientation,
) -> ([Vertex3d; 4], [u32; 6]) {
    let texture_offset = texture_offset(texture_id);
    match orientation {
        FaceOrientation::Back => (
            [
                Vertex3d {
                    position: [offset.0 as f32, offset.1 as f32, offset.2 as f32 + 1.0],
                    normal: [0.0, 0.0, 1.0],
                    uv: [
                        texture_offset.0 + TEXTURE_UV_STEP.0,
                        texture_offset.1 + TEXTURE_UV_STEP.1,
                    ],
                },
                Vertex3d {
                    position: [
                        offset.0 as f32 + 1.0,
                        offset.1 as f32,
                        offset.2 as f32 + 1.0,
                    ],
                    normal: [0.0, 0.0, 1.0],
                    uv: [texture_offset.0, texture_offset.1 + TEXTURE_UV_STEP.1],
                },
                Vertex3d {
                    position: [
                        offset.0 as f32 + 1.0,
                        offset.1 as f32 + 1.0,
                        offset.2 as f32 + 1.0,
                    ],
                    normal: [0.0, 0.0, 1.0],
                    uv: [texture_offset.0, texture_offset.1],
                },
                Vertex3d {
                    position: [
                        offset.0 as f32,
                        offset.1 as f32 + 1.0,
                        offset.2 as f32 + 1.0,
                    ],
                    normal: [0.0, 0.0, 1.0],
                    uv: [texture_offset.0 + TEXTURE_UV_STEP.0, texture_offset.1],
                },
            ],
            [0, 1, 2, 0, 2, 3],
        ),
        FaceOrientation::Front => (
            [
                Vertex3d {
                    position: [offset.0 as f32, offset.1 as f32, offset.2 as f32],
                    normal: [0.0, 0.0, -1.0],
                    uv: [texture_offset.0, texture_offset.1 + TEXTURE_UV_STEP.1],
                },
                Vertex3d {
                    position: [offset.0 as f32 + 1.0, offset.1 as f32, offset.2 as f32],
                    normal: [0.0, 0.0, -1.0],
                    uv: [
                        texture_offset.0 + TEXTURE_UV_STEP.0,
                        texture_offset.1 + TEXTURE_UV_STEP.1,
                    ],
                },
                Vertex3d {
                    position: [
                        offset.0 as f32 + 1.0,
                        offset.1 as f32 + 1.0,
                        offset.2 as f32,
                    ],
                    normal: [0.0, 0.0, -1.0],
                    uv: [texture_offset.0 + TEXTURE_UV_STEP.0, texture_offset.1],
                },
                Vertex3d {
                    position: [offset.0 as f32, offset.1 as f32 + 1.0, offset.2 as f32],
                    normal: [0.0, 0.0, -1.0],
                    uv: [texture_offset.0, texture_offset.1],
                },
            ],
            [0, 2, 1, 0, 3, 2],
        ),
        FaceOrientation::Left => (
            [
                Vertex3d {
                    position: [offset.0 as f32, offset.1 as f32, offset.2 as f32 + 1.0],
                    normal: [-1.0, 0.0, 0.0],
                    uv: [texture_offset.0, texture_offset.1 + TEXTURE_UV_STEP.1],
                },
                Vertex3d {
                    position: [offset.0 as f32, offset.1 as f32, offset.2 as f32],
                    normal: [-1.0, 0.0, 0.0],
                    uv: [
                        texture_offset.0 + TEXTURE_UV_STEP.0,
                        texture_offset.1 + TEXTURE_UV_STEP.1,
                    ],
                },
                Vertex3d {
                    position: [offset.0 as f32, offset.1 as f32 + 1.0, offset.2 as f32],
                    normal: [-1.0, 0.0, 0.0],
                    uv: [texture_offset.0 + TEXTURE_UV_STEP.0, texture_offset.1],
                },
                Vertex3d {
                    position: [
                        offset.0 as f32,
                        offset.1 as f32 + 1.0,
                        offset.2 as f32 + 1.0,
                    ],
                    normal: [-1.0, 0.0, 0.0],
                    uv: [texture_offset.0, texture_offset.1],
                },
            ],
            [0, 2, 1, 0, 3, 2],
        ),
        FaceOrientation::Right => (
            [
                Vertex3d {
                    position: [offset.0 as f32 + 1.0, offset.1 as f32, offset.2 as f32],
                    normal: [1.0, 0.0, 0.0],
                    uv: [texture_offset.0, texture_offset.1 + TEXTURE_UV_STEP.1],
                },
                Vertex3d {
                    position: [
                        offset.0 as f32 + 1.0,
                        offset.1 as f32,
                        offset.2 as f32 + 1.0,
                    ],
                    normal: [1.0, 0.0, 0.0],
                    uv: [
                        texture_offset.0 + TEXTURE_UV_STEP.0,
                        texture_offset.1 + TEXTURE_UV_STEP.1,
                    ],
                },
                Vertex3d {
                    position: [
                        offset.0 as f32 + 1.0,
                        offset.1 as f32 + 1.0,
                        offset.2 as f32 + 1.0,
                    ],
                    normal: [1.0, 0.0, 0.0],
                    uv: [texture_offset.0 + TEXTURE_UV_STEP.0, texture_offset.1],
                },
                Vertex3d {
                    position: [
                        offset.0 as f32 + 1.0,
                        offset.1 as f32 + 1.0,
                        offset.2 as f32,
                    ],
                    normal: [1.0, 0.0, 0.0],
                    uv: [texture_offset.0, texture_offset.1],
                },
            ],
            [0, 2, 1, 0, 3, 2],
        ),
        FaceOrientation::Bottom => (
            [
                Vertex3d {
                    position: [offset.0 as f32, offset.1 as f32, offset.2 as f32],
                    normal: [0.0, -1.0, 0.0],
                    uv: [texture_offset.0, texture_offset.1 + TEXTURE_UV_STEP.1],
                },
                Vertex3d {
                    position: [offset.0 as f32 + 1.0, offset.1 as f32, offset.2 as f32],
                    normal: [0.0, -1.0, 0.0],
                    uv: [
                        texture_offset.0 + TEXTURE_UV_STEP.0,
                        texture_offset.1 + TEXTURE_UV_STEP.1,
                    ],
                },
                Vertex3d {
                    position: [
                        offset.0 as f32 + 1.0,
                        offset.1 as f32,
                        offset.2 as f32 + 1.0,
                    ],
                    normal: [0.0, -1.0, 0.0],
                    uv: [texture_offset.0 + TEXTURE_UV_STEP.0, texture_offset.1],
                },
                Vertex3d {
                    position: [offset.0 as f32, offset.1 as f32, offset.2 as f32 + 1.0],
                    normal: [0.0, -1.0, 0.0],
                    uv: [texture_offset.0, texture_offset.1],
                },
            ],
            [0, 1, 2, 0, 2, 3],
        ),
        FaceOrientation::Top => (
            [
                Vertex3d {
                    position: [offset.0 as f32, offset.1 as f32 + 1.0, offset.2 as f32],
                    normal: [0.0, 1.0, 0.0],
                    uv: [texture_offset.0, texture_offset.1 + TEXTURE_UV_STEP.1],
                },
                Vertex3d {
                    position: [
                        offset.0 as f32 + 1.0,
                        offset.1 as f32 + 1.0,
                        offset.2 as f32,
                    ],
                    normal: [0.0, 1.0, 0.0],
                    uv: [
                        texture_offset.0 + TEXTURE_UV_STEP.0,
                        texture_offset.1 + TEXTURE_UV_STEP.1,
                    ],
                },
                Vertex3d {
                    position: [
                        offset.0 as f32 + 1.0,
                        offset.1 as f32 + 1.0,
                        offset.2 as f32 + 1.0,
                    ],
                    normal: [0.0, 1.0, 0.0],
                    uv: [texture_offset.0 + TEXTURE_UV_STEP.0, texture_offset.1],
                },
                Vertex3d {
                    position: [
                        offset.0 as f32,
                        offset.1 as f32 + 1.0,
                        offset.2 as f32 + 1.0,
                    ],
                    normal: [0.0, 1.0, 0.0],
                    uv: [texture_offset.0, texture_offset.1],
                },
            ],
            [0, 2, 1, 0, 3, 2],
        ),
    }
}

#[inline]
fn get_voxel_wrapper<T>(coord: BlockOffsetCoord, chunk: &Chunk, world: &World<T>) -> Option<Voxel> {
    if coord.x < 0
        || coord.x >= CHUNK_SIZE as i32
        || coord.y < 0
        || coord.y >= CHUNK_SIZE as i32
        || coord.z < 0
        || coord.z >= CHUNK_SIZE as i32
    {
        world.get_voxel(WorldCoord::from_chunk_and_local(chunk.coord, coord))
    } else {
        chunk.get_voxel(coord.into())
    }
}

#[derive(Clone, Copy, Debug)]
pub enum LodLevel {
    _0,
    _1,
    _2,
    _3,
}

pub fn generate_mesh_lod<T>(
    chunk: &Chunk,
    world: &World<T>,
    lod_level: LodLevel,
) -> Option<MeshInfo<Vertex3d>> {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let step = match lod_level {
        LodLevel::_0 => 1,
        LodLevel::_1 => 2,
        LodLevel::_2 => 4,
        LodLevel::_3 => 8,
    };
    let scale = step as f32;

    assert_eq!(CHUNK_SIZE % step, 0);

    for x in (0..CHUNK_SIZE as i32).step_by(step) {
        for y in (0..CHUNK_SIZE as i32).step_by(step) {
            for z in (0..CHUNK_SIZE as i32).step_by(step) {
                let coord = BlockOffsetCoord { x, y, z };
                let current_voxel = get_voxel_wrapper(coord, chunk, world).unwrap();
                let current_block_info = Blocks::BLOCKS[current_voxel.id as usize];

                if current_block_info.transparent {
                    continue;
                }

                const SIDES: [FaceOrientation; 6] = [
                    FaceOrientation::Left, FaceOrientation::Right,
                    FaceOrientation::Top, FaceOrientation::Bottom,
                    FaceOrientation::Back, FaceOrientation::Front,
                ];

                for side in SIDES {
                    let coord = match side {
                        FaceOrientation::Left => coord.left(),
                        FaceOrientation::Right => coord.right(),
                        FaceOrientation::Top => coord.up(),
                        FaceOrientation::Bottom => coord.down(),
                        FaceOrientation::Back => coord.back(),
                        FaceOrientation::Front => coord.front(),
                    };

                    if let Some(voxel) = get_voxel_wrapper(coord, chunk, world) {
                        let block_info = Blocks::BLOCKS[voxel.id as usize];

                        if block_info.transparent {
                            let (mut vx, idx) = face(
                                current_block_info.texture_ids[side.to_texture_id()],
                                (x as usize / step, y as usize / step, z as usize / step),
                                side,
                            );
                            idx.into_iter()
                                .for_each(|i| indices.push(i + vertices.len() as u32));
                            vx.iter_mut()
                                .for_each(|v| {
                                    v.position[0] *= scale;
                                    v.position[1] *= scale;
                                    v.position[2] *= scale;
                                });
                            vx.into_iter()
                                .for_each(|v| vertices.push(v));
                        }
                    } else {
                        let (mut vx, idx) = face(
                            current_block_info.texture_ids[side.to_texture_id()],
                            (x as usize / step, y as usize / step, z as usize / step),
                            side,
                        );
                        idx.into_iter()
                            .for_each(|i| indices.push(i + vertices.len() as u32));
                        vx.iter_mut()
                            .for_each(|v| {
                                v.position[0] *= scale;
                                v.position[1] *= scale;
                                v.position[2] *= scale;
                            });
                        vx.into_iter().for_each(|v| vertices.push(v));
                    }
                }
            }
        }
    }

    if vertices.len() == 0 {
        return None;
    }

    Some(MeshInfo {
        vertices,
        indices,
    })
}

pub fn generate_model<T>(
    device: &wgpu::Device,
    bg_layout: &wgpu::BindGroupLayout,
    chunk: &Chunk,
    world: &World<T>,
) -> Option<Model<Mesh>> {
    let mesh_info = generate_mesh_lod(chunk, world, LodLevel::_0)?;

    let mut model = Model::new(bg_layout, device, Mesh::from_info(device, mesh_info));
    model.position = chunk.coord.into();

    Some(model)
}
