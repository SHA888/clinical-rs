# Architecture

This document describes the technical architecture of the `clinical-rs` workspace, the responsibilities of each crate, their boundaries, and the data flow between them.

## Overview

```
                    ┌──────────────────────────────────────────────────┐
                    │              clinical-rs workspace               │
                    │                                                  │
  Raw Data          │  ┌───────────┐   ┌───────────┐   ┌───────────┐  │   ML Training
  ─────────────────►│  │ mimic-etl │──►│ medcodes  │◄──│ clinical- │  │──► (PyTorch,
  MIMIC CSVs        │  │           │   │           │   │   tasks   │  │    JAX, ONNX)
  eICU, OMOP        │  └─────┬─────┘   └───────────┘   └─────┬─────┘  │
  (future)          │        │    Arrow RecordBatch           │        │
                    │        └────────────────────────────────►│        │
                    │                                          │        │
                    │                              Arrow IPC / Parquet  │
                    └──────────────────────────────────────────────────┘
```

All inter-crate data exchange uses Apache Arrow `RecordBatch` as the universal contract. No custom serialization, no framework-specific types at crate boundaries.

---

## Crate Responsibilities

### `medcodes`

**Purpose:** Medical code ontology lookup, hierarchy traversal, and cross-system mapping.

**Scope boundaries:**
- ✅ ICD-9-CM, ICD-10-CM, ICD-10-PCS, ATC, NDC, RxNorm, LOINC, SNOMED CT, CCS/CCSR, CPT code systems
- ✅ Per-code: description, category, parent/child/ancestor/descendant traversal
- ✅ Cross-system mapping (e.g., ICD-10-CM → CCSR, NDC → ATC, NDC → RxNorm)
- ✅ Code validation and normalization (strip dots, case-fold, version-aware)
- ❌ No dataset parsing, no Arrow dependency, no I/O beyond embedded data
- ❌ No clinical logic (what constitutes "sepsis" is a task concern, not a code concern)

**Data sources:** Code tables are compiled from official public distributions:

