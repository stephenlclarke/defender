#!/usr/bin/env python3
"""Generate local Defender red-label reference trace fixtures with MAME.

The generated ``*.expected.tsv`` files are local verification artifacts. They
must remain under ``docs/fidelity/fixtures/local/`` or another ignored
directory and must not be committed.
"""

from __future__ import annotations

import argparse
import csv
import os
import shutil
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
SCENARIO_TSV = REPO_ROOT / "assets" / "red-label" / "trace-scenarios.tsv"
SCHEMA_TSV = REPO_ROOT / "assets" / "red-label" / "trace-schema.tsv"
CMOS_DEFAULTS_TSV = REPO_ROOT / "assets" / "red-label" / "cmos-defaults.tsv"
MAME_LUA = REPO_ROOT / "tools" / "mame_defender_trace.lua"
MAME_CMOS_CELLS = 256


@dataclass(frozen=True)
class Scenario:
    name: str
    frames: int
    input_program: str


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Generate local MAME-derived Defender reference trace fixtures."
    )
    parser.add_argument(
        "--mame",
        default=os.environ.get("DEFENDER_MAME", "mame"),
        help="MAME executable path. Defaults to DEFENDER_MAME or mame.",
    )
    parser.add_argument(
        "--rom-dir",
        default=os.environ.get("DEFENDER_ROM_DIR", "assets/roms"),
        help="Directory containing user-supplied red-label ROMs.",
    )
    parser.add_argument(
        "--out-dir",
        default=os.environ.get(
            "DEFENDER_REFERENCE_TRACE_DIR", "docs/fidelity/fixtures/local/reference"
        ),
        help="Ignored local fixture output directory.",
    )
    parser.add_argument(
        "--scenario",
        action="append",
        help="Scenario name to generate. May be repeated. Defaults to all.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Write input scripts and print MAME commands without running MAME.",
    )
    parser.add_argument(
        "--blank-nvram",
        action="store_true",
        help="Leave MAME NVRAM blank instead of seeding romc8.src DEFALT CMOS defaults.",
    )
    return parser.parse_args()


def read_scenarios() -> list[Scenario]:
    with SCENARIO_TSV.open(newline="", encoding="utf-8") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        return [
            Scenario(
                name=row["scenario"],
                frames=int(row["frames"]),
                input_program=row["input_program"],
            )
            for row in reader
        ]


def expand_input_program(program: str) -> str:
    frames: list[str] = []
    for segment in program.split(";"):
        segment = segment.strip()
        if "*" in segment:
            frame, repeat = segment.rsplit("*", 1)
            frames.extend([frame.strip()] * int(repeat))
        else:
            frames.append(segment)
    return ";".join(frames) + "\n"


def source_cmos_default_nvram(defaults_tsv: Path = CMOS_DEFAULTS_TSV) -> bytes:
    cells = bytearray([0xF0] * MAME_CMOS_CELLS)
    with defaults_tsv.open(newline="", encoding="utf-8") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        for row_number, row in enumerate(reader, start=2):
            offset = int(row["offset"], 0)
            declared_cells = int(row["cells"], 0)
            values = [int(value, 16) for value in row["bytes"].split()]
            if declared_cells != len(values) * 2:
                raise ValueError(
                    f"{defaults_tsv}:{row_number}: declared {declared_cells} cell(s) "
                    f"for {len(values)} byte(s)"
                )
            if offset + declared_cells > MAME_CMOS_CELLS:
                raise ValueError(
                    f"{defaults_tsv}:{row_number}: CMOS defaults exceed "
                    f"{MAME_CMOS_CELLS} MAME NVRAM cells"
                )
            for index, value in enumerate(values):
                cells[offset + index * 2] = 0xF0 | ((value >> 4) & 0x0F)
                cells[offset + index * 2 + 1] = 0xF0 | (value & 0x0F)
    return bytes(cells)


