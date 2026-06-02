# LaborLens 業務ルール定義書

Date: 2026-06-02
Status: draft
Rule Version: `business-rules-2026-06-02-draft`
Source: `REQUIREMENTS_BRUSHED.md`
Related:

- `GLOSSARY.md`
- `ACCEPTANCE-CRITERIA.md`
- `DATA-DESIGN.md`
- `ARCHITECTURE.md`
- `TEST-PLAN.md`
- `docs/product/USE-CASES.md`
- `docs/product/LEAN-SPEC-PLANNING.md`

## この文書の位置づけ

この文書は、LaborLens の業務ルール、判定条件、issue code、確認候補、抑制条件、結合可否条件を定義する。

`REQUIREMENTS_BRUSHED.md` は、詳細な issue 判定式や具体的なテストケースを `BUSINESS-RULES.md` と後続文書に分離している。本書はそのうち、実装・テスト・Lean 仕様化で参照できる決定的な業務ルールを扱う。

本書で定義するもの:

- CSV 取り込み、スキーマ確認、データ品質検査の判定条件
- 勤怠データの打刻漏れ、時刻逆転、重複、異常値候補の判定条件
- 従業員マスタ照合の判定条件
- データ粒度と結合可否の判定条件
- 長時間労働候補、連続勤務候補、休憩不足候補、有給取得不足候補の初期閾値
- 人件費、売上、シフトを組み合わせる場合の限定集計条件
- 少人数部署、健康関連情報、個人推測リスクに関する抑制条件
- レポート生成前に満たすべき不変条件
- issue code と確認コードの命名規則

本書で定義しないもの:

| 項目 | 扱う文書 |
| --- | --- |
| 用語の完全な辞書 | `GLOSSARY.md` |
| CSV の列定義、内部データ型、DB スキーマ | `DATA-DESIGN.md` |
| UI、画面、操作フロー、レポートレイアウト | `EXTERNAL-DESIGN.md` |
| バックグラウンドジョブ、ローカル DB、ガイド AI の構成 | `ARCHITECTURE.md` |
| 受け入れ基準とテストケース | `ACCEPTANCE-CRITERIA.md`, `TEST-PLAN.md` |
| 法的な適法・違法の最終判断 | 対象外。利用者または専門家が判断する |
| 医療診断、高ストレス者の個別判定、治療指示 | 対象外 |
| 人事評価、配置適性、懲戒対象の判断 | 対象外 |
| 給与計算の確定処理 | 対象外 |

## 0. 法令・外部基準の扱い

LaborLens は、法令値や行政資料上の数値を「確認候補の初期閾値」として利用できる。ただし、LaborLens は適法・違法の最終判断を出力してはならない。

本書の労務系初期閾値は、2026-06-02 時点で確認した次の公的情報を参照する。

| 参照項目 | 参照元 |
| --- | --- |
| 法定労働時間、休憩、休日 | 厚生労働省「労働時間・休日」 https://www.mhlw.go.jp/stf/seisakunitsuite/bunya/koyou_roudou/roudoukijun/roudouzikan/index.html |
| 36 協定、時間外労働の上限、長時間労働と健康リスクの目安 | 厚生労働省「36協定で定める時間外労働及び休日労働について留意すべき事項に関する指針」 https://www.mhlw.go.jp/content/000350731.pdf |
| 年 5 日の年次有給休暇取得義務の確認材料 | 厚生労働省「年5日の年次有給休暇の確実な取得」 https://www.mhlw.go.jp/content/000463186.pdf |
| 年次有給休暇の発生要件・付与日数の一般説明 | 厚生労働省 働き方・休み方改善ポータルサイト「労働者の方へ」 https://work-holiday.mhlw.go.jp/kyuuka-sokushin/roudousya.html |

法令、業種特例、変形労働時間制、裁量労働制、管理監督者、医師・自動車運転者等の特例は、組織ごとの労務設定によって結論が変わり得る。本書の初期値は標準的な確認候補の検出に用いる。業種特例や就業規則固有の条件は、`rule_config` で明示的に上書きしなければならない。

## 1. 基本原則

### 1.1 確認支援に限定する

LaborLens は、確認材料を提示する。出力文言は、次の表現を使用する。

| 使用してよい表現 | 使用してはならない表現 |
| --- | --- |
| `確認候補` | `違法` |
| `要確認` | `法令違反` |
| `データ上の可能性` | `労基法違反確定` |
| `根拠データでは閾値を超過` | `会社に責任がある` |
| `専門家確認を推奨` | `処分対象` |
| `休憩不足候補` | `休憩未付与と断定` |
| `長時間労働候補` | `過労死認定` |

### 1.2 issue と業務確認ポイントを分離する

LaborLens は、データ品質上の不備と業務上の確認ポイントを混在させてはならない。

| 種別 | 意味 | 代表例 | 主な出力先 |
| --- | --- | --- | --- |
| `issue` | 入力、形式、整合性、粒度、結合、抑制など、処理品質または安全境界に関わる不備 | 打刻漏れ、時刻逆転、必須列欠落、未登録従業員、結合キー不足 | `issues.csv`, `profile_report.json`, データ準備状況レポート |
| `business_check` | データが読める状態で、業務担当者が確認すべき候補 | 長時間労働候補、連続勤務候補、有給取得不足候補、人員不足候補 | 勤怠確認レポート、月次労務レポート、人員不足確認レポート |
| `suppression` | 公開または画面表示前に非表示、伏せ字、集計単位変更が必要な状態 | 少人数部署、個人疲労値、個人推測可能な集計 | 抑制済み集計レポート、外部共有前チェックリスト |
| `join_assessment` | 分析目的に対して結合可能、限定集計、結合不可を示す判定 | 従業員 ID なし人件費と個人勤怠の結合不可 | 人件費粒度レポート、データ準備状況レポート |

### 1.3 原本保護

取り込んだ原本 CSV は変更してはならない。正規化、補完、推定、派生列生成は、必ず再生成可能な派生データに対して行う。

原本 CSV に対して行ってよい操作:

- 読み取り
- 入力ハッシュ算出
- ファイルサイズ、文字コード、区切り文字、ヘッダー有無などのメタデータ記録
- 原本参照 ID の付与

原本 CSV に対して行ってはならない操作:

- 上書き保存
- 列名の直接変更
- 値の直接修正
- 欠損値の直接補完
- 行の削除
- 並び替え後の上書き

### 1.4 すべての判定は再現可能にする

各判定結果は、少なくとも次を持たなければならない。

