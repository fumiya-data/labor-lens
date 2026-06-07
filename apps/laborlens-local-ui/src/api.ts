import type {
  ArtifactListing,
  AttendanceReviewIssueSeverity,
  AttendanceReviewIssueStatus,
  AttendanceReviewRowsQuery,
  AttendanceReviewRowsResponse,
  AttendanceReviewRow,
  AttendanceReviewRowStatus,
  AttendanceReviewSummaryResponse,
  GuideMessageResponse,
  RunHistoryItem,
  RunProgress,
  RunResponse,
  UseCaseDefinition,
  UseCaseSampleResponse,
} from "./types";

const apiBase = import.meta.env.VITE_LABORLENS_API_BASE ?? "";
export const DEMO_ATTENDANCE_REVIEW_RUN_ID = "seed";

async function fetchJson<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(`${apiBase}${path}`, init);
  if (!response.ok) {
    let detail = "";
    try {
      const payload = (await response.json()) as { error?: string; message?: string };
      detail = payload.message ?? payload.error ?? "";
    } catch {
      detail = await response.text().catch(() => "");
    }
    throw new Error(detail ? `HTTP ${response.status}: ${detail}` : `HTTP ${response.status}`);
  }
  return response.json() as Promise<T>;
}

export function getUseCases(): Promise<UseCaseDefinition[]> {
  return fetchJson<UseCaseDefinition[]>("/api/use-cases");
}

export function getUseCaseSample(useCaseId: string): Promise<UseCaseSampleResponse> {
  return fetchJson<UseCaseSampleResponse>(
    `/api/use-cases/${encodeURIComponent(useCaseId)}/sample-data`,
  );
}

export function getAttendanceReviewSummary(runId?: string): Promise<AttendanceReviewSummaryResponse> {
  const path = `/api/attendance-review/summary${runId ? `?run_id=${encodeURIComponent(runId)}` : ""}`;
  return fetchJson<RawAttendanceReviewSummary>(path).then((summary) => ({
    run_id: summary.run_id ?? undefined,
    generated_at:
      summary.data_source === "seed"
        ? "seed: demo_japanese_employees.v1"
        : summary.run_id ?? undefined,
    period_start: summary.period_start,
    period_end: summary.period_end,
    total_rows: summary.row_count,
    reviewed_rows: summary.row_count,
    issue_rows: summary.issue_row_count,
    clean_rows: summary.row_count - summary.issue_row_count,
    metrics: [
      {
        key: "employees",
        label: "従業員",
        value: summary.employee_count,
        unit: "人",
        status: "clean",
        helper_text: summary.data_source === "seed" ? "seed 従業員マスタ" : "CSV run",
      },
      {
        key: "rows",
        label: "勤怠行",
        value: summary.row_count,
        unit: "行",
        status: "clean",
        helper_text: `${summary.period_start} - ${summary.period_end}`,
      },
      {
        key: "issues",
        label: "要確認",
        value: summary.issue_count ?? summary.issue_row_count,
        unit: "件",
        status: summary.issue_row_count > 0 ? "issue" : "clean",
        helper_text: "給与計算前のissue件数",
      },
      {
        key: "clean",
        label: "問題なし",
        value: summary.row_count - summary.issue_row_count,
        unit: "行",
        status: "clean",
        helper_text: "この画面で検索・比較可能",
      },
    ],
    issue_counts_by_severity: summary.counts_by_severity.map((count) => ({
      severity: normalizeSeverity(count.key),
      count: count.count,
    })),
    row_counts_by_status: summary.counts_by_status.map((count) => ({
      status: normalizeRowStatus(count.key),
      count: count.count,
    })),
    store_counts: summary.counts_by_store.map((count) => ({
      id: count.key,
      name: count.key,
      employee_count: 0,
      row_count: count.row_count ?? count.count,
      issue_count: count.issue_count ?? count.count,
    })),
    department_counts: summary.counts_by_department.map((count) => ({
      id: count.key,
      name: count.key,
      employee_count: 0,
      row_count: count.row_count ?? count.count,
      issue_count: count.issue_count ?? count.count,
    })),
  }));
}

export function getAttendanceReviewRows(
  query: AttendanceReviewRowsQuery = {},
  runId?: string,
): Promise<AttendanceReviewRowsResponse> {
  const effectiveQuery = { ...query, run_id: runId };
  return fetchJson<RawAttendanceReviewRowsResponse>(
    `/api/attendance-review/rows${toQueryString(effectiveQuery)}`,
  ).then((response) => {
    const rows = response.rows
      .map((row, index) => normalizeAttendanceReviewRow(row, index))
      .filter((row) => matchesAttendanceReviewQuery(row, query))
      .sort((left, right) => compareAttendanceReviewRows(left, right, query));
    const page = query.page ?? 1;
    const pageSize = query.page_size ?? rows.length;
    const start = Math.max(0, page - 1) * pageSize;
    const pagedRows = rows.slice(start, start + pageSize);
    return {
      rows: pagedRows,
      total_rows: response.rows.length,
      filtered_rows: rows.length,
      page,
      page_size: pageSize,
      sort_field: query.sort_field,
      sort_direction: query.sort_direction,
    };
  });
}

