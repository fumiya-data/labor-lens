# TBD（未決事項）

日付: 2026-06-03
出典: `docs/` 現行ドラフト

## 概要

本日確認した 8 件の実質未決テーマは、すべて設計判断としてクローズした。

出典 entry 単位では 9 件だったが、`DD-OPEN-003` と `OPEN-BR-002` は同じテーマを指していたため、統合して 8 テーマとして扱った。

## クローズ済み決定

| No | ID | 決定 | 反映先 |
| --- | --- | --- | --- |
| 1 | `DD-OPEN-003` / `OPEN-BR-002` | 長時間労働候補は法令違反判定ではなく、確認候補を出すルールエンジンとして扱う。業種特例、変形労働時間制、管理監督者などは `EmployeeApplicabilityProfile` と `WorkingTimeRuleSet` に分離し、ハードコードしない。疲労リスク確認候補は別軸で残す。 | `docs/product/BUSINESS-RULES.md`, `docs/product/DATA-DESIGN.md` |
| 2 | `DD-OPEN-005` | 保存時暗号化とログマスキングは初期から必須仕様にする。ローカル DB 領域、Source Archive、Artifact Store、バックアップを暗号化対象にし、ログには原本 CSV 行、氏名、未マスク従業員 ID、詳細勤務実績を出さない。 | `docs/product/ARCHITECTURE.md`, `docs/product/OPERATIONS.md` |
| 3 | `OPEN-BR-003` | 必要人数データは `Store x Department x Role x TimeSlot` を標準粒度にする。 | `docs/product/BUSINESS-RULES.md`, `docs/product/DATA-DESIGN.md` |
| 4 | `REPO-OPEN-001` | 初期実装の system of record は PostgreSQL とする。DuckDB は後段の分析補助候補に留める。 | `docs/product/ARCHITECTURE.md`, `docs/planning/REPOSITORY-PLAN.md` |
| 5 | `REPO-OPEN-002` | 最初の data-ingest slice は Rust `csv` crate による狭い CSV pipeline にする。Polars は後段候補にする。 | `docs/product/ARCHITECTURE.md`, `docs/planning/REPOSITORY-PLAN.md` |
| 6 | `REPO-OPEN-003` | 10000人 x 3年分の scale fixture は固定 seed の合成データ生成器で作る。 | `docs/product/DATA-DESIGN.md`, `docs/planning/REPOSITORY-PLAN.md` |
| 7 | `REPO-OPEN-004` | Rust core は JSON、CSV、Markdown までを生成し、PDF は post-processing layer に残す。 | `docs/product/DATA-DESIGN.md`, `docs/planning/REPOSITORY-PLAN.md` |
| 8 | `REPO-OPEN-005` | local RAG は最初の local UI milestone から外す。初期 UI では deterministic な `RuleExplanation` を表示する。 | `docs/product/ARCHITECTURE.md`, `docs/product/OPERATIONS.md`, `docs/planning/REPOSITORY-PLAN.md` |

## 現在の未決件数

- 実質未決テーマ: 0 件
- 出典 entry 単位の未決: 0 件

## 注記

長時間労働、疲労、必要人数、RAG、暗号化、ログマスキングはいずれも、最終判断や断定ではなく、確認候補、説明、追跡、抑制のための設計境界として扱う。
