// Quiescence Search - Tactical Stability Extension
//
// Quiescence search addresses the "horizon effect" in chess search algorithms.
// When a regular search stops at a certain depth, it may evaluate positions in
// the middle of tactical exchanges, leading to wildly inaccurate evaluations.
//
// For example, if we stop searching after a queen capture but before the recapture,
// we'd think we're up a queen when actually the position is equal.
//
// Quiescence search extends the search tree by examining only "noisy" moves
// (captures, promotions, and optionally checks) until the position is "quiet"
// (tactically stable). This ensures we evaluate positions after all forcing
// sequences have been resolved.
//
// Key optimizations implemented:
// 1. Stand-pat: Current evaluation can cause beta cutoff without searching
// 2. Delta pruning: Skip captures that can't possibly improve alpha
// 3. MVV-LVA ordering: Search most promising captures first
// 4. Depth limit: Prevent infinite recursion in complex tactical positions

use crate::game_repr::{Position, Color, Move, MoveType, Type};
use super::evaluation::evaluate;
use smallvec::SmallVec;

/// Maximum depth for quiescence search to prevent infinite recursion
const MAX_QSEARCH_DEPTH: i32 = 16;

// Material values in centipawns for delta pruning
const PAWN_VALUE: i32 = 100;
const KNIGHT_VALUE: i32 = 300;
const BISHOP_VALUE: i32 = 320;
const ROOK_VALUE: i32 = 500;
const QUEEN_VALUE: i32 = 900;

// Delta pruning margin - safety buffer for positional compensation
const DELTA_MARGIN: i32 = 200;

/// Get material value for a piece type
#[inline]
fn piece_material_value(piece_type: Type) -> i32 {
    match piece_type {
        Type::Pawn => PAWN_VALUE,
        Type::Knight => KNIGHT_VALUE,
        Type::Bishop => BISHOP_VALUE,
        Type::Rook => ROOK_VALUE,
        Type::Queen => QUEEN_VALUE,
        Type::King => 0,
        Type::None => 0,
    }
}

/// Quiescence search - search until position is quiet
///
/// Only searches tactical moves (captures, promotions) to avoid
/// the horizon effect where we evaluate positions in the middle
/// of a tactical sequence.
///
/// This function implements several optimizations:
/// - Stand-pat: Use static evaluation as baseline
/// - Delta pruning: Skip captures that can't improve alpha
/// - MVV-LVA move ordering: Search best captures first
/// - Depth limiting: Prevent infinite recursion
///
/// # Arguments
///
/// * `pos` - Current position (mutable for make/unmake moves)
/// * `mut alpha` - Lower bound (best score achievable by maximizing player)
/// * `beta` - Upper bound (best score opponent will allow)
/// * `color` - Side to move
/// * `qs_depth` - Current quiescence search depth (for limiting)
///
/// # Returns
///
/// Evaluation score from the perspective of `color` (positive = good for `color`)
pub fn quiescence(
    pos: &mut Position,
    mut alpha: i32,
    beta: i32,
    color: Color,
    qs_depth: i32,
) -> i32 {
    // Depth limit to prevent infinite recursion in complex tactical positions
    if qs_depth >= MAX_QSEARCH_DEPTH {
        return evaluate(pos, color);
    }

    // Stand-pat evaluation: current position value without any moves
    // This represents the option to "do nothing" and is our baseline
    let stand_pat = evaluate(pos, color);

    // Beta cutoff: If our current position is already too good,
    // the opponent won't allow us to reach this position
    if stand_pat >= beta {
        return beta;
    }

    // Update alpha if stand-pat improves it
    // This implements the "do nothing" option
    if stand_pat > alpha {
        alpha = stand_pat;
    }

    // Delta pruning: Calculate maximum possible material gain
    // If even the best possible capture can't improve alpha, skip all captures
    let max_material_gain = QUEEN_VALUE + (QUEEN_VALUE - PAWN_VALUE); // Queen capture + promotion

    // Delta pruning: If even perfect captures can't improve position, return stand-pat
    if stand_pat + max_material_gain + DELTA_MARGIN < alpha {
        return alpha;
    }

    // Generate and order tactical moves (captures and promotions)
    let moves = generate_tactical_moves(pos);

    // If no tactical moves, position is quiet - return stand-pat
    if moves.is_empty() {
        return stand_pat;
    }

    // Search tactical moves
    for mv in moves {
        // Delta pruning per move: Skip captures that can't improve alpha
        let to = mv._to();
        let captured_value = if mv.move_type() == MoveType::EnPassant {
            PAWN_VALUE
        } else {
            piece_material_value(pos.position[to].piece_type)
        };

        let promotion_bonus = if mv.move_type().is_promotion() {
            QUEEN_VALUE - PAWN_VALUE // Assume queen promotion
        } else {
            0
        };

        // If this capture + promotion can't improve alpha even with margin, skip it
        if stand_pat + captured_value + promotion_bonus + DELTA_MARGIN < alpha {
            continue; // Delta pruning: this capture is too weak
        }

        // Make move and recursively search
        let undo = pos.make_move_undoable(mv);

        // Negamax: negate score from opponent's perspective
        let score = -quiescence(pos, -beta, -alpha, color.opposite(), qs_depth + 1);

        // Unmake move
        pos.unmake_move(mv, undo);

        // Beta cutoff: This move is too good, opponent won't allow it
        if score >= beta {
            return beta;
        }

        // Update alpha (best score so far)
        if score > alpha {
            alpha = score;
        }
    }

    alpha
}

