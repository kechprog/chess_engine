#![allow(unused)]

/*
 * TODO:
 * MVP of the board
 * Heavy refactoring
 * function -> beautifull object
 */

// use glium::{
//     glutin::{event::WindowEvent, event_loop::ControlFlow},
//     Surface,
// };
// use std::fs::File;
//
// use glium::{
//     glutin::{self, platform::unix::WindowBuilderExtUnix},
//     implement_vertex, program, uniform,
// };
// use image::ImageFormat;
//
// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let ev = glutin::event_loop::EventLoop::new();
//     let wb = glutin::window::WindowBuilder::new()
//         .with_title("textures")
//         .with_wayland_csd_theme(glutin::window::Theme::Dark);
//     let cb = glutin::ContextBuilder::new().with_vsync(true);
//     let display = glium::Display::new(wb, cb, &ev)?;
//
//     /*===== TEXTURE =====*/
//     let opengl_texture = {
//         // TODO figure out how to make it include bytes at compile time
//         let image = image::open("src/assets/wall.jpeg")?;
//         let image = image.to_rgba8();
//         let image_dimensions = image.dimensions();
//
//         glium::texture::Texture2d::new(
//             &display,
//             glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions),
//         )?
//     };
//
//     let vertex_buffer = {
//         #[derive(Copy, Clone)]
//         struct Vertex {
//             position: [f32; 2],
//             tex_coords: [f32; 2],
//         };
//         implement_vertex!(Vertex, position, tex_coords);
//
//         glium::VertexBuffer::new(
//             &display,
//             &[
//                 Vertex {
//                     position: [-0.5, -0.5],
//                     tex_coords: [0.0, 0.0],
//                 },
//                 Vertex {
//                     position: [0.5, -0.5],
//                     tex_coords: [1.0, 0.0],
//                 },
//                 Vertex {
//                     position: [0.5, 0.5],
//                     tex_coords: [1.0, 1.0],
//                 },
//                 Vertex {
//                     position: [-0.5, 0.5],
//                     tex_coords: [0.0, 1.0],
//                 },
//             ],
//         )?
//     };
//
//     let index_buffer = glium::IndexBuffer::new(
//         &display,
//         glium::index::PrimitiveType::TrianglesList,
//         &[0u16, 1, 2, 0, 2, 3],
//     )?;
//
//     let program = program!(&display,
//     140 => {
//          vertex: "
//                 #version 140
//                 in vec2 position;
//                 in vec2 tex_coords;
//                 out vec2 v_tex_coords;
//                 void main() {
//                     gl_Position =vec4(position, 0.0, 1.0);
//                     v_tex_coords = tex_coords;
//                 }
//             ",
//
//              fragment: "
//                 #version 140
//                 uniform sampler2D tex;
//                 in vec2 v_tex_coords;
//                 out vec4 f_color;
//                 void main() {
//                     f_color = texture(tex, v_tex_coords);
//                 }
//             "
//      })?;
//
//     let draw = move || {
//         let uniform = uniform! {
//             tex: &opengl_texture
//         };
//
//         let mut target = display.draw();
//         target.clear_color(0.0, 0.0, 0.0, 1.0);
//         target.draw(
//             &vertex_buffer,
//             &index_buffer,
//             &program,
//             &uniform,
//             &Default::default(),
//         );
//         target.finish().expect("Could not draw wtf");
//     };
//
//     draw();
//
//     ev.run(move |e, _, cf| {
//         *cf = ControlFlow::Poll;
//         draw();
//     });
//
//     Ok(())
// }

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
//
// struct Board {
//     pieces: [[Piece; 8]; 8],
// }
//
// impl std::fmt::Debug for Board {
//     fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         for y in 0..8 {
//             print!("{}|", 8 - y);
//             for x in 0..8 {
//                 print!("{}|", self.pieces[y][x].as_char());
//             }
//             println!();
//         }
//         Ok(())
//     }
// }
//
// impl Board {
//     fn from_fen(fen_str: &str) -> Self {
//         let mut x = 0;
//         let mut y = 0;
//         let mut board = [[Piece::None; 8]; 8];
//
//         for c in fen_str.chars() {
//             if c == '/' {
//                 y += 1;
//                 x = 0;
//                 continue;
//             }
//
//             let piece = Piece::from_char(c);
//             board[y][x] = piece;
//
//             if piece != Piece::None {
//                 x += 1;
//             } else {
//                 x += c.to_digit(10).expect("Expected a valid fen str") as usize;
//             }
//         }
//
//         Self { pieces: board }
//     }
// }
//
// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let fen_str = "rnb1k1nr/ppp2ppp/8/3pp3/1b1PP2q/2P2NP1/PP3PBP/RNBQK2R";
//     let board = Board::from_fen(fen_str);
//     println!("{:?}", board);
//     Ok(())
// }

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

#[derive(Clone, Copy)]
struct VertexWText {
    position: [f32; 2],
    tex_coords: [f32; 2],
}
implement_vertex!(VertexWText, position, tex_coords);

#[derive(Clone, Copy)]
struct VertexWColor {
    position: [f32; 2],
    color: [f32; 3],
}
implement_vertex!(VertexWColor, position, color);

