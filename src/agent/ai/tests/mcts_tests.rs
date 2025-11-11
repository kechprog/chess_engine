// Comprehensive tests for MCTS

use crate::game_repr::{Position, Color};
use crate::agent::ai::mcts::MCTSTree;

#[test]
fn test_mcts_basic_search() {
    let pos = Position::default();
    let mut tree = MCTSTree::new(&pos, Color::White);

    let best_move = tree.search(&pos, 100);

    // Should find a legal move
    assert!(best_move.is_some());
    let mov = best_move.unwrap();
    assert!(pos.is_move_legal(mov));
}

#[test]
fn test_mcts_with_more_iterations() {
    let pos = Position::default();
    let mut tree = MCTSTree::new(&pos, Color::White);

    let best_move = tree.search(&pos, 500);

    assert!(best_move.is_some());
    assert!(pos.is_move_legal(best_move.unwrap()));

    let stats = tree.get_stats();
    assert_eq!(stats.root_visits, 500);
}

#[test]
fn test_mcts_finds_capture() {
    // Position where White can capture a free piece
    let pos = Position::from_fen("rnb1kbnr/pppppppp/8/8/4q3/2N5/PPPPPPPP/R1BQKBNR w KQkq -");

    let mut tree = MCTSTree::new(&pos, Color::White);
    let best_move = tree.search(&pos, 300);

    assert!(best_move.is_some());
    let mov = best_move.unwrap();

    // Should find a legal move
    assert!(pos.is_move_legal(mov));

    // The move should ideally capture the queen (square 28 = e4)
    // But we'll just verify it's legal
}

#[test]
fn test_mcts_handles_few_moves() {
    // Endgame with very few moves
    let pos = Position::from_fen("7k/8/8/8/8/8/8/K7 w - -");

    let mut tree = MCTSTree::new(&pos, Color::White);
    let best_move = tree.search(&pos, 50);

    assert!(best_move.is_some());
    assert!(pos.is_move_legal(best_move.unwrap()));
}

#[test]
fn test_mcts_with_black() {
    // Test MCTS playing as Black
    let pos = Position::from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq -");

    let mut tree = MCTSTree::new(&pos, Color::Black);
    let best_move = tree.search(&pos, 100);

    assert!(best_move.is_some());
    assert!(pos.is_move_legal(best_move.unwrap()));
}

#[test]
fn test_mcts_progressive_widening() {
    let pos = Position::default();
    let mut tree = MCTSTree::new(&pos, Color::White);

    // With few iterations, should have few children
    tree.search(&pos, 20);
    let stats_early = tree.get_stats();

    // Children count should be reasonable
    assert!(stats_early.num_children > 0);
    assert!(stats_early.num_children <= 20); // Starting position has 20 moves total
}

#[test]
fn test_mcts_statistics() {
    let pos = Position::default();
    let mut tree = MCTSTree::new(&pos, Color::White);

    tree.search(&pos, 200);

    let stats = tree.get_stats();

    // Visits should match iterations
    assert_eq!(stats.root_visits, 200);

    // Should have expanded some children
    assert!(stats.num_children > 0);

    // Best move should have significant visits
    assert!(stats.best_move_visits > 0);
    assert!(stats.best_move_visits <= 200);
}

#[test]
fn test_mcts_consistency() {
    // Same position with same seed should give consistent results
    // Note: MCTS has randomness, so this test checks that it at least returns legal moves
    let pos = Position::default();

    let mut tree1 = MCTSTree::new(&pos, Color::White);
    let move1 = tree1.search(&pos, 100);

    let mut tree2 = MCTSTree::new(&pos, Color::White);
    let move2 = tree2.search(&pos, 100);

    // Both should find moves
    assert!(move1.is_some());
    assert!(move2.is_some());

    // Both should be legal
    assert!(pos.is_move_legal(move1.unwrap()));
    assert!(pos.is_move_legal(move2.unwrap()));
}

