// AI Agent - Negamax with Alpha-Beta Pruning
//
// This module implements a classical chess AI using the Negamax algorithm
// with alpha-beta pruning, iterative deepening, and transposition tables.
//
// Key features:
// - Deterministic (same position always gives same move)
// - Uses minimax-based tree search with alpha-beta pruning for efficiency
// - Transposition tables to cache evaluated positions
// - Quiescence search to avoid horizon effect
// - Move ordering for improved pruning

mod transposition_table;
mod negamax;
mod quiescence;
mod search;
mod negamax_player;
mod evaluation;
mod move_ordering;
mod piece_square_tables;

pub use negamax_player::{NegamaxPlayer, Difficulty};

// Re-export useful types
pub use search::SearchResult;
pub use transposition_table::TranspositionTable;
