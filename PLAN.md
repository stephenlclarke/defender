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

Status: `in_progress`

Goal: make red-label process execution the normal core path.

Steps:

- [x] DC-06.1 Add register-aware process entry/resume state where source
  routines require A/B/X/Y/U/S/CC context.
  Completed: `2026-05-04 19:27:41 BST`
- [x] DC-06.2 Complete `PLRES` swarmer respawn once the entry B register value
  is trace-proven.
  Completed: `2026-05-04 20:13:37 BST`
- [ ] DC-06.3 Finish suicide, resume, delay, super-process, and generic body
  dispatch semantics.
- [ ] DC-06.4 Verify object/process order against golden traces.
- [ ] DC-06.5 Remove scaffold state that duplicates exact red-label table
  fields.

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
