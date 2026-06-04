#!/usr/bin/env python3
"""Check accepted MAME-vs-clean media reports as one closure gate."""

from __future__ import annotations

import argparse
import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any


DEFAULT_MANIFEST = Path("docs/fidelity/reference-report-gate.json")
VALID_ACCEPTANCE_MODES = {"all", "visual", "audio"}
REFERENCE_MEDIA_ROOT = Path("target/reference-media/mame")
CANDIDATE_MEDIA_ROOT = Path("target/reference-media/clean")
REPORT_ROOT = Path("target/reference-media")
DEFAULT_WINDOW_SCAN_REPORTS = (
    (
        "all traces",
        Path("target/reference-media/reference-window-scan.json"),
    ),
    (
        "organic only",
        Path("target/reference-media/reference-window-scan-organic.json"),
    ),
)


@dataclass(frozen=True)
class CoverageRequirement:
    name: str
    coverage: str
    accepted_axis: str
    min_reports: int = 1


@dataclass(frozen=True)
class ReportExpectation:
    name: str
    path: Path
    acceptance_mode: str
    coverage: tuple[str, ...] = ()


@dataclass(frozen=True)
class ReportManifest:
    reports: list[ReportExpectation]
    coverage_requirements: list[CoverageRequirement]


@dataclass(frozen=True)
class WindowScanExpectation:
    name: str
    path: Path


@dataclass(frozen=True)
class RequiredMediaArtifact:
    label: str
    path_text: str
    root: Path
    suffixes: tuple[str, ...]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Verify accepted Defender reference-media reports."
    )
    parser.add_argument(
        "--manifest",
        default=str(DEFAULT_MANIFEST),
        help="JSON manifest listing accepted report paths and modes.",
    )
    parser.add_argument(
        "--summary-out",
        help="Write a Markdown owner-review summary for the accepted reports.",
    )
    parser.add_argument(
        "--window-scan-report",
        action="append",
        default=[],
        help=(
            "Window-scan summary as name=path. Defaults to the all-trace and "
            "organic-only target/reference-media scan reports."
        ),
    )
    return parser.parse_args()


def default_window_scan_reports() -> list[WindowScanExpectation]:
    return [
        WindowScanExpectation(name=name, path=path)
        for name, path in DEFAULT_WINDOW_SCAN_REPORTS
    ]


def parse_window_scan_report_arg(value: str) -> WindowScanExpectation:
    try:
        name, report_path = value.split("=", 1)
    except ValueError as error:
        raise argparse.ArgumentTypeError(
            f"window scan report must use name=path, got {value!r}"
        ) from error
    name = name.strip()
    report_path = report_path.strip()
    if not name or not report_path:
        raise argparse.ArgumentTypeError(
            f"window scan report must use non-empty name and path, got {value!r}"
        )
    return WindowScanExpectation(name=name, path=Path(report_path))


def load_json(path: Path) -> dict[str, Any]:
    with path.open(encoding="utf-8") as handle:
        data = json.load(handle)
    if not isinstance(data, dict):
        raise ValueError(f"{path} must contain a JSON object")
    return data


def relative_path_is_under(path: Path, root: Path) -> bool:
    if path.is_absolute() or ".." in path.parts:
        return False
    try:
        path.relative_to(root)
    except ValueError:
        return False
    return True


