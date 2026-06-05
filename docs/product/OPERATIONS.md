# LaborLens 運用設計

日付: 2026-06-03
状態: draft
出典: `ARCHITECTURE.md`, `DATA-DESIGN.md`, `BUSINESS-RULES.md`

## 1. 目的

この文書は、LaborLens のローカル実行における鍵管理、保存時暗号化、ログマスキング、バックアップ、監査ログの運用境界を定義する。

LaborLens は労務データ、勤怠データ、従業員情報、疲労関連の労務リスク指標を扱うため、保存時暗号化とログマスキングを初期から必須仕様にする。

## 2. 保存時暗号化

| 対象 | 方針 |
| --- | --- |
| PostgreSQL data directory | OS ディスク暗号化またはアプリケーションが管理する暗号化領域に配置する |
| Source Archive | 原本 CSV と入力参照を暗号化対象にする |
| Artifact Store | 内部確認用成果物、共有用成果物、UI JSON を暗号化対象にする |
| Backup | 暗号化済みバックアップだけを許可する |
| 一時ファイル | 抑制前データを含む一時ファイルは暗号化領域に置き、処理完了後に削除する |

暗号鍵は、OS の資格情報ストアまたは利用者指定の秘密情報に置く。PostgreSQL、設定ファイル、ログ、成果物に暗号鍵を平文保存してはならない。

## 3. ログマスキング

ログは allowlist 方式とする。ログに出してよいものは、`RunId`、`JobId`、処理段階、件数、`IssueCode`、失敗分類、処理時間など、個人を直接特定しない診断情報に限定する。

ログに出してはならないもの:

| 項目 | 方針 |
| --- | --- |
| 原本 CSV 行 | 出力禁止 |
| 氏名 | 出力禁止 |
| 未マスクの従業員 ID | 出力禁止。必要な場合は HMAC 仮名化 |
| メールアドレス、電話番号、住所 | 出力禁止 |
| 詳細な勤務実績 | 出力禁止。必要な場合は件数、月単位、時間帯などへ粗化 |
| 個人疲労値、睡眠時間、疲労コメント | 出力禁止 |
| 自由記述原文 | 出力禁止 |
| 少人数集計 | 抑制または件数非表示 |

HMAC 仮名化を使う場合、HMAC 秘密鍵はログ、DB、設定ファイルに平文保存しない。

## 4. 監査ログ

抑制前データへのアクセスは Default Deny とする。許可されたアクセスでは、次を監査ログに残す。

| 項目 | 内容 |
| --- | --- |
| actor | 利用者またはシステム主体 |
| role | システム管理者、監査担当、データ保護責任者、限定運用担当など |
| purpose | 明示された目的 |
| ticket_id | 承認または作業チケット |
| run_id / dataset_id | 対象データ |
| scope | 必要最小限の範囲 |
| access_started_at / access_ended_at | 期間 |
| approval_ref | 承認参照 |
| result | 許可、拒否、失敗 |

監査ログは追記型またはハッシュ連鎖など、改ざん困難な形式で保持する。

## 5. バックアップ

バックアップは暗号化済みで作成し、抑制前データ、抑制後成果物、監査ログを区別できるようにする。復元時は、復元先が暗号化領域であることを確認する。

バックアップから復元したデータも、通常 UI、RAG、Guide AI、一般管理者画面から抑制前データを直接参照できない。

## 6. RAG 運用

local RAG は初期 local UI milestone には含めない。初期 UI では `RuleExplanation` により、IssueCode、参照ルール、必要な入力データ、推奨される人手確認を deterministic に表示する。

RAG を導入する場合、検索対象は承認済み、版管理済み、抑制後情報だけに限定する。未承認文書、抑制前データ、個人別データ、監査ログ、下書き文書、Slack、メール、チケット原文はインデックスへ投入してはならない。

## 7. Installer 同梱コンポーネント運用

本番配布では、Tauri installer に PostgreSQL、Ollama、初期モデル `qwen3:8b` を同梱する方針とする。利用者が PostgreSQL や Ollama を別途手動インストールしなくても、初回起動時に local services として利用できる状態を目標にする。

### 7.1 PostgreSQL 同梱運用

PostgreSQL は managed local cluster として扱う。

| 項目 | 方針 |
| --- | --- |
| 初期化 | 初回起動時に app 管理下の local data directory へ data directory を作成する |
| migration | installer 付属の migration を起動時または明示操作で適用する |
| 接続 | UI は直接接続しない。Local Server API / worker だけが接続する |
| 起動停止 | Tauri shell または local server 起動管理が service lifecycle を扱う |
| data directory | OS ディスク暗号化またはアプリケーション管理の暗号化領域に置く |
| backup | 暗号化済み backup だけを許可する |
| update | PostgreSQL runtime 更新時は schema migration、backup、rollback 手順を確認する |

既存 PostgreSQL が利用者環境に存在しても、LaborLens は既定では同梱 managed cluster を使う。外部 PostgreSQL 接続は、運用設計で明示的に許可した場合だけ設定可能にする。

### 7.2 Ollama / model 同梱運用

Ollama と初期モデル `qwen3:8b` は、ガイド AI 用の local runtime として同梱する方針とする。

| 項目 | 方針 |
| --- | --- |
| 初期化 | 初回起動時に Ollama runtime と model availability を確認する |
| model | 初期モデルは `qwen3:8b` とする |
| 接続 | UI は直接接続しない。Local Server API の Guide AI gateway だけが接続する |
| 参照対象 | 承認済み、版管理済み、プライバシー安全確認済み文書と抑制後情報だけ |
| model directory | app 管理下の local data directory に分離して配置する |
| update | model 更新は明示操作とし、model version、更新日時、評価結果を記録する |
| fallback | model が利用できない場合は、AI assistant を無効化し、RuleExplanation を表示する |

Ollama を同梱しても、抑制前データ、原本 CSV、個人別データ、監査ログ、下書き文書を model prompt や RAG index に入れてはならない。

### 7.3 同梱時の確認事項

同梱配布を実装する前に、次を確認する。

- PostgreSQL runtime の再配布条件、version 固定、更新手順
- Ollama runtime と model の再配布条件、model size、更新手順
- installer size と初回起動時間
- data directory、model directory、backup directory の配置
- Windows 環境での service lifecycle、port conflict、permission
- uninstall 時に application binary と user data を分けて扱うこと

## 8. 未決事項

現時点で、本書で保留する主要未決事項はない。保持期間、端末別暗号化方式、鍵ローテーション周期は、導入先の運用ポリシーに合わせて設定値として管理する。
