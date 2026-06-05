# サブエージェント進捗ログ

このログは、ユーザーが進行を許可したサブエージェント作業の区切りを記録する。

## 2026-06-03 Fred 区切り

ユーザー許可: Fred がリポジトリ構造の整理を完了した後、ユーザーが進行を許可した。

担当: Fred、リポジトリ改善者。

範囲:

- レイヤードアーキテクチャと独立 Rust crate 前提から、DDD スタイルの bounded context を持つ modular monolith 方針へ変更する。
- Radomil が Rust 実装を開始できる入口を用意する。
- Leonard が進める Lean Phase 1 プロジェクトを維持する。
- Pike が Python レポート出力アプリを接続できる場所を明確にする。
- TDD 方針として、構造変更前にリポジトリ構造検証を定義する。

報告された成果:

- `tools/validate-repository-structure.ps1` を追加した。
- `docs/planning/REPOSITORY-PLAN.md` を modular monolith と bounded context ownership 方針へ更新した。
- `apps/laborlens-rust` を Rust modular monolith の実装入口として追加した。
- 以前の `crates/laborlens-*` placeholder 構成を retired 扱いにした。
- Radomil、Leonard、Pike の作業開始点が分かるよう、README と関連文書を更新した。

メインエージェント確認:

- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\validate-repository-structure.ps1` 成功。
- `cargo check -p laborlens-rust` 成功。
- `lake build` を `lean/` で実行し成功。

次の許可済み作業:

- Radomil が Rust engine 実装を開始する。
- Fred の構造整理完了後、Leonard が Lean 検証実装を開始する。
- Pike は Radomil の抑制済み report artifact contract が利用可能になるまで待機する。

## 2026-06-03 Radomil 区切り

ユーザー許可: Fred 区切り後の進行許可に基づき、Rust engine 実装を開始した。

担当: Radomil、Rust 実装担当者。

範囲:

- `apps/laborlens-rust` 内で、最初の engine slice を TDD で実装する。
- `privacy_safety` context で、内部の個人疲労値、睡眠時間、疲労コメントを公開用出力へ出さない privacy filter を実装する。
- `reporting` context で、抑制済み `PublicReport` から report artifact contract を作る。
- Pike が Python renderer で利用できる `laborlens.public_report.v1` JSON contract を明確にする。

報告された成果:

- 内部データの `employee_ref`、`fatigue_value`、`sleep_duration_hours`、`fatigue_comment` を公開 DTO に出さない privacy filter を実装した。
- `PublicReportArtifacts` に `run_id`、`artifact_manifest`、`run_summary`、`issues`、`profile_report` を追加した。
- Pike 向け JSON contract `laborlens.public_report.v1` を追加した。
- `cargo run -p laborlens-rust` で抑制済み smoke JSON を stdout へ出力できるようにした。
- `apps/laborlens-rust/README.md` に artifact contract の説明を追加した。

メインエージェント確認:

- `cargo test -p laborlens-rust` 成功。3 tests passed。
- `cargo check -p laborlens-rust` 成功。
- `cargo run -p laborlens-rust` 成功。
- 出力 JSON で `contract_version: laborlens.public_report.v1`、`run_summary`、`profile_report`、`artifact_manifest` を確認した。
- 公開 JSON が個人疲労値、睡眠時間、疲労コメントの raw value を出していないことを確認した。

## 2026-06-03 Leonard 区切り

ユーザー許可: Fred 区切り後の進行許可に基づき、Lean 検証実装を開始した。

担当: Leonard、Lean 実装者。

範囲:

- `lean/` 内で、Phase 2 の少人数部署抑制を Lean 仕様として追加する。
- `RunArtifact` traceability theorem を追加する。
- Rust 側には触れず、Lean project 全体が `lake build` で通る状態を保つ。

報告された成果:

- `lean/LaborLens/Spec/AggregationPrivacy.lean` を追加し、`AggregateGroup`、`AggregationResult`、`safeAggregateGroup`、抑制と表示可能性の predicate を定義した。
- `lean/LaborLens/Theorems/AggregationPrivacyTheorems.lean` を追加し、`suppressUnsafeAggregateGroup` を追加した。
- `lean/LaborLens/Theorems/ArtifactTraceabilityTheorems.lean` を追加し、`attachTraceRefsToRunArtifact` を追加した。
- `lean/LaborLens/Core/Artifact.lean` に `traceableRunArtifact` predicate を追加した。
- `lean/LaborLens.lean` に新規 import を追加した。

メインエージェント確認:

- `lake build` を `lean/` で実行し成功。
- 結果は `Build completed successfully (11 jobs).`

## 2026-06-03 Dabian 区切り

ユーザー許可: ユーザーが PostgreSQL 担当として Dabian の追加を指示した。

担当: Dabian、PostgreSQL DB 実装者。

範囲:

- 大規模 DB 管理の初期 schema、migration、検証スクリプトを作る。
- Radomil、Pike、Leonard との interface を決める。
- Rust context や Lean project とは衝突しないよう、DB schema と文書を中心に作業する。

報告された成果:

- `docs/planning/DB-INTERFACES.md` を追加し、Radomil、Pike、Leonard との DB interface を整理した。
- `db/migrations/0001_initial_postgresql_schema.sql` を追加した。
- `tools/validate-db-schema.ps1` を追加した。
- 初期 migration に `run_records`、`input_refs`、`normalized_refs`、`policy_refs`、`output_refs`、`audit_refs`、`run_artifacts`、`jobs`、`issues`、`privacy_suppressions`、`artifact_manifests`、`report_artifacts` を含めた。
- 主キー、`run_id` 外部キー、run-scoped unique constraints、job / artifact / issue / privacy lookup index を追加した。
- Pike 向けの `report_artifacts` と `artifact_manifests` は抑制済み metadata 専用として整理した。
- Lean の `RunArtifact` と DB カラムの対応を記録した。

メインエージェント確認:

- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\validate-db-schema.ps1` 成功。
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\validate-repository-structure.ps1` 成功。

## 2026-06-03 Pike 区切り

ユーザー許可: Radomil の `laborlens.public_report.v1` contract 完了後、Pike が Python report app 作成を開始した。

担当: Pike、Python レポート作成者。

範囲:

- Rust が出力する `laborlens.public_report.v1` JSON contract を読み取る。
- 抑制済み公開情報だけから Markdown report を生成する。
- 禁止 raw key を含む入力を拒否する。
- Rust / Lean / DB 側のファイルには触れない。

報告された成果:

- `reports/report_app/contract.py` を追加し、contract version、必須 field、禁止 raw key の validation を実装した。
- `reports/report_app/renderer.py` を追加し、公開 aggregate JSON から Markdown を生成する処理を実装した。
- `reports/report_app/main.py` を追加し、`--input` file/stdin と `--output` file/dir/stdout に対応した CLI を実装した。
- `reports/report_app/tests/test_report_app.py` を追加した。
- `reports/examples/public_report_v1.json` と `reports/examples/public_report_run-smoke-001.md` を追加した。
- `reports/README.md` に CLI usage、Rust pipe 例、privacy behavior、将来の PDF renderer hook を追記した。

メインエージェント確認:

- `python -m unittest discover reports/report_app/tests` 成功。4 tests OK。
- `cargo run -p laborlens-rust --quiet | python reports/report_app/main.py --input - --output <temp>` 成功。
- 生成 Markdown に `run-smoke-001`、`PERSONAL_HEALTH_DETAIL_SUPPRESSED`、`group:operations` が含まれることを確認した。
- 生成 Markdown に禁止 raw key の出力がないことを確認した。

## 2026-06-03 メイン統合確認

担当: メインエージェント。

範囲:

- Fred、Radomil、Leonard、Dabian、Pike の成果が同じ作業ツリー上で同時に成立するか確認する。

確認結果:

- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\validate-repository-structure.ps1` 成功。
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\validate-db-schema.ps1` 成功。
- `cargo test -p laborlens-rust` 成功。3 tests passed。
- `cargo check -p laborlens-rust` 成功。
- `lake build` を `lean/` で実行し成功。11 jobs。
- `python -m unittest discover reports/report_app/tests` 成功。4 tests OK。
- `cargo run -p laborlens-rust --quiet | python reports/report_app/main.py --input - --output <temp>` 成功。

現在の注意点:

- `git status` では、`docs/` 直下から `docs/product/` と `docs/planning/` への移動が、削除と未追跡ファイルの組み合わせとして表示されている。
- まだ stage / commit は行っていない。

## 2026-06-03 Highest Priority 実装区切り

ユーザー指示: `IMPL-BACKLOG.md` の `Highest Priority` 相当のタスクを、現在いるサブエージェントを動員して実装する。

担当:

- Fred: `IMPLEMENTATION-PLAN.md` 作成。
- Radomil: Rust ingest と job workflow 実装。
- Dabian: PostgreSQL adapter command model 実装。
- Leonard: source preservation の Lean 補助仕様実装。
- Pike: 今回の Highest Priority では直接作業なし。既存 Python report app は統合検証対象として維持。

報告された成果:

- Fred が `docs/planning/IMPLEMENTATION-PLAN.md` を作成し、`docs/planning/WORKFLOW.md` の実装工程を `初版作成済み` に更新した。
- Radomil が employees / attendance の日本語ヘッダー対応 CSV ingest、schema issue、deterministic fingerprint、input ref、in-memory job workflow、`--ingest-smoke` CLI を実装した。
- Dabian が `apps/laborlens-rust/src/shared/db.rs` に PostgreSQL adapter command model を追加し、`InsertRunRecord`、`InsertInputRef`、`InsertJob`、`UpdateJobState`、`InsertIssue`、`InsertRunArtifact` を定義した。
- Dabian が `docs/planning/DB-INTERFACES.md` と `tools/validate-db-schema.ps1` を更新し、adapter/model の静的検査を追加した。
- Leonard が `lean/LaborLens/Spec/SourcePreservation.lean` と `lean/LaborLens/Theorems/SourcePreservationTheorems.lean` を追加し、保存後の input hash 保持と RunArtifact input ref の hash 保持を定理化した。
- `IMPL-BACKLOG.md` の Highest Priority 1-4 を `[done]` として更新した。

メインエージェント確認:

- `cargo test -p laborlens-rust` 成功。11 tests passed。
- `cargo check -p laborlens-rust` 成功。
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\validate-db-schema.ps1` 成功。
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\validate-repository-structure.ps1` 成功。
- `lake build` を `lean/` で実行し成功。13 jobs。
- `python -m unittest discover reports/report_app/tests` 成功。4 tests OK。
- `cargo run -p laborlens-rust -- --ingest-smoke` 成功。
- `cargo run -p laborlens-rust --quiet | python reports/report_app/main.py --input - --output <temp>` 成功。

現在の注意点:

- PostgreSQL adapter は実 DB 接続ではなく、SQL statement と bind parameter order を明示する command model の段階である。
- `--ingest-smoke` は in-memory workflow であり、DB 書き込みまではまだ接続していない。
- 次の自然な実装作業は、Radomil の `IngestWorkflowResult` を Dabian の DB command model に変換する adapter use case と、実 PostgreSQL integration test 方針の決定である。

## 2026-06-03 残バックログ実装区切り

ユーザー指示: `IMPL-BACKLOG.md` の残りの作業を実行する。

担当:

- Radomil: Rust workforce analysis、small-group suppression、reporting file output、performance smoke、operations basics。
- Pike: Python Markdown renderer の readiness / monthly summaries 表示拡張。
- Leonard: Lean workforce / guide safety 仕様と theorem の追加。
- Fred: local server / local UI の初期境界、repository structure validation の更新。
- Dabian: DB schema は今回変更なし。既存 static validation を維持。

報告された成果:

- `workforce_analysis` に readiness、joinability、master issue、grain issue、business check を実装した。
- `privacy_safety` に `SMALL_GROUP_SUPPRESSED` と 10 人未満 group の公開抑制を追加した。
- `reporting` に `write_public_artifact_files` を追加し、`public_report_model.json`、`artifact_manifest.json`、`run_summary.json`、`issues.csv`、`privacy_suppressions.csv` を固定名で出力できるようにした。
- Python renderer に任意の `readiness` と `monthly_summaries` section を追加した。
- Lean に `Spec/Workforce.lean`、`Spec/GuideSafety.lean`、対応 theorem を追加した。
- `laborlens-local-server` crate を追加し、`LocalServer::start_run` で Rust monolith の `run_ingest_workflow` を呼ぶ contract を追加した。
- `apps/laborlens-local-ui` に静的 UI 初期画面を追加した。
- `fixtures/privacy/fatigue.csv`、`fixtures/scale/scale-seed.json`、`tools/generate-scale-fixture.ps1` を追加した。
- `shared::ops` に log masking、fingerprint 検証、Source Archive / Artifact Store path を追加した。
- `IMPL-BACKLOG.md` の 5-15 を `[done]` に更新した。

メインエージェント確認:

- `cargo test` 成功。`laborlens-local-server` 1 test、`laborlens-rust` 22 tests。
- `python -m unittest discover reports/report_app/tests` 成功。5 tests OK。
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\validate-repository-structure.ps1` 成功。
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\validate-db-schema.ps1` 成功。
- `lake build` を `lean/` で実行し成功。17 jobs。
- `cargo run -p laborlens-rust -- --ingest-smoke` 成功。
- `cargo run -p laborlens-rust --quiet | python reports/report_app/main.py --input - --output -` 成功。
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\generate-scale-fixture.ps1 -EmployeeCount 3 -OutputDir tmp/scale-smoke-verify` 成功。

