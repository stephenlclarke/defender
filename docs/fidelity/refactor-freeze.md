# Refactor Freeze

This file is the `DC-23` refactor contract. The broad module-split refactor was
deferred until the game was fully ROM-complete and playable and the
post-acceptance `wgpu` presentation backend was accepted. Those acceptance gates
are now complete, and the refactor is active on `red-label-refactor`. Refactor
slices may move code between modules, but they must not change the observable
red-label behavior listed here unless `SPEC.md`, `PLAN.md`, and the relevant
fidelity gap record are updated first.

## Frozen Validation Suite

After final acceptance, run this suite before every meaningful module split:

```sh
make fidelity
```

`make fidelity` is the broad gate because it currently runs:

- `cargo fmt --check`.
- `cargo test --all-targets`.
- `cargo clippy --all-targets -- -D warnings`.
- Lua trace-exporter self-test.
- Python reference-trace and coverage-tool tests.
- Local Rust-current trace fixture comparison when
  `docs/fidelity/fixtures/local/rust-current` exists.
- Coverage report generation plus the new-Rust coverage check.

Use focused tests first while moving small slices, then run the full gate before
committing each dev-cycle.

Important focused filters for the first refactor slices:

- `cargo test public_arcade_api --all-targets`
- `cargo test snapshot_restore --all-targets`
- `cargo test save_state_restore --all-targets`
- `cargo test machine_process::tests --all-targets`
- `cargo test machine_scheduler::tests --all-targets`
- `cargo test process_scheduler --all-targets`
- `cargo test trace_text_ --all-targets`
- `cargo test native_video_fixture --all-targets`
- `cargo test sound_ --all-targets`
- `cargo test live_ --all-targets`
- `cargo test session_ --all-targets`
- `cargo test object_collision_dispatch --all-targets`
- `cargo test switch_scan_records_no_process_swtab_entries_without_queueing --all-targets`

Ignored local-reference tests remain known fidelity gaps, not refactor
permission to change behavior. Keep their ignored reasons current.

## Public API Contract

The completion-phase arcade-core API is `ArcadeMachine` in `src/machine.rs`.
Post-completion refactor work may wrap or move this type later, but these
behaviors are frozen:

- `ArcadeMachine::new()` creates the deterministic initialized red-label core
  used by normal tests and live mode.
- `ArcadeMachine::try_new_with_cmos(cmos)` creates the same core with a supplied
  CMOS image and refreshes high-score projection from that image.
- `ArcadeMachine::new_cold_boot_trace()` creates the cold-boot trace core and
  keeps the MAME-observed RAM-fill scheduler active.
- `ArcadeMachine::reset()` returns an existing machine to the same observable
  state as `ArcadeMachine::new()`, including compatibility flags, board/sound
  snapshots, red-label RAM, CMOS, palette RAM, and trace-facing replay output.
- `ArcadeMachine::step(input)` advances exactly one red-label frame and returns
  a `FrameOutput` containing the post-frame machine snapshot, native video
  frame, sound commands, events, main-board snapshot, and sound-board snapshot.
- `ArcadeMachine::step_with_typed_chars(input, typed_chars)` is the frame API
  for live high-score initials entry and must remain equivalent to `step` when
  `typed_chars` is empty.
- `ArcadeMachine::snapshot()` projects public state from source-owned
  red-label RAM/CMOS whenever those tables are initialized.
- `ArcadeMachine::restore(snapshot)` restores the public snapshot and writes
  the corresponding observable red-label RAM fields for scores, credits,
  current player, player runtime, facing, wave/lives/smart bombs, and RNG.
- `ArcadeMachine::save_state()` / `restore_state(state)` round-trip the full
  refactor-sensitive state: public snapshot, red-label RAM, CMOS, palette RAM,
  hardware map, board input/watchdog/video-counter surfaces, sound latch
  surface, and trace scheduler state.

Do not widen this API just to make a move easier. New public entry points should
have a source-backed use, a focused test, and documentation here.

## Module Boundaries

Current accepted boundaries on `red-label-refactor`:

- `src/machine_state.rs` owns data-only arcade-core state and frame-output
  contracts: public snapshots, player/score state, high-score entry state,
  trace state, compatibility flags, machine events, and `FrameOutput`.
- `src/machine_process.rs` owns scheduler data contracts:
  `RedLabelCpuRegisters` and `RedLabelScheduledProcess`.
- `src/machine_scheduler.rs` owns source scheduler routing helpers:
  source-entry register overrides, live-prioritized process routine sets, and
  the scheduler-focused tests that keep routine-address dispatch stable.
