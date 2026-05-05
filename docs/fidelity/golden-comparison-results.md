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

Result: exact TSV comparison failed at line 2, frame 1. This is the original
baseline comparison; `DC-05.5` below records the current narrowed result after
the source-shaped boot/start-ready work.

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
- Boot/start-ready process and super-process RAM state or scheduling was not
  exact enough at this point to promote `attract_boot` as a passing exact
  golden test.
- Later `DC-04` work should add an ignored or failing fixture test for this
  mismatch before changing source-exact boot, executive, or scheduler code.

## 2026-05-04 `DC-05.5` Attract Boot Recheck

Scenario: `attract_boot`

Purpose: re-run the same 900-frame cold boot/attract readiness comparison after
the source-shaped `DC-05` boot, RAM-fill, SINIT, INIT20, board-latch, and
mutation-surface work.

Command:

```sh
cargo run --quiet -- \
  --fidelity-check-trace \
  docs/fidelity/fixtures/local/reference/attract_boot.inputs.txt \
  docs/fidelity/fixtures/local/reference/attract_boot.expected.tsv
```

Result: exact TSV comparison now matches through frame 732 and fails first at
line 734, frame 733.

First mismatch:

- `process_table_crc32`: expected `0x62E1AD30`, actual `0xA424BDF6`.

Column summary:

- `process_table_crc32` differed on 168 of 900 frames. The first mismatch was
  line 734, frame 733. The last mismatch was line 901, frame 900, where the
  reference expected `0x1A0C7932` and Rust produced `0xA424BDF6`.
- All other trace columns matched for the 900-frame comparison: input bits,
  MAME input-port bytes, phase, scores, wave, lives, smart bombs, RNG bytes,
  object-table CRC, super-process-table CRC, shell-table CRC, video CRC
  placeholder, sound commands, and events.

Interpretation:

- The local reference now proves the cold-boot reset/RAM-fill, SINIT clear,
  INIT20 process/super-process/object-list setup, sound command, RNG state, and
  start-ready handoff through frame 732.
- The remaining `attract_boot` exact-fixture blocker is the first post-INIT20
  ATTR/executive scheduler cadence at frame 733 and later process-table state.

## 2026-05-04 `DC-06.4` Process/Object Order Recheck

Scenario: `attract_boot`

Purpose: re-run the local object/process/super-process/shell CRC comparison
after the `DC-06.1` through `DC-06.3` translated scheduler work.

Commands:

```sh
make reference-fixtures-check
cargo run --quiet -- \
  --fidelity-check-trace \
  docs/fidelity/fixtures/local/reference/attract_boot.inputs.txt \
  docs/fidelity/fixtures/local/reference/attract_boot.expected.tsv
cargo run --quiet -- \
  --fidelity-trace-inputs-file \
  docs/fidelity/fixtures/local/reference/attract_boot.inputs.txt \
  > /tmp/dc064-attract_boot.actual.tsv
```

Result: the local reference fixture set is present and valid with 12 complete
Phase 1 fixtures and 22,308 frames. The exact `attract_boot` comparison still
matches through frame 732 and fails first at line 734, frame 733.

First mismatch:

- `process_table_crc32`: expected `0x62E1AD30`, actual `0xA424BDF6`.

Column summary:

- `process_table_crc32` differed on 168 of 900 frames. The first mismatch was
  line 734, frame 733. The last mismatch was line 901, frame 900, where the
  reference expected `0x1A0C7932` and Rust produced `0xA424BDF6`.
- All other trace columns matched for the 900-frame comparison: input bits,
  MAME input-port bytes, phase, scores, wave, lives, smart bombs, RNG bytes,
  object-table CRC, super-process-table CRC, shell-table CRC, video CRC
  placeholder, sound commands, and events.

Interpretation:

- The translated scheduler work did not regress the source-visible object,
  super-process, or shell ordering that already matched the local
  `attract_boot` reference.
- Exact process-table order/state remains blocked at the same post-INIT20
  ATTR/executive cadence boundary. The ignored
  `local_reference_attract_boot_matches_red_label` test stays in place until
  that source timing gap is closed.

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

## 2026-05-04 `DC-04.3` Ignored Local Reference Tests

The current known local reference mismatches are encoded as ignored Rust tests
in `src/app.rs`:

