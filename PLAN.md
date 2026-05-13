# Defender Current Plan

Last reviewed: `2026-05-13`

## Current Baseline

- Active branch: `rewrite`.
- Latest accepted implementation commit before this cycle: `05fdeb2`.
- Phase 13 is complete. The converted implementation has been moved to
  `src_legacy/`; the clean rewrite now owns the primary `src/` tree while
  preserving legacy access through the doc-hidden `compatibility` namespace.
  Root legacy adapters should stay crate-private.
- Live play uses the `wgpu` backend. Kitty is parked in `src_legacy/` as
  historical compatibility evidence and is no longer an active runtime path.
- The next product direction is a `wgpu`-only clean game rewrite. Kitty should
  be removed from the active application surface, and the current
  assembler-shaped machine should become a temporary fidelity oracle rather
  than the production gameplay model.
- Normal runtime is self-contained. ROM files are optional verification inputs.
- No active behavior gap is tracked in this plan. New fidelity gaps should be
  documented in `docs/fidelity/gaps.md` and linked from this file only when
  they become active work.

## Current Validation Gate

Use this gate for behavior, architecture, or release-facing changes:

```sh
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
```

For docs-only changes:

```sh
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md
git diff --check
```

`make fidelity` covers formatting, all Rust targets, clippy, Lua trace exporter
self-tests, Python helper tests, local Rust-current trace fixtures, coverage,
and added executable Rust line coverage.

## Work Protocol

- Keep `README.md`, `SPEC.md`, and `PLAN.md` aligned with the current code.
- Use focused tests for material code changes.
- Preserve source-visible mutation checks for arcade-core behavior.
- Preserve gameplay behavior while the rewrite is underway. `XYZZY` and
  Planetoid controls remain compatibility features unless explicitly removed.
- Kitty removal is now intentional rewrite scope; do not add new terminal
  renderer abstractions.
- Use Conventional Commits for committed work.
- Do not use `codex` in branch names, commit messages, or PR titles.
- Post a start note to `xyzzytools.slack.com#codex` before beginning each
  planned dev-cycle, and post a completion note after validation when the
  dev-cycle closes.

## Rewrite Target

Rewrite the application as a modern `wgpu` game while preserving accepted
gameplay behavior. The production source should describe game concepts instead
of ROM labels, assembler routines, or memory-table implementation details.

Target module ownership:

- `game`: pure gameplay domain types such as `Game`, `World`, `Player`,
  `Enemy`, `Projectile`, `Human`, `Terrain`, `Score`, `Wave`, and
  `HighScore`.
- `systems`: deterministic fixed-step systems for input, player movement,
  projectiles, enemies, collisions, waves, scoring, high-score entry, attract
  mode, and sound-event emission.
- `renderer`: native `wgpu` rendering from scene data using sprite/quad
  pipelines, texture atlases, uniform buffers, instanced draws, HUD/text
  rendering, debug overlays, and viewport scaling.
- `platform`: `winit` event loop, input collection, fixed timestep, persistence,
  smoke runner, and device lifecycle.
- `audio`: gameplay-facing sound events and backend/runtime ownership.
- `fidelity`: legacy oracle, golden fixtures, trace tooling, and any remaining
  ROM/source terminology needed to prove behavioral equivalence.

Rewrite rules:

- Keep the current machine/memory implementation available as an oracle until
  the clean model proves equivalent for accepted scenarios.
- Production modules must not expose `red_label`, ROM file labels, assembler
  routine names, source process names, or memory table names.
- Renderer code must consume clean scene data, not RAM bytes or source layout
  fields.
- Replace memory CRC confidence gradually with clean state, event, sound, and
  rendered-frame equivalence gates.
- Prefer small deterministic systems over a monolithic memory-oriented machine.
- Use `wgpu` directly as the renderer, not as a final framebuffer presenter for
  a hidden memory model once the clean scene path exists.

## Completed Development Cycles

`DC-42` through `DC-63` are complete. `DC-64` is planned, and the standing
maintenance guidance in Ongoing Work still applies.

### DC-42: Documentation Reset

Status: `complete`

Goal: remove stale historical plan/spec/readme material and leave only current,
useful project information.

Scope:

- Rewrite `README.md` as the clean public entry point: badges, screenshot,
  animated GIF, install/run commands, controls, persistence, compatibility
  overlay, development commands, architecture, assets, platform notes, and
  references.
- Reduce `SPEC.md` to the current behavior contract, source-of-truth rules,
  architecture, validation gates, and active constraints.
- Reduce `PLAN.md` to current baseline, validation, work protocol, this active
  docs cycle, and immediate next work.

Work log:

- `2026-05-10 14:42:08 BST` Started `DC-42`: replacing stale completed phase
  history and drift inventory with current project documentation for the
  refactored `red-label-refactor` baseline.