| 項目 | 必須 | 説明 |
| --- | --- | --- |
| `RunId` | 必須 | 実行単位 |
| `rule_version` | 必須 | 使用した業務ルール版 |
| `rule_id` | 必須 | 本書の業務ルール ID |
| `input_hash` | 必須 | 対象入力のハッシュ |
| `source_dataset_id` | 必須 | 元データ参照 |
| `source_row_number` | 原則必須 | 行単位の issue または確認候補の場合 |
| `source_column` | 条件付き必須 | 列単位の issue の場合 |
| `reason` | 必須 | 利用者に表示できる理由 |
| `detected_at` | 必須 | 検出時刻 |
| `rule_config_snapshot` | 必須 | 実行時の閾値と設定 |

## 2. 既定設定

### 2.1 `rule_config` の初期値

実装は、次の初期値を持つ `rule_config` を保存しなければならない。組織固有設定で上書きした場合も、実行時のスナップショットを成果物に保持する。

```yaml
rule_version: "business-rules-2026-06-02-draft"
locale: "ja-JP"
timezone: "Asia/Tokyo"

csv:
  max_file_size_mb: 1024
  accepted_encodings: ["UTF-8", "UTF-8-BOM", "Shift_JIS", "CP932"]
  accepted_delimiters: [",", "\t"]
  require_header: true

attendance:
  allow_cross_midnight_without_explicit_date: false
  infer_cross_midnight_when_end_before_start: false
  max_single_shift_minutes: 960
  max_single_shift_minutes_attention: 720
  min_plausible_work_minutes: 1
  duplicate_interval_tolerance_minutes: 0
  overlap_tolerance_minutes: 0
  missing_break_when_work_over_6h_is_issue: false

labor:
  standard_daily_minutes: 480
  standard_weekly_minutes: 2400
  break_required_over_6h_minutes: 45
  break_required_over_8h_minutes: 60
  overtime_month_attention_minutes: 2700
  overtime_year_attention_minutes: 21600
  overtime_month_health_attention_minutes: 4800
  overtime_month_critical_minutes: 6000
  overtime_year_special_attention_minutes: 43200
  overtime_month_over_45h_max_count: 6
  rolling_average_months: [2, 3, 4, 5, 6]
  consecutive_workdays_attention: 7
  consecutive_workdays_high: 14
  paid_leave_granted_days_threshold: 10
  paid_leave_required_days: 5
  paid_leave_warning_days_before_deadline: 90

privacy:
  min_group_size: 5
  dominance_ratio: 0.8
  suppress_complementary_cells: true
  suppress_health_related_individual_values: true
  suppress_person_identifying_comments: true

ops:
  staff_shortage_ratio_attention: 0.8
  staff_shortage_ratio_high: 0.6
  manager_load_concentration_ratio_attention: 0.5
  manager_load_concentration_ratio_high: 0.7

cost:
  allow_negative_cost_amount: false
  allow_person_level_join_without_employee_id: false

report:
  require_run_id: true
  require_target_period: true
  require_generated_at: true
  require_input_data_types: true
  prohibit_final_judgment_wording: true
```

### 2.2 優先度

`issue` の優先度は次の通りとする。

| 優先度 | 意味 | 既定の扱い |
| --- | --- | --- |
| `critical` | 対象処理を継続できない、または安全境界に違反する可能性が高い | 関連レポートを `blocked` にする |
| `high` | 主要レポートの正確性または安全性に大きく影響する | 関連レポートを原則 `partial` または `blocked` にする |
| `medium` | 一部の集計または確認項目に影響する | 関連レポートを `partial` にできる |
| `low` | 参考情報または軽微な修正候補 | レポート生成は継続可 |

`business_check` の確認レベルは次の通りとする。

| 確認レベル | 意味 | 表示方針 |
| --- | --- | --- |
| `watch` | 参考確認 | 通常の確認リストに表示 |
| `attention` | 担当者確認を推奨 | レポート上部の確認候補に表示 |
| `urgent` | 速やかな確認を推奨 | レポートの高優先確認候補に表示。ただし違法・診断・評価とは書かない |

## 3. ID 命名規則

### 3.1 業務ルール ID

業務ルール ID は、`BR-<領域>-<連番>` とする。

| 領域 | 例 | 内容 |
| --- | --- | --- |
| `CSV` | `BR-CSV-001` | CSV 取り込み、原本保護 |
| `SCHEMA` | `BR-SCHEMA-001` | 必須列、型、形式 |
| `DQ` | `BR-DQ-001` | データ品質検査 |
| `MASTER` | `BR-MASTER-001` | 従業員マスタ照合 |
| `GRAIN` | `BR-GRAIN-001` | 粒度分類 |
| `JOIN` | `BR-JOIN-001` | 結合可否 |
| `LABOR` | `BR-LABOR-001` | 労務確認候補 |
| `OPS` | `BR-OPS-001` | 店舗・部署運用確認 |
| `COST` | `BR-COST-001` | 人件費確認 |
| `PRIVACY` | `BR-PRIVACY-001` | 抑制、公開用出力 |
| `REPORT` | `BR-REPORT-001` | レポート生成 |
| `AI` | `BR-AI-001` | ガイド AI の回答制限 |

### 3.2 issue code

issue code は、`<CATEGORY>_<DETAIL>` とする。英大文字とアンダースコアのみを使う。

例:

- `SCHEMA_REQUIRED_COLUMN_MISSING`
- `SCHEMA_FORMAT_INVALID`
- `DQ_ATTENDANCE_CLOCK_IN_MISSING`
- `DQ_ATTENDANCE_CLOCK_OUT_MISSING`
- `DQ_ATTENDANCE_TIME_REVERSED`
- `DQ_ATTENDANCE_DUPLICATE_ROW`
- `DQ_ATTENDANCE_INTERVAL_OVERLAP`
- `MASTER_EMPLOYEE_NOT_FOUND`
- `GRAIN_INSUFFICIENT_FOR_PURPOSE`
- `JOIN_GRAIN_MISMATCH`
- `PRIVACY_SMALL_GROUP_SUPPRESSED`
- `PROCESSING_CSV_READ_FAILED`

### 3.3 business check code

業務確認コードは、`CHECK_<領域>_<DETAIL>` とする。

例:

- `CHECK_LABOR_OVERTIME_MONTH_45H`
- `CHECK_LABOR_OVERTIME_MONTH_80H_AVG`
- `CHECK_LABOR_OVERTIME_MONTH_100H`
- `CHECK_LABOR_CONSECUTIVE_WORKDAYS`
- `CHECK_LABOR_BREAK_SHORTAGE_CANDIDATE`
- `CHECK_LABOR_PAID_LEAVE_5DAYS_SHORTFALL`
- `CHECK_OPS_STAFF_SHORTAGE_CANDIDATE`
- `CHECK_OPS_MANAGER_LOAD_CONCENTRATION`

## 4. データ準備状態ルール