| System | Source | Update cycle |
|--------|--------|--------------|
| ICD-10-CM/PCS | [CMS](https://www.cms.gov/medicare/coding-billing/icd-10-codes) | Annual (Oct 1) |
| ICD-9-CM | [CMS](https://www.cms.gov/medicare/coding-billing/icd-9-cm-diagnosis-procedure-codes) | Frozen (Oct 2015) |
| ATC | [WHO Collaborating Centre](https://www.whocc.no/atc_ddd_index/) | Annual |
| NDC | [FDA](https://www.fda.gov/drugs/drug-approvals-and-databases/national-drug-code-directory) | Monthly |
| LOINC | [Regenstrief Institute](https://loinc.org/) | Semi-annual |
| SNOMED CT | [NLM](https://www.nlm.nih.gov/healthit/snomedct/) | Biannual (US edition) |
| RxNorm | [NLM](https://www.nlm.nih.gov/research/umls/rxnorm/) | Monthly |
| CCS/CCSR | [AHRQ/HCUP](https://hcup-us.ahrq.gov/toolssoftware/ccsr/ccs_refined.jsp) | Annual |

**Embedding strategy:** Code tables are processed at build time (`build.rs`) into compact binary formats and embedded via `include_bytes!`. This ensures:
- Zero runtime I/O for code lookups
- Deterministic builds (pinned to a specific code table release)
- No external file dependencies for consumers

**Key types:**

```rust
/// A resolved code within a single coding system.
pub struct Code {
    pub system: System,
    pub code: String,
    pub description: String,
}

/// Ontology for a single coding system.
pub trait CodeSystem {
    fn lookup(&self, code: &str) -> Result<Code>;
    fn ancestors(&self, code: &str) -> Result<Vec<String>>;
    fn descendants(&self, code: &str) -> Result<Vec<String>>;
    fn is_valid(&self, code: &str) -> bool;
    fn normalize(&self, code: &str) -> String;
}

/// Cross-system mapping.
pub trait CrossMap {
    fn map(&self, source_code: &str) -> Result<Vec<String>>;
    fn source_system(&self) -> System;
    fn target_system(&self) -> System;
}
```

**Dependencies:** Minimal — `serde`, `thiserror`, `phf` (compile-time hash maps). No `arrow`, no `tokio`, no heavy dependencies.

---

### `mimic-etl`

**Purpose:** Parse MIMIC-III and MIMIC-IV CSV files into a standardized Arrow `RecordBatch` stream of clinical events.

**Scope boundaries:**
- ✅ Parse all MIMIC-III (v1.4) and MIMIC-IV (v2.x, v3.x) CSV tables
- ✅ Emit a unified `ClinicalEvent` Arrow schema (see below)
- ✅ Memory-mapped I/O via `memmap2` for large files
- ✅ Parallel CSV parsing via `rayon`
- ✅ Streaming `RecordBatch` iterator output (constant memory usage)
- ✅ Export to Parquet and Arrow IPC
- ✅ Optional integration with `medcodes` for code normalization during ETL
- ❌ No model training, no task-specific logic
- ❌ No data download or PhysioNet credential management
- ❌ No MIMIC-specific clinical logic (e.g., sepsis cohort extraction belongs in `clinical-tasks`)

**Canonical Arrow schema:**

All parsed tables are normalized into a single event schema:

```
ClinicalEvent Schema
────────────────────────────────────────────────
patient_id      : Utf8          (NOT NULL)
visit_id        : Utf8          (NOT NULL)
event_type      : Utf8          (NOT NULL)  — "diagnosis", "procedure", "medication",
                                              "lab", "vital", "microbiology", "transfer"
code            : Utf8          (NOT NULL)  — raw code value
code_system     : Utf8          (NOT NULL)  — "ICD9CM", "ICD10CM", "ATC", "NDC",
                                              "LOINC", "ITEMID", etc.
timestamp       : Timestamp(μs) (NULLABLE)  — event time (null for undated events)
value_num       : Float64       (NULLABLE)  — numeric value (lab results, vitals)
value_text      : Utf8          (NULLABLE)  — text value (culture results, free text)
unit            : Utf8          (NULLABLE)  — unit of measurement
source_table    : Utf8          (NOT NULL)  — original CSV table name for provenance
```

This schema is the contract between `mimic-etl` and `clinical-tasks`. Any future ETL crate (e.g., `eicu-etl`, `omop-etl`) must emit the same schema.

**Parsing pipeline:**

```
CSV files on disk
  │
  ▼
┌──────────────────┐
│  Memory-mapped   │  mmap2 — avoids loading entire file into RAM
│  file handles    │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│  Parallel CSV    │  rayon + csv crate — one thread per table,
│  deserialization │  chunk-level parallelism within large tables
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│  Schema mapping  │  Table-specific → ClinicalEvent schema
│  + normalization │  Optional medcodes integration for code validation
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│  RecordBatch     │  Iterator<Item = Result<RecordBatch>>
│  stream output   │  Each batch ≈ 64K–256K rows (configurable)
└──────────────────┘
```

**Dependencies:** `arrow`, `parquet`, `csv`, `rayon`, `memmap2`, `chrono`, `thiserror`. Optional: `medcodes`.

---

### `clinical-tasks`

**Purpose:** Transform a stream of `ClinicalEvent` Arrow RecordBatches into ML-ready (features, label) datasets for specific clinical prediction tasks.

**Scope boundaries:**
- ✅ Define temporal windowing logic for clinical prediction tasks
- ✅ Mortality prediction (in-hospital, 30-day, 90-day)
- ✅ Readmission prediction (30-day)
- ✅ Length of stay prediction (bucketed, regression)
- ✅ Drug recommendation (multi-label)
- ✅ Sepsis onset prediction (configurable lookback/lookahead)
- ✅ Custom task definition via `TaskDefinition` trait
- ✅ Patient-level train/validation/test splitting
- ✅ Output as Arrow RecordBatch (features + label columns)
- ❌ No model training, no loss functions, no optimizers
- ❌ No dataset parsing (consumes Arrow from any ETL source)

**Task windowing model:**

Every clinical prediction task follows the same temporal abstraction:

```
Patient timeline
════════════════════════════════════════════════════►  time

    ◄──── observation ────►◄── gap ──►◄── prediction ──►
           window                          window

    │ Features extracted   │ ignored │ Label determined  │
    │ from events here     │         │ from events here  │

    t_start            t_obs_end   t_pred_start      t_pred_end
```

- **Observation window:** interval from which input features are extracted
- **Gap window:** buffer between observation and prediction to prevent data leakage
- **Prediction window:** interval in which the outcome (label) is determined

**Key types:**

```rust
/// Defines a clinical prediction task.
pub trait TaskDefinition {
    /// Task name identifier.
    fn name(&self) -> &str;

    /// Define the temporal windows for this task.
    fn windows(&self) -> TaskWindows;

    /// Extract feature columns from events within the observation window.
    fn extract_features(&self, events: &RecordBatch) -> Result<RecordBatch>;

    /// Determine the label from events within the prediction window.
    fn extract_label(&self, events: &RecordBatch) -> Result<ArrayRef>;

    /// Output schema (feature columns + label column).
    fn output_schema(&self) -> SchemaRef;
}

pub struct TaskWindows {
    pub observation: Duration,
    pub gap: Duration,
    pub prediction: Duration,
    pub anchor: AnchorPoint,  // Admission, Discharge, ICUAdmission, Custom
}
```

**Dependencies:** `arrow`, `chrono`, `thiserror`. Optional: `medcodes` (for code grouping in feature extraction, e.g., mapping ICD-10 codes to CCS categories before one-hot encoding).

---

## Cross-Crate Data Flow

### End-to-end: MIMIC-IV → mortality prediction samples

```
mimic-etl                         clinical-tasks
─────────                         ──────────────

Mimic4Dataset::open(path)
    │
    ├─ Parse DIAGNOSES_ICD.csv ──► RecordBatch (ClinicalEvent schema)
    ├─ Parse PRESCRIPTIONS.csv ──► RecordBatch (ClinicalEvent schema)
    ├─ Parse LABEVENTS.csv ────►   RecordBatch (ClinicalEvent schema)
    │
    └─ Merge + sort by              MortalityPrediction::apply()
       (patient_id, timestamp) ──►      │
                                        ├─ Group by (patient_id, visit_id)
                                        ├─ Apply observation window (48h)
                                        ├─ Apply gap window (0h)
                                        ├─ Apply prediction window (remaining stay)
                                        ├─ Extract features (code frequencies, lab stats)
                                        ├─ Extract label (died_in_hospital: bool)
                                        │
                                        └─► RecordBatch (features + label)
                                                │
                                                ├─ Arrow IPC → PyTorch DataLoader
                                                ├─ Parquet → long-term storage
                                                └─ DataFusion → SQL analysis
```

### Standalone: `medcodes` without any other crate

```rust
use medcodes::icd10cm::Icd10Cm;
use medcodes::crossmap::CrossMap;

// Direct lookup — no Arrow, no ETL, no files
let code = Icd10Cm::lookup("A41.9")?;
assert_eq!(code.description(), "Sepsis, unspecified organism");
assert!(code.ancestors().contains(&"A30-A49".to_string()));

// Cross-mapping
let mapper = CrossMap::icd10cm_to_ccsr()?;
let categories = mapper.map("A41.9")?;  // ["INF003"]
```

---

## Dependency Graph

```
clinical-tasks
    ├── arrow
    ├── chrono
    └── medcodes (optional, for code grouping)

mimic-etl
    ├── arrow
    ├── parquet
    ├── csv
    ├── rayon
    ├── memmap2
    ├── chrono
    └── medcodes (optional, for code normalization)

medcodes
    ├── serde
    ├── phf (compile-time hash maps)
    └── thiserror
```

`medcodes` is the leaf dependency — it depends on nothing in this workspace. Both `mimic-etl` and `clinical-tasks` optionally depend on `medcodes` via Cargo feature flags.

---

## Design Decisions

### Why Arrow, not custom structs?

Arrow RecordBatch is the lingua franca of columnar data processing. By adopting it as the crate boundary contract:
- Python consumers read output via `pyarrow.ipc` with zero-copy — no PyO3 bindings needed for basic interop
- DataFusion, Polars, and DuckDB can query output directly
- Future crates (e.g., `eicu-etl`) only need to emit the same schema to be compatible with `clinical-tasks`

### Why embedded code tables, not runtime downloads?

Clinical code systems are versioned and change annually. Embedding specific versions at build time ensures:
- Reproducible results (same crate version = same code mappings)
- No network dependency at runtime
- Auditability (the exact code table is in the repo, diffable)

Trade-off: crate binary size increases. Mitigation: feature flags per code system (`features = ["icd10cm", "atc"]`), so consumers only embed what they need.

### Why streaming RecordBatch iterators, not materialized DataFrames?

MIMIC-IV `LABEVENTS` alone is ~125M rows. Materializing the full table requires ~30-50 GB RAM. By emitting `Iterator<Item = Result<RecordBatch>>`, each batch occupies ~10-50 MB, and downstream consumers (task windowing, Parquet writers) process incrementally. Peak memory stays bounded regardless of dataset size.

### Why separate ETL crates per dataset?

`mimic-etl` knows MIMIC-specific table schemas, column names, and data quirks (e.g., MIMIC-III uses `HADM_ID`, MIMIC-IV uses `hadm_id`). Mixing this with eICU or OMOP parsing in one crate creates a leaky abstraction. Separate crates keep each parser focused. The shared `ClinicalEvent` Arrow schema is the unifying contract, not shared code.

---

## Future Crates (Planned, Not Yet Started)

| Crate | Purpose |
|-------|---------|
| `eicu-etl` | eICU Collaborative Research Database → Arrow |
| `omop-etl` | OMOP-CDM → Arrow |
| `fhir-etl` | FHIR R4 JSON/NDJSON → Arrow |
| `clinical-signals` | EDF/EDF+, WFDB biosignal I/O with epoch windowing |
| `clinical-metrics` | AUROC, PR-AUC, NRI, DCA, Brier score, C-statistic |
| `clinical-calib` | Conformal prediction for clinical model calibration |
| `clinical-inference` | ONNX Runtime wrapper for clinical model serving on Arrow batches |

These will be added to this workspace as development progresses. Each follows the same principles: Arrow-native, streaming-first, independently publishable.
