use smallvec::SmallVec;
use crate::game_repr::{Color, Move, MoveType, Position, Type};
use crate::game_repr::bitboards::{pop_lsb, tables::PAWN_ATTACKS};

impl Position {
    /// Generate pawn moves into a provided buffer
    pub fn pawn_moves_into(&self, idx: usize, moves: &mut SmallVec<[Move; 64]>) {
        let piece = self.position[idx];
        let start_len = moves.len();  // Track where we started adding moves

        // Single square forward move
        let single_forward = if piece.color == Color::White {
            idx + 8
        } else {
            idx.wrapping_sub(8)
        };

        if single_forward < 64 && self.position[single_forward].piece_type == Type::None {
            moves.push(Move::new(idx as u8, single_forward as u8, MoveType::Normal));

            // Double square forward move from starting rank
            let is_starting_rank = (idx / 8 == 1 && piece.color == Color::White)
                || (idx / 8 == 6 && piece.color == Color::Black);

            if is_starting_rank {
                let double_forward = if piece.color == Color::White {
                    idx + 16
                } else {
                    idx.wrapping_sub(16)
                };

                if double_forward < 64 && self.position[double_forward].piece_type == Type::None {
                    moves.push(Move::new(idx as u8, double_forward as u8, MoveType::Normal));
                }
            }
        }

        // Diagonal captures using PAWN_ATTACKS table
        let color_idx = match piece.color {
            Color::White => 0,
            Color::Black => 1,
        };

        let mut attacks = PAWN_ATTACKS[color_idx][idx];
        let enemy_pieces = self.bitboards.occupied_by_color(piece.color.opposite());

        // Only keep attacks that hit enemy pieces
        attacks &= enemy_pieces;

        // Generate capture moves
        while attacks != 0 {
            let target_sq = pop_lsb(&mut attacks);
            moves.push(Move::new(idx as u8, target_sq as u8, MoveType::Normal));
        }

        // En passant - only check if there was a previous move
        if !self.prev_moves.is_empty() {
            let prev_move = self.prev_moves.last().unwrap();
            let prev_to = prev_move._to();
            let prev_from = prev_move._from();

            // Check if last move was a two-square pawn advance
            if self.position[prev_to].piece_type == Type::Pawn
                && self.position[prev_to].color != piece.color
                && (prev_from as i32 - prev_to as i32).abs() == 16
            {
                // Check if enemy pawn is adjacent on the same rank
                let same_rank = idx / 8 == prev_to / 8;
                let adjacent = (idx as i32 - prev_to as i32).abs() == 1;

                if same_rank && adjacent {
                    // En passant target square is behind the enemy pawn
                    let ep_target = if piece.color == Color::White {
                        prev_to + 8
                    } else {
                        prev_to.wrapping_sub(8)
                    };

                    if ep_target < 64 {
                        moves.push(Move::new(idx as u8, ep_target as u8, MoveType::EnPassant));
                    }
                }
            }
        }

        // Handle promotions: replace moves that reach the back rank with 4 promotion variants
        // We need to check all moves we just added (from start_len to end)
        let end_len = moves.len();
        let mut promotion_moves: SmallVec<[Move; 16]> = SmallVec::new();

        for i in (start_len..end_len).rev() {
            let m = moves[i];
            let is_promotion = (m._to() / 8 == 7 && piece.color == Color::White)
                            || (m._to() / 8 == 0 && piece.color == Color::Black);

            if is_promotion {
                // Remove the original move
                moves.swap_remove(i);
                // Add 4 promotion variants to our temporary vector
                promotion_moves.push(Move::new(m._from() as u8, m._to() as u8, MoveType::PromotionQueen));
                promotion_moves.push(Move::new(m._from() as u8, m._to() as u8, MoveType::PromotionRook));
                promotion_moves.push(Move::new(m._from() as u8, m._to() as u8, MoveType::PromotionBishop));
                promotion_moves.push(Move::new(m._from() as u8, m._to() as u8, MoveType::PromotionKnight));
            }
        }

        // Add all promotion moves to the buffer
        moves.extend(promotion_moves);
    }

    /// Generate pawn moves (backward-compatible wrapper)
    pub fn pawn_moves(&self, idx: usize) -> SmallVec<[Move; 64]> {
        let mut moves = SmallVec::with_capacity(16);  // Max: 4 moves × 4 promotion types
        self.pawn_moves_into(idx, &mut moves);
        moves
    }
}