def write_source_cmos_default_nvram(nvram_dir: Path) -> Path:
    nvram_path = nvram_dir / "defender" / "nvram"
    nvram_path.parent.mkdir(parents=True, exist_ok=True)
    nvram_path.write_bytes(source_cmos_default_nvram())
    return nvram_path


def select_scenarios(all_scenarios: list[Scenario], requested: list[str] | None) -> list[Scenario]:
    if not requested:
        return all_scenarios

    by_name = {scenario.name: scenario for scenario in all_scenarios}
    missing = [name for name in requested if name not in by_name]
    if missing:
        names = ", ".join(missing)
        raise SystemExit(f"unknown trace scenario(s): {names}")
    return [by_name[name] for name in requested]


def require_executable(executable: str) -> str:
    resolved = shutil.which(executable)
    if resolved is None:
        raise SystemExit(
            f"MAME executable {executable!r} was not found. Set DEFENDER_MAME or pass --mame."
        )
    return resolved


def write_inputs(out_dir: Path, scenario: Scenario) -> Path:
    out_dir.mkdir(parents=True, exist_ok=True)
    inputs_path = out_dir / f"{scenario.name}.inputs.txt"
    inputs_path.write_text(expand_input_program(scenario.input_program), encoding="utf-8")
    return inputs_path


def generate_expected(
    mame: str,
    rom_dir: Path,
    out_dir: Path,
    scenario: Scenario,
    inputs_path: Path,
    dry_run: bool,
    seed_cmos_defaults: bool,
) -> None:
    expected_path = out_dir / f"{scenario.name}.expected.tsv"
    mame_state_dir = out_dir / "mame-state" / scenario.name
    cfg_dir = mame_state_dir / "cfg"
    nvram_dir = mame_state_dir / "nvram"
    env = os.environ.copy()
    env.setdefault("SDL_AUDIODRIVER", "dummy")
    env.setdefault("SDL_VIDEODRIVER", "dummy")
    env.update(
        {
            "DEFENDER_TRACE_INPUTS": str(inputs_path),
            "DEFENDER_TRACE_OUTPUT": str(expected_path),
            "DEFENDER_TRACE_SCHEMA": str(SCHEMA_TSV),
            "DEFENDER_TRACE_FRAMES": str(scenario.frames),
        }
    )
    command = [
        mame,
        "defender",
        "-rompath",
        str(rom_dir),
        "-cfg_directory",
        str(cfg_dir),
        "-nvram_directory",
        str(nvram_dir),
        "-skip_gameinfo",
        "-nothrottle",
        "-video",
        "none",
        "-sound",
        "none",
        "-autoboot_delay",
        "0",
        "-autoboot_script",
        str(MAME_LUA),
    ]

    if dry_run:
        print(" ".join(command))
        return

    if mame_state_dir.exists():
        shutil.rmtree(mame_state_dir)
    cfg_dir.mkdir(parents=True, exist_ok=True)
    nvram_dir.mkdir(parents=True, exist_ok=True)
    if seed_cmos_defaults:
        write_source_cmos_default_nvram(nvram_dir)
    subprocess.run(command, cwd=REPO_ROOT, env=env, check=True)


def main() -> int:
    args = parse_args()
    scenarios = select_scenarios(read_scenarios(), args.scenario)
    out_dir = (REPO_ROOT / args.out_dir).resolve()
    rom_dir = (REPO_ROOT / args.rom_dir).resolve()

    if not args.dry_run:
        mame = require_executable(args.mame)
        if not rom_dir.is_dir():
            raise SystemExit(f"ROM directory {rom_dir} does not exist")
    else:
        mame = args.mame

    for scenario in scenarios:
        inputs_path = write_inputs(out_dir, scenario)
        generate_expected(
            mame,
            rom_dir,
            out_dir,
            scenario,
            inputs_path,
            args.dry_run,
            not args.blank_nvram,
        )

    print(f"processed {len(scenarios)} trace scenario(s) in {out_dir}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
