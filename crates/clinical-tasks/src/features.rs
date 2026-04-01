//! Feature extraction from clinical event streams

use crate::types::{
    PatientEvent, Result, SplitConfig, TaskDefinition, TaskError, TaskOutput, TaskWindows,
};
use crate::windowing::extract_task_windows;
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// In-hospital mortality prediction task.
pub struct MortalityPrediction {
    windows: TaskWindows,
    schema: Schema,
}

impl MortalityPrediction {
    /// Create a new mortality prediction task.
    #[must_use]
    pub fn new(windows: TaskWindows) -> Self {
        let schema = Schema::new(vec![
            Field::new("patient_id", DataType::Int64, false),
            Field::new("admission_id", DataType::Int64, true),
            Field::new("age", DataType::Float64, true),
            Field::new("gender_male", DataType::Float64, false),
            Field::new("num_diagnoses", DataType::Float64, false),
            Field::new("num_procedures", DataType::Float64, false),
            Field::new("num_medications", DataType::Float64, false),
            Field::new("num_labs", DataType::Float64, false),
            Field::new("abnormal_labs_ratio", DataType::Float64, false),
            Field::new("label", DataType::Float64, false),
        ]);

        Self { windows, schema }
    }

    /// Extract features from observation window events.
    fn extract_features(obs_events: &[&PatientEvent]) -> HashMap<String, f64> {
        let mut features = HashMap::new();

        let mut num_diagnoses = 0.0;
        let mut num_procedures = 0.0;
        let mut num_medications = 0.0;
        let mut num_labs = 0.0;
        let mut abnormal_labs = 0.0;
        let mut age = None;
        let mut gender_male = 0.0;
        let mut admission_id = None;

        for event in obs_events {
            match event.event_type.as_str() {
                "diagnosis" => num_diagnoses += 1.0,
                "procedure" => num_procedures += 1.0,
                "medication_start" | "medication_stop" => num_medications += 1.0,
                "lab" => {
                    num_labs += 1.0;
                    // Simple heuristic: consider abnormal if value is outside normal range
                    if let Some(value) = event.value_num
                        && !(5.0..=200.0).contains(&value)
                    {
                        abnormal_labs += 1.0;
                    }
                }
                "gender" => {
                    if event.value.as_ref().is_some_and(|v| v == "M") {
                        gender_male = 1.0;
                    }
                }
                "anchor_age" => {
                    if let Some(value) = event.value_num {
                        age = Some(value);
                    }
                }
                "admission" => {
                    admission_id = event.admission_id;
                }
                _ => {}
            }
        }

        let _ = features.insert("num_diagnoses".to_string(), num_diagnoses);
        let _ = features.insert("num_procedures".to_string(), num_procedures);
        let _ = features.insert("num_medications".to_string(), num_medications);
        let _ = features.insert("num_labs".to_string(), num_labs);
        let _ = features.insert(
            "abnormal_labs_ratio".to_string(),
            if num_labs > 0.0 {
                abnormal_labs / num_labs
            } else {
                0.0
            },
        );

        if let Some(age_val) = age {
            let _ = features.insert("age".to_string(), age_val);
        }

        let _ = features.insert("gender_male".to_string(), gender_male);

        if let Some(adm_id) = admission_id {
            let _ = features.insert("admission_id".to_string(), adm_id as f64);
        }

        features
    }

    /// Determine if patient died during hospitalization.
    fn extract_label(pred_events: &[&PatientEvent]) -> f64 {
        for event in pred_events {
            if event.event_type == "death" {
                return 1.0;
            }
        }
        0.0
    }
}

