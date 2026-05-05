# Defender Completion Plan

Last reviewed: `2026-05-05 00:12:36 BST`

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

### Fidelity Gap Review 2026-05-05

This review checked `SPEC.md`, `docs/fidelity/gaps.md`,
`docs/fidelity/golden-comparison-results.md`, the ignored fidelity tests in
`src/fidelity.rs` and `src/app.rs`, and the current completion plan.
Slack report:
`https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777936475910379`

Gap inventory:

- Trace oracle fixtures: local MAME reference fixtures exist as ignored local
  artifacts, but `docs/fidelity/fixtures/local/rust-current` is absent in the
  current local tree. Exact local-reference tests are still ignored until each
  scenario is made source-equivalent.
- `attract_boot`: Rust matches the local reference through frame 732, then
  diverges at frame 733 only in `process_table_crc32`. The remaining blocker is
  post-`INIT20` `ATTR` / executive process-table cadence.
- Focused gameplay traces: `start_game`, `firing`, `thrust_reverse`,
  `smart_bomb`, `hyperspace`, `death`, and `wave_advance` still fail exact
  local-reference comparison. The recurring gaps are credited-start transition
  timing, RNG call order, and post-start object/process scheduler execution.
- CPU, IRQ, and frame ownership: exact main CPU scheduling, IRQ timing,
  A/X/Y/S/CC/B register context, scanline/video counter ownership, watchdog
  timing/reset side effects, palette/rendering timing side effects, and sound
  IRQ ownership are not yet fully source-exact.
- Hardware edge cases: exact Williams power-on RAM contents, physical advance
  switch timing, physical lamp timing, decoder PROM behavior, full ROM/bank
  memory integration, generated derived assets, and remaining collision or
  no-process `SWTAB` effects still need source-backed closure.
- Video: native video rendering exists, but MAME-derived pixel golden fixtures
  are absent. A `2026-05-05` live title-screen capture shows the `DEFENDER`
  wordmark/title graphic corrupted into large red/purple blocky bands.
- Live attract flow: the app was also reported not to advance beyond the
  initial Williams/`DEFENDER` screen. This is separate from the corrupted
  title graphic and needs scheduler/presentation proof that live `ATTR` /
  `AMODES` / `LOGO` / `DEFEND` / `LEDRET` cadence progresses into later
  attract, credit, and start-ready states.
- Remaining untranslated screens still render native black, and remaining HUD,
  general text, boot/game-over presentation, and non-IRQ scanline ownership
  need proof or owner-approved scope decisions.
- Audio: source-shaped sound tables, command flow, and in-repo deterministic
  waveform signatures exist, but live MAME command-sequence fixtures, external
  waveform golden fixtures, cycle-accurate DAC scheduling, exact sound-board
  CPU IRQ cadence, and remaining waveform routines are not complete.
- Session and operator flows: two-player, high-score, operator/service,
  cabinet profile, and longer session paths need end-to-end local MAME golden
  traces before final release.
- Compatibility overlays: `DC-14` closed current Planetoid and `xyzzy`
  isolation, but any future `xyzzy` hook still requires paired disabled/enabled
  tests and red-label trace proof.

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

Status: `complete`

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
- [x] DC-02.3 Add snapshot/restore regression tests that prove restored state
  replays the same trace rows and mutations.
  Completed: `2026-05-04 17:23:46 BST`
- [x] DC-02.4 Extend trace columns only where a source routine needs more
  observable state for equivalence.
  Completed: `2026-05-04 17:37:09 BST`
- [x] DC-02.5 Document the characterization-test pattern in `docs/fidelity/`.
  Completed: `2026-05-04 17:39:52 BST`

Completion gate: new mutation helpers exist, representative high-risk routines
use them, and `make fidelity` remains green or has a recorded external-tool
blocker.

Work log:

- `2026-05-04 17:38:58 BST` Started `DC-02.5`: documenting the
  characterization-test pattern in `docs/fidelity/` so future translation and
  refactor work uses the same before/after mutation and trace-replay approach.
- `2026-05-04 17:39:52 BST` Completed `DC-02.5`: added
  `docs/fidelity/characterization-tests.md` with the required
  characterization-test shape, shared snapshot helpers, trace/replay guidance,
  and later-refactor safety rules. Linked the pattern from
  `docs/fidelity/README.md`. Validation passed with `markdownlint PLAN.md
  SPEC.md README.md docs/fidelity/README.md docs/fidelity/gaps.md
  docs/fidelity/characterization-tests.md assets/red-label/README.md` and
  `git diff --check`. No cargo tests were run for this docs-only step.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777912813670469`
- `2026-05-04 17:26:42 BST` Started `DC-02.4`: reviewing the current trace
  schema/output and recent characterization tests to add new trace columns only
  if a translated source routine needs additional observable state for
  equivalence.
- `2026-05-04 17:37:09 BST` Completed `DC-02.4`: extended the trace schema
  with `process_table_crc32` and `super_process_table_crc32`, backed those
  columns from source-owned red-label process RAM, updated the MAME reference
  exporter/debug stream, refreshed fixture/schema docs and `SPEC.md`, and added
  tests for trace column stability, process-table CRC helpers, and schema column
  counts. Validation passed with `markdownlint PLAN.md SPEC.md README.md
  docs/fidelity/README.md docs/fidelity/gaps.md assets/red-label/README.md`,
  `cargo test trace_ --all-targets`, `make trace-script-test`,
  `cargo test trace_crc_helpers_sample_source_process_tables --all-targets`,
  `git diff --check`, and `make fidelity`. `make fidelity` ran
  `cargo fmt --check`, `cargo test --all-targets` with 798 passed and 5 known
  ignored fidelity tests, `cargo clippy --all-targets -- -D warnings`, trace
  script tests, `make trace-fixtures` which skipped because
  `docs/fidelity/fixtures/local/rust-current` is absent locally, and
  `make coverage` with 122/122 added executable Rust lines covered.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777912688322649`
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
- `2026-05-04 17:20:19 BST` Started `DC-02.3`: adding snapshot/restore
  regression tests that prove restored machine state replays the same trace rows
  and source-visible byte mutations.
- `2026-05-04 17:23:46 BST` Completed `DC-02.3`: added replay regression
  tests for public `snapshot`/`restore` through live attract process creation
  and full `save_state`/`restore_state` through the cold-boot trace RAM-fill
  scheduler. Both tests compare replayed TSV trace rows and exact source-visible
  RAM/process byte mutations. Validation passed with
  `markdownlint PLAN.md SPEC.md`, `git diff --check`, `cargo fmt --check`,
  `cargo test restore_replays --all-targets`, `cargo test --all-targets` with
  797 passed and 5 known ignored fidelity tests,
  `cargo clippy --all-targets -- -D warnings`, `make trace-fixtures` which
  skipped because `docs/fidelity/fixtures/local/rust-current` is absent locally,
  and `make coverage`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777911855566369`

## Phase 1: Reference Fixtures And Golden Equivalence

### DC-03: Local Reference Fixture Environment

Status: `complete`

Goal: make local MAME/source fixtures reliable enough to gate every translated
subsystem.

Steps:

- [x] DC-03.1 Verify `make reference-inputs` writes every scenario from
  `assets/red-label/trace-scenarios.tsv`.
  Completed: `2026-05-04 17:41:35 BST`
- [x] DC-03.2 Verify `make reference-traces` with the local MAME and ROM path,
  or document the missing local dependency.
  Completed: `2026-05-04 17:43:34 BST`
- [x] DC-03.3 Verify `make reference-fixtures-check` catches missing headers,
  missing scenarios, and frame-count drift.
  Completed: `2026-05-04 17:45:36 BST`
- [x] DC-03.4 Add any missing scenario required before the next subsystem is
  translated.
  Completed: `2026-05-04 17:47:12 BST`
- [x] DC-03.5 Record fixture generation date, MAME version, ROM path policy, and
  validation result in `docs/fidelity/`.
  Completed: `2026-05-04 17:51:13 BST`

Completion gate: local reference generation and validation are repeatable, or
all missing external dependencies are documented.

Work log:

- `2026-05-04 17:40:56 BST` Started `DC-03.1`: verifying that
  `make reference-inputs` writes one expanded input script for every scenario in
  `assets/red-label/trace-scenarios.tsv` under the ignored local reference
  fixture tree.
- `2026-05-04 17:41:35 BST` Completed `DC-03.1`: ran
  `make reference-inputs`, which wrote 12 Phase 1 `*.inputs.txt` scripts to the
  ignored `docs/fidelity/fixtures/local/reference` tree. Verified the generated
  filenames exactly match `assets/red-label/trace-scenarios.tsv` with `comm`,
  confirmed the generated count is 12, listed scenarios with
  `cargo run --quiet -- --fidelity-list-scenarios`, and checked each generated
  script expands to the frame count declared in the manifest with `awk`. Git
  status shows only ignored local fixture output plus the tracked `PLAN.md`
  update.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777912920816689`
- `2026-05-04 17:42:33 BST` Started `DC-03.2`: verifying
  `make reference-traces` against the local MAME executable and red-label ROM
  path, or documenting the exact missing local dependency if trace generation
  cannot run here.
- `2026-05-04 17:43:34 BST` Completed `DC-03.2`: verified local MAME at
  `/opt/homebrew/bin/mame` reports version `0.287 (unknown)` and
  `assets/roms/defender` contains the expected red-label ROM files. Ran
  `make reference-traces`, which invoked
  `tools/generate_reference_traces.py --mame "mame" --rom-dir "assets/roms"
  --out-dir "docs/fidelity/fixtures/local/reference"` and processed all 12
  trace scenarios. MAME printed repeated `-video none` / `-seconds_to_run`
  warnings, but the generator completed successfully. Verified that 12
  `*.expected.tsv` files now exist under the ignored local reference fixture
  tree, which is about 3.0 MiB, and git status shows only ignored local fixture
  output plus the tracked `PLAN.md` update.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777913041385469`
- `2026-05-04 17:44:32 BST` Started `DC-03.3`: verifying
  `make reference-fixtures-check` against the generated local reference
  fixtures and proving that the checker catches missing headers, missing
  scenarios, and frame-count drift.
- `2026-05-04 17:45:36 BST` Completed `DC-03.3`: ran
  `make reference-fixtures-check`, which validated 12 complete Phase 1
  reference fixtures and 22,308 frames from
  `docs/fidelity/fixtures/local/reference`. Ran
  `cargo test reference_trace_dir --all-targets` for the existing checker tests.
  Used temporary fixture copies to prove the CLI rejects a bad header, a missing
  `death.expected.tsv` scenario fixture, and a truncated `start_game` expected
  trace with 1,227 frames instead of the manifest-required 1,228. Git status
  shows only ignored local fixture output plus the tracked `PLAN.md` update.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777913165185189`
- `2026-05-04 17:46:40 BST` Started `DC-03.4`: reviewing the Phase 1 scenario
  manifest and requirement evidence against the next golden-equivalence work to
  decide whether a missing scenario must be added before implementation
  continues.
- `2026-05-04 17:47:12 BST` Completed `DC-03.4`: reviewed
  `assets/red-label/trace-scenarios.tsv`, `assets/red-label/trace-requirements.tsv`,
  and the `DC-04` golden-equivalence scope. The existing 12 scenarios cover the
  next boot/start, first 300 frames, firing, thrust/reverse, smart bomb,
  hyperspace, abduction, death, wave progression, planet destruction, and
  high-score-entry work, so no new scenario was required. Validation passed with
  `cargo run --quiet -- --fidelity-list-scenarios`,
  `cargo test trace_scenarios_cover_phase_one_required_cases --all-targets`,
  `cargo test trace_requirements_must_reference_known_scenarios --all-targets`,
  a manifest-name comparison using `comm`, and a scenario count check reporting
  12 entries.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777913256407929`
- `2026-05-04 17:49:46 BST` Started `DC-03.5`: recording the local
  reference fixture generation date, MAME version, ROM path policy, and
  validation result in `docs/fidelity/`.
- `2026-05-04 17:51:13 BST` Completed `DC-03.5`: added
  `docs/fidelity/local-reference-runs.md` and linked it from
  `docs/fidelity/README.md`. The run record documents the
  `2026-05-04 17:43:34 BST` local reference generation, MAME
  `/opt/homebrew/bin/mame` version `0.287 (unknown)`, the
  `assets/roms/defender` red-label ROM path, ignored local fixture policy, and
  validation showing 12 complete Phase 1 fixtures and 22,308 frames. Validation
  passed with `make reference-fixtures-check`,
  `markdownlint PLAN.md SPEC.md README.md docs/fidelity/README.md
  docs/fidelity/gaps.md docs/fidelity/characterization-tests.md
  docs/fidelity/local-reference-runs.md assets/red-label/README.md`, and
  `git diff --check`. No cargo test was run because the change is
  documentation-only.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777913513773309`

### DC-04: Golden Tests For Existing Translated Slices

Status: `complete`

Goal: move existing translated routines from unit-only confidence toward
red-label equivalence.

Steps:

- [x] DC-04.1 Compare boot/cold-start trace columns against local golden traces.
  Completed: `2026-05-04 17:55:34 BST`
- [x] DC-04.2 Compare start, fire, thrust/reverse, smart bomb, hyperspace, death,
  and wave-clear slices against focused traces.
  Completed: `2026-05-04 17:59:19 BST`
