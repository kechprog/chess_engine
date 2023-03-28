use crate::game::{
    helpers::piece::{Piece, Type},
    position::position::Position,
};

impl Position {
    pub fn rook_moves(&self, idx: usize, include_friendly: bool) -> Vec<u8> {
        // n
        let mut moves = vec![];
        let mut p_idx = idx;
        while p_idx / 8 != 7 {
            p_idx += 8;
            match self.board[p_idx] {
                Piece { piece_type, .. } if piece_type == Type::None => moves.push(p_idx as u8),
                p if p.color() != self.board[idx].color() => {
                    moves.push(p_idx as u8);
                    break;
                }
                p if include_friendly && p.color() == self.board[idx].color() => {
                    moves.push(p_idx as u8);
                    break;
                }
                _ => break,
            }
        }

        // s
        p_idx = idx;
        while p_idx / 8 != 0 {
            p_idx -= 8;
            match self.board[p_idx] {
                Piece { piece_type, .. } if piece_type == Type::None => moves.push(p_idx as u8),
                p if p.color() != self.board[idx].color() => {
                    moves.push(p_idx as u8);
                    break;
                }
                p if include_friendly && p.color() == self.board[idx].color() => {
                    moves.push(p_idx as u8);
                    break;
                }
                _ => break,
            }
        }

        // e
        p_idx = idx;
        while p_idx % 8 != 7 {
            p_idx += 1;
            match self.board[p_idx] {
                Piece {
                    piece_type: Type::None,
                    ..
                } => moves.push(p_idx as u8),
                p if p.color() != self.board[idx].color() => {
                    moves.push(p_idx as u8);
                    break;
                }
                p if include_friendly && p.color() == self.board[idx].color() => {
                    moves.push(p_idx as u8);
                    break;
                }
                _ => break,
            }
        }

        // w
        p_idx = idx;
        while p_idx % 8 != 0 {
            p_idx -= 1;
            match self.board[p_idx] {
                Piece {
                    piece_type: Type::None,
                    ..
                } => moves.push(p_idx as u8),
                p if p.color() != self.board[idx].color() => {
                    moves.push(p_idx as u8);
                    break;
                }
                p if include_friendly && p.color() == self.board[idx].color() => {
                    moves.push(p_idx as u8);
                    break;
                }
                _ => break,
            }
        }

        moves
    }
}
