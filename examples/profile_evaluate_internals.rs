// Profile the internals of the evaluate() function
// Run with: cargo run --release --example profile_evaluate_internals

use chess_engine::game_repr::{Color, Position};
use std::time::Instant;

// We can't directly call private functions, but we can recreate simplified versions
// or use black-box testing to estimate costs

fn main() {
    let fen = "r1bqk2r/pp2bppp/2n1pn2/2pp4/3P4/2N1PN2/PPP1BPPP/R1BQK2R w KQkq -";
    let pos = Position::from_fen(fen);

    println!("Profiling evaluate() Internals");
    println!("==============================\n");

    // Full evaluation
    use chess_engine::agent::ai::evaluation::evaluate;
    let start = Instant::now();
    for _ in 0..10000 {
        let _e = evaluate(&pos, Color::White);
    }
    let full_eval_time = start.elapsed();
    println!("Full evaluate(): {:.2}µs per call", full_eval_time.as_micros() as f64 / 10000.0);

    // Quick evaluation (material + PST only, no pawn structure/mobility/etc)
    use chess_engine::agent::ai::evaluation::quick_evaluate;
    let start = Instant::now();
    for _ in 0..10000 {
        let _e = quick_evaluate(&pos, Color::White);
    }
    let quick_eval_time = start.elapsed();
    println!("Quick evaluate(): {:.2}µs per call", quick_eval_time.as_micros() as f64 / 10000.0);

    let overhead = full_eval_time.as_micros() as f64 / 10000.0 - quick_eval_time.as_micros() as f64 / 10000.0;
    println!("\nPositional evaluation overhead: {:.2}µs ({:.1}%)",
             overhead, overhead / (full_eval_time.as_micros() as f64 / 10000.0) * 100.0);

    println!("\n=== Estimating positional evaluation components ===");
    println!("The overhead ({:.2}µs) includes:", overhead);
    println!("- King safety (pawn shield checking)");
    println!("- Pawn structure (doubled, isolated, passed pawns)");
    println!("- Piece mobility (pseudo-legal move counting)");
    println!("- Bishop pair bonus");
    println!("- Rook features (open files, 7th rank, connected rooks)");

    // We can estimate by looking at board scanning
    println!("\n=== Estimating board iteration cost ===");

    // Count pieces
    let start = Instant::now();
    for _ in 0..100000 {
        let mut count = 0;
        for square in 0..64 {
            let piece = pos.position[square];
            if !piece.is_none() {
                count += 1;
            }
        }
        std::hint::black_box(count);
    }
    let scan_time = start.elapsed();
    println!("Single board scan (64 squares): {:.3}µs", scan_time.as_micros() as f64 / 100000.0);

    // The positional evaluation does several board scans:
    // - King safety: 1 scan to find king + local area check
    // - Pawn structure: 1 scan to count pawns per file + check neighbors
    // - Mobility: 1 scan, checking each piece's moves
    // - Bishop pair: 1 scan to count bishops
    // - Rook features: 1 scan to find rooks + 1 scan to check files

    let estimated_scans = 6.0; // Conservative estimate
    let scan_cost = scan_time.as_micros() as f64 / 100000.0;
    println!("Estimated {} board scans: {:.3}µs", estimated_scans, scan_cost * estimated_scans);

    println!("\n=== Mobility calculation cost ===");
    // Mobility is expensive: for each piece, count pseudo-legal moves
    // Knights: 8 checks, Bishops/Rooks/Queens: sliding moves (avg ~8-10 per piece)

    // Simulate knight mobility calculation
    let start = Instant::now();
    for _ in 0..100000 {
        let from = 28; // e4
        let rank = (from / 8) as i32;
        let file = (from % 8) as i32;
        let knight_moves = [
            (rank + 2, file + 1), (rank + 2, file - 1),
            (rank - 2, file + 1), (rank - 2, file - 1),
            (rank + 1, file + 2), (rank + 1, file - 2),
            (rank - 1, file + 2), (rank - 1, file - 2),
        ];
        let mut count = 0;
        for (r, f) in knight_moves {
            if r >= 0 && r < 8 && f >= 0 && f < 8 {
                count += 1;
            }
        }
        std::hint::black_box(count);
    }
    let knight_mob_time = start.elapsed();
    println!("Knight mobility check: {:.3}µs", knight_mob_time.as_micros() as f64 / 100000.0);

    // Simulate rook mobility (sliding piece - more expensive)
    let start = Instant::now();
    for _ in 0..10000 {
        let from = 28;
        let rank = (from / 8) as i32;
        let file = (from % 8) as i32;
        let directions = [(1, 0), (-1, 0), (0, 1), (0, -1)];
        let mut count = 0;
        for (dr, df) in directions {
            let mut r = rank + dr;
            let mut f = file + df;
            while r >= 0 && r < 8 && f >= 0 && f < 8 {
                let to = (r * 8 + f) as usize;
                let target_piece = pos.position[to];
                if target_piece.is_none() {
                    count += 1;
                } else {
                    break;
                }
                r += dr;
                f += df;
            }
        }
        std::hint::black_box(count);
    }
    let rook_mob_time = start.elapsed();
    println!("Rook mobility check: {:.3}µs", rook_mob_time.as_micros() as f64 / 10000.0);

    // Estimate total mobility cost (4 knights, 4 bishops, 4 rooks, 2 queens at start)
    // In middlegame: ~2 knights, 2 bishops, 2 rooks, 1-2 queens per side
    let pieces_per_side = 7.0; // Rough estimate
    let avg_mob_cost = (knight_mob_time.as_micros() as f64 / 100000.0 + rook_mob_time.as_micros() as f64 / 10000.0) / 2.0;
    println!("Estimated mobility for {} pieces: {:.2}µs", pieces_per_side * 2.0, avg_mob_cost * pieces_per_side * 2.0);

    println!("\n=== SUMMARY ===");
    println!("evaluate() breakdown:");
    println!("  Material + PST: {:.2}µs ({:.1}%)",
             quick_eval_time.as_micros() as f64 / 10000.0,
             (quick_eval_time.as_micros() as f64 / 10000.0) / (full_eval_time.as_micros() as f64 / 10000.0) * 100.0);
    println!("  Positional features: {:.2}µs ({:.1}%)",
             overhead, overhead / (full_eval_time.as_micros() as f64 / 10000.0) * 100.0);
    println!("    - Board scanning: ~{:.2}µs", scan_cost * estimated_scans);
    println!("    - Mobility calculation: ~{:.2}µs (likely the biggest cost)", avg_mob_cost * pieces_per_side * 2.0);
    println!("    - Pawn structure: remaining {:.2}µs", overhead - scan_cost * estimated_scans - avg_mob_cost * pieces_per_side * 2.0);

    println!("\n=== OPTIMIZATION OPPORTUNITIES ===");
    println!("1. Mobility calculation is expensive (~{:.1}% of evaluate())",
             (avg_mob_cost * pieces_per_side * 2.0) / (full_eval_time.as_micros() as f64 / 10000.0) * 100.0);
    println!("   - Consider caching mobility or using simpler mobility metric");
    println!("   - Could use pre-computed attack tables like for move generation");
    println!("\n2. Board scanning could use bitboards for faster iteration");
    println!("   - Current: scan all 64 squares");
    println!("   - Better: use bitboards to iterate only occupied squares");
    println!("\n3. MCTS now handles opponent responses naturally through tree expansion");
    println!("   - No longer uses redundant 1-ply mini-search");
    println!("   - 2-ply evaluation happens through MCTS tree structure");
}
