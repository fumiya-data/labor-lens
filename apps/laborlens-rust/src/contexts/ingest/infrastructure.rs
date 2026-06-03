//! Ingest adapters.
//!
//! CSV readers, source archive access, encoding handling, and storage adapters
//! belong here. Business decisions stay in domain/application modules.

use super::domain::DatasetKind;
use super::interfaces::CsvInput;
use std::fs;
use std::io;
use std::path::Path;

pub fn load_csv_input_from_path(
    dataset_kind: DatasetKind,
    path: impl AsRef<Path>,
) -> io::Result<CsvInput> {
    let path = path.as_ref();
    let contents = fs::read_to_string(path)?;
    let source_ref_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let source_ref = source_ref_path
        .to_string_lossy()
        .replace('\\', "/")
        .trim_start_matches("//?/")
        .to_string();
    Ok(CsvInput::new(dataset_kind, source_ref, contents))
}
