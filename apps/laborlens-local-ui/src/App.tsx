import { useEffect, useMemo, useState } from "react";
import {
  fetchReport,
  getAttendanceReviewRows,
  getAttendanceReviewSummary,
  postGuideMessage,
  startRun,
} from "./api";
import { MetricCard } from "./components/MetricCard";
import { modeCategories, workModes } from "./modeNavigation";
import type { ModeCategoryId, WorkModeId } from "./modeNavigation";
import { ModePreviewScreen } from "./screens/ModePreviewScreen";
import type {
  ArtifactListing,
  AttendanceReviewIssueSeverity,
  AttendanceReviewRow,
  AttendanceReviewRowStatus,
  AttendanceReviewRowsQuery,
  AttendanceReviewSortDirection,
  AttendanceReviewSortField,
  AttendanceReviewSummaryResponse,
  GuideMessageResponse,
  ProgressState,
} from "./types";

const initialProgress: ProgressState = {
  status: "未実行",
  value: 0,
  message: "CSVを選ぶか、既存の勤怠確認を読み込んでください。",
};

const rowStatuses: AttendanceReviewRowStatus[] = ["clean", "warning", "issue", "blocked", "reviewed"];
const issueSeverities: AttendanceReviewIssueSeverity[] = ["critical", "high", "medium", "low", "info"];

const sortableColumns: Array<{ field: AttendanceReviewSortField; label: string }> = [
  { field: "employee_id", label: "社員ID" },
  { field: "employee_name", label: "氏名" },
  { field: "work_date", label: "日付" },
  { field: "store_name", label: "店舗" },
  { field: "department_name", label: "部門" },
  { field: "status", label: "状態" },
  { field: "highest_severity", label: "重要度" },
  { field: "issue_count", label: "問題" },
  { field: "actual_minutes", label: "労働時間" },
  { field: "overtime_minutes", label: "残業" },
];

