# Contributing to Clinical RS

## Development Environment Setup

### Prerequisites

- Rust toolchain (automatically managed via `rust-toolchain.toml`)
- Git

### Toolchain

This project uses `rust-toolchain.toml` to pin the Rust version to 1.94.0 (stable, 2024 edition).

To verify your toolchain:
```bash
rustup show
```

### Development Tools

The following tools are required for development:

- `cargo-nextest` 0.9.x — test runner (parallel, better output than `cargo test`)
- `cargo-deny` 0.19.x — license audit, advisory DB, dependency policy
- `cargo-release` 1.1.x — workspace-aware semver release flow
- `cargo-audit` 0.22.x — security advisory checking
- `git-cliff` 2.12.x — conventional-commit changelog generation
- `cargo-machete` — detect unused dependencies
- `cargo-udeps` (nightly only, optional) — detect unused deps at compile time

Install with:
```bash
cargo install cargo-nextest@0.9 --locked
cargo install cargo-deny@0.19 --locked
cargo install cargo-release@1.1 --locked
cargo install cargo-audit@0.22 --locked
cargo install git-cliff@2.12 --locked
cargo install cargo-machete --locked
```

### Conventional Commits

This project follows the Conventional Commits specification:

- `feat:` — new features
- `fix:` — bug fixes
- `docs:` — documentation changes
- `chore:` — maintenance tasks
- `refactor:` — code refactoring
- `test:` — test additions/changes
- `ci:` — CI configuration changes

Format:
```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

Examples:
- `feat: add patient data validation`
- `fix: resolve memory leak in data processing`
- `docs: update API documentation`
- `chore: upgrade dependencies`

### Running Tests

Use `cargo nextest` for running tests:
```bash
cargo nextest run
```

### Code Quality

Run all quality checks:
```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo deny check
cargo audit
```

### Release Process

Releases are automated using `cargo-release`:
```bash
cargo release --dry-run  # Preview changes
cargo release           # Actual release
```
