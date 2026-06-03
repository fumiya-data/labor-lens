use laborlens_rust::contexts::ingest::application::run_ingest_workflow;
use laborlens_rust::contexts::ingest::interfaces::{CsvInput, IngestRunCommand};
use laborlens_rust::shared::RunId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct LocalServer;

impl Default for LocalServer {
    fn default() -> Self {
        Self
    }
}

#[derive(Debug, Clone)]
pub struct LocalServerRunRequest {
    pub run_id: RunId,
    pub employees_csv: CsvInput,
    pub attendance_csv: CsvInput,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactListing {
    pub artifact_name: String,
    pub stable_path: String,
    pub content_type: String,
}

impl ArtifactListing {
    pub fn run_summary(stable_path: impl Into<String>) -> Self {
        Self {
            artifact_name: "run_summary".to_string(),
            stable_path: stable_path.into(),
            content_type: "application/json".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalServerRunResponse {
    pub run_id: RunId,
    pub job_state: String,
    pub progress_percent: u8,
    pub artifacts: Vec<ArtifactListing>,
}

impl LocalServer {
    pub fn start_run(&self, request: LocalServerRunRequest) -> LocalServerRunResponse {
        let result = run_ingest_workflow(IngestRunCommand::new(
            request.run_id,
            request.employees_csv,
            request.attendance_csv,
        ));

        LocalServerRunResponse {
            run_id: result.run_id,
            job_state: result.job.current_state.as_str().to_string(),
            progress_percent: result.job.progress_percent,
            artifacts: artifact_list(),
        }
    }
}

fn artifact_list() -> Vec<ArtifactListing> {
    vec![
        ArtifactListing::run_summary("run_summary.json"),
        ArtifactListing {
            artifact_name: "issues".to_string(),
            stable_path: "issues.csv".to_string(),
            content_type: "text/csv".to_string(),
        },
        ArtifactListing {
            artifact_name: "public_report_model".to_string(),
            stable_path: "public_report_model.json".to_string(),
            content_type: "application/json".to_string(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use crate::{ArtifactListing, LocalServer, LocalServerRunRequest};
    use laborlens_rust::contexts::ingest::domain::DatasetKind;
    use laborlens_rust::contexts::ingest::infrastructure::load_csv_input_from_path;
    use laborlens_rust::shared::RunId;
    use std::path::PathBuf;

    fn fixture_path(relative_path: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join(relative_path)
    }

    #[test]
    fn start_run_returns_job_progress_and_artifact_listing() {
        let server = LocalServer::default();
        let request = LocalServerRunRequest {
            run_id: RunId::new("run-local-server-001"),
            employees_csv: load_csv_input_from_path(
                DatasetKind::Employees,
                fixture_path("fixtures/valid/ingest/employees.csv"),
            )
            .expect("employees fixture should load"),
            attendance_csv: load_csv_input_from_path(
                DatasetKind::Attendance,
                fixture_path("fixtures/valid/ingest/attendance.csv"),
            )
            .expect("attendance fixture should load"),
        };

        let response = server.start_run(request);

        assert_eq!(response.run_id.as_str(), "run-local-server-001");
        assert_eq!(response.job_state, "succeeded");
        assert_eq!(response.progress_percent, 100);
        assert!(response
            .artifacts
            .iter()
            .any(|artifact| artifact == &ArtifactListing::run_summary("run_summary.json")));
    }
}
