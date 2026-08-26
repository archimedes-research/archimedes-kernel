#!/usr/bin/env python3
"""ARCHIMEDES Documentation Grammar 1.0 structural gate.

This checker enforces only the deterministic structural requirements defined by
DOCUMENTATION_GRAMMAR.md. Passing this gate does not establish factual
correctness, provenance, reproducibility, claim correctness, or rendered
correctness.
"""

from __future__ import annotations

import argparse
from dataclasses import dataclass
from pathlib import Path
import re
import subprocess
import sys
from typing import Iterable, Sequence


HEADING_RE = re.compile(r"^[ \t]*#{1,6}(?:[ \t]+|$)")
OPENING_FENCE_RE = re.compile(r"^[ \t]*(`{3,}|~{3,})(.*)$")

SHELL_COMMAND_NAMES = frozenset(
    {
        "awk",
        "bash",
        "cargo",
        "cat",
        "cd",
        "chmod",
        "chown",
        "cmake",
        "cmp",
        "cp",
        "curl",
        "deno",
        "diff",
        "echo",
        "env",
        "fd",
        "ffmpeg",
        "ffprobe",
        "find",
        "git",
        "grep",
        "gzip",
        "head",
        "just",
        "make",
        "mkdir",
        "mv",
        "node",
        "npm",
        "npx",
        "openssl",
        "pip",
        "pip3",
        "pnpm",
        "printf",
        "python",
        "python3",
        "remotion",
        "rm",
        "rustc",
        "rustup",
        "sed",
        "sha256sum",
        "sh",
        "sort",
        "tail",
        "tar",
        "touch",
        "unzip",
        "wc",
        "wget",
        "yarn",
    }
)

ASSIGNMENT_RE = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*=.*$")


@dataclass(frozen=True)
class Finding:
    path: Path
    line: int
    code: str
    message: str

    def render(self, repo: Path) -> str:
        try:
            display = self.path.relative_to(repo)
        except ValueError:
            display = self.path
        return f"FAIL — {display}:{self.line} [{self.code}] {self.message}"


@dataclass
class FileStats:
    opening_fences: int = 0


@dataclass
class GateStats:
    files: int = 0
    opening_fences: int = 0


def _closing_fence(line: str, marker_char: str, marker_len: int) -> bool:
    stripped = line.lstrip(" \t")
    count = 0
    while count < len(stripped) and stripped[count] == marker_char:
        count += 1
    if count < marker_len:
        return False
    return stripped[count:].strip(" \t") == ""


def _language_from_tail(tail: str) -> str:
    tail = tail.strip()
    if not tail:
        return ""
    return tail.split(None, 1)[0].lower()


def _strip_shell_prefixes(line: str) -> str:
    value = line.strip()
    if value.startswith("$ "):
        value = value[2:].lstrip()

    # Skip leading KEY=value environment assignments.
    parts = value.split()
    index = 0
    while index < len(parts) and ASSIGNMENT_RE.match(parts[index]):
        index += 1
    return " ".join(parts[index:])


def looks_like_shell_command(line: str) -> bool:
    """Conservatively recognize executable shell-command lines.

    This recognizer is intentionally deterministic and conservative. It is not
    a shell parser. It catches command forms used by ARCHIMEDES documentation
    without treating arbitrary prose or output as executable input.
    """

    value = _strip_shell_prefixes(line)
    if not value:
        return False

    if value.startswith(("./", "../", "/")):
        return True

    first = value.split(None, 1)[0]
    if first in SHELL_COMMAND_NAMES:
        return True

    return False