- `local_reference_attract_boot_matches_red_label`
- `local_reference_start_game_matches_red_label`
- `local_reference_firing_matches_red_label`
- `local_reference_thrust_reverse_matches_red_label`
- `local_reference_smart_bomb_matches_red_label`
- `local_reference_hyperspace_matches_red_label`
- `local_reference_death_matches_red_label`
- `local_reference_wave_advance_matches_red_label`

Default validation skips these tests:

```sh
cargo test local_reference_ --all-targets
```

When local reference fixtures exist, explicitly running the ignored tests
currently fails through the exact TSV comparison path:

```sh
cargo test local_reference_ --all-targets -- --ignored
```

These tests should stay ignored until the corresponding boot, credited-start,
death, and wave-advance trace gaps are fixed. When a gap is fixed, unignore the
matching test or narrow it to the remaining mismatched scenario.

## 2026-05-05 `DC-15` Trace Oracle Recheck

Purpose: refresh the local trace-oracle state after the native video CRC work,
the later gameplay translation cycles, and the `DC-15` plan update.

Commands:

```sh
make reference-fixtures-check
cargo run --quiet -- \
  --fidelity-check-trace \
  docs/fidelity/fixtures/local/reference/attract_boot.inputs.txt \
  docs/fidelity/fixtures/local/reference/attract_boot.expected.tsv
cargo run --quiet -- \
  --fidelity-write-scenario-inputs \
  docs/fidelity/fixtures/local/rust-current
make trace-fixtures
cargo test local_reference_ --all-targets -- --ignored
```

Reference fixture result:

- `make reference-fixtures-check` passed with 12 complete Phase 1 local
  reference fixtures and 22,308 frames.

Rust-current fixture result:

- Current Rust fixtures were generated and checked for 10 Phase 1 scenarios:
  `abduction`, `attract_boot`, `death`, `firing`, `first_300_frames`,
  `hyperspace`, `smart_bomb`, `start_game`, `thrust_reverse`, and
  `wave_advance`.
- `make trace-fixtures` passed for those 10 ignored local pairs with 15,452
  frames.
- `planet_destruction` and `high_score_entry` were not promoted into
  `rust-current` because current trace generation panics at
  `src/machine.rs:26276` with `red-label OFREE object list is empty`.

`attract_boot` exact result:

- Exact comparison now fails first at line 2, frame 1, because the local MAME
  reference still has `video_crc32=-` while current Rust emits `0x157E98C7`.
- Across 900 frames, `video_crc32` differs on all 900 frames because the local
  reference fixture has no native video CRC values.
- The previous narrowed process-table mismatch is still present:
  `process_table_crc32` differs on 168 frames, first at line 734/frame 733
  where the reference expects `0x62E1AD30` and Rust emits `0xA424BDF6`.
- All other columns match for `attract_boot`: inputs, phase, scores, wave,
  lives, smart bombs, RNG bytes, object-table CRC, super-process-table CRC,
  shell-table CRC, sound commands, and events.

Focused scenario drift summary:

- `start_game`: `video_crc32` differs on all 1,228 frames; `phase`, `wave`,
  `lives`, and `smart_bombs` differ on 203 frames; `seed`, `hseed`, and
  `lseed` differ on 326, 327, and 325 frames; `object_table_crc32` differs on
  55 frames; `process_table_crc32` differs on 496 frames.
- `first_300_frames`, `firing`, `thrust_reverse`, `smart_bomb`, and
  `hyperspace`: `video_crc32` differs on all 1,328 frames; `phase`, `wave`,
  `lives`, and `smart_bombs` differ on 303 frames; `seed`, `hseed`, and
  `lseed` differ on 425 frames each; `object_table_crc32` differs on 155
  frames; `process_table_crc32` differs on 596 frames.
- `abduction` and `death`: `video_crc32` differs on all 1,928 frames; `phase`,
  `wave`, `lives`, and `smart_bombs` differ on 903 frames; `seed`, `hseed`,
  and `lseed` differ on 1,022, 1,024, and 1,022 frames;
  `object_table_crc32` differs on 755 frames; `process_table_crc32` differs on
  1,196 frames.
- `wave_advance`: `video_crc32` differs on all 2,828 frames; `phase`, `wave`,
  `lives`, and `smart_bombs` differ on 1,803 frames; `seed`, `hseed`, and
  `lseed` differ on 1,919, 1,919, and 1,915 frames;
  `object_table_crc32` differs on 1,655 frames; `process_table_crc32` differs
  on 2,096 frames.

