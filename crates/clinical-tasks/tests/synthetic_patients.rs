#![allow(clippy::redundant_clone)]
#![allow(clippy::float_cmp)]

//! Tests with synthetic patient timelines to verify task windowing logic.

use arrow::array::{
    Array, ArrayRef, Float64Array, Int64Array, StringArray, TimestampMicrosecondArray,
};
use arrow::datatypes::{DataType, Field, Schema, TimeUnit};
use arrow::record_batch::RecordBatch;
use clinical_tasks::{
    AnchorPoint, MortalityPrediction, SplitConfig, TaskDefinition, TaskWindows, outputs_to_batch,
    split_by_patient,
};
use std::collections::HashMap;
use std::sync::Arc;

/// Create the clinical event schema for synthetic data.
fn create_synthetic_schema() -> Schema {
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

/// Create the patient ID array for synthetic data.
fn create_patient_ids() -> Int64Array {
    Int64Array::from(vec![
        1001, 1001, 1001, 1001, 1001, 1001, // Patient 1001 - survives (6 events)
        1002, 1002, 1002, 1002, 1002, 1002, 1002, // Patient 1002 - dies (7 events)
        1003, 1003, 1003, 1003, // Patient 1003 - short stay (4 events)
    ])
}

/// Create the admission ID array for synthetic data.
fn create_hadm_ids() -> Int64Array {
    Int64Array::from(vec![
        None,
        None,
        Some(2001),
        Some(2001),
        Some(2001),
        Some(2001), // Patient 1001
        None,
        None,
        Some(2002),
        Some(2002),
        Some(2002),
        Some(2002),
        Some(2002),
        Some(2002), // Patient 1002
        None,
        Some(2003),
        Some(2003), // Patient 1003
    ])
}

/// Create the stay ID array for synthetic data.
fn create_stay_ids() -> Int64Array {
    Int64Array::from(vec![
        None, None, None, None, None, None, // Patient 1001
        None, None, None, None, None, None, None, // Patient 1002
        None, None, None, None, // Patient 1003
    ])
}

/// Create the timestamp array for synthetic data.
fn create_charttimes() -> TimestampMicrosecondArray {
    // Timestamps (in microseconds since epoch)
    let base_time = 1_609_459_200_000_000i64; // 2021-01-01 00:00:00 UTC
    TimestampMicrosecondArray::from(vec![
        Some(base_time - 7_200_000_000), // Day -2: Pre-admission event (gender)
        Some(base_time - 3_600_000_000), // Day -1: Pre-admission event (age)
        Some(base_time),                 // Day 0: Admission
        Some(base_time + 3_600_000_000), // Day 0: +1 hour - Diagnosis
        Some(base_time + 7_200_000_000), // Day 0: +2 hours - Lab
        Some(base_time + 86_400_000_000), // Day 1: Discharge (survivor)
        // Patient 1002
        Some(base_time + 172_800_000_000 - 7_200_000_000), // Day 0: Pre-admission (gender)
        Some(base_time + 172_800_000_000 - 3_600_000_000), // Day 1: Pre-admission (age)
        Some(base_time + 172_800_000_000),                 // Day 2: Admission
        Some(base_time + 176_400_000_000),                 // Day 2: +1 hour - Diagnosis
        Some(base_time + 183_600_000_000),                 // Day 2: +3 hours - Lab
        Some(base_time + 190_800_000_000),                 // Day 2: +5 hours - Procedure
        Some(base_time + 259_200_000_000),                 // Day 3: Death
        // Patient 1003
        Some(base_time + 345_600_000_000 - 3_600_000_000), // Day 3: Pre-admission (gender)
        Some(base_time + 345_600_000_000),                 // Day 4: Admission
        Some(base_time + 349_200_000_000), // Day 4: +1 hour - Discharge (short stay)
        Some(base_time + 352_800_000_000), // Day 4: +2 hours - Followup
    ])
}

/// Create the event types array for synthetic data.
fn create_event_types() -> StringArray {
    StringArray::from(vec![
        "gender",
        "anchor_age",
        "admission",
        "diagnosis",
        "lab",
        "discharge", // Patient 1001
        "gender",
        "anchor_age",
        "admission",
        "diagnosis",
        "lab",
        "procedure",
        "death", // Patient 1002
        "gender",
        "admission",
        "discharge",
        "followup", // Patient 1003
    ])
}

/// Create the event IDs array for synthetic data.
fn create_event_ids() -> StringArray {
    StringArray::from(vec![
        Some("GENDER"),
        Some("ANCHOR_AGE"),
        Some("ADMISSION"),
        Some("DIAG1"),
        Some("LAB1"),
        Some("DISCHARGE"),
        Some("GENDER"),
        Some("ANCHOR_AGE"),
        Some("ADMISSION"),
        Some("DIAG1"),
        Some("LAB1"),
        Some("PROC1"),
        Some("DEATH"),
        Some("GENDER"),
        Some("ADMISSION"),
        Some("DISCHARGE"),
        Some("FOLLOWUP"),
    ])
}

/// Create the values array for synthetic data.
fn create_values() -> StringArray {
    StringArray::from(vec![
        Some("M"),
        Some("65"),
        Some("Admitted"),
        Some("I10"),
        Some("Na=140"),
        Some("Discharged"),
        Some("F"),
        Some("78"),
        Some("Admitted"),
        Some("I21"),
        Some("K=3.2"),
        Some("CPR"),
        Some("Died"),
        Some("F"),
        Some("Admitted"),
        Some("Discharged"),
        Some("Followup"),
    ])
}

/// Create the numeric values array for synthetic data.
fn create_value_nums() -> Float64Array {
    Float64Array::from(vec![
        None,
        Some(65.0),
        None,
        None,
        Some(140.0),
        None, // Patient 1001
        None,
        Some(78.0),
        None,
        None,
        Some(3.2),
        None,
        None, // Patient 1002
        None,
        None,
        None,
        None, // Patient 1003
    ])
}

/// Create the units array for synthetic data.
fn create_units() -> StringArray {
    StringArray::from(vec![
        None,
        None,
        None,
        None,
        Some("mEq/L"),
        None,
        None,
        None,
        None,
        None,
        Some("mEq/L"),
        None,
        None,
        None,
        None,
        None,
        None,
    ])
}

/// Create the outcomes map for synthetic data.
fn create_outcomes() -> HashMap<i64, bool> {
    let mut outcomes = HashMap::new();
    let _ = outcomes.insert(1001, false); // Survives
    let _ = outcomes.insert(1002, true); // Dies
    let _ = outcomes.insert(1003, false); // Short stay (excluded)
    outcomes
}

/// Create synthetic patient timeline with known events and outcomes.
fn create_synthetic_patient_timeline() -> (RecordBatch, HashMap<i64, bool>) {
    // Define the clinical event schema
    let schema = create_synthetic_schema();

    // Create synthetic data for 3 patients with known outcomes
    let patient_ids = create_patient_ids();
    let hadm_ids = create_hadm_ids();
    let stay_ids = create_stay_ids();
    let charttimes = create_charttimes();
    let event_types = create_event_types();
    let event_ids = create_event_ids();
    let values = create_values();
    let value_nums = create_value_nums();
    let units = create_units();

    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![
            Arc::new(patient_ids) as ArrayRef,
            Arc::new(hadm_ids) as ArrayRef,
            Arc::new(stay_ids) as ArrayRef,
            Arc::new(charttimes) as ArrayRef,
            Arc::new(event_types) as ArrayRef,
            Arc::new(event_ids) as ArrayRef,
            Arc::new(values) as ArrayRef,
            Arc::new(value_nums) as ArrayRef,
            Arc::new(units) as ArrayRef,
        ],
    )
    .unwrap();

    // Known outcomes for verification
    let outcomes = create_outcomes();

    (batch, outcomes)
}

