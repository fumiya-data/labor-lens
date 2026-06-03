//! Ingest boundary DTOs.
//!
//! Types exposed to CLI, local server, UI, and tests for starting ingest runs
//! and reading validation results.

use super::domain::DatasetKind;
use crate::shared::RunId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CsvInput {
    pub dataset_kind: DatasetKind,
    pub source_ref: String,
    pub contents: String,
}

impl CsvInput {
    pub fn new(
        dataset_kind: DatasetKind,
        source_ref: impl Into<String>,
        contents: impl Into<String>,
    ) -> Self {
        Self {
            dataset_kind,
            source_ref: source_ref.into(),
            contents: contents.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IngestRunCommand {
    pub run_id: RunId,
    pub employees_csv: CsvInput,
    pub attendance_csv: CsvInput,
}

impl IngestRunCommand {
    pub fn new(run_id: RunId, employees_csv: CsvInput, attendance_csv: CsvInput) -> Self {
        Self {
            run_id,
            employees_csv,
            attendance_csv,
        }
    }
}