No mismatches were observed in these columns for the 10 compared scenarios:

- `input_bits`
- `input_in0`
- `input_in1`
- `input_in2`
- `p1_score`
- `p2_score`
- `super_process_table_crc32`
- `shell_table_crc32`
- `sound_commands`
- `events`

Ignored local-reference tests:

- `cargo test local_reference_ --all-targets -- --ignored` ran all eight
  local-reference tests. All eight failed at line 2/frame 1 on the missing
  reference `video_crc32` value.
- No local-reference test was unignored. The ignored reasons in `src/app.rs`
  were narrowed to `DC-15` and now name the missing-reference-video-CRC blocker
  plus each scenario's remaining process, credited-start, gameplay, death, or
  wave drift.

Interpretation:

- The checked local reference traces remain present and structurally valid, but
  they are no longer schema-complete for exact comparison because the MAME-side
  reference fixtures do not populate `video_crc32`.
- At this point, once the reference video column was regenerated or
  intentionally normalized, the next `attract_boot` blocker remained the
  post-`INIT20` process-table cadence at frame 733. `DC-16.3` supersedes this
  boundary with the frame-746 result below.
- Longer gameplay/session traces still show the same broad state classes of
  drift: credited-start phase timing, RNG call order, object-table state, and
  process-table state. The current counts differ from the older `DC-04.2`
  baseline because later translation work changed the Rust side.

## 2026-05-05 `DC-16.3` Attract Cadence Recheck

Scenario: `attract_boot`

Purpose: re-run the 900-frame cold boot/attract readiness comparison after the
cold-boot `GameOver` `ATTR` handoff began applying the source-visible
frame-733 and frame-739 process mutations.

Commands:

```sh
cargo run --quiet -- \
  --fidelity-check-trace \
  docs/fidelity/fixtures/local/reference/attract_boot.inputs.txt \
  docs/fidelity/fixtures/local/reference/attract_boot.expected.tsv
cargo run --quiet -- \
  --fidelity-trace-inputs-file \
  docs/fidelity/fixtures/local/reference/attract_boot.inputs.txt \
  > /tmp/dc16-attract_boot.actual.tsv
```

Exact result:

- Exact comparison still fails first at line 2, frame 1, because the local MAME
  reference has `video_crc32=-` while current Rust emits `0x157E98C7`.
- Ignoring that absent reference `video_crc32` column, Rust now matches the
  local reference through frame 745 and first fails at line 747, frame 746.

First non-video mismatch:

- `process_table_crc32`: expected `0xF9878193`, actual `0xE2155086`.

Column summary:

- `video_crc32` differs on all 900 frames because the local reference fixture
  does not populate native video CRCs.
- `process_table_crc32` now differs on 155 of 900 frames. The first non-video
  mismatch is line 747, frame 746. The last mismatch is line 901, frame 900,
  where the reference expects `0x1A0C7932` and Rust emits `0xE2155086`.
- All other trace columns match for `attract_boot`: input bits, MAME input-port
  bytes, phase, scores, wave, lives, smart bombs, RNG bytes, object-table CRC,
  super-process-table CRC, shell-table CRC, sound commands, and events.

Interpretation:

- The source-visible `ATTR` scheduler row and initial `AMODES` support-process
  mutations now match the reference from frame 733 through frame 745.
- The remaining process-table blocker is the next executive cadence slice at
  frame 746, where the source reference has advanced to the next process-table
  sequence while Rust still holds the in-flight `ATTR`/Williams-page process
  state at `0xE2155086`.

## 2026-05-05 `DC-17.1` Attract Non-Video Closure

Scenario: `attract_boot`

Purpose: re-run the 900-frame cold boot/attract readiness comparison after the
frame-746 cold-boot executive color-process cadence was modeled.

Commands:

```sh
cargo run --quiet -- \
  --fidelity-trace-inputs-file \
  docs/fidelity/fixtures/local/reference/attract_boot.inputs.txt \
  > /tmp/dc17-attract_boot.actual.tsv
cargo run --quiet -- \
  --fidelity-check-trace \
  docs/fidelity/fixtures/local/reference/attract_boot.inputs.txt \
  docs/fidelity/fixtures/local/reference/attract_boot.expected.tsv
```

