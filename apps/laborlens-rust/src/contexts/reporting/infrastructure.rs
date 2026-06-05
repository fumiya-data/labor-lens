//! Reporting adapters.
//!
//! Filesystem writes, serialization adapters, and generated-output paths belong
//! here. PDF rendering remains a downstream renderer concern.

use super::domain::PublicReportArtifacts;
use super::interfaces::to_python_renderer_json;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WrittenArtifact {
    pub artifact_name: String,
    pub path: PathBuf,
}

pub fn write_python_renderer_json(
    path: impl AsRef<Path>,
    artifacts: &PublicReportArtifacts,
) -> io::Result<()> {
    let json = to_python_renderer_json(artifacts)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    fs::write(path, json)
}

pub fn write_public_artifact_files(
    output_dir: impl AsRef<Path>,
    artifacts: &PublicReportArtifacts,
) -> io::Result<Vec<WrittenArtifact>> {
    let output_dir = output_dir.as_ref();
    fs::create_dir_all(output_dir)?;

    let mut written = Vec::new();
    write_text_artifact(
        output_dir,
        "public_report_model",
        "public_report_model.json",
        &to_python_renderer_json(artifacts)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?,
        &mut written,
    )?;
    write_json_artifact(
        output_dir,
        "artifact_manifest",
        "artifact_manifest.json",
        &artifacts.artifact_manifest,
        &mut written,
    )?;
    write_json_artifact(
        output_dir,
        "run_summary",
        "run_summary.json",
        &artifacts.run_summary,
        &mut written,
    )?;
    write_issues_csv(output_dir.join("issues.csv"), artifacts)?;
    written.push(WrittenArtifact {
        artifact_name: "issues".to_string(),
        path: output_dir.join("issues.csv"),
    });
    write_privacy_suppressions_csv(output_dir.join("privacy_suppressions.csv"), artifacts)?;
    written.push(WrittenArtifact {
        artifact_name: "privacy_suppressions".to_string(),
        path: output_dir.join("privacy_suppressions.csv"),
    });

    Ok(written)
}

fn write_json_artifact<T: serde::Serialize>(
    output_dir: &Path,
    artifact_name: &str,
    file_name: &str,
    value: &T,
    written: &mut Vec<WrittenArtifact>,
) -> io::Result<()> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    write_text_artifact(output_dir, artifact_name, file_name, &json, written)
}

fn write_text_artifact(
    output_dir: &Path,
    artifact_name: &str,
    file_name: &str,
    contents: &str,
    written: &mut Vec<WrittenArtifact>,
) -> io::Result<()> {
    let path = output_dir.join(file_name);
    fs::write(&path, contents)?;
    written.push(WrittenArtifact {
        artifact_name: artifact_name.to_string(),
        path,
    });
    Ok(())
}

fn write_issues_csv(path: PathBuf, artifacts: &PublicReportArtifacts) -> io::Result<()> {
    let mut writer = csv::Writer::from_path(path).map_err(csv_error)?;
    writer
        .write_record(["issue_id", "severity", "suppression_code", "message"])
        .map_err(csv_error)?;
    for issue in &artifacts.issues {
        writer
            .write_record([
                issue.issue_id.as_str(),
                issue.severity.as_str(),
                issue.suppression_code.as_deref().unwrap_or_default(),
                issue.message.as_str(),
            ])
            .map_err(csv_error)?;
    }
    writer.flush()?;
    Ok(())
}

fn write_privacy_suppressions_csv(
    path: PathBuf,
    artifacts: &PublicReportArtifacts,
) -> io::Result<()> {
    let mut writer = csv::Writer::from_path(path).map_err(csv_error)?;
    writer
        .write_record([
            "suppression_code",
            "category",
            "affected_record_count",
            "suppressed_field_count",
            "reason",
        ])
        .map_err(csv_error)?;
    for suppression in &artifacts.profile_report.suppression_summary {
        writer
            .write_record([
                suppression.suppression_code.as_str(),
                suppression.category.as_str(),
                &suppression.affected_record_count.to_string(),
                &suppression.suppressed_field_count.to_string(),
                suppression.reason.as_str(),
            ])
            .map_err(csv_error)?;
    }
    writer.flush()?;
    Ok(())
}