- [x] DC-04.3 Add ignored or failing fidelity tests for every still-unknown
  trace mismatch before implementing a fix.
  Completed: `2026-05-04 18:03:50 BST`
- [x] DC-04.4 Keep exact TSV comparison as the first gate, then add narrower
  mutation tests for the routine causing each mismatch.
  Completed: `2026-05-04 18:05:59 BST`

Completion gate: translated slices either match local golden fixtures or have
explicit ignored/failing tests and `SPEC.md` gap entries.

Work log:

- `2026-05-04 17:52:35 BST` Started `DC-04.1`: comparing boot/cold-start
  Rust trace output against the local MAME/source golden reference fixtures
  and recording any exact-match blocker before adding later subsystem tests.
- `2026-05-04 17:55:34 BST` Completed `DC-04.1`: ran exact
  `attract_boot` comparison against the local MAME/source reference fixture.
  The compare failed at line 2, frame 1, only in `process_table_crc32` and
  `super_process_table_crc32`; column analysis showed process CRC drift on 723
  frames and super-process CRC drift on 553 frames, while all other trace
  columns matched for the 900-frame scenario. Added
  `docs/fidelity/golden-comparison-results.md`, linked it from
  `docs/fidelity/README.md`, and updated `SPEC.md` plus
  `docs/fidelity/gaps.md` with the boot/start-ready process RAM scheduling gap.
  Validation passed with the expected-failing
  `cargo run --quiet -- --fidelity-check-trace
  docs/fidelity/fixtures/local/reference/attract_boot.inputs.txt
  docs/fidelity/fixtures/local/reference/attract_boot.expected.tsv`,
  `markdownlint PLAN.md SPEC.md README.md docs/fidelity/README.md
  docs/fidelity/gaps.md docs/fidelity/characterization-tests.md
  docs/fidelity/local-reference-runs.md
  docs/fidelity/golden-comparison-results.md assets/red-label/README.md`, and
  `git diff --check`. No cargo test was run because this step only records the
  comparison result and gap.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777913765944929`
- `2026-05-04 17:56:53 BST` Started `DC-04.2`: comparing start, fire,
  thrust/reverse, smart bomb, hyperspace, death, and wave-advance trace slices
  against the focused local MAME/source reference fixtures.
- `2026-05-04 17:59:19 BST` Completed `DC-04.2`: compared `start_game`,
  `firing`, `thrust_reverse`, `smart_bomb`, `hyperspace`, `death`, and
  `wave_advance` Rust traces against the local MAME/source references. All
  seven exact comparisons failed first at line 2, frame 1, on the inherited
  boot process/super-process CRC drift. Column summaries showed line counts and
  input-port columns stayed aligned, while phase, wave, lives, smart bombs, RNG
  bytes, object CRCs, and process CRCs diverged after the credited-start window.
  Added the focused-slice results to
  `docs/fidelity/golden-comparison-results.md` and updated `SPEC.md` plus
  `docs/fidelity/gaps.md` with the credited-start timing, RNG order, and
  post-start object/process scheduler gaps. Validation passed with
  expected-failing `--fidelity-check-trace` runs for the seven scenarios,
  `markdownlint PLAN.md SPEC.md README.md docs/fidelity/README.md
  docs/fidelity/gaps.md docs/fidelity/characterization-tests.md
  docs/fidelity/local-reference-runs.md
  docs/fidelity/golden-comparison-results.md assets/red-label/README.md`, and
  `git diff --check`. No cargo test was run because this step only records the
  comparison results and gaps.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777913989696059`
- `2026-05-04 18:00:45 BST` Started `DC-04.3`: adding ignored local
  fidelity tests for the known boot, focused gameplay, death, and wave-advance
  trace mismatches before changing scheduler or boot behavior.
- `2026-05-04 18:03:50 BST` Completed `DC-04.3`: added eight ignored
  `local_reference_*_matches_red_label` tests in `src/app.rs` for
  `attract_boot`, `start_game`, `firing`, `thrust_reverse`, `smart_bomb`,
  `hyperspace`, `death`, and `wave_advance`. The tests skip default validation,
  use the same exact TSV comparison path as `--fidelity-check-trace`, skip
  cleanly when local fixtures are absent, and fail when explicitly run against
  the current local references. Updated `docs/fidelity/README.md`,
  `docs/fidelity/golden-comparison-results.md`, `docs/fidelity/gaps.md`, and
  `SPEC.md` to document the ignored-test workflow. Validation passed with
  `cargo fmt --check`, `cargo test local_reference_ --all-targets`,
  expected-failing `cargo test local_reference_ --all-targets -- --ignored`,
  `cargo test --all-targets`, `cargo clippy --all-targets -- -D warnings`,
  `markdownlint PLAN.md SPEC.md README.md docs/fidelity/README.md
  docs/fidelity/gaps.md docs/fidelity/characterization-tests.md
  docs/fidelity/local-reference-runs.md
  docs/fidelity/golden-comparison-results.md assets/red-label/README.md`, and
  `git diff --check`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777914263794419`
- `2026-05-04 18:05:05 BST` Started `DC-04.4`: documenting the golden-fix
  workflow so exact TSV comparisons stay the first gate and each future
  mismatch fix adds a narrow source-visible mutation test at the responsible
  routine boundary.
- `2026-05-04 18:05:59 BST` Completed `DC-04.4`: added the exact-first
  golden-fix workflow to `docs/fidelity/characterization-tests.md`, linked it
  from `docs/fidelity/README.md`, and updated `SPEC.md`. The workflow requires
  future fixes to run exact TSV comparison first, record the first mismatch,
  add a narrow source-visible mutation test at the responsible routine
  boundary, rerun the exact TSV gate, and unignore local reference tests only
  once the fixture is stable. Validation passed with
  `markdownlint PLAN.md SPEC.md README.md docs/fidelity/README.md
  docs/fidelity/gaps.md docs/fidelity/characterization-tests.md
  docs/fidelity/local-reference-runs.md
  docs/fidelity/golden-comparison-results.md assets/red-label/README.md` and
  `git diff --check`. No cargo test was run because the change is
  documentation-only.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777914395566329`

## Phase 2: Hardware, Scheduler, And Frame Execution

### DC-05: Source-Exact Boot And Hardware Step

Status: `completed`

Goal: replace trace-only boot scaffolding with source-shaped reset, ROM-test,
RAM-test, CMOS, and start-ready execution.

Steps:

- [x] DC-05.1 Model exact power-on RAM, reset path, ROM/RAM/CMOS diagnostic
  progression, and start-ready state from red-label source or MAME traces.
  Completed: `2026-05-04 18:19:19 BST`
- [x] DC-05.2 Wire the main-board RAM, CMOS, PIA, palette, watchdog, and video
  counter surfaces into `ArcadeMachine::step`.
  Completed: `2026-05-04 18:28:29 BST`
- [x] DC-05.3 Wire the sound-command latch and sound-board PIA boundary into the
  frame output path without high-level cue shortcuts.
  Completed: `2026-05-04 18:35:07 BST`
- [x] DC-05.4 Add mutation tests for every board and machine byte range touched.
  Completed: `2026-05-04 18:44:31 BST`
- [x] DC-05.5 Prove boot/start-ready trace equivalence with local fixtures.
  Completed: `2026-05-04 19:05:30 BST`

Completion gate: boot/start-ready state is source-shaped and no longer depends
on trace-only scheduling shortcuts.

Work log:

- `2026-05-04 18:45:55 BST` Started `DC-05.5`: running the exact
  `attract_boot` local reference comparison to prove or isolate the remaining
  boot/start-ready trace gap before deciding whether the current source-shaped
  boot surfaces are sufficient to close `DC-05`.
- `2026-05-04 19:05:30 BST` Completed `DC-05.5`: removed the premature cold
  boot process-list initialization, aligned the MAME-observed RAM-fill pass
  targets, extended the SINIT clear boundary, split INIT20 process-list and
  object-list handoff side effects, and created the boot ATTR process from the
  source-shaped object-list handoff. The exact local `attract_boot` comparison
  now matches through frame 732 and first fails at line 734, frame 733, only in
  `process_table_crc32` (`0x62E1AD30` expected, `0xA424BDF6` actual), isolating
  the remaining work to post-INIT20 ATTR/executive scheduler cadence. Updated
  `SPEC.md`, `docs/fidelity/gaps.md`, and
  `docs/fidelity/golden-comparison-results.md` with the narrowed fixture gap.
  Adjusted refactor-safety tests for the new boot surfaces and kept the
  credited-start trace test pinned to current behavior while the exact local
  gameplay references remain ignored. Validation passed with `cargo fmt
  --check`, `cargo test --all-targets`, the exact local `attract_boot`
  comparison as an expected frame-733 failure, `markdownlint PLAN.md SPEC.md
  docs/fidelity/gaps.md docs/fidelity/golden-comparison-results.md`,
  `git diff --check`, and `make fidelity`. `make fidelity` passed regular tests
  with 806 passed and 13 known ignored tests, main tests with 2 passed, clippy,
  trace script tests, and coverage with 19/19 added executable Rust lines
  covered; `make trace-fixtures` skipped only because the ignored local
  `docs/fidelity/fixtures/local/rust-current` directory is absent.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777917986222729`
- `2026-05-04 18:36:18 BST` Started `DC-05.4`: auditing the `DC-05`
  board and machine byte surfaces already wired into frame stepping, then
  adding focused mutation tests for any touched RAM, CMOS, palette, latch, or
  source-visible status ranges that are not already protected.
- `2026-05-04 18:44:31 BST` Completed `DC-05.4`: added refactor-safety
  mutation tests for the cold-boot RAM-fill pass ranges, SINIT observed clear
  segments, INIT20 today's-high-score/process/super-process/object/list/status
  byte ranges, and sound-latch preservation across INIT20 object-list setup.
  Expanded the main-board live IRQ test to prove palette RAM mutates from the
  PCRAM source bytes, and expanded save-state coverage to round-trip a CMOS
  cell and palette RAM byte alongside RAM, hardware-map, main-board, sound-latch,
  and trace-scheduler state. Validation passed with
  `cargo test power_up_ram_fill_mutates_observed_pass_ranges --all-targets`,
  `cargo test sinit_handoff_clears_each_observed_ram_segment --all-targets`,
  `cargo test init20_handoff_mutates_list_score_status_and_latch_ranges
  --all-targets`,
  `cargo test step_records_live_irq_watchdog_and_video_counter_surface
  --all-targets`,
  `cargo test
  save_state_restore_round_trips_red_label_memory_and_trace_scheduler
  --all-targets`, `cargo fmt --check`, and `make fidelity`.
  `make fidelity` passed regular tests with 806 passed and 13 known ignored
  tests, main tests with 2 passed, clippy, trace script tests, and coverage
  with 0/0 added executable Rust lines because this step only added test code;
  `make trace-fixtures` skipped only because the ignored local
  `docs/fidelity/fixtures/local/rust-current` directory is absent.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777916705268889`
- `2026-05-04 18:29:44 BST` Started `DC-05.3`: routing frame-level
  sound-command writes through the MAME-modeled main-board latch and
  sound-board PIA boundary while preserving the existing command sequence as
  the observable output until the sound CPU is cycle-scheduled.
- `2026-05-04 18:35:07 BST` Completed `DC-05.3`: added
  `RedLabelSoundBoardSnapshot` and routed every emitted frame sound command
  through `SoundCommandLatch::from_main_board_pia_port_b`, preserving the
  current frame command sequence while exposing the MAME-modeled port-B and CB1
  latch boundary for later sound-CPU scheduling. Save-state restore now includes
  the sound-board latch and write count. Updated `SPEC.md` and
  `docs/fidelity/gaps.md` to document that the latch boundary is modeled while
  full sound-CPU execution remains a gap. Validation passed with
  `cargo test frame_sound_commands_update_sound_board_latch_surface
  --all-targets`, `cargo test
  idle_sound_command_updates_latch_without_asserting_cb1 --all-targets`,
  `cargo test
  save_state_restore_round_trips_red_label_memory_and_trace_scheduler
  --all-targets`,
  `markdownlint PLAN.md SPEC.md README.md docs/fidelity/README.md
  docs/fidelity/gaps.md docs/fidelity/characterization-tests.md
  docs/fidelity/local-reference-runs.md
  docs/fidelity/golden-comparison-results.md assets/red-label/README.md`,
  `git diff --check`, `cargo fmt --check`, and `make fidelity`.
  `make fidelity` passed regular tests with 803 passed and 13 known ignored
  tests, main tests with 2 passed, clippy, trace script tests, and coverage
  with 24/24 added executable Rust lines covered; `make trace-fixtures` skipped
  only because the ignored local `docs/fidelity/fixtures/local/rust-current`
  directory is absent.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777916151274269`
- `2026-05-04 18:21:16 BST` Started `DC-05.2`: wiring the existing
  MAME-modeled main-board RAM, CMOS, PIA, palette, watchdog, and video-counter
  surfaces into the `ArcadeMachine::step` path without changing gameplay
  semantics yet, so later exact CPU/IRQ work can replace the remaining
  high-level scheduler safely.
- `2026-05-04 18:28:29 BST` Completed `DC-05.2`: added a
  `RedLabelMainBoardSnapshot` surface and step-level main-board state for PIA
  input-port bytes, RAM/CMOS CRCs, palette RAM, hardware-map state, watchdog
  reset recognition, and the modeled video-counter sample. `ArcadeMachine::step`
  now refreshes the PIA input-port surface every frame and records watchdog and
  video-counter effects from the translated live IRQ video frame while
  preserving existing gameplay semantics. Save-state restore now includes the
  main-board input/watchdog/video-counter state. Added focused tests for the
  input/memory/palette snapshot, live IRQ watchdog/video-counter recording, and
  save-state restoration of the new surface. Updated `SPEC.md` and
  `docs/fidelity/gaps.md` to state that this is a board-facing snapshot, not
  full CPU/IRQ/sound execution. Validation passed with
  `cargo test step_updates_main_board_input_and_memory_surfaces --all-targets`,
  `cargo test step_records_live_irq_watchdog_and_video_counter_surface
  --all-targets`,
  `cargo test save_state_restore_round_trips_red_label_memory_and_trace_scheduler
  --all-targets`,
  `markdownlint PLAN.md SPEC.md README.md docs/fidelity/README.md
  docs/fidelity/gaps.md docs/fidelity/characterization-tests.md
  docs/fidelity/local-reference-runs.md
  docs/fidelity/golden-comparison-results.md assets/red-label/README.md`,
  `git diff --check`, `cargo fmt --check`, and `make fidelity`.
  `make fidelity` passed regular tests with 801 passed and 13 known ignored
  tests, clippy, trace script tests, and coverage with 35/35 added executable
  Rust lines covered; `make trace-fixtures` skipped only because the ignored
  local `docs/fidelity/fixtures/local/rust-current` directory is absent.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777915754689049`
