//! Benchmark for clinical-tasks windowing operations

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

fn bench_windowing(c: &mut Criterion) {
    // TODO: Implement benchmark
    let _ = c.bench_function("windowing", |b| b.iter(|| black_box(42)));
}

criterion_group!(benches, bench_windowing);
criterion_main!(benches);
