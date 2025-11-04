use crate::assets;
use crate::game_repr::{Color, Piece, Position, Type};
use crate::renderer::Renderer;
use std::collections::HashMap;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalPosition;
use winit::window::Window;

// WGSL Shaders

const TILE_SHADER: &str = r#"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = vec4<f32>(input.position, 0.0, 1.0);
    output.color = input.color;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}
"#;

const PIECE_SHADER: &str = r#"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@group(0) @binding(0)
var texture: texture_2d<f32>;
@group(0) @binding(1)
var texture_sampler: sampler;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = vec4<f32>(input.position, 0.0, 1.0);
    output.tex_coords = input.tex_coords;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture, texture_sampler, input.tex_coords);
}
"#;

const DOT_SHADER: &str = r#"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@group(0) @binding(0)
var texture: texture_2d<f32>;
@group(0) @binding(1)
var texture_sampler: sampler;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = vec4<f32>(input.position, 0.0, 1.0);
    output.tex_coords = input.tex_coords;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(texture, texture_sampler, input.tex_coords);
    // Reduce alpha for semi-transparent effect
    return vec4<f32>(color.rgb, color.a - 0.5);
}
"#;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct TileVertex {
    position: [f32; 2],
    color: [f32; 4],
}

impl TileVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TileVertex>() as wgpu::BufferAddress,
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
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct TexturedVertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

impl TexturedVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TexturedVertex>() as wgpu::BufferAddress,
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
        }
    }
}

pub struct WgpuRenderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    window: Arc<Window>,

    tile_pipeline: wgpu::RenderPipeline,
    piece_pipeline: wgpu::RenderPipeline,
    dot_pipeline: wgpu::RenderPipeline,

    index_buffer: wgpu::Buffer,

    texture_cache: HashMap<Piece, (wgpu::Texture, wgpu::TextureView, wgpu::BindGroup)>,
    dot_texture: Option<(wgpu::Texture, wgpu::TextureView, wgpu::BindGroup)>,
    sampler: wgpu::Sampler,

    board_dimensions: (f32, f32),
    window_size: (u32, u32),
}