| Rule ID | 条件 | 状態 | 理由 | 関連要求 |
| --- | --- | --- | --- | --- |
| `BR-STATE-001` | 必須入力、必須列、結合キー、形式確認がすべて通過し、対象レポート生成に必要な粒度がある | `ready` | 主要レポートを生成できる | FLOW-005, FR-SCHEMA-001, FR-GRAIN-001 |
| `BR-STATE-002` | 一部の入力、列、粒度、マスタ照合に不足があるが、対象範囲を限定すればレポート生成できる | `partial` | 限定集計または一部除外で確認材料を作れる | FR-MASTER-003, FR-GRAIN-002 |
| `BR-STATE-003` | 必須入力がない、CSV が読めない、必須列がない、結合キーがない、安全境界を通せない | `blocked` | 対象レポートを生成できない | FR-CSV-005, FLOW-006 |
| `BR-STATE-004` | `critical` issue が存在し、対象レポートの根拠データが不確実 | `blocked` | 誤った確認材料を出すリスクがある | FR-DQ-003, NFR-UX-002 |
| `BR-STATE-005` | `high` issue が存在するが、当該行または当該集計単位を除外すればレポート生成できる | `partial` | 除外条件付きで利用可能 | FR-DQ-004 |

## 5. CSV 取り込み・原本保護ルール

| Rule ID | 条件 | 出力 | 優先度 | issue code | 関連要求 |
| --- | --- | --- | --- | --- | --- |
| `BR-CSV-001` | 指定ファイルが存在しない、または読み取り権限がない | `issue` | `critical` | `PROCESSING_CSV_READ_FAILED` | FR-CSV-001, FR-CSV-005 |
| `BR-CSV-002` | 文字コードを判定できない、または `rule_config.csv.accepted_encodings` に含まれない | `issue` | `high` | `PROCESSING_ENCODING_UNSUPPORTED` | FR-CSV-004 |
| `BR-CSV-003` | 区切り文字を判定できない、または許容区切り文字に含まれない | `issue` | `high` | `PROCESSING_DELIMITER_UNSUPPORTED` | FR-CSV-004 |
| `BR-CSV-004` | ヘッダー必須設定でヘッダーがない | `issue` | `high` | `SCHEMA_HEADER_MISSING` | FR-CSV-004, FR-SCHEMA-001 |
| `BR-CSV-005` | 入力ハッシュを算出できない | `issue` | `critical` | `PROCESSING_INPUT_HASH_FAILED` | FR-CSV-003 |
| `BR-CSV-006` | 原本保存後に入力ハッシュが変化した | `issue` | `critical` | `PROCESSING_RAW_CSV_MUTATED` | FR-CSV-002, FR-RECHECK-003 |
| `BR-CSV-007` | 同一 `RunId` 内で同じデータ種別に複数 CSV が指定され、結合ルールが未設定 | `issue` | `high` | `PROCESSING_DATASET_AMBIGUOUS` | FR-CSV-001 |

## 6. スキーマ確認ルール

### 6.1 共通必須列

`DATA-DESIGN.md` が確定するまでは、次を暫定的な標準列として扱う。列名の別名許容範囲は `DATA-DESIGN.md` で確定する。

| データ種別 | 暫定必須列 | 用途 |
| --- | --- | --- |
| `attendance` | `employee_id`, `work_date`, `clock_in`, `clock_out` | 勤怠検査、勤務時間計算 |
| `employee_master` | `employee_id`, `employee_name`, `department_id`, `employment_type`, `hire_date` | マスタ照合 |
| `labor_cost` | `period_start`, `period_end`, `amount`, `grain_level` | 人件費粒度確認 |
| `sales` | `store_id`, `sales_date`, `amount` | 店舗・時間帯分析 |
| `shift` | `employee_id`, `shift_date`, `scheduled_start`, `scheduled_end` | 予定対実績比較 |
| `fatigue` | `employee_id`, `observed_date`, `fatigue_value` または `fatigue_comment` | 抑制対象情報の確認 |

### 6.2 スキーマ issue

| Rule ID | 条件 | 出力 | 優先度 | issue code | 関連要求 |
| --- | --- | --- | --- | --- | --- |
| `BR-SCHEMA-001` | 対象データ種別の必須列が欠落している | `issue` | `critical` | `SCHEMA_REQUIRED_COLUMN_MISSING` | FR-SCHEMA-001 |
| `BR-SCHEMA-002` | 日付列が ISO 日付または許容日付形式に変換できない | `issue` | `high` | `SCHEMA_DATE_FORMAT_INVALID` | FR-SCHEMA-002 |
| `BR-SCHEMA-003` | 時刻列が許容時刻形式に変換できない | `issue` | `high` | `SCHEMA_TIME_FORMAT_INVALID` | FR-SCHEMA-002 |
| `BR-SCHEMA-004` | 数値列が数値に変換できない | `issue` | `high` | `SCHEMA_NUMBER_FORMAT_INVALID` | FR-SCHEMA-002 |
| `BR-SCHEMA-005` | ID 列が空文字、空白のみ、または正規化後に空になる | `issue` | `high` | `SCHEMA_ID_FORMAT_INVALID` | FR-SCHEMA-002 |
| `BR-SCHEMA-006` | 同一意味の列が複数存在し、標準列に一意に対応できない | `issue` | `high` | `SCHEMA_COLUMN_ALIAS_AMBIGUOUS` | FR-SCHEMA-003 |
| `BR-SCHEMA-007` | 未知の列があるが処理に影響しない | `issue` | `low` | `SCHEMA_UNKNOWN_COLUMN` | FR-SCHEMA-003 |

## 7. 勤怠データ品質ルール

### 7.1 日時正規化

勤怠行の日時は次の順で正規化する。

1. `work_date` と `clock_in` から `clock_in_at` を生成する。
2. `work_date` と `clock_out` から `clock_out_at` を生成する。
3. `clock_out_date`、`day_offset`、`next_day_flag` のいずれかがある場合は、それを優先して退勤日時を生成する。
4. 退勤時刻が出勤時刻より前で、日跨ぎを示す列がない場合は、既定では日跨ぎ推定を行わず、時刻逆転 issue とする。
5. `rule_config.attendance.infer_cross_midnight_when_end_before_start = true` の場合に限り、退勤日時に 1 日を加算できる。ただし、計算後の拘束時間が `max_single_shift_minutes` を超える場合は推定してはならない。

### 7.2 勤務時間計算

`gross_minutes` は `clock_out_at - clock_in_at` の分数とする。

`break_minutes` は次の優先順位で決定する。

1. 明示的な `break_minutes`
2. `break_start` と `break_end` から算出した分数
3. 複数休憩区間がある場合は各休憩区間の合計
4. 休憩情報がない場合は `null`

`net_work_minutes` は、`break_minutes` がある場合は `gross_minutes - break_minutes`、ない場合は `gross_minutes` とする。ただし、休憩情報がない場合は、休憩を控除済みとみなしてはならない。レポートには「休憩情報未確認」と明示する。

