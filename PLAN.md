# Defender Completion Plan

Last reviewed: `2026-05-04 16:48:21 BST`

## Purpose

This plan turns `SPEC.md` into an executable sequence of development cycles.
`SPEC.md` remains the source of truth for behavior, exactness rules, and known
fidelity gaps. This file owns work ordering, step status, completion notes, and
the communication protocol for each completed step.

## Execution Protocol

- Before starting any step, add a timestamped note to that cycle's work log
  naming the exact `PLAN.md` step, for example `DC-03.2`.
- After finishing a step, mark the checkbox complete, add a completion timestamp
  next to that step, and add a short work-log note describing what changed.
- After finishing a step, post the same breakdown to
  `xyzzytools.slack.com#codex`. The Slack message must mention the `PLAN.md`
  cycle and step number, changed files, tests run, and any skipped validation.
- If Slack is unavailable, add a `Slack pending` note under the step and post it
  as soon as the integration is available.
- Each completed dev-cycle must result in a push to the `red-label` branch.
  When a step is delivered independently, commit and push that step before
  starting the next independent step. Commit messages must use Conventional
  Commits format and reference the completed dev-cycle or step, for example
  `test: add mutation snapshots for DC-02.1`.
- New code must include focused unit tests. Coverage does not need to reach
  100%, but new behavior must be covered at the routine boundary or public API
  boundary.
- For later refactor safety, tests must assert source-visible mutations, not
  only returned values. RAM, CMOS, video RAM, palette RAM, process-list,
  object-list, shell-list, sound-command, scheduler, and snapshot mutations
  should be checked with before/after assertions or golden traces.
- Keep `make fidelity` as the broad local gate when practical. If only a
  narrower command is run, record why in the work log.
- Do not check generated ROM payloads or MAME-derived golden traces into the
  repository. Keep local generated fixtures under ignored fixture paths.

Status values:

- `planned`: not started.
- `in_progress`: started and work-log note exists.
- `blocked`: waiting on a dependency that is recorded in the work log.
- `complete`: step checkbox is marked, timestamped, and Slack update was sent or
  explicitly marked pending.

## Current Baseline

- The project is a Rust clean-slate red-label Defender translation with a large
  table-backed core in `src/machine.rs`.
- The repository already has extensive unit tests, trace tooling, ROM metadata,
  MAME memory/input metadata, and local MAME reference trace generation.
- `SPEC.md` now records the 2026-05-04 review findings and the refactor-safety
  testing rule.
- The major gaps are still source-exact hardware/frame scheduling, full
  frame/cycle integration, golden-trace proof for translated gameplay/session
  paths, pixel/audio golden fixtures, cycle-scheduled sound-board execution,
  and the later large refactor.

## Phase 0: Documentation And Guardrails

### DC-00: Repo Review, Spec Validation, And Plan Creation

Status: `complete`

Goal: establish a valid current baseline and create this execution plan.

Steps:

- [x] DC-00.1 Review agent rules, repository structure, current docs, build
  metadata, source modules, assets, tests, and known gap markers.
  Completed: `2026-05-04 16:48:21 BST`
- [x] DC-00.2 Validate the key source-of-truth links used by `SPEC.md`.
  Completed: `2026-05-04 16:48:21 BST`
- [x] DC-00.3 Update `SPEC.md` with review findings, stale-plan correction, and
  refactor-safety testing requirements.
  Completed: `2026-05-04 16:48:21 BST`
- [x] DC-00.4 Create `PLAN.md` with phased dev cycles and the required
  per-step update protocol.
  Completed: `2026-05-04 16:48:21 BST`

Work log:

- `2026-05-04 16:48:21 BST` Started and completed `DC-00`: reviewed the repo,
  confirmed the spec is broadly aligned, found a stale Mutant-score future task,
  added the review addendum to `SPEC.md`, and created this plan.
  Slack update: `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777909986049479`

### DC-01: Baseline Validation Gate

Status: `complete`

Goal: prove the current tree is green before feature work continues.

Steps:

- [x] DC-01.1 Run Markdown validation across edited docs and fix lint issues.
  Completed: `2026-05-04 16:52:12 BST`
- [x] DC-01.2 Run `cargo fmt --check`.
  Completed: `2026-05-04 16:52:12 BST`
- [x] DC-01.3 Run `cargo test --all-targets`.
  Completed: `2026-05-04 16:52:12 BST`
- [x] DC-01.4 Run `cargo clippy --all-targets -- -D warnings`.
  Completed: `2026-05-04 16:52:12 BST`
