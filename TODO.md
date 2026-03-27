# Roadmap

Comprehensive checklist organized by phase and [SemVer](https://semver.org/) release milestone. Each item is a concrete deliverable.

> **Versioning policy:** All three crates version independently. A breaking change in `medcodes` does not force a major bump in `mimic-etl`. Pre-1.0 releases (0.x.y) may contain breaking changes in minor versions per SemVer §4.

---

## Table of Contents

- [Phase 0 — Project Bootstrap](#phase-0--project-bootstrap)
- [medcodes](#medcodes)
- [mimic-etl](#mimic-etl)
- [clinical-tasks](#clinical-tasks)
- [Future Crates (Post-1.0)](#future-crates-post-10)

---

## Phase 0 — Project Bootstrap

Everything needed before writing the first line of library code. This phase produces a fully configured, CI-protected, release-ready Cargo workspace with zero library functionality.

### 0.0.1 — Toolchain & Environment

- [x] **Rust toolchain**
  - [x] `rust-toolchain.toml` pinning `channel = "1.94.0"` (current stable, 2024 edition)
  - [x] Components: `rustfmt`, `clippy`, `rust-src`, `rust-analyzer`
  - [ ] Verify: `rustup show` matches on fresh clone
- [x] **Dev tools (documented in `CONTRIBUTING.md`)**
  - [x] `cargo-nextest` 0.9.x — test runner (parallel, better output than `cargo test`)
  - [x] `cargo-deny` 0.19.x — license audit, advisory DB, dependency policy
  - [x] `cargo-release` 1.1.x — workspace-aware semver release flow
  - [x] `cargo-audit` 0.22.x — security advisory checking
  - [x] `git-cliff` 2.12.x — conventional-commit changelog generation
  - [x] `cargo-machete` — detect unused dependencies
  - [x] `cargo-udeps` (nightly only, optional) — detect unused deps at compile time
- [x] **Git configuration**
  - [x] `.gitignore`: `target/`, `data/`, `*.parquet`, `*.arrow`, `*.ipc`, `.env`, `*.csv.gz`
  - [x] `.gitattributes`: `*.rs diff=rust`, LF line endings enforced
  - [x] Conventional Commits enforced in `CONTRIBUTING.md` (feat/fix/docs/chore/refactor/test/ci)

### 0.0.2 — Workspace Manifest

- [x] **Root `Cargo.toml`**
  ```toml
  [workspace]
  resolver = "2"
  members = ["crates/*"]

  [workspace.package]
  edition = "2024"
  rust-version = "1.94.0"
  license = "MIT OR Apache-2.0"
  repository = "https://github.com/SHA888/clinical-rs"
  homepage = "https://github.com/SHA888/clinical-rs"
  authors = ["Kresna Sucandra"]
  categories = ["science", "data-structures"]
  keywords = ["clinical", "healthcare", "arrow", "medical", "ehr"]

  [workspace.dependencies]
  # Core
  arrow = "58.1"
  parquet = "58.1"
  chrono = { version = "0.4.44", default-features = false, features = ["std", "clock"] }
  serde = { version = "1.0.228", features = ["derive"] }
  thiserror = "2.0.18"

  # ETL
  csv = "1.4"
  rayon = "1.11"
  memmap2 = "0.9.10"

  # Code tables
  phf = { version = "0.13.1", features = ["macros"] }

  # Dev / test
  criterion = { version = "0.8.2", features = ["html_reports"] }
  proptest = "1"
  tempfile = "3"
  insta = "1"          # snapshot testing
  ```
- [x] Verify: `cargo check --workspace` passes with empty `lib.rs` stubs
- [x] Verify: `cargo fmt --all -- --check` passes
- [x] Verify: `cargo clippy --workspace -- -D warnings` passes

### 0.0.3 — Crate Scaffolding (Empty Shells)

Each crate gets a publishable-but-empty skeleton.

- [x] **`crates/medcodes/Cargo.toml`**
  ```toml
  [package]
  name = "medcodes"
  version = "0.0.0"                         # 0.0.0 = unpublished placeholder
  description = "Medical code ontologies, hierarchy traversal, and cross-system mapping"
  edition.workspace = true
  rust-version.workspace = true
  license.workspace = true
  repository.workspace = true
  authors.workspace = true
  categories = ["science", "data-structures"]
  keywords = ["icd10", "medical-codes", "snomed", "loinc", "healthcare"]
  readme = "README.md"

  [dependencies]
  thiserror = { workspace = true }
  phf = { workspace = true }
  serde = { workspace = true, optional = true }

  [dev-dependencies]
  criterion = { workspace = true }
  proptest = { workspace = true }

  [features]
  default = ["icd10cm"]
  icd10cm = []
  serde = ["dep:serde"]

  [[bench]]
  name = "lookup"
  harness = false
  ```
  - [x] `crates/medcodes/src/lib.rs` — module stubs, `#![doc]` header, public re-exports
  - [x] `crates/medcodes/README.md` — crate-level README (rendered on crates.io)
  - [x] `crates/medcodes/CHANGELOG.md` — initialized with `## [Unreleased]`
- [x] **`crates/mimic-etl/Cargo.toml`**
  ```toml
  [package]
  name = "mimic-etl"
  version = "0.0.0"
  description = "MIMIC-III/IV clinical database ETL — CSV to Apache Arrow"
  edition.workspace = true
  rust-version.workspace = true
  license.workspace = true
  repository.workspace = true
  authors.workspace = true
  categories = ["science", "parser-implementations"]
  keywords = ["mimic", "clinical", "arrow", "ehr", "healthcare"]
  readme = "README.md"

  [dependencies]
  arrow = { workspace = true }
  parquet = { workspace = true }
  csv = { workspace = true }
  rayon = { workspace = true }
  memmap2 = { workspace = true }
  chrono = { workspace = true }
  thiserror = { workspace = true }
  medcodes = { path = "../medcodes", optional = true }

  [dev-dependencies]
  criterion = { workspace = true }
  tempfile = { workspace = true }

  [features]
  default = []
  medcodes = ["dep:medcodes"]
  cli = ["dep:clap"]          # future: CLI binary

  [[bench]]
  name = "parse"
  harness = false
  ```
  - [x] `crates/mimic-etl/src/lib.rs` — module stubs
  - [x] `crates/mimic-etl/README.md`
  - [x] `crates/mimic-etl/CHANGELOG.md`
- [x] **`crates/clinical-tasks/Cargo.toml`**
  ```toml
  [package]
  name = "clinical-tasks"
  version = "0.0.0"
  description = "Clinical prediction task windowing — Arrow event streams to ML-ready datasets"
  edition.workspace = true
  rust-version.workspace = true
  license.workspace = true
  repository.workspace = true
  authors.workspace = true
  categories = ["science", "data-structures"]
  keywords = ["clinical", "machine-learning", "arrow", "mortality", "healthcare"]
  readme = "README.md"

  [dependencies]
  arrow = { workspace = true }
  chrono = { workspace = true }
  thiserror = { workspace = true }
  medcodes = { path = "../medcodes", optional = true }

  [dev-dependencies]
  criterion = { workspace = true }
  tempfile = { workspace = true }

  [features]
  default = []
  medcodes = ["dep:medcodes"]

  [[bench]]
  name = "windowing"
  harness = false
  ```
  - [x] `crates/clinical-tasks/src/lib.rs` — module stubs
  - [x] `crates/clinical-tasks/README.md`
  - [x] `crates/clinical-tasks/CHANGELOG.md`
- [x] **Final check:** `cargo check --workspace` compiles all three empty crates

### 0.0.4 — Code Quality Configuration

- [x] **`rustfmt.toml`**
  ```toml
  edition = "2024"
  max_width = 100
  use_field_init_shorthand = true
  use_try_shorthand = true
  ```
- [x] **Workspace-level clippy lints** (in root `Cargo.toml`)
  ```toml
  [workspace.lints.rust]
  unsafe_code = "forbid"
  missing_docs = "warn"
  unused_results = "warn"

  [workspace.lints.clippy]
  all = { level = "warn", priority = -1 }
  pedantic = { level = "warn", priority = -1 }
  nursery = { level = "warn", priority = -1 }
  unwrap_used = "warn"
  expect_used = "warn"
  panic = "warn"
  ```
  Each crate inherits via `[lints] workspace = true` in its `Cargo.toml`.
- [x] **`deny.toml`** (cargo-deny config)
  ```toml
  [advisories]
  vulnerability = "deny"
  unmaintained = "warn"
  yanked = "warn"

  [licenses]
  allow = ["MIT", "Apache-2.0", "BSD-2-Clause", "BSD-3-Clause", "ISC", "Unicode-3.0", "Zlib"]
  confidence-threshold = 0.8

  [bans]
  multiple-versions = "warn"
  wildcards = "deny"

  [sources]
  unknown-registry = "deny"
  unknown-git = "deny"
  allow-registry = ["https://github.com/rust-lang/crates.io-index"]
  ```
- [x] **Verify:** `cargo deny check` passes on empty workspace

### 0.0.5 — CI / CD Pipeline (GitHub Actions)

- [x] **`.github/workflows/ci.yml`** — runs on every push and PR
  - [x] Matrix: `ubuntu-latest` (primary), `macos-latest`, `windows-latest`
  - [x] Steps:
    1. `rustup` install with `rust-toolchain.toml`
    2. `cargo fmt --all -- --check`
    3. `cargo clippy --workspace --all-targets -- -D warnings`
    4. `cargo nextest run --workspace` (with `cargo-nextest`)
    5. `cargo doc --workspace --no-deps` (verify docs build)
    6. `cargo deny check`
  - [x] Rust cache via `Swatinem/rust-cache@v2`
  - [x] Fail-fast: `false` (report all platform failures, not just first)
- [x] **`.github/workflows/release.yml`** — runs on `v*` tag push
  - [x] Determine which crate(s) changed
  - [x] `cargo publish -p <crate>` with `CARGO_REGISTRY_TOKEN` secret
  - [x] Create GitHub Release with changelog from `git-cliff`
- [x] **`.github/workflows/audit.yml`** — scheduled nightly
  - [x] `cargo audit` against RustSec advisory DB
  - [x] Opens issue on vulnerability found
- [x] **`.github/workflows/msrv.yml`** — weekly
  - [x] Install MSRV toolchain (1.94.0)
  - [x] `cargo check --workspace` to verify MSRV holds
- [ ] **Branch protection rules** (manual, in GitHub settings)
  - [ ] `main` branch: require PR, require CI pass, require 1 approval (when collaborators exist)
  - [ ] No force push to `main`

### 0.0.6 — Repository Files

- [x] **`LICENSE-MIT`** — MIT license text with `Kresna Sucandra` and current year
- [x] **`LICENSE-APACHE`** — Apache 2.0 full text
- [x] **`CONTRIBUTING.md`**
  - [x] Development setup instructions (clone, `rustup`, tool installs)
  - [x] Commit message format (Conventional Commits)
  - [x] PR process: fork → branch → PR → CI pass → review → merge
  - [ ] How to add a new code system to `medcodes`
  - [ ] How to add a new dataset parser
  - [ ] How to add a new task to `clinical-tasks`
  - [x] Code style: follow `rustfmt.toml`, satisfy clippy pedantic
- [x] **`SECURITY.md`**
  - [x] Responsible disclosure policy
  - [x] Contact: email or GitHub security advisory
  - [x] Scope: clinical data correctness bugs are treated as security-severity
- [x] **`CODE_OF_CONDUCT.md`** — Contributor Covenant v2.1
- [x] **`.github/ISSUE_TEMPLATE/`**
  - [x] `bug_report.md` — steps to reproduce, expected vs actual, crate + version
  - [x] `feature_request.md` — use case, proposed API, which crate
- [x] **`.github/PULL_REQUEST_TEMPLATE.md`**
  - [x] Checklist: tests added, docs updated, CHANGELOG entry, `cargo fmt`, `cargo clippy`
- [ ] **`.github/FUNDING.yml`** (optional, if sponsorship desired)

### 0.0.7 — Release Infrastructure

- [x] **`cliff.toml`** (git-cliff config)
  - [x] Conventional Commits parsing
  - [x] Group by: feat, fix, docs, refactor, perf, test, ci, chore
  - [x] Per-crate changelog generation (filter commits by path `crates/<name>`)
  - [x] Template: Keep a Changelog format
- [x] **`release.toml`** (cargo-release config)
  - [x] Per-crate tags: `medcodes-v0.1.0`, `mimic-etl-v0.1.0`, `clinical-tasks-v0.1.0`
  - [x] Pre-release hook: `cargo deny check && cargo nextest run --workspace`
- [x] **Dry-run test:** `cargo release patch --workspace --dry-run` completes without error

### 0.0.8 — Verification Checkpoint

All of the following must pass on a fresh `git clone`:

- [x] `cargo check --workspace` — compiles
- [x] `cargo fmt --all -- --check` — formatted
- [x] `cargo clippy --workspace --all-targets -- -D warnings` — no warnings
- [x] `cargo nextest run --workspace` — tests pass (trivially, since no tests yet)
- [x] `cargo doc --workspace --no-deps` — docs build
- [x] `cargo deny check` — license + advisory clean
- [x] `cargo release patch --workspace --dry-run` — release flow works
- [x] GitHub Actions CI is green on `main`
- [x] README renders correctly on GitHub
- [x] ARCHITECTURE.md and TODO.md are linked and accessible

**Phase 0 is complete when a contributor can clone, build, test, lint, and dry-run a release with zero manual setup beyond `rustup` and tool installs.**

---

## `medcodes`

### v0.1.0 — Foundation

Core ontology engine with ICD-10-CM and the first cross-mapping. First crate published to crates.io.

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
  - [ ] Property tests (`proptest`): `normalize()` idempotent, `is_valid(normalize(x))` holds
  - [ ] Snapshot tests (`insta`): hierarchy traversal output for known codes
- [ ] **Documentation**
  - [ ] Rustdoc for all public types and methods with examples
  - [ ] Crate-level `README.md` with usage examples
  - [ ] `CHANGELOG.md` entry
- [ ] **Release**
  - [ ] Version bump: `0.0.0` → `0.1.0`
  - [ ] `cargo release patch --execute -p medcodes`
  - [ ] Verify crates.io page renders correctly

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
- [ ] **Cross-mappings**
  - [ ] ICD-10-CM → CCS (single-level)
  - [ ] ICD-9-CM → CCS
  - [ ] NDC → ATC
  - [ ] NDC → RxNorm
- [ ] **Serde support**
  - [ ] `Serialize`/`Deserialize` for `Code`, `System`
  - [ ] Feature flag: `serde`
- [ ] **Benchmark suite**
  - [ ] `criterion` benchmarks for lookup, traversal, and cross-mapping
  - [ ] Baseline numbers documented in README

### v0.3.0 — Clinical Terminologies

- [ ] **LOINC** — code lookup + hierarchy, feature flag: `loinc`
- [ ] **SNOMED CT** (US edition) — IS-A hierarchy, feature flag: `snomed`
  - [ ] Note: requires NLM UMLS license acknowledgment in docs
- [ ] **RxNorm** — concept lookup + RxNorm→ATC mapping, feature flag: `rxnorm`
- [ ] **CPT** — Category I codes, feature flag: `cpt`
  - [ ] Note: AMA-licensed — evaluate embedding feasibility
- [ ] **ICD-10-PCS** — 7-character structure parsing, feature flag: `icd10pcs`
- [ ] **Multi-version support** — build-time code table version selection

### v1.0.0 — Stable API

- [ ] Public API review: all types finalized
- [ ] Migration guide from 0.x
- [ ] Default features: `icd10cm` + `atc` + `loinc`
- [ ] MSRV policy documented
- [ ] Published to crates.io

---

## `mimic-etl`

### v0.1.0 — MIMIC-IV Core Tables

Parse the most commonly used MIMIC-IV (v2.x) tables into Arrow.

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
  - [ ] `rayon`-based parallel table parsing
  - [ ] Chunk-level parallelism within large tables
- [ ] **Memory-mapped I/O**
  - [ ] `memmap2` for all CSV reads
  - [ ] Configurable batch size (default 128K rows per RecordBatch)
- [ ] **Output formats**
  - [ ] `to_parquet(batches, path)`
  - [ ] `to_arrow_ipc(batches, path)`
  - [ ] Streaming: `into_event_stream()` → `Iterator<Item = Result<RecordBatch>>`
- [ ] **Tests**
  - [ ] Synthetic MIMIC-like CSV fixtures (no real PHI in repo)
  - [ ] Schema validation: output matches `ClinicalEvent` schema
  - [ ] Round-trip: CSV → Arrow → Parquet → read back, verify row counts
- [ ] **Documentation + release**
  - [ ] Rustdoc, crate `README.md`, `CHANGELOG.md`
  - [ ] Supported tables matrix
  - [ ] Version bump `0.0.0` → `0.1.0`, publish

### v0.2.0 — MIMIC-III + Performance

- [ ] **MIMIC-III (v1.4) support**
  - [ ] All core table parsers (ADMISSIONS, PATIENTS, DIAGNOSES_ICD, etc.)
  - [ ] Column name normalization (UPPER → lowercase)
- [ ] **Optional `medcodes` integration** (feature flag)
  - [ ] Code normalization during parsing
  - [ ] Auto-detect ICD-9 vs ICD-10 by MIMIC version
- [ ] **Performance benchmarks**
  - [ ] `criterion` suite on MIMIC-IV demo dataset (public)
  - [ ] Wall time + peak memory comparison methodology documented
- [ ] **CLI tool** (feature flag: `cli`)
  - [ ] `mimic-etl convert --input <path> --output <path> --format parquet`
  - [ ] `mimic-etl schema` — print ClinicalEvent Arrow schema
  - [ ] `mimic-etl info --input <path>` — table row counts

### v0.3.0 — MIMIC-IV v3.x + Notes + ED

- [ ] MIMIC-IV v3.0 schema changes + version auto-detection
- [ ] MIMIC-IV-Note (discharge summaries, radiology reports)
- [ ] MIMIC-IV-ED (edstays, triage, vitalsign)
- [ ] Incremental/resumable parsing with checkpoint files

### v1.0.0 — Stable API

- [ ] `ClinicalEvent` schema finalized
- [ ] MIMIC-III v1.4 + MIMIC-IV v2.x fully supported
- [ ] Migration guide, MSRV policy, published

---

## `clinical-tasks`

### v0.1.0 — Core Task Engine + Mortality

Minimal task windowing engine with one fully implemented task.

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
  - [ ] Configurable observation window (default: 48h)
  - [ ] Label: binary (died during hospitalization)
  - [ ] Features: code frequency vectors, lab value statistics
  - [ ] Exclusion: stays < observation window, patients < 18
- [ ] **Patient-level splitting**
  - [ ] `split_by_patient(batches, ratios)` → (train, val, test)
  - [ ] Deterministic split via patient_id hashing
  - [ ] No patient in multiple splits
- [ ] **Output**
  - [ ] RecordBatch with feature + label columns
  - [ ] Parquet / Arrow IPC export
  - [ ] Schema metadata (task name, windows, split)
- [ ] **Tests**
  - [ ] Synthetic patient timeline tests
  - [ ] Data leakage verification
  - [ ] Patient-level split verification
  - [ ] Known-answer labels for synthetic patients
- [ ] **Documentation + release**
  - [ ] Rustdoc, `README.md` with end-to-end example, `CHANGELOG.md`
  - [ ] Version bump `0.0.0` → `0.1.0`, publish

### v0.2.0 — Task Expansion

- [ ] **30-day readmission prediction** (binary, anchor: discharge)
- [ ] **Length of stay prediction** (multiclass bucketed + regression variant)
- [ ] **Drug recommendation** (multi-label, optional DDI matrix)
- [ ] **Optional `medcodes` integration** (feature flag, code grouping for features)
- [ ] **Custom task API** (docs + example for `TaskDefinition` implementors)

### v0.3.0 — Sepsis + Advanced Windowing

- [ ] **Sepsis onset prediction**
  - [ ] Configurable Sepsis-3 criteria
  - [ ] Variable lookback windows (1h, 3h, 6h, 12h)
  - [ ] Negative sampling strategy
  - [ ] Temporal advantage metric
- [ ] **Sliding window mode** (samples at regular intervals)
- [ ] **Feature engineering utilities** (temporal bins, code co-occurrence, lab trends, missingness)

### v1.0.0 — Stable API

- [ ] `TaskDefinition` trait finalized
- [ ] All v0.x tasks stable
- [ ] Migration guide, MSRV policy, published

---

## Future Crates (Post-1.0)

Tracked for roadmap visibility. Will not block any 1.0 release.

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

## Release Sequence

```
Phase 0 (bootstrap)      ← YOU ARE HERE
    │
    ▼
medcodes v0.1.0          ← first crate published (zero internal deps)
    │
    ├─► mimic-etl v0.1.0  ← second (optionally depends on medcodes)
    │
    └─► clinical-tasks v0.1.0  ← third (consumes Arrow, optionally uses medcodes)
    │
    ▼
Iterate: v0.2.0, v0.3.0 across all crates in parallel
    │
    ▼
API review → v1.0.0 per crate (independent timelines)
```

Within each release, checklist items are ordered by implementation priority (top = first).
