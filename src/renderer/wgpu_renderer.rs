use crate::agent::ai::Difficulty;
use crate::agent::player::GameResult;
use crate::assets;
use crate::game_repr::{Color, Piece, Position, Type};
use crate::menu::{layout, MenuState};
use crate::orchestrator::AISetupButton;
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

        // Reserve space for right panel (20% of width in NDC = 0.4 units)
        // Board takes 80% of width when window is wider than tall
        let panel_width_ndc = 0.4_f32;
        let max_board_width = 2.0 - panel_width_ndc; // 1.6 in NDC

        if w_to_h > 1.0 {
            // Window is wider than tall
            // Board should be square and fit within height
            let board_width = (2.0 / w_to_h).min(max_board_width);
            self.board_dimensions = (board_width, 2.0);
        } else {
            // Window is taller than wide - use full width (minus panel) and scale height
            let board_width = max_board_width.min(2.0);
            let board_height = board_width * w_to_h;
            self.board_dimensions = (board_width, board_height.min(2.0));
        }
    }

    /// Get the right edge of the board in NDC coordinates (for panel positioning)
    fn board_right_edge_ndc(&self) -> f32 {
        -1.0 + self.board_dimensions.0
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

    // ===========================
    // Menu Helper Methods
    // ===========================

    /// Draw a button rectangle with the given color
    fn draw_button_rect<'a>(
        &self,
        render_pass: &mut wgpu::RenderPass<'a>,
        rect: &layout::ButtonRect,
        color: [f32; 4],
    ) {
        let positions = rect.positions();
        let vertices = [
            TileVertex { position: positions[0], color },
            TileVertex { position: positions[1], color },
            TileVertex { position: positions[2], color },
            TileVertex { position: positions[3], color },
        ];

        let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Menu Button Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.draw_indexed(0..6, 0, 0..1);
    }

    /// Convert NDC Y coordinate to screen Y coordinate
    fn ndc_to_screen_y(&self, ndc_y: f32, viewport_height: f32) -> f32 {
        (1.0 - ndc_y) / 2.0 * viewport_height
    }

    /// Calculate centered X position for text
    fn center_text_x(&self, text_width: f32, viewport_width: f32) -> f32 {
        (viewport_width - text_width) / 2.0
    }

    /// Get text width from a buffer
    fn get_text_width(&self, buffer: &Buffer) -> f32 {
        buffer.layout_runs().map(|r| r.line_w).fold(0.0_f32, |a, b| a.max(b))
    }

    /// Prepare text areas for ModeSelection state
    fn prepare_mode_selection_text(&mut self, viewport_width: f32, viewport_height: f32) -> Vec<OwnedTextArea> {
        let mut result = Vec::new();

        // PvP button text
        let mut pvp_buffer = Buffer::new(&mut self.font_system, Metrics::new(32.0, 40.0));
        pvp_buffer.set_size(&mut self.font_system, Some(viewport_width), Some(viewport_height));
        pvp_buffer.set_text(&mut self.font_system, "Player vs Player", Attrs::new().family(Family::SansSerif), glyphon::Shaping::Advanced);
        let pvp_width = self.get_text_width(&pvp_buffer);
        let pvp_center_y = self.ndc_to_screen_y((layout::main_menu::PVP.top + layout::main_menu::PVP.bottom()) / 2.0, viewport_height);
        result.push(OwnedTextArea {
            buffer: pvp_buffer,
            left: self.center_text_x(pvp_width, viewport_width),
            top: pvp_center_y - 16.0,
            color: glyphon::Color::rgb(0, 0, 0),
        });

        // PvAI button text
        let mut pvai_buffer = Buffer::new(&mut self.font_system, Metrics::new(32.0, 40.0));
        pvai_buffer.set_size(&mut self.font_system, Some(viewport_width), Some(viewport_height));
        pvai_buffer.set_text(&mut self.font_system, "Player vs AI", Attrs::new().family(Family::SansSerif), glyphon::Shaping::Advanced);
        let pvai_width = self.get_text_width(&pvai_buffer);
        let pvai_center_y = self.ndc_to_screen_y((layout::main_menu::PVAI.top + layout::main_menu::PVAI.bottom()) / 2.0, viewport_height);
        result.push(OwnedTextArea {
            buffer: pvai_buffer,
            left: self.center_text_x(pvai_width, viewport_width),
            top: pvai_center_y - 16.0,
            color: glyphon::Color::rgb(0, 0, 0),
        });

        // AIvAI button text
        let mut aivai_buffer = Buffer::new(&mut self.font_system, Metrics::new(32.0, 40.0));
        aivai_buffer.set_size(&mut self.font_system, Some(viewport_width), Some(viewport_height));
        aivai_buffer.set_text(&mut self.font_system, "AI vs AI", Attrs::new().family(Family::SansSerif), glyphon::Shaping::Advanced);
        let aivai_width = self.get_text_width(&aivai_buffer);
        let aivai_center_y = self.ndc_to_screen_y((layout::main_menu::AIVAI.top + layout::main_menu::AIVAI.bottom()) / 2.0, viewport_height);
        result.push(OwnedTextArea {
            buffer: aivai_buffer,
            left: self.center_text_x(aivai_width, viewport_width),
            top: aivai_center_y - 16.0,
            color: glyphon::Color::rgb(0, 0, 0),
        });

        result
    }

    /// Prepare text areas for SideSelection state
    fn prepare_side_selection_text(&mut self, viewport_width: f32, viewport_height: f32) -> Vec<OwnedTextArea> {
        let mut result = Vec::new();

        // Title
        let mut title_buffer = Buffer::new(&mut self.font_system, Metrics::new(40.0, 50.0));
        title_buffer.set_size(&mut self.font_system, Some(viewport_width), Some(viewport_height));
        title_buffer.set_text(&mut self.font_system, "Choose Your Side", Attrs::new().family(Family::SansSerif), glyphon::Shaping::Advanced);
        let title_width = self.get_text_width(&title_buffer);
        result.push(OwnedTextArea {
            buffer: title_buffer,
            left: self.center_text_x(title_width, viewport_width),
            top: 50.0,
            color: glyphon::Color::rgb(255, 255, 255),
        });

        // Play as White button text
        let mut white_buffer = Buffer::new(&mut self.font_system, Metrics::new(32.0, 40.0));
        white_buffer.set_size(&mut self.font_system, Some(viewport_width), Some(viewport_height));
        white_buffer.set_text(&mut self.font_system, "Play as White", Attrs::new().family(Family::SansSerif), glyphon::Shaping::Advanced);
        let white_width = self.get_text_width(&white_buffer);
        let white_center_y = self.ndc_to_screen_y((layout::side_selection::WHITE.top + layout::side_selection::WHITE.bottom()) / 2.0, viewport_height);
        result.push(OwnedTextArea {
            buffer: white_buffer,
            left: self.center_text_x(white_width, viewport_width),
            top: white_center_y - 16.0,
            color: glyphon::Color::rgb(0, 0, 0),
        });

        // Play as Black button text
        let mut black_buffer = Buffer::new(&mut self.font_system, Metrics::new(32.0, 40.0));
        black_buffer.set_size(&mut self.font_system, Some(viewport_width), Some(viewport_height));
        black_buffer.set_text(&mut self.font_system, "Play as Black", Attrs::new().family(Family::SansSerif), glyphon::Shaping::Advanced);
        let black_width = self.get_text_width(&black_buffer);
        let black_center_y = self.ndc_to_screen_y((layout::side_selection::BLACK.top + layout::side_selection::BLACK.bottom()) / 2.0, viewport_height);
        result.push(OwnedTextArea {
            buffer: black_buffer,
            left: self.center_text_x(black_width, viewport_width),
            top: black_center_y - 16.0,
            color: glyphon::Color::rgb(255, 255, 255),
        });

        result
    }

    /// Prepare text areas for DifficultySelection state
    fn prepare_difficulty_selection_text(&mut self, viewport_width: f32, viewport_height: f32, user_color: Color) -> Vec<OwnedTextArea> {
        let mut result = Vec::new();

        // Title based on color
        let title_text = match user_color {
            Color::White => "Select AI Difficulty",
            Color::Black => "Select AI Difficulty",
        };
        let mut title_buffer = Buffer::new(&mut self.font_system, Metrics::new(40.0, 50.0));
        title_buffer.set_size(&mut self.font_system, Some(viewport_width), Some(viewport_height));
        title_buffer.set_text(&mut self.font_system, title_text, Attrs::new().family(Family::SansSerif), glyphon::Shaping::Advanced);
        let title_width = self.get_text_width(&title_buffer);
        result.push(OwnedTextArea {
            buffer: title_buffer,
            left: self.center_text_x(title_width, viewport_width),
            top: 50.0,
            color: glyphon::Color::rgb(255, 255, 255),
        });

        // Difficulty button labels
        let difficulties = ["Easy", "Medium", "Hard", "Expert"];
        let buttons = layout::difficulty::single_buttons();
        for (i, text) in difficulties.iter().enumerate() {
            let mut buffer = Buffer::new(&mut self.font_system, Metrics::new(18.0, 24.0));
            buffer.set_size(&mut self.font_system, Some(viewport_width), Some(viewport_height));
            buffer.set_text(&mut self.font_system, text, Attrs::new().family(Family::SansSerif), glyphon::Shaping::Advanced);
            let text_width = self.get_text_width(&buffer);

            let button_center_x = buttons[i].left + buttons[i].width / 2.0;
            let button_center_y = (buttons[i].top + buttons[i].bottom()) / 2.0;
            let screen_x = (button_center_x + 1.0) / 2.0 * viewport_width - text_width / 2.0;
            let screen_y = self.ndc_to_screen_y(button_center_y, viewport_height) - 10.0;

            result.push(OwnedTextArea {
                buffer,
                left: screen_x,
                top: screen_y,
                color: glyphon::Color::rgb(255, 255, 255),
            });
        }

        result
    }

    /// Prepare text areas for AIvAISetup state
    fn prepare_aivai_setup_text(&mut self, viewport_width: f32, viewport_height: f32) -> Vec<OwnedTextArea> {
        let mut result = Vec::new();

        // Title
        let mut title_buffer = Buffer::new(&mut self.font_system, Metrics::new(32.0, 40.0));
        title_buffer.set_size(&mut self.font_system, Some(viewport_width), Some(viewport_height));
        title_buffer.set_text(&mut self.font_system, "AI vs AI Setup", Attrs::new().family(Family::SansSerif), glyphon::Shaping::Advanced);
        let title_width = self.get_text_width(&title_buffer);
        result.push(OwnedTextArea {
            buffer: title_buffer,
            left: self.center_text_x(title_width, viewport_width),
            top: self.ndc_to_screen_y(0.7, viewport_height),
            color: glyphon::Color::rgb(255, 255, 255),
        });

        // White AI label
        let mut white_label = Buffer::new(&mut self.font_system, Metrics::new(24.0, 30.0));
        white_label.set_size(&mut self.font_system, Some(viewport_width), Some(viewport_height));
        white_label.set_text(&mut self.font_system, "White AI", Attrs::new().family(Family::SansSerif), glyphon::Shaping::Advanced);
        let white_label_width = self.get_text_width(&white_label);
        result.push(OwnedTextArea {
            buffer: white_label,
            left: self.center_text_x(white_label_width, viewport_width),
            top: self.ndc_to_screen_y(0.45, viewport_height),
            color: glyphon::Color::rgb(255, 255, 255),
        });

        // White difficulty button labels
        let difficulties = ["Easy", "Medium", "Hard", "Expert"];
        let white_buttons = layout::difficulty::white_buttons();
        for (i, text) in difficulties.iter().enumerate() {
            let mut buffer = Buffer::new(&mut self.font_system, Metrics::new(16.0, 20.0));
            buffer.set_size(&mut self.font_system, Some(viewport_width), Some(viewport_height));
            buffer.set_text(&mut self.font_system, text, Attrs::new().family(Family::SansSerif), glyphon::Shaping::Advanced);
            let text_width = self.get_text_width(&buffer);

            let button_center_x = white_buttons[i].left + white_buttons[i].width / 2.0;
            let button_center_y = (white_buttons[i].top + white_buttons[i].bottom()) / 2.0;
            let screen_x = (button_center_x + 1.0) / 2.0 * viewport_width - text_width / 2.0;
            let screen_y = self.ndc_to_screen_y(button_center_y, viewport_height) - 8.0;

            result.push(OwnedTextArea {
                buffer,
                left: screen_x,
                top: screen_y,
                color: glyphon::Color::rgb(255, 255, 255),
            });
        }

        // Black AI label
        let mut black_label = Buffer::new(&mut self.font_system, Metrics::new(24.0, 30.0));
        black_label.set_size(&mut self.font_system, Some(viewport_width), Some(viewport_height));
        black_label.set_text(&mut self.font_system, "Black AI", Attrs::new().family(Family::SansSerif), glyphon::Shaping::Advanced);
        let black_label_width = self.get_text_width(&black_label);
        result.push(OwnedTextArea {
            buffer: black_label,
            left: self.center_text_x(black_label_width, viewport_width),
            top: self.ndc_to_screen_y(-0.05, viewport_height),
            color: glyphon::Color::rgb(255, 255, 255),
        });

        // Black difficulty button labels
        let black_buttons = layout::difficulty::black_buttons();
        for (i, text) in difficulties.iter().enumerate() {
            let mut buffer = Buffer::new(&mut self.font_system, Metrics::new(16.0, 20.0));
            buffer.set_size(&mut self.font_system, Some(viewport_width), Some(viewport_height));
            buffer.set_text(&mut self.font_system, text, Attrs::new().family(Family::SansSerif), glyphon::Shaping::Advanced);
            let text_width = self.get_text_width(&buffer);

            let button_center_x = black_buttons[i].left + black_buttons[i].width / 2.0;
            let button_center_y = (black_buttons[i].top + black_buttons[i].bottom()) / 2.0;
            let screen_x = (button_center_x + 1.0) / 2.0 * viewport_width - text_width / 2.0;
            let screen_y = self.ndc_to_screen_y(button_center_y, viewport_height) - 8.0;

            result.push(OwnedTextArea {
                buffer,
                left: screen_x,
                top: screen_y,
                color: glyphon::Color::rgb(255, 255, 255),
            });
        }

        // Start button text
        let mut start_buffer = Buffer::new(&mut self.font_system, Metrics::new(24.0, 30.0));
        start_buffer.set_size(&mut self.font_system, Some(viewport_width), Some(viewport_height));
        start_buffer.set_text(&mut self.font_system, "Start Game", Attrs::new().family(Family::SansSerif), glyphon::Shaping::Advanced);
        let start_width = self.get_text_width(&start_buffer);
        let start_center_y = (layout::difficulty::START.top + layout::difficulty::START.bottom()) / 2.0;
        result.push(OwnedTextArea {
            buffer: start_buffer,
            left: self.center_text_x(start_width, viewport_width),
            top: self.ndc_to_screen_y(start_center_y, viewport_height) - 12.0,
            color: glyphon::Color::rgb(255, 255, 255),
        });

        result
    }
}

