use crate::voxelgame::{draw::Model, mesh::{Mesh, Vertex3d}};

use super::{chunk::{BlockOffsetCoord, Chunk, WorldCoord, CHUNK_SIZE}, voxel::Blocks, World};

pub const TEXTURE_COUNT: (usize, usize) = (32, 32);
pub const TEXTURE_UV_STEP: (f32, f32) = (1.0 / TEXTURE_COUNT.0 as f32, 1.0 / TEXTURE_COUNT.1 as f32);

enum FaceOrientation {
    Left, Right, Top, Bottom, Front, Back,
}

const fn texture_offset(texture_id: usize) -> (f32, f32) {
    (
        (texture_id % TEXTURE_COUNT.0) as f32 * TEXTURE_UV_STEP.0,
        (texture_id / TEXTURE_COUNT.0) as f32 * TEXTURE_UV_STEP.1,
    )
}

fn face(texture_id: usize, offset: (usize, usize, usize), orientation: FaceOrientation) -> ([Vertex3d; 4], [u32; 6]) {
    let texture_offset = texture_offset(texture_id);
    match orientation {
        FaceOrientation::Back => ([
            Vertex3d {
                position: [
                    offset.0 as f32, 
                    offset.1 as f32,
                    offset.2 as f32 + 1.0,
                ],
                normal: [0.0, 0.0, 1.0],
                uv: [
                    texture_offset.0 + TEXTURE_UV_STEP.0,
                    texture_offset.1
                ],
            },
            Vertex3d {
                position: [
                    offset.0 as f32 + 1.0, 
                    offset.1 as f32,
                    offset.2 as f32 + 1.0,
                ],
                normal: [0.0, 0.0, 1.0],
                uv: [
                    texture_offset.0,
                    texture_offset.1
                ],
            },
            Vertex3d {
                position: [
                    offset.0 as f32 + 1.0, 
                    offset.1 as f32 + 1.0,
                    offset.2 as f32 + 1.0,
                ],
                normal: [0.0, 0.0, 1.0],
                uv: [
                    texture_offset.0,
                    texture_offset.1 + TEXTURE_UV_STEP.1,
                ],
            },
            Vertex3d {
                position: [
                    offset.0 as f32, 
                    offset.1 as f32 + 1.0,
                    offset.2 as f32 + 1.0,
                ],
                normal: [0.0, 0.0, 1.0],
                uv: [
                    texture_offset.0 + TEXTURE_UV_STEP.0,
                    texture_offset.1 + TEXTURE_UV_STEP.1,
                ],
            },
        ], [0, 1, 2, 0, 2, 3]),
        FaceOrientation::Front => ([
            Vertex3d {
                position: [
                    offset.0 as f32, 
                    offset.1 as f32,
                    offset.2 as f32,
                ],
                normal: [0.0, 0.0, -1.0],
                uv: [
                    texture_offset.0,
                    texture_offset.1,
                ],
            },
            Vertex3d {
                position: [
                    offset.0 as f32 + 1.0, 
                    offset.1 as f32,
                    offset.2 as f32,
                ],
                normal: [0.0, 0.0, -1.0],
                uv: [
                    texture_offset.0 + TEXTURE_UV_STEP.0,
                    texture_offset.1,
                ],
            },
            Vertex3d {
                position: [
                    offset.0 as f32 + 1.0, 
                    offset.1 as f32 + 1.0,
                    offset.2 as f32,
                ],
                normal: [0.0, 0.0, -1.0],
                uv: [
                    texture_offset.0 + TEXTURE_UV_STEP.0,
                    texture_offset.1 + TEXTURE_UV_STEP.1,
                ],
            },
            Vertex3d {
                position: [
                    offset.0 as f32, 
                    offset.1 as f32 + 1.0,
                    offset.2 as f32,
                ],
                normal: [0.0, 0.0, -1.0],
                uv: [
                    texture_offset.0,
                    texture_offset.1 + TEXTURE_UV_STEP.1,
                ],
            },
        ], [0, 2, 1, 0, 3, 2]),
        FaceOrientation::Left => ([
            Vertex3d {
                position: [
                    offset.0 as f32, 
                    offset.1 as f32,
                    offset.2 as f32 + 1.0,
                ],
                normal: [-1.0, 0.0, 0.0],
                uv: [
                    texture_offset.0,
                    texture_offset.1,
                ],
            },
            Vertex3d {
                position: [
                    offset.0 as f32, 
                    offset.1 as f32,
                    offset.2 as f32,
                ],
                normal: [-1.0, 0.0, 0.0],
                uv: [
                    texture_offset.0 + TEXTURE_UV_STEP.0,
                    texture_offset.1,
                ],
            },
            Vertex3d {
                position: [
                    offset.0 as f32, 
                    offset.1 as f32 + 1.0,
                    offset.2 as f32,
                ],
                normal: [-1.0, 0.0, 0.0],
                uv: [
                    texture_offset.0 + TEXTURE_UV_STEP.0,
                    texture_offset.1 + TEXTURE_UV_STEP.1,
                ],
            },
            Vertex3d {
                position: [
                    offset.0 as f32, 
                    offset.1 as f32 + 1.0,
                    offset.2 as f32 + 1.0,
                ],
                normal: [-1.0, 0.0, 0.0],
                uv: [
                    texture_offset.0,
                    texture_offset.1 + TEXTURE_UV_STEP.1,
                ],
            },
        ], [0, 2, 1, 0, 3, 2]),
        FaceOrientation::Right => ([
            Vertex3d {
                position: [
                    offset.0 as f32 + 1.0, 
                    offset.1 as f32,
                    offset.2 as f32,
                ],
                normal: [1.0, 0.0, 0.0],
                uv: [
                    texture_offset.0,
                    texture_offset.1,
                ],
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
                    texture_offset.1,
                ],
            },
            Vertex3d {
                position: [
                    offset.0 as f32 + 1.0, 
                    offset.1 as f32 + 1.0,
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
                    offset.2 as f32,
                ],
                normal: [1.0, 0.0, 0.0],
                uv: [
                    texture_offset.0,
                    texture_offset.1 + TEXTURE_UV_STEP.1,
                ],
            },
        ], [0, 2, 1, 0, 3, 2]),
        FaceOrientation::Bottom => ([
            Vertex3d {
                position: [
                    offset.0 as f32,
                    offset.1 as f32,
                    offset.2 as f32,
                ],
                normal: [0.0, -1.0, 0.0],
                uv: [
                    texture_offset.0,
                    texture_offset.1,
                ],
            },
            Vertex3d {
                position: [
                    offset.0 as f32 + 1.0, 
                    offset.1 as f32,
                    offset.2 as f32,
                ],
                normal: [0.0, -1.0, 0.0],
                uv: [
                    texture_offset.0 + TEXTURE_UV_STEP.0,
                    texture_offset.1,
                ],
            },
            Vertex3d {
                position: [
                    offset.0 as f32 + 1.0, 
                    offset.1 as f32,
                    offset.2 as f32 + 1.0,
                ],
                normal: [0.0, -1.0, 0.0],
                uv: [
                    texture_offset.0 + TEXTURE_UV_STEP.0,
                    texture_offset.1 + TEXTURE_UV_STEP.1,
                ],
            },
            Vertex3d {
                position: [
                    offset.0 as f32, 
                    offset.1 as f32,
                    offset.2 as f32 + 1.0,
                ],
                normal: [0.0, -1.0, 0.0],
                uv: [
                    texture_offset.0,
                    texture_offset.1 + TEXTURE_UV_STEP.1,
                ],
            },
        ], [0, 1, 2, 0, 2, 3]),
        FaceOrientation::Top => ([
            Vertex3d {
                position: [
                    offset.0 as f32, 
                    offset.1 as f32 + 1.0,
                    offset.2 as f32,
                ],
                normal: [0.0, 1.0, 0.0],
                uv: [
                    texture_offset.0,
                    texture_offset.1,
                ],
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
                    texture_offset.1,
                ],
            },
            Vertex3d {
                position: [
                    offset.0 as f32 + 1.0, 
                    offset.1 as f32 + 1.0,
                    offset.2 as f32 + 1.0,
                ],
                normal: [0.0, 1.0, 0.0],
                uv: [
                    texture_offset.0 + TEXTURE_UV_STEP.0,
                    texture_offset.1 + TEXTURE_UV_STEP.1,
                ],
            },
            Vertex3d {
                position: [
                    offset.0 as f32, 
                    offset.1 as f32 + 1.0,
                    offset.2 as f32 + 1.0,
                ],
                normal: [0.0, 1.0, 0.0],
                uv: [
                    texture_offset.0,
                    texture_offset.1 + TEXTURE_UV_STEP.1,
                ],
            },
        ], [0, 2, 1, 0, 3, 2])
    }
}