- `2026-05-10 14:44:36 BST` Completed `DC-42`: reduced `PLAN.md` to the
  active baseline, validation gate, work protocol, docs cycle, and next useful
  work; reduced `SPEC.md` to the current source-of-truth contract,
  architecture, validation, and active constraints; rewrote `README.md` as the
  current public entry point with badges, screenshot, animated GIF, commands,
  controls, persistence, compatibility overlay, development targets,
  architecture, assets, platform notes, and references. Validation passed with
  `markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md`,
  `git diff --check`, and `cargo run -- --help`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778420701682049`

### DC-43: Threaded Fidelity Fixture Runner

Status: `complete`

Goal: continue the refactor by replacing serial fixture orchestration with a
small trait-based runner that can execute independent Rust fidelity fixture
checks on scoped worker threads while preserving manifest ordering and
first-error reporting.

Scope:

- Add a `TraceFixtureChecker` trait to separate fixture discovery/order from
  fixture execution.
- Run fixture checks in ordered chunks across available scoped worker threads.
- Preserve existing `--fidelity-check-trace-dir` output and first-error
  behavior.
- Add focused tests for parallel result aggregation, ordered error reporting,
  and worker panic handling.

Work log:

- `2026-05-12` Completed `DC-43`: `src/app.rs` now checks fidelity fixture
  pairs through a trait-based checker on scoped threads. Validation passed with
  `cargo fmt --check`, targeted `app::tests::check_trace_fixtures`,
  targeted `app::tests::fidelity_check_trace_dir_text`, and
  `cargo clippy --all-targets -- -D warnings`, `cargo test --all-targets`,
  `make trace-script-test`, `make trace-fixtures`,
  `make coverage NEW_CODE_COVERAGE_BASE=origin/main`, `markdownlint README.md
  SPEC.md PLAN.md docs/fidelity/refactor-freeze.md`, and `git diff --check`.

### DC-44: Shared Live Core Driver

Status: `complete`

Goal: continue carving the live runtime into explicit, reusable boundaries
before a later presentation/core thread split.

Scope:

- Add a `LiveCoreDriver` that owns the live input mapper, XYZZY overlay,
  machine, timing clock, pending pulse input, and pending typed characters.
- Reuse the driver from both Kitty and `wgpu` live presentation paths.
- Preserve pulse buffering, held-input catch-up frames, typed-character
  high-score entry behavior, and XYZZY overlay behavior.
- Add focused tests for driver-owned held input, overlay state, and realtime
  pulse buffering.

Work log:

- `2026-05-12` Completed `DC-44`: Kitty and `wgpu` now advance the arcade core
  through the shared `LiveCoreDriver` instead of duplicating input/clock/overlay
  logic. Validation passed with `cargo fmt --check`, `cargo check`, targeted
  `live::tests::live_core_driver`, targeted `wgpu_presenter::tests::wgpu_smoke`,
  `cargo test --all-targets`, and `cargo clippy --all-targets -- -D warnings`.

### DC-45: Threaded Live Core Runtime

Status: `complete`

Goal: move the `wgpu` live path onto an explicit presentation/core thread
boundary without changing gameplay behavior.

Scope:

- Add a `LiveCoreRuntime` trait and `LiveCoreThread` worker that owns
  `LiveCoreDriver`, live `Renderer`, pending input, and CMOS access.
- Move `wgpu` input, resize, advance, render, and CMOS-save calls through the
  worker command protocol.
- Preserve realtime and deterministic smoke-frame stepping, pulse buffering,
  held input, typed characters, XYZZY overlay state, and live smoke reporting.
- Add focused tests for threaded input/overlay advancement, realtime pulse
  buffering, renderer resize, and CMOS snapshots.

Work log:

- `2026-05-12` Completed `DC-45`: `wgpu` presentation now draws the latest
  `LiveCoreFrame` returned from a dedicated live core worker thread instead of
  owning the arcade machine directly. Validation passed with
  `cargo fmt --check`, `cargo check`, targeted
  `live::tests::live_core_thread`, targeted
  `wgpu_presenter::tests::wgpu_smoke`, and
  `cargo clippy --all-targets -- -D warnings`, `cargo test --all-targets`,
  `cargo run -- --live-smoke`, `make trace-script-test`,
  `make trace-fixtures`, `make coverage NEW_CODE_COVERAGE_BASE=origin/main`,
  `markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md`,
  and `git diff --check`.

## Completed Follow-On Cycles

These ordered cycles are complete. If a future behavior gap appears, document
the gap in `docs/fidelity/gaps.md` and add a source-backed fixture or
characterization test before implementation.

### DC-46: Kitty Runtime Unification

Status: `complete`

Goal: put the Kitty compatibility backend behind the same live runtime
boundary as `wgpu`, so presentation code no longer owns arcade-core state in
either live path.

Scope:

- Route Kitty input, frame advancement, rendering, resize handling, sleep
  timing, and CMOS save/load through `LiveCoreRuntime`.
- Keep the existing Kitty graphics protocol, terminal-session handling, and
  terminal geometry code unchanged except for the runtime call sites.
- Preserve pulse buffering, held-input catch-up frames, typed-character high
  score entry, XYZZY overlay behavior, and CMOS persistence.
- Add tests around any extracted Kitty/runtime adapter seams rather than
  requiring an interactive terminal for coverage.

Acceptance criteria:

- `run_kitty_live` does not directly own or mutate `ArcadeMachine`,
  `LiveCoreDriver`, `InputMapper`, `XyzzyOverlay`, or `Renderer`.
- Kitty and `wgpu` share the same runtime command contract for input, resize,
  advance/render, and CMOS access.
- Existing Kitty renderer tests still cover double buffering, clear behavior,
  resize behavior, and environment gating.

Validation:

```sh
cargo fmt --check
cargo test --lib live::tests::live_core
cargo test --lib kitty::tests
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

Work log:

