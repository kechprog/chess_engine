use chess_engine::game_repr::Position;
use std::time::Instant;

fn main() {
    let pos = Position::default();

    println!("Running perft depth 6 on starting position...");
    let start = Instant::now();
    let result = pos.perft(6);
    let duration = start.elapsed();

    println!("Result: {} nodes", result);
    println!("Time: {:.2}s", duration.as_secs_f64());
    println!("Nodes/sec: {:.0}", result as f64 / duration.as_secs_f64());
}
