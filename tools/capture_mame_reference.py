#!/usr/bin/env python3
"""Capture local Defender red-label MAME reference media.

Generated recordings are local golden artifacts for visual/audio comparison and
must stay in ignored output directories.
"""

from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
TOOLS_DIR = Path(__file__).resolve().parent
if str(TOOLS_DIR) not in sys.path:
    sys.path.insert(0, str(TOOLS_DIR))

from generate_reference_traces import (
    MAME_LUA,
    SCHEMA_TSV,
    Scenario,
    expand_input_program,
    read_scenarios,
    select_scenarios,
    source_cmos_default_nvram,
)

DEFAULT_OUT_DIR = REPO_ROOT / "target" / "reference-media" / "mame"
DEFAULT_ROM_DIR = REPO_ROOT / "assets" / "roms"
DEFAULT_BASENAME = "defender-red-label-golden"
DEFAULT_SECONDS = 60
DEFAULT_WIDTH = 768
DEFAULT_HEIGHT = 576
DEFAULT_SAMPLE_RATE = 48_000
CMOS_ALL_TIME_HIGH_SCORE_CELL_OFFSET = 0x1D
CMOS_HIGH_SCORE_ENTRY_CELLS = 12
CMOS_HIGH_SCORE_ENTRIES = 8
CMOS_HIGH_SCORE_BYTES = 3


@dataclass(frozen=True)
class CapturePaths:
    out_dir: Path
    cfg_dir: Path
    nvram_dir: Path
    input_dir: Path
    state_dir: Path
    snapshot_dir: Path
    diff_dir: Path
    comment_dir: Path
    share_dir: Path
    trace_dir: Path
    raw_avi: Path
    wav: Path
    mp4: Path


@dataclass(frozen=True)
class ScriptedCapture:
    inputs_path: Path
    output_path: Path
    debug_path: Path
    sound_dac_path: Path
    schema_path: Path
    frame_limit: int


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Record local Defender red-label MAME video and audio."
    )
    parser.add_argument("--mame", default="mame", help="MAME executable.")
    parser.add_argument("--ffmpeg", default="ffmpeg", help="ffmpeg executable.")
    parser.add_argument(
        "--rom-dir",
        default=str(DEFAULT_ROM_DIR),
        help="Directory containing the local red-label defender ROM directory.",
    )
    parser.add_argument(
        "--out-dir",
        default=str(DEFAULT_OUT_DIR),
        help="Ignored output directory for generated media.",
    )
    parser.add_argument("--seconds", type=int, default=DEFAULT_SECONDS)
    parser.add_argument("--basename", default=DEFAULT_BASENAME)
    parser.add_argument("--width", type=int, default=DEFAULT_WIDTH)
    parser.add_argument("--height", type=int, default=DEFAULT_HEIGHT)
    parser.add_argument("--sample-rate", type=int, default=DEFAULT_SAMPLE_RATE)
    parser.add_argument(
        "--scenario",
        help=(
            "Trace scenario name from assets/red-label/trace-scenarios.tsv. "
            "When set, MAME is driven by the same input program used by clean "
            "candidate capture."
        ),
    )
    parser.add_argument(
        "--input-program",
        help=(
            "Explicit semicolon-delimited input program. Mutually exclusive "
            "with --scenario."
        ),
    )
    parser.add_argument(
        "--blank-nvram",
        action="store_true",
        help="Do not seed source CMOS defaults before capture.",
    )
    parser.add_argument(
        "--zero-high-scores",
        action="store_true",
        help=(
            "Seed source CMOS defaults with all all-time high-score values set "
            "to zero. Useful for deterministic high-score-entry reference clips."
        ),
    )
    parser.add_argument(
        "--keep-raw",
        action="store_true",
        help="Keep MAME's very large raw AVI after MP4 compression.",
    )
    parser.add_argument(
        "--trace-only",
        action="store_true",
        help=(
            "Run scripted MAME with video and sound disabled, writing only "
            "expected/debug trace TSVs. Requires --scenario or --input-program."
        ),
    )
    parser.add_argument(
        "--state-steer",
        choices=(
            "afall_fall",
            "afall_player_catch",
            "afall_safe_landing",
            "terrain_blow",
            "enemy_explosion_matrix",
            "enemy_materialize_matrix",
            "sound_command_matrix",
            "sound_baiter_hit",
            "sound_bomber_hit",
            "sound_command_f3",
            "sound_command_f8",
            "sound_command_fa",
            "sound_command_fe",
            "sound_pod_hit",
            "sound_swarmer_hit",
            "sound_swarmer_shot",
            "sound_tie_hit",
        ),
        help=(
            "Optional trace-tool RAM steering mode for isolating a red-label "
            "routine or expanded-object path in MAME. Requires --scenario or "
            "--input-program."
        ),
    )
    parser.add_argument(
        "--state-steer-frame",
        type=int,
        default=1400,
        help="Scripted frame where --state-steer is applied. Defaults to 1400.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print the MAME and ffmpeg commands without running them.",
    )
    return parser.parse_args()


