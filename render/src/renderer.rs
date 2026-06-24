//! WGPU renderer for VoxelNaut
//!
//! Main rendering pipeline using WGPU.

use wgpu;
use wgpu::util::DeviceExt;
use std::sync::Arc;
use parking_lot::RwLock;
use core::config::GraphicsSettings;
use core::math::Vec3;
use super::{Camera, Mesh, Vertex};

/// Render pipeline
pub struct Renderer {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    surface: Option<wgpu::Surface>,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    camera: Arc<RwLock<Camera>>,
    chunk_mesh_buffer: Arc<RwLock<Option<ChunkMeshData>>>,
}

struct ChunkMeshData {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
}

/// Uniforms for shaders
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Uniforms {
    view_projection: [[f32; 4]; 4],
    camera_pos: [f32; 3],
    _padding1: u32,
    time: f32,
    _padding2: [u32; 3],
}

impl Renderer {
    pub async fn new(window: &winit::window::Window, settings: &GraphicsSettings) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
        });

        let surface = instance.create_surface(window);
        
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("VoxelNaut Device"),
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
            },
            None,
        ).await.unwrap();

        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: size.width,
            height: size.height,
            present_mode: if settings.vsync {
                wgpu::PresentMode::Fifo
            } else {
                wgpu::PresentMode::Mailbox
            },
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };

        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Voxel Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            flags: wgpu::ShaderFlags::empty(),
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Main Pipeline"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24Plus,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multiview: None,
            multisample: wgpu::MultisampleState::default(),
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniforms"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            device: Arc::new(device),
            queue: Arc::new(queue),
            surface: Some(surface),
            config,
            render_pipeline,
            uniform_buffer,
            camera: Arc::new(RwLock::new(Camera::new(Vec3::ZERO, 70.0, size.width as f32 / size.height as f32))),
            chunk_mesh_buffer: Arc::new(RwLock::new(None)),
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        if let Some(ref surface) = self.surface {
            surface.configure(&self.device, &self.config);
        }
        self.camera.write().aspect_ratio = width as f32 / height as f32;
    }

    pub fn update_camera(&self, camera: Camera) {
        *self.camera.write() = camera;
    }

    pub fn update_chunk_mesh(&self, mesh: &Mesh) {
        let vertex_data: Vec<u8> = mesh.vertices.iter()
            .flat_map(|v| {
                let pos: &[f32; 3] = &[v.position[0], v.position[1], v.position[2]];
                let norm: &[f32; 3] = &[v.normal[0], v.normal[1], v.normal[2]];
                let color: &[u8; 4] = &v.color.to_le_bytes();
                let tex: &[f32; 2] = &[v.tex_coord[0], v.tex_coord[1]];
                let layer: &[u8; 4] = &v.tex_layer.to_le_bytes();
                pos.iter().copied()
                    .chain(norm.iter().copied())
                    .chain(color.iter().copied())
                    .chain(tex.iter().copied())
                    .chain(layer.iter().copied())
            })
            .collect();

        let index_data: Vec<u8> = mesh.indices.iter()
            .flat_map(|&i| i.to_le_bytes())
            .collect();

        let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Chunk Vertices"),
            contents: &vertex_data,
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Chunk Indices"),
            contents: &index_data,
            usage: wgpu::BufferUsages::INDEX,
        });

        *self.chunk_mesh_buffer.write() = Some(ChunkMeshData {
            vertex_buffer,
            index_buffer,
            index_count: mesh.indices.len() as u32,
        });
    }

    pub fn render(&self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.as_ref().unwrap().get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let camera = self.camera.read();
        let uniforms = Uniforms {
            view_projection: camera.view_projection(),
            camera_pos: camera.position.to_array(),
            _padding1: 0,
            time: 0.0,
            _padding2: [0; 3],
        };
        drop(camera);

        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.529,
                            g: 0.808,
                            b: 0.922,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.create_uniform_bind_group(), &[]);

            if let Some(chunk_data) = self.chunk_mesh_buffer.read().as_ref() {
                render_pass.set_vertex_buffer(0, chunk_data.vertex_buffer.slice(..));
                render_pass.set_index_buffer(chunk_data.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..chunk_data.index_count, 0, 0..1);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn create_uniform_bind_group(&self) -> wgpu::BindGroup {
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Uniforms"),
            layout: &self.render_pipeline.get_bind_group_layout(0),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.uniform_buffer.as_entire_binding(),
            }],
        })
    }

    pub fn device(&self) -> &Arc<wgpu::Device> {
        &self.device
    }

    pub fn queue(&self) -> &Arc<wgpu::Queue> {
        &self.queue
    }
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 24,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Unorm8x4,
                },
                wgpu::VertexAttribute {
                    offset: 28,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 36,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Uint32,
                },
            ],
        }
    }
}