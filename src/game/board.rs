use std::rc::Rc;

use crate::game::tile_drawer::TileDrawer;
use glium::{
    implement_vertex, index::PrimitiveType, program, uniform, Display, IndexBuffer, Program,
    Surface, Texture2d, VertexBuffer,
};

use super::piece::Piece;


pub struct Board {
    board: [Piece; 64],

    /* STUFF FOR DRAWING */
    display: Rc<Display>,
    tile_drawer: TileDrawer,
}

impl std::fmt::Debug for Board {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        for (idx, p) in self.board.iter().enumerate() {
            print!("{}|", p.as_char());
            if idx % 8 == 7 {
                println!();
            }
        }
        Ok(())
    }
}


impl Board {
    pub fn from_fen(fen_str: &str, display: Rc<Display>) -> Self {
        let mut idx = 0;
        let mut board = [Piece::None; 64];

        for c in fen_str.chars().filter(|x| *x != '/') {
            if c.is_digit(10) {
                idx += c.to_digit(10).unwrap() as usize ;
                continue;
            }
            
            board[idx] = Piece::from_char(c);
            idx += 1;
        }

        let tile_drawer = TileDrawer::new(display.clone());
        Self {
            board, // 0 is a1
            display: display,
            tile_drawer,
        }
    }

    // TODO: This is a temporary function to test drawing
    pub fn draw_position(&mut self) {
        let mut target = self.display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);
        for (idx, p) in self.board.iter().enumerate() {
            let pos = (idx as u8 % 8, idx as u8 / 8);
            self.tile_drawer.draw(idx as u8, p, &mut target);
        }
        target.finish().unwrap();
    }
}