- `2026-05-12` Completed `DC-46`: Kitty now uses `LiveCoreThread` for input,
  resize, advance/render, and CMOS snapshots, matching the `wgpu` live runtime
  command boundary while keeping the existing Kitty graphics and terminal
  session code in the presentation path. Focused adapter coverage was added for
  terminal geometry updates, and the shared CMOS-save helper is used by both
  live backends. Validation passed with `cargo fmt --check`, `cargo check`,
  targeted `live::tests::live_core_thread`, targeted
  `live::tests::terminal_geometry_update_reports_runtime_and_kitty_sizes`,
  targeted `kitty::tests`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test --all-targets`, `cargo run -- --live-smoke`,
  `make trace-script-test`, `make trace-fixtures`, and
  `make coverage NEW_CODE_COVERAGE_BASE=origin/main`, `markdownlint README.md
  SPEC.md PLAN.md docs/fidelity/refactor-freeze.md`, and `git diff --check`.

### DC-47: Non-Blocking Wgpu Frame Pipeline

Status: `complete`

Goal: separate `wgpu` event-loop scheduling from core frame production so the
window thread draws the latest completed frame instead of synchronously waiting
for every arcade-core advance/render command.

Scope:

- Introduce a small latest-frame mailbox or bounded channel between the live
  core worker and `WgpuLiveApp`.
- Keep input and resize commands ordered relative to core advancement.
- Preserve deterministic `--live-smoke` behavior by keeping smoke mode on an
  explicit fixed-frame cadence with observable frame counts.
- Keep normal realtime mode tied to `FRAME_RATE_MILLIHZ` deadlines from the
  core runtime.
- Add tests for latest-frame replacement, stale-frame dropping, resize
  ordering, clean shutdown, and smoke determinism.

Acceptance criteria:

- `WgpuLiveApp::draw_frame` can render the latest available frame without
  blocking on an arcade-core step.
- Normal mode does not busy-spin when no frame is due.
- Smoke reports still include window creation, rendered frame count, distinct
  frame CRCs, attract/credit/playing evidence, injected inputs, and clean exit.
- Worker shutdown cannot leave the event loop waiting forever on a channel
  receive.

Validation:

```sh
cargo fmt --check
cargo test --lib live::tests::live_core
cargo test --lib wgpu_presenter::tests::wgpu_smoke
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo run -- --live-smoke
```

Work log:

- `2026-05-12` Completed `DC-47`: `LiveCoreThread` now supports a
  replacement latest-frame mailbox and non-blocking frame requests, while
  keeping the synchronous advance path for Kitty. `WgpuLiveApp` requests at
  most one core frame at a time, drains completed frames without blocking the
  redraw path, schedules normal mode from the worker-owned frame deadline, and
  keeps smoke mode on explicit fixed-frame requests. Focused coverage was
  added for mailbox replacement, stale-frame dropping, resize ordering, async
  fixed-frame determinism, and in-flight shutdown joining. Validation passed
  with `cargo fmt --check`, targeted `live::tests::live_core`, targeted
  `wgpu_presenter::tests::wgpu_smoke`, clippy with warnings denied,
  `cargo test --all-targets`, `cargo run -- --live-smoke`, `make fidelity`,
  Markdown lint, and the whitespace diff check.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778624044098319`

### DC-48: Runtime Lifecycle And Error Contracts

Status: `complete`

Goal: make live runtime startup, shutdown, worker panic reporting, and CMOS
persistence explicit and testable before adding more long-running runtime
features.

Scope:

- Replace stringly worker failures with a small runtime error type or structured
  context that distinguishes command send failures, worker termination, render
  failures, and worker panics.
- Ensure live workers shut down and join predictably from normal exit, smoke
  exit, window close, suspended windows, and error paths.
- Confirm CMOS save uses the final core-owned CMOS state after live runtime
  shutdown.
- Add tests for worker drop, failed command response, render error propagation,
  and CMOS snapshot retrieval after gameplay mutations.

Acceptance criteria:

- Runtime errors include enough context to identify the failed command.
- Dropping the runtime cannot leak a worker thread.
- `run_wgpu_live` and `run_wgpu_live_smoke` preserve their public error
  behavior while using the structured runtime errors internally.
- CMOS save remains best-effort only where it already was; no new gameplay
  behavior depends on persistence.

Validation:

```sh
cargo fmt --check
cargo test --lib live::tests::live_core_thread
cargo test --lib wgpu_presenter::tests
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

Work log:

- `2026-05-12` Completed `DC-48`: live runtime failures now report structured
  command-scoped errors for command send failures, worker termination, worker
  panics, mailbox failures, and render failures. Live shutdown now joins the
  worker and returns the final core-owned CMOS snapshot before persistence, so
  Kitty and `wgpu` both save after runtime shutdown. Focused coverage was added
  for drop/join behavior, failed command context, worker panic context,
  sync/async render error propagation, and CMOS mutation persistence.
  Validation passed with `cargo fmt --check`, targeted
  `live::tests::live_core_thread`, targeted `wgpu_presenter::tests`, clippy
  with warnings denied, `cargo test --all-targets`, and
  `cargo run -- --live-smoke`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778624792340579`

### DC-49: Public Arcade API Narrowing

Status: `complete`

Goal: reduce the public `machine::...` compatibility surface without breaking
existing internal callers or hiding source-shaped contracts that tests still
need.

Scope:

- Inventory current `machine::...` re-exports and classify them as public API,
  test-only support, internal implementation detail, or source fixture
  contract.
- Move internal-only imports to their owning modules where practical.
- Keep compatibility re-exports temporarily when the migration would otherwise
  create broad churn.
- Add compile-time caller checks or focused tests for any moved public surface.
- Update `README.md` and `SPEC.md` only if the user-facing API changes.

Acceptance criteria:

- No behavior code changes are mixed into this cycle.
- Existing external-facing commands and live behavior are unchanged.
- Any removed or moved symbol has a documented replacement path or is proven
  private to the crate.

Validation:

```sh
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make coverage NEW_CODE_COVERAGE_BASE=origin/main
```

Work log:

- `2026-05-12` Completed `DC-49`: `machine_state` and `machine_process` are
  now public canonical contract modules, while the existing `machine::...`
  re-exports remain as compatibility aliases. Internal callers in fidelity,
  live runtime, and `wgpu` presentation now import state/process contracts from
  the owning modules where practical. Compile-time API checks cover both direct
  and compatibility paths, and README/SPEC now document the canonical paths.
  Validation passed with `cargo fmt --check`, `cargo test --all-targets`,
  `cargo clippy --all-targets -- -D warnings`, and
  `make coverage NEW_CODE_COVERAGE_BASE=origin/main`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778626931639039`

### DC-50: Machine Module Ergonomics Pass

Status: `complete`

Goal: keep shrinking assembler-shaped bulk into readable Rust modules while
preserving source-visible behavior and fixture coverage.

Scope:

- Pick one high-cohesion area at a time: scoring/high-score flow, object list
  mutations, shell/projectile handling, terrain/world flow, or operator/audit
  flow.
- Extract small typed data structures or helper traits only when they remove
  real duplication or clarify a source contract.
- Keep source routine names visible in tests and error messages where they are
  part of fidelity evidence.
- Avoid broad rewrites of `machine.rs` and `machine_memory.rs`; each pass must
  have a narrow ownership boundary and focused tests.

Acceptance criteria:

- The selected module area has less duplicated address/list manipulation or
  less incidental register plumbing than before the cycle.
- Source-visible mutations remain covered by existing or new tests.
- Public behavior, trace output, live play, and fixture checks are unchanged.

Validation:

```sh
cargo fmt --check
cargo test --lib machine::tests::<focused_filter>
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make trace-fixtures
make coverage NEW_CODE_COVERAGE_BASE=origin/main
```

Work log:

- `2026-05-13` Completed `DC-50`: the high-score game-over dispatch path and
  test-only one-player game setup now reuse the live high-score session reset
  helper instead of duplicating entry/submission/player-mask cleanup. Focused
  regression coverage now asserts `GameOverSleeping` clears the entry,
  submission, active-entry player, completed-player mask, and phase state.
  Validation passed with `cargo fmt --check`, targeted
  `machine::tests::game_over_sleeping_dispatch_clears_live_high_score_session`,
  `cargo test --all-targets`, `cargo clippy --all-targets -- -D warnings`,
  `make trace-fixtures`, and
  `make coverage NEW_CODE_COVERAGE_BASE=origin/main`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778628102721249`

### DC-51: Live Audio Acceptance Design

Status: `complete`

Goal: prepare live audio output with source-backed acceptance criteria before
implementing any audio device or mixer code.

Scope:

- Define which sound-board surface is authoritative for live audio:
  per-frame sound commands, sound-board state snapshots, DAC byte signatures,
  or a combination.
- Add a documented live-audio design note that covers cadence, buffering,
  sample rate, worker-thread ownership, and how audio interacts with pause,
  window suspend, smoke mode, and CMOS persistence.
- Add or extend fixtures that prove command timing for coin, start, thrust,
  smart bomb, hyperspace, explosion, terrain blow, and high-score paths.
- Do not add audible output in this cycle.

Acceptance criteria:

- `docs/fidelity/gaps.md` has no unresolved live-audio behavior question that
  would block implementation.
- The plan identifies the exact fixture or test that will fail if audio timing
  drifts.
- Runtime threading ownership for audio is specified before implementation.

Validation:

```sh
cargo fmt --check
cargo test --all-targets
make trace-script-test
make trace-fixtures
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md
```

Work log:

- `2026-05-13` Completed `DC-51`: added the source-backed live audio
  acceptance design in `docs/fidelity/live-audio.md`, added
  `assets/red-label/live-audio-acceptance.tsv`, and wired the matrix into
  embedded-asset tests so each accepted path is backed by existing command or
  sound-table fixtures. README, SPEC, fidelity README, gap notes, and the
  refactor freeze inventory now point to the accepted timing, diagnostic, and
  content-guard surfaces before runtime audio implementation begins.
  Validation passed with `cargo fmt --check`, `cargo test --all-targets`,
  `make trace-script-test`, `make trace-fixtures`, and `markdownlint README.md
  SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/README.md
  docs/fidelity/gaps.md docs/fidelity/live-audio.md`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778628605842599`

### DC-52: Live Audio Runtime Prototype

Status: `complete`

Goal: add the first live audio runtime path only after `DC-51` establishes the
source-backed acceptance contract.

Scope:

- Add an audio backend behind a trait so live audio can be disabled, mocked, or
  swapped without changing the arcade core.
- Feed audio from the accepted sound-board surface without changing trace
  output or machine stepping cadence.
- Keep audio commands on a runtime-owned thread or channel boundary that does
  not block `wgpu` redraw or core stepping.
- Add a CLI/config path only if the default behavior and platform failure modes
  are clear.

Acceptance criteria:

- Audio can be disabled for tests and unsupported platforms.
- `--live-smoke` remains deterministic and does not require an audio device.
- Sound command fixtures and DAC signature tests still pass unchanged unless
  `DC-51` explicitly updated their accepted contract.

Validation:

```sh
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
```

Work log:

- `2026-05-13` Completed `DC-52`: added `src/audio.rs` with a live audio
  backend trait, disabled mode, no-device null backend, bounded non-blocking
  command queue, drop accounting, flush/shutdown handling, and focused tests.
  Live core stepping now copies accepted `FrameOutput::sound_commands()`
  batches to the audio runtime without changing arcade-core ownership, trace
  output, or step cadence. Normal live play uses the null backend, `--mute`
  disables the runtime path, and `--live-smoke` uses a disabled no-device path.
  README, SPEC, fidelity docs, gap notes, and refactor-freeze ownership notes
  now describe the prototype boundary and the remaining audible-device work.
  Validation passed with `cargo fmt --check`, `cargo test --all-targets`,
  `cargo clippy --all-targets -- -D warnings`, `make fidelity`,
  `cargo run -- --live-smoke`, `markdownlint README.md SPEC.md PLAN.md
  docs/fidelity/refactor-freeze.md docs/fidelity/README.md
  docs/fidelity/gaps.md docs/fidelity/live-audio.md`, and `git diff --check`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778630482569359`

### DC-53: Release And CI Hardening

Status: `complete`

Goal: make the refactored runtime cheaper to maintain by tightening local and
GitHub validation around the expensive fidelity, smoke, and coverage gates.

Scope:

- Review CI runtime after the threaded fixture runner and live runtime changes.
- Keep `make ci`, `make fidelity`, Sonar, coverage baseline, and local docs in
  sync.
- Add targeted CI diagnostics for `wgpu` smoke failures, coverage baseline
  drift, missing Lua/Mesa tools, and Slack update failures.
- Keep generated artifacts out of git unless the project explicitly accepts
  them as source fixtures or documentation media.

Acceptance criteria:

- A CI failure points at the failed subsystem instead of requiring full log
  archaeology.
- Coverage baseline refresh remains an intentional command, not an implicit
  side effect.