def resolve_repo_path(path_text: str) -> Path:
    path = Path(path_text)
    if not path.is_absolute():
        path = REPO_ROOT / path
    return path.resolve()


def require_executable(executable: str) -> str:
    resolved = shutil.which(executable)
    if resolved is None:
        raise SystemExit(f"required executable not found: {executable}")
    return resolved


def capture_paths(out_dir: Path, basename: str) -> CapturePaths:
    return CapturePaths(
        out_dir=out_dir,
        cfg_dir=out_dir / "cfg",
        nvram_dir=out_dir / "nvram",
        input_dir=out_dir / "input",
        state_dir=out_dir / "state",
        snapshot_dir=out_dir / "raw",
        diff_dir=out_dir / "diff",
        comment_dir=out_dir / "comments",
        share_dir=out_dir / "share",
        trace_dir=out_dir / "traces",
        raw_avi=out_dir / "raw" / f"{basename}.avi",
        wav=out_dir / f"{basename}.wav",
        mp4=out_dir / f"{basename}.mp4",
    )


def zeroed_high_score_cmos_defaults() -> bytes:
    cells = bytearray(source_cmos_default_nvram())
    for entry in range(CMOS_HIGH_SCORE_ENTRIES):
        entry_offset = CMOS_ALL_TIME_HIGH_SCORE_CELL_OFFSET + entry * CMOS_HIGH_SCORE_ENTRY_CELLS
        for byte_index in range(CMOS_HIGH_SCORE_BYTES):
            cell_offset = entry_offset + byte_index * 2
            cells[cell_offset] = 0xF0
            cells[cell_offset + 1] = 0xF0
    return bytes(cells)


def write_capture_nvram(paths: CapturePaths, image: bytes) -> None:
    nvram_path = paths.nvram_dir / "defender" / "nvram"
    nvram_path.parent.mkdir(parents=True, exist_ok=True)
    nvram_path.write_bytes(image)


def prepare_capture_dirs(paths: CapturePaths, seed_cmos_defaults: bool, zero_high_scores: bool) -> None:
    for directory in (
        paths.cfg_dir,
        paths.nvram_dir,
        paths.input_dir,
        paths.state_dir,
        paths.snapshot_dir,
        paths.diff_dir,
        paths.comment_dir,
        paths.share_dir,
        paths.trace_dir,
    ):
        directory.mkdir(parents=True, exist_ok=True)

    if zero_high_scores:
        write_capture_nvram(paths, zeroed_high_score_cmos_defaults())
    elif seed_cmos_defaults:
        write_capture_nvram(paths, source_cmos_default_nvram())


def build_verify_command(mame: str, rom_dir: Path) -> list[str]:
    return [mame, "-rompath", str(rom_dir), "-verifyroms", "defender"]


