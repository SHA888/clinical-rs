# Roadmap

Comprehensive checklist organized by crate and [SemVer](https://semver.org/) release milestone. Each item is a concrete deliverable.

> **Versioning policy:** All three crates version independently. A breaking change in `medcodes` does not force a major bump in `mimic-etl`. Pre-1.0 releases (0.x.y) may contain breaking changes in minor versions per SemVer §4.

---

## Table of Contents

- [medcodes](#medcodes)
- [mimic-etl](#mimic-etl)
- [clinical-tasks](#clinical-tasks)
- [Workspace / Infrastructure](#workspace--infrastructure)
- [Future Crates (Post-1.0)](#future-crates-post-10)

---

## `medcodes`

### v0.1.0 — Foundation

Core ontology engine with ICD-10-CM and the first cross-mapping. Minimum viable crate for crates.io publish.

- [ ] **Project scaffolding**
  - [ ] `Cargo.toml` with metadata (description, license, repository, keywords, categories)
  - [ ] `lib.rs` with public module structure
  - [ ] CI: `cargo test`, `cargo clippy`, `cargo fmt --check`
- [ ] **Core types**
  - [ ] `System` enum (ICD9CM, ICD10CM, ICD10PCS, ATC, NDC, LOINC, SNOMED, RxNorm, CCS, CCSR, CPT)
  - [ ] `Code` struct (system, code, description)
  - [ ] `CodeSystem` trait (lookup, ancestors, descendants, is_valid, normalize)
  - [ ] `CrossMap` trait (map, source_system, target_system)
  - [ ] `Error` type via `thiserror`
- [ ] **ICD-10-CM implementation**
  - [ ] Download and process CMS FY2025 ICD-10-CM code table
  - [ ] `build.rs` pipeline: TSV source data → `phf::Map` source generation
  - [ ] `Icd10Cm::lookup(code) → Result<Code>`
  - [ ] `Icd10Cm::is_valid(code) → bool`
  - [ ] `Icd10Cm::normalize(code) → String` (strip dots, uppercase)
  - [ ] Hierarchy traversal: `ancestors()`, `descendants()`, `parent()`, `children()`
  - [ ] Feature flag: `icd10cm` (enabled by default)
- [ ] **CCS/CCSR cross-mapping**
  - [ ] Download and process AHRQ CCSR v2024.1 mapping files
  - [ ] `CrossMap::icd10cm_to_ccsr() → impl CrossMap`
  - [ ] Bidirectional lookup support
- [ ] **Tests**
  - [ ] Unit tests for every public method
  - [ ] Known-answer tests against CMS reference data (≥50 code lookups verified)
  - [ ] Property tests: `normalize(code)` is idempotent, `is_valid(normalize(code))` holds
- [ ] **Documentation**
  - [ ] Rustdoc for all public types and methods
  - [ ] `README.md` with usage examples
  - [ ] `CHANGELOG.md` initialized

### v0.2.0 — Code System Expansion

- [ ] **ICD-9-CM** (frozen Oct 2015 release)
  - [ ] Code table processing and embedding
  - [ ] Full `CodeSystem` trait implementation
  - [ ] Feature flag: `icd9cm`
- [ ] **ATC** (WHO Collaborating Centre)
  - [ ] 5-level hierarchy (anatomical → chemical substance)
  - [ ] DDD (Defined Daily Dose) as optional metadata
  - [ ] Feature flag: `atc`
- [ ] **NDC** (FDA National Drug Code Directory)
  - [ ] Labeler-product-package structure
  - [ ] Feature flag: `ndc`
- [ ] **ICD-10-CM → CCS (single-level) mapping**
- [ ] **ICD-9-CM → CCS mapping**
- [ ] **NDC → ATC cross-mapping**
- [ ] **NDC → RxNorm cross-mapping**
- [ ] **Serde support**
  - [ ] `Serialize`/`Deserialize` for `Code`, `System`
  - [ ] Feature flag: `serde`
- [ ] **Benchmark suite**
  - [ ] `criterion` benchmarks for lookup, traversal, and cross-mapping
  - [ ] Baseline numbers documented in README

### v0.3.0 — Clinical Terminologies

- [ ] **LOINC** (laboratory/clinical observations)
  - [ ] Code lookup + hierarchy (class, system, component)
  - [ ] Feature flag: `loinc`
- [ ] **SNOMED CT** (US edition)
  - [ ] Concept lookup + IS-A hierarchy traversal
  - [ ] Feature flag: `snomed`
  - [ ] Note: SNOMED requires NLM UMLS license acknowledgment in docs
- [ ] **RxNorm**
  - [ ] Concept lookup (ingredient, brand, clinical drug)
  - [ ] RxNorm → ATC mapping
  - [ ] Feature flag: `rxnorm`
- [ ] **CPT** (Current Procedural Terminology)
  - [ ] Category I codes + hierarchy
  - [ ] Feature flag: `cpt`
  - [ ] Note: CPT is AMA-licensed — evaluate embedding feasibility
- [ ] **ICD-10-PCS**
  - [ ] 7-character code structure parsing
  - [ ] Feature flag: `icd10pcs`
- [ ] **Multi-version support**
  - [ ] API for selecting code table version at build time
  - [ ] e.g., `features = ["icd10cm-fy2025"]` vs `features = ["icd10cm-fy2024"]`

### v1.0.0 — Stable API

- [ ] API review: all public types finalized, no planned breaking changes
- [ ] Migration guide from 0.x
- [ ] All code systems individually feature-gated
- [ ] Default features: ICD-10-CM + ATC + LOINC
- [ ] MSRV policy documented
- [ ] Published to crates.io with stable version

---

## `mimic-etl`

### v0.1.0 — MIMIC-IV Core Tables

Parse the most commonly used MIMIC-IV (v2.x) tables into Arrow. Enough to run mortality and readmission tasks.

- [ ] **Project scaffolding**
  - [ ] `Cargo.toml` with metadata
  - [ ] `lib.rs` with public module structure
  - [ ] CI integrated into workspace pipeline
- [ ] **Core ETL types**
  - [ ] `ClinicalEvent` Arrow schema definition (shared constant)
  - [ ] `DatasetConfig` struct (root path, table selection, batch size, parallelism)
  - [ ] `Error` type
- [ ] **MIMIC-IV hosp module**
  - [ ] `admissions.csv` parser → ClinicalEvent batches
  - [ ] `patients.csv` parser → patient demographics Arrow table
  - [ ] `diagnoses_icd.csv` parser
  - [ ] `procedures_icd.csv` parser
  - [ ] `prescriptions.csv` parser
  - [ ] `labevents.csv` parser
  - [ ] `microbiologyevents.csv` parser
  - [ ] `transfers.csv` parser
- [ ] **MIMIC-IV icu module**
  - [ ] `icustays.csv` parser
  - [ ] `chartevents.csv` parser (streaming — ~330M rows)
  - [ ] `inputevents.csv` parser
  - [ ] `outputevents.csv` parser
  - [ ] `procedureevents.csv` parser
- [ ] **Parallel processing**
  - [ ] `rayon`-based parallel table parsing (one thread per table)
  - [ ] Chunk-level parallelism within large tables (chartevents, labevents)
- [ ] **Memory-mapped I/O**
  - [ ] `memmap2` for all CSV reads
  - [ ] Configurable batch size (default 128K rows per RecordBatch)
- [ ] **Output formats**
  - [ ] `to_parquet(batches, path)` — write RecordBatch stream to Parquet
  - [ ] `to_arrow_ipc(batches, path)` — write to Arrow IPC file
  - [ ] Streaming: `into_event_stream()` → `Iterator<Item = Result<RecordBatch>>`
- [ ] **Tests**
  - [ ] Unit tests with synthetic MIMIC-like CSV fixtures (no real PHI in repo)
  - [ ] Schema validation: output matches `ClinicalEvent` schema
  - [ ] Round-trip: CSV → Arrow → Parquet → read back, verify row counts
- [ ] **Documentation**
  - [ ] Rustdoc for all public types
  - [ ] `README.md` with usage examples
  - [ ] Supported tables matrix

### v0.2.0 — MIMIC-III + Performance

- [ ] **MIMIC-III (v1.4) support**
  - [ ] `ADMISSIONS.csv`, `PATIENTS.csv` parsers
  - [ ] `DIAGNOSES_ICD.csv`, `PROCEDURES_ICD.csv` parsers
  - [ ] `PRESCRIPTIONS.csv`, `LABEVENTS.csv` parsers
  - [ ] `CHARTEVENTS.csv` parser (streaming)
  - [ ] `ICUSTAYS.csv` parser
  - [ ] Column name normalization (MIMIC-III UPPER → lowercase)
- [ ] **Optional `medcodes` integration**
  - [ ] Feature flag: `medcodes`
  - [ ] Code normalization during parsing
  - [ ] Automatic ICD-9 vs ICD-10 detection based on MIMIC version
- [ ] **Performance benchmarks**
  - [ ] `criterion` suite on MIMIC-IV demo dataset (public, no credential needed)
  - [ ] Wall time and peak memory comparison methodology documented
  - [ ] Benchmark results in README
- [ ] **CLI tool**
  - [ ] `mimic-etl convert --input <path> --output <path> --format parquet`
  - [ ] `mimic-etl schema` — print ClinicalEvent Arrow schema
  - [ ] `mimic-etl info --input <path>` — table row counts, size on disk
  - [ ] Feature flag: `cli`

### v0.3.0 — MIMIC-IV v3.x + Notes

- [ ] **MIMIC-IV v3.0 schema changes**
  - [ ] Version auto-detection
  - [ ] Handle renamed/restructured tables
- [ ] **MIMIC-IV-Note**
  - [ ] `discharge.csv` (discharge summaries)
  - [ ] `radiology.csv` (radiology reports)
- [ ] **MIMIC-IV-ED**
  - [ ] `edstays.csv`, `triage.csv`, `vitalsign.csv` parsers
- [ ] **Incremental parsing**
  - [ ] Checkpoint file for long-running parses
  - [ ] Resume from last completed table on failure

### v1.0.0 — Stable API

- [ ] `ClinicalEvent` schema finalized
- [ ] MIMIC-III v1.4 and MIMIC-IV v2.x fully supported
- [ ] Migration guide from 0.x
- [ ] MSRV policy documented
- [ ] Published to crates.io with stable version

---

## `clinical-tasks`

### v0.1.0 — Core Task Engine + Mortality

Minimal task windowing engine with one fully implemented task.

- [ ] **Project scaffolding**
  - [ ] `Cargo.toml` with metadata
  - [ ] `lib.rs` with public module structure
  - [ ] CI integrated into workspace pipeline
- [ ] **Core types**
  - [ ] `TaskDefinition` trait
  - [ ] `TaskWindows` struct (observation, gap, prediction, anchor)
  - [ ] `AnchorPoint` enum (Admission, Discharge, ICUAdmission, ICUDischarge, Custom)
  - [ ] `TaskRunner` — applies a `TaskDefinition` to a RecordBatch stream
  - [ ] `Error` type
- [ ] **Patient grouping**
  - [ ] Group RecordBatch stream by `(patient_id, visit_id)`
  - [ ] Sort events by timestamp within each group
  - [ ] Handle multi-visit patients
- [ ] **In-hospital mortality prediction**
  - [ ] `MortalityPrediction` implementing `TaskDefinition`
  - [ ] Configurable observation window (default: 48h from admission)
  - [ ] Label: binary (died during hospitalization)
  - [ ] Features: code frequency vectors, lab value statistics (min/max/mean/last)
  - [ ] Exclusion: stays < observation window, patients < 18
- [ ] **Patient-level splitting**
  - [ ] `split_by_patient(batches, ratios)` → (train, val, test) streams
  - [ ] Deterministic split via patient_id hashing (reproducible)
  - [ ] No patient appears in multiple splits
- [ ] **Output**
  - [ ] RecordBatch with feature columns + label column
  - [ ] Export to Parquet / Arrow IPC
  - [ ] Schema metadata (task name, window parameters, split assignment)
- [ ] **Tests**
  - [ ] Synthetic patient timeline tests
  - [ ] Data leakage verification: no prediction-window events in features
  - [ ] Patient-level split verification
  - [ ] Known-answer test: hand-computed labels for 5 synthetic patients
- [ ] **Documentation**
  - [ ] Rustdoc for all public types
  - [ ] `README.md` with end-to-end example

### v0.2.0 — Task Expansion

- [ ] **30-day readmission prediction**
  - [ ] Label: binary (readmitted within 30d of discharge)
  - [ ] Anchor: discharge time
  - [ ] Exclusion: in-hospital deaths, transfers
- [ ] **Length of stay prediction**
  - [ ] Multiclass: bucketed (0-1d, 1-3d, 3-7d, 7-14d, 14d+)
  - [ ] Regression variant: continuous LOS in hours
- [ ] **Drug recommendation**
  - [ ] Multi-label: medication set per visit
  - [ ] Features: diagnosis + procedure codes from current + prior visits
  - [ ] Optional DDI matrix integration
- [ ] **Optional `medcodes` integration**
  - [ ] Feature flag: `medcodes`
  - [ ] Code grouping (ICD-10 → CCS categories) to reduce feature dimensionality
- [ ] **Custom task API**
  - [ ] Documentation + example for implementing `TaskDefinition`
  - [ ] Builder pattern for common task patterns

### v0.3.0 — Sepsis + Advanced Windowing

- [ ] **Sepsis onset prediction**
  - [ ] Configurable Sepsis-3 criteria (suspected infection + SOFA ≥ 2)
  - [ ] Variable lookback windows (1h, 3h, 6h, 12h before onset)
  - [ ] Negative sampling strategy for non-sepsis controls
  - [ ] Temporal advantage metric
- [ ] **Sliding window mode**
  - [ ] Samples at regular intervals (e.g., every 1h) during a stay
  - [ ] Useful for real-time prediction simulation
- [ ] **Feature engineering utilities**
  - [ ] Temporal binning (sub-intervals within observation window)
  - [ ] Code co-occurrence features
  - [ ] Lab trend features (slope, acceleration)
  - [ ] Missing data indicators

### v1.0.0 — Stable API

- [ ] `TaskDefinition` trait finalized
- [ ] All v0.x tasks stable
- [ ] Migration guide from 0.x
- [ ] MSRV policy documented
- [ ] Published to crates.io with stable version

---

## Workspace / Infrastructure

### Initial Setup

- [ ] **Workspace manifest**
  - [ ] Root `Cargo.toml` with `[workspace]` members
  - [ ] Shared `rust-version`, `edition`, `license` in `[workspace.package]`
  - [ ] Shared dependency versions in `[workspace.dependencies]`
- [ ] **CI / CD (GitHub Actions)**
  - [ ] Test all crates on Linux (ubuntu-latest)
  - [ ] `cargo clippy -- -D warnings`
  - [ ] `cargo fmt --all -- --check`
  - [ ] `cargo doc --no-deps`
  - [ ] Dependabot for dependency updates
- [ ] **Licensing**
  - [ ] `LICENSE-MIT` file
  - [ ] `LICENSE-APACHE` file
  - [ ] SPDX identifiers in all `Cargo.toml`
- [ ] **Repository files**
  - [ ] `.gitignore` (target/, data/, *.parquet, *.arrow)
  - [ ] `CONTRIBUTING.md`
  - [ ] `SECURITY.md`
- [ ] **Code quality**
  - [ ] `rustfmt.toml`
  - [ ] Workspace-level clippy lints in root `Cargo.toml`
  - [ ] `deny.toml` for `cargo-deny` (license + advisory audit)

### Ongoing

- [ ] CI matrix: macOS + Windows runners
- [ ] MSRV check in CI
- [ ] `cargo-release` config for workspace publishing
- [ ] GitHub Actions: auto-publish to crates.io on tag push
- [ ] Per-crate `CHANGELOG.md` via `git-cliff`
- [ ] GitHub Pages docs site (`mdbook` or hosted rustdoc)
- [ ] `criterion` benchmarks in CI with regression detection

---

## Future Crates (Post-1.0)

Tracked for roadmap visibility. Will not block any 1.0 release above.

| Crate | Purpose | Depends on |
|-------|---------|------------|
| `eicu-etl` | eICU → Arrow ClinicalEvent schema | `medcodes` (optional) |
| `omop-etl` | OMOP-CDM v5.4 → Arrow | `medcodes` (optional) |
| `fhir-etl` | FHIR R4 JSON/NDJSON → Arrow | `medcodes` (optional) |
| `clinical-signals` | EDF/EDF+, WFDB biosignal I/O + epoch windowing | — |
| `clinical-metrics` | AUROC, PR-AUC, NRI, DCA, Brier, C-statistic | — |
| `clinical-calib` | Conformal prediction for model calibration | `clinical-metrics` |
| `clinical-inference` | ONNX Runtime wrapper for Arrow batch inference | — |

---

## Priority Order

Releases are sequenced by dependency:

```
medcodes v0.1.0          ← start here (zero internal dependencies)
    │
    ├─► mimic-etl v0.1.0  ← second (optionally depends on medcodes)
    │
    └─► clinical-tasks v0.1.0  ← third (consumes Arrow, optionally uses medcodes)
```

Within each release, checklist items are ordered by implementation priority (top = first).