/// Owned text area data for preparing text rendering
struct OwnedTextArea {
    buffer: Buffer,
    left: f32,
    top: f32,
    color: glyphon::Color,
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

            // Draw controls bar buttons
            render_pass.set_pipeline(&self.tile_pipeline);
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            // Controls bar at bottom: y from 0.75 to 0.95 in NDC
            let bar_top = 0.75_f32;
            let bar_bottom = 0.95_f32;
            let button_width = 0.15_f32;
            let button_spacing = 0.05_f32;

            // Undo button (left) - always enabled for now (state handled by orchestrator)
            let undo_color = [0.4, 0.5, 0.4, 1.0];
            let undo_left = -0.25_f32;
            let undo_right = undo_left + button_width;

            let undo_vertices = [
                TileVertex { position: [undo_left, bar_top], color: undo_color },
                TileVertex { position: [undo_right, bar_top], color: undo_color },
                TileVertex { position: [undo_left, bar_bottom], color: undo_color },
                TileVertex { position: [undo_right, bar_bottom], color: undo_color },
            ];

            let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Undo Button Vertex Buffer"),
                contents: bytemuck::cast_slice(&undo_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.draw_indexed(0..6, 0, 0..1);

            // Redo button (center)
            let redo_color = [0.4, 0.5, 0.4, 1.0];
            let redo_left = undo_right + button_spacing;
            let redo_right = redo_left + button_width;

            let redo_vertices = [
                TileVertex { position: [redo_left, bar_top], color: redo_color },
                TileVertex { position: [redo_right, bar_top], color: redo_color },
                TileVertex { position: [redo_left, bar_bottom], color: redo_color },
                TileVertex { position: [redo_right, bar_bottom], color: redo_color },
            ];

            let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Redo Button Vertex Buffer"),
                contents: bytemuck::cast_slice(&redo_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.draw_indexed(0..6, 0, 0..1);

            // Flip button (right)
            let flip_color = [0.4, 0.4, 0.5, 1.0];
            let flip_left = redo_right + button_spacing;
            let flip_right = flip_left + button_width;

            let flip_vertices = [
                TileVertex { position: [flip_left, bar_top], color: flip_color },
                TileVertex { position: [flip_right, bar_top], color: flip_color },
                TileVertex { position: [flip_left, bar_bottom], color: flip_color },
                TileVertex { position: [flip_right, bar_bottom], color: flip_color },
            ];

            let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Flip Button Vertex Buffer"),
                contents: bytemuck::cast_slice(&flip_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.draw_indexed(0..6, 0, 0..1);
        }

        // Prepare and render control button text labels
        let viewport_width = self.window_size.0 as f32;
        let viewport_height = self.window_size.1 as f32;

        self.viewport.update(&self.queue, glyphon::Resolution {
            width: self.window_size.0,
            height: self.window_size.1,
        });

        let mut undo_buffer = Buffer::new(&mut self.font_system, Metrics::new(24.0, 28.0));
        undo_buffer.set_size(&mut self.font_system, Some(viewport_width), Some(viewport_height));
        undo_buffer.set_text(&mut self.font_system, "<", Attrs::new().family(Family::SansSerif), glyphon::Shaping::Advanced);

        let mut redo_buffer = Buffer::new(&mut self.font_system, Metrics::new(24.0, 28.0));
        redo_buffer.set_size(&mut self.font_system, Some(viewport_width), Some(viewport_height));
        redo_buffer.set_text(&mut self.font_system, ">", Attrs::new().family(Family::SansSerif), glyphon::Shaping::Advanced);

        let mut flip_buffer = Buffer::new(&mut self.font_system, Metrics::new(24.0, 28.0));
        flip_buffer.set_size(&mut self.font_system, Some(viewport_width), Some(viewport_height));
        flip_buffer.set_text(&mut self.font_system, "R", Attrs::new().family(Family::SansSerif), glyphon::Shaping::Advanced);

        let bar_center_ndc_y = (0.75 + 0.95) / 2.0;
        let bar_center_y = (1.0 + bar_center_ndc_y) / 2.0 * viewport_height;

        let undo_center_x = (1.0 + (-0.25 + 0.075)) / 2.0 * viewport_width;
        let redo_center_x = (1.0 + (-0.05 + 0.075)) / 2.0 * viewport_width;
        let flip_center_x = (1.0 + (0.15 + 0.075)) / 2.0 * viewport_width;

        let text_areas = [
            TextArea {
                buffer: &undo_buffer,
                left: undo_center_x - 8.0,
                top: bar_center_y - 12.0,
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
                buffer: &redo_buffer,
                left: redo_center_x - 8.0,
                top: bar_center_y - 12.0,
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
                buffer: &flip_buffer,
                left: flip_center_x - 8.0,
                top: bar_center_y - 12.0,
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
                label: Some("Controls Text Render Pass"),
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

        // Convert screen coordinates to NDC (-1 to 1)
        let ndc_x = (adjusted_x / self.window_size.0 as f64) * 2.0 - 1.0;
        let ndc_y = 1.0 - (adjusted_y / self.window_size.1 as f64) * 2.0;

        // Board bounds in NDC: x from -1 to -1+board_w, y from 1-board_h to 1
        let board_left = -1.0;
        let board_right = -1.0 + self.board_dimensions.0 as f64;
        let board_top = 1.0;
        let board_bottom = 1.0 - self.board_dimensions.1 as f64;

        // Check if click is within board bounds
        if ndc_x < board_left || ndc_x >= board_right || ndc_y > board_top || ndc_y <= board_bottom {
            return None;
        }

        // Convert NDC to board-relative coordinates (0 to 1)
        let board_x = (ndc_x - board_left) / self.board_dimensions.0 as f64;
        let board_y = (board_top - ndc_y) / self.board_dimensions.1 as f64;

        // Convert to tile indices (0-7)
        let tile_x = (board_x * 8.0).floor() as usize;
        let tile_y = (board_y * 8.0).floor() as usize;

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
                // Draw three buttons: PvP, PvAI, AIvAI
                // PvP button (top) - greenish
                let pvp_vertices = [
                    TileVertex { position: [-0.5, 0.45], color: [0.5, 0.7, 0.5, 1.0] },
                    TileVertex { position: [0.5, 0.45], color: [0.5, 0.7, 0.5, 1.0] },
                    TileVertex { position: [-0.5, 0.25], color: [0.5, 0.7, 0.5, 1.0] },
                    TileVertex { position: [0.5, 0.25], color: [0.5, 0.7, 0.5, 1.0] },
                ];

                let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("PvP Button Vertex Buffer"),
                    contents: bytemuck::cast_slice(&pvp_vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });

                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.draw_indexed(0..6, 0, 0..1);

                // PvAI button (middle) - blueish
                let pvai_vertices = [
                    TileVertex { position: [-0.5, 0.1], color: [0.5, 0.6, 0.8, 1.0] },
                    TileVertex { position: [0.5, 0.1], color: [0.5, 0.6, 0.8, 1.0] },
                    TileVertex { position: [-0.5, -0.1], color: [0.5, 0.6, 0.8, 1.0] },
                    TileVertex { position: [0.5, -0.1], color: [0.5, 0.6, 0.8, 1.0] },
                ];

                let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("PvAI Button Vertex Buffer"),
                    contents: bytemuck::cast_slice(&pvai_vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });

                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.draw_indexed(0..6, 0, 0..1);

                // AIvAI button (bottom) - purplish
                let aivai_vertices = [
                    TileVertex { position: [-0.5, -0.25], color: [0.6, 0.5, 0.7, 1.0] },
                    TileVertex { position: [0.5, -0.25], color: [0.6, 0.5, 0.7, 1.0] },
                    TileVertex { position: [-0.5, -0.45], color: [0.6, 0.5, 0.7, 1.0] },
                    TileVertex { position: [0.5, -0.45], color: [0.6, 0.5, 0.7, 1.0] },
                ];

                let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("AIvAI Button Vertex Buffer"),
                    contents: bytemuck::cast_slice(&aivai_vertices),
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

            // PvP button is at [-0.5, 0.45] to [0.5, 0.25] in NDC
            // NDC to Screen: screen_y = (1 - ndc_y) / 2 * height
            let pvp_ndc_center_y = (0.45 + 0.25) / 2.0; // 0.35
            let pvp_center_y = (1.0 - pvp_ndc_center_y) / 2.0 * viewport_height;

            let layout = pvp_buffer.layout_runs();
            let mut text_width: f32 = 0.0;
            for run in layout {
                text_width = text_width.max(run.line_w);
            }

            let pvp_text_x = (viewport_width / 2.0 - text_width / 2.0).max(0.0);
            let pvp_text_y = (pvp_center_y - 16.0).max(0.0);

            // "Player vs AI" text - centered on blue button
            let mut pvai_buffer = Buffer::new(&mut self.font_system, Metrics::new(32.0, 40.0));
            pvai_buffer.set_size(&mut self.font_system, Some(viewport_width), Some(viewport_height));
            pvai_buffer.set_text(&mut self.font_system, "Player vs AI", Attrs::new().family(Family::SansSerif), glyphon::Shaping::Advanced);

            // PvAI button is at [-0.5, 0.1] to [0.5, -0.1] in NDC
            let pvai_ndc_center_y = (0.1 + -0.1) / 2.0; // 0.0
            let pvai_center_y = (1.0 - pvai_ndc_center_y) / 2.0 * viewport_height;

            let layout2 = pvai_buffer.layout_runs();
            let mut text_width2: f32 = 0.0;
            for run in layout2 {
                text_width2 = text_width2.max(run.line_w);
            }

            let pvai_text_x = (viewport_width / 2.0 - text_width2 / 2.0).max(0.0);
            let pvai_text_y = (pvai_center_y - 16.0).max(0.0);

            // "AI vs AI" text - centered on purple button
            let mut aivai_buffer = Buffer::new(&mut self.font_system, Metrics::new(32.0, 40.0));
            aivai_buffer.set_size(&mut self.font_system, Some(viewport_width), Some(viewport_height));
            aivai_buffer.set_text(&mut self.font_system, "AI vs AI", Attrs::new().family(Family::SansSerif), glyphon::Shaping::Advanced);

            // AIvAI button is at [-0.5, -0.25] to [0.5, -0.45] in NDC
            let aivai_ndc_center_y = (-0.25 + -0.45) / 2.0; // -0.35
            let aivai_center_y = (1.0 - aivai_ndc_center_y) / 2.0 * viewport_height;

            let layout3 = aivai_buffer.layout_runs();
            let mut text_width3: f32 = 0.0;
            for run in layout3 {
                text_width3 = text_width3.max(run.line_w);
            }

            let aivai_text_x = (viewport_width / 2.0 - text_width3 / 2.0).max(0.0);
            let aivai_text_y = (aivai_center_y - 16.0).max(0.0);

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
                TextArea {
                    buffer: &aivai_buffer,
                    left: aivai_text_x,
                    top: aivai_text_y,
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
                // PvP button (top): NDC y [0.45, 0.25] -> screen norm_y in [-0.45, -0.25]
                norm_x >= -0.5 && norm_x <= 0.5 && norm_y >= -0.45 && norm_y <= -0.25
            }
            1 => {
                // PvAI button (middle): NDC y [0.1, -0.1] -> screen norm_y in [-0.1, 0.1]
                norm_x >= -0.5 && norm_x <= 0.5 && norm_y >= -0.1 && norm_y <= 0.1
            }
            2 => {
                // AIvAI button (bottom): NDC y [-0.25, -0.45] -> screen norm_y in [0.25, 0.45]
                norm_x >= -0.5 && norm_x <= 0.5 && norm_y >= 0.25 && norm_y <= 0.45
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

    // ===========================
    // Game Controls Panel (Right Side)
    // ===========================

    fn draw_controls_bar(&mut self, can_undo: bool, can_redo: bool) {
        // Controls panel on the right side of the board
        // Two buttons stacked vertically: [Prev] and [Next]

        let output = self.surface.get_current_texture().unwrap();
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Controls Panel Render Encoder"),
        });

        // Calculate panel position based on board dimensions
        let board_right = self.board_right_edge_ndc();
        let panel_left = board_right + 0.05; // Small gap from board
        let panel_right = 0.95_f32; // Near right edge

        // Button dimensions in NDC
        let button_height = 0.15_f32;
        let button_spacing = 0.1_f32;

        // Prev button (top) - corresponds to Undo
        let prev_top = -0.1_f32;
        let prev_bottom = prev_top + button_height;

        // Next button (bottom) - corresponds to Redo
        let next_top = prev_bottom + button_spacing;
        let next_bottom = next_top + button_height;

        // Draw button backgrounds
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Controls Panel Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // Load existing content (board)
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.tile_pipeline);
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            // Prev button
            let prev_color = if can_undo { [0.35, 0.45, 0.35, 1.0] } else { [0.25, 0.25, 0.25, 0.7] };
            let prev_vertices = [
                TileVertex { position: [panel_left, prev_top], color: prev_color },
                TileVertex { position: [panel_right, prev_top], color: prev_color },
                TileVertex { position: [panel_left, prev_bottom], color: prev_color },
                TileVertex { position: [panel_right, prev_bottom], color: prev_color },
            ];

            let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Prev Button Vertex Buffer"),
                contents: bytemuck::cast_slice(&prev_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.draw_indexed(0..6, 0, 0..1);

            // Next button
            let next_color = if can_redo { [0.35, 0.45, 0.35, 1.0] } else { [0.25, 0.25, 0.25, 0.7] };
            let next_vertices = [
                TileVertex { position: [panel_left, next_top], color: next_color },
                TileVertex { position: [panel_right, next_top], color: next_color },
                TileVertex { position: [panel_left, next_bottom], color: next_color },
                TileVertex { position: [panel_right, next_bottom], color: next_color },
            ];

            let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Next Button Vertex Buffer"),
                contents: bytemuck::cast_slice(&next_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.draw_indexed(0..6, 0, 0..1);
        }

        // Render text labels
        let viewport_width = self.window_size.0 as f32;
        let viewport_height = self.window_size.1 as f32;

        self.viewport.update(&self.queue, glyphon::Resolution {
            width: self.window_size.0,
            height: self.window_size.1,
        });

        // Create text buffers for button labels
        let mut prev_buffer = Buffer::new(&mut self.font_system, Metrics::new(20.0, 24.0));
        prev_buffer.set_size(&mut self.font_system, Some(viewport_width), Some(viewport_height));
        prev_buffer.set_text(&mut self.font_system, "Prev", Attrs::new().family(Family::SansSerif), glyphon::Shaping::Advanced);
        let prev_text_width: f32 = prev_buffer.layout_runs().map(|r| r.line_w).sum();

        let mut next_buffer = Buffer::new(&mut self.font_system, Metrics::new(20.0, 24.0));
        next_buffer.set_size(&mut self.font_system, Some(viewport_width), Some(viewport_height));
        next_buffer.set_text(&mut self.font_system, "Next", Attrs::new().family(Family::SansSerif), glyphon::Shaping::Advanced);
        let next_text_width: f32 = next_buffer.layout_runs().map(|r| r.line_w).sum();

        // Calculate button centers in screen coordinates
        let button_center_x_ndc = (panel_left + panel_right) / 2.0;
        let button_center_x = (1.0 + button_center_x_ndc) / 2.0 * viewport_width;

        let prev_center_y_ndc = (prev_top + prev_bottom) / 2.0;
        let prev_center_y = (1.0 - prev_center_y_ndc) / 2.0 * viewport_height;

        let next_center_y_ndc = (next_top + next_bottom) / 2.0;
        let next_center_y = (1.0 - next_center_y_ndc) / 2.0 * viewport_height;

        let text_areas = [
            TextArea {
                buffer: &prev_buffer,
                left: button_center_x - prev_text_width / 2.0,
                top: prev_center_y - 10.0,
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
                buffer: &next_buffer,
                left: button_center_x - next_text_width / 2.0,
                top: next_center_y - 10.0,
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
                label: Some("Controls Text Render Pass"),
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

    fn get_control_action_at_coords(&self, coords: PhysicalPosition<f64>) -> Option<super::ControlAction> {
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

        // Panel position (must match draw_controls_bar)
        let board_right = self.board_right_edge_ndc() as f64;
        let panel_left = board_right + 0.05;
        let panel_right = 0.95_f64;

        // Button dimensions
        let button_height = 0.15_f64;
        let button_spacing = 0.1_f64;

        // Prev button bounds
        let prev_top = -0.1_f64;
        let prev_bottom = prev_top + button_height;

        // Next button bounds
        let next_top = prev_bottom + button_spacing;
        let next_bottom = next_top + button_height;

        // Check X range (must be in panel)
        if norm_x < panel_left || norm_x > panel_right {
            return None;
        }

        // Check which button (Y is inverted in NDC)
        if norm_y >= prev_top && norm_y <= prev_bottom {
            Some(super::ControlAction::Undo)
        } else if norm_y >= next_top && norm_y <= next_bottom {
            Some(super::ControlAction::Redo)
        } else {
            None
        }
    }

    // ===========================
    // AI Setup Screen (Combined)
    // ===========================

    fn draw_ai_setup(
        &mut self,
        _ai_types: &[crate::agent::ai::AIType],
        _white_type_index: usize,
        white_difficulty: crate::agent::ai::Difficulty,
        _black_type_index: usize,
        black_difficulty: crate::agent::ai::Difficulty,
        pressed_button: Option<AISetupButton>,
    ) {
        let output = self.surface.get_current_texture().unwrap();
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("AI Setup Render Encoder"),
        });

        // Layout constants - combined screen with White and Black sections
        let diff_button_width = 0.18_f32;
        let diff_button_height = 0.12_f32;
        let diff_start_x = -0.45_f32;
        let diff_spacing = 0.05_f32;
        let border_width = 0.008_f32; // Border thickness

        // White section: y from 0.15 to 0.55
        let white_diff_y_top = 0.15_f32;
        let white_diff_y_bottom = white_diff_y_top + diff_button_height;

        // Black section: y from -0.35 to 0.05
        let black_diff_y_top = -0.35_f32;
        let black_diff_y_bottom = black_diff_y_top + diff_button_height;

        // Clear background and draw buttons
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("AI Setup Background Render Pass"),
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

            // Draw White difficulty buttons with borders
            for i in 0..4 {
                let is_selected = match white_difficulty {
                    crate::agent::ai::Difficulty::Easy => i == 0,
                    crate::agent::ai::Difficulty::Medium => i == 1,
                    crate::agent::ai::Difficulty::Hard => i == 2,
                    crate::agent::ai::Difficulty::Expert => i == 3,
                };

                let is_pressed = matches!(pressed_button, Some(AISetupButton::WhiteDifficulty(idx)) if idx == i);

                let left = diff_start_x + (i as f32) * (diff_button_width + diff_spacing);
                let right = left + diff_button_width;

                // Border color - brighter when pressed
                let border_color = if is_pressed {
                    [0.9, 1.0, 0.9, 1.0] // Bright green border when pressed
                } else {
                    [0.6, 0.7, 0.6, 1.0] // Normal border
                };

                // Draw border (outer rectangle)
                let border_vertices = [
                    TileVertex { position: [left - border_width, white_diff_y_top - border_width], color: border_color },
                    TileVertex { position: [right + border_width, white_diff_y_top - border_width], color: border_color },
                    TileVertex { position: [left - border_width, white_diff_y_bottom + border_width], color: border_color },
                    TileVertex { position: [right + border_width, white_diff_y_bottom + border_width], color: border_color },
                ];

                let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("White Difficulty Button Border"),
                    contents: bytemuck::cast_slice(&border_vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.draw_indexed(0..6, 0, 0..1);

                // Inner button color - slightly darker when pressed
                let color = if is_pressed {
                    [0.4, 0.55, 0.4, 1.0] // Darker when pressed
                } else if is_selected {
                    [0.5, 0.7, 0.5, 1.0] // Highlighted green
                } else {
                    [0.35, 0.4, 0.35, 1.0] // Normal
                };

                // Draw inner button
                let vertices = [
                    TileVertex { position: [left, white_diff_y_top], color },
                    TileVertex { position: [right, white_diff_y_top], color },
                    TileVertex { position: [left, white_diff_y_bottom], color },
                    TileVertex { position: [right, white_diff_y_bottom], color },
                ];

                let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("White Difficulty Button"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.draw_indexed(0..6, 0, 0..1);
            }

            // Draw Black difficulty buttons with borders
            for i in 0..4 {
                let is_selected = match black_difficulty {
                    crate::agent::ai::Difficulty::Easy => i == 0,
                    crate::agent::ai::Difficulty::Medium => i == 1,
                    crate::agent::ai::Difficulty::Hard => i == 2,
                    crate::agent::ai::Difficulty::Expert => i == 3,
                };

                let is_pressed = matches!(pressed_button, Some(AISetupButton::BlackDifficulty(idx)) if idx == i);

                let left = diff_start_x + (i as f32) * (diff_button_width + diff_spacing);
                let right = left + diff_button_width;

                // Border color - brighter when pressed
                let border_color = if is_pressed {
                    [0.9, 0.9, 1.0, 1.0] // Bright blue border when pressed
                } else {
                    [0.6, 0.6, 0.7, 1.0] // Normal border
                };

                // Draw border (outer rectangle)
                let border_vertices = [
                    TileVertex { position: [left - border_width, black_diff_y_top - border_width], color: border_color },
                    TileVertex { position: [right + border_width, black_diff_y_top - border_width], color: border_color },
                    TileVertex { position: [left - border_width, black_diff_y_bottom + border_width], color: border_color },
                    TileVertex { position: [right + border_width, black_diff_y_bottom + border_width], color: border_color },
                ];

                let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Black Difficulty Button Border"),
                    contents: bytemuck::cast_slice(&border_vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.draw_indexed(0..6, 0, 0..1);

                // Inner button color - slightly darker when pressed
                let color = if is_pressed {
                    [0.4, 0.4, 0.55, 1.0] // Darker when pressed
                } else if is_selected {
                    [0.5, 0.5, 0.7, 1.0] // Highlighted blue
                } else {
                    [0.35, 0.35, 0.4, 1.0] // Normal
                };

                // Draw inner button
                let vertices = [
                    TileVertex { position: [left, black_diff_y_top], color },
                    TileVertex { position: [right, black_diff_y_top], color },
                    TileVertex { position: [left, black_diff_y_bottom], color },
                    TileVertex { position: [right, black_diff_y_bottom], color },
                ];

                let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Black Difficulty Button"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.draw_indexed(0..6, 0, 0..1);
            }

            // Draw Start button at bottom with border
            let is_start_pressed = matches!(pressed_button, Some(AISetupButton::Start));

            // Start button border
            let start_border_color = if is_start_pressed {
                [0.9, 1.0, 0.9, 1.0] // Bright when pressed
            } else {
                [0.5, 0.65, 0.5, 1.0] // Normal border
            };

            let start_border_vertices = [
                TileVertex { position: [-0.3 - border_width, -0.6 - border_width], color: start_border_color },
                TileVertex { position: [0.3 + border_width, -0.6 - border_width], color: start_border_color },
                TileVertex { position: [-0.3 - border_width, -0.75 + border_width], color: start_border_color },
                TileVertex { position: [0.3 + border_width, -0.75 + border_width], color: start_border_color },
            ];

            let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Start Button Border"),
                contents: bytemuck::cast_slice(&start_border_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.draw_indexed(0..6, 0, 0..1);

            // Start button inner
            let start_color = if is_start_pressed {
                [0.3, 0.45, 0.3, 1.0] // Darker when pressed
            } else {
                [0.4, 0.55, 0.4, 1.0] // Normal
            };

            let start_vertices = [
                TileVertex { position: [-0.3, -0.6], color: start_color },
                TileVertex { position: [0.3, -0.6], color: start_color },
                TileVertex { position: [-0.3, -0.75], color: start_color },
                TileVertex { position: [0.3, -0.75], color: start_color },
            ];

            let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Start Button"),
                contents: bytemuck::cast_slice(&start_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.draw_indexed(0..6, 0, 0..1);
        }

        // Render text
        let viewport_width = self.window_size.0 as f32;
        let viewport_height = self.window_size.1 as f32;

        self.viewport.update(&self.queue, glyphon::Resolution {
            width: self.window_size.0,
            height: self.window_size.1,
        });

        // Title
        let mut title_buffer = Buffer::new(&mut self.font_system, Metrics::new(32.0, 40.0));
        title_buffer.set_size(&mut self.font_system, Some(viewport_width), Some(viewport_height));
        title_buffer.set_text(&mut self.font_system, "AI vs AI Setup", Attrs::new().family(Family::SansSerif), glyphon::Shaping::Advanced);

        // Get title width for centering
        let title_width: f32 = title_buffer.layout_runs().map(|r| r.line_w).sum();
        let title_x = (viewport_width - title_width) / 2.0;

        // White AI label
        let mut white_label = Buffer::new(&mut self.font_system, Metrics::new(24.0, 30.0));
        white_label.set_size(&mut self.font_system, Some(viewport_width), Some(viewport_height));
        white_label.set_text(&mut self.font_system, "White AI", Attrs::new().family(Family::SansSerif), glyphon::Shaping::Advanced);
        let white_label_width: f32 = white_label.layout_runs().map(|r| r.line_w).sum();
        let white_label_x = (viewport_width - white_label_width) / 2.0;

        // Black AI label
        let mut black_label = Buffer::new(&mut self.font_system, Metrics::new(24.0, 30.0));
        black_label.set_size(&mut self.font_system, Some(viewport_width), Some(viewport_height));
        black_label.set_text(&mut self.font_system, "Black AI", Attrs::new().family(Family::SansSerif), glyphon::Shaping::Advanced);
        let black_label_width: f32 = black_label.layout_runs().map(|r| r.line_w).sum();
        let black_label_x = (viewport_width - black_label_width) / 2.0;

        // Difficulty labels
        let difficulties = ["Easy", "Medium", "Hard", "Expert"];
        let diff_buffers: Vec<Buffer> = difficulties.iter().map(|text| {
            let mut buffer = Buffer::new(&mut self.font_system, Metrics::new(16.0, 20.0));
            buffer.set_size(&mut self.font_system, Some(viewport_width), Some(viewport_height));
            buffer.set_text(&mut self.font_system, text, Attrs::new().family(Family::SansSerif), glyphon::Shaping::Advanced);
            buffer
        }).collect();

        // Start button text
        let mut start_buffer = Buffer::new(&mut self.font_system, Metrics::new(24.0, 30.0));
        start_buffer.set_size(&mut self.font_system, Some(viewport_width), Some(viewport_height));
        start_buffer.set_text(&mut self.font_system, "Start Game", Attrs::new().family(Family::SansSerif), glyphon::Shaping::Advanced);
        let start_width: f32 = start_buffer.layout_runs().map(|r| r.line_w).sum();
        let start_x = (viewport_width - start_width) / 2.0;

        // Calculate Y positions (NDC to screen)
        let title_y = (1.0 - 0.7) / 2.0 * viewport_height;
        let white_label_y = (1.0 - 0.45) / 2.0 * viewport_height;
        let white_diff_center_y = (1.0 - (white_diff_y_top + white_diff_y_bottom) / 2.0) / 2.0 * viewport_height;
        let black_label_y = (1.0 - (-0.05)) / 2.0 * viewport_height;
        let black_diff_center_y = (1.0 - (black_diff_y_top + black_diff_y_bottom) / 2.0) / 2.0 * viewport_height;
        let start_y = (1.0 - (-0.675)) / 2.0 * viewport_height;

        let mut text_areas = vec![
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
                buffer: &white_label,
                left: white_label_x,
                top: white_label_y,
                scale: 1.0,
                bounds: glyphon::TextBounds {
                    left: 0,
                    top: 0,
                    right: viewport_width as i32,
                    bottom: viewport_height as i32,
                },
                default_color: glyphon::Color::rgb(220, 220, 220),
                custom_glyphs: &[],
            },
            TextArea {
                buffer: &black_label,
                left: black_label_x,
                top: black_label_y,
                scale: 1.0,
                bounds: glyphon::TextBounds {
                    left: 0,
                    top: 0,
                    right: viewport_width as i32,
                    bottom: viewport_height as i32,
                },
                default_color: glyphon::Color::rgb(180, 180, 220),
                custom_glyphs: &[],
            },
            TextArea {
                buffer: &start_buffer,
                left: start_x,
                top: start_y - 12.0,
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

        // Add White difficulty button text areas (centered)
        for (i, buffer) in diff_buffers.iter().enumerate() {
            let left = diff_start_x + (i as f32) * (diff_button_width + diff_spacing);
            let button_center_x = (1.0 + (left + diff_button_width / 2.0)) / 2.0 * viewport_width;
            let text_width: f32 = buffer.layout_runs().map(|r| r.line_w).sum();
            text_areas.push(TextArea {
                buffer,
                left: button_center_x - text_width / 2.0,
                top: white_diff_center_y - 10.0,
                scale: 1.0,
                bounds: glyphon::TextBounds {
                    left: 0,
                    top: 0,
                    right: viewport_width as i32,
                    bottom: viewport_height as i32,
                },
                default_color: glyphon::Color::rgb(255, 255, 255),
                custom_glyphs: &[],
            });
        }

        // Add Black difficulty button text areas (centered)
        for (i, buffer) in diff_buffers.iter().enumerate() {
            let left = diff_start_x + (i as f32) * (diff_button_width + diff_spacing);
            let button_center_x = (1.0 + (left + diff_button_width / 2.0)) / 2.0 * viewport_width;
            let text_width: f32 = buffer.layout_runs().map(|r| r.line_w).sum();
            text_areas.push(TextArea {
                buffer,
                left: button_center_x - text_width / 2.0,
                top: black_diff_center_y - 10.0,
                scale: 1.0,
                bounds: glyphon::TextBounds {
                    left: 0,
                    top: 0,
                    right: viewport_width as i32,
                    bottom: viewport_height as i32,
                },
                default_color: glyphon::Color::rgb(255, 255, 255),
                custom_glyphs: &[],
            });
        }

        self.text_renderer.prepare(
            &self.device,
            &self.queue,
            &mut self.font_system,
            &mut self.text_atlas,
            &self.viewport,
            text_areas,
            &mut self.swash_cache,
        ).unwrap();

        // Render text pass
        {
            let mut text_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("AI Setup Text Render Pass"),
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

    fn get_white_difficulty_at_coords(&self, coords: PhysicalPosition<f64>) -> Option<usize> {
        #[cfg(target_arch = "wasm32")]
        let (adjusted_x, adjusted_y) = {
            let scale_factor = self.window.scale_factor();
            (coords.x / scale_factor, coords.y / scale_factor)
        };

        #[cfg(not(target_arch = "wasm32"))]
        let (adjusted_x, adjusted_y) = (coords.x, coords.y);

        let norm_x = (adjusted_x / self.window_size.0 as f64) * 2.0 - 1.0;
        let norm_y = (adjusted_y / self.window_size.1 as f64) * 2.0 - 1.0;

        // White difficulty buttons: y in [0.15, 0.27] (NDC), screen y in [-0.27, -0.15]
        if norm_y < -0.27 || norm_y > -0.15 {
            return None;
        }

        let diff_button_width = 0.18_f64;
        let diff_start_x = -0.45_f64;
        let diff_spacing = 0.05_f64;

        for i in 0..4 {
            let left = diff_start_x + (i as f64) * (diff_button_width + diff_spacing);
            let right = left + diff_button_width;
            if norm_x >= left && norm_x <= right {
                return Some(i);
            }
        }

        None
    }

    fn get_black_difficulty_at_coords(&self, coords: PhysicalPosition<f64>) -> Option<usize> {
        #[cfg(target_arch = "wasm32")]
        let (adjusted_x, adjusted_y) = {
            let scale_factor = self.window.scale_factor();
            (coords.x / scale_factor, coords.y / scale_factor)
        };

        #[cfg(not(target_arch = "wasm32"))]
        let (adjusted_x, adjusted_y) = (coords.x, coords.y);

        let norm_x = (adjusted_x / self.window_size.0 as f64) * 2.0 - 1.0;
        let norm_y = (adjusted_y / self.window_size.1 as f64) * 2.0 - 1.0;

        // Black difficulty buttons: y in [-0.35, -0.23] (NDC), screen y in [0.23, 0.35]
        if norm_y < 0.23 || norm_y > 0.35 {
            return None;
        }

        let diff_button_width = 0.18_f64;
        let diff_start_x = -0.45_f64;
        let diff_spacing = 0.05_f64;

        for i in 0..4 {
            let left = diff_start_x + (i as f64) * (diff_button_width + diff_spacing);
            let right = left + diff_button_width;
            if norm_x >= left && norm_x <= right {
                return Some(i);
            }
        }

        None
    }

    fn is_coord_in_start_button(&self, coords: PhysicalPosition<f64>) -> bool {
        #[cfg(target_arch = "wasm32")]
        let (adjusted_x, adjusted_y) = {
            let scale_factor = self.window.scale_factor();
            (coords.x / scale_factor, coords.y / scale_factor)
        };

        #[cfg(not(target_arch = "wasm32"))]
        let (adjusted_x, adjusted_y) = (coords.x, coords.y);

        let norm_x = (adjusted_x / self.window_size.0 as f64) * 2.0 - 1.0;
        let norm_y = (adjusted_y / self.window_size.1 as f64) * 2.0 - 1.0;

        // Start button: x in [-0.3, 0.3], y in [-0.75, -0.6] (NDC), screen y in [0.6, 0.75]
        norm_x >= -0.3 && norm_x <= 0.3 && norm_y >= 0.6 && norm_y <= 0.75
    }

    // ===========================
    // New Menu System Implementation
    // ===========================

    fn window_size(&self) -> (u32, u32) {
        self.window_size
    }

    fn draw_menu_state(&mut self, state: &MenuState) {
        let output = self.surface.get_current_texture().unwrap();
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Menu State Render Encoder"),
        });

        // Clear background
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Menu State Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: layout::colors::BACKGROUND[0] as f64,
                            g: layout::colors::BACKGROUND[1] as f64,
                            b: layout::colors::BACKGROUND[2] as f64,
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

            match state {
                MenuState::ModeSelection => {
                    self.draw_button_rect(&mut render_pass, &layout::main_menu::PVP, layout::colors::PVP);
                    self.draw_button_rect(&mut render_pass, &layout::main_menu::PVAI, layout::colors::PVAI);
                    self.draw_button_rect(&mut render_pass, &layout::main_menu::AIVAI, layout::colors::AIVAI);
                }
                MenuState::SideSelection => {
                    self.draw_button_rect(&mut render_pass, &layout::side_selection::WHITE, layout::colors::SIDE_WHITE);
                    self.draw_button_rect(&mut render_pass, &layout::side_selection::BLACK, layout::colors::SIDE_BLACK);
                }
                MenuState::DifficultySelection { .. } => {
                    let buttons = layout::difficulty::single_buttons();
                    for button in &buttons {
                        self.draw_button_rect(&mut render_pass, button, layout::colors::white_ai::NORMAL);
                    }
                }
                MenuState::AIvAISetup(setup) => {
                    // Draw white difficulty buttons
                    let white_buttons = layout::difficulty::white_buttons();
                    for (i, button) in white_buttons.iter().enumerate() {
                        let is_selected = matches!(
                            (i, setup.white_difficulty),
                            (0, Difficulty::Easy) | (1, Difficulty::Medium) |
                            (2, Difficulty::Hard) | (3, Difficulty::Expert)
                        ) && i == difficulty_to_index(setup.white_difficulty);
                        let color = if is_selected {
                            layout::colors::white_ai::SELECTED
                        } else {
                            layout::colors::white_ai::NORMAL
                        };
                        self.draw_button_rect(&mut render_pass, button, color);
                    }

                    // Draw black difficulty buttons
                    let black_buttons = layout::difficulty::black_buttons();
                    for (i, button) in black_buttons.iter().enumerate() {
                        let is_selected = i == difficulty_to_index(setup.black_difficulty);
                        let color = if is_selected {
                            layout::colors::black_ai::SELECTED
                        } else {
                            layout::colors::black_ai::NORMAL
                        };
                        self.draw_button_rect(&mut render_pass, button, color);
                    }

                    // Draw start button
                    self.draw_button_rect(&mut render_pass, &layout::difficulty::START, layout::colors::START);
                }
            }
        }

        // Render text
        let viewport_width = self.window_size.0 as f32;
        let viewport_height = self.window_size.1 as f32;

        self.viewport.update(&self.queue, glyphon::Resolution {
            width: self.window_size.0,
            height: self.window_size.1,
        });

        // Create text buffers based on state
        let text_areas = match state {
            MenuState::ModeSelection => {
                self.prepare_mode_selection_text(viewport_width, viewport_height)
            }
            MenuState::SideSelection => {
                self.prepare_side_selection_text(viewport_width, viewport_height)
            }
            MenuState::DifficultySelection { user_color } => {
                self.prepare_difficulty_selection_text(viewport_width, viewport_height, *user_color)
            }
            MenuState::AIvAISetup(_) => {
                self.prepare_aivai_setup_text(viewport_width, viewport_height)
            }
        };

        self.text_renderer.prepare(
            &self.device,
            &self.queue,
            &mut self.font_system,
            &mut self.text_atlas,
            &self.viewport,
            text_areas.iter().map(|ta| TextArea {
                buffer: &ta.buffer,
                left: ta.left,
                top: ta.top,
                scale: 1.0,
                bounds: glyphon::TextBounds {
                    left: 0,
                    top: 0,
                    right: viewport_width as i32,
                    bottom: viewport_height as i32,
                },
                default_color: ta.color,
                custom_glyphs: &[],
            }),
            &mut self.swash_cache,
        ).unwrap();

        // Render text pass
        {
            let mut text_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Menu Text Render Pass"),
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
}

/// Helper function to convert Difficulty to button index
fn difficulty_to_index(diff: Difficulty) -> usize {
    match diff {
        Difficulty::Easy => 0,
        Difficulty::Medium => 1,
        Difficulty::Hard => 2,
        Difficulty::Expert => 3,
    }
}