fn csv_error(error: csv::Error) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}

#[cfg(test)]
mod tests {
    use crate::contexts::privacy_safety::application::filter_public_report;
    use crate::contexts::privacy_safety::domain::{
        InputTrace, InternalEmployeeProfile, InternalWorkforceDataset, PrivacyPolicy,
    };
    use crate::contexts::reporting::application::build_public_artifacts;
    use crate::contexts::reporting::infrastructure::write_public_artifact_files;
    use crate::shared::RunId;
    use std::fs;
    use std::path::PathBuf;

    fn temp_output_dir() -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("laborlens-reporting-test-{}", std::process::id()));
        if dir.exists() {
            fs::remove_dir_all(&dir).expect("old temp output dir should be removable");
        }
        fs::create_dir_all(&dir).expect("temp output dir should be creatable");
        dir
    }

    fn safe_public_artifacts() -> crate::contexts::reporting::domain::PublicReportArtifacts {
        let mut employees = Vec::new();
        employees.push(InternalEmployeeProfile {
            employee_ref: "EMP-PRIVATE-001".to_string(),
            group_key: "operations".to_string(),
            attendance_days_observed: 20,
            fatigue_value: Some(88),
            sleep_duration_hours: Some(5.5),
            fatigue_comment: Some("private fatigue comment".to_string()),
        });
        employees.extend((2..=10).map(|index| InternalEmployeeProfile {
            employee_ref: format!("EMP-PRIVATE-{index:03}"),
            group_key: "operations".to_string(),
            attendance_days_observed: 20,
            fatigue_value: None,
            sleep_duration_hours: None,
            fatigue_comment: None,
        }));
        let public_report = filter_public_report(InternalWorkforceDataset {
            run_id: RunId::new("run-artifact-files-001"),
            input_traces: vec![InputTrace {
                dataset_id: "attendance_by_employee".to_string(),
                source_ref: "fixtures/internal/attendance.csv".to_string(),
                fingerprint: "sha256:attendance".to_string(),
                record_count: 10,
            }],
            policy: PrivacyPolicy {
                policy_id: "privacy-safety-v1".to_string(),
                version: "2026-06-03".to_string(),
            },
            employees,
        });
        build_public_artifacts(&public_report)
    }

    #[test]
    fn writes_split_public_artifact_files_with_stable_names() {
        let artifacts = safe_public_artifacts();
        let output_dir = temp_output_dir();

        let written = write_public_artifact_files(&output_dir, &artifacts)
            .expect("public artifact files should be written");

        let names: Vec<String> = written
            .iter()
            .map(|artifact| {
                artifact
                    .path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            })
            .collect();
        assert_eq!(
            names,
            vec![
                "public_report_model.json",
                "artifact_manifest.json",
                "run_summary.json",
                "issues.csv",
                "privacy_suppressions.csv"
            ]
        );

        let issues_csv = fs::read_to_string(output_dir.join("issues.csv"))
            .expect("issues.csv should be readable");
        assert!(issues_csv.starts_with("issue_id,severity,suppression_code,message"));
        assert!(issues_csv.contains("PERSONAL_HEALTH_DETAIL_SUPPRESSED"));

        let privacy_csv = fs::read_to_string(output_dir.join("privacy_suppressions.csv"))
            .expect("privacy_suppressions.csv should be readable");
        assert!(privacy_csv.starts_with(
            "suppression_code,category,affected_record_count,suppressed_field_count,reason"
        ));
        assert!(!privacy_csv.contains("private fatigue comment"));

        fs::remove_dir_all(output_dir).expect("temp output dir should be removed");
    }
}