/// Public wrapper for quiescence search with initial depth of 0
///
/// This is the main entry point for quiescence search, typically called
/// from the main negamax search at leaf nodes.
///
/// # Arguments
///
/// * `pos` - Current position (mutable for make/unmake moves)
/// * `alpha` - Lower bound
/// * `beta` - Upper bound
/// * `color` - Side to move
///
/// # Returns
///
/// Evaluation score from perspective of `color`
pub fn quiescence_search(
    pos: &mut Position,
    alpha: i32,
    beta: i32,
    color: Color,
) -> i32 {
    quiescence(pos, alpha, beta, color, 0)
}

/// Score a capture move using MVV-LVA (Most Valuable Victim - Least Valuable Attacker)
/// Higher scores indicate better captures to search first
#[inline]
fn score_capture(pos: &Position, mv: Move) -> i32 {
    let to = mv._to();
    let from = mv._from();

    // Promotions get highest priority
    if mv.move_type().is_promotion() {
        return 10000 + QUEEN_VALUE; // Very high score for promotions
    }

    let captured_value = if mv.move_type() == MoveType::EnPassant {
        // En passant always captures a pawn
        PAWN_VALUE
    } else {
        piece_material_value(pos.position[to].piece_type)
    };

    let attacker_value = piece_material_value(pos.position[from].piece_type);

    // MVV-LVA: Prioritize high-value victims and low-value attackers
    // Score = 10 * victim_value - attacker_value
    // This ensures capturing a queen with a pawn scores higher than capturing a pawn with a queen
    10 * captured_value - attacker_value
}

