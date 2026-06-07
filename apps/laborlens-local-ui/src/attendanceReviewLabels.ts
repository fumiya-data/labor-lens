import type { AttendanceReviewIssueSeverity } from "./types";

export function severityLabel(severity: AttendanceReviewIssueSeverity | string | undefined): string {
  switch (severity) {
    case "critical":
      return "最優先";
    case "high":
      return "高";
    case "medium":
      return "中";
    case "low":
      return "低";
    case "info":
      return "参考";
    default:
      return "-";
  }
}

export function statusLabel(status: string | undefined): string {
  switch (status) {
    case "clean":
    case "ok":
      return "問題なし";
    case "warning":
      return "注意";
    case "issue":
    case "needs_review":
      return "要確認";
    case "blocked":
      return "保留";
    case "reviewed":
      return "確認済み";
    case "open":
      return "未対応";
    case "acknowledged":
      return "確認中";
    case "resolved":
      return "解決済み";
    case "suppressed":
      return "非表示";
    default:
      return status || "-";
  }
}

export function issueCodeLabel(issueCode: string | undefined): string {
  switch (issueCode) {
    case "missing_clock_in":
      return "出勤打刻漏れ";
    case "missing_clock_out":
      return "退勤打刻漏れ";
    case "time_reversal":
      return "時刻逆転";
    case "duplicate_candidate":
      return "重複候補";
    case "long_hours_candidate":
      return "長時間勤務候補";
    case "master_department_mismatch_candidate":
      return "従業員マスタ部署不一致候補";
    case "master_inactive_employee_candidate":
      return "在籍状態確認候補";
    case "missing_employee":
      return "従業員マスタ未登録";
    case "missing_required_header":
      return "必須列不足";
    case "csv_read_error":
      return "CSV形式エラー";
    case "none":
      return "問題なし";
    default:
      return issueCode || "-";
  }
}

export function issueCategoryLabel(category: string | undefined): string {
  switch (category) {
    case "data_quality_issue":
      return "データ不備";
    case "business_check":
      return "業務確認";
    case "master_issue":
      return "マスタ確認";
    case "schema_issue":
      return "列・形式";
    default:
      return category || "-";
  }
}
