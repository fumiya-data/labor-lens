import { useEffect, useMemo, useState } from "react";
import {
  fetchReport,
  getAttendanceReviewRows,
  getAttendanceReviewSummary,
  postGuideMessage,
  startRun,
} from "./api";
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
  message: "CSV を選択するか、既存の勤怠レビューを読み込んでください。",
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
  { field: "highest_severity", label: "重大度" },
  { field: "issue_count", label: "指摘" },
  { field: "actual_minutes", label: "実働" },
  { field: "overtime_minutes", label: "残業" },
];

const modeCategories = [
  { id: "attendance", label: "勤怠" },
  { id: "store", label: "店舗運用" },
  { id: "data", label: "データ整備" },
  { id: "analysis", label: "分析・集計" },
  { id: "outputs", label: "成果物" },
] as const;

const workModes = [
  {
    id: "attendance",
    category: "attendance",
    label: "勤怠レビュー",
    title: "勤怠レビュー",
    summary: "給与計算前に、打刻漏れ・時刻逆転・重複候補を確認します。",
  },
  {
    id: "labor-risk",
    category: "attendance",
    label: "労働時間",
    title: "労務確認",
    summary: "長時間勤務候補、連続勤務候補、休暇取得状況を確認材料として整理します。",
  },
  {
    id: "paid-leave",
    category: "attendance",
    label: "年次有給休暇",
    title: "年次有給休暇確認",
    summary: "年次有給休暇の取得状況、取得率の偏り、年5日取得義務の確認候補を整理します。",
  },
  {
    id: "manager-load",
    category: "attendance",
    label: "店長負荷",
    title: "店長負荷確認",
    summary: "店長・責任者の欠員対応、週末連続勤務、負荷集中を確認します。",
  },
  {
    id: "store-corrections",
    category: "store",
    label: "店舗差戻し",
    title: "店舗別修正依頼",
    summary: "店舗ごとの未確認件数、差し戻し対象、修正依頼CSVを整理します。",
  },
  {
    id: "staffing",
    category: "store",
    label: "人員不足",
    title: "人員不足確認",
    summary: "店舗・曜日・時間帯別の不足傾向、応援候補、採用検討材料を整理します。",
  },
  {
    id: "master",
    category: "data",
    label: "マスタ照合",
    title: "従業員マスタ不一致",
    summary: "未登録、退職済み、部署不一致、雇用区分不一致を確認します。",
  },
  {
    id: "csv-schema",
    category: "data",
    label: "CSV仕様変更",
    title: "CSV仕様変更確認",
    summary: "ヘッダー差分、未知列、必須列欠落、別名辞書の影響を確認します。",
  },
  {
    id: "data-readiness",
    category: "data",
    label: "データ整備状況",
    title: "データ整備状況",
    summary: "データセット別の ready / partial / blocked と整備順を確認します。",
  },
  {
    id: "cost",
    category: "analysis",
    label: "人件費",
    title: "人件費・粒度確認",
    summary: "部署別、店舗別、雇用区分別の粒度と勤怠との結合可否を確認します。",
  },
  {
    id: "group-analysis",
    category: "analysis",
    label: "集団分析",
    title: "抑制済み集団分析",
    summary: "少人数抑制済みの部署別集計と公開可能な確認材料を整理します。",
  },
  {
    id: "privacy",
    category: "outputs",
    label: "外部共有前",
    title: "プライバシー確認",
    summary: "識別情報候補、少人数集計、抑制対象を外部共有前に確認します。",
  },
  {
    id: "report",
    category: "outputs",
    label: "月次レポート",
    title: "月次レポート",
    summary: "勤怠、人件費、CSV不備、再確認結果を月次で横並びにします。",
  },
  {
    id: "recheck",
    category: "outputs",
    label: "再確認",
    title: "修正後再確認",
    summary: "修正前後のRunId、入力ハッシュ、issue件数、主要issueの変化を比較します。",
  },
] as const;

