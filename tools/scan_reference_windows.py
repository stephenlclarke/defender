#!/usr/bin/env python3
"""Scan generated MAME traces for bounded Defender fidelity windows.

The scanner is intentionally local and deterministic. It looks for target sound
commands in expected TSVs, non-lander source picture evidence in debug TSVs,
and reports command hits that have nearby object evidence in the same trace.
"""

from __future__ import annotations

import argparse
import json
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Any


DEFAULT_ROOT = Path("target/reference-media/mame")
DEFAULT_TARGET_COMMANDS = ("0xFE", "0xFA", "0xF8", "0xF3")
DEFAULT_OBJECT_PROXIMITY_FRAMES = 24
TERRAIN_BLOW_PROCESS_ADDRESS = "0xEDEA"

DEFAULT_PICTURE_LABELS = {
    "0xF8CE": "SCZP1 converted mutant",
    "0xF8E2": "SWXP1 swarmer explosion",
    "0xF8F7": "PRBP1 pod",
    "0xF93D": "TIEP3 bomber",
    "0xF9B7": "UFOP3 baiter",
    "0xF951": "BXPIC bomb explosion",
}


@dataclass(frozen=True)
class SoundHit:
    path: Path
    frame: int
    commands: tuple[str, ...]

    def to_report(self) -> dict[str, Any]:
        return {
            "path": str(self.path),
            "frame": self.frame,
            "commands": list(self.commands),
        }


@dataclass(frozen=True)
class ObjectHit:
    path: Path
    frame: int
    label: str
    address: str

    def to_report(self) -> dict[str, Any]:
        return {
            "path": str(self.path),
            "frame": self.frame,
            "label": self.label,
            "address": self.address,
        }


@dataclass(frozen=True)
class ObjectSpan:
    path: Path
    label: str
    address: str
    start_frame: int
    end_frame: int
    hit_count: int

    @property
    def duration_frames(self) -> int:
        return self.end_frame - self.start_frame + 1

    def to_report(self) -> dict[str, Any]:
        return {
            "path": str(self.path),
            "label": self.label,
            "address": self.address,
            "start_frame": self.start_frame,
            "end_frame": self.end_frame,
            "duration_frames": self.duration_frames,
            "hit_count": self.hit_count,
        }


@dataclass(frozen=True)
class WindowCandidate:
    expected_path: Path
    debug_path: Path
    sound: SoundHit
    objects: tuple[ObjectHit, ...]

    def to_report(self) -> dict[str, Any]:
        return {
            "expected_path": str(self.expected_path),
            "debug_path": str(self.debug_path),
            "frame": self.sound.frame,
            "commands": list(self.sound.commands),
            "nearby_objects": [hit.to_report() for hit in self.objects],
        }


@dataclass(frozen=True)
class SoundObjectMiss:
    expected_path: Path
    debug_path: Path
    sound: SoundHit
    object: ObjectHit
    distance_frames: int

    def to_report(self) -> dict[str, Any]:
        return {
            "expected_path": str(self.expected_path),
            "debug_path": str(self.debug_path),
            "sound_frame": self.sound.frame,
            "commands": list(self.sound.commands),
            "object_frame": self.object.frame,
            "distance_frames": self.distance_frames,
            "nearest_object": self.object.to_report(),
        }


@dataclass(frozen=True)
class TerrainHit:
    path: Path
    frame: int
    pc: str
    astcnt: str
    terrain_blown: bool
    process_addresses: tuple[str, ...]

    @property
    def has_terrain_blow_process(self) -> bool:
        return TERRAIN_BLOW_PROCESS_ADDRESS in self.process_addresses

    @property
    def is_last_human_candidate(self) -> bool:
        return self.has_terrain_blow_process and self.astcnt == "0x00"

    def to_report(self) -> dict[str, Any]:
        return {
            "path": str(self.path),
            "frame": self.frame,
            "pc": self.pc,
            "astcnt": self.astcnt,
            "terrain_blown": self.terrain_blown,
            "process_addresses": list(self.process_addresses),
            "has_terrain_blow_process": self.has_terrain_blow_process,
            "is_last_human_candidate": self.is_last_human_candidate,
        }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Scan MAME expected/debug TSVs for Defender fidelity windows."
    )
    parser.add_argument("--root", default=str(DEFAULT_ROOT), help="Trace tree to scan.")
    parser.add_argument(
        "--target-command",
        action="append",
        default=[],
        help="Target sound command byte such as 0xFE. Defaults to non-lander bytes.",
    )
    parser.add_argument(
        "--picture",
        action="append",
        default=[],
        help="Picture evidence as address=label, for example 0xF8CE=SCZP1.",
    )
    parser.add_argument(
        "--object-proximity-frames",
        type=int,
        default=DEFAULT_OBJECT_PROXIMITY_FRAMES,
        help="Frame distance for pairing sound hits with object evidence.",
    )
    parser.add_argument(
        "--exclude-path-fragment",
        action="append",
        default=[],
        help="Skip traces whose path contains this fragment. Repeatable.",
    )
    parser.add_argument("--out-json", help="Optional JSON report output path.")
    return parser.parse_args()


