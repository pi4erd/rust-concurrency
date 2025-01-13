use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use super::draw::Drawable;

pub trait Vertex: Pod + Zeroable {
    fn attribs() -> &'static [wgpu::VertexAttribute];
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: Self::attribs(),
        }
    }
}

pub trait Instance: Pod + Zeroable {
    fn attribs() -> &'static [wgpu::VertexAttribute];
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: Self::attribs(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Vertex3d {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

impl Vertex3d {
    const ATTRIBS: &'static [wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x3,
        2 => Float32x2
    ];
}

impl Vertex for Vertex3d {
    fn attribs() -> &'static [wgpu::VertexAttribute] {
        Self::ATTRIBS
    }
}

pub struct Mesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    element_count: usize,
}

impl Mesh {
    pub fn create(device: &wgpu::Device, vertices: &[impl Vertex], indices: &[u32]) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        log::debug!("Created model with {} vertices and {} triangles.", vertices.len(), indices.len());

        Self {
            vertex_buffer,
            index_buffer,
            element_count: indices.len(),
        }
    }
}

impl Drawable for Mesh {
    fn draw(&self, render_pass: &mut wgpu::RenderPass) {
        // TODO: Move to bundle?
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        render_pass.draw_indexed(0..self.element_count as u32, 0, 0..1);
    }
}
