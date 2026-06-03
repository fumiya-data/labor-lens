# LaborLens 公開レポート

- 契約バージョン: `laborlens.public_report.v1`
- 実行 ID: `run-smoke-001`
- ポリシー: `privacy-safety-v1 (2026-06-03)`

## 実行サマリー

| 指標 | 値 |
| --- | --- |
| 従業員数 | 1 |
| プロファイル数 | 1 |
| 抑制カテゴリ数 | 1 |
| 抑制フィールド数 | 3 |
| issue 数 | 1 |

## グループプロファイル概要

| プロファイル ID | グループ | 従業員数 | 観測した勤怠日数 | 健康関連詳細の状態 |
| --- | --- | --- | --- | --- |
| group:operations | operations | 1 | 20 | suppressed |

## 抑制サマリー

| 抑制コード | カテゴリ | 影響レコード数 | 抑制フィールド数 | 理由 |
| --- | --- | --- | --- | --- |
| PERSONAL_HEALTH_DETAIL_SUPPRESSED | personal_health_detail | 1 | 3 | 個人の健康関連詳細は公開レポートの対象外である。 |

## 公開 issue

| 重要度 | issue ID | 抑制コード | メッセージ |
| --- | --- | --- | --- |
| info | issue:personal_health_detail_suppressed | PERSONAL_HEALTH_DETAIL_SUPPRESSED | 個人の健康関連詳細は公開レポート生成前に抑制された。 |

## 成果物一覧

### 入力トレース

| データセット | 入力元参照 | フィンガープリント | レコード数 |
| --- | --- | --- | --- |
| attendance_by_employee | fixtures/internal/attendance.csv | sha256:smoke-attendance | 1 |
| fatigue_by_employee | fixtures/internal/fatigue.csv | sha256:smoke-fatigue | 1 |

### 出力トレース

| 成果物 | 種別 | 安定パス | スキーマ |
| --- | --- | --- | --- |
| artifact_manifest | json | artifact_manifest.json | laborlens.artifact_manifest.v1 |
| run_summary | json | run_summary.json | laborlens.run_summary.v1 |
| issues | json | issues.json | laborlens.issues.v1 |
| profile_report | json | profile_report.json | laborlens.profile_report.v1 |

### ポリシートレース

| ポリシー項目 | 値 |
| --- | --- |
| policy_id | privacy-safety-v1 |
| version | 2026-06-03 |
| safety_boundary | 公開 artifact 生成前に個人の健康関連詳細を抑制する |
