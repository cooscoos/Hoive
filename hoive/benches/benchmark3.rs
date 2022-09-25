// cargo benchmark
// Which is more efficient for a small board - a beetle check or an ant check?
// Conclusion - beetle checks are 10x faster. Ant checks are worth it only if the ant is moving 10 spaces?
use criterion::{criterion_group, criterion_main, Criterion};

use hoive::game::animals;
use hoive::game::{board::Board, history};
use hoive::maths::coord::{Coord, Cube, DoubleHeight};

fn beetle_test_setup(filename: String) -> Board<Cube> {
    // Some set up used by beetle tests

    // Create and emulate a board from a named reference/tests/snapshots file
    let mut board = Board::new(Cube::default());
    history::emulate(&mut board, filename, true);
    board
}

fn criterion_benchmark(c: &mut Criterion) {
    let board = beetle_test_setup("snapshot_12".to_string());

    let source = board
        .coord
        .mapfrom_doubleheight(DoubleHeight::from((-1, 1)));
    let dest = board
        .coord
        .mapfrom_doubleheight(DoubleHeight::from((-1, -1)));

    c.bench_function("ant_check", |b| {
        b.iter(|| animals::ant_check(&board, &source, &dest))
    });

    c.bench_function("beetle_check", |b| {
        b.iter(|| animals::beetle_check(&board, &source, &dest))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
