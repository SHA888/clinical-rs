//! CSV reading utilities for MIMIC datasets

use crate::types::{EtlError, Result};
use arrow::array::{ArrayRef, Float64Array, Int64Array, StringArray, TimestampMicrosecondArray};
use arrow::record_batch::RecordBatch;
use csv::ReaderBuilder;
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

/// A CSV reader that converts MIMIC tables to `RecordBatches`.
pub struct MimicCsvReader {
    config: crate::types::DatasetConfig,
}

impl MimicCsvReader {
    /// Create a new reader with the given configuration.
    #[must_use]
    pub const fn new(config: crate::types::DatasetConfig) -> Self {
        Self { config }
    }

    /// Read a MIMIC CSV file and convert to `RecordBatches`.
    ///
    /// # Errors
    /// Returns an error if the file cannot be read or the conversion fails.
    pub fn read_table<P: AsRef<Path>>(
        &self,
        table_name: &str,
        csv_path: P,
    ) -> Result<Vec<RecordBatch>> {
        let csv_path = csv_path.as_ref();

        // Normalize table name to lowercase (MIMIC-III uses UPPERCASE names)
        // Also handle MIMIC-III specific table name aliases
        let table_name = normalize_table_name(table_name);

        // Create CSV reader (simplified without memmap for now)
        let mut rdr = ReaderBuilder::new().has_headers(true).from_path(csv_path)?;

        // Get headers
        let headers = rdr.headers()?.clone();

        // Read all records with explicit types
        let records: Vec<Vec<String>> = rdr
            .records()
            .map(|r: std::result::Result<csv::StringRecord, csv::Error>| {
                r.map(|record: csv::StringRecord| {
                    record
                        .iter()
                        .map(std::string::ToString::to_string)
                        .collect()
                })
            })
            .collect::<std::result::Result<_, _>>()?;

        // Convert to RecordBatches based on table type
        match table_name.as_str() {
            "admissions" => self.convert_admissions(&headers, &records),
            "patients" => self.convert_patients(&headers, &records),
            "diagnoses_icd" => self.convert_diagnoses_icd(&headers, &records),
            "procedures_icd" => self.convert_procedures_icd(&headers, &records),
            "prescriptions" => self.convert_prescriptions(&headers, &records),
            "labevents" => self.convert_labevents(&headers, &records),
            "icustays" => self.convert_icustays(&headers, &records),
            "chartevents" => self.convert_chartevents(&headers, &records),
            "inputevents" => self.convert_inputevents(&headers, &records),
            "outputevents" => self.convert_outputevents(&headers, &records),
            "procedureevents" => self.convert_procedureevents(&headers, &records),
            "microbiologyevents" => self.convert_microbiologyevents(&headers, &records),
            "transfers" => self.convert_transfers(&headers, &records),
            _ => Err(EtlError::Config(format!("Unknown table: {table_name}"))),
        }
    }

    fn convert_admissions(
        &self,
        headers: &csv::StringRecord,
        records: &[Vec<String>],
    ) -> Result<Vec<RecordBatch>> {
        let col_indices = get_column_indices(
            headers,
            &[
                "subject_id",
                "hadm_id",
                "admittime",
                "dischtime",
                "deathtime",
            ],
        )?;

        let batches: Result<Vec<RecordBatch>> = records
            .par_chunks(self.config.batch_size)
            .map(|chunk| {
                let mut subject_ids = Vec::new();
                let mut hadm_ids = Vec::new();
                let mut charttimes = Vec::new();
                let mut event_types = Vec::new();
                let mut event_ids = Vec::new();
                let mut values = Vec::new();

                for record in chunk {
                    let subject_id = record[col_indices["subject_id"]]
                        .parse::<i64>()
                        .unwrap_or(0);
                    let hadm_id = record[col_indices["hadm_id"]].parse::<i64>().ok();

                    // Add admission event (always present)
                    subject_ids.push(subject_id);
                    hadm_ids.push(hadm_id);
                    charttimes.push(Some(
                        record[col_indices["admittime"]].parse::<i64>().unwrap_or(0),
                    ));
                    event_types.push("admission".to_string());
                    event_ids.push(Some("ADMISSION".to_string()));
                    values.push(Some("Admitted".to_string()));

                    // Add discharge event if timestamp is non-empty
                    let dischtime = record
                        .get(col_indices["dischtime"])
                        .map_or("", String::as_str);
                    if !dischtime.is_empty() {
                        subject_ids.push(subject_id);
                        hadm_ids.push(hadm_id);
                        charttimes.push(Some(dischtime.parse::<i64>().unwrap_or(0)));
                        event_types.push("discharge".to_string());
                        event_ids.push(Some("DISCHARGE".to_string()));
                        values.push(Some("Discharged".to_string()));
                    }

                    // Add death event if timestamp is non-empty
                    let deathtime = record
                        .get(col_indices["deathtime"])
                        .map_or("", String::as_str);
                    if !deathtime.is_empty() {
                        subject_ids.push(subject_id);
                        hadm_ids.push(hadm_id);
                        charttimes.push(Some(deathtime.parse::<i64>().unwrap_or(0)));
                        event_types.push("death".to_string());
                        event_ids.push(Some("DEATH".to_string()));
                        values.push(Some("Died".to_string()));
                    }
                }

                let schema = crate::types::clinical_event_schema();
                let batch = RecordBatch::try_new(
                    Arc::new(schema),
                    vec![
                        Arc::new(Int64Array::from(subject_ids)) as ArrayRef,
                        Arc::new(Int64Array::from(hadm_ids)) as ArrayRef,
                        Arc::new(Int64Array::from(vec![None; charttimes.len()])) as ArrayRef,
                        Arc::new(TimestampMicrosecondArray::from(charttimes)) as ArrayRef,
                        Arc::new(StringArray::from(event_types)) as ArrayRef,
                        Arc::new(StringArray::from(event_ids)) as ArrayRef,
                        Arc::new(StringArray::from(values.clone())) as ArrayRef,
                        Arc::new(Float64Array::from(vec![None; values.len()])) as ArrayRef,
                        Arc::new(StringArray::from(vec![None as Option<&str>; values.len()]))
                            as ArrayRef,
                    ],
                )?;

                Ok(batch)
            })
            .collect();

        batches
    }

