// Move ordering heuristics
//
// Good move ordering is crucial for alpha-beta pruning efficiency.
// By searching the best moves first, we can prune more branches.

use crate::game_repr::{Move, Position, MoveType, Color};
use smallvec::SmallVec;

/// Killer move table - stores moves that caused beta cutoffs at each depth
pub struct KillerMoves {
    // Store 2 killer moves per depth (most recent beta cutoffs)
    table: [[Option<Move>; 2]; 64],
}

impl KillerMoves {
    pub fn new() -> Self {
        Self {
            table: [[None; 2]; 64],
        }
    }

    /// Record a killer move at the given depth
    pub fn store(&mut self, depth: u8, mv: Move) {
        let d = depth as usize;
        if d >= 64 {
            return;
        }

        // Shift existing killer and add new one
        if self.table[d][0] != Some(mv) {
            self.table[d][1] = self.table[d][0];
            self.table[d][0] = Some(mv);
        }
    }

    /// Check if a move is a killer move at the given depth
    pub fn is_killer(&self, depth: u8, mv: Move) -> bool {
        let d = depth as usize;
        if d >= 64 {
            return false;
        }
        self.table[d][0] == Some(mv) || self.table[d][1] == Some(mv)
    }

    /// Clear all killer moves
    pub fn clear(&mut self) {
        self.table = [[None; 2]; 64];
    }
}

/// History heuristic table - records move success rates
pub struct HistoryTable {
    // Indexed by [from_square][to_square]
    table: [[i32; 64]; 64],
}

impl HistoryTable {
    pub fn new() -> Self {
        Self {
            table: [[0; 64]; 64],
        }
    }

    /// Update history for a move (increase score for good moves)
    pub fn update(&mut self, mv: Move, depth: u8) {
        let from = mv._from();
        let to = mv._to();
        // Bonus based on depth (deeper = more valuable)
        self.table[from][to] += (depth as i32) * (depth as i32);
    }

    /// Get history score for a move
    pub fn score(&self, mv: Move) -> i32 {
        self.table[mv._from()][mv._to()]
    }

    /// Clear all history scores
    pub fn clear(&mut self) {
        self.table = [[0; 64]; 64];
    }
}

/// Generate moves in a good order for alpha-beta pruning
///
/// Move ordering priority:
/// 1. Hash move (from transposition table)
/// 2. Captures (MVV-LVA: Most Valuable Victim - Least Valuable Attacker)
/// 3. Killer moves (non-captures that caused beta cutoffs)
/// 4. History heuristic (moves that were good in other positions)
/// 5. Other moves
pub fn generate_ordered_moves(
    pos: &Position,
    hash_move: Option<Move>,
    killers: &KillerMoves,
    history: &HistoryTable,
    depth: u8,
) -> SmallVec<[Move; 64]> {
    let mut moves = pos.all_legal_moves();

    // Sort moves by priority
    moves.sort_by_cached_key(|&mv| {
        // Hash move has highest priority
        if Some(mv) == hash_move {
            return i32::MIN; // Lowest value = highest priority
        }

        let from = mv._from();
        let to = mv._to();
        let moving_piece = pos.position[from];
        let captured_piece = pos.position[to];

        // Captures: order by MVV-LVA (Most Valuable Victim - Least Valuable Attacker)
        if captured_piece.piece_type != crate::game_repr::Type::None {
            let victim_value = piece_value(captured_piece.piece_type);
            let attacker_value = piece_value(moving_piece.piece_type);
            return -(victim_value * 10 - attacker_value); // Negative for higher priority
        }

        // Promotions
        if mv.move_type().is_promotion() {
            return -8000; // High priority
        }

        // Killer moves
        if killers.is_killer(depth, mv) {
            return -5000;
        }

        // History heuristic
        -history.score(mv)
    });

    moves
}

/// Get the value of a piece type for move ordering
fn piece_value(piece_type: crate::game_repr::Type) -> i32 {
    use crate::game_repr::Type;
    match piece_type {
        Type::Pawn => 1,
        Type::Knight => 3,
        Type::Bishop => 3,
        Type::Rook => 5,
        Type::Queen => 9,
        Type::King => 0,
        Type::None => 0,
    }
}
