from __future__ import annotations

import argparse
import sys
from pathlib import Path
from typing import Sequence


if __package__ in (None, ""):
    sys.path.insert(0, str(Path(__file__).resolve().parents[2]))

from reports.report_app.contract import (  # noqa: E402
    PublicReportContractError,
    load_public_report,
    load_public_report_from_stream,
    report_run_id,
)
from reports.report_app.renderer import render_markdown  # noqa: E402


def main(argv: Sequence[str] | None = None) -> int:
    parser = _build_parser()
    args = parser.parse_args(argv)

    try:
        report = _read_report(args.input)
        markdown = render_markdown(report)
        if args.output == "-":
            sys.stdout.write(markdown)
            return 0

        output_path = _resolve_output_path(args.output, report)
        output_path.write_text(markdown, encoding="utf-8")
    except PublicReportContractError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2
    except OSError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1

    print(f"wrote {output_path}", file=sys.stderr)
    return 0


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Render LaborLens public report JSON to Markdown."
    )
    parser.add_argument(
        "--input",
        default="-",
        help="Path to laborlens.public_report.v1 JSON, or '-' for stdin.",
    )
    parser.add_argument(
        "--output",
        default=None,
        help=(
            "Markdown output path, output directory, or '-' for stdout. "
            "Defaults to reports/examples/public_report_<run_id>.md."
        ),
    )
    return parser


def _read_report(input_arg: str):
    if input_arg == "-":
        return load_public_report_from_stream(sys.stdin)
    return load_public_report(input_arg)


def _resolve_output_path(output_arg: str | None, report) -> Path:
    default_name = f"public_report_{_safe_file_fragment(report_run_id(report))}.md"
    if output_arg is None:
        output_path = Path(__file__).resolve().parents[1] / "examples" / default_name
    else:
        output_path = Path(output_arg)
        if _is_directory_target(output_path):
            output_path = output_path / default_name

    output_path.parent.mkdir(parents=True, exist_ok=True)
    return output_path


def _is_directory_target(path: Path) -> bool:
    return path.is_dir() or path.suffix.lower() != ".md"


def _safe_file_fragment(value: str) -> str:
    cleaned = []
    for character in value:
        if character.isalnum() or character in ("-", "_", "."):
            cleaned.append(character)
        else:
            cleaned.append("_")
    return "".join(cleaned).strip("._-") or "report"


if __name__ == "__main__":
    raise SystemExit(main())