現在の注意点:

- local server は HTTP 実装ではなく、HTTP 化前の Rust contract crate である。
- local UI は静的初期画面であり、実 server endpoint との browser integration は次 slice の対象である。
- PostgreSQL は引き続き static schema / command model 段階であり、実 connection adapter は次 slice の対象である。

## 2026-06-03 モジュール別レビュー区切り

ユーザー指示: repository 全体について各モジュールごとにレビューを行い、テスト可能なものについては性質テストを実施する。レビュー結果は `docs/` に保管する。

担当:

- Radomil: Rust engine review と Rust 性質テスト。
- Fred: repository structure review。
- Leonard: Lean build review。
- Pike: Python report app review と Python 性質テスト。
- Dabian: PostgreSQL schema / command model review。
- メインエージェント: 全体検証とレビュー文書化。

報告された成果:

- `docs/planning/MODULE-REVIEW-2026-06-03.md` を追加し、モジュール別レビュー結果、追加した性質テスト、検証結果、残リスクを記録した。
- `ingest` に、生成 CSV の行数、input ref 件数、fingerprint 安定性を確認する性質テストを追加した。
- `workforce_analysis` に、`labor_cost` 側だけの `missing_employee` issue が employee-attendance joinability を落とさないことを確認する性質テストを追加した。
- 上記テストにより、`workforce_analysis` の joinability 判定が issue の `dataset_id` を見ていない不具合を検出し、修正した。
- `privacy_safety` に、group size と small-group suppression の境界を確認する性質テストを追加した。
- `reports/report_app` に、禁止 raw key が nested path のどこに現れても拒否されることを確認する性質テストを追加した。