def normalized_hex(value: str) -> str:
    text = value.strip().upper()
    if not text:
        raise ValueError("empty hex value")
    if text.startswith("0X"):
        body = text[2:]
    else:
        body = text
    return f"0x{int(body, 16):02X}"


def parse_picture_arg(value: str) -> tuple[str, str]:
    try:
        address, label = value.split("=", 1)
    except ValueError as error:
        raise argparse.ArgumentTypeError(
            f"picture must use address=label, got {value!r}"
        ) from error
    label = label.strip()
    if not label:
        raise argparse.ArgumentTypeError(f"picture label is empty in {value!r}")
    try:
        return normalized_hex(address), label
    except ValueError as error:
        raise argparse.ArgumentTypeError(f"invalid picture address in {value!r}") from error


def read_tsv(path: Path) -> tuple[list[str], list[list[str]]]:
    lines = path.read_text(encoding="utf-8", errors="ignore").splitlines()
    if not lines:
        return [], []
    header = lines[0].split("\t")
    rows = [line.split("\t") for line in lines[1:] if line]
    return header, rows


def column_index(header: list[str], name: str) -> int | None:
    try:
        return header.index(name)
    except ValueError:
        return None


def field(row: list[str], index: int | None) -> str:
    if index is None or index >= len(row):
        return ""
    return row[index]


def split_sound_commands(value: str) -> tuple[str, ...]:
    commands: list[str] = []
    for chunk in value.replace(",", ";").split(";"):
        chunk = chunk.strip()
        if not chunk or chunk == "-":
            continue
        try:
            command = normalized_hex(chunk)
        except ValueError:
            continue
        if int(command[2:], 16) <= 0xFF:
            commands.append(command)
    return tuple(commands)


def normalized_byte(value: str) -> str:
    try:
        return f"0x{int(value.strip(), 16) & 0xFF:02X}"
    except ValueError:
        return value.strip()


def truthy_flag(value: str) -> bool:
    return value.strip().lower() in {"1", "true", "yes"}


def scan_sound_hits(path: Path, target_commands: set[str]) -> list[SoundHit]:
    header, rows = read_tsv(path)
    frame_index = column_index(header, "frame")
    sound_index = column_index(header, "sound_commands")
    if frame_index is None or sound_index is None:
        return []
    hits: list[SoundHit] = []
    for row in rows:
        commands = tuple(
            command for command in split_sound_commands(field(row, sound_index))
            if command in target_commands
        )
        if commands:
            hits.append(SoundHit(path=path, frame=int(field(row, frame_index)), commands=commands))
    return hits


def process_addresses(value: str) -> tuple[str, ...]:
    addresses: list[str] = []
    for match in re.finditer(r"paddr=(0x[0-9a-fA-F]{4})", value):
        address = normalized_hex(match.group(1))
        if address not in addresses:
            addresses.append(address)
    return tuple(addresses)


def scan_terrain_hits(path: Path) -> list[TerrainHit]:
    header, rows = read_tsv(path)
    frame_index = column_index(header, "frame")
    if frame_index is None:
        return []
    pc_index = column_index(header, "pc")
    astcnt_index = column_index(header, "astcnt")
    terrain_blown_index = column_index(header, "terrain_blown")
    active_processes_index = column_index(header, "active_processes")
    if (
        astcnt_index is None
        and terrain_blown_index is None
        and active_processes_index is None
    ):
        return []

    hits: list[TerrainHit] = []
    for row in rows:
        addresses = process_addresses(field(row, active_processes_index))
        terrain_blown = truthy_flag(field(row, terrain_blown_index))
        if not terrain_blown and TERRAIN_BLOW_PROCESS_ADDRESS not in addresses:
            continue
        hits.append(
            TerrainHit(
                path=path,
                frame=int(field(row, frame_index)),
                pc=field(row, pc_index),
                astcnt=normalized_byte(field(row, astcnt_index)),
                terrain_blown=terrain_blown,
                process_addresses=addresses,
            )
        )
    return hits


