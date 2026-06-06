import { MetricCard } from "../components/MetricCard";
import type { WorkMode } from "../modeNavigation";
import { getModePreview } from "../modePreviews";
import type { AttendanceReviewSummaryResponse } from "../types";

export function ModePreviewScreen({
  mode,
  summary,
}: {
  mode: WorkMode;
  summary: AttendanceReviewSummaryResponse | null;
}) {
  const preview = getModePreview(mode.id, summary);
  return (
    <>
      <section className="panel mode-overview" aria-labelledby={`${mode.id}-heading`}>
        <div className="panel-header">
          <div>
            <h2 id={`${mode.id}-heading`}>{mode.title}</h2>
            <p className="summary-text">{mode.summary}</p>
          </div>
          <span className="status-chip">画面案</span>
        </div>
        <div className="metric-grid">
          {preview.metrics.map((metric) => (
            <MetricCard
              helper={metric.helper}
              key={metric.label}
              label={metric.label}
              status={metric.status}
              unit={metric.unit}
              value={metric.value}
            />
          ))}
        </div>
      </section>

      <section className="review-layout" aria-label={`${mode.title} 画面案`}>
        <aside className="panel filter-panel">
          <h2>絞り込み</h2>
          {preview.filters.map((filter) => (
            <label key={filter}>
              {filter}
              <select disabled>
                <option>例を表示</option>
              </select>
            </label>
          ))}
        </aside>
        <section className="panel review-table-panel">
          <div className="panel-header">
            <h2>{preview.tableTitle}</h2>
            <span className="muted">サンプルデータ</span>
          </div>
          <div className="table-wrap">
            <table>
              <thead>
                <tr>
                  {preview.columns.map((column) => (
                    <th key={column}>{column}</th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {preview.rows.map((row) => (
                  <tr key={`${mode.id}-${row.cells.join("-")}`}>
                    {row.cells.map((cell, index) => (
                      <td key={`${cell}-${index}`}>{cell}</td>
                    ))}
                    <td className={`status-cell is-${row.status}`}>{row.status}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <p className="muted">{preview.note}</p>
        </section>
      </section>
    </>
  );
}