- `src/machine_memory.rs` owns the source-visible `RedLabelRuntimeMemory`
  implementation: RAM/CMOS/palette/hardware-map mutation helpers, list
  primitives, translated source routine bodies, trace helper mutations,
  save/restore-sensitive memory surfaces, and fixture-backed byte helpers.
- `src/machine_session.rs` owns the public `ArcadeMachine` API and live/session
  orchestration: construction, reset, snapshot/restore, save/restore,
  frame stepping, board/sound snapshots, coin/start/high-score flow, live
  attract/gameplay dispatch, and compatibility-overlay handling.
- `src/machine_sound.rs` owns red-label sound command contracts and fixture
  helpers: direct/table/thrust command TSV generation, timeline expansion,
  source `SNDOUT` modeling, command priority, and sound sequence structs.
- `src/machine_video.rs` owns reusable video helper primitives: laser address
  stepping, star-table generation/output helpers, terrain bit-state
  advancement, and terrain pointer validation.
- `src/machine_player.rs` owns reusable player/projectile/object helper
  primitives: switch-derived vertical action, player scroll clamping,
  signed-word helpers, object band checks, tie vertical deltas, and swarmer
  damping.
- `src/machine_world.rs` owns small wave/world helper primitives: `GETWV`
  restore-wave checks, inter-wall delta iteration counts, and BCD display
  helpers shared by session/runtime code.
- `src/machine.rs` re-exports public contracts to preserve existing
  `machine::...` import paths while retaining source-derived type definitions
  that have not yet justified a separate public module.

Continuing ownership rules:

- CPU and board: `src/board.rs`, `src/pia.rs`, `src/rom.rs`, and the
  board-facing parts of `src/machine.rs`.
- Scheduler and process dispatch: keep source-shaped `MKPROC`, `MSPROC`,
  `SLEEP`, `KILL`, `DISP`, executive iteration, switch-process dispatch, and
  translated process-body routing byte-compatible inside `machine_memory`
  unless a later narrow slice proves a smaller extraction.
- Memory and source assets: `src/red_label_memory.rs`, source-owned runtime RAM
  helpers in `src/machine_memory.rs`, and embedded TSVs under
  `assets/red-label/`.
- Video: `src/video.rs`, native video-frame construction and renderer-only
  scaling in `src/live.rs`, `src/kitty.rs`, and `src/wgpu_presenter.rs`, plus
  source helper primitives in `src/machine_video.rs`.
- Sound: `src/sound.rs`, red-label sound command fixtures in
  `src/machine_sound.rs`, sound-board snapshots, and main-board command-latch
  plumbing in `src/machine_session.rs`.
- Input and session: `src/input.rs`, live input mapping in `src/live.rs`,
  high-score/session flow in `src/machine_session.rs`, and CMOS persistence
  in `src/cmos_storage.rs`.
- Compatibility: `CompatibilityState`, `xyzzy` hook behavior, Planetoid input
  mapping, and any future overlay code. Compatibility must stay outside the
  red-label trace contract unless paired disabled/enabled tests prove the
  difference.
- Assets and tooling: `src/assets.rs`, parser modules, `tools/`, and
  `docs/fidelity/`. Runtime code must continue to embed source-derived assets
  and must not depend on local ROMs, MAME, generated fixtures, or archived
  prototype assets.

Move code in these ownership slices. Avoid mixed refactors that touch scheduler,
video, sound, and session behavior in the same commit.

## Byte-Compatible Surfaces

The refactor must preserve these observable surfaces:

- Trace TSV schema and row values for all currently passing Rust-current
  fixtures.
- `FrameOutput` snapshots, events, sound commands, native video frame, and
  main-board/sound-board snapshots.
- Public `snapshot` and `restore` behavior, plus full `save_state` /
  `restore_state` replay.
- Source-owned main RAM, CMOS, palette RAM, hardware-map, input-port,
  watchdog, video-counter, sound-latch, and trace scheduler mutations.
- Process, super-process, object, inactive-object, active-object, shell-list,
  switch-history, switch-queue, and player table bytes.
- Native visible pixel-nibble CRCs and source-native video fixture signatures.
- Red-label sound command sequences, source-visible DAC buffers, and sound-board
  latch/IRQ state.
- Session and operator-visible behavior already covered by tests: credits,
  starts, two-player state, high-score entry, CMOS defaults/persistence,
  diagnostics/audits, service inputs, and reset paths.
- Compatibility overlays with `xyzzy` disabled. Enabled overlays may differ
  only through documented hook tests.

If a moved slice cannot preserve one of these surfaces, stop and add the missing
fixture or gap entry before changing behavior.
