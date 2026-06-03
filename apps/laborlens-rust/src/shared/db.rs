//! PostgreSQL command model.
//!
//! This module does not open database connections. It keeps SQL statements and
//! bind parameter order explicit so repository adapters can be added after the
//! command shape is stable.

use std::fmt;

const INSERT_RUN_RECORD_SQL: &str = "INSERT INTO laborlens.run_records \
    (run_id, tenant_id, run_status, readiness_status, schema_version, settings) \
    VALUES ($1, $2, $3, $4, $5, $6)";

const INSERT_INPUT_REF_SQL: &str = "INSERT INTO laborlens.input_refs \
    (input_ref_id, run_id, dataset_kind, original_filename, source_uri, \
     file_hash_sha256, size_bytes, encoding, delimiter, has_header, \
     schema_version, detected_row_count, detected_column_count, metadata) \
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)";

const INSERT_JOB_SQL: &str = "INSERT INTO laborlens.jobs \
    (job_id, run_id, job_type, state, priority, max_attempts, payload) \
    VALUES ($1, $2, $3, $4, $5, $6, $7)";

const UPDATE_JOB_STATE_SQL: &str = "UPDATE laborlens.jobs \
    SET state = $1, \
        progress_percent = COALESCE($2, progress_percent), \
        stage = COALESCE($3, stage), \
        failure_kind = $4, \
        failure_message_masked = $5, \
        updated_at = now(), \
        completed_at = CASE \
            WHEN $1 IN ('succeeded', 'failed', 'canceled') THEN now() \
            ELSE completed_at \
        END \
    WHERE job_id = $6 AND run_id = $7";

const INSERT_ISSUE_SQL: &str = "INSERT INTO laborlens.issues \
    (issue_id, run_id, input_ref_id, normalized_ref_id, output_ref_id, \
     dataset_kind, source_row_number, column_name, raw_column_name, \
     issue_category, issue_code, severity, readiness_effect, message, \
     evidence_ref, evidence_value_masked, status, privacy_status, metadata) \
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, \
            $15, $16, $17, $18, $19)";

const INSERT_RUN_ARTIFACT_SQL: &str = "INSERT INTO laborlens.run_artifacts \
    (run_artifact_id, run_id, tenant_id, input_ref_id, normalized_ref_id, \
     policy_ref_id, output_ref_id, audit_ref_id) \
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8)";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlCommand {
    statement: &'static str,
    params: Vec<SqlParam>,
}

impl SqlCommand {
    pub fn new(statement: &'static str, params: Vec<SqlParam>) -> Self {
        Self { statement, params }
    }

    pub fn statement(&self) -> &str {
        self.statement
    }

    pub fn params(&self) -> &[SqlParam] {
        &self.params
    }

    pub fn bind_names(&self) -> Vec<&'static str> {
        self.params.iter().map(SqlParam::name).collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlParam {
    name: &'static str,
    value: Option<String>,
}

impl SqlParam {
    pub fn required(name: &'static str, value: impl Into<String>) -> Self {
        Self {
            name,
            value: Some(value.into()),
        }
    }

