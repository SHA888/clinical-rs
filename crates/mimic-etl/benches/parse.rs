//! Benchmarks for mimic-etl CSV parsing and Arrow conversion.
//!
//! Uses synthetic MIMIC-like CSV data (no real PHI) to measure
//! wall-time performance of the ETL pipeline.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use criterion::{Criterion, criterion_group, criterion_main};
use mimic_etl::{DatasetConfig, MimicCsvReader};
use std::hint::black_box;
use std::io::Write;
use tempfile::NamedTempFile;

/// Generate a synthetic `diagnoses_icd` CSV with `n` rows (MIMIC-IV style).
fn generate_diagnoses_csv(n: usize) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("create temp file");
    writeln!(file, "subject_id,hadm_id,icd_code,seq_num").expect("write header");
    for i in 0..n {
        writeln!(
            file,
            "{},{},A{:02}.{},{}",
            10_000 + i,
            20_000 + i,
            i % 100,
            i % 10,
            (i % 20) + 1,
        )
        .expect("write row");
    }
    file.flush().expect("flush");
    file
}

/// Generate a synthetic `DIAGNOSES_ICD` CSV with `n` rows (MIMIC-III UPPERCASE).
fn generate_mimic_iii_diagnoses_csv(n: usize) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("create temp file");
    writeln!(file, "SUBJECT_ID,HADM_ID,ICD_CODE,SEQ_NUM").expect("write header");
    for i in 0..n {
        writeln!(
            file,
            "{},{},{:03}.{},{}",
            10_000 + i,
            20_000 + i,
            i % 999,
            i % 10,
            (i % 20) + 1,
        )
        .expect("write row");
    }
    file.flush().expect("flush");
    file
}

fn bench_parse_diagnoses_1k(c: &mut Criterion) {
    let csv = generate_diagnoses_csv(1_000);
    let config = DatasetConfig::default();
    let reader = MimicCsvReader::new(config);

    let _ = c.bench_function("parse_diagnoses_icd_1k", |b| {
        b.iter(|| {
            let batches = reader.read_table("diagnoses_icd", black_box(csv.path()));
            let _ = black_box(batches).expect("parse diagnoses");
        });
    });
}

fn bench_parse_diagnoses_10k(c: &mut Criterion) {
    let csv = generate_diagnoses_csv(10_000);
    let config = DatasetConfig::default();
    let reader = MimicCsvReader::new(config);

    let _ = c.bench_function("parse_diagnoses_icd_10k", |b| {
        b.iter(|| {
            let batches = reader.read_table("diagnoses_icd", black_box(csv.path()));
            let _ = black_box(batches).expect("parse diagnoses");
        });
    });
}

fn bench_parse_mimic_iii_diagnoses_10k(c: &mut Criterion) {
    let csv = generate_mimic_iii_diagnoses_csv(10_000);
    let config = DatasetConfig::default();
    let reader = MimicCsvReader::new(config);

    let _ = c.bench_function("parse_mimic_iii_diagnoses_10k", |b| {
        b.iter(|| {
            let batches = reader.read_table("DIAGNOSES_ICD", black_box(csv.path()));
            let _ = black_box(batches).expect("parse MIMIC-III diagnoses");
        });
    });
}

criterion_group!(
    benches,
    bench_parse_diagnoses_1k,
    bench_parse_diagnoses_10k,
    bench_parse_mimic_iii_diagnoses_10k,
);
criterion_main!(benches);
