use criterion::{black_box, criterion_group, criterion_main, Criterion};
use chess_engine::game_repr::Position;

fn bench_perft_depth_6(c: &mut Criterion) {
    let pos = Position::default();
    c.bench_function("perft depth 6", |b| {
        b.iter(|| black_box(pos.perft(6)))
    });
}

fn bench_perft_depth_5(c: &mut Criterion) {
    let pos = Position::default();
    c.bench_function("perft depth 5", |b| {
        b.iter(|| black_box(pos.perft(5)))
    });
}

criterion_group!(benches, bench_perft_depth_5, bench_perft_depth_6);
criterion_main!(benches);
