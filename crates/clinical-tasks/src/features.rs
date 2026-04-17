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

/// 30-day readmission prediction task (binary classification).
///
/// Predicts whether a patient will be readmitted within 30 days of discharge.
/// Anchor point: Discharge.
pub struct ReadmissionPrediction {
    windows: TaskWindows,
    schema: Schema,
    /// Readmission window in days (default: 30)
    readmission_days: i64,
}

impl ReadmissionPrediction {
    /// Create a new 30-day readmission prediction task.
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
            Field::new("length_of_stay_hours", DataType::Float64, false),
            Field::new("had_surgery", DataType::Float64, false),
            Field::new("label", DataType::Float64, false),
        ]);

        Self {
            windows,
            schema,
            readmission_days: 30,
        }
    }

    /// Set custom readmission window (default is 30 days).
    #[must_use]
    pub const fn with_readmission_days(mut self, days: i64) -> Self {
        self.readmission_days = days;
        self
    }

    /// Extract features from observation window events.
    fn extract_features(obs_events: &[&PatientEvent]) -> HashMap<String, f64> {
        let mut features = HashMap::new();

        let mut num_diagnoses = 0.0;
        let mut num_procedures = 0.0;
        let mut num_medications = 0.0;
        let mut num_labs = 0.0;
        let mut age = None;
        let mut gender_male = 0.0;
        let mut admission_id = None;
        let mut had_surgery = 0.0;
        let mut los_hours = 0.0;

        for event in obs_events {
            match event.event_type.as_str() {
                "diagnosis" => num_diagnoses += 1.0,
                "procedure" => {
                    num_procedures += 1.0;
                    // Check for surgery-related procedure codes
                    if let Some(event_id) = &event.event_id {
                        if event_id.starts_with("00")
                            || event_id.starts_with('1')
                            || event_id.starts_with('2')
                            || event_id.starts_with("30")
                            || event_id.starts_with("31")
                            || event_id.starts_with("32")
                            || event_id.starts_with("33")
                            || event_id.starts_with("34")
                            || event_id.starts_with("35")
                            || event_id.starts_with("36")
                            || event_id.starts_with("37")
                            || event_id.starts_with("38")
                            || event_id.starts_with("39")
                            || event_id.starts_with("40")
                            || event_id.starts_with("41")
                            || event_id.starts_with("42")
                            || event_id.starts_with("43")
                            || event_id.starts_with("44")
                            || event_id.starts_with("45")
                            || event_id.starts_with("46")
                            || event_id.starts_with("47")
                            || event_id.starts_with("48")
                            || event_id.starts_with("49")
                            || event_id.starts_with("50")
                            || event_id.starts_with("51")
                            || event_id.starts_with("52")
                            || event_id.starts_with("53")
                            || event_id.starts_with("54")
                        {
                            had_surgery = 1.0;
                        }
                    }
                }
                "medication_start" | "medication_stop" => num_medications += 1.0,
                "lab" => num_labs += 1.0,
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
                "discharge" => {
                    // Track discharge time for LOS calculation
                    if let Some(ts) = event.timestamp {
                        los_hours = ts as f64 / 3_600_000_000.0; // Convert micros to hours
                    }
                }
                _ => {}
            }
        }

        let _ = features.insert("num_diagnoses".to_string(), num_diagnoses);
        let _ = features.insert("num_procedures".to_string(), num_procedures);
        let _ = features.insert("num_medications".to_string(), num_medications);
        let _ = features.insert("num_labs".to_string(), num_labs);
        let _ = features.insert("length_of_stay_hours".to_string(), los_hours);
        let _ = features.insert("had_surgery".to_string(), had_surgery);

        if let Some(age_val) = age {
            let _ = features.insert("age".to_string(), age_val);
        }

        let _ = features.insert("gender_male".to_string(), gender_male);

        if let Some(adm_id) = admission_id {
            let _ = features.insert("admission_id".to_string(), adm_id as f64);
        }

        features
    }

    /// Determine if patient was readmitted within window.
    fn extract_label(pred_events: &[&PatientEvent], current_admission_id: Option<i64>) -> f64 {
        // The prediction window already enforces the time bound through TaskWindows
        // configuration based on readmission_days

        for event in pred_events {
            if event.event_type == "admission" {
                // Check if this is a different admission (readmission)
                if let Some(event_adm_id) = event.admission_id {
                    if current_admission_id.is_none_or(|id| id != event_adm_id) {
                        // Valid readmission found within prediction window
                        return 1.0;
                    }
                }
            }
        }
        0.0
    }
}

