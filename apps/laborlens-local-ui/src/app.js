const state = {
  status: document.querySelector("#run-status"),
  progress: document.querySelector("#run-progress"),
  message: document.querySelector("#progress-message"),
  artifacts: document.querySelector("#artifact-list"),
  report: document.querySelector("#report-viewer"),
  start: document.querySelector("#start-run"),
  employees: document.querySelector("#employees-csv"),
  attendance: document.querySelector("#attendance-csv"),
  demoDbStatus: document.querySelector("#demo-db-status"),
  useCaseButtons: document.querySelector("#use-case-buttons"),
  selectedUseCaseHeading: document.querySelector("#selected-use-case-heading"),
  selectedUseCaseActor: document.querySelector("#selected-use-case-actor"),
  selectedUseCaseSummary: document.querySelector("#selected-use-case-summary"),
  useCaseMetrics: document.querySelector("#use-case-metrics"),
  useCaseRows: document.querySelector("#use-case-rows"),
  useCaseFindings: document.querySelector("#use-case-findings"),
  useCaseActions: document.querySelector("#use-case-actions"),
};

let activeUseCaseId = null;

loadUseCases();

state.start.addEventListener("click", async () => {
  const employees = state.employees.files[0];
  const attendance = state.attendance.files[0];
  if (!employees || !attendance) {
    setProgress("入力待ち", 0, "employees CSV と attendance CSV を選択してください。");
    return;
  }

  setProgress("送信中", 10, "local server に run を登録しています。");
  const body = new FormData();
  body.append("employees_csv", employees);
  body.append("attendance_csv", attendance);

  try {
    const response = await fetch("/api/runs", { method: "POST", body });
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`);
    }
    const payload = await response.json();
    setProgress(payload.job_state ?? "running", payload.progress_percent ?? 0, "run を開始しました。");
    renderArtifacts(payload.artifacts ?? []);
    await loadReport(payload.report_markdown_path);
  } catch (error) {
    setProgress("失敗", 100, `local server に接続できません: ${error.message}`);
  }
});

async function loadUseCases() {
  try {
    const response = await fetch("/api/use-cases");
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`);
    }
    const useCases = await response.json();
    renderUseCaseButtons(useCases);
    state.demoDbStatus.textContent = "DB seed 準備済み";
    if (useCases.length > 0) {
      await loadUseCaseSample(useCases[0].use_case_id);
    }
  } catch (error) {
    state.demoDbStatus.textContent = "API 未接続";
    state.selectedUseCaseSummary.textContent = `local server API に接続できません: ${error.message}`;
  }
}

function renderUseCaseButtons(useCases) {
  state.useCaseButtons.replaceChildren(
    ...useCases.map((useCase) => {
      const button = document.createElement("button");
      button.type = "button";
      button.className = "use-case-button";
      button.dataset.useCaseId = useCase.use_case_id;
      button.textContent = useCase.button_label;
      button.title = useCase.title;
      button.addEventListener("click", () => loadUseCaseSample(useCase.use_case_id));
      return button;
    }),
  );
}

async function loadUseCaseSample(useCaseId) {
  setActiveUseCase(useCaseId);
  setProgress("DB 読み込み中", 35, `${useCaseId} のサンプルデータを読み込んでいます。`);

  try {
    const response = await fetch(`/api/use-cases/${encodeURIComponent(useCaseId)}/sample-data`);
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`);
    }
    const sample = await response.json();
    renderUseCaseSample(sample);
    setProgress("DB 読み込み完了", 100, `${sample.source.table_name} から ${sample.source.employee_count} 人分の seed を確認しました。`);
  } catch (error) {
    setProgress("失敗", 100, `ユースケースデータを読み込めません: ${error.message}`);
  }
}

function setActiveUseCase(useCaseId) {
  activeUseCaseId = useCaseId;
  for (const button of state.useCaseButtons.querySelectorAll("button")) {
    button.classList.toggle("is-active", button.dataset.useCaseId === activeUseCaseId);
  }
}

function renderUseCaseSample(sample) {
  state.demoDbStatus.textContent = `${sample.source.employee_count} 人 seed`;
  state.selectedUseCaseHeading.textContent = sample.use_case.title;
  state.selectedUseCaseActor.textContent = sample.use_case.actor;
  state.selectedUseCaseSummary.textContent = sample.use_case.summary;
  renderMetrics(sample.metrics);
  renderRows(sample.rows);
  renderList(state.useCaseFindings, sample.findings);
  renderList(state.useCaseActions, sample.next_actions);
}

function renderMetrics(metrics) {
  state.useCaseMetrics.replaceChildren(
    ...metrics.map((metric) => {
      const item = document.createElement("div");
      item.className = `metric-card is-${metric.status}`;

      const label = document.createElement("span");
      label.className = "metric-label";
      label.textContent = metric.label;

      const value = document.createElement("strong");
      value.textContent = `${metric.value}${metric.unit}`;

      item.append(label, value);
      return item;
    }),
  );
}

function renderRows(rows) {
  state.useCaseRows.replaceChildren(
    ...rows.map((row) => {
      const tableRow = document.createElement("tr");
      tableRow.append(
        tableCell(row.subject),
        tableCell(row.group),
        tableCell(row.primary_value),
        tableCell(row.status, `status-cell is-${row.status}`),
        tableCell(row.note),
      );
      return tableRow;
    }),
  );
}

function tableCell(text, className = "") {
  const cell = document.createElement("td");
  cell.textContent = text;
  if (className) {
    cell.className = className;
  }
  return cell;
}

function renderList(target, items) {
  target.replaceChildren(
    ...items.map((text) => {
      const item = document.createElement("li");
      item.textContent = text;
      return item;
    }),
  );
}

function setProgress(status, value, message) {
  state.status.textContent = status;
  state.progress.value = value;
  state.message.textContent = message;
}

function renderArtifacts(artifacts) {
  state.artifacts.replaceChildren(
    ...artifacts.map((artifact) => {
      const item = document.createElement("li");
      item.textContent = `${artifact.artifact_name}: ${artifact.stable_path}`;
      return item;
    }),
  );
}

async function loadReport(path) {
  if (!path) {
    state.report.textContent = "";
    return;
  }
  const response = await fetch(path);
  state.report.textContent = await response.text();
}
