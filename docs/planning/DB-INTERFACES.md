# DB-INTERFACES.md

日付: 2026-06-03
状態: initial implementation contract
担当: Dabian, PostgreSQL DB implementation
migration ファイル: `db/migrations/0001_initial_postgresql_schema.sql`
検証: `tools/validate-db-schema.ps1`

## 0. 適用範囲

この文書は、現在の agent 間で使う最初の PostgreSQL interface を固定する。対象は DB contract に限定し、Rust context code、Lean code、Python report rendering は変更しない。

初期 schema は次を保存する。

- run registration と job state
- source input reference
- normalized dataset reference
- policy reference
- output reference
- audit reference
- RunArtifact traceability
- issues
- privacy_suppressions
- 抑制済み artifact manifests と report_artifacts

raw CSV row storage と materialized normalized row table は後続に回す。最初の migration では、Rust engine、Lean specification name、Python renderer を接続するために必要な reference、version、hash、count、metadata を保存する。

## 1. agent 間 interface

| agent | interface | 許可される DB surface |
| --- | --- | --- |
| Radomil | Rust engine and report contract | application service 経由で `run_records`、`input_refs`、`normalized_refs`、`policy_refs`、`output_refs`、`audit_refs`、`run_artifacts`、`jobs`、`issues`、`privacy_suppressions`、`artifact_manifests`、`report_artifacts` を書き込む。 |
| Pike | Python report renderer | `artifact_manifests` と `report_artifacts` に listed された抑制済み file だけを読む。local server が suppressed-artifact endpoint を公開しない限り、raw source CSV、`normalized_refs` internals、internal detail table、PostgreSQL を直接読んではならない。 |
| Leonard | Lean spec and theorem alignment | RunArtifact traceability と privacy suppression の安定した名称として、`input_refs`、`normalized_refs`、`policy_refs`、`output_refs`、`audit_refs`、`run_artifacts`、`privacy_suppressions` を使う。この DB slice では Lean file を変更しない。 |

## 2. Radomil 契約

Radomil は PostgreSQL を run と job state の system of record として扱う。ただし、Rust domain logic を SQL trigger の中に入れてはならない。Rust engine は ID を生成し、次の順で row を書き込む。

1. `run_records`: RunId と tenant scope を作成する。
2. `input_refs`: Source Archive reference と SHA-256 hash を登録する。
3. `jobs`: analysis job を enqueue し、state/progress を更新する。
4. `normalized_refs`: parsing と normalization 後に normalized dataset reference を登録する。
5. `policy_refs`: suppression/access policy version と threshold を記録する。
6. `issues`: masked data-quality、joinability、privacy、processing issues を書き込む。
7. `privacy_suppressions`: privacy と small-group suppression decisions を書き込む。
8. `output_refs`: privacy-filtered output refs と hashes を登録する。
9. `audit_refs`: masked traceability と approved access refs を記録する。
10. `run_artifacts`: RunArtifact に必要な refs を紐づける。
11. `artifact_manifests` と `report_artifacts`: UI/Pike 用の抑制済み file を登録する。

`jobs.state` values:

```text
queued
running
retry_wait
succeeded
failed
cancel_requested
canceled
```

Worker polling は `(state, available_at, priority DESC)` に対する `idx_jobs_state_available` を使う。Run status は `run_records` で別に更新し、run lifecycle 全体を `jobs` だけから推測してはならない。

### 2.1 Rust PostgreSQL adapter command model

最初の Rust DB slice は `apps/laborlens-rust/src/shared/db.rs` に置く。ここでは command builder だけを公開し、PostgreSQL connection を開かず、driver dependency も追加しない。

現在の command builder:

| builder | SQL 対象 |
| --- | --- |
| `InsertRunRecord` | `INSERT INTO laborlens.run_records` |
| `InsertInputRef` | `INSERT INTO laborlens.input_refs` |
| `InsertJob` | `INSERT INTO laborlens.jobs` |
| `UpdateJobState` | `UPDATE laborlens.jobs` |
| `InsertIssue` | `INSERT INTO laborlens.issues` |
| `InsertRunArtifact` | `INSERT INTO laborlens.run_artifacts` |

各 builder は statement と ordered `SqlParam` value を持つ `SqlCommand` を返す。将来の repository adapter は、ingest や reporting context DTO を変更せず、この param を PostgreSQL driver に map する。

Radomil 接続 path:

```text
IngestRunCommand / ingest result
  -> InsertRunRecord
  -> InsertInputRef per loaded CSV
  -> InsertJob::queued
  -> UpdateJobState for queued/running/succeeded/failed transitions
  -> InsertIssue for schema/data-quality/privacy/process issues
  -> InsertRunArtifact after normalized, policy, output, and audit refs exist
```

この構成により、ingest context から database driver detail を分離しつつ、migration column name と bind order を保つ。

## 3. Pike 契約

Pike は抑制済み artifact だけを受け取る。DB schema は次でこの境界を表す。

- `output_refs.privacy_filtered IS TRUE`
- `artifact_manifests.privacy_filtered IS TRUE`
- `report_artifacts.privacy_filtered IS TRUE`
- `report_artifacts` に raw または sensitive public/report columns を持たせない。

