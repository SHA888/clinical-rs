#![doc = include_str!("../README.md")]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::redundant_clone)]
#![allow(clippy::float_cmp)]
#![allow(clippy::unreadable_literal)]

//! Clinical prediction task windowing — Arrow event streams to ML-ready datasets.
//!
//! This crate provides:
//! - Time-based windowing of clinical events
//! - Feature extraction from event streams
//! - ML-ready dataset generation
//! - Prediction tasks:
//!   - In-hospital mortality prediction
//!   - 30-day readmission prediction
//!   - Length of stay prediction (multiclass + regression)
//!   - Drug recommendation (multi-label with optional DDI safety)

pub mod features;
pub mod types;
pub mod windowing;

#[cfg(feature = "medcodes")]
pub mod code_grouping;

#[cfg(feature = "longevity")]
pub mod longevity;

// Re-export commonly used types
pub use features::{
    DrugClass, DrugRecommendation, LengthOfStayPrediction, LosBucket, LosTarget,
    MortalityPrediction, ReadmissionPrediction, outputs_to_batch, split_by_patient,
};
pub use types::{
    AnchorPoint, PatientEvent, Result, SplitConfig, TaskDefinition, TaskError, TaskOutput,
    TaskWindows,
};
pub use windowing::{TaskRunner, extract_task_windows, group_and_sort_events};

#[cfg(feature = "medcodes")]
pub use code_grouping::{CodeGrouper, GroupedFeatureExtractor, IcdVersion};

#[cfg(feature = "longevity")]
pub use longevity::{
    BiologicalAgeDelta, CalibrationStatus, ClockVersion, FunctionalTrajectory, LongevitySignals,
    PaceOfAgeDelta, SaspComposite, SenescenceScore,
};
