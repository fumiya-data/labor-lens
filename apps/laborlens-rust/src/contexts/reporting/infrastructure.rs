//! Reporting adapters.
//!
//! Filesystem writes, serialization adapters, and generated-output paths belong
//! here. PDF rendering remains a downstream renderer concern.

use super::domain::PublicReportArtifacts;
use super::interfaces::to_python_renderer_json;
use std::fs;
use std::io;
use std::path::Path;

pub fn write_python_renderer_json(
    path: impl AsRef<Path>,
    artifacts: &PublicReportArtifacts,
) -> io::Result<()> {
    let json = to_python_renderer_json(artifacts)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    fs::write(path, json)
}