def scan_object_hits(path: Path, picture_labels: dict[str, str]) -> list[ObjectHit]:
    header, rows = read_tsv(path)
    frame_index = column_index(header, "frame")
    if frame_index is None:
        return []
    evidence_indexes = [
        index
        for name in ("active_objects", "object_slots", "expanded_objects", "shell_objects")
        if (index := column_index(header, name)) is not None
    ]
    hits: list[ObjectHit] = []
    for row in rows:
        frame = int(field(row, frame_index))
        evidence = "\t".join(field(row, index).upper() for index in evidence_indexes)
        for address, label in picture_labels.items():
            if address.upper() in evidence:
                hits.append(ObjectHit(path=path, frame=frame, label=label, address=address))
    return hits


def object_label_counts(hits: list[ObjectHit]) -> dict[str, int]:
    counts: dict[str, int] = {}
    for hit in hits:
        counts[hit.label] = counts.get(hit.label, 0) + 1
    return dict(sorted(counts.items()))


def object_spans(hits: list[ObjectHit]) -> list[ObjectSpan]:
    spans: list[ObjectSpan] = []
    current: ObjectSpan | None = None
    for hit in sorted(hits, key=lambda item: (str(item.path), item.label, item.address, item.frame)):
        if (
            current is not None
            and current.path == hit.path
            and current.label == hit.label
            and current.address == hit.address
            and current.end_frame + 1 == hit.frame
        ):
            current = ObjectSpan(
                path=current.path,
                label=current.label,
                address=current.address,
                start_frame=current.start_frame,
                end_frame=hit.frame,
                hit_count=current.hit_count + 1,
            )
            continue
        if current is not None:
            spans.append(current)
        current = ObjectSpan(
            path=hit.path,
            label=hit.label,
            address=hit.address,
            start_frame=hit.frame,
            end_frame=hit.frame,
            hit_count=1,
        )
    if current is not None:
        spans.append(current)
    return spans


def best_object_spans_by_label(spans: list[ObjectSpan]) -> list[ObjectSpan]:
    best_spans: dict[str, ObjectSpan] = {}
    for span in spans:
        existing = best_spans.get(span.label)
        candidate_key = (-span.duration_frames, span.start_frame, str(span.path))
        existing_key = (
            (-existing.duration_frames, existing.start_frame, str(existing.path))
            if existing is not None
            else None
        )
        if existing is None or candidate_key < existing_key:
            best_spans[span.label] = span
    return [best_spans[label] for label in sorted(best_spans)]


def debug_path_for_expected(expected_path: Path) -> Path:
    name = expected_path.name
    if name.endswith(".expected.tsv"):
        return expected_path.with_name(name.replace(".expected.tsv", ".debug.tsv"))
    return expected_path.with_suffix(".debug.tsv")


def path_is_excluded(path: Path, fragments: tuple[str, ...]) -> bool:
    path_text = path.as_posix().lower()
    return any(fragment.lower() in path_text for fragment in fragments)


