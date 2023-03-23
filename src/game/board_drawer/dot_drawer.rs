use glium::{implement_vertex, program, uniform, Display, Frame, Surface, VertexBuffer};
use std::rc::Rc;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}
implement_vertex!(Vertex, position, tex_coords);

pub struct DotDrawer {
    display: Rc<Display>,
    texture: glium::texture::Texture2d,
    shader: glium::Program,
    index_buffer: glium::IndexBuffer<u16>,
}

impl DotDrawer {
    pub fn new(display: Rc<Display>) -> Self {
        let index_buffer = glium::IndexBuffer::new(
            display.as_ref(),
            glium::index::PrimitiveType::TrianglesList,
            &[0u16, 1, 2, 1, 2, 3],
        )
        .unwrap();

        let shader = program!(display.as_ref(),
        140 => {
            vertex: "
                #version 140
                in vec2 position;
                in vec2 tex_coords;
                out vec2 v_tex_coords;
                void main() {
                    v_tex_coords = tex_coords;
                    gl_Position = vec4(position, 0.0, 1.0);
                }
            ",

            fragment: "
                #version 140
                in vec2 v_tex_coords;
                out vec4 color;
                uniform sampler2D tex;
                void main() {
                    color = texture(tex, v_tex_coords);
                }
            ",
        })
        .unwrap();

        let img = image::open("src/assets/circle.png").unwrap().to_rgba8();
        let img_dimensions = img.dimensions();
        let img = glium::texture::RawImage2d::from_raw_rgba(img.into_raw(), img_dimensions);
        let texture = glium::texture::Texture2d::new(display.as_ref(), img).unwrap();

        Self {
            display,
            texture,
            shader,
            index_buffer,
        }
    }

    const DOT_PADDING_FRACTION: f32 = 0.1;
    pub fn dot_at(&self, idx: usize, board_dimensions: (f32, f32), target: &mut Frame) {
        let pos = (idx % 8, idx / 8);

        let tile_w = board_dimensions.0 / 8.0;
        let tile_h = board_dimensions.1 / 8.0;
        let dot_h = tile_h * (1.0 - 2.0 * Self::DOT_PADDING_FRACTION);
        let dot_w = tile_w * (1.0 - 2.0 * Self::DOT_PADDING_FRACTION);

        let top_left = (
            -1f32 + pos.0 as f32 * tile_w + Self::DOT_PADDING_FRACTION * tile_w,  
            1f32 - pos.1 as f32 * tile_h - Self::DOT_PADDING_FRACTION * tile_h,
        );

        let vertex_buffer = VertexBuffer::new(
            self.display.as_ref(),
            &[
                Vertex {
                    // top left
                    position: [top_left.0, top_left.1],
                    tex_coords: [0.0, 0.0],
                },
                Vertex {
                    // top right
                    position: [top_left.0 + dot_w, top_left.1],
                    tex_coords: [1.0, 0.0],
                },
                Vertex {
                    // bottom left
                    position: [top_left.0, top_left.1 - dot_h],
                    tex_coords: [0.0, 1.0],
                },
                Vertex {
                    position: [top_left.0 + dot_w, top_left.1 - dot_h],
                    tex_coords: [1.0, 1.0],
                },
            ],
        )
        .unwrap();

        let uniforms = uniform! {
            tex: &self.texture,
        };

        target.draw(
            &vertex_buffer,
            &self.index_buffer,
            &self.shader,
            &uniforms,
            &Default::default(),
        );
    }
}
