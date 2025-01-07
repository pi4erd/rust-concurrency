use crate::voxelgame::{draw::Model, mesh::{Mesh, Vertex}};

use super::{chunk::{Chunk, CHUNK_SIZE}, voxel::Blocks};

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

fn face(texture_id: usize, offset: (usize, usize, usize), orientation: FaceOrientation) -> ([Vertex; 4], [u32; 6]) {
    let texture_offset = texture_offset(texture_id);
    match orientation {
        FaceOrientation::Back => ([
            Vertex {
                position: [
                    offset.0 as f32, 
                    offset.1 as f32,
                    offset.2 as f32 + 1.0,
                ],
                uv: [
                    texture_offset.0 + TEXTURE_UV_STEP.0,
                    texture_offset.1
                ],
            },
            Vertex {
                position: [
                    offset.0 as f32 + 1.0, 
                    offset.1 as f32,
                    offset.2 as f32 + 1.0,
                ],
                uv: [
                    texture_offset.0,
                    texture_offset.1
                ],
            },
            Vertex {
                position: [
                    offset.0 as f32 + 1.0, 
                    offset.1 as f32 + 1.0,
                    offset.2 as f32 + 1.0,
                ],
                uv: [
                    texture_offset.0,
                    texture_offset.1 + TEXTURE_UV_STEP.1,
                ],
            },
            Vertex {
                position: [
                    offset.0 as f32, 
                    offset.1 as f32 + 1.0,
                    offset.2 as f32 + 1.0,
                ],
                uv: [
                    texture_offset.0 + TEXTURE_UV_STEP.0,
                    texture_offset.1 + TEXTURE_UV_STEP.1,
                ],
            },
        ], [0, 2, 1, 0, 3, 2]),
        FaceOrientation::Front => ([
            Vertex {
                position: [
                    offset.0 as f32, 
                    offset.1 as f32,
                    offset.2 as f32,
                ],
                uv: [
                    texture_offset.0,
                    texture_offset.1,
                ],
            },
            Vertex {
                position: [
                    offset.0 as f32 + 1.0, 
                    offset.1 as f32,
                    offset.2 as f32,
                ],
                uv: [
                    texture_offset.0 + TEXTURE_UV_STEP.0,
                    texture_offset.1,
                ],
            },
            Vertex {
                position: [
                    offset.0 as f32 + 1.0, 
                    offset.1 as f32 + 1.0,
                    offset.2 as f32,
                ],
                uv: [
                    texture_offset.0 + TEXTURE_UV_STEP.0,
                    texture_offset.1 + TEXTURE_UV_STEP.1,
                ],
            },
            Vertex {
                position: [
                    offset.0 as f32, 
                    offset.1 as f32 + 1.0,
                    offset.2 as f32,
                ],
                uv: [
                    texture_offset.0,
                    texture_offset.1 + TEXTURE_UV_STEP.1,
                ],
            },
        ], [0, 1, 2, 0, 2, 3]),
        FaceOrientation::Left => ([
            Vertex {
                position: [
                    offset.0 as f32, 
                    offset.1 as f32,
                    offset.2 as f32 + 1.0,
                ],
                uv: [
                    texture_offset.0,
                    texture_offset.1,
                ],
            },
            Vertex {
                position: [
                    offset.0 as f32, 
                    offset.1 as f32,
                    offset.2 as f32,
                ],
                uv: [
                    texture_offset.0 + TEXTURE_UV_STEP.0,
                    texture_offset.1,
                ],
            },
            Vertex {
                position: [
                    offset.0 as f32, 
                    offset.1 as f32 + 1.0,
                    offset.2 as f32,
                ],
                uv: [
                    texture_offset.0 + TEXTURE_UV_STEP.0,
                    texture_offset.1 + TEXTURE_UV_STEP.1,
                ],
            },
            Vertex {
                position: [
                    offset.0 as f32, 
                    offset.1 as f32 + 1.0,
                    offset.2 as f32 + 1.0,
                ],
                uv: [
                    texture_offset.0,
                    texture_offset.1 + TEXTURE_UV_STEP.1,
                ],
            },
        ], [0, 1, 2, 0, 2, 3]),
        FaceOrientation::Right => ([
            Vertex {
                position: [
                    offset.0 as f32 + 1.0, 
                    offset.1 as f32,
                    offset.2 as f32,
                ],
                uv: [
                    texture_offset.0,
                    texture_offset.1,
                ],
            },
            Vertex {
                position: [
                    offset.0 as f32 + 1.0, 
                    offset.1 as f32,
                    offset.2 as f32 + 1.0,
                ],
                uv: [
                    texture_offset.0 + TEXTURE_UV_STEP.0,
                    texture_offset.1,
                ],
            },
            Vertex {
                position: [
                    offset.0 as f32 + 1.0, 
                    offset.1 as f32 + 1.0,
                    offset.2 as f32 + 1.0,
                ],
                uv: [
                    texture_offset.0 + TEXTURE_UV_STEP.0,
                    texture_offset.1 + TEXTURE_UV_STEP.1,
                ],
            },
            Vertex {
                position: [
                    offset.0 as f32 + 1.0, 
                    offset.1 as f32 + 1.0,
                    offset.2 as f32,
                ],
                uv: [
                    texture_offset.0,
                    texture_offset.1 + TEXTURE_UV_STEP.1,
                ],
            },
        ], [0, 1, 2, 0, 2, 3]),
        FaceOrientation::Bottom => ([
            Vertex {
                position: [
                    offset.0 as f32, 
                    offset.1 as f32,
                    offset.2 as f32,
                ],
                uv: [
                    texture_offset.0,
                    texture_offset.1,
                ],
            },
            Vertex {
                position: [
                    offset.0 as f32 + 1.0, 
                    offset.1 as f32,
                    offset.2 as f32,
                ],
                uv: [
                    texture_offset.0 + TEXTURE_UV_STEP.0,
                    texture_offset.1,
                ],
            },
            Vertex {
                position: [
                    offset.0 as f32 + 1.0, 
                    offset.1 as f32,
                    offset.2 as f32 + 1.0,
                ],
                uv: [
                    texture_offset.0 + TEXTURE_UV_STEP.0,
                    texture_offset.1 + TEXTURE_UV_STEP.1,
                ],
            },
            Vertex {
                position: [
                    offset.0 as f32, 
                    offset.1 as f32,
                    offset.2 as f32 + 1.0,
                ],
                uv: [
                    texture_offset.0,
                    texture_offset.1 + TEXTURE_UV_STEP.1,
                ],
            },
        ], [0, 2, 1, 0, 3, 2]),
        FaceOrientation::Top => ([
            Vertex {
                position: [
                    offset.0 as f32, 
                    offset.1 as f32 + 1.0,
                    offset.2 as f32,
                ],
                uv: [
                    texture_offset.0,
                    texture_offset.1,
                ],
            },
            Vertex {
                position: [
                    offset.0 as f32 + 1.0, 
                    offset.1 as f32 + 1.0,
                    offset.2 as f32,
                ],
                uv: [
                    texture_offset.0 + TEXTURE_UV_STEP.0,
                    texture_offset.1,
                ],
            },
            Vertex {
                position: [
                    offset.0 as f32 + 1.0, 
                    offset.1 as f32 + 1.0,
                    offset.2 as f32 + 1.0,
                ],
                uv: [
                    texture_offset.0 + TEXTURE_UV_STEP.0,
                    texture_offset.1 + TEXTURE_UV_STEP.1,
                ],
            },
            Vertex {
                position: [
                    offset.0 as f32, 
                    offset.1 as f32 + 1.0,
                    offset.2 as f32 + 1.0,
                ],
                uv: [
                    texture_offset.0,
                    texture_offset.1 + TEXTURE_UV_STEP.1,
                ],
            },
        ], [0, 1, 2, 0, 2, 3])
    }
}