- `2026-05-04 18:09:14 BST` Started `DC-05.1`: modeling the source-shaped
  power-on/reset diagnostic progression and start-ready transition as a
  deterministic boot-state surface before wiring board hardware directly into
  `ArcadeMachine::step` in later `DC-05` steps.
- `2026-05-04 18:19:19 BST` Completed `DC-05.1`: added
  `RedLabelPowerOnFrameModel` and `RedLabelPowerOnStage` to centralize the
  source/MAME-observed reset hold, RAM-test fill targets, `SINIT` clears,
  `INIT20` sound/list handoff, `EXEC` idle seeding, live-input holdoff, and
  start-ready transition. Rewired the cold-boot trace handoff to consume that
  model and added focused unit/mutation tests for the modeled frame boundaries,
  SINIT RAM clears, RNG writes, INIT20 sound command, STATUS write, phase
  changes, and start-ready RAND advance. Updated `SPEC.md` and
  `docs/fidelity/gaps.md` so the remaining gap is explicit board/CPU/IRQ/sound
  execution and fixture proof, not the frame model itself. Validation passed
  with `cargo test power_on_frame_model --all-targets`,
  `cargo test trace_power_on_handoff_applies_model_mutations --all-targets`,
  `markdownlint PLAN.md SPEC.md README.md docs/fidelity/README.md
  docs/fidelity/gaps.md docs/fidelity/characterization-tests.md
  docs/fidelity/local-reference-runs.md
  docs/fidelity/golden-comparison-results.md assets/red-label/README.md`,
  `git diff --check`, `cargo fmt --check`, and `make fidelity`.
  `make fidelity` passed regular tests with 799 passed and 13 known ignored tests,
  clippy, trace script tests, and coverage with 74/74 added executable Rust
  lines covered; `make trace-fixtures` skipped only because the ignored local
  `docs/fidelity/fixtures/local/rust-current` directory is absent.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777915229545099`

### DC-06: Process Scheduler Completion

Status: `complete`

Goal: make red-label process execution the normal core path.

Steps:

- [x] DC-06.1 Add register-aware process entry/resume state where source
  routines require A/B/X/Y/U/S/CC context.
  Completed: `2026-05-04 19:27:41 BST`
- [x] DC-06.2 Complete `PLRES` swarmer respawn once the entry B register value
  is trace-proven.
  Completed: `2026-05-04 20:13:37 BST`
- [x] DC-06.3 Finish suicide, resume, delay, super-process, and generic body
  dispatch semantics.
  Completed: `2026-05-04 20:17:12 BST`
- [x] DC-06.4 Verify object/process order against golden traces.
  Completed: `2026-05-04 20:20:58 BST`
- [x] DC-06.5 Remove scaffold state that duplicates exact red-label table
  fields.
  Completed: `2026-05-04 20:29:59 BST`

Completion gate: translated process dispatch drives gameplay state by default,
with explicit gaps only for unimplemented routine bodies.

Work log:

- `2026-05-04 19:19:05 BST` Started `DC-06.1`: adding a
  register-aware scheduled-process surface so translated routines can receive
  source-shaped A/B/X/Y/U/S/CC entry and resume context instead of depending
  only on `PADDR` and ambient RAM.
- `2026-05-04 19:27:41 BST` Completed `DC-06.1`: added
  `RedLabelCpuRegisters` and register-aware scheduled-process state, wired the
  process scheduler and translated dispatch paths through the source `DISP`
  context, and validate `U`/`CRPROC`/process-cell agreement before dispatching.
  Source evidence currently proves `U` is the due process cell; A/B/X/Y/S/CC
  remain explicit unknowns until CPU/IRQ scheduling or traces prove exact values.
  Added unit coverage for the register snapshot, mismatched source context, and
  lower-level runtime dispatch path, and updated `SPEC.md` plus
  `docs/fidelity/gaps.md`. Validation passed with focused scheduler/dispatch
  tests, `make fidelity` (`cargo fmt --check`, `cargo test --all-targets` with
  809 passed and 13 known ignored library tests plus 2 binary tests, clippy,
  trace script tests, skipped absent local fixture directory, and coverage with
  44/44 added executable Rust lines covered), `markdownlint PLAN.md SPEC.md
  docs/fidelity/gaps.md`, and `git diff --check`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777919311049749`
- `2026-05-04 20:06:47 BST` Started `DC-06.2`: moving the source `PLS1`/`PLRES`
  B-register value used by mini-swarmer reserve restore into explicit scheduled
  entry-register context, with tests for both the proven B value and the
  missing-context failure mode.
- `2026-05-04 20:13:37 BST` Completed `DC-06.2`: wired scheduled
  `PLS1` entry context through translated dispatch so targetless `PLRES`
  mini-swarmer reserve restore consumes source-proven B=`0x07` instead of a
  hard-coded fallback, added guards for mismatched `U` and missing B before
  list mutations, and added mutation-preserving unit tests for the proven B
  snapshot plus both failure modes. Updated `SPEC.md`, `README.md`, and
  `docs/fidelity/gaps.md` to record that A/X/Y/S/CC and other B values remain
  unknown. Validation passed with focused scheduler/PLRES tests,
  `markdownlint PLAN.md SPEC.md README.md docs/fidelity/gaps.md`,
  `git diff --check`, and `make fidelity` (`cargo fmt --check`,
  `cargo test --all-targets` with 812 passed and 13 known ignored library tests
  plus 2 binary tests, clippy, trace script tests, skipped absent local fixture
  directory, and coverage with 77/77 added executable Rust lines covered).
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777922045347629`
- `2026-05-04 20:14:44 BST` Started `DC-06.3`: auditing the translated
  process tails and dispatch loop for source-shaped `SUCIDE`, `SLEEP`,
  resume-through-`DISP2`, super-process allocation/freeing, and generic
  translated body dispatch, with new unit coverage for any scheduler mutation
  semantics still handled by scaffolding.
- `2026-05-04 20:17:12 BST` Completed `DC-06.3`: added executive-loop
  coverage proving a super-process can run a translated generic body, sleep to
  its resume address, preserve the `SPFREE` list, keep its `PCOD` super-process
  marker, and still allow surrounding regular `SUCIDE` processes to continue
  through the same `DISP2` pass with regular cells returned to `FREE`. Updated
  `SPEC.md` and `docs/fidelity/gaps.md` to record the regular/super-process
  scheduler semantics while leaving untranslated body coverage as an explicit
  gap. Validation passed with the focused super-process executive-loop unit
  test,
  `markdownlint PLAN.md SPEC.md README.md docs/fidelity/gaps.md`,
  `git diff --check`, and `make fidelity` (`cargo fmt --check`,
  `cargo test --all-targets` with 813 passed and 13 known ignored library tests
  plus 2 binary tests, clippy, trace script tests, skipped absent local fixture
  directory, and coverage with no new executable Rust lines outside tests).
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777922256258889`
- `2026-05-04 20:18:24 BST` Started `DC-06.4`: running the local
  object/process golden-trace comparison gates for the translated scheduler,
  then recording either the exact passing fixture proof or the current
  source-visible drift as a documented fidelity gap with tests.
- `2026-05-04 20:20:58 BST` Completed `DC-06.4`: rechecked the local
  `attract_boot` object/process/super-process/shell CRC comparison after the
  translated scheduler work. `make reference-fixtures-check` validated 12
  Phase 1 reference fixtures and 22,308 frames. The exact `attract_boot`
  comparison still intentionally fails first at line 734, frame 733, solely in
  `process_table_crc32` (`0x62E1AD30` expected, `0xA424BDF6` actual), with 168
  process-table mismatches through frame 900. Object-table, super-process-table,
  and shell-table CRCs match the 900-frame reference, so the remaining gap is
  post-INIT20 ATTR/executive process cadence. Recorded the result in
  `docs/fidelity/golden-comparison-results.md`, `docs/fidelity/gaps.md`, and
  `SPEC.md`. Validation passed with
  `markdownlint PLAN.md SPEC.md README.md docs/fidelity/gaps.md
  docs/fidelity/golden-comparison-results.md`, `git diff --check`, and
  `make fidelity` (`cargo fmt --check`, `cargo test --all-targets` with 813
  passed and 13 known ignored library tests plus 2 binary tests, clippy, trace
  script tests, skipped absent local Rust-current fixture directory, and
  coverage with no new executable Rust lines outside docs).
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777922489392159`
- `2026-05-04 20:21:47 BST` Started `DC-06.5`: auditing cached scaffold state
  that duplicates exact red-label table fields now owned by the translated
  process/object/player RAM surfaces, then removing only redundant fields whose
  behavior can be preserved with existing mutation and trace tests.
- `2026-05-04 20:29:59 BST` Completed `DC-06.5`: introduced a table-backed
  snapshot projection over red-label RAM/CMOS for credits, current player, wave,
  player runtime state, scores, high score, and RNG, with the cached fallback
  limited to cold-boot trace frames before source tables are initialized and to
  compatibility/control state. Added a regression test that intentionally stales
  cached fields and proves public snapshots read RAM/CMOS, adjusted the `xyzzy`
  overlay test to seed smart-bomb inventory through source RAM, and updated
  `README.md`, `SPEC.md`, and `docs/fidelity/gaps.md`. Validation passed with
  `cargo test snapshot_ --all-targets`,
  `cargo test xyzzy_auto_fire_and_unlimited_bombs_are_overlay_state --all-targets`,
  `markdownlint PLAN.md SPEC.md README.md docs/fidelity/gaps.md`,
  `git diff --check`, and `make fidelity`
  (`cargo fmt --check`, `cargo test --all-targets` with 814 passed and 13 known
  ignored library tests plus 2 binary tests, clippy, trace script tests, skipped
  absent local Rust-current fixture directory, and coverage with 70/70 added
  executable Rust lines covered).
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777923063018929`

### DC-07: IRQ, Scanline, And Palette Integration

Status: `complete`

Goal: integrate frame timing, scanline ownership, palette copy, and hardware map
restoration into the core.

Steps:

- [x] DC-07.1 Implement source-exact CPU IRQ cadence and screen scanline
  scheduling from MAME/source measurements.
  Completed: `2026-05-04 21:16:00 BST`
- [x] DC-07.2 Finish `BGOUT` live stack-context wiring and hardware-map
  restoration.
  Completed: `2026-05-04 21:17:16 BST`
- [x] DC-07.3 Finish palette-copy side effects and validate palette RAM
  mutations.
  Completed: `2026-05-04 21:18:54 BST`
- [x] DC-07.4 Emit native video CRCs from actual red-label video RAM in trace
  output.
  Completed: `2026-05-04 21:22:56 BST`
- [x] DC-07.5 Add focused pixel/frame fixtures for scanline-sensitive slices.
  Completed: `2026-05-04 21:26:33 BST`

Completion gate: one full core frame mutates RAM, video RAM, palette RAM,
object lists, process lists, and sound commands in source order.

Work log:

- `2026-05-04 21:14:20 BST` Started `DC-07.1`: turning the existing upright
  `IRQ` / flipped `IRQB` live scanline constants into an explicit frame
  schedule surface, then validating that live frame execution uses the
  source-selected `IRQHK` mode and the measured upper/lower `VERTCT` points.
