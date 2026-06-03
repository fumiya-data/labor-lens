//! Ingest domain terms.
//!
//! Owns source dataset identity, header mapping vocabulary, row validity, and
//! normalized handoff concepts. It should not create business recommendations.

use crate::shared::RunId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DatasetKind {
    Employees,
    Attendance,
}

impl DatasetKind {
    pub fn dataset_id(self) -> &'static str {
        match self {
            Self::Employees => "employees",
            Self::Attendance => "attendance_by_employee",
        }
    }

    pub fn schema_version(self) -> &'static str {
        match self {
            Self::Employees => "employees_csv.v1",
            Self::Attendance => "attendance_csv.v1",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeaderSpec {
    pub canonical_name: &'static str,
    pub accepted_headers: &'static [&'static str],
    pub required: bool,
}

pub const EMPLOYEE_HEADER_SPECS: &[HeaderSpec] = &[
    HeaderSpec {
        canonical_name: "employee_id",
        accepted_headers: &["employee_id", "社員ID", "従業員ID"],
        required: true,
    },
    HeaderSpec {
        canonical_name: "employee_name",
        accepted_headers: &["employee_name", "氏名", "社員名"],
        required: true,
    },
    HeaderSpec {
        canonical_name: "department",
        accepted_headers: &["department", "部署", "部門"],
        required: true,
    },
    HeaderSpec {
        canonical_name: "hire_date",
        accepted_headers: &["hire_date", "入社日"],
        required: false,
    },
    HeaderSpec {
        canonical_name: "employment_status",
        accepted_headers: &["employment_status", "雇用状態", "在籍状態"],
        required: false,
    },
];

pub const ATTENDANCE_HEADER_SPECS: &[HeaderSpec] = &[
    HeaderSpec {
        canonical_name: "employee_id",
        accepted_headers: &["employee_id", "社員ID", "従業員ID"],
        required: true,
    },
    HeaderSpec {
        canonical_name: "work_date",
        accepted_headers: &["work_date", "勤務日", "日付"],
        required: true,
    },
    HeaderSpec {
        canonical_name: "clock_in",
        accepted_headers: &["clock_in", "出勤時刻", "開始時刻"],
        required: true,
    },
    HeaderSpec {
        canonical_name: "clock_out",
        accepted_headers: &["clock_out", "退勤時刻", "終了時刻"],
        required: true,
    },
    HeaderSpec {
        canonical_name: "break_minutes",
        accepted_headers: &["break_minutes", "休憩分", "休憩時間"],
        required: false,
    },
];

pub fn header_specs_for(dataset_kind: DatasetKind) -> &'static [HeaderSpec] {
    match dataset_kind {
        DatasetKind::Employees => EMPLOYEE_HEADER_SPECS,
        DatasetKind::Attendance => ATTENDANCE_HEADER_SPECS,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputRef {
    pub run_id: RunId,
    pub dataset_id: String,
    pub source_ref: String,
    pub fingerprint: String,
    pub record_count: usize,
    pub schema_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SchemaIssueKind {
    MissingRequiredHeader,
    CsvReadError,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaIssue {
    pub issue_id: String,
    pub dataset_id: String,
    pub source_ref: String,
    pub issue_kind: SchemaIssueKind,
    pub message: String,
    pub canonical_header: Option<String>,
    pub accepted_headers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmployeeRecord {
    pub employee_id: String,
    pub employee_name: String,
    pub department: String,
    pub hire_date: Option<String>,
    pub employment_status: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttendanceRecord {
    pub employee_id: String,
    pub work_date: String,
    pub clock_in: String,
    pub clock_out: String,
    pub break_minutes: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct NormalizedIngestRecords {
    pub employees: Vec<EmployeeRecord>,
    pub attendance: Vec<AttendanceRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct IngestRowCounts {
    pub employee_rows: usize,
    pub attendance_rows: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobState {
    Queued,
    Running,
    Succeeded,
    Failed,
}

impl JobState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobTransition {
    pub state: JobState,
    pub progress_percent: u8,
    pub stage: String,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobWorkflow {
    pub job_id: String,
    pub run_id: RunId,
    pub current_state: JobState,
    pub progress_percent: u8,
    pub failure_reason: Option<String>,
    pub history: Vec<JobTransition>,
}

impl JobWorkflow {
    pub fn queued(run_id: RunId) -> Self {
        let job_id = format!("job:{}", run_id.as_str());
        Self {
            job_id,
            run_id,
            current_state: JobState::Queued,
            progress_percent: 0,
            failure_reason: None,
            history: vec![JobTransition {
                state: JobState::Queued,
                progress_percent: 0,
                stage: "queued".to_string(),
                message: Some("ingest run をキューに登録した".to_string()),
            }],
        }
    }

    pub fn mark_running(&mut self) {
        self.transition(
            JobState::Running,
            25,
            "input_registration",
            Some("input ref を登録した".to_string()),
        );
    }

    pub fn mark_succeeded(&mut self) {
        self.transition(
            JobState::Succeeded,
            100,
            "completed",
            Some("ingest が完了した".to_string()),
        );
    }

    pub fn mark_failed(&mut self, reason: impl Into<String>) {
        let reason = reason.into();
        self.failure_reason = Some(reason.clone());
        self.transition(JobState::Failed, 100, "failed", Some(reason));
    }

    fn transition(
        &mut self,
        state: JobState,
        progress_percent: u8,
        stage: impl Into<String>,
        message: Option<String>,
    ) {
        self.current_state = state.clone();
        self.progress_percent = progress_percent;
        self.history.push(JobTransition {
            state,
            progress_percent,
            stage: stage.into(),
            message,
        });
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IngestRunSummary {
    pub run_id: RunId,
    pub input_count: usize,
    pub employee_rows: usize,
    pub attendance_rows: usize,
    pub issue_count: usize,
    pub succeeded: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IngestWorkflowResult {
    pub run_id: RunId,
    pub job: JobWorkflow,
    pub input_refs: Vec<InputRef>,
    pub row_counts: IngestRowCounts,
    pub issues: Vec<SchemaIssue>,
    pub run_summary: IngestRunSummary,
    pub records: NormalizedIngestRecords,
}
