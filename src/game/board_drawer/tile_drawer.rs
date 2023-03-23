use std::{collections::HashMap, rc::Rc};

use glium::{
    glutin::dpi::Pixel, implement_vertex, index::PrimitiveType, program, uniform, Display, Frame,
    IndexBuffer, Program, Surface, Texture2d, VertexBuffer,
};
use image::Rgba;

use crate::game::helpers::piece::Piece;

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
    textures: HashMap<Piece, Texture2d>,
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


const TILE_PADDING_FRACTION: f32 = 0.15;
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
            textures: HashMap::new(),
        }
    }

    pub fn draw(&mut self, pos: usize, piece: Piece, board_dimensions: (f32, f32), target: &mut Frame) {     
        let pos = (pos % 8, pos / 8); // 0 - x, 1 - y (from left to right, from bottom to top)
        let bg_color = if (pos.0 + pos.1) % 2 == 0 {
            WHITE_SQUARE_COLOR
        } else {
            BLACK_SQUARE_COLOR
        };

        let tile_w = board_dimensions.0 / 8.0;
        let tile_h = board_dimensions.1 / 8.0 ;
        let piece_h = tile_h * (1.0 - 2.0 * TILE_PADDING_FRACTION);
        let piece_w = tile_w * (1.0 - 2.0 * TILE_PADDING_FRACTION);

        let tile_top_left = (
            -1f32 + pos.0 as f32 * tile_w,
            1f32 - pos.1 as f32 * tile_h,
        );

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
                    pos: [tile_top_left.0 + tile_w, tile_top_left.1],
                    col: bg_color,
                    text_cord: [0.0, 0.0],
                },
                Vertex {
                    // bottom left
                    pos: [tile_top_left.0, tile_top_left.1 - tile_h],
                    col: bg_color,
                    text_cord: [0.0, 0.0],
                },
                Vertex {
                    // bottom right
                    pos: [tile_top_left.0 + tile_w, tile_top_left.1 - tile_h],
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
        if piece == Piece::None {
            return;
        }

        let texture = self.textures
            .entry(piece)
            .or_insert_with(|| piece.get_texture(&*self.display)) // make it laaaazy
            as &Texture2d; // a dirty hack that compiler forces on me

        let piece_top_left = (
            tile_top_left.0 + tile_w * TILE_PADDING_FRACTION,
            tile_top_left.1 - tile_h * TILE_PADDING_FRACTION,
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
                    pos: [piece_top_left.0 + piece_w, piece_top_left.1],
                    col: [1.0, 1.0, 1.0, 1.0],
                    text_cord: [1.0, 0.0],
                },
                Vertex {
                    // bottom left
                    pos: [piece_top_left.0, piece_top_left.1 - piece_h],
                    col: [1.0, 1.0, 1.0, 1.0],
                    text_cord: [0.0, 1.0],
                },
                Vertex {
                    // bottom right
                    pos: [piece_top_left.0 + piece_w, piece_top_left.1 - piece_h],
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
                &uniform! {use_color: false, text: texture},
                &Default::default(),
            )
            .unwrap();
    }
}