use laborlens_rust::contexts::ingest::application::run_ingest_workflow;
use laborlens_rust::contexts::ingest::domain::DatasetKind;
use laborlens_rust::contexts::ingest::infrastructure::load_csv_input_from_path;
use laborlens_rust::contexts::ingest::interfaces::IngestRunCommand;
use laborlens_rust::contexts::privacy_safety::application::filter_public_report;
use laborlens_rust::contexts::privacy_safety::domain::{
    InputTrace, InternalEmployeeProfile, InternalWorkforceDataset, PrivacyPolicy,
};
use laborlens_rust::contexts::reporting::application::build_public_artifacts;
use laborlens_rust::contexts::reporting::interfaces::to_python_renderer_json;
use laborlens_rust::shared::RunId;
use std::path::PathBuf;

fn main() {
    if std::env::args().any(|argument| argument == "--ingest-smoke") {
        run_ingest_smoke();
        return;
    }

    run_public_report_smoke();
}

fn run_public_report_smoke() {
    let public_report = filter_public_report(smoke_internal_dataset());
    let artifacts = build_public_artifacts(&public_report);
    let json = to_python_renderer_json(&artifacts)
        .expect("smoke public report artifacts should serialize to JSON");

    println!("{json}");
}

fn run_ingest_smoke() {
    let employees = load_csv_input_from_path(
        DatasetKind::Employees,
        workspace_fixture_path("fixtures/valid/ingest/employees.csv"),
    )
    .expect("valid employees fixture should load for ingest smoke");
    let attendance = load_csv_input_from_path(
        DatasetKind::Attendance,
        workspace_fixture_path("fixtures/valid/ingest/attendance.csv"),
    )
    .expect("valid attendance fixture should load for ingest smoke");

    let result = run_ingest_workflow(IngestRunCommand::new(
        RunId::new("run-ingest-smoke-001"),
        employees,
        attendance,
    ));
    let json = serde_json::to_string_pretty(&result).expect("ingest smoke result should serialize");

    println!("{json}");
}

fn workspace_fixture_path(relative_path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(relative_path)
}

fn smoke_internal_dataset() -> InternalWorkforceDataset {
    InternalWorkforceDataset {
        run_id: RunId::new("run-smoke-001"),
        input_traces: vec![
            InputTrace {
                dataset_id: "attendance_by_employee".to_string(),
                source_ref: "fixtures/internal/attendance.csv".to_string(),
                fingerprint: "sha256:smoke-attendance".to_string(),
                record_count: 1,
            },
            InputTrace {
                dataset_id: "fatigue_by_employee".to_string(),
                source_ref: "fixtures/internal/fatigue.csv".to_string(),
                fingerprint: "sha256:smoke-fatigue".to_string(),
                record_count: 1,
            },
        ],
        policy: PrivacyPolicy {
            policy_id: "privacy-safety-v1".to_string(),
            version: "2026-06-03".to_string(),
        },
        employees: vec![InternalEmployeeProfile {
            employee_ref: "EMP-SMOKE-001".to_string(),
            group_key: "operations".to_string(),
            attendance_days_observed: 20,
            fatigue_value: Some(88),
            sleep_duration_hours: Some(5.5),
            fatigue_comment: Some("internal smoke fatigue note".to_string()),
        }],
    }
}
