param(
    [string]$Root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path,
    [string]$MigrationRelativePath = "db\migrations\0001_initial_postgresql_schema.sql",
    [string]$DemoSeedRelativePath = "db\seeds\0001_demo_japanese_employees.sql",
    [string]$InterfaceDocRelativePath = "docs\planning\DB-INTERFACES.md",
    [string]$RustAdapterRelativePath = "apps\laborlens-rust\src\shared\db.rs",
    [string]$RustSharedModRelativePath = "apps\laborlens-rust\src\shared\mod.rs"
)

$ErrorActionPreference = "Stop"

function Join-RepoPath {
    param([string]$RelativePath)
    return Join-Path $Root $RelativePath
}

function Assert-Exists {
    param([string]$RelativePath)
    $path = Join-RepoPath $RelativePath
    if (-not (Test-Path $path)) {
        throw "Missing required path: $RelativePath"
    }
}

function Assert-Matches {
    param(
        [string]$Content,
        [string]$Pattern,
        [string]$Reason
    )
    if ($Content -notmatch $Pattern) {
        throw "Expected pattern missing: $Reason"
    }
}

function Assert-NotMatches {
    param(
        [string]$Content,
        [string]$Pattern,
        [string]$Reason
    )
    if ($Content -match $Pattern) {
        throw "Forbidden pattern found: $Reason"
    }
}

function Get-TableBlock {
    param(
        [string]$Sql,
        [string]$Table
    )
    $pattern = "(?is)CREATE\s+TABLE\s+(?:IF\s+NOT\s+EXISTS\s+)?(?:laborlens\.)?$Table\s*\((.*?)\)\s*;"
    $match = [regex]::Match($Sql, $pattern)
    if (-not $match.Success) {
        throw "Missing required table: $Table"
    }
    return $match.Groups[1].Value
}

$migrationPath = Join-RepoPath $MigrationRelativePath
$interfaceDocPath = Join-RepoPath $InterfaceDocRelativePath

Assert-Exists $MigrationRelativePath
Assert-Exists $DemoSeedRelativePath
Assert-Exists $InterfaceDocRelativePath
Assert-Exists $RustAdapterRelativePath
Assert-Exists $RustSharedModRelativePath

$sql = Get-Content -Raw -Encoding UTF8 $migrationPath
$demoSeed = Get-Content -Raw -Encoding UTF8 (Join-RepoPath $DemoSeedRelativePath)
$doc = Get-Content -Raw -Encoding UTF8 $interfaceDocPath
$rustAdapter = Get-Content -Raw -Encoding UTF8 (Join-RepoPath $RustAdapterRelativePath)
$rustSharedMod = Get-Content -Raw -Encoding UTF8 (Join-RepoPath $RustSharedModRelativePath)

$requiredTables = @(
    "run_records",
    "input_refs",
    "normalized_refs",
    "policy_refs",
    "output_refs",
    "audit_refs",
    "run_artifacts",
    "jobs",
    "issues",
    "privacy_suppressions",
    "artifact_manifests",
    "report_artifacts"
)

$tableBlocks = @{}
foreach ($table in $requiredTables) {
    $block = Get-TableBlock -Sql $sql -Table $table
    $tableBlocks[$table] = $block
    Assert-Matches $block "(?is)CONSTRAINT\s+pk_$table\s+PRIMARY\s+KEY" "$table has a named primary key"
}

foreach ($table in $requiredTables | Where-Object { $_ -ne "run_records" }) {
    Assert-Matches $tableBlocks[$table] "(?is)\brun_id\s+TEXT\s+NOT\s+NULL" "$table has a non-null run_id"
    Assert-Matches $tableBlocks[$table] "(?is)FOREIGN\s+KEY\s*\(\s*run_id\s*\)\s+REFERENCES\s+(?:laborlens\.)?run_records\s*\(\s*run_id\s*\)" "$table references run_records(run_id)"
}

$requiredNamedConstraints = @(
    "uq_input_refs_run_dataset_hash",
    "uq_normalized_refs_run_dataset",
    "uq_policy_refs_run_policy",
    "uq_output_refs_run_artifact",
    "uq_run_artifacts_output_ref",
    "uq_report_artifacts_run_kind_path",
    "ck_jobs_state",
    "ck_issues_severity",
    "ck_privacy_suppressions_status",
    "ck_report_artifacts_privacy_filtered"
)

foreach ($constraint in $requiredNamedConstraints) {
    Assert-Matches $sql "(?is)CONSTRAINT\s+$constraint\b" "named constraint $constraint exists"
}

$requiredIndexes = @(
    "idx_input_refs_run_id",
    "idx_normalized_refs_run_id",
    "idx_policy_refs_run_id",
    "idx_output_refs_run_artifact",
    "idx_audit_refs_run_id",
    "idx_run_artifacts_run_id",
    "idx_jobs_state_available",
    "idx_jobs_run_state",
    "idx_issues_run_category",
    "idx_issues_run_severity",
    "idx_privacy_suppressions_run_reason",
    "idx_artifact_manifests_run_id",
    "idx_report_artifacts_lookup"
)

foreach ($index in $requiredIndexes) {
    Assert-Matches $sql "(?is)CREATE\s+INDEX\s+$index\b" "index $index exists"
}

$runArtifactRequiredColumns = @(
    "input_ref_id",
    "normalized_ref_id",
    "policy_ref_id",
    "output_ref_id",
    "audit_ref_id"
)

