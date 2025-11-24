// Iterative Deepening Search Orchestrator
//
// This module implements iterative deepening search for chess move selection.
// It progressively searches deeper depths (1, 2, 3, ...) up to max_depth,
// using results from previous iterations to improve move ordering.

use crate::game_repr::{Position, Move, Color};
use super::transposition_table::TranspositionTable;
use super::negamax::negamax;
use super::move_ordering::{generate_ordered_moves, KillerMoves, HistoryTable};
use std::time::Instant;

/// Result of a search operation
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub best_move: Option<Move>,
    pub score: i32,
    pub depth: u8,
    pub depth_reached: u8, // Alias for depth (for compatibility)
    pub nodes_searched: u64,
    pub time_ms: u64,
    pub principal_variation: Option<Vec<Move>>, // PV line (optional)
}

impl SearchResult {
    /// Create a new search result with no move found
    pub fn new() -> Self {
        Self {
            best_move: None,
            score: 0,
            depth: 0,
            depth_reached: 0,
            nodes_searched: 0,
            time_ms: 0,
            principal_variation: None,
        }
    }
}

/// Perform iterative deepening search to find the best move
///
/// # Arguments
/// * `pos` - Current position to search
/// * `color` - Color to move
/// * `max_depth` - Maximum search depth
/// * `time_limit_ms` - Optional time limit in milliseconds
///
/// # Returns
/// SearchResult containing the best move and search statistics
pub fn iterative_deepening_search(
    pos: &Position,
    color: Color,
    max_depth: u8,
    time_limit_ms: Option<u64>,
) -> SearchResult {
    let start_time = Instant::now();
    let mut best_result = SearchResult::new();
    let mut total_nodes = 0u64;

    // Clone position for searching (we need a mutable copy)
    let mut search_pos = pos.clone();

    // Initialize transposition table (shared across all iterations)
    let mut tt = TranspositionTable::new(); // Default size: 1M entries (~40-80MB)

    // Initialize killer moves and history table
    let mut killers = KillerMoves::new();
    let mut history = HistoryTable::new();

    // Principal variation (best move from previous iteration)
    let mut pv_move: Option<Move> = None;

    // Ensure we search at least depth 1
    let max_depth = max_depth.max(1);

    // Iterative deepening loop
    for depth in 1..=max_depth {
        // Check time limit before starting new depth
        if let Some(time_limit) = time_limit_ms {
            if is_time_up(&start_time, time_limit) {
                // Time is up, return best result from previous iteration
                if depth > 1 {
                    print_search_info(depth - 1, &best_result, &start_time);
                }
                break;
            }
        }

        // Generate and order moves for this position
        let moves = generate_ordered_moves(&search_pos, pv_move, &killers, &history, depth);

        // If no legal moves, position is checkmate or stalemate
        if moves.is_empty() {
            best_result.depth = depth;
            best_result.depth_reached = depth;
            best_result.time_ms = start_time.elapsed().as_millis() as u64;

            if search_pos.is_in_check(color) {
                // Checkmate - very bad score
                best_result.score = -100000;
            } else {
                // Stalemate - draw
                best_result.score = 0;
            }
            return best_result;
        }

        // Search each move
        let mut best_score = i32::MIN + 1;
        let mut best_move_this_depth: Option<Move> = None;
        let mut nodes_this_depth = 0u64;

        let alpha = i32::MIN + 1;
        let beta = i32::MAX;

        for &mov in &moves {
            // Make the move
            let undo = search_pos.make_move_undoable(mov);

            // Search this position
            let (score, _) = negamax(
                &mut search_pos,
                depth - 1,
                -beta,
                -alpha,
                color.opposite(),
                &mut tt,
                &mut killers,
                &mut history,
            );
            let score = -score;
            nodes_this_depth += 1; // Count this node

            // Unmake the move
            search_pos.unmake_move(mov, undo);

            // Check if this is the best move so far
            if score > best_score {
                best_score = score;
                best_move_this_depth = Some(mov);
            }

            // Check time limit during search
            if let Some(time_limit) = time_limit_ms {
                if is_time_up(&start_time, time_limit) {
                    // Time is up mid-search
                    // If we completed at least one move, use results from this depth
                    // Otherwise, use results from previous depth
                    if best_move_this_depth.is_some() {
                        best_result.best_move = best_move_this_depth;
                        best_result.score = best_score;
                        best_result.depth = depth;
                        best_result.depth_reached = depth;
                        best_result.nodes_searched = total_nodes + nodes_this_depth;
                        best_result.time_ms = start_time.elapsed().as_millis() as u64;
                        print_search_info(depth, &best_result, &start_time);
                    }
                    return best_result;
                }
            }
        }

        // Update total nodes
        total_nodes += nodes_this_depth;

        // Update best result for this depth
        best_result.best_move = best_move_this_depth;
        best_result.score = best_score;
        best_result.depth = depth;
        best_result.depth_reached = depth;
        best_result.nodes_searched = total_nodes;
        best_result.time_ms = start_time.elapsed().as_millis() as u64;

        // Update PV move for next iteration
        pv_move = best_move_this_depth;

        // Print search info for this depth
        print_search_info(depth, &best_result, &start_time);

        // Early exit conditions
        // If we found a mate, no need to search deeper
        // MATE_SCORE is 30000, so mate scores are around 29900+
        if best_score.abs() > 29000 {
            break;
        }
    }

    best_result
}