def build_mame_capture_command(
    mame: str,
    rom_dir: Path,
    paths: CapturePaths,
    seconds: int | None,
    width: int,
    height: int,
    sample_rate: int,
    basename: str,
    scripted: ScriptedCapture | None = None,
    trace_only: bool = False,
) -> list[str]:
    command = [
        mame,
        "defender",
        "-rompath",
        str(rom_dir),
        "-skip_gameinfo",
        "-nothrottle",
        "-cfg_directory",
        str(paths.cfg_dir),
        "-nvram_directory",
        str(paths.nvram_dir),
        "-input_directory",
        str(paths.input_dir),
        "-state_directory",
        str(paths.state_dir),
        "-snapshot_directory",
        str(paths.snapshot_dir),
        "-diff_directory",
        str(paths.diff_dir),
        "-comment_directory",
        str(paths.comment_dir),
        "-share_directory",
        str(paths.share_dir),
    ]
    if trace_only:
        command.extend(["-video", "none", "-sound", "none"])
    else:
        command.extend(
            [
                "-window",
                "-video",
                "soft",
                "-sound",
                "coreaudio",
                "-samplerate",
                str(sample_rate),
                "-snapsize",
                f"{width}x{height}",
                "-snapview",
                "native",
                "-nosnapbilinear",
                "-aviwrite",
                basename,
                "-wavwrite",
                str(paths.wav),
            ]
        )
    if scripted is None:
        command[command.index("-skip_gameinfo") + 1:command.index("-nothrottle")] = [
            "-seconds_to_run",
            str(seconds),
        ]
    else:
        command.extend(
            [
                "-autoboot_delay",
                "0",
                "-autoboot_script",
                str(MAME_LUA),
            ]
        )
    return command


def build_scripted_capture_env(
    scripted: ScriptedCapture,
    trace_only: bool = False,
    state_steer: str | None = None,
    state_steer_frame: int | None = None,
) -> dict[str, str]:
    env = {
        "DEFENDER_TRACE_INPUTS": str(scripted.inputs_path),
        "DEFENDER_TRACE_OUTPUT": str(scripted.output_path),
        "DEFENDER_TRACE_DEBUG": str(scripted.debug_path),
        "DEFENDER_TRACE_SOUND_DAC_OUTPUT": str(scripted.sound_dac_path),
        "DEFENDER_TRACE_SCHEMA": str(scripted.schema_path),
        "DEFENDER_TRACE_FRAMES": str(scripted.frame_limit),
    }
    if trace_only:
        env["DEFENDER_TRACE_SKIP_VIDEO_CRC"] = "1"
    if state_steer:
        env["DEFENDER_TRACE_STEER"] = state_steer
        env["DEFENDER_TRACE_STEER_FRAME"] = str(state_steer_frame)
    return env


def scripted_capture_from_args(args: argparse.Namespace, paths: CapturePaths) -> ScriptedCapture | None:
    if args.scenario and args.input_program:
        raise SystemExit("--scenario and --input-program are mutually exclusive")
    if not args.scenario and not args.input_program:
        return None

    if args.scenario:
        scenario = select_scenarios(read_scenarios(), [args.scenario])[0]
    else:
        expanded = expand_input_program(args.input_program)
        scenario = Scenario(
            name=args.basename,
            frames=count_expanded_input_frames(expanded),
            input_program=args.input_program,
        )

    expanded_inputs = expand_input_program(scenario.input_program)
    frame_limit = count_expanded_input_frames(expanded_inputs)
    if frame_limit != scenario.frames:
        raise SystemExit(
            f"scripted input program expands to {frame_limit} frame(s), "
            f"but scenario declares {scenario.frames}"
        )

    inputs_path = paths.trace_dir / f"{args.basename}.inputs.txt"
    output_path = paths.trace_dir / f"{args.basename}.expected.tsv"
    debug_path = paths.trace_dir / f"{args.basename}.debug.tsv"
    sound_dac_path = paths.trace_dir / f"{args.basename}.sound-dac.tsv"
    inputs_path.parent.mkdir(parents=True, exist_ok=True)
    inputs_path.write_text(expanded_inputs, encoding="utf-8")
    return ScriptedCapture(
        inputs_path=inputs_path,
        output_path=output_path,
        debug_path=debug_path,
        sound_dac_path=sound_dac_path,
        schema_path=SCHEMA_TSV,
        frame_limit=frame_limit,
    )


