//! Time-based windowing of clinical events

use crate::types::{AnchorPoint, PatientEvent, Result, TaskError, TaskWindows};
use arrow::array::{Array, Float64Array, Int64Array, StringArray, TimestampMicrosecondArray};
use arrow::record_batch::RecordBatch;
use std::collections::HashMap;

/// Groups events by patient and sorts them by timestamp.
///
/// # Errors
/// Returns an error if required columns are missing from the batch.
pub fn group_and_sort_events(batch: &RecordBatch) -> Result<HashMap<i64, Vec<PatientEvent>>> {
    let mut patient_events: HashMap<i64, Vec<PatientEvent>> = HashMap::new();

    // Extract arrays from the batch
    let subject_ids = batch
        .column(0)
        .as_any()
        .downcast_ref::<Int64Array>()
        .ok_or_else(|| TaskError::MissingColumn("subject_id".to_string()))?;

    let hadm_ids = batch.column(1).as_any().downcast_ref::<Int64Array>();

    let stay_ids = batch.column(2).as_any().downcast_ref::<Int64Array>();

    let charttimes = batch
        .column(3)
        .as_any()
        .downcast_ref::<TimestampMicrosecondArray>();

    let event_types = batch
        .column(4)
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| TaskError::MissingColumn("event_type".to_string()))?;

    let event_ids = batch.column(5).as_any().downcast_ref::<StringArray>();

    let values = batch.column(6).as_any().downcast_ref::<StringArray>();

    let value_nums = batch.column(7).as_any().downcast_ref::<Float64Array>();

    let units = batch.column(8).as_any().downcast_ref::<StringArray>();

    // Process each row
    for i in 0..batch.num_rows() {
        let patient_id = subject_ids.value(i);

        let event = PatientEvent {
            patient_id,
            admission_id: hadm_ids.map(|arr| arr.value(i)),
            icu_stay_id: stay_ids.map(|arr| arr.value(i)),
            timestamp: charttimes.map(|arr| arr.value(i)),
            event_type: event_types.value(i).to_string(),
            event_id: event_ids.map(|arr| arr.value(i).to_string()),
            value: values.map(|arr| arr.value(i).to_string()),
            value_num: value_nums.map(|arr| arr.value(i)),
            units: units.map(|arr| arr.value(i).to_string()),
        };

        patient_events.entry(patient_id).or_default().push(event);
    }

    // Sort events by timestamp for each patient
    for events in patient_events.values_mut() {
        events.sort_by(|a, b| match (a.timestamp, b.timestamp) {
            (Some(a_ts), Some(b_ts)) => a_ts.cmp(&b_ts),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        });
    }

    Ok(patient_events)
}

/// Extract events within a specific time window.
#[must_use]
pub fn extract_window_events(
    events: &[PatientEvent],
    anchor_time: i64,
    window_start_offset: i64,
    window_end_offset: i64,
) -> Vec<&PatientEvent> {
    let window_start = anchor_time + window_start_offset;
    let window_end = anchor_time + window_end_offset;

    events
        .iter()
        .filter(|event| {
            event
                .timestamp
                .is_some_and(|timestamp| timestamp >= window_start && timestamp <= window_end)
        })
        .collect()
}

/// Find anchor times for a patient based on the anchor point type.
#[must_use]
pub fn find_anchor_times(events: &[PatientEvent], anchor: &AnchorPoint) -> Vec<i64> {
    let mut anchor_times = Vec::new();

    match anchor {
        AnchorPoint::Admission => {
            // Find admission events
            for event in events {
                if event.event_type == "admission" {
                    if let Some(timestamp) = event.timestamp {
                        anchor_times.push(timestamp);
                    }
                }
            }
        }
        AnchorPoint::Discharge => {
            // Find discharge events
            for event in events {
                if event.event_type == "discharge" {
                    if let Some(timestamp) = event.timestamp {
                        anchor_times.push(timestamp);
                    }
                }
            }
        }
        AnchorPoint::ICUAdmission => {
            // Find ICU admission events
            for event in events {
                if event.event_type == "icu_admission" {
                    if let Some(timestamp) = event.timestamp {
                        anchor_times.push(timestamp);
                    }
                }
            }
        }
        AnchorPoint::ICUDischarge => {
            // Find ICU discharge events
            for event in events {
                if event.event_type == "icu_discharge" {
                    if let Some(timestamp) = event.timestamp {
                        anchor_times.push(timestamp);
                    }
                }
            }
        }
        AnchorPoint::Custom(timestamp) => {
            anchor_times.push(*timestamp);
        }
    }

    anchor_times
}

/// Extract observation, gap, and prediction windows for a patient.
#[must_use]
pub fn extract_task_windows<'a>(
    events: &'a [PatientEvent],
    windows: &TaskWindows,
) -> Vec<(Vec<&'a PatientEvent>, Vec<&'a PatientEvent>)> {
    let anchor_times = find_anchor_times(events, &windows.anchor);
    let mut task_windows = Vec::new();

    for anchor_time in anchor_times {
        // Observation window (before anchor)
        let obs_start = -windows.observation_micros();
        let obs_events = extract_window_events(events, anchor_time, obs_start, 0);

        // Prediction window (after anchor + gap)
        let pred_start = windows.gap_micros();
        let pred_end = windows.gap_micros() + windows.prediction_micros();
        let pred_events = extract_window_events(events, anchor_time, pred_start, pred_end);

        task_windows.push((obs_events, pred_events));
    }

    task_windows
}

/// A task runner that applies task definitions to event streams.
pub struct TaskRunner {
    /// Configuration for the task
    #[allow(dead_code)]
    config: TaskWindows,
}

impl TaskRunner {
    /// Create a new task runner.
    #[must_use]
    pub fn new(config: TaskWindows) -> Self {
        Self { config }
    }

    /// Process a batch of events through the task windows.
    ///
    /// # Errors
    /// Returns an error if the input batch is invalid or processing fails.
    pub fn process_batch(
        &self,
        batch: &RecordBatch,
        task: &dyn crate::types::TaskDefinition,
    ) -> Result<Vec<crate::types::TaskOutput>> {
        // Validate input batch
        task.validate_input(batch)?;

        // Group and sort events by patient
        let patient_events = group_and_sort_events(batch)?;

        // Process each patient
        let mut outputs = Vec::new();
        for (patient_id, events) in patient_events {
            if let Ok(output) = task.process_patient(patient_id, &events) {
                outputs.push(output);
            }
        }

        Ok(outputs)
    }
}