def load_manifest(path: Path) -> ReportManifest:
    data = load_json(path)
    reports = data.get("reports")
    if not isinstance(reports, list):
        raise ValueError(f"{path} must contain a reports list")

    requirements = data.get("coverage_requirements", [])
    if not isinstance(requirements, list):
        raise ValueError(f"{path} coverage_requirements must be a list")

    coverage_requirements: list[CoverageRequirement] = []
    seen_requirement_names: set[str] = set()
    seen_requirement_keys: set[tuple[str, str]] = set()
    requirement_axes_by_coverage: dict[str, set[str]] = {}
    for index, item in enumerate(requirements):
        if not isinstance(item, dict):
            raise ValueError(f"{path} coverage_requirements[{index}] must be an object")
        name = item.get("name")
        coverage = item.get("coverage")
        accepted_axis = item.get("accepted_axis")
        min_reports = item.get("min_reports", 1)
        if not isinstance(name, str) or not name:
            raise ValueError(
                f"{path} coverage_requirements[{index}] needs a non-empty name"
            )
        if not isinstance(coverage, str) or not coverage:
            raise ValueError(
                f"{path} coverage_requirements[{index}] needs non-empty coverage"
            )
        if accepted_axis not in VALID_ACCEPTANCE_MODES:
            raise ValueError(
                f"{path} coverage_requirements[{index}] has invalid accepted_axis "
                f"{accepted_axis!r}"
            )
        if not isinstance(min_reports, int) or min_reports <= 0:
            raise ValueError(
                f"{path} coverage_requirements[{index}] needs positive min_reports"
            )
        if name in seen_requirement_names:
            raise ValueError(
                f"{path} coverage_requirements[{index}] duplicates requirement "
                f"name {name!r}"
            )
        seen_requirement_names.add(name)

        requirement_key = (coverage, accepted_axis)
        if requirement_key in seen_requirement_keys:
            raise ValueError(
                f"{path} coverage_requirements[{index}] duplicates coverage "
                f"{coverage!r} for accepted_axis {accepted_axis!r}"
            )
        seen_requirement_keys.add(requirement_key)
        requirement_axes_by_coverage.setdefault(coverage, set()).add(accepted_axis)

        coverage_requirements.append(
            CoverageRequirement(
                name=name,
                coverage=coverage,
                accepted_axis=accepted_axis,
                min_reports=min_reports,
            )
        )

    expectations: list[ReportExpectation] = []
    seen_report_names: set[str] = set()
    seen_report_paths: set[str] = set()
    for index, item in enumerate(reports):
        if not isinstance(item, dict):
            raise ValueError(f"{path} reports[{index}] must be an object")
        name = item.get("name")
        report_path = item.get("path")
        acceptance_mode = item.get("acceptance_mode")
        coverage = item.get("coverage", [])
        if not isinstance(name, str) or not name:
            raise ValueError(f"{path} reports[{index}] needs a non-empty name")
        if not isinstance(report_path, str) or not report_path:
            raise ValueError(f"{path} reports[{index}] needs a non-empty path")
        if not isinstance(acceptance_mode, str) or not acceptance_mode:
            raise ValueError(
                f"{path} reports[{index}] needs an explicit acceptance_mode"
            )
        if acceptance_mode not in VALID_ACCEPTANCE_MODES:
            raise ValueError(
                f"{path} reports[{index}] has invalid acceptance_mode "
                f"{acceptance_mode!r}"
            )
        if not isinstance(coverage, list) or not all(
            isinstance(item, str) and item for item in coverage
        ):
            raise ValueError(
                f"{path} reports[{index}] coverage must be a list of non-empty strings"
            )
        report_manifest_path = Path(report_path)
        if not relative_path_is_under(report_manifest_path, REPORT_ROOT):
            raise ValueError(
                f"{path} reports[{index}] path must be under {REPORT_ROOT}"
            )
        if name in seen_report_names:
            raise ValueError(f"{path} reports[{index}] duplicates report name {name!r}")
        seen_report_names.add(name)

        path_key = report_manifest_path.as_posix()
        if path_key in seen_report_paths:
            raise ValueError(
                f"{path} reports[{index}] duplicates report path {path_key!r}"
            )
        seen_report_paths.add(path_key)

        coverage_tags = tuple(coverage)
        if len(set(coverage_tags)) != len(coverage_tags):
            raise ValueError(
                f"{path} reports[{index}] coverage must not contain duplicate tags"
            )
        if not coverage_tags:
            raise ValueError(
                f"{path} reports[{index}] coverage must include at least one tag"
            )
        for coverage_tag in coverage_tags:
            accepted_axes = requirement_axes_by_coverage.get(coverage_tag)
            if accepted_axes is None:
                raise ValueError(
                    f"{path} reports[{index}] coverage tag {coverage_tag!r} has "
                    "no matching coverage requirement"
                )
            if not any(
                report_accepts_axis(
                    ReportExpectation(name, Path(report_path), acceptance_mode),
                    axis,
                )
                for axis in accepted_axes
            ):
                raise ValueError(
                    f"{path} reports[{index}] coverage tag {coverage_tag!r} "
                    f"is not accepted by {acceptance_mode!r} mode"
                )

        expectations.append(
            ReportExpectation(
                name=name,
                path=report_manifest_path,
                acceptance_mode=acceptance_mode,
                coverage=coverage_tags,
            )
        )

    return ReportManifest(
        reports=expectations,
        coverage_requirements=coverage_requirements,
    )