- [x] DC-01.5 Run `make coverage` or document missing local coverage tooling.
  Completed: `2026-05-04 16:52:12 BST`
- [x] DC-01.6 Run `make trace-fixtures`; if local fixtures are absent, record
  the skip behavior and the fixture path.
  Completed: `2026-05-04 16:52:12 BST`
- [x] DC-01.7 Update this work log and Slack with the exact baseline result.
  Completed: `2026-05-04 16:52:12 BST`

Completion gate: baseline command results are recorded in this file and any
newly discovered repo gaps are copied into `SPEC.md`.

Work log:

- `2026-05-04 16:52:12 BST` Started and completed `DC-01`: validated edited
  docs with `markdownlint SPEC.md PLAN.md`; ran `cargo fmt --check`;
  ran `cargo test --all-targets` with 540 passed and 5 known ignored fidelity
  tests; ran `cargo clippy --all-targets -- -D warnings`; ran
  `make trace-fixtures`, which matched 12 local fixtures and 9,600 frames; and
  ran `make coverage`, which passed the configured 80% line coverage gate and
  wrote `target/coverage/coverage.xml`.
  Slack update: `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777909986049479`

### DC-02: Refactor-Safety Test Harness

Status: `in_progress`

Goal: make future translation and later refactor work easy to verify by
capturing source-visible mutations deliberately.

Steps:

- [x] DC-02.1 Add shared test helpers for byte-range before/after assertions
  over red-label RAM, CMOS, object cells, process cells, shell lists, and video
  RAM.
  Completed: `2026-05-04 17:05:10 BST`
- [x] DC-02.2 Add characterization tests around the highest-risk translated
  routines that currently rely on many scattered byte mutations.
  Completed: `2026-05-04 17:19:14 BST`
- [ ] DC-02.3 Add snapshot/restore regression tests that prove restored state
  replays the same trace rows and mutations.
- [ ] DC-02.4 Extend trace columns only where a source routine needs more
  observable state for equivalence.
- [ ] DC-02.5 Document the characterization-test pattern in `docs/fidelity/`.

Completion gate: new mutation helpers exist, representative high-risk routines
use them, and `make fidelity` remains green or has a recorded external-tool
blocker.

Work log:

- `2026-05-04 16:58:21 BST` Started `DC-02.1`: adding shared
  mutation-assertion test helpers for red-label RAM, CMOS, object/process cells,
  shell-list state, and video RAM so future translations and the later refactor
  can prove byte-level behavior preservation.
- `2026-05-04 17:05:10 BST` Completed `DC-02.1`: added test-only
  byte-range snapshot helpers for red-label RAM, CMOS, visible video RAM,
  object cells, process cells, super-process cells, and the SPTR shell-list
  head. Added helper tests that prove unchanged and changed assertions against
  process allocation, object allocation, video writes, CMOS replacement, and
  super-process allocation. Validation passed with `cargo fmt --check`,
  `cargo test test_support --all-targets`, `cargo test --all-targets`,
  `cargo clippy --all-targets -- -D warnings`, `make trace-fixtures`, and
  `make coverage`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777910762207089`
- `2026-05-04 17:13:16 BST` Rebased `DC-02.1` onto the latest
  `origin/red-label` before push. Conflict resolution kept the newer completed
  `SPEC.md` items from the remote branch and narrowed the review addendum to
  the remaining fixture, scheduling, sound, and refactor-safety gaps. Re-ran
  validation on the rebased branch: `markdownlint PLAN.md SPEC.md`,
  `git diff --check HEAD~1..HEAD`, `cargo fmt --check`,
  `cargo test --all-targets` with 792 passed and 5 known ignored fidelity
  tests, `cargo clippy --all-targets -- -D warnings`, `make trace-fixtures`
  which skipped because `docs/fidelity/fixtures/local/rust-current` is absent
  locally, and `make coverage` which passed the configured coverage gate.
- `2026-05-04 17:13:57 BST` Started `DC-02.2`: adding characterization tests
  around translated routines with broad byte-level side effects, using the
  `DC-02.1` mutation snapshot helpers to lock down source-visible RAM, CMOS,
  process-list, object-list, and video mutations before later refactors.
- `2026-05-04 17:19:14 BST` Completed `DC-02.2`: added mutation-preserving
  characterization tests for the live credited-start path, `PLSTR3` player
  runtime initialization, and `HALL6` high-score submission. These tests assert
  exact RAM, CMOS, process-cell, process-list, and video-RAM byte mutations
  through the `DC-02.1` snapshot helpers. Validation passed with
  `markdownlint PLAN.md SPEC.md`, `git diff --check`, `cargo fmt --check`,
  `cargo test characterization_ --all-targets`, `cargo test --all-targets`
  with 795 passed and 5 known ignored fidelity tests,
  `cargo clippy --all-targets -- -D warnings`, `make trace-fixtures` which
  skipped because `docs/fidelity/fixtures/local/rust-current` is absent locally,
  and `make coverage`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777911583450689`