- `2026-05-04 21:16:00 BST` Completed `DC-07.1`: added
  `RedLabelLiveIrqFrameSchedule` as the public source-derived live IRQ frame
  schedule for upright `IRQ` and flipped `IRQB`, exposed it through
  `ArcadeMachine`, and wired the live frame runner through that schedule rather
  than directly embedding the scanline counters. Extended the upright/flipped
  live IRQ tests to assert the `IRQHK`-selected schedule and the exact
  upper/lower `VERTCT` points. Validation passed with `cargo fmt --check` and
  `cargo test live_irq_video_frame_uses_ --all-targets`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777925778062459`
- `2026-05-04 21:16:35 BST` Started `DC-07.2`: checking the live IRQ terrain
  branch end to end so `BGOUT` receives the derived source IRQ stack pointer
  and the `MAPC` clear/select/restore sequence leaves hardware-map selection
  restored from `MAPCR` after the full frame pass.
- `2026-05-04 21:17:16 BST` Completed `DC-07.2`: added live-frame regression
  coverage proving upright live IRQ execution runs translated `BGOUT` with the
  derived source stack pointer `0xBFF1`, records the source `SSTACK` bytes, and
  restores hardware-map selection from `MAPCR` after both upper and lower
  scanline passes. Validation passed with `cargo fmt --check` and
  `cargo test live_irq_video_frame_runs_bgout_and_restores_hardware_map_from_mapcr
  --all-targets`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777925859203229`
- `2026-05-04 21:18:12 BST` Started `DC-07.3`: adding focused live-frame
  palette mutation coverage around the source `PSHU` copy from `PCRAM` into
  hardware color RAM, including both the scheduler's copy record and the actual
  palette-RAM bytes used by native frame rendering.
- `2026-05-04 21:18:54 BST` Completed `DC-07.3`: added live-frame regression
  coverage proving the lower IRQ pass copies `PCRAM` into hardware palette RAM,
  returns the exact `RedLabelPaletteCopy` record, mutates the machine palette
  RAM bytes, and drives native RGBA rendering through the copied palette values.
  Validation passed with `cargo fmt --check` and
  `cargo test live_irq_video_frame_copies_pcram_into_palette_ram_for_native_rendering
  --all-targets`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777925952570069`
- `2026-05-04 21:19:32 BST` Started `DC-07.4`: wiring native visible-video
  CRC emission into `FrameOutput` and `TraceFrame::from_output` so trace TSV
  rows no longer leave `video_crc32` empty when the red-label video RAM surface
  can render a native frame.
- `2026-05-04 21:22:56 BST` Completed `DC-07.4`: added native visible-video
  CRCs to `FrameOutput`, computed from the decoded red-label visible
  pixel-nibble frame, and wired `TraceFrame::from_output` so generated TSV rows
  now carry `video_crc32` directly from machine output. Updated trace docs in
  `README.md`, `SPEC.md`, `docs/fidelity/README.md`, and
  `docs/fidelity/gaps.md`. Validation passed with `cargo fmt --check`,
  `cargo test trace_frame_records_machine_output_with_red_label_ram_observed_state
  --all-targets`,
  `cargo test trace_output_includes_header_and_empty_event_marker --all-targets`,
  `cargo test fidelity_trace_text_emits_header_and_requested_frame_count
  --all-targets`, `markdownlint PLAN.md README.md SPEC.md
  docs/fidelity/README.md docs/fidelity/gaps.md`, and `git diff --check`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777926227787119`
- `2026-05-04 21:24:14 BST` Started `DC-07.5`: adding a focused native
  pixel-nibble fixture for the live IRQ scanline frame so upper/lower
  `PRDISP`/`OPROC`/`SHELL`/`VELO` band work is tied to a stable rendered
  red-label video-RAM checksum before later refactors.
- `2026-05-04 21:26:33 BST` Completed `DC-07.5`: added a live IRQ
  scanline-sensitive native pixel-nibble fixture that runs the normal live IRQ
  frame with an active descriptor object, checks the upper/lower scanline band
  phases, proves video RAM changed from the pre-frame image, and locks the
  resulting visible pixel CRC at `0xB0E35EDE`. Updated `README.md`, `SPEC.md`,
  and `docs/fidelity/gaps.md` to remove stale main-IRQ scanline-gap wording and
  record the new live IRQ pixel fixture. Validation passed with
  `cargo fmt --check`,
  `cargo test live_irq_video_frame_has_scanline_sensitive_pixel_fixture --all-targets`,
  `markdownlint PLAN.md README.md SPEC.md docs/fidelity/README.md
  docs/fidelity/gaps.md`, and `git diff --check`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777926418052399`
- `2026-05-04 21:29:58 BST` Completed `DC-07` gate: `make fidelity`
  passed after the DC-07 live IRQ schedule, palette, video CRC, and
  scanline-sensitive pixel fixture work. The gate covered `cargo fmt --check`,
  `cargo test --all-targets` with 817 passed, 0 failed, and 13 known ignored
  library tests plus 2 passed binary tests, new-line coverage for 5 of 5 added
  executable Rust lines, and regenerated lcov/cobertura reports.

## Phase 3: Player, Session, And Cabinet Flow

### DC-08: Player Control And Respawn Completion

Status: `complete`

Goal: finish the exact player path around controls, death, respawn, and carried
humans.

Steps:

- [x] DC-08.1 Finish reverse integration with player movement and rendering.
  Completed: `2026-05-04 21:35:48 BST`
- [x] DC-08.2 Prove laser cap, laser lifetime, collision setup, and fizzle/erase
  timing against traces.
  Completed: `2026-05-04 21:38:30 BST`
- [x] DC-08.3 Finish smart-bomb world integration, including what is and is not
  destroyed.
  Completed: `2026-05-04 21:41:32 BST`
- [x] DC-08.4 Finish hyperspace failure/safe paths and rematerialization
  movement.
  Completed: `2026-05-04 21:44:47 BST`
- [x] DC-08.5 Finish carried-human, falling-human, catch, landing, and rescue
  paths.
  Completed: `2026-05-04 21:48:08 BST`
- [x] DC-08.6 Prove death, respawn, wave-clear, and game-over branches with
  golden traces.
  Completed: `2026-05-04 21:54:58 BST`

Completion gate: the player path can be exercised from live inputs through
source-shaped process execution and trace-proven state mutations.

Work log:

- `2026-05-04 21:33:53 BST` Started `DC-08.1`: adding live-frame reverse
  coverage that ties the translated `REV` switch process to player motion,
  `PRDISP` direction commit, and native video RAM output before later player
  refactors.
- `2026-05-04 21:35:48 BST` Completed `DC-08.1`: added
  `live_irq_frame_commits_reverse_process_to_motion_and_player_rendering`,
  proving translated `REV` writes the reversed direction, the live IRQ frame
  keeps source `PLAYER` motion stable, `PRDISP` commits `PLBPIC` rendering, RAM
  mirrors `NPLAD` / `PLADIR`, and the native visible-video CRC locks at
  `0x6B88AF41`. Validation passed with `cargo fmt --check` and
  `cargo test live_irq_frame_commits_reverse_process_to_motion_and_player_rendering
  --all-targets`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777926987448799`
- `2026-05-04 21:37:29 BST` Started `DC-08.2`: adding a consolidated laser
  lifecycle regression that covers the source four-laser cap plus the
  fizzle/body/tail erase/collision finish mutations used by firing traces.
- `2026-05-04 21:38:30 BST` Completed `DC-08.2`: added
  `laser_lifecycle_fixture_covers_cap_fizzle_collision_and_erase_mutations`,
  proving the four-laser `LFIRE` cap suicides without mutating sound state,
  `LASR0` advances `FISX`, records the body/fizzle/tail process pointers,
  dispatches collision through `LCOL`/`NOKILL`, erases the full beam trail, and
  decrements `LFLG` on finish. Validation passed with `cargo fmt --check` and
  `cargo test laser_lifecycle_fixture_covers_cap_fizzle_collision_and_erase_mutations
  --all-targets`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777927131955319`
- `2026-05-04 21:40:33 BST` Started `DC-08.3`: adding a smart-bomb world
  integration fixture that proves the active-object walk destroys qualifying
  enemies, preserves non-enemy object types, and leaves the source flash/debounce
  state ready for the existing tail tests.
- `2026-05-04 21:41:32 BST` Completed `DC-08.3`: added
  `smart_bomb_world_fixture_kills_enemies_and_preserves_non_enemy_objects`,
  proving `SBOMB` decrements current-player stock, kills a qualifying
  mini-swarmer through `MSWKIL`, clears its visible footprint and active count,
  preserves a non-enemy active object with `OTYP=0x02`, and leaves the source
  `SBMBX1` flash sleep/`SBFLG` state ready for the existing debounce-tail
  tests. Validation passed with `cargo fmt --check` and
  `cargo test smart_bomb_world_fixture_kills_enemies_and_preserves_non_enemy_objects
  --all-targets`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777927313145899`
- `2026-05-04 21:42:42 BST` Started `DC-08.4`: adding a hyperspace refactor
  fixture that proves the guarded entry path, rematerialization movement/object
  creation, safe `HYP2` cleanup, and high-`LSEED` death-risk branch together.
- `2026-05-04 21:44:47 BST` Completed `DC-08.4`: added
  `hyperspace_fixture_covers_guard_rematerialization_safe_tail_and_death_risk`,
  proving `HYPER` suppresses blocked status without mutation, `HYP02` derives
  background/player position/direction from `SEED`/`HSEED`, creates the phony
  player appearance object, safe `HYP2` clears status and suicides, and high
  `LSEED` branches to `PLEND` without killing the hyperspace process.
  Validation passed with `cargo fmt --check` and
  `cargo test hyperspace_fixture_covers_guard_rematerialization_safe_tail_and_death_risk
  --all-targets`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777927508535499`
- `2026-05-04 21:46:26 BST` Started `DC-08.5`: adding a human rescue
  refactor fixture that ties `AFALL2` player-carried motion, `AKIL1` catch,
  `AFALL` safe landing, and `P500` rescue scoring to stable RAM mutations.
- `2026-05-04 21:48:08 BST` Completed `DC-08.5`: added
  `human_rescue_fixture_covers_carried_catch_landing_and_score_paths`, proving
  player-carried `AFALL2` updates astronaut `OX16`/`OY16` from `PLABX` and
  `PLAY16`, `AKIL1` switches the fall process to `AFALL2` and starts `P500`,
  safe `AFALL` landing resets the astronaut and starts `P250`, and `P500`
  rescue scoring creates the score sprite, carries player X velocity, and adds
  500 points. Validation passed with `cargo fmt --check` and
  `cargo test human_rescue_fixture_covers_carried_catch_landing_and_score_paths
  --all-targets`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777927711664639`
- `2026-05-04 21:49:31 BST` Started `DC-08.6`: adding a player-death branch
  fixture for respawn, one-player game over, two-player switch/game-over text,
  and wave-clear bonus restart before running the full DC-08 validation gate.
- `2026-05-04 21:54:58 BST` Completed `DC-08.6`: added
  `player_death_branch_fixture_covers_respawn_game_over_switch_and_wave_bonus`,
  proving the one-player respawn branch, one-player game-over sleep,
  two-player switch/game-over text branch, wave-clear survivor bonus loop, wave
  advance, and `BC3` restart into `PLSTR0`. Updated `README.md`, `SPEC.md`,
  and `docs/fidelity/gaps.md` to remove stale DC-08 gaps and keep the remaining
  documented gaps focused on frame/cycle integration, session/operator flow, and
  non-gameplay presentation. Validation passed with `cargo fmt --check`,
  `cargo test player_death_branch_fixture_covers_respawn_game_over_switch_and_wave_bonus
  --all-targets`, `make fidelity`, markdownlint, and `git diff --check`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777928097936399`
- `2026-05-04 21:54:58 BST` Completed `DC-08` gate: `make fidelity`
  passed after the DC-08.6 fixture and docs updates. The gate included
  `cargo fmt --check`, `cargo test --all-targets` (`823 passed; 0 failed;
  13 ignored` library tests, plus 2 CLI tests), `cargo clippy --all-targets
  -- -D warnings`, Lua trace self-test, Python trace/coverage helper tests,
  skipped missing local Rust trace fixture directory as expected, generated
  `target/coverage/lcov.info` and `target/coverage/coverage.xml`, and reported
  `new Rust line coverage: 0/0 added executable line(s)`.

### DC-09: Cabinet Session And Operator Flow

Status: `complete`

Goal: complete exact credits, starts, high scores, two-player alternation, and
operator behavior.

Steps:

- [x] DC-09.1 Finish one-player and two-player session flow, including
  alternating turns and current-player table swaps.
  Completed: `2026-05-04 22:01:58 BST`
- [x] DC-09.2 Finish credit, coinage, free-play, replay, and bonus-stock rules.
  Completed: `2026-05-04 22:03:46 BST`
- [x] DC-09.3 Translate exact initials-entry UI, high-score screens, and
  game-over-to-attract timing.
  Completed: `2026-05-04 22:05:20 BST`
- [x] DC-09.4 Decide and document the default live CMOS persistence policy.
  Completed: `2026-05-04 22:07:24 BST`
- [x] DC-09.5 Complete diagnostics, audit, and adjustment flow in live cabinet
  mode.
  Completed: `2026-05-04 22:09:16 BST`
- [x] DC-09.6 Add regression tests for P1/P2 score, lives, smart bombs, credits,
  high-score insertion, and CMOS mutations.
  Completed: `2026-05-04 22:17:15 BST`

Completion gate: cabinet session behavior is covered by translated
source-shaped regression fixtures, remaining end-to-end golden-trace gaps are
documented, and operator-facing documentation is current.

Work log:

- `2026-05-04 21:59:36 BST` Started `DC-09.1`: adding a
  mutation-preserving live session fixture that starts a two-player game through
  the translated switch path, then proves the later death/switch handoff moves
  source current-player state to the surviving player.
- `2026-05-04 22:01:58 BST` Completed `DC-09.1`: added
  `session_flow_fixture_covers_two_player_start_and_player_switch_respawn`,
  proving a credited live two-player `ST2` start mutates credit/player-count
  state, then the translated `PDTH5R` / `PLE02` branch renders player-switch
  text, moves `CURPLR` to player 2, updates `PDFLG`, and points the process
  back to `PLSTRT`. Validation passed with `cargo fmt --check`,
  `cargo test session_flow_fixture_covers_two_player_start_and_player_switch_respawn
  --all-targets`, `markdownlint PLAN.md`, and `git diff --check`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777928532126079`
