# DATA-DESIGN.md

Date: 2026-06-02
Status: draft
Project: 労務コンパス / LaborLens
Scope: CSV、正規化データ、ローカル DB、成果物 JSON/CSV の構造定義

Related:

- `REQUIREMENTS_BRUSHED.md`
- `GLOSSARY.md`
- `BUSINESS-RULES.md`
- `ACCEPTANCE-CRITERIA.md`
- `EXTERNAL-DESIGN.md`
- `ARCHITECTURE.md`
- `TEST-PLAN.md`
- `LEAN-SPEC-PLANNING.md`

## 0. この文書の位置づけ

この文書は、労務コンパス / LaborLens が扱うデータ構造を固定するためのデータ設計書である。対象は、入力 CSV、原本保存後のメタデータ、正規化データ、検査結果、結合可否、プライバシー抑制、ローカル DB の論理スキーマ、成果物 JSON/CSV である。

この文書では、画面レイアウト、ジョブ実行方式、認証、保存時暗号化、バックグラウンドキュー実装、PDF や Markdown の見た目は定義しない。それらは `EXTERNAL-DESIGN.md`、`ARCHITECTURE.md`、`OPERATIONS.md`、`TEST-PLAN.md` で扱う。

この文書は実装に近い設計を含むが、DB エンジン固有の最適化は固定しない。ローカル DB は SQLite、DuckDB、PostgreSQL embedded 相当のいずれでも実現できるよう、論理スキーマと制約を中心に定義する。

## 1. 設計原則

| 原則 | データ設計上のルール |
| --- | --- |
| 原本保護 | 取り込んだ原本 CSV は変更しない。加工、補正、列名変換、型変換は派生データとして保持する。 |
| 状態分離 | 原本、保存済み原本、解析済み CSV、正規化データ、検査済みデータ、結合判定、分析用データ、公開用出力を区別する。 |
| 再生成可能性 | 正規化データ、検査結果、集計結果、成果物は、原本 CSV、設定、スキーマ定義、コードバージョンから再生成できる状態にする。 |
| トレーサビリティ | すべての正規化行、issue、集計、成果物は `run_id`、`source_file_id`、必要に応じて `source_row_number` または集計条件を持つ。 |
| issue と確認ポイントの分離 | CSV・データ品質の不備と、労務・経理・店舗運営上の確認材料を混在させない。 |
| 粒度の明示 | 従業員別、部署別、店舗別、日別、月別、時間帯別などの粒度を明示し、推定結合を禁止する。 |
| プライバシー境界 | 抑制前の内部データと、画面・ファイル・ガイド AI が参照できる公開用出力を分離する。 |
| ローカル優先 | 原本、正規化データ、issue、集計結果、成果物メタデータ、実行履歴をローカル DB に保存する。 |
| スキーマバージョン管理 | 入力 CSV プロファイル、正規化スキーマ、成果物スキーマは独立してバージョンを持つ。 |

## 2. 共通命名規則と型

### 2.1 命名規則

| 対象 | 規則 | 例 |
| --- | --- | --- |
| DB テーブル | `snake_case`、複数形または用途名 | `source_files`, `norm_attendance_records` |
| DB カラム | `snake_case` | `employee_id`, `source_row_number` |
| JSON フィールド | `snake_case` | `run_id`, `generated_at` |
| CSV ヘッダー | `snake_case` | `issue_code`, `readiness_status` |
| enum 値 | 小文字 `snake_case` | `employee_daily`, `not_joinable` |
| ID 文字列 | 接頭辞 + ULID または UUID | `run_01HX...`, `src_01HX...` |

### 2.2 共通 ID

| ID | 型 | 形式 | 用途 |
| --- | --- | --- | --- |
| `run_id` | string | `run_` + ULID 推奨 | 実行単位 |
| `source_file_id` | string | `src_` + ULID 推奨 | 取り込まれた CSV ファイル単位 |
| `dataset_id` | string | `ds_` + ULID 推奨 | 同一種別・同一実行内の論理データセット |
| `artifact_id` | string | `art_` + ULID 推奨 | 出力成果物単位 |
| `issue_id` | string | `iss_` + ULID 推奨 | issue 単位 |
| `employee_id` | string | 入力元 ID を正規化 | 従業員識別子 |
| `department_id` | string | 入力元 ID を正規化 | 部署識別子 |
| `store_id` | string | 入力元 ID を正規化 | 店舗識別子 |

`employee_id`、`department_id`、`store_id` は原則として入力元の業務 ID を保持する。ただし、公開用成果物に個人識別リスクがある場合は、表示前に `display_employee_id`、`masked_employee_id`、または集計単位へ変換する。

### 2.3 共通型

| 型 | 表現 | 備考 |
| --- | --- | --- |
| `date` | `YYYY-MM-DD` | 事業日、勤務日、売上日など |
| `time` | `HH:MM[:SS]` | ローカル時刻。日跨ぎは別フィールドで表す。 |
| `datetime` | ISO 8601 | `2026-06-02T10:20:30+09:00` または UTC `Z` |
| `month` | `YYYY-MM` | 月次人件費、月次労務など |
| `decimal` | 文字列または固定小数 | 金額、時間、比率。浮動小数の丸め誤差を避ける。 |
| `integer` | 64-bit signed | 行番号、件数、分数など |
| `boolean` | `true` / `false` | CSV では `true` / `false` を標準形とする。 |
| `json` | JSON object / array | DB では JSON 文字列または JSON 型を利用する。 |

### 2.4 Null と空文字

| 入力 | 正規化後 | 意味 |
| --- | --- | --- |
| 空セル | `null` | 値なし |
| 空白のみ | `null` | 値なし。原本値は raw 側に保持する。 |
| `0` | `0` | 数値ゼロ。欠損ではない。 |
| `N/A`, `-`, `不明` | `null` + issue 候補 | データ種別と列ごとのルールで扱う。 |

## 3. データ状態モデル

データは次の状態を通る。前段のデータを破壊せず、後段は再生成可能な派生データとして扱う。

```mermaid
flowchart LR
    A[RawCsvDataset] --> B[StoredSourceDataset]
    B --> C[ParsedDataset]
    C --> D[NormalizedDataset]
    D --> E[ValidatedDataset]
    E --> F[JoinAssessment]
    F --> G[AnalysisDataset]
    G --> H[PrivacyFilteredReport]
```

| 状態 | 内容 | 保存先 | 変更可否 |
| --- | --- | --- | --- |
| `RawCsvDataset` | 利用者が指定した原本 CSV | ファイルシステム | 変更不可 |
| `StoredSourceDataset` | 原本保存後、ハッシュ、文字コード、区切り文字などを持つ状態 | DB + ファイルシステム | 変更不可 |
| `ParsedDataset` | CSV として行・列へ読めた状態 | DB | 再生成可能 |
| `NormalizedDataset` | 列名、ID、日付、時刻、数値、粒度を内部形式へ揃えた状態 | DB | 再生成可能 |
| `ValidatedDataset` | スキーマ確認、データ品質検査、マスタ照合を通した状態 | DB | 再生成可能 |
| `JoinAssessment` | データ間の結合可否と理由を保持する状態 | DB | 再生成可能 |
| `AnalysisDataset` | 集計、確認ポイント、レポート下書きに使う状態 | DB | 再生成可能 |
| `PrivacyFilteredReport` | 安全境界を通したユーザー向け出力 | DB + ファイル | 生成結果として保存 |

## 4. 入力 CSV 設計

### 4.1 入力データ種別

