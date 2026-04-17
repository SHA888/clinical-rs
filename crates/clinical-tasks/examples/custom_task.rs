//! Example: Implementing a custom clinical prediction task.
//!
//! This example demonstrates how to implement the `TaskDefinition` trait
//! to create a custom clinical prediction task. We'll create a simple
//! "ICU Admission Prediction" task that predicts whether a patient will
//! be admitted to the ICU within 24 hours of hospital admission.
//!
//! To run this example:
//! ```bash
//! cargo run --example custom_task
//! ```

#![allow(clippy::cast_precision_loss, clippy::must_use_candidate)]

use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use clinical_tasks::{
    AnchorPoint, PatientEvent, Result, TaskDefinition, TaskError, TaskOutput, TaskWindows,
    extract_task_windows,
};
use std::collections::HashMap;

/// ICU Admission Prediction Task
///
/// Predicts whether a patient will be admitted to the ICU within 24 hours
/// of hospital admission. This is a binary classification task.
pub struct IcuAdmissionPrediction {
    windows: TaskWindows,
    schema: Schema,
}

impl IcuAdmissionPrediction {
    /// Create a new ICU admission prediction task.
    pub fn new() -> Self {
        // Define the time windows:
        // - Observation: First 6 hours after admission (look for early signs)
        // - Gap: 0 hours (immediate prediction)
        // - Prediction: 24 hours (will ICU admission happen in this window?)
        let windows = TaskWindows::new(6.0, 0.0, 24.0, AnchorPoint::Admission);

        // Define the output schema with features and label
        let schema = Schema::new(vec![
            Field::new("patient_id", DataType::Int64, false),
            Field::new("age", DataType::Float64, true),
            Field::new("gender_male", DataType::Float64, false),
            Field::new("heart_rate_max", DataType::Float64, true),
            Field::new("systolic_bp_min", DataType::Float64, true),
            Field::new("respiratory_rate_max", DataType::Float64, true),
            Field::new("has_fever", DataType::Float64, false),
            Field::new("num_abnormal_labs", DataType::Float64, false),
            Field::new("label", DataType::Float64, false),
        ]);

        Self { windows, schema }
    }