メインエージェント確認:

- `cargo test` 成功。`laborlens-local-server` 1 test、`laborlens-rust` 25 tests。
- `python -m unittest discover reports/report_app/tests` 成功。6 tests OK。
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\validate-repository-structure.ps1` 成功。
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\validate-db-schema.ps1` 成功。
- `lake build` を `lean/` で実行し成功。17 jobs。
- `cargo run -p laborlens-rust --quiet | python reports/report_app/main.py --input - --output -` 成功。

現在の注意点:

- `shared::db` は static command model 段階であり、live PostgreSQL adapter と integration test は未実装である。
- `apps/laborlens-local-server` は contract crate 段階であり、HTTP endpoint は未実装である。
- `apps/laborlens-local-ui` は静的初期画面であり、自動 DOM test と browser integration は未実装である。

## 2026-06-03 ユースケース UI / デモ DB seed 区切り

ユーザー指示: DB に 1000 人分の日本人ダミーレコードを入れ、UI ではユースケースに応じたボタンを用意し、そのボタンからデータを読み込めるようにする。

担当:

- Fred: UI 構成、local server HTTP 境界、repository validation 更新。
- Dabian: PostgreSQL demo seed、seed 適用スクリプト、DB static validation 更新。
- Radomil: local server から Rust monolith contract を保ちながらユースケース sample API を追加。
- メインエージェント: PostgreSQL 起動、seed 投入、API/UI 起動確認、全体検証。