def check_bytes(path: Path, data: bytes) -> tuple[list[Finding], FileStats]:
    findings: list[Finding] = []
    stats = FileStats()

    try:
        text = data.decode("utf-8", errors="strict")
    except UnicodeDecodeError as exc:
        findings.append(
            Finding(
                path=path,
                line=1,
                code="INVALID_UTF8",
                message=f"file is not valid UTF-8 ({exc.reason})",
            )
        )
        return findings, stats

    inside = False
    marker_char = ""
    marker_len = 0
    language = ""
    start_line = 0

    for number, line in enumerate(text.splitlines(), 1):
        if inside:
            if _closing_fence(line, marker_char, marker_len):
                inside = False
                marker_char = ""
                marker_len = 0
                language = ""
                start_line = 0
                continue

            if HEADING_RE.match(line):
                findings.append(
                    Finding(
                        path=path,
                        line=number,
                        code="HEADING_IN_FENCE",
                        message="Markdown heading appears inside fenced block",
                    )
                )

            if language == "text" and looks_like_shell_command(line):
                findings.append(
                    Finding(
                        path=path,
                        line=number,
                        code="SHELL_IN_TEXT",
                        message="executable shell command appears inside text fence",
                    )
                )
            continue

        match = OPENING_FENCE_RE.match(line)
        if not match:
            continue

        marker = match.group(1)
        tail = match.group(2)

        inside = True
        marker_char = marker[0]
        marker_len = len(marker)
        language = _language_from_tail(tail)
        start_line = number
        stats.opening_fences += 1

        if not language:
            findings.append(
                Finding(
                    path=path,
                    line=number,
                    code="UNLABELED_FENCE",
                    message="opening fenced block has no language label",
                )
            )

    if inside:
        findings.append(
            Finding(
                path=path,
                line=start_line,
                code="UNCLOSED_FENCE",
                message="fenced block is not closed",
            )
        )

    return findings, stats


def check_file(path: Path) -> tuple[list[Finding], FileStats]:
    try:
        data = path.read_bytes()
    except OSError as exc:
        return (
            [
                Finding(
                    path=path,
                    line=1,
                    code="READ_ERROR",
                    message=str(exc),
                )
            ],
            FileStats(),
        )
    return check_bytes(path, data)


MARKDOWN_SUFFIXES = frozenset(
    {
        ".md",
        ".markdown",
    }
)


def tracked_markdown(repo: Path) -> list[Path]:
    try:
        result = subprocess.run(
            ["git", "-C", str(repo), "ls-files", "-z"],
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
    except subprocess.CalledProcessError as exc:
        detail = exc.stderr.decode("utf-8", errors="replace").strip()
        raise RuntimeError(
            f"cannot enumerate tracked Markdown in {repo}: {detail}"
        ) from exc

    names = [
        item.decode("utf-8", errors="strict")
        for item in result.stdout.split(b"\0")
        if item
    ]

    return [
        repo / name
        for name in names
        if Path(name).suffix.lower() in MARKDOWN_SUFFIXES
    ]


def resolve_paths(repo: Path, requested: Sequence[str]) -> list[Path]:
    if not requested:
        return tracked_markdown(repo)

    paths: list[Path] = []
    for raw in requested:
        path = Path(raw)
        if not path.is_absolute():
            path = repo / path
        paths.append(path)
    return paths


def run_gate(repo: Path, paths: Iterable[Path]) -> tuple[list[Finding], GateStats]:
    findings: list[Finding] = []
    stats = GateStats()

    for path in paths:
        stats.files += 1
        file_findings, file_stats = check_file(path)
        findings.extend(file_findings)
        stats.opening_fences += file_stats.opening_fences

    return findings, stats


def parse_args(argv: Sequence[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Check ARCHIMEDES Markdown structural grammar."
    )
    parser.add_argument(
        "--repo",
        type=Path,
        default=Path.cwd(),
        help="repository root; defaults to current directory",
    )
    parser.add_argument(
        "paths",
        nargs="*",
        help="optional Markdown paths relative to --repo",
    )
    return parser.parse_args(argv)


def main(argv: Sequence[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    repo = args.repo.expanduser().resolve()

    print("ARCHIMEDES Documentation Grammar 1.0 structural gate")
    print(f"repo: {repo}")

    try:
        paths = resolve_paths(repo, args.paths)
    except (RuntimeError, UnicodeDecodeError) as exc:
        print(f"FAIL — [GATE_SETUP] {exc}")
        return 2

    findings, stats = run_gate(repo, paths)

    print(f"Markdown files inspected: {stats.files}")
    print(f"opening fenced blocks: {stats.opening_fences}")

    if findings:
        for finding in findings:
            print(finding.render(repo))
        print(f"FAIL — structural findings: {len(findings)}")
        return 1

    print("PASS — Documentation Grammar 1.0 structural gate")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