impl WgpuRenderer {
    pub async fn new(window: Arc<Window>) -> Self {
        let mut window_size = window.inner_size();

        // Handle WASM canvas initialization timing - dimensions might be 0x0 initially
        if window_size.width == 0 || window_size.height == 0 {
            window_size.width = 800;
            window_size.height = 800;
        }

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window_size.width,
            height: window_size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Create shaders
        let tile_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Tile Shader"),
            source: wgpu::ShaderSource::Wgsl(TILE_SHADER.into()),
        });

        let piece_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Piece Shader"),
            source: wgpu::ShaderSource::Wgsl(PIECE_SHADER.into()),
        });

        let dot_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Dot Shader"),
            source: wgpu::ShaderSource::Wgsl(DOT_SHADER.into()),
        });

        // Create texture bind group layout
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        // Create pipelines
        let tile_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Tile Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let tile_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Tile Pipeline"),
            layout: Some(&tile_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &tile_shader,
                entry_point: Some("vs_main"),
                buffers: &[TileVertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &tile_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
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
            multiview: None,
            cache: None,
        });

        let piece_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Piece Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let piece_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Piece Pipeline"),
            layout: Some(&piece_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &piece_shader,
                entry_point: Some("vs_main"),
                buffers: &[TexturedVertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &piece_shader,
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
            multiview: None,
            cache: None,
        });

        let dot_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Dot Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let dot_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Dot Pipeline"),
            layout: Some(&dot_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &dot_shader,
                entry_point: Some("vs_main"),
                buffers: &[TexturedVertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &dot_shader,
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
            multiview: None,
            cache: None,
        });

        // Create index buffer (shared across all quads)
        // Counter-clockwise winding: top-left, bottom-left, top-right, then top-right, bottom-left, bottom-right
        let indices: [u16; 6] = [0, 2, 1, 1, 2, 3];
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Create sampler
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            surface,
            device,
            queue,
            config,
            window,
            tile_pipeline,
            piece_pipeline,
            dot_pipeline,
            index_buffer,
            texture_cache: HashMap::new(),
            dot_texture: None,
            sampler,
            board_dimensions: (0.0, 0.0),
            window_size: (window_size.width, window_size.height),
        }
    }

    fn update_board_dimensions(&mut self) {
        let w_to_h = self.window_size.0 as f32 / self.window_size.1 as f32;

        self.board_dimensions = if w_to_h > 1.0 {
            (2.0 / w_to_h, 2.0)
        } else {
            (2.0, 2.0 * w_to_h)
        };
    }

    fn load_texture(&mut self, piece: Piece) -> &(wgpu::Texture, wgpu::TextureView, wgpu::BindGroup) {
        if !self.texture_cache.contains_key(&piece) {
            let texture_data = self.load_piece_texture(piece);
            self.texture_cache.insert(piece, texture_data);
        }
        self.texture_cache.get(&piece).unwrap()
    }

    fn load_piece_texture(&self, piece: Piece) -> (wgpu::Texture, wgpu::TextureView, wgpu::BindGroup) {
        let prefix = match piece.color {
            Color::White => "w",
            Color::Black => "b",
        };
        let name = match piece.piece_type {
            Type::Pawn => "pawn",
            Type::Knight => "knight",
            Type::Bishop => "bishop",
            Type::Rook => "rook",
            Type::Queen => "queen",
            Type::King => "king",
            Type::None => panic!("Cannot load texture for empty piece"),
        };

        // Get embedded bytes from assets module
        let bytes = assets::get_asset_bytes(&piece);
        let img = image::load_from_memory(bytes)
            .expect("Failed to load piece texture from embedded bytes")
            .to_rgba8();

        let dimensions = img.dimensions();

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("{}_{} Texture", prefix, name)),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &img,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture_size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.piece_pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
            label: Some(&format!("{}_{} Bind Group", prefix, name)),
        });

        (texture, view, bind_group)
    }

    fn load_dot_texture(&mut self) {
        if self.dot_texture.is_none() {
            // Get embedded bytes from assets module
            let bytes = assets::get_circle_asset_bytes();
            let img = image::load_from_memory(bytes)
                .expect("Failed to load circle texture from embedded bytes")
                .to_rgba8();

            let dimensions = img.dimensions();

            let texture_size = wgpu::Extent3d {
                width: dimensions.0,
                height: dimensions.1,
                depth_or_array_layers: 1,
            };

            let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Dot Texture"),
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });

            self.queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &img,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * dimensions.0),
                    rows_per_image: Some(dimensions.1),
                },
                texture_size,
            );

            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

            let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.dot_pipeline.get_bind_group_layout(0),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
                label: Some("Dot Bind Group"),
            });

            self.dot_texture = Some((texture, view, bind_group));
        }
    }

    fn create_tile_quad(&self, idx: usize, is_selected: bool) -> [TileVertex; 4] {
        let pos = (idx % 8, 7 - idx / 8);

        let tile_w = self.board_dimensions.0 / 8.0;
        let tile_h = self.board_dimensions.1 / 8.0;

        let top_left = (
            -1f32 + pos.0 as f32 * tile_w,
            1f32 - pos.1 as f32 * tile_h,
        );

        // Catppuccin-inspired colors
        let color = if is_selected {
            [0.84, 0.72, 0.56, 1.0] // Highlighted color
        } else if (idx / 8 + idx % 8) % 2 == 0 {
            [0.89, 0.89, 0.89, 1.0] // Light square
        } else {
            [0.47, 0.61, 0.68, 1.0] // Dark square
        };

        [
            TileVertex { position: [top_left.0, top_left.1], color },
            TileVertex { position: [top_left.0 + tile_w, top_left.1], color },
            TileVertex { position: [top_left.0, top_left.1 - tile_h], color },
            TileVertex { position: [top_left.0 + tile_w, top_left.1 - tile_h], color },
        ]
    }

    fn create_piece_quad(&self, idx: usize) -> [TexturedVertex; 4] {
        const PADDING: f32 = 0.15;

        let pos = (idx % 8, 7 - idx / 8);

        let tile_w = self.board_dimensions.0 / 8.0;
        let tile_h = self.board_dimensions.1 / 8.0;

        let piece_w = tile_w * (1.0 - 2.0 * PADDING);
        let piece_h = tile_h * (1.0 - 2.0 * PADDING);

        let top_left = (
            -1f32 + pos.0 as f32 * tile_w + PADDING * tile_w,
            1f32 - pos.1 as f32 * tile_h - PADDING * tile_h,
        );

        [
            TexturedVertex { position: [top_left.0, top_left.1], tex_coords: [0.0, 0.0] },
            TexturedVertex { position: [top_left.0 + piece_w, top_left.1], tex_coords: [1.0, 0.0] },
            TexturedVertex { position: [top_left.0, top_left.1 - piece_h], tex_coords: [0.0, 1.0] },
            TexturedVertex { position: [top_left.0 + piece_w, top_left.1 - piece_h], tex_coords: [1.0, 1.0] },
        ]
    }

    fn create_dot_quad(&self, idx: usize) -> [TexturedVertex; 4] {
        const PADDING: f32 = 0.1;

        let pos = (idx % 8, 7 - idx / 8);

        let tile_w = self.board_dimensions.0 / 8.0;
        let tile_h = self.board_dimensions.1 / 8.0;

        let dot_w = tile_w * (1.0 - 2.0 * PADDING);
        let dot_h = tile_h * (1.0 - 2.0 * PADDING);

        let top_left = (
            -1f32 + pos.0 as f32 * tile_w + PADDING * tile_w,
            1f32 - pos.1 as f32 * tile_h - PADDING * tile_h,
        );

        [
            TexturedVertex { position: [top_left.0, top_left.1], tex_coords: [0.0, 0.0] },
            TexturedVertex { position: [top_left.0 + dot_w, top_left.1], tex_coords: [1.0, 0.0] },
            TexturedVertex { position: [top_left.0, top_left.1 - dot_h], tex_coords: [0.0, 1.0] },
            TexturedVertex { position: [top_left.0 + dot_w, top_left.1 - dot_h], tex_coords: [1.0, 1.0] },
        ]
    }
}