報告された成果:

- `db/seeds/0001_demo_japanese_employees.sql` を追加し、`laborlens.demo_employees` に 1000 人分の架空日本人従業員 seed を投入できるようにした。
- `tools/seed-demo-db.ps1` を追加し、migration と demo seed を PostgreSQL へ適用できるようにした。
- `apps/laborlens-local-server` に `GET /api/use-cases` と `GET /api/use-cases/{use_case_id}/sample-data` を追加した。
- local server は `LABORLENS_DEMO_DATABASE_URL` がある場合、実 PostgreSQL の `laborlens.demo_employees` から seed を読み込む。接続できない場合は同じ 1000 人 seed repository に fallback する。
- `apps/laborlens-local-ui` に 14 個のユースケースボタン、DB seed 状態表示、metrics、sample rows、検出結果、次の確認欄を追加した。
- `tools/validate-db-schema.ps1` と `tools/validate-repository-structure.ps1` に demo seed / seed script の静的検証を追加した。

メインエージェント確認:

- 専用 PostgreSQL cluster を `tmp/laborlens-pgdata` に作成し、`127.0.0.1:55432` で起動した。
- `laborlens.demo_employees` に 1000 件投入済み。確認結果: `1000|EMP-0001|EMP-1000`。
- local server を `LABORLENS_DEMO_DATABASE_URL=postgres://laborlens@127.0.0.1:55432/laborlens` で起動し、`http://127.0.0.1:5174/` を配信した。
- `GET /api/use-cases` は 14 件を返した。
- `GET /api/use-cases/uc-01/sample-data` は `employee_count: 1000`、`table_name: laborlens.demo_employees`、sample row を返した。
- `cargo test` 成功。`laborlens-local-server` 5 tests、`laborlens-rust` 25 tests。
- `python -m unittest discover reports/report_app/tests` 成功。6 tests OK。
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\validate-repository-structure.ps1` 成功。
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\validate-db-schema.ps1` 成功。
- `lake build` を `lean/` で実行し成功。17 jobs。
- `cargo run -p laborlens-rust --quiet | python reports/report_app/main.py --input - --output -` 成功。

