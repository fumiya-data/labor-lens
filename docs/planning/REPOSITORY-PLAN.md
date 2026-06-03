# 本番リポジトリ計画

日付: 2026-06-01
更新日: 2026-06-03
状態: draft
整合先: `docs/product/REQUIREMENTS.md`

## 目的

本番リポジトリは、単発の prototype delivery ではなく、10000人規模・3年分の労務データを扱うローカル実行型の業務アプリケーションを支える必要がある。

製品の基本形は、Web サービスではなく、利用者のPCまたは社内端末上で動く次の構成とする。

- ローカル UI
- ローカルサーバー
- ローカル DB
- バックグラウンドジョブ
- Rust 製の検査・集計・抑制・レポートモデル生成
- 任意の Python レポート描画
- safety and privacy boundary

この計画では、Rust 側の本番構成を複数の independent crates に分けない。初期実装は `apps/laborlens-rust` の中に置く modular monolith とし、DDD スタイルの bounded context を Rust module と責務メモで整理する。

## トップレベル構成

```text
labor-lens/
  Cargo.toml
  apps/
    laborlens-rust/
      Cargo.toml
      README.md
      src/
        main.rs
        shared/
          mod.rs
        contexts/
          mod.rs
          ingest/
            domain.rs
            application.rs
            infrastructure.rs
            interfaces.rs
          workforce_analysis/
            domain.rs
            application.rs
            infrastructure.rs
            interfaces.rs
          privacy_safety/
            domain.rs
            application.rs
            infrastructure.rs
            interfaces.rs
          reporting/
            domain.rs
            application.rs
            infrastructure.rs
            interfaces.rs
          guidance/
            domain.rs
            application.rs
            infrastructure.rs
            interfaces.rs
    laborlens-local-server/
    laborlens-local-ui/
  crates/
    README.md
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
  lean/
  reports/
    examples/
    generated/
  tools/
```

`Cargo.toml` は単一 workspace の入口である。初期 member は `apps/laborlens-rust` のみとし、業務ロジックを分散した crate ownership map にしない。将来 crate 抽出が必要になった場合は、安定した境界、テスト、ADR がそろった後に行う。

`apps/laborlens-local-server` と `apps/laborlens-local-ui` は runnable boundary の placeholder として残す。core behavior は `apps/laborlens-rust` から呼び出す。UI や HTTP routing に業務判断を重複実装しない。

## Modular Monolith 方針

Rust 本番コードは、単一 app/workspace の中で bounded context を分ける。これは、依存関係を crate 分割で固定する前に、業務上の意味、入力成果物、公開用出力、安全境界を同じ実行単位で検証しやすくするためである。

DDD 用語は必要最小限にする。LaborLens では次の bounded context を初期単位とする。

| Bounded context | 主な責務 | 先に実装しないこと |
| --- | --- | --- |
| `ingest` | 原本 CSV 参照、列名マッピング、行単位読取、schema validation、正規化前後の参照作成 | 業務推奨、UI 表示、PDF 描画 |
| `workforce_analysis` | workforce analysis。勤怠、人件費、売上、疲労リスク入力の結合可否、集計、readiness、確認ポイント抽出 | 医療判断、人事評価、外部共有可否の最終判断 |
| `privacy/safety` | 個人疲労値非表示、少人数部署抑制、公開用出力の安全境界、Lean 仕様との対応 | 売上最大化だけを目的にした推奨 |
| `reporting` | `run_summary.json`、`issues.csv`、`public_report_model.json`、`report.md` などの機械可読成果物 | PDF の印刷レイアウト、見た目調整 |
| `guidance` | deterministic な `RuleExplanation`、将来の local guide AI が参照できる文書境界 | 抑制前データ、監査ログ、未承認文書の検索 |

各 context は、同じ Rust package の中に次の module group を持つ。

| Module group | 意味 |
| --- | --- |
| `domain` | その context の業務上の型、値オブジェクト、判定語彙 |
| `application` | use case、workflow、context 間の呼び出し contract |
| `infrastructure` | CSV、DB、filesystem、外部 tool などの接続点 |
| `interfaces` | CLI、local server、UI、report renderer に渡す入出力 DTO |

これは layered architecture を採用するという意味ではない。module group は各 bounded context の内側を読みやすくするための整理であり、責務の中心は context 単位に置く。

## shared kernel

`apps/laborlens-rust/src/shared/` には、複数 context にまたがって意味が変わってはいけない最小型だけを置く。

- `RunId`
- `TenantId`
- `DatasetId`
- `EmployeeId`
- `IssueCode`
- `IssueSeverity`
- artifact refs

ここを便利置き場にしない。業務判断が増えた場合は、まず該当 context の `domain` に置く。

## 依存と呼び出し方針

- context 間の直接呼び出しは、原則として `application` contract 経由にする。
- `domain` module は CSV、DB、filesystem、HTTP、PDF tool に依存しない。
- `infrastructure` module は external detail を閉じ込め、業務判断を持たない。
- `interfaces` module は外部境界の形を保つ。内部 model をそのまま公開用出力として返さない。
- `privacy/safety` context は、reporting、guidance、UI 表示前の共通 gate として扱う。

## 最初の実装範囲

Radomil は `apps/laborlens-rust` から Rust engine 実装を開始する。

