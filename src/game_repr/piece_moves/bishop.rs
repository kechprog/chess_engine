use crate::game_repr::{Move, MoveType};

use super::super::{
    piece::{Piece, Type},
    position::Position
};

impl Position {
    pub fn bishop_moves(&self, idx: usize, include_friendly: bool) -> Vec<Move> {
        // ne
        let mut moves: Vec<Move> = vec![];
        let mut p_idx = idx;
        while p_idx % 8 != 7 && p_idx / 8 != 7 {
            p_idx += 9;
            match self.position[p_idx] {
                Piece { piece_type, color } if piece_type == Type::None => moves.push(Move::new(idx as u8, p_idx as u8, MoveType::Normal)),
                p if p.color != self.position[idx].color => {
                    moves.push(Move::new(idx as u8, p_idx as u8, MoveType::Normal));
                    break;
                }
                // p if include_friendly && p.color == self.position[idx].color => {
                //     // moves.push(p_idx as u8);
                //     break;
                // }
                _ => break,
            }
        }

        // nw
        p_idx = idx;
        while p_idx % 8 != 0 && p_idx / 8 != 7 {
            p_idx += 7;
            match self.position[p_idx] {
                Piece { piece_type, .. } if piece_type == Type::None => moves.push(Move::new(idx as u8, p_idx as u8, MoveType::Normal)),
                p if p.color != self.position[idx].color => {
                    moves.push(Move::new(idx as u8, p_idx as u8, MoveType::Normal));
                    break;
                }
                // p if include_friendly && p.color() == self.position[idx].color() => {
                //     moves.push(p_idx as u8);
                //     break;
                // }
                _ => break,
            }
        }

        // se
        p_idx = idx;
        while p_idx % 8 != 7 && p_idx / 8 != 0 {
            p_idx -= 7;
            match self.position[p_idx] {
                Piece{piece_type, ..} if piece_type == Type::None => moves.push(Move::new(idx as u8, p_idx as u8, MoveType::Normal)),
                p if p.color != self.position[idx].color => {
                    moves.push(Move::new(idx as u8, p_idx as u8, MoveType::Normal));
                    break;
                }
                // p if include_friendly && p.color == self.position[idx].color => {
                //     moves.push(p_idx as u8);
                //     break;
                // }
                _ => break,
            }
        }

        // sw
        p_idx = idx;
        while p_idx % 8 != 0 && p_idx / 8 != 0 {
            p_idx -= 9;
            match self.position[p_idx] {
                Piece {piece_type, ..} if piece_type == Type::None => moves.push(Move::new(idx as u8, p_idx as u8, MoveType::Normal)),
                p if p.color() != self.position[idx].color() => {
                    moves.push(Move::new(idx as u8, p_idx as u8, MoveType::Normal));
                    break;
                }
                // p if include_friendly && p.color() == self.position[idx].color() => {
                //     moves.push(p_idx as u8);
                //     break;
                // }
                _ => break,
            }
        }

        moves
    }
}