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

    // True - long, False - short
    fn can_castle(self, kcolor: Color, direction: bool) -> bool {
        todo!("It is hard!")
    }
}