- `2026-05-04 22:02:42 BST` Started `DC-09.2`: adding a combined credit
  accounting fixture that proves coin multiplier credits, free-play start
  credits, CMOS credit backup writes, replay sound loading, and bonus-stock
  mutations remain stable for the later refactor.
- `2026-05-04 22:03:46 BST` Completed `DC-09.2`: added
  `credit_rules_fixture_covers_coinage_freeplay_replay_and_bonus_stock`, proving
  center-slot multiplier credit accounting, paid-credit audit mutation, CMOS
  `CREDST` backup, free-play two-player start without a coin event, replay-award
  bonus-stock mutation, and `RPSND` sound loading. Validation passed with
  `cargo fmt --check`,
  `cargo test credit_rules_fixture_covers_coinage_freeplay_replay_and_bonus_stock
  --all-targets`, `markdownlint PLAN.md`, and `git diff --check`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777928635916079`
- `2026-05-04 22:04:25 BST` Started `DC-09.3`: adding a high-score flow
  fixture that joins live initials acceptance/submission, source hall-of-fame
  screen mutation, and translated game-over no-entry delay back to attract.
- `2026-05-04 22:05:20 BST` Completed `DC-09.3`: added
  `high_score_flow_fixture_covers_initials_hall_display_and_no_entry_attract_delay`,
  proving live initials acceptance/submission inserts `ACE` into all-time CMOS,
  renders the source hall-of-fame screen, emits high-score entry/submission
  events, and the non-qualifying game-over path waits through translated
  `PLE3` / `HALL13` timing before returning to attract. Validation passed with
  `cargo fmt --check`,
  `cargo test high_score_flow_fixture_covers_initials_hall_display_and_no_entry_attract_delay
  --all-targets`, `markdownlint PLAN.md`, and `git diff --check`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777928733411969`
- `2026-05-04 22:06:02 BST` Started `DC-09.4`: documenting the default
  live CMOS persistence policy as explicit opt-in storage via `--cmos-path`,
  with embedded red-label defaults used when no path is provided.
- `2026-05-04 22:07:24 BST` Completed `DC-09.4`: documented the live CMOS
  persistence policy in `README.md` and `SPEC.md`, removed the stale default
  CMOS path gap from `docs/fidelity/gaps.md`, and kept the policy explicit:
  embedded red-label defaults are used unless `--cmos-path` is provided, and no
  implicit platform default CMOS file is created. Validation passed with
  `cargo test live_cmos_storage_loads_and_saves_machine_cmos --all-targets`,
  `markdownlint PLAN.md README.md SPEC.md docs/fidelity/gaps.md`, and
  `git diff --check`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777928856353239`
- `2026-05-04 22:08:19 BST` Started `DC-09.5`: adding a live admin-switch
  fixture for diagnostics, audits, and high-score reset events/mutations, then
  rechecking the existing board audit-adjustment regression that owns the
  source `AUDITG` row mutation behavior.
- `2026-05-04 22:09:16 BST` Completed `DC-09.5`: added
  `operator_flow_fixture_covers_live_diagnostics_audits_and_reset_events`,
  proving live service advance selects diagnostics, auto+advance selects audits,
  high-score reset emits the live event, resets today's table from ROM defaults,
  mutates `HSRFLG`, and clears the admin switch process. Re-ran the board-level
  `AUDITG` adjustment regression proving high-score reset adjustment mutations
  through the source operator row path. Validation passed with `cargo fmt
  --check`,
  `cargo test operator_flow_fixture_covers_live_diagnostics_audits_and_reset_events
  --all-targets`,
  `cargo test main_board_auditg_operator_step_applies_high_score_reset_adjustments
  --all-targets`, `markdownlint PLAN.md`, and `git diff --check`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777928969712859`
- `2026-05-04 22:10:25 BST` Started `DC-09.6`: adding a consolidated
  refactor-safety regression fixture for P1/P2 scores, lives, smart bombs,
  credits, high-score insertion, and CMOS mutations.
- `2026-05-04 22:17:15 BST` Completed `DC-09.6`: added
  `session_regression_fixture_locks_scores_stocks_credits_high_scores_and_cmos`,
  proving a translated two-credit/two-player start consumes credits and writes
  CMOS credit backup state, player-one and player-two score/smart-bomb paths
  mutate the correct source RAM while preserving lives, and submitted initials
  insert packed high-score entries into both all-time CMOS and today's RAM.
  Updated `SPEC.md` and `docs/fidelity/gaps.md` to keep cabinet-session logic
  separate from the still-open end-to-end golden-trace proof gap. Validation
  passed with `cargo fmt --check`,
  `cargo test session_regression_fixture_locks_scores_stocks_credits_high_scores_and_cmos
  --all-targets`, `markdownlint PLAN.md SPEC.md docs/fidelity/gaps.md`,
  `make fidelity` (`828` lib tests passed, `13` ignored; `2` main tests passed;
  clippy, Lua, Python, local trace-fixture skip, coverage, and new-code
  coverage checks passed), `markdownlint PLAN.md README.md SPEC.md
  docs/fidelity/README.md docs/fidelity/gaps.md`, and `git diff --check`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777929459523379`

## Phase 4: World, Waves, Humans, And Enemies

### DC-10: Wave And World State

Status: `complete`

Goal: make wave setup and world state source-exact.

Steps:

- [x] DC-10.1 Translate wave launch process order, enemy reserve allocation, and
  spawn timing from source.
  Completed: `2026-05-04 22:27:15 BST`
- [x] DC-10.2 Finish terrain scheduling, destroyed-planet terrain, scanner
  terrain, and fifth-wave human restoration.
  Completed: `2026-05-04 22:29:39 BST`
- [x] DC-10.3 Prove `GETWV`, `WDELT`, target restoration, and survivor bonus
  behavior with source-backed fixtures; keep end-to-end golden-trace proof in
  the documented fidelity gaps.
  Completed: `2026-05-04 22:31:53 BST`
- [x] DC-10.4 Add tests for wave-to-wave RAM mutation and object/process list
  changes.
  Completed: `2026-05-04 22:40:38 BST`

Completion gate: wave setup and world transitions are covered by source-backed
regression fixtures, remaining end-to-end golden-trace gaps are documented, and
fixture/local-reference hooks remain usable.

Work log:

- `2026-05-04 22:20:32 BST` Started `DC-10.1`: adding a combined
  refactor-safety fixture for translated `GEXEC` / `GEX0` launch order, UFO
  pacing, lander reserve allocation, and process sleep timing.
- `2026-05-04 22:27:15 BST` Completed `DC-10.1`: added
  `wave_launch_fixture_covers_exec_order_reserve_allocation_and_spawn_timing`,
  proving translated `GEXEC` seeds `PD`/`UFOTMR`/`WAVTMR`, applies source
  `WVCHK` gating, consumes lander reserves through the translated `LANDST` /
  schizoid-fallback path, mutates enemy counters and free-process state, sleeps
  back to `GEX0`, and separately proves timed `UFOST` launch increments
  `UFOCNT` and returns to the same `GEX0` cadence. Validation passed with
  `cargo fmt --check`,
  `cargo test wave_launch_fixture_covers_exec_order_reserve_allocation_and_spawn_timing
  --all-targets`, `markdownlint PLAN.md`, and `git diff --check`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777930065731819`
- `2026-05-04 22:28:28 BST` Started `DC-10.2`: adding a terrain/world
  fixture for live `BGOUT` scheduling, scanner mini-terrain writes,
  destroyed-planet `TERBLO` mutation, and fifth-wave human restoration.
- `2026-05-04 22:29:39 BST` Completed `DC-10.2`: added
  `world_terrain_fixture_covers_bgout_scanner_terblo_and_restore_wave`, proving
  `ALINIT`/`BGINIT`/`BGOUT` mutate `STBL` and preserve the IRQ stack context,
  scanner rendering writes 64 `MTERR` terrain blips through `STETAB`, `TERBLO`
  sets terrain-blown status, erases terrain/scanner footprints, spawns its
  first explosion pass, and fifth-wave `GETWV` restores `PTARG` to 10 humans.
  Validation passed with `cargo fmt --check`,
  `cargo test world_terrain_fixture_covers_bgout_scanner_terblo_and_restore_wave
  --all-targets`, `markdownlint PLAN.md`, and `git diff --check`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777930206530419`
- `2026-05-04 22:30:38 BST` Started `DC-10.3`: adding a wave-advance
  regression fixture for `GETWV`, inter-wall and intra-wall `WDELT`, target
  restoration, and survivor-bonus mutation while keeping end-to-end local MAME
  trace equivalence tracked as a fidelity gap.
- `2026-05-04 22:31:53 BST` Completed `DC-10.3`: added
  `wave_advance_fixture_covers_getwv_wdelt_targets_and_survivor_bonus`,
  proving fifth-wave `GETWV` restores `PTARG` and refreshes `PENEMY`,
  inter-wall `WDELT` applies the CMOS difficulty/ceiling progression,
  intra-wall `WDELT` mutates `ELIST` when `GEX6` wraps `PD`, survivor bonus
  scoring mutates the current-player score for each remaining astronaut, and
  the bonus handoff advances to `BC3` with refreshed wave parameters.
  Validation passed with `cargo fmt --check`,
  `cargo test wave_advance_fixture_covers_getwv_wdelt_targets_and_survivor_bonus
  --all-targets`, `markdownlint PLAN.md`, and `git diff --check`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777930343866209`
- `2026-05-04 22:33:07 BST` Started `DC-10.4`: adding the final DC-10
  regression fixture for wave-to-wave player RAM mutation, process-list cleanup,
  and player-restore object/list allocation side effects.
- `2026-05-04 22:40:38 BST` Completed `DC-10.4`: added
  `wave_to_wave_regression_fixture_locks_ram_and_process_object_lists`, proving
  wave-clear `GEX0` kills non-current processes, advances player wave RAM
  through `BONUS` / `GETWV`, restarts through `GEXBON` into the translated
  `PLSTRT` / `PLSTR3` handoff with a life increment, and `PLS1` restore
  allocates target objects while mutating target-list, free-object, and
  active-count RAM. Updated an older wave-launch test to expect one schizoid
  fallback when source reserve/wave size request one enemy, and updated
  `README.md`, `SPEC.md`, and `docs/fidelity/gaps.md` to mark DC-10 complete at
  the source-backed fixture level while keeping end-to-end MAME golden-trace
  proof as a documented fidelity gap. Validation passed with `cargo fmt --check`,
  `cargo test
  wave_to_wave_regression_fixture_locks_ram_and_process_object_lists
  --all-targets`,
  `cargo test game_exec_entry_launches_initial_lander_wave_and_sleeps_to_gex0
  --all-targets`, `markdownlint PLAN.md README.md SPEC.md
  docs/fidelity/gaps.md docs/fidelity/README.md`, `git diff --check`, and
  `make fidelity` (`832` lib tests passed, `13` ignored; `2` main tests passed;
  clippy, Lua, Python, local trace-fixture skip, coverage, and new-code coverage
  checks passed).
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777930882314369`

### DC-11: Remaining Enemy And Collision Behavior

Status: `complete`

Goal: finish all enemy, shell, mine, and collision routines.

Steps:

- [x] DC-11.1 Finish Baiter spawn, pursuit, firing, and lifetime behavior.
  Completed: `2026-05-04 22:46:31 BST`
- [x] DC-11.2 Finish Bomber, Pod, minefield, and Pod/Swarmer burst behavior.
  Completed: `2026-05-04 22:50:38 BST`
- [x] DC-11.3 Finish remaining mutant behavior and firing checks.
  Completed: `2026-05-04 22:56:23 BST`
- [x] DC-11.4 Finish remaining shell rendering/output and object collision
  vectors.
  Completed: `2026-05-04 22:59:23 BST`
- [x] DC-11.5 Add score regression tests for enemies, bullets, mines, humans,
  rescue, Pods, Swarmers, extra stocks, and smart bombs.
  Completed: `2026-05-04 23:03:57 BST`
- [x] DC-11.6 Verify each enemy subsystem with focused traces before moving on.
  Completed: `2026-05-04 23:06:03 BST`

Completion gate: all red-label enemy processes are either translated and tested
or explicitly recorded as gaps with ignored/failing tests.

Work log:

- `2026-05-04 22:42:54 BST` Started `DC-11.1`: adding a focused regression
  fixture for `UFOST` / `UFOLP` baiter spawn, pursuit, firing, picture cycle,
  and kill/lifetime counter mutations before moving to the other enemy families.
- `2026-05-04 22:46:31 BST` Completed `DC-11.1`: added
  `baiter_regression_fixture_covers_spawn_pursuit_firing_and_lifetime`, proving
  `UFOST` creates the `UFOLP` process/object pair with `UFOKIL` collision,
  `UFOLP` fires through `SHOOT` / `GETSHL`, wraps `UFOP3` back to `UFOP1`,
  updates pursuit velocity, mutates shell-list, object, process, and sound RAM,
  and `UFOKIL` decrements `UFOCNT`, kills the owning process/object, and scores
  200 points. Validation passed with `cargo fmt --check`,
  `cargo test baiter_regression_fixture_covers_spawn_pursuit_firing_and_lifetime
  --all-targets`, `markdownlint PLAN.md`, and `git diff --check`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777931213795719`