export function App() {
  const [activeCategory, setActiveCategory] = useState<ModeCategoryId>("attendance");
  const [activeMode, setActiveMode] = useState<WorkModeId>("attendance");
  const [summary, setSummary] = useState<AttendanceReviewSummaryResponse | null>(null);
  const [rows, setRows] = useState<AttendanceReviewRow[]>([]);
  const [filteredRowCount, setFilteredRowCount] = useState(0);
  const [reviewStatus, setReviewStatus] = useState("未読込");
  const [search, setSearch] = useState("");
  const [statusFilter, setStatusFilter] = useState("");
  const [severityFilter, setSeverityFilter] = useState("");
  const [storeFilter, setStoreFilter] = useState("");
  const [departmentFilter, setDepartmentFilter] = useState("");
  const [dateFrom, setDateFrom] = useState("");
  const [dateTo, setDateTo] = useState("");
  const [sortField, setSortField] = useState<AttendanceReviewSortField>("work_date");
  const [sortDirection, setSortDirection] = useState<AttendanceReviewSortDirection>("desc");
  const [selectedRowId, setSelectedRowId] = useState<string | null>(null);
  const [progress, setProgress] = useState<ProgressState>(initialProgress);
  const [employeesCsv, setEmployeesCsv] = useState<File | null>(null);
  const [attendanceCsv, setAttendanceCsv] = useState<File | null>(null);
  const [artifacts, setArtifacts] = useState<ArtifactListing[]>([]);
  const [report, setReport] = useState("");
  const [activeRunId, setActiveRunId] = useState<string | undefined>();
  const [dbPersistenceStatus, setDbPersistenceStatus] = useState("not_started");
  const [guideMessage, setGuideMessage] = useState("");
  const [guideResponse, setGuideResponse] = useState<GuideMessageResponse | null>(null);

  useEffect(() => {
    void loadReview();
  }, []);

  const selectedRow = useMemo(
    () => rows.find((row) => row.row_id === selectedRowId) ?? rows[0] ?? null,
    [rows, selectedRowId],
  );

  const reviewCompletion = summary ? percent(summary.reviewed_rows, summary.total_rows) : 0;
  const issueRate = summary ? percent(summary.issue_rows, summary.total_rows) : 0;
  const currentMode = workModes.find((mode) => mode.id === activeMode) ?? workModes[0];
  const categoryModes = workModes.filter((mode) => mode.category === activeCategory);

  async function loadReview() {
    setReviewStatus("読み込み中");
    setProgressState("読み込み中", 25, "勤怠確認の集計と一覧を読み込んでいます。");
    try {
      const [nextSummary, nextRows] = await Promise.all([
        getAttendanceReviewSummary(),
        getAttendanceReviewRows(buildRowsQuery()),
      ]);
      applyReviewData(nextSummary, nextRows.rows, nextRows.filtered_rows);
      setReviewStatus("表示できます");
      setProgressState(
        "読み込み完了",
        100,
        `${nextRows.filtered_rows}件の勤怠データを表示できます。`,
      );
    } catch (error) {
      setReviewStatus("API 未接続");
      setProgressState("失敗", 100, `勤怠確認APIにつながりません: ${messageFor(error)}`);
    }
  }

  async function loadRows() {
    await loadRowsWithQuery(buildRowsQuery(), "検索条件を勤怠一覧に反映しています。");
  }

  async function loadRowsWithQuery(query: AttendanceReviewRowsQuery, message: string) {
    setReviewStatus("絞り込み中");
    setProgressState("一覧を更新中", 45, message);
    try {
      const nextRows = await getAttendanceReviewRows(query);
      setRows(nextRows.rows);
      setFilteredRowCount(nextRows.filtered_rows);
      setSelectedRowId((current) =>
        current && nextRows.rows.some((row) => row.row_id === current)
          ? current
          : nextRows.rows[0]?.row_id ?? null,
      );
      setReviewStatus("表示できます");
      setProgressState("一覧を更新しました", 100, `${nextRows.filtered_rows}件に絞り込みました。`);
    } catch (error) {
      setReviewStatus("API 未接続");
      setProgressState("失敗", 100, `勤怠一覧を取得できません: ${messageFor(error)}`);
    }
  }

  function buildRowsQuery(): AttendanceReviewRowsQuery {
    return {
      search: search.trim() || undefined,
      status: statusFilter ? (statusFilter as AttendanceReviewRowStatus) : undefined,
      severity: severityFilter ? (severityFilter as AttendanceReviewIssueSeverity) : undefined,
      store_id: storeFilter || undefined,
      department_id: departmentFilter || undefined,
      work_date_from: dateFrom || undefined,
      work_date_to: dateTo || undefined,
      sort_field: sortField,
      sort_direction: sortDirection,
      page: 1,
      page_size: 100,
    };
  }

  function applyReviewData(
    nextSummary: AttendanceReviewSummaryResponse,
    nextRows: AttendanceReviewRow[],
    nextFilteredRows: number,
  ) {
    setSummary(nextSummary);
    setRows(nextRows);
    setFilteredRowCount(nextFilteredRows);
    setActiveRunId(nextSummary.run_id);
    setSelectedRowId((current) =>
      current && nextRows.some((row) => row.row_id === current)
        ? current
        : nextRows[0]?.row_id ?? null,
    );
  }

  function handleSort(nextField: AttendanceReviewSortField) {
    const nextDirection =
      nextField === sortField ? (sortDirection === "asc" ? "desc" : "asc") : "asc";
    if (nextField === sortField) {
      setSortDirection(nextDirection);
    } else {
      setSortField(nextField);
      setSortDirection(nextDirection);
    }
    void loadRowsWithQuery(
      { ...buildRowsQuery(), sort_field: nextField, sort_direction: nextDirection },
      `${sortableColumns.find((column) => column.field === nextField)?.label ?? nextField} で並び替えています。`,
    );
  }

  async function handleStartRun() {
    if (!employeesCsv || !attendanceCsv) {
      setProgressState("入力待ち", 0, "employees CSV と attendance CSV を選択してください。");
      return;
    }

    setProgressState("送信中", 10, "local server に run を登録しています。");
    try {
      const payload = await startRun(employeesCsv, attendanceCsv);
      setActiveRunId(payload.run_id);
      setDbPersistenceStatus(payload.db_persistence_status);
      setProgressState(payload.job_state, payload.progress_percent, "CSV run を開始しました。");
      setArtifacts(payload.artifacts);
      if (payload.report_markdown_path) {
        setReport(await fetchReport(payload.report_markdown_path));
      } else {
        setReport("");
      }
      await loadReview();
    } catch (error) {
      setProgressState("失敗", 100, `local server に接続できません: ${messageFor(error)}`);
    }
  }

  async function handleGuideSend() {
    try {
      setGuideResponse(await postGuideMessage(guideMessage, activeRunId));
    } catch (error) {
      setGuideResponse({
        answer: `ガイド API に接続できません: ${messageFor(error)}`,
        mode: "error",
        safety_boundary: "Local Server API 経由のみ",
        references: [],
      });
    }
  }

  function resetFilters() {
    setSearch("");
    setStatusFilter("");
    setSeverityFilter("");
    setStoreFilter("");
    setDepartmentFilter("");
    setDateFrom("");
    setDateTo("");
  }

  function setProgressState(status: string, value: number, message: string) {
    setProgress({ status, value, message });
  }

  return (
    <main className="shell">
      <header className="toolbar">
        <div>
          <h1>{currentMode.title}</h1>
          <p className="muted">
            {activeMode === "attendance" && summary?.period_start && summary.period_end
              ? `${summary.period_start} - ${summary.period_end}`
              : currentMode.summary}
          </p>
        </div>
        <span className="status" aria-live="polite">
          {reviewStatus}
        </span>
      </header>

      <nav className="mode-nav" aria-label="メインカテゴリ">
        {modeCategories.map((category) => (
          <button
            className={category.id === activeCategory ? "is-active" : ""}
            key={category.id}
            type="button"
            onClick={() => {
              setActiveCategory(category.id);
              setActiveMode(workModes.find((mode) => mode.category === category.id)?.id ?? "attendance");
            }}
          >
            {category.label}
          </button>
        ))}
      </nav>

      <nav className="submode-nav" aria-label="サブ画面">
        {categoryModes.map((mode) => (
          <button
            className={mode.id === activeMode ? "is-active" : ""}
            key={mode.id}
            type="button"
            onClick={() => setActiveMode(mode.id)}
          >
            {mode.label}
          </button>
        ))}
      </nav>

      {activeMode === "attendance" ? (
        <>
      <section className="panel" aria-labelledby="dashboard-heading">
        <div className="panel-header">
          <h2 id="dashboard-heading">勤務データ全体の要約</h2>
          <span className="status-chip">{summary?.generated_at ?? progress.status}</span>
        </div>
        <div className="metric-grid">
          <MetricCard label="対象行数" value={summary?.total_rows ?? 0} unit="行" />
          <MetricCard label="要確認" value={summary?.issue_rows ?? 0} unit="行" status="blocked" />
          <MetricCard label="確認済み" value={reviewCompletion} unit="%" status="ready" />
          <MetricCard label="指摘率" value={issueRate} unit="%" status={issueRate > 0 ? "attention" : "ready"} />
          {(summary?.metrics ?? []).map((metric) => (
            <MetricCard
              key={metric.key}
              label={metric.label}
              value={metric.value}
              unit={metric.unit}
              status={metric.status ?? metric.severity}
              helper={metric.helper_text}
            />
          ))}
        </div>
        <div className="result-columns">
          <CountList
            heading="重要度別"
            items={(summary?.issue_counts_by_severity ?? []).map((item) => ({
              key: item.severity,
              label: severityLabel(item.severity),
              count: item.count,
            }))}
          />
          <CountList
            heading="状態別"
            items={(summary?.row_counts_by_status ?? []).map((item) => ({
              key: item.status,
              label: statusLabel(item.status),
              count: item.count,
            }))}
          />
        </div>
      </section>

      <section className="review-layout" aria-label="レビュー操作">
        <aside className="panel filter-panel" aria-labelledby="filter-heading">
          <div className="panel-header">
            <h2 id="filter-heading">絞り込み</h2>
            <button type="button" onClick={resetFilters}>
              クリア
            </button>
          </div>
          <label>
            検索
            <input
              type="search"
              value={search}
              placeholder="社員名、社員ID、issue code"
              onChange={(event) => setSearch(event.currentTarget.value)}
            />
          </label>
          <label>
            状態
            <select value={statusFilter} onChange={(event) => setStatusFilter(event.currentTarget.value)}>
              <option value="">すべて</option>
              {rowStatuses.map((status) => (
                <option key={status} value={status}>
                  {status}
                </option>
              ))}
            </select>
          </label>
          <label>
            重要度
            <select value={severityFilter} onChange={(event) => setSeverityFilter(event.currentTarget.value)}>
              <option value="">すべて</option>
              {issueSeverities.map((severity) => (
                <option key={severity} value={severity}>
                  {severity}
                </option>
              ))}
            </select>
          </label>
          <label>
            店舗
            <select value={storeFilter} onChange={(event) => setStoreFilter(event.currentTarget.value)}>
              <option value="">すべて</option>
              {(summary?.store_counts ?? []).map((store) => (
                <option key={store.id} value={store.id}>
                  {store.name} ({store.row_count})
                </option>
              ))}
            </select>
          </label>
          <label>
            部門
            <select
              value={departmentFilter}
              onChange={(event) => setDepartmentFilter(event.currentTarget.value)}
            >
              <option value="">すべて</option>
              {(summary?.department_counts ?? []).map((department) => (
                <option key={department.id} value={department.id}>
                  {department.name} ({department.row_count})
                </option>
              ))}
            </select>
          </label>
          <label>
            開始日
            <input type="date" value={dateFrom} onChange={(event) => setDateFrom(event.currentTarget.value)} />
          </label>
          <label>
            終了日
            <input type="date" value={dateTo} onChange={(event) => setDateTo(event.currentTarget.value)} />
          </label>
          <button type="button" onClick={() => void loadRows()}>
            検索
          </button>
        </aside>

        <section className="panel review-table-panel" aria-labelledby="table-heading">
          <div className="panel-header">
            <h2 id="table-heading">勤怠一覧</h2>
            <span className="muted">
              {filteredRowCount} / {summary?.total_rows ?? 0} 行
            </span>
          </div>
          <div className="table-wrap">
            <table>
              <thead>
                <tr>
                  {sortableColumns.map((column) => (
                    <th key={column.field}>
                      <button type="button" onClick={() => handleSort(column.field)}>
                        {column.label}
                        {sortField === column.field ? ` ${sortDirection === "asc" ? "▲" : "▼"}` : ""}
                      </button>
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {rows.length === 0 ? (
                  <tr>
                    <td colSpan={sortableColumns.length}>表示できる勤怠行がありません。</td>
                  </tr>
                ) : (
                  rows.map((row) => (
                    <tr key={row.row_id}>
                      <td>
                        <button type="button" onClick={() => setSelectedRowId(row.row_id)}>
                          {row.employee_id}
                        </button>
                      </td>
                      <td>{row.employee_name}</td>
                      <td>{row.work_date}</td>
                      <td>{row.store_name ?? "-"}</td>
                      <td>{row.department_name ?? "-"}</td>
                      <td className={`status-cell is-${row.status}`}>{row.status}</td>
                      <td>{row.highest_severity ?? "-"}</td>
                      <td>{row.issue_count}</td>
                      <td>{formatHours(row.actual_minutes)}</td>
                      <td>{formatHours(row.overtime_minutes)}</td>
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          </div>
        </section>
      </section>

      <section className="panel" aria-labelledby="detail-heading">
        <div className="panel-header">
          <h2 id="detail-heading">選んだ行の詳細</h2>
          <span className="status-chip">{selectedRow?.status ?? "未選択"}</span>
        </div>
        {selectedRow ? (
          <>
            <div className="result-columns">
              <DetailList
                heading="勤務"
                items={[
                  ["社員", `${selectedRow.employee_id} ${selectedRow.employee_name}`],
                  ["日付", selectedRow.work_date],
                  ["店舗", selectedRow.store_name ?? "-"],
                  ["部門", selectedRow.department_name ?? "-"],
                  ["雇用区分", selectedRow.employment_type ?? "-"],
                ]}
              />
              <DetailList
                heading="時刻"
                items={[
                  ["予定", `${selectedRow.scheduled_start ?? "-"} - ${selectedRow.scheduled_end ?? "-"}`],
                  ["打刻", `${selectedRow.clock_in ?? "-"} - ${selectedRow.clock_out ?? "-"}`],
                  ["休憩", formatMinutes(selectedRow.break_minutes)],
                  ["実働", formatMinutes(selectedRow.actual_minutes)],
                  [
                    "遅刻/早退",
                    `${formatMinutes(selectedRow.late_minutes)} / ${formatMinutes(selectedRow.early_leave_minutes)}`,
                  ],
                ]}
              />
              <DetailList
                heading="入力元"
                items={[
                  ["ファイル", selectedRow.source_file_name ?? "-"],
                  ["行番号", selectedRow.source_row_number ? String(selectedRow.source_row_number) : "-"],
                  ["Issue code", selectedRow.issue_codes.join(", ") || "-"],
                ]}
              />
            </div>
            <div className="table-wrap">
              <table>
                <thead>
                  <tr>
                    <th>重要度</th>
                    <th>状態</th>
                    <th>カテゴリ</th>
                    <th>内容</th>
                    <th>次の確認</th>
                  </tr>
                </thead>
                <tbody>
                  {selectedRow.issues.length === 0 ? (
                    <tr>
                      <td colSpan={5}>この行に問題はありません。</td>
                    </tr>
                  ) : (
                    selectedRow.issues.map((issue) => (
                      <tr key={issue.issue_id}>
                        <td>{issue.severity}</td>
                        <td>{issue.status}</td>
                        <td>{issue.category}</td>
                        <td>
                          <strong>{issue.title}</strong>
                          <p className="muted">{issue.message}</p>
                        </td>
                        <td>{issue.suggested_action ?? "-"}</td>
                      </tr>
                    ))
                  )}
                </tbody>
              </table>
            </div>
          </>
        ) : (
          <p className="muted">勤怠一覧から行を選んでください。</p>
        )}
      </section>
        </>
      ) : (
        <ModePreviewScreen mode={currentMode} summary={summary} />
      )}

      <section className="panel" aria-labelledby="run-setup-heading">
        <h2 id="run-setup-heading">CSVを読み込む</h2>
        <label>
          employees CSV
          <input
            type="file"
            accept=".csv,text/csv"
            onChange={(event) => setEmployeesCsv(event.currentTarget.files?.[0] ?? null)}
          />
        </label>
        <label>
          attendance CSV
          <input
            type="file"
            accept=".csv,text/csv"
            onChange={(event) => setAttendanceCsv(event.currentTarget.files?.[0] ?? null)}
          />
        </label>
        <button type="button" onClick={() => void handleStartRun()}>
          読み込み開始
        </button>
      </section>

      <section className="panel" aria-labelledby="progress-heading">
        <h2 id="progress-heading">進捗</h2>
        <progress max="100" value={progress.value}></progress>
        <p>{progress.message}</p>
        {activeRunId ? <p className="muted">RunId: {activeRunId}</p> : null}
        <p className="muted">DB persistence: {dbPersistenceStatus}</p>
      </section>

      <section className="panel" aria-labelledby="artifact-heading">
        <h2 id="artifact-heading">出力ファイル</h2>
        {artifacts.length === 0 ? (
          <p className="muted">まだ出力ファイルはありません。</p>
        ) : (
          <div className="artifact-list">
            {artifacts.map((artifact) => (
              <article className="artifact-row" key={`${artifact.artifact_name}-${artifact.stable_path}`}>
                <div className="artifact-main">
                  <strong>{artifact.artifact_name}</strong>
                  <span>{artifact.stable_path}</span>
                </div>
                <span className="artifact-type">{artifact.content_type}</span>
                <a className="button-link" href={artifact.stable_path} target="_blank" rel="noreferrer">
                  開く
                </a>
              </article>
            ))}
          </div>
        )}
      </section>

      <section className="panel report" aria-labelledby="report-heading">
        <h2 id="report-heading">レポート</h2>
        <pre>{report || "CSVを読み込むと、ここにレポートを表示します。"}</pre>
      </section>

      <section className="panel" aria-labelledby="guide-heading">
        <h2 id="guide-heading">確認ガイド</h2>
        <label>
          質問
          <textarea
            value={guideMessage}
            onChange={(event) => setGuideMessage(event.currentTarget.value)}
            placeholder="この勤怠の問題を、どの順番で確認すればよいですか"
          />
        </label>
        <button type="button" onClick={() => void handleGuideSend()}>
          ガイドに送る
        </button>
        {guideResponse ? (
          <div className="guide-answer">
            <p>{guideResponse.answer}</p>
            <p className="muted">mode: {guideResponse.mode}</p>
            <p className="muted">boundary: {guideResponse.safety_boundary}</p>
            <ul>
              {guideResponse.references.map((reference) => (
                <li key={reference}>{reference}</li>
              ))}
            </ul>
          </div>
        ) : null}
      </section>
    </main>
  );
}

function CountList({
  heading,
  items,
}: {
  heading: string;
  items: Array<{ key: string; label: string; count: number }>;
}) {
  return (
    <section aria-labelledby={`${heading}-heading`}>
      <h3 id={`${heading}-heading`}>{heading}</h3>
      <ul>
        {items.length === 0 ? (
          <li className="muted">データなし</li>
        ) : (
          items.map((item) => (
            <li key={item.key}>
              {item.label}: {item.count.toLocaleString()}
            </li>
          ))
        )}
      </ul>
    </section>
  );
}

function DetailList({ heading, items }: { heading: string; items: Array<[string, string]> }) {
  return (
    <section aria-labelledby={`${heading}-heading`}>
      <h3 id={`${heading}-heading`}>{heading}</h3>
      <dl>
        {items.map(([label, value]) => (
          <div key={label}>
            <dt className="muted">{label}</dt>
            <dd>{value}</dd>
          </div>
        ))}
      </dl>
    </section>
  );
}

function formatMinutes(value: number | undefined): string {
  return value === undefined ? "-" : `${value.toLocaleString()}分`;
}

function formatHours(value: number | undefined): string {
  if (value === undefined) {
    return "-";
  }
  const hours = Math.floor(value / 60);
  const minutes = value % 60;
  if (hours === 0) {
    return `${minutes}分`;
  }
  if (minutes === 0) {
    return `${hours}時間`;
  }
  return `${hours}時間${minutes}分`;
}

function severityLabel(severity: AttendanceReviewIssueSeverity): string {
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
  }
}

function statusLabel(status: string): string {
  switch (status) {
    case "clean":
      return "問題なし";
    case "warning":
      return "注意";
    case "issue":
      return "要確認";
    case "blocked":
      return "保留";
    case "reviewed":
      return "確認済み";
    case "open":
      return "未対応";
    case "needs_review":
      return "確認待ち";
    case "acknowledged":
      return "確認中";
    case "resolved":
      return "解決済み";
    case "suppressed":
      return "非表示";
    default:
      return status;
  }
}

function percent(numerator: number, denominator: number): number {
  return denominator === 0 ? 0 : Math.round((numerator / denominator) * 100);
}

function messageFor(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
