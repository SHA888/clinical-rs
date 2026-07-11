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

fn bench_loinc_traversal(c: &mut Criterion) {
    let loinc = Loinc::new();

    // "2160-0" (Creatinine SerPl-mCnc) is a leaf term with a 7-level ancestor
    // chain up to the hierarchy root — representative of a typical term.
    let _ = c.bench_function("loinc_parent", |b| {
        b.iter(|| black_box(loinc.parent(black_box("2160-0"))));
    });
    let _ = c.bench_function("loinc_ancestors", |b| {
        b.iter(|| black_box(loinc.ancestors(black_box("2160-0"))));
    });

    // "LP385359-7" is the Part grouping the creatinine panel (18 children) —
    // a small, non-root subtree, avoiding a full hierarchy walk in the bench.
    let _ = c.bench_function("loinc_children", |b| {
        b.iter(|| black_box(loinc.children(black_box("LP385359-7"))));
    });
    let _ = c.bench_function("loinc_descendants", |b| {
        b.iter(|| black_box(loinc.descendants(black_box("LP385359-7"))));
    });
}

criterion_group!(benches, bench_loinc_lookup, bench_loinc_traversal);
criterion_main!(benches);
