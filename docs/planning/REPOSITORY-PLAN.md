# 本番リポジトリ計画

Date: 2026-06-01
Status: draft
Aligned with: `docs/product/REQUIREMENTS.md`

## 目的

本番リポジトリは、単発の prototype delivery ではなく、10000人規模・3年分の労務データを扱うローカル実行型の業務アプリケーションを支える必要がある。

製品の基本形は、Web サービスではなく、利用者のPCまたは社内端末上で動く次の構成とする。

- ローカル UI
- ローカルサーバー
- ローカルDB
- バックグラウンドジョブ
- Rust 製の検査・集計エンジン
- レポート生成
- safety and privacy boundaries

## 提案するトップレベル構成

```text
labor-lens/
  apps/
    laborlens-local-ui/
      src/
    laborlens-local-server/
      src/
  crates/
    laborlens-domain/
    laborlens-ingest/
    laborlens-storage/
    laborlens-engine/
    laborlens-jobs/
    laborlens-safety/
    laborlens-report/
    laborlens-observability/
    laborlens-cli/
  docs/
    adr/
    planning/
    product/
    user-guides/
  fixtures/
    valid/
    invalid/
    privacy/
    performance/
    scale/
  reports/
    examples/
  tools/
  Cargo.toml
  package.json
```

最初の implementation slice で必要になるまでは、この layout 全体を一度に作らない。上記の directories は、本番設計上の ownership map である。

## レイヤー責務

| レイヤー | 責務 | 持つべきではない責務 |
| --- | --- | --- |
| `laborlens-domain` | core value objects、dataset identifiers、issue codes、units、safety concepts | CSV parsing、UI、file I/O、DB access |
| `laborlens-ingest` | CSV reading、日本語 header mapping、schema validation、raw-to-normalized conversion | business recommendations、UI state |
| `laborlens-storage` | ローカルDB schema、repositories、run artifact metadata、migration | domain rules、CSV parsing |
| `laborlens-engine` | join readiness、aggregate metrics、labor-hour calculations、fatigue-risk inputs | UI state、HTTP routing、DB migration |
| `laborlens-jobs` | CSV 取り込み、検査、集計、レポート生成を非同期 job として実行する | 画面表示、domain model 定義 |
| `laborlens-safety` | Safety Boundary checks、privacy suppression、legal/health/fairness guardrails | sales optimization as a sole objective |
| `laborlens-report` | stable report model、JSON/CSV/Markdown exports、run artifact contracts | interactive dashboard widgets |
| `laborlens-observability` | run IDs、structured logs、timing、diagnostic metrics | domain rules |
| `laborlens-cli` | 同じ core logic を使う developer and batch entry point | separate business logic |
| `apps/laborlens-local-server` | ローカル API、job orchestration、local DB connection、artifact serving | core validation rules |
| `apps/laborlens-local-ui` | run start、progress view、dataset summary、issue list、report details | core validation rules、heavy calculation |

## 依存方向

基本の dependency direction は次の通りにする。

```text
local-ui
  -> local-server
      -> jobs / report / engine / ingest / safety / observability / storage
          -> domain

cli
  -> jobs / report / engine / ingest / safety / observability / storage
      -> domain
```

domain crate は infrastructure、UI、file-system code、DB code に依存してはいけない。ローカルサーバーと CLI は同じ production engine と storage contract を呼び出し、test coverage と behavior を一致させる。

## 最初の実装範囲

CLI-only ではなく、最初からローカルサーバーとローカルDBの contract を薄く作る。ただし、UI は最小限に抑え、先に data contract と job contract を安定させる。

1. typed identifiers、dates、issue severity、stable check-code types を持つ `laborlens-domain` を作成する。
2. run、dataset、issue、artifact を保存する `laborlens-storage` の最小 schema を作成する。
3. employees and attendance CSV loading 用の `laborlens-ingest` を作成する。
4. CSV 取り込みと検査を job として実行する `laborlens-jobs` を作成する。
5. `run_summary.json`、`issues.csv`、`profile_report.json` contracts を持つ `laborlens-report` を追加する。
6. 最初の executable boundary として `laborlens-cli` と `apps/laborlens-local-server` を追加する。
7. local server が job progress と generated artifacts を返せるようになってから、最小 UI を追加する。

これにより、初期の data-contract decisions が UI work に引きずられるのを避ける。

## 製品アーキテクチャ原則

- CSV validation を import side effect ではなく product feature として扱う。
- raw input、normalized data、issues、aggregates、reports をローカルDB上で区別する。
- stable issue codes を user-facing support contract として使う。
- privacy suppression は UI work の前に data-model level で testable にする。
- business recommendations と data-quality findings を分離する。
- heavy CSV processing は UI thread ではなく background job として扱う。
- repeatable reviews and regression tests のため、deterministic output ordering を優先する。
- 10000人規模・3年分の検証データを扱えるよう、storage と processing は streaming または chunking を前提に設計する。

## 初期テスト戦略

| テスト領域 | 目的 |
| --- | --- |
| Unit tests | value objects、parsing、issue-code generation、privacy suppression |
| Storage tests | local DB schema、migration、repository behavior |
| Job tests | 取り込み、検査、集計、レポート生成が job として完了すること |
| Fixture tests | valid and invalid CSV datasets |
| Golden output tests | `run_summary.json`、`issues.csv`、report JSON shape |
| Determinism tests | 同じ input and config が equivalent artifacts を生成すること |
| Raw input protection tests | execution 前後で input hashes が変わらないこと |
| Performance smoke | 10000人 × 3年分の勤怠データを想定した scale fixture を処理できること |
| UI smoke | local UI が core logic を再計算せず、local server から progress と reports を読み込むこと |

## 未決事項

- ローカルDBに SQLite を使うか、DuckDB など分析向きの選択肢を使うか。
- 最初の data-ingest slice で `polars`、`csv`、またはより狭い Rust CSV pipeline のどれを使うか。
- 10000人 × 3年分の scale fixture を実データ風に生成する方法。
- report Markdown/PDF rendering を最初から Rust に置くか、post-processing layer に残すか。
- local RAG を最初の local UI milestone に含めるか、data-quality workflow が安定するまで後回しにするか。