現在の注意点:

- 既存システム PostgreSQL `127.0.0.1:5432` は起動しているが、認証情報がないため seed 適用はできなかった。今回の実投入は repository local の専用 PostgreSQL cluster `127.0.0.1:55432` に対して行った。
- `/api/runs` の HTTP worker 接続は未実装である。今回の UI 操作はユースケース別 DB sample 読み込みに対応した。

## 2026-06-03 UI 技術選定 / 画面分割設計区切り

ユーザー指示: 将来の local UI について、Ollama AI assistant を組み込む前提で UI 技術スタックを決定し、一画面に情報を詰め込まずページやタブで分割する方針を `docs/` に記録する。

担当:

- メインエージェント: 現行 UI、アーキテクチャ文書、外部設計文書を確認し、設計決定として文書化。

報告された成果:

- `docs/product/EXTERNAL-DESIGN.md` に `UI 実装スタック` を追加した。
- 将来の local UI は `Vite + React + TypeScript`、`shadcn/ui`、`TanStack Table`、`TanStack Query`、`assistant-ui` を使う方針にした。
- Ollama assistant は UI から直接呼ばず、必ず Local Server API 経由で接続する方針を明記した。
- `docs/product/ARCHITECTURE.md` の基本構成に UI 実装スタックと Guide AI 接続方針を追加した。
- `docs/product/ARCHITECTURE.md` に `ADR-ARCH-009` として UI 技術スタック採用を追加した。
- `docs/product/EXTERNAL-DESIGN.md` に `画面分割方針` を追加した。
- local UI は一画面に全情報を詰め込まず、`実行`、`結果`、`不備一覧`、`レポート`、`再確認`、`ガイド`、`設定` に分ける方針にした。
- 詳細表示は、作業目的に応じてページ遷移、タブ、ドロワー、モーダル、別タブまたは別ウィンドウを使い分ける方針にした。
- `docs/product/EXTERNAL-DESIGN.md` の未決事項対応表に `UI-STACK-001` と `UI-LAYOUT-001` を追加した。

