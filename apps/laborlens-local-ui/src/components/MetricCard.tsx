export type MetricCardProps = {
  label: string;
  value: number;
  unit?: string;
  status?: string;
  helper?: string;
};

export function MetricCard({ label, value, unit, status, helper }: MetricCardProps) {
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