- `2026-05-04 22:47:15 BST` Started `DC-11.2`: adding a combined regression
  fixture for bomber/tie bomb-shell allocation, probe/pod kill-to-swarmer burst,
  and mini-swarmer movement/firing mutations.
- `2026-05-04 22:50:38 BST` Completed `DC-11.2`: added
  `bomber_pod_swarmer_fixture_covers_bombs_bursts_and_swarm_fire`, proving
  TIE bomb-shell allocation and `BMBCNT` mutation, probe kill-to-mini-swarmer
  burst creation and count mutation, and mini-swarmer movement, firing,
  shell-list, object, process, and sound-command mutations. Validation passed
  with `cargo fmt --check`,
  `cargo test bomber_pod_swarmer_fixture_covers_bombs_bursts_and_swarm_fire
  --all-targets`, `markdownlint PLAN.md`, and `git diff --check`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777931464832619`
- `2026-05-04 22:52:02 BST` Started `DC-11.3`: adding a mutant/schizoid
  characterization fixture that ties together reserve restore, `SCZ0`
  movement, firing, shell/sound mutations, and `SCZKIL` kill scoring.
- `2026-05-04 22:56:23 BST` Completed `DC-11.3`: added
  `mutant_regression_fixture_covers_restore_movement_fire_and_kill`, proving
  saved schizoid reserve restore, `SCZ0` horizontal/vertical movement and
  random hop mutation, due-shot shell allocation through `SHOOT`, shot sound
  RAM pointer state, `SCZKIL` count mutation, 150-point score mutation, killed
  object/process mutation, and kill sound RAM pointer state. Validation passed
  with `cargo fmt --check`,
  `cargo test mutant_regression_fixture_covers_restore_movement_fire_and_kill
  --all-targets`, `markdownlint PLAN.md`, and `git diff --check`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777931802683199`
- `2026-05-04 22:57:03 BST` Started `DC-11.4`: reviewing shell output and
  object collision-vector dispatch coverage, then adding a regression fixture
  for remaining shell rendering/output and collision-vector mutations.
- `2026-05-04 22:59:23 BST` Completed `DC-11.4`: added
  `shell_output_and_collision_fixture_covers_render_dispatch_and_vectors`,
  proving `SHELL` movement into `BMBOUT`, bomb-shell old-footprint erase and
  new video writes, `FBOUT` fireball erase/write behavior, `BKIL` collision
  dispatch through `OCVECT`, shell-list/free-list/count mutation, 25-point
  score mutation, shell-collision sound RAM pointer state, and source no-op
  `NOKILL` object/vector behavior without mutation. Validation passed with
  `cargo fmt --check`,
  `cargo test shell_output_and_collision_fixture_covers_render_dispatch_and_vectors
  --all-targets`, `markdownlint PLAN.md`, and `git diff --check`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777931983454029`
- `2026-05-04 23:00:05 BST` Started `DC-11.5`: adding score regression
  coverage for translated enemy, shell/mine, human/rescue, Pod/Swarmer,
  replay-stock, and smart-bomb award paths.
- `2026-05-04 23:03:57 BST` Completed `DC-11.5`: added
  `score_regression_fixture_covers_enemy_human_stock_and_smart_bomb_awards`,
  proving score words for bullet/mine shells, Landers, Mutants, Swarmers,
  Baiters, UFOs, Bombers, rescue values, and Pods; actual `P250` / `P500`
  human score-sprite mutations; actual `PRBKIL` Pod score/count mutation;
  replay extra-life and smart-bomb award mutation; and smart-bomb mini-swarmer
  kill score, inventory, and swarmer-count mutation. Validation passed with
  `cargo fmt --check`,
  `cargo test score_regression_fixture_covers_enemy_human_stock_and_smart_bomb_awards
  --all-targets`, `markdownlint PLAN.md`, and `git diff --check`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777932258187679`
- `2026-05-04 23:04:51 BST` Started `DC-11.6`: running focused verification
  for the DC-11 enemy, shell, collision, score, smart-bomb, and trace-style
  characterization fixtures before closing the cycle.
- `2026-05-04 23:06:03 BST` Completed `DC-11.6`: verified the completed
  enemy subsystem coverage with `cargo test fixture_covers --all-targets`
  (16 passed), `cargo test collision --all-targets` (20 passed),
  `cargo test shell --all-targets` (23 passed), and
  `cargo test trace_ --all-targets` (65 passed, 5 known ignored fidelity
  tests). `make trace-fixtures` passed with the documented local skip because
  `docs/fidelity/fixtures/local/rust-current` is absent. Final checks passed
  with `cargo fmt --check`, `markdownlint PLAN.md`, and `git diff --check`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777932385273839`

## Phase 5: Native Video And Presentation

### DC-12: Verified Cabinet Video

Status: `complete`

Goal: replace the live scaffold renderer with verified native cabinet frames.

Steps:

- [x] DC-12.1 Drive native visible frames from red-label video RAM and palette
  RAM after scanline scheduling is trace-proven.
  Completed: `2026-05-04 23:10:59 BST`
- [x] DC-12.2 Implement HUD, score fields, lives, smart-bomb icons, scanner,
  text, game-over, high-score, initials, and attract screens from source.
  Completed: `2026-05-04 23:12:31 BST`
- [x] DC-12.3 Add pixel checksum and perceptual diff fixtures for boot, attract,
  start, gameplay, death, high-score, and operator frames.
  Completed: `2026-05-04 23:23:09 BST`
- [x] DC-12.4 Switch live mode from `render_scaffold` to `render_cabinet_frame`
  only after fixture proof.
  Completed: `2026-05-04 23:24:16 BST`
- [x] DC-12.5 Remove or regenerate archived prototype visual assets from
  red-label source/ROM-derived data.
  Completed: `2026-05-04 23:28:02 BST`

Completion gate: verified native frames are the only live cabinet image source,
and temporary visuals are outside the verified frame.

Work log:

- `2026-05-04 23:09:44 BST` Started `DC-12.1`: verifying that live native
  cabinet frames are driven from red-label video RAM and palette RAM after the
  scanline-sensitive frame scheduling fixtures already added in Phase 3.
- `2026-05-04 23:10:59 BST` Completed `DC-12.1`: verified the existing native
  frame path with the scanline-sensitive live IRQ video-frame fixtures, the
  PCRAM-to-palette-RAM copy fixture, the direct red-label visible RGBA fixture,
  and the live renderer handoff from machine video RAM into the cabinet-frame
  scaler. Validation passed with `cargo test live_irq_video_frame --all-targets`,
  `cargo test render_live_machine_frame --all-targets`, and
  `cargo test red_label_visible_rgba_image_uses_video_ram_and_color_mapping_copy
  --all-targets`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777932684700969`
- `2026-05-04 23:11:49 BST` Started `DC-12.2`: auditing the translated
  source presentation routines for HUD, score fields, lives, smart-bomb icons,
  scanner, text, game-over, high-score, initials, and attract screens before
  fixture-locking the video outputs.
- `2026-05-04 23:12:31 BST` Completed `DC-12.2`: verified the translated
  presentation surfaces already present in `src/machine.rs`: `TDISP` HUD,
  player score transfers, lives and smart-bomb icons, scanner raster/text,
  source message glyphs, game-over handoff, high-score tables, initials entry,
  attract logo/credits/screens, and operator diagnostics/audits/reset surfaces.
  The remaining risk is MAME golden-frame equivalence, which stays with
  `DC-12.3`. Validation passed with `cargo test top_display --all-targets`,
  `cargo test scanner --all-targets`, `cargo test high_score --all-targets`,
  `cargo test attract --all-targets`, `cargo test game_over --all-targets`,
  `cargo test operator_flow_fixture_covers_live_diagnostics_audits_and_reset_events
  --all-targets`, and `cargo test score_current_player --all-targets`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777932776615679`
- `2026-05-04 23:13:23 BST` Started `DC-12.3`: adding deterministic native
  video checksum and coarse visual-signature fixtures for boot, attract, start,
  gameplay, death, high-score, and operator frames without checking external
  ROM or MAME golden captures into the repository.
- `2026-05-04 23:23:09 BST` Completed `DC-12.3`: added source-native video
  fixture signatures in `src/machine.rs` for boot, attract, start, gameplay,
  death, and high-score frames, plus a `src/board.rs` operator/AUDITG fixture.
  Each fixture locks exact native pixel data with CRC-32 and a coarse
  visible-shape signature with nonzero-pixel bounds so later refactors can
  prove the same frame output without storing external captures. Updated
  `SPEC.md` and `docs/fidelity/gaps.md` to record that these source-native
  fixtures exist while MAME-derived golden pixel proof remains open. Validation
  passed with `cargo test native_video --all-targets`, `cargo fmt --check`,
  `cargo clippy --all-targets -- -D warnings`,
  `markdownlint PLAN.md SPEC.md docs/fidelity/gaps.md`, and
  `git diff --check`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777933418941519`
- `2026-05-04 23:24:01 BST` Started `DC-12.4`: verifying the live presentation
  path now stays on `render_cabinet_frame` after the `DC-12.3` fixture proof
  and has no remaining `render_scaffold` live fallback.