def report_acceptance_mode(report: dict[str, Any]) -> str:
    mode = report.get("acceptance_mode")
    return mode if isinstance(mode, str) else ""


def axis_status(report: dict[str, Any], axis: str) -> str:
    axis_report = report.get(axis)
    if not isinstance(axis_report, dict):
        return ""
    status = axis_report.get("status")
    return status if isinstance(status, str) else ""


def report_string_field(report: dict[str, Any], field_name: str) -> str:
    value = report.get(field_name)
    return value if isinstance(value, str) else ""


def required_media_artifacts(
    expectation: ReportExpectation,
    report: dict[str, Any],
) -> list[RequiredMediaArtifact]:
    artifacts: list[RequiredMediaArtifact] = []
    if expectation.acceptance_mode in {"all", "visual"}:
        reference_visual = report_string_field(
            report,
            "reference_video",
        ) or report_string_field(report, "reference_media")
        artifacts.append(
            RequiredMediaArtifact(
                "reference visual media",
                reference_visual,
                REFERENCE_MEDIA_ROOT,
                (".mp4",),
            )
        )
        artifacts.append(
            RequiredMediaArtifact(
                "candidate visual media",
                report_string_field(report, "candidate_media"),
                CANDIDATE_MEDIA_ROOT,
                (".gif",),
            )
        )
    if expectation.acceptance_mode in {"all", "audio"}:
        artifacts.append(
            RequiredMediaArtifact(
                "reference audio",
                report_string_field(report, "reference_audio"),
                REFERENCE_MEDIA_ROOT,
                (".wav",),
            )
        )
        artifacts.append(
            RequiredMediaArtifact(
                "candidate audio",
                report_string_field(report, "candidate_audio"),
                CANDIDATE_MEDIA_ROOT,
                (".wav",),
            )
        )
    return artifacts


def check_required_media_artifacts(
    expectation: ReportExpectation,
    report: dict[str, Any],
) -> list[str]:
    failures: list[str] = []
    for artifact in required_media_artifacts(expectation, report):
        if not artifact.path_text:
            failures.append(f"{expectation.name}: missing {artifact.label} path")
            continue
        path = Path(artifact.path_text)
        if relative_path_is_under(
            expectation.path,
            REPORT_ROOT,
        ) and not relative_path_is_under(path, artifact.root):
            failures.append(
                f"{expectation.name}: {artifact.label} artifact {path} "
                f"is outside {artifact.root}"
            )
            continue
        if path.suffix.lower() not in artifact.suffixes:
            allowed_suffixes = ", ".join(artifact.suffixes)
            failures.append(
                f"{expectation.name}: {artifact.label} artifact {path} "
                f"must use {allowed_suffixes}"
            )
            continue
        if not path.exists():
            failures.append(
                f"{expectation.name}: missing {artifact.label} artifact {path}"
            )
        elif path.stat().st_size == 0:
            failures.append(
                f"{expectation.name}: empty {artifact.label} artifact {path}"
            )
    return failures


def positive_axis_int(report: dict[str, Any], axis: str, field_name: str) -> bool:
    axis_report = report.get(axis)
    if not isinstance(axis_report, dict):
        return False
    value = axis_report.get(field_name)
    return isinstance(value, int) and value > 0


