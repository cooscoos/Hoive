// cargo benchmark
// https://bheisler.github.io/criterion.rs/book/getting_started.html
use criterion::{black_box, criterion_group, criterion_main, Criterion};


pub fn benchmarko() {
    println!("BOOYAH");
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("testymctestface", |b| b.iter(|| benchmarko()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);