### 7.3 打刻・時刻・重複 issue

| Rule ID | 条件 | 出力 | 優先度 | issue code | 関連要求 |
| --- | --- | --- | --- | --- | --- |
| `BR-DQ-ATT-001` | `employee_id` が欠損 | `issue` | `critical` | `DQ_ATTENDANCE_EMPLOYEE_ID_MISSING` | FR-DQ-001, FR-MASTER-001 |
| `BR-DQ-ATT-002` | `work_date` が欠損または日付化できない | `issue` | `critical` | `DQ_ATTENDANCE_WORK_DATE_INVALID` | FR-DQ-001 |
| `BR-DQ-ATT-003` | 出勤時刻が欠損 | `issue` | `high` | `DQ_ATTENDANCE_CLOCK_IN_MISSING` | FR-DQ-001 |
| `BR-DQ-ATT-004` | 退勤時刻が欠損 | `issue` | `high` | `DQ_ATTENDANCE_CLOCK_OUT_MISSING` | FR-DQ-001 |
| `BR-DQ-ATT-005` | 退勤日時が出勤日時より前で、日跨ぎ根拠がない | `issue` | `high` | `DQ_ATTENDANCE_TIME_REVERSED` | FR-DQ-001 |
| `BR-DQ-ATT-006` | `gross_minutes <= 0` | `issue` | `high` | `DQ_ATTENDANCE_NON_POSITIVE_DURATION` | FR-DQ-001 |
| `BR-DQ-ATT-007` | `gross_minutes > max_single_shift_minutes` | `issue` | `high` | `DQ_ATTENDANCE_WORK_DURATION_OUTLIER` | FR-DQ-001 |
| `BR-DQ-ATT-008` | 同一 employee_id、work_date、clock_in_at、clock_out_at の完全重複がある | `issue` | `medium` | `DQ_ATTENDANCE_DUPLICATE_ROW` | FR-DQ-001 |
| `BR-DQ-ATT-009` | 同一 employee_id の勤務区間が重複している | `issue` | `high` | `DQ_ATTENDANCE_INTERVAL_OVERLAP` | FR-DQ-001 |
| `BR-DQ-ATT-010` | `break_minutes < 0` または休憩区間が逆転している | `issue` | `high` | `DQ_ATTENDANCE_BREAK_INVALID` | FR-DQ-001 |
| `BR-DQ-ATT-011` | `break_minutes > gross_minutes` | `issue` | `high` | `DQ_ATTENDANCE_BREAK_EXCEEDS_DURATION` | FR-DQ-001 |
| `BR-DQ-ATT-012` | `net_work_minutes` が `min_plausible_work_minutes` 未満 | `issue` | `medium` | `DQ_ATTENDANCE_WORK_DURATION_TOO_SHORT` | FR-DQ-001 |
| `BR-DQ-ATT-013` | `work_date` が実行日より未来で、将来予定データとして指定されていない | `issue` | `medium` | `DQ_ATTENDANCE_FUTURE_DATE` | FR-DQ-001 |

### 7.4 休憩確認候補

休憩情報が存在する場合、次を業務確認候補として出力する。休憩情報が存在しない場合、休憩不足を断定してはならない。

| Rule ID | 条件 | 出力 | 確認レベル | check code | 関連要求 |
| --- | --- | --- | --- | --- | --- |
| `BR-LABOR-BREAK-001` | `net_work_minutes > 360` かつ `break_minutes < 45` | `business_check` | `attention` | `CHECK_LABOR_BREAK_SHORTAGE_CANDIDATE` | FR-LABOR-001 |
| `BR-LABOR-BREAK-002` | `net_work_minutes > 480` かつ `break_minutes < 60` | `business_check` | `urgent` | `CHECK_LABOR_BREAK_SHORTAGE_CANDIDATE` | FR-LABOR-001 |
| `BR-LABOR-BREAK-003` | `gross_minutes > 360` かつ休憩情報がない | `issue` | `medium` | `DQ_ATTENDANCE_BREAK_DATA_MISSING` | FR-DQ-001 |

## 8. 従業員マスタ照合ルール

| Rule ID | 条件 | 出力 | 優先度 | issue code | 関連要求 |
| --- | --- | --- | --- | --- | --- |
| `BR-MASTER-001` | 従業員マスタに同一 `employee_id` が複数ある | `issue` | `critical` | `MASTER_EMPLOYEE_ID_DUPLICATE` | FR-MASTER-001 |
| `BR-MASTER-002` | 勤怠、人件費、疲労関連データに含まれる `employee_id` がマスタに存在しない | `issue` | `high` | `MASTER_EMPLOYEE_NOT_FOUND` | FR-MASTER-001, FR-MASTER-002 |
| `BR-MASTER-003` | 勤務日が `hire_date` より前 | `issue` | `high` | `MASTER_EMPLOYMENT_DATE_CONFLICT` | FR-MASTER-002 |
| `BR-MASTER-004` | 勤務日が `termination_date` より後 | `issue` | `high` | `MASTER_EMPLOYMENT_DATE_CONFLICT` | FR-MASTER-002 |
| `BR-MASTER-005` | 勤怠データの部署 ID とマスタ部署 ID が一致しない | `issue` | `medium` | `MASTER_DEPARTMENT_MISMATCH` | FR-MASTER-002 |
| `BR-MASTER-006` | 勤怠データの雇用区分とマスタ雇用区分が一致しない | `issue` | `medium` | `MASTER_EMPLOYMENT_TYPE_MISMATCH` | FR-MASTER-002 |
| `BR-MASTER-007` | マスタの `department_id` が欠損している | `issue` | `high` | `MASTER_DEPARTMENT_ID_MISSING` | FR-MASTER-001 |
| `BR-MASTER-008` | マスタの `employment_type` が欠損している | `issue` | `medium` | `MASTER_EMPLOYMENT_TYPE_MISSING` | FR-MASTER-001 |

マスタ不一致がある場合、関連レポートの状態は次の通りとする。

| 条件 | 状態 |
| --- | --- |
| `employee_id` が未登録で、個人別または部署別集計の所属を確定できない | `blocked` または当該行除外の `partial` |
| 部署不一致があるが、マスタ優先または入力優先の方針が `rule_config` に記録されている | `partial` |
| 雇用区分不一致があるが、対象レポートに雇用区分を使わない | `partial` または `ready` |

## 9. 粒度分類ルール

### 9.1 粒度ベクトル

各データセットは、次の粒度ベクトルを持つ。

