#![allow(unused)]

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
        glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions),
    )?)
}

#[derive(Clone, Copy)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}
implement_vertex!(Vertex, position, tex_coords);

// pos starts from 0
fn draw_piece(piece: Piece, pos: (u8, u8), display: &Display) {
    const BOARD_PADDING: f32 = 0.03;
    const SQUARE_OUTER_SIZE: f32 = 1f32 / 8f32;
    const BORDER_THICNESS: f32 = 0.02;
    const PADDING: f32 = 0.05;
    const SQUARE_INNER_SIZE: f32 = SQUARE_OUTER_SIZE - BORDER_THICNESS * 2f32 - PADDING * 2f32;
    
    // FIXME this does not consider the fact
    // that coord system is from -1 to 1
    let top_left: (f32, f32) = (
        pos.0 as f32 * SQUARE_OUTER_SIZE + BOARD_PADDING + BORDER_THICNESS + PADDING,
        pos.1 as f32 * SQUARE_OUTER_SIZE + BOARD_PADDING + BORDER_THICNESS + PADDING,
    );

    // FIXME same problem as above
    let vb = VertexBuffer::new(
        display,
        &[
            Vertex { // top left
                position: [top_left.0, top_left.1],
                tex_coords: [-1.0, 1.0],
            },
            Vertex { // top right
                position: [top_left.0 + SQUARE_INNER_SIZE, top_left.1],
                tex_coords: [1.0, 1.0],
            },
            Vertex { // bottom left
                position: [
                    top_left.0,
                    top_left.1 - SQUARE_INNER_SIZE,
                ],
                tex_coords: [-1.0, -1.0],
            },
            Vertex { // bottom right
                position: [top_left.0, top_left.1 - SQUARE_INNER_SIZE],
                tex_coords: [1.0, -1.0],
            },
        ],
    ).unwrap();

    let ib = IndexBuffer::new(
        display,
        PrimitiveType::TriangleStrip,
        &[1 as u16, 2, 0, 3],
    ).unwrap();

    // TODO load textures depending on the piece
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
    }).unwrap();

    let draw = move || {
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);
        target
            .draw(
                &vb,
                &ib,
                &shader,
                &glium::uniform! { tex: &texture },
                &Default::default(),
            )
            .unwrap();
        target.finish().unwrap();
    };
    
    draw();
}

fn main() {
    let ev = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new().with_title("textures");
    let cb = glutin::ContextBuilder::new().with_vsync(true);
    let display = glium::Display::new(wb, cb, &ev).unwrap();


    ev.run(move |event, _, control_flow| {
        *control_flow = glutin::event_loop::ControlFlow::Wait;
        match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit
                }
                glutin::event::WindowEvent::Resized(_) => draw_piece(Piece::BKing, (0,0), &display),
                _ => (),
            },
            glutin::event::Event::RedrawRequested(_) => draw_piece(Piece::BKing, (0,0), &display),
            _ => (),
        }
    });
}