`dataset_kind` は次の値を標準とする。

| `dataset_kind` | 日本語名 | 主用途 | 標準粒度 |
| --- | --- | --- | --- |
| `employee_master` | 従業員マスタ | 従業員 ID、所属、雇用区分、在籍状態の照合 | 従業員 × 有効期間 |
| `attendance` | 勤怠データ | 打刻漏れ、時刻逆転、重複、労働時間、休暇の確認 | 従業員 × 日、または従業員 × 打刻 |
| `labor_cost` | 人件費データ | 部署別、店舗別、従業員別、雇用区分別の費用確認 | 入力に依存 |
| `sales` | 売上データ | 店舗、日付、時間帯別の忙しさ確認 | 店舗 × 日、または店舗 × 時間帯 |
| `shift` | シフトデータ | 予定人員、実績人員、欠員対応の確認 | 従業員 × シフト |
| `fatigue` | 疲労関連データ | 部署単位または店舗単位の負荷傾向確認 | 従業員 × 日、または部署/店舗 × 期間 |
| `leave` | 休暇情報 | 有給取得状況、取得率、残日数の確認 | 従業員 × 年度、または従業員 × 休暇日 |
| `share_candidate` | 共有予定データ | 外部共有前の識別子、健康関連情報、推測リスク確認 | 任意 |

### 4.2 CSV 読み込み設定

各 CSV は、ファイルごとに次の読み込み設定を持つ。

| 項目 | 型 | 必須 | 内容 |
| --- | --- | --- | --- |
| `source_file_id` | string | yes | 取り込みファイル ID |
| `run_id` | string | yes | 実行 ID |
| `dataset_kind` | enum | yes | 入力データ種別 |
| `original_filename` | string | yes | 利用者が指定したファイル名 |
| `stored_path` | string | yes | ローカル保存後の相対パス |
| `input_hash_sha256` | string | yes | 原本 CSV の SHA-256 |
| `size_bytes` | integer | yes | ファイルサイズ |
| `encoding` | string | yes | `utf-8`, `utf-8-sig`, `shift_jis`, `cp932` など |
| `delimiter` | string | yes | `,`, `\t` など |
| `quote_char` | string | no | 通常 `"` |
| `has_header` | boolean | yes | ヘッダー有無 |
| `detected_row_count` | integer | no | 読み取り可能行数 |
| `detected_column_count` | integer | no | 読み取り可能列数 |
| `ingested_at` | datetime | yes | 取り込み時刻 |
| `schema_profile_version` | string | yes | 入力プロファイル定義バージョン |

### 4.3 共通 CSV 取り扱いルール

- 原本 CSV は読み取り専用として保存し、正規化や修正を原本へ書き戻さない。
- 文字コード、区切り文字、ヘッダー有無は自動検出してよいが、検出結果を記録する。
- 列名の別名許容範囲は `column_aliases` として管理し、どの原本列がどの標準列へ対応したかを保存する。
- 行番号はヘッダー行を除いたデータ行番号ではなく、原則として原本ファイル上の 1 始まり行番号を `source_row_number` として保存する。
- パースできない行は捨てず、`raw_csv_rows` と `issues` に保存する。
- 自動補正した値は、原本値、正規化値、補正理由を分離して保存する。

## 5. 入力 CSV 列定義

以下の列名は標準形である。実際の入力 CSV では別名を許容してよいが、別名マッピングは DB に保存する。

### 5.1 `employee_master.csv`

| 列 | 必須 | 型 | 内容 | 公開可否 |
| --- | --- | --- | --- | --- |
| `employee_id` | yes | string | 従業員 ID | 条件付き |
| `employee_name` | no | string | 氏名 | 内部のみ |
| `department_id` | yes | string | 所属部署 ID | 公開可 |
| `department_name` | no | string | 所属部署名 | 公開可。ただし少人数部署は抑制対象 |
| `store_id` | no | string | 所属店舗 ID | 公開可 |
| `store_name` | no | string | 所属店舗名 | 公開可。ただし少人数店舗は抑制対象 |
| `employment_type` | yes | string | 正社員、契約、パート等 | 公開可 |
| `status` | yes | enum | `active`, `retired`, `leave`, `unknown` | 公開不可。集計は可 |
| `hire_date` | no | date | 入社日 | 内部のみ |
| `retire_date` | no | date | 退職日 | 内部のみ |
| `valid_from` | yes | date | マスタ有効開始日 | 内部のみ |
| `valid_to` | no | date | マスタ有効終了日 | 内部のみ |
| `email` | no | string | メールアドレス | 内部のみ |

標準キーは `employee_id + valid_from` である。同一従業員の部署異動や雇用区分変更は有効期間で表す。

### 5.2 `attendance.csv`

| 列 | 必須 | 型 | 内容 | 公開可否 |
| --- | --- | --- | --- | --- |
| `employee_id` | yes | string | 従業員 ID | 条件付き |
| `work_date` | yes | date | 勤務日 | 公開可 |
| `clock_in_at` | no | datetime/time | 出勤打刻 | 内部のみ。個人別明細は条件付き |
| `clock_out_at` | no | datetime/time | 退勤打刻 | 内部のみ。個人別明細は条件付き |
| `break_minutes` | no | integer | 休憩分 | 集計可 |
| `work_minutes` | no | integer | 勤務分 | 集計可 |
| `overtime_minutes` | no | integer | 残業分 | 集計可。個人別表示は注意 |
| `leave_type` | no | string | 休暇種別 | 条件付き |
| `department_id` | no | string | 勤務部署 ID | 公開可 |
| `store_id` | no | string | 勤務店舗 ID | 公開可 |
| `shift_id` | no | string | シフト ID | 内部のみ |

`clock_in_at` と `clock_out_at` の両方が欠損している場合、行は無効候補として扱う。時刻逆転、重複、異常勤務時間候補の判定式は `BUSINESS-RULES.md` で定義する。

### 5.3 `labor_cost.csv`

| 列 | 必須 | 型 | 内容 | 公開可否 |
| --- | --- | --- | --- | --- |
| `target_month` | yes | month | 対象月 | 公開可 |
| `employee_id` | no | string | 従業員 ID | 条件付き |
| `department_id` | no | string | 部署 ID | 公開可 |
| `store_id` | no | string | 店舗 ID | 公開可 |
| `employment_type` | no | string | 雇用区分 | 公開可 |
| `cost_type` | no | enum | `salary`, `wage`, `allowance`, `social_insurance`, `other` | 公開可 |
| `amount` | yes | decimal | 金額 | 集計可 |
| `currency` | no | string | 通貨。標準 `JPY` | 公開可 |
| `payroll_run_id` | no | string | 給与計算側実行 ID | 内部のみ |

`employee_id` がない人件費データは、個人勤怠との結合可能データとして扱ってはならない。部署別、店舗別、雇用区分別などの限定集計として扱う。

### 5.4 `sales.csv`

| 列 | 必須 | 型 | 内容 | 公開可否 |
| --- | --- | --- | --- | --- |
| `store_id` | yes | string | 店舗 ID | 公開可 |
| `sales_date` | yes | date | 売上日 | 公開可 |
| `time_slot` | no | string | 時間帯。例 `09:00-10:00` | 公開可 |
| `department_id` | no | string | 部門 ID | 公開可 |
| `net_sales_amount` | yes | decimal | 税抜または純売上 | 公開可 |
| `gross_sales_amount` | no | decimal | 税込または総売上 | 公開可 |
| `transaction_count` | no | integer | 客数または取引数 | 公開可 |