pub fn generate_model(
    device: &wgpu::Device,
    bg_layout: &wgpu::BindGroupLayout,
    chunk: &Chunk,
) -> Option<Model<Mesh>> {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for x in 0..CHUNK_SIZE.0 as i32 {
        for y in 0..CHUNK_SIZE.1 as i32 {
            for z in 0..CHUNK_SIZE.2 as i32 {
                // TODO: Add WORLD voxel sampler

                let current_voxel = chunk.get_voxel(
                    x as usize,
                    y as usize,
                    z as usize
                ).unwrap();
                let current_block_info = Blocks::BLOCKS[current_voxel.id as usize];

                if current_block_info.transparent {
                    continue;
                }

                // Left
                if let Some(voxel) = chunk.get_voxel((x - 1) as usize, y as usize, z as usize) {
                    let block_info = Blocks::BLOCKS[voxel.id as usize];
                    
                    if block_info.transparent {
                        let (vx, idx) = face(
                            current_block_info.texture_id,
                            (x as usize, y as usize, z as usize),
                            FaceOrientation::Left
                        );
                        idx.into_iter().for_each(|i| indices.push(i + vertices.len() as u32));
                        vx.into_iter().for_each(|v| vertices.push(v));
                    }
                }

                // Right
                if let Some(voxel) = chunk.get_voxel((x + 1) as usize, y as usize, z as usize) {
                    let block_info = Blocks::BLOCKS[voxel.id as usize];
                    
                    if block_info.transparent {
                        let (vx, idx) = face(
                            current_block_info.texture_id,
                            (x as usize, y as usize, z as usize),
                            FaceOrientation::Right
                        );
                        idx.into_iter().for_each(|i| indices.push(i + vertices.len() as u32));
                        vx.into_iter().for_each(|v| vertices.push(v));
                    }
                }

                // Top
                if let Some(voxel) = chunk.get_voxel(x as usize, (y + 1) as usize, z as usize) {
                    let block_info = Blocks::BLOCKS[voxel.id as usize];
                    
                    if block_info.transparent {
                        let (vx, idx) = face(
                            current_block_info.texture_id,
                            (x as usize, y as usize, z as usize),
                            FaceOrientation::Top
                        );
                        idx.into_iter().for_each(|i| indices.push(i + vertices.len() as u32));
                        vx.into_iter().for_each(|v| vertices.push(v));
                    }
                }
                
                // Bottom
                if let Some(voxel) = chunk.get_voxel(x as usize, (y - 1) as usize, z as usize) {
                    let block_info = Blocks::BLOCKS[voxel.id as usize];
                    
                    if block_info.transparent {
                        let (vx, idx) = face(
                            current_block_info.texture_id,
                            (x as usize, y as usize, z as usize),
                            FaceOrientation::Bottom
                        );
                        idx.into_iter().for_each(|i| indices.push(i + vertices.len() as u32));
                        vx.into_iter().for_each(|v| vertices.push(v));
                    }
                }

                // Front
                if let Some(voxel) = chunk.get_voxel(x as usize, y as usize, (z - 1) as usize) {
                    let block_info = Blocks::BLOCKS[voxel.id as usize];
                    
                    if block_info.transparent {
                        let (vx, idx) = face(
                            current_block_info.texture_id,
                            (x as usize, y as usize, z as usize),
                            FaceOrientation::Front
                        );
                        idx.into_iter().for_each(|i| indices.push(i + vertices.len() as u32));
                        vx.into_iter().for_each(|v| vertices.push(v));
                    }
                }

                // Back
                if let Some(voxel) = chunk.get_voxel(x as usize, y as usize, (z + 1) as usize) {
                    let block_info = Blocks::BLOCKS[voxel.id as usize];
                    
                    if block_info.transparent {
                        let (vx, idx) = face(
                            current_block_info.texture_id,
                            (x as usize, y as usize, z as usize),
                            FaceOrientation::Back
                        );
                        idx.into_iter().for_each(|i| indices.push(i + vertices.len() as u32));
                        vx.into_iter().for_each(|v| vertices.push(v));
                    }
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
    model.position = chunk.coord.to_world();

    Some(model)
}
