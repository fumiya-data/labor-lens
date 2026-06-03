import copy
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from reports.report_app.contract import (
    PrivacyViolation,
    load_public_report,
    validate_public_report,
)
from reports.report_app.renderer import render_markdown


REPO_ROOT = Path(__file__).resolve().parents[3]
FIXTURE_PATH = REPO_ROOT / "reports" / "examples" / "public_report_v1.json"
FORBIDDEN_KEYS = [
    "employee_ref",
    "fatigue_value",
    "sleep_duration_hours",
    "fatigue_comment",
]


class ReportAppTests(unittest.TestCase):
    def test_valid_public_report_fixture_renders_markdown(self):
        report = load_public_report(FIXTURE_PATH)

        markdown = render_markdown(report)

        self.assertIn("# LaborLens 公開レポート", markdown)
        self.assertIn("## グループプロファイル概要", markdown)

    def test_markdown_contains_run_suppression_and_group_profile(self):
        report = load_public_report(FIXTURE_PATH)

        markdown = render_markdown(report)

        self.assertIn("run-smoke-001", markdown)
        self.assertIn("PERSONAL_HEALTH_DETAIL_SUPPRESSED", markdown)
        self.assertIn("group:operations", markdown)
        self.assertIn("| operations | 1 | 20 | suppressed |", markdown)

    def test_forbidden_raw_keys_are_rejected(self):
        base_report = json.loads(FIXTURE_PATH.read_text(encoding="utf-8"))

        for forbidden_key in FORBIDDEN_KEYS:
            candidate = copy.deepcopy(base_report)
            candidate["profile_report"]["profiles"][0][forbidden_key] = "private"

            with self.subTest(forbidden_key=forbidden_key):
                with self.assertRaises(PrivacyViolation):
                    validate_public_report(
                        candidate,
                        source=f"test fixture with {forbidden_key}",
                    )

    def test_cli_processes_rust_smoke_pipe_input(self):
        fixture_text = FIXTURE_PATH.read_text(encoding="utf-8")

        with tempfile.TemporaryDirectory() as temp_dir:
            output_path = Path(temp_dir) / "public_report.md"
            command = [
                sys.executable,
                str(REPO_ROOT / "reports" / "report_app" / "main.py"),
                "--input",
                "-",
                "--output",
                str(output_path),
            ]

            completed = subprocess.run(
                command,
                input=fixture_text,
                text=True,
                capture_output=True,
                cwd=REPO_ROOT,
                check=False,
            )

            self.assertEqual(completed.returncode, 0, completed.stderr)
            markdown = output_path.read_text(encoding="utf-8")
            self.assertIn("run-smoke-001", markdown)
            self.assertIn("PERSONAL_HEALTH_DETAIL_SUPPRESSED", markdown)


if __name__ == "__main__":
    unittest.main()