impl TaskDefinition for MortalityPrediction {
    fn name(&self) -> &'static str {
        "mortality_prediction"
    }

    fn windows(&self) -> &TaskWindows {
        &self.windows
    }

    fn output_schema(&self) -> Schema {
        self.schema.clone()
    }

    /// Process a patient and extract features and label.
    fn process_patient(&self, patient_id: i64, events: &[PatientEvent]) -> Result<TaskOutput> {
        let task_windows = extract_task_windows(events, &self.windows);

        // For now, process the first window (could extend to multiple windows)
        if let Some((obs_events, pred_events)) = task_windows.first() {
            // Exclude patients with very short observation windows
            if obs_events.len() < 2 {
                return Err(TaskError::Validation(format!(
                    "Insufficient observation events: got {}, need at least 2",
                    obs_events.len()
                )));
            }

            assert!(
                pred_events.len() <= 5,
                "Too many prediction events for patient {}: {}",
                patient_id,
                pred_events.len()
            );

            let features = Self::extract_features(obs_events);
            let label = Self::extract_label(pred_events);

            // Add patient_id to features
            let mut final_features = features;
            let _ = final_features.insert("patient_id".to_string(), patient_id as f64);

            let mut metadata = HashMap::new();
            let _ = metadata.insert("task".to_string(), self.name().to_string());
            let _ = metadata.insert("num_obs_events".to_string(), obs_events.len().to_string());
            let _ = metadata.insert("num_pred_events".to_string(), pred_events.len().to_string());

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
            "stay_id",
            "charttime",
            "event_type",
            "event_id",
            "value",
            "value_num",
            "units",
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

/// Split patients into train/validation/test sets.
/// Split task outputs by patient for train/val/test.
///
/// # Errors
/// Returns an error if the split configuration is invalid.
pub fn split_by_patient(
    outputs: &[TaskOutput],
    config: &SplitConfig,
) -> Result<(Vec<TaskOutput>, Vec<TaskOutput>, Vec<TaskOutput>)> {
    config.validate()?;

    let mut train = Vec::new();
    let mut val = Vec::new();
    let mut test = Vec::new();

    for output in outputs {
        let mut hasher = DefaultHasher::new();
        output.patient_id.hash(&mut hasher);
        let hash = hasher.finish();

        // Use hash to determine split
        let ratio = (hash % 100) as f64 / 100.0;

        if ratio < config.train_ratio {
            train.push(output.clone());
        } else if ratio < config.train_ratio + config.val_ratio {
            val.push(output.clone());
        } else {
            test.push(output.clone());
        }
    }

    Ok((train, val, test))
}

/// Convert task outputs to Arrow `RecordBatch`.
///
/// # Errors
/// Returns an error if the batch creation fails or schema is invalid.
pub fn outputs_to_batch(outputs: &[TaskOutput], schema: &Schema) -> Result<RecordBatch> {
    use arrow::array::{Float64Array, Int64Array};

    // Build arrays in schema order
    let mut arrays: Vec<arrow::array::ArrayRef> = Vec::new();

    for field in &schema.fields {
        let field_name = field.name();
        let data_type = field.data_type();

        match (field_name.as_str(), data_type) {
            ("patient_id", DataType::Int64) => {
                let patient_ids: Vec<i64> = outputs.iter().map(|o| o.patient_id).collect();
                arrays.push(std::sync::Arc::new(Int64Array::from(patient_ids)));
            }
            ("admission_id", DataType::Int64) => {
                let admission_ids: Vec<Option<i64>> = outputs
                    .iter()
                    .map(|o| o.features.get("admission_id").map(|v| *v as i64))
                    .collect();
                arrays.push(std::sync::Arc::new(Int64Array::from(admission_ids)));
            }
            ("label", DataType::Float64) => {
                let labels: Vec<f64> = outputs.iter().map(|o| o.label).collect();
                arrays.push(std::sync::Arc::new(Float64Array::from(labels)));
            }
            (_, DataType::Float64) => {
                // Feature field - get from output.features or default to 0.0
                let values: Vec<f64> = outputs
                    .iter()
                    .map(|o| o.features.get(field_name).copied().unwrap_or(0.0))
                    .collect();
                arrays.push(std::sync::Arc::new(Float64Array::from(values)));
            }
            _ => {
                return Err(TaskError::Execution(format!(
                    "Unsupported field type in schema: {field_name} {data_type:?}"
                )));
            }
        }
    }

    RecordBatch::try_new(std::sync::Arc::new(schema.clone()), arrays).map_err(TaskError::Arrow)
}
