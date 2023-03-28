use crate::game::{
    helpers::piece::{Piece, Type},
    position::position::Position,
};

impl Position {
    pub fn knight_moves(&self, idx: usize) -> Vec<u8> {
        let mut moves = vec![0; 0];
        let idx_x = idx % 8;
        let idx_y = idx / 8;

        // nee
        if idx_x < 6 && idx_y < 7 {
            let m_idx = idx as u8 + 8 + 2;
            if !(self.board[m_idx as usize].piece_type != Type::None 
            && self.board[m_idx as usize].color == self.board[idx].color) {
                moves.push(m_idx);
            }
        }

        // nne
        if idx_x < 7 && idx_y < 6 {
            let m_idx = idx as u8 + 16 + 1;
            if !(self.board[m_idx as usize].piece_type != Type::None 
            && self.board[m_idx as usize].color == self.board[idx].color) {
                moves.push(m_idx);
            }
        }

        // nnw
        if idx_x > 0 && idx_y < 6 {
            let m_idx = idx as u8 + 16 - 1;
            if !(self.board[m_idx as usize].piece_type != Type::None 
            && self.board[m_idx as usize].color == self.board[idx].color) {
                moves.push(m_idx);
            }
        }

        // nww
        if idx_x > 1 && idx_y < 7 {
            let m_idx = idx as u8 + 8 - 2;
            if !(self.board[m_idx as usize].piece_type != Type::None 
            && self.board[m_idx as usize].color == self.board[idx].color) {
                moves.push(m_idx);
            }
        }

        // sww
        if idx_x > 1 && idx_y > 0 {
            let m_idx = idx as u8 - 8 - 2;
            if !(self.board[m_idx as usize].piece_type != Type::None 
            && self.board[m_idx as usize].color == self.board[idx].color) {
                moves.push(m_idx);
            }
        }

        // ssw
        if idx_x > 0 && idx_y > 1 {
            let m_idx = idx as u8 - 16 - 1;
            if !(self.board[m_idx as usize].piece_type != Type::None 
            && self.board[m_idx as usize].color == self.board[idx].color) {
                moves.push(m_idx);
            }
        }

        // sse
        if idx_x < 7 && idx_y > 1 {
            let m_idx = idx as u8 - 16 + 1;
            if !(self.board[m_idx as usize].piece_type != Type::None 
            && self.board[m_idx as usize].color == self.board[idx].color) {
                moves.push(m_idx);
            }
        }

        // see
        if idx_x < 6 && idx_y > 0 {
            let m_idx = idx as u8 - 8 + 2;
            if !(self.board[m_idx as usize].piece_type != Type::None 
            && self.board[m_idx as usize].color == self.board[idx].color) {
                moves.push(m_idx);
            }
        }

        moves
    }
}