impl TaskDefinition for ReadmissionPrediction {
    fn name(&self) -> &'static str {
        "readmission_prediction"
    }

    fn windows(&self) -> &TaskWindows {
        &self.windows
    }

    fn output_schema(&self) -> Schema {
        self.schema.clone()
    }

    fn process_patient(&self, patient_id: i64, events: &[PatientEvent]) -> Result<TaskOutput> {
        let task_windows = extract_task_windows(events, &self.windows);

        if let Some((obs_events, pred_events)) = task_windows.first() {
            if obs_events.len() < 2 {
                return Err(TaskError::Validation(format!(
                    "Insufficient observation events: got {}, need at least 2",
                    obs_events.len()
                )));
            }

            let features = Self::extract_features(obs_events);
            let current_adm_id = features.get("admission_id").map(|v| *v as i64);
            let label = Self::extract_label(pred_events, current_adm_id);

            let mut final_features = features;
            let _ = final_features.insert("patient_id".to_string(), patient_id as f64);

            let mut metadata = HashMap::new();
            let _ = metadata.insert("task".to_string(), self.name().to_string());
            let _ = metadata.insert("num_obs_events".to_string(), obs_events.len().to_string());
            let _ = metadata.insert("num_pred_events".to_string(), pred_events.len().to_string());
            let _ = metadata.insert(
                "readmission_days".to_string(),
                self.readmission_days.to_string(),
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

/// Length of stay prediction task (multiclass bucketed + regression variant).
///
/// Predicts the length of stay category or continuous value.
/// Anchor point: Admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LosTarget {
    /// Multiclass classification into buckets
    Buckets,
    /// Regression for continuous LOS in hours
    Regression,
}

/// LOS buckets for multiclass classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LosBucket {
    /// 0-1 days
    Short = 0,
    /// 2-3 days
    Medium = 1,
    /// 4-7 days
    Long = 2,
    /// 8+ days
    Extended = 3,
}

impl LosBucket {
    /// Convert hours to bucket.
    #[must_use]
    pub fn from_hours(hours: f64) -> Self {
        match hours {
            h if h <= 24.0 => Self::Short,
            h if h <= 72.0 => Self::Medium,
            h if h <= 168.0 => Self::Long,
            _ => Self::Extended,
        }
    }

    /// Get bucket as f64 label.
    #[must_use]
    pub const fn as_f64(self) -> f64 {
        self as i32 as f64
    }
}

/// Length of stay prediction task.
pub struct LengthOfStayPrediction {
    windows: TaskWindows,
    schema: Schema,
    target: LosTarget,
}

impl LengthOfStayPrediction {
    /// Create a new LOS prediction task with specified target type.
    #[must_use]
    pub fn new(windows: TaskWindows, target: LosTarget) -> Self {
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
            Field::new("emergency_admission", DataType::Float64, false),
            Field::new("label", DataType::Float64, false),
        ]);

        Self {
            windows,
            schema,
            target,
        }
    }

    /// Create multiclass bucketed variant.
    #[must_use]
    pub fn buckets(windows: TaskWindows) -> Self {
        Self::new(windows, LosTarget::Buckets)
    }

    /// Create regression variant.
    #[must_use]
    pub fn regression(windows: TaskWindows) -> Self {
        Self::new(windows, LosTarget::Regression)
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
        let mut emergency_admission = 0.0;

        for event in obs_events {
            match event.event_type.as_str() {
                "diagnosis" => num_diagnoses += 1.0,
                "procedure" => num_procedures += 1.0,
                "medication_start" | "medication_stop" => num_medications += 1.0,
                "lab" => {
                    num_labs += 1.0;
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
                    // Check admission type if available
                    if let Some(event_id) = &event.event_id {
                        if event_id == "EMERGENCY" || event_id == "URGENT" {
                            emergency_admission = 1.0;
                        }
                    }
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
        let _ = features.insert("emergency_admission".to_string(), emergency_admission);

        if let Some(age_val) = age {
            let _ = features.insert("age".to_string(), age_val);
        }

        let _ = features.insert("gender_male".to_string(), gender_male);

        if let Some(adm_id) = admission_id {
            let _ = features.insert("admission_id".to_string(), adm_id as f64);
        }

        features
    }

    /// Extract LOS label from prediction window.
    ///
    /// # Note
    /// Timestamps are expected to be in **microseconds** since epoch.
    /// The calculation converts to hours by dividing by `3_600_000_000`.
    fn extract_label(&self, pred_events: &[&PatientEvent]) -> f64 {
        // Look for discharge event to compute actual LOS
        let mut admission_ts = None;
        let mut discharge_ts = None;

        for event in pred_events {
            if event.event_type == "admission" {
                if admission_ts.is_none() {
                    admission_ts = event.timestamp;
                }
            } else if event.event_type == "discharge" {
                discharge_ts = event.timestamp;
            }
        }

        if let (Some(adm_ts), Some(disch_ts)) = (admission_ts, discharge_ts) {
            let los_hours = (disch_ts - adm_ts) as f64 / 3_600_000_000.0;
            match self.target {
                LosTarget::Buckets => LosBucket::from_hours(los_hours).as_f64(),
                LosTarget::Regression => los_hours,
            }
        } else {
            // Default: couldn't determine LOS (treat as censored/unknown)
            match self.target {
                LosTarget::Buckets => LosBucket::Extended.as_f64(),
                LosTarget::Regression => 168.0, // 7 days as default
            }
        }
    }
}

impl TaskDefinition for LengthOfStayPrediction {
    fn name(&self) -> &'static str {
        match self.target {
            LosTarget::Buckets => "length_of_stay_buckets",
            LosTarget::Regression => "length_of_stay_regression",
        }
    }

    fn windows(&self) -> &TaskWindows {
        &self.windows
    }

    fn output_schema(&self) -> Schema {
        self.schema.clone()
    }

    fn process_patient(&self, patient_id: i64, events: &[PatientEvent]) -> Result<TaskOutput> {
        let task_windows = extract_task_windows(events, &self.windows);

        if let Some((obs_events, pred_events)) = task_windows.first() {
            if obs_events.len() < 2 {
                return Err(TaskError::Validation(format!(
                    "Insufficient observation events: got {}, need at least 2",
                    obs_events.len()
                )));
            }

            let features = Self::extract_features(obs_events);
            let label = self.extract_label(pred_events);

            let mut final_features = features;
            let _ = final_features.insert("patient_id".to_string(), patient_id as f64);

            let mut metadata = HashMap::new();
            let _ = metadata.insert("task".to_string(), self.name().to_string());
            let _ = metadata.insert("num_obs_events".to_string(), obs_events.len().to_string());
            let _ = metadata.insert("num_pred_events".to_string(), pred_events.len().to_string());
            let _ = metadata.insert(
                "target_type".to_string(),
                match self.target {
                    LosTarget::Buckets => "buckets".to_string(),
                    LosTarget::Regression => "regression".to_string(),
                },
            );

            Ok(TaskOutput {
                patient_id,
                features: final_features,
                label,
                binary_label: matches!(self.target, LosTarget::Buckets),
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

/// Drug recommendation task (multi-label classification).
///
/// Recommends appropriate medications based on patient history.
/// Returns a multi-label vector indicating which drug classes to prescribe.
/// Optional DDI (drug-drug interaction) matrix for safety checking.
/// Anchor point: Admission.
pub struct DrugRecommendation {
    windows: TaskWindows,
    schema: Schema,
    /// Number of drug classes (output dimension)
    num_drug_classes: usize,
    /// Optional DDI matrix: (`drug_i`, `drug_j`) -> interaction severity (0-1)
    ddi_matrix: Option<HashMap<(usize, usize), f64>>,
}

/// Drug class indices for common medication categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DrugClass {
    /// Antibiotics (e.g., vancomycin, piperacillin)
    Antibiotics = 0,
    /// Anticoagulants (e.g., heparin, warfarin)
    Anticoagulants = 1,
    /// Antihypertensives (e.g., ACE inhibitors, beta blockers)
    Antihypertensives = 2,
    /// Analgesics (e.g., opioids, NSAIDs)
    Analgesics = 3,
    /// Sedatives (e.g., propofol, midazolam)
    Sedatives = 4,
    /// Vasopressors (e.g., norepinephrine, vasopressin)
    Vasopressors = 5,
    /// Diuretics (e.g., furosemide)
    Diuretics = 6,
    /// Insulin / diabetes meds
    DiabetesMeds = 7,
    /// Bronchodilators (e.g., albuterol)
    Bronchodilators = 8,
    /// Corticosteroids
    Corticosteroids = 9,
}

impl DrugClass {
    /// Total number of drug classes.
    pub const COUNT: usize = 10;

    /// Get all drug classes as a vector.
    #[must_use]
    pub fn all() -> Vec<Self> {
        vec![
            Self::Antibiotics,
            Self::Anticoagulants,
            Self::Antihypertensives,
            Self::Analgesics,
            Self::Sedatives,
            Self::Vasopressors,
            Self::Diuretics,
            Self::DiabetesMeds,
            Self::Bronchodilators,
            Self::Corticosteroids,
        ]
    }

    /// Get drug class index.
    #[must_use]
    pub const fn idx(self) -> usize {
        self as usize
    }
}

impl DrugRecommendation {
    /// Create a new drug recommendation task.
    #[must_use]
    pub fn new(windows: TaskWindows) -> Self {
        let num_classes = DrugClass::COUNT;
        let mut fields = vec![
            Field::new("patient_id", DataType::Int64, false),
            Field::new("admission_id", DataType::Int64, true),
            Field::new("age", DataType::Float64, true),
            Field::new("gender_male", DataType::Float64, false),
            Field::new("num_diagnoses", DataType::Float64, false),
            Field::new("num_active_meds", DataType::Float64, false),
            Field::new("has_infection", DataType::Float64, false),
            Field::new("has_pain", DataType::Float64, false),
            Field::new("has_hypertension", DataType::Float64, false),
            Field::new("has_diabetes", DataType::Float64, false),
            Field::new("has_copd", DataType::Float64, false),
            Field::new("renal_impairment", DataType::Float64, false),
        ];

        // Add label fields for each drug class
        for i in 0..num_classes {
            fields.push(Field::new(
                format!("drug_class_{i}"),
                DataType::Float64,
                false,
            ));
        }

        let schema = Schema::new(fields);

        Self {
            windows,
            schema,
            num_drug_classes: num_classes,
            ddi_matrix: None,
        }
    }

    /// Set a custom DDI matrix.
    #[must_use]
    pub fn with_ddi_matrix(mut self, ddi: HashMap<(usize, usize), f64>) -> Self {
        self.ddi_matrix = Some(ddi);
        self
    }

    /// Get the number of drug classes.
    #[must_use]
    pub const fn num_drug_classes(&self) -> usize {
        self.num_drug_classes
    }

    /// Extract features from observation window events.
    fn extract_features(obs_events: &[&PatientEvent]) -> HashMap<String, f64> {
        let mut features = HashMap::new();

        let mut num_diagnoses = 0.0;
        let mut num_active_meds = 0.0;
        let mut age = None;
        let mut gender_male = 0.0;
        let mut admission_id = None;

        // Clinical indicators for drug recommendations
        let mut has_infection = 0.0;
        let mut has_pain = 0.0;
        let mut has_hypertension = 0.0;
        let mut has_diabetes = 0.0;
        let mut has_copd = 0.0;
        let mut renal_impairment = 0.0;

        for event in obs_events {
            match event.event_type.as_str() {
                "diagnosis" => {
                    num_diagnoses += 1.0;
                    // Check for specific conditions based on ICD codes
                    if let Some(code) = &event.event_id {
                        let code_upper = code.to_uppercase();
                        // Infection codes (ICD-9: 001-139, ICD-10: A00-B99)
                        if code_upper.starts_with('A')
                            || code_upper.starts_with('B')
                            || code_upper.starts_with("038")
                            || code_upper.starts_with("486")
                            || code_upper.starts_with("507")
                        {
                            has_infection = 1.0;
                        }
                        // Pain codes
                        if code_upper.starts_with("338") || code_upper.starts_with("7809") {
                            has_pain = 1.0;
                        }
                        // Hypertension (ICD-9: 401-405, ICD-10: I10-I15)
                        if code_upper.starts_with("401")
                            || code_upper.starts_with("402")
                            || code_upper.starts_with("403")
                            || code_upper.starts_with("404")
                            || code_upper.starts_with("405")
                            || code_upper.starts_with("I10")
                            || code_upper.starts_with("I11")
                            || code_upper.starts_with("I12")
                            || code_upper.starts_with("I13")
                            || code_upper.starts_with("I14")
                            || code_upper.starts_with("I15")
                        {
                            has_hypertension = 1.0;
                        }
                        // Diabetes (ICD-9: 250, ICD-10: E10-E14)
                        if code_upper.starts_with("250") || code_upper.starts_with('E') {
                            has_diabetes = 1.0;
                        }
                        // COPD (ICD-9: 490-496, ICD-10: J40-J47)
                        if code_upper.starts_with("490")
                            || code_upper.starts_with("491")
                            || code_upper.starts_with("492")
                            || code_upper.starts_with("493")
                            || code_upper.starts_with("494")
                            || code_upper.starts_with("495")
                            || code_upper.starts_with("496")
                            || code_upper.starts_with('J')
                        {
                            has_copd = 1.0;
                        }
                        // Renal impairment (ICD-9: 580-589, ICD-10: N17-N19)
                        if code_upper.starts_with("580")
                            || code_upper.starts_with("581")
                            || code_upper.starts_with("582")
                            || code_upper.starts_with("583")
                            || code_upper.starts_with("584")
                            || code_upper.starts_with("585")
                            || code_upper.starts_with("586")
                            || code_upper.starts_with("587")
                            || code_upper.starts_with("588")
                            || code_upper.starts_with("589")
                            || code_upper.starts_with("N17")
                            || code_upper.starts_with("N18")
                            || code_upper.starts_with("N19")
                        {
                            renal_impairment = 1.0;
                        }
                    }
                }
                "medication_start" | "medication" => {
                    num_active_meds += 1.0;
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
                "lab" => {
                    // Check for creatinine elevation indicating renal impairment
                    if let Some(event_id) = &event.event_id {
                        if event_id == "CREATININE" || event_id == "creatinine" {
                            if let Some(val) = event.value_num {
                                if val > 1.5 {
                                    // > 1.5 mg/dL suggests impairment
                                    renal_impairment = 1.0;
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        let _ = features.insert("num_diagnoses".to_string(), num_diagnoses);
        let _ = features.insert("num_active_meds".to_string(), num_active_meds);
        let _ = features.insert("has_infection".to_string(), has_infection);
        let _ = features.insert("has_pain".to_string(), has_pain);
        let _ = features.insert("has_hypertension".to_string(), has_hypertension);
        let _ = features.insert("has_diabetes".to_string(), has_diabetes);
        let _ = features.insert("has_copd".to_string(), has_copd);
        let _ = features.insert("renal_impairment".to_string(), renal_impairment);

        if let Some(age_val) = age {
            let _ = features.insert("age".to_string(), age_val);
        }

        let _ = features.insert("gender_male".to_string(), gender_male);

        if let Some(adm_id) = admission_id {
            let _ = features.insert("admission_id".to_string(), adm_id as f64);
        }

        features
    }

    /// Extract multi-label drug recommendations from prediction window.
    /// Returns a vector of drug class indices that should be prescribed.
    fn extract_labels(&self, pred_events: &[&PatientEvent]) -> Vec<f64> {
        let mut labels = vec![0.0; self.num_drug_classes];

        for event in pred_events {
            if event.event_type == "medication_start" || event.event_type == "medication" {
                if let Some(drug_code) = &event.event_id {
                    // Map drug codes to drug classes
                    // This is a simplified mapping - real implementation would use
                    // a comprehensive drug class database
                    let drug_upper = drug_code.to_uppercase();

                    // Antibiotics
                    if drug_upper.contains("VANCOMYCIN")
                        || drug_upper.contains("PIPERACILLIN")
                        || drug_upper.contains("CEF")
                        || drug_upper.contains("MEROPENEM")
                        || drug_upper.contains("AZITHROMYCIN")
                    {
                        labels[DrugClass::Antibiotics.idx()] = 1.0;
                    }

                    // Anticoagulants
                    // Check specific drug names to avoid substring matches (e.g., ENOXAPARIN contains HEPARIN)
                    let is_anticoagulant = [
                        "HEPARIN",
                        "WARFARIN",
                        "ENOXAPARIN",
                        "APIXABAN",
                        "RIVAROXABAN",
                        "DALTEPARIN",
                        "TINZAPARIN",
                        "FONDAPARINUX",
                        "DABIGATRAN",
                        "EDOXABAN",
                    ]
                    .iter()
                    .any(|&drug| drug_upper == drug || drug_upper.starts_with(drug));
                    if is_anticoagulant {
                        labels[DrugClass::Anticoagulants.idx()] = 1.0;
                    }

                    // Antihypertensives
                    if drug_upper.contains("LISINOPRIL")
                        || drug_upper.contains("METOPROLOL")
                        || drug_upper.contains("AMLODIPINE")
                        || drug_upper.contains("LOSARTAN")
                        || drug_upper.contains("PROPRANOLOL")
                    {
                        labels[DrugClass::Antihypertensives.idx()] = 1.0;
                    }

                    // Analgesics
                    if drug_upper.contains("FENTANYL")
                        || drug_upper.contains("MORPHINE")
                        || drug_upper.contains("IBUPROFEN")
                        || drug_upper.contains("ACETAMINOPHEN")
                        || drug_upper.contains("OXYCODONE")
                    {
                        labels[DrugClass::Analgesics.idx()] = 1.0;
                    }

                    // Sedatives
                    if drug_upper.contains("PROPOFOL")
                        || drug_upper.contains("MIDAZOLAM")
                        || drug_upper.contains("DEXMEDETOMIDINE")
                        || drug_upper.contains("LORAZEPAM")
                    {
                        labels[DrugClass::Sedatives.idx()] = 1.0;
                    }

                    // Vasopressors
                    if drug_upper.contains("NOREPINEPHRINE")
                        || drug_upper.contains("EPINEPHRINE")
                        || drug_upper.contains("PHENYLEPHRINE")
                        || drug_upper.contains("VASOPRESSIN")
                        || drug_upper.contains("DOPAMINE")
                    {
                        labels[DrugClass::Vasopressors.idx()] = 1.0;
                    }

                    // Diuretics
                    if drug_upper.contains("FUROSEMIDE")
                        || drug_upper.contains("HYDROCHLOROTHIAZIDE")
                        || drug_upper.contains("SPIRONOLACTONE")
                    {
                        labels[DrugClass::Diuretics.idx()] = 1.0;
                    }

                    // Diabetes meds
                    if drug_upper.contains("INSULIN")
                        || drug_upper.contains("METFORMIN")
                        || drug_upper.contains("GLIPIZIDE")
                    {
                        labels[DrugClass::DiabetesMeds.idx()] = 1.0;
                    }

                    // Bronchodilators
                    if drug_upper.contains("ALBUTEROL")
                        || drug_upper.contains("IPRATROPIUM")
                        || drug_upper.contains("SALMETEROL")
                    {
                        labels[DrugClass::Bronchodilators.idx()] = 1.0;
                    }

                    // Corticosteroids
                    if drug_upper.contains("PREDNISONE")
                        || drug_upper.contains("METHYLPREDNISOLONE")
                        || drug_upper.contains("DEXAMETHASONE")
                        || drug_upper.contains("HYDROCORTISONE")
                    {
                        labels[DrugClass::Corticosteroids.idx()] = 1.0;
                    }
                }
            }
        }

        // Apply DDI safety checks if matrix is provided
        if let Some(ref ddi) = self.ddi_matrix {
            labels = self.apply_ddi_safety(&labels, ddi);
        }

        labels
    }

    /// Apply DDI safety constraints to predicted labels.
    fn apply_ddi_safety(&self, labels: &[f64], ddi: &HashMap<(usize, usize), f64>) -> Vec<f64> {
        let mut safe_labels = labels.to_owned();

        // Check for high-severity interactions (>0.7) between prescribed drugs
        // Iterate through upper triangle of interaction matrix
        #[allow(clippy::needless_range_loop)]
        for i in 0..self.num_drug_classes {
            if safe_labels[i] > 0.5 {
                for j in (i + 1)..self.num_drug_classes {
                    if safe_labels[j] > 0.5 {
                        if let Some(&severity) = ddi.get(&(i, j)).or_else(|| ddi.get(&(j, i))) {
                            if severity > 0.7 {
                                // High interaction - keep the one with lower index
                                // (simplistic resolution - real impl would use clinical rules)
                                safe_labels[j] = 0.0;
                            }
                        }
                    }
                }
            }
        }

        safe_labels
    }
}

impl TaskDefinition for DrugRecommendation {
    fn name(&self) -> &'static str {
        "drug_recommendation"
    }

    fn windows(&self) -> &TaskWindows {
        &self.windows
    }

    fn output_schema(&self) -> Schema {
        self.schema.clone()
    }

    fn process_patient(&self, patient_id: i64, events: &[PatientEvent]) -> Result<TaskOutput> {
        let task_windows = extract_task_windows(events, &self.windows);

        if let Some((obs_events, pred_events)) = task_windows.first() {
            if obs_events.len() < 2 {
                return Err(TaskError::Validation(format!(
                    "Insufficient observation events: got {}, need at least 2",
                    obs_events.len()
                )));
            }

            let mut features = Self::extract_features(obs_events);
            let drug_labels = self.extract_labels(pred_events);

            // Add drug class labels as features
            for (i, label) in drug_labels.iter().enumerate() {
                let _ = features.insert(format!("drug_class_{i}"), *label);
            }

            let _ = features.insert("patient_id".to_string(), patient_id as f64);

            // Multi-label "label" is the sum of all drug classes (for simple metric)
            // or we could store a vector - here we use count as scalar summary
            let label_sum: f64 = drug_labels.iter().sum();

            let mut metadata = HashMap::new();
            let _ = metadata.insert("task".to_string(), self.name().to_string());
            let _ = metadata.insert("num_obs_events".to_string(), obs_events.len().to_string());
            let _ = metadata.insert("num_pred_events".to_string(), pred_events.len().to_string());
            let _ = metadata.insert(
                "num_drug_classes".to_string(),
                self.num_drug_classes.to_string(),
            );
            let _ = metadata.insert("prescribed_count".to_string(), label_sum.to_string());

            Ok(TaskOutput {
                patient_id,
                features,
                label: label_sum,
                binary_label: false, // Multi-label, not binary
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