## Phase 1: Reference Fixtures And Golden Equivalence

### DC-03: Local Reference Fixture Environment

Status: `planned`

Goal: make local MAME/source fixtures reliable enough to gate every translated
subsystem.

Steps:

- [ ] DC-03.1 Verify `make reference-inputs` writes every scenario from
  `assets/red-label/trace-scenarios.tsv`.
- [ ] DC-03.2 Verify `make reference-traces` with the local MAME and ROM path,
  or document the missing local dependency.
- [ ] DC-03.3 Verify `make reference-fixtures-check` catches missing headers,
  missing scenarios, and frame-count drift.
- [ ] DC-03.4 Add any missing scenario required before the next subsystem is
  translated.
- [ ] DC-03.5 Record fixture generation date, MAME version, ROM path policy, and
  validation result in `docs/fidelity/`.

Completion gate: local reference generation and validation are repeatable, or
all missing external dependencies are documented.

### DC-04: Golden Tests For Existing Translated Slices

Status: `planned`

Goal: move existing translated routines from unit-only confidence toward
red-label equivalence.

Steps:

- [ ] DC-04.1 Compare boot/cold-start trace columns against local golden traces.
- [ ] DC-04.2 Compare start, fire, thrust/reverse, smart bomb, hyperspace, death,
  and wave-clear slices against focused traces.
- [ ] DC-04.3 Add ignored or failing fidelity tests for every still-unknown
  trace mismatch before implementing a fix.
- [ ] DC-04.4 Keep exact TSV comparison as the first gate, then add narrower
  mutation tests for the routine causing each mismatch.

Completion gate: translated slices either match local golden fixtures or have
explicit ignored/failing tests and `SPEC.md` gap entries.

## Phase 2: Hardware, Scheduler, And Frame Execution

### DC-05: Source-Exact Boot And Hardware Step

Status: `planned`

Goal: replace trace-only boot scaffolding with source-shaped reset, ROM-test,
RAM-test, CMOS, and start-ready execution.

Steps:

- [ ] DC-05.1 Model exact power-on RAM, reset path, ROM/RAM/CMOS diagnostic
  progression, and start-ready state from red-label source or MAME traces.
- [ ] DC-05.2 Wire the main-board RAM, CMOS, PIA, palette, watchdog, and video
  counter surfaces into `ArcadeMachine::step`.
- [ ] DC-05.3 Wire the sound-command latch and sound-board PIA boundary into the
  frame output path without high-level cue shortcuts.
- [ ] DC-05.4 Add mutation tests for every board and machine byte range touched.
- [ ] DC-05.5 Prove boot/start-ready trace equivalence with local fixtures.

Completion gate: boot/start-ready state is source-shaped and no longer depends
on trace-only scheduling shortcuts.

### DC-06: Process Scheduler Completion

Status: `planned`

Goal: make red-label process execution the normal core path.

Steps:

- [ ] DC-06.1 Add register-aware process entry/resume state where source
  routines require A/B/X/Y/U/S/CC context.
- [ ] DC-06.2 Complete `PLRES` swarmer respawn once the entry B register value
  is trace-proven.
- [ ] DC-06.3 Finish suicide, resume, delay, super-process, and generic body
  dispatch semantics.
- [ ] DC-06.4 Verify object/process order against golden traces.
- [ ] DC-06.5 Remove scaffold state that duplicates exact red-label table
  fields.

Completion gate: translated process dispatch drives gameplay state by default,
with explicit gaps only for unimplemented routine bodies.

### DC-07: IRQ, Scanline, And Palette Integration

Status: `planned`

Goal: integrate frame timing, scanline ownership, palette copy, and hardware map
restoration into the core.

Steps:

- [ ] DC-07.1 Implement source-exact CPU IRQ cadence and screen scanline
  scheduling from MAME/source measurements.
- [ ] DC-07.2 Finish `BGOUT` live stack-context wiring and hardware-map
  restoration.
- [ ] DC-07.3 Finish palette-copy side effects and validate palette RAM
  mutations.
