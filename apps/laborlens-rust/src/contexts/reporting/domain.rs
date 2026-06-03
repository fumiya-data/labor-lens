//! Reporting domain terms.
//!
//! Owns report model vocabulary, artifact identity, issue output contracts, and
//! public report structure after privacy/safety has approved the data.

use crate::contexts::privacy_safety::domain::{
    InputTrace, PolicyTrace, PublicProfileGroup, SuppressionSummary,
};
use crate::shared::RunId;
use serde::{Deserialize, Serialize};

pub const PUBLIC_REPORT_CONTRACT_VERSION: &str = "laborlens.public_report.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputTrace {
    pub artifact_name: String,
    pub artifact_kind: String,
    pub stable_path: String,
    pub content_schema: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactManifest {
    pub run_id: RunId,
    pub contract_version: String,
    pub input_traces: Vec<InputTrace>,
    pub policy_trace: PolicyTrace,
    pub output_traces: Vec<OutputTrace>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunSummary {
    pub run_id: RunId,
    pub employee_count: usize,
    pub profile_count: usize,
    pub suppressed_category_count: usize,
    pub suppressed_field_count: usize,
    pub issue_count: usize,
    pub policy_id: String,
    pub policy_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicIssue {
    pub issue_id: String,
    pub severity: String,
    pub message: String,
    pub suppression_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileReport {
    pub run_id: RunId,
    pub profiles: Vec<PublicProfileGroup>,
    pub suppression_summary: Vec<SuppressionSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicReportArtifacts {
    pub run_id: RunId,
    pub artifact_manifest: ArtifactManifest,
    pub run_summary: RunSummary,
    pub issues: Vec<PublicIssue>,
    pub profile_report: ProfileReport,
}
