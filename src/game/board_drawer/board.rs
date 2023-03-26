use super::{dot_drawer::DotDrawer, tile_drawer::TileDrawer};
use crate::game::helpers::{
    game_state::{GameState, Pov},
    piece::Piece,
};
use glium::{glutin::dpi::PhysicalPosition, Display, Surface};
use std::rc::Rc;

pub struct BoardDrawer {
    display: Rc<Display>,
    board_dimensions: (f32, f32),
    tile_drawer: super::tile_drawer::TileDrawer,
    dot_drawer: super::dot_drawer::DotDrawer,
}

impl BoardDrawer {
    pub fn new(display: Display) -> Self {
        let rc_disp = Rc::new(display);
        Self {
            display: rc_disp.clone(),
            tile_drawer: TileDrawer::new(rc_disp.clone()),
            dot_drawer: DotDrawer::new(rc_disp),
            board_dimensions: (0.0, 0.0),
        }
    }

    fn update_board_dimensions(&mut self) {
        let w_to_h = self.display.gl_window().window().inner_size().width as f32
            / self.display.gl_window().window().inner_size().height as f32;

        self.board_dimensions = if w_to_h > 1.0 {
            (2.0 / w_to_h, 2.0)
        } else {
            (2.0, 2.0 * w_to_h)
        }
    }

    pub fn draw_position(&mut self, state: &GameState) {
        let mut target = self.display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);

        self.update_board_dimensions();

        // draw tiles
        match state.pov {
            Pov::White => state
                .position
                .board
                .iter()
                .enumerate()
                .for_each(|(idx, p)| {
                    self.tile_drawer.draw(
                        idx as usize,
                        *p,
                        self.board_dimensions,
                        false,
                        &mut target,
                    )
                }),

            Pov::Black => state
                .position
                .board
                .iter()
                .enumerate()
                .for_each(|(idx, p)| {
                    self.tile_drawer.draw(
                        63 - idx as usize,
                        *p,
                        self.board_dimensions,
                        false,
                        &mut target,
                    )
                }),
        }

        // draw selected piece
        if let Some(selected_tile) = state.selected_tile {
            if state.position.board[selected_tile] != Piece::None {
                self.tile_drawer.draw(
                    selected_tile,
                    state.position.board[selected_tile],
                    self.board_dimensions,
                    true,
                    &mut target,
                );
            }
        }

        // TODO: FIX AFTER NORMAL INDEXING
        // draw dots(legal moves)
        if let Some(selected_tile) = state.selected_tile {
            state
                .position
                .get_legal_moves(selected_tile)
                .iter()
                .for_each(|&idx| {
                    self.dot_drawer
                        .dot_at(idx as usize, self.board_dimensions, &mut target)
                });
        }

        target.finish().unwrap();
    }

    // returns none if the click was outside the board
    pub fn coord_to_tile(&mut self, coords: PhysicalPosition<f64>) -> Option<usize> {
        self.update_board_dimensions();
        let (x, y) = (
            (coords.x / self.display.gl_window().window().inner_size().width as f64) * 2.0,
            (coords.y / self.display.gl_window().window().inner_size().height as f64) * 2.0,
        );

        let tile_w = self.board_dimensions.0 / 8.0;
        let tile_h = self.board_dimensions.1 / 8.0;

        let tile_x = (x / tile_w as f64).floor() as usize;
        let tile_y = (y / tile_h as f64).floor() as usize;

        if tile_x > 7 || tile_y > 7 {
            return None;
        }

        let tile_from_bottom = 7 - tile_y;

        Some(tile_from_bottom * 8 + tile_x)
    }
}