// pos starts from 0
fn draw_piece(piece: Piece, pos: (u8, u8), display: &Display, target: &mut glium::Frame) {
    const SQUARE_OUTER_SIZE: f32 = 2f32 / 8f32;
    const BORDER_THICNESS: f32 = 0.002;
    const PADDING: f32 = 0.05;
    const SQUARE_INNER_SIZE: f32 = SQUARE_OUTER_SIZE - BORDER_THICNESS * 2f32 - PADDING * 2f32;

    const BORDER_COLOR: [f32; 3] = [239.0/255.0, 222.0/255.0, 205.0/255.0]; // brown
    const WHITE_SQUARE_COLOR: [f32; 3] = [1.0, 1.0, 1.0];
    const BLACK_SQUARE_COLOR: [f32; 3] = [0.0, 0.0, 0.0];

    // let mut target = display.draw();
    // target.clear_color(1.0, 0.0, 0.0, 0.0);
    // target.finish().unwrap();

    /*--------- SHADER FOR ONLY COLOR ---------*/
    let shader = program!(display,
    140 => {
        vertex: "
                #version 140
                in vec2 position;
                in vec3 color;
                out vec3 v_color;

                void main() {
                    v_color = color;
                    gl_Position = vec4(position, 0.0, 1.0);
                }",
        fragment: "
                #version 140
                in vec3 v_color;
                out vec4 color;

                void main() {
                    color = vec4(v_color, 1.0);
                }"
    })
    .unwrap();

    /*======================== DRAW BORDER =======================================*/
    let top_left: (f32, f32) = (
        -1f32 + pos.0 as f32 * SQUARE_OUTER_SIZE,
        1f32 - pos.1 as f32 * SQUARE_OUTER_SIZE,
    );
    let vertex_buffer = VertexBuffer::new(
        display,
        &[
            VertexWColor {
                // top left
                position: [top_left.0, top_left.1],
                color: BORDER_COLOR,
            },
            VertexWColor {
                // top right
                position: [top_left.0 + SQUARE_OUTER_SIZE, top_left.1],
                color: BORDER_COLOR,
            },
            VertexWColor {
                // bottom left
                position: [top_left.0, top_left.1 - SQUARE_OUTER_SIZE],
                color: BORDER_COLOR,
            },
            VertexWColor {
                // bottom right
                position: [
                    top_left.0 + SQUARE_OUTER_SIZE,
                    top_left.1 - SQUARE_OUTER_SIZE,
                ],
                color: BORDER_COLOR,
            },
        ],
    )
    .unwrap();
    let index_buffer = IndexBuffer::new(
        display,
        PrimitiveType::TrianglesList,
        &[0, 1, 2, 2, 1, 3u16],
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
        -1f32 + pos.0 as f32 * SQUARE_OUTER_SIZE + BORDER_THICNESS + PADDING,
        1f32 - pos.1 as f32 * SQUARE_OUTER_SIZE - BORDER_THICNESS - PADDING,
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
                position: [top_left.0 + SQUARE_INNER_SIZE, top_left.1],
                tex_coords: [1.0, 0.0],
            },
            VertexWText {
                // bottom left
                position: [top_left.0, top_left.1 - SQUARE_INNER_SIZE],
                tex_coords: [0.0, 1.0],
            },
            VertexWText {
                // bottom right
                position: [
                    top_left.0 + SQUARE_INNER_SIZE,
                    top_left.1 - SQUARE_INNER_SIZE,
                ],
                tex_coords: [1.0, 1.0],
            },
        ],
    )
    .unwrap();

    let ib = IndexBuffer::new(
        display,
        PrimitiveType::TrianglesList,
        &[0, 1, 2, 2, 1, 3u16],
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
            &ib,
            &shader,
            &glium::uniform! { tex: &texture },
            &Default::default(),
        )
        .unwrap();
}

fn main() {
    let ev = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new().with_title("textures");
    let cb = glutin::ContextBuilder::new().with_vsync(true);
    let display = glium::Display::new(wb, cb, &ev).unwrap();

    ev.run(move |event, _, control_flow| {
        *control_flow = match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => glutin::event_loop::ControlFlow::Poll,
                glutin::event::WindowEvent::Resized(_) => {
                    let mut target = display.draw();
                    target.clear_color(0.0, 0.0, 1.0, 1.0);
                    draw_piece(Piece::BKing, (7, 7), &display, &mut target);
                    draw_piece(Piece::BKing, (0, 0), &display, &mut target);
                    target.finish().unwrap();
                    ControlFlow::Poll
                }
                _ => ControlFlow::Poll,
            },
            glutin::event::Event::RedrawRequested(_) => {
                    let mut target = display.draw();
                    target.clear_color(0.0, 0.0, 1.0, 1.0);
                    draw_piece(Piece::BKing, (7, 7), &display, &mut target);
                    draw_piece(Piece::BKing, (0, 0), &display, &mut target);
                    target.finish().unwrap();
                ControlFlow::Poll
            }
            _ => ControlFlow::Poll,
        }
    });
}
