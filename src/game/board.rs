use std::rc::Rc;

use crate::game::tile::TileDrawer;
use glium::{
    implement_vertex, index::PrimitiveType, program, uniform, Display, IndexBuffer, Program,
    Surface, Texture2d, VertexBuffer,
};

use super::piece::Piece;


pub struct Board {
    board: [[Piece; 8]; 8],

    /* STUFF FOR DRAWING */
    display: Rc<Display>,
    tile_drawer: TileDrawer,
}

impl std::fmt::Debug for Board {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..8 {
            print!("{}|", 8 - y);
            for x in 0..8 {
                print!("{}|", self.board[y][x].as_char());
            }
            println!();
        }
        Ok(())
    }
}


impl Board {
    pub fn from_fen(fen_str: &str, display: Rc<Display>) -> Self {
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

        let tile_drawer = TileDrawer::new(display.clone());
        Self {
            board,
            display: display,
            tile_drawer,
        }
    }

    fn draw_tile(&mut self, piece: Piece, pos: (u8, u8), target: &mut glium::Frame) {
        self.tile_drawer.draw(pos, &piece, target);
    }

    pub fn draw_position(&mut self) {
        let mut target = self.display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);
        for i in 0..=7 {
            for j in 0..=7 {
                self.draw_tile(Piece::BKing, (i, j), &mut target);
            }
        }
        target.finish().unwrap();
    }
}
