use laborlens_rust::contexts::ingest::application::run_ingest_workflow;
use laborlens_rust::contexts::ingest::domain::{DatasetKind, IngestWorkflowResult, SchemaIssue};
use laborlens_rust::contexts::ingest::interfaces::{CsvInput, IngestRunCommand};
use laborlens_rust::shared::db::postgres::PostgresCommandAdapter;
use laborlens_rust::shared::db::{
    InsertInputRef, InsertIssue, InsertJob, InsertRunRecord, JobState as DbJobState, SqlCommand,
    UpdateJobState,
};
use laborlens_rust::shared::RunId;
use postgres::{Client, NoTls};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static RUN_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone)]
pub struct LocalServer {
    demo_database: DemoDataStore,
    artifact_root: PathBuf,
    run_database_url: Option<String>,
}

impl Default for LocalServer {
    fn default() -> Self {
        let demo_database = env::var("LABORLENS_DEMO_DATABASE_URL")
            .ok()
            .and_then(|database_url| DemoDataStore::from_postgres(&database_url).ok())
            .filter(|database| database.employee_count() == 1_000)
            .unwrap_or_else(DemoDataStore::seeded);
        let artifact_root = env::var("LABORLENS_ARTIFACT_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| env::temp_dir().join("laborlens-local-artifacts"));
        let run_database_url = env::var("LABORLENS_RUN_DATABASE_URL").ok();

        Self {
            demo_database,
            artifact_root,
            run_database_url,
        }
    }
}

pub const USE_CASE_IDS: [&str; 14] = [
    "uc-01", "uc-02", "uc-03", "uc-04", "uc-05", "uc-06", "uc-07", "uc-08", "uc-09", "uc-10",
    "uc-11", "uc-12", "uc-13", "uc-14",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DemoEmployee {
    pub employee_id: String,
    pub display_name: String,
    pub department: String,
    pub store_name: String,
    pub employment_type: String,
    pub role_name: String,
    pub hired_on: String,
}

#[derive(Debug, Clone)]
pub struct DemoDataStore {
    employees: Vec<DemoEmployee>,
}

impl DemoDataStore {
    pub fn seeded() -> Self {
        Self {
            employees: (1..=1_000).map(build_demo_employee).collect(),
        }
    }

    pub fn from_postgres(database_url: &str) -> Result<Self, postgres::Error> {
        let mut client = Client::connect(database_url, NoTls)?;
        let rows = client.query(
            "SELECT employee_id, display_name, department, store_name, \
                    employment_type, role_name, hired_on::text \
             FROM laborlens.demo_employees \
             WHERE seed_version = 'demo_japanese_employees.v1' \
             ORDER BY employee_id",
            &[],
        )?;

        Ok(Self {
            employees: rows
                .into_iter()
                .map(|row| DemoEmployee {
                    employee_id: row.get(0),
                    display_name: row.get(1),
                    department: row.get(2),
                    store_name: row.get(3),
                    employment_type: row.get(4),
                    role_name: row.get(5),
                    hired_on: row.get(6),
                })
                .collect(),
        })
    }

    pub fn employee_count(&self) -> usize {
        self.employees.len()
    }

    pub fn employees(&self) -> &[DemoEmployee] {
        &self.employees
    }

    fn sample_employees(&self, use_case_index: usize) -> Vec<&DemoEmployee> {
        if self.employees.is_empty() {
            return Vec::new();
        }

        let start = (use_case_index * 37) % self.employees.len();
        [
            start,
            (start + 113) % self.employees.len(),
            (start + 271) % self.employees.len(),
        ]
        .into_iter()
        .map(|index| &self.employees[index])
        .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UseCaseDefinition {
    pub use_case_id: String,
    pub button_label: String,
    pub title: String,
    pub actor: String,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DemoDbSource {
    pub table_name: String,
    pub seed_version: String,
    pub employee_count: usize,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetricCard {
    pub label: String,
    pub value: String,
    pub unit: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UseCaseSampleRow {
    pub subject: String,
    pub group: String,
    pub primary_value: String,
    pub status: String,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UseCaseSampleResponse {
    pub use_case: UseCaseDefinition,
    pub source: DemoDbSource,
    pub metrics: Vec<MetricCard>,
    pub rows: Vec<UseCaseSampleRow>,
    pub findings: Vec<String>,
    pub next_actions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiResponse {
    pub status_code: u16,
    pub content_type: String,
    pub body: String,
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
    pub report_markdown_path: String,
    pub db_persistence_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuideMessageRequest {
    pub message: String,
    pub run_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuideMessageResponse {
    pub answer: String,
    pub mode: String,
    pub safety_boundary: String,
    pub references: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunHistoryItem {
    pub run_id: String,
    pub progress_path: String,
    pub artifacts_path: String,
    pub report_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttendanceReviewRow {
    pub row_id: String,
    pub employee_id: String,
    pub display_name: String,
    pub store_name: String,
    pub department: String,
    pub work_date: String,
    pub clock_in: Option<String>,
    pub clock_out: Option<String>,
    pub scheduled_clock_in: String,
    pub scheduled_clock_out: String,
    pub worked_minutes: Option<u16>,
    pub issue_type: String,
    pub issue_label: String,
    pub severity: String,
    pub status: String,
    pub review_hint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttendanceReviewCount {
    pub key: String,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttendanceReviewSummary {
    pub period_start: String,
    pub period_end: String,
    pub employee_count: usize,
    pub row_count: usize,
    pub issue_row_count: usize,
    pub counts_by_store: Vec<AttendanceReviewCount>,
    pub counts_by_department: Vec<AttendanceReviewCount>,
    pub counts_by_severity: Vec<AttendanceReviewCount>,
    pub counts_by_status: Vec<AttendanceReviewCount>,
    pub counts_by_issue_type: Vec<AttendanceReviewCount>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttendanceReviewRowsResponse {
    pub period_start: String,
    pub period_end: String,
    pub rows: Vec<AttendanceReviewRow>,
}

impl LocalServer {
    pub fn start_run(&self, request: LocalServerRunRequest) -> LocalServerRunResponse {
        let result = run_ingest_workflow(IngestRunCommand::new(
            request.run_id,
            request.employees_csv,
            request.attendance_csv,
        ));
        self.write_run_artifacts(&result)
            .expect("local server should write run artifacts");
        let db_persistence_status = self.persist_run_if_configured(&result);

        LocalServerRunResponse {
            run_id: result.run_id.clone(),
            job_state: result.job.current_state.as_str().to_string(),
            progress_percent: result.job.progress_percent,
            artifacts: artifact_list(result.run_id.as_str()),
            report_markdown_path: artifact_path(result.run_id.as_str(), "public_report.md"),
            db_persistence_status,
        }
    }

    pub fn start_run_from_csv_contents(
        &self,
        employees_csv: impl Into<String>,
        attendance_csv: impl Into<String>,
    ) -> LocalServerRunResponse {
        let run_id = RunId::new(generate_run_id());
        self.start_run(LocalServerRunRequest {
            run_id,
            employees_csv: CsvInput::new(
                DatasetKind::Employees,
                "upload://employees.csv",
                employees_csv,
            ),
            attendance_csv: CsvInput::new(
                DatasetKind::Attendance,
                "upload://attendance.csv",
                attendance_csv,
            ),
        })
    }

    fn write_run_artifacts(&self, result: &IngestWorkflowResult) -> io::Result<()> {
        let output_dir = self.artifact_root.join(result.run_id.as_str());
        fs::create_dir_all(&output_dir)?;
        write_json(output_dir.join("run_summary.json"), &result.run_summary)?;
        write_json(
            output_dir.join("artifact_manifest.json"),
            &artifact_list(result.run_id.as_str()),
        )?;
        fs::write(output_dir.join("issues.csv"), issues_csv(&result.issues))?;
        fs::write(
            output_dir.join("public_report.md"),
            public_report_markdown(result),
        )?;
        Ok(())
    }

    fn persist_run_if_configured(&self, result: &IngestWorkflowResult) -> String {
        let Some(database_url) = &self.run_database_url else {
            return "not_configured".to_string();
        };

        let mut adapter = match PostgresCommandAdapter::connect(database_url) {
            Ok(adapter) => adapter,
            Err(error) => return format!("connect_failed:{error}"),
        };
        match adapter.execute_transaction(&db_commands_for_result(result)) {
            Ok(_) => "saved".to_string(),
            Err(error) => format!("save_failed:{error}"),
        }
    }

    pub fn artifact_response(&self, run_id: &str, file_name: &str) -> ApiResponse {
        if !is_safe_artifact_name(file_name) {
            return json_response(404, &serde_json::json!({ "error": "not_found" }));
        }
        let path = self.artifact_root.join(run_id).join(file_name);
        match fs::read_to_string(&path) {
            Ok(body) => ApiResponse {
                status_code: 200,
                content_type: content_type_for(file_name).to_string(),
                body,
            },
            Err(_) => json_response(404, &serde_json::json!({ "error": "not_found" })),
        }
    }

    pub fn run_progress_response(&self, run_id: &str) -> ApiResponse {
        let summary_path = self.artifact_root.join(run_id).join("run_summary.json");
        let exists = summary_path.exists();
        json_response(
            if exists { 200 } else { 404 },
            &serde_json::json!({
                "run_id": run_id,
                "job_state": if exists { "succeeded" } else { "unknown" },
                "progress_percent": if exists { 100 } else { 0 }
            }),
        )
    }

    pub fn artifact_listing_response(&self, run_id: &str) -> ApiResponse {
        let run_dir = self.artifact_root.join(run_id);
        if !run_dir.exists() {
            return json_response(404, &serde_json::json!({ "error": "not_found" }));
        }
        json_response(200, &artifact_list(run_id))
    }

    pub fn use_case_catalog(&self) -> Vec<UseCaseDefinition> {
        use_case_definitions()
    }

    pub fn use_case_sample(&self, use_case_id: &str) -> Option<UseCaseSampleResponse> {
        let use_case_index = USE_CASE_IDS
            .iter()
            .position(|candidate| *candidate == use_case_id)?;
        let definition = use_case_definitions()
            .into_iter()
            .find(|definition| definition.use_case_id == use_case_id)?;
        let employees = self.demo_database.sample_employees(use_case_index);
        let rows = build_use_case_rows(use_case_id, &employees);

        Some(UseCaseSampleResponse {
            use_case: definition,
            source: DemoDbSource {
                table_name: "laborlens.demo_employees".to_string(),
                seed_version: "demo_japanese_employees.v1".to_string(),
                employee_count: self.demo_database.employee_count(),
                note: "1000人分の架空日本人従業員 seed から読み込み".to_string(),
            },
            metrics: build_metrics(use_case_id, self.demo_database.employee_count()),
            rows,
            findings: build_findings(use_case_id),
            next_actions: build_next_actions(use_case_id),
        })
    }

    pub fn run_history_response(&self) -> ApiResponse {
        let mut runs = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.artifact_root) {
            for entry in entries.flatten() {
                let Ok(file_type) = entry.file_type() else {
                    continue;
                };
                if !file_type.is_dir() {
                    continue;
                }
                let run_id = entry.file_name().to_string_lossy().to_string();
                runs.push(RunHistoryItem {
                    progress_path: format!("/api/runs/{run_id}/progress"),
                    artifacts_path: format!("/api/runs/{run_id}/artifacts"),
                    report_path: format!("/api/runs/{run_id}/reports/public-report"),
                    run_id,
                });
            }
        }
        runs.sort_by(|left, right| right.run_id.cmp(&left.run_id));
        json_response(200, &runs)
    }

    pub fn guide_message_response(&self, request: GuideMessageRequest) -> ApiResponse {
        let references = vec![
            "docs/product/BUSINESS-RULES.md".to_string(),
            "docs/product/ACCEPTANCE-CRITERIA.md".to_string(),
            "docs/product/OPERATIONS.md".to_string(),
        ];
        let answer = if request.message.trim().is_empty() {
            "質問内容が空です。確認したい issue、成果物、または run を指定してください。"
                .to_string()
        } else {
            format!(
                "このガイドは RuleExplanation の初期境界です。質問「{}」に対して、抑制前データや個人値は参照せず、公開済み成果物と承認済みルール文書だけを根拠に説明します。",
                request.message.trim()
            )
        };

        json_response(
            200,
            &GuideMessageResponse {
                answer,
                mode: "deterministic_rule_explanation".to_string(),
                safety_boundary: "no raw CSV, no personal fatigue values, no direct Ollama access"
                    .to_string(),
                references,
            },
        )
    }

    pub fn attendance_review_summary(&self) -> AttendanceReviewSummary {
        let rows = build_attendance_review_rows(self.demo_database.employees());
        AttendanceReviewSummary {
            period_start: attendance_review_period_start().to_string(),
            period_end: attendance_review_period_end().to_string(),
            employee_count: self.demo_database.employee_count(),
            row_count: rows.len(),
            issue_row_count: rows.iter().filter(|row| row.issue_type != "none").count(),
            counts_by_store: count_attendance_review_values(&rows, |row| &row.store_name),
            counts_by_department: count_attendance_review_values(&rows, |row| &row.department),
            counts_by_severity: count_attendance_review_values(&rows, |row| &row.severity),
            counts_by_status: count_attendance_review_values(&rows, |row| &row.status),
            counts_by_issue_type: count_attendance_review_values(&rows, |row| &row.issue_type),
        }
    }

    pub fn attendance_review_rows_response(&self) -> AttendanceReviewRowsResponse {
        AttendanceReviewRowsResponse {
            period_start: attendance_review_period_start().to_string(),
            period_end: attendance_review_period_end().to_string(),
            rows: build_attendance_review_rows(self.demo_database.employees()),
        }
    }

    pub fn api_get(&self, path: &str) -> ApiResponse {
        match path.trim_end_matches('/') {
            "/api/attendance-review/summary" => {
                json_response(200, &self.attendance_review_summary())
            }
            "/api/attendance-review/rows" => {
                json_response(200, &self.attendance_review_rows_response())
            }
            "/api/use-cases" => json_response(200, &self.use_case_catalog()),
            "/api/runs" => self.run_history_response(),
            endpoint if endpoint.starts_with("/api/runs/") => {
                let parts = endpoint
                    .trim_start_matches("/api/runs/")
                    .split('/')
                    .collect::<Vec<_>>();
                match parts.as_slice() {
                    [run_id, "progress"] => self.run_progress_response(run_id),
                    [run_id, "artifacts"] => self.artifact_listing_response(run_id),
                    [run_id, "artifacts", file_name] => self.artifact_response(run_id, file_name),
                    [run_id, "reports", "public-report"] => {
                        self.artifact_response(run_id, "public_report.md")
                    }
                    _ => json_response(404, &serde_json::json!({ "error": "not_found" })),
                }
            }
            endpoint if endpoint.starts_with("/api/use-cases/") => {
                let mut parts = endpoint.trim_start_matches("/api/use-cases/").split('/');
                match (parts.next(), parts.next(), parts.next()) {
                    (Some(use_case_id), Some("sample-data"), None) => {
                        match self.use_case_sample(use_case_id) {
                            Some(sample) => json_response(200, &sample),
                            None => json_response(
                                404,
                                &serde_json::json!({
                                    "error": "unknown_use_case",
                                    "use_case_id": use_case_id
                                }),
                            ),
                        }
                    }
                    _ => json_response(404, &serde_json::json!({ "error": "not_found" })),
                }
            }
            _ => json_response(404, &serde_json::json!({ "error": "not_found" })),
        }
    }
}

fn attendance_review_period_start() -> &'static str {
    "2026-01-01"
}

fn attendance_review_period_end() -> &'static str {
    "2026-01-31"
}

fn build_attendance_review_rows(employees: &[DemoEmployee]) -> Vec<AttendanceReviewRow> {
    employees
        .iter()
        .enumerate()
        .map(|(index, employee)| build_attendance_review_row(index, employee))
        .collect()
}

fn build_attendance_review_row(index: usize, employee: &DemoEmployee) -> AttendanceReviewRow {
    let day = 1 + (index % 28);
    let work_date = format!("2026-01-{day:02}");
    let scheduled_clock_in = "09:00".to_string();
    let scheduled_clock_out = "18:00".to_string();
    let (
        clock_in,
        clock_out,
        worked_minutes,
        issue_type,
        issue_label,
        severity,
        status,
        review_hint,
    ) = attendance_review_issue_profile(index, employee);

    AttendanceReviewRow {
        row_id: format!("att-review-{:04}", index + 1),
        employee_id: employee.employee_id.clone(),
        display_name: employee.display_name.clone(),
        store_name: employee.store_name.clone(),
        department: employee.department.clone(),
        work_date,
        clock_in,
        clock_out,
        scheduled_clock_in,
        scheduled_clock_out,
        worked_minutes,
        issue_type: issue_type.to_string(),
        issue_label: issue_label.to_string(),
        severity: severity.to_string(),
        status: status.to_string(),
        review_hint,
    }
}

fn attendance_review_issue_profile(
    index: usize,
    employee: &DemoEmployee,
) -> (
    Option<String>,
    Option<String>,
    Option<u16>,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    String,
) {
    match index {
        0..=899 => (
            Some("09:00".to_string()),
            Some("18:00".to_string()),
            Some(540),
            "none",
            "問題なし",
            "none",
            "ok",
            "通常勤務として自動確認できます。".to_string(),
        ),
        900..=924 => (
            None,
            Some("18:05".to_string()),
            None,
            "missing_clock_in",
            "出勤打刻漏れ",
            "high",
            "needs_review",
            "出勤打刻の原票または店舗修正依頼を確認してください。".to_string(),
        ),
        925..=949 => (
            Some("08:55".to_string()),
            None,
            None,
            "missing_clock_out",
            "退勤打刻漏れ",
            "high",
            "needs_review",
            "退勤打刻の原票または店舗修正依頼を確認してください。".to_string(),
        ),
        950..=964 => (
            Some("18:10".to_string()),
            Some("09:00".to_string()),
            None,
            "time_reversal",
            "時刻逆転",
            "critical",
            "blocked",
            "出勤時刻と退勤時刻の前後関係を修正するまで集計対象外です。".to_string(),
        ),
        965..=979 => (
            Some("09:00".to_string()),
            Some("18:00".to_string()),
            Some(540),
            "duplicate_candidate",
            "重複候補",
            "medium",
            "needs_review",
            duplicate_review_hint(employee),
        ),
        980..=994 => (
            Some("07:30".to_string()),
            Some("22:45".to_string()),
            Some(915),
            "long_hours_candidate",
            "長時間勤務候補",
            "medium",
            "needs_review",
            "会社の運用閾値に照らして、休憩・残業申請・勤務実態を確認してください。".to_string(),
        ),
        995..=997 => (
            Some("09:00".to_string()),
            Some("18:00".to_string()),
            Some(540),
            "master_department_mismatch_candidate",
            "従業員マスタ部署不一致候補",
            "medium",
            "needs_review",
            master_department_mismatch_hint(employee),
        ),
        _ => (
            Some("09:00".to_string()),
            Some("18:00".to_string()),
            Some(540),
            "master_inactive_employee_candidate",
            "在籍状態確認候補",
            "medium",
            "needs_review",
            "勤怠に行がありますが、マスタ上の在籍状態確認が必要な候補として扱います。".to_string(),
        ),
    }
}

fn duplicate_review_hint(employee: &DemoEmployee) -> String {
    format!(
        "{} / {} の同日同時刻に近い勤怠行がある候補です。",
        employee.store_name, employee.department
    )
}

fn master_department_mismatch_hint(employee: &DemoEmployee) -> String {
    format!(
        "勤怠側部署が {} と異なる可能性があります。従業員マスタを確認してください。",
        employee.department
    )
}

fn count_attendance_review_values(
    rows: &[AttendanceReviewRow],
    value: impl Fn(&AttendanceReviewRow) -> &str,
) -> Vec<AttendanceReviewCount> {
    let mut counts = BTreeMap::new();
    for row in rows {
        *counts.entry(value(row).to_string()).or_insert(0) += 1;
    }
    counts
        .into_iter()
        .map(|(key, count)| AttendanceReviewCount { key, count })
        .collect()
}

fn db_commands_for_result(result: &IngestWorkflowResult) -> Vec<SqlCommand> {
    let readiness_status = if result.run_summary.succeeded {
        "ready"
    } else {
        "blocked"
    };
    let run_status = if result.run_summary.succeeded {
        "completed"
    } else {
        "failed"
    };
    let mut commands = vec![InsertRunRecord::new(
        result.run_id.as_str(),
        "tenant_local",
        run_status,
        readiness_status,
        "local_db.v1",
    )
    .with_settings_json(r#"{"source":"local_server"}"#)
    .to_sql_command()];

    for input_ref in &result.input_refs {
        commands.push(
            InsertInputRef::new(
                db_input_ref_id(&input_ref.dataset_id),
                result.run_id.as_str(),
                input_ref.dataset_id.as_str(),
                source_file_name(&input_ref.source_ref),
                input_ref.source_ref.as_str(),
                fingerprint_to_db_hash(&input_ref.fingerprint),
                0,
                "utf-8",
                ",",
                true,
                input_ref.schema_version.as_str(),
            )
            .with_detected_shape(input_ref.record_count as i64, 0)
            .with_metadata_json(format!(
                r#"{{"fingerprint":"{}"}}"#,
                json_escape(&input_ref.fingerprint)
            ))
            .to_sql_command(),
        );
    }

    let db_job_state = if result.run_summary.succeeded {
        DbJobState::Succeeded
    } else {
        DbJobState::Failed
    };
    commands.push(
        InsertJob::queued(
            db_job_id(result.run_id.as_str()),
            result.run_id.as_str(),
            "ingest",
        )
        .with_payload_json(r#"{"source":"local_server"}"#)
        .to_sql_command(),
    );
    commands.push(
        UpdateJobState::new(
            db_job_id(result.run_id.as_str()),
            result.run_id.as_str(),
            db_job_state,
        )
        .with_progress(result.job.progress_percent.into())
        .with_stage(result.job.current_state.as_str())
        .to_sql_command(),
    );

    for (index, issue) in result.issues.iter().enumerate() {
        commands.push(
            InsertIssue::new(
                format!(
                    "iss_{}_{}",
                    safe_db_suffix(result.run_id.as_str()),
                    index + 1
                ),
                result.run_id.as_str(),
                "schema_issue",
                format!("{:?}", issue.issue_kind).to_ascii_uppercase(),
                "high",
                "blocked",
                issue.message.as_str(),
            )
            .with_input_ref(db_input_ref_id(&issue.dataset_id))
            .with_dataset_kind(issue.dataset_id.as_str())
            .with_status("open")
            .with_privacy_status("public")
            .to_sql_command(),
        );
    }

    commands
}

fn db_input_ref_id(dataset_id: &str) -> String {
    format!("src_{}", safe_db_suffix(dataset_id))
}

fn db_job_id(run_id: &str) -> String {
    format!("job_{}", safe_db_suffix(run_id))
}

fn safe_db_suffix(value: &str) -> String {
    value
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect()
}

fn source_file_name(source_ref: &str) -> &str {
    source_ref
        .rsplit(['/', '\\'])
        .next()
        .filter(|value| !value.is_empty())
        .unwrap_or(source_ref)
}

fn fingerprint_to_db_hash(fingerprint: &str) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in fingerprint.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}{hash:016x}{hash:016x}{hash:016x}")
}

fn json_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn artifact_list(run_id: &str) -> Vec<ArtifactListing> {
    vec![
        ArtifactListing::run_summary(artifact_path(run_id, "run_summary.json")),
        ArtifactListing {
            artifact_name: "issues".to_string(),
            stable_path: artifact_path(run_id, "issues.csv"),
            content_type: "text/csv".to_string(),
        },
        ArtifactListing {
            artifact_name: "artifact_manifest".to_string(),
            stable_path: artifact_path(run_id, "artifact_manifest.json"),
            content_type: "application/json".to_string(),
        },
        ArtifactListing {
            artifact_name: "public_report".to_string(),
            stable_path: artifact_path(run_id, "public_report.md"),
            content_type: "text/markdown".to_string(),
        },
    ]
}

fn artifact_path(run_id: &str, file_name: &str) -> String {
    format!("/api/runs/{run_id}/artifacts/{file_name}")
}

fn json_response<T: Serialize>(status_code: u16, value: &T) -> ApiResponse {
    ApiResponse {
        status_code,
        content_type: "application/json; charset=utf-8".to_string(),
        body: serde_json::to_string(value).expect("API response should serialize"),
    }
}

fn write_json(path: impl AsRef<Path>, value: &impl Serialize) -> io::Result<()> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    fs::write(path, json)
}

fn issues_csv(issues: &[SchemaIssue]) -> String {
    let mut output = String::from("issue_id,dataset_id,source_ref,issue_kind,message\n");
    for issue in issues {
        output.push_str(&format!(
            "{},{},{},{:?},{}\n",
            csv_escape(&issue.issue_id),
            csv_escape(&issue.dataset_id),
            csv_escape(&issue.source_ref),
            issue.issue_kind,
            csv_escape(&issue.message)
        ));
    }
    output
}

fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn public_report_markdown(result: &IngestWorkflowResult) -> String {
    format!(
        "# LaborLens Run Report\n\n- RunId: `{}`\n- Job: `{}`\n- Employees rows: {}\n- Attendance rows: {}\n- Issues: {}\n- Succeeded: {}\n\n## Next\n\n{}\n",
        result.run_id.as_str(),
        result.job.current_state.as_str(),
        result.row_counts.employee_rows,
        result.row_counts.attendance_rows,
        result.issues.len(),
        result.run_summary.succeeded,
        if result.issues.is_empty() {
            "CSV 取込は完了しました。成果物と入力 hash を確認してください。"
        } else {
            "issues.csv を確認し、修正後 CSV で再確認してください。"
        }
    )
}

fn content_type_for(file_name: &str) -> &'static str {
    match file_name {
        "issues.csv" => "text/csv; charset=utf-8",
        "public_report.md" => "text/markdown; charset=utf-8",
        _ if file_name.ends_with(".json") => "application/json; charset=utf-8",
        _ => "text/plain; charset=utf-8",
    }
}

fn is_safe_artifact_name(file_name: &str) -> bool {
    matches!(
        file_name,
        "run_summary.json" | "issues.csv" | "artifact_manifest.json" | "public_report.md"
    )
}

fn generate_run_id() -> String {
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    let counter = RUN_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("run_{millis}_{counter}")
}

fn build_demo_employee(number: usize) -> DemoEmployee {
    const FAMILY_NAMES: [&str; 20] = [
        "佐藤",
        "鈴木",
        "高橋",
        "田中",
        "伊藤",
        "渡辺",
        "山本",
        "中村",
        "小林",
        "加藤",
        "吉田",
        "山田",
        "佐々木",
        "山口",
        "松本",
        "井上",
        "木村",
        "林",
        "清水",
        "斎藤",
    ];
    const GIVEN_NAMES: [&str; 20] = [
        "陽菜", "結衣", "葵", "凛", "美咲", "翔太", "蓮", "悠真", "大和", "湊", "直樹", "拓也",
        "真央", "彩", "優子", "健太", "誠", "舞", "亮", "恵",
    ];
    const DEPARTMENTS: [&str; 10] = [
        "営業部",
        "販売部",
        "人事部",
        "経理部",
        "物流部",
        "製造部",
        "情報システム部",
        "商品管理部",
        "カスタマー支援部",
        "企画部",
    ];
    const STORES: [&str; 12] = [
        "東京東店",
        "東京西店",
        "横浜店",
        "千葉店",
        "さいたま店",
        "名古屋店",
        "大阪北店",
        "大阪南店",
        "京都店",
        "神戸店",
        "福岡店",
        "札幌店",
    ];
    const EMPLOYMENT_TYPES: [&str; 4] = ["正社員", "契約社員", "パート", "アルバイト"];
    const ROLES: [&str; 5] = ["スタッフ", "主任", "店長", "事務担当", "部門責任者"];

    let index = number - 1;
    DemoEmployee {
        employee_id: format!("EMP-{number:04}"),
        display_name: format!(
            "{} {}",
            FAMILY_NAMES[index % FAMILY_NAMES.len()],
            GIVEN_NAMES[(index / FAMILY_NAMES.len()) % GIVEN_NAMES.len()]
        ),
        department: DEPARTMENTS[index % DEPARTMENTS.len()].to_string(),
        store_name: STORES[index % STORES.len()].to_string(),
        employment_type: EMPLOYMENT_TYPES[index % EMPLOYMENT_TYPES.len()].to_string(),
        role_name: ROLES[index % ROLES.len()].to_string(),
        hired_on: format!("20{:02}-{:02}-01", 14 + (index % 11), 1 + (index % 12)),
    }
}

fn use_case_definitions() -> Vec<UseCaseDefinition> {
    vec![
        use_case(
            "uc-01",
            "勤怠不備",
            "給与計算前に勤怠データの不備を確認したい",
            "事務員・労務担当者",
            "打刻漏れ、二重打刻、時刻逆転を給与計算前に整理する。",
        ),
        use_case(
            "uc-02",
            "店長負荷",
            "店長の労働時間が増えすぎている原因を整理したい",
            "店長・エリアマネージャー",
            "欠員対応や繁忙時間帯により負荷が集中していないか確認する。",
        ),
        use_case(
            "uc-03",
            "人件費配分",
            "部署ごとの人件費配分を確認したい",
            "経理担当者",
            "部署別、雇用区分別の人件費割合と結合可否を確認する。",
        ),
        use_case(
            "uc-04",
            "集団分析",
            "ストレスチェックの集団分析を安全に労務改善へつなげたい",
            "人事担当者",
            "個人値を出さず、部署単位の負荷傾向と少人数抑制を確認する。",
        ),
        use_case(
            "uc-05",
            "店舗差戻し",
            "本部が店舗から集めた CSV の不備を一覧で返したい",
            "本部管理担当者",
            "店舗別の不備件数と修正依頼チェックリストを作る。",
        ),
        use_case(
            "uc-06",
            "修正対象",
            "Excel で直す前にどこを直せばよいかだけ知りたい",
            "事務員・店舗担当者",
            "原本を変えず、修正対象の行、列、理由だけを表示する。",
        ),
        use_case(
            "uc-07",
            "整備状況",
            "経営者が分析を始める前にデータ整備の状況を知りたい",
            "経営者・事業責任者",
            "データセット別の ready / blocked と次の整備対象を要約する。",
        ),
        use_case(
            "uc-08",
            "仕様変更",
            "システム担当が CSV 仕様変更の影響を確認したい",
            "情報システム担当者",
            "旧形式と新形式のヘッダー差分、足りない項目、移行メモを確認する。",
        ),
        use_case(
            "uc-09",
            "マスタ照合",
            "入社・退職・異動後の従業員マスタ不一致を見つけたい",
            "人事・労務担当者",
            "従業員 ID、所属部署、在籍状態の不一致を優先表示する。",
        ),
        use_case(
            "uc-10",
            "労働時間",
            "長時間労働や残業上限の確認材料を作りたい",
            "労務担当者・店長",
            "法的判断ではなく、早めに確認すべき労働時間リスクを整理する。",
        ),
        use_case(
            "uc-11",
            "有給取得",
            "有給休暇の取得状況を確認したい",
            "人事・労務担当者",
            "有給取得の偏りと取得促進対象を部署別に確認する。",
        ),
        use_case(
            "uc-12",
            "人員不足",
            "採用や応援要請の判断材料を作りたい",
            "店長・エリアマネージャー",
            "曜日・時間帯別の不足傾向と採用、応援、配置見直しの材料を示す。",
        ),
        use_case(
            "uc-13",
            "月次レポート",
            "毎月の労務レポートを自動で作りたい",
            "本部管理担当者",
            "勤怠、人件費、CSV 不備を毎月同じ形式で横並びにする。",
        ),
        use_case(
            "uc-14",
            "外部共有前",
            "データを外部へ渡す前に個人情報が含まれていないか確認したい",
            "管理部門・システム担当者",
            "識別情報らしき列、推測リスク、マスキング対象を確認する。",
        ),
    ]
}

fn use_case(
    use_case_id: &str,
    button_label: &str,
    title: &str,
    actor: &str,
    summary: &str,
) -> UseCaseDefinition {
    UseCaseDefinition {
        use_case_id: use_case_id.to_string(),
        button_label: button_label.to_string(),
        title: title.to_string(),
        actor: actor.to_string(),
        summary: summary.to_string(),
    }
}

fn build_metrics(use_case_id: &str, employee_count: usize) -> Vec<MetricCard> {
    let scenario_number = use_case_number(use_case_id);
    vec![
        metric("seed 従業員数", employee_count.to_string(), "人", "ready"),
        metric(
            "対象データセット",
            dataset_label(use_case_id).to_string(),
            "",
            "ready",
        ),
        metric(
            "確認対象",
            (12 + scenario_number * 3).to_string(),
            "件",
            if scenario_number % 4 == 0 {
                "suppressed"
            } else {
                "attention"
            },
        ),
    ]
}

fn build_use_case_rows(use_case_id: &str, employees: &[&DemoEmployee]) -> Vec<UseCaseSampleRow> {
    let labels = row_labels(use_case_id);
    employees
        .iter()
        .enumerate()
        .map(|(index, employee)| UseCaseSampleRow {
            subject: if use_case_id == "uc-04" || use_case_id == "uc-07" {
                employee.department.clone()
            } else {
                format!("{} {}", employee.employee_id, employee.display_name)
            },
            group: format!("{} / {}", employee.store_name, employee.department),
            primary_value: labels[index % labels.len()].to_string(),
            status: row_status(use_case_id, index).to_string(),
            note: row_note(use_case_id, employee, index),
        })
        .collect()
}

fn build_findings(use_case_id: &str) -> Vec<String> {
    match use_case_id {
        "uc-01" => vec![
            "給与計算へ進める前に確認すべき勤怠 issue が残っています。",
            "未登録従業員と時刻逆転は優先度高として扱います。",
        ],
        "uc-02" => vec![
            "店長ロールの欠員対応が週末に集中しています。",
            "忙しい時間帯と勤務人数の不足が同じ期間に重なっています。",
        ],
        "uc-03" => vec![
            "人件費データは部署別月次と従業員別月次が混在しています。",
            "部署別配分は可能ですが、個人勤怠との直接結合には追加確認が必要です。",
        ],
        "uc-04" => vec![
            "少人数部署は集計結果を抑制しています。",
            "個人の疲労値やコメントは UI に出していません。",
        ],
        "uc-05" => vec![
            "店舗ごとに列名と日付形式の不一致があります。",
            "修正依頼は店舗単位で返せる状態です。",
        ],
        "uc-06" => vec![
            "原本 hash は変わっていません。",
            "修正対象の行、列、理由だけを抽出しています。",
        ],
        "uc-07" => vec![
            "勤怠と従業員マスタは ready、人件費と売上は partial です。",
            "分析開始前に追加で整えるべきデータが見えています。",
        ],
        "uc-08" => vec![
            "旧 CSV と新 CSV のヘッダー差分があります。",
            "移行期間中は fixture による再現確認が必要です。",
        ],
        "uc-09" => vec![
            "退職済み従業員と部署不一致が見つかっています。",
            "給与計算前にマスタ修正の確認が必要です。",
        ],
        "uc-10" => vec![
            "長時間労働の確認候補があります。",
            "適法・違法判断ではなく、労務担当者の確認材料として出力しています。",
        ],
        "uc-11" => vec![
            "有給取得が少ない部署が一部あります。",
            "個人を責める表示ではなく、取得しづらさの傾向として扱います。",
        ],
        "uc-12" => vec![
            "人員不足は特定曜日に偏っています。",
            "採用、応援、配置見直しの候補を分けて表示します。",
        ],
        "uc-13" => vec![
            "月次の勤怠、人件費、不備件数を同じ形式で表示できます。",
            "前月比較により改善と残課題を分けています。",
        ],
        "uc-14" => vec![
            "外部共有前に確認すべき識別情報らしき列があります。",
            "少人数部署や特殊勤務パターンは推測リスクとして扱います。",
        ],
        _ => vec![],
    }
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn build_next_actions(use_case_id: &str) -> Vec<String> {
    match use_case_id {
        "uc-01" => vec![
            "本人確認が必要な勤怠を店舗へ返す",
            "修正後 CSV で再実行する",
        ],
        "uc-02" => vec![
            "週末シフトの応援候補を確認する",
            "店長代替勤務の発生理由を記録する",
        ],
        "uc-03" => vec![
            "部署別月次データと従業員別月次データを分けて扱う",
            "個人勤怠と結合できない行を経理へ確認する",
        ],
        "uc-04" => vec![
            "少人数部署の集計を表示しない",
            "セルフケア案内文を人事担当者が確認する",
        ],
        "uc-05" => vec![
            "店舗別チェックリストを出力する",
            "前回提出分との差分を比較する",
        ],
        "uc-06" => vec![
            "Excel で対象セルだけ修正する",
            "raw input hash を再確認する",
        ],
        "uc-07" => vec![
            "blocked データセットの整備順を決める",
            "導入ステップをロードマップへ反映する",
        ],
        "uc-08" => vec![
            "新旧ヘッダー対応表を更新する",
            "移行 fixture で run を再実行する",
        ],
        "uc-09" => vec![
            "従業員マスタを人事へ確認する",
            "退職済み従業員の勤怠残存を調査する",
        ],
        "uc-10" => vec!["確認候補を労務担当者へ渡す", "会社の運用閾値を設定する"],
        "uc-11" => vec![
            "取得促進対象の部署を確認する",
            "管理者向け声かけ文面を確認する",
        ],
        "uc-12" => vec![
            "一時的欠員と慢性的不足を分ける",
            "応援要請または採用の検討材料にする",
        ],
        "uc-13" => vec![
            "月次レポートを成果物として保存する",
            "改善した点と残課題を分けて共有する",
        ],
        "uc-14" => vec![
            "不要な識別列をマスキングする",
            "外部共有可否を会社ルールで確認する",
        ],
        _ => vec![],
    }
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn metric(label: &str, value: String, unit: &str, status: &str) -> MetricCard {
    MetricCard {
        label: label.to_string(),
        value,
        unit: unit.to_string(),
        status: status.to_string(),
    }
}

fn dataset_label(use_case_id: &str) -> &'static str {
    match use_case_id {
        "uc-01" | "uc-02" | "uc-06" | "uc-10" => "勤怠",
        "uc-03" => "人件費",
        "uc-04" => "集団分析",
        "uc-05" => "店舗 CSV",
        "uc-07" => "準備状況",
        "uc-08" => "CSV 仕様",
        "uc-09" => "従業員マスタ",
        "uc-11" => "有給休暇",
        "uc-12" => "人員配置",
        "uc-13" => "月次レポート",
        "uc-14" => "外部共有前",
        _ => "デモ",
    }
}

fn row_labels(use_case_id: &str) -> &'static [&'static str] {
    match use_case_id {
        "uc-01" => &["打刻漏れ", "時刻逆転", "未登録従業員"],
        "uc-02" => &["欠員対応 18h", "週末連続勤務", "繁忙帯不足"],
        "uc-03" => &["部署別月次", "従業員別月次", "結合不可"],
        "uc-04" => &["集計抑制", "部署傾向のみ", "REDACTED"],
        "uc-05" => &["必須列不足", "日付形式エラー", "ID 表記揺れ"],
        "uc-06" => &["修正対象セル", "原本 hash 維持", "再確認待ち"],
        "uc-07" => &["ready", "partial", "blocked"],
        "uc-08" => &["旧ヘッダー", "新ヘッダー", "不足項目"],
        "uc-09" => &["部署不一致", "退職済み", "未登録"],
        "uc-10" => &["残業確認", "連続勤務", "休日取得確認"],
        "uc-11" => &["取得率低", "部署偏り", "声かけ候補"],
        "uc-12" => &["慢性不足", "一時欠員", "応援候補"],
        "uc-13" => &["前月改善", "残課題", "横並び比較"],
        "uc-14" => &["識別列候補", "推測リスク", "マスキング候補"],
        _ => &["確認対象"],
    }
}

fn row_status(use_case_id: &str, index: usize) -> &'static str {
    if use_case_id == "uc-04" && index == 0 {
        "suppressed"
    } else if index == 2 {
        "blocked"
    } else if index == 1 {
        "attention"
    } else {
        "ready"
    }
}

fn row_note(use_case_id: &str, employee: &DemoEmployee, index: usize) -> String {
    match use_case_id {
        "uc-04" => format!(
            "{} は個人値を表示せず、部署単位で扱います。",
            employee.department
        ),
        "uc-07" => format!(
            "{} のデータ整備状態を経営層向けに要約します。",
            employee.department
        ),
        "uc-14" => format!(
            "{} の外部共有前チェックで識別情報候補を確認します。",
            employee.store_name
        ),
        _ => format!(
            "{} の {} データからサンプル {} を読み込みました。",
            employee.store_name,
            employee.department,
            index + 1
        ),
    }
}

fn use_case_number(use_case_id: &str) -> usize {
    use_case_id
        .trim_start_matches("uc-")
        .parse::<usize>()
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use crate::{
        db_commands_for_result, ArtifactListing, DemoDataStore, GuideMessageRequest, LocalServer,
        LocalServerRunRequest, USE_CASE_IDS,
    };
    use laborlens_rust::contexts::ingest::application::run_ingest_workflow;
    use laborlens_rust::contexts::ingest::domain::DatasetKind;
    use laborlens_rust::contexts::ingest::infrastructure::load_csv_input_from_path;
    use laborlens_rust::contexts::ingest::interfaces::{CsvInput, IngestRunCommand};
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
        assert_eq!(response.db_persistence_status, "not_configured");
        assert!(response.artifacts.iter().any(|artifact| artifact
            == &ArtifactListing::run_summary(
                "/api/runs/run-local-server-001/artifacts/run_summary.json"
            )));
        let report = server.artifact_response("run-local-server-001", "public_report.md");
        assert_eq!(report.status_code, 200);
        assert!(report.body.contains("LaborLens Run Report"));
    }

    #[test]
    fn seeded_demo_database_contains_one_thousand_japanese_dummy_employees() {
        let database = DemoDataStore::seeded();

        assert_eq!(database.employee_count(), 1_000);
        assert_eq!(database.employees()[0].employee_id, "EMP-0001");
        assert_eq!(database.employees()[999].employee_id, "EMP-1000");
        assert!(database
            .employees()
            .iter()
            .all(|employee| employee.display_name.chars().any(|ch| !ch.is_ascii())));
    }

    #[test]
    fn use_case_catalog_exposes_all_documented_buttons() {
        let server = LocalServer::default();
        let catalog = server.use_case_catalog();

        assert_eq!(USE_CASE_IDS.len(), 14);
        assert_eq!(catalog.len(), USE_CASE_IDS.len());
        for use_case_id in USE_CASE_IDS {
            let definition = catalog
                .iter()
                .find(|definition| definition.use_case_id == use_case_id)
                .expect("documented use case should have a UI definition");
            assert!(!definition.button_label.is_empty());
            assert!(!definition.title.is_empty());
        }
    }

    #[test]
    fn every_use_case_button_loads_sample_data_from_seeded_database() {
        let server = LocalServer::default();

        for use_case_id in USE_CASE_IDS {
            let sample = server
                .use_case_sample(use_case_id)
                .expect("documented use case should load a sample");
            assert_eq!(sample.source.employee_count, 1_000);
            assert_eq!(sample.source.table_name, "laborlens.demo_employees");
            assert_eq!(sample.use_case.use_case_id, use_case_id);
            assert!(!sample.rows.is_empty());
            assert!(!sample.findings.is_empty());
            assert!(!sample.next_actions.is_empty());
        }
    }

    #[test]
    fn attendance_review_summary_counts_seeded_monthly_issue_mix() {
        let server = LocalServer::default();
        let summary = server.attendance_review_summary();

        assert_eq!(summary.period_start, "2026-01-01");
        assert_eq!(summary.period_end, "2026-01-31");
        assert_eq!(summary.employee_count, 1_000);
        assert_eq!(summary.row_count, 1_000);
        assert_eq!(summary.issue_row_count, 100);
        assert_eq!(count_for(&summary.counts_by_issue_type, "none"), Some(900));
        assert_eq!(
            count_for(&summary.counts_by_issue_type, "missing_clock_in"),
            Some(25)
        );
        assert_eq!(
            count_for(&summary.counts_by_issue_type, "missing_clock_out"),
            Some(25)
        );
        assert_eq!(
            count_for(&summary.counts_by_issue_type, "time_reversal"),
            Some(15)
        );
        assert_eq!(
            count_for(&summary.counts_by_issue_type, "duplicate_candidate"),
            Some(15)
        );
        assert_eq!(
            count_for(&summary.counts_by_issue_type, "long_hours_candidate"),
            Some(15)
        );
        assert_eq!(
            count_for(
                &summary.counts_by_issue_type,
                "master_department_mismatch_candidate"
            ),
            Some(3)
        );
        assert_eq!(
            count_for(
                &summary.counts_by_issue_type,
                "master_inactive_employee_candidate"
            ),
            Some(2)
        );
        assert_eq!(count_for(&summary.counts_by_status, "ok"), Some(900));
        assert_eq!(
            count_for(&summary.counts_by_status, "needs_review"),
            Some(85)
        );
        assert_eq!(count_for(&summary.counts_by_status, "blocked"), Some(15));
    }

    #[test]
    fn attendance_review_api_routes_return_summary_and_rows() {
        let server = LocalServer::default();

        let summary = server.api_get("/api/attendance-review/summary");
        assert_eq!(summary.status_code, 200);
        assert!(summary.body.contains("\"row_count\":1000"));
        assert!(summary.body.contains("missing_clock_in"));

        let rows = server.api_get("/api/attendance-review/rows");
        assert_eq!(rows.status_code, 200);
        assert!(rows.body.contains("\"rows\""));
        assert!(rows.body.contains("\"row_id\":\"att-review-0001\""));
        assert!(rows.body.contains("master_inactive_employee_candidate"));
    }

    #[test]
    fn unknown_use_case_is_not_loaded() {
        let server = LocalServer::default();

        assert!(server.use_case_sample("uc-999").is_none());
    }

    #[test]
    fn run_api_artifact_routes_return_progress_listing_and_report() {
        let server = LocalServer::default();
        let response = server.start_run_from_csv_contents(
            "社員ID,氏名,部署\nEMP-0001,佐藤 陽菜,営業部\n",
            "社員ID,勤務日,出勤時刻,退勤時刻\nEMP-0001,2026-01-01,09:00,18:00\n",
        );

        let progress = server.run_progress_response(response.run_id.as_str());
        assert_eq!(progress.status_code, 200);
        assert!(progress.body.contains("\"progress_percent\":100"));

        let listing = server.artifact_listing_response(response.run_id.as_str());
        assert_eq!(listing.status_code, 200);
        assert!(listing.body.contains("public_report.md"));

        let report = server.artifact_response(response.run_id.as_str(), "public_report.md");
        assert_eq!(report.status_code, 200);
        assert!(report.body.contains("Employees rows: 1"));

        let history = server.run_history_response();
        assert_eq!(history.status_code, 200);
        assert!(history.body.contains(response.run_id.as_str()));
    }

    #[test]
    fn db_command_projection_uses_postgresql_safe_ids_and_hashes() {
        let server = LocalServer::default();
        let response = server.start_run_from_csv_contents(
            "社員ID,氏名,部署\nEMP-0001,佐藤 陽菜,営業部\n",
            "社員ID,勤務日,出勤時刻,退勤時刻\nEMP-0001,2026-01-01,09:00,18:00\n",
        );
        let report = server.artifact_response(response.run_id.as_str(), "run_summary.json");
        assert_eq!(report.status_code, 200);

        let result = run_ingest_workflow(IngestRunCommand::new(
            RunId::new("run_projection_001"),
            CsvInput::new(
                DatasetKind::Employees,
                "upload://employees.csv",
                "社員ID,氏名,部署\nEMP-0001,佐藤 陽菜,営業部\n",
            ),
            CsvInput::new(
                DatasetKind::Attendance,
                "upload://attendance.csv",
                "社員ID,勤務日,出勤時刻,退勤時刻\nEMP-0001,2026-01-01,09:00,18:00\n",
            ),
        ));
        let commands = db_commands_for_result(&result);
        let statements = commands
            .iter()
            .map(|command| command.statement())
            .collect::<Vec<_>>();
        assert!(statements
            .iter()
            .any(|statement| statement.contains("laborlens.run_records")));
        assert!(statements
            .iter()
            .any(|statement| statement.contains("laborlens.input_refs")));
        assert!(statements
            .iter()
            .any(|statement| statement.contains("laborlens.jobs")));
    }

    #[test]
    fn guide_message_endpoint_returns_rule_explanation_boundary() {
        let server = LocalServer::default();
        let response = server.guide_message_response(GuideMessageRequest {
            message: "この issue はなぜ出ましたか".to_string(),
            run_id: Some("run_demo".to_string()),
        });

        assert_eq!(response.status_code, 200);
        assert!(response.body.contains("deterministic_rule_explanation"));
        assert!(response.body.contains("no raw CSV"));
    }

    fn count_for(counts: &[crate::AttendanceReviewCount], key: &str) -> Option<usize> {
        counts
            .iter()
            .find(|count| count.key == key)
            .map(|count| count.count)
    }
}