    fn convert_patients(
        &self,
        headers: &csv::StringRecord,
        records: &[Vec<String>],
    ) -> Result<Vec<RecordBatch>> {
        let col_indices = get_column_indices(
            headers,
            &["subject_id", "gender", "anchor_age", "anchor_year"],
        )?;

        let batches: Result<Vec<RecordBatch>> = records
            .par_chunks(self.config.batch_size)
            .map(|chunk| {
                let mut subject_ids = Vec::new();
                let mut event_types = Vec::new();
                let mut event_ids = Vec::new();
                let mut values = Vec::new();

                for record in chunk {
                    let subject_id = record[col_indices["subject_id"]]
                        .parse::<i64>()
                        .unwrap_or(0);

                    // Add gender event
                    subject_ids.push(subject_id);
                    event_types.push("gender".to_string());
                    event_ids.push(Some("GENDER".to_string()));
                    values.push(
                        record
                            .get(col_indices["gender"])
                            .map(std::string::ToString::to_string),
                    );

                    // Add anchor_age event
                    subject_ids.push(subject_id);
                    event_types.push("anchor_age".to_string());
                    event_ids.push(Some("ANCHOR_AGE".to_string()));
                    values.push(
                        record
                            .get(col_indices["anchor_age"])
                            .map(std::string::ToString::to_string),
                    );

                    // Add anchor_year event
                    subject_ids.push(subject_id);
                    event_types.push("anchor_year".to_string());
                    event_ids.push(Some("ANCHOR_YEAR".to_string()));
                    values.push(
                        record
                            .get(col_indices["anchor_year"])
                            .map(std::string::ToString::to_string),
                    );
                }

                let schema = crate::types::clinical_event_schema();
                let batch = RecordBatch::try_new(
                    Arc::new(schema),
                    vec![
                        Arc::new(Int64Array::from(subject_ids)) as ArrayRef,
                        Arc::new(Int64Array::from(vec![None; event_types.len()])) as ArrayRef,
                        Arc::new(Int64Array::from(vec![None; event_types.len()])) as ArrayRef,
                        Arc::new(TimestampMicrosecondArray::from(vec![
                            None;
                            event_types.len()
                        ])) as ArrayRef,
                        Arc::new(StringArray::from(event_types)) as ArrayRef,
                        Arc::new(StringArray::from(event_ids)) as ArrayRef,
                        Arc::new(StringArray::from(values.clone())) as ArrayRef,
                        Arc::new(Float64Array::from(vec![None; values.len()])) as ArrayRef,
                        Arc::new(StringArray::from(vec![None as Option<&str>; values.len()]))
                            as ArrayRef,
                    ],
                )?;

                Ok(batch)
            })
            .collect();

        batches
    }

    fn convert_diagnoses_icd(
        &self,
        headers: &csv::StringRecord,
        records: &[Vec<String>],
    ) -> Result<Vec<RecordBatch>> {
        let col_indices =
            get_column_indices(headers, &["subject_id", "hadm_id", "icd_code", "seq_num"])?;

        let batches: Result<Vec<RecordBatch>> = records
            .par_chunks(self.config.batch_size)
            .map(|chunk| {
                let mut subject_ids = Vec::new();
                let mut hadm_ids = Vec::new();
                let mut charttimes = Vec::new();
                let mut event_types = Vec::new();
                let mut event_ids = Vec::new();
                let mut values = Vec::new();

                for record in chunk {
                    subject_ids.push(
                        record[col_indices["subject_id"]]
                            .parse::<i64>()
                            .unwrap_or(0),
                    );
                    hadm_ids.push(record[col_indices["hadm_id"]].parse::<i64>().ok());
                    charttimes.push(None); // Diagnoses don't have timestamps
                    event_types.push("diagnosis".to_string());
                    event_ids.push(
                        record
                            .get(col_indices["icd_code"])
                            .map(std::string::ToString::to_string),
                    );
                    values.push(
                        record
                            .get(col_indices["seq_num"])
                            .map(std::string::ToString::to_string),
                    );
                }

                let schema = crate::types::clinical_event_schema();
                let batch = RecordBatch::try_new(
                    Arc::new(schema),
                    vec![
                        Arc::new(Int64Array::from(subject_ids)) as ArrayRef,
                        Arc::new(Int64Array::from(hadm_ids)) as ArrayRef,
                        Arc::new(Int64Array::from(vec![None; charttimes.len()])) as ArrayRef,
                        Arc::new(TimestampMicrosecondArray::from(charttimes)) as ArrayRef,
                        Arc::new(StringArray::from(event_types)) as ArrayRef,
                        Arc::new(StringArray::from(event_ids)) as ArrayRef,
                        Arc::new(StringArray::from(values.clone())) as ArrayRef,
                        Arc::new(Float64Array::from(vec![None; values.len()])) as ArrayRef,
                        Arc::new(StringArray::from(vec![None as Option<&str>; values.len()]))
                            as ArrayRef,
                    ],
                )?;

                Ok(batch)
            })
            .collect();

        batches
    }