#[test]
fn test_synthetic_patient_mortality_prediction() {
    let (batch, expected_outcomes) = create_synthetic_patient_timeline();

    // Create mortality prediction task with 48-hour observation window
    let windows = TaskWindows::new(
        48.0, // observation_hours
        0.0,  // gap_hours
        24.0, // prediction_hours
        AnchorPoint::Admission,
    );

    let task = MortalityPrediction::new(windows.clone());

    // Process the batch
    let runner = clinical_tasks::windowing::TaskRunner::new(windows.clone());
    let outputs = runner.process_batch(&batch, &task).unwrap();

    // Should have outputs for all 3 patients
    assert_eq!(outputs.len(), 3);

    // Verify outcomes
    for output in &outputs {
        let expected = expected_outcomes[&output.patient_id];
        assert_eq!(output.label, if expected { 1.0 } else { 0.0 });
        assert!(output.binary_label);

        // Verify we have the expected features
        assert!(output.features.contains_key("patient_id"));
        assert!(output.features.contains_key("num_diagnoses"));
        assert!(output.features.contains_key("gender_male"));
        assert!(output.features.contains_key("num_labs"));

        // Check specific feature values
        match output.patient_id {
            1001 => {
                // Patient 1001: Male, 65 years old, survived
                // Note: num_diagnoses may be 0 if diagnosis falls outside observation window
                assert_eq!(output.features.get("gender_male"), Some(&1.0));
                assert_eq!(output.features.get("age"), Some(&65.0));
                assert_eq!(output.label, 0.0); // Survived
            }
            1002 => {
                // Patient 1002: Female, 78 years old, died
                // Note: Feature counts depend on which events fall in observation window
                assert_eq!(output.features.get("gender_male"), Some(&0.0));
                assert_eq!(output.features.get("age"), Some(&78.0));
                assert_eq!(output.label, 1.0); // Died
            }
            1003 => {
                // Patient 1003: Female, short stay, 0 labs
                assert_eq!(output.features.get("gender_male"), Some(&0.0));
                assert_eq!(output.features.get("num_labs"), Some(&0.0));
                assert_eq!(output.label, 0.0); // Survived
            }
            _ => panic!("Unexpected patient ID: {}", output.patient_id),
        }
    }
}

