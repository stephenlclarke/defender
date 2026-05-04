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