    fn convert_procedures_icd(
        &self,
        headers: &csv::StringRecord,
        records: &[Vec<String>],
    ) -> Result<Vec<RecordBatch>> {
        // Similar to diagnoses but for procedures
        let col_indices =
            get_column_indices(headers, &["subject_id", "hadm_id", "icd_code", "seq_num"])?;

        let batches: Result<Vec<RecordBatch>> = records
            .par_chunks(self.config.batch_size)
            .map(|chunk| {
                let mut subject_ids = Vec::new();
                let mut hadm_ids = Vec::new();
                let mut charttimes = Vec::new();
                let mut event_types = Vec::new();
                let mut event_ids = Vec::new();
                let mut values = Vec::new();

                for record in chunk {
                    subject_ids.push(
                        record[col_indices["subject_id"]]
                            .parse::<i64>()
                            .unwrap_or(0),
                    );
                    hadm_ids.push(record[col_indices["hadm_id"]].parse::<i64>().ok());
                    charttimes.push(None);
                    event_types.push("procedure".to_string());
                    event_ids.push(
                        record
                            .get(col_indices["icd_code"])
                            .map(std::string::ToString::to_string),
                    );
                    values.push(
                        record
                            .get(col_indices["seq_num"])
                            .map(std::string::ToString::to_string),
                    );
                }

                let schema = crate::types::clinical_event_schema();
                let batch = RecordBatch::try_new(
                    Arc::new(schema),
                    vec![
                        Arc::new(Int64Array::from(subject_ids)) as ArrayRef,
                        Arc::new(Int64Array::from(hadm_ids)) as ArrayRef,
                        Arc::new(Int64Array::from(vec![None; charttimes.len()])) as ArrayRef,
                        Arc::new(TimestampMicrosecondArray::from(charttimes)) as ArrayRef,
                        Arc::new(StringArray::from(event_types)) as ArrayRef,
                        Arc::new(StringArray::from(event_ids)) as ArrayRef,
                        Arc::new(StringArray::from(values.clone())) as ArrayRef,
                        Arc::new(Float64Array::from(vec![None; values.len()])) as ArrayRef,
                        Arc::new(StringArray::from(vec![None as Option<&str>; values.len()]))
                            as ArrayRef,
                    ],
                )?;

                Ok(batch)
            })
            .collect();

        batches
    }

    fn convert_prescriptions(
        &self,
        headers: &csv::StringRecord,
        records: &[Vec<String>],
    ) -> Result<Vec<RecordBatch>> {
        let col_indices = get_column_indices(
            headers,
            &[
                "subject_id",
                "hadm_id",
                "drug",
                "dose_val_rx",
                "dose_unit_rx",
                "starttime",
                "stoptime",
            ],
        )?;

        let batches: Result<Vec<RecordBatch>> = records
            .par_chunks(self.config.batch_size)
            .map(|chunk| {
                let mut subject_ids = Vec::new();
                let mut hadm_ids = Vec::new();
                let mut charttimes = Vec::new();
                let mut event_types = Vec::new();
                let mut event_ids = Vec::new();
                let mut values = Vec::new();
                let mut value_nums = Vec::new();
                let mut units = Vec::new();

                for record in chunk {
                    subject_ids.push(
                        record[col_indices["subject_id"]]
                            .parse::<i64>()
                            .unwrap_or(0),
                    );
                    hadm_ids.push(record[col_indices["hadm_id"]].parse::<i64>().ok());

                    // Add start event
                    if let Some(starttime) = record.get(col_indices["starttime"]) {
                        charttimes.push(Some(starttime.parse::<i64>().unwrap_or(0)));
                        event_types.push("medication_start".to_string());
                        event_ids.push(
                            record
                                .get(col_indices["drug"])
                                .map(std::string::ToString::to_string),
                        );
                        values.push(
                            record
                                .get(col_indices["dose_val_rx"])
                                .map(std::string::ToString::to_string),
                        );
                        value_nums.push(
                            record
                                .get(col_indices["dose_val_rx"])
                                .and_then(|s| s.parse().ok()),
                        );
                        units.push(
                            record
                                .get(col_indices["dose_unit_rx"])
                                .map(std::string::ToString::to_string),
                        );
                    }

                    // Add stop event if available
                    if let Some(stoptime) = record.get(col_indices["stoptime"]) {
                        charttimes.push(Some(stoptime.parse::<i64>().unwrap_or(0)));
                        event_types.push("medication_stop".to_string());
                        event_ids.push(
                            record
                                .get(col_indices["drug"])
                                .map(std::string::ToString::to_string),
                        );
                        values.push(None);
                        value_nums.push(None);
                        units.push(None);
                    }
                }

                let schema = crate::types::clinical_event_schema();
                let batch = RecordBatch::try_new(
                    Arc::new(schema),
                    vec![
                        Arc::new(Int64Array::from(subject_ids)) as ArrayRef,
                        Arc::new(Int64Array::from(hadm_ids)) as ArrayRef,
                        Arc::new(Int64Array::from(vec![None; charttimes.len()])) as ArrayRef,
                        Arc::new(TimestampMicrosecondArray::from(charttimes)) as ArrayRef,
                        Arc::new(StringArray::from(event_types)) as ArrayRef,
                        Arc::new(StringArray::from(event_ids)) as ArrayRef,
                        Arc::new(StringArray::from(values)) as ArrayRef,
                        Arc::new(Float64Array::from(value_nums)) as ArrayRef,
                        Arc::new(StringArray::from(units)) as ArrayRef,
                    ],
                )?;

                Ok(batch)
            })
            .collect();

        batches
    }

