#![doc = include_str!("../README.md")]

//! Medical code ontologies, hierarchy traversal, and cross-system mapping.
//!
//! This crate provides:
//! - ICD-10-CM code definitions and hierarchy
//! - Cross-system code mapping capabilities
//! - Efficient lookup using compile-time hash maps

pub mod icd10;
pub mod types;

pub use icd10::Icd10Cm;
pub use types::{Code, CodeSystem, CrossMap, MedCodeError, System};
