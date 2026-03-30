# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.0.2-rc.2] - 2026-03-29

### Fixed
- Fixed ICD-10-CM hierarchy traversal methods (`ancestors()`, `parent()`) to properly return codes with descriptions
- Fixed `normalize_code()` to strip all whitespace (including internal spaces)
- Updated `lookup()` and `is_valid()` to handle spaces in input codes
- Fixed test invalid codes (X999 is valid as X99.9 in CMS dataset)

### Added
- Added `dotted_form_for_normalized()` helper for proper description lookups
- Added comprehensive examples to all public API methods
- Enhanced README with detailed usage examples for ICD-10-CM and CCSR mapping
- Added complete test suite (64 tests) including:
  - 118 verified CMS reference code tests (exceeds 50+ requirement)
  - Property-based tests with proptest
  - Snapshot tests for hierarchy operations
  - Unit tests for all public methods

### Improved
- Improved code clarity and fixed clippy warnings
- Updated pre-commit config to allow test-specific warnings
- All tests now pass successfully

## [0.0.2-rc.1] - 2026-03-26

### Added
- Release candidate for v0.0.2
- Improved crate structure and documentation
- Enhanced benchmark setup

## [0.0.1] - 2026-03-26

### Added
- Initial release with empty crate structure
- ICD-10-CM module stubs
- Comprehensive documentation and examples
- Benchmark setup for lookup operations
- Feature flags for optional serde support

## [Unreleased]
