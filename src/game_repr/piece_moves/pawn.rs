use crate::game_repr::{Color, Move, MoveType, Position, Type};

impl Position {
    // TODO: add on passant

    pub fn pawn_moves(&self, idx: usize) -> Vec<Move> {
        // (l,f,r) - left diagonal, forward, right diagonal
        // For Black: left diagonal = -9 (row-1, col-1), right diagonal = -7 (row-1, col+1)
        // For White: left diagonal = 7 (row+1, col-1), right diagonal = 9 (row+1, col+1)
        let offset: (i64, i64, i64) = match self.position[idx].color {
            Color::White => if idx % 8 != 0 && idx % 8 != 7{
                    (7,8,9)  // Can capture both diagonals
                } else if idx % 8 == 0 {  // Left edge (a-file)
                    (0,8,9)  // Can't go left, can capture right
                } else {  // Right edge (h-file)
                    (7,8,0)  // Can capture left, can't go right
                }
            Color::Black => if idx % 8 != 0 && idx % 8 != 7{
                    (-9,-8,-7)  // Can capture both diagonals
                } else if idx % 8 == 0 {  // Left edge (a-file)
                    (0,-8,-7)  // Can't go left, can capture right
                } else {  // Right edge (h-file)
                    (-9,-8,0)  // Can capture left, can't go right
                }
        };
        let piece = self.position[idx];
        let idx = idx as i64;
        let mut moves = Vec::with_capacity(16);  // Max: 4 moves × 4 promotion types

        // Pawns on starting rank can move 1 or 2 squares forward
        if (idx / 8 == 1 && piece.color == Color::White)
            || (idx / 8 == 6 && piece.color == Color::Black)
        {
            for m in 1..=2 {
                let m_idx = (idx + offset.1*m) as usize;
                if self.position[m_idx].piece_type == Type::None {
                    moves.push(Move::new(idx as u8, m_idx as u8, MoveType::Normal))
                } else {
                    break;
                }
            }
        } else {
            // Pawns not on starting rank can only move 1 square forward
            if self.position[(idx + offset.1) as usize].piece_type == Type::None {
                moves.push(Move::new(idx as u8, (idx + offset.1) as u8, MoveType::Normal))
            }
        }

        // on passant - only check if there was a previous move
        if !self.prev_moves.is_empty() {
            let prev_to = self.prev_moves
                .last().unwrap()._to();
            let prev_move = self.prev_moves
                .last().unwrap();
            if self.position[prev_to].color != piece.color
            && self.position[prev_to].piece_type == Type::Pawn
            && ((prev_move._from() as i64 - prev_move._to() as i64).abs() == 16 )
            {
                // Check if enemy pawn is to the left (prev_to + 1 == idx means enemy at lower column)
                if prev_to + 1 == idx as usize && prev_to / 8 == idx as usize / 8 {
                    moves.push(Move::new(idx as u8, match piece.color {
                        Color::White => (idx + offset.0) as u8,  // Capture left-forward
                        Color::Black => (idx + offset.0) as u8   // Capture left-forward (toward enemy)
                    }, MoveType::EnPassant))
                }
                // Check if enemy pawn is to the right (prev_to - 1 == idx means enemy at higher column)
                else if prev_to > 0 && prev_to - 1 == idx as usize && prev_to / 8 == idx as usize / 8 {
                    moves.push(Move::new(idx as u8, match piece.color {
                        Color::White => (idx + offset.2) as u8,  // Capture right-forward
                        Color::Black => (idx + offset.2) as u8   // Capture right-forward (toward enemy)
                    }, MoveType::EnPassant))
                }
            }
        }

        // Diagonal captures - only include non-zero offsets (edge squares have 0 to prevent wrapping)
        let mut capture_moves = Vec::with_capacity(2);  // Max 2 diagonal captures
        if offset.0 != 0 {
            let target = idx + offset.0;
            if target >= 0 && target < 64 {
                let target_piece = self.position[target as usize];
                if target_piece.color != piece.color && target_piece.piece_type != Type::None {
                    capture_moves.push(Move::new(idx as u8, target as u8, MoveType::Normal));
                }
            }
        }
        if offset.2 != 0 {
            let target = idx + offset.2;
            if target >= 0 && target < 64 {
                let target_piece = self.position[target as usize];
                if target_piece.color != piece.color && target_piece.piece_type != Type::None {
                    capture_moves.push(Move::new(idx as u8, target as u8, MoveType::Normal));
                }
            }
        }
        moves.append(&mut capture_moves);

        // Generate promotion moves: for each move reaching the back rank, create 4 moves (Q, R, B, N)
        let mut result = Vec::new();
        for m in moves {
            let is_promotion = (m._to() / 8 == 7 && piece.color == Color::White)
                            || (m._to() / 8 == 0 && piece.color == Color::Black);

            if is_promotion {
                // Generate 4 promotion moves (Queen, Rook, Bishop, Knight)
                result.push(Move::new(m._from() as u8, m._to() as u8, MoveType::PromotionQueen));
                result.push(Move::new(m._from() as u8, m._to() as u8, MoveType::PromotionRook));
                result.push(Move::new(m._from() as u8, m._to() as u8, MoveType::PromotionBishop));
                result.push(Move::new(m._from() as u8, m._to() as u8, MoveType::PromotionKnight));
            } else {
                result.push(m);
            }
        }

        result
    }
}
