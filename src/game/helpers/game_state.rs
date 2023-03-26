use crate::game::position::position::Position;
use super::piece::Piece;


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Pov {
    White,
    Black,
}

pub struct GameState {
    pub pov: Pov,
    pub position: Position,
    pub selected_tile: Option<usize>
}

impl GameState {
    pub fn from_fen(fen_str: &str, pov: Pov) -> GameState {
        GameState {
            pov: pov,
            position: Position::from_fen(fen_str),
            selected_tile: None
        }
    }

    pub fn set_pov(&mut self, pov: Pov) {
        self.pov = pov;
    }
}