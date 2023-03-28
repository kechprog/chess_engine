use crate::game::{position::position::Position, helpers::piece::Type};

impl Position {
    // FIXME
    pub fn king_moves(&self, idx: usize) -> Vec<u8> {
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
            self.board[idx as usize].color == self.board[idx as usize].color
            && self.board[idx as usize].piece_type != Type::None
        ))
        .map(|&x| x as u8)
        .collect()
    }
}