- [ ] DC-07.4 Emit native video CRCs from actual red-label video RAM in trace
  output.
- [ ] DC-07.5 Add focused pixel/frame fixtures for scanline-sensitive slices.

Completion gate: one full core frame mutates RAM, video RAM, palette RAM,
object lists, process lists, and sound commands in source order.

## Phase 3: Player, Session, And Cabinet Flow

### DC-08: Player Control And Respawn Completion

Status: `planned`

Goal: finish the exact player path around controls, death, respawn, and carried
humans.

Steps:

- [ ] DC-08.1 Finish reverse integration with player movement and rendering.
- [ ] DC-08.2 Prove laser cap, laser lifetime, collision setup, and fizzle/erase
  timing against traces.
- [ ] DC-08.3 Finish smart-bomb world integration, including what is and is not
  destroyed.
- [ ] DC-08.4 Finish hyperspace failure/safe paths and rematerialization
  movement.
- [ ] DC-08.5 Finish carried-human, falling-human, catch, landing, and rescue
  paths.
- [ ] DC-08.6 Prove death, respawn, wave-clear, and game-over branches with
  golden traces.

Completion gate: the player path can be exercised from live inputs through
source-shaped process execution and trace-proven state mutations.

### DC-09: Cabinet Session And Operator Flow

Status: `planned`

Goal: complete exact credits, starts, high scores, two-player alternation, and
operator behavior.

Steps:

- [ ] DC-09.1 Finish one-player and two-player session flow, including
  alternating turns and current-player table swaps.
- [ ] DC-09.2 Finish credit, coinage, free-play, replay, and bonus-stock rules.
- [ ] DC-09.3 Translate exact initials-entry UI, high-score screens, and
  game-over-to-attract timing.
- [ ] DC-09.4 Decide and document the default live CMOS persistence policy.
- [ ] DC-09.5 Complete diagnostics, audit, and adjustment flow in live cabinet
  mode.
- [ ] DC-09.6 Add regression tests for P1/P2 score, lives, smart bombs, credits,
  high-score insertion, and CMOS mutations.

Completion gate: cabinet session behavior matches red-label traces and
operator-facing documentation is current.

## Phase 4: World, Waves, Humans, And Enemies

### DC-10: Wave And World State

Status: `planned`

Goal: make wave setup and world state source-exact.

Steps:

- [ ] DC-10.1 Translate wave launch process order, enemy reserve allocation, and
  spawn timing from source.
- [ ] DC-10.2 Finish terrain scheduling, destroyed-planet terrain, scanner
  terrain, and fifth-wave human restoration.
- [ ] DC-10.3 Prove `GETWV`, `WDELT`, target restoration, and survivor bonus
  behavior against golden traces.
- [ ] DC-10.4 Add tests for wave-to-wave RAM mutation and object/process list
  changes.

Completion gate: wave setup and world transitions can be compared against local
golden traces without hand-placed opener state.

### DC-11: Remaining Enemy And Collision Behavior

Status: `planned`

Goal: finish all enemy, shell, mine, and collision routines.

Steps:

- [ ] DC-11.1 Finish Baiter spawn, pursuit, firing, and lifetime behavior.
- [ ] DC-11.2 Finish Bomber, Pod, minefield, and Pod/Swarmer burst behavior.
- [ ] DC-11.3 Finish remaining mutant behavior and firing checks.
- [ ] DC-11.4 Finish remaining shell rendering/output and object collision
  vectors.
- [ ] DC-11.5 Add score regression tests for enemies, bullets, mines, humans,
  rescue, Pods, Swarmers, extra stocks, and smart bombs.
- [ ] DC-11.6 Verify each enemy subsystem with focused traces before moving on.

Completion gate: all red-label enemy processes are either translated and tested
or explicitly recorded as gaps with ignored/failing tests.

## Phase 5: Native Video And Presentation

### DC-12: Verified Cabinet Video

Status: `planned`

Goal: replace the live scaffold renderer with verified native cabinet frames.

Steps:

- [ ] DC-12.1 Drive native visible frames from red-label video RAM and palette
  RAM after scanline scheduling is trace-proven.
- [ ] DC-12.2 Implement HUD, score fields, lives, smart-bomb icons, scanner,
  text, game-over, high-score, initials, and attract screens from source.
- [ ] DC-12.3 Add pixel checksum and perceptual diff fixtures for boot, attract,
  start, gameplay, death, high-score, and operator frames.
- [ ] DC-12.4 Switch live mode from `render_scaffold` to `render_cabinet_frame`
  only after fixture proof.
