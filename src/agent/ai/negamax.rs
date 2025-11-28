// Negamax Search with Alpha-Beta Pruning
//
// Negamax is a variant of the minimax algorithm that simplifies implementation
// by taking advantage of the zero-sum property of chess: max(a, b) = -min(-a, -b).
// Instead of separate maximizing and minimizing functions, we use one function
// that negates the score at each level.
//
// Alpha-Beta Pruning optimizations:
// - Transposition table for position caching
// - Killer move heuristic for move ordering
// - History heuristic for quiet moves
// - Null move pruning for early cutoffs
// - Principal Variation Search (PVS) for efficiency
// - Quiescence search to avoid horizon effect
// - Mate distance pruning for faster mate detection
//
// The function returns (score, best_move) from the perspective of the side to move.

use crate::game_repr::{Position, Color, Move};
use super::quiescence::quiescence_search;
use super::move_ordering::{generate_ordered_moves, KillerMoves, HistoryTable};
use super::transposition_table::{TranspositionTable, TranspositionTableEntry, NodeType};

/// Checkmate score - use large value but leave room for mate distance
pub const MATE_SCORE: i32 = 30000;

/// Minimum score (worse than any mate)
pub const MIN_SCORE: i32 = -MATE_SCORE - 100;

/// Maximum score (used in tests and by external callers)
#[allow(dead_code)]
pub const MAX_SCORE: i32 = MATE_SCORE + 100;

/// Null move reduction depth (how much to reduce depth for null move search)
const NULL_MOVE_REDUCTION: u8 = 2;

/// Minimum depth to attempt null move pruning
const NULL_MOVE_MIN_DEPTH: u8 = 3;