- README development commands match the Makefile and workflows.

Validation:

```sh
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md
git diff --check
```

Work log:

- `2026-05-13` Completed `DC-53`: added Makefile doctor targets for trace,
  coverage, and `wgpu` smoke prerequisites; split GitHub CI into explicit
  prerequisite diagnostics, fidelity, and `xvfb` smoke steps; added Sonar
  coverage diagnostics; and updated README/SPEC development guidance so local
  commands, workflow steps, and the intentional coverage-baseline refresh path
  stay aligned. Validation passed with `make trace-doctor`,
  `make coverage-doctor`, `make trace-script-test`, `make fidelity`,
  `cargo run -- --live-smoke`, `markdownlint README.md SPEC.md PLAN.md
  docs/fidelity/README.md docs/fidelity/gaps.md
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md`, and
  `git diff --check`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778631713574199`

## Planned Rewrite Cycles

These cycles replace the completed refactor track with a clean `wgpu`-only
rewrite track. Keep each cycle narrow enough to finish with focused tests and
the full fidelity gate before moving on.

### DC-54: Wgpu-Only Rewrite Foundation

Status: `complete`

Goal: establish the clean rewrite boundary without changing gameplay behavior.

Scope:

- Remove Kitty from the active app surface: CLI renderer selection, runtime
  routing, docs, active tests, and CI expectations.
- Make `wgpu` the only supported live runtime and `--live-smoke` path.
- Introduce `game`, `systems`, `renderer`, and `platform` module shells with
  clean public contracts.
- Move the converted `src/` tree to `src_legacy/` and make the clean rewrite
  the primary `src/` tree.
- Move or wrap the current assembler-shaped implementation behind an explicit
  `fidelity::oracle` or `legacy` boundary.
- Narrow `src/lib.rs` public exports to the intended clean API plus temporary
  oracle/test surfaces.
- Replace fragile new-code coverage baseline matching with a line-and-context
  or source-hash keyed baseline.
- Convert CLI parsing to typed command parsing so mixed commands and flags are
  rejected predictably.

Acceptance criteria:

- `cargo run` and `cargo run -- --live-smoke` use `wgpu` without backend
  selection.
- No active production path depends on Kitty or terminal graphics.
- The current gameplay model is still available to tests as an oracle.
- Public crate exports distinguish clean API from temporary oracle internals.
- Coverage baseline entries cannot accidentally accept unrelated repeated
  source lines.

Validation:

```sh
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md
git diff --check
```

Work log:

- `2026-05-13` Started `DC-54` on branch `rewrite`: posted the cycle start
  update, moved the converted implementation to `src_legacy/`, promoted the
  clean rewrite modules into `src/`, and kept the legacy implementation wired
  as the temporary oracle/compatibility runtime.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778657320848069`
- `2026-05-13` Continued `DC-54`: removed active Kitty renderer selection from
  CLI parsing, help text, Make targets, and live runtime routing. `cargo run`
  and `cargo run -- --live-smoke` now use the `wgpu` path without backend
  selection.
- `2026-05-13` Completed `DC-54`: moved the old implementation under
  `src_legacy/`, made the clean rewrite tree the primary `src/`, kept the
  legacy machine available as an oracle/compatibility runtime, strengthened the
  new-code coverage baseline to line-and-source-hash matching, and refreshed
  the baseline to zero accepted uncovered additions. Validation passed with
  `make fidelity`, `cargo run -- --live-smoke`, `markdownlint README.md
  SPEC.md PLAN.md docs/fidelity/refactor-freeze.md
  docs/fidelity/live-audio.md`, `git diff --check`, and
  `cargo run --quiet -- --help`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778660955629679`

### DC-55: Clean Simulation Contracts

Status: `complete`

Goal: define the domain-first frame API that future gameplay systems will own.

Scope:

- Add clean `GameState`, `GameInput`, `GameFrame`, `GameEvents`, and
  `RenderScene` contracts.
- Add an oracle adapter that converts current machine output into the clean
  contracts for comparison.
- Add fixtures that compare clean frame events and scene summaries against the
  current accepted behavior.
- Keep all red-label/source terminology inside the oracle adapter and fidelity
  tests.

Acceptance criteria:

- The `wgpu` runtime can advance through the clean frame API even while the
  oracle still produces the underlying behavior.
- New gameplay code can be written against clean contracts without importing
  machine memory modules.
- Accepted trace scenarios have clean event/scene comparison coverage.

Validation:

```sh
cargo fmt --check
cargo test --lib game
cargo test --lib systems
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
```

Work log:

- `2026-05-13` Started `DC-55` on branch `rewrite`: posted the cycle start
  update and began extending the clean `GameState`/`GameFrame`/`GameEvents`,
  `RenderScene`, and simulation trait contracts while keeping the converted
  implementation behind the oracle boundary.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778661176512439`
- `2026-05-13` Completed `DC-55`: `GameFrame` now carries clean
  `GameState`, `GameEvents`, sound events, and a `RenderScene`; renderer
  scenes expose stable summaries and layer counts; `systems` exposes a
  `GameSimulation` trait for clean frame advancement; and the gameplay oracle
  implements that trait while converting accepted machine output into clean
  state, event, sound, and scene-summary frames. Clean fixture coverage now
  compares credited-start events and scene summaries against the accepted
  oracle behavior. Validation passed with `cargo fmt --check`, focused
  `game`, `systems`, `renderer`, and `oracle` tests, `cargo test
  --all-targets`, `cargo clippy --all-targets -- -D warnings`,
  `make fidelity`, `cargo run -- --live-smoke`, `markdownlint README.md
  SPEC.md PLAN.md docs/fidelity/refactor-freeze.md
  docs/fidelity/live-audio.md`, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778664173986969`

### DC-56: Native Wgpu Scene Renderer

Status: `complete`

Goal: replace framebuffer presentation with a native `wgpu` scene renderer
while keeping visual behavior equivalent.

Scope:

- Build renderer-owned pipelines for terrain/starfield, sprites, projectiles,
  explosions, HUD/text, and debug overlays.
- Introduce texture atlas and palette/font resources owned by the renderer.
- Render from `RenderScene` instead of machine RAM.
- Keep a temporary framebuffer comparison path for golden visual fixtures.

Acceptance criteria:

- Live play and smoke render through native `wgpu` scene data.
- Golden visual evidence still catches behavioral or visual drift.
- Renderer modules do not import machine memory or oracle-specific types.

Validation:

```sh
cargo fmt --check
cargo test --lib renderer
cargo test --lib wgpu_presenter::tests::wgpu_smoke
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
```

Work log:

- `2026-05-13` Started `DC-56` on branch `rewrite`: posted the cycle start
  update and began replacing direct frame upload with clean renderer scene data.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778664443957909`
