# モジュール別レビュー 2026-06-03

この文書は、リポジトリ全体をモジュール単位でレビューし、テスト可能な箇所へ性質テストを追加した結果を記録する。

## レビュー体制

| 観点 | 担当 | 確認範囲 |
| --- | --- | --- |
| Rust engine | Radomil | `apps/laborlens-rust` の bounded context、公開 artifact、shared module |
| リポジトリ構造 | Fred | modular monolith 方針、文書配置、検証 script |
| Lean 検証 | Leonard | `lean/` の仕様、定理、`lake build` |
| Python レポート | Pike | `reports/report_app` の contract validation と Markdown renderer |
| PostgreSQL | Dabian | `db/migrations`、`shared::db`、DB interface 文書、schema validation |
| 統合管理 | メインエージェント | Rust、Python、Lean、DB、local app の接続確認 |

## 総評

現時点のリポジトリは、`apps/laborlens-rust` を中心にした modular monolith として成立している。`ingest`、`workforce_analysis`、`privacy_safety`、`reporting` は bounded context として分離され、`shared::db` と `shared::ops` は cross-cutting concern を担っている。Python report app は Rust の `laborlens.public_report.v1` contract に接続でき、Lean project は安全性仕様の軽量な形式化を維持している。

レビュー中に、`workforce_analysis` の結合可能性判定で、`labor_cost` 側だけの `missing_employee` issue が `employee_attendance` の結合可能性まで落としてしまう不具合を検出した。この問題は性質テストで再現し、`attendance_by_employee` の issue だけを `employee_attendance` 判定へ反映するよう修正した。

## 修正済み不具合

| 箇所 | 内容 | 対応 |
| --- | --- | --- |
| `apps/laborlens-rust/src/contexts/workforce_analysis/application.rs:123` | `missing_employee` issue の dataset を見ずに `employee_attendance_joinable` を落としていた | `dataset_id == Some("attendance_by_employee")` の場合だけ employee-attendance 結合不可とするよう修正 |
| `apps/laborlens-rust/src/contexts/workforce_analysis/application.rs:336` | 上記の回帰を検出する性質テストがなかった | `property_employee_attendance_joinability_ignores_labor_cost_only_issues` を追加 |

## 追加した性質テスト

| モジュール | テスト | 性質 |
| --- | --- | --- |
| `ingest` | `property_generated_csv_counts_match_row_counts_and_fingerprints_are_stable` | 0 から 32 件の生成 CSV で、取込行数、input ref 件数、fingerprint の安定性が入力件数と一致する |
| `workforce_analysis` | `property_employee_attendance_joinability_ignores_labor_cost_only_issues` | `labor_cost` だけに master issue がある場合でも、employee-attendance の joinability は落ちない |
| `privacy_safety` | `property_group_visibility_matches_minimum_safe_group_size` | group size が `MINIMUM_SAFE_AGGREGATE_GROUP_SIZE` 以上の場合だけ公開 profile が残り、1 から 9 人では small-group suppression が出る |
| `reports/report_app` | `test_property_forbidden_raw_keys_are_rejected_at_any_nested_path` | 禁止 raw key は nested path のどこへ注入されても `PrivacyViolation` になる |

## モジュール別レビュー結果