def check_axis_evidence_counts(
    expectation: ReportExpectation,
    report: dict[str, Any],
) -> list[str]:
    failures: list[str] = []
    if expectation.acceptance_mode in {"all", "visual"}:
        for field_name in ("reference_frames", "candidate_frames", "compared_frames"):
            if not positive_axis_int(report, "visual", field_name):
                failures.append(
                    f"{expectation.name}: visual {field_name} is not positive"
                )
    if expectation.acceptance_mode in {"all", "audio"}:
        for field_name in (
            "reference_samples",
            "candidate_samples",
            "compared_samples",
        ):
            if not positive_axis_int(report, "audio", field_name):
                failures.append(
                    f"{expectation.name}: audio {field_name} is not positive"
                )
    return failures


def axis_failures(report: dict[str, Any], axis: str) -> list[object]:
    axis_report = report.get(axis)
    if not isinstance(axis_report, dict):
        return []
    failures = axis_report.get("failures")
    return failures if isinstance(failures, list) else []


def check_accepted_axis_failures(
    expectation: ReportExpectation,
    report: dict[str, Any],
) -> list[str]:
    failures: list[str] = []
    for axis in ("visual", "audio"):
        if not report_accepts_axis(expectation, axis):
            continue
        axis_failure_list = axis_failures(report, axis)
        if axis_failure_list:
            failures.append(
                f"{expectation.name}: {axis} failures are present on accepted axis"
            )
    return failures


def check_report(expectation: ReportExpectation) -> list[str]:
    failures: list[str] = []
    if not expectation.path.exists():
        return [f"{expectation.name}: missing report {expectation.path}"]

    report = load_json(expectation.path)
    status = report.get("status")
    if status != "pass":
        failures.append(f"{expectation.name}: top-level status is {status!r}")

    actual_mode = report_acceptance_mode(report)
    if actual_mode != expectation.acceptance_mode:
        failures.append(
            f"{expectation.name}: acceptance_mode is {actual_mode!r}, "
            f"expected {expectation.acceptance_mode!r}"
        )

    if expectation.acceptance_mode in {"all", "visual"}:
        visual_status = axis_status(report, "visual")
        if visual_status != "pass":
            failures.append(
                f"{expectation.name}: visual status is {visual_status!r}"
            )

    if expectation.acceptance_mode in {"all", "audio"}:
        audio_status = axis_status(report, "audio")
        if audio_status != "pass":
            failures.append(f"{expectation.name}: audio status is {audio_status!r}")

    failures.extend(check_required_media_artifacts(expectation, report))
    failures.extend(check_axis_evidence_counts(expectation, report))
    failures.extend(check_accepted_axis_failures(expectation, report))
    return failures


def check_reports(expectations: list[ReportExpectation]) -> list[str]:
    failures: list[str] = []
    seen_paths: set[Path] = set()
    for expectation in expectations:
        if expectation.path in seen_paths:
            failures.append(f"{expectation.name}: duplicate report path")
            continue
        seen_paths.add(expectation.path)
        failures.extend(check_report(expectation))
    return failures


def report_accepts_axis(expectation: ReportExpectation, axis: str) -> bool:
    if axis == "all":
        return expectation.acceptance_mode == "all"
    return expectation.acceptance_mode in {"all", axis}


def check_coverage_requirements(
    expectations: list[ReportExpectation],
    requirements: list[CoverageRequirement],
) -> list[str]:
    failures: list[str] = []
    for requirement in requirements:
        matching_reports = [
            expectation.name
            for expectation in expectations
            if requirement.coverage in expectation.coverage
            and report_accepts_axis(expectation, requirement.accepted_axis)
        ]
        if len(matching_reports) < requirement.min_reports:
            failures.append(
                f"{requirement.name}: {len(matching_reports)} accepted "
                f"{requirement.accepted_axis} report(s) cover "
                f"{requirement.coverage!r}, expected at least "
                f"{requirement.min_reports}"
            )
    return failures


