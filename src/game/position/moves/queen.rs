use crate::game::position::position::Position;

impl Position {
    pub fn queen_moves(&self, idx: usize) -> Vec<u8> {
        self.bishop_moves(idx, false)
            .into_iter()
            .chain(self.rook_moves(idx, false).into_iter())
            .collect()
    }
}