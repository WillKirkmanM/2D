use crate::ecs::{EcsWorld};
use std::sync::Arc;
use wgpu::{Device, Queue, Surface, SurfaceConfiguration};
use wgpu::util::DeviceExt;
use winit::window::Window;
use std::collections::HashMap;

pub struct Renderer<'window> {
    surface: Surface<'window>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    size: (u32, u32),
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_groups: HashMap<String, wgpu::BindGroup>,
    transform_buffers: HashMap<String, wgpu::Buffer>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct TransformUniforms {
    model_matrix: [f32; 16],
    view_proj_matrix: [f32; 16],
}

impl<'window> Renderer<'window> {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        
        let surface = instance.create_surface(window)
            .expect("Failed to create surface");
            
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.expect("Failed to find appropriate adapter");
        
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor::default()
        ).await.expect("Failed to create device");
        
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_capabilities(&adapter).formats[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Sprite Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!("../shaders/sprite.wgsl"))),
        });
        
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bind Group Layout"),
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
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 0,
                                format: wgpu::VertexFormat::Float32x2,
                            },
                            wgpu::VertexAttribute {
                                offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                                shader_location: 1,
                                format: wgpu::VertexFormat::Float32x2,
                            },
                        ],
                    },
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
        });

        let vertices = [
            [-0.5, -0.5, 0.0, 1.0],
            [ 0.5, -0.5, 1.0, 1.0],
            [ 0.5,  0.5, 1.0, 0.0],
            [-0.5,  0.5, 0.0, 0.0],
        ];
        
        let indices = [
            0, 1, 2,
            2, 3, 0,
        ];
        
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            surface,
            device,
            queue,
            config,
            size: (size.width, size.height),
            pipeline,
            vertex_buffer,
            index_buffer,
            bind_group_layout,
            texture_bind_groups: HashMap::new(),
            transform_buffers: HashMap::new(),
        }
    }
    
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.size = (width, height);
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }
    
    pub fn render(&mut self, world: &EcsWorld) {
        let frame = self.surface.get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        
        let renderables = world.get_renderables();
        
        println!("Rendering {} sprites", renderables.len());
        for (i, (_entity, transform, sprite)) in renderables.iter().enumerate() {
            println!("Sprite {}: {} at pos: ({:.1}, {:.1}), scale: ({:.1}, {:.1})", 
                     i, sprite.texture_name, transform.position.x, transform.position.y,
                     transform.scale.x, transform.scale.y);
        }
        
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            
            for (_entity, transform, sprite) in renderables {
                println!("Drawing sprite: {}, pos: ({}, {})", 
                    sprite.texture_name, transform.position.x, transform.position.y);
                
                if let Some(bind_group) = self.texture_bind_groups.get(&sprite.texture_name) {
                    println!("  Found bind group for: {}", sprite.texture_name);
                    let model_matrix = glam::Mat4::from_scale_rotation_translation(
                        glam::Vec3::new(sprite.width * transform.scale.x, 
                                       sprite.height * transform.scale.y, 1.0),
                        glam::Quat::from_rotation_z(transform.rotation),
                        glam::Vec3::new(transform.position.x, transform.position.y, 0.0),
                    );
                    
                    let proj = glam::Mat4::orthographic_lh(
                        0.0,
                        self.size.0 as f32, 
                        0.0, 
                        self.size.1 as f32,
                        -1.0, 
                        1.0,              
                    );
                    
                    let transform_data = TransformUniforms {
                        model_matrix: model_matrix.to_cols_array(),
                        view_proj_matrix: proj.to_cols_array(),
                    };

                    if !self.transform_buffers.contains_key(&sprite.texture_name) {
                        let transform_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some(&format!("Transform Buffer for {}", sprite.texture_name)),
                            contents: bytemuck::cast_slice(&[transform_data]),
                            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                        });
                        self.transform_buffers.insert(sprite.texture_name.clone(), transform_buffer);
                    }

                    let buffer = self.transform_buffers.get(&sprite.texture_name).unwrap();
                    self.queue.write_buffer(buffer, 0, bytemuck::cast_slice(&[transform_data]));
                    
                    render_pass.set_bind_group(0, bind_group, &[]);
                    render_pass.draw_indexed(0..6, 0, 0..1);
                } else {
                    println!("  No bind group found for: {}", sprite.texture_name);
                }
            }
        }
        
        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }

    pub fn load_texture(&mut self, name: &str, data: &[u8], width: u32, height: u32) {
        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("Texture {}", name)),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            texture_size,
        );
        
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        
        let default_transform = TransformUniforms {
            model_matrix: glam::Mat4::IDENTITY.to_cols_array(),
            view_proj_matrix: glam::Mat4::IDENTITY.to_cols_array(),
        };
        
        let transform_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Transform Buffer for {}", name)),
            contents: bytemuck::cast_slice(&[default_transform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        
        self.transform_buffers.insert(name.to_string(), transform_buffer);
        
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("Texture Bind Group {}", name)),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.transform_buffers.get(name).unwrap().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });
        
        self.texture_bind_groups.insert(name.to_string(), bind_group);
    }
}