    fn convert_labevents(
        &self,
        headers: &csv::StringRecord,
        records: &[Vec<String>],
    ) -> Result<Vec<RecordBatch>> {
        let col_indices = get_column_indices(
            headers,
            &[
                "subject_id",
                "hadm_id",
                "itemid",
                "charttime",
                "valuenum",
                "valueuom",
            ],
        )?;

        let batches: Result<Vec<RecordBatch>> = records
            .par_chunks(self.config.batch_size)
            .map(|chunk| {
                let mut subject_ids = Vec::new();
                let mut hadm_ids = Vec::new();
                let mut charttimes = Vec::new();
                let mut event_types = Vec::new();
                let mut event_ids = Vec::new();
                let mut values: Vec<Option<String>> = Vec::new();
                let mut value_nums = Vec::new();
                let mut units = Vec::new();

                for record in chunk {
                    subject_ids.push(
                        record[col_indices["subject_id"]]
                            .parse::<i64>()
                            .unwrap_or(0),
                    );
                    hadm_ids.push(record[col_indices["hadm_id"]].parse::<i64>().ok());
                    charttimes.push(
                        record
                            .get(col_indices["charttime"])
                            .and_then(|s| s.parse().ok()),
                    );
                    event_types.push("lab".to_string());
                    event_ids.push(
                        record
                            .get(col_indices["itemid"])
                            .map(std::string::ToString::to_string),
                    );
                    values.push(None); // Lab values are typically numeric
                    value_nums.push(
                        record
                            .get(col_indices["valuenum"])
                            .and_then(|s| s.parse().ok()),
                    );
                    units.push(
                        record
                            .get(col_indices["valueuom"])
                            .map(std::string::ToString::to_string),
                    );
                }

                let schema = crate::types::clinical_event_schema();
                let batch = RecordBatch::try_new(
                    Arc::new(schema),
                    vec![
                        Arc::new(Int64Array::from(subject_ids)) as ArrayRef,
                        Arc::new(Int64Array::from(hadm_ids)) as ArrayRef,
                        Arc::new(Int64Array::from(vec![None; charttimes.len()])) as ArrayRef,
                        Arc::new(TimestampMicrosecondArray::from(charttimes)) as ArrayRef,
                        Arc::new(StringArray::from(event_types)) as ArrayRef,
                        Arc::new(StringArray::from(event_ids)) as ArrayRef,
                        Arc::new(StringArray::from(values)) as ArrayRef,
                        Arc::new(Float64Array::from(value_nums)) as ArrayRef,
                        Arc::new(StringArray::from(units)) as ArrayRef,
                    ],
                )?;

                Ok(batch)
            })
            .collect();

        batches
    }

    fn convert_icustays(
        &self,
        headers: &csv::StringRecord,
        records: &[Vec<String>],
    ) -> Result<Vec<RecordBatch>> {
        let col_indices = get_column_indices(
            headers,
            &["subject_id", "hadm_id", "icustay_id", "intime", "outtime"],
        )?;

        let batches: Result<Vec<RecordBatch>> = records
            .par_chunks(self.config.batch_size)
            .map(|chunk| {
                let mut subject_ids = Vec::new();
                let mut hadm_ids = Vec::new();
                let mut stay_ids = Vec::new();
                let mut charttimes = Vec::new();
                let mut event_types = Vec::new();
                let mut event_ids = Vec::new();
                let mut values = Vec::new();

                for record in chunk {
                    subject_ids.push(
                        record[col_indices["subject_id"]]
                            .parse::<i64>()
                            .unwrap_or(0),
                    );
                    hadm_ids.push(record[col_indices["hadm_id"]].parse::<i64>().ok());
                    stay_ids.push(record[col_indices["icustay_id"]].parse::<i64>().ok());

                    // Add ICU admission event
                    charttimes.push(
                        record
                            .get(col_indices["intime"])
                            .and_then(|s| s.parse().ok()),
                    );
                    event_types.push("icu_admission".to_string());
                    event_ids.push(Some("ICU_ADMISSION".to_string()));
                    values.push(Some("ICU Admitted".to_string()));

                    // Add ICU discharge event if available
                    if let Some(outtime) = record.get(col_indices["outtime"]) {
                        charttimes.push(outtime.parse().ok());
                        event_types.push("icu_discharge".to_string());
                        event_ids.push(Some("ICU_DISCHARGE".to_string()));
                        values.push(Some("ICU Discharged".to_string()));
                    }
                }

                let schema = crate::types::clinical_event_schema();
                let batch = RecordBatch::try_new(
                    Arc::new(schema),
                    vec![
                        Arc::new(Int64Array::from(subject_ids)) as ArrayRef,
                        Arc::new(Int64Array::from(hadm_ids)) as ArrayRef,
                        Arc::new(Int64Array::from(stay_ids)) as ArrayRef,
                        Arc::new(TimestampMicrosecondArray::from(charttimes)) as ArrayRef,
                        Arc::new(StringArray::from(event_types)) as ArrayRef,
                        Arc::new(StringArray::from(event_ids)) as ArrayRef,
                        Arc::new(StringArray::from(values.clone())) as ArrayRef,
                        Arc::new(Float64Array::from(vec![None; values.len()])) as ArrayRef,
                        Arc::new(StringArray::from(vec![None as Option<&str>; values.len()]))
                            as ArrayRef,
                    ],
                )?;

                Ok(batch)
            })
            .collect();

        batches
    }

