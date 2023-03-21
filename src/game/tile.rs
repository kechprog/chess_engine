use std::rc::Rc;

use glium::{
    implement_vertex, index::PrimitiveType, program, uniform, Display, Frame, IndexBuffer, Program,
    Surface, Texture2d, VertexBuffer,
};

use super::piece::Piece;

#[derive(Clone, Copy)]
struct Vertex {
    pos: [f32; 2],
    col: [f32; 4],
    text_cord: [f32; 2],
}
implement_vertex!(Vertex, pos, col, text_cord);

pub struct TileDrawer {
    shader: Program,
    display: Rc<Display>,
    idx_buffer: IndexBuffer<u16>,
}

macro_rules! rgba {
    ($r:expr, $g:expr, $b:expr, $a:expr) => {
        [
            $r as f32 / 255f32,
            $g as f32 / 255f32,
            $b as f32 / 255f32,
            $a as f32 / 255f32,
        ]
    };
}

fn load_ogl_texture(
    path: &str,
    display: &Display,
) -> Result<Texture2d, Box<dyn std::error::Error>> {
    let image = image::open(path)?;
    let image = image.to_rgba8();
    let image_dimensions = image.dimensions();
    Ok(glium::texture::Texture2d::new(
        display,
        glium::texture::RawImage2d::from_raw_rgba(image.into_raw(), image_dimensions),
    )?)
}

const TILE_SIZE: f32 = 2f32 / 8f32;
const TILE_PADDING: f32 = 0.05;
const PIECE_SIZE: f32 = TILE_SIZE - TILE_PADDING * 2f32;
const WHITE_SQUARE_COLOR: [f32; 4] = rgba![202, 211, 245, 255];
const BLACK_SQUARE_COLOR: [f32; 4] = rgba![24, 25, 38, 255];

impl TileDrawer {
    pub fn new(display: Rc<Display>) -> Self {
        let shader = program!(&*display,
        140 => {
            vertex: "
                            #version 140
                            in vec2 pos;
                            in vec4 col;
                            in vec2 text_cord;

                            out vec4 v_color;
                            out vec2 v_text_cord;
                            void main() {
                                v_color = col;
                                v_text_cord = text_cord;
                                gl_Position = vec4(pos, 0.0, 1.0);
                            }",
            fragment: "
                            #version 140
                            uniform bool use_color;
                            uniform sampler2D text;
                            in vec4 v_color;
                            in vec2 v_text_cord;
                            out vec4 color;
            
                            void main() {
                                if (use_color) {
                                    color = v_color;
                                } else {
                                    color = texture(text, v_text_cord);
                                }
                            }",
        })
        .unwrap();

        let idx_buffer = IndexBuffer::new(
            &*display,
            PrimitiveType::TrianglesList,
            &[0, 1, 2, 2, 1, 3u16],
        )
        .unwrap();

        Self {
            shader,
            display,
            idx_buffer,
        }
    }

    pub fn draw(&mut self, pos: (u8, u8), piece: &Piece, target: &mut Frame) {
        let bg_color = if (pos.0 + pos.1) % 2 == 0 {
            WHITE_SQUARE_COLOR
        } else {
            BLACK_SQUARE_COLOR
        };

        let tile_top_left = (
            -1f32 + pos.0 as f32 * TILE_SIZE,
            1f32 - pos.1 as f32 * TILE_SIZE,
        );

        // TODO: get right texture
        // let texture = piece.get_texture(self.display);
        let texture = load_ogl_texture("src/assets/b_pawn_png_1024px.png", &*self.display).unwrap();

        //----------------------------------- draw bg
        let vertex_buffer = VertexBuffer::new(
            &*self.display,
            &[
                Vertex {
                    // top left
                    pos: [tile_top_left.0, tile_top_left.1],
                    col: bg_color,
                    text_cord: [0.0, 0.0],
                },
                Vertex {
                    // top right
                    pos: [tile_top_left.0 + TILE_SIZE, tile_top_left.1],
                    col: bg_color,
                    text_cord: [0.0, 0.0],
                },
                Vertex {
                    // bottom left
                    pos: [tile_top_left.0, tile_top_left.1 - TILE_SIZE],
                    col: bg_color,
                    text_cord: [0.0, 0.0],
                },
                Vertex {
                    // bottom right
                    pos: [tile_top_left.0 + TILE_SIZE, tile_top_left.1 - TILE_SIZE],
                    col: bg_color,
                    text_cord: [0.0, 0.0],
                },
            ],
        )
        .unwrap();

        target
            .draw(
                &vertex_buffer,
                &self.idx_buffer,
                &self.shader,
                &uniform! {use_color: true},
                &Default::default(),
            )
            .unwrap();

        //----------------------------------- draw piece
        let piece_top_left = (
            tile_top_left.0 + TILE_PADDING,
            tile_top_left.1 - TILE_PADDING,
        );



        let vertex_buffer = VertexBuffer::new(
            &*self.display,
            &[
                Vertex {
                    // top left
                    pos: [piece_top_left.0, piece_top_left.1],
                    col: [1.0, 1.0, 1.0, 1.0],
                    text_cord: [0.0, 0.0],
                },
                Vertex {
                    // top right
                    pos: [piece_top_left.0 + PIECE_SIZE, piece_top_left.1],
                    col: [1.0, 1.0, 1.0, 1.0],
                    text_cord: [1.0, 0.0],
                },
                Vertex {
                    // bottom left
                    pos: [piece_top_left.0, piece_top_left.1 - PIECE_SIZE],
                    col: [1.0, 1.0, 1.0, 1.0],
                    text_cord: [0.0, 1.0],
                },
                Vertex {
                    // bottom right
                    pos: [piece_top_left.0 + PIECE_SIZE, piece_top_left.1 - PIECE_SIZE],
                    col: [1.0, 1.0, 1.0, 1.0],
                    text_cord: [1.0, 1.0],
                },
            ],
        )
        .unwrap();

        target
            .draw(
                &vertex_buffer,
                &self.idx_buffer,
                &self.shader,
                &uniform! {use_color: false, text: &texture},
                &Default::default(),
            )
            .unwrap();
    }
}