/// Perform iterative deepening search with aspiration windows
///
/// This is an optimization that narrows the alpha-beta window based on
/// the previous iteration's score, which can lead to more cutoffs.
///
/// # Arguments
/// * `pos` - Current position to search
/// * `color` - Color to move
/// * `max_depth` - Maximum search depth
/// * `time_limit_ms` - Optional time limit in milliseconds
///
/// # Returns
/// SearchResult containing the best move and search statistics
#[allow(dead_code)]
pub fn iterative_deepening_search_with_aspiration(
    pos: &Position,
    color: Color,
    max_depth: u8,
    time_limit_ms: Option<u64>,
) -> SearchResult {
    let start_time = Instant::now();
    let mut best_result = SearchResult::new();
    let mut total_nodes = 0u64;

    // Clone position for searching (we need a mutable copy)
    let mut search_pos = pos.clone();

    // Initialize transposition table (shared across all iterations)
    let mut tt = TranspositionTable::new(); // Default size: 1M entries (~40-80MB)

    // Initialize killer moves and history table
    let mut killers = KillerMoves::new();
    let mut history = HistoryTable::new();

    // Principal variation (best move from previous iteration)
    let mut pv_move: Option<Move> = None;
    let mut prev_score = 0;

    // Ensure we search at least depth 1
    let max_depth = max_depth.max(1);

    // Iterative deepening loop
    for depth in 1..=max_depth {
        // Check time limit before starting new depth
        if let Some(time_limit) = time_limit_ms {
            if is_time_up(&start_time, time_limit) {
                if depth > 1 {
                    print_search_info(depth - 1, &best_result, &start_time);
                }
                break;
            }
        }

        // Generate and order moves for this position
        let moves = generate_ordered_moves(&search_pos, pv_move, &killers, &history, depth);

        // If no legal moves, position is checkmate or stalemate
        if moves.is_empty() {
            best_result.depth = depth;
            best_result.depth_reached = depth;
            best_result.time_ms = start_time.elapsed().as_millis() as u64;

            if search_pos.is_in_check(color) {
                best_result.score = -100000;
            } else {
                best_result.score = 0;
            }
            return best_result;
        }

        // Aspiration window settings
        let window_size = if depth <= 3 { 500 } else { 50 };
        let mut alpha = prev_score - window_size;
        let mut beta = prev_score + window_size;

        // For shallow depths, use full window
        if depth <= 2 {
            alpha = i32::MIN + 1;
            beta = i32::MAX;
        }

        // Search with aspiration window (with re-search if needed)
        let mut search_attempt = 0;
        let max_attempts = 3;

        let (best_score, best_move_this_depth, nodes_this_depth) = loop {
            let mut best_score = i32::MIN + 1;
            let mut best_move_this_depth: Option<Move> = None;
            let mut nodes_this_depth = 0u64;

            for &mov in &moves {
                // Make the move
                let undo = search_pos.make_move_undoable(mov);

                // Search this position
                let (score, _) = negamax(
                    &mut search_pos,
                    depth - 1,
                    -beta,
                    -alpha,
                    color.opposite(),
                    &mut tt,
                    &mut killers,
                    &mut history,
                );
                let score = -score;
                nodes_this_depth += 1;

                // Unmake the move
                search_pos.unmake_move(mov, undo);

                // Check if this is the best move so far
                if score > best_score {
                    best_score = score;
                    best_move_this_depth = Some(mov);
                }

                // Check time limit during search
                if let Some(time_limit) = time_limit_ms {
                    if is_time_up(&start_time, time_limit) {
                        if best_move_this_depth.is_some() {
                            best_result.best_move = best_move_this_depth;
                            best_result.score = best_score;
                            best_result.depth = depth;
                            best_result.depth_reached = depth;
                            best_result.nodes_searched = total_nodes + nodes_this_depth;
                            best_result.time_ms = start_time.elapsed().as_millis() as u64;
                            print_search_info(depth, &best_result, &start_time);
                        }
                        return best_result;
                    }
                }
            }

            // Check if we need to re-search with wider window
            if best_score <= alpha {
                // Failed low, widen alpha
                alpha = i32::MIN + 1;
                search_attempt += 1;
                if search_attempt >= max_attempts {
                    break (best_score, best_move_this_depth, nodes_this_depth);
                }
                continue;
            } else if best_score >= beta {
                // Failed high, widen beta
                beta = i32::MAX;
                search_attempt += 1;
                if search_attempt >= max_attempts {
                    break (best_score, best_move_this_depth, nodes_this_depth);
                }
                continue;
            } else {
                // Search succeeded within window
                break (best_score, best_move_this_depth, nodes_this_depth);
            }
        };

        // Update total nodes
        total_nodes += nodes_this_depth;

        // Update best result for this depth
        best_result.best_move = best_move_this_depth;
        best_result.score = best_score;
        best_result.depth = depth;
        best_result.depth_reached = depth;
        best_result.nodes_searched = total_nodes;
        best_result.time_ms = start_time.elapsed().as_millis() as u64;

        // Update PV move and score for next iteration
        pv_move = best_move_this_depth;
        prev_score = best_score;

        // Print search info for this depth
        print_search_info(depth, &best_result, &start_time);

        // Early exit conditions
        // MATE_SCORE is 30000, so mate scores are around 29900+
        if best_score.abs() > 29000 {
            break;
        }
    }

    best_result
}

