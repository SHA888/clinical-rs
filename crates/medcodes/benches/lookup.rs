//! Benchmark for medcodes lookup operations

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

fn bench_lookup(c: &mut Criterion) {
    // TODO: Implement benchmark
    let _ = c.bench_function("lookup", |b| b.iter(|| black_box(42)));
}

criterion_group!(benches, bench_lookup);
criterion_main!(benches);
