use crate::game::{helpers::piece::Type, position::position::Position};

impl Position {
    // TODO: add on passant
    // TODO: add promotion
    // TODO: PLACK PAWNS cause panic
    pub fn pawn_moves(&self, idx: usize) -> Vec<u8> {
        if idx / 8 == 1 || idx / 8 == 6 {
            return [idx + 8, idx + 16]
                .iter()
                .filter(|&&i| self.board[i].piece_type == Type::None)
                .map(|&i| i as u8)
                .collect();
        }

        let mut moves = vec![];
        if self.board[idx + 8].piece_type == Type::None {
            moves.push(idx + 8);
        }
        if self.board[idx + 9].piece_type != Type::None 
        && self.board[idx + 9].color != self.board[idx].color {
            moves.push(idx + 9);
        }
        if self.board[idx + 7].piece_type != Type::None
        && self.board[idx + 7].color != self.board[idx].color {
            moves.push(idx + 7);
        }

        moves.iter().map(|&i| i as u8).collect()
    }
}
