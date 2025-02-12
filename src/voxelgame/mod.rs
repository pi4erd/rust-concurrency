mod texture;
mod camera;
mod mesh;
mod draw;
mod generator;
mod debug;
mod tests;

use std::{collections::HashMap, sync::Arc, time::Instant};

use camera::{Camera, CameraController};
use cgmath::EuclideanSpace;
use debug::{DebugDrawer, DebugModelInstance, DebugVertex};
use draw::Drawable;
use generator::{NoiseGenerator, Ray, World};
use mesh::{Instance, Vertex, Vertex3d};
use pollster::FutureExt;
use rand::Rng;
use texture::Texture2d;
use wgpu::util::DeviceExt;
use winit::{dpi::PhysicalSize, event::WindowEvent, keyboard::{KeyCode, PhysicalKey}, window::{CursorGrabMode, Window}};

use crate::window::Game;

#[allow(dead_code)]
pub struct VoxelGame<'w> {
    window: Arc<Window>,

    instance: wgpu::Instance,
    surface: wgpu::Surface<'w>,
    surface_config: wgpu::SurfaceConfiguration,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,

    start_time: std::time::Instant,
    prev_time: f32,

    depth_texture: Texture2d,
    world: World<NoiseGenerator>,
    debug: DebugDrawer,

    pipelines: HashMap<String, wgpu::RenderPipeline>,
    bind_groups: HashMap<String, wgpu::BindGroup>,
    bind_layouts: HashMap<String, wgpu::BindGroupLayout>,
    uniform_buffers: HashMap<String, wgpu::Buffer>,
    textures: HashMap<String, Texture2d>,
    camera: Camera,
    camera_controller: CameraController,

    egui_context: egui::Context,
    egui_state: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,

    generate: bool,
    draw_debug: bool,
}