    /// Extract features from observation window events.
    fn extract_features(obs_events: &[&PatientEvent]) -> HashMap<String, f64> {
        let mut features = HashMap::new();

        // Demographics
        let mut age = None;
        let mut gender_male = 0.0;

        // Vital signs tracking
        let mut heart_rate_max: f64 = 0.0;
        let mut systolic_bp_min = f64::MAX;
        let mut respiratory_rate_max: f64 = 0.0;
        let mut has_fever: f64 = 0.0;

        // Lab tracking
        let mut num_abnormal_labs = 0.0;

        for event in obs_events {
            match event.event_type.as_str() {
                "anchor_age" => {
                    if let Some(value) = event.value_num {
                        age = Some(value);
                    }
                }
                "gender" => {
                    if event.value.as_ref().is_some_and(|v| v == "M") {
                        gender_male = 1.0;
                    }
                }
                "vital" | "vitals" => {
                    if let Some(event_id) = &event.event_id {
                        match event_id.as_str() {
                            "heart_rate" | "HR" => {
                                if let Some(val) = event.value_num {
                                    heart_rate_max = heart_rate_max.max(val);
                                }
                            }
                            "systolic_bp" | "SBP" => {
                                if let Some(val) = event.value_num {
                                    systolic_bp_min = systolic_bp_min.min(val);
                                }
                            }
                            "respiratory_rate" | "RR" => {
                                if let Some(val) = event.value_num {
                                    respiratory_rate_max = respiratory_rate_max.max(val);
                                }
                            }
                            "temperature" | "Temp" => {
                                if let Some(val) = event.value_num
                                    && val > 38.0
                                {
                                    // Fever threshold in Celsius
                                    has_fever = 1.0;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                "lab" => {
                    // Count abnormal labs (simplified heuristic)
                    if let Some(val) = event.value_num {
                        // This is a simplified check - real implementation would
                        // use reference ranges specific to each lab type
                        if !(4.0..=10.0).contains(&val) {
                            num_abnormal_labs += 1.0;
                        }
                    }
                }
                _ => {}
            }
        }

        // Insert features into map
        if let Some(age_val) = age {
            let _ = features.insert("age".to_string(), age_val);
        }
        let _ = features.insert("gender_male".to_string(), gender_male);

        // Vital signs (use sentinel values if not found)
        let _ = features.insert(
            "heart_rate_max".to_string(),
            if heart_rate_max > 0.0 {
                heart_rate_max
            } else {
                0.0
            },
        );
        let _ = features.insert(
            "systolic_bp_min".to_string(),
            if systolic_bp_min < f64::MAX {
                systolic_bp_min
            } else {
                0.0
            },
        );
        let _ = features.insert(
            "respiratory_rate_max".to_string(),
            if respiratory_rate_max > 0.0 {
                respiratory_rate_max
            } else {
                0.0
            },
        );
        let _ = features.insert("has_fever".to_string(), has_fever);
        let _ = features.insert("num_abnormal_labs".to_string(), num_abnormal_labs);

        features
    }

    /// Extract label: 1.0 if ICU admission occurs, 0.0 otherwise.
    fn extract_label(pred_events: &[&PatientEvent]) -> f64 {
        for event in pred_events {
            if event.event_type == "icu_admission" || event.event_type == "ICUAdmission" {
                return 1.0;
            }
        }
        0.0
    }
}

impl Default for IcuAdmissionPrediction {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskDefinition for IcuAdmissionPrediction {
    fn name(&self) -> &'static str {
        "icu_admission_prediction"
    }

    fn windows(&self) -> &TaskWindows {
        &self.windows
    }

    fn output_schema(&self) -> Schema {
        self.schema.clone()
    }

    fn process_patient(&self, patient_id: i64, events: &[PatientEvent]) -> Result<TaskOutput> {
        // Extract task windows using the library function
        let task_windows = extract_task_windows(events, &self.windows);

        // Process the first valid window
        if let Some((obs_events, pred_events)) = task_windows.first() {
            // Validate we have enough observation data
            if obs_events.len() < 2 {
                return Err(TaskError::Validation(format!(
                    "Insufficient observation events: got {}, need at least 2",
                    obs_events.len()
                )));
            }

            // Extract features and label
            let features = Self::extract_features(obs_events);
            let label = Self::extract_label(pred_events);

            // Add patient_id to features
            let mut final_features = features;
            let _ = final_features.insert("patient_id".to_string(), patient_id as f64);

            // Create metadata
            let mut metadata = HashMap::new();
            let _ = metadata.insert("task".to_string(), self.name().to_string());
            let _ = metadata.insert("num_obs_events".to_string(), obs_events.len().to_string());
            let _ = metadata.insert("num_pred_events".to_string(), pred_events.len().to_string());
            let _ = metadata.insert(
                "window_description".to_string(),
                "6h_obs_24h_pred".to_string(),
            );

            Ok(TaskOutput {
                patient_id,
                features: final_features,
                label,
                binary_label: true,
                metadata,
            })
        } else {
            Err(TaskError::Validation(
                "No valid task windows found".to_string(),
            ))
        }
    }

    fn validate_input(&self, batch: &RecordBatch) -> Result<()> {
        let required_columns = [
            "subject_id",
            "hadm_id",
            "charttime",
            "event_type",
            "event_id",
            "value",
            "value_num",
        ];

        let schema = batch.schema();
        for col_name in &required_columns {
            if schema.index_of(col_name).is_err() {
                return Err(TaskError::MissingColumn(col_name.to_string()));
            }
        }

        Ok(())
    }
}

fn main() {
    println!("Custom Task API Example: ICU Admission Prediction");
    println!("====================================================\n");

    // Create the custom task
    let task = IcuAdmissionPrediction::new();

    println!("Task: {}", task.name());
    println!("Windows: {:?}", task.windows());
    println!("Output schema fields:");
    for field in task.output_schema().fields() {
        println!("  - {}: {:?}", field.name(), field.data_type());
    }
    println!();

    // Example: Create synthetic patient events
    let patient_id = 12345_i64;
    let events = create_synthetic_patient_events(patient_id);

    println!(
        "Processing patient {patient_id} with {} events...",
        events.len()
    );

    // Process the patient
    match task.process_patient(patient_id, &events) {
        Ok(output) => {
            println!("\nTask output:");
            println!("  Patient ID: {}", output.patient_id);
            let label_desc = if output.label > 0.5 {
                "ICU admission"
            } else {
                "No ICU admission"
            };
            println!("  Label: {} ({label_desc})", output.label);
            println!("  Binary label: {}", output.binary_label);
            println!("  Features:");
            for (name, value) in &output.features {
                println!("    - {name}: {value:.2}");
            }
            println!("  Metadata:");
            for (key, value) in &output.metadata {
                println!("    - {key}: {value}");
            }
        }
        Err(e) => {
            eprintln!("Error processing patient: {e}");
        }
    }

    println!("\nExample completed successfully!");
    println!("\nTo implement your own task:");
    println!("1. Define a struct holding TaskWindows and Schema");
    println!("2. Implement TaskDefinition trait");
    println!("3. Use extract_task_windows() for time-based windowing");
    println!("4. Use outputs_to_batch() to convert to Arrow format");
    println!("5. Use split_by_patient() for train/val/test splitting");
}

/// Create synthetic patient events for demonstration.
#[allow(clippy::too_many_lines)]
fn create_synthetic_patient_events(patient_id: i64) -> Vec<PatientEvent> {
    use clinical_tasks::PatientEvent;

    vec![
        // Admission event (anchor point)
        PatientEvent {
            patient_id,
            admission_id: Some(98765),
            icu_stay_id: None,
            timestamp: Some(1_600_000_000_000_000), // Admission time
            event_type: "admission".to_string(),
            event_id: Some("ADMISSION".to_string()),
            value: Some("EMERGENCY".to_string()),
            value_num: None,
            units: None,
        },
        // Demographics
        PatientEvent {
            patient_id,
            admission_id: Some(98765),
            icu_stay_id: None,
            timestamp: Some(1_600_000_000_000_000),
            event_type: "anchor_age".to_string(),
            event_id: None,
            value: Some("65".to_string()),
            value_num: Some(65.0),
            units: Some("years".to_string()),
        },
        PatientEvent {
            patient_id,
            admission_id: Some(98765),
            icu_stay_id: None,
            timestamp: Some(1_600_000_000_000_000),
            event_type: "gender".to_string(),
            event_id: None,
            value: Some("M".to_string()),
            value_num: None,
            units: None,
        },
        // Vital signs (within 6-hour observation window)
        PatientEvent {
            patient_id,
            admission_id: Some(98765),
            icu_stay_id: None,
            timestamp: Some(1_600_000_001_000_000), // 1 hour later
            event_type: "vital".to_string(),
            event_id: Some("heart_rate".to_string()),
            value: Some("110".to_string()),
            value_num: Some(110.0),
            units: Some("bpm".to_string()),
        },
        PatientEvent {
            patient_id,
            admission_id: Some(98765),
            icu_stay_id: None,
            timestamp: Some(1_600_000_001_000_000),
            event_type: "vital".to_string(),
            event_id: Some("systolic_bp".to_string()),
            value: Some("95".to_string()),
            value_num: Some(95.0),
            units: Some("mmHg".to_string()),
        },
        PatientEvent {
            patient_id,
            admission_id: Some(98765),
            icu_stay_id: None,
            timestamp: Some(1_600_000_002_000_000), // 2 hours later
            event_type: "vital".to_string(),
            event_id: Some("respiratory_rate".to_string()),
            value: Some("24".to_string()),
            value_num: Some(24.0),
            units: Some("breaths/min".to_string()),
        },
        PatientEvent {
            patient_id,
            admission_id: Some(98765),
            icu_stay_id: None,
            timestamp: Some(1_600_000_003_000_000), // 3 hours later
            event_type: "vital".to_string(),
            event_id: Some("temperature".to_string()),
            value: Some("38.5".to_string()),
            value_num: Some(38.5),
            units: Some("C".to_string()),
        },
        // Lab results
        PatientEvent {
            patient_id,
            admission_id: Some(98765),
            icu_stay_id: None,
            timestamp: Some(1_600_000_004_000_000), // 4 hours later
            event_type: "lab".to_string(),
            event_id: Some("WBC".to_string()),
            value: Some("15.2".to_string()),
            value_num: Some(15.2),
            units: Some("K/uL".to_string()),
        },
        // ICU Admission (within 24-hour prediction window)
        PatientEvent {
            patient_id,
            admission_id: Some(98765),
            icu_stay_id: Some(11111),
            timestamp: Some(1_600_000_020_000_000), // 20 hours later
            event_type: "icu_admission".to_string(),
            event_id: Some("ICU_ADMISSION".to_string()),
            value: Some("ICU".to_string()),
            value_num: None,
            units: None,
        },
    ]
}
