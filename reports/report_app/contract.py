from __future__ import annotations

import json
from pathlib import Path
from typing import Any, TextIO


CONTRACT_VERSION = "laborlens.public_report.v1"
FORBIDDEN_RAW_KEYS = frozenset(
    {
        "employee_ref",
        "fatigue_value",
        "sleep_duration_hours",
        "fatigue_comment",
    }
)
REQUIRED_TOP_LEVEL_FIELDS = (
    "contract_version",
    "artifact_manifest",
    "run_summary",
    "issues",
    "profile_report",
)


class PublicReportContractError(ValueError):
    """Raised when public report JSON does not match the renderer contract."""


class PrivacyViolation(PublicReportContractError):
    """Raised when private raw fields appear in renderer input."""


def load_public_report(path: str | Path) -> dict[str, Any]:
    report_path = Path(path)
    text = report_path.read_text(encoding="utf-8")
    return load_public_report_from_text(text, source=str(report_path))


def load_public_report_from_stream(
    stream: TextIO, source: str = "<stdin>"
) -> dict[str, Any]:
    return load_public_report_from_text(stream.read(), source=source)


def load_public_report_from_text(
    text: str, source: str = "<input>"
) -> dict[str, Any]:
    try:
        data = json.loads(text)
    except json.JSONDecodeError as exc:
        message = (
            f"{source}: invalid JSON at line {exc.lineno}, "
            f"column {exc.colno}: {exc.msg}"
        )
        raise PublicReportContractError(message) from exc

    return validate_public_report(data, source=source)


def validate_public_report(
    data: Any, source: str = "<input>"
) -> dict[str, Any]:
    if not isinstance(data, dict):
        raise PublicReportContractError(f"{source}: report root must be a JSON object")

    forbidden_paths = list(_iter_forbidden_key_paths(data))
    if forbidden_paths:
        joined_paths = ", ".join(forbidden_paths)
        raise PrivacyViolation(
            f"{source}: forbidden raw personal field(s) present: {joined_paths}"
        )

    missing_fields = [field for field in REQUIRED_TOP_LEVEL_FIELDS if field not in data]
    if missing_fields:
        joined_fields = ", ".join(missing_fields)
        raise PublicReportContractError(
            f"{source}: missing required top-level field(s): {joined_fields}"
        )

    if data["contract_version"] != CONTRACT_VERSION:
        raise PublicReportContractError(
            f"{source}: unsupported contract_version {data['contract_version']!r}; "
            f"expected {CONTRACT_VERSION!r}"
        )

    artifact_manifest = _expect_object(data, "artifact_manifest", source)
    if artifact_manifest.get("contract_version") != CONTRACT_VERSION:
        raise PublicReportContractError(
            f"{source}: artifact_manifest.contract_version must be {CONTRACT_VERSION!r}"
        )

    _expect_object(data, "run_summary", source)
    _expect_list(data, "issues", source)
    _expect_object(data, "profile_report", source)
    return data


def report_run_id(report: dict[str, Any]) -> str:
    run_summary = report.get("run_summary", {})
    artifact_manifest = report.get("artifact_manifest", {})
    profile_report = report.get("profile_report", {})
    for candidate in (
        run_summary.get("run_id") if isinstance(run_summary, dict) else None,
        artifact_manifest.get("run_id") if isinstance(artifact_manifest, dict) else None,
        profile_report.get("run_id") if isinstance(profile_report, dict) else None,
    ):
        if candidate:
            return str(candidate)
    return "unknown-run"


def _expect_object(data: dict[str, Any], field: str, source: str) -> dict[str, Any]:
    value = data[field]
    if not isinstance(value, dict):
        raise PublicReportContractError(f"{source}: {field} must be a JSON object")
    return value


def _expect_list(data: dict[str, Any], field: str, source: str) -> list[Any]:
    value = data[field]
    if not isinstance(value, list):
        raise PublicReportContractError(f"{source}: {field} must be a JSON array")
    return value


def _iter_forbidden_key_paths(value: Any, path: str = "$") -> list[str]:
    paths: list[str] = []
    if isinstance(value, dict):
        for key, item in value.items():
            next_path = f"{path}.{key}"
            if key in FORBIDDEN_RAW_KEYS:
                paths.append(next_path)
            paths.extend(_iter_forbidden_key_paths(item, next_path))
    elif isinstance(value, list):
        for index, item in enumerate(value):
            paths.extend(_iter_forbidden_key_paths(item, f"{path}[{index}]"))
    return paths