現在の進捗状況:

- 現行 UI はまだ `apps/laborlens-local-ui` の静的 HTML / JavaScript / CSS であり、React 化は未実装である。
- ユースケース別 DB sample 読み込みは実装済みで、14 個のユースケースボタンから 1000 人 seed のサンプルを表示できる。
- CSV run 本線は未接続であり、`/api/runs` は HTTP worker 接続前の状態である。
- PostgreSQL は demo seed の実投入確認済みだが、本番 run の state / artifact / job を書き込む live adapter は未実装である。
- Rust core、privacy/safety、reporting、Python Markdown renderer、Lean safety spec、DB schema / command model は初期接続済みである。
- UI 設計上は、情報密度を分ける方針と AI assistant の接続境界が決定済みになった。

残りの作業:

- `apps/laborlens-local-ui` を Vite + React + TypeScript へ移行する。
- shadcn/ui の初期設定、レイアウト shell、左サイドバー、ページルーティングを追加する。
- TanStack Query で Local Server API の run、progress、use case、artifact、report、guide response を取得する境界を作る。
- TanStack Table で issues、readiness、artifact、run history、修正依頼一覧を表示する。
- assistant-ui を組み込み、Guide AI / RuleExplanation を表示する。ただし Ollama へは UI から直接接続しない。
- Local Server API に `/api/runs`、job progress、artifact listing、report fetch、guide assistant endpoint を接続する。
- IngestWorkflowResult を live PostgreSQL adapter に接続し、run state、input refs、jobs、issues、artifacts を transaction で保存する。
- Source Archive / Artifact Store の実 file adapter を追加し、原本 CSV と抑制後成果物を分離して保存する。
- UI browser integration test、DOM test、API integration smoke を追加する。
- 大量 CSV を UI スレッドで処理しないこと、進捗が取得できること、抑制前データが UI / Guide AI に出ないことをテストする。

現在の注意点:

- UI 技術選定は設計決定として保存済みだが、package.json、Vite config、React component 実装はまだ作っていない。
- assistant-ui と Ollama の組み込みは、Local Server API の安全境界、許可済み文脈、ログマスキング、プロンプト注入対策と同時に設計する必要がある。
- 画面分割方針により、最初の React 実装では「全情報ダッシュボード」ではなく、実行サマリーから各ページへ誘導する構成を優先する。

## 2026-06-03 Tauri desktop app 配布方針区切り

ユーザー指示: UI は Tauri + React で desktop app 化し、installer 付きの単体配布を目標にする。

担当:

- メインエージェント: 既存の UI 技術選定、アーキテクチャ、README、進捗ログへ Tauri 配布方針を反映。

報告された成果:

- `docs/product/EXTERNAL-DESIGN.md` の UI 実装スタックに `desktop shell: Tauri` を追加した。
- `docs/product/EXTERNAL-DESIGN.md` に `Desktop 配布方針` を追加した。
- 最終的な local UI は、Tauri + React により Windows desktop app として installer 付きで配布する方針にした。
- 初期開発では Vite dev server と Rust local server を個別に起動してよいが、本番配布では Tauri installer から起動できる単体 desktop app を目標にした。
- `docs/product/ARCHITECTURE.md` の基本構成に `配布形態` と Tauri を含む UI 実装スタックを追加した。
- 論理コンポーネントに `Tauri Desktop Shell` を追加し、React UI と Local Server API の間の desktop shell として整理した。
- 責務分担表に `Tauri Desktop Shell` を追加し、Tauri は配布、file dialog、アプリ設定、local server / worker 起動管理を担当し、業務ロジックや AI 安全境界は担当しないと明記した。
- `docs/product/ARCHITECTURE.md` に `ADR-ARCH-010` として Tauri installer 配布採用を追加した。
- `README.md` の初期本番スタックに、Tauri + React desktop app と installer 付き配布方針を追加した。
- PostgreSQL、Ollama、初期モデル `qwen3:8b` を installer に同梱する方針を `README.md`、`EXTERNAL-DESIGN.md`、`ARCHITECTURE.md`、`OPERATIONS.md` に追記した。
- `docs/product/ARCHITECTURE.md` に `ADR-ARCH-011` として PostgreSQL / Ollama 同梱配布採用を追加した。
- `docs/product/OPERATIONS.md` に `Installer 同梱コンポーネント運用` を追加し、PostgreSQL managed local cluster、Ollama / model 同梱、更新、fallback、確認事項を整理した。

現在の進捗状況:

- 現行 UI はまだ静的 HTML / JavaScript / CSS であり、Tauri project は未作成である。
- React 化、Vite config、TypeScript、shadcn/ui、TanStack、assistant-ui の package 導入は未実装である。
- Tauri は設計上の最終配布形態として決定済みだが、installer、sidecar、local server 起動管理、worker 起動管理は未実装である。
- AI assistant は引き続き Local Server API 経由で Ollama に接続する方針であり、Tauri から Ollama を直接呼ぶ方針ではない。
- PostgreSQL と Ollama は installer 同梱方針に変更済みである。ただし runtime の再配布条件、installer size、更新手順、port conflict、uninstall 時の user data 扱いは実装前確認が必要である。

残りの作業:

- `apps/laborlens-local-ui` を Tauri + Vite + React + TypeScript 構成へ移行する。
- Tauri の `src-tauri`、desktop app metadata、dev / build command、React build output の設定を追加する。
- local server / worker を Tauri sidecar にするか、同一 Rust binary 内の command boundary にするかを実装前に決める。
- installer に含める対象を整理する。候補は Tauri shell、React build asset、local server / worker binary、migration helper、initial directory helper、起動 script。
- PostgreSQL を installer に同梱し、managed local cluster として初期化する実装を追加する。
- Ollama runtime と初期モデル `qwen3:8b` を installer に同梱し、初回起動時に availability を確認する実装を追加する。
- PostgreSQL data directory、Ollama model directory、Source Archive、Artifact Store を app 管理下の local data directory に分離して配置する。
- PostgreSQL / Ollama / model の version 固定、更新確認、backup、rollback、uninstall 時の user data 保持方針を実装する。
- Tauri app から `/api/runs`、progress、artifact、guide assistant endpoint へ接続する integration smoke を追加する。
- installer 作成後、Windows 環境で初回起動、DB 初期化、Ollama 接続確認、CSV run、artifact 表示まで確認する。

現在の注意点:

- 「単体配布」は利用者の入口として installer 付き desktop app にするという配布方針であり、PostgreSQL と Ollama も同梱する方針である。サイズ、ライセンス、運用、アップデート方法は実装前に確認する。
- Tauri を入れても、UI が抑制前データ、PostgreSQL、Ollama、RAG index へ直接接続してよいわけではない。安全境界は引き続き Local Server API 側に置く。