impl<'w> VoxelGame<'w> {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN | wgpu::Backends::DX12 |
                wgpu::Backends::METAL,
            flags: wgpu::InstanceFlags::default(),
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone())
            .expect("Failed to create window");

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.expect("Failed to request adapter");

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("graphics_device"),
            memory_hints: wgpu::MemoryHints::Performance,
            required_features: wgpu::Features::POLYGON_MODE_LINE,
            required_limits: wgpu::Limits::default(),
        }, None).await.expect("Failed to request device");

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_caps.formats.into_iter()
                .find(|f| f.is_srgb()).expect("No srgb format was available"),
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes.into_iter()
                .find(|p| *p == wgpu::PresentMode::Mailbox)
                .unwrap_or(wgpu::PresentMode::Fifo),
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
        };
        surface.configure(&device, &surface_config);

        let camera = Camera::new(size.width as f32 / size.height as f32);
        let camera_controller = CameraController::new(5.0, 0.003);

        let uniform_buffers = Self::create_uniform_buffers(&device, &camera);
        let textures = Self::create_textures(&device, &queue);
        let (bind_groups, bind_layouts) = Self::create_bind_groups(&device, &uniform_buffers, &textures);

        let depth_texture = Texture2d::create_depth_texture(
            &device,
            &surface_config,
            Some("depth_texture")
        );

        let pipelines = Self::create_pipelines(
            &device,
            &surface_config,
            &bind_layouts,
        );

        let debug = DebugDrawer::new(&device);

        let mut rng = rand::rng();
        let mut world = World::new(NoiseGenerator::new(rng.random_range(i32::MIN..i32::MAX)));

        world.dispatch_threads(3);

        window.set_cursor_visible(false);
        window.set_cursor_grab(CursorGrabMode::Locked)
            .unwrap_or_else(
                |_| window.set_cursor_grab(CursorGrabMode::Confined)
                    .expect("No cursor confine/lock available.")
            );
        
        // Setup EGUI

        let egui_context = egui::Context::default();
        let egui_state = egui_winit::State::new(
            egui_context.clone(),
            egui::ViewportId(egui::Id::new("viewport")),
            &window,
            None,
            None,
            None,
        );
        let egui_renderer = egui_wgpu::Renderer::new(
            &device,
            surface_config.format,
            Some(Texture2d::DEPTH_FORMAT),
            1, false,
        );

        Self {
            window,

            instance,
            surface,
            surface_config,
            adapter,
            device,
            queue,

            depth_texture, // TODO: Move depth texture to textures hashmap
            world,
            debug,

            start_time: Instant::now(),
            prev_time: 0.0,

            pipelines,
            bind_groups,
            bind_layouts,
            uniform_buffers,
            textures,
            camera,
            camera_controller,

            egui_state,
            egui_context,
            egui_renderer,

            generate: true,
            draw_debug: false,
        }
    }

    fn create_uniform_buffers(
        device: &wgpu::Device,
        camera: &Camera,
    ) -> HashMap<String, wgpu::Buffer> {
        let camera = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera"),
            contents: bytemuck::cast_slice(&[camera.uniform()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let mut map = HashMap::new();
        map.insert(String::from("camera"), camera);

        return map;
    }

    fn create_textures(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> HashMap<String, Texture2d> {
        let terrain_texture = Texture2d::from_image_bytes(
            include_bytes!("../../assets/textureatlas.png"),
            device,
            queue,
            Some("terrain_texture")
        ).expect("Failed to load image");

        let mut textures = HashMap::new();

        textures.insert(String::from("terrain"), terrain_texture);

        textures
    }

    fn create_bind_groups(
        device: &wgpu::Device,
        uniform_buffers: &HashMap<String, wgpu::Buffer>,
        textures: &HashMap<String, Texture2d>,
    ) -> (HashMap<String, wgpu::BindGroup>, HashMap<String, wgpu::BindGroupLayout>) {
        let model_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("model_bind_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ]
        });

        let camera_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
        });

        let texture_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ]
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &camera_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffers["camera"].as_entire_binding(),
                }
            ]
        });

        let terrain_texture_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &texture_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&textures["terrain"].view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&textures["terrain"].sampler),
                },
            ]
        });

        let mut bind_layouts = HashMap::new();
        let mut bind_groups = HashMap::new();

        bind_layouts.insert(String::from("camera"), camera_layout);
        bind_groups.insert(String::from("camera"), camera_bind_group);
        bind_layouts.insert(String::from("model"), model_layout);
        bind_layouts.insert(String::from("texture"), texture_layout);
        bind_groups.insert(String::from("terrain_texture"), terrain_texture_group);

        (bind_groups, bind_layouts)
    }

    fn create_pipelines(
        device: &wgpu::Device,
        surface_config: &wgpu::SurfaceConfiguration,
        bind_layouts: &HashMap<String, wgpu::BindGroupLayout>,
    ) -> HashMap<String, wgpu::RenderPipeline> {
        let mut map = HashMap::new();

        let opaque_module = device.create_shader_module(wgpu::include_wgsl!("shaders/opaque.wgsl"));
        let debug_module = device.create_shader_module(wgpu::include_wgsl!("shaders/debug.wgsl"));
        let sky_module = device.create_shader_module(wgpu::include_wgsl!("shaders/sky.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                &bind_layouts["model"],
                &bind_layouts["texture"],
                &bind_layouts["camera"],
            ],
            push_constant_ranges: &[],
        });
        
        let opaque_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &opaque_module,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[
                    Vertex3d::desc(),
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &opaque_module,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format: surface_config.format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })
                ]
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                polygon_mode: wgpu::PolygonMode::Fill,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                conservative: false,
            },
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture2d::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multiview: None,
            cache: None,
        });

        let camera_only_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                &bind_layouts["camera"],
            ],
            push_constant_ranges: &[],
        });

        let debug_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("debug_pipeline"),
            layout: Some(&camera_only_layout),
            vertex: wgpu::VertexState {
                module: &debug_module,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[
                    DebugVertex::desc(),
                    DebugModelInstance::desc(),
                ]
            },
            fragment: Some(wgpu::FragmentState {
                module: &debug_module,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format: surface_config.format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent::OVER,
                            alpha: wgpu::BlendComponent::OVER,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })
                ]
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Line,
                conservative: false,
            },
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture2d::DEPTH_FORMAT,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multiview: None,
            cache: None,
        });

        let sky_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sky_pipeline"),
            layout: Some(&camera_only_layout),
            vertex: wgpu::VertexState {
                module: &sky_module,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &sky_module,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format: surface_config.format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })
                ]
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            depth_stencil: None,
            multiview: None,
            cache: None,
        });

        map.insert(String::from("opaque"), opaque_pipeline);
        map.insert(String::from("debug"), debug_pipeline);
        map.insert(String::from("sky"), sky_pipeline);

        map
    }

    fn update_uniform_buffers(&mut self) {
        self.queue.write_buffer(
            &self.uniform_buffers["camera"],
            0,
            bytemuck::cast_slice(&[
                self.camera.uniform(),
            ]),
        );
    }

    fn update(&mut self, delta: f32) {
        self.camera_controller.update(&mut self.camera, delta);

        if self.generate {
            self.world.enqueue_chunks_around(&self.camera, 7, 7);
        }

        for _ in 0..64 {
            self.world.receive_chunk();
        }

        for _ in 0..64 {
            self.world.dequeue_meshgen(&self.device, &self.queue, &self.bind_layouts["model"]);
        }

        self.update_uniform_buffers();

        self.debug.new_frame();
        // self.world.append_debug(
        //     &mut self.debug,
        // );
        
        let hit = self.world.ray_hit(Ray {
            origin: self.camera.eye,
            direction: -self.camera.direction // TODO: Figure out why negative
        }, None);

        if let Some((position, _)) = hit {
            self.debug.append_mesh(
                debug::ModelName::Cube,
                cgmath::Vector3::new(
                    position.x as f32,
                    position.y as f32,
                    position.z as f32,
                ) - cgmath::Vector3::new(0.05, 0.05, 0.05),
                cgmath::Vector3::new(1.1, 1.1, 1.1),
                cgmath::Vector4::new(0.0, 0.0, 0.0, 1.0),
            );
        }

        self.debug.update_buffer(&self.queue);
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let image = self.surface.get_current_texture()?;

        let view = image.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        {
            let mut sky_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        }
                    })
                ],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            sky_pass.set_pipeline(&self.pipelines["sky"]);

            sky_pass.set_bind_group(0, &self.bind_groups["camera"], &[]);

            sky_pass.draw(0..4, 0..1);
        }

        {
            let mut opaque_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        }
                    })
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            opaque_pass.set_pipeline(&self.pipelines["opaque"]);

            opaque_pass.set_bind_group(1, &self.bind_groups["terrain_texture"], &[]);
            opaque_pass.set_bind_group(2, &self.bind_groups["camera"], &[]);

            self.world.draw_distance(&mut opaque_pass, self.camera.eye.to_vec(), 16);
        }

        if self.draw_debug {
            let mut debug_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        }
                    })
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            debug_pass.set_pipeline(&self.pipelines["debug"]);

            debug_pass.set_bind_group(0, &self.bind_groups["camera"], &[]);

            self.debug.draw(&mut debug_pass);
        }

        let mut ui_encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut ui_pass = ui_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ui_pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        }
                    })
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            }).forget_lifetime(); // NOTE: Reaaally hope this doesn't leak

            let input = self.egui_state.egui_input();

            let full_output = self.egui_context.run(input.clone(), |ctx| {
                egui::Window::new("Stats").show(&ctx, |ui| {
                    // Draw UI
                    ui.label("Hello, world!");
                    if ui.button("Press this").clicked() {
                        log::info!("Button pressed")
                    }
                });
            });

            self.egui_state.handle_platform_output(&self.window, full_output.platform_output);

            let paint_jobs = self.egui_context.tessellate(
                full_output.shapes,
                full_output.pixels_per_point
            );

            for (id, image_delta) in &full_output.textures_delta.set {
                self.egui_renderer.update_texture(
                    &self.device,
                    &self.queue,
                    *id,
                    image_delta,
                );
            }

            let screen_descriptor = egui_wgpu::ScreenDescriptor {
                size_in_pixels: [self.surface_config.width, self.surface_config.height],
                pixels_per_point: self.window.scale_factor() as f32,
            };

            _ = self.egui_renderer.update_buffers(
                &self.device,
                &self.queue,
                &mut ui_encoder,
                &paint_jobs,
                &screen_descriptor
            );
            
            self.egui_renderer.render(
                &mut ui_pass,
                &paint_jobs,
                &screen_descriptor,
            );
        }

        self.queue.submit([encoder.finish(), ui_encoder.finish()]);

        self.window.pre_present_notify();
        image.present();

        Ok(())
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }
        self.surface_config.width = new_size.width;
        self.surface_config.height = new_size.height;
        self.surface.configure(&self.device, &self.surface_config);
        self.camera.change_aspect(
            new_size.width as f32 / new_size.height as f32,
        );
        self.depth_texture = Texture2d::create_depth_texture(&self.device, &self.surface_config, Some("depth_texture"));
    }
}