impl Renderer for WgpuRenderer {
    fn draw_position(&mut self, position: &Position, selected_tile: Option<u8>, pov: Color) {
        self.update_board_dimensions();

        let output = self.surface.get_current_texture().unwrap();
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 1.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // Draw tiles
            render_pass.set_pipeline(&self.tile_pipeline);
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            match pov {
                Color::White => {
                    for (idx, _) in position.position.iter().enumerate() {
                        let is_selected = selected_tile.map(|t| t as usize == idx).unwrap_or(false);
                        let vertices = self.create_tile_quad(idx, is_selected);

                        let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Tile Vertex Buffer"),
                            contents: bytemuck::cast_slice(&vertices),
                            usage: wgpu::BufferUsages::VERTEX,
                        });

                        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                        render_pass.draw_indexed(0..6, 0, 0..1);
                    }
                }
                Color::Black => {
                    for (idx, _) in position.position.iter().rev().enumerate() {
                        let is_selected = selected_tile.map(|t| 63 - t as usize == idx).unwrap_or(false);
                        let vertices = self.create_tile_quad(idx, is_selected);

                        let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Tile Vertex Buffer"),
                            contents: bytemuck::cast_slice(&vertices),
                            usage: wgpu::BufferUsages::VERTEX,
                        });

                        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                        render_pass.draw_indexed(0..6, 0, 0..1);
                    }
                }
            }

            // Draw pieces
            render_pass.set_pipeline(&self.piece_pipeline);

            // Pre-load all textures to avoid borrowing issues
            let pieces_to_draw: Vec<_> = match pov {
                Color::White => position.position.iter().enumerate()
                    .filter(|(_, p)| p.piece_type != Type::None)
                    .map(|(idx, piece)| (idx, *piece))
                    .collect(),
                Color::Black => position.position.iter().rev().enumerate()
                    .filter(|(_, p)| p.piece_type != Type::None)
                    .map(|(idx, piece)| (idx, *piece))
                    .collect(),
            };

            for (idx, piece) in pieces_to_draw {
                // Load texture first (mutable borrow)
                self.load_texture(piece);
                // Now we can access the cached texture and other fields
                let (_, _, bind_group) = self.texture_cache.get(&piece).unwrap();
                let vertices = self.create_piece_quad(idx);

                let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Piece Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });

                render_pass.set_bind_group(0, bind_group, &[]);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.draw_indexed(0..6, 0, 0..1);
            }

            // Draw legal move dots
            if let Some(selected) = selected_tile {
                self.load_dot_texture();
                let (_, _, dot_bind_group) = self.dot_texture.as_ref().unwrap();

                let legal_moves = position.legal_moves(selected as usize);

                render_pass.set_pipeline(&self.dot_pipeline);
                render_pass.set_bind_group(0, dot_bind_group, &[]);

                for _move in legal_moves {
                    let idx = match pov {
                        Color::White => _move._to(),
                        Color::Black => 63 - _move._to(),
                    };

                    let vertices = self.create_dot_quad(idx);

                    let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Dot Vertex Buffer"),
                        contents: bytemuck::cast_slice(&vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    });

                    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    render_pass.draw_indexed(0..6, 0, 0..1);
                }
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    fn coord_to_tile(&self, coords: PhysicalPosition<f64>, pov: Color) -> Option<u8> {
        // Get scale factor to convert physical pixels to logical pixels
        let scale_factor = self.window.scale_factor();

        // Adjust coordinates for scale factor
        let adjusted_x = coords.x / scale_factor;
        let adjusted_y = coords.y / scale_factor;

        let (x, y) = (
            (adjusted_x / self.window_size.0 as f64) * 2.0,
            (adjusted_y / self.window_size.1 as f64) * 2.0,
        );

        let tile_w = self.board_dimensions.0 / 8.0;
        let tile_h = self.board_dimensions.1 / 8.0;

        let tile_x = (x / tile_w as f64).floor() as usize;
        let tile_y = (y / tile_h as f64).floor() as usize;

        if tile_x > 7 || tile_y > 7 {
            return None;
        }

        let tile_from_bottom = 7 - tile_y;
        let sel_tile = tile_from_bottom * 8 + tile_x;

        let sel_tile = if pov == Color::Black {
            63 - sel_tile
        } else {
            sel_tile
        };

        Some(sel_tile as u8)
    }

    fn resize(&mut self, new_size: (u32, u32)) {
        if new_size.0 > 0 && new_size.1 > 0 {
            self.window_size = new_size;
            self.config.width = new_size.0;
            self.config.height = new_size.1;
            self.surface.configure(&self.device, &self.config);
        }
    }
}
