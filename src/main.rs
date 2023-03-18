#![allow(unused)]

/*
 * TODO:
 * MVP of the board
 * Heavy refactoring
 * function -> beautifull object
 * board is a square not a rectangle, fix it
 */

/*===============================================================*/
/*==================== GAME RELATED STAFF =======================*/
/*===============================================================*/
#[derive(Debug, Clone, Copy, PartialEq)]
enum Piece {
    None = 0,

    /*--- WHITE ---*/
    WPawn = 1,
    WKnight = 2,
    WBishop = 3,
    WRook = 4,
    WQueen = 5,
    WKing = 6,

    /*--- BLACK ---*/
    BPawn = 7,
    BKnight = 8,
    BBishop = 9,
    BRook = 10,
    BQueen = 11,
    BKing = 12,
}
impl Piece {
    fn from_char(c: char) -> Self {
        match c {
            'p' => Self::BPawn,
            'n' => Self::BKnight,
            'b' => Self::BBishop,
            'r' => Self::BRook,
            'q' => Self::BQueen,
            'k' => Self::BKing,
            'P' => Self::WPawn,
            'N' => Self::WKnight,
            'B' => Self::WBishop,
            'R' => Self::WRook,
            'Q' => Self::WQueen,
            'K' => Self::WKing,
            '1'..='8' => Self::None,
            _ => panic!("Invalid character, unable to transfrom into piece"),
        }
    }

    fn as_char(&self) -> char {
        match self {
            Self::WPawn => 'P',
            Self::WKnight => 'N',
            Self::WBishop => 'B',
            Self::WRook => 'R',
            Self::WQueen => 'Q',
            Self::WKing => 'K',
            Self::BPawn => 'p',
            Self::BKnight => 'n',
            Self::BBishop => 'b',
            Self::BRook => 'r',
            Self::BQueen => 'q',
            Self::BKing => 'k',
            Self::None => '_',
        }
    }
}

struct Board {
    pieces: [[Piece; 8]; 8],
}

impl std::fmt::Debug for Board {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..8 {
            print!("{}|", 8 - y);
            for x in 0..8 {
                print!("{}|", self.pieces[y][x].as_char());
            }
            println!();
        }
        Ok(())
    }
}

impl Board {
    fn from_fen(fen_str: &str) -> Self {
        let mut x = 0;
        let mut y = 0;
        let mut board = [[Piece::None; 8]; 8];

        for c in fen_str.chars() {
            if c == '/' {
                y += 1;
                x = 0;
                continue;
            }

            let piece = Piece::from_char(c);
            board[y][x] = piece;

            if piece != Piece::None {
                x += 1;
            } else {
                x += c.to_digit(10).expect("Expected a valid fen str") as usize;
            }
        }

        Self { pieces: board }
    }
}

use glium::glutin;
use glium::glutin::event_loop::ControlFlow;
use glium::implement_vertex;
use glium::index::PrimitiveType;
use glium::program;
use glium::Display;
use glium::IndexBuffer;
use glium::Surface;
use glium::Texture2d;
use glium::VertexBuffer;

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

#[derive(Clone, Copy)]
struct VertexWText {
    position: [f32; 2],
    tex_coords: [f32; 2],
}
implement_vertex!(VertexWText, position, tex_coords);

#[derive(Clone, Copy)]
struct VertexWColor {
    position: [f32; 2],
    color: [f32; 4],
}
implement_vertex!(VertexWColor, position, color);