| 粒度軸 | 値の例 | 意味 |
| --- | --- | --- |
| `entity_grain` | `employee`, `department`, `store`, `company`, `unknown` | どの主体単位か |
| `time_grain` | `timestamp`, `shift`, `day`, `week`, `month`, `year`, `period`, `unknown` | どの時間単位か |
| `org_grain` | `department`, `store`, `company`, `none`, `unknown` | 組織単位の有無 |
| `measure_grain` | `row`, `amount`, `count`, `hours`, `score`, `comment` | 値の意味 |
| `privacy_class` | `public_candidate`, `personal`, `health_related`, `sensitive_comment` | 表示前の抑制要否 |

### 9.2 粒度判定

| Rule ID | 条件 | 出力 | 優先度 | issue code | 関連要求 |
| --- | --- | --- | --- | --- | --- |
| `BR-GRAIN-001` | 必要な粒度軸を判定できない | `issue` | `high` | `GRAIN_UNKNOWN` | FR-GRAIN-001 |
| `BR-GRAIN-002` | 分析目的に必要な `entity_grain` より粗い | `join_assessment` | - | `GRAIN_INSUFFICIENT_FOR_PURPOSE` | FR-GRAIN-002 |
| `BR-GRAIN-003` | 分析目的に必要な `time_grain` より粗い | `join_assessment` | - | `GRAIN_INSUFFICIENT_FOR_PURPOSE` | FR-GRAIN-002 |
| `BR-GRAIN-004` | 個人単位分析に対し、データが部署・店舗・会社単位のみ | `join_assessment` | - | `GRAIN_PERSON_LEVEL_UNAVAILABLE` | FR-GRAIN-002, FR-GRAIN-003 |
| `BR-GRAIN-005` | 部署別集計に対し、部署 ID がなく会社単位のみ | `join_assessment` | - | `GRAIN_DEPARTMENT_LEVEL_UNAVAILABLE` | FR-GRAIN-002 |

### 9.3 粒度変換の許容範囲

| 変換 | 許容 | 条件 |
| --- | --- | --- |
| 個人日次勤怠 → 部署月次集計 | 可 | マスタで部署を確定でき、対象月へ集計できる |
| 個人日次勤怠 → 店舗時間帯集計 | 可 | 店舗 ID と時間帯が確定できる |
| 部署月次人件費 → 個人日次勤怠 | 不可 | 粗い集計を個人へ按分してはならない |
| 店舗日次売上 → 個人時間帯売上 | 不可 | 店舗売上を個人へ推定配賦してはならない |
| 月次人件費 → 部署月次勤怠集計 | 限定可 | 同一部署・同一対象月であり、個人別に展開しない |
| 会社月次人件費 → 部署月次勤怠集計 | 不可 | 部署別の金額根拠がない |

## 10. 結合可否ルール

### 10.1 結合分類

| 分類 | 意味 | 表示文言 |
| --- | --- | --- |
| `joinable` | 必要なキーと粒度が揃い、目的に対して結合できる | `結合可能` |
| `limited_aggregate` | 個人展開はできないが、限定された集計単位なら利用できる | `限定集計` |
| `not_joinable` | 必要なキーまたは粒度が不足し、目的に対して結合してはならない | `結合不可` |
| `blocked_by_privacy` | 技術的には集計できるが、少人数・健康関連・個人推測リスクにより表示できない | `抑制により表示不可` |

### 10.2 結合判定

| Rule ID | 条件 | 出力 | 優先度 | issue code | 関連要求 |
| --- | --- | --- | --- | --- | --- |
| `BR-JOIN-001` | 目的が個人勤怠との結合で、相手データに `employee_id` がない | `join_assessment` | - | `JOIN_EMPLOYEE_ID_MISSING` | FR-GRAIN-003, FR-COST-004 |
| `BR-JOIN-002` | 結合キーが存在するが null または空値がある | `issue` | `high` | `JOIN_KEY_MISSING` | FR-GRAIN-004 |
| `BR-JOIN-003` | 結合キーの型または正規化形式が一致しない | `issue` | `high` | `JOIN_KEY_FORMAT_MISMATCH` | FR-GRAIN-004 |
| `BR-JOIN-004` | 一対一を想定する結合で相手側に重複キーがある | `issue` | `high` | `JOIN_KEY_DUPLICATE` | FR-GRAIN-004 |
| `BR-JOIN-005` | 分析目的より粗い粒度のデータを細かい粒度へ展開しようとしている | `join_assessment` | - | `JOIN_GRAIN_MISMATCH` | FR-GRAIN-002, FR-COST-004 |
| `BR-JOIN-006` | 結合後の集計単位が少人数抑制対象になる | `suppression` | - | `PRIVACY_SMALL_GROUP_SUPPRESSED` | FR-PRIVACY-002 |
| `BR-JOIN-007` | 結合結果に含まれるデータ種別、期間、粒度を記録できない | `issue` | `high` | `JOIN_LINEAGE_MISSING` | FR-GRAIN-005 |

## 11. 長時間労働・労務確認ルール

### 11.1 労働時間候補の計算

LaborLens は、法的な時間外労働を確定計算するシステムではない。次の `labor_check_minutes` は、確認候補を出すための内部値である。

`labor_check_minutes` の算出順序:

1. シフトまたは所定労働時間がある場合: `max(0, net_work_minutes - scheduled_work_minutes)`
2. シフトがなく、日次実績だけがある場合: `max(0, net_work_minutes - rule_config.labor.standard_daily_minutes)`
3. 週次確認では、週内の `net_work_minutes` 合計から `standard_weekly_minutes` を超える分も別途算出する。
4. 変形労働時間制、裁量労働制、管理監督者、業種特例が設定されている場合は、`rule_config` の上書き値を使用する。
5. 算出根拠が不足する場合は、確認候補を出さず、`partial` または `issue` として不足理由を表示する。

### 11.2 月次・年次長時間労働候補

| Rule ID | 条件 | 出力 | 確認レベル | check code | 関連要求 |
| --- | --- | --- | --- | --- | --- |
| `BR-LABOR-OT-001` | 月次 `labor_check_minutes > 2700` | `business_check` | `attention` | `CHECK_LABOR_OVERTIME_MONTH_45H` | FR-LABOR-001 |
| `BR-LABOR-OT-002` | 年次 `labor_check_minutes > 21600` | `business_check` | `attention` | `CHECK_LABOR_OVERTIME_YEAR_360H` | FR-LABOR-001 |
| `BR-LABOR-OT-003` | 2〜6 か月平均の月次 `labor_check_minutes > 4800` | `business_check` | `urgent` | `CHECK_LABOR_OVERTIME_MONTH_80H_AVG` | FR-LABOR-001 |
| `BR-LABOR-OT-004` | 月次 `labor_check_minutes >= 6000` | `business_check` | `urgent` | `CHECK_LABOR_OVERTIME_MONTH_100H` | FR-LABOR-001 |
| `BR-LABOR-OT-005` | 年次 `labor_check_minutes > 43200` | `business_check` | `urgent` | `CHECK_LABOR_OVERTIME_YEAR_720H` | FR-LABOR-001 |
| `BR-LABOR-OT-006` | 月次 2700 分超の月数が年 6 か月を超える | `business_check` | `urgent` | `CHECK_LABOR_OVERTIME_45H_OVER_6_MONTHS` | FR-LABOR-001 |
| `BR-LABOR-OT-007` | `labor_check_minutes` の算出に必要な勤怠またはシフトデータが不足 | `issue` | `medium` | `DQ_LABOR_CHECK_INPUT_INSUFFICIENT` | FR-LABOR-003 |