売上データは従業員単位の労務評価に使わない。勤怠・シフトとの結合は店舗、日付、時間帯などの集計粒度で行う。

### 5.5 `shift.csv`

| 列 | 必須 | 型 | 内容 | 公開可否 |
| --- | --- | --- | --- | --- |
| `employee_id` | yes | string | 従業員 ID | 条件付き |
| `shift_date` | yes | date | シフト日 | 公開可 |
| `scheduled_start_at` | yes | datetime/time | 予定開始 | 条件付き |
| `scheduled_end_at` | yes | datetime/time | 予定終了 | 条件付き |
| `department_id` | no | string | 予定部署 ID | 公開可 |
| `store_id` | no | string | 予定店舗 ID | 公開可 |
| `role` | no | string | 役割 | 条件付き |
| `planned_headcount` | no | decimal | 予定人数。集計シフトの場合 | 公開可 |

個人シフトと集計シフトが混在する場合は、`data_grain` で区別する。

### 5.6 `fatigue.csv`

| 列 | 必須 | 型 | 内容 | 公開可否 |
| --- | --- | --- | --- | --- |
| `employee_id` | no | string | 従業員 ID | 内部のみ |
| `department_id` | no | string | 部署 ID | 集計後のみ |
| `store_id` | no | string | 店舗 ID | 集計後のみ |
| `measurement_date` | yes | date | 測定日 | 集計後のみ |
| `fatigue_score` | no | decimal | 疲労値 | 内部のみ |
| `sleep_hours` | no | decimal | 睡眠時間 | 内部のみ |
| `fatigue_comment` | no | string | 自由記述 | 内部のみ |
| `source_type` | no | string | アンケート、面談記録等 | 内部のみ |

個人別疲労値、睡眠時間、疲労コメントは、画面表示、成果物、ガイド AI 回答に平文で出してはならない。出力可能なのは、プライバシー抑制後の集計、抑制理由、件数、傾向に限定する。

### 5.7 `leave.csv`

| 列 | 必須 | 型 | 内容 | 公開可否 |
| --- | --- | --- | --- | --- |
| `employee_id` | yes | string | 従業員 ID | 条件付き |
| `fiscal_year` | yes | string | 年度 | 公開可 |
| `leave_type` | yes | string | 有給、特別休暇等 | 条件付き |
| `granted_days` | no | decimal | 付与日数 | 集計可 |
| `used_days` | no | decimal | 使用日数 | 集計可 |
| `remaining_days` | no | decimal | 残日数 | 条件付き |
| `leave_date` | no | date | 取得日 | 条件付き |

個人別の休暇残日数は、人事評価につながる文脈では表示しない。レポートでは部署別、店舗別、雇用区分別などの集計を優先する。

### 5.8 `share_candidate.csv`

| 列 | 必須 | 型 | 内容 | 公開可否 |
| --- | --- | --- | --- | --- |
| `record_id` | yes | string | レコード ID | 公開可 |
| `dataset_label` | yes | string | 共有予定データ名 | 公開可 |
| `field_name` | no | string | 検査対象列名 | 公開可 |
| `field_value` | no | string | 検査対象値 | 内部のみ。公開用ではマスク |
| `employee_id` | no | string | 従業員 ID | 内部のみ |
| `employee_name` | no | string | 氏名 | 内部のみ |
| `email` | no | string | メールアドレス | 内部のみ |
| `department_id` | no | string | 部署 ID | 条件付き |
| `contains_health_info` | no | boolean | 健康関連情報候補 | 公開可 |
| `free_text` | no | string | 自由記述 | 内部のみ |

外部共有前チェックは、共有可否の最終判断を出さない。識別子や推測リスクの確認材料だけを出力する。

## 6. 正規化データ設計

### 6.1 全正規化テーブル共通列

正規化テーブルは、次の列を共通で持つ。

| 列 | 型 | 内容 |
| --- | --- | --- |
| `id` | string | テーブル内レコード ID |
| `run_id` | string | 実行 ID |
| `source_file_id` | string | 入力 CSV ID |
| `source_row_number` | integer/null | 原本 CSV 行番号。集計入力の場合は null 可 |
| `source_record_hash` | string | 原本行のハッシュ |
| `dataset_kind` | enum | 入力データ種別 |
| `normalized_at` | datetime | 正規化時刻 |
| `schema_version` | string | 正規化スキーマバージョン |
| `parse_status` | enum | `ok`, `warning`, `failed` |
| `privacy_class` | enum | `public`, `personal`, `health`, `sensitive`, `internal` |

### 6.2 `norm_employees`

| 列 | 型 | 内容 |
| --- | --- | --- |
| `employee_key` | string | 内部主キー |
| `employee_id` | string | 従業員 ID |
| `employee_name` | string/null | 氏名。内部のみ |
| `department_id` | string | 部署 ID |
| `department_name` | string/null | 部署名 |
| `store_id` | string/null | 店舗 ID |
| `store_name` | string/null | 店舗名 |
| `employment_type` | string | 雇用区分 |
| `status` | enum | `active`, `retired`, `leave`, `unknown` |
| `hire_date` | date/null | 入社日 |
| `retire_date` | date/null | 退職日 |
| `valid_from` | date | 有効開始日 |
| `valid_to` | date/null | 有効終了日 |

### 6.3 `norm_attendance_records`

| 列 | 型 | 内容 |
| --- | --- | --- |
| `attendance_key` | string | 内部主キー |
| `employee_id` | string | 従業員 ID |
| `work_date` | date | 勤務日 |
| `clock_in_at` | datetime/null | 出勤打刻 |
| `clock_out_at` | datetime/null | 退勤打刻 |
| `break_minutes` | integer/null | 休憩分 |
| `work_minutes` | integer/null | 勤務分 |
| `overtime_minutes` | integer/null | 残業分 |
| `leave_type` | string/null | 休暇種別 |
| `department_id` | string/null | 部署 ID |
| `store_id` | string/null | 店舗 ID |
| `shift_id` | string/null | シフト ID |
| `is_cross_day` | boolean | 日跨ぎ勤務か |
| `master_match_status` | enum | `matched`, `missing`, `retired`, `department_mismatch`, `employment_type_mismatch`, `unknown` |

### 6.4 `norm_labor_costs`

| 列 | 型 | 内容 |
| --- | --- | --- |
| `labor_cost_key` | string | 内部主キー |
| `target_month` | string | 対象月 `YYYY-MM` |
| `employee_id` | string/null | 従業員 ID |
| `department_id` | string/null | 部署 ID |
| `store_id` | string/null | 店舗 ID |
| `employment_type` | string/null | 雇用区分 |
| `cost_type` | enum/null | 費目 |
| `amount` | decimal | 金額 |
| `currency` | string | 通貨 |
| `data_grain_id` | string | 粒度プロファイル ID |

### 6.5 `norm_sales_records`

| 列 | 型 | 内容 |
| --- | --- | --- |
| `sales_key` | string | 内部主キー |
| `store_id` | string | 店舗 ID |
| `sales_date` | date | 売上日 |
| `time_slot_start` | time/null | 時間帯開始 |
| `time_slot_end` | time/null | 時間帯終了 |
| `department_id` | string/null | 部門 ID |
| `net_sales_amount` | decimal | 純売上 |
| `gross_sales_amount` | decimal/null | 総売上 |
| `transaction_count` | integer/null | 取引数 |
| `data_grain_id` | string | 粒度プロファイル ID |

### 6.6 `norm_shift_records`

