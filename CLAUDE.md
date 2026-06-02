# CLAUDE.md — clinical-rs Development

This document describes the clinical-rs codebase, development practices, and collaboration model with Claude Code.

## Codebase Overview

**clinical-rs** is a Rust workspace for medical code ontology and clinical data processing with Apache Arrow.

**Repository**: [SHA888/clinical-rs](https://github.com/SHA888/clinical-rs)

**Crates**:
- `medcodes` — Medical code lookups (ICD-10, ATC, NDC, LOINC, RxNorm, SNOMED CT, CCS/CCSR, CPT)
- `mimic-etl` — MIMIC dataset ETL pipeline
- `clinical-tasks` — Task-based clinical data pipelines (longevity, comorbidity, etc.)

**Dependencies**: Arrow 58, Parquet 58, Serde, Chrono, Thiserror, Rayon (parallelization)

**Edition**: 2024, MSRV 1.94.0

## Architecture Principles

1. **Arrow as the universal contract**: All inter-crate data exchange uses Apache Arrow `RecordBatch`. No custom serialization at boundaries.
2. **Separation of concerns**:
   - `medcodes` = code tables only, no Arrow dependency, no I/O beyond embedded data
   - `mimic-etl` = ETL and data transformation
   - `clinical-tasks` = Clinical logic and derived signals
3. **No unsafe code**: Forbid unsafe_code lint; all safety via Rust's type system.
4. **Embedded code tables**: ICD-10-CM/PCS, ATC, NDC, RxNorm, LOINC, SNOMED CT compiled at build time.

See [ARCHITECTURE.md](ARCHITECTURE.md) for full data flow and crate responsibilities.

## Development Workflow

### Rust Standards

- **Linting**:
  ```bash
  cargo clippy --all-targets --all-features -- -D warnings
  ```
- **Testing**:
  ```bash
  cargo test --all
  ```
- **Documentation**:
  ```bash
  cargo doc --all --no-deps --open
  ```
- **Formatting**: `rustfmt` is enforced by CI.

### Pull Requests

1. **Branch name** suggests intent: `feat/`, `fix/`, `docs/`, `refactor/`, `perf/`
2. **Commit messages** follow Conventional Commits: `type(scope): description`
   - **Do not include `Co-Authored-By:` trailers** in assistant-generated commits. Commit attribution remains with the human author. Trailers add noise without conveying meaningful authorship.
3. **PR title** = first commit message (auto-formatted by CI)
4. **Tests** are required; snapshot tests use `insta` crate (auto-update with `--test-thread=1 -- --nocapture`)
5. **Changelog** update required for user-facing changes (CHANGELOG.md)

### Common Tasks

| Task | Command |
|------|---------|
| Run all tests | `cargo test --all` |
| Lint | `cargo clippy --all-targets --all-features -- -D warnings` |
| Build docs | `cargo doc --all --no-deps` |
| Snapshot test update | `cargo test --all -- --test-thread=1 -- --nocapture` |
| Publish dry-run | `cargo publish --dry-run --allow-dirty` |

## Code Patterns

### Error Handling

Use `thiserror` for custom errors:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("code not found: {code}")]
    NotFound { code: String },
}
```

No unwrap/expect in library code (lint enforces `unwrap_used` and `expect_used` warnings).

### Arrow Schema Patterns

Schemas are often defined as const static:

```rust
pub fn schema() -> arrow::datatypes::SchemaRef {
    arrow::datatypes::SchemaRef::new(arrow::datatypes::Schema::new(vec![
        arrow::datatypes::Field::new("column_name", arrow::datatypes::DataType::Int64, false),
    ]))
}
```

### Code Tables / Lookups

Code tables (ICD-10, RxNorm, etc.) are **compiled at build time** using `phf` crate for zero-cost perfect hashing. See `medcodes/build.rs`.

## Language & Documentation Standards

### English-Only Requirement

- **Plans.md**: All content must be in English (headers, table columns, task descriptions, status markers)
  - Use English status markers: `pending`, `in_progress`, `completed`, `blocked`
  - No Japanese or other non-English characters in tracked files
- **All harness output and documentation** must be in English
- **Commit messages and code comments** use English

This constraint ensures consistency and accessibility across the team and CI systems.

## Collaboration Guidelines

### When to Use Claude Code

✅ **Do use** Claude Code for:
- New features or refactoring in existing crates
- Writing or updating tests
- Documentation and CHANGELOG updates
- Cross-crate integration
- Performance analysis or optimization
- Bug fixes with test coverage

❌ **Avoid or discuss first**:
- Changes to build.rs or code generation (complex and easy to break)
- Dependency upgrades (verify in CI)
- Public API changes (discuss in issue first)
- Breaking changes to existing crates

### Code Review Checklist

Before creating a PR:
1. ✅ All tests pass: `cargo test --all`
2. ✅ Clippy clean: `cargo clippy --all-targets --all-features -- -D warnings`
3. ✅ Docs build: `cargo doc --all --no-deps`
4. ✅ CHANGELOG.md updated (user-facing changes only)
5. ✅ Commit messages follow Conventional Commits

## Tools & Configuration

- **CI**: GitHub Actions (`.github/workflows/`)
- **Pre-commit**: `.pre-commit-config.yaml` enforces formatting
- **Snapshot tests**: `insta` (review with INSTA_REVIEW=1)
- **Benchmarks**: `criterion` (run with `cargo bench --all`)

## Contact & Issues

**GitHub Issues**: Feature requests, bug reports, discussions
**Maintainer**: Kresna Sucandra (@SHA888)