- `2026-05-13` Completed `DC-56`: `RenderScene` now supports a validated
  temporary raster payload for visual equivalence, renderer-owned atlas,
  palette, font, native pipeline, and draw-plan resources live in the clean
  renderer module, and the live `wgpu` path uploads scene raster data instead
  of drawing directly from `RenderedImage`. Smoke visual evidence now derives
  from scene metrics while the temporary raster path keeps golden visual drift
  detectable. Validation passed with `cargo fmt --check`, focused
  `renderer`, `wgpu_smoke`, and live scene tests, `cargo test --all-targets`,
  `cargo clippy --all-targets -- -D warnings`, `make fidelity`, and
  `cargo run -- --live-smoke`; the live smoke reported 240 rendered frames,
  74 distinct scene CRCs, attract/credit/playing evidence, all required
  injected inputs, and clean exit.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778667397382589`

### DC-57: Player Control System Migration

Status: `complete`

Goal: begin migrating gameplay from memory-oriented routines to clean
deterministic systems with the first input/player-control slice.

Scope:

- Add a clean player-control system that separates held movement/fire intent
  from two-clear-sample action triggers.
- Preserve vertical-control priority and input history behavior without exposing
  RAM-layout fields or assembler routine names in production code.
- Compare the clean control history against the oracle before replacing live
  gameplay paths.
- Keep the remaining gameplay migration sequence explicit for follow-on cycles.

Acceptance criteria:

- The migrated player-control slice has clean domain tests and oracle
  equivalence tests.
- Production player-control code no longer reads or writes RAM-layout fields.
- Behavior, trace output, live smoke, and accepted visual/audio evidence remain
  stable.

Validation:

```sh
cargo fmt --check
cargo test --lib systems
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
```

Work log:

- `2026-05-13` Started `DC-57` on branch `rewrite`: posted the cycle start
  update and began migrating player input/control behavior into clean
  deterministic systems while keeping accepted behavior behind the oracle.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778667591192949`
- `2026-05-13` Completed `DC-57`: added clean `PlayerControlIntent`,
  `PlayerActionTriggers`, `PlayerControlFrame`, and `PlayerControlSystem`
  contracts for held control intent and two-clear-sample action triggers;
  exported the contracts through the clean public API; and added both clean
  domain tests and oracle switch-scan equivalence coverage. `README.md` and
  `SPEC.md` now describe the clean player-control system, and `DC-58` now
  carries the next player-motion/projectile migration slice. Validation passed
  with `cargo fmt --check`, `cargo test --lib systems`, `cargo test
  --all-targets`, `cargo clippy --all-targets -- -D warnings`,
  `make fidelity`, `cargo run -- --live-smoke`, `markdownlint README.md
  SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md`,
  and `git diff --check`; live smoke rendered 239 frames with 74 distinct scene
  CRCs, attract/credit/playing evidence, all required injected inputs, and a
  clean exit.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778669085679879`

### DC-58: Player Motion And Projectile Systems

Status: `complete`

Goal: continue gameplay migration by moving player motion and projectile
launch behavior into clean deterministic systems.

Scope:

- Drive player motion from `PlayerControlIntent` while preserving accepted
  damping, thrust, vertical priority, bounds, and scroll behavior.
- Add clean projectile launch/capacity state and compare fire entry timing
  against the oracle.
- Keep update order explicit for controls, motion, projectiles, collision, and
  rendering scene emission.
- Remove assembler-derived names from newly migrated production code.

Acceptance criteria:

- Player motion and projectile launch/capacity slices have clean domain tests
  and oracle equivalence tests.
- Production player-motion and projectile modules do not read or write
  RAM-layout fields.
- Behavior, trace output, live smoke, and accepted visual evidence remain
  stable unless an intentional difference is documented.

Validation:

```sh
cargo fmt --check
cargo test --lib systems
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
```

Work log:

- `2026-05-13` Started `DC-58` on branch `rewrite`: posted the cycle start
  update and began moving player motion plus projectile launch/capacity
  behavior into clean deterministic systems while keeping accepted behavior
  behind the oracle.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778669202479739`
- `2026-05-13` Completed `DC-58`: added clean `ScreenPosition`,
  `PlayerMotionState`, `PlayerMotionFrame`, `PlayerMotionSystem`,
  `ProjectileState`, `ProjectileLaunchOutcome`, and `ProjectileSystem`
  contracts; exported them through the clean public API; and added focused
  systems tests plus oracle equivalence for accepted player motion and laser
  fire entry behavior. Validation passed with `cargo fmt --check`, `cargo test
  --lib systems`, `cargo test --lib oracle`, `cargo test --all-targets`,
  `cargo clippy --all-targets -- -D warnings`, `make fidelity`, `cargo run --
  --live-smoke`, `markdownlint README.md SPEC.md PLAN.md
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md`, and
  `git diff --check`; the coverage gate reported 196/196 non-baselined added
  executable Rust lines covered, and live smoke rendered 239 frames with 74
  distinct scene CRCs, attract/credit/playing evidence, all required injected
  inputs, and a clean exit.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778672203264889`

### DC-59: Audio Device And Event Model

Status: `complete`

Goal: replace command-byte delivery with gameplay-facing sound events and a
diagnosable audio runtime.

Scope:

- Map accepted command timing to clean `SoundEvent` values.
- Add structured audio worker errors, backend lifecycle reporting, and smoke
  diagnostics.
- Add an audible backend only after null/disabled/event equivalence is stable.
- Keep sound fixture timing as the oracle for migrated sound events.

Acceptance criteria:

- Gameplay systems emit semantic sound events.
- Audio worker failure is visible in tests and diagnostics.
- `--live-smoke` stays deterministic and device-independent.

Validation:

```sh
cargo fmt --check
cargo test --lib audio
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
```

Work log:

- `2026-05-13` Started `DC-59` on branch `rewrite`: posted the cycle start
  update and began moving live audio from raw command-batch delivery to
  gameplay-facing sound events while keeping accepted frame-output timing as
  the oracle boundary.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778672338922919`