Pike と互換性のある artifact kind:

```text
public_report_model
report_markdown
issues_csv
privacy_suppressions_csv
artifact_manifest
```

Pike はこれらの file から PDF、HTML、print layout、chart を render できる。ただし core analysis の再計算、raw source file への access、internal normalized data の inspect、privacy/safety context の bypass を行ってはならない。

## 4. Leonard 契約

schema name は Lean `RunArtifact` に次のように対応する。

| Lean field | DB source |
| --- | --- |
| `RunArtifact.runId` | `run_artifacts.run_id` |
| `RunArtifact.tenantId` | `run_artifacts.tenant_id` |
| `InputRef.inputId` | `run_artifacts.input_ref_id` -> `input_refs.input_ref_id` |
| `InputRef.fileHashSha256` | `input_refs.file_hash_sha256` |
| `InputRef.schemaVersion` | `input_refs.schema_version` |
| `NormalizedRef.normalizedDatasetId` | `normalized_refs.normalized_dataset_id` |
| `NormalizedRef.normalizationRuleVersion` | `normalized_refs.normalization_rule_version` |
| `NormalizedRef.columnMappingVersion` | `normalized_refs.column_mapping_version` |
| `PolicyRef.suppressionPolicyVersion` | `policy_refs.suppression_policy_version` |
| `PolicyRef.inferenceThresholdK` | `policy_refs.inference_threshold_k` |
| `PolicyRef.ragIndexVersion` | `policy_refs.rag_index_version` |
| `PolicyRef.accessPolicyVersion` | `policy_refs.access_policy_version` |
| `OutputRef.artifactId` | `output_refs.artifact_id` |
| `OutputRef.outputHashSha256` | `output_refs.output_hash_sha256` |
| `AuditRef.actorId` | `audit_refs.actor_id` |
| `AuditRef.actorRole` | `audit_refs.actor_role` |
| `AuditRef.executionReason` | `audit_refs.execution_reason` |
| `AuditRef.accessLogRef` | `audit_refs.access_log_ref` |

Privacy と small-group suppression evidence は、`privacy_suppressions.reason_code`、`threshold_name`、`threshold_value`、`affected_count`、`target_type`、`target_ref` で表す。Lean small-group theorem path では、k-anonymity style minimum group rule による suppression の場合、`reason_code = 'small_group'` と `threshold_name = 'small_group_min_effective_data_count'` を使う。

## 5. table policy

| table | 目的 |
| --- | --- |
| `run_records` | Run registration、tenant scope、lifecycle status、readiness、period、settings。 |
| `input_refs` | Source input reference、hash、file metadata、input schema version。 |
| `normalized_refs` | Normalized dataset reference、versioned normalization rule、row count。 |
| `policy_refs` | Suppression、access、RAG、small-group threshold policy version。 |
| `output_refs` | Privacy-filtered output reference と hash。 |
| `audit_refs` | Masked audit traceability ref。 |
| `run_artifacts` | input、normalized、policy、output、audit refs を横断する RunArtifact traceability join。 |
| `jobs` | PostgreSQL backed job queue と worker progress state。 |
| `issues` | data quality、joinability、privacy、processing failure 用 masked issue record。 |
| `privacy_suppressions` | Suppression decisions と target/threshold evidence。 |
| `artifact_manifests` | artifact lookup 用 suppressed manifest metadata。 |
| `report_artifacts` | Pike と UI consumer 用 suppressed report file metadata。 |

`run_records` 以外のすべての table は、`run_records(run_id)` への必須 `run_id` foreign key を持つ。run をまたいで drift し得る cross-table reference は、composite `(run_id, ref_id)` foreign key を使う。

## 6. raw data 境界

初期 public/report tables は raw CSV rows や sensitive values を保存してはならない。具体的に、`output_refs`、`artifact_manifests`、`report_artifacts` は metadata-only tables とする。

public/report data として禁止するもの:

- employee names
- emails
- individual fatigue values
- sleep durations
- fatigue comments
- free-text raw evidence
- raw CSV row payloads

`issues` は column label を示すため `raw_column_name` を保存してよい。ただし user-facing evidence value には `evidence_value_masked` を使わなければならない。

## 7. 大規模 DB 管理

最初の migration には、primary key、run-scoped unique constraint、foreign key、次の index を含める。

- run-scoped lookup
- job queue polling
- artifact lookup
- issue severity/category filtering
- privacy suppression lookup

Partitioning は文書化するが、初期 migration では実装しない。次の DB slice では、高 volume の materialized normalized row table が存在し、workload evidence が必要性を示した後にだけ partitioning を検討する。reference、job、artifact metadata table は当面 unpartitioned のままとする。

## 8. 検証

repository root から static validation を実行する。

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\validate-db-schema.ps1
```

validator は required table、named primary/unique/check constraint、run_id foreign key、required index、RunArtifact reference column、suppressed artifact boundary、raw-data protection comment、partitioning policy text、Radomil/Pike/Leonard interface note の存在を検査する。
