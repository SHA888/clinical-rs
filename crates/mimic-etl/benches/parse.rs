//! Benchmark for mimic-etl parse operations

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

fn bench_parse(c: &mut Criterion) {
    // TODO: Implement benchmark
    let _ = c.bench_function("parse", |b| b.iter(|| black_box(42)));
}

criterion_group!(benches, bench_parse);
criterion_main!(benches);
