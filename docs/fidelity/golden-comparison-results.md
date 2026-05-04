# Golden Comparison Results

This file records local Rust-vs-reference trace comparisons. The reference
fixtures remain ignored local artifacts; only the comparison findings are
checked in.

## 2026-05-04 `DC-04.1` Attract Boot

Scenario: `attract_boot`

Purpose: compare the 900-frame cold boot/attract readiness trace against the
local MAME/source golden reference before promoting broader golden tests.

Command:

```sh
cargo run --quiet -- \
  --fidelity-check-trace \
  docs/fidelity/fixtures/local/reference/attract_boot.inputs.txt \
  docs/fidelity/fixtures/local/reference/attract_boot.expected.tsv
```

Result: exact TSV comparison failed at line 2, frame 1.

First mismatch:

- `process_table_crc32`: expected `0xC4C53DA1`, actual `0xB2B258E3`.
- `super_process_table_crc32`: expected `0x05B7E865`, actual `0x5EDF4A6B`.

Column summary:

- `process_table_crc32` differed on 723 of 900 frames. The first mismatch was
  line 2, frame 1. The last mismatch was line 901, frame 900, where the
  reference expected `0x1A0C7932` and Rust produced `0xB2B258E3`.
- `super_process_table_crc32` differed on 553 of 900 frames. The first mismatch
  was line 2, frame 1. The last mismatch was line 732, frame 731, where the
  reference expected `0x5EDF4A6B` and Rust produced `0x05B7E865`.
- All other trace columns matched for the 900-frame comparison: input bits,
  MAME input-port bytes, phase, scores, wave, lives, smart bombs, RNG bytes,
  object-table CRC, shell-table CRC, video CRC placeholder, sound commands, and
  events.

Interpretation:

- The current Rust trace has source-visible object and shell-list state aligned
  with the local `attract_boot` reference for this window.
- Boot/start-ready process and super-process RAM state or scheduling is still
  not exact enough to promote `attract_boot` as a passing exact golden test.
- Later `DC-04` work should add an ignored or failing fixture test for this
  mismatch before changing source-exact boot, executive, or scheduler code.

## 2026-05-04 `DC-04.2` Focused Gameplay Slices

Scenarios:

- `start_game`
- `firing`
- `thrust_reverse`
- `smart_bomb`
- `hyperspace`
- `death`
- `wave_advance`

Purpose: compare the focused credited-start, player-input, death, and
wave-advance traces against local MAME/source references.

Each exact comparison failed first at line 2, frame 1, with the same inherited
boot/start-ready `process_table_crc32` and `super_process_table_crc32` mismatch
recorded for `DC-04.1`.

Line counts matched for every generated Rust trace and local reference trace:

- `start_game`: 1,228 frames plus header.
- `firing`: 1,328 frames plus header.
- `thrust_reverse`: 1,328 frames plus header.
- `smart_bomb`: 1,328 frames plus header.
- `hyperspace`: 1,328 frames plus header.
- `death`: 1,928 frames plus header.
- `wave_advance`: 2,828 frames plus header.

Drift summary:

- `start_game`: `phase`, `wave`, `lives`, and `smart_bombs` differed on 203
  frames; `seed`, `hseed`, and `lseed` differed on 324, 325, and 324 frames;
  `object_table_crc32` differed on 55 frames; `process_table_crc32` differed
  on 1,051 frames; `super_process_table_crc32` differed on 553 frames.
- `firing`, `thrust_reverse`, `smart_bomb`, and `hyperspace` had the same
  counts: `phase`, `wave`, `lives`, and `smart_bombs` differed on 303 frames;
  `seed`, `hseed`, and `lseed` differed on 423, 425, and 424 frames;
  `object_table_crc32` differed on 155 frames; `process_table_crc32` differed
  on 1,151 frames; `super_process_table_crc32` differed on 553 frames.
- `death`: `phase`, `wave`, `lives`, and `smart_bombs` differed on 903 frames;
  `seed`, `hseed`, and `lseed` differed on 1,022, 1,024, and 1,024 frames;
  `object_table_crc32` differed on 755 frames; `process_table_crc32` differed
  on 1,751 frames; `super_process_table_crc32` differed on 553 frames.
- `wave_advance`: `phase`, `wave`, `lives`, and `smart_bombs` differed on
  1,803 frames; `seed`, `hseed`, and `lseed` differed on 1,918, 1,923, and
  1,921 frames; `object_table_crc32` differed on 1,655 frames;
  `process_table_crc32` differed on 2,651 frames;
  `super_process_table_crc32` differed on 553 frames.

No mismatches were observed in these columns for the compared scenarios:

- `input_bits`
- `input_in0`
- `input_in1`
- `input_in2`
- `p1_score`
- `p2_score`
- `video_crc32`
- `sound_commands`
- `events`

Interpretation:

- Input script expansion and MAME input-port projection are aligned for the
  focused scenarios.
- The credited-start evidence carried by sound commands and events still
  aligns, but the current Rust core enters gameplay state too early relative to
  the local MAME/source reference.
- The largest remaining blockers are source-exact boot/start-ready process
  state, credited-start state transition timing, RNG call order, and
  object/process scheduler execution after play begins.