    pub fn nullable(name: &'static str, value: Option<impl Into<String>>) -> Self {
        Self {
            name,
            value: value.map(Into::into),
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn value(&self) -> Option<&str> {
        self.value.as_deref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InsertRunRecord {
    run_id: String,
    tenant_id: String,
    run_status: String,
    readiness_status: String,
    schema_version: String,
    settings_json: String,
}

impl InsertRunRecord {
    pub fn new(
        run_id: impl Into<String>,
        tenant_id: impl Into<String>,
        run_status: impl Into<String>,
        readiness_status: impl Into<String>,
        schema_version: impl Into<String>,
    ) -> Self {
        Self {
            run_id: run_id.into(),
            tenant_id: tenant_id.into(),
            run_status: run_status.into(),
            readiness_status: readiness_status.into(),
            schema_version: schema_version.into(),
            settings_json: "{}".to_string(),
        }
    }

    pub fn with_settings_json(mut self, settings_json: impl Into<String>) -> Self {
        self.settings_json = settings_json.into();
        self
    }

    pub fn to_sql_command(&self) -> SqlCommand {
        SqlCommand::new(
            INSERT_RUN_RECORD_SQL,
            vec![
                SqlParam::required("run_id", self.run_id.clone()),
                SqlParam::required("tenant_id", self.tenant_id.clone()),
                SqlParam::required("run_status", self.run_status.clone()),
                SqlParam::required("readiness_status", self.readiness_status.clone()),
                SqlParam::required("schema_version", self.schema_version.clone()),
                SqlParam::required("settings", self.settings_json.clone()),
            ],
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InsertInputRef {
    input_ref_id: String,
    run_id: String,
    dataset_kind: String,
    original_filename: String,
    source_uri: String,
    file_hash_sha256: String,
    size_bytes: i64,
    encoding: String,
    delimiter: String,
    has_header: bool,
    schema_version: String,
    detected_row_count: Option<i64>,
    detected_column_count: Option<i32>,
    metadata_json: String,
}

impl InsertInputRef {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        input_ref_id: impl Into<String>,
        run_id: impl Into<String>,
        dataset_kind: impl Into<String>,
        original_filename: impl Into<String>,
        source_uri: impl Into<String>,
        file_hash_sha256: impl Into<String>,
        size_bytes: i64,
        encoding: impl Into<String>,
        delimiter: impl Into<String>,
        has_header: bool,
        schema_version: impl Into<String>,
    ) -> Self {
        Self {
            input_ref_id: input_ref_id.into(),
            run_id: run_id.into(),
            dataset_kind: dataset_kind.into(),
            original_filename: original_filename.into(),
            source_uri: source_uri.into(),
            file_hash_sha256: file_hash_sha256.into(),
            size_bytes,
            encoding: encoding.into(),
            delimiter: delimiter.into(),
            has_header,
            schema_version: schema_version.into(),
            detected_row_count: None,
            detected_column_count: None,
            metadata_json: "{}".to_string(),
        }
    }

    pub fn with_detected_shape(mut self, row_count: i64, column_count: i32) -> Self {
        self.detected_row_count = Some(row_count);
        self.detected_column_count = Some(column_count);
        self
    }

    pub fn with_metadata_json(mut self, metadata_json: impl Into<String>) -> Self {
        self.metadata_json = metadata_json.into();
        self
    }

    pub fn to_sql_command(&self) -> SqlCommand {
        SqlCommand::new(
            INSERT_INPUT_REF_SQL,
            vec![
                SqlParam::required("input_ref_id", self.input_ref_id.clone()),
                SqlParam::required("run_id", self.run_id.clone()),
                SqlParam::required("dataset_kind", self.dataset_kind.clone()),
                SqlParam::required("original_filename", self.original_filename.clone()),
                SqlParam::required("source_uri", self.source_uri.clone()),
                SqlParam::required("file_hash_sha256", self.file_hash_sha256.clone()),
                SqlParam::required("size_bytes", self.size_bytes.to_string()),
                SqlParam::required("encoding", self.encoding.clone()),
                SqlParam::required("delimiter", self.delimiter.clone()),
                SqlParam::required("has_header", self.has_header.to_string()),
                SqlParam::required("schema_version", self.schema_version.clone()),
                SqlParam::nullable(
                    "detected_row_count",
                    self.detected_row_count.map(|value| value.to_string()),
                ),
                SqlParam::nullable(
                    "detected_column_count",
                    self.detected_column_count.map(|value| value.to_string()),
                ),
                SqlParam::required("metadata", self.metadata_json.clone()),
            ],
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InsertJob {
    job_id: String,
    run_id: String,
    job_type: String,
    state: JobState,
    priority: i32,
    max_attempts: i32,
    payload_json: String,
}

impl InsertJob {
    pub fn queued(
        job_id: impl Into<String>,
        run_id: impl Into<String>,
        job_type: impl Into<String>,
    ) -> Self {
        Self {
            job_id: job_id.into(),
            run_id: run_id.into(),
            job_type: job_type.into(),
            state: JobState::Queued,
            priority: 0,
            max_attempts: 3,
            payload_json: "{}".to_string(),
        }
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_max_attempts(mut self, max_attempts: i32) -> Self {
        self.max_attempts = max_attempts;
        self
    }

    pub fn with_payload_json(mut self, payload_json: impl Into<String>) -> Self {
        self.payload_json = payload_json.into();
        self
    }

    pub fn to_sql_command(&self) -> SqlCommand {
        SqlCommand::new(
            INSERT_JOB_SQL,
            vec![
                SqlParam::required("job_id", self.job_id.clone()),
                SqlParam::required("run_id", self.run_id.clone()),
                SqlParam::required("job_type", self.job_type.clone()),
                SqlParam::required("state", self.state.as_str()),
                SqlParam::required("priority", self.priority.to_string()),
                SqlParam::required("max_attempts", self.max_attempts.to_string()),
                SqlParam::required("payload", self.payload_json.clone()),
            ],
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateJobState {
    job_id: String,
    run_id: String,
    state: JobState,
    progress_percent: Option<i32>,
    stage: Option<String>,
    failure_kind: Option<String>,
    failure_message_masked: Option<String>,
}

impl UpdateJobState {
    pub fn new(job_id: impl Into<String>, run_id: impl Into<String>, state: JobState) -> Self {
        Self {
            job_id: job_id.into(),
            run_id: run_id.into(),
            state,
            progress_percent: None,
            stage: None,
            failure_kind: None,
            failure_message_masked: None,
        }
    }

    pub fn with_progress(mut self, progress_percent: i32) -> Self {
        self.progress_percent = Some(progress_percent);
        self
    }

    pub fn with_stage(mut self, stage: impl Into<String>) -> Self {
        self.stage = Some(stage.into());
        self
    }

    pub fn with_failure(
        mut self,
        failure_kind: impl Into<String>,
        failure_message_masked: impl Into<String>,
    ) -> Self {
        self.failure_kind = Some(failure_kind.into());
        self.failure_message_masked = Some(failure_message_masked.into());
        self
    }

    pub fn to_sql_command(&self) -> SqlCommand {
        SqlCommand::new(
            UPDATE_JOB_STATE_SQL,
            vec![
                SqlParam::required("state", self.state.as_str()),
                SqlParam::nullable(
                    "progress_percent",
                    self.progress_percent.map(|value| value.to_string()),
                ),
                SqlParam::nullable("stage", self.stage.clone()),
                SqlParam::nullable("failure_kind", self.failure_kind.clone()),
                SqlParam::nullable(
                    "failure_message_masked",
                    self.failure_message_masked.clone(),
                ),
                SqlParam::required("job_id", self.job_id.clone()),
                SqlParam::required("run_id", self.run_id.clone()),
            ],
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InsertIssue {
    issue_id: String,
    run_id: String,
    input_ref_id: Option<String>,
    normalized_ref_id: Option<String>,
    output_ref_id: Option<String>,
    dataset_kind: Option<String>,
    source_row_number: Option<i64>,
    column_name: Option<String>,
    raw_column_name: Option<String>,
    issue_category: String,
    issue_code: String,
    severity: String,
    readiness_effect: String,
    message: String,
    evidence_ref: Option<String>,
    evidence_value_masked: Option<String>,
    status: String,
    privacy_status: String,
    metadata_json: String,
}

impl InsertIssue {
    pub fn new(
        issue_id: impl Into<String>,
        run_id: impl Into<String>,
        issue_category: impl Into<String>,
        issue_code: impl Into<String>,
        severity: impl Into<String>,
        readiness_effect: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            issue_id: issue_id.into(),
            run_id: run_id.into(),
            input_ref_id: None,
            normalized_ref_id: None,
            output_ref_id: None,
            dataset_kind: None,
            source_row_number: None,
            column_name: None,
            raw_column_name: None,
            issue_category: issue_category.into(),
            issue_code: issue_code.into(),
            severity: severity.into(),
            readiness_effect: readiness_effect.into(),
            message: message.into(),
            evidence_ref: None,
            evidence_value_masked: None,
            status: "open".to_string(),
            privacy_status: "public".to_string(),
            metadata_json: "{}".to_string(),
        }
    }

    pub fn with_input_ref(mut self, input_ref_id: impl Into<String>) -> Self {
        self.input_ref_id = Some(input_ref_id.into());
        self
    }

    pub fn with_normalized_ref(mut self, normalized_ref_id: impl Into<String>) -> Self {
        self.normalized_ref_id = Some(normalized_ref_id.into());
        self
    }

    pub fn with_output_ref(mut self, output_ref_id: impl Into<String>) -> Self {
        self.output_ref_id = Some(output_ref_id.into());
        self
    }

    pub fn with_dataset_kind(mut self, dataset_kind: impl Into<String>) -> Self {
        self.dataset_kind = Some(dataset_kind.into());
        self
    }

    pub fn with_source_location(
        mut self,
        source_row_number: i64,
        column_name: impl Into<String>,
        raw_column_name: impl Into<String>,
    ) -> Self {
        self.source_row_number = Some(source_row_number);
        self.column_name = Some(column_name.into());
        self.raw_column_name = Some(raw_column_name.into());
        self
    }

    pub fn with_evidence(
        mut self,
        evidence_ref: impl Into<String>,
        evidence_value_masked: impl Into<String>,
    ) -> Self {
        self.evidence_ref = Some(evidence_ref.into());
        self.evidence_value_masked = Some(evidence_value_masked.into());
        self
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = status.into();
        self
    }

    pub fn with_privacy_status(mut self, privacy_status: impl Into<String>) -> Self {
        self.privacy_status = privacy_status.into();
        self
    }

    pub fn with_metadata_json(mut self, metadata_json: impl Into<String>) -> Self {
        self.metadata_json = metadata_json.into();
        self
    }

    pub fn to_sql_command(&self) -> SqlCommand {
        SqlCommand::new(
            INSERT_ISSUE_SQL,
            vec![
                SqlParam::required("issue_id", self.issue_id.clone()),
                SqlParam::required("run_id", self.run_id.clone()),
                SqlParam::nullable("input_ref_id", self.input_ref_id.clone()),
                SqlParam::nullable("normalized_ref_id", self.normalized_ref_id.clone()),
                SqlParam::nullable("output_ref_id", self.output_ref_id.clone()),
                SqlParam::nullable("dataset_kind", self.dataset_kind.clone()),
                SqlParam::nullable(
                    "source_row_number",
                    self.source_row_number.map(|value| value.to_string()),
                ),
                SqlParam::nullable("column_name", self.column_name.clone()),
                SqlParam::nullable("raw_column_name", self.raw_column_name.clone()),
                SqlParam::required("issue_category", self.issue_category.clone()),
                SqlParam::required("issue_code", self.issue_code.clone()),
                SqlParam::required("severity", self.severity.clone()),
                SqlParam::required("readiness_effect", self.readiness_effect.clone()),
                SqlParam::required("message", self.message.clone()),
                SqlParam::nullable("evidence_ref", self.evidence_ref.clone()),
                SqlParam::nullable("evidence_value_masked", self.evidence_value_masked.clone()),
                SqlParam::required("status", self.status.clone()),
                SqlParam::required("privacy_status", self.privacy_status.clone()),
                SqlParam::required("metadata", self.metadata_json.clone()),
            ],
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InsertRunArtifact {
    run_artifact_id: String,
    run_id: String,
    tenant_id: String,
    input_ref_id: String,
    normalized_ref_id: String,
    policy_ref_id: String,
    output_ref_id: String,
    audit_ref_id: String,
}

impl InsertRunArtifact {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        run_artifact_id: impl Into<String>,
        run_id: impl Into<String>,
        tenant_id: impl Into<String>,
        input_ref_id: impl Into<String>,
        normalized_ref_id: impl Into<String>,
        policy_ref_id: impl Into<String>,
        output_ref_id: impl Into<String>,
        audit_ref_id: impl Into<String>,
    ) -> Self {
        Self {
            run_artifact_id: run_artifact_id.into(),
            run_id: run_id.into(),
            tenant_id: tenant_id.into(),
            input_ref_id: input_ref_id.into(),
            normalized_ref_id: normalized_ref_id.into(),
            policy_ref_id: policy_ref_id.into(),
            output_ref_id: output_ref_id.into(),
            audit_ref_id: audit_ref_id.into(),
        }
    }

    pub fn to_sql_command(&self) -> SqlCommand {
        SqlCommand::new(
            INSERT_RUN_ARTIFACT_SQL,
            vec![
                SqlParam::required("run_artifact_id", self.run_artifact_id.clone()),
                SqlParam::required("run_id", self.run_id.clone()),
                SqlParam::required("tenant_id", self.tenant_id.clone()),
                SqlParam::required("input_ref_id", self.input_ref_id.clone()),
                SqlParam::required("normalized_ref_id", self.normalized_ref_id.clone()),
                SqlParam::required("policy_ref_id", self.policy_ref_id.clone()),
                SqlParam::required("output_ref_id", self.output_ref_id.clone()),
                SqlParam::required("audit_ref_id", self.audit_ref_id.clone()),
            ],
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobState {
    Queued,
    Running,
    RetryWait,
    Succeeded,
    Failed,
    CancelRequested,
    Canceled,
}

impl JobState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::RetryWait => "retry_wait",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::CancelRequested => "cancel_requested",
            Self::Canceled => "canceled",
        }
    }
}

impl TryFrom<&str> for JobState {
    type Error = InvalidJobState;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "queued" => Ok(Self::Queued),
            "running" => Ok(Self::Running),
            "retry_wait" => Ok(Self::RetryWait),
            "succeeded" => Ok(Self::Succeeded),
            "failed" => Ok(Self::Failed),
            "cancel_requested" => Ok(Self::CancelRequested),
            "canceled" => Ok(Self::Canceled),
            _ => Err(InvalidJobState(value.to_string())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidJobState(String);

impl fmt::Display for InvalidJobState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid PostgreSQL job state: {}", self.0)
    }
}

impl std::error::Error for InvalidJobState {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_record_insert_targets_run_records_and_binds_run_id() {
        let command = InsertRunRecord::new(
            "run_01J00000000000000000000000",
            "tenant_01J000000000000000000000",
            "created",
            "unknown",
            "local_db.v1",
        )
        .to_sql_command();

        assert!(command
            .statement()
            .contains("INSERT INTO laborlens.run_records"));
        assert!(command.statement().contains("run_id"));
        assert_eq!(command.bind_names()[0], "run_id");
    }

    #[test]
    fn input_ref_insert_targets_input_refs_and_binds_hash_and_schema_version() {
        let command = InsertInputRef::new(
            "src_01J00000000000000000000000",
            "run_01J00000000000000000000000",
            "attendance",
            "attendance.csv",
            "source://attendance.csv",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            120,
            "utf-8",
            ",",
            true,
            "attendance_csv.v1",
        )
        .to_sql_command();

        assert!(command
            .statement()
            .contains("INSERT INTO laborlens.input_refs"));
        assert!(command.statement().contains("file_hash_sha256"));
        assert!(command.statement().contains("schema_version"));
        assert_eq!(command.bind_names()[5], "file_hash_sha256");
        assert_eq!(command.bind_names()[10], "schema_version");
    }

    #[test]
    fn job_state_update_uses_allowed_state_values() {
        assert!(JobState::try_from("running").is_ok());
        assert!(JobState::try_from("not_a_state").is_err());

        let command = UpdateJobState::new(
            "job_01J00000000000000000000000",
            "run_01J00000000000000000000000",
            JobState::Running,
        )
        .with_progress(25)
        .with_stage("schema_checking")
        .to_sql_command();

        assert!(command.statement().contains("UPDATE laborlens.jobs"));
        assert_eq!(command.bind_names()[0], "state");
        assert_eq!(command.params()[0].value(), Some("running"));
    }

    #[test]
    fn run_artifact_insert_binds_all_traceability_refs() {
        let command = InsertRunArtifact::new(
            "rart_01J0000000000000000000000",
            "run_01J00000000000000000000000",
            "tenant_01J000000000000000000000",
            "src_01J00000000000000000000000",
            "norm_01J0000000000000000000000",
            "policy_01J00000000000000000000",
            "out_01J00000000000000000000000",
            "audit_01J00000000000000000000",
        )
        .to_sql_command();

        assert!(command
            .statement()
            .contains("INSERT INTO laborlens.run_artifacts"));
        assert!(command.statement().contains("input_ref_id"));
        assert!(command.statement().contains("normalized_ref_id"));
        assert!(command.statement().contains("policy_ref_id"));
        assert!(command.statement().contains("output_ref_id"));
        assert!(command.statement().contains("audit_ref_id"));
        assert_eq!(
            command.bind_names(),
            vec![
                "run_artifact_id",
                "run_id",
                "tenant_id",
                "input_ref_id",
                "normalized_ref_id",
                "policy_ref_id",
                "output_ref_id",
                "audit_ref_id",
            ]
        );
    }
}
