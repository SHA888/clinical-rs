//! Longevity module — post-critical-illness biological age acceleration signals.
//!
//! This module provides types for tracking and analyzing biological age changes
//! following critical illness, including senescence markers, functional trajectories,
//! and epigenetic clock-based age acceleration measurements.
//!
//! All types are feature-gated behind the `longevity` feature flag.

#[cfg(feature = "longevity")]
pub use clock::{BiologicalAgeDelta, CalibrationStatus, ClockVersion};
#[cfg(feature = "longevity")]
pub use senescence::{FunctionalTrajectory, SaspComposite, SenescenceScore};
#[cfg(feature = "longevity")]
pub use signals::LongevitySignals;

#[cfg(feature = "longevity")]
pub mod clock;
#[cfg(feature = "longevity")]
pub mod senescence;
#[cfg(feature = "longevity")]
pub mod signals;
