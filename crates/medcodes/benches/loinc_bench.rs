#![allow(missing_docs)]

use criterion::{Criterion, criterion_group, criterion_main};
use medcodes::{CodeSystem, Loinc};
use std::hint::black_box;

fn bench_loinc_lookup(c: &mut Criterion) {
    let loinc = Loinc::new();
    let _ = c.bench_function("loinc_lookup_success", |b| {
        b.iter(|| {
            let _ = black_box(loinc.lookup(black_box("2160-0")));
        });
    });
}

criterion_group!(benches, bench_loinc_lookup);
criterion_main!(benches);