def markdown_cell(value: object) -> str:
    return str(value).replace("|", "\\|").replace("\n", " ")


def metric_text(value: object) -> str:
    if isinstance(value, float):
        return f"{value:.3f}"
    if isinstance(value, bool):
        return "true" if value else "false"
    if value is None:
        return "-"
    return str(value)


def visual_metric(report: dict[str, Any], region_name: str, metric_name: str) -> object:
    visual = report.get("visual")
    if not isinstance(visual, dict):
        return None
    if region_name == "full":
        region = visual.get("full")
        return region.get(metric_name) if isinstance(region, dict) else None
    regions = visual.get("regions")
    if not isinstance(regions, list):
        return None
    for region in regions:
        if isinstance(region, dict) and region.get("name") == region_name:
            return region.get(metric_name)
    return None


def visual_summary(report: dict[str, Any], accepted: bool) -> str:
    status = axis_status(report, "visual") or "-"
    summary = (
        f"{status}; full RMS {metric_text(visual_metric(report, 'full', 'rms'))}; "
        f"playfield RMS {metric_text(visual_metric(report, 'playfield', 'rms'))}; "
        f"laser RMS {metric_text(visual_metric(report, 'laser_band', 'rms'))}"
    )
    return summary if accepted else f"diagnostic only; {summary}"


def audio_summary(report: dict[str, Any], accepted: bool) -> str:
    audio = report.get("audio")
    if not isinstance(audio, dict):
        return "-"
    status = axis_status(report, "audio") or "-"
    summary = (
        f"{status}; env {metric_text(audio.get('envelope_correlation'))}; "
        f"rms ratio {metric_text(audio.get('rms_ratio'))}; "
        f"zcr ratio {metric_text(audio.get('zero_crossing_ratio'))}; "
        f"noise-like {metric_text(audio.get('noise_like_pass'))}"
    )
    return summary if accepted else f"diagnostic only; {summary}"


def report_reference_path(report: dict[str, Any]) -> str:
    for key in ("reference_video", "reference_media"):
        value = report.get(key)
        if isinstance(value, str) and value:
            return value
    return "-"


def report_candidate_path(report: dict[str, Any]) -> str:
    value = report.get("candidate_media")
    return value if isinstance(value, str) and value else "-"


def required_artifact_count(
    expectation: ReportExpectation,
    report: dict[str, Any],
) -> int:
    return len(required_media_artifacts(expectation, report))


def accepted_axis_count(expectations: list[ReportExpectation], axis: str) -> int:
    return sum(1 for expectation in expectations if report_accepts_axis(expectation, axis))


def int_field(report: dict[str, Any], field_name: str) -> int:
    value = report.get(field_name)
    if isinstance(value, int):
        return value
    return 0


def window_scan_astcnt_summary(report: dict[str, Any]) -> str:
    counts = report.get("terrain_astcnt_counts")
    if not isinstance(counts, dict) or not counts:
        return "-"
    return ", ".join(f"{key}={counts[key]}" for key in sorted(counts))


def window_scan_exclusions(report: dict[str, Any]) -> str:
    exclusions = report.get("excluded_path_fragments")
    if not isinstance(exclusions, list) or not exclusions:
        return "-"
    return ", ".join(str(item) for item in exclusions)


def window_scan_nearest_sound_object_miss(report: dict[str, Any]) -> str:
    misses = report.get("nearest_sound_object_misses")
    if not isinstance(misses, list) or not misses:
        return "-"
    miss = misses[0]
    if not isinstance(miss, dict):
        return "-"
    nearest = miss.get("nearest_object")
    label = "-"
    if isinstance(nearest, dict):
        label = str(nearest.get("label") or "-")
    commands = miss.get("commands")
    command_text = (
        ", ".join(str(command) for command in commands)
        if isinstance(commands, list)
        else "-"
    )
    return (
        f"sound {miss.get('sound_frame', '-')} {command_text}; "
        f"{label} at {miss.get('object_frame', '-')} "
        f"({miss.get('distance_frames', '-')} frame(s))"
    )