| 列 | 型 | 内容 |
| --- | --- | --- |
| `shift_key` | string | 内部主キー |
| `employee_id` | string/null | 従業員 ID。集計シフトでは null 可 |
| `shift_date` | date | シフト日 |
| `scheduled_start_at` | datetime | 予定開始 |
| `scheduled_end_at` | datetime | 予定終了 |
| `department_id` | string/null | 部署 ID |
| `store_id` | string/null | 店舗 ID |
| `role` | string/null | 役割 |
| `planned_headcount` | decimal/null | 予定人数 |
| `data_grain_id` | string | 粒度プロファイル ID |

### 6.7 `norm_fatigue_signals`

| 列 | 型 | 内容 |
| --- | --- | --- |
| `fatigue_key` | string | 内部主キー |
| `employee_id` | string/null | 従業員 ID。内部のみ |
| `department_id` | string/null | 部署 ID |
| `store_id` | string/null | 店舗 ID |
| `measurement_date` | date | 測定日 |
| `fatigue_score` | decimal/null | 個人疲労値。内部のみ |
| `sleep_hours` | decimal/null | 睡眠時間。内部のみ |
| `fatigue_comment` | string/null | 自由記述。内部のみ |
| `source_type` | string/null | 入力元種別 |
| `privacy_class` | enum | 必ず `health` または `sensitive` |

### 6.8 `norm_leave_records`

| 列 | 型 | 内容 |
| --- | --- | --- |
| `leave_key` | string | 内部主キー |
| `employee_id` | string | 従業員 ID |
| `fiscal_year` | string | 年度 |
| `leave_type` | string | 休暇種別 |
| `granted_days` | decimal/null | 付与日数 |
| `used_days` | decimal/null | 使用日数 |
| `remaining_days` | decimal/null | 残日数 |
| `leave_date` | date/null | 取得日 |
| `department_id` | string/null | マスタ照合後の部署 ID |
| `store_id` | string/null | マスタ照合後の店舗 ID |

### 6.9 `norm_share_candidates`

| 列 | 型 | 内容 |
| --- | --- | --- |
| `share_candidate_key` | string | 内部主キー |
| `record_id` | string | レコード ID |
| `dataset_label` | string | 共有予定データ名 |
| `field_name` | string/null | 対象列名 |
| `field_value_raw` | string/null | 原本値。内部のみ |
| `field_value_masked` | string/null | マスク後表示値 |
| `risk_category` | enum/null | `personal_identifier`, `health_info`, `small_group`, `free_text`, `unknown` |
| `risk_detected` | boolean | リスク候補あり |
| `privacy_status` | enum | `public`, `masked`, `suppressed`, `internal_only` |

## 7. 粒度設計

### 7.1 `data_grains`

粒度は、結合可否と成果物説明のために独立テーブルとして保持する。

| 列 | 型 | 内容 |
| --- | --- | --- |
| `data_grain_id` | string | 粒度プロファイル ID |
| `run_id` | string | 実行 ID |
| `dataset_kind` | enum | 対象データ種別 |
| `source_file_id` | string | 入力 CSV ID |
| `entity_grain` | enum | `employee`, `department`, `store`, `employment_type`, `organization`, `mixed`, `unknown` |
| `time_grain` | enum | `timestamp`, `shift`, `daily`, `weekly`, `monthly`, `period`, `unknown` |
| `location_grain` | enum | `store`, `department`, `organization`, `none`, `mixed`, `unknown` |
| `has_employee_id` | boolean | 従業員 ID を持つか |
| `has_department_id` | boolean | 部署 ID を持つか |
| `has_store_id` | boolean | 店舗 ID を持つか |
| `period_start` | date/null | 対象開始日 |
| `period_end` | date/null | 対象終了日 |
| `grain_signature` | string | 粒度の標準表現 |
| `confidence` | enum | `confirmed`, `inferred`, `unknown` |
| `notes` | string/null | 補足 |

`grain_signature` の例:

- `employee_daily`
- `employee_shift`
- `store_hourly`
- `department_monthly`
- `employment_type_monthly`
- `organization_monthly`

### 7.2 結合可否

| `join_status` | 意味 |
| --- | --- |
| `joinable` | キーと粒度が一致し、結合してよい |
| `limited_aggregate` | 個人単位では結合できないが、部署、店舗、月次などの限定集計は可能 |
| `not_joinable` | 必須キー、期間、粒度が不足し、対象分析には使えない |
| `unknown` | 判定不能。確認が必要 |

| `join_reason_code` | 意味 |
| --- | --- |
| `matching_employee_and_date` | 従業員 ID と日付が一致する |
| `matching_store_and_time_slot` | 店舗と日付・時間帯が一致する |
| `missing_employee_id` | 従業員 ID がない |
| `missing_store_id` | 店舗 ID がない |
| `time_grain_mismatch` | 日次、月次、時間帯などの時間粒度が一致しない |
| `entity_grain_mismatch` | 従業員、部署、店舗などの主体粒度が一致しない |
| `period_not_overlapping` | 対象期間が重ならない |
| `privacy_blocked` | プライバシー境界により結合または表示不可 |

従業員 ID を持たない人件費データは、個人勤怠と `joinable` にしてはならない。必要な場合は `limited_aggregate` とし、部署別、店舗別、雇用区分別などの集計に限定する。

## 8. issue 設計

### 8.1 issue 分類

| `issue_category` | 内容 |
| --- | --- |
| `schema_issue` | 列、型、形式、必須列に関する不備 |
| `data_quality_issue` | 行単位または値単位の不備 |
| `master_issue` | 従業員マスタとの照合不一致 |
| `grain_issue` | データ粒度が分析目的に合わない状態 |
| `join_issue` | キー不足または粒度不一致による結合不可 |
| `privacy_issue` | 表示前に抑制が必要な状態 |
| `processing_issue` | 読み込み、ジョブ、成果物生成などの処理問題 |

### 8.2 issue 優先度

| `severity` | 意味 |
| --- | --- |
| `critical` | 対象処理を継続できない、または安全境界違反の可能性がある |
| `high` | 主要レポートの正確性に大きく影響する |
| `medium` | 一部の集計または確認項目に影響する |
| `low` | 参考情報または軽微な修正候補 |

### 8.3 `issues` テーブル / `issues.csv` 共通列

| 列 | 型 | 内容 |
| --- | --- | --- |
| `issue_id` | string | issue ID |
| `run_id` | string | 実行 ID |
| `source_file_id` | string/null | 入力 CSV ID |
| `dataset_kind` | enum/null | データ種別 |
| `source_row_number` | integer/null | 原本行番号 |
| `column_name` | string/null | 標準列名 |
| `raw_column_name` | string/null | 原本列名 |
| `issue_category` | enum | issue 分類 |
| `issue_code` | string | issue code。詳細体系は `BUSINESS-RULES.md` |
| `severity` | enum | 優先度 |
| `readiness_effect` | enum | `none`, `partial`, `blocked` |
| `message` | string | 利用者向け説明 |
| `evidence_ref` | string/null | 根拠参照。例 `src_x:row=123:col=clock_in_at` |
| `evidence_value_masked` | string/null | マスク済み根拠値 |
| `detected_at` | datetime | 検出時刻 |
| `related_rule_id` | string/null | 業務ルール ID |
| `status` | enum | `open`, `acknowledged`, `resolved_by_recheck`, `ignored` |

`issues.csv` に個人疲労値、睡眠時間、疲労コメント、メールアドレス、氏名の平文を出してはならない。必要な場合は `evidence_value_masked` に伏せ字を出す。

