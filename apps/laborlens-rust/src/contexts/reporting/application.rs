//! Reporting use cases.
//!
//! Creates deterministic JSON, CSV, Markdown, and artifact manifest outputs
//! for Rust tests and downstream Python rendering.

use super::domain::{
    ArtifactManifest, OutputTrace, ProfileReport, PublicIssue, PublicReportArtifacts, RunSummary,
    PUBLIC_REPORT_CONTRACT_VERSION,
};
use crate::contexts::privacy_safety::domain::PublicReport;

pub fn build_public_artifacts(public_report: &PublicReport) -> PublicReportArtifacts {
    let issues = build_issues(public_report);
    let run_summary = RunSummary {
        run_id: public_report.run_id.clone(),
        employee_count: public_report
            .profiles
            .iter()
            .map(|profile| profile.employee_count)
            .sum(),
        profile_count: public_report.profiles.len(),
        suppressed_category_count: public_report.suppression_summary.len(),
        suppressed_field_count: public_report
            .suppression_summary
            .iter()
            .map(|summary| summary.suppressed_field_count)
            .sum(),
        issue_count: issues.len(),
        policy_id: public_report.policy_trace.policy_id.clone(),
        policy_version: public_report.policy_trace.version.clone(),
    };
    let profile_report = ProfileReport {
        run_id: public_report.run_id.clone(),
        profiles: public_report.profiles.clone(),
        suppression_summary: public_report.suppression_summary.clone(),
    };
    let artifact_manifest = ArtifactManifest {
        run_id: public_report.run_id.clone(),
        contract_version: PUBLIC_REPORT_CONTRACT_VERSION.to_string(),
        input_traces: public_report.input_traces.clone(),
        policy_trace: public_report.policy_trace.clone(),
        output_traces: output_traces(),
    };

    PublicReportArtifacts {
        run_id: public_report.run_id.clone(),
        artifact_manifest,
        run_summary,
        issues,
        profile_report,
    }
}

fn build_issues(public_report: &PublicReport) -> Vec<PublicIssue> {
    public_report
        .suppression_summary
        .iter()
        .map(|summary| PublicIssue {
            issue_id: format!("issue:{}", summary.suppression_code.to_ascii_lowercase()),
            severity: "info".to_string(),
            message: "個人の健康関連詳細は公開レポート生成前に抑制された。".to_string(),
            suppression_code: Some(summary.suppression_code.clone()),
        })
        .collect()
}

