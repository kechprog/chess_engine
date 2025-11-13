// Detailed profiling example with timing breakdowns
// Run with: cargo run --release --example profile_ai_detailed

use chess_engine::agent::ai::mcts::MCTSTree;
use chess_engine::game_repr::{Color, Position};
use smallvec::SmallVec;
use std::time::Instant;

fn main() {
    // Use a complex middle-game position for profiling
    // This position has many tactical possibilities
    let fen = "r1bqk2r/pp2bppp/2n1pn2/2pp4/3P4/2N1PN2/PPP1BPPP/R1BQK2R w KQkq -";
    let pos = Position::from_fen(fen);

    println!("Detailed Profiling of AI Performance");
    println!("=====================================");
    println!("Position: {}", fen);
    println!("Running 5000 MCTS iterations...\n");

    let start = Instant::now();

    // Run MCTS with 5000 iterations
    let mut mcts = MCTSTree::new(&pos, Color::White);
    let best_move = mcts.search(&pos, 5000);

    let elapsed = start.elapsed();

    println!("\n=== RESULTS ===");
    println!("Best move found: {:?}", best_move);
    println!("Total time: {:.2?}", elapsed);
    println!("Iterations per second: {:.0}", 5000.0 / elapsed.as_secs_f64());

    println!("\n=== PERFORMANCE BREAKDOWN ===");
    println!("Time per iteration: {:.2}ms", elapsed.as_secs_f64() * 1000.0 / 5000.0);

    // Additional profiling: Run smaller batches to measure individual phases
    println!("\n=== PHASE TIMING (100 iterations sample) ===");

    let mut test_mcts = MCTSTree::new(&pos, Color::White);

    // Measure first 100 iterations
    let t1 = Instant::now();
    test_mcts.search(&pos, 100);
    let first_100 = t1.elapsed();
    println!("First 100 iterations: {:.2?} ({:.2}ms per iteration)",
             first_100, first_100.as_secs_f64() * 1000.0 / 100.0);

    // Measure next 100 iterations (when tree is more developed)
    let t2 = Instant::now();
    test_mcts.search(&pos, 100);
    let next_100 = t2.elapsed();
    println!("Next 100 iterations: {:.2?} ({:.2}ms per iteration)",
             next_100, next_100.as_secs_f64() * 1000.0 / 100.0);

    println!("\n=== ESTIMATED BOTTLENECKS ===");

    // Estimate Position::clone() overhead
    let clone_start = Instant::now();
    for _ in 0..10000 {
        let _p = pos.clone();
    }
    let clone_time = clone_start.elapsed();
    println!("Position::clone() time: {:.2}µs per call",
             clone_time.as_micros() as f64 / 10000.0);

    // Estimate all_legal_moves overhead
    let legal_start = Instant::now();
    let mut moves = SmallVec::new();
    for _ in 0..1000 {
        pos.all_legal_moves_into(&mut moves);
    }
    let legal_time = legal_start.elapsed();
    println!("Position::all_legal_moves_into() time: {:.2}µs per call",
             legal_time.as_micros() as f64 / 1000.0);

    // Estimate evaluation overhead
    use chess_engine::agent::ai::evaluation::evaluate;
    let eval_start = Instant::now();
    for _ in 0..10000 {
        let _e = evaluate(&pos, Color::White);
    }
    let eval_time = eval_start.elapsed();
    println!("evaluate() time: {:.2}µs per call",
             eval_time.as_micros() as f64 / 10000.0);

    // Estimate quick_evaluate overhead
    use chess_engine::agent::ai::evaluation::quick_evaluate;
    let qeval_start = Instant::now();
    for _ in 0..10000 {
        let _e = quick_evaluate(&pos, Color::White);
    }
    let qeval_time = qeval_start.elapsed();
    println!("quick_evaluate() time: {:.2}µs per call",
             qeval_time.as_micros() as f64 / 10000.0);

    println!("\n=== EXTRAPOLATED COST PER ITERATION ===");

    // Rough estimates based on MCTS structure
    // Each iteration involves:
    // - 1-10 Position clones (selection path depth)
    // - 1-2 legal_moves calls (expansion)
    // - Multiple evaluations (node creation, playout)
    // - Playout simulation (50 moves deep, each with evaluation)

    let est_clones_per_iter = 5.0; // Average path depth
    let est_legal_per_iter = 2.0;
    let est_evals_per_iter = 50.0; // Playout evaluations

    let est_clone_cost = est_clones_per_iter * (clone_time.as_micros() as f64 / 10000.0);
    let est_legal_cost = est_legal_per_iter * (legal_time.as_micros() as f64 / 1000.0);
    let est_eval_cost = est_evals_per_iter * (eval_time.as_micros() as f64 / 10000.0);

    let total_est = est_clone_cost + est_legal_cost + est_eval_cost;

    println!("Position clones: {:.2}µs ({:.1}%)", est_clone_cost, est_clone_cost / total_est * 100.0);
    println!("Legal move generation: {:.2}µs ({:.1}%)", est_legal_cost, est_legal_cost / total_est * 100.0);
    println!("Evaluations: {:.2}µs ({:.1}%)", est_eval_cost, est_eval_cost / total_est * 100.0);
    println!("Total estimated: {:.2}µs", total_est);
    println!("Actual per iteration: {:.2}µs", elapsed.as_micros() as f64 / 5000.0);
}