## 9. プライバシー抑制設計

### 9.1 `privacy_status`

| 値 | 意味 |
| --- | --- |
| `public` | 表示・出力してよい |
| `masked` | 値を伏せ字、丸め、範囲化して表示する |
| `suppressed` | 行、セル、集計値を非表示にする |
| `internal_only` | 内部処理専用。成果物、画面、ガイド AI 参照対象にしない |

### 9.2 `privacy_suppressions`

| 列 | 型 | 内容 |
| --- | --- | --- |
| `suppression_id` | string | 抑制 ID |
| `run_id` | string | 実行 ID |
| `artifact_id` | string/null | 対象成果物 ID |
| `dataset_kind` | enum/null | 対象データ種別 |
| `target_type` | enum | `row`, `cell`, `aggregate`, `report_section`, `artifact` |
| `target_ref` | string | 対象参照 |
| `privacy_status` | enum | 抑制後状態 |
| `reason_code` | string | 抑制理由コード |
| `reason_message` | string | 利用者向け抑制理由 |
| `affected_count` | integer/null | 抑制対象件数 |
| `threshold_name` | string/null | 閾値名。例 `small_group_min_count` |
| `threshold_value` | string/null | 実行時閾値 |
| `created_at` | datetime | 作成時刻 |

### 9.3 抑制対象

次の値は公開用成果物に平文で含めてはならない。

- 個人別疲労値
- 個人の睡眠時間
- 疲労コメント
- 個人別疲労ランキング
- 医療診断または治療指示に読める文言
- 適法・違法の断定
- 人事評価、配置適性、懲戒対象の断定
- 少人数部署または個人が推測され得る健康関連集計
- ガイド AI が抑制対象情報を復元できる根拠値

少人数部署とみなす閾値、丸め、伏せ字、範囲化の具体ルールは `BUSINESS-RULES.md` で定義する。この文書では、抑制結果を保存する構造だけを定義する。

## 10. ローカル DB 論理スキーマ

### 10.1 テーブル一覧

| テーブル | 主な内容 | 保持期間 |
| --- | --- | --- |
| `runs` | 実行履歴、状態、対象期間 | 長期 |
| `run_settings` | 実行設定、閾値、対象データセット | 長期 |
| `source_files` | 原本 CSV の保存情報、ハッシュ、読み込み設定 | 長期 |
| `raw_csv_rows` | 原本行の参照、ハッシュ、パース状態 | 任意。監査要件に応じて長期 |
| `column_mappings` | 原本列と標準列の対応 | 長期 |
| `schema_checks` | 必須列、型、形式の検査結果 | 実行単位 |
| `norm_employees` | 正規化従業員マスタ | 再生成可能 |
| `norm_attendance_records` | 正規化勤怠 | 再生成可能 |
| `norm_labor_costs` | 正規化人件費 | 再生成可能 |
| `norm_sales_records` | 正規化売上 | 再生成可能 |
| `norm_shift_records` | 正規化シフト | 再生成可能 |
| `norm_fatigue_signals` | 正規化疲労関連データ。内部のみ | 再生成可能、アクセス制限 |
| `norm_leave_records` | 正規化休暇情報 | 再生成可能 |
| `norm_share_candidates` | 共有予定データ検査対象 | 再生成可能 |
| `data_grains` | 粒度プロファイル | 実行単位 |
| `join_assessments` | 結合可否判定 | 実行単位 |
| `issues` | issue | 実行単位 |
| `analysis_checkpoints` | 業務確認ポイント | 実行単位 |
| `aggregate_metrics` | 集計メトリクス | 実行単位 |
| `privacy_suppressions` | 抑制結果 | 実行単位 |
| `artifacts` | 成果物メタデータ | 長期 |
| `recheck_comparisons` | 修正前後比較 | 長期 |
| `guide_documents` | ガイド AI 検索対象メタデータ | 任意 |

### 10.2 コア DDL 例

以下は論理スキーマを示す例である。実際の型は採用 DB に合わせて調整する。

```sql
CREATE TABLE runs (
  run_id TEXT PRIMARY KEY,
  run_status TEXT NOT NULL,
  readiness_status TEXT,
  period_start DATE,
  period_end DATE,
  started_at TEXT NOT NULL,
  finished_at TEXT,
  code_version TEXT,
  schema_version TEXT NOT NULL,
  created_by TEXT,
  notes TEXT
);

CREATE TABLE run_settings (
  run_id TEXT NOT NULL,
  setting_key TEXT NOT NULL,
  setting_value TEXT NOT NULL,
  value_type TEXT NOT NULL,
  PRIMARY KEY (run_id, setting_key),
  FOREIGN KEY (run_id) REFERENCES runs(run_id)
);

CREATE TABLE source_files (
  source_file_id TEXT PRIMARY KEY,
  run_id TEXT NOT NULL,
  dataset_kind TEXT NOT NULL,
  original_filename TEXT NOT NULL,
  stored_path TEXT NOT NULL,
  input_hash_sha256 TEXT NOT NULL,
  size_bytes INTEGER NOT NULL,
  encoding TEXT NOT NULL,
  delimiter TEXT NOT NULL,
  quote_char TEXT,
  has_header INTEGER NOT NULL,
  detected_row_count INTEGER,
  detected_column_count INTEGER,
  schema_profile_version TEXT NOT NULL,
  ingested_at TEXT NOT NULL,
  immutable INTEGER NOT NULL DEFAULT 1,
  FOREIGN KEY (run_id) REFERENCES runs(run_id)
);

CREATE TABLE column_mappings (
  column_mapping_id TEXT PRIMARY KEY,
  run_id TEXT NOT NULL,
  source_file_id TEXT NOT NULL,
  raw_column_name TEXT NOT NULL,
  standard_column_name TEXT,
  mapping_status TEXT NOT NULL,
  confidence TEXT,
  issue_id TEXT,
  FOREIGN KEY (run_id) REFERENCES runs(run_id),
  FOREIGN KEY (source_file_id) REFERENCES source_files(source_file_id)
);
```

### 10.3 `issues` DDL 例

```sql
CREATE TABLE issues (
  issue_id TEXT PRIMARY KEY,
  run_id TEXT NOT NULL,
  source_file_id TEXT,
  dataset_kind TEXT,
  source_row_number INTEGER,
  column_name TEXT,
  raw_column_name TEXT,
  issue_category TEXT NOT NULL,
  issue_code TEXT NOT NULL,
  severity TEXT NOT NULL,
  readiness_effect TEXT NOT NULL,
  message TEXT NOT NULL,
  evidence_ref TEXT,
  evidence_value_masked TEXT,
  related_rule_id TEXT,
  status TEXT NOT NULL DEFAULT 'open',
  detected_at TEXT NOT NULL,
  FOREIGN KEY (run_id) REFERENCES runs(run_id),
  FOREIGN KEY (source_file_id) REFERENCES source_files(source_file_id)
);

CREATE INDEX idx_issues_run_category ON issues(run_id, issue_category);
CREATE INDEX idx_issues_run_severity ON issues(run_id, severity);
CREATE INDEX idx_issues_source_row ON issues(source_file_id, source_row_number);
```

### 10.4 粒度・結合 DDL 例