    fn convert_chartevents(
        &self,
        headers: &csv::StringRecord,
        records: &[Vec<String>],
    ) -> Result<Vec<RecordBatch>> {
        let col_indices = get_column_indices(
            headers,
            &[
                "subject_id",
                "hadm_id",
                "icustay_id",
                "charttime",
                "itemid",
                "valuenum",
                "valueuom",
            ],
        )?;

        let batches: Result<Vec<RecordBatch>> = records
            .par_chunks(self.config.batch_size)
            .map(|chunk| {
                let mut subject_ids = Vec::new();
                let mut hadm_ids = Vec::new();
                let mut stay_ids = Vec::new();
                let mut charttimes = Vec::new();
                let mut event_types = Vec::new();
                let mut event_ids = Vec::new();
                let mut values: Vec<Option<String>> = Vec::new();
                let mut value_nums = Vec::new();
                let mut units = Vec::new();

                for record in chunk {
                    subject_ids.push(
                        record[col_indices["subject_id"]]
                            .parse::<i64>()
                            .unwrap_or(0),
                    );
                    hadm_ids.push(record[col_indices["hadm_id"]].parse::<i64>().ok());
                    stay_ids.push(record[col_indices["icustay_id"]].parse::<i64>().ok());
                    charttimes.push(
                        record
                            .get(col_indices["charttime"])
                            .and_then(|s| s.parse().ok()),
                    );
                    event_types.push("vital_sign".to_string());
                    event_ids.push(
                        record
                            .get(col_indices["itemid"])
                            .map(std::string::ToString::to_string),
                    );
                    values.push(None); // Vitals are typically numeric
                    value_nums.push(
                        record
                            .get(col_indices["valuenum"])
                            .and_then(|s| s.parse().ok()),
                    );
                    units.push(
                        record
                            .get(col_indices["valueuom"])
                            .map(std::string::ToString::to_string),
                    );
                }

                let schema = crate::types::clinical_event_schema();
                let batch = RecordBatch::try_new(
                    Arc::new(schema),
                    vec![
                        Arc::new(Int64Array::from(subject_ids)) as ArrayRef,
                        Arc::new(Int64Array::from(hadm_ids)) as ArrayRef,
                        Arc::new(Int64Array::from(stay_ids)) as ArrayRef,
                        Arc::new(TimestampMicrosecondArray::from(charttimes)) as ArrayRef,
                        Arc::new(StringArray::from(event_types)) as ArrayRef,
                        Arc::new(StringArray::from(event_ids)) as ArrayRef,
                        Arc::new(StringArray::from(values.clone())) as ArrayRef,
                        Arc::new(Float64Array::from(value_nums)) as ArrayRef,
                        Arc::new(StringArray::from(units)) as ArrayRef,
                    ],
                )?;

                Ok(batch)
            })
            .collect();

        batches
    }

    fn convert_inputevents(
        &self,
        headers: &csv::StringRecord,
        records: &[Vec<String>],
    ) -> Result<Vec<RecordBatch>> {
        let col_indices = get_column_indices(
            headers,
            &[
                "subject_id",
                "hadm_id",
                "icustay_id",
                "starttime",
                "endtime",
                "itemid",
                "volume",
                "volumeuom",
            ],
        )?;

        let batches: Result<Vec<RecordBatch>> = records
            .par_chunks(self.config.batch_size)
            .map(|chunk| {
                let mut subject_ids = Vec::new();
                let mut hadm_ids = Vec::new();
                let mut stay_ids = Vec::new();
                let mut charttimes = Vec::new();
                let mut event_types = Vec::new();
                let mut event_ids = Vec::new();
                let mut values = Vec::new();
                let mut value_nums = Vec::new();
                let mut units = Vec::new();

                for record in chunk {
                    subject_ids.push(
                        record[col_indices["subject_id"]]
                            .parse::<i64>()
                            .unwrap_or(0),
                    );
                    hadm_ids.push(record[col_indices["hadm_id"]].parse::<i64>().ok());
                    stay_ids.push(record[col_indices["icustay_id"]].parse::<i64>().ok());

                    // Add input event
                    charttimes.push(
                        record
                            .get(col_indices["starttime"])
                            .and_then(|s| s.parse().ok()),
                    );
                    event_types.push("input".to_string());
                    event_ids.push(
                        record
                            .get(col_indices["itemid"])
                            .map(std::string::ToString::to_string),
                    );
                    values.push(
                        record
                            .get(col_indices["volume"])
                            .map(std::string::ToString::to_string),
                    );
                    value_nums.push(
                        record
                            .get(col_indices["volume"])
                            .and_then(|s| s.parse().ok()),
                    );
                    units.push(
                        record
                            .get(col_indices["volumeuom"])
                            .map(std::string::ToString::to_string),
                    );
                }

                let schema = crate::types::clinical_event_schema();
                let batch = RecordBatch::try_new(
                    Arc::new(schema),
                    vec![
                        Arc::new(Int64Array::from(subject_ids)) as ArrayRef,
                        Arc::new(Int64Array::from(hadm_ids)) as ArrayRef,
                        Arc::new(Int64Array::from(stay_ids)) as ArrayRef,
                        Arc::new(TimestampMicrosecondArray::from(charttimes)) as ArrayRef,
                        Arc::new(StringArray::from(event_types)) as ArrayRef,
                        Arc::new(StringArray::from(event_ids)) as ArrayRef,
                        Arc::new(StringArray::from(values.clone())) as ArrayRef,
                        Arc::new(Float64Array::from(value_nums)) as ArrayRef,
                        Arc::new(StringArray::from(units)) as ArrayRef,
                    ],
                )?;

                Ok(batch)
            })
            .collect();

        batches
    }

