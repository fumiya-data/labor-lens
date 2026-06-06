# LaborLens

LaborLens は、勤怠 CSV、従業員マスタ、人件費、売上、休暇情報などをローカル環境で取り込み、給与計算前の不備確認、修正依頼、再確認、労務確認レポートを支援する業務アプリケーションです。

現在は、Rust のローカルサーバー、React/Vite/Tauri のローカル UI、PostgreSQL 向けコマンドモデル、1000 人 seed による勤怠レビュー画面、成果物配信 API の実装を進めています。外部公開 Web サービスではなく、利用者の PC または社内端末で動くローカルアプリケーションを前提にしています。

このリポジトリは、実務利用に近い労務分析アプリを、要求定義、業務ルール、データ設計、アーキテクチャ、検証、実装の順に積み上げていくための開発リポジトリです。検証用データは架空データに限定し、原本 CSV の保護、プライバシー抑制、データ品質 issue と業務確認ポイントの分離を重視します。

## 架空データのみを使用

このリポジトリに含まれる seed、fixture、サンプル CSV、レポート例、画面表示用データは、すべて検証用の架空データです。

実在する個人、従業員、顧客、企業、店舗、医療・健康情報、勤怠実績、給与情報、ストレスチェック結果は含めません。公開用の成果物、README、スクリーンショット、デモ UI でも、実データを使わない方針です。

## 現在の状態

このリポジトリには、計画文書に加えて、初期のローカル実行基盤と業務 UI が含まれています。`docs/planning/WORKFLOW.md` に従い、要求仕様、用語、業務ルール、受け入れ基準、データ設計、アーキテクチャ設計、Lean 仕様化計画を管理しています。

Rust 側は `apps/laborlens-rust` を単一のアプリケーション兼 workspace の入口にし、複数の独立 crate ではなく、bounded context ごとの module で整理します。Local Server は `apps/laborlens-local-server`、UI は `apps/laborlens-local-ui` に分離しています。

## 製品の方向性

LaborLens は、単なる可視化ツールではなく、健康、法令遵守、プライバシー、公平性を強い制約として扱う業務アプリケーションです。

本番構成では、外部公開する Web サービスではなく、利用者の PC または社内端末で動作するローカルアプリケーションを想定しています。ローカルサーバー、ローカル DB、バックグラウンドジョブ、ローカル UI を組み合わせて動作させます。

本番実装では、試作品コードをそのまま移植しません。試作品は、要件、スキーマ、検証ケース、レポート形状、配布上の教訓を得るための証拠として扱い、Rust を中心に再設計します。

## 初期本番スタック

| 領域 | 方針 |
| --- | --- |
| Rust モノリス | `apps/laborlens-rust`。ingest、workforce analysis、privacy/safety、reporting、guidance の bounded context を module として持つ |
| ローカル UI | Tauri + Vite + React + TypeScript で desktop app 化し、ローカルサーバーに接続して実行指示、進捗確認、結果閲覧を行う。コアロジックは再実装しない |
| 配布形態 | 最終配布は Tauri installer 付き desktop app とする。初期開発時は browser + local server 起動を許容する |
| ローカルサーバー | CSV 取り込み、ジョブ管理、レポート配信、ローカル DB 接続を担当する薄い境界 |
| ローカル DB | PostgreSQL を installer に同梱し、原本メタデータ、正規化データ、issue、集計結果、実行履歴を保存する |
| バックグラウンドジョブ | 重い CSV 検査、集計、レポート生成を画面操作から分離して実行する |
| コアロジック | Rust で実装する |
| CSV 取り込み・検証 | Rust で実装する |
| 指標・計算エンジン | Rust で実装する |
| レポートモデル・機械可読出力 | Rust で実装する |
| 任意のリッチレポート描画 | 必要な場合のみ、`reports/` と接続する薄い Python renderer を使う |
| ローカル使い方ガイド AI / RAG | Ollama と初期モデル `qwen3:8b` を installer に同梱する方針とし、許可済み文書と抑制後レポートだけを参照する |

## 計画文書

- `docs/planning/WORKFLOW.md`: ウォーターフォール型の開発工程目次と、各工程文書への参照。
- `docs/product/USE-CASES.md`: 想定利用者と主要ユースケース。
- `docs/product/REQUIREMENTS.md`: 製品目的、対象範囲、機能要求、安全境界、非機能要求。
- `docs/product/GLOSSARY.md`: 要求、設計、実装で意味をそろえる用語定義。
- `docs/product/BUSINESS-RULES.md`: issue 判定、集計可否、抑制条件、業務上の分類。
- `docs/product/ACCEPTANCE-CRITERIA.md`: 要求 ID と検証可能な完成条件。
- `docs/product/DATA-DESIGN.md`: CSV、正規化データ、ローカル DB、成果物 JSON/CSV の構造。
- `docs/product/ARCHITECTURE.md`: ローカルサーバー、DB、ジョブ、UI、ガイド AI の責務境界。
- `docs/product/LEAN-SPEC-PLANNING.md`: Lean で検証する安全性、信頼性、データ分類の計画。
- `docs/planning/REPOSITORY-PLAN.md`: modular monolith を前提にした repository structure、bounded context、実装開始点。

## 実装開始点

- Radomil: `apps/laborlens-rust` から Rust エンジンを開始します。最初は `ingest`、`privacy/safety`、`reporting` の契約を小さく作ります。
- Leonard: `lean/` と `docs/product/LEAN-SPEC-PLANNING.md` を正として Phase 1 privacy spec を進めます。
- Pike: `reports/README.md` を入口に、Rust が生成する抑制済み JSON / CSV / Markdown / artifact manifest を Python report app または renderer に接続します。

## 構造検証

リポジトリ構造と docs 参照方針は次で検証します。

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\validate-repository-structure.ps1
```


## 設計上の制約

### データ保全

- 原本 CSV ファイルは変更しない。
- 顧客の日本語 CSV ヘッダーは、安定した内部フィールド名へマッピングする。
- 生成されるすべての成果物には、安定した run identifier を含める。

### データ粒度

- 売上データは、日付と時間帯の粒度を扱えるようにする。
- 人件費データは、従業員別月次、部門別月次、雇用区分別月次など、複数の粒度を扱えるようにする。
- シフト、休暇、共有予定データは、必要な粒度と安全境界を確認したうえで扱う。

### プライバシーと説明責任

- 個人の疲労値は、平文レポートやユーザー向け画面に表示しない。
- データ品質の指摘と、業務上の推奨を分離する。

### 性能と実行方式

- 10,000 人規模・3 年分の勤怠データを扱える設計を目標にする。
- 重い処理は、ローカルサーバー側のバックグラウンドジョブとして実行する。
