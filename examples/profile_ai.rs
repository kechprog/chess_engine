// Profiling example for AI performance analysis
// Run with: cargo run --release --example profile_ai
// For flamegraph: cargo flamegraph --example profile_ai

use chess_engine::agent::ai::mcts::MCTSTree;
use chess_engine::game_repr::{Color, Position};

fn main() {
    // Use a complex middle-game position for profiling
    // This position has many tactical possibilities
    let fen = "r1bqk2r/pp2bppp/2n1pn2/2pp4/3P4/2N1PN2/PPP1BPPP/R1BQK2R w KQkq -";
    let pos = Position::from_fen(fen);

    println!("Profiling AI on position: {}", fen);
    println!("Running 5000 MCTS iterations...");

    let start = std::time::Instant::now();

    // Run MCTS with 5000 iterations
    let mut mcts = MCTSTree::new(&pos, Color::White);
    let best_move = mcts.search(&pos, 5000);

    let elapsed = start.elapsed();

    println!("Best move found: {:?}", best_move);
    println!("Time elapsed: {:.2?}", elapsed);
    println!("Iterations per second: {:.0}", 5000.0 / elapsed.as_secs_f64());
}