foreach ($column in $runArtifactRequiredColumns) {
    Assert-Matches $tableBlocks["run_artifacts"] "(?is)\b$column\s+TEXT\s+NOT\s+NULL" "run_artifacts has $column"
}

Assert-Matches $tableBlocks["policy_refs"] "(?is)\bsuppression_policy_version\s+TEXT\s+NOT\s+NULL" "policy_refs maps Lean PolicyRef.suppressionPolicyVersion"
Assert-Matches $tableBlocks["policy_refs"] "(?is)\binference_threshold_k\s+INTEGER\s+NOT\s+NULL" "policy_refs records inference_threshold_k"
Assert-Matches $tableBlocks["privacy_suppressions"] "(?is)\bthreshold_name\s+TEXT" "privacy_suppressions records threshold_name"
Assert-Matches $tableBlocks["privacy_suppressions"] "(?is)\bthreshold_value\s+TEXT" "privacy_suppressions records threshold_value"
Assert-Matches $tableBlocks["privacy_suppressions"] "(?is)\breason_code\s+TEXT\s+NOT\s+NULL" "privacy_suppressions records reason_code"

$publicTableNames = @("output_refs", "artifact_manifests", "report_artifacts")
$forbiddenPublicColumns = "(?is)\b(employee_name|email|fatigue_value|sleep_duration|fatigue_comment|raw_csv_row|raw_value|raw_payload)\b"
foreach ($table in $publicTableNames) {
    Assert-NotMatches $tableBlocks[$table] $forbiddenPublicColumns "$table must not define raw or sensitive public/report columns"
}

Assert-Matches $tableBlocks["report_artifacts"] "(?is)privacy_filtered\s+BOOLEAN\s+NOT\s+NULL" "report_artifacts records privacy_filtered"
Assert-Matches $tableBlocks["report_artifacts"] "(?is)ck_report_artifacts_privacy_filtered.*privacy_filtered\s+IS\s+TRUE|ck_report_artifacts_privacy_filtered.*privacy_filtered\s*=\s*TRUE" "report_artifacts is limited to privacy-filtered rows"
Assert-Matches $sql "(?is)COMMENT\s+ON\s+TABLE\s+(?:laborlens\.)?report_artifacts\s+IS\s+'.*No raw CSV rows.*suppressed.*'" "report_artifacts raw-data protection comment exists"
Assert-Matches $sql "(?is)COMMENT\s+ON\s+TABLE\s+(?:laborlens\.)?artifact_manifests\s+IS\s+'.*suppressed.*metadata.*'" "artifact_manifests suppressed metadata comment exists"
Assert-Matches $sql "(?is)Partitioning policy" "migration records the initial partitioning policy"

Assert-Matches $demoSeed "(?is)CREATE\s+TABLE\s+IF\s+NOT\s+EXISTS\s+laborlens\.demo_employees" "demo seed creates demo_employees table"
Assert-Matches $demoSeed "(?is)generate_series\s*\(\s*1\s*,\s*1000\s*\)" "demo seed generates 1000 records"
Assert-Matches $demoSeed "(?is)demo_japanese_employees\.v1" "demo seed has stable seed version"
Assert-Matches $demoSeed "(?is)SELECT\s+count\(\*\).*seeded_count" "demo seed verifies inserted row count"

foreach ($name in @("Radomil", "Pike", "Leonard", "RunArtifact", "privacy_suppressions", "report_artifacts", "jobs")) {
    Assert-Matches $doc "(?is)\b$name\b" "interface document mentions $name"
}

Assert-Matches $rustSharedMod "(?is)pub\s+mod\s+db\s*;" "shared module exposes db command model"

$requiredRustCommands = @(
    "SqlCommand",
    "SqlParam",
    "InsertRunRecord",
    "InsertInputRef",
    "InsertJob",
    "UpdateJobState",
    "InsertIssue",
    "InsertRunArtifact",
    "JobState"
)

foreach ($commandName in $requiredRustCommands) {
    Assert-Matches $rustAdapter "(?is)\b$commandName\b" "Rust adapter defines $commandName"
}

$requiredRustSqlTargets = @(
    "INSERT\s+INTO\s+laborlens\.run_records",
    "INSERT\s+INTO\s+laborlens\.input_refs",
    "INSERT\s+INTO\s+laborlens\.jobs",
    "UPDATE\s+laborlens\.jobs",
    "INSERT\s+INTO\s+laborlens\.issues",
    "INSERT\s+INTO\s+laborlens\.run_artifacts"
)

foreach ($target in $requiredRustSqlTargets) {
    Assert-Matches $rustAdapter "(?is)$target" "Rust adapter SQL target $target exists"
}

foreach ($columnName in @("run_id", "file_hash_sha256", "schema_version", "input_ref_id", "normalized_ref_id", "policy_ref_id", "output_ref_id", "audit_ref_id")) {
    Assert-Matches $rustAdapter "(?is)\b$columnName\b" "Rust adapter mentions required column $columnName"
}

foreach ($stateName in @("queued", "running", "retry_wait", "succeeded", "failed", "cancel_requested", "canceled")) {
    Assert-Matches $rustAdapter "(?is)`"$stateName`"" "Rust adapter supports job state $stateName"
}

Write-Host "DB schema static validation passed."
