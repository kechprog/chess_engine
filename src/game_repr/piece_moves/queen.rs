use crate::game_repr::Move;

use super::super::{
    piece::{Piece, Type},
    position::Position
};

impl Position {
    pub fn queen_moves(&self, idx: usize) -> Vec<Move> {
        self.bishop_moves(idx, false)
            .into_iter()
            .chain(self.rook_moves(idx, false).into_iter())
            .collect()
    }
}