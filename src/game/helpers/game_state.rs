use super::piece::Piece;
use super::helper_fn::position_from_fen;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Pov {
    White,
    Black,
}

pub struct GameState {
    pub pov: Pov,
    pub position: [Piece; 64],
    pub selected_tile: Option<usize>
}

impl GameState {
    pub fn from_fen(fen_str: &str, pov: Pov) -> GameState {
        GameState {
            pov: pov,
            position: position_from_fen(fen_str),
            selected_tile: None
        }
    }

    pub fn set_pov(&mut self, pov: Pov) {
        self.pov = pov;
    }
}