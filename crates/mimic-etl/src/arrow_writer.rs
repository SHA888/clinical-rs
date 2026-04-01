//! Arrow writing utilities for MIMIC datasets

use crate::types::EtlError;
use crate::types::Result;
use arrow::ipc::writer::FileWriter;
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use parquet::file::properties::WriterProperties;
use std::fs::File;
use std::path::Path;

/// Write `RecordBatches` to a Parquet file.
///
/// # Errors
/// Returns an error if file creation or writing fails.
pub fn to_parquet<P: AsRef<Path>>(batches: &[RecordBatch], path: P) -> Result<()> {
    let path = path.as_ref();
    let file = File::create(path)?;

    // Get schema from first batch
    let schema = batches
        .first()
        .ok_or_else(|| EtlError::Config("No batches to write".to_string()))?
        .schema();

    // Create Parquet writer with default properties
    let props = WriterProperties::builder().build();
    let mut writer = ArrowWriter::try_new(file, schema, Some(props))?;

    // Write all batches
    for batch in batches {
        writer.write(batch)?;
    }

    // Close the writer
    let _ = writer.close()?;

    Ok(())
}

/// Write `RecordBatches` to an Arrow IPC file.
///
/// # Errors
/// Returns an error if file creation or writing fails.
pub fn to_arrow_ipc<P: AsRef<Path>>(batches: &[RecordBatch], path: P) -> Result<()> {
    let path = path.as_ref();
    let file = File::create(path)?;

    // Get schema from first batch
    let schema = batches
        .first()
        .ok_or_else(|| EtlError::Config("No batches to write".to_string()))?
        .schema();

    // Create IPC writer
    let mut writer = FileWriter::try_new(file, schema.as_ref())?;

    // Write all batches
    for batch in batches {
        writer.write(batch)?;
    }

    // Close the writer
    writer.finish()?;

    Ok(())
}

/// A streaming writer that can write batches as they arrive.
pub struct StreamingArrowWriter {
    writer: ArrowWriter<File>,
}

impl StreamingArrowWriter {
    /// Create a new streaming writer.
    ///
    /// # Errors
    /// Returns an error if file creation or writer initialization fails.
    pub fn new<P: AsRef<Path>>(path: P, schema: &arrow::datatypes::Schema) -> Result<Self> {
        let file = File::create(path.as_ref())?;
        let props = WriterProperties::builder().build();
        let writer = ArrowWriter::try_new(file, std::sync::Arc::new(schema.clone()), Some(props))?;

        Ok(Self { writer })
    }

    /// Write a single batch.
    ///
    /// # Errors
    /// Returns an error if writing the batch fails.
    pub fn write_batch(&mut self, batch: &RecordBatch) -> Result<()> {
        self.writer.write(batch)?;
        Ok(())
    }

    /// Finish writing and close the file.
    ///
    /// # Errors
    /// Returns an error if closing the writer fails.
    pub fn finish(self) -> Result<()> {
        let _ = self.writer.close()?;
        Ok(())
    }
}
