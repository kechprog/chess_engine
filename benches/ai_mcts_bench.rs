use std::time::Duration;

use chess_engine::agent::ai::mcts::{search_multithreaded, MCTSTree};
use chess_engine::game_repr::{Color, Position};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

const MIDDLE_GAME_FEN: &str =
    "r1bqk2r/pp2bppp/2n1pn2/2pp4/3P4/2N1PN2/PPP1BPPP/R1BQK2R w KQkq -";
const ITERATIONS: u32 = 5_000;

fn bench_mcts_iterations(c: &mut Criterion) {
    let mut group = c.benchmark_group("mcts_5000_iterations");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(15));

    let pos = Position::from_fen(MIDDLE_GAME_FEN);
    let color = Color::White;

    group.bench_function("single_threaded", |b| {
        b.iter(|| {
            let mut tree = MCTSTree::new(&pos, color);
            black_box(tree.search(&pos, ITERATIONS));
        });
    });

    group.bench_function("multithreaded_auto", |b| {
        b.iter(|| {
            black_box(search_multithreaded(&pos, color, ITERATIONS, None));
        });
    });

    group.finish();
}

criterion_group!(benches, bench_mcts_iterations);
criterion_main!(benches);