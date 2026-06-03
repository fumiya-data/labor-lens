-- LaborLens PostgreSQL initial schema.
-- This migration defines reference, traceability, job, issue, privacy, and
-- suppressed-report artifact metadata tables. Application code generates IDs.
--
-- Raw data protection:
-- - Source CSV bytes stay in Source Archive, not in public/report tables.
-- - Public/report tables store suppressed artifact metadata only.
-- - Raw CSV rows, fatigue values, sleep durations, fatigue comments, employee
--   names, and emails are not columns in report artifact tables.
--
-- Partitioning policy:
-- Initial tables stay unpartitioned to keep the first migration auditable.
-- When materialized high-volume normalized row tables are added, partition
-- those detail tables by run_id hash or by period range after workload proof.
-- Keep run_records, jobs, and artifact metadata unpartitioned until measured
-- query or retention pressure justifies a separate migration.

CREATE SCHEMA IF NOT EXISTS laborlens;

CREATE TABLE IF NOT EXISTS laborlens.run_records (
    run_id TEXT NOT NULL,
    tenant_id TEXT NOT NULL,
    run_status TEXT NOT NULL,
    readiness_status TEXT NOT NULL,
    period_start DATE,
    period_end DATE,
    schema_version TEXT NOT NULL,
    code_version TEXT,
    settings JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    failure_kind TEXT,
    failure_message_masked TEXT,
    CONSTRAINT pk_run_records PRIMARY KEY (run_id),
    CONSTRAINT ck_run_records_run_id_prefix CHECK (run_id LIKE 'run_%'),
    CONSTRAINT ck_run_records_tenant_id_prefix CHECK (tenant_id LIKE 'tenant_%'),
    CONSTRAINT ck_run_records_status CHECK (run_status IN (
        'created',
        'queued',
        'running',
        'completed',
        'failed',
        'cancel_requested',
        'canceled'
    )),
    CONSTRAINT ck_run_records_readiness CHECK (readiness_status IN (
        'unknown',
        'ready',
        'partial',
        'blocked'
    )),
    CONSTRAINT ck_run_records_period_order CHECK (
        period_start IS NULL OR period_end IS NULL OR period_start <= period_end
    ),
    CONSTRAINT ck_run_records_settings_object CHECK (jsonb_typeof(settings) = 'object')
);