```sql
CREATE TABLE data_grains (
  data_grain_id TEXT PRIMARY KEY,
  run_id TEXT NOT NULL,
  dataset_kind TEXT NOT NULL,
  source_file_id TEXT,
  entity_grain TEXT NOT NULL,
  time_grain TEXT NOT NULL,
  location_grain TEXT NOT NULL,
  has_employee_id INTEGER NOT NULL,
  has_department_id INTEGER NOT NULL,
  has_store_id INTEGER NOT NULL,
  period_start DATE,
  period_end DATE,
  grain_signature TEXT NOT NULL,
  confidence TEXT NOT NULL,
  notes TEXT,
  FOREIGN KEY (run_id) REFERENCES runs(run_id),
  FOREIGN KEY (source_file_id) REFERENCES source_files(source_file_id)
);

CREATE TABLE join_assessments (
  join_assessment_id TEXT PRIMARY KEY,
  run_id TEXT NOT NULL,
  left_dataset_kind TEXT NOT NULL,
  right_dataset_kind TEXT NOT NULL,
  left_data_grain_id TEXT,
  right_data_grain_id TEXT,
  join_status TEXT NOT NULL,
  join_reason_code TEXT NOT NULL,
  join_keys TEXT,
  period_start DATE,
  period_end DATE,
  explanation TEXT NOT NULL,
  created_at TEXT NOT NULL,
  FOREIGN KEY (run_id) REFERENCES runs(run_id)
);
```

### 10.5 成果物 DDL 例

```sql
CREATE TABLE artifacts (
  artifact_id TEXT PRIMARY KEY,
  run_id TEXT NOT NULL,
  artifact_kind TEXT NOT NULL,
  artifact_format TEXT NOT NULL,
  schema_version TEXT NOT NULL,
  relative_path TEXT NOT NULL,
  content_hash_sha256 TEXT,
  generated_at TEXT NOT NULL,
  privacy_filtered INTEGER NOT NULL,
  row_count INTEGER,
  byte_size INTEGER,
  generation_status TEXT NOT NULL,
  failure_reason TEXT,
  FOREIGN KEY (run_id) REFERENCES runs(run_id)
);

CREATE TABLE recheck_comparisons (
  comparison_id TEXT PRIMARY KEY,
  base_run_id TEXT NOT NULL,
  recheck_run_id TEXT NOT NULL,
  comparison_status TEXT NOT NULL,
  base_issue_count INTEGER,
  recheck_issue_count INTEGER,
  resolved_issue_count INTEGER,
  new_issue_count INTEGER,
  generated_at TEXT NOT NULL,
  summary_json TEXT,
  FOREIGN KEY (base_run_id) REFERENCES runs(run_id),
  FOREIGN KEY (recheck_run_id) REFERENCES runs(run_id)
);
```

### 10.6 推奨インデックス

| テーブル | インデックス | 目的 |
| --- | --- | --- |
| `source_files` | `(run_id, dataset_kind)` | 実行内データ種別検索 |
| `raw_csv_rows` | `(source_file_id, source_row_number)` | 原本行参照 |
| `norm_attendance_records` | `(run_id, employee_id, work_date)` | 勤怠確認、マスタ照合 |
| `norm_attendance_records` | `(run_id, store_id, work_date)` | 店舗別確認 |
| `norm_labor_costs` | `(run_id, target_month, department_id)` | 部署別人件費集計 |
| `norm_sales_records` | `(run_id, store_id, sales_date)` | 売上・人員不足確認 |
| `issues` | `(run_id, severity, issue_category)` | issue 一覧 |
| `join_assessments` | `(run_id, left_dataset_kind, right_dataset_kind)` | 結合可否参照 |
| `artifacts` | `(run_id, artifact_kind)` | 成果物検索 |

## 11. 分析用データ

### 11.1 `analysis_checkpoints`

業務確認ポイントは、issue とは分離する。確認ポイントは判断材料であり、適法・違法、医療診断、人事評価の結論ではない。

| 列 | 型 | 内容 |
| --- | --- | --- |
| `checkpoint_id` | string | 確認ポイント ID |
| `run_id` | string | 実行 ID |
| `checkpoint_category` | enum | `labor`, `staffing`, `labor_cost`, `monthly`, `share`, `privacy` |
| `target_grain` | string | 対象粒度 |
| `target_ref` | string/null | 部署、店舗、雇用区分など |
| `period_start` | date | 対象開始日 |
| `period_end` | date | 対象終了日 |
| `title` | string | 見出し |
| `message` | string | 確認材料としての説明 |
| `evidence_ref` | string | 根拠データまたは集計条件 |
| `severity_hint` | enum | `high`, `medium`, `low`, `info` |
| `privacy_status` | enum | 公開状態 |
| `created_at` | datetime | 作成時刻 |

### 11.2 `aggregate_metrics`

| 列 | 型 | 内容 |
| --- | --- | --- |
| `metric_id` | string | メトリクス ID |
| `run_id` | string | 実行 ID |
| `metric_name` | string | 指標名 |
| `metric_category` | enum | `attendance`, `labor_cost`, `sales`, `shift`, `fatigue`, `leave`, `privacy` |
| `target_grain` | string | 集計粒度 |
| `target_ref` | string/null | 店舗、部署、雇用区分など |
| `period_start` | date | 対象開始日 |
| `period_end` | date | 対象終了日 |
| `value_decimal` | decimal/null | 数値 |
| `value_text` | string/null | テキスト値 |
| `unit` | string/null | 単位 |
| `sample_count` | integer/null | 集計対象件数 |
| `privacy_status` | enum | 公開状態 |
| `suppression_id` | string/null | 抑制 ID |
| `evidence_query_hash` | string/null | 集計条件のハッシュ |

疲労関連メトリクスは、個人別値ではなく、抑制判定後の集計値のみ公開対象にできる。

## 12. 成果物ファイル構造

### 12.1 ディレクトリ構造

標準の成果物出力先は次の形とする。

```text
out/
  {run_id}/
    artifact_manifest.json
    run_summary.json
    profile_report.json
    data_readiness.csv
    issues.csv
    join_assessments.csv
    labor_checkpoints.csv
    labor_cost_summary.csv
    staffing_checkpoints.csv
    monthly_labor_summary.csv
    privacy_suppressions.csv
    external_share_checklist.csv
    recheck_comparison.json        # 再確認実行の場合
    recheck_comparison.csv         # 再確認実行の場合
```

すべての成果物は `artifact_manifest.json` に登録する。ファイルパス、ハッシュ、生成時刻、スキーマバージョン、プライバシー抑制済みかどうかを追跡する。

### 12.2 `artifact_manifest.json`

```json
{
  "schema_version": "artifact_manifest.v1",
  "run_id": "run_01HX0000000000000000000000",
  "generated_at": "2026-06-02T10:20:30+09:00",
  "artifacts": [
    {
      "artifact_id": "art_01HX0000000000000000000001",
      "artifact_kind": "run_summary",
      "format": "json",
      "path": "run_summary.json",
      "content_hash_sha256": "...",
      "privacy_filtered": true,
      "schema_version": "run_summary.v1",
      "row_count": null,
      "byte_size": 12345
    }
  ]
}
```

### 12.3 `run_summary.json`