pub fn generate_model<T>(
    device: &wgpu::Device,
    bg_layout: &wgpu::BindGroupLayout,
    chunk: &Chunk,
    world: &World<T>,
) -> Option<Model<Mesh>> {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for x in 0..CHUNK_SIZE as i32 {
        for y in 0..CHUNK_SIZE as i32 {
            for z in 0..CHUNK_SIZE as i32 {
                let sampled_coord = WorldCoord::from_chunk_and_local(
                    chunk.coord,
                    BlockOffsetCoord {
                        x, y, z
                    }
                );
                let current_voxel = world.get_voxel(sampled_coord).unwrap();
                let current_block_info = Blocks::BLOCKS[current_voxel.id as usize];

                if current_block_info.transparent {
                    continue;
                }

                // Left
                if let Some(voxel) = world.get_voxel(sampled_coord.left()) {
                    let block_info = Blocks::BLOCKS[voxel.id as usize];
                    
                    if block_info.transparent {
                        let (vx, idx) = face(
                            current_block_info.texture_ids[0],
                            (x as usize, y as usize, z as usize),
                            FaceOrientation::Left
                        );
                        idx.into_iter().for_each(|i| indices.push(i + vertices.len() as u32));
                        vx.into_iter().for_each(|v| vertices.push(v));
                    }
                } else {
                    let (vx, idx) = face(
                        current_block_info.texture_ids[0],
                        (x as usize, y as usize, z as usize),
                        FaceOrientation::Left
                    );
                    idx.into_iter().for_each(|i| indices.push(i + vertices.len() as u32));
                    vx.into_iter().for_each(|v| vertices.push(v));
                }

                // Right
                if let Some(voxel) = world.get_voxel(sampled_coord.right()) {
                    let block_info = Blocks::BLOCKS[voxel.id as usize];
                    
                    if block_info.transparent {
                        let (vx, idx) = face(
                            current_block_info.texture_ids[1],
                            (x as usize, y as usize, z as usize),
                            FaceOrientation::Right
                        );
                        idx.into_iter().for_each(|i| indices.push(i + vertices.len() as u32));
                        vx.into_iter().for_each(|v| vertices.push(v));
                    }
                } else {
                    let (vx, idx) = face(
                        current_block_info.texture_ids[1],
                        (x as usize, y as usize, z as usize),
                        FaceOrientation::Right
                    );
                    idx.into_iter().for_each(|i| indices.push(i + vertices.len() as u32));
                    vx.into_iter().for_each(|v| vertices.push(v));
                }

                // Top
                if let Some(voxel) = world.get_voxel(sampled_coord.up()) {
                    let block_info = Blocks::BLOCKS[voxel.id as usize];
                    
                    if block_info.transparent {
                        let (vx, idx) = face(
                            current_block_info.texture_ids[2],
                            (x as usize, y as usize, z as usize),
                            FaceOrientation::Top
                        );
                        idx.into_iter().for_each(|i| indices.push(i + vertices.len() as u32));
                        vx.into_iter().for_each(|v| vertices.push(v));
                    }
                } else {
                    let (vx, idx) = face(
                        current_block_info.texture_ids[2],
                        (x as usize, y as usize, z as usize),
                        FaceOrientation::Top
                    );
                    idx.into_iter().for_each(|i| indices.push(i + vertices.len() as u32));
                    vx.into_iter().for_each(|v| vertices.push(v));
                }
                
                // Bottom
                if let Some(voxel) = world.get_voxel(sampled_coord.down()) {
                    let block_info = Blocks::BLOCKS[voxel.id as usize];
                    
                    if block_info.transparent {
                        let (vx, idx) = face(
                            current_block_info.texture_ids[3],
                            (x as usize, y as usize, z as usize),
                            FaceOrientation::Bottom
                        );
                        idx.into_iter().for_each(|i| indices.push(i + vertices.len() as u32));
                        vx.into_iter().for_each(|v| vertices.push(v));
                    }
                } else {
                    let (vx, idx) = face(
                        current_block_info.texture_ids[3],
                        (x as usize, y as usize, z as usize),
                        FaceOrientation::Bottom
                    );
                    idx.into_iter().for_each(|i| indices.push(i + vertices.len() as u32));
                    vx.into_iter().for_each(|v| vertices.push(v));
                }

                // Front
                if let Some(voxel) = world.get_voxel(sampled_coord.front()) {
                    let block_info = Blocks::BLOCKS[voxel.id as usize];
                    
                    if block_info.transparent {
                        let (vx, idx) = face(
                            current_block_info.texture_ids[4],
                            (x as usize, y as usize, z as usize),
                            FaceOrientation::Front
                        );
                        idx.into_iter().for_each(|i| indices.push(i + vertices.len() as u32));
                        vx.into_iter().for_each(|v| vertices.push(v));
                    }
                } else {
                    let (vx, idx) = face(
                        current_block_info.texture_ids[4],
                        (x as usize, y as usize, z as usize),
                        FaceOrientation::Front
                    );
                    idx.into_iter().for_each(|i| indices.push(i + vertices.len() as u32));
                    vx.into_iter().for_each(|v| vertices.push(v));
                }

                // Back
                if let Some(voxel) = world.get_voxel(sampled_coord.back()) {
                    let block_info = Blocks::BLOCKS[voxel.id as usize];
                    
                    if block_info.transparent {
                        let (vx, idx) = face(
                            current_block_info.texture_ids[5],
                            (x as usize, y as usize, z as usize),
                            FaceOrientation::Back
                        );
                        idx.into_iter().for_each(|i| indices.push(i + vertices.len() as u32));
                        vx.into_iter().for_each(|v| vertices.push(v));
                    }
                } else {
                    let (vx, idx) = face(
                        current_block_info.texture_ids[5],
                        (x as usize, y as usize, z as usize),
                        FaceOrientation::Back
                    );
                    idx.into_iter().for_each(|i| indices.push(i + vertices.len() as u32));
                    vx.into_iter().for_each(|v| vertices.push(v));
                }
            }
        }
    }

    if vertices.len() == 0 {
        return None
    }

    let mut model = Model::new(
        bg_layout,
        device,
        Mesh::create(device, &vertices, &indices)
    );
    model.position = chunk.coord.into();

    Some(model)
}
