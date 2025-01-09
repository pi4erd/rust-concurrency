use bytemuck::{Pod, Zeroable};
use cgmath::{Quaternion, Vector3};

use super::{draw::{Model, Drawable}, mesh::{Mesh, Vertex}};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct DebugVertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct DebugModelInstance {
    model_matrix: [[f32; 4]; 4],
    color: [f32; 4],
}

impl DebugModelInstance {
    const ATTRIBS: &'static [wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
        0 => Float32x4,
        1 => Float32x4,
        2 => Float32x4,
        3 => Float32x4,
        4 => Float32x4,
    ];
}

impl Vertex for DebugModelInstance {
    fn attribs() -> &'static [wgpu::VertexAttribute] {
        Self::ATTRIBS
    }

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<DebugModelInstance>() as u64,
            attributes: Self::attribs(),
            step_mode: wgpu::VertexStepMode::Instance,
        }
    }
}

impl DebugVertex {
    const ATTRIBS: &'static [wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x3,
    ];
}

impl Vertex for DebugVertex {
    fn attribs() -> &'static [wgpu::VertexAttribute] {
        Self::ATTRIBS
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelName {
    Cube,
}

pub struct DebugDrawer {
    model_pool: Vec<Model<Mesh>>,
}

impl DebugDrawer {
    pub fn new() -> Self {
        Self {
            model_pool: Vec::new(),
        }
    }

    pub fn new_frame(&mut self) {
        self.model_pool.clear();
    }

    pub fn append_model(
        &mut self,
        model_name: ModelName,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        model_bg_layout: &wgpu::BindGroupLayout,
        position: Vector3<f32>,
        rotation: Quaternion<f32>,
        scale: Vector3<f32>,
        color: [f32; 3],
    ) {
        let mut model = Model::new(
            model_bg_layout,
            device,
            match model_name {
                ModelName::Cube => Self::cube_mesh(device, color),
            }
        );

        model.position = position;
        model.rotation = rotation;
        model.scale = scale;
        model.update_buffer(queue);
        self.model_pool.push(model);
    }

    fn sphere_mesh(
        _device: &wgpu::Device,
        _color: [f32; 3]
    ) -> Mesh {
        todo!()
    }

    fn cube_mesh(
        device: &wgpu::Device,
        color: [f32; 3],
    ) -> Mesh {
        Mesh::create(device, &[
            DebugVertex { position: [0.0, 0.0, 0.0] , color },
            DebugVertex { position: [1.0, 0.0, 0.0] , color },
            DebugVertex { position: [1.0, 0.0, 1.0] , color },
            DebugVertex { position: [0.0, 0.0, 1.0] , color },

            DebugVertex { position: [0.0, 1.0, 0.0] , color },
            DebugVertex { position: [1.0, 1.0, 0.0] , color },
            DebugVertex { position: [1.0, 1.0, 1.0] , color },
            DebugVertex { position: [0.0, 1.0, 1.0] , color },
        ], &[
            0, 1, 0, 3, 1, 2, 2, 3, // bottom
            4, 5, 4, 7, 5, 6, 6, 7, // top
            0, 4, 1, 5, 2, 6, 3, 7, // four lines
        ])
    }
}

impl Drawable for DebugDrawer {
    fn draw(&self, render_pass: &mut wgpu::RenderPass) {
        self.model_pool.iter().for_each(|m| m.draw(render_pass));
    }
}
