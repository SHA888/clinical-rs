//! Core types for clinical task definitions and execution.

use arrow::datatypes::Schema;
use arrow::record_batch::RecordBatch;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur during task operations.
#[derive(Debug, Error)]
pub enum TaskError {
    /// IO error from standard library
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Arrow error from arrow crate
    #[error("Arrow error: {0}")]
    Arrow(#[from] arrow::error::ArrowError),

    /// Invalid window configuration
    #[error("Invalid window configuration: {0}")]
    InvalidWindow(String),

    /// Missing required column
    #[error("Missing required column: {0}")]
    MissingColumn(String),

    /// Data validation error
    #[error("Data validation error: {0}")]
    Validation(String),

    /// Task execution error
    #[error("Task execution error: {0}")]
    Execution(String),
}

/// Anchor points for task windows.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnchorPoint {
    /// Hospital admission
    Admission,
    /// Hospital discharge
    Discharge,
    /// ICU admission
    ICUAdmission,
    /// ICU discharge
    ICUDischarge,
    /// Custom timestamp (in microseconds since epoch)
    Custom(i64),
}

/// Time windows for task definitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskWindows {
    /// Observation window duration in hours
    pub observation_hours: f64,
    /// Gap window duration in hours (between observation and prediction)
    pub gap_hours: f64,
    /// Prediction window duration in hours
    pub prediction_hours: f64,
    /// Anchor point for the windows
    pub anchor: AnchorPoint,
}

impl Default for TaskWindows {
    fn default() -> Self {
        Self {
            observation_hours: 48.0,
            gap_hours: 0.0,
            prediction_hours: 24.0,
            anchor: AnchorPoint::Admission,
        }
    }
}

impl TaskWindows {
    /// Create new windows with specified durations.
    #[must_use]
    pub fn new(
        observation_hours: f64,
        gap_hours: f64,
        prediction_hours: f64,
        anchor: AnchorPoint,
    ) -> Self {
        Self {
            observation_hours,
            gap_hours,
            prediction_hours,
            anchor,
        }
    }

    /// Convert hours to microseconds for Arrow timestamps.
    fn hours_to_microseconds(hours: f64) -> i64 {
        (hours * 3600.0 * 1_000_000.0) as i64
    }

    /// Get observation window duration in microseconds.
    #[must_use]
    pub fn observation_micros(&self) -> i64 {
        Self::hours_to_microseconds(self.observation_hours)
    }

    /// Get gap window duration in microseconds.
    #[must_use]
    pub fn gap_micros(&self) -> i64 {
        Self::hours_to_microseconds(self.gap_hours)
    }

    /// Get prediction window duration in microseconds.
    #[must_use]
    pub fn prediction_micros(&self) -> i64 {
        Self::hours_to_microseconds(self.prediction_hours)
    }
}

/// Trait for defining clinical prediction tasks.
pub trait TaskDefinition: Send + Sync {
    /// Get the name of this task.
    fn name(&self) -> &str;

    /// Get the task windows configuration.
    fn windows(&self) -> &TaskWindows;

    /// Get the output schema for this task.
    fn output_schema(&self) -> Schema;

    /// Process a single patient's events to generate features and labels.
    ///
    /// # Errors
    /// Returns an error if processing fails.
    fn process_patient(&self, patient_id: i64, events: &[PatientEvent]) -> Result<TaskOutput>;

    /// Validate that the input data has required columns.
    ///
    /// # Errors
    /// Returns an error if validation fails.
    fn validate_input(&self, batch: &RecordBatch) -> Result<()>;
}

/// A patient event extracted from clinical data.
#[derive(Debug, Clone)]
pub struct PatientEvent {
    /// Patient identifier
    pub patient_id: i64,
    /// Hospital admission identifier
    pub admission_id: Option<i64>,
    /// ICU stay identifier
    pub icu_stay_id: Option<i64>,
    /// Timestamp of the event (microseconds since epoch)
    pub timestamp: Option<i64>,
    /// Type of event
    pub event_type: String,
    /// Event identifier (e.g., ICD code, lab item)
    pub event_id: Option<String>,
    /// String value
    pub value: Option<String>,
    /// Numeric value
    pub value_num: Option<f64>,
    /// Units
    pub units: Option<String>,
}

/// Output from processing a patient for a task.
#[derive(Debug, Clone)]
pub struct TaskOutput {
    /// Patient identifier
    pub patient_id: i64,
    /// Features as a map of feature name to value
    pub features: HashMap<String, f64>,
    /// Label value
    pub label: f64,
    /// Whether the label is binary (0/1) or continuous
    pub binary_label: bool,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Result type for task operations.
pub type Result<T> = std::result::Result<T, TaskError>;

/// Configuration for patient-level data splitting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitConfig {
    /// Training set ratio (0.0 to 1.0)
    pub train_ratio: f64,
    /// Validation set ratio (0.0 to 1.0)
    pub val_ratio: f64,
    /// Test set ratio (0.0 to 1.0)
    pub test_ratio: f64,
    /// Random seed for reproducible splits
    pub seed: u64,
}

impl Default for SplitConfig {
    fn default() -> Self {
        Self {
            train_ratio: 0.7,
            val_ratio: 0.15,
            test_ratio: 0.15,
            seed: 42,
        }
    }
}

impl SplitConfig {
    /// Validate the split ratios sum to 1.0.
    ///
    /// # Errors
    /// Returns an error if the ratios don't sum to 1.0.
    pub fn validate(&self) -> Result<()> {
        let sum = self.train_ratio + self.val_ratio + self.test_ratio;
        if (sum - 1.0).abs() > 1e-6 {
            return Err(TaskError::Validation(format!(
                "Split ratios must sum to 1.0, got {}",
                sum
            )));
        }
        Ok(())
    }
}