- `2026-05-13` Completed `DC-59`: added active clean `src/audio.rs` event
  batches, semantic `SoundEvent` mapping for accepted startup, credit, start,
  and thrust cues, structured audio shutdown diagnostics, backend lifecycle
  and sample-rate reporting, queue drop stats, and worker panic visibility.
  The legacy live core now feeds `SoundEvent` batches through the clean runtime,
  and documentation now describes event delivery with `FrameOutput` retained
  as the timing adapter. Validation passed with `cargo fmt --check`, `cargo
  test --lib audio`, `cargo test --all-targets`, `cargo clippy --all-targets
  -- -D warnings`, `make fidelity`, `cargo run -- --live-smoke`,
  `markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md
  docs/fidelity/live-audio.md`, and `git diff --check`; the coverage gate
  reported 17/17 non-baselined added executable Rust lines covered, and live
  smoke rendered 239 frames with 74 distinct scene CRCs, attract/credit/playing
  evidence, all required injected inputs, and a clean exit.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778675086403679`

### DC-60: Compatibility Surface Quarantine

Status: `complete`

Goal: begin oracle retirement by hiding legacy compatibility modules from the
supported clean public API documentation while current runtime and fidelity
tooling still depend on them.

Scope:

- Mark all `src_legacy/` path modules in `src/lib.rs` as doc-hidden
  compatibility modules.
- Add a focused architecture test that fails if a legacy path module is exposed
  without the compatibility quarantine marker.
- Keep the binary, README media tooling, oracle tests, and fidelity gates
  working without changing gameplay behavior.
- Document that compatibility modules remain wired but are not the supported
  clean API surface.

Acceptance criteria:

- Supported docs expose clean modules first and legacy modules are explicitly
  hidden.
- Tests guard against adding new unhidden legacy path modules.
- Public clean contracts and existing compatibility runtime remain intact.

Validation:

```sh
cargo fmt --check
cargo test --lib public_api_tests::legacy_compatibility_modules_are_hidden_from_supported_api_docs
cargo check --all-targets
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-13` Started `DC-60` on branch `rewrite`: posted the cycle start
  update and began the first oracle-retirement step by auditing legacy module
  exposure from the clean crate root.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778675232663129`
- `2026-05-13` Completed `DC-60`: marked every `src_legacy/` path module in
  `src/lib.rs` as a doc-hidden compatibility module, added
  `public_api_tests::legacy_compatibility_modules_are_hidden_from_supported_api_docs`
  to guard the quarantine marker, and updated `README.md`, `SPEC.md`, and this
  plan to describe the hidden compatibility surface while moving full oracle
  retirement to `DC-61`. Validation passed with the documented DC-60 gate:
  formatting, the focused public API guard, `cargo check --all-targets`, the
  full Rust test suite, clippy with warnings denied, `make fidelity`,
  `cargo run -- --live-smoke`, markdownlint, and `git diff --check`; the
  coverage gate reported 0/0 non-baselined added executable Rust lines, and
  live smoke rendered 240 frames with 74 distinct scene CRCs,
  attract/credit/playing evidence, all required injected inputs, and a clean
  exit.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778676669257919`

### DC-61: Runtime Entrypoint Facade

Status: `complete`

Goal: continue oracle retirement by removing the binary entrypoint's direct
dependency on the doc-hidden legacy `app` module.

Scope:

- Add a clean platform-facing runtime launcher that owns the production
  entrypoint contract.
- Point `src/main.rs` at the clean platform launcher instead of
  `defender::app::run()`.
- Add a focused architecture guard that rejects direct binary calls into the
  legacy `app` module.
- Document that the binary now enters through the clean runtime boundary while
  the compatibility runtime remains the temporary accepted behavior owner.

Acceptance criteria:

- `src/main.rs` depends on `defender::platform::run()`.
- The legacy `app` module remains hidden compatibility plumbing.
- CLI behavior and fidelity gates are unchanged.

Validation:

```sh
cargo fmt --check
cargo test --lib public_api_tests::binary_entrypoint_uses_clean_platform_runtime_boundary
cargo test --lib platform::tests::runtime_entrypoint_delegates_to_compatibility_runtime
cargo check --all-targets
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-13` Started `DC-61` on branch `rewrite`: posted the cycle start
  update and began moving the production binary entrypoint behind a clean
  platform launcher.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778676810486839`