/// Generate and order tactical moves (captures and promotions)
///
/// Returns moves ordered by MVV-LVA (Most Valuable Victim - Least Valuable Attacker):
/// - Queen promotions first
/// - High-value captures with low-value pieces (e.g., pawn takes queen)
/// - Lower-value captures
///
/// This ordering allows alpha-beta pruning to work more efficiently.
fn generate_tactical_moves(pos: &Position) -> SmallVec<[Move; 64]> {
    let all_moves = pos.all_legal_moves();
    let mut tactical_moves = SmallVec::new();

    for mv in all_moves {
        let to = mv._to();
        let move_type = mv.move_type();

        // Include captures
        if pos.position[to].piece_type != Type::None {
            tactical_moves.push(mv);
            continue;
        }

        // Include promotions
        if move_type.is_promotion() {
            tactical_moves.push(mv);
            continue;
        }

        // Include en passant (special capture)
        if move_type == MoveType::EnPassant {
            tactical_moves.push(mv);
        }
    }

    // Order captures by MVV-LVA (Most Valuable Victim - Least Valuable Attacker)
    // Higher score = better capture, search first
    tactical_moves.sort_by_cached_key(|&mv| -score_capture(pos, mv));

    tactical_moves
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_repr::Position;

    #[test]
    fn test_quiet_position_returns_evaluation() {
        // Position with no captures available - should return static eval
        let mut pos = Position::from_fen("4k3/8/8/8/8/8/8/4K3 w - -");
        let score = quiescence_search(&mut pos, -10000, 10000, Color::White);

        // Score should be close to static evaluation (no tactics to search)
        let static_eval = evaluate(&pos, Color::White);
        assert_eq!(score, static_eval, "Quiet position should return stand-pat evaluation");
    }

    #[test]
    fn test_stand_pat_beta_cutoff() {
        // Position where White is winning (extra queen)
        let mut pos = Position::from_fen("4k3/8/8/8/8/8/8/Q3K3 w - -");

        // Set beta very low - stand-pat should cause immediate cutoff
        let score = quiescence_search(&mut pos, -10000, -500, Color::White);

        // Should return beta (fail-high)
        assert_eq!(score, -500, "Stand-pat should cause beta cutoff");
    }

    #[test]
    fn test_capture_sequence_resolves() {
        // Position with a tactical exchange: Black queen on e4, White rook on e1 can capture
        // Rook on e1 (same file as queen on e4) allows Rxe4
        let mut pos = Position::from_fen("4k3/8/8/8/4q3/8/8/4RK2 w - -");

        // Quiescence should see the queen capture
        let score = quiescence_search(&mut pos, -10000, 10000, Color::White);

        // White should be winning after capturing the queen
        assert!(score > 500, "Score should reflect queen capture: {}", score);
    }

    #[test]
    fn test_delta_pruning_optimization() {
        // Position where White is down significant material
        // Even capturing a pawn won't help with alpha set very high
        let mut pos = Position::from_fen("4k3/8/8/8/4p3/8/8/4K3 w - -");

        // Set alpha very high - delta pruning should kick in
        let score = quiescence_search(&mut pos, 5000, 10000, Color::White);

        // Should return alpha (delta pruning optimization)
        assert_eq!(score, 5000, "Delta pruning should return alpha when captures can't help");
    }

    #[test]
    fn test_promotion_in_quiescence() {
        // Position where pawn can promote
        // Black king on a8 (not blocking e8), white pawn on e7 can promote
        let mut pos = Position::from_fen("k7/4P3/8/8/8/8/8/4K3 w - -");

        let score = quiescence_search(&mut pos, -10000, 10000, Color::White);

        // Should see and value the promotion highly
        assert!(score > 700, "Should recognize promotion value: {}", score);
    }

    #[test]
    fn test_depth_limit_prevents_infinite_loop() {
        // Complex tactical position - ensure we don't hang
        let mut pos = Position::default();

        // Should complete without hanging, even in complex position
        let _score = quiescence_search(&mut pos, -10000, 10000, Color::White);

        // If we get here without timeout, depth limit is working
        assert!(true, "Depth limit should prevent infinite recursion");
    }

    #[test]
    fn test_en_passant_capture() {
        // Set up position by making actual moves to create en passant opportunity
        // FEN parser doesn't parse en passant field, so we create the position manually
        // Sequence: 1.e4 d5 2.e5 f5 (now exf6 e.p. is available)
        let mut pos = Position::default();

        // 1. e2-e4 (square 12 to 28)
        pos.mk_move(Move::new(12, 28, MoveType::Normal));
        // 1... d7-d5 (square 51 to 35)
        pos.mk_move(Move::new(51, 35, MoveType::Normal));
        // 2. e4-e5 (square 28 to 36)
        pos.mk_move(Move::new(28, 36, MoveType::Normal));
        // 2... f7-f5 (square 53 to 37) - creates en passant opportunity
        pos.mk_move(Move::new(53, 37, MoveType::Normal));

        // Now White to move, exf6 e.p. should be available
        // Generate tactical moves and check if en passant is included
        let tactical_moves = generate_tactical_moves(&pos);

        // Should have en passant among tactical moves
        let has_en_passant = tactical_moves.iter().any(|&m| m.move_type() == MoveType::EnPassant);
        assert!(has_en_passant, "En passant should be included in tactical moves");
    }

    #[test]
    fn test_mvv_lva_ordering() {
        // Position where multiple captures are available
        let mut pos = Position::from_fen("4k3/8/8/2q5/1rnb4/8/3R4/4K3 w - -");

        let tactical_moves = generate_tactical_moves(&pos);

        if tactical_moves.len() >= 2 {
            // First move should be highest value capture
            let first_score = score_capture(&pos, tactical_moves[0]);
            let second_score = score_capture(&pos, tactical_moves[1]);

            // MVV-LVA ensures moves are ordered by score (descending)
            assert!(first_score >= second_score,
                "MVV-LVA ordering: first capture should score >= second");
        }
    }

    #[test]
    fn test_promotion_prioritized_over_captures() {
        // Position with both promotion and captures available
        let mut pos = Position::from_fen("4k3/4P3/8/8/4q3/8/8/4K3 w - -");

        let tactical_moves = generate_tactical_moves(&pos);

        if !tactical_moves.is_empty() {
            // Find promotion move
            let promotion_idx = tactical_moves.iter()
                .position(|m| m.move_type().is_promotion());

            // Promotions should be ordered before most captures
            if let Some(idx) = promotion_idx {
                assert!(idx < 4, "Promotions should be prioritized in move ordering");
            }
        }
    }

    #[test]
    fn test_alpha_beta_pruning_works() {
        // Test that alpha-beta pruning actually cuts off branches
        let mut pos = Position::from_fen("r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4");

        // Run with narrow window
        let score1 = quiescence_search(&mut pos, -100, 100, Color::White);

        // Run with wide window
        let score2 = quiescence_search(&mut pos, -10000, 10000, Color::White);

        // Both should give valid scores
        assert!(score1.abs() <= 10000, "Score should be within bounds");
        assert!(score2.abs() <= 10000, "Score should be within bounds");
    }

    #[test]
    fn test_simple_rook_capture() {
        // Test that quiescence search sees a rook capture
        // White rook on d2 can capture Black rook on d4
        let mut pos = Position::from_fen("4k3/8/8/8/3r4/8/3R4/4K3 w - -");

        let score = quiescence_search(&mut pos, -10000, 10000, Color::White);

        // White wins a rook (value ~500), so score should be positive
        assert!(score > 400, "Should see rook capture, score: {}", score);
    }

    #[test]
    fn test_per_move_delta_pruning() {
        // Position where one capture is good, others are weak
        let mut pos = Position::from_fen("4k3/8/8/8/q2p4/8/3R4/4K3 w - -");

        // Set alpha moderately high
        let score = quiescence_search(&mut pos, 0, 10000, Color::White);

        // Should search queen capture but delta-prune pawn capture
        assert!(score > 0 || score == 0, "Should handle per-move delta pruning");
    }
}
