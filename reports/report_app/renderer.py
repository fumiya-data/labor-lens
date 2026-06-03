from __future__ import annotations

from typing import Any, Iterable

from reports.report_app.contract import report_run_id, validate_public_report


def render_markdown(report: dict[str, Any]) -> str:
    report = validate_public_report(report)

    artifact_manifest = report["artifact_manifest"]
    run_summary = report["run_summary"]
    profile_report = report["profile_report"]
    policy_trace = artifact_manifest.get("policy_trace", {})

    lines: list[str] = [
        "# LaborLens 公開レポート",
        "",
        f"- 契約バージョン: `{_cell(report['contract_version'])}`",
        f"- 実行 ID: `{_cell(report_run_id(report))}`",
        f"- ポリシー: `{_cell(_policy_label(run_summary, policy_trace))}`",
        "",
        "## 実行サマリー",
        "",
    ]

    lines.extend(
        _table(
            ["指標", "値"],
            [
                ["従業員数", run_summary.get("employee_count", "")],
                ["プロファイル数", run_summary.get("profile_count", "")],
                [
                    "抑制カテゴリ数",
                    run_summary.get("suppressed_category_count", ""),
                ],
                [
                    "抑制フィールド数",
                    run_summary.get("suppressed_field_count", ""),
                ],
                ["issue 数", run_summary.get("issue_count", "")],
            ],
        )
    )

    lines.extend(["", "## グループプロファイル概要", ""])
    profile_rows = []
    for profile in profile_report.get("profiles", []):
        if not isinstance(profile, dict):
            continue
        profile_rows.append(
            [
                profile.get("profile_id", ""),
                profile.get("group_key", ""),
                profile.get("employee_count", ""),
                profile.get("attendance_days_observed", ""),
                profile.get("health_detail_status", ""),
            ]
        )
    lines.extend(
        _table(
            [
                "プロファイル ID",
                "グループ",
                "従業員数",
                "観測した勤怠日数",
                "健康関連詳細の状態",
            ],
            profile_rows,
        )
    )

    lines.extend(["", "## 抑制サマリー", ""])
    suppression_rows = []
    for item in profile_report.get("suppression_summary", []):
        if not isinstance(item, dict):
            continue
        suppression_rows.append(
            [
                item.get("suppression_code", ""),
                item.get("category", ""),
                item.get("affected_record_count", ""),
                item.get("suppressed_field_count", ""),
                item.get("reason", ""),
            ]
        )
    lines.extend(
        _table(
            [
                "抑制コード",
                "カテゴリ",
                "影響レコード数",
                "抑制フィールド数",
                "理由",
            ],
            suppression_rows,
        )
    )

    lines.extend(["", "## 公開 issue", ""])
    issue_rows = []
    for issue in report.get("issues", []):
        if not isinstance(issue, dict):
            continue
        issue_rows.append(
            [
                issue.get("severity", ""),
                issue.get("issue_id", ""),
                issue.get("suppression_code", ""),
                issue.get("message", ""),
            ]
        )
    lines.extend(
        _table(
            ["重要度", "issue ID", "抑制コード", "メッセージ"],
            issue_rows,
        )
    )

    lines.extend(["", "## 成果物一覧", "", "### 入力トレース", ""])
    input_rows = []
    for trace in artifact_manifest.get("input_traces", []):
        if not isinstance(trace, dict):
            continue
        input_rows.append(
            [
                trace.get("dataset_id", ""),
                trace.get("source_ref", ""),
                trace.get("fingerprint", ""),
                trace.get("record_count", ""),
            ]
        )
    lines.extend(
        _table(
            ["データセット", "入力元参照", "フィンガープリント", "レコード数"],
            input_rows,
        )
    )

    lines.extend(["", "### 出力トレース", ""])
    output_rows = []
    for trace in artifact_manifest.get("output_traces", []):
        if not isinstance(trace, dict):
            continue
        output_rows.append(
            [
                trace.get("artifact_name", ""),
                trace.get("artifact_kind", ""),
                trace.get("stable_path", ""),
                trace.get("content_schema", ""),
            ]
        )
    lines.extend(
        _table(
            ["成果物", "種別", "安定パス", "スキーマ"],
            output_rows,
        )
    )

    lines.extend(["", "### ポリシートレース", ""])
    lines.extend(
        _table(
            ["ポリシー項目", "値"],
            [
                ["policy_id", policy_trace.get("policy_id", "")],
                ["version", policy_trace.get("version", "")],
                ["safety_boundary", policy_trace.get("safety_boundary", "")],
            ],
        )
    )

    return "\n".join(lines).rstrip() + "\n"


def _policy_label(run_summary: dict[str, Any], policy_trace: dict[str, Any]) -> str:
    policy_id = run_summary.get("policy_id") or policy_trace.get("policy_id") or "unknown"
    version = (
        run_summary.get("policy_version") or policy_trace.get("version") or "unknown"
    )
    return f"{policy_id} ({version})"


def _table(headers: list[str], rows: Iterable[Iterable[Any]]) -> list[str]:
    normalized_rows = [list(row) for row in rows]
    if not normalized_rows:
        return ["_なし._"]

    header_line = "| " + " | ".join(_cell(header) for header in headers) + " |"
    separator_line = "| " + " | ".join("---" for _ in headers) + " |"
    body_lines = [
        "| " + " | ".join(_cell(value) for value in row) + " |"
        for row in normalized_rows
    ]
    return [header_line, separator_line, *body_lines]


def _cell(value: Any) -> str:
    if value is None:
        return ""
    return str(value).replace("\r", " ").replace("\n", " ").replace("|", "\\|")