def scan_trace_tree(
    root: Path,
    target_commands: set[str],
    picture_labels: dict[str, str],
    proximity_frames: int,
    excluded_path_fragments: tuple[str, ...] = (),
) -> dict[str, Any]:
    sound_hits: list[SoundHit] = []
    object_hits_by_path: dict[Path, list[ObjectHit]] = {}
    terrain_hits_by_path: dict[Path, list[TerrainHit]] = {}
    candidates: list[WindowCandidate] = []
    nearest_sound_object_misses: list[SoundObjectMiss] = []
    expected_paths = [
        path
        for path in sorted(root.glob("**/*.expected.tsv"))
        if not path_is_excluded(path, excluded_path_fragments)
    ]

    for expected_path in expected_paths:
        trace_sound_hits = scan_sound_hits(expected_path, target_commands)
        sound_hits.extend(trace_sound_hits)
        debug_path = debug_path_for_expected(expected_path)
        if not debug_path.exists():
            continue
        trace_object_hits = scan_object_hits(debug_path, picture_labels)
        object_hits_by_path[debug_path] = trace_object_hits
        terrain_hits_by_path[debug_path] = scan_terrain_hits(debug_path)
        for sound_hit in trace_sound_hits:
            nearby = tuple(
                object_hit
                for object_hit in trace_object_hits
                if abs(object_hit.frame - sound_hit.frame) <= proximity_frames
            )
            if nearby:
                candidates.append(
                    WindowCandidate(
                        expected_path=expected_path,
                        debug_path=debug_path,
                        sound=sound_hit,
                        objects=nearby,
                    )
                )
            elif trace_object_hits:
                nearest_object = min(
                    trace_object_hits,
                    key=lambda object_hit: abs(object_hit.frame - sound_hit.frame),
                )
                nearest_sound_object_misses.append(
                    SoundObjectMiss(
                        expected_path=expected_path,
                        debug_path=debug_path,
                        sound=sound_hit,
                        object=nearest_object,
                        distance_frames=abs(nearest_object.frame - sound_hit.frame),
                    )
                )

    all_object_hits = [hit for hits in object_hits_by_path.values() for hit in hits]
    all_terrain_hits = [hit for hits in terrain_hits_by_path.values() for hit in hits]
    terrain_status_hits = [hit for hit in all_terrain_hits if hit.terrain_blown]
    terrain_process_hits = [
        hit for hit in all_terrain_hits if hit.has_terrain_blow_process
    ]
    last_human_terrain_candidates = [
        hit for hit in terrain_process_hits if hit.is_last_human_candidate
    ]
    terrain_astcnt_counts: dict[str, int] = {}
    for hit in all_terrain_hits:
        terrain_astcnt_counts[hit.astcnt] = terrain_astcnt_counts.get(hit.astcnt, 0) + 1
    labels = sorted({hit.label for hit in all_object_hits})
    all_object_spans = object_spans(all_object_hits)
    object_span_reports = [
        span.to_report()
        for span in sorted(
            all_object_spans,
            key=lambda span: (
                -span.duration_frames,
                str(span.path),
                span.label,
                span.start_frame,
            ),
        )[:200]
    ]
    return {
        "root": str(root),
        "excluded_path_fragments": sorted(excluded_path_fragments),
        "target_commands": sorted(target_commands),
        "object_proximity_frames": proximity_frames,
        "expected_trace_count": len(expected_paths),
        "debug_trace_count": len(object_hits_by_path),
        "target_sound_hit_count": len(sound_hits),
        "object_hit_count": len(all_object_hits),
        "object_labels": labels,
        "object_label_counts": object_label_counts(all_object_hits),
        "object_label_spans": object_span_reports,
        "object_label_best_spans": [
            span.to_report() for span in best_object_spans_by_label(all_object_spans)
        ],
        "candidate_count": len(candidates),
        "terrain_trace_count": len(
            [hits for hits in terrain_hits_by_path.values() if hits]
        ),
        "terrain_status_hit_count": len(terrain_status_hits),
        "terrain_terblo_process_hit_count": len(terrain_process_hits),
        "terrain_last_human_candidate_count": len(last_human_terrain_candidates),
        "terrain_astcnt_counts": dict(sorted(terrain_astcnt_counts.items())),
        "target_sound_hits": [hit.to_report() for hit in sound_hits[:200]],
        "window_candidates": [candidate.to_report() for candidate in candidates[:200]],
        "terrain_hits": [hit.to_report() for hit in all_terrain_hits[:200]],
        "terrain_process_hits": [
            hit.to_report() for hit in terrain_process_hits[:200]
        ],
        "terrain_last_human_candidates": [
            hit.to_report() for hit in last_human_terrain_candidates[:200]
        ],
        "nearest_sound_object_misses": [
            miss.to_report()
            for miss in sorted(
                nearest_sound_object_misses,
                key=lambda miss: (
                    miss.distance_frames,
                    str(miss.expected_path),
                    miss.sound.frame,
                ),
            )[:200]
        ],
    }


