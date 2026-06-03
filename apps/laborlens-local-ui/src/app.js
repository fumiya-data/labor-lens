const state = {
  status: document.querySelector("#run-status"),
  progress: document.querySelector("#run-progress"),
  message: document.querySelector("#progress-message"),
  artifacts: document.querySelector("#artifact-list"),
  report: document.querySelector("#report-viewer"),
  start: document.querySelector("#start-run"),
  employees: document.querySelector("#employees-csv"),
  attendance: document.querySelector("#attendance-csv"),
};

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
