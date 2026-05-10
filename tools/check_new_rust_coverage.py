#!/usr/bin/env python3
"""Fail when added executable Rust lines have no coverage."""

from __future__ import annotations

import argparse
from collections import Counter
from pathlib import Path
import re
import subprocess
import sys


HUNK_RE = re.compile(r"@@ -\d+(?:,\d+)? \+(\d+)(?:,\d+)? @@")
DA_RE = re.compile(r"DA:(\d+),(\d+)")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Check that added executable Rust lines have 100% line coverage."
    )
    parser.add_argument("--lcov", required=True, type=Path, help="LCOV report path.")
    parser.add_argument(
        "--base",
        default="HEAD",
        help="Git revision to diff against. Defaults to HEAD for local dirty worktrees.",
    )
    parser.add_argument(
        "--repo-root",
        default=Path.cwd(),
        type=Path,
        help="Repository root. Defaults to the current working directory.",
    )
    return parser.parse_args()


def git_diff_text(repo_root: Path, base: str) -> str:
    command = ["git", "diff", "--unified=0", "--diff-filter=AM"]
    if base:
        command.append(base)
    command.extend(["--", "*.rs"])
    return subprocess.check_output(command, cwd=repo_root, text=True)


def parse_added_rust_lines(
    diff_text: str,
    repo_root: Path,
    *,
    ignore_moved: bool = False,
) -> dict[Path, set[int]]:
    added: dict[Path, set[int]] = {}
    removed_lines: Counter[str] = Counter()
    added_lines: list[tuple[Path, int, str]] = []
    current_file: Path | None = None
    current_line: int | None = None

    for line in diff_text.splitlines():
        if line.startswith("+++ "):
            path = line[4:].strip()
            current_file = None if path == "/dev/null" else normalize_diff_path(path, repo_root)
            current_line = None
            continue
        if line.startswith("@@ "):
            match = HUNK_RE.match(line)
            current_line = int(match.group(1)) if match else None
            continue
        if current_file is None or current_line is None:
            continue
        if line.startswith("+") and not line.startswith("+++"):
            if ignore_moved:
                added_lines.append((current_file, current_line, line[1:]))
            else:
                added.setdefault(current_file, set()).add(current_line)
            current_line += 1
        elif line.startswith("-") and not line.startswith("---"):
            if ignore_moved:
                removed_lines[moved_line_fingerprint(line[1:])] += 1
            continue
        else:
            current_line += 1

    if ignore_moved:
        for path, line_number, line_text in added_lines:
            fingerprint = moved_line_fingerprint(line_text)
            if removed_lines[fingerprint] > 0:
                removed_lines[fingerprint] -= 1
                continue
            added.setdefault(path, set()).add(line_number)

    return added


def moved_line_fingerprint(line: str) -> str:
    normalized = re.sub(r"\s+", " ", line.strip())
    return re.sub(r"^pub(?:\([^)]*\))?\s+", "", normalized)


def normalize_diff_path(path: str, repo_root: Path) -> Path:
    if path.startswith("a/") or path.startswith("b/"):
        path = path[2:]
    return (repo_root / path).resolve()


def parse_lcov_line_counts(lcov_text: str, repo_root: Path) -> dict[Path, dict[int, int]]:
    coverage: dict[Path, dict[int, int]] = {}
    current_file: Path | None = None

    for line in lcov_text.splitlines():
        if line.startswith("SF:"):
            current_file = normalize_lcov_path(line[3:], repo_root)
            coverage.setdefault(current_file, {})
            continue
        if current_file is None:
            continue
        match = DA_RE.match(line)
        if match:
            coverage[current_file][int(match.group(1))] = int(match.group(2))

    return coverage


def normalize_lcov_path(path: str, repo_root: Path) -> Path:
    candidate = Path(path)
    if not candidate.is_absolute():
        candidate = repo_root / candidate
    return candidate.resolve()


def uncovered_added_lines(
    added_lines: dict[Path, set[int]],
    coverage: dict[Path, dict[int, int]],
) -> tuple[int, list[tuple[Path, int]]]:
    instrumented = 0
    uncovered = []
    for path, lines in sorted(added_lines.items()):
        file_coverage = coverage.get(path, {})
        for line in sorted(lines):
            if line not in file_coverage:
                continue
            instrumented += 1
            if file_coverage[line] == 0:
                uncovered.append((path, line))
    return instrumented, uncovered


