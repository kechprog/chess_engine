use crate::game::{position::position::Position, helpers::piece::{Type, Piece}};

impl Position {
    pub fn bishop_moves(&self, idx: usize, include_friendly: bool) -> Vec<u8> {
        // ne
        let mut moves = vec![0; 0];
        let mut p_idx = idx;
        while p_idx % 8 != 7 && p_idx / 8 != 7 {
            p_idx += 9;
            match self.board[p_idx] {
                Piece { piece_type, color } if piece_type == Type::None => moves.push(p_idx as u8),
                p if p.color != self.board[idx].color => {
                    moves.push(p_idx as u8);
                    break;
                }
                p if include_friendly && p.color == self.board[idx].color => {
                    moves.push(p_idx as u8);
                    break;
                }
                _ => break,
            }
        }

        // nw
        p_idx = idx;
        while p_idx % 8 != 0 && p_idx / 8 != 7 {
            p_idx += 7;
            match self.board[p_idx] {
                Piece { piece_type, .. } if piece_type == Type::None => moves.push(p_idx as u8),
                p if p.color != self.board[idx].color => {
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

        // se
        p_idx = idx;
        while p_idx % 8 != 7 && p_idx / 8 != 0 {
            p_idx -= 7;
            match self.board[p_idx] {
                Piece{piece_type, ..} if piece_type == Type::None => moves.push(p_idx as u8),
                p if p.color != self.board[idx].color => {
                    moves.push(p_idx as u8);
                    break;
                }
                p if include_friendly && p.color == self.board[idx].color => {
                    moves.push(p_idx as u8);
                    break;
                }
                _ => break,
            }
        }

        // sw
        p_idx = idx;
        while p_idx % 8 != 0 && p_idx / 8 != 0 {
            p_idx -= 9;
            match self.board[p_idx] {
                Piece {piece_type, ..} if piece_type == Type::None => moves.push(p_idx as u8),
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