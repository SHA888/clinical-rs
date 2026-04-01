# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-04-01

### Added
- **Stable release** of all three crates: `medcodes`, `mimic-etl`, and `clinical-tasks`
- **End-to-end example** in README showing complete pipeline from MIMIC-IV CSV to ML-ready dataset
- **Comprehensive documentation** with Rustdoc for all public APIs
- **Mortality prediction task** with configurable time windows and patient-level splitting
- **MIMIC-III/IV ETL** with parallel processing and Arrow output
- **ICD-10-CM code hierarchy** with ancestor/descendant traversal
- **Cross-system mapping** between ICD-10-CM and CCSR categories

### Fixed
- All clippy warnings resolved with `-D warnings`
- Proper error handling throughout (no more panics in production code)
- Memory-efficient streaming processing for large datasets
- Deterministic patient-level train/val/test splitting

### Changed
- Updated all crate versions from `0.0.2-rc.2` to `0.1.0`
- Improved API consistency across crates
- Enhanced documentation with examples

## [0.0.2-rc.1] - 2026-03-26

### Added
- Release candidate for v0.0.2
- Improved workflow configuration
- Fixed GitHub Actions release workflow

## [0.0.1] - 2026-03-26

### Added
- Initial release of clinical-rs workspace
- **medcodes** crate: Medical code ontologies and hierarchy traversal
- **mimic-etl** crate: MIMIC-III/IV clinical database ETL to Apache Arrow
- **clinical-tasks** crate: Clinical prediction task windowing and ML datasets
- Complete project bootstrap with CI/CD pipeline
- Comprehensive documentation and examples
- Code quality tools configuration (rustfmt, clippy, cargo-deny)
- GitHub Actions workflows for automated releases

### Infrastructure
- Rust 1.94.0 toolchain with 2024 edition
- Workspace-level dependency management
- Automated testing, formatting, and linting
- Security vulnerability scanning
- License compliance checking

## [Unreleased]