fn output_traces() -> Vec<OutputTrace> {
    vec![
        OutputTrace {
            artifact_name: "artifact_manifest".to_string(),
            artifact_kind: "json".to_string(),
            stable_path: "artifact_manifest.json".to_string(),
            content_schema: "laborlens.artifact_manifest.v1".to_string(),
        },
        OutputTrace {
            artifact_name: "run_summary".to_string(),
            artifact_kind: "json".to_string(),
            stable_path: "run_summary.json".to_string(),
            content_schema: "laborlens.run_summary.v1".to_string(),
        },
        OutputTrace {
            artifact_name: "issues".to_string(),
            artifact_kind: "json".to_string(),
            stable_path: "issues.json".to_string(),
            content_schema: "laborlens.issues.v1".to_string(),
        },
        OutputTrace {
            artifact_name: "profile_report".to_string(),
            artifact_kind: "json".to_string(),
            stable_path: "profile_report.json".to_string(),
            content_schema: "laborlens.profile_report.v1".to_string(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use crate::contexts::privacy_safety::application::filter_public_report;
    use crate::contexts::privacy_safety::domain::{
        InputTrace, InternalEmployeeProfile, InternalWorkforceDataset, PrivacyPolicy,
    };
    use crate::contexts::reporting::application::build_public_artifacts;
    use crate::contexts::reporting::interfaces::to_python_renderer_json;
    use crate::shared::RunId;

    fn sample_internal_dataset() -> InternalWorkforceDataset {
        InternalWorkforceDataset {
            run_id: RunId::new("run-privacy-smoke-001"),
            input_traces: vec![
                InputTrace {
                    dataset_id: "attendance_by_employee".to_string(),
                    source_ref: "fixtures/internal/attendance.csv".to_string(),
                    fingerprint: "sha256:attendance-demo".to_string(),
                    record_count: 1,
                },
                InputTrace {
                    dataset_id: "fatigue_by_employee".to_string(),
                    source_ref: "fixtures/internal/fatigue.csv".to_string(),
                    fingerprint: "sha256:fatigue-demo".to_string(),
                    record_count: 1,
                },
            ],
            policy: PrivacyPolicy {
                policy_id: "privacy-safety-v1".to_string(),
                version: "2026-06-03".to_string(),
            },
            employees: vec![InternalEmployeeProfile {
                employee_ref: "EMP-PRIVATE-001".to_string(),
                group_key: "operations".to_string(),
                attendance_days_observed: 21,
                fatigue_value: Some(97),
                sleep_duration_hours: Some(4.25),
                fatigue_comment: Some(
                    "raw fatigue marker comment should never reach Pike".to_string(),
                ),
            }],
        }
    }

    #[test]
    fn public_report_json_suppresses_personal_fatigue_sleep_and_comments() {
        let public_report = filter_public_report(sample_internal_dataset());
        let artifacts = build_public_artifacts(&public_report);
        let json = to_python_renderer_json(&artifacts).expect("renderer JSON should serialize");

        for private_fragment in [
            "EMP-PRIVATE-001",
            "97",
            "4.25",
            "raw fatigue marker comment",
            "fatigue_value",
            "sleep_duration_hours",
            "fatigue_comment",
        ] {
            assert!(
                !json.contains(private_fragment),
                "public renderer JSON leaked `{private_fragment}`: {json}"
            );
        }

        assert!(json.contains("PERSONAL_HEALTH_DETAIL_SUPPRESSED"));
    }

    #[test]
    fn public_artifact_manifest_tracks_run_input_policy_and_outputs() {
        let public_report = filter_public_report(sample_internal_dataset());
        let artifacts = build_public_artifacts(&public_report);

        assert_eq!(artifacts.run_id.as_str(), "run-privacy-smoke-001");
        assert_eq!(
            artifacts.artifact_manifest.run_id.as_str(),
            "run-privacy-smoke-001"
        );
        assert!(artifacts
            .artifact_manifest
            .input_traces
            .iter()
            .any(|trace| trace.dataset_id == "fatigue_by_employee"));
        assert_eq!(
            artifacts.artifact_manifest.policy_trace.policy_id,
            "privacy-safety-v1"
        );
        assert!(artifacts
            .artifact_manifest
            .output_traces
            .iter()
            .any(|trace| trace.artifact_name == "profile_report"));
    }

    #[test]
    fn python_renderer_contract_contains_only_suppressed_public_information() {
        let public_report = filter_public_report(sample_internal_dataset());
        let artifacts = build_public_artifacts(&public_report);
        let json = to_python_renderer_json(&artifacts).expect("renderer JSON should serialize");
        let contract: serde_json::Value =
            serde_json::from_str(&json).expect("renderer JSON should be parseable");

        assert_eq!(contract["contract_version"], "laborlens.public_report.v1");
        assert_eq!(contract["run_summary"]["run_id"], "run-privacy-smoke-001");
        assert!(contract.get("artifact_manifest").is_some());
        assert!(contract.get("profile_report").is_some());

        let profile = &contract["profile_report"]["profiles"][0];
        assert_eq!(profile["profile_id"], "group:operations");
        assert!(profile.get("fatigue_value").is_none());
        assert!(profile.get("sleep_duration_hours").is_none());
        assert!(profile.get("fatigue_comment").is_none());

        assert_eq!(
            contract["profile_report"]["suppression_summary"][0]["suppression_code"],
            "PERSONAL_HEALTH_DETAIL_SUPPRESSED"
        );
        assert!(!json.contains("raw fatigue marker comment"));
    }
}
