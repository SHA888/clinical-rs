//! Benchmark for medcodes lookup, traversal, and cross-mapping operations

use criterion::{Criterion, criterion_group, criterion_main};
use medcodes::{Atc, CodeSystem, CrossMap, Icd10Cm, Icd10CmToCcs};
use std::hint::black_box;

fn bench_icd10cm_lookup(c: &mut Criterion) {
    let icd10 = Icd10Cm::new();

    // Benchmark successful lookup
    let _ = c.bench_function("icd10cm_lookup_success", |b| {
        b.iter(|| black_box(icd10.lookup(black_box("A00.0"))));
    });

    // Benchmark failed lookup
    let _ = c.bench_function("icd10cm_lookup_not_found", |b| {
        b.iter(|| black_box(icd10.lookup(black_box("X99.9"))));
    });

    // Benchmark is_valid
    let _ = c.bench_function("icd10cm_is_valid", |b| {
        b.iter(|| black_box(icd10.is_valid(black_box("A00.0"))));
    });
}

fn bench_icd10cm_traversal(c: &mut Criterion) {
    let icd10 = Icd10Cm::new();

    // Benchmark parent lookup
    let _ = c.bench_function("icd10cm_parent", |b| {
        b.iter(|| black_box(icd10.parent(black_box("A00.0"))));
    });

    // Benchmark children lookup
    let _ = c.bench_function("icd10cm_children", |b| {
        b.iter(|| black_box(icd10.children(black_box("A00"))));
    });

    // Benchmark ancestors lookup
    let _ = c.bench_function("icd10cm_ancestors", |b| {
        b.iter(|| black_box(icd10.ancestors(black_box("A00.0"))));
    });

    // Benchmark descendants lookup
    let _ = c.bench_function("icd10cm_descendants", |b| {
        b.iter(|| black_box(icd10.descendants(black_box("A00"))));
    });
}

fn bench_atc_lookup(c: &mut Criterion) {
    let atc = Atc::new();

    // Benchmark successful lookup
    let _ = c.bench_function("atc_lookup_success", |b| {
        b.iter(|| black_box(atc.lookup(black_box("C10AA01"))));
    });

    // Benchmark is_valid
    let _ = c.bench_function("atc_is_valid", |b| {
        b.iter(|| black_box(atc.is_valid(black_box("C10AA01"))));
    });
}

fn bench_atc_traversal(c: &mut Criterion) {
    let atc = Atc::new();

    // Benchmark parent lookup
    let _ = c.bench_function("atc_parent", |b| {
        b.iter(|| black_box(atc.parent(black_box("C10AA01"))));
    });

    // Benchmark children lookup
    let _ = c.bench_function("atc_children", |b| {
        b.iter(|| black_box(atc.children(black_box("C10AA"))));
    });
}

fn bench_cross_mapping(c: &mut Criterion) {
    let mapper = Icd10CmToCcs::new();

    // Benchmark successful mapping
    let _ = c.bench_function("icd10cm_to_ccs_map", |b| {
        b.iter(|| black_box(mapper.map(black_box("A00.0"), black_box(medcodes::System::Ccs))));
    });
}

criterion_group!(
    benches,
    bench_icd10cm_lookup,
    bench_icd10cm_traversal,
    bench_atc_lookup,
    bench_atc_traversal,
    bench_cross_mapping
);
criterion_main!(benches);