    fn convert_outputevents(
        &self,
        headers: &csv::StringRecord,
        records: &[Vec<String>],
    ) -> Result<Vec<RecordBatch>> {
        let col_indices = get_column_indices(
            headers,
            &[
                "subject_id",
                "hadm_id",
                "icustay_id",
                "charttime",
                "itemid",
                "value",
                "valueuom",
            ],
        )?;

        let batches: Result<Vec<RecordBatch>> = records
            .par_chunks(self.config.batch_size)
            .map(|chunk| {
                let mut subject_ids = Vec::new();
                let mut hadm_ids = Vec::new();
                let mut stay_ids = Vec::new();
                let mut charttimes = Vec::new();
                let mut event_types = Vec::new();
                let mut event_ids = Vec::new();
                let mut values = Vec::new();
                let mut value_nums = Vec::new();
                let mut units = Vec::new();

                for record in chunk {
                    subject_ids.push(
                        record[col_indices["subject_id"]]
                            .parse::<i64>()
                            .unwrap_or(0),
                    );
                    hadm_ids.push(record[col_indices["hadm_id"]].parse::<i64>().ok());
                    stay_ids.push(record[col_indices["icustay_id"]].parse::<i64>().ok());
                    charttimes.push(
                        record
                            .get(col_indices["charttime"])
                            .and_then(|s| s.parse().ok()),
                    );
                    event_types.push("output".to_string());
                    event_ids.push(
                        record
                            .get(col_indices["itemid"])
                            .map(std::string::ToString::to_string),
                    );
                    values.push(
                        record
                            .get(col_indices["value"])
                            .map(std::string::ToString::to_string),
                    );
                    value_nums.push(
                        record
                            .get(col_indices["value"])
                            .and_then(|s| s.parse().ok()),
                    );
                    units.push(
                        record
                            .get(col_indices["valueuom"])
                            .map(std::string::ToString::to_string),
                    );
                }

                let schema = crate::types::clinical_event_schema();
                let batch = RecordBatch::try_new(
                    Arc::new(schema),
                    vec![
                        Arc::new(Int64Array::from(subject_ids)) as ArrayRef,
                        Arc::new(Int64Array::from(hadm_ids)) as ArrayRef,
                        Arc::new(Int64Array::from(stay_ids)) as ArrayRef,
                        Arc::new(TimestampMicrosecondArray::from(charttimes)) as ArrayRef,
                        Arc::new(StringArray::from(event_types)) as ArrayRef,
                        Arc::new(StringArray::from(event_ids)) as ArrayRef,
                        Arc::new(StringArray::from(values.clone())) as ArrayRef,
                        Arc::new(Float64Array::from(value_nums)) as ArrayRef,
                        Arc::new(StringArray::from(units)) as ArrayRef,
                    ],
                )?;

                Ok(batch)
            })
            .collect();

        batches
    }