/// Check if time limit has been exceeded
fn is_time_up(start_time: &Instant, time_limit_ms: u64) -> bool {
    start_time.elapsed().as_millis() as u64 >= time_limit_ms
}

/// Print search information for a completed depth
fn print_search_info(depth: u8, result: &SearchResult, start_time: &Instant) {
    let elapsed_ms = start_time.elapsed().as_millis() as u64;
    let nps = if elapsed_ms > 0 {
        (result.nodes_searched as f64 / elapsed_ms as f64 * 1000.0) as u64
    } else {
        result.nodes_searched
    };

    println!(
        "info depth {} score cp {} nodes {} time {} nps {} pv {:?}",
        depth,
        result.score,
        result.nodes_searched,
        elapsed_ms,
        nps,
        result.best_move
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_repr::Position;

    #[test]
    fn test_search_starting_position() {
        let mut pos = Position::default();
        let result = iterative_deepening_search(&mut pos, Color::White, 3, None);

        // Should find a move
        assert!(result.best_move.is_some());
        assert!(result.depth > 0);
        assert!(result.nodes_searched > 0);
    }

    #[test]
    fn test_search_with_time_limit() {
        let mut pos = Position::default();
        // Very short time limit should still complete at least depth 1
        let result = iterative_deepening_search(&mut pos, Color::White, 10, Some(100));

        assert!(result.best_move.is_some());
        assert!(result.depth >= 1);
        assert!(result.time_ms <= 150); // Allow some tolerance
    }

    #[test]
    fn test_search_mate_in_one() {
        // Position with mate in one: white to move
        // Back rank mate pattern
        let mut pos = Position::from_fen("6k1/5ppp/8/8/8/8/5PPP/R5K1 w - - 0 1");
        let result = iterative_deepening_search(&mut pos, Color::White, 5, None);

        // Should find the mate
        assert!(result.best_move.is_some());
        // Mate score should be high (MATE_SCORE = 30000, so mate scores are around 29900+)
        assert!(result.score > 29000 || result.score < -29000,
            "Expected mate score (>29000 or <-29000), got {}", result.score);
    }

    #[test]
    fn test_search_no_legal_moves_checkmate() {
        // Checkmate position
        let mut pos = Position::from_fen("rnb1kbnr/pppp1ppp/4p3/8/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 0 1");
        let result = iterative_deepening_search(&mut pos, Color::White, 3, None);

        // No legal moves, in check = checkmate
        assert!(result.best_move.is_none());
        assert_eq!(result.score, -100000);
    }

    #[test]
    fn test_search_no_legal_moves_stalemate() {
        // Stalemate position
        let mut pos = Position::from_fen("k7/8/1Q6/8/8/8/8/K7 b - - 0 1");
        let result = iterative_deepening_search(&mut pos, Color::Black, 3, None);

        // No legal moves, not in check = stalemate
        assert!(result.best_move.is_none());
        assert_eq!(result.score, 0);
    }

    #[test]
    fn test_aspiration_window_search() {
        let mut pos = Position::default();
        let result = iterative_deepening_search_with_aspiration(&mut pos, Color::White, 3, None);

        // Should find a move
        assert!(result.best_move.is_some());
        assert!(result.depth > 0);
        assert!(result.nodes_searched > 0);
    }
}
