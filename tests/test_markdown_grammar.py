#!/usr/bin/env python3
"""Tests for the ARCHIMEDES Documentation Grammar 1.0 structural gate."""

from __future__ import annotations

import importlib.util
from pathlib import Path
import subprocess
import tempfile
import unittest
import sys


CHECKER_PATH = (
    Path(__file__).resolve().parents[1]
    / "scripts"
    / "check_markdown_grammar.py"
)

SPEC = importlib.util.spec_from_file_location(
    "check_markdown_grammar",
    CHECKER_PATH,
)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"cannot load checker from {CHECKER_PATH}")

CHECKER = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = CHECKER
SPEC.loader.exec_module(CHECKER)


class DocumentationGrammarGateTests(unittest.TestCase):
    def check_bytes(self, data: bytes):
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / "fixture.md"
            path.write_bytes(data)
            findings, stats = CHECKER.check_file(path)
            return findings, stats

    def codes(self, data: bytes) -> list[str]:
        findings, _ = self.check_bytes(data)
        return [finding.code for finding in findings]

    def test_invalid_utf8_fails(self):
        self.assertEqual(
            self.codes(b"# valid\n\n\xff\n"),
            ["INVALID_UTF8"],
        )

    def test_unclosed_fence_fails(self):
        data = (
            b"# Example\n\n"
            b"```bash\n"
            b"cargo test\n"
        )
        self.assertEqual(
            self.codes(data),
            ["UNCLOSED_FENCE"],
        )

    def test_unlabeled_fence_fails(self):
        fence = b"`" * 3
        data = (
            b"# Example\n\n"
            + fence
            + b"\n"
            + b"plain material\n"
            + fence
            + b"\n"
        )
        self.assertEqual(
            self.codes(data),
            ["UNLABELED_FENCE"],
        )

    def test_heading_inside_fence_fails(self):
        fence = b"`" * 3
        data = (
            b"# Example\n\n"
            + fence
            + b"text\n"
            + b"# Swallowed heading\n"
            + fence
            + b"\n"
        )
        self.assertEqual(
            self.codes(data),
            ["HEADING_IN_FENCE"],
        )

    def test_shell_command_inside_text_fence_fails(self):
        fence = b"`" * 3
        data = (
            b"# Example\n\n"
            + fence
            + b"text\n"
            + b"cargo test\n"
            + fence
            + b"\n"
        )
        self.assertEqual(
            self.codes(data),
            ["SHELL_IN_TEXT"],
        )

    def test_rust_crate_attribute_is_not_a_heading(self):
        fence = b"`" * 3
        data = (
            b"# Example\n\n"
            + fence
            + b"rust\n"
            + b"#![forbid(unsafe_code)]\n"
            + b"pub fn verify() {}\n"
            + fence
            + b"\n"
        )
        self.assertEqual(self.codes(data), [])

    def test_plain_text_output_is_not_a_shell_command(self):
        fence = b"`" * 3
        data = (
            b"# Example\n\n"
            + fence
            + b"text\n"
            + b"23 total tests\n"
            + b"0 failures\n"
            + b"cargo: warning emitted by build script\n"
            + fence
            + b"\n"
        )
        self.assertEqual(self.codes(data), [])

    def test_bash_and_text_blocks_are_separate_and_valid(self):
        fence = b"`" * 3
        data = (
            b"# Validation\n\n"
            + fence
            + b"bash\n"
            + b"cargo test\n"
            + fence
            + b"\n\n"
            + fence
            + b"text\n"
            + b"23 tests passed\n"
            + fence
            + b"\n"
        )
        findings, stats = self.check_bytes(data)
        self.assertEqual(findings, [])
        self.assertEqual(stats.opening_fences, 2)

    def test_tilde_fence_is_supported(self):
        data = (
            b"# Example\n\n"
            b"~~~text\n"
            b"verification succeeded\n"
            b"~~~\n"
        )
        self.assertEqual(self.codes(data), [])

    def test_longer_closing_fence_is_supported(self):
        data = (
            b"# Example\n\n"
            b"```text\n"
            b"verification succeeded\n"
            b"````\n"
        )
        self.assertEqual(self.codes(data), [])

    def test_environment_assignment_command_in_text_fails(self):
        fence = b"`" * 3
        data = (
            b"# Example\n\n"
            + fence
            + b"text\n"
            + b"RUST_BACKTRACE=1 cargo test\n"
            + fence
            + b"\n"
        )
        self.assertEqual(
            self.codes(data),
            ["SHELL_IN_TEXT"],
        )

    def test_prompted_shell_command_in_text_fails(self):
        fence = b"`" * 3
        data = (
            b"# Example\n\n"
            + fence
            + b"text\n"
            + b"$ git status\n"
            + fence
            + b"\n"
        )
        self.assertEqual(
            self.codes(data),
            ["SHELL_IN_TEXT"],
        )

    def test_tracked_markdown_discovers_supported_suffixes(self):
        with tempfile.TemporaryDirectory() as temp:
            repo = Path(temp)

            subprocess.run(
                ["git", "init", "-q", str(repo)],
                check=True,
            )

            paths = {
                "README.md": b"# One\n",
                "UPPER.MD": b"# Two\n",
                "design.markdown": b"# Three\n",
                "ignored.txt": b"not Markdown\n",
            }

            for name, data in paths.items():
                path = repo / name
                path.write_bytes(data)

            subprocess.run(
                [
                    "git",
                    "-C",
                    str(repo),
                    "add",
                    "--",
                    *paths.keys(),
                ],
                check=True,
            )

            discovered = {
                path.relative_to(repo).as_posix()
                for path in CHECKER.tracked_markdown(repo)
            }

            self.assertEqual(
                discovered,
                {
                    "README.md",
                    "UPPER.MD",
                    "design.markdown",
                },
            )

    def test_main_returns_zero_for_valid_explicit_path(self):
        with tempfile.TemporaryDirectory() as temp:
            repo = Path(temp)
            path = repo / "valid.md"
            path.write_bytes(b"# Valid\n")

            self.assertEqual(
                CHECKER.main(
                    [
                        "--repo",
                        str(repo),
                        "valid.md",
                    ]
                ),
                0,
            )

    def test_main_returns_one_for_structural_finding(self):
        with tempfile.TemporaryDirectory() as temp:
            repo = Path(temp)
            path = repo / "invalid.md"
            path.write_bytes(
                b"# Invalid\n\n"
                + (b"`" * 3)
                + b"bash\n"
                + b"cargo test\n"
            )

            self.assertEqual(
                CHECKER.main(
                    [
                        "--repo",
                        str(repo),
                        "invalid.md",
                    ]
                ),
                1,
            )

    def test_main_returns_two_for_repository_setup_failure(self):
        with tempfile.TemporaryDirectory() as temp:
            repo = Path(temp)

            self.assertEqual(
                CHECKER.main(
                    [
                        "--repo",
                        str(repo),
                    ]
                ),
                2,
            )


if __name__ == "__main__":
    unittest.main()
