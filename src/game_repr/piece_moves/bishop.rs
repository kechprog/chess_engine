use crate::game_repr::{Move, MoveType};
use crate::game_repr::bitboards::{pop_lsb, bitscan_forward, tables::*};

use super::super::position::Position;

impl Position {
    /// Generate bishop moves into a provided buffer
    pub fn bishop_moves_into(&self, idx: usize, include_friendly: bool, moves: &mut Vec<Move>) {
        let moving_piece = self.position[idx];
        let occupied = self.bitboards.all_occupied();
        let friendly_pieces = self.bitboards.occupied_by_color(moving_piece.color);

        // Process each diagonal direction (NE, NW, SE, SW)
        for &direction in &[NORTH_EAST, NORTH_WEST, SOUTH_EAST, SOUTH_WEST] {
            let mut ray = RAYS[direction][idx];

            // Find first blocker in this direction
            let blockers = ray & occupied;
            if blockers != 0 {
                let blocker_sq = if direction == NORTH_EAST || direction == NORTH_WEST {
                    bitscan_forward(blockers)  // First blocker going forward (toward higher rank)
                } else {
                    63 - blockers.leading_zeros() as usize  // First blocker going backward (toward lower rank)
                };

                // Mask out squares beyond the blocker
                ray &= !(RAYS[direction][blocker_sq]);

                // Include the blocker square if it's an enemy piece (capture) or if include_friendly
                let blocker_piece = self.position[blocker_sq];
                if blocker_piece.color != moving_piece.color || include_friendly {
                    ray |= 1u64 << blocker_sq;
                }
            }

            // Remove friendly pieces unless include_friendly is true
            if !include_friendly {
                ray &= !friendly_pieces;
            }

            // Generate moves for each target square in this ray
            while ray != 0 {
                let target_sq = pop_lsb(&mut ray);
                moves.push(Move::new(idx as u8, target_sq as u8, MoveType::Normal));
            }
        }
    }

    /// Generate bishop moves (backward-compatible wrapper)
    pub fn bishop_moves(&self, idx: usize, include_friendly: bool) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::with_capacity(13);  // Bishops have max 13 moves (7+6 diagonals)
        self.bishop_moves_into(idx, include_friendly, &mut moves);
        moves
    }
}