export function startRun(employeesCsv: File, attendanceCsv: File): Promise<RunResponse> {
  const body = new FormData();
  body.append("employees_csv", employeesCsv);
  body.append("attendance_csv", attendanceCsv);
  return fetchJson<RunResponse>("/api/runs", { method: "POST", body });
}

export function getRuns(): Promise<RunHistoryItem[]> {
  return fetchJson<RunHistoryItem[]>("/api/runs");
}

export function getRunProgress(runId: string): Promise<RunProgress> {
  return fetchJson<RunProgress>(`/api/runs/${encodeURIComponent(runId)}/progress`);
}

export function getRunArtifacts(runId: string): Promise<ArtifactListing[]> {
  return fetchJson<ArtifactListing[]>(`/api/runs/${encodeURIComponent(runId)}/artifacts`);
}

export function getPublicReport(runId: string): Promise<string> {
  return fetchText(`/api/runs/${encodeURIComponent(runId)}/reports/public-report`);
}

export async function fetchReport(path: string): Promise<string> {
  return fetchText(path);
}

export function postGuideMessage(message: string, runId?: string): Promise<GuideMessageResponse> {
  return fetchJson<GuideMessageResponse>("/api/guide/messages", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      message,
      run_id: runId || null,
    }),
  });
}

async function fetchText(path: string): Promise<string> {
  const response = await fetch(`${apiBase}${path}`);
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}`);
  }
  return response.text();
}

function toQueryString(query: AttendanceReviewRowsQuery): string {
  const params = new URLSearchParams();
  for (const [key, value] of Object.entries(query)) {
    if (value === undefined || value === null || value === "") {
      continue;
    }
    if (Array.isArray(value)) {
      for (const item of value) {
        params.append(key, String(item));
      }
      continue;
    }
    params.set(key, String(value));
  }
  const serialized = params.toString();
  return serialized ? `?${serialized}` : "";
}

type RawAttendanceReviewCount = {
  key: string;
  count: number;
  row_count?: number;
  issue_count?: number;
  affected_row_count?: number;
};

type RawAttendanceReviewSummary = {
  run_id?: string | null;
  data_source?: string;
  period_start: string;
  period_end: string;
  employee_count: number;
  row_count: number;
  issue_row_count: number;
  issue_count?: number;
  counts_by_store: RawAttendanceReviewCount[];
  counts_by_department: RawAttendanceReviewCount[];
  counts_by_severity: RawAttendanceReviewCount[];
  counts_by_status: RawAttendanceReviewCount[];
  counts_by_issue_type: RawAttendanceReviewCount[];
};

type RawAttendanceReviewRow = {
  row_id: string;
  employee_id: string;
  display_name: string;
  store_name: string;
  department: string;
  work_date: string;
  clock_in: string | null;
  clock_out: string | null;
  scheduled_clock_in: string;
  scheduled_clock_out: string;
  worked_minutes: number | null;
  issue_type: string;
  issue_label: string;
  severity: string;
  status: string;
  review_hint: string;
  source_ref?: string;
  source_row_number?: number;
  issues?: RawAttendanceReviewIssue[];
};

type RawAttendanceReviewRowsResponse = {
  run_id?: string | null;
  data_source?: string;
  period_start: string;
  period_end: string;
  rows: RawAttendanceReviewRow[];
};

type RawAttendanceReviewIssue = {
  issue_id: string;
  run_id?: string;
  dataset_kind?: string;
  source_ref?: string;
  source_row_number?: number;
  employee_id?: string;
  work_date?: string;
  store_name?: string;
  department_name?: string;
  issue_code: string;
  issue_category?: string;
  category?: string;
  severity: string;
  status: string;
  message: string;
  suggested_action?: string;
};

function normalizeAttendanceReviewRow(row: RawAttendanceReviewRow, index: number): AttendanceReviewRow {
  const status = normalizeRowStatus(row.status);
  const severity = normalizeSeverity(row.severity);
  const rawIssues = row.issues ?? [];
  const normalizedIssues = rawIssues.map((issue) => normalizeAttendanceReviewIssue(issue, row, index));
  const issueCount = rawIssues.length > 0 ? rawIssues.length : row.issue_type === "none" ? 0 : 1;
  return {
    row_id: row.row_id,
    employee_id: row.employee_id,
    employee_name: row.display_name,
    work_date: row.work_date,
    store_id: row.store_name,
    store_name: row.store_name,
    department_id: row.department,
    department_name: row.department,
    status,
    highest_severity: severity,
    issue_count: issueCount,
    issue_codes: rawIssues.length > 0 ? rawIssues.map((issue) => issue.issue_code) : issueCount ? [row.issue_type] : [],
    scheduled_start: row.scheduled_clock_in,
    scheduled_end: row.scheduled_clock_out,
    clock_in: row.clock_in ?? undefined,
    clock_out: row.clock_out ?? undefined,
    break_minutes: row.worked_minutes === null ? undefined : 60,
    actual_minutes: row.worked_minutes ?? undefined,
    overtime_minutes: row.worked_minutes === null ? undefined : Math.max(0, row.worked_minutes - 480),
    source_file_name: row.source_ref ?? "seed:demo_japanese_employees.v1",
    source_row_number: row.source_row_number ?? index + 2,
    search_text: [
      row.employee_id,
      row.display_name,
      row.store_name,
      row.department,
      row.work_date,
      row.issue_label,
      row.issue_type,
    ].join(" ").toLowerCase(),
    issues: rawIssues.length > 0
      ? normalizedIssues
      : issueCount
      ? [
          {
            issue_id: `${row.row_id}-${row.issue_type}`,
            issue_code: row.issue_type,
            category: row.issue_type.startsWith("master") ? "master_issue" : "data_quality_issue",
            status: "needs_review" as AttendanceReviewIssueStatus,
            severity,
            title: row.issue_label,
            message: row.review_hint,
            source_row_number: row.source_row_number ?? index + 2,
            suggested_action: row.review_hint,
          },
        ]
      : [],
  };
}

function normalizeAttendanceReviewIssue(
  issue: RawAttendanceReviewIssue,
  row: RawAttendanceReviewRow,
  index: number,
) {
  const issueCode = issue.issue_code;
  return {
    issue_id: issue.issue_id,
    issue_code: issueCode,
    category: issue.issue_category ?? issue.category ?? "data_quality_issue",
    status: normalizeIssueStatus(issue.status),
    severity: normalizeSeverity(issue.severity),
    title: issueCode,
    message: issue.message,
    source_row_number: issue.source_row_number ?? row.source_row_number ?? index + 2,
    suggested_action: issue.suggested_action,
  };
}

function normalizeIssueStatus(value: string): AttendanceReviewIssueStatus {
  if (
    value === "open" ||
    value === "needs_review" ||
    value === "acknowledged" ||
    value === "resolved" ||
    value === "suppressed"
  ) {
    return value;
  }
  return "open";
}

function normalizeRowStatus(value: string): AttendanceReviewRowStatus {
  if (value === "ok") return "clean";
  if (value === "blocked") return "blocked";
  if (value === "needs_review") return "issue";
  if (value === "reviewed") return "reviewed";
  return "warning";
}

function normalizeSeverity(value: string): AttendanceReviewIssueSeverity {
  if (value === "critical" || value === "high" || value === "medium" || value === "low") {
    return value;
  }
  return "info";
}

function matchesAttendanceReviewQuery(
  row: AttendanceReviewRow,
  query: AttendanceReviewRowsQuery,
): boolean {
  const search = query.search?.trim().toLowerCase();
  if (search && !(row.search_text ?? "").includes(search)) return false;
  if (query.status && !matchesOneOrMany(row.status, query.status)) return false;
  if (
    query.severity &&
    (!row.highest_severity || !matchesOneOrMany(row.highest_severity, query.severity))
  ) {
    return false;
  }
  if (query.store_id && !matchesOneOrMany(row.store_id ?? "", query.store_id)) return false;
  if (query.department_id && !matchesOneOrMany(row.department_id ?? "", query.department_id)) {
    return false;
  }
  if (query.issue_code && !row.issue_codes.some((code) => matchesOneOrMany(code, query.issue_code!))) {
    return false;
  }
  if (query.work_date_from && row.work_date < query.work_date_from) return false;
  if (query.work_date_to && row.work_date > query.work_date_to) return false;
  return true;
}

function matchesOneOrMany(value: string, expected: string | string[]): boolean {
  return Array.isArray(expected) ? expected.includes(value) : value === expected;
}

function compareAttendanceReviewRows(
  left: AttendanceReviewRow,
  right: AttendanceReviewRow,
  query: AttendanceReviewRowsQuery,
): number {
  const field = query.sort_field ?? "work_date";
  const direction = query.sort_direction === "desc" ? -1 : 1;
  if (field === "highest_severity") {
    return (severityRank(left.highest_severity) - severityRank(right.highest_severity)) * direction;
  }
  if (
    field === "issue_count" ||
    field === "actual_minutes" ||
    field === "overtime_minutes"
  ) {
    return ((left[field] ?? -1) - (right[field] ?? -1)) * direction;
  }
  return String(left[field] ?? "").localeCompare(String(right[field] ?? ""), "ja") * direction;
}

function severityRank(severity: string | undefined): number {
  return { critical: 5, high: 4, medium: 3, low: 2, info: 1 }[severity ?? "info"] ?? 0;
}