def count_expanded_input_frames(expanded_inputs: str) -> int:
    body = expanded_inputs.strip()
    if not body:
        return 0
    return len(body.split(";"))


def build_ffmpeg_command(ffmpeg: str, paths: CapturePaths) -> list[str]:
    return [
        ffmpeg,
        "-hide_banner",
        "-loglevel",
        "error",
        "-y",
        "-i",
        str(paths.raw_avi),
        "-c:v",
        "libx264",
        "-preset",
        "veryfast",
        "-crf",
        "18",
        "-pix_fmt",
        "yuv420p",
        "-c:a",
        "aac",
        "-b:a",
        "128k",
        str(paths.mp4),
    ]


def run_command(command: list[str], dry_run: bool, env: dict[str, str] | None = None) -> None:
    if dry_run:
        if env:
            print(" ".join(f"{key}={value}" for key, value in sorted(env.items())))
        print(" ".join(command))
        return
    run_env = None
    if env:
        run_env = os.environ.copy()
        run_env.update(env)
    subprocess.run(command, cwd=REPO_ROOT, env=run_env, check=True)


def require_non_empty_output(path: Path, label: str) -> None:
    if not path.is_file():
        raise SystemExit(f"{label} output was not created: {path}")
    if path.stat().st_size == 0:
        raise SystemExit(f"{label} output is empty: {path}")


def verify_media_outputs(paths: CapturePaths) -> None:
    require_non_empty_output(paths.mp4, "MP4")
    require_non_empty_output(paths.wav, "WAV")


def main() -> int:
    args = parse_args()
    mame = require_executable(args.mame)
    ffmpeg = None if args.trace_only else require_executable(args.ffmpeg)
    rom_dir = resolve_repo_path(args.rom_dir)
    out_dir = resolve_repo_path(args.out_dir)
    paths = capture_paths(out_dir, args.basename)

    if not rom_dir.is_dir():
        raise SystemExit(f"ROM directory does not exist: {rom_dir}")
    if args.seconds <= 0:
        raise SystemExit("--seconds must be positive")
    if args.trace_only and not (args.scenario or args.input_program):
        raise SystemExit("--trace-only requires --scenario or --input-program")
    if args.state_steer and not (args.scenario or args.input_program):
        raise SystemExit("--state-steer requires --scenario or --input-program")
    if args.state_steer and args.state_steer_frame <= 0:
        raise SystemExit("--state-steer-frame must be positive")

    if not args.dry_run:
        prepare_capture_dirs(paths, not args.blank_nvram, args.zero_high_scores)
        if not args.trace_only:
            paths.raw_avi.unlink(missing_ok=True)
            paths.wav.unlink(missing_ok=True)
            paths.mp4.unlink(missing_ok=True)

    scripted = scripted_capture_from_args(args, paths)
    run_command(build_verify_command(mame, rom_dir), args.dry_run)
    run_command(
        build_mame_capture_command(
            mame,
            rom_dir,
            paths,
            None if scripted else args.seconds,
            args.width,
            args.height,
            args.sample_rate,
            args.basename,
            scripted,
            args.trace_only,
        ),
        args.dry_run,
        build_scripted_capture_env(
            scripted,
            args.trace_only,
            args.state_steer,
            args.state_steer_frame,
        )
        if scripted
        else None,
    )
    if not args.trace_only:
        assert ffmpeg is not None
        run_command(build_ffmpeg_command(ffmpeg, paths), args.dry_run)

    if not args.trace_only and not args.dry_run and not args.keep_raw:
        paths.raw_avi.unlink(missing_ok=True)

    if args.trace_only:
        assert scripted is not None
        print(f"wrote {scripted.output_path}")
        print(f"wrote {scripted.debug_path}")
        print(f"wrote {scripted.sound_dac_path}")
    else:
        if not args.dry_run:
            verify_media_outputs(paths)
        print(f"wrote {paths.mp4}")
        print(f"wrote {paths.wav}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
