use crate::game_repr::{Move, MoveType};
use crate::game_repr::bitboards::{pop_lsb, tables::KNIGHT_ATTACKS};
use super::super::position::Position;

impl Position {
    /// Generate knight moves into a provided buffer
    pub fn knight_moves_into(&self, idx: usize, moves: &mut Vec<Move>) {
        let moving_piece = self.position[idx];

        // Get all squares the knight can attack
        let mut attacks = KNIGHT_ATTACKS[idx];

        // Get friendly pieces to avoid capturing them
        let friendly_pieces = self.bitboards.occupied_by_color(moving_piece.color);

        // Remove friendly pieces from attacks
        attacks &= !friendly_pieces;

        // Generate moves for each target square
        while attacks != 0 {
            let target_sq = pop_lsb(&mut attacks);
            moves.push(Move::new(idx as u8, target_sq as u8, MoveType::Normal));
        }
    }

    /// Generate knight moves (backward-compatible wrapper)
    pub fn knight_moves(&self, idx: usize) -> Vec<Move> {
        let mut moves = Vec::with_capacity(8);  // Knights have max 8 moves
        self.knight_moves_into(idx, &mut moves);
        moves
    }
}
