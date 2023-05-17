use glium::glutin::platform::unix::x11::ffi::PictFormat;

use crate::game_repr::{Move, MoveType};

use super::super::{
    piece::{Color, Piece, Type},
    position::Position,
};

impl Position {
    // TODO: add on passant

    pub fn pawn_moves(&self, idx: usize) -> Vec<Move> {
        // (l,f,r)
        let offset: (i64, i64, i64) = match self.position[idx].color {
            Color::White => if idx % 8 != 0 && idx % 8 != 7{
                    (7,8,9)
                } else if idx % 8 == 0 {
                    (0,8,9)
                } else {
                    (7,8,0)
                }   
            Color::Black => if idx % 8 != 0 && idx % 8 != 7{
                    (-7,-8,-9)
                } else if idx % 8 == 0 {
                    (0,-8,-9)
                } else {
                    (-7,-8,0)
                }
        };
        let piece = self.position[idx];
        let idx = idx as i64;
        let mut moves = vec![];

        if (idx / 8 == 1 && piece.color == Color::White)
            || (idx / 8 == 6 && piece.color == Color::Black)
        {
            for m in (1..=2){
                let m_idx = (idx + offset.1*m) as usize;
                if self.position[m_idx].piece_type == Type::None {
                    moves.push(Move::new(idx as u8, m_idx as u8, MoveType::Normal))
                } else {
                    break;
                }
            }

            // quick fix for on passant check if it is the first move
            if self.prev_moves.is_empty() {
                return moves;
            }
        }

        if self.position[(idx + offset.1) as usize].piece_type == Type::None {
            moves.push(Move::new(idx as u8, (idx + offset.1) as u8, MoveType::Normal))
        }

        // on passant
        let prev_to = self.prev_moves
            .last().unwrap()._to();
        if self.position[prev_to].color != piece.color
        && self.position[prev_to].piece_type == Type::Pawn
        {
            if prev_to + 1 == idx as usize {
                moves.push(Move::new(idx as u8, match piece.color {
                    Color::White => (idx + offset.0) as u8,
                    Color::Black => (idx + offset.2) as u8
                }, MoveType::EnPassant))
            } else if prev_to - 1 == idx as usize {
                moves.push(Move::new(idx as u8, match piece.color {
                    Color::White => (idx + offset.2) as u8,
                    Color::Black => (idx + offset.0) as u8
                }, MoveType::EnPassant))
            }
        }

        moves.append(
            &mut [idx + offset.0, idx + offset.2]
                .iter()
                .filter(|&&i| {
                    self.position[i as usize].color != piece.color
                        && self.position[i as usize].piece_type != Type::None
                })
                .map(|&i| Move::new(idx as u8, i as u8, MoveType::Normal))
                .collect(),
        );

        moves.iter()
            .map(|&m| if ((m._to() / 8) == 7 && piece.color == Color::White) 
                      || ((m._to() / 8) == 0 && piece.color == Color::Black)
            {
                Move::new(m._from() as u8, m._to() as u8, MoveType::Promotion)
            } else {
                m
            }).collect()
    }
}