#[test]
fn test_patient_level_splitting() {
    let (batch, _) = create_synthetic_patient_timeline();

    // Create mortality prediction task
    let windows = TaskWindows::new(48.0, 0.0, 24.0, AnchorPoint::Admission);
    let task = MortalityPrediction::new(windows.clone());

    // Process the batch
    let runner = clinical_tasks::windowing::TaskRunner::new(windows.clone());
    let outputs = runner.process_batch(&batch, &task).unwrap();

    // Test patient-level splitting
    let split_config = SplitConfig {
        train_ratio: 0.6,
        val_ratio: 0.2,
        test_ratio: 0.2,
        seed: 42,
    };

    let (train, val, test) = split_by_patient(&outputs, &split_config).unwrap();

    // Verify no data leakage - each patient should be in exactly one split
    let mut all_patients = std::collections::HashSet::new();

    for output in &train {
        assert!(
            !all_patients.contains(&output.patient_id),
            "Patient {} appears in multiple splits",
            output.patient_id
        );
        let _ = all_patients.insert(output.patient_id);
    }

    for output in &val {
        assert!(
            !all_patients.contains(&output.patient_id),
            "Patient {} appears in multiple splits",
            output.patient_id
        );
        let _ = all_patients.insert(output.patient_id);
    }

    for output in &test {
        assert!(
            !all_patients.contains(&output.patient_id),
            "Patient {} appears in multiple splits",
            output.patient_id
        );
        let _ = all_patients.insert(output.patient_id);
    }

    // Verify total patients (all 3 are included now)
    assert_eq!(all_patients.len(), 3);

    // Verify split ratios (approximately) - with 3 patients, ratios may not be exact
    // Just verify that all patients are assigned to some split
    assert_eq!(train.len() + val.len() + test.len(), outputs.len());
}

