# 試作からの移行

Date: 2026-06-01
Status: 下書き
Aligned with: `docs/product/REQUIREMENTS.md`

## 役割

引き継ぎ対象:

- dataset scope
- CSV header mapping requirements
- issue-code ideas
- report artifact names and user-facing concepts
- privacy suppression behavior
- verification で見つかった acceptance gaps
- installer and delivery lessons

再設計対象:

- type boundaries
- error model
- configuration schema
- local DB schema
- background job model
- data normalization contracts
- deterministic output ordering
- test fixture strategy
- local UI workflow

## 試作品の保存すべき対象

| 試作品の証拠 | 本番での使い道 |
| --- | --- |
| `docs/product/REQUIREMENTS.md` | 現在の主要求仕様。10000人規模、3年分、ローカルサーバー、ローカルDB、バックグラウンド処理、ローカル使い方ガイドAIの前提 |
| `docs/product/LEAN-SPEC-PLANNING.md` | Lean で表現する safety and invariant の計画 |
| `prototype/PROTOTYPE-SPEC.md` | validation rules と artifact contract candidates |
| `prototype/productization/spec-contract-alignment-report.md` | sales grain、header mapping、labor cost grain に関する決定 |
| `prototype/productization/release/reports/verification-report.md` | acceptance gaps と release risks |
| `prototype/productization/app/out/*` | example outputs と regression-test inspiration |
| `data/raw/*` | privacy と suitability review 後の fixture design source |

実行固有の risk を production tests または release gates に変換するため、`PROTOTYPE-EXECUTION-FINDINGS.md` も参照する。

## 本番で必要な改善

### 1. 入力契約

本番では、CSV contracts を versioned schemas として定義する必要がある。顧客向けの日本語 headers と内部英語 field names は config で分離する。

schema layer は次を明示的に support する。

- date and time-slot sales data
- employee monthly labor costs
- department monthly labor costs
- employment-type monthly labor costs
- missing employee ID を、常に fatal parse error とするのではなく join-readiness issue として扱うこと

### 2. ローカルDBと正規化契約

10000人規模・3年分のデータを扱うため、CSV から直接レポートを作るだけの構成にはしない。

本番では、少なくとも次をローカルDB上で区別する。

- 原本入力の metadata
- 正規化済みデータ
- schema validation results
- issues
- aggregates
- generated artifacts
- run history

### 3. バックグラウンドジョブ

CSV の読み込み、検査、集計、レポート生成は UI 操作から分離し、background job として実行する。

job は次を持つ。

- run identifier
- status
- started/finished timestamps
- progress
- error summary
- artifact references

### 4. プライバシー境界

privacy behavior は、report rendering の前、および UI state が作られる前に適用する。個人の fatigue values は、すでに伏せ字または安全に aggregated されていない限り、user-facing report models に渡してはいけない。

### 5. 検証カバレッジ

本番リポジトリでは、試作品で部分的にしか実行されなかった fixture coverage が必要である。

- duplicate employee IDs
- unknown employee IDs
- `clock_out <= clock_in`
- negative sales amounts
- missing required columns
- invalid labor cost grain
- small health-related groups
- raw input file immutability
- deterministic repeated runs
- local DB migration
- background job recovery
- 10000人 × 3年分の scale fixtures

### 6. 配布モデル

最初の製品 milestone は installer 作業に依存させない。開発用の local server 起動と CLI により、まず core contracts、storage contracts、job contracts を証明する。

Packaging は次の条件を満たした後に扱う。

- core validation output が安定している
- local DB schema と migration が安定している
- background job が progress と result を返せる
- local UI が reports を読み込める
- application logs の場所が予測可能である
- start and stop behavior を Windows で test できる

## 移行手順

1. Rust types で production artifact contract を固定する。
2. local DB schema で run、dataset、issue、artifact metadata を固定する。
3. prototype sample scenarios を fixtures に変換する。
4. CSV ingest and schema mapping を実装する。
5. GUI なしで background job と report output を実装する。
6. deterministic and raw-input-protection tests を追加する。
7. 10000人 × 3年分の scale fixture と performance smoke を追加する。
8. generated reports を閲覧・説明する local UI を追加する。
9. product documentation と report semantics が安定した後に local assistant/RAG を追加する。
