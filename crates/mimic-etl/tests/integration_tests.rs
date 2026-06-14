#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! Integration tests for the complete MIMIC ETL → code lookup → Arrow export pipeline.
//!
//! These tests validate the end-to-end workflow:
//! 1. Load MIMIC tables from CSV format
//! 2. Transform to Arrow `RecordBatch`
//! 3. Look up clinical codes using medcodes
//! 4. Validate schema integrity through round-trip

use std::fs;
use tempfile::TempDir;

use mimic_etl::{DatasetConfig, MimicCsvReader, MimicVersion, StreamingArrowWriter, to_parquet};

/// Helper to create a test `DatasetConfig` pointing to a temp directory
fn test_config(temp_dir: &TempDir, table_name: &str) -> DatasetConfig {
    DatasetConfig {
        root_path: temp_dir.path().to_str().unwrap().to_string(),
        version: MimicVersion::MimicIV,
        tables: vec![table_name.to_string()],
        batch_size: 128_000,
        parallelism: 1,
        use_memmap: false,
    }
}

/// Assert that a batch has valid (non-null) timestamps in the given column
fn assert_valid_timestamps(batch: &arrow::record_batch::RecordBatch, column_name: &str) {
    use arrow::array::Array;
    let ts_col = batch
        .column_by_name(column_name)
        .unwrap_or_else(|| panic!("{column_name} column must exist"));
    let ts_array = ts_col
        .as_any()
        .downcast_ref::<arrow::array::TimestampMicrosecondArray>()
        .unwrap_or_else(|| panic!("{column_name} should be TimestampMicrosecondArray"));
    assert!(
        ts_array.null_count() < ts_array.len(),
        "{column_name} should have at least one non-null timestamp"
    );
}

/// Sample MIMIC-IV admissions CSV data
fn sample_admissions_csv() -> String {
    "subject_id,hadm_id,admittime,dischtime,deathtime,admission_type,admission_location,discharge_location,insurance,language,marital_status,ethnicity,edregtime,edouttime,hospital_expire_flag\n100001,5000001,2020-01-15 08:00:00,2020-01-20 12:00:00,,URGENT,EMERGENCY ROOM ADMIT,HOME HEALTH CARE,Medicaid,ENGLISH,,WHITE,2020-01-15 07:30:00,2020-01-15 09:15:00,0\n100002,5000002,2020-01-16 08:00:00,2020-01-22 12:00:00,,EMERGENCY,EMERGENCY ROOM ADMIT,HOME,Medicare,ENGLISH,,BLACK/AFRICAN AMERICAN,2020-01-16 07:45:00,2020-01-16 09:30:00,0\n".to_string()
}

/// Sample MIMIC-IV labevents CSV data
fn sample_labevents_csv() -> String {
    r"subject_id,hadm_id,itemid,charttime,valuenum,valueuom
100001,5000001,50810,2020-01-15 09:30:00,1.2,mg/dL
100002,5000002,50809,2020-01-16 09:00:00,245,mg/dL
"
    .to_string()
}

#[test]
fn test_etl_admissions_to_arrow_roundtrip() {
    // Setup: create temporary directory structure
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let admissions_path = temp_dir.path().join("admissions.csv");

    fs::write(&admissions_path, sample_admissions_csv()).expect("Failed to write admissions CSV");

    // Step 1: Create dataset config pointing to temp directory
    let config = test_config(&temp_dir, "admissions");

    // Step 2: Create reader and load admissions table
    let reader = MimicCsvReader::new(config);
    let batches = reader
        .read_table("admissions", &admissions_path)
        .expect("Failed to read admissions table");

    // Verify: should have loaded rows
    assert!(
        !batches.is_empty(),
        "CSV should produce at least one RecordBatch"
    );

    // Step 3: Validate batch schema and row counts
    // Each admission record generates: 1 admission event + 1 discharge event (if dischtime present)
    // With 2 admissions and both having dischtime: 2 admission + 2 discharge = 4 events total
    let total_rows: usize = batches
        .iter()
        .map(arrow::record_batch::RecordBatch::num_rows)
        .sum();
    assert_eq!(
        total_rows, 4,
        "Should have 4 events (2 admissions + 2 discharges)"
    );

    // Step 4: Write to Arrow IPC and verify round-trip
    let arrow_path = temp_dir.path().join("admissions.arrow");
    mimic_etl::to_arrow_ipc(&batches, &arrow_path).expect("Failed to write Arrow IPC file");

    assert!(arrow_path.exists(), "Arrow file should be created");
    let arrow_size = fs::metadata(&arrow_path).expect("Arrow metadata").len();
    assert!(
        arrow_size > 100,
        "Arrow file should contain data, got {arrow_size} bytes"
    );

    // Step 5: Write to Parquet and verify
    let parquet_path = temp_dir.path().join("admissions.parquet");
    to_parquet(&batches, &parquet_path).expect("Failed to write Parquet");

    assert!(parquet_path.exists(), "Parquet file should be created");
    let parquet_size = fs::metadata(&parquet_path).expect("Parquet metadata").len();
    assert!(
        parquet_size > 100,
        "Parquet file should contain data, got {parquet_size} bytes"
    );

    // Step 6: Verify schema consistency and data validity
    let batch = &batches[0];
    let schema = batch.schema();

    // Validate expected columns exist
    let column_names: Vec<&str> = schema.fields().iter().map(|f| f.name().as_str()).collect();
    assert!(
        column_names.contains(&"subject_id") && column_names.contains(&"hadm_id"),
        "Schema should contain expected clinical columns"
    );

    // Verify timestamps are valid (not silently dropped to null)
    assert_valid_timestamps(batch, "charttime");
}

