use glium::{
    Display,
    Texture2d,
    Surface,
    implement_vertex,
    uniform,
    program, VertexBuffer, IndexBuffer, index::PrimitiveType,
};
use super::piece::Piece;

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
        [$r as f32 / 255f32, $g as f32 / 255f32, $b as f32 / 255f32, $a as f32 / 255f32]
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



pub struct Board {
    pieces: [[Piece; 8]; 8],
    display: Display,
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
    pub fn from_fen(fen_str: &str, display: Display) -> Self {
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

        Self { pieces: board,
        display }
    }

    pub fn draw_position(&mut self) {

        let mut target = self.display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);
        for i in 0..=7 {
            for j in 0..=7 {
                draw_piece(Piece::BKing, (i, j), &self.display, &mut target);
            }
        }
        target.finish().unwrap();
    }
}