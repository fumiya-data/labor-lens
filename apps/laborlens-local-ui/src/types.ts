export type UseCaseDefinition = {
  use_case_id: string;
  button_label: string;
  title: string;
  actor: string;
  summary: string;
};

export type DemoDbSource = {
  table_name: string;
  seed_version: string;
  employee_count: number;
  note: string;
};

export type MetricCard = {
  label: string;
  value: string;
  unit: string;
  status: string;
};

export type UseCaseSampleRow = {
  subject: string;
  group: string;
  primary_value: string;
  status: string;
  note: string;
};

export type UseCaseSampleResponse = {
  use_case: UseCaseDefinition;
  source: DemoDbSource;
  metrics: MetricCard[];
  rows: UseCaseSampleRow[];
  findings: string[];
  next_actions: string[];
};

export type ArtifactListing = {
  artifact_name: string;
  stable_path: string;
  content_type: string;
};

export type RunResponse = {
  run_id: string;
  job_state: string;
  progress_percent: number;
  artifacts: ArtifactListing[];
  report_markdown_path: string;
  db_persistence_status: string;
};

export type RunHistoryItem = {
  run_id: string;
  progress_path: string;
  artifacts_path: string;
  report_path: string;
};

export type RunProgress = {
  run_id: string;
  job_state: string;
  progress_percent: number;
};

export type ProgressState = {
  status: string;
  value: number;
  message: string;
};

export type GuideMessageResponse = {
  answer: string;
  mode: string;
  safety_boundary: string;
  references: string[];
};

export type AttendanceReviewIssueSeverity = "critical" | "high" | "medium" | "low" | "info";

export type AttendanceReviewIssueStatus =
  | "open"
  | "needs_review"
  | "acknowledged"
  | "resolved"
  | "suppressed";

export type AttendanceReviewRowStatus =
  | "clean"
  | "warning"
  | "issue"
  | "blocked"
  | "reviewed";

export type AttendanceReviewSortDirection = "asc" | "desc";

export type AttendanceReviewSortField =
  | "employee_id"
  | "employee_name"
  | "work_date"
  | "store_name"
  | "department_name"
  | "status"
  | "highest_severity"
  | "issue_count"
  | "actual_minutes"
  | "overtime_minutes";

export type AttendanceReviewDashboardMetric = {
  key: string;
  label: string;
  value: number;
  unit?: string;
  status?: AttendanceReviewRowStatus;
  severity?: AttendanceReviewIssueSeverity;
  helper_text?: string;
};

export type AttendanceReviewIssueCount = {
  severity: AttendanceReviewIssueSeverity;
  count: number;
};

export type AttendanceReviewStatusCount = {
  status: AttendanceReviewIssueStatus | AttendanceReviewRowStatus;
  count: number;
};

export type AttendanceReviewGroupCount = {
  id: string;
  name: string;
  employee_count: number;
  row_count: number;
  issue_count: number;
  highest_severity?: AttendanceReviewIssueSeverity;
};

export type AttendanceReviewSummaryResponse = {
  run_id?: string;
  generated_at?: string;
  period_start?: string;
  period_end?: string;
  total_rows: number;
  reviewed_rows: number;
  issue_rows: number;
  clean_rows: number;
  metrics: AttendanceReviewDashboardMetric[];
  issue_counts_by_severity: AttendanceReviewIssueCount[];
  row_counts_by_status: AttendanceReviewStatusCount[];
  store_counts: AttendanceReviewGroupCount[];
  department_counts: AttendanceReviewGroupCount[];
};

export type AttendanceReviewIssue = {
  issue_id: string;
  issue_code: string;
  category: string;
  status: AttendanceReviewIssueStatus;
  severity: AttendanceReviewIssueSeverity;
  title: string;
  message: string;
  field_name?: string;
  source_column?: string;
  source_row_number?: number;
  suggested_action?: string;
};

export type AttendanceReviewRow = {
  row_id: string;
  employee_id: string;
  employee_name: string;
  work_date: string;
  store_id?: string;
  store_name?: string;
  department_id?: string;
  department_name?: string;
  employment_type?: string;
  status: AttendanceReviewRowStatus;
  highest_severity?: AttendanceReviewIssueSeverity;
  issue_count: number;
  issue_codes: string[];
  scheduled_start?: string;
  scheduled_end?: string;
  clock_in?: string;
  clock_out?: string;
  break_minutes?: number;
  actual_minutes?: number;
  overtime_minutes?: number;
  late_minutes?: number;
  early_leave_minutes?: number;
  source_file_name?: string;
  source_row_number?: number;
  search_text?: string;
  issues: AttendanceReviewIssue[];
};

export type AttendanceReviewRowsResponse = {
  rows: AttendanceReviewRow[];
  total_rows: number;
  filtered_rows: number;
  page: number;
  page_size: number;
  sort_field?: AttendanceReviewSortField;
  sort_direction?: AttendanceReviewSortDirection;
};

export type AttendanceReviewRowsQuery = {
  search?: string;
  status?: AttendanceReviewRowStatus | AttendanceReviewRowStatus[];
  severity?: AttendanceReviewIssueSeverity | AttendanceReviewIssueSeverity[];
  store_id?: string | string[];
  department_id?: string | string[];
  issue_code?: string | string[];
  work_date_from?: string;
  work_date_to?: string;
  sort_field?: AttendanceReviewSortField;
  sort_direction?: AttendanceReviewSortDirection;
  page?: number;
  page_size?: number;
};