Exact result:

- Exact comparison still fails first at line 2, frame 1, because the local MAME
  reference has `video_crc32=-` while current Rust emits `0x157E98C7`.

Column summary:

- Ignoring the absent reference `video_crc32` column, all 900 `attract_boot`
  frames now match every non-video trace column.
- The previously open `process_table_crc32` drift is closed: object-table CRC,
  process-table CRC, super-process-table CRC, shell-table CRC, inputs, phase,
  scores, wave, lives, smart bombs, RNG bytes, sound commands, and events all
  match the local reference through frame 900.

Interpretation:

- `attract_boot` is ready to promote once the local reference fixture gains
  MAME-derived `video_crc32` values or the acceptance policy explicitly permits
  comparing this scenario without the video column.

## 2026-05-05 `DC-18.1` Gameplay Reference Recheck

Scenarios:

- `start_game`
- `firing`
- `thrust_reverse`
- `smart_bomb`
- `hyperspace`

Purpose: re-run the focused credited-start and player-action comparisons after
`DC-16` and `DC-17` closed the boot/attract process-table drift and promoted
frame-output ownership surfaces.

Commands:

```sh
cargo run --quiet -- \
  --fidelity-check-reference-trace-dir \
  docs/fidelity/fixtures/local/reference
cargo test local_reference_ --all-targets -- --ignored
target/debug/defender --fidelity-trace-inputs-file \
  docs/fidelity/fixtures/local/reference/start_game.inputs.txt
```

The final command was repeated for `firing`, `thrust_reverse`, `smart_bomb`,
and `hyperspace`, with an ad hoc TSV column-count comparison against each local
`*.expected.tsv` reference.

Reference fixture result:

- The local reference directory is complete for 12 Phase 1 fixtures and 22,308
  frames.
- The ignored exact local-reference tests still fail first at line 2/frame 1
  because the local MAME references have `video_crc32=-` while current Rust
  emits native CRCs starting with `0x157E98C7`.

Line counts:

- `start_game`: 1,228 frames plus header.
- `firing`: 1,328 frames plus header.
- `thrust_reverse`: 1,328 frames plus header.
- `smart_bomb`: 1,328 frames plus header.
- `hyperspace`: 1,328 frames plus header.

First non-video mismatch:

- All five focused gameplay scenarios now match non-video columns through frame
  900.
- The first remaining non-video mismatch is line 902/frame 901 in `seed`, where
  the reference expects `0x81` and Rust emits `0xDB`.

Column summary:

- `start_game`: `video_crc32` differs on all 1,228 frames; `phase`, `wave`,
  `lives`, and `smart_bombs` differ on 203 frames; `seed`, `hseed`, and
  `lseed` differ on 324, 325, and 324 frames; `object_table_crc32` differs on
  55 frames; `process_table_crc32` differs on 325 frames.
- `firing`, `thrust_reverse`, `smart_bomb`, and `hyperspace`: `video_crc32`
  differs on all 1,328 frames; `phase`, `wave`, `lives`, and `smart_bombs`
  differ on 303 frames; `seed`, `hseed`, and `lseed` differ on 423, 425, and
  424 frames; `object_table_crc32` differs on 155 frames;
  `process_table_crc32` differs on 425 frames.

No mismatches were observed in these columns for the five focused gameplay
scenarios:

- `input_bits`
- `input_in0`
- `input_in1`
- `input_in2`
- `p1_score`
- `p2_score`
- `super_process_table_crc32`
- `shell_table_crc32`
- `sound_commands`
- `events`

Interpretation:

- `DC-17` removed the inherited boot/attract process-table blocker from the
  focused gameplay slices. The next exact gameplay blocker is the frame-901 RNG
  call-order/player-start transition drift, followed by object/process-table
  drift after the credited start.
- The ignored `local_reference_start_game_matches_red_label`,
  `local_reference_firing_matches_red_label`,
  `local_reference_thrust_reverse_matches_red_label`,
  `local_reference_smart_bomb_matches_red_label`, and
  `local_reference_hyperspace_matches_red_label` tests remain ignored until the
  missing reference video column and the frame-901 gameplay drift are closed or
  explicitly scoped.

## 2026-05-05 `DC-18.2` Credited-Start RNG Recheck

