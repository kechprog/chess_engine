use smallvec::SmallVec;
use crate::game_repr::Move;

use super::super::position::Position;

impl Position {
    /// Generate queen moves into a provided buffer
    pub fn queen_moves_into(&self, idx: usize, moves: &mut SmallVec<[Move; 64]>) {
        self.bishop_moves_into(idx, false, moves);
        self.rook_moves_into(idx, false, moves);
    }

    /// Generate queen moves (backward-compatible wrapper)
    pub fn queen_moves(&self, idx: usize) -> SmallVec<[Move; 64]> {
        let mut moves = SmallVec::with_capacity(27);  // Queens have max 27 moves (13 + 14)
        self.queen_moves_into(idx, &mut moves);
        moves
    }
}