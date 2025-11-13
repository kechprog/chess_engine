use smallvec::SmallVec;
use crate::game_repr::{MoveType, Move, Color};
use crate::game_repr::bitboards::{pop_lsb, tables::KING_ATTACKS};

use super::super::{
    piece::Type,
    position::Position
};

impl Position {
    /// Generate king moves into a provided buffer
    pub fn king_moves_into(&self, idx: usize, moves: &mut SmallVec<[Move; 64]>) {
        let king_color = self.position[idx].color;

        // Get all squares the king can attack using the precomputed table
        let mut attacks = KING_ATTACKS[idx];

        // Get friendly pieces to avoid capturing them
        let friendly_pieces = self.bitboards.occupied_by_color(king_color);

        // Remove friendly pieces from attacks
        attacks &= !friendly_pieces;

        // Generate moves for each target square
        while attacks != 0 {
            let target_sq = pop_lsb(&mut attacks);
            moves.push(Move::new(idx as u8, target_sq as u8, MoveType::Normal));
        }

        // Add castling moves
        // Check if king is in its starting position
        let is_king_on_starting_square = match king_color {
            Color::White => idx == 4,
            Color::Black => idx == 60,
        };

        if !is_king_on_starting_square {
            return;
        }

        // Check if king is currently in check (can't castle out of check)
        if self.is_in_check(king_color) {
            return;
        }

        // Get castling condition indices for this color
        let (kingside_rook_cond, queenside_rook_cond, king_cond) = match king_color {
            Color::White => (0, 1, 2),
            Color::Black => (3, 4, 5),
        };

        let opponent_color = king_color.opposite();

        // Try kingside castling
        if self.castling_cond[kingside_rook_cond] && self.castling_cond[king_cond] {
            let (f_square, g_square, h_square) = match king_color {
                Color::White => (5, 6, 7),   // f1, g1, h1
                Color::Black => (61, 62, 63), // f8, g8, h8
            };

            // Check that squares between king and rook are empty
            let squares_empty = self.position[f_square].piece_type == Type::None &&
                               self.position[g_square].piece_type == Type::None;

            // Check that king doesn't pass through or land in check
            let king_safe = !self.is_square_attacked(f_square, opponent_color) &&
                           !self.is_square_attacked(g_square, opponent_color);

            // Verify rook is on starting square
            let rook_present = self.position[h_square].piece_type == Type::Rook &&
                              self.position[h_square].color == king_color;

            if squares_empty && king_safe && rook_present {
                moves.push(Move::new(idx as u8, g_square as u8, MoveType::Castling));
            }
        }

        // Try queenside castling
        if self.castling_cond[queenside_rook_cond] && self.castling_cond[king_cond] {
            let (a_square, b_square, c_square, d_square) = match king_color {
                Color::White => (0, 1, 2, 3),     // a1, b1, c1, d1
                Color::Black => (56, 57, 58, 59), // a8, b8, c8, d8
            };

            // Check that squares between king and rook are empty
            let squares_empty = self.position[b_square].piece_type == Type::None &&
                               self.position[c_square].piece_type == Type::None &&
                               self.position[d_square].piece_type == Type::None;

            // Check that king doesn't pass through or land in check
            // Note: b_square doesn't need to be checked as king doesn't pass through it
            let king_safe = !self.is_square_attacked(c_square, opponent_color) &&
                           !self.is_square_attacked(d_square, opponent_color);

            // Verify rook is on starting square
            let rook_present = self.position[a_square].piece_type == Type::Rook &&
                              self.position[a_square].color == king_color;

            if squares_empty && king_safe && rook_present {
                moves.push(Move::new(idx as u8, c_square as u8, MoveType::Castling));
            }
        }
    }

    /// Generate king moves (backward-compatible wrapper)
    pub fn king_moves(&self, idx: usize) -> SmallVec<[Move; 64]> {
        let mut moves = SmallVec::with_capacity(10);  // Kings have max 8 normal moves + 2 castling
        self.king_moves_into(idx, &mut moves);
        moves
    }
}
