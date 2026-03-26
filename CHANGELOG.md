# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