- `2026-05-04 23:24:16 BST` Completed `DC-12.4`: verified `src/live.rs`
  renders the live machine frame by copying PCRAM into palette RAM, building a
  native red-label RGBA frame, and handing it to `renderer.render_cabinet_frame`.
  Repository search found no source `render_scaffold` symbol or live fallback;
  the only remaining `render_scaffold` text is this plan's historical step
  wording. Validation passed with `cargo test render_live_frame --all-targets`,
  `cargo test render_live_machine_frame --all-targets`, `rg -n
  "render_scaffold|render_cabinet_frame|render_live_frame|render_live_machine_frame"
  src README.md SPEC.md docs/fidelity/gaps.md PLAN.md`, `markdownlint PLAN.md`,
  and `git diff --check`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777933481608959`
- `2026-05-04 23:25:02 BST` Started `DC-12.5`: removing the remaining runtime
  embedding of archived prototype visual assets and documenting archived visual
  files as reference-only material outside the verified native cabinet frame.
- `2026-05-04 23:28:02 BST` Completed `DC-12.5`: removed the
  `ARCADE_LOGO_PAGE_PNG` runtime embedding from `src/assets.rs`, removed the
  now-unused test-only PNG decoder and embedded-logo decode test from
  `src/video.rs`, and updated `README.md`, `SPEC.md`, and
  `assets/arcade/README.md` to state that `assets/arcade/*.png` files are
  archived prototype references only and not runtime presentation inputs.
  Validation passed with `cargo test
  embedded_red_label_assets_have_expected_headers --all-targets`,
  `cargo test video::tests --all-targets`, `cargo fmt --check`,
  `cargo clippy --all-targets -- -D warnings`, the asset-embed `rg` check
  returning no matches, `markdownlint
  PLAN.md SPEC.md README.md assets/arcade/README.md docs/fidelity/gaps.md`,
  `git diff --check`, and the final `make fidelity` DC-12 gate.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777933840717529`
- `2026-05-04 23:30:01 BST` Completed `DC-12` gate: `make fidelity` passed
  after the verified-cabinet-video work, including `cargo fmt --check`,
  all-target Rust tests, coverage generation, and the new-Rust-coverage check.
  The coverage run reported 837 passed, 13 known ignored, and 2 main tests
  passed.

## Phase 6: Sound Board And Audio

### DC-13: Source-Shaped Sound Board

Status: `complete`

Goal: make audio come from source-shaped sound-board execution rather than
semantic gameplay cues.

Steps:

- [x] DC-13.1 Translate the `VSNDRM1.SRC` IRQ dispatch path and Defender sound
  routines needed by the main board.
  Completed: `2026-05-04 23:34:26 BST`
- [x] DC-13.2 Model sound CPU timing, command latch consumption, DAC writes, and
  sample generation.
  Completed: `2026-05-04 23:41:27 BST`
- [x] DC-13.3 Add command-sequence fixtures from red-label traces.
  Completed: `2026-05-04 23:43:58 BST`
- [x] DC-13.4 Add waveform tests with deterministic tolerance.
  Completed: `2026-05-04 23:47:34 BST`
- [x] DC-13.5 Keep `--mute` as an output-layer mixer switch only.
  Completed: `2026-05-04 23:50:46 BST`

Completion gate: sound command traces and generated audio pass fixtures, and no
gameplay code triggers high-level named cues directly.

Work log:

- `2026-05-04 23:34:10 BST` Started `DC-13.1`: verifying the translated
  `VSNDRM1.SRC` IRQ dispatch, command-flow branches, and source-visible sound
  routine mutations already present in `src/sound.rs` before closing the step.
- `2026-05-04 23:34:26 BST` Completed `DC-13.1`: verified the existing
  `VSNDRM1.SRC` IRQ dispatch path, command latch prelude, normal GWAVE/VARI
  branches, special routine branches, organ continuation gate, IRQ3 background
  handoff, NMI diagnostic branch, and source-visible RAM/DAC mutations already
  translated in `src/sound.rs`. Validation passed with
  `cargo test vsnd_irq --all-targets`, `cargo test vsnd_nmi --all-targets`,
  and `cargo test vsnd_organ --all-targets`. Broader `make fidelity` is
  deferred until the full `DC-13` cycle because this step changed only
  `PLAN.md`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777934086750959`
- `2026-05-04 23:36:23 BST` Started `DC-13.2`: adding source-visible sound CPU
  timing/sample reporting around IRQ cycles and tightening DAC latch updates
  for generated GWAVE samples so command consumption, DAC writes, and sample
  order survive later refactors.
- `2026-05-04 23:41:27 BST` Completed `DC-13.2`: added source-visible timed
  IRQ cycle reporting in `src/sound.rs`, including monotonic DAC-write ticks,
  generated sample windows, final DAC state, and post-cycle IRQ state. GWAVE
  periods now update the board DAC latch through the same sample emission path
  as the other translated waveform routines. `SPEC.md` now records the new
  timed IRQ/DAC behavior while keeping exact 6808-cycle sample spacing and
  independent sound CPU IRQ cadence as fidelity gaps. Validation passed with
  `markdownlint PLAN.md SPEC.md`, `cargo fmt --check`,
  `cargo test vsnd_irq_timed_cycle --all-targets`,
  `cargo test vsnd_gwave --all-targets`, `cargo test vsnd_irq --all-targets`,
  and `cargo clippy --all-targets -- -D warnings`. Broader `make fidelity` is
  deferred until the full `DC-13` cycle closes.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777934506876679`
- `2026-05-04 23:42:09 BST` Started `DC-13.3`: verifying the embedded
  red-label sound command-sequence fixtures and validators, then adding any
  missing fixture guard needed to lock command traces for later refactors.
- `2026-05-04 23:43:58 BST` Completed `DC-13.3`: verified the existing
  embedded sound command fixtures for table, direct, thrust, and timeline
  command rows, then added an asset-level guard that every MAME-observed
  trace-required sound command in `trace-requirements.tsv` is present in the
  command-sequence fixtures. Updated the stale ignored sound-equivalence test
  reason to reflect that command fixtures now exist and only waveform/end-to-end
  sound trace proof remains. Validation passed with `markdownlint PLAN.md
  SPEC.md assets/red-label/README.md`, `cargo fmt --check`,
  `cargo test red_label_trace_required_sound_commands_are_fixture_backed
  --all-targets`, and `cargo test sound_ --all-targets`. Broader
  `make fidelity` is deferred until the full `DC-13` cycle closes.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777934661712669`
- `2026-05-04 23:44:43 BST` Started `DC-13.4`: adding deterministic waveform
  signature coverage for representative translated sound-board generators so
  later refactors can compare generated buffers and source-visible mutations.
- `2026-05-04 23:47:34 BST` Completed `DC-13.4`: added a deterministic
  generated-waveform signature matrix covering GWAVE, VARI, LITE, TURBO,
  CANNON, RADIO, HYPER, SCREAM, and ORGAN note buffers. The signatures lock
  sample counts, sample CRC-32 values, first/last samples, final DAC latch
  values, and direct-page RAM mutation CRCs; the deterministic tolerance is
  currently exact byte-for-byte matching. `SPEC.md` now records that in-repo
  waveform signatures exist while external waveform golden fixtures remain a
  fidelity gap. Validation passed with `markdownlint PLAN.md SPEC.md`,
  `cargo fmt --check`,
  `cargo test vsnd_waveform_signature_matrix_locks_deterministic_dac_buffers
  --all-targets`, `cargo test vsnd_ --all-targets`, and
  `cargo clippy --all-targets -- -D warnings`. Broader `make fidelity` is
  deferred until the full `DC-13` cycle closes.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777934878763819`
- `2026-05-04 23:48:18 BST` Started `DC-13.5`: verifying `--mute` remains an
  output-layer switch and adding regression coverage that muted/unmuted CLI
  configuration does not change core trace sound commands or sound-board state.
- `2026-05-04 23:50:46 BST` Completed `DC-13.5`: added parser coverage proving
  `--mute` changes only the live `play_audio` output flag, plus live-core
  coverage proving muted and unmuted output modes share the same core
  sound-board latch state after cold-boot stepping. `SPEC.md` now states that
  mute must not alter frame stepping, sound-command generation, sound-board
  latch state, or trace output. Validation passed with `markdownlint PLAN.md
  SPEC.md`, `cargo fmt --check`,
  `cargo test parse_args_mute_changes_only_live_audio_output_flag --all-targets`,
  `cargo test live_core_sound_state_is_independent_of_audio_output_mode
  --all-targets`, `rg -n "play_audio" src`, and
  `cargo clippy --all-targets -- -D warnings`. Broader `make fidelity` is
  deferred until the full `DC-13` cycle closes.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777935066683769`
- `2026-05-04 23:51:31 BST` Started final `DC-13` completion gate: running the
  broad fidelity validation and checking for uncited high-level sound cue
  dispatch before marking the sound-board dev cycle complete.
- `2026-05-04 23:53:43 BST` Completed final `DC-13` completion gate:
  `make fidelity` passed, including Rust unit tests, clippy, trace script
  self-tests, trace tooling unit tests, missing-local-fixture skip handling,
  LLVM coverage generation, and the new Rust coverage gate. The local fixture
  directory `docs/fidelity/fixtures/local/rust-current` was absent, so the
  fidelity fixture check reported its documented skip. The high-level cue
  search found only replay/sound-command fixture tests and output-layer
  `play_audio` wiring in `src/app.rs` and `src/live.rs`; no gameplay code
  triggers named semantic sound cues directly.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777935267641179`

## Phase 7: Compatibility Features

### DC-14: `xyzzy` And Input Profiles

Status: `complete`

Goal: finish supported non-arcade compatibility features without polluting the
arcade core.

Steps:

- [x] DC-14.1 Keep Planetoid and cabinet mappings in input-profile code only.
  Completed: `2026-05-04 23:56:39 BST`
- [x] DC-14.2 Add every future `xyzzy` hook as an explicit overlay hook.
  Completed: `2026-05-04 23:59:18 BST`
- [x] DC-14.3 For each hook, add paired tests proving arcade behavior is
  unchanged when `xyzzy` is disabled and compatibility behavior applies when it
  is enabled.
  Completed: `2026-05-05 00:01:48 BST`
- [x] DC-14.4 Add trace checks that `xyzzy` disabled remains red-label
  equivalent.
  Completed: `2026-05-05 00:04:21 BST`
- [x] DC-14.5 Document compatibility behavior without presenting it as arcade
  behavior.
  Completed: `2026-05-05 00:05:36 BST`

Completion gate: compatibility behavior is isolated, documented, and covered by
paired tests.

Work log:

- `2026-05-04 23:56:03 BST` Started `DC-14.1`: auditing the live input
  profile and overlay paths, then adding a regression guard that Planetoid and
  cabinet key mappings remain isolated to the input-profile layer.
- `2026-05-04 23:56:39 BST` Completed `DC-14.1`: added a focused
  `src/input.rs` regression proving Planetoid and cabinet profile differences
  stop at cabinet-action and typed-character projection, with `xyzzy` overlay
  state left separate. Validation passed with `cargo fmt --check`,
  `cargo test input_profiles_only_map_keys_to_cabinet_actions --all-targets`,
  and the source search for key/profile references outside `src/input.rs`.
  Broader `make fidelity` is deferred until the full `DC-14` cycle closes.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777935421675149`
- `2026-05-04 23:57:27 BST` Started `DC-14.2`: replacing direct gameplay
  checks of raw `xyzzy` flags with explicit named overlay hooks for current
  behavior and the documented future compatibility hooks.
- `2026-05-04 23:59:18 BST` Completed `DC-14.2`: added
  `XyzzyOverlayHook` names for the implemented auto-fire, unlimited smart-bomb,
  and invincibility surfaces plus the documented future shot-cap,
  bullet/mine-clear, safe-hyperspace, collision-death, and falling-humanoid
  hooks. Live gameplay now gates auto-fire and unlimited smart bombs through
  `CompatibilityState::overlay_hook` instead of direct raw-flag checks.
  Validation passed with `cargo fmt --check`, `cargo test xyzzy_ --all-targets`,
  the raw-flag source search across `src/machine.rs`, `src/live.rs`, and
  `src/input.rs`, and `cargo clippy --all-targets -- -D warnings`. Broader
  `make fidelity` is deferred until the full `DC-14` cycle closes.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777935581955939`
- `2026-05-05 00:00:21 BST` Started `DC-14.3`: adding paired disabled/enabled
  tests for each currently implemented `xyzzy` hook, with trace and RAM-visible
  assertions where compatibility must not pollute arcade state.
- `2026-05-05 00:01:48 BST` Completed `DC-14.3`: added paired hook tests for
  auto-fire, unlimited smart bombs, and invincibility. The auto-fire pair proves
  the disabled hook does not emit fire while the enabled hook emits
  `FirePressed` without mutating raw cabinet input bits. The smart-bomb pair
  proves empty-inventory arcade behavior stays silent with `xyzzy` off while
  enabled `xyzzy` emits the overlay event without changing the red-label smart
  bomb RAM cell. The invincibility pair proves the current flag is
  trace-invisible and leaves main-board and sound-board snapshots unchanged
  until a future arcade hook is implemented. Validation passed with
  `cargo fmt --check`, the three focused paired-hook tests,
  `cargo test xyzzy_ --all-targets`, and
  `cargo clippy --all-targets -- -D warnings`. Broader `make fidelity` is
  deferred until the full `DC-14` cycle closes.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777935729220189`
- `2026-05-05 00:02:40 BST` Started `DC-14.4`: adding a trace-level
  regression that explicitly disabled `xyzzy` remains byte-for-byte equivalent
  to the default red-label trace path.
- `2026-05-05 00:04:21 BST` Completed `DC-14.4`: added an internal
  compatibility-aware trace generator and a regression that expands a
  coin/start/player-action trace program, then proves an explicitly disabled
  `xyzzy` state with hidden toggles set still matches the default red-label
  trace byte-for-byte. Validation passed with `cargo fmt --check`,
  `cargo test trace_text_with_xyzzy_disabled_is_red_label_equivalent
  --all-targets`, `cargo test trace_text_ --all-targets`, and
  `cargo clippy --all-targets -- -D warnings`. Broader `make fidelity` is
  deferred until the full `DC-14` cycle closes.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777935880819719`
- `2026-05-05 00:05:00 BST` Started `DC-14.5`: updating README and SPEC
  compatibility wording so Planetoid and `xyzzy` remain documented as
  non-arcade overlays and input profiles.
- `2026-05-05 00:05:36 BST` Completed `DC-14.5`: updated `README.md` and
  `SPEC.md` to state that the Planetoid layout is only an input profile and
  that `xyzzy` is a deliberate compatibility overlay, not Williams red-label
  arcade behavior. The docs now name implemented overlay hooks and reserved
  future hooks, and state the paired-test requirement for any future enabled
  hook. Validation passed with `markdownlint PLAN.md README.md SPEC.md` and a
  hook-name drift search across `README.md`, `SPEC.md`, and `src/machine.rs`.
  Broader `make fidelity` is deferred until the full `DC-14` cycle closes.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777935954418139`
- `2026-05-05 00:06:11 BST` Started final `DC-14` completion gate: running
  broad fidelity validation and checking that compatibility behavior remains
  isolated in input/profile, live wiring, documented overlay hooks, and tests.
- `2026-05-05 00:08:29 BST` Completed final `DC-14` completion gate:
  `make fidelity` passed, including formatting, all Rust targets, clippy, trace
  script self-tests, trace tooling unit tests, fixture-check skip handling, LLVM
  coverage generation, and new Rust coverage gating. The local trace fixture
  directory `docs/fidelity/fixtures/local/rust-current` was absent, so the
  fixture check used its documented skip path. The compatibility isolation
  search found profile key handling confined to `src/input.rs`, live `xyzzy`
  state wiring in `src/live.rs`, overlay hook gates and paired tests in
  `src/machine.rs`, disabled-trace coverage in `src/fidelity.rs`, and matching
  compatibility documentation in `README.md` and `SPEC.md`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1777936144932939`

## Phase 8: Fidelity Gap Closure Before Refactor

### DC-15: Trace Oracle Promotion And Gap Recheck

Status: `complete`

Goal: restore a repeatable local Rust-vs-MAME proof loop before new fidelity
closure work resumes.

Steps:

- [x] DC-15.1 Re-run `make reference-fixtures-check` and exact
  `attract_boot` comparison against the local reference fixture.
  Completed: `2026-05-05 19:26:55 BST`
- [x] DC-15.2 Generate or refresh the ignored
  `docs/fidelity/fixtures/local/rust-current` fixture set only for scenarios
  whose current Rust output is intentionally being compared.
  Completed: `2026-05-05 19:36:35 BST`
- [x] DC-15.3 Re-run every ignored `local_reference_*_matches_red_label` test,
  then unignore passing scenarios or narrow ignored reasons to the remaining
  mismatch.
  Completed: `2026-05-05 19:38:27 BST`
- [x] DC-15.4 Record the comparison deltas in
  `docs/fidelity/golden-comparison-results.md`, update gap text in
  `SPEC.md`, and post the result to Slack.
  Completed: `2026-05-05 19:45:06 BST`

Completion gate: the local trace oracle state is current, every ignored
reference test has a specific reason, and the next mismatch boundary is known.

Work log:

- `2026-05-05 19:25:58 BST` Started `DC-15.1`: re-running the local reference
  fixture gate and exact `attract_boot` Rust-vs-reference comparison so the
  trace oracle state is current before any fixture promotion or ignored-test
  review.
- `2026-05-05 19:26:55 BST` Completed `DC-15.1`: `make
  reference-fixtures-check` validated 12 complete local reference fixtures and
  22,308 frames. The exact `attract_boot` comparison failed at line 2, frame 1,
  because the reference still has `video_crc32=-` while current Rust emits
  `0x157E98C7`. A full column-delta check over 900 frames found 900
  `video_crc32` mismatches and the known 168 `process_table_crc32` mismatches
  from frame 733 through frame 900.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778005641744419`
- `2026-05-05 19:27:37 BST` Started `DC-15.2`: generating the ignored
  `docs/fidelity/fixtures/local/rust-current` fixture set from the current Rust
  core for the Phase 1 scenarios, then checking the directory with
  `make trace-fixtures`.
- `2026-05-05 19:36:35 BST` Completed `DC-15.2`: generated ignored
  Rust-current inputs for the Phase 1 scenarios and promoted only the 10
  scenarios whose current Rust trace could be generated and checked:
  `abduction`, `attract_boot`, `death`, `firing`, `first_300_frames`,
  `hyperspace`, `smart_bomb`, `start_game`, `thrust_reverse`, and
  `wave_advance`. `make trace-fixtures` passed for those 10 pairs with 15,452
  frames. `planet_destruction` and `high_score_entry` were excluded from the
  Rust-current fixture set because current trace generation panics at
  `src/machine.rs:26276` with `red-label OFREE object list is empty`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778006215777289`
- `2026-05-05 19:37:11 BST` Started `DC-15.3`: running the ignored
  `local_reference_*_matches_red_label` tests against the local reference
  fixture set, then updating ignored-test reasons to match the current first
  blockers.
- `2026-05-05 19:38:27 BST` Completed `DC-15.3`: ran
  `cargo test local_reference_ --all-targets -- --ignored`; all eight
  local-reference tests failed at line 2, frame 1 because the local reference
  fixtures still have `video_crc32=-` while Rust emits `0x157E98C7`. No test
  was safe to unignore. Updated the ignored-test reasons in `src/app.rs` to
  name the current missing-reference-video-CRC blocker plus each scenario's
  remaining process, credited-start, gameplay, death, or wave drift. Validation
  passed with `cargo fmt --check` and `cargo test local_reference_
  --all-targets`, which reports the eight tests ignored with the narrowed
  `DC-15` reasons.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778006325800959`
- `2026-05-05 19:39:04 BST` Started `DC-15.4`: recording the current
  Rust-vs-reference comparison deltas in
  `docs/fidelity/golden-comparison-results.md`, updating `SPEC.md`, and then
  running the final DC-15 validation gate.
- `2026-05-05 19:45:06 BST` Completed `DC-15.4` and final `DC-15` gate:
  recorded the `DC-15` trace-oracle results in
  `docs/fidelity/golden-comparison-results.md`, updated `SPEC.md` and
  `docs/fidelity/gaps.md`, and completed the cycle with `make fidelity`.
  Validation passed with `markdownlint README.md SPEC.md PLAN.md
  docs/fidelity/gaps.md docs/fidelity/golden-comparison-results.md`,
  `git diff --check`, and `make fidelity`. The broad gate included
  `cargo fmt --check`, `cargo test --all-targets` with 850 passed and 13
  known ignored tests, `cargo clippy --all-targets -- -D warnings`, trace Lua
  and Python self-tests, `make trace-fixtures` against the 10 refreshed
  Rust-current pairs and 15,452 frames, and LLVM coverage/new-code coverage.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778006733565279`

### DC-16: Post-`INIT20` `ATTR` And Executive Cadence

Status: `planned`

Goal: close the current `attract_boot` process-table mismatch at frame 733.

Steps:

- [ ] DC-16.1 Trace frame-733 `ATTR` / `EXEC` process-table mutations from the
  local reference and source listings.
- [ ] DC-16.2 Add source-visible mutation tests for the affected process cells
  and scheduler rows.
- [ ] DC-16.3 Fix the translated scheduler cadence until `attract_boot` matches
  the local reference through frame 900.
- [ ] DC-16.4 Promote the passing `attract_boot` local-reference test by
  unignoring it, or narrow any remaining ignored reason to the next exact
  mismatch.
- [ ] DC-16.5 Add a live attract-flow smoke or trace check proving the app
  advances beyond the initial Williams/`DEFENDER` screen into later attract,
  credit, and start-ready states.

Completion gate: `local_reference_attract_boot_matches_red_label` passes
locally with the reference fixture, or the next mismatch is documented with
frame, column, expected value, and actual value. Live attract flow also
progresses beyond the initial Williams/`DEFENDER` screen.

### DC-17: Full Frame And CPU/IRQ Ownership

Status: `planned`

Goal: replace frame-model shortcuts with source-shaped board, CPU, IRQ, and
scanline ownership wherever current proof still depends on scaffolding.

Steps:

- [ ] DC-17.1 Model the missing CPU IRQ scheduling and register context needed
  by translated dispatch paths, including A/X/Y/S/CC/B where traces or source
  evidence prove the values.
- [ ] DC-17.2 Integrate main IRQ, scanline/video counter, watchdog,
  palette/rendering side effects, and sound IRQ ownership into frame stepping.
- [ ] DC-17.3 Replace trace-only scheduling shortcuts with source-shaped
  execution for reset, boot, attract, and gameplay frame boundaries.
- [ ] DC-17.4 Add trace columns or mutation tests only where they expose
  source-visible state needed for equivalence.

Completion gate: frame stepping has explicit ownership for CPU, IRQ, video,
watchdog, palette, and sound-board handoff behavior, with characterization tests
locking every new mutation surface.

### DC-18: Gameplay Golden Trace Closure

Status: `planned`

Goal: make the credited-start and active-player scenarios match local
MAME/source references.

Steps:

- [ ] DC-18.1 Recompare `start_game`, `firing`, `thrust_reverse`,
  `smart_bomb`, and `hyperspace` after `DC-16` and `DC-17`.
- [ ] DC-18.2 Close credited-start transition timing, player setup, and RNG
  call-order drift.
- [ ] DC-18.3 Close post-start object/process scheduler drift for firing,
  thrust, reverse, smart bomb, and hyperspace paths.
- [ ] DC-18.4 Unignore passing gameplay local-reference tests and update
  `SPEC.md` / `docs/fidelity/gaps.md` with remaining exact mismatches.

Completion gate: the focused start and player-action traces either pass exact
local-reference comparison or have only newly documented, narrower blockers.

### DC-19: Death, Wave, Session, And Operator Trace Closure

Status: `planned`

Goal: prove longer gameplay, session, and cabinet/service paths against local
references.

Steps:

- [ ] DC-19.1 Close `death` and `wave_advance` scheduler, RNG, phase, lives,
  smart-bomb, object-table, and process-table drift.
- [ ] DC-19.2 Add or refresh local reference scenarios for two-player sessions,
  high-score entry, operator/service screens, and cabinet input profiles.
- [ ] DC-19.3 Verify high-score, audit, operator, coin, credit, and game-over
  mutations with before/after tests and exact traces.
- [ ] DC-19.4 Confirm `xyzzy` disabled stays red-label equivalent and enabled
  behavior differs only through documented overlay hooks.

Completion gate: session/operator traces are source-equivalent or each
remaining delta is explicitly recorded in the gap register.

### DC-20: MAME Pixel Golden Fixtures

Status: `planned`

Goal: prove native video output against MAME-derived pixel fixtures rather than
only source-native CRCs.

Steps:

- [ ] DC-20.1 Add local MAME frame capture tooling and an ignored fixture
  convention for pixel golden frames.
- [ ] DC-20.2 Compare boot, title/`DEFENDER` logo, attract, credited start,
  gameplay, death, high-score, and operator/service frames.
- [ ] DC-20.3 Fix the corrupted title/`DEFENDER` wordmark graphic and replace
  remaining native-black untranslated screens, or record an owner-approved
  out-of-scope decision for each screen.
- [ ] DC-20.4 Keep renderer tests focused on stable source-visible pixels,
  palette indices, and video RAM mutations.

Completion gate: representative MAME-derived pixel fixtures pass, and any
remaining non-rendered screens are explicitly scoped.

### DC-21: Sound-Board Cycle And Waveform Golden Fixtures

Status: `planned`

Goal: close sound fidelity beyond command dispatch and deterministic in-repo
waveform signatures.

Steps:

- [ ] DC-21.1 Integrate source-shaped 6808 cycle/IRQ cadence and DAC scheduling
  for live sound-board execution.
- [ ] DC-21.2 Generate and check local MAME command-sequence fixtures for
  trace-required sound commands.
- [ ] DC-21.3 Generate and check external waveform golden fixtures for
  representative red-label sound routines.
- [ ] DC-21.4 Finish remaining `VSNDRM1` waveform routine translations and
  unignore or narrow the sound equivalence tests.

Completion gate: command timing and representative DAC output are proven
against local MAME/source fixtures, with remaining audio gaps narrowed to named
routines.

### DC-22: Hardware Edge Cases, ROM Assets, And Scaffold Removal

Status: `planned`

Goal: close remaining hardware and asset fidelity gaps before the large
refactor begins.

Steps:

- [ ] DC-22.1 Model exact Williams power-on RAM contents, physical advance
  switch timing, physical lamp timing, watchdog reset side effects, and decoder
  PROM behavior where source or MAME evidence is available.
- [ ] DC-22.2 Complete full ROM/bank memory integration and regenerate any
  source/ROM-derived assets needed by runtime paths.
- [ ] DC-22.3 Close remaining collision callbacks and no-process `SWTAB` input
  effects.
- [ ] DC-22.4 Remove obsolete scaffold or archived prototype runtime
  dependencies only after replacement tests prove exact behavior.

Completion gate: no known pre-refactor fidelity gap remains without a passing
fixture, a focused mutation test, or a recorded owner-approved scope decision.

## Phase 9: Planned Large Refactor

### DC-23: Refactor Freeze And API Shape

Status: `planned`

Goal: prepare for the large refactor only after behavior is well characterized.

Steps:

- [ ] DC-23.1 Freeze a full characterization suite covering traces, mutation
  tests, pixel fixtures, audio fixtures, and live/session edge cases.
- [ ] DC-23.2 Define the public arcade API: `new`, `reset`, `step`,
  `snapshot`, and `restore`.
- [ ] DC-23.3 Identify module boundaries for CPU/board, scheduler, memory,
  video, sound, input/session, compatibility, and assets.
- [ ] DC-23.4 Document every behavior that must remain byte-for-byte compatible
  during the refactor.

Completion gate: the refactor has a test-backed API contract and no major
untested mutation surfaces.

### DC-24: Module Split And Behavior Preservation

Status: `planned`

Goal: split the large core safely without changing behavior.

Steps:

- [ ] DC-24.1 Move code in small ownership-based slices with no unrelated
  rewrites.
- [ ] DC-24.2 Run the full characterization gate after every slice.
- [ ] DC-24.3 Keep public surfaces narrow and source-cited.
- [ ] DC-24.4 Remove dead scaffold paths only after tests prove exact
  replacements.
- [ ] DC-24.5 Update docs after each module boundary becomes stable.

Completion gate: the refactored code produces the same traces, byte mutations,
pixel fixtures, audio fixtures, and live/session behavior as the pre-refactor
implementation.

## Phase 10: Release Readiness

### DC-25: Final Acceptance And Documentation

Status: `planned`

Goal: prepare the project as a complete exact red-label implementation with
supported compatibility overlays.

Steps:

- [ ] DC-25.1 Run `make fidelity`, local reference fixtures, pixel fixtures,
  audio fixtures, and any CI/Sonar gates.
- [ ] DC-25.2 Perform live terminal smoke tests in Kitty-compatible terminals
  and record rendering, input, sound, and quit behavior.
- [ ] DC-25.3 Update `README.md`, `SPEC.md`, `docs/fidelity/`, and install/run
  documentation to match final behavior.
- [ ] DC-25.4 Confirm no runtime dependency on local ROMs or generated fixture
  files.
- [ ] DC-25.5 Confirm `xyzzy` disabled is red-label equivalent and `xyzzy`
  enabled differs only through documented overlay hooks.
- [ ] DC-25.6 Close remaining gap entries or explicitly mark them out of scope.

Completion gate: every acceptance item in `SPEC.md` is satisfied or explicitly
documented as out of scope by project owner decision.
