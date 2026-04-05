#![doc = include_str!("../README.md")]

//! Medical code ontologies, hierarchy traversal, and cross-system mapping.
//!
//! This crate provides:
//! - ICD-10-CM code definitions and hierarchy
//! - ICD-9-CM code definitions and hierarchy
//! - ATC (Anatomical Therapeutic Chemical) classification
//! - CCSR (Clinical Classifications Software Refined) cross-mapping
//! - Cross-system code mapping capabilities
//! - Efficient lookup using compile-time hash maps

pub mod atc;
pub mod ccs;
pub mod ccsr;
pub mod icd10;
pub mod icd9;
pub mod ndc;
pub mod types;

pub use atc::{Atc, AtcLevel};
pub use ccs::{CcsCategory, Icd9CmToCcs, Icd10CmToCcs};
pub use ccsr::{CcsrCategory, CcsrContext, CcsrMapping, CcsrToIcd10Cm, Icd10CmToCcsr};
pub use icd9::Icd9Cm;
pub use icd10::Icd10Cm;
pub use ndc::Ndc;
pub use types::{Code, CodeSystem, CrossMap, MedCodeError, System};