CREATE TABLE IF NOT EXISTS laborlens.input_refs (
    input_ref_id TEXT NOT NULL,
    run_id TEXT NOT NULL,
    dataset_kind TEXT NOT NULL,
    original_filename TEXT NOT NULL,
    source_uri TEXT NOT NULL,
    file_hash_sha256 TEXT NOT NULL,
    size_bytes BIGINT NOT NULL,
    encoding TEXT NOT NULL,
    delimiter TEXT NOT NULL,
    has_header BOOLEAN NOT NULL,
    detected_row_count BIGINT,
    detected_column_count INTEGER,
    schema_version TEXT NOT NULL,
    uploaded_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    CONSTRAINT pk_input_refs PRIMARY KEY (input_ref_id),
    CONSTRAINT uq_input_refs_run_input UNIQUE (run_id, input_ref_id),
    CONSTRAINT uq_input_refs_run_dataset_hash UNIQUE (run_id, dataset_kind, file_hash_sha256),
    CONSTRAINT fk_input_refs_run_id FOREIGN KEY (run_id) REFERENCES laborlens.run_records(run_id),
    CONSTRAINT ck_input_refs_input_ref_id_prefix CHECK (input_ref_id LIKE 'src_%'),
    CONSTRAINT ck_input_refs_size_nonnegative CHECK (size_bytes >= 0),
    CONSTRAINT ck_input_refs_file_hash_sha256 CHECK (file_hash_sha256 ~ '^[0-9a-fA-F]{64}$'),
    CONSTRAINT ck_input_refs_counts_nonnegative CHECK (
        (detected_row_count IS NULL OR detected_row_count >= 0)
        AND (detected_column_count IS NULL OR detected_column_count >= 0)
    ),
    CONSTRAINT ck_input_refs_metadata_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE TABLE IF NOT EXISTS laborlens.normalized_refs (
    normalized_ref_id TEXT NOT NULL,
    run_id TEXT NOT NULL,
    input_ref_id TEXT NOT NULL,
    dataset_kind TEXT NOT NULL,
    normalized_dataset_id TEXT NOT NULL,
    normalization_rule_version TEXT NOT NULL,
    column_mapping_version TEXT NOT NULL,
    data_state TEXT NOT NULL,
    internal_storage_ref TEXT,
    row_count BIGINT,
    generated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    CONSTRAINT pk_normalized_refs PRIMARY KEY (normalized_ref_id),
    CONSTRAINT uq_normalized_refs_run_ref UNIQUE (run_id, normalized_ref_id),
    CONSTRAINT uq_normalized_refs_run_dataset UNIQUE (run_id, dataset_kind, normalized_dataset_id),
    CONSTRAINT fk_normalized_refs_run_id FOREIGN KEY (run_id) REFERENCES laborlens.run_records(run_id),
    CONSTRAINT fk_normalized_refs_input_ref FOREIGN KEY (run_id, input_ref_id)
        REFERENCES laborlens.input_refs(run_id, input_ref_id),
    CONSTRAINT ck_normalized_refs_id_prefix CHECK (normalized_ref_id LIKE 'norm_%'),
    CONSTRAINT ck_normalized_refs_dataset_id_prefix CHECK (normalized_dataset_id LIKE 'ds_%'),
    CONSTRAINT ck_normalized_refs_data_state CHECK (data_state IN (
        'parsed',
        'normalized',
        'validated',
        'join_assessed',
        'analysis_ready'
    )),
    CONSTRAINT ck_normalized_refs_row_count_nonnegative CHECK (row_count IS NULL OR row_count >= 0),
    CONSTRAINT ck_normalized_refs_metadata_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE TABLE IF NOT EXISTS laborlens.policy_refs (
    policy_ref_id TEXT NOT NULL,
    run_id TEXT NOT NULL,
    suppression_policy_version TEXT NOT NULL,
    inference_threshold_k INTEGER NOT NULL,
    small_group_min_effective_data_count INTEGER NOT NULL DEFAULT 10,
    caution_group_min_effective_data_count INTEGER NOT NULL DEFAULT 30,
    rag_index_version TEXT NOT NULL,
    access_policy_version TEXT NOT NULL,
    policy_hash_sha256 TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    CONSTRAINT pk_policy_refs PRIMARY KEY (policy_ref_id),
    CONSTRAINT uq_policy_refs_run_ref UNIQUE (run_id, policy_ref_id),
    CONSTRAINT uq_policy_refs_run_policy UNIQUE (
        run_id,
        suppression_policy_version,
        access_policy_version,
        rag_index_version
    ),
    CONSTRAINT fk_policy_refs_run_id FOREIGN KEY (run_id) REFERENCES laborlens.run_records(run_id),
    CONSTRAINT ck_policy_refs_id_prefix CHECK (policy_ref_id LIKE 'policy_%'),
    CONSTRAINT ck_policy_refs_threshold_positive CHECK (
        inference_threshold_k > 0
        AND small_group_min_effective_data_count > 0
        AND caution_group_min_effective_data_count >= small_group_min_effective_data_count
    ),
    CONSTRAINT ck_policy_refs_hash_sha256 CHECK (
        policy_hash_sha256 IS NULL OR policy_hash_sha256 ~ '^[0-9a-fA-F]{64}$'
    ),
    CONSTRAINT ck_policy_refs_metadata_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE TABLE IF NOT EXISTS laborlens.output_refs (
    output_ref_id TEXT NOT NULL,
    run_id TEXT NOT NULL,
    artifact_id TEXT NOT NULL,
    artifact_kind TEXT NOT NULL,
    artifact_format TEXT NOT NULL,
    artifact_uri TEXT NOT NULL,
    output_hash_sha256 TEXT NOT NULL,
    schema_version TEXT NOT NULL,
    privacy_filtered BOOLEAN NOT NULL,
    generated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    CONSTRAINT pk_output_refs PRIMARY KEY (output_ref_id),
    CONSTRAINT uq_output_refs_run_ref UNIQUE (run_id, output_ref_id),
    CONSTRAINT uq_output_refs_run_artifact UNIQUE (run_id, artifact_id),
    CONSTRAINT fk_output_refs_run_id FOREIGN KEY (run_id) REFERENCES laborlens.run_records(run_id),
    CONSTRAINT ck_output_refs_output_ref_id_prefix CHECK (output_ref_id LIKE 'out_%'),
    CONSTRAINT ck_output_refs_artifact_id_prefix CHECK (artifact_id LIKE 'art_%'),
    CONSTRAINT ck_output_refs_hash_sha256 CHECK (output_hash_sha256 ~ '^[0-9a-fA-F]{64}$'),
    CONSTRAINT ck_output_refs_privacy_filtered CHECK (privacy_filtered IS TRUE),
    CONSTRAINT ck_output_refs_metadata_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE TABLE IF NOT EXISTS laborlens.audit_refs (
    audit_ref_id TEXT NOT NULL,
    run_id TEXT NOT NULL,
    access_log_ref TEXT NOT NULL,
    actor_id TEXT NOT NULL,
    actor_role TEXT NOT NULL,
    execution_reason TEXT NOT NULL,
    action TEXT NOT NULL,
    purpose_ticket_ref TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    CONSTRAINT pk_audit_refs PRIMARY KEY (audit_ref_id),
    CONSTRAINT uq_audit_refs_run_ref UNIQUE (run_id, audit_ref_id),
    CONSTRAINT uq_audit_refs_run_access_log UNIQUE (run_id, access_log_ref),
    CONSTRAINT fk_audit_refs_run_id FOREIGN KEY (run_id) REFERENCES laborlens.run_records(run_id),
    CONSTRAINT ck_audit_refs_id_prefix CHECK (audit_ref_id LIKE 'audit_%'),
    CONSTRAINT ck_audit_refs_metadata_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE TABLE IF NOT EXISTS laborlens.run_artifacts (
    run_artifact_id TEXT NOT NULL,
    run_id TEXT NOT NULL,
    tenant_id TEXT NOT NULL,
    input_ref_id TEXT NOT NULL,
    normalized_ref_id TEXT NOT NULL,
    policy_ref_id TEXT NOT NULL,
    output_ref_id TEXT NOT NULL,
    audit_ref_id TEXT NOT NULL,
    generated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    CONSTRAINT pk_run_artifacts PRIMARY KEY (run_artifact_id),
    CONSTRAINT uq_run_artifacts_output_ref UNIQUE (run_id, output_ref_id),
    CONSTRAINT fk_run_artifacts_run_id FOREIGN KEY (run_id) REFERENCES laborlens.run_records(run_id),
    CONSTRAINT fk_run_artifacts_input_ref FOREIGN KEY (run_id, input_ref_id)
        REFERENCES laborlens.input_refs(run_id, input_ref_id),
    CONSTRAINT fk_run_artifacts_normalized_ref FOREIGN KEY (run_id, normalized_ref_id)
        REFERENCES laborlens.normalized_refs(run_id, normalized_ref_id),
    CONSTRAINT fk_run_artifacts_policy_ref FOREIGN KEY (run_id, policy_ref_id)
        REFERENCES laborlens.policy_refs(run_id, policy_ref_id),
    CONSTRAINT fk_run_artifacts_output_ref FOREIGN KEY (run_id, output_ref_id)
        REFERENCES laborlens.output_refs(run_id, output_ref_id),
    CONSTRAINT fk_run_artifacts_audit_ref FOREIGN KEY (run_id, audit_ref_id)
        REFERENCES laborlens.audit_refs(run_id, audit_ref_id),
    CONSTRAINT ck_run_artifacts_id_prefix CHECK (run_artifact_id LIKE 'rart_%'),
    CONSTRAINT ck_run_artifacts_tenant_id_prefix CHECK (tenant_id LIKE 'tenant_%'),
    CONSTRAINT ck_run_artifacts_metadata_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE TABLE IF NOT EXISTS laborlens.jobs (
    job_id TEXT NOT NULL,
    run_id TEXT NOT NULL,
    job_type TEXT NOT NULL,
    state TEXT NOT NULL,
    priority INTEGER NOT NULL DEFAULT 0,
    attempt INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,
    available_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    locked_by TEXT,
    locked_at TIMESTAMPTZ,
    progress_percent INTEGER NOT NULL DEFAULT 0,
    stage TEXT,
    failure_kind TEXT,
    failure_message_masked TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ,
    payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    CONSTRAINT pk_jobs PRIMARY KEY (job_id),
    CONSTRAINT fk_jobs_run_id FOREIGN KEY (run_id) REFERENCES laborlens.run_records(run_id),
    CONSTRAINT ck_jobs_job_id_prefix CHECK (job_id LIKE 'job_%'),
    CONSTRAINT ck_jobs_state CHECK (state IN (
        'queued',
        'running',
        'retry_wait',
        'succeeded',
        'failed',
        'cancel_requested',
        'canceled'
    )),
    CONSTRAINT ck_jobs_attempts CHECK (attempt >= 0 AND max_attempts >= 1 AND attempt <= max_attempts),
    CONSTRAINT ck_jobs_progress CHECK (progress_percent >= 0 AND progress_percent <= 100),
    CONSTRAINT ck_jobs_payload_object CHECK (jsonb_typeof(payload) = 'object')
);

CREATE TABLE IF NOT EXISTS laborlens.issues (
    issue_id TEXT NOT NULL,
    run_id TEXT NOT NULL,
    input_ref_id TEXT,
    normalized_ref_id TEXT,
    output_ref_id TEXT,
    dataset_kind TEXT,
    source_row_number BIGINT,
    column_name TEXT,
    raw_column_name TEXT,
    issue_category TEXT NOT NULL,
    issue_code TEXT NOT NULL,
    severity TEXT NOT NULL,
    readiness_effect TEXT NOT NULL,
    message TEXT NOT NULL,
    evidence_ref TEXT,
    evidence_value_masked TEXT,
    status TEXT NOT NULL,
    privacy_status TEXT NOT NULL DEFAULT 'public',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    resolved_at TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    CONSTRAINT pk_issues PRIMARY KEY (issue_id),
    CONSTRAINT fk_issues_run_id FOREIGN KEY (run_id) REFERENCES laborlens.run_records(run_id),
    CONSTRAINT fk_issues_input_ref FOREIGN KEY (run_id, input_ref_id)
        REFERENCES laborlens.input_refs(run_id, input_ref_id),
    CONSTRAINT fk_issues_normalized_ref FOREIGN KEY (run_id, normalized_ref_id)
        REFERENCES laborlens.normalized_refs(run_id, normalized_ref_id),
    CONSTRAINT fk_issues_output_ref FOREIGN KEY (run_id, output_ref_id)
        REFERENCES laborlens.output_refs(run_id, output_ref_id),
    CONSTRAINT ck_issues_issue_id_prefix CHECK (issue_id LIKE 'iss_%'),
    CONSTRAINT ck_issues_source_row_positive CHECK (source_row_number IS NULL OR source_row_number > 0),
    CONSTRAINT ck_issues_category CHECK (issue_category IN (
        'schema_issue',
        'data_quality_issue',
        'master_issue',
        'grain_issue',
        'join_issue',
        'privacy_issue',
        'processing_issue'
    )),
    CONSTRAINT ck_issues_severity CHECK (severity IN ('critical', 'high', 'medium', 'low')),
    CONSTRAINT ck_issues_readiness_effect CHECK (readiness_effect IN ('none', 'partial', 'blocked')),
    CONSTRAINT ck_issues_status CHECK (status IN ('open', 'acknowledged', 'resolved', 'ignored')),
    CONSTRAINT ck_issues_privacy_status CHECK (privacy_status IN ('public', 'masked', 'suppressed', 'internal_only')),
    CONSTRAINT ck_issues_metadata_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE TABLE IF NOT EXISTS laborlens.privacy_suppressions (
    suppression_id TEXT NOT NULL,
    run_id TEXT NOT NULL,
    policy_ref_id TEXT NOT NULL,
    output_ref_id TEXT,
    artifact_id TEXT,
    artifact_kind TEXT,
    target_type TEXT NOT NULL,
    target_ref TEXT NOT NULL,
    privacy_status TEXT NOT NULL,
    reason_code TEXT NOT NULL,
    reason_message TEXT NOT NULL,
    affected_count BIGINT,
    threshold_name TEXT,
    threshold_value TEXT,
    suppressed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    CONSTRAINT pk_privacy_suppressions PRIMARY KEY (suppression_id),
    CONSTRAINT fk_privacy_suppressions_run_id FOREIGN KEY (run_id) REFERENCES laborlens.run_records(run_id),
    CONSTRAINT fk_privacy_suppressions_policy_ref FOREIGN KEY (run_id, policy_ref_id)
        REFERENCES laborlens.policy_refs(run_id, policy_ref_id),
    CONSTRAINT fk_privacy_suppressions_output_ref FOREIGN KEY (run_id, output_ref_id)
        REFERENCES laborlens.output_refs(run_id, output_ref_id),
    CONSTRAINT ck_privacy_suppressions_id_prefix CHECK (suppression_id LIKE 'sup_%'),
    CONSTRAINT ck_privacy_suppressions_target_type CHECK (target_type IN (
        'row',
        'cell',
        'aggregate',
        'report_section',
        'artifact'
    )),
    CONSTRAINT ck_privacy_suppressions_status CHECK (privacy_status IN ('masked', 'suppressed', 'internal_only')),
    CONSTRAINT ck_privacy_suppressions_affected_nonnegative CHECK (affected_count IS NULL OR affected_count >= 0),
    CONSTRAINT ck_privacy_suppressions_metadata_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE TABLE IF NOT EXISTS laborlens.artifact_manifests (
    manifest_id TEXT NOT NULL,
    run_id TEXT NOT NULL,
    output_ref_id TEXT NOT NULL,
    manifest_uri TEXT NOT NULL,
    manifest_hash_sha256 TEXT NOT NULL,
    manifest_schema_version TEXT NOT NULL,
    privacy_filtered BOOLEAN NOT NULL,
    generated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    CONSTRAINT pk_artifact_manifests PRIMARY KEY (manifest_id),
    CONSTRAINT uq_artifact_manifests_run_manifest UNIQUE (run_id, manifest_id),
    CONSTRAINT uq_artifact_manifests_run_output UNIQUE (run_id, output_ref_id),
    CONSTRAINT fk_artifact_manifests_run_id FOREIGN KEY (run_id) REFERENCES laborlens.run_records(run_id),
    CONSTRAINT fk_artifact_manifests_output_ref FOREIGN KEY (run_id, output_ref_id)
        REFERENCES laborlens.output_refs(run_id, output_ref_id),
    CONSTRAINT ck_artifact_manifests_id_prefix CHECK (manifest_id LIKE 'manifest_%'),
    CONSTRAINT ck_artifact_manifests_hash_sha256 CHECK (manifest_hash_sha256 ~ '^[0-9a-fA-F]{64}$'),
    CONSTRAINT ck_artifact_manifests_privacy_filtered CHECK (privacy_filtered IS TRUE),
    CONSTRAINT ck_artifact_manifests_metadata_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE TABLE IF NOT EXISTS laborlens.report_artifacts (
    artifact_id TEXT NOT NULL,
    run_id TEXT NOT NULL,
    output_ref_id TEXT NOT NULL,
    manifest_id TEXT,
    artifact_kind TEXT NOT NULL,
    artifact_format TEXT NOT NULL,
    relative_path TEXT NOT NULL,
    content_hash_sha256 TEXT NOT NULL,
    schema_version TEXT NOT NULL,
    byte_size BIGINT,
    row_count BIGINT,
    privacy_filtered BOOLEAN NOT NULL,
    generated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    CONSTRAINT pk_report_artifacts PRIMARY KEY (artifact_id),
    CONSTRAINT uq_report_artifacts_run_artifact UNIQUE (run_id, artifact_id),
    CONSTRAINT uq_report_artifacts_run_kind_path UNIQUE (run_id, artifact_kind, relative_path),
    CONSTRAINT fk_report_artifacts_run_id FOREIGN KEY (run_id) REFERENCES laborlens.run_records(run_id),
    CONSTRAINT fk_report_artifacts_output_ref FOREIGN KEY (run_id, output_ref_id)
        REFERENCES laborlens.output_refs(run_id, output_ref_id),
    CONSTRAINT fk_report_artifacts_manifest FOREIGN KEY (run_id, manifest_id)
        REFERENCES laborlens.artifact_manifests(run_id, manifest_id),
    CONSTRAINT ck_report_artifacts_artifact_id_prefix CHECK (artifact_id LIKE 'art_%'),
    CONSTRAINT ck_report_artifacts_hash_sha256 CHECK (content_hash_sha256 ~ '^[0-9a-fA-F]{64}$'),
    CONSTRAINT ck_report_artifacts_size_nonnegative CHECK (byte_size IS NULL OR byte_size >= 0),
    CONSTRAINT ck_report_artifacts_row_count_nonnegative CHECK (row_count IS NULL OR row_count >= 0),
    CONSTRAINT ck_report_artifacts_privacy_filtered CHECK (privacy_filtered IS TRUE),
    CONSTRAINT ck_report_artifacts_metadata_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE INDEX idx_input_refs_run_id ON laborlens.input_refs(run_id, dataset_kind);
CREATE INDEX idx_normalized_refs_run_id ON laborlens.normalized_refs(run_id, dataset_kind);
CREATE INDEX idx_policy_refs_run_id ON laborlens.policy_refs(run_id);
CREATE INDEX idx_output_refs_run_artifact ON laborlens.output_refs(run_id, artifact_kind, artifact_id);
CREATE INDEX idx_audit_refs_run_id ON laborlens.audit_refs(run_id, created_at);
CREATE INDEX idx_run_artifacts_run_id ON laborlens.run_artifacts(run_id, generated_at);
CREATE INDEX idx_jobs_state_available ON laborlens.jobs(state, available_at, priority DESC);
CREATE INDEX idx_jobs_run_state ON laborlens.jobs(run_id, state);
CREATE INDEX idx_issues_run_category ON laborlens.issues(run_id, issue_category);
CREATE INDEX idx_issues_run_severity ON laborlens.issues(run_id, severity);
CREATE INDEX idx_issues_source_row ON laborlens.issues(input_ref_id, source_row_number);
CREATE INDEX idx_privacy_suppressions_run_reason ON laborlens.privacy_suppressions(run_id, reason_code);
CREATE INDEX idx_privacy_suppressions_artifact_target ON laborlens.privacy_suppressions(run_id, artifact_id, target_type);
CREATE INDEX idx_artifact_manifests_run_id ON laborlens.artifact_manifests(run_id, generated_at);
CREATE INDEX idx_report_artifacts_lookup ON laborlens.report_artifacts(run_id, artifact_kind, artifact_format);

COMMENT ON TABLE laborlens.run_records IS 'Run-level system of record for LaborLens local PostgreSQL execution.';
COMMENT ON TABLE laborlens.input_refs IS 'Source input reference metadata only; source bytes remain in Source Archive.';
COMMENT ON TABLE laborlens.normalized_refs IS 'Normalized dataset references and versions; row data lives in future internal detail tables.';
COMMENT ON TABLE laborlens.policy_refs IS 'Policy versions used for privacy suppression, guide access, and small-group thresholds.';
COMMENT ON TABLE laborlens.output_refs IS 'Privacy-filtered output references for user-visible artifacts.';
COMMENT ON TABLE laborlens.audit_refs IS 'Masked audit reference rows for traceability and approved access events.';
COMMENT ON TABLE laborlens.run_artifacts IS 'Traceability join for Lean RunArtifact input, normalized, policy, output, and audit refs.';
COMMENT ON TABLE laborlens.jobs IS 'PostgreSQL backed job queue for local server and worker coordination.';
COMMENT ON TABLE laborlens.issues IS 'Masked issue records for data quality, joinability, privacy, and processing diagnostics.';
COMMENT ON TABLE laborlens.privacy_suppressions IS 'Privacy suppression decisions including small-group threshold evidence and target refs.';
COMMENT ON TABLE laborlens.artifact_manifests IS 'Suppressed artifact metadata manifest table for privacy-filtered report outputs.';
COMMENT ON TABLE laborlens.report_artifacts IS 'No raw CSV rows are stored; this table contains suppressed report file metadata only for Pike and UI consumers.';