type ModeCategoryId = (typeof modeCategories)[number]["id"];
type WorkModeId = (typeof workModes)[number]["id"];

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
    setProgressState("レビュー読込中", 25, "勤怠レビューのサマリーと一覧を読み込んでいます。");
    try {
      const [nextSummary, nextRows] = await Promise.all([
        getAttendanceReviewSummary(),
        getAttendanceReviewRows(buildRowsQuery()),
      ]);
      applyReviewData(nextSummary, nextRows.rows, nextRows.filtered_rows);
      setReviewStatus("レビュー準備済み");
      setProgressState(
        "レビュー読込完了",
        100,
        `${nextRows.filtered_rows} 件の勤怠行を表示できます。`,
      );
    } catch (error) {
      setReviewStatus("API 未接続");
      setProgressState("失敗", 100, `勤怠レビュー API に接続できません: ${messageFor(error)}`);
    }
  }

  async function loadRows() {
    await loadRowsWithQuery(buildRowsQuery(), "検索条件を勤怠レビュー一覧に反映しています。");
  }

  async function loadRowsWithQuery(query: AttendanceReviewRowsQuery, message: string) {
    setReviewStatus("絞り込み中");
    setProgressState("一覧更新中", 45, message);
    try {
      const nextRows = await getAttendanceReviewRows(query);
      setRows(nextRows.rows);
      setFilteredRowCount(nextRows.filtered_rows);
      setSelectedRowId((current) =>
        current && nextRows.rows.some((row) => row.row_id === current)
          ? current
          : nextRows.rows[0]?.row_id ?? null,
      );
      setReviewStatus("レビュー準備済み");
      setProgressState("一覧更新完了", 100, `${nextRows.filtered_rows} 件に絞り込みました。`);
    } catch (error) {
      setReviewStatus("API 未接続");
      setProgressState("失敗", 100, `勤怠レビュー一覧を取得できません: ${messageFor(error)}`);
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
          <h2 id="dashboard-heading">ダッシュボード</h2>
          <span className="status-chip">{summary?.generated_at ?? progress.status}</span>
        </div>
        <div className="metric-grid">
          <MetricCard label="対象行" value={summary?.total_rows ?? 0} unit="行" />
          <MetricCard label="要確認行" value={summary?.issue_rows ?? 0} unit="行" status="blocked" />
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
            heading="重大度別"
            items={(summary?.issue_counts_by_severity ?? []).map((item) => ({
              key: item.severity,
              label: item.severity,
              count: item.count,
            }))}
          />
          <CountList
            heading="状態別"
            items={(summary?.row_counts_by_status ?? []).map((item) => ({
              key: item.status,
              label: item.status,
              count: item.count,
            }))}
          />
        </div>
      </section>

      <section className="review-layout" aria-label="レビュー操作">
        <aside className="panel filter-panel" aria-labelledby="filter-heading">
          <div className="panel-header">
            <h2 id="filter-heading">フィルター</h2>
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
            行状態
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
            重大度
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
                      <td>{formatMinutes(row.actual_minutes)}</td>
                      <td>{formatMinutes(row.overtime_minutes)}</td>
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
          <h2 id="detail-heading">行詳細</h2>
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
                    <th>重大度</th>
                    <th>状態</th>
                    <th>カテゴリ</th>
                    <th>内容</th>
                    <th>推奨対応</th>
                  </tr>
                </thead>
                <tbody>
                  {selectedRow.issues.length === 0 ? (
                    <tr>
                      <td colSpan={5}>この行に指摘はありません。</td>
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
          <p className="muted">勤怠一覧から行を選択してください。</p>
        )}
      </section>
        </>
      ) : (
        <ModePlaceholder mode={currentMode} summary={summary} />
      )}

      <section className="panel" aria-labelledby="run-setup-heading">
        <h2 id="run-setup-heading">CSV run</h2>
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
          run 開始
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
        <h2 id="artifact-heading">成果物</h2>
        {artifacts.length === 0 ? (
          <p className="muted">まだ成果物はありません。</p>
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
        <pre>{report || "CSV run 後に Markdown レポートを表示します。"}</pre>
      </section>

      <section className="panel" aria-labelledby="guide-heading">
        <h2 id="guide-heading">ガイド</h2>
        <label>
          質問
          <textarea
            value={guideMessage}
            onChange={(event) => setGuideMessage(event.currentTarget.value)}
            placeholder="この勤怠指摘の確認順を教えてください"
          />
        </label>
        <button type="button" onClick={() => void handleGuideSend()}>
          ガイドに送信
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

function MetricCard({
  label,
  value,
  unit,
  status,
  helper,
}: {
  label: string;
  value: number;
  unit?: string;
  status?: string;
  helper?: string;
}) {
  return (
    <div className={`metric-card ${status ? `is-${status}` : ""}`}>
      <span className="metric-label">{label}</span>
      <strong>
        {value.toLocaleString()}
        {unit}
      </strong>
      {helper ? <span className="muted">{helper}</span> : null}
    </div>
  );
}

function ModePlaceholder({
  mode,
  summary,
}: {
  mode: (typeof workModes)[number];
  summary: AttendanceReviewSummaryResponse | null;
}) {
  const demoRows = placeholderRows(mode.id);
  return (
    <>
      <section className="panel mode-overview" aria-labelledby={`${mode.id}-heading`}>
        <div className="panel-header">
          <div>
            <h2 id={`${mode.id}-heading`}>{mode.title}</h2>
            <p className="summary-text">{mode.summary}</p>
          </div>
          <span className="status-chip">次回作り込み</span>
        </div>
        <div className="metric-grid">
          <MetricCard label="対象従業員" value={summary?.metrics.find((metric) => metric.key === "employees")?.value ?? 1000} unit="人" status="ready" />
          <MetricCard label="関連行" value={summary?.total_rows ?? 1000} unit="行" status="ready" />
          <MetricCard label="確認候補" value={demoRows.reduce((total, row) => total + row.count, 0)} unit="件" status="attention" />
          <MetricCard label="画面状態" value={1} unit="案" status="review" helper="専用画面化前の仮表示" />
        </div>
      </section>

      <section className="review-layout" aria-label={`${mode.title} デモ`}>
        <aside className="panel filter-panel">
          <h2>想定フィルター</h2>
          {placeholderFilters(mode.id).map((filter) => (
            <label key={filter}>
              {filter}
              <select disabled>
                <option>次回実装</option>
              </select>
            </label>
          ))}
        </aside>
        <section className="panel review-table-panel">
          <div className="panel-header">
            <h2>確認一覧</h2>
            <span className="muted">デモ表示</span>
          </div>
          <div className="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>分類</th>
                  <th>対象</th>
                  <th>件数</th>
                  <th>状態</th>
                  <th>次の確認</th>
                </tr>
              </thead>
              <tbody>
                {demoRows.map((row) => (
                  <tr key={`${mode.id}-${row.subject}`}>
                    <td>{row.category}</td>
                    <td>{row.subject}</td>
                    <td>{row.count.toLocaleString()}</td>
                    <td className={`status-cell is-${row.status}`}>{row.status}</td>
                    <td>{row.nextAction}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>
      </section>
    </>
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

function placeholderFilters(modeId: WorkModeId): string[] {
  switch (modeId) {
    case "store-corrections":
      return ["店舗", "提出状態", "不備種別", "修正依頼状態"];
    case "master":
      return ["不一致種別", "部署", "雇用区分", "在籍状態"];
    case "labor-risk":
      return ["確認項目", "店舗", "部署", "対象期間"];
    case "paid-leave":
      return ["取得状態", "部署", "雇用区分", "対象年度"];
    case "manager-load":
      return ["役割", "店舗", "曜日", "負荷要因"];
    case "cost":
      return ["粒度", "部署", "店舗", "結合可否"];
    case "staffing":
      return ["店舗", "曜日", "時間帯", "不足状態"];
    case "csv-schema":
      return ["データ種別", "列状態", "別名辞書", "影響度"];
    case "data-readiness":
      return ["データセット", "準備状態", "粒度", "優先度"];
    case "group-analysis":
      return ["部署", "抑制状態", "集計粒度", "公開可否"];
    case "privacy":
      return ["抑制理由", "外部共有先", "データ種別", "リスク状態"];
    case "report":
      return ["対象月", "レポート種別", "出力状態", "再確認状態"];
    case "recheck":
      return ["比較run", "解消状態", "新規issue", "再提出状態"];
    default:
      return ["状態"];
  }
}

function placeholderRows(modeId: WorkModeId) {
  switch (modeId) {
    case "store-corrections":
      return [
        row("店舗別", "東京東店", 18, "attention", "店舗担当へ打刻漏れ確認を返す"),
        row("店舗別", "横浜店", 12, "attention", "日付形式と退勤漏れを確認する"),
        row("店舗別", "大阪北店", 9, "ready", "修正後CSVの再提出を待つ"),
      ];
    case "master":
      return [
        row("未登録", "EMP-外部候補", 7, "blocked", "人事マスタへの登録状況を確認する"),
        row("退職済み", "退職者勤怠", 5, "attention", "対象期間の在籍状態を確認する"),
        row("部署不一致", "異動後部署", 11, "attention", "異動日と勤怠部署を突合する"),
      ];
    case "labor-risk":
      return [
        row("長時間候補", "月間残業確認", 15, "attention", "会社の運用閾値に照らして確認する"),
        row("連続勤務", "週次連続勤務", 8, "attention", "休日取得状況を確認する"),
        row("休暇", "有休取得状況", 24, "ready", "部署別傾向として確認する"),
      ];
    case "paid-leave":
      return [
        row("年5日確認", "取得5日未満候補", 18, "attention", "付与日と取得実績を確認する"),
        row("部署別", "取得率が低い部署", 6, "attention", "取得しづらい時期や体制を確認する"),
        row("雇用区分", "パート・アルバイト", 12, "ready", "付与対象者と勤務日数を照合する"),
      ];
    case "manager-load":
      return [
        row("欠員対応", "店長代替勤務", 14, "attention", "欠員理由と応援可否を確認する"),
        row("週末", "週末連続勤務", 7, "attention", "週末シフトの偏りを確認する"),
        row("繁忙帯", "ピーク時間帯対応", 11, "ready", "人員不足画面と突合する"),
      ];
    case "cost":
      return [
        row("粒度", "部署別月次", 10, "ready", "部署別集計として利用する"),
        row("結合不可", "従業員IDなし人件費", 6, "blocked", "個人勤怠と直接結合しない"),
        row("雇用区分", "雇用区分別配分", 4, "attention", "配分ルールを経理へ確認する"),
      ];
    case "staffing":
      return [
        row("慢性不足", "土曜夕方", 9, "attention", "応援または採用検討の材料にする"),
        row("一時欠員", "急な欠勤", 6, "attention", "代替勤務記録を確認する"),
        row("配置", "時間帯別過不足", 12, "ready", "売上・シフト粒度と突合する"),
      ];
    case "csv-schema":
      return [
        row("必須列", "退勤時刻列欠落", 3, "blocked", "CSV出力設定を確認する"),
        row("未知列", "未登録ヘッダー", 8, "attention", "別名辞書への追加要否を確認する"),
        row("形式", "日付形式変更", 5, "attention", "移行fixtureで再実行する"),
      ];
    case "data-readiness":
      return [
        row("ready", "勤怠・従業員マスタ", 2, "ready", "主要確認へ進める"),
        row("partial", "人件費・売上", 2, "attention", "粒度不足を確認する"),
        row("blocked", "シフト", 1, "blocked", "必須入力の有無を確認する"),
      ];
    case "group-analysis":
      return [
        row("抑制", "5人未満部署", 3, "suppressed", "集計を表示しない"),
        row("部署傾向", "負荷集中部署", 5, "attention", "個人値を出さず部署単位で確認する"),
        row("公開可", "安全な集計", 9, "ready", "公開用レポートに利用する"),
      ];
    case "privacy":
      return [
        row("少人数", "5人未満部署", 3, "suppressed", "集計を表示しない"),
        row("識別列", "自由記述列", 2, "blocked", "外部共有前に除外する"),
        row("共有前", "公開用成果物", 4, "attention", "マスキング状態を確認する"),
      ];
    case "report":
      return [
        row("月次", "勤怠不備サマリー", 1, "ready", "今月分の確認結果を保存する"),
        row("比較", "前月比", 1, "attention", "改善と残課題を分ける"),
        row("再確認", "修正後run", 1, "ready", "issue件数の差分を見る"),
      ];
    case "recheck":
      return [
        row("解消", "修正済みissue", 42, "ready", "解消したissueを確認する"),
        row("残存", "未解決issue", 11, "attention", "店舗へ再確認する"),
        row("新規", "修正後に出たissue", 3, "blocked", "修正内容の副作用を確認する"),
      ];
    default:
      return [];
  }
}

function row(category: string, subject: string, count: number, status: string, nextAction: string) {
  return { category, subject, count, status, nextAction };
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

function percent(numerator: number, denominator: number): number {
  return denominator === 0 ? 0 : Math.round((numerator / denominator) * 100);
}

function messageFor(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