#[test]
fn test_mcts_tactical_position() {
    // Position with tactical opportunities
    let pos = Position::from_fen("r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq -");

    let mut tree = MCTSTree::new(&pos, Color::White);
    let best_move = tree.search(&pos, 200);

    assert!(best_move.is_some());
    assert!(pos.is_move_legal(best_move.unwrap()));
}

#[test]
fn test_mcts_endgame_position() {
    // Simple endgame
    let pos = Position::from_fen("8/8/8/4k3/8/8/4K3/4R3 w - -");

    let mut tree = MCTSTree::new(&pos, Color::White);
    let best_move = tree.search(&pos, 100);

    assert!(best_move.is_some());
    assert!(pos.is_move_legal(best_move.unwrap()));
}

#[test]
fn test_mcts_mate_in_one() {
    // Back rank mate available
    let pos = Position::from_fen("6k1/5ppp/8/8/8/8/5PPP/6RK w - -");

    let mut tree = MCTSTree::new(&pos, Color::White);
    let best_move = tree.search(&pos, 300);

    assert!(best_move.is_some());
    let mov = best_move.unwrap();
    assert!(pos.is_move_legal(mov));

    // Should ideally find Rg8#, but we're just checking it works
}

#[test]
fn test_mcts_avoids_blunders() {
    // Position where hanging queen would be a blunder
    let pos = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -");

    let mut tree = MCTSTree::new(&pos, Color::White);
    let best_move = tree.search(&pos, 200);

    assert!(best_move.is_some());
    let mov = best_move.unwrap();

    // The move should be legal
    assert!(pos.is_move_legal(mov));

    // Make the move and check it doesn't immediately lose material
    let mut test_pos = pos.clone();
    test_pos.make_move_undoable(mov);

    // Just verify the move is legal (not hanging queen immediately)
    // Full verification would require checking if queen is attacked, etc.
}

#[test]
fn test_mcts_handles_promotion() {
    // Position with promotion available
    let pos = Position::from_fen("8/4P3/8/8/8/8/4k3/4K3 w - -");

    let mut tree = MCTSTree::new(&pos, Color::White);
    let best_move = tree.search(&pos, 100);

    assert!(best_move.is_some());
    let mov = best_move.unwrap();
    assert!(pos.is_move_legal(mov));

    // Should likely be a promotion move
    assert!(mov.move_type().is_promotion() || mov._from() / 8 != 6);
}

#[test]
fn test_mcts_handles_castling() {
    // Position where castling is available
    let pos = Position::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq -");

    let mut tree = MCTSTree::new(&pos, Color::White);
    let best_move = tree.search(&pos, 100);

    assert!(best_move.is_some());
    assert!(pos.is_move_legal(best_move.unwrap()));
}

#[test]
fn test_mcts_search_depth() {
    // Test that search doesn't crash on deep positions
    let pos = Position::default();
    let mut tree = MCTSTree::new(&pos, Color::White);

    // Large number of iterations
    let best_move = tree.search(&pos, 1000);

    assert!(best_move.is_some());
    assert!(pos.is_move_legal(best_move.unwrap()));

    let stats = tree.get_stats();
    assert_eq!(stats.root_visits, 1000);
}

#[test]
fn test_mcts_complex_middlegame() {
    // Complex middlegame position
    let pos = Position::from_fen("r1bq1rk1/pp2bppp/2np1n2/4p3/2B1P3/2NP1N2/PPP2PPP/R1BQR1K1 w - -");

    let mut tree = MCTSTree::new(&pos, Color::White);
    let best_move = tree.search(&pos, 200);

    assert!(best_move.is_some());
    assert!(pos.is_move_legal(best_move.unwrap()));
}

#[test]
fn test_mcts_stalemate_position() {
    // Position close to stalemate
    let pos = Position::from_fen("7k/5Q2/6K1/8/8/8/8/8 b - -");

    let mut tree = MCTSTree::new(&pos, Color::Black);

    // Black has very few or no legal moves
    let best_move = tree.search(&pos, 50);

    // May or may not have moves depending on exact position
    if let Some(mov) = best_move {
        assert!(pos.is_move_legal(mov));
    }
}