    fn convert_procedureevents(
        &self,
        headers: &csv::StringRecord,
        records: &[Vec<String>],
    ) -> Result<Vec<RecordBatch>> {
        let col_indices = get_column_indices(
            headers,
            &[
                "subject_id",
                "hadm_id",
                "icustay_id",
                "starttime",
                "endtime",
                "itemid",
                "value",
                "valueuom",
            ],
        )?;

        let batches: Result<Vec<RecordBatch>> = records
            .par_chunks(self.config.batch_size)
            .map(|chunk| {
                let mut subject_ids = Vec::new();
                let mut hadm_ids = Vec::new();
                let mut stay_ids = Vec::new();
                let mut charttimes = Vec::new();
                let mut event_types = Vec::new();
                let mut event_ids = Vec::new();
                let mut values = Vec::new();
                let mut value_nums = Vec::new();
                let mut units = Vec::new();

                for record in chunk {
                    subject_ids.push(
                        record[col_indices["subject_id"]]
                            .parse::<i64>()
                            .unwrap_or(0),
                    );
                    hadm_ids.push(record[col_indices["hadm_id"]].parse::<i64>().ok());
                    stay_ids.push(record[col_indices["icustay_id"]].parse::<i64>().ok());
                    charttimes.push(
                        record
                            .get(col_indices["starttime"])
                            .and_then(|s| s.parse().ok()),
                    );
                    event_types.push("procedure".to_string());
                    event_ids.push(
                        record
                            .get(col_indices["itemid"])
                            .map(std::string::ToString::to_string),
                    );
                    values.push(
                        record
                            .get(col_indices["value"])
                            .map(std::string::ToString::to_string),
                    );
                    value_nums.push(
                        record
                            .get(col_indices["value"])
                            .and_then(|s| s.parse().ok()),
                    );
                    units.push(
                        record
                            .get(col_indices["valueuom"])
                            .map(std::string::ToString::to_string),
                    );
                }

                let schema = crate::types::clinical_event_schema();
                let batch = RecordBatch::try_new(
                    Arc::new(schema),
                    vec![
                        Arc::new(Int64Array::from(subject_ids)) as ArrayRef,
                        Arc::new(Int64Array::from(hadm_ids)) as ArrayRef,
                        Arc::new(Int64Array::from(stay_ids)) as ArrayRef,
                        Arc::new(TimestampMicrosecondArray::from(charttimes)) as ArrayRef,
                        Arc::new(StringArray::from(event_types)) as ArrayRef,
                        Arc::new(StringArray::from(event_ids)) as ArrayRef,
                        Arc::new(StringArray::from(values.clone())) as ArrayRef,
                        Arc::new(Float64Array::from(value_nums)) as ArrayRef,
                        Arc::new(StringArray::from(units)) as ArrayRef,
                    ],
                )?;

                Ok(batch)
            })
            .collect();

        batches
    }

    fn convert_microbiologyevents(
        &self,
        headers: &csv::StringRecord,
        records: &[Vec<String>],
    ) -> Result<Vec<RecordBatch>> {
        let col_indices = get_column_indices(
            headers,
            &[
                "subject_id",
                "hadm_id",
                "micro_specimen_id",
                "charttime",
                "org_name",
                "ab_name",
            ],
        )?;

        let batches: Result<Vec<RecordBatch>> = records
            .par_chunks(self.config.batch_size)
            .map(|chunk| {
                let mut subject_ids = Vec::new();
                let mut hadm_ids = Vec::new();
                let mut stay_ids = Vec::new();
                let mut charttimes = Vec::new();
                let mut event_types = Vec::new();
                let mut event_ids = Vec::new();
                let mut values = Vec::new();
                let mut value_nums = Vec::new();
                let mut units: Vec<Option<String>> = Vec::new();

                for record in chunk {
                    subject_ids.push(
                        record[col_indices["subject_id"]]
                            .parse::<i64>()
                            .unwrap_or(0),
                    );
                    hadm_ids.push(record[col_indices["hadm_id"]].parse::<i64>().ok());
                    stay_ids.push(record[col_indices["micro_specimen_id"]].parse::<i64>().ok());
                    charttimes.push(
                        record
                            .get(col_indices["charttime"])
                            .and_then(|s| s.parse().ok()),
                    );
                    event_types.push("microbiology".to_string());
                    event_ids.push(
                        record
                            .get(col_indices["org_name"])
                            .map(std::string::ToString::to_string),
                    );
                    values.push(
                        record
                            .get(col_indices["ab_name"])
                            .map(std::string::ToString::to_string),
                    );
                    value_nums.push(None);
                    units.push(None);
                }

                let schema = crate::types::clinical_event_schema();
                let batch = RecordBatch::try_new(
                    Arc::new(schema),
                    vec![
                        Arc::new(Int64Array::from(subject_ids)) as ArrayRef,
                        Arc::new(Int64Array::from(hadm_ids)) as ArrayRef,
                        Arc::new(Int64Array::from(stay_ids)) as ArrayRef,
                        Arc::new(TimestampMicrosecondArray::from(charttimes)) as ArrayRef,
                        Arc::new(StringArray::from(event_types)) as ArrayRef,
                        Arc::new(StringArray::from(event_ids)) as ArrayRef,
                        Arc::new(StringArray::from(values.clone())) as ArrayRef,
                        Arc::new(Float64Array::from(value_nums)) as ArrayRef,
                        Arc::new(StringArray::from(units)) as ArrayRef,
                    ],
                )?;

                Ok(batch)
            })
            .collect();

        batches
    }

