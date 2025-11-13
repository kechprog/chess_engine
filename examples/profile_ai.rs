// Profiling example for AI performance analysis
// Run with: cargo run --release --example profile_ai
// For flamegraph: cargo flamegraph --example profile_ai

use chess_engine::agent::ai::mcts::{MCTSTree, search_multithreaded};
use chess_engine::game_repr::{Color, Position};

fn main() {
    // Use a complex middle-game position for profiling
    // This position has many tactical possibilities
    let fen = "r1bqk2r/pp2bppp/2n1pn2/2pp4/3P4/2N1PN2/PPP1BPPP/R1BQK2R w KQkq -";
    let pos = Position::from_fen(fen);

    println!("Profiling AI on position: {}", fen);
    println!();

    // Test single-threaded version
    println!("=== Single-threaded MCTS ===");
    println!("Running 5000 iterations...");
    let start = std::time::Instant::now();
    let mut mcts = MCTSTree::new(&pos, Color::White);
    let best_move = mcts.search(&pos, 5000);
    let elapsed_single = start.elapsed();
    let stats_single = mcts.get_stats();

    println!("Best move: {:?}", best_move);
    println!("Time elapsed: {:.2?}", elapsed_single);
    println!("Iterations per second: {:.0}", 5000.0 / elapsed_single.as_secs_f64());
    println!("Stats: {:?}", stats_single);
    println!();

    // Test multithreaded version with auto-detect
    println!("=== Multithreaded MCTS (auto-detect threads) ===");
    println!("Running 5000 iterations...");
    let start = std::time::Instant::now();
    let (best_move_mt, stats_mt) = search_multithreaded(&pos, Color::White, 5000, None);
    let elapsed_multi = start.elapsed();

    println!("Best move: {:?}", best_move_mt);
    println!("Time elapsed: {:.2?}", elapsed_multi);
    println!("Iterations per second: {:.0}", 5000.0 / elapsed_multi.as_secs_f64());
    println!("Stats: {:?}", stats_mt);
    println!();

    // Compare performance
    println!("=== Performance Comparison ===");
    let speedup = elapsed_single.as_secs_f64() / elapsed_multi.as_secs_f64();
    println!("Speedup: {:.2}×", speedup);
    println!("Threads used: {}", stats_mt.num_threads);
    println!("Efficiency: {:.1}% (ideal: 100%)", (speedup / stats_mt.num_threads as f64) * 100.0);
}
