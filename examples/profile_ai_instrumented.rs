// Instrumented profiling to count actual operations
// Run with: cargo run --release --example profile_ai_instrumented

use chess_engine::game_repr::{Color, Position};
use std::time::Instant;

// Simplified MCTS with operation counting
struct InstrumentedMCTS {
    position_clones: u64,
    moves_executed: u64,
    evaluations: u64,
    legal_move_gens: u64,
    playout_depth_total: u64,
}

impl InstrumentedMCTS {
    fn new() -> Self {
        Self {
            position_clones: 0,
            moves_executed: 0,
            evaluations: 0,
            legal_move_gens: 0,
            playout_depth_total: 0,
        }
    }

    // Simulate one MCTS iteration by running the actual code path
    fn run_iteration(&mut self, pos: &Position) {
        use chess_engine::agent::ai::move_ordering::generate_ordered_moves;
        use chess_engine::agent::ai::evaluation::evaluate;

        // Selection phase: clone position to traverse tree (estimate 3-5 clones)
        self.position_clones += 4; // Average path depth

        // Expansion: generate legal moves
        self.legal_move_gens += 1;
        let moves = generate_ordered_moves(pos, Color::White, 15);
        if moves.is_empty() {
            return;
        }

        // Create child node: evaluate with quick_evaluate
        self.evaluations += 1;
        let mut child_pos = pos.clone();
        self.position_clones += 1;
        child_pos.make_move_undoable(moves[0]);
        self.moves_executed += 1;
        let _eval = evaluate(&child_pos, Color::White);

        // Simulation/Playout phase: play out to depth limit
        const PLAYOUT_DEPTH_LIMIT: usize = 50;
        const PLAYOUT_MOVES_CONSIDERED: usize = 12;

        let mut playout_pos = child_pos.clone();
        self.position_clones += 1;
        let mut playout_color = Color::Black;
        let mut depth = 0;

        while depth < PLAYOUT_DEPTH_LIMIT {
            // Generate moves for playout
            self.legal_move_gens += 1;
            let playout_moves = generate_ordered_moves(&playout_pos, playout_color, PLAYOUT_MOVES_CONSIDERED);

            if playout_moves.is_empty() {
                break;
            }

            // Select move: evaluate each move
            for &mov in &playout_moves {
                let mut test_pos = playout_pos.clone();
                self.position_clones += 1;
                test_pos.make_move_undoable(mov);
                self.moves_executed += 1;
                self.evaluations += 1;
                let _eval = evaluate(&test_pos, playout_color);
            }

            // Make the best move
            playout_pos.make_move_undoable(playout_moves[0]);
            self.moves_executed += 1;
            playout_color = playout_color.opposite();
            depth += 1;
        }

        self.playout_depth_total += depth as u64;

        // Final evaluation
        self.evaluations += 1;
        let _final_eval = evaluate(&playout_pos, Color::White);
    }

    fn print_stats(&self, iterations: u64, elapsed_secs: f64) {
        println!("\n=== OPERATION COUNTS ===");
        println!("Position clones: {} ({:.0} per iteration)",
                 self.position_clones, self.position_clones as f64 / iterations as f64);
        println!("Moves executed: {} ({:.0} per iteration)",
                 self.moves_executed, self.moves_executed as f64 / iterations as f64);
        println!("Legal move generations: {} ({:.0} per iteration)",
                 self.legal_move_gens, self.legal_move_gens as f64 / iterations as f64);
        println!("Evaluations: {} ({:.0} per iteration)",
                 self.evaluations, self.evaluations as f64 / iterations as f64);
        println!("Average playout depth: {:.1}",
                 self.playout_depth_total as f64 / iterations as f64);

        println!("\n=== TIME ESTIMATES ===");
        // Use measured costs from profile_ai_detailed
        let clone_cost_us = 0.01;
        let legal_cost_us = 0.48;
        let eval_cost_us = 1.26;

        let clone_time = (self.position_clones as f64 * clone_cost_us) / iterations as f64;
        let legal_time = (self.legal_move_gens as f64 * legal_cost_us) / iterations as f64;
        let eval_time = (self.evaluations as f64 * eval_cost_us) / iterations as f64;

        let total_est = clone_time + legal_time + eval_time;
        let actual_per_iter = (elapsed_secs * 1_000_000.0) / iterations as f64;

        println!("Position clones: {:.0}µs ({:.1}%)",
                 clone_time, clone_time / total_est * 100.0);
        println!("Legal move generation: {:.0}µs ({:.1}%)",
                 legal_time, legal_time / total_est * 100.0);
        println!("Evaluations: {:.0}µs ({:.1}%)",
                 eval_time, eval_time / total_est * 100.0);
        println!("\nTotal estimated: {:.0}µs", total_est);
        println!("Actual per iteration: {:.0}µs", actual_per_iter);
        println!("Unaccounted overhead: {:.0}µs ({:.1}%)",
                 actual_per_iter - total_est,
                 (actual_per_iter - total_est) / actual_per_iter * 100.0);
    }
}

fn main() {
    let fen = "r1bqk2r/pp2bppp/2n1pn2/2pp4/3P4/2N1PN2/PPP1BPPP/R1BQK2R w KQkq -";
    let pos = Position::from_fen(fen);

    println!("Instrumented Profiling of MCTS Operations");
    println!("==========================================");
    println!("Position: {}", fen);
    println!("\nSimulating 100 MCTS iterations to count operations...\n");

    let mut stats = InstrumentedMCTS::new();
    let start = Instant::now();

    for _ in 0..100 {
        stats.run_iteration(&pos);
    }

    let elapsed = start.elapsed();
    println!("Time for 100 iterations: {:.2?}", elapsed);
    println!("Time per iteration: {:.2}ms", elapsed.as_secs_f64() * 1000.0 / 100.0);

    stats.print_stats(100, elapsed.as_secs_f64());

    println!("\n=== COMPONENT BREAKDOWN ===");
    println!("\nBased on operation counts, the major time sinks are:");
    println!("1. Evaluations - called ~{} times per iteration", stats.evaluations / 100);
    println!("2. Position clones - ~{} per iteration", stats.position_clones / 100);
    println!("3. Move executions - ~{} per iteration", stats.moves_executed / 100);
}