def production_added_lines(added_lines: dict[Path, set[int]]) -> dict[Path, set[int]]:
    production = {}
    for path, lines in added_lines.items():
        cfg_test_lines = cfg_test_item_lines(path)
        kept = {line for line in lines if line not in cfg_test_lines}
        if kept:
            production[path] = kept
    return production


def executable_added_lines(added_lines: dict[Path, set[int]]) -> dict[Path, set[int]]:
    executable = {}
    for path, lines in added_lines.items():
        source_lines = source_text_lines(path)
        kept = {
            line
            for line in lines
            if line <= len(source_lines) and is_executable_rust_line(source_lines[line - 1])
        }
        if kept:
            executable[path] = kept
    return executable


def source_text_lines(path: Path) -> list[str]:
    try:
        return path.read_text(encoding="utf-8").splitlines()
    except FileNotFoundError:
        return []


def is_executable_rust_line(line: str) -> bool:
    stripped = line.strip()
    if not stripped or stripped.startswith("//") or stripped.startswith("#["):
        return False
    if stripped in {
        "{",
        "}",
        "};",
        "),",
        ");",
        ")?;",
        "})",
        "});",
        "],",
        "};",
    }:
        return False
    if set(stripped) <= set("(){}[];,?"):
        return False
    return True


def cfg_test_item_lines(path: Path) -> set[int]:
    test_lines: set[int] = set()
    pending_cfg_non_production = False
    in_cfg_non_production_item = False
    brace_depth = 0

    for line_number, line in enumerate(source_text_lines(path), start=1):
        stripped = line.strip()

        if in_cfg_non_production_item:
            test_lines.add(line_number)
            brace_depth += rust_brace_delta(line)
            if brace_depth <= 0:
                in_cfg_non_production_item = False
            continue

        if pending_cfg_non_production:
            test_lines.add(line_number)
            if not stripped or stripped.startswith("//") or stripped.startswith("#["):
                continue
            code = rust_code_without_strings_or_comments(line)
            if "{" in code:
                pending_cfg_non_production = False
                brace_depth = rust_brace_delta(line)
                if brace_depth > 0:
                    in_cfg_non_production_item = True
                continue
            if stripped.endswith(";"):
                pending_cfg_non_production = False
            continue

        if is_non_production_cfg_attr(stripped):
            test_lines.add(line_number)
            pending_cfg_non_production = True

    return test_lines


def is_non_production_cfg_attr(stripped: str) -> bool:
    if stripped in {"#[cfg(test)]", "#[cfg(coverage)]"}:
        return True
    if not (stripped.startswith("#[cfg(any(") and stripped.endswith("))]")):
        return False
    cfg = stripped[len("#[cfg(any(") : -3]
    tokens = {token.strip() for token in cfg.split(",")}
    return bool(tokens.intersection({"test", "coverage"}))


def rust_brace_delta(line: str) -> int:
    code = rust_code_without_strings_or_comments(line)
    return code.count("{") - code.count("}")


def rust_code_without_strings_or_comments(line: str) -> str:
    code = []
    index = 0
    in_string = False
    in_char = False
    while index < len(line):
        char = line[index]
        next_char = line[index + 1] if index + 1 < len(line) else ""

        if in_string:
            if char == "\\":
                index += 2
                continue
            if char == '"':
                in_string = False
            index += 1
            continue

        if in_char:
            if char == "\\":
                index += 2
                continue
            if char == "'":
                in_char = False
            index += 1
            continue

        if char == "/" and next_char == "/":
            break
        if char == '"':
            in_string = True
            index += 1
            continue
        if char == "'":
            in_char = True
            index += 1
            continue

        code.append(char)
        index += 1

    return "".join(code)


def main() -> int:
    args = parse_args()
    repo_root = args.repo_root.resolve()
    diff_added_lines = parse_added_rust_lines(
        git_diff_text(repo_root, args.base),
        repo_root,
        ignore_moved=True,
    )
    added_lines = executable_added_lines(
        production_added_lines(diff_added_lines)
    )
    coverage = parse_lcov_line_counts(args.lcov.read_text(encoding="utf-8"), repo_root)
    instrumented, uncovered = uncovered_added_lines(added_lines, coverage)

    if uncovered:
        print("Added executable Rust lines without coverage:", file=sys.stderr)
        for path, line in uncovered:
            print(f"  {path.relative_to(repo_root)}:{line}", file=sys.stderr)
        return 1

    print(f"new Rust line coverage: {instrumented}/{instrumented} added executable line(s)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