- [ ] DC-12.5 Remove or regenerate archived prototype visual assets from
  red-label source/ROM-derived data.

Completion gate: verified native frames are the only live cabinet image source,
and temporary visuals are outside the verified frame.

## Phase 6: Sound Board And Audio

### DC-13: Source-Shaped Sound Board

Status: `planned`

Goal: make audio come from source-shaped sound-board execution rather than
semantic gameplay cues.

Steps:

- [ ] DC-13.1 Translate the `VSNDRM1.SRC` IRQ dispatch path and Defender sound
  routines needed by the main board.
- [ ] DC-13.2 Model sound CPU timing, command latch consumption, DAC writes, and
  sample generation.
- [ ] DC-13.3 Add command-sequence fixtures from red-label traces.
- [ ] DC-13.4 Add waveform tests with deterministic tolerance.
- [ ] DC-13.5 Keep `--mute` as an output-layer mixer switch only.

Completion gate: sound command traces and generated audio pass fixtures, and no
gameplay code triggers high-level named cues directly.

## Phase 7: Compatibility Features

### DC-14: `xyzzy` And Input Profiles

Status: `planned`

Goal: finish supported non-arcade compatibility features without polluting the
arcade core.

Steps:

- [ ] DC-14.1 Keep Planetoid and cabinet mappings in input-profile code only.
- [ ] DC-14.2 Add every future `xyzzy` hook as an explicit overlay hook.
- [ ] DC-14.3 For each hook, add paired tests proving arcade behavior is
  unchanged when `xyzzy` is disabled and compatibility behavior applies when it
  is enabled.
- [ ] DC-14.4 Add trace checks that `xyzzy` disabled remains red-label
  equivalent.
- [ ] DC-14.5 Document compatibility behavior without presenting it as arcade
  behavior.

Completion gate: compatibility behavior is isolated, documented, and covered by
paired tests.

## Phase 8: Planned Large Refactor

### DC-15: Refactor Freeze And API Shape

Status: `planned`

Goal: prepare for the large refactor only after behavior is well characterized.

Steps:

- [ ] DC-15.1 Freeze a full characterization suite covering traces, mutation
  tests, pixel fixtures, audio fixtures, and live/session edge cases.
- [ ] DC-15.2 Define the public arcade API: `new`, `reset`, `step`,
  `snapshot`, and `restore`.
- [ ] DC-15.3 Identify module boundaries for CPU/board, scheduler, memory,
  video, sound, input/session, compatibility, and assets.
- [ ] DC-15.4 Document every behavior that must remain byte-for-byte compatible
  during the refactor.

Completion gate: the refactor has a test-backed API contract and no major
untested mutation surfaces.

### DC-16: Module Split And Behavior Preservation

Status: `planned`

Goal: split the large core safely without changing behavior.

Steps:

- [ ] DC-16.1 Move code in small ownership-based slices with no unrelated
  rewrites.
- [ ] DC-16.2 Run the full characterization gate after every slice.
- [ ] DC-16.3 Keep public surfaces narrow and source-cited.
- [ ] DC-16.4 Remove dead scaffold paths only after tests prove exact
  replacements.
- [ ] DC-16.5 Update docs after each module boundary becomes stable.

Completion gate: the refactored code produces the same traces, byte mutations,
pixel fixtures, audio fixtures, and live/session behavior as the pre-refactor
implementation.

## Phase 9: Release Readiness

### DC-17: Final Acceptance And Documentation

Status: `planned`

Goal: prepare the project as a complete exact red-label implementation with
supported compatibility overlays.

Steps:

- [ ] DC-17.1 Run `make fidelity`, local reference fixtures, pixel fixtures,
  audio fixtures, and any CI/Sonar gates.
- [ ] DC-17.2 Perform live terminal smoke tests in Kitty-compatible terminals
  and record rendering, input, sound, and quit behavior.
- [ ] DC-17.3 Update `README.md`, `SPEC.md`, `docs/fidelity/`, and install/run
  documentation to match final behavior.
- [ ] DC-17.4 Confirm no runtime dependency on local ROMs or generated fixture
  files.
- [ ] DC-17.5 Confirm `xyzzy` disabled is red-label equivalent and `xyzzy`
  enabled differs only through documented overlay hooks.
- [ ] DC-17.6 Close remaining gap entries or explicitly mark them out of scope.

Completion gate: every acceptance item in `SPEC.md` is satisfied or explicitly
documented as out of scope by project owner decision.