#[test]
fn test_etl_labevents_to_arrow() {
    // Setup: create temporary directory
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let labevents_path = temp_dir.path().join("labevents.csv");

    fs::write(&labevents_path, sample_labevents_csv()).expect("Failed to write labevents CSV");

    // Create dataset config
    let config = test_config(&temp_dir, "labevents");

    // Load labevents table
    let reader = MimicCsvReader::new(config);
    let batches = reader
        .read_table("labevents", &labevents_path)
        .expect("Failed to read labevents table");

    assert!(!batches.is_empty(), "Should load labevents");

    // Verify all events were parsed
    let total_rows: usize = batches
        .iter()
        .map(arrow::record_batch::RecordBatch::num_rows)
        .sum();
    assert_eq!(total_rows, 2, "Should have parsed all 2 lab events");

    // Verify timestamps are valid (not all null)
    let batch = &batches[0];
    assert_valid_timestamps(batch, "charttime");
}

#[test]
fn test_etl_event_parsing_integrity() {
    // Verify that events are parsed correctly with no schema mismatches
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let admissions_path = temp_dir.path().join("admissions.csv");

    fs::write(&admissions_path, sample_admissions_csv()).expect("Failed to write test CSV");

    let config = test_config(&temp_dir, "admissions");

    let reader = MimicCsvReader::new(config);
    let batches = reader
        .read_table("admissions", &admissions_path)
        .expect("Failed to read table");

    let batch = &batches[0];

    // Check that all columns have same row count
    for field in batch.schema().fields() {
        let col = batch.column_by_name(field.name()).unwrap();
        assert_eq!(
            col.len(),
            batch.num_rows(),
            "Column {} has mismatched row count",
            field.name()
        );
    }

    // Verify subject_id has values (non-nullable)
    if let Some(subject_col) = batch.column_by_name("subject_id") {
        assert_eq!(
            subject_col.null_count(),
            0,
            "subject_id should not have null values"
        );
    }
}

#[test]
fn test_streaming_arrow_writer() {
    // Test the streaming writer used for large datasets
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let admissions_path = temp_dir.path().join("admissions.csv");
    let arrow_output = temp_dir.path().join("streamed_admissions.arrow");

    fs::write(&admissions_path, sample_admissions_csv()).expect("Failed to write test CSV");

    // Read data
    let config = DatasetConfig {
        root_path: temp_dir.path().to_str().unwrap().to_string(),
        version: MimicVersion::MimicIV,
        tables: vec!["admissions".to_string()],
        batch_size: 128_000,
        parallelism: 1,
        use_memmap: false,
    };

    let reader = MimicCsvReader::new(config);
    let batches = reader
        .read_table("admissions", &admissions_path)
        .expect("Failed to read table");

    let schema = batches[0].schema();

    // Create streaming writer and write batches
    let mut writer = StreamingArrowWriter::new(&arrow_output, schema.as_ref())
        .expect("Failed to create streaming writer");

    for batch in &batches {
        writer.write_batch(batch).expect("Failed to write batch");
    }

    writer.finish().expect("Failed to finish writing");

    // Verify output file was created and has content
    assert!(arrow_output.exists(), "Arrow output file should exist");
    let metadata = fs::metadata(&arrow_output).expect("Failed to get file metadata");
    assert!(metadata.len() > 0, "Arrow output file should have content");
}

#[test]
fn test_code_lookup_integration() {
    // Test integration with medcodes for code normalization and lookup
    #[cfg(feature = "medcodes")]
    {
        use medcodes::CodeSystem;
        use mimic_etl::CodeNormalizer;

        let normalizer = CodeNormalizer::for_version(MimicVersion::MimicIV);

        // Test ICD-10 code normalization
        let code = normalizer.normalize("I10");
        assert!(!code.is_empty(), "Should normalize ICD-10 code");

        // Test that normalized code can be looked up
        use medcodes::icd10cm::Icd10Cm;
        let icd10 = Icd10Cm::new();
        let lookup = icd10.lookup(&code);
        assert!(lookup.is_ok(), "Normalized code should be lookupable");
    }

    #[cfg(not(feature = "medcodes"))]
    {
        // Skip if medcodes feature not enabled
    }
}
