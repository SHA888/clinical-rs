#![doc = include_str!("../README.md")]

//! MIMIC-III/IV clinical database ETL — CSV to Apache Arrow.
//!
//! This crate provides:
//! - Efficient CSV to Arrow conversion for MIMIC datasets
//! - Parallel processing using Rayon
//! - Memory-mapped file support for large datasets

pub mod arrow_writer;
pub mod csv_reader;