    fn convert_transfers(
        &self,
        headers: &csv::StringRecord,
        records: &[Vec<String>],
    ) -> Result<Vec<RecordBatch>> {
        let col_indices = get_column_indices(
            headers,
            &[
                "subject_id",
                "hadm_id",
                "transfer_id",
                "intime",
                "outtime",
                "eventtype",
            ],
        )?;

        let batches: Result<Vec<RecordBatch>> = records
            .par_chunks(self.config.batch_size)
            .map(|chunk| {
                let mut subject_ids = Vec::new();
                let mut hadm_ids = Vec::new();
                let mut stay_ids = Vec::new();
                let mut charttimes = Vec::new();
                let mut event_types = Vec::new();
                let mut event_ids = Vec::new();
                let mut values = Vec::new();
                let mut value_nums = Vec::new();
                let mut units: Vec<Option<String>> = Vec::new();

                for record in chunk {
                    subject_ids.push(
                        record[col_indices["subject_id"]]
                            .parse::<i64>()
                            .unwrap_or(0),
                    );
                    hadm_ids.push(record[col_indices["hadm_id"]].parse::<i64>().ok());
                    stay_ids.push(record[col_indices["transfer_id"]].parse::<i64>().ok());

                    // Add transfer in event
                    if let Some(intime) = record.get(col_indices["intime"]) {
                        charttimes.push(intime.parse().ok());
                        event_types.push("transfer_in".to_string());
                        event_ids.push(
                            record
                                .get(col_indices["eventtype"])
                                .map(std::string::ToString::to_string),
                        );
                        values.push(Some("Transfer In".to_string()));
                        value_nums.push(None);
                        units.push(None);
                    }

                    // Add transfer out event if available
                    if let Some(outtime) = record.get(col_indices["outtime"]) {
                        charttimes.push(outtime.parse().ok());
                        event_types.push("transfer_out".to_string());
                        event_ids.push(
                            record
                                .get(col_indices["eventtype"])
                                .map(std::string::ToString::to_string),
                        );
                        values.push(Some("Transfer Out".to_string()));
                        value_nums.push(None);
                        units.push(None);
                    }
                }

                let schema = crate::types::clinical_event_schema();
                let batch = RecordBatch::try_new(
                    Arc::new(schema),
                    vec![
                        Arc::new(Int64Array::from(subject_ids)) as ArrayRef,
                        Arc::new(Int64Array::from(hadm_ids)) as ArrayRef,
                        Arc::new(Int64Array::from(stay_ids)) as ArrayRef,
                        Arc::new(TimestampMicrosecondArray::from(charttimes)) as ArrayRef,
                        Arc::new(StringArray::from(event_types)) as ArrayRef,
                        Arc::new(StringArray::from(event_ids)) as ArrayRef,
                        Arc::new(StringArray::from(values.clone())) as ArrayRef,
                        Arc::new(Float64Array::from(value_nums)) as ArrayRef,
                        Arc::new(StringArray::from(units)) as ArrayRef,
                    ],
                )?;

                Ok(batch)
            })
            .collect();

        batches
    }
}

/// Normalize a table name to its canonical lowercase form.
///
/// Handles MIMIC-III specific table name aliases:
/// - `INPUTEVENTS_MV` / `INPUTEVENTS_CV` → `inputevents`
/// - All other names are lowercased as-is
fn normalize_table_name(name: &str) -> String {
    let lower = name.to_lowercase();
    match lower.as_str() {
        "inputevents_mv" | "inputevents_cv" => "inputevents".to_string(),
        other => other.to_string(),
    }
}

/// Get column indices for required columns from CSV headers.
///
/// Column matching is case-insensitive, so both MIMIC-III (UPPERCASE)
/// and MIMIC-IV (lowercase) column names are accepted. The returned
/// map always uses the lowercase column name as the key.
fn get_column_indices(
    headers: &csv::StringRecord,
    required: &[&str],
) -> Result<HashMap<String, usize>> {
    let mut indices = HashMap::new();

    for &col_name in required {
        let lower = col_name.to_lowercase();
        let index = headers
            .iter()
            .position(|h| h.eq_ignore_ascii_case(col_name))
            .ok_or_else(|| EtlError::MissingColumn(col_name.to_string()))?;
        let _ = indices.insert(lower, index);
    }

    Ok(indices)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn test_normalize_table_name_lowercase() {
        assert_eq!(normalize_table_name("admissions"), "admissions");
        assert_eq!(normalize_table_name("patients"), "patients");
        assert_eq!(normalize_table_name("diagnoses_icd"), "diagnoses_icd");
    }

    #[test]
    fn test_normalize_table_name_uppercase() {
        assert_eq!(normalize_table_name("ADMISSIONS"), "admissions");
        assert_eq!(normalize_table_name("PATIENTS"), "patients");
        assert_eq!(normalize_table_name("DIAGNOSES_ICD"), "diagnoses_icd");
    }

    #[test]
    fn test_normalize_table_name_mimic_iii_aliases() {
        assert_eq!(normalize_table_name("INPUTEVENTS_MV"), "inputevents");
        assert_eq!(normalize_table_name("INPUTEVENTS_CV"), "inputevents");
        assert_eq!(normalize_table_name("inputevents_mv"), "inputevents");
        assert_eq!(normalize_table_name("inputevents_cv"), "inputevents");
    }

    #[test]
    fn test_get_column_indices_case_insensitive() {
        let mut headers = csv::StringRecord::new();
        headers.push_field("SUBJECT_ID");
        headers.push_field("HADM_ID");
        headers.push_field("ADMITTIME");

        let result = get_column_indices(&headers, &["subject_id", "hadm_id", "admittime"]);
        let indices = result.unwrap();
        assert_eq!(indices["subject_id"], 0);
        assert_eq!(indices["hadm_id"], 1);
        assert_eq!(indices["admittime"], 2);
    }

    #[test]
    fn test_get_column_indices_lowercase_headers() {
        let mut headers = csv::StringRecord::new();
        headers.push_field("subject_id");
        headers.push_field("hadm_id");

        let result = get_column_indices(&headers, &["subject_id", "hadm_id"]);
        let indices = result.unwrap();
        assert_eq!(indices["subject_id"], 0);
        assert_eq!(indices["hadm_id"], 1);
    }

    #[test]
    fn test_get_column_indices_missing_column() {
        let mut headers = csv::StringRecord::new();
        headers.push_field("subject_id");

        let result = get_column_indices(&headers, &["subject_id", "nonexistent"]);
        assert!(result.is_err());
    }
}