Scenarios:

- `start_game`
- `firing`
- `thrust_reverse`
- `smart_bomb`
- `hyperspace`

Purpose: fix the first credited coin/start RNG call-order drift found by
`DC-18.1`, then remeasure the remaining gameplay mismatch boundary.

Commands:

```sh
cargo test trace_text_advances_rand_on_first_credited_coin_frame --all-targets
cargo test \
  trace_text_aligns_delayed_coin_credit_event_with_source_sound_command \
  --all-targets
cargo test \
  trace_text_aligns_debounced_start_event_with_source_sound_command \
  --all-targets
cargo run --quiet -- \
  --fidelity-check-trace \
  docs/fidelity/fixtures/local/reference/start_game.inputs.txt \
  docs/fidelity/fixtures/local/reference/start_game.expected.tsv
make trace-fixtures
make fidelity
```

Exact result:

- Exact comparison still fails first at line 2/frame 1 because the local MAME
  reference has `video_crc32=-` while current Rust emits `0x157E98C7`.
- Ignoring that absent reference `video_crc32` column, the first mismatch is
  now `process_table_crc32` at line 902/frame 901. The frame-901 RNG state now
  matches the reference.
- Ignored local Rust-current expected traces were refreshed from current Rust
  output after the RNG fix. `make trace-fixtures` matched 10 fixture pairs and
  15,452 frames; `make fidelity` passed with 31/31 added executable Rust lines
  covered.

Column summary:

- `start_game`: `video_crc32` differs on all 1,228 frames; `phase`, `wave`,
  `lives`, and `smart_bombs` differ on 203 frames; `seed`, `hseed`, and
  `lseed` differ on 211, 210, and 210 frames; `object_table_crc32` differs on
  55 frames; `process_table_crc32` differs on 325 frames.
- `firing`, `thrust_reverse`, `smart_bomb`, and `hyperspace`: `video_crc32`
  differs on all 1,328 frames; `phase`, `wave`, `lives`, and `smart_bombs`
  differ on 303 frames; `seed`, `hseed`, and `lseed` differ on 311, 310, and
  310 frames; `object_table_crc32` differs on 155 frames;
  `process_table_crc32` differs on 425 frames.

First remaining non-video mismatches:

- `process_table_crc32`: line 902/frame 901, expected `0xDEFE9590`, actual
  `0x640191A2`.
- `seed`/`hseed`/`lseed`: line 1019/frame 1018, expected
  `0x44/0xE2/0x2F`, actual `0x65/0x71/0x17`.
- `phase`: line 1027/frame 1026, expected `game_over`, actual `playing`.
- `object_table_crc32`: line 1175/frame 1174, expected `0x81575057`, actual
  `0xD4938A5C`.

Interpretation:

- The first coin frame no longer skips the frame-level `RAND` advance. The
  power-on handoff now suppresses that advance only when the attract executive
  slice already ran `EXEC`/`RAND`.
- The remaining gameplay blocker is scheduler/state timing rather than the
  first credited input RNG call order: process-table cadence diverges when coin
  input interrupts the attract process, and Rust applies `START` player setup
  earlier than the local reference.

## 2026-05-05 `DC-18.3` Credited-Start Scheduler Recheck

Scenarios:

- `start_game`
- `firing`
- `thrust_reverse`
- `smart_bomb`
- `hyperspace`

Purpose: keep the cold-boot attract process cadence running after a credit is
awarded, then remeasure the remaining credited-start scheduler/player setup
drift.

Commands:

```sh
cargo test \
  trace_text_keeps_cold_boot_attract_process_cadence_after_credit \
  --all-targets
cargo run --quiet -- \
  --fidelity-trace-inputs-file \
  docs/fidelity/fixtures/local/reference/start_game.inputs.txt
```

The final command was repeated for `firing`, `thrust_reverse`, `smart_bomb`,
and `hyperspace`, with the generated TSVs compared against the local reference
fixtures.

Exact result:

- Exact comparison still fails first at line 2/frame 1 because the local MAME
  reference has `video_crc32=-` while current Rust emits `0x157E98C7`.
- Ignoring that absent reference `video_crc32` column, the first mismatch
  remains `process_table_crc32` at line 902/frame 901, expected `0xDEFE9590`,
  actual `0x640191A2`.
