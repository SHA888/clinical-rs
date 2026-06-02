# Plans — clinical-rs Development Roadmap

This document tracks active development tasks, feature work, and known issues for the clinical-rs workspace using the harness v2 task format.

---

## Current Sprint

| Task | Content | DoD | Depends | Status |
|------|---------|-----|---------|--------|
| 1.1 | LOINC documentation: module-level docs, README, examples, license attribution | Module-level docs explain 6-axis LOINC classification; README lists LOINC version & data source; rustdoc includes lab test lookup example; LOINC terms-of-use acknowledgment in docs | - | cc:completed [07296b6] |

---

## Backlog

### Code Quality & Maintenance

| Task | Content | DoD | Depends | Status |
|------|---------|-----|---------|--------|
| 2.1 | Documentation coverage — Fill missing rustdoc in public APIs | All public items in medcodes, mimic-etl, clinical-tasks have rustdoc with examples; `cargo doc --all` builds without warnings | - | cc:completed [07296b6] |
| 2.2 | Integration tests — Round-trip ETL → code lookup → Arrow export | Test loads MIMIC CSV → transforms to Arrow → looks up codes → validates schema round-trip; tests pass with real data sample | - | cc:TODO |
| 2.3 | Dependency audit — Regular `cargo deny check` in CI | CI enforces `cargo deny` on license, advisories, sources; all dependencies have approved licenses | - | cc:TODO |
| 2.4 | MSRV bump candidate — Evaluate 1.95+ once stable | Research Rust 1.95 compatibility; identify breaking changes if any; update MSRV in Cargo.toml if feasible | - | cc:TODO |

### Features & Enhancements

| Task | Content | DoD | Depends | Status |
|------|---------|-----|---------|--------|
| 3.1 | LOINC expansion — Add LOINC hierarchy and relationships | LOINC parent/child/ancestor/descendant methods implemented; tests verify hierarchy traversal; benchmarks included | - | cc:TODO |
| 3.2 | SNOMED CT relationships — Implement `is_a`, `part_of`, `has_component` traversal | SNOMED CT module with polyhierarchy support; transitive closure computed; typed relationships implemented; tests + benchmarks | 3.1 | cc:TODO |
| 3.3 | eICU ETL — Add eICU dataset support (beyond MIMIC-IV) | eICU CSV parser similar to mimic-etl; Arrow schema compatible with existing code lookup; tests with sample eICU data | - | cc:TODO |
| 3.4 | Comorbidity scores — Implement Charlson, Elixhauser scoring | clinical-tasks module with score computation; input: Arrow schema with diagnoses; output: Arrow schema with scores; tests with known inputs | - | cc:TODO |
| 3.5 | Clinical prediction models — Schema extensions for model inputs/outputs | Arrow schema extensions for model artifacts; serialization/deserialization tested; integration with clinical-tasks | - | cc:TODO |
| 3.6 | Performance benchmarks — Arrow batch processing at scale (1M+ records) | criterion benchmarks for Arrow operations at 1M, 10M row scales; baseline established; results documented | - | cc:TODO |

### Known Issues / Bug Fixes

| Task | Content | DoD | Depends | Status |
|------|---------|-----|---------|--------|
| 4.1 | Arrow 59+ compatibility — Test upgrade path from Arrow 58 | Arrow 59 dependency evaluated; breaking changes identified; upgrade path documented or deferred with reason | - | cc:TODO |
| 4.2 | Large code table perf — Profile NDC lookup speed with 100k+ entries | NDC lookup profiled with `cargo flamegraph`; performance baseline established; optimization opportunities identified | - | cc:TODO |

---

## Completed

(Move completed items here with date)

---

## Process Notes

- **Snapshot test review**: `INSTA_REVIEW=1 cargo test --all` to interactively accept changes
- **Publish workflow**: Run `cargo publish --dry-run --allow-dirty` before tagging
- **Pre-commit hook**: Enforces `rustfmt` and `clippy` on staged files
- **No `Co-Authored-By:` trailers** in commit messages — attribution stays with human author
- **English-only**: All Plans.md content, harness output, and documentation must be in English
