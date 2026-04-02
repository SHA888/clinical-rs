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

## Adding New Components

### How to Add a New Code System to `medcodes`

The `medcodes` crate supports multiple medical coding systems. To add a new one:

1. **Create the module structure**:
   ```bash
   mkdir crates/medcodes/src/new_system
   touch crates/medcodes/src/new_system/mod.rs
   ```

2. **Define the code type** in `mod.rs`:
   ```rust
   use serde::{Deserialize, Serialize};
   use crate::types::{Code, System};

   #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
   pub struct NewSystemCode {
       pub code: String,
       pub description: String,
   }

   impl NewSystemCode {
       pub fn new(code: &str, description: &str) -> Self {
           Self { code: code.to_string(), description: description.to_string() }
       }
   }
   ```

3. **Implement the `CodeSystem` trait**:
   ```rust
   use crate::traits::CodeSystem;

   impl CodeSystem for NewSystemCode {
       fn system() -> System { System::NewSystem }
       fn code(&self) -> &str { &self.code }
       fn description(&self) -> &str { &self.description }
   }
   ```

4. **Add data files** (if needed):
   - Place CSV/XML data files in `crates/medcodes/data/new_system/`
   - Update `build.rs` to process the data
   - Use `phf` to generate compile-time lookup tables

5. **Add to lib.rs**:
   ```rust
   pub mod new_system;
   pub use new_system::NewSystemCode;
   ```

6. **Add tests** in `crates/medcodes/tests/new_system_tests.rs`

### How to Add a New Dataset Parser to `mimic-etl`

To add support for a new clinical dataset:

1. **Create the parser module**:
   ```bash
   touch crates/mimic-etl/src/new_dataset.rs
   ```

2. **Define the configuration**:
   ```rust
   use crate::types::DatasetConfig;

   #[derive(Debug, Clone)]
   pub struct NewDatasetConfig {
       pub root_path: String,
       pub tables: Vec<String>,
       pub batch_size: usize,
       // Add dataset-specific options
   }
   ```

3. **Implement the reader**:
   ```rust
   use arrow::record_batch::RecordBatch;
   use crate::error::EtlError;

   pub struct NewDatasetReader {
       config: NewDatasetConfig,
   }

   impl NewDatasetReader {
       pub fn new(config: NewDatasetConfig) -> Self { Self { config } }

       pub fn read_table(&self, table: &str, path: &str) -> Result<Vec<RecordBatch>, EtlError> {
           // Implement CSV/JSON/Parquet reading logic
           // Use arrow-csv, arrow-json, or arrow-parquet
       }
   }
   ```

4. **Map to standard schema**:
   ```rust
   // Convert dataset-specific columns to standard PatientEvent format
   fn map_to_patient_event(row: &CsvRow) -> Result<PatientEvent, EtlError> {
       // Map patient_id, timestamp, event_type, values
   }
   ```

5. **Add to lib.rs** and update `src/lib.rs` exports

6. **Add integration tests** in `tests/new_dataset_tests.rs`

### How to Add a New Task to `clinical-tasks`

To add a new clinical prediction task:

1. **Define the task struct**:
   ```rust
   use crate::types::{TaskWindows, TaskOutput};
   use crate::traits::TaskDefinition;

   pub struct NewPredictionTask {
       windows: TaskWindows,
       // Add task-specific configuration
   }
   ```

2. **Implement `TaskDefinition`**:
   ```rust
   impl TaskDefinition for NewPredictionTask {
       type Output = TaskOutput;

       fn name(&self) -> &str { "new_prediction" }

       fn output_schema(&self) -> Arc<Schema> {
           // Define Arrow schema for features + label
       }

       fn process_patient(&self, events: &[PatientEvent]) -> Result<Vec<Self::Output>, TaskError> {
           // Implement task logic:
           // 1. Extract observation window events
           // 2. Extract features
           // 3. Extract label
           // 4. Return TaskOutput(s)
       }
   }
   ```

3. **Define feature extraction**:
   ```rust
   impl NewPredictionTask {
       fn extract_features(&self, events: &[PatientEvent]) -> HashMap<String, f64> {
           let mut features = HashMap::new();

           // Example: count events by type
           features.insert("num_events".to_string(), events.len() as f64);

           // Example: time-based features
           features.insert("hour_of_day".to_string(), self.hour_of_day(events));

           features
       }

       fn extract_label(&self, events: &[PatientEvent]) -> f64 {
           // Define the prediction target
           // Return 0.0 or 1.0 for binary, or continuous value for regression
       }
   }
   ```

4. **Add constructor**:
   ```rust
   impl NewPredictionTask {
       pub fn new(windows: TaskWindows) -> Self {
           Self { windows }
       }
   }
   ```

5. **Add to lib.rs** and create tests in `tests/new_task_tests.rs`

6. **Update documentation** with task description and use case