// pos starts from 0
fn draw_piece(piece: Piece, pos: (u8, u8), display: &Display, target: &mut glium::Frame) {
    const TILE_SIZE: f32 = 2f32 / 8f32;
    const TILE_PADDING: f32 = 0.05;
    const PIECE_SIZE: f32 = TILE_SIZE - TILE_PADDING * 2f32;

    const WHITE_SQUARE_COLOR: [f32; 4] = rgba![202, 211, 245, 255];
    const BLACK_SQUARE_COLOR: [f32; 4] = rgba![24, 25, 38, 255];

    /*--------- SHADER FOR ONLY COLOR ---------*/
    let shader = program!(display,
    140 => {
        vertex: "
                #version 140
                in vec2 position;
                in vec4 color;
                out vec4 v_color;

                void main() {
                    v_color = color;
                    gl_Position = vec4(position, 0.0, 1.0);
                }",
        fragment: "
                #version 140
                in vec4 v_color;
                out vec4 color;

                void main() {
                    color = v_color;
                }"
    })
    .unwrap();

    let index_buffer = IndexBuffer::new(
        display,
        PrimitiveType::TrianglesList,
        &[0, 1, 2, 2, 1, 3u16],
    )
    .unwrap();

    /*======================== DRAW BACKGROUND =======================================*/
    let top_left: (f32, f32) = (
        -1f32 + pos.0 as f32 * TILE_SIZE,
        1f32 - pos.1 as f32 * TILE_SIZE,
    );
    let background = if (pos.0 + pos.1) % 2 == 0 {
        WHITE_SQUARE_COLOR
    } else {
        BLACK_SQUARE_COLOR
    };

    let vertex_buffer = VertexBuffer::new(
        display,
        &[
            VertexWColor {
                // top left
                position: [top_left.0, top_left.1],
                color: background,
            },
            VertexWColor {
                // top right
                position: [top_left.0 + TILE_SIZE, top_left.1],
                color: background,
            },
            VertexWColor {
                // bottom left
                position: [top_left.0, top_left.1 - TILE_SIZE],
                color: background,
            },
            VertexWColor {
                // bottom right
                position: [top_left.0 + TILE_SIZE, top_left.1 - TILE_SIZE],
                color: background,
            },
        ],
    )
    .unwrap();

    target
        .draw(
            &vertex_buffer,
            &index_buffer,
            &shader,
            &glium::uniforms::EmptyUniforms,
            &Default::default(),
        )
        .unwrap();

    /*======================== DRAW A PIECE =======================================*/
    let top_left: (f32, f32) = (
        -1f32 + pos.0 as f32 * TILE_SIZE + TILE_PADDING,
        1f32 - pos.1 as f32 * TILE_SIZE - TILE_PADDING,
    );

    let vb = VertexBuffer::new(
        display,
        &[
            VertexWText {
                // top left
                position: [top_left.0, top_left.1],
                tex_coords: [0.0, 0.0],
            },
            VertexWText {
                // top right
                position: [top_left.0 + PIECE_SIZE, top_left.1],
                tex_coords: [1.0, 0.0],
            },
            VertexWText {
                // bottom left
                position: [top_left.0, top_left.1 - PIECE_SIZE],
                tex_coords: [0.0, 1.0],
            },
            VertexWText {
                // bottom right
                position: [top_left.0 + PIECE_SIZE, top_left.1 - PIECE_SIZE],
                tex_coords: [1.0, 1.0],
            },
        ],
    )
    .unwrap();

    // TODO: load textures depending on the piece
    let texture = load_ogl_texture("src/assets/b_pawn_png_1024px.png", display).unwrap();

    let shader = program!(display,
    140 => {
        vertex: "
                #version 140

                in vec2 position;
                in vec2 tex_coords;

                out vec2 v_tex_coords;

                void main() {
                    v_tex_coords = tex_coords;
                    gl_Position = vec4(position, 0.0, 1.0);
                }",
        fragment: "
                #version 140

                in vec2 v_tex_coords;

                out vec4 color;

                uniform sampler2D tex;

                void main() {
                    color = texture(tex, v_tex_coords);
                }"
    })
    .unwrap();

    target
        .draw(
            &vb,
            &index_buffer,
            &shader,
            &glium::uniform! { tex: &texture },
            &Default::default(),
        )
        .unwrap();
}

fn draw_start_pos(display: &Display) {
    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 1.0, 1.0);
    for i in 0..=7 {
        for j in 0..=7 {
            draw_piece(Piece::BKing, (i, j), &display, &mut target);
        }
    }
    target.finish().unwrap();
}

fn main() {
    let ev = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new().with_title("textures");
    let cb = glutin::ContextBuilder::new().with_vsync(true);
    let display = glium::Display::new(wb, cb, &ev).unwrap();

    ev.run(move |event, _, control_flow| {
        *control_flow = match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => glutin::event_loop::ControlFlow::Exit,
                glutin::event::WindowEvent::Resized(_) => {
                    draw_start_pos(&display);
                    ControlFlow::Poll
                }
                _ => ControlFlow::Poll,
            },
            glutin::event::Event::RedrawRequested(_) => {
                draw_start_pos(&display);
                ControlFlow::Poll
            }
            _ => ControlFlow::Poll,
        }
    });
}
