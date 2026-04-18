#!/usr/bin/env python3
"""Extract the red-label Defender wave table from blk71.src.

The generated records are written into `assets/arcade/arcade-rules.txt`, which
is embedded by the Rust runtime so gameplay does not depend on a local
ROM/source checkout at compile time or runtime.
"""

from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SRC_PATH = Path("/tmp/defender-src/src/blk71.src")
ARCADE_RULES_PATH = ROOT / "assets/arcade/arcade-rules.txt"
START_MARKER = "# BEGIN RED_LABEL_WAVE_TABLE"
END_MARKER = "# END RED_LABEL_WAVE_TABLE"

RECORDS = [
    ("landers", "LANDERS"),
    ("bombers", "TIES"),
    ("pods", "PROBES"),
    ("mutants", "SCHITZOS"),
    ("swarmers", "SWARMERS"),
    ("wave_time", "WAVE TIME"),
    ("wave_size", "WAVE SIZE"),
    ("lander_x_velocity", "LANDER XV"),
    ("lander_y_velocity_msb", "LANDER YV MSB"),
    ("lander_y_velocity_lsb", "LSB"),
    ("lander_shot_time", "LDSTIM"),
    ("bomber_x_velocity", "TIE XV"),
    ("mutant_random_y", "SZRY"),
    ("mutant_y_velocity_msb", "SZYV MSB"),
    ("mutant_y_velocity_lsb", '" LSB'),
    ("mutant_x_velocity", "SZXV"),
    ("mutant_shot_time", "SZSTIM"),
    ("swarmer_x_velocity", "SWXV"),
    ("swarmer_shot_time", "SWSTIM"),
    ("swarmer_acceleration_mask", "SWAC"),
    ("baiter_time", "UFOTIM"),
    ("baiter_shot_time", "UFSTIM"),
    ("baiter_seek_probability", "UFOSK"),
]


def main() -> None:
    lines = SRC_PATH.read_text().splitlines()
    start = next(index for index, line in enumerate(lines) if line.startswith("WVTAB"))
    end = next(index for index, line in enumerate(lines[start + 1 :], start + 1) if line.startswith("WVTEND"))

    rows: list[list[int]] = []
    for line in lines[start:end]:
        stripped = line.split(";", 1)[0].strip()
        if "FCB" not in stripped:
            continue
        rhs = stripped.split("FCB", 1)[1]
        values = [parse_token(token.strip()) for token in rhs.split(",") if token.strip()]
        rows.append(values)

    if len(rows) != len(RECORDS) * 2:
        raise SystemExit(f"expected {len(RECORDS) * 2} FCB rows, found {len(rows)}")

    output = [
        START_MARKER,
        "# Extracted from blk71.src WVTAB in the Williams red-label source release.",
        "# Format: name=max,min,intra_delta,inter_delta|wave1,wave2,wave3,wave4",
    ]

    row_iter = iter(rows)
    for (name, comment), limits, waves in zip(RECORDS, row_iter, row_iter):
        output.append(
            f"{name}="
            f"{format_signed(limits[0], signed=False)},"
            f"{format_signed(limits[1], signed=False)},"
            f"{format_signed(limits[2])},"
            f"{format_signed(limits[3])}|"
            f"{format_signed(waves[0], signed=False)},"
            f"{format_signed(waves[1], signed=False)},"
            f"{format_signed(waves[2], signed=False)},"
            f"{format_signed(waves[3], signed=False)}"
            f" # {comment}"
        )

    output.append(END_MARKER)

    arcade_rules = ARCADE_RULES_PATH.read_text()
    start = arcade_rules.find(START_MARKER)
    end = arcade_rules.find(END_MARKER)
    if start == -1 or end == -1 or end < start:
        raise SystemExit("red-label wave markers not found in arcade-rules.txt")

    end += len(END_MARKER)
    replacement = "\n".join(output)
    updated = f"{arcade_rules[:start]}{replacement}{arcade_rules[end:]}"
    ARCADE_RULES_PATH.write_text(updated)


def parse_token(token: str) -> int:
    if token.startswith("$"):
        return int(token[1:], 16)
    return int(token)


def format_signed(value: int, *, signed: bool = True) -> str:
    if signed and value >= 0x80:
        value -= 0x100
    return str(value)


if __name__ == "__main__":
    main()
