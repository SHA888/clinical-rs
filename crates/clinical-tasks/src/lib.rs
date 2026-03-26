#![doc = include_str!("../README.md")]

//! Clinical prediction task windowing — Arrow event streams to ML-ready datasets.
//!
//! This crate provides:
//! - Time-based windowing of clinical events
//! - Feature extraction from event streams
//! - ML-ready dataset generation

pub mod features;
pub mod windowing;
