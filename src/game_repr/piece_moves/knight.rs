use crate::game_repr::{Move, MoveType};
use super::super::{
    piece::Type,
    position::Position
};

impl Position {
    pub fn knight_moves(&self, idx: usize) -> Vec<Move> {
        let mut moves = Vec::with_capacity(8);  // Knights have max 8 moves
        let idx_x = idx % 8;
        let idx_y = idx / 8;

        // nee
        if idx_x < 6 && idx_y < 7 {
            let m_idx = idx as u8 + 8 + 2;
            if !(self.position[m_idx as usize].piece_type != Type::None 
            && self.position[m_idx as usize].color == self.position[idx].color) {
                moves.push(Move::new(idx as u8, m_idx as u8, MoveType::Normal));
            }
        }

        // nne
        if idx_x < 7 && idx_y < 6 {
            let m_idx = idx as u8 + 16 + 1;
            if !(self.position[m_idx as usize].piece_type != Type::None 
            && self.position[m_idx as usize].color == self.position[idx].color) {
                moves.push(Move::new(idx as u8, m_idx as u8, MoveType::Normal));
            }
        }

        // nnw
        if idx_x > 0 && idx_y < 6 {
            let m_idx = idx as u8 + 16 - 1;
            if !(self.position[m_idx as usize].piece_type != Type::None 
            && self.position[m_idx as usize].color == self.position[idx].color) {
                moves.push(Move::new(idx as u8, m_idx as u8, MoveType::Normal));
            }
        }

        // nww
        if idx_x > 1 && idx_y < 7 {
            let m_idx = idx as u8 + 8 - 2;
            if !(self.position[m_idx as usize].piece_type != Type::None 
            && self.position[m_idx as usize].color == self.position[idx].color) {
                moves.push(Move::new(idx as u8, m_idx as u8, MoveType::Normal));
            }
        }

        // sww
        if idx_x > 1 && idx_y > 0 {
            let m_idx = idx as u8 - 8 - 2;
            if !(self.position[m_idx as usize].piece_type != Type::None 
            && self.position[m_idx as usize].color == self.position[idx].color) {
                moves.push(Move::new(idx as u8, m_idx as u8, MoveType::Normal));
            }
        }

        // ssw
        if idx_x > 0 && idx_y > 1 {
            let m_idx = idx as u8 - 16 - 1;
            if !(self.position[m_idx as usize].piece_type != Type::None 
            && self.position[m_idx as usize].color == self.position[idx].color) {
                moves.push(Move::new(idx as u8, m_idx as u8, MoveType::Normal));
            }
        }

        // sse
        if idx_x < 7 && idx_y > 1 {
            let m_idx = idx as u8 - 16 + 1;
            if !(self.position[m_idx as usize].piece_type != Type::None 
            && self.position[m_idx as usize].color == self.position[idx].color) {
                moves.push(Move::new(idx as u8, m_idx as u8, MoveType::Normal));
            }
        }

        // see
        if idx_x < 6 && idx_y > 0 {
            let m_idx = idx as u8 - 8 + 2;
            if !(self.position[m_idx as usize].piece_type != Type::None 
            && self.position[m_idx as usize].color == self.position[idx].color) {
                moves.push(Move::new(idx as u8, m_idx as u8, MoveType::Normal));
            }
        }

        moves
    }
}