def window_scan_object_label_counts(report: dict[str, Any]) -> str:
    counts = report.get("object_label_counts")
    if not isinstance(counts, dict) or not counts:
        return "-"
    return ", ".join(f"{key}={counts[key]}" for key in sorted(counts))


def window_scan_object_span_summary(report: dict[str, Any]) -> str:
    spans = report.get("object_label_best_spans") or report.get("object_label_spans")
    if not isinstance(spans, list) or not spans:
        return "-"
    parts: list[str] = []
    for span in spans[:6]:
        if not isinstance(span, dict):
            continue
        parts.append(
            f"{span.get('label', '-')} frames "
            f"{span.get('start_frame', '-')}-{span.get('end_frame', '-')} "
            f"({span.get('duration_frames', '-')} frame(s))"
        )
    return "; ".join(parts) if parts else "-"


def window_scan_terrain_process_misses(report: dict[str, Any]) -> str:
    hits = report.get("terrain_process_hits")
    if not isinstance(hits, list) or not hits:
        return "-"
    parts: list[str] = []
    for hit in hits[:3]:
        if not isinstance(hit, dict):
            continue
        parts.append(
            f"frame {hit.get('frame', '-')} ASTCNT {hit.get('astcnt', '-')}"
        )
    return ", ".join(parts) if parts else "-"


def accepted_report_names(
    expectations: list[ReportExpectation],
    requirement: CoverageRequirement,
) -> list[str]:
    return [
        expectation.name
        for expectation in expectations
        if requirement.coverage in expectation.coverage
        and report_accepts_axis(expectation, requirement.accepted_axis)
    ]