1. `cargo check -p laborlens-rust` が通る状態を維持する。
2. `shared` に `RunId`、typed identifiers、issue severity、artifact refs の最小型を追加する。
3. `ingest` に employees / attendance の狭い CSV 読取と header mapping を追加する。
4. `privacy/safety` に Lean Phase 1 と対応する公開用出力 contract を追加する。
5. `reporting` に `run_summary.json`、`issues.csv`、`public_report_model.json`、`report.md` の contract を追加する。
6. `workforce_analysis` は、readiness と結合可否を先に実装し、複雑な最適化は後回しにする。
7. local server と UI は、上記 contract が安定してから薄く接続する。

CLI-only に閉じない。ただし、最初から UI を厚く作らず、data contract、job contract、公開用出力 contract を先に安定させる。

## Lean との接続

Leonard は既存の `lean/` と `docs/product/LEAN-SPEC-PLANNING.md` を正として進める。

初期 Phase 1 は privacy boundary であり、次の既存構造を維持する。

```text
lean/
  LaborLens/
    Core/
    Spec/
      Privacy.lean
    Theorems/
      PrivacyTheorems.lean
```

Rust 側は Lean の module 名や証明構造を直接コピーしない。ただし、Lean で検証した性質は `privacy/safety` と `reporting` の実装 contract に反映する。特に、公開用出力は抑制済み model だけを受け取り、個人疲労値、睡眠時間、疲労コメントを直接表現できないようにする。

## reports と Python 接続

Pike は `reports/README.md` を入口に、Python report app または report rendering tooling を接続する。

Rust monolith は core result として JSON、CSV、Markdown、artifact manifest までを生成する。Python 側はそれを入力として PDF、HTML、印刷向け layout、図表描画を行う薄い renderer にする。

Python 側で次をしてはいけない。

- 原本 CSV を再解釈して core result を作り直す。
- privacy/safety を通っていない内部データを読む。
- issue と業務確認ポイントを再分類する。
- `RunId` と input hash を落とした成果物を生成する。

## 製品アーキテクチャ原則

- CSV validation を import side effect ではなく product feature として扱う。
- raw input、normalized data、issues、aggregates、reports をローカル DB 上で区別する。
- stable issue codes を user-facing support contract として使う。
- privacy suppression は UI work の前に data-model level で testable にする。
- business recommendations と data-quality findings を分離する。
- heavy CSV processing は UI thread ではなく background job として扱う。
- repeatable reviews and regression tests のため、deterministic output ordering を優先する。
- 10000人規模・3年分の検証データを扱えるよう、storage と processing は streaming または chunking を前提に設計する。

## 初期テスト戦略

| テスト領域 | 目的 |
| --- | --- |
| Repository structure validation | modular monolith、bounded context、docs reference、Lean / reports 接続を検査する |
| Unit tests | value objects、parsing、issue-code generation、privacy suppression |
| Storage tests | local DB schema、migration、repository behavior |
| Job tests | 取り込み、検査、集計、レポート生成が job として完了すること |
| Fixture tests | valid and invalid CSV datasets |
| Golden output tests | `run_summary.json`、`issues.csv`、report JSON shape |
| Determinism tests | 同じ input and config が equivalent artifacts を生成すること |
| Raw input protection tests | execution 前後で input hashes が変わらないこと |
| Performance smoke | 10000人 x 3年分の勤怠データを想定した scale fixture を処理できること |
| UI smoke | local UI が core logic を再計算せず、local server から progress と reports を読み込むこと |

## 設計判断

未決事項は、初期実装の段階導入方針として次の通り閉じる。

| ID | 決定 | 理由 |
| --- | --- | --- |
| `REPO-OPEN-001` | System of record は PostgreSQL とする。DuckDB は後段の分析補助候補に留める。 | RunId、正規化データ、issue、監査、成果物メタデータ、ジョブ状態を制約とトランザクションで扱う必要があるため。 |
| `REPO-OPEN-002` | 最初の data-ingest slice は Rust `csv` package による狭い CSV pipeline にする。Polars は後段候補。 | 初期目的は高速 DataFrame 集計ではなく、汚れた CSV の行単位読取、標準列名解決、検査、issue 化を安定させることだから。 |
| `REPO-OPEN-003` | 10000人 x 3年分の scale fixture は固定 seed の合成データ生成器で作る。 | 実データ風の分布、少人数部署、長時間労働、打刻漏れ、変形労働時間制、管理監督者候補などを再現可能に混ぜるため。 |
| `REPO-OPEN-004` | Rust monolith は JSON、CSV、Markdown、artifact manifest までを生成し、PDF は Python などの thin renderer に残す。 | core logic は検査、集計、抑制、再現性に集中し、PDF のレイアウト、フォント、印刷調整を後段で変更しやすくするため。 |
| `REPO-OPEN-005` | local RAG は最初の local UI milestone から外す。 | 先に data-quality workflow、IssueCode、RuleExplanation、評価トレースを安定させ、RAG は検索対象、更新条件、評価データが固まってから導入するため。 |
| `REPO-OPEN-006` | independent crate scaffold は retired とする。 | 初期段階では crate 分割より、同一 app 内の context boundary とテストで業務意味を固定する方が安全だから。 |

初期 UI では、RAG の代わりに deterministic な `RuleExplanation` を表示する。
