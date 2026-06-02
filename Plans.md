# Plans — clinical-rs Development Roadmap

This document tracks active development tasks, feature work, and known issues for the clinical-rs workspace.

## Current Sprint

(Add sprint tasks here as you work)

---

## Backlog

### Features & Enhancements

- [ ] **LOINC expansion** — Add LOINC hierarchy and relationships (parents, components)
- [ ] **SNOMED CT relationships** — Implement `is_a`, `part_of`, `has_component` traversal
- [ ] **eICU ETL** — Add eICU dataset support (beyond MIMIC-IV)
- [ ] **Comorbidity scores** — Implement Charlson, Elixhauser scoring in clinical-tasks
- [ ] **Clinical prediction models** — Schema extensions for model inputs/outputs
- [ ] **Performance benchmarks** — Arrow batch processing at scale (1M+ records)

### Code Quality & Maintenance

- [ ] **Documentation coverage** — Fill missing rustdoc in public APIs
- [ ] **Integration tests** — Add round-trip tests for ETL → code lookup → Arrow export
- [ ] **Dependency audit** — Regular `cargo deny check` in CI
- [ ] **MSRV bump candidate** — Evaluate 1.95+ once stable

### Known Issues

- [ ] **Arrow 59+ compatibility** — Test upgrade path from Arrow 58
- [ ] **Large code table perf** — Profile NDC lookup speed with 100k+ entries

---

## Completed

(Move completed items here with date)

---

## Process Notes

- Snapshot test review: `INSTA_REVIEW=1 cargo test --all` to interactively accept changes
- Publish workflow: Run `cargo publish --dry-run --allow-dirty` before tagging
- Pre-commit hook enforces `rustfmt` and `clippy` on staged files
