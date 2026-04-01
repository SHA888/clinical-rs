#![doc = include_str!("../README.md")]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]

//! MIMIC-III/IV clinical database ETL — CSV to Apache Arrow.
//!
//! This crate provides:
//! - Efficient CSV to Arrow conversion for MIMIC datasets
//! - Parallel processing using Rayon
//! - Memory-mapped file support for large datasets
//! - Support for core MIMIC-IV tables

pub mod arrow_writer;
pub mod csv_reader;
pub mod types;

// Re-export commonly used types
pub use arrow_writer::{StreamingArrowWriter, to_arrow_ipc, to_parquet};
pub use csv_reader::MimicCsvReader;
pub use types::{ClinicalEvent, DatasetConfig, EtlError, Result};
