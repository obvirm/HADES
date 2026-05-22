use std::sync::Arc;
use winit::window::Window;
use hades_gpu::TripleBuffer;
use hades_scene::SceneSoA;

const MAX_PRIMITIVES: u64 = 10000;

pub struct Renderer<'a> {
    pub surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub render_pipeline: wgpu::RenderPipeline,
    
    pub positions_buffers: TripleBuffer<wgpu::Buffer>,
    pub sizes_buffers: TripleBuffer<wgpu::Buffer>,
    pub colors_buffers: TripleBuffer<wgpu::Buffer>,
    pub radii_buffers: TripleBuffer<wgpu::Buffer>,
    pub bind_groups: TripleBuffer<wgpu::BindGroup>,

    pub compute_pipeline: wgpu::ComputePipeline,
    pub tiles_buffers: TripleBuffer<wgpu::Buffer>,
    pub compute_bind_groups: TripleBuffer<wgpu::BindGroup>,
    pub tile_read_bind_groups: TripleBuffer<wgpu::BindGroup>,
}

impl<'a> Renderer<'a> {
    pub async fn new(window: Arc<Window>) -> Renderer<'static> {
        let size = window.inner_size();

        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window).unwrap();

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor::default(),
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Primitive Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("SDF Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../../assets/shaders/shader.wgsl").into()),
        });

        let compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../../assets/shaders/binning.wgsl").into()),
        });

        let compute_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Compute Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let tile_read_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Tile Read Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout), Some(&compute_bind_group_layout)],
            immediate_size: 0,
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: Some("cs_main"),
            compilation_options: Default::default(),
            cache: None,
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout), Some(&tile_read_bind_group_layout)],
            immediate_size: 0,
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        // Initialize Triple Buffers
        let create_buffer = |label: String| device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&label),
            size: MAX_PRIMITIVES * 16, // vec4 = 16 bytes. radii is f32 (4 bytes), but we allocate generously
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let positions_buffers = TripleBuffer::new(|i| create_buffer(format!("Positions {}", i)));
        let sizes_buffers = TripleBuffer::new(|i| create_buffer(format!("Sizes {}", i)));
        let colors_buffers = TripleBuffer::new(|i| create_buffer(format!("Colors {}", i)));
        let radii_buffers = TripleBuffer::new(|i| create_buffer(format!("Radii {}", i)));

        let bind_groups = TripleBuffer::new(|i| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("Bind Group {}", i)),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: positions_buffers.current().as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: sizes_buffers.current().as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: colors_buffers.current().as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: radii_buffers.current().as_entire_binding(),
                    },
                ],
            })
        });

        let tiles_buffer_size = 80 * 45 * 1028; // 80x45 grid, 1028 bytes per tile
        let create_tiles_buffer = |label: String| device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&label),
            size: tiles_buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let tiles_buffers = TripleBuffer::new(|i| create_tiles_buffer(format!("Tiles {}", i)));

        let compute_bind_groups = TripleBuffer::new(|i| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("Compute Bind Group {}", i)),
                layout: &compute_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: tiles_buffers.current().as_entire_binding(),
                    },
                ],
            })
        });

        // The exact same buffer but bound with a read-only layout for the fragment shader
        let tile_read_bind_groups = TripleBuffer::new(|i| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("Tile Read Bind Group {}", i)),
                layout: &tile_read_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: tiles_buffers.current().as_entire_binding(),
                    },
                ],
            })
        });

        Renderer {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            positions_buffers,
            sizes_buffers,
            colors_buffers,
            radii_buffers,
            bind_groups,
            compute_pipeline,
            tiles_buffers,
            compute_bind_groups,
            tile_read_bind_groups,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn upload_scene(&mut self, scene: &SceneSoA) {
        let num_primitives = scene.positions.len() as u32;
        if num_primitives > 0 {
            self.queue.write_buffer(self.positions_buffers.current(), 0, bytemuck::cast_slice(&scene.positions));
            self.queue.write_buffer(self.sizes_buffers.current(), 0, bytemuck::cast_slice(&scene.sizes));
            self.queue.write_buffer(self.colors_buffers.current(), 0, bytemuck::cast_slice(&scene.colors));
            self.queue.write_buffer(self.radii_buffers.current(), 0, bytemuck::cast_slice(&scene.radii));
        }
    }

    pub fn advance_frames(&mut self) {
        self.positions_buffers.next_frame();
        self.sizes_buffers.next_frame();
        self.colors_buffers.next_frame();
        self.radii_buffers.next_frame();
        self.bind_groups.next_frame();
        self.tiles_buffers.next_frame();
        self.compute_bind_groups.next_frame();
        self.tile_read_bind_groups.next_frame();
    }
}

use crate::graph::RenderPass;

pub struct ComputeBinningPass;

impl RenderPass for ComputeBinningPass {
    fn name(&self) -> &str { "Compute Binning" }
    fn execute(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        _view: &wgpu::TextureView,
        renderer: &mut Renderer,
        scene: &SceneSoA,
    ) {
        let num_primitives = scene.positions.len() as u32;
        encoder.clear_buffer(renderer.tiles_buffers.current(), 0, None);

        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Compute Pass"),
            timestamp_writes: None,
        });
        compute_pass.set_pipeline(&renderer.compute_pipeline);
        compute_pass.set_bind_group(0, renderer.bind_groups.current(), &[]);
        compute_pass.set_bind_group(1, renderer.compute_bind_groups.current(), &[]);
        
        let workgroup_count = (num_primitives + 63) / 64;
        if workgroup_count > 0 {
            compute_pass.dispatch_workgroups(workgroup_count, 1, 1);
        }
    }
}

pub struct SdfEvaluationPass;

impl RenderPass for SdfEvaluationPass {
    fn name(&self) -> &str { "SDF Evaluation" }
    fn execute(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        renderer: &mut Renderer,
        scene: &SceneSoA,
    ) {
        let num_primitives = scene.positions.len() as u32;
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1, g: 0.2, b: 0.3, a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });

        render_pass.set_pipeline(&renderer.render_pipeline);
        render_pass.set_bind_group(0, renderer.bind_groups.current(), &[]);
        render_pass.set_bind_group(1, renderer.tile_read_bind_groups.current(), &[]);
        
        if num_primitives > 0 {
            render_pass.draw(0..3, 0..1); // Fullscreen triangle
        }
    }
}