表示文言は次の形式とする。

```text
対象者または対象部署について、対象期間の勤怠データ上、長時間労働の確認候補があります。これは適法・違法を判定するものではありません。対象期間、計算方法、シフト情報、36協定、就業規則、休暇・休日の扱いを確認してください。
```

### 11.3 連続勤務候補

| Rule ID | 条件 | 出力 | 確認レベル | check code | 関連要求 |
| --- | --- | --- | --- | --- | --- |
| `BR-LABOR-CONT-001` | 同一 employee_id で `net_work_minutes > 0` の日が 7 日以上連続 | `business_check` | `attention` | `CHECK_LABOR_CONSECUTIVE_WORKDAYS` | FR-LABOR-001 |
| `BR-LABOR-CONT-002` | 同一 employee_id で `net_work_minutes > 0` の日が 14 日以上連続 | `business_check` | `urgent` | `CHECK_LABOR_CONSECUTIVE_WORKDAYS_HIGH` | FR-LABOR-001 |
| `BR-LABOR-CONT-003` | 休日データがなく、勤務日のみから連続勤務を推定している | `issue` | `low` | `DQ_HOLIDAY_DATA_MISSING_FOR_CONTINUOUS_WORK_CHECK` | FR-LABOR-003 |

### 11.4 有給取得状況候補

| Rule ID | 条件 | 出力 | 確認レベルまたは優先度 | code | 関連要求 |
| --- | --- | --- | --- | --- | --- |
| `BR-LABOR-LEAVE-001` | 年次有給休暇付与日数が 10 日以上、基準日から 1 年以内の取得日数が 5 日未満、かつ期限到来済み | `business_check` | `urgent` | `CHECK_LABOR_PAID_LEAVE_5DAYS_SHORTFALL` | FR-LABOR-001 |
| `BR-LABOR-LEAVE-002` | 年次有給休暇付与日数が 10 日以上、期限まで 90 日以内、取得日数が 5 日未満 | `business_check` | `attention` | `CHECK_LABOR_PAID_LEAVE_5DAYS_AT_RISK` | FR-LABOR-001 |
| `BR-LABOR-LEAVE-003` | 付与日、付与日数、取得日数、基準日のいずれかが欠損し、確認できない | `issue` | `medium` | `DQ_PAID_LEAVE_DATA_INSUFFICIENT` | FR-LABOR-003 |
| `BR-LABOR-LEAVE-004` | 時間単位年休が含まれるが日換算ルールが設定されていない | `issue` | `medium` | `DQ_PAID_LEAVE_HOURLY_CONVERSION_MISSING` | FR-LABOR-003 |

有給取得状況の表示では、対象者名の表示可否を権限と抑制ルールに従って判定する。外部共有用出力では、個人名を原則表示しない。

## 12. 店舗・部署運用確認ルール

店舗・部署運用確認は、採用、応援、配置見直しの検討材料を提示するためのものであり、店長または従業員個人の評価として出力してはならない。

| Rule ID | 条件 | 出力 | 確認レベル | check code | 関連要求 |
| --- | --- | --- | --- | --- | --- |
| `BR-OPS-001` | 必要人数データがあり、実績人数 ÷ 必要人数 < 0.8 | `business_check` | `attention` | `CHECK_OPS_STAFF_SHORTAGE_CANDIDATE` | FR-OPS-001 |
| `BR-OPS-002` | 必要人数データがあり、実績人数 ÷ 必要人数 < 0.6 | `business_check` | `urgent` | `CHECK_OPS_STAFF_SHORTAGE_HIGH` | FR-OPS-001 |
| `BR-OPS-003` | 必要人数データがなく、売上・客数・シフトから人員不足を推定しようとしている | `join_assessment` | - | `GRAIN_REQUIRED_STAFFING_UNAVAILABLE` | FR-OPS-001 |
| `BR-OPS-004` | 店長または特定個人の勤務時間が部署・店舗合計の 50% を超える | `business_check` | `attention` | `CHECK_OPS_MANAGER_LOAD_CONCENTRATION` | FR-OPS-002 |
| `BR-OPS-005` | 店長または特定個人の勤務時間が部署・店舗合計の 70% を超える | `business_check` | `urgent` | `CHECK_OPS_MANAGER_LOAD_CONCENTRATION_HIGH` | FR-OPS-002 |
| `BR-OPS-006` | 店舗・部署単位の人数が `privacy.min_group_size` 未満 | `suppression` | - | `PRIVACY_SMALL_GROUP_SUPPRESSED` | FR-PRIVACY-002 |

禁止文言:

- `店長の能力不足`
- `配置不適性`
- `懲戒対象`
- `低評価`
- `問題社員`

許容文言:

- `特定担当者への勤務時間集中候補`
- `人員不足の確認候補`
- `応援・採用・シフト調整の検討材料`

## 13. 人件費・経理確認ルール

人件費データは給与計算の確定結果として扱わない。LaborLens は、粒度、結合可否、集計結果を確認材料として提示する。

| Rule ID | 条件 | 出力 | 優先度 | issue code | 関連要求 |
| --- | --- | --- | --- | --- | --- |
| `BR-COST-001` | `amount` が欠損または数値化できない | `issue` | `high` | `COST_AMOUNT_INVALID` | FR-COST-001 |
| `BR-COST-002` | `amount < 0` かつ負値許容設定がない | `issue` | `medium` | `COST_AMOUNT_NEGATIVE` | FR-COST-001 |
| `BR-COST-003` | `period_start` または `period_end` が欠損 | `issue` | `high` | `COST_PERIOD_MISSING` | FR-COST-001 |
| `BR-COST-004` | 人件費データに `employee_id` がなく、個人勤怠へ結合しようとしている | `join_assessment` | - | `JOIN_EMPLOYEE_ID_MISSING` | FR-COST-004 |
| `BR-COST-005` | 人件費が月次部署単位、勤怠が個人日次単位で、個人別人件費を出そうとしている | `join_assessment` | - | `JOIN_GRAIN_MISMATCH` | FR-COST-004 |
| `BR-COST-006` | 人件費と勤怠の対象期間が重ならない | `issue` | `high` | `COST_ATTENDANCE_PERIOD_MISMATCH` | FR-COST-001 |
| `BR-COST-007` | 人件費データと勤怠データを限定集計する場合、期間・部署・雇用区分が一致する | `join_assessment` | - | `LIMITED_AGGREGATE_ALLOWED` | FR-COST-002, FR-COST-004 |

