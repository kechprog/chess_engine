use crate::game_repr::{MoveType, Move, Color};

use super::super::{
    piece::{Piece, Type},
    position::Position
};

impl Position {
    // TODO: add castaling
    pub fn king_moves(&self, idx: usize) -> Vec<Move> {
        [
            idx as i64 + 1,
            idx as i64 - 1,
            idx as i64 + 8,
            idx as i64 - 8,
            idx as i64 + 7,
            idx as i64 - 7,
            idx as i64 + 9,
            idx as i64 - 9,
        ]
        .iter()
        .filter(|&&x| x < 64 && x >= 0)
        .filter(|&&idx| !(
            self.position[idx as usize].color == self.position[idx as usize].color
            && self.position[idx as usize].piece_type != Type::None
        ))
        .map(move |&x| Move::new(idx as u8, x as u8, MoveType::Normal))
        .collect()
    }

    fn can_castle_long(&self, idx: usize) -> bool {
        let color = self.position[idx].color;
        let king_moved = match color {
            Color::White => !self.castling_cond[2],
            Color::Black => !self.castling_cond[5]
        };
        if king_moved { return false }

        return true
    }
}