```json
{
  "schema_version": "run_summary.v1",
  "run_id": "run_01HX0000000000000000000000",
  "run_status": "completed",
  "readiness_status": "partial",
  "period": {
    "start": "2026-04-01",
    "end": "2026-04-30"
  },
  "generated_at": "2026-06-02T10:20:30+09:00",
  "inputs": [
    {
      "source_file_id": "src_01HX0000000000000000000001",
      "dataset_kind": "attendance",
      "original_filename": "attendance_202604.csv",
      "input_hash_sha256": "...",
      "row_count": 120000,
      "schema_profile_version": "attendance_csv.v1"
    }
  ],
  "settings": {
    "small_group_min_count": 5,
    "target_dataset_kinds": ["employee_master", "attendance", "labor_cost"]
  },
  "issue_counts": {
    "total": 128,
    "critical": 3,
    "high": 20,
    "medium": 72,
    "low": 33
  },
  "readiness": [
    {
      "report_kind": "labor_cost_summary",
      "status": "partial",
      "reason": "人件費データが月次部署粒度のため、個人勤怠とは限定集計のみ可能"
    }
  ],
  "privacy": {
    "privacy_filtered": true,
    "suppressed_count": 12,
    "masked_count": 30
  },
  "artifacts": [
    {
      "artifact_kind": "issues",
      "format": "csv",
      "path": "issues.csv"
    }
  ]
}
```

### 12.4 `profile_report.json`

```json
{
  "schema_version": "profile_report.v1",
  "run_id": "run_01HX0000000000000000000000",
  "generated_at": "2026-06-02T10:20:30+09:00",
  "datasets": [
    {
      "source_file_id": "src_01HX0000000000000000000001",
      "dataset_kind": "attendance",
      "row_count": 120000,
      "column_count": 12,
      "required_columns": ["employee_id", "work_date"],
      "missing_required_columns": [],
      "column_profiles": [
        {
          "column_name": "work_date",
          "raw_column_name": "勤務日",
          "type_status": "ok",
          "null_count": 0,
          "invalid_format_count": 0
        }
      ],
      "data_grain": {
        "entity_grain": "employee",
        "time_grain": "daily",
        "location_grain": "store",
        "grain_signature": "employee_daily"
      }
    }
  ]
}
```

## 13. 成果物 CSV 列定義

### 13.1 `data_readiness.csv`

| 列 | 型 | 内容 |
| --- | --- | --- |
| `run_id` | string | 実行 ID |
| `report_kind` | string | 対象レポート種別 |
| `readiness_status` | enum | `ready`, `partial`, `blocked` |
| `reason_code` | string | 理由コード |
| `reason_message` | string | 理由説明 |
| `required_dataset_kinds` | string | 必要データ種別。セミコロン区切り |
| `available_dataset_kinds` | string | 利用可能データ種別。セミコロン区切り |
| `blocking_issue_count` | integer | blocked 原因 issue 数 |
| `partial_issue_count` | integer | partial 原因 issue 数 |
| `generated_at` | datetime | 生成時刻 |

### 13.2 `issues.csv`

`issues.csv` は `issues` テーブルの公開可能列から生成する。

| 列 | 型 | 内容 |
| --- | --- | --- |
| `issue_id` | string | issue ID |
| `run_id` | string | 実行 ID |
| `dataset_kind` | enum | データ種別 |
| `source_file_id` | string | 入力 CSV ID |
| `source_row_number` | integer/null | 原本行番号 |
| `column_name` | string/null | 標準列名 |
| `raw_column_name` | string/null | 原本列名 |
| `issue_category` | enum | issue 分類 |
| `issue_code` | string | issue code |
| `severity` | enum | 優先度 |
| `readiness_effect` | enum | 準備状態への影響 |
| `message` | string | 修正依頼に使える説明 |
| `evidence_ref` | string/null | 根拠参照 |
| `evidence_value_masked` | string/null | マスク済み根拠値 |
| `status` | enum | issue 状態 |

### 13.3 `join_assessments.csv`

| 列 | 型 | 内容 |
| --- | --- | --- |
| `join_assessment_id` | string | 結合判定 ID |
| `run_id` | string | 実行 ID |
| `left_dataset_kind` | enum | 左データ種別 |
| `right_dataset_kind` | enum | 右データ種別 |
| `left_grain_signature` | string | 左粒度 |
| `right_grain_signature` | string | 右粒度 |
| `join_status` | enum | `joinable`, `limited_aggregate`, `not_joinable`, `unknown` |
| `join_reason_code` | string | 理由コード |
| `join_keys` | string | 利用可能な結合キー。セミコロン区切り |
| `period_start` | date/null | 対象開始日 |
| `period_end` | date/null | 対象終了日 |
| `explanation` | string | 利用者向け説明 |

### 13.4 `labor_checkpoints.csv`

| 列 | 型 | 内容 |
| --- | --- | --- |
| `checkpoint_id` | string | 確認ポイント ID |
| `run_id` | string | 実行 ID |
| `checkpoint_category` | enum | `labor` |
| `target_grain` | string | 対象粒度 |
| `target_ref` | string/null | 部署、店舗等。個人識別子は原則出さない |
| `period_start` | date | 対象開始日 |
| `period_end` | date | 対象終了日 |
| `title` | string | 見出し |
| `message` | string | 確認材料としての説明 |
| `evidence_ref` | string | 根拠データまたは集計条件 |
| `severity_hint` | enum | `high`, `medium`, `low`, `info` |
| `privacy_status` | enum | 公開状態 |
| `caution` | string | 最終判断ではないこと等の注意文 |

### 13.5 `labor_cost_summary.csv`

| 列 | 型 | 内容 |
| --- | --- | --- |
| `run_id` | string | 実行 ID |
| `target_month` | month | 対象月 |
| `summary_grain` | string | `department_monthly`, `store_monthly`, `employment_type_monthly` 等 |
| `department_id` | string/null | 部署 ID |
| `store_id` | string/null | 店舗 ID |
| `employment_type` | string/null | 雇用区分 |
| `cost_type` | string/null | 費目 |
| `amount` | decimal | 金額 |
| `currency` | string | 通貨 |
| `source_count` | integer | 集計元件数 |
| `join_status_with_attendance` | enum | 勤怠との結合可否 |
| `privacy_status` | enum | 公開状態 |
| `suppression_reason` | string/null | 抑制理由 |

### 13.6 `staffing_checkpoints.csv`

| 列 | 型 | 内容 |
| --- | --- | --- |
| `checkpoint_id` | string | 確認ポイント ID |
| `run_id` | string | 実行 ID |
| `store_id` | string/null | 店舗 ID |
| `department_id` | string/null | 部署 ID |
| `weekday` | string/null | 曜日 |
| `time_slot` | string/null | 時間帯 |
| `period_start` | date | 対象開始日 |
| `period_end` | date | 対象終了日 |
| `signal_name` | string | 不足傾向などの指標名 |
| `signal_value` | decimal/string | 指標値 |
| `message` | string | 確認材料としての説明 |
| `evidence_ref` | string | 根拠データまたは集計条件 |
| `privacy_status` | enum | 公開状態 |

### 13.7 `privacy_suppressions.csv`

| 列 | 型 | 内容 |
| --- | --- | --- |
| `suppression_id` | string | 抑制 ID |
| `run_id` | string | 実行 ID |
| `artifact_kind` | string/null | 対象成果物種別 |
| `target_type` | enum | `row`, `cell`, `aggregate`, `report_section`, `artifact` |
| `target_ref` | string | 対象参照 |
| `privacy_status` | enum | 抑制後状態 |
| `reason_code` | string | 理由コード |
| `reason_message` | string | 抑制理由 |
| `affected_count` | integer/null | 対象件数 |
| `threshold_name` | string/null | 閾値名 |
| `threshold_value` | string/null | 閾値 |

### 13.8 `external_share_checklist.csv`