人件費レポートは、次の注意文を含めなければならない。

```text
この人件費確認は、入力 CSV に基づく確認材料です。給与計算の確定処理、支給額の確定、会計仕訳の確定を行うものではありません。
```

## 14. プライバシー抑制ルール

### 14.1 抑制対象

| 対象 | 扱い |
| --- | --- |
| 個人疲労値 | ユーザー向け出力に平文表示しない |
| 睡眠時間 | ユーザー向け出力に平文表示しない |
| 疲労コメント | ユーザー向け出力に平文表示しない |
| 個人別疲労ランキング | 生成しない |
| 少人数部署の健康関連集計 | 抑制する |
| 個人が推測され得る集計 | 抑制する |
| 個人特定可能な自由記述 | 伏せ字または非表示 |
| 外部共有用の氏名、社員番号、メール | 既定で非表示または伏せ字 |

### 14.2 少人数抑制

| Rule ID | 条件 | 出力 | issue code | 関連要求 |
| --- | --- | --- | --- | --- |
| `BR-PRIVACY-001` | 集計単位のユニーク従業員数 `< privacy.min_group_size` | `suppression` | `PRIVACY_SMALL_GROUP_SUPPRESSED` | FR-PRIVACY-002, FR-PRIVACY-006 |
| `BR-PRIVACY-002` | 健康関連データを含む集計単位のユニーク従業員数 `< privacy.min_group_size` | `suppression` | `PRIVACY_HEALTH_SMALL_GROUP_SUPPRESSED` | FR-PRIVACY-001, FR-PRIVACY-002 |
| `BR-PRIVACY-003` | 集計値の 80% 以上を 1 人が占める | `suppression` | `PRIVACY_DOMINANCE_SUPPRESSED` | FR-PRIVACY-002 |
| `BR-PRIVACY-004` | 合計値と他の表示セルから抑制セルを逆算できる | `suppression` | `PRIVACY_COMPLEMENTARY_SUPPRESSED` | FR-PRIVACY-002 |
| `BR-PRIVACY-005` | 個人疲労値、睡眠時間、疲労コメントが公開用出力候補に含まれる | `issue` | `PRIVACY_HEALTH_FIELD_EXPOSED` | FR-PRIVACY-001, FR-PRIVACY-004 |
| `BR-PRIVACY-006` | ガイド AI の参照対象に抑制前データが含まれる | `issue` | `PRIVACY_AI_SOURCE_UNFILTERED` | FR-PRIVACY-005 |

### 14.3 抑制後の表示

抑制された値は、次のいずれかで表示する。

| 表示状態 | 表示例 | 用途 |
| --- | --- | --- |
| `hidden` | `非表示` | 値を一切出さない |
| `masked` | `***` | 表形式の構造を維持する |
| `suppressed` | `抑制済み` | 抑制理由を示す |
| `aggregated` | `部署単位ではなく会社単位で表示` | より粗い粒度に変更する |
| `omitted` | 行自体を出さない | 外部共有用出力 |

抑制理由は、可能な範囲で次の形式で表示する。

```text
この集計は、対象人数が少ない、または個人が推測される可能性があるため抑制しました。
```

健康関連情報では、対象人数や構成情報自体が個人推測につながる場合がある。その場合、詳細な人数や閾値を表示してはならない。

## 15. レポート生成ルール

### 15.1 共通メタデータ

すべてのレポートは、次を持たなければならない。

| 項目 | 必須 | 説明 |
| --- | --- | --- |
| `RunId` | 必須 | 実行 ID |
| `rule_version` | 必須 | ルール版 |
| `target_period_start` | 必須 | 対象期間開始 |
| `target_period_end` | 必須 | 対象期間終了 |
| `input_data_types` | 必須 | 使用したデータ種別 |
| `input_hashes` | 必須 | 使用した入力ハッシュ |
| `generated_at` | 必須 | 生成時刻 |
| `preparation_status` | 必須 | `ready`, `partial`, `blocked` |
| `limitations` | 条件付き必須 | `partial` または抑制がある場合 |
| `suppression_summary` | 条件付き必須 | 抑制がある場合 |

### 15.2 出力分類

レポートは、次のセクションを分離しなければならない。

| セクション | 内容 |
| --- | --- |
| `data_quality_issues` | CSV、形式、打刻、マスタ、粒度、結合などの issue |
| `business_checks` | 長時間労働、人員不足、有給取得などの確認候補 |
| `join_assessments` | 結合可能、限定集計、結合不可 |
| `privacy_suppressions` | 抑制件数、抑制理由、表示状態 |
| `limitations` | データ不足、対象外条件、法的判断ではない旨 |

### 15.3 禁止出力

| Rule ID | 条件 | 出力 | 優先度 | issue code | 関連要求 |
| --- | --- | --- | --- | --- | --- |
| `BR-REPORT-001` | レポート本文に適法・違法の断定表現が含まれる | `issue` | `critical` | `REPORT_FINAL_LEGAL_JUDGMENT_WORDING` | FR-REPORT-004 |
| `BR-REPORT-002` | 医療診断または治療指示として読める表現が含まれる | `issue` | `critical` | `REPORT_MEDICAL_JUDGMENT_WORDING` | SAFETY-003 |
| `BR-REPORT-003` | 個人評価、配置適性、懲戒対象として読める表現が含まれる | `issue` | `critical` | `REPORT_HR_EVALUATION_WORDING` | SAFETY-005 |
| `BR-REPORT-004` | 外部共有可否を最終判断する表現が含まれる | `issue` | `high` | `REPORT_EXTERNAL_SHARING_FINAL_JUDGMENT` | FR-REPORT-004 |
| `BR-REPORT-005` | `RunId`、対象期間、入力データ種別、生成時刻のいずれかが欠損 | `issue` | `critical` | `REPORT_METADATA_MISSING` | FR-REPORT-002 |
| `BR-REPORT-006` | 根拠データまたは集計条件を確認できない | `issue` | `high` | `REPORT_EVIDENCE_MISSING` | FR-REPORT-005 |

## 16. ガイド AI ルール

ガイド AI は、製品ドキュメント、レポート定義、制約条件、生成済みレポートに基づいて説明する補助機能である。判断を拡張してはならない。

