use crate::game_repr::{MoveType, Move};

use super::super::{
    piece::{Piece, Type},
    position::Position
};

impl Position {
    pub fn rook_moves(&self, idx: usize, include_friendly: bool) -> Vec<Move> {
        // n
        let mut moves = vec![];
        let mut p_idx = idx;
        while p_idx / 8 != 7 {
            p_idx += 8;
            match self.position[p_idx] {
                Piece { piece_type, .. } if piece_type == Type::None => moves.push(Move::new(idx as u8, p_idx as u8, MoveType::Normal)),
                p if p.color() != self.position[idx].color() => {
                    moves.push(Move::new(idx as u8, p_idx as u8, MoveType::Normal));
                    break;
                }
                p if include_friendly && p.color() == self.position[idx].color() => {
                    moves.push(Move::new(idx as u8, p_idx as u8, MoveType::Normal));
                    break;
                }
                _ => break,
            }
        }

        // s
        p_idx = idx;
        while p_idx / 8 != 0 {
            p_idx -= 8;
            match self.position[p_idx] {
                Piece { piece_type, .. } if piece_type == Type::None => moves.push(Move::new(idx as u8, p_idx as u8, MoveType::Normal)),
                p if p.color() != self.position[idx].color() => {
                    moves.push(Move::new(idx as u8, p_idx as u8, MoveType::Normal));
                    break;
                }
                p if include_friendly && p.color() == self.position[idx].color() => {
                    moves.push(Move::new(idx as u8, p_idx as u8, MoveType::Normal));
                    break;
                }
                _ => break,
            }
        }

        // e
        p_idx = idx;
        while p_idx % 8 != 7 {
            p_idx += 1;
            match self.position[p_idx] {
                Piece {
                    piece_type: Type::None,
                    ..
                } => moves.push(Move::new(idx as u8, p_idx as u8, MoveType::Normal)),
                p if p.color() != self.position[idx].color() => {
                    moves.push(Move::new(idx as u8, p_idx as u8, MoveType::Normal));
                    break;
                }
                p if include_friendly && p.color() == self.position[idx].color() => {
                    moves.push(Move::new(idx as u8, p_idx as u8, MoveType::Normal));
                    break;
                }
                _ => break,
            }
        }

        // w
        p_idx = idx;
        while p_idx % 8 != 0 {
            p_idx -= 1;
            match self.position[p_idx] {
                Piece {
                    piece_type: Type::None,
                    ..
                } => moves.push(Move::new(idx as u8, p_idx as u8, MoveType::Normal)),
                p if p.color() != self.position[idx].color() => {
                    moves.push(Move::new(idx as u8, p_idx as u8, MoveType::Normal));
                    break;
                }
                p if include_friendly && p.color() == self.position[idx].color() => {
                    moves.push(Move::new(idx as u8, p_idx as u8, MoveType::Normal));
                    break;
                }
                _ => break,
            }
        }

        moves
    }
}