| 列 | 型 | 内容 |
| --- | --- | --- |
| `check_id` | string | チェック ID |
| `run_id` | string | 実行 ID |
| `dataset_label` | string | 共有予定データ名 |
| `record_id` | string/null | レコード ID |
| `field_name` | string/null | 対象列名 |
| `risk_category` | enum | `personal_identifier`, `health_info`, `small_group`, `free_text`, `unknown` |
| `risk_detected` | boolean | リスク候補あり |
| `masked_example` | string/null | マスク済み例 |
| `message` | string | 確認材料としての説明 |
| `final_decision` | string | 固定値 `not_provided` |

### 13.9 `recheck_comparison.csv`

| 列 | 型 | 内容 |
| --- | --- | --- |
| `comparison_id` | string | 比較 ID |
| `base_run_id` | string | 修正前 RunId |
| `recheck_run_id` | string | 修正後 RunId |
| `dataset_kind` | enum/null | データ種別 |
| `base_input_hash_sha256` | string | 修正前入力ハッシュ |
| `recheck_input_hash_sha256` | string | 修正後入力ハッシュ |
| `base_issue_count` | integer | 修正前 issue 件数 |
| `recheck_issue_count` | integer | 修正後 issue 件数 |
| `resolved_issue_count` | integer | 解消 issue 件数 |
| `new_issue_count` | integer | 新規 issue 件数 |
| `message` | string | 比較結果の説明 |

## 14. 公開用 JSON/CSV の安全要件

公開用成果物は、次を満たさなければならない。

1. `run_id` を持つ。
2. 対象期間を持つ。対象期間が不明な場合は `period_unknown_reason` を持つ。
3. 入力データ種別または入力参照を持つ。
4. 生成時刻を持つ。
5. データ品質 issue、業務確認ポイント、プライバシー抑制結果を別フィールドまたは別ファイルで表す。
6. 個人疲労値、睡眠時間、疲労コメントを平文で含まない。
7. 抑制理由を、可能な範囲で `reason_code` と `reason_message` として含む。
8. 結合済みデータには、利用したデータ種別、期間、粒度、結合キーを明示する。
9. 法的判断、医療判断、人事評価、外部共有可否の結論として読める文言を含まない。
10. ガイド AI が参照する成果物は、必ずプライバシー抑制後のファイルまたは抑制済み DB ビューに限定する。

## 15. RAG / ガイド AI 向けデータ境界

ガイド AI が検索対象にしてよいデータは次に限定する。

| 対象 | 参照可否 | 条件 |
| --- | --- | --- |
| 製品ドキュメント | 可 | 公開用仕様文書のみ |
| レポート定義 | 可 | 抑制対象の例値を含まないこと |
| 制約条件 | 可 | 業務ルール、抑制ルール、注意文 |
| 生成済みレポート | 可 | `privacy_filtered = true` の成果物のみ |
| `issues.csv` | 条件付き | 氏名、メール、疲労値、睡眠時間、自由記述がマスク済みであること |
| `norm_fatigue_signals` | 不可 | 内部データのため直接参照不可 |
| 原本 CSV | 不可 | 原本保護と個人情報保護のため直接参照不可 |
| 抑制前集計 | 不可 | 個人推測リスクがあるため直接参照不可 |

必要に応じて、`guide_documents` テーブルに検索対象メタデータを保存する。

| 列 | 型 | 内容 |
| --- | --- | --- |
| `guide_document_id` | string | 文書 ID |
| `run_id` | string/null | 実行 ID。仕様文書は null 可 |
| `artifact_id` | string/null | 成果物 ID |
| `document_kind` | enum | `product_doc`, `report_definition`, `generated_report`, `constraint` |
| `title` | string | タイトル |
| `relative_path` | string | ローカルパス |
| `content_hash_sha256` | string | 内容ハッシュ |
| `privacy_filtered` | boolean | 抑制済みか |
| `indexed_at` | datetime | インデックス化時刻 |

## 16. スキーマバージョン管理

### 16.1 バージョン体系

| 対象 | 例 | 変更単位 |
| --- | --- | --- |
| 入力 CSV プロファイル | `attendance_csv.v1` | 必須列、任意列、別名、型 |
| 正規化スキーマ | `normalized.v1` | 正規化テーブルと列 |
| 成果物スキーマ | `run_summary.v1`, `issues_csv.v1` | JSON/CSV 出力形式 |
| DB スキーマ | `local_db.v1` | テーブル、制約、インデックス |

後方互換のない変更は major バージョンを上げる。列追加や任意フィールド追加は minor 相当として扱ってよいが、成果物の `schema_version` には必ず反映する。

### 16.2 マイグレーション方針

- 原本 CSV と入力ハッシュはマイグレーション対象にしない。
- 正規化データと分析データは、原本と設定から再生成できるため、必要に応じて破棄・再生成してよい。
- 実行履歴、成果物メタデータ、再確認比較は監査用途があるため、互換ビューまたは移行スクリプトを用意する。
- 旧成果物は `schema_version` と `content_hash_sha256` を保持し、生成時点の意味を壊さない。

## 17. データ保持と削除

この文書では保持期間の具体値は固定しない。運用上の保持期間は `OPERATIONS.md` で定義する。ただし、データ分類ごとの扱いは次の通りとする。

| データ | 推奨扱い |
| --- | --- |
| 原本 CSV | 原本保護対象。削除は明示操作と監査ログが必要 |
| 入力ハッシュ、実行履歴 | 長期保持推奨 |
| 正規化データ | 再生成可能。容量制約に応じて削除可 |
| issue | 再確認履歴との比較に使うため保持推奨 |
| 疲労関連内部データ | 最小保持。アクセス制限必須 |
| 抑制後成果物 | 共有・確認用途のため保持可 |
| ガイド AI インデックス | 抑制済みデータだけを保持 |

## 18. 受け入れ基準への対応観点

| 観点 | データ設計で確認する内容 |
| --- | --- |
| 原本保護 | `source_files.input_hash_sha256` と保存ファイルが実行前後で変わらないこと |
| issue 出力 | `issues.csv` に行、列、理由、優先度、RunId または入力参照があること |
| プライバシー | 公開用成果物に個人疲労値、睡眠時間、疲労コメントが含まれないこと |
| 少人数抑制 | `privacy_suppressions` に抑制理由と対象が記録されること |
| 粒度判定 | `data_grains` と `join_assessments` に粒度と結合可否が記録されること |
| マスタ照合 | `master_issue` が未登録、退職済み、部署不一致、雇用区分不一致を表せること |
| 再確認 | `recheck_comparisons` に修正前後の RunId、入力ハッシュ、issue 件数差分があること |
| ローカル DB 処理 | 10000 人 × 3 年分の勤怠行を想定し、検索キーとインデックスが設計されていること |

## 19. 未決事項

| ID | 未決事項 | 決定先 |
| --- | --- | --- |
| DD-OPEN-001 | CSV 列名の別名辞書の初期セット | `DATA-DESIGN.md` 改訂または実装設定 |
| DD-OPEN-002 | 少人数部署の閾値 | `BUSINESS-RULES.md` |
| DD-OPEN-003 | 長時間労働候補などの具体判定式 | `BUSINESS-RULES.md` |
| DD-OPEN-004 | ローカル DB エンジンの最終選定 | `ARCHITECTURE.md` |
| DD-OPEN-005 | 保存時暗号化、アクセス制御、ログマスキング | `ARCHITECTURE.md`, `OPERATIONS.md` |
| DD-OPEN-006 | PDF / Markdown レポートの見た目 | `EXTERNAL-DESIGN.md` |
| DD-OPEN-007 | ガイド AI のインデックス更新条件 | `ARCHITECTURE.md`, `OPERATIONS.md` |

