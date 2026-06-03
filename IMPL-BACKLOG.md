# 実装バックログ

日付: 2026-06-03
状態: draft

このバックログは、初期のリポジトリ構造、Rust 公開レポート契約、Lean 検査、PostgreSQL schema、Python Markdown renderer を作成した後に残っている実装作業を記録する。

## 最優先

1. [done] `IMPLEMENTATION-PLAN.md` を作成する
   - 実装順序、所有境界、TDD 対象、レビュー単位を定義する。
   - `docs/planning/WORKFLOW.md` では作成済みに更新済み。

2. [done] Rust ingest を実装する
   - 作業領域: `apps/laborlens-rust/src/contexts/ingest/`
   - employees と attendance の最小 CSV 読み取りを追加する。
   - 日本語 header mapping を追加する。
   - schema issue を生成する。
   - source hash と input ref を生成する。
   - fixture test から開始した。

3. [done] PostgreSQL adapters を実装する
   - Rust コードを `db/migrations/0001_initial_postgresql_schema.sql` に接続する。
   - `run_records`、`input_refs`、`jobs`、`issues`、`run_artifacts` のコマンドモデルを追加した。
   - 初期 DB テスト形状は、実 PostgreSQL 接続ではなく repository コマンドモデルテストと静的 SQL 検証とした。

4. [done] job workflow を実装する
   - run creation、input registration、queued/running/succeeded/failed state を扱う。
   - progress と failure reason を保持する。
   - local server / UI より先に CLI smoke から開始した。

## Rust エンジン

5. [done] workforce analysis を実装する
   - 準備状態: `ready`、`partial`、`blocked`。
   - 結合可否: grain mismatch と employee ID を持たない labor-cost data。
   - マスタ確認: missing employee、retired employee、department mismatch。
   - `issue` と `business_check` を分離する。
   - `apps/laborlens-rust/src/contexts/workforce_analysis/` に readiness、joinability、master issue、grain issue、business check を実装した。

6. [done] privacy and safety を拡張する
   - 現在の slice は personal fatigue value、sleep duration、fatigue comment を抑制する。
   - 次の slice では small-group suppression を実装する。
   - Lean の `safeAggregateGroup` と `suppressUnsafeAggregateGroup` に対応させる。
   - `SMALL_GROUP_SUPPRESSED` と 10 人未満 group の公開抑制を追加した。

7. [done] reporting を拡張する
   - 現在の slice は `laborlens.public_report.v1` JSON と Python Markdown 接続を提供する。
   - `issues.csv`、`privacy_suppressions.csv`、`artifact_manifest.json`、`run_summary.json` の実ファイル出力を追加する。
   - golden output test を追加する。
   - `write_public_artifact_files` と CSV/JSON 出力 test を追加した。

## Python レポート

8. [done] Markdown renderer を拡張する
   - 現在の renderer は最小公開レポート契約を扱う。
   - readiness、issues、privacy suppressions、monthly summaries 用のレポート固有 template を追加する。
   - PDF、HTML、chart は後続 renderer hook として残す。
   - Python は raw CSV や PostgreSQL を直接読んではならない。
   - 任意の `readiness` と `monthly_summaries` section を public JSON から描画する test を追加した。

## Lean 検証

9. [done] Lean phases を継続する
   - Source preservation: original input hash が変わらないこと。
   - Joinability: employee ID を持たない labor-cost data は personal attendance に join できないこと。
   - Master check: missing employee が master issue を生成すること。
   - Issue/report separation。
   - Guide safety。
   - `Workforce` と `GuideSafety` の Lean spec/theorem を追加した。

## ローカルサーバーと UI

10. [done] local server を初期化する
    - 作業領域: `apps/laborlens-local-server/`
    - Rust modular monolith の契約を呼び出す薄い API として保つ。
    - run creation、job progress、artifact listing から開始する。
    - `laborlens-local-server` crate を追加し、`LocalServer::start_run` で Rust monolith workflow を呼ぶ contract test を追加した。

11. [done] local UI を初期化する
    - 作業領域: `apps/laborlens-local-ui/`
    - CSV selection、run start、progress display、artifact list、Markdown report display から開始する。
    - UI にコアロジックを再実装してはならない。
    - 静的 UI の `index.html`、`src/app.js`、`src/styles.css` を追加した。

## 検証と運用

12. [done] fixtures を生成する
    - 作業領域: `fixtures/valid`、`fixtures/invalid`、`fixtures/privacy`、`fixtures/scale`
    - 最小 employees、attendance、fatigue fixture から開始する。
    - 10000 人、3 年分の scale data 用固定 seed generator を追加する。
    - `fixtures/privacy/fatigue.csv`、`fixtures/scale/scale-seed.json`、`tools/generate-scale-fixture.ps1` を追加した。

13. [done] performance smoke tests を追加する
    - streaming / chunking の前提を検証する。
    - full scale fixture の前に 10k、100k、1M row の段階で開始する。
    - 10,000 employees / attendance row の Rust ingest smoke test を追加した。

14. [done] security and operations basics を実装する
    - log masking を追加する。
    - raw value が log に出ない test を追加する。
    - Source Archive と Artifact Store の場所を定義する。
    - processing 前後で input hashes を検証する。
    - `shared::ops` に log masking、fingerprint 検証、Source Archive / Artifact Store path を追加した。

15. [done] Git staging を整理する
    - `docs/` は `docs/product/` と `docs/planning/` に再編された。
    - 現在の `git status` では、旧ファイルの削除と新しい未追跡ファイルの組み合わせとして表示される。
    - commit 前に diff を確認し、移動として意図的に stage する。
    - 前回 commit `1a67745` で初期 scaffold と日本語 docs は push 済み。今回の残 backlog 実装は、この更新後に別 commit として stage する。

## 推奨される次の slice

今回の残 backlog 実装後、次の実装 slice は次が自然である。

```text
IngestWorkflowResult
  -> 実 PostgreSQL connection adapter
  -> local server HTTP endpoint
  -> local UI integration smoke
```

これにより、in-memory workflow、PostgreSQL state、local server、local UI を実 runtime で接続できる。
