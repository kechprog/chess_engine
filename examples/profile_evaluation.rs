// Profile evaluation functions in detail
// Run with: cargo run --release --example profile_evaluation

use chess_engine::game_repr::{Color, Position};
use chess_engine::agent::ai::evaluation::{evaluate, quick_evaluate};
use chess_engine::agent::ai::move_ordering::generate_ordered_moves;
use std::time::Instant;

fn main() {
    let fen = "r1bqk2r/pp2bppp/2n1pn2/2pp4/3P4/2N1PN2/PPP1BPPP/R1BQK2R w KQkq -";
    let pos = Position::from_fen(fen);

    println!("Detailed Evaluation Function Profiling");
    println!("======================================\n");

    // Test 1: evaluate() breakdown
    println!("=== evaluate() function ===");
    let start = Instant::now();
    for _ in 0..10000 {
        let _e = evaluate(&pos, Color::White);
    }
    let eval_time = start.elapsed();
    println!("10,000 calls: {:.2?}", eval_time);
    println!("Per call: {:.2}µs", eval_time.as_micros() as f64 / 10000.0);

    // Test 2: quick_evaluate() (material + PST only)
    println!("\n=== quick_evaluate() function ===");
    let start = Instant::now();
    for _ in 0..10000 {
        let _e = quick_evaluate(&pos, Color::White);
    }
    let qeval_time = start.elapsed();
    println!("10,000 calls: {:.2?}", qeval_time);
    println!("Per call: {:.2}µs", qeval_time.as_micros() as f64 / 10000.0);

    // Test 3: Break down evaluation with opponent moves
    println!("\n=== Evaluation with opponent moves considered ===");

    // Measure move generation cost
    let start = Instant::now();
    for _ in 0..1000 {
        let _moves = generate_ordered_moves(&pos, Color::Black, 10);
    }
    let gen_moves_time = start.elapsed();
    println!("generate_ordered_moves(10): {:.2}µs per call",
             gen_moves_time.as_micros() as f64 / 1000.0);

    // Measure evaluation after move execution
    let moves = generate_ordered_moves(&pos, Color::Black, 10);
    println!("Number of opponent moves to consider: {}", moves.len());

    let start = Instant::now();
    for _ in 0..1000 {
        for &mov in &moves {
            let mut temp_pos = pos.clone();
            temp_pos.make_move_undoable(mov);
            let _eval = evaluate(&temp_pos, Color::White);
        }
    }
    let loop_time = start.elapsed();
    println!("Loop through {} moves (clone + make_move + evaluate): {:.2}µs per iteration",
             moves.len(), loop_time.as_micros() as f64 / 1000.0);
    println!("  Cost per move: {:.2}µs", loop_time.as_micros() as f64 / 1000.0 / moves.len() as f64);

    // Component breakdown
    println!("\n=== Component costs ===");

    // Position::clone()
    let start = Instant::now();
    for _ in 0..10000 {
        let _p = pos.clone();
    }
    let clone_time = start.elapsed();
    println!("Position::clone(): {:.2}µs", clone_time.as_micros() as f64 / 10000.0);

    // Position::make_move_undoable()
    let mov = moves[0];
    let start = Instant::now();
    for _ in 0..10000 {
        let mut p = pos.clone();
        p.make_move_undoable(mov);
    }
    let make_move_time = start.elapsed();
    let make_move_only = make_move_time.as_micros() as f64 / 10000.0 - clone_time.as_micros() as f64 / 10000.0;
    println!("Position::make_move_undoable(): {:.2}µs", make_move_only);

    println!("\n=== CONCLUSION ===");
    println!("MCTS now uses 2-ply evaluation through natural tree expansion:");
    println!("- Node expansion uses quick_evaluate() for UCB guidance");
    println!("- Playouts use evaluate() for move selection");
    println!("- Opponent responses handled by MCTS tree structure");
    println!("\nKey metrics:");
    println!("- evaluate() cost: {:.2}µs per call", eval_time.as_micros() as f64 / 10000.0);
    println!("- quick_evaluate() cost: {:.2}µs per call", qeval_time.as_micros() as f64 / 10000.0);
    println!("- Move generation (10 moves): {:.2}µs", gen_moves_time.as_micros() as f64 / 1000.0);
}
