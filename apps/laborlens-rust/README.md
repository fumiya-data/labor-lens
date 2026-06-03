# laborlens-rust

LaborLens の本番 behavior を置く、単一 Rust modular monolith。

この app/workspace は Radomil の Rust engine 作業開始点である。現時点では小さなひな形と初期 slice を含み、責務の置き場所と context 境界を明示する。

## Context（文脈）

- `ingest`: source CSV references、header mapping、row parsing、schema validation、normalized handoff。
- `workforce_analysis`: readiness、joinability、aggregate metrics、labor-cost と workforce signals。
- `privacy_safety`: privacy suppression、safety boundaries、Lean Phase 1 implementation contracts。
- `reporting`: stable JSON、CSV、Markdown、artifact manifest outputs。
- `guidance`: deterministic rule explanations と将来の local guide AI source boundaries。

各 context は次の module group を持つ。

- `domain.rs`: context 固有の用語、value objects。
- `application.rs`: use cases と contracts。
- `infrastructure.rs`: CSV、DB、filesystem、tool adapter などの接続点。
- `interfaces.rs`: CLI、local server、UI、renderer に渡す DTO。

別 crate へ抽出する理由が ADR で確定するまでは、本番 Rust behavior はこの package 内に保つ。

## 公開レポート artifact 契約

最初の renderer 向け契約は `laborlens.public_report.v1` である。
`reporting` が artifact を作る前に、`privacy_safety` が internal records を filter しなければならない。

Pike/Python 向け top-level JSON field:

```json
{
  "contract_version": "laborlens.public_report.v1",
  "artifact_manifest": {
    "run_id": "run-smoke-001",
    "contract_version": "laborlens.public_report.v1",
    "input_traces": [],
    "policy_trace": {},
    "output_traces": []
  },
  "run_summary": {},
  "issues": [],
  "profile_report": {}
}
```

必須 artifact member:

- `artifact_manifest`: `run_id`、`input_traces`、`policy_trace`、`output_traces` を含む。
- `run_summary`: `run_id`、profile/employee count、suppression count、policy id/version を含む。
- `issues`: public issue notice のみ。suppression notice に個人 raw value を含めてはならない。
- `profile_report`: group-level profile と `suppression_summary`。

プライバシー保証: public JSON は `employee_ref`、`fatigue_value`、`sleep_duration_hours`、`fatigue_comment` を含んではならず、それらの raw value も含んではならない。個人疲労、睡眠、疲労コメントは `PERSONAL_HEALTH_DETAIL_SUPPRESSED` などの suppression metadata だけで表す。

Rust 入口:

- DTO は `reporting::application::build_public_artifacts` で作る。
- Python renderer contract は `reporting::interfaces::to_python_renderer_json` で serialize する。
- 任意の file output は `reporting::infrastructure::write_python_renderer_json` で利用できる。

## ingest smoke

employees / attendance CSV ingest の最小 smoke は次で実行する。

```powershell
cargo run -p laborlens-rust -- --ingest-smoke
```

この smoke は、valid fixture を読み取り、`input_refs`、job state、row counts、issues、run summary を JSON で出力する。現時点では in-memory workflow であり、PostgreSQL への実書き込みは行わない。
