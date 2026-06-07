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
        <div className="result-columns">
          <CountList
            heading="確認対象"
            items={preview.metrics.map((metric) => ({
              key: metric.label,
              label: metric.label,
              count: metric.value,
              unit: metric.unit,
            }))}
          />
          <CountList
            heading="状態別"
            items={preview.rows.map((row, index) => ({
              key: `${row.status}-${index}`,
              label: statusLabel(row.status),
              count: 1,
            }))}
          />
        </div>
      </section>

      <section className="review-layout" aria-label={`${mode.title} 画面案`}>
        <aside className="panel filter-panel">
          <div className="panel-header">
            <h2>絞り込み</h2>
            <button type="button" disabled>
              クリア
            </button>
          </div>
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

      <section className="panel" aria-labelledby={`${mode.id}-detail-heading`}>
        <div className="panel-header">
          <h2 id={`${mode.id}-detail-heading`}>選んだ行の詳細</h2>
          <span className="status-chip">{statusLabel(preview.rows[0]?.status)}</span>
        </div>
        <div className="result-columns">
          {preview.detailSections.map((section) => (
            <DetailList heading={section.heading} items={section.items} key={section.heading} />
          ))}
        </div>
        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th>状態</th>
                <th>内容</th>
                <th>次の確認</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td>{statusLabel(preview.rows[0]?.status)}</td>
                <td>
                  <strong>{preview.rows[0]?.cells[0] ?? mode.title}</strong>
                  <p className="muted">{preview.note}</p>
                </td>
                <td>{preview.detailSections[2]?.items[2]?.[1] ?? "-"}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </section>
    </>
  );
}

function CountList({
  heading,
  items,
}: {
  heading: string;
  items: Array<{ key: string; label: string; count: number; unit?: string }>;
}) {
  return (
    <section aria-labelledby={`${heading}-heading`}>
      <h3 id={`${heading}-heading`}>{heading}</h3>
      <ul>
        {items.map((item) => (
          <li key={item.key}>
            {item.label}: {item.count.toLocaleString()}
            {item.unit ?? ""}
          </li>
        ))}
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

function statusLabel(status: string | undefined): string {
  switch (status) {
    case "ready":
      return "問題なし";
    case "attention":
      return "要確認";
    case "blocked":
      return "保留";
    case "suppressed":
      return "非表示";
    case "warning":
      return "注意";
    default:
      return status || "-";
  }
}
