//! Reporting boundary DTOs.
//!
//! Shapes handed to CLI, local server, UI, and Python rendering tools.

use super::domain::{
    ArtifactManifest, ProfileReport, PublicIssue, PublicReportArtifacts, RunSummary,
    PUBLIC_REPORT_CONTRACT_VERSION,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct PythonRendererContract<'a> {
    pub contract_version: &'static str,
    pub artifact_manifest: &'a ArtifactManifest,
    pub run_summary: &'a RunSummary,
    pub issues: &'a [PublicIssue],
    pub profile_report: &'a ProfileReport,
}

impl<'a> From<&'a PublicReportArtifacts> for PythonRendererContract<'a> {
    fn from(artifacts: &'a PublicReportArtifacts) -> Self {
        Self {
            contract_version: PUBLIC_REPORT_CONTRACT_VERSION,
            artifact_manifest: &artifacts.artifact_manifest,
            run_summary: &artifacts.run_summary,
            issues: &artifacts.issues,
            profile_report: &artifacts.profile_report,
        }
    }
}

pub fn to_python_renderer_json(
    artifacts: &PublicReportArtifacts,
) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&PythonRendererContract::from(artifacts))
}