| Rule ID | 条件 | 出力 | 優先度 | issue code | 関連要求 |
| --- | --- | --- | --- | --- | --- |
| `BR-AI-001` | ガイド AI が抑制前データを直接参照しようとしている | `issue` | `critical` | `PRIVACY_AI_SOURCE_UNFILTERED` | FR-PRIVACY-005 |
| `BR-AI-002` | ガイド AI が個人疲労値、睡眠時間、疲労コメントを回答に含めようとしている | `issue` | `critical` | `AI_HEALTH_FIELD_EXPOSURE` | SAFETY-002, SAFETY-007 |
| `BR-AI-003` | ガイド AI が適法・違法を断定している | `issue` | `critical` | `AI_FINAL_LEGAL_JUDGMENT` | SAFETY-004 |
| `BR-AI-004` | ガイド AI が医療診断または治療指示をしている | `issue` | `critical` | `AI_MEDICAL_JUDGMENT` | SAFETY-003 |
| `BR-AI-005` | ガイド AI が個人評価、配置適性、懲戒判断をしている | `issue` | `critical` | `AI_HR_EVALUATION` | SAFETY-005 |
| `BR-AI-006` | ガイド AI が根拠文書またはレポート箇所を示さずに仕様を説明している | `issue` | `medium` | `AI_EVIDENCE_MISSING` | NFR-UX-004 |

許容回答例:

```text
この項目は、勤怠データ上の長時間労働確認候補です。適法・違法を判断するものではありません。対象期間、計算方法、就業規則、36協定の内容を確認してください。
```

拒否または制限回答例:

```text
この質問は、個人の健康関連情報または人事評価につながるため回答できません。抑制済み集計または公開用レポートで確認してください。
```

## 17. 成果物別ルール対応

| 成果物 | 必ず適用するルール群 |
| --- | --- |
| `run_summary.json` | `BR-CSV-*`, `BR-STATE-*`, `BR-REPORT-*` |
| `issues.csv` | `BR-SCHEMA-*`, `BR-DQ-*`, `BR-MASTER-*`, `BR-GRAIN-*`, `BR-JOIN-*`, `BR-PRIVACY-*` |
| `profile_report.json` | `BR-CSV-*`, `BR-SCHEMA-*`, `BR-GRAIN-*` |
| データ準備状況レポート | `BR-STATE-*`, `BR-JOIN-*`, `BR-REPORT-*` |
| 勤怠確認レポート | `BR-DQ-ATT-*`, `BR-LABOR-*`, `BR-PRIVACY-*`, `BR-REPORT-*` |
| 人件費粒度レポート | `BR-GRAIN-*`, `BR-JOIN-*`, `BR-COST-*`, `BR-REPORT-*` |
| 人員不足確認レポート | `BR-OPS-*`, `BR-PRIVACY-*`, `BR-REPORT-*` |
| 月次労務レポート | `BR-LABOR-*`, `BR-MASTER-*`, `BR-PRIVACY-*`, `BR-REPORT-*` |
| 抑制済み集計レポート | `BR-PRIVACY-*`, `BR-REPORT-*` |
| 外部共有前チェックリスト | `BR-PRIVACY-*`, `BR-REPORT-*` |
| 再確認結果 | `BR-CSV-*`, `BR-DQ-*`, `BR-MASTER-*`, `BR-RECHECK-*` |

## 18. 再確認ルール

| Rule ID | 条件 | 出力 | 優先度 | issue code | 関連要求 |
| --- | --- | --- | --- | --- | --- |
| `BR-RECHECK-001` | 修正後 CSV を原本と同じ入力として上書きしようとしている | `issue` | `critical` | `RECHECK_RAW_CSV_OVERWRITE_ATTEMPT` | FR-RECHECK-001, FR-RECHECK-003 |
| `BR-RECHECK-002` | 修正前後の `RunId` を関連付けられない | `issue` | `high` | `RECHECK_RUN_LINK_MISSING` | FR-RECHECK-002 |
| `BR-RECHECK-003` | 修正前後の入力ハッシュを比較できない | `issue` | `high` | `RECHECK_INPUT_HASH_MISSING` | FR-RECHECK-002 |
| `BR-RECHECK-004` | issue 件数の差分を算出できない | `issue` | `medium` | `RECHECK_ISSUE_DIFF_FAILED` | FR-RECHECK-004 |
| `BR-RECHECK-005` | 修正後に `critical` issue が残る | `issue` | `critical` | `RECHECK_CRITICAL_ISSUE_REMAINS` | FR-RECHECK-004 |

再確認結果は、修正依頼の完了確認材料であり、修正の妥当性を最終承認するものではない。

## 19. Lean 仕様化候補

本書のうち、Lean で優先して扱う候補は次の通りとする。

| 候補 | 期待する性質 | 関連ルール |
| --- | --- | --- |
| 原本保護 | 原本 CSV のハッシュが処理前後で変化しない | `BR-CSV-006` |
| 個人疲労値非表示 | 公開用出力に個人疲労値が現れない | `BR-PRIVACY-005` |
| 少人数抑制 | `min_group_size` 未満の集計が表示可能出力に現れない | `BR-PRIVACY-001` |
| 結合不可の保持 | 結合不可データが結合済みとして扱われない | `BR-JOIN-001`, `BR-JOIN-005` |
| 未登録従業員 issue | マスタに存在しない従業員 ID を含む入力が issue を生成する | `BR-MASTER-002` |
| issue と確認候補の分離 | `issue` と `business_check` が同じ出力分類に混在しない | `BR-REPORT-*` |
| 成果物メタデータ | すべての成果物が `RunId` と `rule_version` を持つ | `BR-REPORT-005` |

## 20. 未決事項

| ID | 未決事項 | 影響 | 扱う文書 |
| --- | --- | --- | --- |
| `OPEN-BR-001` | 標準 CSV 列名と別名許容範囲 | スキーマ確認、正規化、テストに影響 | `DATA-DESIGN.md` |
| `OPEN-BR-002` | 業種特例、変形労働時間制、管理監督者などの設定モデル | 長時間労働候補の精度に影響 | `DATA-DESIGN.md`, `ARCHITECTURE.md` |
| `OPEN-BR-003` | 店舗・部署別の必要人数データの標準形式 | 人員不足確認に影響 | `DATA-DESIGN.md` |
| `OPEN-BR-004` | 外部共有用レポートの具体的レイアウト | 抑制表示とチェックリストに影響 | `EXTERNAL-DESIGN.md` |
| `OPEN-BR-005` | ガイド AI が参照できるレポート範囲 | 抑制前データへのアクセス制御に影響 | `ARCHITECTURE.md`, `OPERATIONS.md` |
| `OPEN-BR-006` | 受け入れテスト用の架空データセット | ルール検証に影響 | `TEST-PLAN.md` |

## 21. 変更履歴

| Date | Version | 変更内容 |
| --- | --- | --- |
| 2026-06-02 | `business-rules-2026-06-02-draft` | 初版。要求仕様から、issue code、勤怠・労務・粒度・結合・抑制・レポート生成の業務ルールを定義。 |
