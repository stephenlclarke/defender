# Characterization Test Pattern

Characterization tests protect translated red-label behavior before the later
large refactor. They should assert source-visible bytes and trace rows, not
only Rust-facing return values.

## When To Add One

Add or extend a characterization test when a translated routine mutates more
than one source-owned surface, schedules or kills processes, rewrites object
cells, touches CMOS, changes visible video RAM, emits sound commands, or affects
future trace rows.

Good targets include source routines such as `CSCAN`, `SSCAN`, `ST1`,
`PLSTR3`, `HALL6`, scheduler primitives, shell/object scanners, and any
gameplay routine that will be difficult to reason about after module splitting.

## Required Shape

1. Name the source routine or source path in the test name.
2. Arrange the machine through public or existing source-shaped helpers.
3. Capture before-state from source-owned bytes with `src/test_support.rs`.
4. Execute one routine boundary or one frame boundary.
5. Assert exact after-state for every intended mutation.
6. Assert important non-mutated source-owned ranges stay unchanged.
7. Prefer trace-row equality or replay checks when the behavior affects future
   frames.

The important test question is: if the implementation is refactored later, will
this test prove that the same red-label bytes changed in the same way?

## Snapshot Helpers

Use the shared helpers instead of ad hoc byte reads:

- `red_label_ram_snapshot` for named RAM fields and list heads.
- `red_label_cmos_snapshot` for packed CMOS/SRAM cells.
- `red_label_video_ram_snapshot` for visible video bytes.
- `red_label_object_cell_snapshot` for one object table entry.
- `red_label_process_cell_snapshot` for one normal process entry.
- `red_label_super_process_cell_snapshot` for one super-process entry.
- `red_label_shell_list_head_snapshot` for the `SPTR` shell-list head.

Each snapshot can assert:

- `assert_current_unchanged` when a source-owned range must not move.
- `assert_current_changed` when exact bytes are unstable but mutation is
  required.
- `assert_current_changed_to` when the translated source routine defines exact
  after bytes.

## Trace And Replay Checks

Use trace rows when behavior must remain equivalent across frames:

- Compare `TraceFrame::from_output(...).to_tsv_line()` before and after
  `snapshot` / `restore` or `save_state` / `restore_state`.
- Include process-table, super-process-table, object-table, shell-list,
  video-frame, sound-command, and event columns when those surfaces are relevant
  to the routine.
- Do not add broad trace columns speculatively. Add a column only when a
  translated source routine needs that observable state for equivalence.

## Refactor Safety Rules

- Tests should be deterministic and local; do not require user ROMs, MAME, or
  generated golden fixtures unless the test is explicitly ignored or the local
  fixture path is optional.
- Prefer routine-boundary tests near the implementation for byte-level
  mutations, then add trace fixture checks when MAME/source expected traces are
  available.
- Keep expected byte ranges small and named after the source meaning, not the
  current Rust struct that happens to own them.
- If exact bytes are not known yet, assert the smallest useful mutation and add
  the missing source proof to `docs/fidelity/gaps.md` or `SPEC.md`.
- Never replace mutation assertions with only high-level snapshots. Returned
  values can stay correct while red-label RAM, CMOS, object lists, process
  lists, video RAM, or sound-command side effects drift.
