use chess_engine::game_repr::Position;
use std::time::Instant;

fn main() {
    let pos = Position::default();

    // Measure move generation
    let start = Instant::now();
    let mut total_moves = 0;
    for _ in 0..100000 {
        let moves = pos.all_legal_moves();
        total_moves += moves.len();
    }
    let move_gen_time = start.elapsed();

    // Measure clone
    let start = Instant::now();
    for _ in 0..1000000 {
        let _ = pos.clone();
    }
    let clone_time = start.elapsed();

    // Measure mk_move
    let moves = pos.all_legal_moves();
    let start = Instant::now();
    for _ in 0..100000 {
        for mv in &moves {
            let mut new_pos = pos.clone();
            new_pos.mk_move(*mv);
        }
    }
    let mk_move_time = start.elapsed();

    println!("Move generation (100k iterations): {:.2}ms", move_gen_time.as_secs_f64() * 1000.0);
    println!("Clone (1M iterations): {:.2}ms", clone_time.as_secs_f64() * 1000.0);
    println!("mk_move + clone (100k * 20 moves): {:.2}ms", mk_move_time.as_secs_f64() * 1000.0);

    println!("\nPer operation:");
    println!("Move generation: {:.2}ns", move_gen_time.as_nanos() as f64 / 100000.0);
    println!("Clone: {:.2}ns", clone_time.as_nanos() as f64 / 1000000.0);
    println!("mk_move: {:.2}ns", mk_move_time.as_nanos() as f64 / (100000.0 * 20.0));
}
