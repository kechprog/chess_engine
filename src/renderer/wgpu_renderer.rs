use crate::agent::player::GameResult;
use crate::assets;
use crate::game_repr::{Color, Piece, Position, Type};
use crate::renderer::Renderer;
use std::collections::HashMap;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalPosition;
use winit::window::Window;
use glyphon::{FontSystem, SwashCache, TextAtlas, TextRenderer, TextArea, Buffer, Metrics, Family, Attrs, Cache, Viewport};

// Embed font at compile time for WASM compatibility
const FONT_DATA: &[u8] = include_bytes!("../assets/Roboto-Regular.ttf");

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
    // Reduce alpha for semi-transparent effect (multiply to avoid negative alpha)
    return vec4<f32>(color.rgb, color.a * 0.5);
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
    #[allow(dead_code)] // Used only in WASM builds for scale_factor()
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

    // Text rendering components
    font_system: FontSystem,
    swash_cache: SwashCache,
    _cache: Cache,
    text_atlas: TextAtlas,
    text_renderer: TextRenderer,
    viewport: Viewport,
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

        // Initialize text rendering components with embedded font
        let mut font_system = FontSystem::new();
        // Load embedded font data for WASM compatibility (system fonts not available in WASM)
        font_system.db_mut().load_font_data(FONT_DATA.to_vec());
        let swash_cache = SwashCache::new();
        let cache = Cache::new(&device);
        let mut text_atlas = TextAtlas::new(&device, &queue, &cache, surface_format);
        let text_renderer = TextRenderer::new(&mut text_atlas, &device, wgpu::MultisampleState::default(), None);
        let viewport = Viewport::new(&device, &cache);

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
            font_system,
            swash_cache,
            _cache: cache,
            text_atlas,
            text_renderer,
            viewport,
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
        // WASM needs scale factor adjustment due to browser CSS pixel coordinate system
        // Native platforms receive coords already in physical pixels matching window_size
        #[cfg(target_arch = "wasm32")]
        let (adjusted_x, adjusted_y) = {
            let scale_factor = self.window.scale_factor();
            (coords.x / scale_factor, coords.y / scale_factor)
        };

        #[cfg(not(target_arch = "wasm32"))]
        let (adjusted_x, adjusted_y) = (coords.x, coords.y);

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
            self.text_atlas.trim();
        }
    }

    fn draw_menu(&mut self, show_coming_soon: bool) {
        let output = self.surface.get_current_texture().unwrap();
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Menu Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Menu Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.15,
                            g: 0.15,
                            b: 0.18,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.tile_pipeline);
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            if show_coming_soon {
                // Draw "Coming Soon!" overlay - large centered rectangle
                let vertices = [
                    TileVertex { position: [-0.6, 0.3], color: [0.84, 0.72, 0.56, 1.0] },
                    TileVertex { position: [0.6, 0.3], color: [0.84, 0.72, 0.56, 1.0] },
                    TileVertex { position: [-0.6, -0.3], color: [0.84, 0.72, 0.56, 1.0] },
                    TileVertex { position: [0.6, -0.3], color: [0.84, 0.72, 0.56, 1.0] },
                ];

                let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Coming Soon Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });

                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.draw_indexed(0..6, 0, 0..1);
            } else {
                // Draw two buttons: PvP and PvAI
                // PvP button (top) - greenish
                let pvp_vertices = [
                    TileVertex { position: [-0.5, 0.3], color: [0.5, 0.7, 0.5, 1.0] },
                    TileVertex { position: [0.5, 0.3], color: [0.5, 0.7, 0.5, 1.0] },
                    TileVertex { position: [-0.5, 0.1], color: [0.5, 0.7, 0.5, 1.0] },
                    TileVertex { position: [0.5, 0.1], color: [0.5, 0.7, 0.5, 1.0] },
                ];

                let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("PvP Button Vertex Buffer"),
                    contents: bytemuck::cast_slice(&pvp_vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });

                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.draw_indexed(0..6, 0, 0..1);

                // PvAI button (bottom) - blueish
                let pvai_vertices = [
                    TileVertex { position: [-0.5, -0.1], color: [0.5, 0.6, 0.8, 1.0] },
                    TileVertex { position: [0.5, -0.1], color: [0.5, 0.6, 0.8, 1.0] },
                    TileVertex { position: [-0.5, -0.3], color: [0.5, 0.6, 0.8, 1.0] },
                    TileVertex { position: [0.5, -0.3], color: [0.5, 0.6, 0.8, 1.0] },
                ];

                let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("PvAI Button Vertex Buffer"),
                    contents: bytemuck::cast_slice(&pvai_vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });

                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.draw_indexed(0..6, 0, 0..1);
            }
        }

        // Prepare text rendering
        let viewport_width = self.window_size.0 as f32;
        let viewport_height = self.window_size.1 as f32;

        // Update viewport with current window size
        self.viewport.update(&self.queue, glyphon::Resolution {
            width: self.window_size.0,
            height: self.window_size.1,
        });

        // Create text buffers and areas
        if show_coming_soon {
            // "Coming Soon!" text - centered on golden overlay
            let mut coming_soon_buffer = Buffer::new(&mut self.font_system, Metrics::new(48.0, 60.0));
            coming_soon_buffer.set_size(&mut self.font_system, Some(viewport_width), Some(viewport_height));
            coming_soon_buffer.set_text(&mut self.font_system, "Coming Soon!", Attrs::new().family(Family::SansSerif), glyphon::Shaping::Advanced);

            // Calculate text position to center it on the overlay
            // Overlay is at [-0.6, 0.3] to [0.6, -0.3] in NDC
            // Convert NDC to pixel coordinates
            let overlay_center_x = viewport_width / 2.0;
            let overlay_center_y = viewport_height / 2.0;

            // Get text width to center it
            let layout = coming_soon_buffer.layout_runs();
            let mut text_width: f32 = 0.0;
            for run in layout {
                text_width = text_width.max(run.line_w);
            }

            let text_x = (overlay_center_x - text_width / 2.0).max(0.0);
            let text_y = (overlay_center_y - 30.0).max(0.0);

            let text_area = TextArea {
                buffer: &coming_soon_buffer,
                left: text_x,
                top: text_y,
                scale: 1.0,
                bounds: glyphon::TextBounds {
                    left: 0,
                    top: 0,
                    right: viewport_width as i32,
                    bottom: viewport_height as i32,
                },
                default_color: glyphon::Color::rgb(50, 50, 50),
                custom_glyphs: &[],
            };

            self.text_renderer.prepare(
                &self.device,
                &self.queue,
                &mut self.font_system,
                &mut self.text_atlas,
                &self.viewport,
                [text_area],
                &mut self.swash_cache,
            ).unwrap();
        } else {
            // "Player vs Player" text - centered on green button
            let mut pvp_buffer = Buffer::new(&mut self.font_system, Metrics::new(32.0, 40.0));
            pvp_buffer.set_size(&mut self.font_system, Some(viewport_width), Some(viewport_height));
            pvp_buffer.set_text(&mut self.font_system, "Player vs Player", Attrs::new().family(Family::SansSerif), glyphon::Shaping::Advanced);

            // PvP button is at [-0.5, 0.3] to [0.5, 0.1] in NDC
            // Convert to pixel coordinates
            // NDC Y=0.2 (center of button at (0.3+0.1)/2) -> Screen Y
            // NDC goes from -1 (bottom) to +1 (top), Screen goes from 0 (top) to height (bottom)
            // NDC to Screen: screen_y = (1 - ndc_y) / 2 * height
            let pvp_ndc_center_y = (0.3 + 0.1) / 2.0; // 0.2
            let pvp_center_y = (1.0 - pvp_ndc_center_y) / 2.0 * viewport_height;

            let layout = pvp_buffer.layout_runs();
            let mut text_width: f32 = 0.0;
            for run in layout {
                text_width = text_width.max(run.line_w);
            }

            let pvp_text_x = (viewport_width / 2.0 - text_width / 2.0).max(0.0);
            let pvp_text_y = (pvp_center_y - 16.0).max(0.0); // Adjust for font size

            // "Player vs AI" text - centered on blue button
            let mut pvai_buffer = Buffer::new(&mut self.font_system, Metrics::new(32.0, 40.0));
            pvai_buffer.set_size(&mut self.font_system, Some(viewport_width), Some(viewport_height));
            pvai_buffer.set_text(&mut self.font_system, "Player vs AI", Attrs::new().family(Family::SansSerif), glyphon::Shaping::Advanced);

            // PvAI button is at [-0.5, -0.1] to [0.5, -0.3] in NDC
            let pvai_ndc_center_y = (-0.1 + -0.3) / 2.0; // -0.2
            let pvai_center_y = (1.0 - pvai_ndc_center_y) / 2.0 * viewport_height;

            let layout2 = pvai_buffer.layout_runs();
            let mut text_width2: f32 = 0.0;
            for run in layout2 {
                text_width2 = text_width2.max(run.line_w);
            }

            let pvai_text_x = (viewport_width / 2.0 - text_width2 / 2.0).max(0.0);
            let pvai_text_y = (pvai_center_y - 16.0).max(0.0); // Adjust for font size

            let text_areas = [
                TextArea {
                    buffer: &pvp_buffer,
                    left: pvp_text_x,
                    top: pvp_text_y,
                    scale: 1.0,
                    bounds: glyphon::TextBounds {
                        left: 0,
                        top: 0,
                        right: viewport_width as i32,
                        bottom: viewport_height as i32,
                    },
                    default_color: glyphon::Color::rgb(0, 0, 0),
                    custom_glyphs: &[],
                },
                TextArea {
                    buffer: &pvai_buffer,
                    left: pvai_text_x,
                    top: pvai_text_y,
                    scale: 1.0,
                    bounds: glyphon::TextBounds {
                        left: 0,
                        top: 0,
                        right: viewport_width as i32,
                        bottom: viewport_height as i32,
                    },
                    default_color: glyphon::Color::rgb(0, 0, 0),
                    custom_glyphs: &[],
                },
            ];

            self.text_renderer.prepare(
                &self.device,
                &self.queue,
                &mut self.font_system,
                &mut self.text_atlas,
                &self.viewport,
                text_areas,
                &mut self.swash_cache,
            ).unwrap();
        }

        // Render text
        {
            let mut text_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Text Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            self.text_renderer.render(&self.text_atlas, &self.viewport, &mut text_pass).unwrap();
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    fn is_coord_in_button(&self, coords: PhysicalPosition<f64>, button_index: usize) -> bool {
        // Convert physical coordinates to normalized device coordinates (-1 to 1)
        #[cfg(target_arch = "wasm32")]
        let (adjusted_x, adjusted_y) = {
            let scale_factor = self.window.scale_factor();
            (coords.x / scale_factor, coords.y / scale_factor)
        };

        #[cfg(not(target_arch = "wasm32"))]
        let (adjusted_x, adjusted_y) = (coords.x, coords.y);

        let norm_x = (adjusted_x / self.window_size.0 as f64) * 2.0 - 1.0;
        let norm_y = (adjusted_y / self.window_size.1 as f64) * 2.0 - 1.0;

        match button_index {
            0 => {
                // PvP button (top): x in [-0.5, 0.5], y in [-0.3, -0.1] (in NDC, -Y is down)
                norm_x >= -0.5 && norm_x <= 0.5 && norm_y >= -0.3 && norm_y <= -0.1
            }
            1 => {
                // PvAI button (bottom): x in [-0.5, 0.5], y in [0.1, 0.3]
                norm_x >= -0.5 && norm_x <= 0.5 && norm_y >= 0.1 && norm_y <= 0.3
            }
            _ => false,
        }
    }

    fn draw_game_end(&mut self, position: &Position, selected_tile: Option<u8>, pov: Color, result: GameResult) {
        // First, draw the board position underneath
        self.draw_position(position, selected_tile, pov);

        // Now draw overlay on top
        let output = self.surface.get_current_texture().unwrap();
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Game End Overlay Encoder"),
        });

        // Determine overlay color and text based on result
        let (overlay_color, text, text_color) = match result {
            GameResult::WhiteWins => {
                // Light background for white wins
                ([0.95, 0.95, 0.95, 0.95], "White Wins!", glyphon::Color::rgb(20, 20, 20))
            }
            GameResult::BlackWins => {
                // Dark background for black wins
                ([0.15, 0.15, 0.15, 0.95], "Black Wins!", glyphon::Color::rgb(240, 240, 240))
            }
            GameResult::Draw | GameResult::Stalemate => {
                // Neutral gray background for draw/stalemate
                ([0.55, 0.55, 0.55, 0.95], "Draw!", glyphon::Color::rgb(240, 240, 240))
            }
        };

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Game End Overlay Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // Load existing frame (the board)
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.tile_pipeline);
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            // Draw semi-transparent overlay rectangle
            let vertices = [
                TileVertex { position: [-0.7, 0.4], color: overlay_color },
                TileVertex { position: [0.7, 0.4], color: overlay_color },
                TileVertex { position: [-0.7, -0.4], color: overlay_color },
                TileVertex { position: [0.7, -0.4], color: overlay_color },
            ];

            let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Game End Overlay Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.draw_indexed(0..6, 0, 0..1);
        }

        // Prepare text rendering
        let viewport_width = self.window_size.0 as f32;
        let viewport_height = self.window_size.1 as f32;

        // Update viewport with current window size
        self.viewport.update(&self.queue, glyphon::Resolution {
            width: self.window_size.0,
            height: self.window_size.1,
        });

        // Create text buffer for game result
        let mut text_buffer = Buffer::new(&mut self.font_system, Metrics::new(48.0, 60.0));
        text_buffer.set_size(&mut self.font_system, Some(viewport_width), Some(viewport_height));
        text_buffer.set_text(&mut self.font_system, text, Attrs::new().family(Family::SansSerif), glyphon::Shaping::Advanced);

        // Calculate text position to center it on the overlay
        let overlay_center_x = viewport_width / 2.0;
        let overlay_center_y = viewport_height / 2.0;

        // Get text width to center it
        let layout = text_buffer.layout_runs();
        let mut text_width: f32 = 0.0;
        for run in layout {
            text_width = text_width.max(run.line_w);
        }

        let text_x = (overlay_center_x - text_width / 2.0).max(0.0);
        let text_y = (overlay_center_y - 24.0).max(0.0); // Adjust for font size

        let text_area = TextArea {
            buffer: &text_buffer,
            left: text_x,
            top: text_y,
            scale: 1.0,
            bounds: glyphon::TextBounds {
                left: 0,
                top: 0,
                right: viewport_width as i32,
                bottom: viewport_height as i32,
            },
            default_color: text_color,
            custom_glyphs: &[],
        };

        self.text_renderer.prepare(
            &self.device,
            &self.queue,
            &mut self.font_system,
            &mut self.text_atlas,
            &self.viewport,
            [text_area],
            &mut self.swash_cache,
        ).unwrap();

        // Render text
        {
            let mut text_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Game End Text Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            self.text_renderer.render(&self.text_atlas, &self.viewport, &mut text_pass).unwrap();
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    fn draw_promotion_selection(&mut self, position: &Position, selected_tile: Option<u8>, pov: Color, promoting_color: Color) {
        // First, draw the board position underneath
        self.draw_position(position, selected_tile, pov);

        // Now draw promotion selection overlay on top
        let output = self.surface.get_current_texture().unwrap();
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Promotion Selection Overlay Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Promotion Selection Overlay Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // Load existing frame (the board)
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.tile_pipeline);
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            // Draw semi-transparent dark background overlay
            let overlay_color = [0.0, 0.0, 0.0, 0.7]; // Dark semi-transparent
            let vertices = [
                TileVertex { position: [-1.0, 1.0], color: overlay_color },
                TileVertex { position: [1.0, 1.0], color: overlay_color },
                TileVertex { position: [-1.0, -1.0], color: overlay_color },
                TileVertex { position: [1.0, -1.0], color: overlay_color },
            ];

            let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Promotion Overlay Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.draw_indexed(0..6, 0, 0..1);

            // Draw the 4 promotion pieces horizontally centered
            render_pass.set_pipeline(&self.piece_pipeline);

            // Piece positions (centered horizontally in NDC coordinates)
            // We'll space them evenly: Queen, Rook, Bishop, Knight
            let piece_size = 0.3; // Size of each piece in NDC
            let spacing = 0.35; // Spacing between pieces
            let y_pos = 0.0; // Centered vertically

            let piece_positions = [
                -spacing * 1.5, // Queen (leftmost)
                -spacing * 0.5, // Rook
                spacing * 0.5,  // Bishop
                spacing * 1.5,  // Knight (rightmost)
            ];

            let piece_types = [Type::Queen, Type::Rook, Type::Bishop, Type::Knight];

            for (i, &piece_type) in piece_types.iter().enumerate() {
                let piece = Piece {
                    piece_type,
                    color: promoting_color,
                };

                // Load texture
                self.load_texture(piece);
                let (_, _, bind_group) = self.texture_cache.get(&piece).unwrap();

                // Create quad for this piece
                let x_center = piece_positions[i];
                let vertices = [
                    TexturedVertex {
                        position: [x_center - piece_size / 2.0, y_pos + piece_size / 2.0],
                        tex_coords: [0.0, 0.0],
                    },
                    TexturedVertex {
                        position: [x_center + piece_size / 2.0, y_pos + piece_size / 2.0],
                        tex_coords: [1.0, 0.0],
                    },
                    TexturedVertex {
                        position: [x_center - piece_size / 2.0, y_pos - piece_size / 2.0],
                        tex_coords: [0.0, 1.0],
                    },
                    TexturedVertex {
                        position: [x_center + piece_size / 2.0, y_pos - piece_size / 2.0],
                        tex_coords: [1.0, 1.0],
                    },
                ];

                let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Promotion Piece Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });

                render_pass.set_bind_group(0, bind_group, &[]);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.draw_indexed(0..6, 0, 0..1);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    fn get_promotion_piece_at_coords(&self, coords: PhysicalPosition<f64>) -> Option<Type> {
        // Convert screen coordinates to NDC
        #[cfg(target_arch = "wasm32")]
        let (adjusted_x, adjusted_y) = {
            let scale_factor = self.window.scale_factor();
            (coords.x / scale_factor, coords.y / scale_factor)
        };

        #[cfg(not(target_arch = "wasm32"))]
        let (adjusted_x, adjusted_y) = (coords.x, coords.y);

        // Convert to NDC coordinates (-1 to 1)
        let ndc_x = (adjusted_x / self.window_size.0 as f64) * 2.0 - 1.0;
        let ndc_y = -((adjusted_y / self.window_size.1 as f64) * 2.0 - 1.0); // Flip Y axis

        // Piece positions and sizes (matching draw_promotion_selection)
        let piece_size = 0.3;
        let spacing = 0.35;
        let y_pos = 0.0;

        let piece_positions = [
            -spacing * 1.5, // Queen
            -spacing * 0.5, // Rook
            spacing * 0.5,  // Bishop
            spacing * 1.5,  // Knight
        ];

        let piece_types = [Type::Queen, Type::Rook, Type::Bishop, Type::Knight];

        // Check each piece's bounding box
        for (i, &piece_type) in piece_types.iter().enumerate() {
            let x_center = piece_positions[i];
            let x_min = x_center - piece_size / 2.0;
            let x_max = x_center + piece_size / 2.0;
            let y_min = y_pos - piece_size / 2.0;
            let y_max = y_pos + piece_size / 2.0;

            if ndc_x >= x_min && ndc_x <= x_max && ndc_y >= y_min && ndc_y <= y_max {
                return Some(piece_type);
            }
        }

        None
    }

    fn draw_side_selection(&mut self) {
        let output = self.surface.get_current_texture().unwrap();
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Side Selection Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Side Selection Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.15,
                            g: 0.15,
                            b: 0.18,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.tile_pipeline);
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            // Draw "Play as White" button (top) - light colored
            let white_vertices = [
                TileVertex { position: [-0.5, 0.3], color: [0.9, 0.9, 0.9, 1.0] },
                TileVertex { position: [0.5, 0.3], color: [0.9, 0.9, 0.9, 1.0] },
                TileVertex { position: [-0.5, 0.1], color: [0.9, 0.9, 0.9, 1.0] },
                TileVertex { position: [0.5, 0.1], color: [0.9, 0.9, 0.9, 1.0] },
            ];

            let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("White Button Vertex Buffer"),
                contents: bytemuck::cast_slice(&white_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.draw_indexed(0..6, 0, 0..1);

            // Draw "Play as Black" button (bottom) - dark colored
            let black_vertices = [
                TileVertex { position: [-0.5, -0.1], color: [0.2, 0.2, 0.2, 1.0] },
                TileVertex { position: [0.5, -0.1], color: [0.2, 0.2, 0.2, 1.0] },
                TileVertex { position: [-0.5, -0.3], color: [0.2, 0.2, 0.2, 1.0] },
                TileVertex { position: [0.5, -0.3], color: [0.2, 0.2, 0.2, 1.0] },
            ];

            let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Black Button Vertex Buffer"),
                contents: bytemuck::cast_slice(&black_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.draw_indexed(0..6, 0, 0..1);
        }

        // Prepare text rendering
        let viewport_width = self.window_size.0 as f32;
        let viewport_height = self.window_size.1 as f32;

        // Update viewport with current window size
        self.viewport.update(&self.queue, glyphon::Resolution {
            width: self.window_size.0,
            height: self.window_size.1,
        });

        // Create text buffers for the buttons
        let mut title_buffer = Buffer::new(&mut self.font_system, Metrics::new(40.0, 50.0));
        title_buffer.set_size(&mut self.font_system, Some(viewport_width), Some(viewport_height));
        title_buffer.set_text(&mut self.font_system, "Choose Your Side", Attrs::new().family(Family::SansSerif), glyphon::Shaping::Advanced);

        let mut white_buffer = Buffer::new(&mut self.font_system, Metrics::new(32.0, 40.0));
        white_buffer.set_size(&mut self.font_system, Some(viewport_width), Some(viewport_height));
        white_buffer.set_text(&mut self.font_system, "Play as White", Attrs::new().family(Family::SansSerif), glyphon::Shaping::Advanced);

        let mut black_buffer = Buffer::new(&mut self.font_system, Metrics::new(32.0, 40.0));
        black_buffer.set_size(&mut self.font_system, Some(viewport_width), Some(viewport_height));
        black_buffer.set_text(&mut self.font_system, "Play as Black", Attrs::new().family(Family::SansSerif), glyphon::Shaping::Advanced);

        // Calculate text positions
        // Title at top
        let title_layout = title_buffer.layout_runs();
        let mut title_width: f32 = 0.0;
        for run in title_layout {
            title_width = title_width.max(run.line_w);
        }
        let title_x = (viewport_width / 2.0 - title_width / 2.0).max(0.0);
        let title_y = 50.0;

        // White button text (centered on light button at y=0.2 in NDC)
        let white_ndc_center_y = (0.3 + 0.1) / 2.0; // 0.2
        let white_center_y = (1.0 - white_ndc_center_y) / 2.0 * viewport_height;

        let white_layout = white_buffer.layout_runs();
        let mut white_width: f32 = 0.0;
        for run in white_layout {
            white_width = white_width.max(run.line_w);
        }
        let white_text_x = (viewport_width / 2.0 - white_width / 2.0).max(0.0);
        let white_text_y = (white_center_y - 16.0).max(0.0);

        // Black button text (centered on dark button at y=-0.2 in NDC)
        let black_ndc_center_y = (-0.1 + -0.3) / 2.0; // -0.2
        let black_center_y = (1.0 - black_ndc_center_y) / 2.0 * viewport_height;

        let black_layout = black_buffer.layout_runs();
        let mut black_width: f32 = 0.0;
        for run in black_layout {
            black_width = black_width.max(run.line_w);
        }
        let black_text_x = (viewport_width / 2.0 - black_width / 2.0).max(0.0);
        let black_text_y = (black_center_y - 16.0).max(0.0);

        let text_areas = [
            TextArea {
                buffer: &title_buffer,
                left: title_x,
                top: title_y,
                scale: 1.0,
                bounds: glyphon::TextBounds {
                    left: 0,
                    top: 0,
                    right: viewport_width as i32,
                    bottom: viewport_height as i32,
                },
                default_color: glyphon::Color::rgb(255, 255, 255),
                custom_glyphs: &[],
            },
            TextArea {
                buffer: &white_buffer,
                left: white_text_x,
                top: white_text_y,
                scale: 1.0,
                bounds: glyphon::TextBounds {
                    left: 0,
                    top: 0,
                    right: viewport_width as i32,
                    bottom: viewport_height as i32,
                },
                default_color: glyphon::Color::rgb(0, 0, 0),
                custom_glyphs: &[],
            },
            TextArea {
                buffer: &black_buffer,
                left: black_text_x,
                top: black_text_y,
                scale: 1.0,
                bounds: glyphon::TextBounds {
                    left: 0,
                    top: 0,
                    right: viewport_width as i32,
                    bottom: viewport_height as i32,
                },
                default_color: glyphon::Color::rgb(255, 255, 255),
                custom_glyphs: &[],
            },
        ];

        self.text_renderer.prepare(
            &self.device,
            &self.queue,
            &mut self.font_system,
            &mut self.text_atlas,
            &self.viewport,
            text_areas,
            &mut self.swash_cache,
        ).unwrap();

        // Render text
        {
            let mut text_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Text Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            self.text_renderer.render(&self.text_atlas, &self.viewport, &mut text_pass).unwrap();
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    // TODO: Add is_coord_in_side_button to Renderer trait
    #[allow(dead_code)]
    fn is_coord_in_side_button(&self, coords: PhysicalPosition<f64>, button_index: usize) -> bool {
        // Convert physical coordinates to normalized device coordinates (-1 to 1)
        #[cfg(target_arch = "wasm32")]
        let (adjusted_x, adjusted_y) = {
            let scale_factor = self.window.scale_factor();
            (coords.x / scale_factor, coords.y / scale_factor)
        };

        #[cfg(not(target_arch = "wasm32"))]
        let (adjusted_x, adjusted_y) = (coords.x, coords.y);

        let norm_x = (adjusted_x / self.window_size.0 as f64) * 2.0 - 1.0;
        let norm_y = (adjusted_y / self.window_size.1 as f64) * 2.0 - 1.0;

        match button_index {
            0 => {
                // Play as White button (top): x in [-0.5, 0.5], y in [-0.3, -0.1] (in NDC, -Y is down)
                norm_x >= -0.5 && norm_x <= 0.5 && norm_y >= -0.3 && norm_y <= -0.1
            }
            1 => {
                // Play as Black button (bottom): x in [-0.5, 0.5], y in [0.1, 0.3]
                norm_x >= -0.5 && norm_x <= 0.5 && norm_y >= 0.1 && norm_y <= 0.3
            }
            _ => false,
        }
    }
}
