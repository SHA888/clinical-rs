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

### Pre-commit Hooks

This project uses pre-commit hooks to ensure code quality and consistency.

#### Quick Setup
```bash
./scripts/setup-dev.sh
```

#### Manual Setup
```bash
# Install pre-commit (if not already installed)
pip install pre-commit

# Install hooks
pre-commit install
```

#### Available Hooks
- **rustfmt**: Auto-formats Rust code
- **clippy**: Runs linter with strict warnings
- **cargo-test**: Runs tests with cargo-nextest
- **cargo-audit**: Security vulnerability checking
- **cargo-deny**: License and security compliance
- **commit-msg**: Enforces conventional commit format

#### Usage
```bash
# Run all hooks on all files
pre-commit run --all-files

# Run hooks on staged files (automatic on commit)
git commit -m "feat: add new feature"
```

See [docs/pre-commit-hooks.md](docs/pre-commit-hooks.md) for detailed information.

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