impl<'w> Game for VoxelGame<'w> {
    fn init(window: Arc<Window>) -> Self {
        Self::new(window).block_on()
    }
    
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        self.camera_controller.process_window_events(&event);
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                self.window.request_redraw();

                match self.render() {
                    Ok(()) => {}
                    Err(
                        wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated
                    ) => self.resize(PhysicalSize::new(
                        self.surface_config.width,
                        self.surface_config.height,
                    )),
                    Err(wgpu::SurfaceError::Timeout) => {
                        log::warn!("Surface timeout!");
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        log::error!("Surface out of memory!");
                        event_loop.exit();
                    }
                    Err(wgpu::SurfaceError::Other) => {
                        log::warn!("Other surface error.");
                    }
                }
            }
            WindowEvent::Resized(new_size) => {
                self.resize(new_size);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if state.is_pressed() {
                    match button {
                        winit::event::MouseButton::Left => {
                            let hit = self.world.ray_hit(Ray {
                                origin:self.camera.eye,
                                direction: -self.camera.direction,
                            }, None);

                            if let Some((world_coord, _)) = hit {
                                self.world.break_block(world_coord);
                            }
                        }
                        _ => {}
                    }
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state.is_pressed() {
                    match event.physical_key {
                        PhysicalKey::Code(KeyCode::Escape) => event_loop.exit(),
                        PhysicalKey::Code(KeyCode::KeyF) => {
                            self.window.set_fullscreen(match self.window.fullscreen() {
                                Some(_) => None,
                                None => Some(winit::window::Fullscreen::Borderless(None)),
                            });
                        }
                        PhysicalKey::Code(KeyCode::KeyR) => {
                            self.world.reset();
                        }
                        PhysicalKey::Code(KeyCode::KeyL) => {
                            self.draw_debug = !self.draw_debug;
                        }
                        PhysicalKey::Code(KeyCode::KeyG) => {
                            self.generate = !self.generate;
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    
    fn device_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        self.camera_controller.process_device_events(&event);
    }
    
    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        let time = (std::time::Instant::now() - self.start_time).as_secs_f32();
        let delta = time - self.prev_time;
        self.prev_time = time;
        self.update(delta);
    }

    fn exiting(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        log::info!("Stopping application.");
    }
}