- `2026-05-13` Completed `DC-61`: added `platform::run()` as the clean runtime
  launcher, pointed `src/main.rs` at `defender::platform::run()`, guarded the
  binary entrypoint against direct legacy `app` calls, and documented that the
  binary now enters through the clean platform boundary while the compatibility
  runtime remains the temporary accepted behavior owner. Validation passed with
  the documented DC-61 gate: formatting, the focused public API and platform
  entrypoint tests, `cargo check --all-targets`, the full Rust test suite,
  clippy with warnings denied, `make fidelity`, `cargo run -- --live-smoke`,
  markdownlint, and `git diff --check`; the coverage gate reported 2/2
  non-baselined added executable Rust lines, and live smoke rendered 239 frames
  with 74 distinct scene CRCs, attract/credit/playing evidence, all required
  injected inputs, and a clean exit.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778679271201939`

### DC-62: Compatibility Namespace Facade

Status: `complete`

Goal: make the remaining legacy access explicit by routing clean runtime and
oracle callers through a single doc-hidden compatibility namespace.

Scope:

- Add a `defender::compatibility` facade over the parked `src_legacy/`
  adapters.
- Route `platform` runtime launch and `oracle` accepted-behavior calls through
  the compatibility namespace instead of direct legacy crate-root paths.
- Preserve the existing doc-hidden legacy modules for legacy internals until a
  later deletion pass can retire them safely.
- Add architecture tests that keep clean runtime and oracle callers on the
  compatibility boundary.

Acceptance criteria:

- Clean runtime and oracle modules no longer call `crate::app`,
  `crate::machine_state`, `crate::video`, or related legacy root paths
  directly.
- The compatibility facade re-exports the machine-state and process contracts
  still needed by oracle tests.
- README, SPEC, and PLAN describe compatibility access as a temporary boundary,
  not a clean production dependency.

Validation:

```sh
cargo fmt --check
cargo test --lib public_api_tests::compatibility_namespace
cargo test --lib platform::tests::runtime_entrypoint_delegates_to_compatibility_runtime
cargo check --all-targets
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-13 18:56:46 BST` Started `DC-62`: posted the cycle start update and
  began introducing a compatibility namespace boundary for clean runtime and
  oracle legacy access.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778695006846469`
- `2026-05-13 19:34:22 BST` Completed `DC-62`: added the doc-hidden
  `defender::compatibility` facade, routed `platform` and `oracle` legacy
  calls through that namespace, added architecture tests to keep clean runtime
  and oracle callers off direct legacy crate-root paths, added an oracle phase
  contract test to cover all accepted phase mappings, and updated README,
  SPEC, and PLAN to describe the compatibility boundary and next oracle
  retirement slice. Validation passed with the documented DC-62 gate:
  formatting, focused compatibility and platform tests, `cargo check
  --all-targets`, the full Rust test suite, clippy with warnings denied,
  `make fidelity`, `cargo run -- --live-smoke`, markdownlint, and
  `git diff --check`; the first `make fidelity` run exposed missing new-line
  coverage for two phase mapping arms, which was fixed before rerunning the
  gate successfully. The final coverage gate reported 7/7 non-baselined added
  executable Rust lines, and live smoke rendered 239 frames with 74 distinct
  scene CRCs, attract/credit/playing evidence, all required injected inputs,
  and a clean exit.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778697300468119`

### DC-63: Public Legacy Export Retirement

Status: `complete`

Goal: stop exposing parked legacy modules from the public crate root while
keeping temporary oracle and tooling access behind the doc-hidden compatibility
namespace.

Scope:

- Change `src_legacy/` root adapters in `src/lib.rs` from public modules to
  crate-private modules.
- Rebuild `defender::compatibility` as explicit doc-hidden submodules that
  re-export the public legacy items needed by the oracle and temporary tools.
- Route the README media example through `defender::compatibility` instead of
  direct legacy public crate-root paths.
- Add architecture tests that fail if legacy root adapters become public again.

Acceptance criteria:

- External callers can no longer import `defender::machine`,
  `defender::input`, `defender::video`, or related parked legacy modules from
  the root crate namespace.
- Temporary oracle and README media tooling still compile through the
  compatibility namespace.
- README, SPEC, and PLAN describe root legacy adapters as crate-private.

Validation:

```sh
cargo fmt --check
cargo test --lib public_api_tests::compatibility_namespace
cargo test --lib public_api_tests::legacy_compatibility_modules_are_crate_private_at_root
cargo check --all-targets
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-13 19:38:28 BST` Started `DC-63`: posted the cycle start update and
  began retiring public root exports for the parked legacy adapters while
  preserving temporary oracle and tooling access through the compatibility
  namespace.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778697445868899`
- `2026-05-13 19:58:11 BST` Completed `DC-63`: made parked legacy adapters
  crate-private at the root, rebuilt `defender::compatibility` as explicit
  doc-hidden submodules, routed README media generation through that boundary,
  and updated architecture docs/tests to preserve the split. Validation passed:
  `cargo fmt --check`, targeted public API tests, `cargo check --all-targets`,
  `cargo test --all-targets`, `cargo clippy --all-targets -- -D warnings`,
  `make fidelity`, `cargo run -- --live-smoke`, `markdownlint README.md
  SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md`,
  and `git diff --check`. `make fidelity` reported new Rust line coverage
  `0/0` non-baselined added executable lines. Live smoke rendered 239 frames,
  saw 74 distinct frame CRCs, observed attract, credit, and playing states,
  injected all required controls, and exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778698710074379`

### DC-64: Oracle Retirement

Status: `planned`

Goal: remove the memory-oriented production model after clean systems own
accepted gameplay behavior.

Scope:

- Delete or quarantine obsolete machine memory/session modules from production
  builds.
- Keep only fixture parsers and historical oracle tooling that still provide
  review value.
- Replace memory CRC gates with clean state/event/render/sound equivalence
  gates.
- Ensure no production symbol names expose red-label, ROM, source routine, or
  assembler process terminology.

Acceptance criteria:

- Production gameplay no longer depends on the legacy memory model.
- Fidelity tooling remains available for historical comparison where needed.
- Public API and module names read as a clean game implementation.

Validation:

```sh
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
rg -n 'red_label|RED_LABEL|defend\\.|src/machine_memory|source routine' src
```

## Ongoing Work

- Keep `README.md`, `SPEC.md`, and `PLAN.md` synchronized with CLI help,
  Makefile targets, workflows, and module boundaries.
- Keep added executable Rust lines covered or explicitly refresh the accepted
  uncovered baseline only when accepting existing debt.
- Keep Slack completion notes best-effort until the connector authentication
  is restored; do not treat a Slack token failure as a code or validation
  failure.