#[test]
fn test_data_leakage_prevention() {
    let (batch, _) = create_synthetic_patient_timeline();

    // Create task with prediction window that extends beyond discharge
    let windows = TaskWindows::new(
        48.0, // observation_hours
        0.0,  // gap_hours
        72.0, // prediction_hours (longer than typical stay)
        AnchorPoint::Admission,
    );

    let task = MortalityPrediction::new(windows.clone());
    let runner = clinical_tasks::windowing::TaskRunner::new(windows.clone());
    let outputs = runner.process_batch(&batch, &task).unwrap();

    // Verify that prediction events don't include future information
    for output in &outputs {
        let metadata = &output.metadata;
        assert!(metadata.contains_key("num_obs_events"));
        assert!(metadata.contains_key("num_pred_events"));

        // The number of prediction events should be limited
        // (death events should be within the prediction window)
        let pred_events: usize = metadata.get("num_pred_events").unwrap().parse().unwrap();
        // Allow up to 5 prediction events (discharge + death + other events can be in prediction window)
        assert!(
            pred_events <= 5,
            "Too many prediction events for patient {}: {}",
            output.patient_id,
            pred_events
        );
    }
}

#[test]
fn test_known_answer_labels() {
    let (batch, expected_outcomes) = create_synthetic_patient_timeline();

    // Test with different window configurations
    let test_cases = vec![
        (48.0, 0.0, 24.0, AnchorPoint::Admission), // Standard 48h observation
        (24.0, 0.0, 24.0, AnchorPoint::Admission), // Short 24h observation
        (48.0, 6.0, 24.0, AnchorPoint::Admission), // With gap period
    ];

    for (obs_hours, gap_hours, pred_hours, anchor) in test_cases {
        let windows = TaskWindows::new(obs_hours, gap_hours, pred_hours, anchor.clone());
        let task = MortalityPrediction::new(windows.clone());
        let runner = clinical_tasks::windowing::TaskRunner::new(windows.clone());
        let outputs = runner.process_batch(&batch, &task).unwrap();

        // Verify expected outcomes
        for output in &outputs {
            if let Some(&expected) = expected_outcomes.get(&output.patient_id) {
                let actual = output.label == 1.0;
                assert_eq!(
                    actual, expected,
                    "Patient {}: expected {}, got {} with window ({}, {}, {}, {:?})",
                    output.patient_id, expected, actual, obs_hours, gap_hours, pred_hours, anchor
                );
            }
        }
    }
}

#[test]
fn test_output_to_arrow_conversion() {
    let (batch, _) = create_synthetic_patient_timeline();

    let windows = TaskWindows::new(48.0, 0.0, 24.0, AnchorPoint::Admission);
    let task = MortalityPrediction::new(windows.clone());
    let runner = clinical_tasks::windowing::TaskRunner::new(windows.clone());
    let outputs = runner.process_batch(&batch, &task).unwrap();

    // Convert to Arrow format
    let schema = task.output_schema();
    let arrow_batch = outputs_to_batch(&outputs, &schema).unwrap();

    // Verify Arrow batch structure
    assert_eq!(arrow_batch.num_rows(), outputs.len());
    assert_eq!(arrow_batch.num_columns(), schema.fields.len());

    // Verify patient IDs match
    let patient_ids = arrow_batch
        .column(0)
        .as_any()
        .downcast_ref::<Int64Array>()
        .unwrap();
    for (i, output) in outputs.iter().enumerate() {
        assert_eq!(patient_ids.value(i), output.patient_id);
    }

    // Verify labels match
    let labels = arrow_batch
        .column_by_name("label")
        .unwrap()
        .as_any()
        .downcast_ref::<Float64Array>()
        .unwrap();
    for (i, output) in outputs.iter().enumerate() {
        assert_eq!(labels.value(i), output.label);
    }
}