def build_summary(
    manifest: ReportManifest,
    window_scan_reports: list[WindowScanExpectation] | None = None,
) -> str:
    loaded_reports = [
        (expectation, load_json(expectation.path)) for expectation in manifest.reports
    ]
    window_scan_reports = window_scan_reports or []
    mode_counts = {
        mode: sum(
            1
            for expectation in manifest.reports
            if expectation.acceptance_mode == mode
        )
        for mode in sorted(VALID_ACCEPTANCE_MODES)
    }

    lines = [
        "<!-- markdownlint-disable MD013 -->",
        "",
        "# Defender Reference Report Signoff Summary",
        "",
        "Generated from `docs/fidelity/reference-report-gate.json` and local",
        "`target/reference-media/**/report.json` artifacts. This file is an",
        "owner-review aid; `make reference-report-gate` remains the acceptance",
        "gate.",
        "",
        "## Gate Summary",
        "",
        f"- Reports: `{len(manifest.reports)}`",
        f"- All-axis reports: `{mode_counts['all']}`",
        f"- Audio-only reports: `{mode_counts['audio']}`",
        f"- Visual-only reports: `{mode_counts['visual']}`",
        f"- Coverage requirements: `{len(manifest.coverage_requirements)}`",
        f"- Required non-empty media artifacts: `{sum(required_artifact_count(expectation, report) for expectation, report in loaded_reports)}`",
        f"- Accepted visual comparisons: `{accepted_axis_count(manifest.reports, 'visual')}`",
        f"- Accepted audio comparisons: `{accepted_axis_count(manifest.reports, 'audio')}`",
        "",
        "## Coverage Matrix",
        "",
        "| Requirement | Coverage | Axis | Minimum Reports | Accepted Reports |",
        "| --- | --- | --- | --- | --- |",
    ]

    for requirement in manifest.coverage_requirements:
        report_names = accepted_report_names(manifest.reports, requirement)
        lines.append(
            "| "
            + " | ".join(
                markdown_cell(cell)
                for cell in (
                    requirement.name,
                    requirement.coverage,
                    requirement.accepted_axis,
                    requirement.min_reports,
                    ", ".join(report_names),
                )
            )
            + " |"
        )

    lines.extend(
        [
            "",
            "## Report Matrix",
            "",
            "| Report | Manifest Mode | Report Mode | Coverage | Visual Summary | Audio Summary | Reference Media | Candidate Media |",
            "| --- | --- | --- | --- | --- | --- | --- | --- |",
        ]
    )

    for expectation, report in loaded_reports:
        lines.append(
            "| "
            + " | ".join(
                markdown_cell(cell)
                for cell in (
                    expectation.name,
                    expectation.acceptance_mode,
                    report_acceptance_mode(report),
                    ", ".join(expectation.coverage),
                    visual_summary(
                        report,
                        expectation.acceptance_mode in {"all", "visual"},
                    ),
                    audio_summary(
                        report,
                        expectation.acceptance_mode in {"all", "audio"},
                    ),
                    report_reference_path(report),
                    report_candidate_path(report),
                )
            )
            + " |"
        )

    if window_scan_reports:
        lines.extend(
            [
                "",
                "## Reference Window Scans",
                "",
                "These scans are proof-boundary evidence, not accepted media reports.",
                "",
                "| Scan | Path | Exclusions | Expected | Debug | Target Sound Hits | Object Hits | Object Labels | Long Object Spans | Candidate Windows | Nearest Sound/Object Miss | Terrain Status Rows | TERBLO Rows | Last-Human Terrain Candidates | TERBLO Misses | ASTCNT Values |",
                "| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |",
            ]
        )
        for expectation in window_scan_reports:
            if expectation.path.exists():
                report = load_json(expectation.path)
                row = (
                    expectation.name,
                    expectation.path,
                    window_scan_exclusions(report),
                    int_field(report, "expected_trace_count"),
                    int_field(report, "debug_trace_count"),
                    int_field(report, "target_sound_hit_count"),
                    int_field(report, "object_hit_count"),
                    window_scan_object_label_counts(report),
                    window_scan_object_span_summary(report),
                    int_field(report, "candidate_count"),
                    window_scan_nearest_sound_object_miss(report),
                    int_field(report, "terrain_status_hit_count"),
                    int_field(report, "terrain_terblo_process_hit_count"),
                    int_field(report, "terrain_last_human_candidate_count"),
                    window_scan_terrain_process_misses(report),
                    window_scan_astcnt_summary(report),
                )
            else:
                row = (
                    expectation.name,
                    expectation.path,
                    "missing report",
                    "-",
                    "-",
                    "-",
                    "-",
                    "-",
                    "-",
                    "-",
                    "-",
                    "-",
                    "-",
                    "-",
                    "-",
                    "-",
                )
            lines.append(
                "| "
                + " | ".join(markdown_cell(cell) for cell in row)
                + " |"
            )

    lines.append("")
    return "\n".join(lines)


def main() -> int:
    args = parse_args()
    manifest_path = Path(args.manifest)
    manifest = load_manifest(manifest_path)
    expectations = manifest.reports
    failures = check_reports(expectations)
    failures.extend(
        check_coverage_requirements(expectations, manifest.coverage_requirements)
    )
    if failures:
        print("reference report gate failed:")
        for failure in failures:
            print(f"- {failure}")
        return 1

    if args.summary_out:
        summary_path = Path(args.summary_out)
        summary_path.parent.mkdir(parents=True, exist_ok=True)
        if args.window_scan_report:
            window_scan_reports = [
                parse_window_scan_report_arg(value)
                for value in args.window_scan_report
            ]
        else:
            window_scan_reports = default_window_scan_reports()
        summary_path.write_text(
            build_summary(manifest, window_scan_reports),
            encoding="utf-8",
        )
        print(f"wrote reference report summary: {summary_path}")

    mode_counts = {
        mode: sum(1 for expectation in expectations if expectation.acceptance_mode == mode)
        for mode in sorted(VALID_ACCEPTANCE_MODES)
    }
    print(
        "reference report gate passed: "
        f"{len(expectations)} reports "
        f"(all={mode_counts['all']}, audio={mode_counts['audio']}, "
        f"visual={mode_counts['visual']}), "
        f"{len(manifest.coverage_requirements)} coverage requirements"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