| モジュール | 結果 | テスト状況 | 残リスク |
| --- | --- | --- | --- |
| `apps/laborlens-rust/src/contexts/ingest` | 日本語 CSV header、必須列、input ref、fingerprint、job workflow は現在の slice と整合している | 既存 unit test と追加性質テストを実行 | CSV parser は簡易実装であり、quote、改行入り field、大規模 streaming は次の検討対象 |
| `apps/laborlens-rust/src/contexts/workforce_analysis` | readiness、master issue、business check、joinability の責務は bounded context 内に収まっている | 既存 unit test と追加性質テストを実行 | labor cost と attendance の月次粒度対応は初期 model のため、実データ投入後に粒度 issue を拡張する余地がある |
| `apps/laborlens-rust/src/contexts/privacy_safety` | 個人健康関連詳細と少人数 group の抑制境界は公開 artifact 前に適用されている | 既存 unit test と追加性質テストを実行 | 複数属性の組合せによる差分推測リスクは、現時点では Lean 仕様と文書側で管理している |
| `apps/laborlens-rust/src/contexts/reporting` | `PublicReportArtifacts` と file output は固定名 artifact contract を作れている | Rust unit test と Python pipe integration で確認 | PDF renderer や artifact store への永続化は未接続 |
| `apps/laborlens-rust/src/shared/db.rs` | PostgreSQL command model は run、input、job、issue、artifact の insert/update 境界を持つ | Rust unit test と DB schema static validation を実行 | 実 PostgreSQL 接続、transaction、placeholder 数と bind 数の網羅的性質テストは未実装 |
| `apps/laborlens-rust/src/shared/ops.rs` | source archive、artifact store path、log masking、fingerprint 検証の基礎がある | Rust unit test を実行 | 実 file store への write/read adapter は未実装 |
| `apps/laborlens-local-server` | local app 入口として Rust monolith の ingest workflow を呼ぶ contract crate になっている | `cargo test` で contract test を実行 | HTTP server ではなく、endpoint、auth、progress API は未実装 |
| `apps/laborlens-local-ui` | 静的 UI 初期画面として workflow と API 境界を示している | 現時点では自動テストなし | DOM test、browser integration、実 server endpoint 接続が未実装 |
| `reports/report_app` | Rust の公開 JSON contract を検証し、Markdown report を生成できる | Python unit test、追加性質テスト、Rust pipe integration を実行 | PDF 出力、i18n、artifact store 連携は未実装 |
| `lean` | privacy、aggregation、source preservation、workforce、guide safety、artifact traceability の仕様と theorem が build 可能 | `lake build` を実行 | Rust 実装との機械的対応付けは未接続 |
| `db/migrations` | 初期 PostgreSQL schema は run、input、policy、output、audit、job、issue、suppression、report artifact を保持する構成 | `tools/validate-db-schema.ps1` を実行 | migration runner と live DB integration test は未実装 |
| `tools` | repository structure、DB schema、scale fixture の検証入口がある | `validate-repository-structure.ps1` と `validate-db-schema.ps1` を実行 | CI での常時実行は未接続 |
| `fixtures` | valid、invalid、privacy、scale の初期 fixture がある | Rust ingest test と生成 script の対象 | 大規模 fixture の実測 benchmark は未実施 |
| `docs` | product、planning、ADR、user guide の配置は整理済み | 文書配置を確認 | 実装進行に合わせた ADR 追加が今後必要 |

## 検証結果

| コマンド | 結果 |
| --- | --- |
| `cargo test` | 成功。`laborlens-local-server` 1 test、`laborlens-rust` 25 tests |
| `python -m unittest discover reports/report_app/tests` | 成功。6 tests |
| `powershell -NoProfile -ExecutionPolicy Bypass -File tools\validate-repository-structure.ps1` | 成功 |
| `powershell -NoProfile -ExecutionPolicy Bypass -File tools\validate-db-schema.ps1` | 成功 |
| `lake build` | 成功。17 jobs |
| `cargo run -p laborlens-rust --quiet | python reports/report_app/main.py --input - --output -` | 成功。公開 Markdown report を stdout に生成 |

## 次に残るレビュー指摘

| 優先度 | 指摘 | 推奨対応 |
| --- | --- | --- |
| 高 | `shared::db` は static command model であり、live PostgreSQL adapter ではない | transaction boundary、connection pool、migration runner、integration test を追加する |
| 高 | local server は HTTP endpoint ではない | `start_run` contract を HTTP route に接続し、job progress API を追加する |
| 中 | local UI は自動テストなし | DOM test と browser integration test を追加する |
| 中 | Rust 実装と Lean 仕様の対応が手動確認に留まる | Rust model から Lean 仕様へ写像する traceability 文書または検証補助を追加する |
| 中 | DB command model の placeholder と bind parameter の対応は個別 test 中心 | SQL placeholder 数、bind name、必須 parameter を横断する性質テストを追加する |
