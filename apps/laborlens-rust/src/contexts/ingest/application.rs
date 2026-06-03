//! Ingest use cases.
//!
//! Coordinates source registration, CSV reading, validation, and normalized
//! output contracts without exposing filesystem details to other contexts.

use super::domain::{
    header_specs_for, AttendanceRecord, EmployeeRecord, IngestRowCounts, IngestRunSummary,
    IngestWorkflowResult, InputRef, JobWorkflow, NormalizedIngestRecords, SchemaIssue,
    SchemaIssueKind,
};
use super::interfaces::{CsvInput, IngestRunCommand};
use csv::{ReaderBuilder, StringRecord, Trim};
use std::collections::HashMap;

pub fn run_ingest_workflow(command: IngestRunCommand) -> IngestWorkflowResult {
    let run_id = command.run_id.clone();
    let mut job = JobWorkflow::queued(run_id.clone());
    let input_refs = vec![
        input_ref_for(&run_id, &command.employees_csv),
        input_ref_for(&run_id, &command.attendance_csv),
    ];

    job.mark_running();

    let employees = parse_employees(&command.employees_csv);
    let attendance = parse_attendance(&command.attendance_csv);
    let mut issues = Vec::new();
    issues.extend(employees.issues);
    issues.extend(attendance.issues);

    let row_counts = IngestRowCounts {
        employee_rows: employees.records.len(),
        attendance_rows: attendance.records.len(),
    };
    let records = NormalizedIngestRecords {
        employees: employees.records,
        attendance: attendance.records,
    };

    if issues.is_empty() {
        job.mark_succeeded();
    } else {
        job.mark_failed(format!(
            "ingest 中に schema issue が {} 件見つかった",
            issues.len()
        ));
    }

    let run_summary = IngestRunSummary {
        run_id: run_id.clone(),
        input_count: input_refs.len(),
        employee_rows: row_counts.employee_rows,
        attendance_rows: row_counts.attendance_rows,
        issue_count: issues.len(),
        succeeded: issues.is_empty(),
    };

    IngestWorkflowResult {
        run_id,
        job,
        input_refs,
        row_counts,
        issues,
        run_summary,
        records,
    }
}

struct ParsedDataset<T> {
    records: Vec<T>,
    issues: Vec<SchemaIssue>,
}

fn parse_employees(input: &CsvInput) -> ParsedDataset<EmployeeRecord> {
    parse_csv(input, |record, headers| EmployeeRecord {
        employee_id: required_value(record, headers, "employee_id"),
        employee_name: required_value(record, headers, "employee_name"),
        department: required_value(record, headers, "department"),
        hire_date: optional_value(record, headers, "hire_date"),
        employment_status: optional_value(record, headers, "employment_status"),
    })
}

fn parse_attendance(input: &CsvInput) -> ParsedDataset<AttendanceRecord> {
    parse_csv(input, |record, headers| AttendanceRecord {
        employee_id: required_value(record, headers, "employee_id"),
        work_date: required_value(record, headers, "work_date"),
        clock_in: required_value(record, headers, "clock_in"),
        clock_out: required_value(record, headers, "clock_out"),
        break_minutes: optional_value(record, headers, "break_minutes")
            .and_then(|value| value.parse::<u32>().ok()),
    })
}

fn parse_csv<T>(
    input: &CsvInput,
    build_record: impl Fn(&StringRecord, &HashMap<&'static str, usize>) -> T,
) -> ParsedDataset<T> {
    let mut reader = ReaderBuilder::new()
        .trim(Trim::All)
        .from_reader(input.contents.as_bytes());
    let headers = match reader.headers() {
        Ok(headers) => headers.clone(),
        Err(error) => {
            return ParsedDataset {
                records: Vec::new(),
                issues: vec![csv_read_issue(input, error.to_string())],
            };
        }
    };

    let header_map = canonical_header_map(input, &headers);
    let missing_issues = missing_required_header_issues(input, &header_map);
    if !missing_issues.is_empty() {
        return ParsedDataset {
            records: Vec::new(),
            issues: missing_issues,
        };
    }

    let mut records = Vec::new();
    let mut issues = Vec::new();
    for record in reader.records() {
        match record {
            Ok(record) => records.push(build_record(&record, &header_map)),
            Err(error) => issues.push(csv_read_issue(input, error.to_string())),
        }
    }

    ParsedDataset { records, issues }
}

fn canonical_header_map(input: &CsvInput, headers: &StringRecord) -> HashMap<&'static str, usize> {
    let mut mapped = HashMap::new();
    for spec in header_specs_for(input.dataset_kind) {
        if let Some(index) = headers.iter().position(|header| {
            spec.accepted_headers
                .iter()
                .any(|accepted| header.trim() == *accepted)
        }) {
            mapped.insert(spec.canonical_name, index);
        }
    }
    mapped
}