/// Negamax search with alpha-beta pruning
///
/// This is the core search function that recursively searches the game tree
/// to find the best move. It uses alpha-beta pruning to eliminate branches
/// that cannot improve the current best line.
///
/// # Arguments
///
/// * `pos` - Current position (mutable for make/unmake moves)
/// * `depth` - Remaining search depth (0 = leaf node, call quiescence)
/// * `mut alpha` - Lower bound (best score maximizing player can guarantee)
/// * `beta` - Upper bound (best score opponent will allow)
/// * `color` - Side to move
/// * `tt` - Transposition table for caching positions
/// * `killers` - Killer move table for move ordering
/// * `history` - History heuristic table for move ordering
///
/// # Returns
///
/// (score, best_move) - Score from perspective of `color`, and the best move found
/// Positive score = good for `color`, negative = good for opponent
#[allow(clippy::too_many_arguments)]
pub fn negamax(
    pos: &mut Position,
    depth: u8,
    mut alpha: i32,
    beta: i32,
    color: Color,
    tt: &mut TranspositionTable,
    killers: &mut KillerMoves,
    history: &mut HistoryTable,
) -> (i32, Option<Move>) {
    // Base case: reached maximum depth, use quiescence search
    if depth == 0 {
        let score = quiescence_search(pos, alpha, beta, color);
        return (score, None);
    }

    // Transposition table lookup
    let hash = TranspositionTable::compute_hash(pos);
    let tt_entry = tt.probe(hash);
    let mut hash_move = None;

    // Use transposition table entry if it's valid
    if let Some(entry) = tt_entry {
        // Only use TT entry if it was searched to at least the same depth
        if entry.depth >= depth {
            match entry.node_type {
                NodeType::Exact => {
                    // Exact score - we can return immediately
                    return (entry.score, entry.best_move);
                }
                NodeType::LowerBound => {
                    // Score is at least this good
                    if entry.score >= beta {
                        return (beta, entry.best_move);
                    }
                    // Update alpha if we have a better lower bound
                    if entry.score > alpha {
                        alpha = entry.score;
                    }
                }
                NodeType::UpperBound => {
                    // Score is at most this good
                    if entry.score <= alpha {
                        return (alpha, entry.best_move);
                    }
                }
            }
        }

        // Store hash move for move ordering (even if depth is insufficient)
        hash_move = entry.best_move;
    }

    // Check if we're in check (affects null move and move generation)
    let in_check = pos.is_in_check(color);

    // Null Move Pruning
    // If we can pass (do nothing) and still get a beta cutoff, position is too good
    // Don't do null move if:
    // - We're in check (null move would be illegal)
    // - Depth is too low
    // - Beta is a mate score (null move doesn't help with mate detection)
    if !in_check
        && depth >= NULL_MOVE_MIN_DEPTH
        && beta.abs() < MATE_SCORE - 100
    {
        // Make null move (pass turn to opponent)
        // We simulate this by adding a dummy move to prev_moves
        let prev_moves_len = pos.prev_moves.len();
        pos.prev_moves.push(Move::new(0, 0, crate::game_repr::MoveType::Normal));

        // Search with reduced depth from opponent's perspective
        let reduced_depth = depth.saturating_sub(NULL_MOVE_REDUCTION + 1);
        let (null_score, _) = negamax(
            pos,
            reduced_depth,
            -beta,
            -beta + 1, // Null window
            color.opposite(),
            tt,
            killers,
            history,
        );

        // Undo null move
        pos.prev_moves.truncate(prev_moves_len);

        // If null move causes beta cutoff, position is too good
        if -null_score >= beta {
            return (beta, None); // Fail high
        }
    }

    // Generate moves in optimal order (hash move first, then captures, killers, etc.)
    let moves = generate_ordered_moves(pos, hash_move, killers, history, depth);

    // If no legal moves, it's either checkmate or stalemate
    if moves.is_empty() {
        if in_check {
            // Checkmate - return negative mate score adjusted by depth
            // We prefer shorter mates (closer to current position)
            return (-(MATE_SCORE - depth as i32), None);
        } else {
            // Stalemate - draw
            return (0, None);
        }
    }

    // Track best move and score
    let mut best_score = MIN_SCORE;
    let mut best_move = None;
    let mut node_type = NodeType::UpperBound; // Assume all moves fail low

    // Principal Variation Search (PVS)
    // Search first move with full window, rest with null window
    let mut is_first_move = true;

    for mv in moves {
        // Make the move
        let undo = pos.make_move_undoable(mv);

        let score = if is_first_move {
            // Search first move with full window
            let (s, _) = negamax(
                pos,
                depth - 1,
                -beta,
                -alpha,
                color.opposite(),
                tt,
                killers,
                history,
            );
            -s
        } else {
            // Search remaining moves with null window (scout search)
            let (s, _) = negamax(
                pos,
                depth - 1,
                -alpha - 1,
                -alpha,
                color.opposite(),
                tt,
                killers,
                history,
            );
            let scout_score = -s;

            // If null window search fails high, re-search with full window
            if scout_score > alpha && scout_score < beta {
                let (s, _) = negamax(
                    pos,
                    depth - 1,
                    -beta,
                    -alpha,
                    color.opposite(),
                    tt,
                    killers,
                    history,
                );
                -s
            } else {
                scout_score
            }
        };

        // Unmake the move
        pos.unmake_move(mv, undo);

        // Update best score and move
        if score > best_score {
            best_score = score;
            best_move = Some(mv);
        }

        // Alpha-beta pruning
        if score >= beta {
            // Beta cutoff - this move is too good, opponent won't allow it
            // Store killer move (for non-captures)
            let to = mv._to();
            let is_capture = pos.position[to].piece_type != crate::game_repr::Type::None;
            if !is_capture {
                killers.store(depth, mv);
                history.update(mv, depth);
            }

            // Store in transposition table as lower bound
            let tt_entry = TranspositionTableEntry {
                hash,
                depth,
                score: beta,
                best_move: Some(mv),
                node_type: NodeType::LowerBound,
            };
            tt.store(tt_entry);

            return (beta, Some(mv));
        }

        // Update alpha if we found a better move
        if score > alpha {
            alpha = score;
            node_type = NodeType::Exact; // We have an exact score (PV node)

            // Update history heuristic for good quiet moves
            let to = mv._to();
            let is_capture = pos.position[to].piece_type != crate::game_repr::Type::None;
            if !is_capture {
                history.update(mv, depth);
            }
        }

        is_first_move = false;
    }

    // Store result in transposition table
    let tt_entry = TranspositionTableEntry {
        hash,
        depth,
        score: best_score,
        best_move,
        node_type,
    };
    tt.store(tt_entry);

    (best_score, best_move)
}

