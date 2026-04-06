<div align="center">

# clinical-rs

**Composable Rust crates for clinical data engineering.**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE-MIT)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE-APACHE)
[![Rust](https://img.shields.io/badge/rust-1.94%2B-orange.svg)](https://www.rust-lang.org)

[Architecture](ARCHITECTURE.md) · [Roadmap](TODO.md) · [Contributing](CONTRIBUTING.md)

</div>

---

## What is this?

`clinical-rs` is a Cargo workspace containing three independent crates for working with clinical healthcare data in Rust:

| Crate | Purpose | Status |
|-------|---------|--------|
| [`medcodes`](crates/medcodes) | Medical code ontologies, hierarchy traversal, and cross-system mapping (ICD-10, ATC, LOINC, SNOMED CT, etc.) | 🚧 Pre-release |
| [`mimic-etl`](crates/mimic-etl) | MIMIC-III/IV CSV parser → Apache Arrow RecordBatches with memory-mapped I/O and parallel processing | 🚧 Pre-release |
| [`clinical-tasks`](crates/clinical-tasks) | Task windowing engine — transforms clinical event streams into ML-ready (features, label) Arrow tables; includes post-critical-illness longevity signal module (feature-gated: `longevity`) | 🚧 Pre-release |

Each crate publishes independently to [crates.io](https://crates.io) and can be used standalone. Together, they form an end-to-end pipeline from raw clinical data to model-ready datasets.

## Why?

Clinical ML data pipelines are bottlenecked by data loading, not model training. Python-based tools like [PyHealth](https://pyhealth.dev/) and pandas struggle with memory pressure and parallelism on large datasets like MIMIC-IV (300K+ patients, tens of millions of events).

`clinical-rs` targets that bottleneck:

- **Arrow-native** — every crate speaks Apache Arrow as its interchange format. Zero-copy interop with PyArrow, Polars, DataFusion, DuckDB, and Spark.
- **Streaming-first** — all ETL crates emit `RecordBatch` iterators, not materialized collections. Same code path works for batch (collect → Parquet) and streaming (process → infer → emit).
- **Parallel by default** — `rayon`-based work-stealing parallelism without Python's GIL. Memory-mapped I/O via `memmap2` for datasets larger than RAM.
- **Composable, not monolithic** — use `medcodes` alone for code lookups, `mimic-etl` alone for data loading, or wire them together through `clinical-tasks`.

## Quick Start

Add the crate(s) you need:

```toml
# Cargo.toml
[dependencies]
medcodes = "0.1"         # medical code ontologies
mimic-etl = "0.1"        # MIMIC-III/IV → Arrow
clinical-tasks = "0.1"   # task windowing for ML

# Enable post-critical-illness longevity signal module
clinical-tasks = { version = "0.1", features = ["longevity"] }
```

### Medical code lookup

```rust
use medcodes::icd10cm::Icd10Cm;

let code = Icd10Cm::lookup("A41.9")?;       // Sepsis, unspecified organism
let ancestors = code.ancestors();             // ["A41", "A30-A49", "A00-B99"]
let description = code.description();         // "Sepsis, unspecified organism"
```

### Cross-system mapping

```rust
use medcodes::crossmap::CrossMap;

let icd_to_ccs = CrossMap::load(System::Icd10Cm, System::CcsCm)?;
let mapped = icd_to_ccs.map("A41.9")?;      // ["2"]  (CCS category: Septicemia)
```

### MIMIC-IV to Arrow

```rust
use mimic_etl::Mimic4Dataset;

let dataset = Mimic4Dataset::open("path/to/mimic-iv/")?;
let batches = dataset
    .tables(&["diagnoses_icd", "prescriptions", "labevents"])
    .into_event_stream()?;  // Iterator<Item = RecordBatch>

// Write to Parquet
mimic_etl::to_parquet(batches, "output/events.parquet")?;
```

### Task windowing

```rust
use clinical_tasks::{MortalityPrediction, TaskConfig};
use arrow::ipc::reader::FileReader;

let events = FileReader::try_new(File::open("events.arrow")?)?;
let task = MortalityPrediction::new(TaskConfig {
    observation_window: Duration::hours(48),
    prediction_window: Duration::hours(24),
    ..Default::default()
});

let samples = task.apply(events)?;  // Iterator<Item = RecordBatch> with features + label columns
```

### Longevity signals (post-critical-illness, feature-gated)

Requires `features = ["longevity"]`. Scoped to post-critical-illness biological age
acceleration — not a general longevity platform. `CalibrationStatus` is first-class:
it propagates into downstream hypothesis confidence scoring. All clock-derived fields
carry `CalibrationStatus::Uncalibrated` until a Southeast Asian cohort recalibration
is completed.

```rust
use clinical_tasks::longevity::{
    LongevitySignals, BiologicalAgeDelta, CalibrationStatus,
    SaspComposite, FunctionalTrajectory,
};

// Construct from post-ICU follow-up measurements
let signals = LongevitySignals {
    biological_age_delta: Some(BiologicalAgeDelta {
        value: 8.3,                              // GrimAge - chronological age (years)
        clock_version: ClockVersion::GrimAge2,
        calibration_status: CalibrationStatus::Uncalibrated,  // SEA population, no local calibration
    }),
    il6_pgml:  Some(14.2),
    il8_pgml:  Some(22.7),
    gdf15_pgml: Some(1840.0),
    mmp3_ngml: Some(6.1),
    p16_relative_expression: Some(2.4),
    sasp_composite_score: None,   // computed downstream
    post_icu_functional_trajectory: Some(FunctionalTrajectory::Pics),
};
```

## End-to-End Example

Here's a complete pipeline from MIMIC-IV CSV to a mortality prediction dataset:

```rust
use std::sync::Arc;
use arrow::record_batch::RecordBatch;
use clinical_tasks::{MortalityPrediction, TaskWindows, AnchorPoint, split_by_patient, outputs_to_batch};
use mimic_etl::{MimicCsvReader, DatasetConfig};
use medcodes::icd10cm::Icd10Cm;

// 1. Load MIMIC-IV data
let config = DatasetConfig {
    root_path: "path/to/mimic-iv/".to_string(),
    tables: vec![
        "admissions".to_string(),
        "patients".to_string(),
        "icustays".to_string(),
        "diagnoses_icd".to_string(),
        "labevents".to_string(),
    ],
    batch_size: 10000,
};

let reader = MimicCsvReader::new(config);
let mut all_events = Vec::new();

// 2. Convert each table to Arrow RecordBatches
for table in &reader.config.tables {
    let path = format!("{}/{}.csv", reader.config.root_path, table);
    let batches = reader.read_table(table, path)?;
    all_events.extend(batches);
}

// 3. Combine all events into a single stream
let combined_batch = // ... combine batches from all tables ...

// 4. Configure the mortality prediction task
let windows = TaskWindows::new(
    48.0,    // 48-hour observation window
    0.0,     // No gap between observation and prediction
    24.0,    // 24-hour prediction window
    AnchorPoint::Admission,
);

let task = MortalityPrediction::new(windows);

// 5. Process patients through the task
let outputs = task.process_batch(&combined_batch)?;

// 6. Split into train/validation/test sets (patient-level)
let split_config = SplitConfig {
    train_ratio: 0.7,
    val_ratio: 0.15,
    test_ratio: 0.15,
    seed: 42,
};

let (train, val, test) = split_by_patient(&outputs, &split_config)?;

// 7. Convert to Arrow format for ML
let schema = task.output_schema();
let train_batch = outputs_to_batch(&train, &schema)?;
let val_batch = outputs_to_batch(&val, &schema)?;
let test_batch = outputs_to_batch(&test, &schema)?;

// 8. Save to Parquet
use parquet::arrow::ArrowWriter;
use std::fs::File;

let train_file = File::create("train.parquet")?;
let mut train_writer = ArrowWriter::try_new(train_file, schema.as_ref(), None)?;
train_writer.write(&train_batch)?;
train_writer.close()?;

println!("Created ML-ready dataset:");
println!("- Train: {} patients", train.len());
println!("- Validation: {} patients", val.len());
println!("- Test: {} patients", test.len());
println!("- Features: {}", schema.fields.len() - 1); // -1 for label
```

### Output Schema

The mortality prediction task generates the following Arrow schema:

```rust
Schema {
    fields: [
        Field("patient_id", DataType::Int64, false),      // Patient identifier
        Field("admission_id", DataType::Int64, true),      // Hospital admission
        Field("age", DataType::Float64, true),             // Age at admission
        Field("gender_male", DataType::Float64, false),    // Gender (1=male, 0=female)
        Field("num_diagnoses", DataType::Float64, false),  // Count of diagnoses
        Field("num_procedures", DataType::Float64, false), // Count of procedures
        Field("num_medications", DataType::Float64, false), // Count of medications
        Field("num_labs", DataType::Float64, false),       // Count of lab measurements
        Field("abnormal_labs_ratio", DataType::Float64, false), // Ratio of abnormal labs
        Field("label", DataType::Float64, false),          // 1=died, 0=survived
    ]
}
```

This dataset is ready for training with any ML framework that supports Arrow (PyTorch, TensorFlow, XGBoost, etc.).

## Design Principles

1. **Arrow is the contract.** Crates communicate via Arrow RecordBatches. No custom serialization formats, no framework lock-in.
2. **Each crate stands alone.** `medcodes` has zero dependencies on `mimic-etl`. A consumer building a FHIR pipeline can use `medcodes` + `clinical-tasks` without ever touching MIMIC data.
3. **Correctness over cleverness.** Medical code mappings are validated against official source files (CMS, WHO, NLM). Wrong mappings in clinical contexts cause harm.
4. **No model training.** This project handles everything *before* and *after* the GPU. Train models in PyTorch/JAX, export to ONNX, run inference in Rust via the `ort` crate.

## Project Structure

```
clinical-rs/
├── crates/
│   ├── medcodes/             # Medical code ontologies + cross-mapping
│   │   ├── src/
│   │   ├── data/             # Embedded code tables (build-time)
│   │   └── Cargo.toml
│   ├── mimic-etl/            # MIMIC-III/IV → Arrow ETL
│   │   ├── src/
│   │   └── Cargo.toml
│   └── clinical-tasks/       # Task windowing engine
│       ├── src/
│       │   └── longevity/    # Post-critical-illness longevity signals (feature-gated)
│       └── Cargo.toml
├── ARCHITECTURE.md
├── TODO.md
├── CONTRIBUTING.md
├── LICENSE-MIT
├── LICENSE-APACHE
└── Cargo.toml                # Workspace manifest
```

## Relationship to Existing Tools

| Tool | Language | Focus | How clinical-rs differs |
|------|----------|-------|------------------------|
| [PyHealth](https://pyhealth.dev/) | Python | End-to-end clinical ML toolkit (data + models + training) | We do data only — faster, Arrow-native, no model training |
| [MedModels](https://github.com/limebit/medmodels) | Rust + Python | Graph-based RWE analysis (treatment effects, propensity matching) | We use columnar/Arrow, not graph. ML data loading, not RWE analytics |
| [MEDS](https://github.com/Medical-Event-Data-Standard) | Python | Medical event data standard | Complementary — we could emit MEDS-compatible schemas |

## Requirements

- Rust 1.94+ (2024 edition)
- MIMIC-III/IV access via [PhysioNet](https://physionet.org/) credentialed access (for `mimic-etl`)

## License

Dual-licensed under [MIT](LICENSE-MIT) and [Apache 2.0](LICENSE-APACHE), at your option.

## Citation

If you use `clinical-rs` in academic work, please cite:

```bibtex
@software{clinical_rs,
  author       = {Kresna Sucandra},
  title        = {clinical-rs: Composable Rust crates for clinical data engineering},
  url          = {https://github.com/SHA888/clinical-rs},
  license      = {MIT OR Apache-2.0},
}
```