def report_text(report: dict[str, Any]) -> str:
    lines = [
        f"root: {report['root']}",
        f"expected traces: {report['expected_trace_count']}",
        f"debug traces: {report['debug_trace_count']}",
        f"target commands: {', '.join(report['target_commands'])}",
        f"target sound hits: {report['target_sound_hit_count']}",
        f"object hits: {report['object_hit_count']}",
        f"window candidates: {report['candidate_count']}",
        f"terrain traces: {report.get('terrain_trace_count', 0)}",
        f"terrain status rows: {report.get('terrain_status_hit_count', 0)}",
        f"terrain TERBLO process rows: "
        f"{report.get('terrain_terblo_process_hit_count', 0)}",
        f"terrain last-human candidates: "
        f"{report.get('terrain_last_human_candidate_count', 0)}",
    ]
    if report["excluded_path_fragments"]:
        lines.append(
            "excluded path fragments: "
            + ", ".join(report["excluded_path_fragments"])
        )
    if report["object_labels"]:
        lines.append("object labels: " + ", ".join(report["object_labels"]))
    object_label_counts = report.get("object_label_counts", {})
    if object_label_counts:
        label_summary = ", ".join(
            f"{label}={count}" for label, count in object_label_counts.items()
        )
        lines.append(f"object label counts: {label_summary}")
    object_label_spans = report.get("object_label_spans", [])
    if object_label_spans:
        lines.append("longest object spans:")
        for span in object_label_spans[:10]:
            lines.append(
                f"- {span['label']} frames {span['start_frame']}-{span['end_frame']} "
                f"({span['duration_frames']} frame(s)) path {span['path']}"
            )
    object_label_best_spans = report.get("object_label_best_spans", [])
    if object_label_best_spans:
        lines.append("best object spans by label:")
        for span in object_label_best_spans:
            lines.append(
                f"- {span['label']} frames {span['start_frame']}-{span['end_frame']} "
                f"({span['duration_frames']} frame(s)) path {span['path']}"
            )
    terrain_astcnt_counts = report.get("terrain_astcnt_counts", {})
    if terrain_astcnt_counts:
        astcnt_summary = ", ".join(
            f"{astcnt}={count}" for astcnt, count in terrain_astcnt_counts.items()
        )
        lines.append(f"terrain ASTCNT values: {astcnt_summary}")
    if report["window_candidates"]:
        lines.append("candidate windows:")
        for candidate in report["window_candidates"][:20]:
            labels = ", ".join(
                sorted({hit["label"] for hit in candidate["nearby_objects"]})
            )
            commands = ", ".join(candidate["commands"])
            lines.append(f"- frame {candidate['frame']} commands {commands}: {labels}")
    elif report["target_sound_hits"]:
        lines.append("target sound hits exist, but none have nearby object evidence.")
        nearest_misses = report.get("nearest_sound_object_misses", [])
        if nearest_misses:
            lines.append("nearest sound/object misses:")
            for miss in nearest_misses[:10]:
                commands = ", ".join(miss["commands"])
                nearest = miss["nearest_object"]
                lines.append(
                    f"- sound frame {miss['sound_frame']} commands {commands}: "
                    f"nearest {nearest['label']} at frame {miss['object_frame']} "
                    f"({miss['distance_frames']} frame(s))"
                )
    else:
        lines.append("no target sound hits found in scanned expected traces.")
    if report.get("terrain_last_human_candidates"):
        lines.append("terrain last-human candidates:")
        for candidate in report["terrain_last_human_candidates"][:20]:
            lines.append(
                f"- frame {candidate['frame']} astcnt {candidate['astcnt']} "
                f"pc {candidate['pc']} path {candidate['path']}"
            )
    elif report.get("terrain_terblo_process_hit_count", 0):
        lines.append(
            "terrain TERBLO process rows exist, but none have ASTCNT 0x00."
        )
        process_hits = report.get("terrain_process_hits", [])
        if process_hits:
            lines.append("terrain TERBLO process misses:")
            for hit in process_hits[:10]:
                lines.append(
                    f"- frame {hit['frame']} astcnt {hit['astcnt']} "
                    f"pc {hit['pc']} path {hit['path']}"
                )
    elif report.get("terrain_status_hit_count", 0):
        lines.append("terrain status rows exist, but no TERBLO process rows were found.")
    else:
        lines.append("no terrain status or TERBLO process rows found.")
    return "\n".join(lines)


def main() -> int:
    args = parse_args()
    target_commands = {
        normalized_hex(command)
        for command in (args.target_command or DEFAULT_TARGET_COMMANDS)
    }
    picture_labels = dict(DEFAULT_PICTURE_LABELS)
    for value in args.picture:
        address, label = parse_picture_arg(value)
        picture_labels[address] = label

    report = scan_trace_tree(
        Path(args.root),
        target_commands,
        picture_labels,
        max(0, args.object_proximity_frames),
        tuple(args.exclude_path_fragment),
    )
    if args.out_json:
        output_path = Path(args.out_json)
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(
            json.dumps(report, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )
    print(report_text(report))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
