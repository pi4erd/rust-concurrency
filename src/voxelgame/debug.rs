use std::collections::HashMap;

use bytemuck::{Pod, Zeroable};

use super::{draw::Drawable, mesh::{Instance, Mesh, Vertex}};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct DebugVertex {
    pub position: [f32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct DebugModelInstance {
    position: [f32; 3],
    scale: [f32; 3],
    color: [f32; 4],
}

impl DebugModelInstance {
    const ATTRIBS: &'static [wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
        1 => Float32x3,
        2 => Float32x3,
        3 => Float32x4,
    ];
}

impl Instance for DebugModelInstance {
    fn attribs() -> &'static [wgpu::VertexAttribute] {
        Self::ATTRIBS
    }
}

impl DebugVertex {
    const ATTRIBS: &'static [wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
        0 => Float32x3,
    ];
}

impl Vertex for DebugVertex {
    fn attribs() -> &'static [wgpu::VertexAttribute] {
        Self::ATTRIBS
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ModelName {
    Cube,
}

pub struct DebugDrawer {
    meshes: HashMap<ModelName, Mesh>,
    instances: HashMap<ModelName, Vec<DebugModelInstance>>,
    instance_buffer: wgpu::Buffer,
}

impl DebugDrawer {
    pub const INSTANCE_LIMIT: usize = 100000;

    pub fn new(device: &wgpu::Device) -> Self {
        let mut meshes = HashMap::new();
        let mut instances = HashMap::new();
        meshes.insert(ModelName::Cube, Self::cube_mesh(device));
        instances.insert(ModelName::Cube, Vec::new());

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("debug_instance_buffer"),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            size: (Self::INSTANCE_LIMIT * std::mem::size_of::<DebugModelInstance>()) as u64,
            mapped_at_creation: false,
        });

        Self {
            meshes,
            instances,
            instance_buffer,
        }
    }

    pub fn new_frame(&mut self) {
        for (_, instances) in self.instances.iter_mut() {
            instances.clear();
        }
    }

    pub fn append_mesh(
        &mut self,
        model: ModelName,
        position: cgmath::Vector3<f32>,
        scale: cgmath::Vector3<f32>,
        color: cgmath::Vector4<f32>,
    ) {
        if self.instances.iter().fold(0, |a, (_, v)| a + v.len()) >= Self::INSTANCE_LIMIT {
            log::warn!("Instance limit reached!");
            return;
        }

        let instance = DebugModelInstance {
            position: position.into(),
            scale: scale.into(),
            color: color.into(),
        };

        self.instances.get_mut(&model).unwrap().push(instance);
    }

    pub fn update_buffer(&self, queue: &wgpu::Queue) {
        let mut offset = 0;
        // NOTE: Consider using queue.write_buffer_with
        for (_, instances) in self.instances.iter() {
            queue.write_buffer(
                &self.instance_buffer,
                offset as u64,
                bytemuck::cast_slice(&instances)
            );
            offset += instances.len() * std::mem::size_of::<DebugModelInstance>();
        }
    }

    fn cube_mesh(
        device: &wgpu::Device,
    ) -> Mesh {
        Mesh::create(device, &[
            DebugVertex { position: [0.0, 0.0, 0.0] },
            DebugVertex { position: [1.0, 0.0, 0.0] },
            DebugVertex { position: [1.0, 0.0, 1.0] },
            DebugVertex { position: [0.0, 0.0, 1.0] },

            DebugVertex { position: [0.0, 1.0, 0.0] },
            DebugVertex { position: [1.0, 1.0, 0.0] },
            DebugVertex { position: [1.0, 1.0, 1.0] },
            DebugVertex { position: [0.0, 1.0, 1.0] },
        ], &[
            0, 1, 0, 3, 1, 2, 2, 3, // bottom
            4, 5, 4, 7, 5, 6, 6, 7, // top
            0, 4, 1, 5, 2, 6, 3, 7, // four lines
        ])
    }
}

impl Drawable for DebugDrawer {
    fn draw(&self, render_pass: &mut wgpu::RenderPass) {
        let mut offset = 0;
        for (model, instances) in self.instances.iter() {
            let mesh = &self.meshes[&model];

            mesh.draw_instanced(render_pass, &self.instance_buffer, offset..offset + instances.len() as u32);
            offset += instances.len() as u32;
        }
    }
}
