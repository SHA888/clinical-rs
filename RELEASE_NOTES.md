# Release Notes v0.0.1

## Overview

This is the initial release of the clinical-rs workspace, containing three foundational crates for clinical data processing in Rust.

## Published Crates

### 🏥 medcodes v0.0.1
**Medical code ontologies, hierarchy traversal, and cross-system mapping**

- ICD-10-CM code definitions and hierarchy (stub implementation)
- Cross-system code mapping capabilities (planned)
- Efficient lookup using compile-time hash maps (planned)
- Feature flags for optional serde support

### 📊 mimic-etl v0.0.1  
**MIMIC-III/IV clinical database ETL — CSV to Apache Arrow**

- CSV reader module for MIMIC datasets (stub implementation)
- Arrow writer module for data output (stub implementation)
- Parallel processing using Rayon (planned)
- Memory-mapped file support for large datasets (planned)
- Feature flags for optional medcodes integration and CLI

### 🤖 clinical-tasks v0.0.1
**Clinical prediction task windowing — Arrow event streams to ML-ready datasets**

- Time-based windowing of clinical events (stub implementation)
- Feature extraction from event streams (stub implementation)
- ML-ready dataset generation (planned)
- Feature flags for optional medcodes integration

## Infrastructure

- ✅ Rust 1.94.0 toolchain with 2024 edition
- ✅ Workspace-level dependency management
- ✅ Automated testing, formatting, and linting
- ✅ Security vulnerability scanning
- ✅ License compliance checking
- ✅ GitHub Actions CI/CD pipeline
- ✅ Automated release workflow

## Publishing Status

All crates are configured and ready for publishing to crates.io:

1. **medcodes** - No external dependencies, ready to publish first
2. **mimic-etl** - Depends on medcodes, publishes second  
3. **clinical-tasks** - Depends on medcodes, publishes third

The GitHub Actions release workflow will automatically:
- Publish crates in the correct dependency order
- Create GitHub release with changelog
- Wait between publications to ensure registry propagation

## Next Steps

After v0.0.1 release:
- Implement actual functionality in each crate
- Add comprehensive tests and examples
- Expand documentation and API coverage
- Add more medical code systems (SNOMED, LOINC)
- Add support for more clinical datasets

## License

Dual licensed under:
- MIT License ([LICENSE-MIT](LICENSE-MIT))
- Apache License 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
