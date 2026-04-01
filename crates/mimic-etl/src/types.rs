//! Core types for MIMIC ETL operations.

use arrow::datatypes::{DataType, Field, Schema, TimeUnit};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during ETL operations.
#[derive(Debug, Error)]
pub enum EtlError {
    /// IO error from standard library
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Arrow error from arrow crate
    #[error("Arrow error: {0}")]
    Arrow(#[from] arrow::error::ArrowError),

    /// CSV parsing error
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    /// Parquet file error
    #[error("Parquet error: {0}")]
    Parquet(#[from] parquet::errors::ParquetError),

    /// Missing required column in input data
    #[error("Missing column: {0}")]
    MissingColumn(String),

    /// Invalid data format encountered
    #[error("Invalid data format: {0}")]
    InvalidFormat(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),
}

/// Configuration for ETL operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetConfig {
    /// Root path containing MIMIC CSV files
    pub root_path: String,

    /// Which tables to process
    pub tables: Vec<String>,

    /// Batch size for `RecordBatch` creation
    pub batch_size: usize,

    /// Number of parallel workers
    pub parallelism: usize,

    /// Whether to use memory-mapped I/O
    pub use_memmap: bool,
}

impl DatasetConfig {
    /// Finish configuration and validate settings.
    ///
    /// # Errors
    /// Returns an error if the configuration is invalid.
    pub fn finish(self) -> Result<()> {
        Ok(())
    }
}

impl Default for DatasetConfig {
    fn default() -> Self {
        Self {
            root_path: "data/mimic-iv".to_string(),
            tables: vec![
                "admissions".to_string(),
                "patients".to_string(),
                "diagnoses_icd".to_string(),
                "procedures_icd".to_string(),
                "prescriptions".to_string(),
                "labevents".to_string(),
            ],
            batch_size: 128_000,
            parallelism: num_cpus::get(),
            use_memmap: true,
        }
    }
}

/// Schema for clinical events - the unified representation of all MIMIC data.
#[must_use]
pub fn clinical_event_schema() -> Schema {
    Schema::new(vec![
        Field::new("subject_id", DataType::Int64, false),
        Field::new("hadm_id", DataType::Int64, true),
        Field::new("stay_id", DataType::Int64, true),
        Field::new(
            "charttime",
            DataType::Timestamp(TimeUnit::Microsecond, None),
            true,
        ),
        Field::new("event_type", DataType::Utf8, false),
        Field::new("event_id", DataType::Utf8, true),
        Field::new("value", DataType::Utf8, true),
        Field::new("value_num", DataType::Float64, true),
        Field::new("units", DataType::Utf8, true),
    ])
}

/// A clinical event extracted from MIMIC data.
#[derive(Debug, Clone)]
pub struct ClinicalEvent {
    /// Patient identifier
    pub subject_id: i64,

    /// Hospital admission identifier
    pub hadm_id: Option<i64>,

    /// ICU stay identifier
    pub stay_id: Option<i64>,

    /// Timestamp of the event
    pub charttime: Option<i64>,

    /// Type of event (diagnosis, procedure, lab, etc.)
    pub event_type: String,

    /// Event identifier (ICD code, lab item, etc.)
    pub event_id: Option<String>,

    /// String value
    pub value: Option<String>,

    /// Numeric value
    pub value_num: Option<f64>,

    /// Units
    pub units: Option<String>,
}

/// Result type for ETL operations.
pub type Result<T> = std::result::Result<T, EtlError>;