fn missing_required_header_issues(
    input: &CsvInput,
    header_map: &HashMap<&'static str, usize>,
) -> Vec<SchemaIssue> {
    header_specs_for(input.dataset_kind)
        .iter()
        .filter(|spec| spec.required && !header_map.contains_key(spec.canonical_name))
        .map(|spec| SchemaIssue {
            issue_id: format!(
                "schema:{}:missing_required_header:{}",
                input.dataset_kind.dataset_id(),
                spec.canonical_name
            ),
            dataset_id: input.dataset_kind.dataset_id().to_string(),
            source_ref: input.source_ref.clone(),
            issue_kind: SchemaIssueKind::MissingRequiredHeader,
            message: format!(
                "必須 header `{}` が見つからない。受け付ける header: {}。",
                spec.canonical_name,
                spec.accepted_headers.join(", ")
            ),
            canonical_header: Some(spec.canonical_name.to_string()),
            accepted_headers: spec
                .accepted_headers
                .iter()
                .map(|header| (*header).to_string())
                .collect(),
        })
        .collect()
}

fn csv_read_issue(input: &CsvInput, message: String) -> SchemaIssue {
    SchemaIssue {
        issue_id: format!("schema:{}:csv_read_error", input.dataset_kind.dataset_id()),
        dataset_id: input.dataset_kind.dataset_id().to_string(),
        source_ref: input.source_ref.clone(),
        issue_kind: SchemaIssueKind::CsvReadError,
        message,
        canonical_header: None,
        accepted_headers: Vec::new(),
    }
}

