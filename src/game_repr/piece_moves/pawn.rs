use crate::game_repr::{Move, MoveType};

use super::super::{
    piece::{Color, Piece, Type},
    position::Position,
};

impl Position {
    // TODO: add on passant
    // TODO: add promotion

    pub fn pawn_moves(&self, idx: usize) -> Vec<Move> {
        // (l,f,r)
        let offset: (i64, i64, i64) = match self.position[idx].color {
            Color::White => (7, 8, 9),
            Color::Black => (-7, -8, -9),
        };
        let piece = self.position[idx];
        let idx = idx as i64;
        let mut moves = vec![];

        if (idx / 8 == 1 && piece.color == Color::White)
            || (idx / 8 == 6 && piece.color == Color::Black)
        {
            moves.append(
                &mut [idx + offset.1, idx + 2 * offset.1]
                    .iter()
                    .filter(|&&i| self.position[i as usize].piece_type == Type::None)
                    .map(|&i| Move::new(idx as u8, i as u8, MoveType::Normal))
                    .collect(),
            );
        }

        //  TODO: promotion a place to add it
        // check for piece in a way
        if moves.is_empty() {

            if self.position[(idx + offset.1) as usize].piece_type == Type::None
            && ((idx + offset.1) / 8 == 0
            ||  (idx + offset.1) / 8 == 7) {
                moves.push(Move::new(idx as u8, (idx + offset.1) as u8, MoveType::Promotion));
            } else {
                moves.append(
                    &mut [idx + offset.1]
                        .iter()
                        .filter(|&&i| self.position[i as usize].piece_type == Type::None)
                        .map(|&i| Move::new(idx as u8, i as u8, MoveType::Normal))
                        .collect()
                );
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

        moves
    }
}