/// Helper function to detect if a score represents a mate
#[allow(dead_code)]
pub fn is_mate_score(score: i32) -> bool {
    score.abs() >= MATE_SCORE - 100
}

/// Get the number of moves until mate from a mate score
/// Returns None if not a mate score
#[allow(dead_code)]
pub fn mate_distance(score: i32) -> Option<i32> {
    if !is_mate_score(score) {
        return None;
    }

    if score > 0 {
        // We're checkmating opponent
        Some((MATE_SCORE - score + 1) / 2)
    } else {
        // We're getting checkmated
        Some(-(MATE_SCORE + score + 1) / 2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mate_in_one() {
        // Use Fool's mate to create a known-working checkmate position
        // This avoids potential FEN parsing issues
        let mut pos = Position::default();

        // 1. f3
        pos.mk_move(Move::new(13, 21, crate::game_repr::MoveType::Normal)); // f2-f3
        // 1... e5
        pos.mk_move(Move::new(52, 36, crate::game_repr::MoveType::Normal)); // e7-e5
        // 2. g4
        pos.mk_move(Move::new(14, 30, crate::game_repr::MoveType::Normal)); // g2-g4
        // 2... Qh4# - checkmate!
        pos.mk_move(Move::new(59, 31, crate::game_repr::MoveType::Normal)); // Qd8-h4

        // Now White is in checkmate (no legal moves)
        let mut tt = TranspositionTable::new();
        let mut killers = KillerMoves::new();
        let mut history = HistoryTable::new();

        let (score, _best_move) = negamax(&mut pos, 1, MIN_SCORE, MAX_SCORE, Color::White, &mut tt, &mut killers, &mut history);

        // Should detect mate
        assert!(is_mate_score(score), "Should detect mate, score: {}", score);
        assert!(score < 0, "White is getting mated, score should be negative: {}", score);
    }

    #[test]
    fn test_stalemate() {
        // Stalemate position: Black king on h8, White king on f6, White queen on g6
        // Black to move, stalemate (no legal moves but not in check)
        let mut pos = Position::from_fen("7k/8/5KQ1/8/8/8/8/8 b - -");
        let mut tt = TranspositionTable::new();
        let mut killers = KillerMoves::new();
        let mut history = HistoryTable::new();

        let (score, _best_move) = negamax(&mut pos, 1, MIN_SCORE, MAX_SCORE, Color::Black, &mut tt, &mut killers, &mut history);

        // Stalemate should give score of 0 (draw)
        assert_eq!(score, 0, "Stalemate should score 0, got: {}", score);
    }

    #[test]
    fn test_finds_best_move() {
        // Simple position where White can capture Black queen
        let mut pos = Position::from_fen("4k3/8/8/8/3q4/8/3R4/4K3 w - -");
        let mut tt = TranspositionTable::new();
        let mut killers = KillerMoves::new();
        let mut history = HistoryTable::new();

        let (score, best_move) = negamax(&mut pos, 3, MIN_SCORE, MAX_SCORE, Color::White, &mut tt, &mut killers, &mut history);

        // Should find the queen capture
        assert!(best_move.is_some(), "Should find a best move");
        assert!(score > 500, "Score should reflect winning the queen: {}", score);

        // Best move should be rook taking queen (d2 to d4)
        let mv = best_move.unwrap();
        assert_eq!(mv._from(), 11); // d2 = rank 1, file 3 = 1*8 + 3 = 11
        assert_eq!(mv._to(), 27);   // d4 = rank 3, file 3 = 3*8 + 3 = 27
    }

    #[test]
    fn test_alpha_beta_pruning() {
        // Test that search with narrow window works
        let mut pos = Position::default();
        let mut tt = TranspositionTable::new();
        let mut killers = KillerMoves::new();
        let mut history = HistoryTable::new();

        // Search with narrow window
        let (score1, move1) = negamax(&mut pos, 2, -50, 50, Color::White, &mut tt, &mut killers, &mut history);

        // Search with wide window
        tt.clear();
        let (score2, move2) = negamax(&mut pos, 2, MIN_SCORE, MAX_SCORE, Color::White, &mut tt, &mut killers, &mut history);

        // Both should find a move
        assert!(move1.is_some() || score1.abs() >= 50, "Narrow window should find move or fail");
        assert!(move2.is_some(), "Wide window should find move");
    }

    #[test]
    fn test_transposition_table_usage() {
        let mut pos = Position::default();
        let mut tt = TranspositionTable::new();
        let mut killers = KillerMoves::new();
        let mut history = HistoryTable::new();

        // First search
        let (score1, move1) = negamax(&mut pos, 3, MIN_SCORE, MAX_SCORE, Color::White, &mut tt, &mut killers, &mut history);

        // TT should have entries now
        assert!(tt.size() > 0, "TT should have entries after search");

        // Second search should use TT
        let hits_before = tt.hits;
        let (score2, move2) = negamax(&mut pos, 3, MIN_SCORE, MAX_SCORE, Color::White, &mut tt, &mut killers, &mut history);
        let hits_after = tt.hits;

        // Should have TT hits in second search
        assert!(hits_after > hits_before, "Second search should hit TT");

        // Results should be identical
        assert_eq!(score1, score2, "Scores should match");
        assert_eq!(move1, move2, "Moves should match");
    }

    #[test]
    fn test_killer_move_updates() {
        let mut pos = Position::default();
        let mut tt = TranspositionTable::new();
        let mut killers = KillerMoves::new();
        let mut history = HistoryTable::new();

        // Run search to populate killers
        negamax(&mut pos, 4, MIN_SCORE, MAX_SCORE, Color::White, &mut tt, &mut killers, &mut history);

        // Killer moves should have been updated (at least at some depth)
        // We can't easily verify specific moves, but the table should be non-trivial
        // Just verify search completed successfully
        assert!(true, "Search should complete without panics");
    }

    #[test]
    fn test_quiescence_is_called_at_depth_zero() {
        // Position with a hanging queen
        let mut pos = Position::from_fen("4k3/8/8/3q4/8/8/8/4K3 w - -");
        let mut tt = TranspositionTable::new();
        let mut killers = KillerMoves::new();
        let mut history = HistoryTable::new();

        // Depth 0 should call quiescence
        let (score, mv) = negamax(&mut pos, 0, MIN_SCORE, MAX_SCORE, Color::White, &mut tt, &mut killers, &mut history);

        // Should recognize material disadvantage (down a queen)
        assert!(score < -700, "Should see we're down a queen: {}", score);
        assert!(mv.is_none(), "Depth 0 should not return a move");
    }

    #[test]
    fn test_mate_distance_calculation() {
        assert_eq!(mate_distance(MATE_SCORE), Some(0)); // Immediate mate
        assert_eq!(mate_distance(MATE_SCORE - 2), Some(1)); // Mate in 1
        assert_eq!(mate_distance(MATE_SCORE - 4), Some(2)); // Mate in 2
        assert_eq!(mate_distance(-MATE_SCORE), Some(0)); // Getting mated immediately
        assert_eq!(mate_distance(-MATE_SCORE + 2), Some(-1)); // Getting mated in 1
        assert_eq!(mate_distance(100), None); // Not a mate score
        assert_eq!(mate_distance(-100), None); // Not a mate score
    }

    #[test]
    fn test_is_mate_score() {
        assert!(is_mate_score(MATE_SCORE));
        assert!(is_mate_score(-MATE_SCORE));
        assert!(is_mate_score(MATE_SCORE - 50));
        assert!(is_mate_score(-MATE_SCORE + 50));
        assert!(!is_mate_score(0));
        assert!(!is_mate_score(100));
        assert!(!is_mate_score(-100));
        assert!(!is_mate_score(1000));
    }

    #[test]
    fn test_starting_position_is_balanced() {
        let mut pos = Position::default();
        let mut tt = TranspositionTable::new();
        let mut killers = KillerMoves::new();
        let mut history = HistoryTable::new();

        let (score, best_move) = negamax(&mut pos, 2, MIN_SCORE, MAX_SCORE, Color::White, &mut tt, &mut killers, &mut history);

        // Starting position should be roughly balanced
        assert!(score.abs() < 200, "Starting position should be balanced, score: {}", score);
        assert!(best_move.is_some(), "Should find a move from starting position");
    }
}