fn required_value(
    record: &StringRecord,
    headers: &HashMap<&'static str, usize>,
    canonical_name: &'static str,
) -> String {
    record
        .get(headers[canonical_name])
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn optional_value(
    record: &StringRecord,
    headers: &HashMap<&'static str, usize>,
    canonical_name: &'static str,
) -> Option<String> {
    headers
        .get(canonical_name)
        .and_then(|index| record.get(*index))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn input_ref_for(run_id: &crate::shared::RunId, input: &CsvInput) -> InputRef {
    InputRef {
        run_id: run_id.clone(),
        dataset_id: input.dataset_kind.dataset_id().to_string(),
        source_ref: input.source_ref.clone(),
        fingerprint: deterministic_fingerprint(input.contents.as_bytes()),
        record_count: csv_record_count(&input.contents),
        schema_version: input.dataset_kind.schema_version().to_string(),
    }
}

fn csv_record_count(contents: &str) -> usize {
    ReaderBuilder::new()
        .trim(Trim::All)
        .from_reader(contents.as_bytes())
        .records()
        .filter(Result::is_ok)
        .count()
}

fn deterministic_fingerprint(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

#[cfg(test)]
mod tests {
    use crate::contexts::ingest::application::run_ingest_workflow;
    use crate::contexts::ingest::domain::{DatasetKind, JobState, SchemaIssueKind};
    use crate::contexts::ingest::infrastructure::load_csv_input_from_path;
    use crate::contexts::ingest::interfaces::IngestRunCommand;
    use crate::shared::RunId;
    use std::path::PathBuf;

    fn fixture_path(relative_path: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join(relative_path)
    }

    fn valid_command() -> IngestRunCommand {
        let employees = load_csv_input_from_path(
            DatasetKind::Employees,
            fixture_path("fixtures/valid/ingest/employees.csv"),
        )
        .expect("valid employees fixture should load");
        let attendance = load_csv_input_from_path(
            DatasetKind::Attendance,
            fixture_path("fixtures/valid/ingest/attendance.csv"),
        )
        .expect("valid attendance fixture should load");

        IngestRunCommand::new(RunId::new("run-ingest-valid-001"), employees, attendance)
    }

    fn invalid_command() -> IngestRunCommand {
        let employees = load_csv_input_from_path(
            DatasetKind::Employees,
            fixture_path("fixtures/invalid/ingest/employees_missing_required_header.csv"),
        )
        .expect("invalid employees fixture should load");
        let attendance = load_csv_input_from_path(
            DatasetKind::Attendance,
            fixture_path("fixtures/valid/ingest/attendance.csv"),
        )
        .expect("valid attendance fixture should load");

        IngestRunCommand::new(RunId::new("run-ingest-invalid-001"), employees, attendance)
    }

    #[test]
    fn reads_valid_japanese_employees_and_attendance_with_input_refs() {
        let result = run_ingest_workflow(valid_command());

        assert_eq!(result.job.current_state, JobState::Succeeded);
        assert!(result.issues.is_empty());
        assert_eq!(result.row_counts.employee_rows, 2);
        assert_eq!(result.row_counts.attendance_rows, 3);
        assert_eq!(result.input_refs.len(), 2);
        assert!(result
            .input_refs
            .iter()
            .any(|input| input.dataset_id == "employees"
                && input
                    .source_ref
                    .ends_with("fixtures/valid/ingest/employees.csv")
                && input.fingerprint.starts_with("fnv1a64:")
                && input.record_count == 2));
        assert!(result
            .input_refs
            .iter()
            .any(|input| input.dataset_id == "attendance_by_employee"
                && input
                    .source_ref
                    .ends_with("fixtures/valid/ingest/attendance.csv")
                && input.fingerprint.starts_with("fnv1a64:")
                && input.record_count == 3));
        assert_eq!(result.records.employees[0].employee_id, "E001");
        assert_eq!(result.records.attendance[0].work_date, "2026-01-05");
    }

    #[test]
    fn missing_required_header_becomes_schema_issue() {
        let result = run_ingest_workflow(invalid_command());

        let issue = result
            .issues
            .iter()
            .find(|issue| issue.issue_kind == SchemaIssueKind::MissingRequiredHeader)
            .expect("missing required header should be reported");

        assert_eq!(issue.dataset_id, "employees");
        assert_eq!(issue.canonical_header.as_deref(), Some("employee_id"));
        assert!(issue
            .accepted_headers
            .iter()
            .any(|header| header == "社員ID"));
        assert!(issue.message.contains("employee_id"));
    }

    #[test]
    fn job_workflow_records_queued_running_succeeded() {
        let result = run_ingest_workflow(valid_command());
        let states: Vec<JobState> = result
            .job
            .history
            .iter()
            .map(|transition| transition.state.clone())
            .collect();

        assert_eq!(
            states,
            vec![JobState::Queued, JobState::Running, JobState::Succeeded]
        );
        assert_eq!(result.job.progress_percent, 100);
        assert!(result.job.failure_reason.is_none());
        assert!(result.run_summary.succeeded);
    }

    #[test]
    fn invalid_ingest_records_failed_state_and_failure_reason() {
        let result = run_ingest_workflow(invalid_command());
        let states: Vec<JobState> = result
            .job
            .history
            .iter()
            .map(|transition| transition.state.clone())
            .collect();

        assert_eq!(result.job.current_state, JobState::Failed);
        assert_eq!(
            states,
            vec![JobState::Queued, JobState::Running, JobState::Failed]
        );
        assert_eq!(result.run_summary.issue_count, 1);
        assert!(!result.run_summary.succeeded);
        assert!(result
            .job
            .failure_reason
            .as_deref()
            .expect("failed job should carry a reason")
            .contains("schema issue"));
    }
}