- The post-credit process table no longer freezes after `credit_added`; the
  trace now continues the cold-boot attract cadence after frame 912. Exact
  process ordering is still wrong, so this is a narrowed scheduler-order
  blocker rather than a passing gameplay fixture.

Column summary:

- `start_game`: `video_crc32` differs on all 1,228 frames; `phase`, `wave`,
  `lives`, and `smart_bombs` differ on 203 frames; `seed`, `hseed`, and
  `lseed` differ on 211, 210, and 210 frames; `object_table_crc32` differs on
  55 frames; `process_table_crc32` differs on 328 frames.
- `firing`, `thrust_reverse`, `smart_bomb`, and `hyperspace`: `video_crc32`
  differs on all 1,328 frames; `phase`, `wave`, `lives`, and `smart_bombs`
  differ on 303 frames; `seed`, `hseed`, and `lseed` differ on 311, 310, and
  310 frames; `object_table_crc32` differs on 155 frames;
  `process_table_crc32` differs on 428 frames.

First remaining non-video mismatches:

- `process_table_crc32`: line 902/frame 901, expected `0xDEFE9590`, actual
  `0x640191A2`.
- `seed`/`hseed`/`lseed`: line 1019/frame 1018, expected
  `0x44/0xE2/0x2F`, actual `0x65/0x71/0x17`.
- `phase`: line 1027/frame 1026, expected `game_over`, actual `playing`.
- `object_table_crc32`: line 1175/frame 1174, expected `0x81575057`, actual
  `0xD4938A5C`.

Interpretation:

- The remaining trace blocker is the source scheduler/sample boundary around
  coin/start work, not the first credited-input RNG advance and not a frozen
  post-credit attract cadence.
- A generic full-scheduler swap is not safe yet: it reaches the currently
  untranslated red-label `0xF4CC` attract sleep-return path during the
  credited-start window. That path needs a source-shaped owner before the live
  coin/start path can stop prioritizing targeted switch processes.

## 2026-05-05 `DC-19.1` Death And Wave Recheck

Scenarios:

- `death`
- `wave_advance`

Purpose: remeasure the long gameplay reference slices after `DC-18.3` so the
death/wave gap register reflects current runtime behavior before the remaining
session/operator proof work.

Commands:

```sh
cargo run --quiet -- \
  --fidelity-trace-inputs-file \
  docs/fidelity/fixtures/local/reference/death.inputs.txt
cargo run --quiet -- \
  --fidelity-trace-inputs-file \
  docs/fidelity/fixtures/local/reference/wave_advance.inputs.txt
```

Exact result:

- Exact comparison still fails first at line 2/frame 1 because the local MAME
  references have `video_crc32=-` while current Rust emits `0x157E98C7`.
- Ignoring that absent reference `video_crc32` column, both scenarios first
  fail at line 902/frame 901 in `process_table_crc32`, expected `0xDEFE9590`,
  actual `0x640191A2`.
- The long death/wave drift is therefore downstream of the same credited-start
  scheduler/sample boundary recorded for `DC-18.3`; it is not yet a separate
  proven death-tail or wave-advance routine failure.

Column summary:

- `death`: `video_crc32` differs on all 1,928 frames; `phase`, `wave`, `lives`,
  and `smart_bombs` differ on 903 frames; `seed`, `hseed`, and `lseed` differ
  on 909, 908, and 908 frames; `object_table_crc32` differs on 755 frames;
  `process_table_crc32` differs on 1,028 frames.
- `wave_advance`: `video_crc32` differs on all 2,828 frames; `phase`, `wave`,
  `lives`, and `smart_bombs` differ on 1,803 frames; `seed`, `hseed`, and
  `lseed` differ on 1,807, 1,807, and 1,808 frames; `object_table_crc32`
  differs on 1,655 frames; `process_table_crc32` differs on 1,928 frames.

Interpretation:

- `local_reference_death_matches_red_label` and
  `local_reference_wave_advance_matches_red_label` stay ignored. Their ignore
  reasons now point at the current `DC-19` measurement rather than the older
  `DC-15` broad drift label.
- Two-player, high-score, operator/service, and cabinet-profile behavior is
  currently covered by source-native mutation fixtures. End-to-end MAME golden
  traces for those paths remain absent and are carried as explicit pre-refactor
  fidelity gaps until local references can be generated and promoted.
