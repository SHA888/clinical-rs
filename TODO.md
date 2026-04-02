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
  - [eicu-etl](#eicu-etl)
  - [omop-etl](#omop-etl)
  - [fhir-etl](#fhir-etl)

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
  - [x] How to add a new code system to `medcodes`
  - [x] How to add a new dataset parser
  - [x] How to add a new task to `clinical-tasks`
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

- [x] **Core types**
  - [x] `System` enum (ICD9CM, ICD10CM, ICD10PCS, ATC, NDC, LOINC, SNOMED, RxNorm, CCS, CCSR, CPT)
  - [x] `Code` struct (system, code, description)
  - [x] `CodeSystem` trait (lookup, ancestors, descendants, is_valid, normalize)
  - [x] `CrossMap` trait (map, source_system, target_system)
  - [x] `Error` type via `thiserror`
- [x] **ICD-10-CM implementation**
  - [x] Download and process CMS FY2025 ICD-10-CM code table
  - [x] `build.rs` pipeline: TSV source data → `phf::Map` source generation
  - [x] `Icd10Cm::lookup(code) → Result<Code>`
  - [x] `Icd10Cm::is_valid(code) → bool`
  - [x] `Icd10Cm::normalize(code) → String` (strip dots, uppercase)
  - [x] Hierarchy traversal: `ancestors()`, `descendants()`, `parent()`, `children()`
  - [x] Feature flag: `icd10cm` (enabled by default)
- [x] **CCS/CCSR cross-mapping**
  - [x] Download and process AHRQ CCSR v2024.1 & CCSR v2026.1 mapping files
  - [x] `CrossMap::icd10cm_to_ccsr() → impl CrossMap`
  - [x] Bidirectional lookup support
- [x] **Tests**
  - [x] Unit tests for every public method
  - [x] Known-answer tests against CMS reference data (≥50 code lookups verified)
  - [x] Property tests (`proptest`): `normalize()` idempotent, `is_valid(normalize(x))` holds
  - [x] Snapshot tests (`insta`): hierarchy traversal output for known codes
- [x] **Documentation**
  - [x] Rustdoc for all public types and methods with examples
  - [x] Crate-level `README.md` with usage examples
  - [x] `CHANGELOG.md` entry
- [x] **Release**
  - [x] Version bump: `0.0.0` → `0.1.0`
  - [x] `cargo release patch --execute -p medcodes`
  - [x] Verify crates.io page renders correctly

### v0.2.0 — Code System Expansion

- [x] **ICD-9-CM** (frozen Oct 2015 release)
  - [x] Code table processing and embedding
  - [x] Full `CodeSystem` trait implementation
  - [x] Feature flag: `icd9cm`
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

### v0.1.0

Parse the most commonly used MIMIC-IV (v2.x) tables into Arrow.

- [x] **Core ETL types**
  - [x] `ClinicalEvent` Arrow schema definition (shared constant)
  - [x] `DatasetConfig` struct (root path, table selection, batch size, parallelism)
  - [x] `Error` type
- [x] **MIMIC-IV hosp module**
  - [x] `admissions.csv` parser → ClinicalEvent batches
  - [x] `patients.csv` parser → patient demographics Arrow table
  - [x] `diagnoses_icd.csv` parser
  - [x] `procedures_icd.csv` parser
  - [x] `prescriptions.csv` parser
  - [x] `labevents.csv` parser
  - [x] `microbiologyevents.csv` parser
  - [x] `transfers.csv` parser
- [x] **MIMIC-IV icu module**
  - [x] `icustays.csv` parser
  - [x] `chartevents.csv` parser (streaming — ~330M rows)
  - [x] `inputevents.csv` parser
  - [x] `outputevents.csv` parser
  - [x] `procedureevents.csv` parser
- [x] **Parallel processing**
  - [x] `rayon`-based parallel table parsing
  - [x] Chunk-level parallelism within large tables
- [x] **Memory-mapped I/O**
  - [x] `memmap2` for all CSV reads
  - [x] Configurable batch size (default 128K rows per RecordBatch)
- [x] **Output formats**
  - [x] `to_parquet(batches, path)`
  - [x] `to_arrow_ipc(batches, path)`
  - [x] Streaming: `into_event_stream()` → `Iterator<Item = Result<RecordBatch>>`
- [x] **Tests**
  - [x] Synthetic MIMIC-like CSV fixtures (no real PHI in repo)
  - [x] Schema validation: output matches `ClinicalEvent` schema
  - [x] Round-trip: CSV → Arrow → Parquet → read back, verify row counts
- [x] **Documentation + release**
  - [x] Rustdoc, crate `README.md`, `CHANGELOG.md`
  - [x] Supported tables matrix
  - [x] Version bump `0.0.0` → `0.1.0`, publish

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

- [x] **Core types**
  - [x] `TaskDefinition` trait
  - [x] `TaskWindows` struct (observation, gap, prediction, anchor)
  - [x] `AnchorPoint` enum (Admission, Discharge, ICUAdmission, ICUDischarge, Custom)
  - [x] `TaskRunner` — applies a `TaskDefinition` to a RecordBatch stream
  - [x] `Error` type
- [x] **Patient grouping**
  - [x] Group RecordBatch stream by `(patient_id, visit_id)`
  - [x] Sort events by timestamp within each group
  - [x] Handle multi-visit patients
- [x] **In-hospital mortality prediction**
  - [x] `MortalityPrediction` implementing `TaskDefinition`
  - [x] Configurable observation window (default: 48h)
  - [x] Label: binary (died during hospitalization)
  - [x] Features: code frequency vectors, lab value statistics
  - [x] Exclusion: stays < observation window, patients < 18
- [x] **Patient-level splitting**
  - [x] `split_by_patient(batches, ratios)` → (train, val, test)
  - [x] Deterministic split via patient_id hashing
  - [x] No patient in multiple splits
- [x] **Output**
  - [x] RecordBatch with feature + label columns
  - [x] Parquet / Arrow IPC export
  - [x] Schema metadata (task name, windows, split)
- [x] **Tests**
  - [x] Synthetic patient timeline tests
  - [x] Data leakage verification
  - [x] Patient-level split verification
  - [x] Known-answer labels for synthetic patients
- [x] **Documentation + release**
  - [x] Rustdoc, `README.md` with end-to-end example, `CHANGELOG.md`
  - [x] Version bump `0.0.2-rc.2` → `0.1.0`, publish

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
| [`eicu-etl`](#eicu-etl) | eICU → Arrow ClinicalEvent schema | `medcodes` (optional) |
| [`omop-etl`](#omop-etl) | OMOP-CDM v5.4 → Arrow | `medcodes` (optional) |
| [`fhir-etl`](#fhir-etl) | FHIR R4 JSON/NDJSON → Arrow | `medcodes` (optional) |
| `clinical-signals` | EDF/EDF+, WFDB biosignal I/O + epoch windowing | — |
| `clinical-metrics` | AUROC, PR-AUC, NRI, DCA, Brier, C-statistic | — |
| `clinical-calib` | Conformal prediction for model calibration | `clinical-metrics` |
| `clinical-inference` | ONNX Runtime wrapper for Arrow batch inference | — |

---

## Release Sequence

```
Phase 0 (bootstrap)           ✓ complete
    │
    ▼
medcodes v0.1.0               ✓ published
    │
    ├─► mimic-etl v0.1.0      ✓ published
    │
    └─► clinical-tasks v0.1.0 ✓ published
    │
    ▼
Iterate: v0.2.0, v0.3.0 across all crates in parallel  ← YOU ARE HERE
    │
    ├─► eicu-etl v0.1.0       high priority — data already on local machine
    │
    ▼
API review → v1.0.0 per crate (independent timelines)
    │
    ▼
omop-etl v0.1.0               medium priority
fhir-etl v0.1.0               medium priority
```

Within each release, checklist items are ordered by implementation priority (top = first).

---

## `eicu-etl`

eICU Collaborative Research Database → Arrow, aligned to the `ClinicalEvent` schema.

eICU key differences from MIMIC-IV: multi-site (208 ICUs), `patientunitstayid` as primary
join key, explicit vasopressor and ventilator tables, APACHE IVa severity scoring.

### v0.1.0 — Core parsers

- [ ] **Crate scaffold**
  - [ ] `crates/eicu-etl/Cargo.toml` (same structure as `mimic-etl`, `medcodes` optional)
  - [ ] `crates/eicu-etl/src/lib.rs` — module stubs
  - [ ] `crates/eicu-etl/README.md`, `CHANGELOG.md`
  - [ ] Add to workspace `members` in root `Cargo.toml`
- [ ] **Core ETL types**
  - [ ] `DatasetConfig` struct (root path, table selection, batch size)
  - [ ] `Error` type
- [ ] **Core table parsers**
  - [ ] `patient.csv` → Arrow (patientunitstayid, age, gender, ethnicity, unittype,
    unitstaytype, admissionheight, admissionweight, unitdischargestatus, LOS)
  - [ ] `diagnosis.csv` → Arrow (ICD-9 codes, active problems, diagnosisoffset)
  - [ ] `lab.csv` → Arrow (labname, labresult, labresultoffset, labresulttypeindex)
  - [ ] `vitalperiodic.csv` → Arrow (time-series: HR, SBP, DBP, MAP, SpO2, RR, temp)
  - [ ] `vitalaperiodic.csv` → Arrow (non-periodic vitals: GCS, pupil, non-invasive BP)
  - [ ] `medication.csv` → Arrow (drugname, dosage, routeadmin, drugstartoffset)
  - [ ] `infusiondrug.csv` → Arrow (vasopressors, sedation infusions, drugrate)
  - [ ] `respiratorycare.csv` → Arrow (airwaytype, priorventstartoffset, ventendoffset)
    - [ ] Note: SAT/SBT attempts derivable from airwaytype + vent timing — document derivation
  - [ ] `apachepatientresult.csv` → Arrow (apachescore, predictedicumortality, actualhospitalmortality)
  - [ ] `apachepredvar.csv` → Arrow (APACHE IVa predictor variables)
- [ ] **Output formats**
  - [ ] `to_parquet(batches, path)`
  - [ ] `to_arrow_ipc(batches, path)`
  - [ ] Streaming: `into_event_stream()` → `Iterator<Item = Result<RecordBatch>>`
- [ ] **Tests**
  - [ ] Synthetic eICU-like CSV fixtures (no real PHI in repo)
  - [ ] Schema validation: output matches `ClinicalEvent` schema
  - [ ] Round-trip: CSV → Arrow → Parquet → read back, verify row counts
- [ ] **Documentation + release**
  - [ ] Rustdoc, `README.md` with supported tables matrix, `CHANGELOG.md`
  - [ ] Version bump `0.0.0` → `0.1.0`, publish

### v0.2.0 — Schema alignment + medcodes integration

- [ ] **`ClinicalEvent` schema alignment** — align eICU output with MIMIC-IV shared schema
- [ ] **`medcodes` integration** (feature flag)
  - [ ] ICD-9-CM → ICD-10-CM crosswalk via `medcodes` during parsing
- [ ] **Multi-site field** — `hospitalid` preserved in all Arrow outputs
- [ ] **SAT/SBT event struct** — `RespiratoryEvent` mirroring MIMIC-IV equivalent
- [ ] **Performance benchmarks** — `criterion` suite, comparison methodology documented

### v0.3.0 — Full table coverage + CLI

- [ ] Remaining tables: `nursecharting`, `note`, `microlab`, `physicalexam`, `treatment`
- [ ] CLI tool (feature flag: `cli`): `eicu-etl convert --input <path> --output <path>`
- [ ] Version auto-detection (eICU v2.0 vs v2.0.1 schema differences)

### v1.0.0 — Stable API

- [ ] `ClinicalEvent` schema finalized and aligned with `mimic-etl`
- [ ] Migration guide, MSRV policy, published

---

## `omop-etl`

OMOP Common Data Model v5.4 → Arrow, aligned to the `ClinicalEvent` schema.
Enables clinical-rs to handle any OMOP-compliant EHR export (Epic, Cerner, etc.).

### v0.1.0 — Core CDM v5.4 tables

- [ ] **Crate scaffold**
  - [ ] `crates/omop-etl/Cargo.toml`
  - [ ] `crates/omop-etl/src/lib.rs` — module stubs
  - [ ] `crates/omop-etl/README.md`, `CHANGELOG.md`
  - [ ] Add to workspace `members`
- [ ] **Core table parsers** (CSV exports from any OMOP-compliant source)
  - [ ] `PERSON` → Arrow (demographics, birth_datetime, gender_concept_id, race, ethnicity)
  - [ ] `VISIT_OCCURRENCE` → Arrow (visit_start/end_datetime, visit_type_concept_id)
  - [ ] `CONDITION_OCCURRENCE` → Arrow (condition_concept_id, condition_start_datetime)
  - [ ] `DRUG_EXPOSURE` → Arrow (drug_concept_id, drug_exposure_start_datetime, quantity)
  - [ ] `MEASUREMENT` → Arrow (measurement_concept_id, value_as_number, unit_concept_id)
  - [ ] `PROCEDURE_OCCURRENCE` → Arrow (procedure_concept_id, procedure_datetime)
  - [ ] `OBSERVATION` → Arrow (observation_concept_id, value_as_string)
  - [ ] `VISIT_DETAIL` → Arrow (ICU stay granularity, care_site_id)
- [ ] **Concept ID resolution**
  - [ ] `CONCEPT` table parser — embedded concept vocabulary lookup at parse time
  - [ ] Concept ID → standard name + domain + vocabulary_id
- [ ] **`ClinicalEvent` schema output** — aligned with `mimic-etl` shared schema
- [ ] **Tests** — synthetic OMOP-like fixtures, round-trip validation
- [ ] **Documentation + release** — version `0.0.0` → `0.1.0`, publish

### v0.2.0 — Vocabularies + medcodes integration

- [ ] **Full OMOP vocabulary embedding** — `CONCEPT.csv` processing at build time via `build.rs`
- [ ] **`medcodes` integration** (feature flag) — bidirectional OMOP concept ↔ ICD/SNOMED mapping
- [ ] **CDM version auto-detection** — v5.3 vs v5.4 schema differences handled
- [ ] **CLI tool** (feature flag: `cli`): `omop-etl convert --cdm-version 5.4`

### v1.0.0 — Stable API

- [ ] OMOP CDM v5.4 fully covered
- [ ] Migration guide, MSRV policy, published

---

## `fhir-etl`

FHIR R4 JSON/NDJSON → Arrow, aligned to the `ClinicalEvent` schema.
Enables ingestion of modern EHR exports and SMART on FHIR data streams.

### v0.1.0 — Core R4 resources

- [ ] **Crate scaffold**
  - [ ] `crates/fhir-etl/Cargo.toml` (adds `serde_json` + `serde` deps)
  - [ ] `crates/fhir-etl/src/lib.rs` — module stubs
  - [ ] `crates/fhir-etl/README.md`, `CHANGELOG.md`
  - [ ] Add to workspace `members`
- [ ] **Core resource parsers** (JSON single-resource and NDJSON streaming)
  - [ ] `Patient` → Arrow (id, birthDate, gender, address)
  - [ ] `Encounter` → Arrow (id, status, class, period.start/end, subject)
  - [ ] `Condition` → Arrow (code.coding[0] → ICD via `medcodes`, recordedDate)
  - [ ] `MedicationRequest` → Arrow (medication, authoredOn, dosageInstruction)
  - [ ] `MedicationAdministration` → Arrow (medication, effective[x], dosage)
  - [ ] `Observation` → Arrow (code → LOINC via `medcodes`, value[x], effectiveDateTime)
  - [ ] `Procedure` → Arrow (code, performed[x])
- [ ] **NDJSON streaming parser** — line-by-line, no full document load into memory
- [ ] **Polymorphic type handling** — FHIR `value[x]` variants (valueQuantity, valueString, etc.)
- [ ] **`ClinicalEvent` schema output** — aligned with `mimic-etl` shared schema
- [ ] **Tests** — synthetic FHIR R4 JSON fixtures, round-trip validation
- [ ] **Documentation + release** — version `0.0.0` → `0.1.0`, publish

### v0.2.0 — Bundle + SMART on FHIR

- [ ] **FHIR Bundle parsing** — searchset, transaction, history bundle types
- [ ] **Pagination** — `Bundle.link[rel=next]` cursor following
- [ ] **SMART on FHIR client** (feature flag: `smart-auth`)
  - [ ] OAuth2 client credentials flow
  - [ ] `FhirClient::search(resource, params)` → streaming Arrow output
- [ ] **`medcodes` integration** (feature flag) — FHIR coding systems ↔ ICD/SNOMED/LOINC

### v0.3.0 — R5 compatibility

- [ ] FHIR R5 resource compatibility layer (additive, no breaking changes to R4 API)

### v1.0.0 — Stable API

- [ ] FHIR R4 fully covered
- [ ] Migration guide, MSRV policy, published
