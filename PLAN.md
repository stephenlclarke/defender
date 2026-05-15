# Defender Current Plan

Last reviewed: `2026-05-15`

## Current Baseline

- Active branch: `rewrite`.
- Latest accepted implementation commit before DC-88: `c645d1b`.
- Phase 13 is complete. The converted implementation has been moved to
  `src_legacy/`; the clean rewrite now owns the primary `src/` tree while
  preserving targeted legacy access through doc-hidden tool facades and
  crate-private adapters. Root legacy adapters should stay crate-private.
- Live play uses the `wgpu` backend. Kitty is parked in `src_legacy/` as
  historical compatibility evidence and is no longer an active runtime path.
- Clean `wgpu` rendering should be sprite-first: gameplay visuals should use
  renderer-owned sprite assets, texture atlases, and batched sprite draws.
  Full-frame raster upload remains temporary fidelity evidence, not the final
  gameplay representation.
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
- Slack cycle updates are mandatory for planned dev-cycles: post a start note
  to `xyzzytools.slack.com#codex` before implementation begins, and post a
  completion note after validation when the dev-cycle closes.

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
- `renderer`: native `wgpu` rendering from scene data using sprite assets,
  sprite/quad pipelines, texture atlases, uniform buffers, instanced draws,
  HUD/text rendering, debug overlays, and viewport scaling.
- `platform`: `winit` event loop, input collection, fixed timestep, persistence,
  smoke runner, and device lifecycle.
- `audio`: gameplay-facing sound events and backend/runtime ownership.
- `fidelity`: clean state, event, sound, and render-summary equivalence
  signatures. Legacy oracle traces and any remaining source terminology should
  stay in `src_legacy/` or historical fixture documentation.

Rewrite rules:

- Keep the current machine/memory implementation available as an oracle until
  the clean model proves equivalent for accepted scenarios.
- Production modules must not expose `red_label`, ROM file labels, assembler
  routine names, source process names, or memory table names.
- Renderer code must consume clean scene data, not RAM bytes or source layout
  fields.
- Sprites are the primary gameplay-art primitive. Use renderer-owned sprite
  assets, atlases, and batched draws for the player, enemies, humans,
  projectiles, explosions, terrain details, and UI glyphs instead of treating
  ROM bytemaps or full-frame raster uploads as the production representation.
- Replace memory CRC confidence gradually with clean state, event, sound, and
  rendered-frame equivalence gates.
- Prefer small deterministic systems over a monolithic memory-oriented machine.
- Use `wgpu` directly as the renderer, not as a final framebuffer presenter for
  a hidden memory model once the clean scene path exists.

## Completed Development Cycles

`DC-42` through `DC-112` are complete. The standing maintenance guidance in
Ongoing Work still applies.

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
  frame signatures, attract/credit/playing evidence, injected inputs, and clean
  exit.
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
  74 distinct scene signatures, attract/credit/playing evidence, all required
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
  distinct scene signatures, attract/credit/playing evidence, all required injected
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
  smoke rendered 239 frames with 74 distinct scene signatures, attract/credit/playing
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
  live smoke rendered 240 frames with 74 distinct scene signatures,
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
  with 74 distinct scene signatures, attract/credit/playing evidence, all required
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
  scene signatures, attract/credit/playing evidence, all required injected inputs,
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
  saw 74 distinct frame signatures, observed attract, credit, and playing states,
  injected all required controls, and exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778698710074379`

### DC-64: Accepted-Behavior Facade

Status: `complete`

Goal: isolate production clean runtime and oracle code from direct legacy
module names while preserving the accepted gameplay implementation as the
temporary oracle.

Scope:

- Add a crate-private `src/accepted.rs` facade over the temporary accepted
  implementation.
- Convert accepted machine output into neutral accepted-behavior frame,
  snapshot, phase, direction, event, sound, and visual-signature contracts before
  `src/oracle.rs` adapts them to clean gameplay types.
- Route `src/platform.rs` through the accepted facade instead of calling the
  doc-hidden compatibility runtime directly.
- Keep low-level legacy method access in tests and temporary tooling only.
- Update architecture tests and docs so future clean production callers use
  the accepted facade rather than `defender::compatibility`.

Acceptance criteria:

- `src/oracle.rs` production code imports `crate::accepted::...`, not
  `crate::compatibility::...` or direct legacy root modules.
- `src/platform.rs` dispatches through `crate::accepted::run_runtime()`.
- Focused accepted-facade, oracle, and public API tests pass.
- README, SPEC, and PLAN describe the accepted facade as the current retirement
  boundary.

Validation:

```sh
cargo fmt --check
cargo test --lib accepted::tests
cargo test --lib oracle::tests
cargo test --lib public_api_tests
cargo check --all-targets
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-13 20:01:11 BST` Started `DC-64`: posted the cycle start update and
  began isolating production runtime/oracle access behind a neutral
  accepted-behavior facade while keeping the legacy machine available for
  behavior comparison.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778698871347899`
- `2026-05-13 20:43:21 BST` Completed `DC-64`: added the crate-private
  `src/accepted.rs` facade, routed `platform` and `GameplayOracle` through
  `crate::accepted`, converted accepted machine output into neutral frame,
  snapshot, phase, direction, event, sound-command, and visual-signature
  contracts,
  and updated docs/tests to preserve the boundary. Validation passed with the
  DC-64 gate: formatting, focused accepted/oracle/API tests, all-target
  check/test/clippy, `make fidelity`, live smoke, markdownlint, and
  `git diff --check`. `make fidelity` reported new Rust line coverage `38/38`
  non-baselined added executable lines. Live smoke rendered 239 frames, saw 74
  distinct frame signatures, observed attract, credit, and playing states, injected
  all required controls, and exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778701401764609`

### DC-65: Oracle Equivalence Quarantine

Status: `complete`

Goal: move legacy-specific oracle equivalence checks out of clean oracle source
without weakening the clean-system regression coverage.

Scope:

- Keep `src/oracle.rs` focused on clean gameplay adapter contracts.
- Move low-level accepted-behavior equivalence checks that need legacy process,
  memory, and red-label names into a `src_legacy/` test module.
- Wire the quarantined tests through `src/lib.rs` only for `cfg(test)`.
- Add a public API guard so `src/oracle.rs` does not reintroduce legacy
  terminology while the accepted facade remains temporary.
- Document the test quarantine boundary in README and SPEC.

Acceptance criteria:

- `src/oracle.rs` contains no direct compatibility or red-label terminology.
- The moved equivalence tests still run in the normal library test suite.
- Focused oracle, quarantined equivalence, and public API tests pass.
- README, SPEC, and PLAN describe the new test boundary.

Validation:

```sh
cargo fmt --check
cargo test --lib oracle::tests
cargo test --lib oracle_equivalence_tests
cargo test --lib public_api_tests
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
ORACLE_LEGACY_TERMS='red_label|RED_LABEL|defend\\.|src/machine_memory|source routine|assembler|compatibility'
! rg -n "$ORACLE_LEGACY_TERMS" src/oracle.rs
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-13 21:03:12 BST` Started `DC-65`: posted the cycle start update and
  began quarantining legacy-heavy oracle equivalence checks out of
  `src/oracle.rs` while preserving the clean-system regression coverage.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778702592948259`
- `2026-05-13 21:28:13 BST` Completed `DC-65`: moved the low-level legacy
  oracle equivalence checks into `src_legacy/oracle_equivalence_tests.rs`, left
  `src/oracle.rs` with clean accepted/gameplay adapter tests, added a public
  API guard against legacy terminology in `src/oracle.rs`, and documented the
  test quarantine boundary. Validation passed with the DC-65 gate: focused
  oracle/equivalence/API tests, all-target tests, clippy, `make fidelity`, live
  smoke, oracle terminology search, markdownlint, and `git diff --check`.
  `make fidelity` reported new Rust line coverage `0/0` non-baselined added
  executable lines. Live smoke rendered 239 frames, saw 74 distinct frame signatures,
  observed attract, credit, and playing states, injected all required controls,
  and exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778704093033349`

### DC-66: Accepted Adapter Quarantine

Status: `complete`

Goal: move the legacy-importing accepted-machine adapter out of the clean
accepted-behavior facade while preserving the current temporary oracle.

Scope:

- Keep `src/accepted.rs` focused on neutral accepted frame, snapshot, phase,
  direction, event, sound, and runtime delegation contracts.
- Move accepted-machine adaptation, clean-input-to-cabinet-input projection,
  legacy snapshot conversion, legacy event conversion, and runtime/video-size
  bridge calls into `src_legacy/accepted_behavior.rs`.
- Keep `src/oracle.rs` and `src/platform.rs` on the crate-private
  `crate::accepted` facade.
- Keep low-level equivalence tests in `src_legacy/` and point their
  test-only helpers at the legacy adapter.
- Add a public API guard so `src/accepted.rs` does not reintroduce direct
  compatibility or legacy root imports.
- Document the accepted adapter quarantine in README and SPEC.

Acceptance criteria:

- `src/accepted.rs` contains no direct compatibility, legacy root module, or
  red-label imports.
- Accepted facade, accepted adapter, oracle, and public API focused tests pass.
- Normal runtime and oracle behavior remains unchanged.
- README, SPEC, and PLAN describe the accepted adapter boundary.

Validation:

```sh
cargo fmt --check
cargo test --lib accepted::tests
cargo test --lib accepted_behavior::tests
cargo test --lib oracle::tests
cargo test --lib oracle_equivalence_tests
cargo test --lib public_api_tests
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
ACCEPTED_LEGACY_TERMS='compatibility::|red_label|RED_LABEL|crate::input'
ACCEPTED_LEGACY_TERMS="$ACCEPTED_LEGACY_TERMS|crate::machine|crate::machine_state"
ACCEPTED_LEGACY_TERMS="$ACCEPTED_LEGACY_TERMS|crate::video|crate::app"
! rg -n "$ACCEPTED_LEGACY_TERMS" src/accepted.rs src/oracle.rs
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-13 21:30:32 BST` Started `DC-66`: posted the cycle start update and
  began moving the legacy-importing accepted-machine adapter out of
  `src/accepted.rs` into `src_legacy/accepted_behavior.rs` while preserving
  current oracle behavior.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778704232856249`
- `2026-05-13 21:53:54 BST` Completed `DC-66`: moved the legacy-importing
  accepted-machine adapter into `src_legacy/accepted_behavior.rs`, kept
  `src/accepted.rs` as neutral accepted-behavior contracts plus delegation,
  added a public API guard against direct legacy imports in `src/accepted.rs`,
  and updated the legacy equivalence tests to use the quarantined adapter.
  Validation passed with the DC-66 gate: focused accepted/adapter/oracle/API
  tests, all-target tests, clippy, `make fidelity`, live smoke, clean
  accepted/oracle terminology search, markdownlint, and `git diff --check`.
  `make fidelity` reported new Rust line coverage `5/5` non-baselined added
  executable lines. Live smoke rendered 239 frames, saw 74 distinct frame signatures,
  observed attract, credit, and playing states, injected all required controls,
  and exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778705634783139`

### DC-67: Compatibility Namespace Quarantine

Status: `complete`

Goal: keep temporary oracle and tool access intact while moving the
compatibility re-export details out of the clean crate root.

Scope:

- Add `src_legacy/compatibility.rs` as the owner of the doc-hidden
  `defender::compatibility` re-export namespace.
- Keep `src/lib.rs` focused on clean public exports, crate-private legacy path
  adapters, and the one doc-hidden compatibility path declaration.
- Preserve README media tooling and legacy equivalence tests that still use
  the temporary compatibility namespace.
- Add a public API guard that fails if the compatibility re-export map moves
  back into `src/lib.rs`.

Acceptance criteria:

- `defender::compatibility` behavior is unchanged for current temporary users.
- `src/lib.rs` no longer contains inline compatibility re-export details.
- README, SPEC, and PLAN document the compatibility namespace ownership.

Validation:

```sh
cargo fmt --check
cargo test --lib public_api_tests
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-13 21:57:04 BST` Started `DC-67`: posted the cycle start update and
  began moving the doc-hidden compatibility re-export details from clean
  `src/lib.rs` to `src_legacy/compatibility.rs` while preserving temporary
  tooling and oracle access.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778705824813039`
- `2026-05-13 22:18:12 BST` Completed `DC-67`: moved
  `defender::compatibility` re-export ownership to
  `src_legacy/compatibility.rs`, left `src/lib.rs` with only the doc-hidden
  path declaration, added a public API guard against moving the re-export map
  back into the clean crate root, and updated README/SPEC/PLAN to document the
  ownership boundary. Validation passed with the DC-67 gate: formatting,
  focused public API tests, all-target tests, clippy, `make fidelity`, live
  smoke, markdownlint, and `git diff --check`. `make fidelity` reported new
  Rust line coverage `0/0` non-baselined added executable lines. Live smoke
  rendered 239 frames, saw 74 distinct frame signatures, observed attract, credit,
  and playing states, injected all required controls, and exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778707109325719`

### DC-68: Terminal Session Retirement

Status: `complete`

Goal: remove parked terminal-session code from active crate wiring after Kitty
left the runtime surface.

Scope:

- Move the remaining `TerminalGeometry` value type into the legacy video
  renderer that still consumes it.
- Remove the `src_legacy/terminal.rs` path module from active `src/lib.rs`
  wiring.
- Remove `defender::compatibility::terminal` from temporary compatibility
  re-exports.
- Keep `src_legacy/terminal.rs` parked as historical Kitty terminal-session
  evidence, but leave it unwired from production builds.
- Add public API guards that fail if terminal-session code is rewired through
  the clean crate root or compatibility namespace.

Acceptance criteria:

- Current `wgpu` live behavior and fidelity tooling remain unchanged.
- No active root module or compatibility namespace exposes terminal-session
  setup.
- README, SPEC, and PLAN describe the parked terminal-session boundary.

Validation:

```sh
cargo fmt --check
cargo test --lib public_api_tests
cargo test --lib video::tests::raster_size_uses_terminal_pixels_when_available
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-13 22:21:20 BST` Started `DC-68`: posted the cycle start update and
  began removing legacy terminal-session code from active clean crate wiring
  while keeping the legacy video renderer's geometry value type local to that
  renderer.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778707270698529`
- `2026-05-13 22:42:36 BST` Completed `DC-68`: moved `TerminalGeometry` into
  the legacy video renderer, removed `src_legacy/terminal.rs` from active
  clean crate wiring, removed `defender::compatibility::terminal`, and added
  public API guards that keep terminal-session setup out of the active root and
  compatibility namespace. Validation passed with the DC-68 gate: formatting,
  focused public API and video tests, all-target tests, clippy,
  `make fidelity`, live smoke, markdownlint, and `git diff --check`.
  `make fidelity` reported new Rust line coverage `0/0` non-baselined added
  executable lines. Live smoke rendered 239 frames, saw 74 distinct frame signatures,
  observed attract, credit, and playing states, injected all required controls,
  and exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778708566793249`

### DC-69: Trace Sample Oracle Quarantine

Status: `complete`

Goal: keep generated long-trace sample fixture data out of clean crate-root
wiring while preserving the legacy machine oracle behavior that still consumes
that evidence.

Scope:

- Move `src_legacy/red_label_trace_samples.rs` from the active clean crate root
  into the private legacy machine oracle module tree.
- Remove the generated fixture module from the root legacy adapter guard in
  `src/lib.rs`.
- Add a public API guard that fails if generated trace samples become
  root-wired or compatibility-exported again.
- Keep generated trace sample tests and current oracle behavior intact.
- Document that generated long-trace fixture data is historical oracle evidence,
  not a clean root adapter.

Acceptance criteria:

- `src/lib.rs` no longer declares `red_label_trace_samples`.
- The fixture module is private to `src_legacy/machine.rs`.
- Current oracle fixture behavior and live behavior remain unchanged.
- README, SPEC, and PLAN describe the private trace-sample oracle boundary.

Validation:

```sh
cargo fmt --check
cargo test --lib public_api_tests
cargo test --lib machine::red_label_trace_samples::tests
cargo test --lib oracle_equivalence_tests::clean_fixture_matches_accepted_oracle_events_and_scene_summaries
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
TRACE_SAMPLE_TERMS='red_label_trace_samples|crate::red_label_trace_samples'
TRACE_SAMPLE_TERMS="$TRACE_SAMPLE_TERMS|compatibility::red_label_trace_samples"
! rg -n "$TRACE_SAMPLE_TERMS" src src_legacy/compatibility.rs
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-13 22:47:43 BST` Started `DC-69`: posted the cycle start update and
  began moving generated long-trace sample fixture data out of clean crate-root
  wiring while keeping it available to the legacy machine oracle.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778708773431689`
- `2026-05-13 23:07:38 BST` Completed `DC-69`: moved generated long-trace
  sample fixture data out of clean crate-root wiring and into the private
  legacy machine oracle module tree, added a public API guard against root or
  compatibility re-export regressions, and documented the private oracle
  boundary in README, SPEC, and PLAN. Validation passed with the DC-69 gate:
  formatting, focused public API/private fixture/oracle equivalence tests,
  all-target tests, clippy, `make fidelity`, live smoke, trace-sample root
  search, markdownlint, and `git diff --check`. `make fidelity` reported new
  Rust line coverage `0/0` non-baselined added executable lines. Live smoke
  rendered 240 frames, saw 74 distinct frame signatures, observed attract, credit,
  and playing states, injected all required controls, and exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778710071723699`

### DC-70: Compatibility Re-export Narrowing

Status: `complete`

Goal: shrink the doc-hidden compatibility namespace to the temporary tool and
equivalence contracts still used in this repo, without changing gameplay
behavior.

Scope:

- Remove compatibility re-exports for low-level legacy modules such as assets,
  board, memory layout, ROM verification, sound internals, live runtime, PIA,
  and `wgpu` presenter ownership.
- Keep only compatibility modules required by README media tooling and clean
  equivalence tests: input, machine, machine process/state, red-label math
  types, and video.
- Mark parked low-level legacy adapters as dead-code-tolerant while they remain
  crate-private evidence for the oracle and tests.
- Add a public API guard that fails if low-level compatibility re-exports are
  restored.
- Document the narrowed compatibility boundary in README, SPEC, and PLAN.

Acceptance criteria:

- `defender::compatibility` no longer exposes asset, board, memory, ROM, sound,
  live, PIA, or `wgpu` presenter modules.
- README media generation and clean equivalence tests still compile through the
  reduced temporary compatibility surface.
- Existing gameplay, fidelity fixtures, and live smoke behavior remain
  unchanged.

Validation:

```sh
cargo fmt --check
cargo test --lib public_api_tests::compatibility_namespace_exposes_only_temporary_tool_contracts
cargo test --lib oracle_equivalence_tests
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
RETIRED_COMPAT='app|assets|board|cmos_storage|fidelity|live|pia'
RETIRED_COMPAT="$RETIRED_COMPAT|red_label_memory|red_label_message"
RETIRED_COMPAT="$RETIRED_COMPAT|red_label_wave|rom|sound|terminal"
! rg -n "pub mod ($RETIRED_COMPAT|wgpu_presenter)" src_legacy/compatibility.rs
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-13 23:12:14 BST` Started `DC-70`: posted the cycle start update and
  began narrowing the doc-hidden compatibility namespace to the temporary
  contracts still used by README media tooling and clean equivalence tests.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778710163561459`
- `2026-05-13 23:33:27 BST` Completed `DC-70`: removed low-level compatibility
  re-exports for asset, board, memory, ROM, sound, live, PIA, and `wgpu`
  presenter internals; kept only the temporary README media and clean
  equivalence contracts; added the public API regression guard; and documented
  the narrowed boundary in README, SPEC, and PLAN. Validation passed with the
  DC-70 gate: formatting, focused compatibility API guard, all-target tests,
  clippy, `make fidelity`, live smoke, retired compatibility export search,
  markdownlint, and `git diff --check`. `make fidelity` reported new Rust line
  coverage `0/0` non-baselined added executable lines. Live smoke rendered 239
  frames, saw 74 distinct frame signatures, observed attract, credit, and playing
  states, injected all required controls, and exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778711608813249`

### DC-71: Red-Label Compatibility Export Retirement

Status: `complete`

Goal: remove the remaining red-label math-type export from the doc-hidden
compatibility namespace while preserving crate-private oracle behavior.

Scope:

- Remove `defender::compatibility::red_label` now that in-repo callers no
  longer require that temporary export.
- Keep the legacy `red_label` adapter crate-private for the accepted-behavior
  bridge and oracle internals.
- Strengthen the public API guard so restoring the red-label compatibility
  export fails a focused test.
- Update README, SPEC, and PLAN to describe the narrower compatibility surface.

Acceptance criteria:

- `defender::compatibility` exposes only input, machine, machine process/state,
  and video temporary contracts.
- Existing accepted-machine, oracle, README media, and live smoke behavior
  remain unchanged.
- Docs describe red-label math types as crate-private oracle wiring, not
  compatibility API.

Validation:

```sh
cargo fmt --check
cargo test --lib public_api_tests::compatibility_namespace_exposes_only_temporary_tool_contracts
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
! rg -n "pub mod red_label" src_legacy/compatibility.rs
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-13 23:35:58 BST` Started `DC-71`: posted the cycle start update and
  began retiring the remaining red-label math-type export from the doc-hidden
  compatibility namespace.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778711721691489`
- `2026-05-13 23:56:56 BST` Completed `DC-71`: removed
  `defender::compatibility::red_label`, moved the legacy equivalence test
  import to crate-private oracle wiring, kept the red-label adapter hidden at
  the root with the other parked oracle modules, and updated README, SPEC, and
  PLAN. Validation passed with the DC-71 gate: formatting, focused
  compatibility API guard, focused oracle-equivalence tests, all-target tests,
  clippy, `make fidelity`, live smoke, retired export search, markdownlint,
  and `git diff --check`. `make fidelity` reported new Rust line coverage
  `0/0` non-baselined added executable lines. Live smoke rendered 240 frames,
  saw 74 distinct frame signatures, observed attract, credit, and playing states,
  injected all required controls, and exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778713016113959`

### DC-72: Process/State Compatibility Export Retirement

Status: `complete`

Goal: remove unused machine process/state exports from the doc-hidden
compatibility namespace while preserving crate-private oracle behavior.

Scope:

- Remove `defender::compatibility::machine_process` and
  `defender::compatibility::machine_state` now that in-repo callers no longer
  require those temporary exports.
- Keep the legacy process/state adapters crate-private for the accepted
  behavior bridge and oracle internals.
- Move legacy clean-equivalence test imports to crate-private oracle wiring.
- Strengthen the public API guard so restoring these process/state
  compatibility exports fails a focused test.
- Update README, SPEC, and PLAN to describe the compatibility namespace as the
  remaining README media surface: input, machine, and video.

Acceptance criteria:

- `defender::compatibility` exposes only input, machine, and video temporary
  contracts.
- Existing accepted-machine, oracle equivalence, README media, and live smoke
  behavior remain unchanged.
- Docs describe machine process/state contracts as crate-private oracle wiring,
  not compatibility API.

Validation:

```sh
cargo fmt --check
cargo test --lib public_api_tests::compatibility_namespace_exposes_only_temporary_tool_contracts
cargo test --lib oracle_equivalence_tests
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
! rg -n "pub mod (machine_process|machine_state)" src_legacy/compatibility.rs
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-13 23:59:21 BST` Started `DC-72`: posted the cycle start update and
  began removing the unused machine process/state exports from the doc-hidden
  compatibility namespace while keeping the underlying legacy adapters
  crate-private for oracle internals.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778713129737859`
- `2026-05-14 00:20:02 BST` Completed `DC-72`: removed
  `defender::compatibility::machine_process` and
  `defender::compatibility::machine_state`, moved the legacy equivalence test
  import to crate-private oracle wiring, left the process/state adapters
  crate-private at the root, and updated README, SPEC, and PLAN. Validation
  passed with the DC-72 gate: formatting, focused compatibility API guard,
  focused oracle-equivalence tests, all-target tests, clippy, `make fidelity`,
  live smoke, retired export search, markdownlint, and `git diff --check`.
  `make fidelity` reported new Rust line coverage `0/0` non-baselined added
  executable lines. Live smoke rendered 239 frames, saw 74 distinct frame signatures,
  observed attract, credit, and playing states, injected all required controls,
  and exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778714403527089`

### DC-73: Sprite-First Plan and Internal Compatibility Import Retirement

Status: `complete`

Goal: record sprite-first rendering as an explicit rewrite requirement and
remove the remaining internal oracle-equivalence test dependency on the
temporary compatibility namespace.

Scope:

- Document that clean `wgpu` rendering should use sprite assets, texture
  atlases, and batched sprite draws as the production representation.
- Move `src_legacy/oracle_equivalence_tests.rs` imports from
  `defender::compatibility` to crate-private legacy oracle modules.
- Add a focused public API guard so internal equivalence tests cannot drift
  back to the compatibility namespace.
- Strengthen the plan requirement that every planned dev-cycle posts Slack
  start and completion updates.
- Update README, SPEC, and PLAN to describe compatibility as README media
  tooling only for this slice.

Acceptance criteria:

- `PLAN.md` explicitly requires sprite-first `wgpu` rendering with atlases and
  batched sprite draws.
- Internal oracle-equivalence tests use crate-private legacy wiring, not the
  doc-hidden `defender::compatibility` namespace.
- `defender::compatibility` remains limited to the temporary README media
  tooling surface for this cycle.
- The work protocol explicitly requires Slack start and completion updates for
  every planned dev-cycle.

Validation:

```sh
cargo fmt --check
cargo test --lib public_api_tests::legacy_equivalence_tests_use_crate_private_oracle_wiring
cargo test --lib oracle_equivalence_tests
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
! rg -n "compatibility::|compatibility\\{" src_legacy/oracle_equivalence_tests.rs
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 07:28:10 BST` Started `DC-73`: posted the cycle start update and
  began recording sprite-first rendering as explicit rewrite scope while
  moving legacy equivalence tests off the doc-hidden compatibility namespace.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778740070294709`
- `2026-05-14 07:50:12 BST` Completed `DC-73`: documented sprite-first
  `wgpu` rendering with sprite assets, texture atlases, and batched sprite
  draws; moved legacy oracle-equivalence tests from the doc-hidden
  compatibility namespace to crate-private oracle wiring; strengthened the
  Slack start/completion update requirement in the work protocol; and updated
  README, SPEC, and PLAN. Validation passed with the DC-73 gate: formatting,
  focused public API guard, oracle-equivalence tests, all-target tests, clippy,
  `make fidelity`, live smoke, retired-import search, markdownlint, and
  `git diff --check`. `make fidelity` reported new Rust line coverage `0/0`
  non-baselined added executable lines. Live smoke rendered 239 frames, saw
  74 distinct frame signatures, observed attract, credit, and playing states,
  injected all required controls, and exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778741426762209`

### DC-74: README Media Compatibility Namespace Retirement

Status: `complete`

Goal: retire the final public compatibility namespace by moving README media
generation behind a narrow high-level facade.

Scope:

- Add a doc-hidden `defender::readme_media` facade owned by `src_legacy/` so
  README media generation no longer imports low-level machine, input, or video
  contracts.
- Update `examples/generate_readme_media.rs` to consume the high-level media
  facade.
- Delete the final `defender::compatibility` re-export namespace and its
  `src_legacy/compatibility.rs` export map.
- Strengthen public API guards so restoring the compatibility namespace or
  README media low-level imports fails a focused test.
- Update README, SPEC, and PLAN to describe README media as the only
  doc-hidden tool facade left from this slice.

Acceptance criteria:

- `examples/generate_readme_media.rs` imports `defender::readme_media`, not
  `defender::compatibility`.
- `src/lib.rs` no longer wires `pub mod compatibility`, and
  `src_legacy/compatibility.rs` is removed.
- Root legacy machine, input, and video modules remain crate-private.
- README and SPEC no longer describe the compatibility namespace as active
  tool API.

Validation:

```sh
cargo fmt --check
cargo test --lib public_api_tests::compatibility_namespace_is_retired
cargo test --lib public_api_tests::readme_media_facade_is_legacy_owned_and_doc_hidden
cargo test --lib readme_media::tests
cargo test --example generate_readme_media
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
! rg -n \
  -e "defender::compatibility" \
  -e "use defender::compatibility" \
  -e '#\\[path = "\\.\\./src_legacy/compatibility\\.rs"\\]' \
  src src_legacy examples README.md SPEC.md
test ! -e src_legacy/compatibility.rs
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 07:53:51 BST` Started `DC-74`: posted the cycle start update and
  began retiring the final public compatibility namespace by moving README
  media generation to a narrow high-level facade.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778741631993389`
- `2026-05-14 08:19:18 BST` Completed `DC-74`: added the doc-hidden
  `defender::readme_media` facade, moved README media generation off
  low-level machine/video imports, removed `src_legacy/compatibility.rs`, and
  retired the public `defender::compatibility` namespace. Validation passed
  with formatting, focused public API/readme-media tests, all-target tests,
  clippy, `make fidelity`, live smoke, compatibility grep guard, deleted-file
  guard, markdownlint, and `git diff --check`. `make fidelity` reported new
  Rust line coverage `0/0` non-baselined added executable lines. Live smoke
  rendered 239 frames, saw 74 distinct frame signatures, observed attract, credit,
  and playing states, injected all required controls, and exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778743156254659`

### DC-75: Clean Equivalence Gate Foundation

Status: `complete`

Goal: establish clean state/event/render/sound equivalence signatures before
retiring memory-oriented trace gates.

Scope:

- Add a clean `src/fidelity.rs` frame signature contract over `GameSnapshot`,
  gameplay events, sound events, and `RenderSceneSummary`.
- Root-wire legacy trace tooling as `legacy_fidelity` so the public clean
  `fidelity` module no longer points at memory-oriented trace code.
- Add focused tests comparing clean frame signatures with the accepted facade
  for credited start and live control input.
- Update README, SPEC, and PLAN to describe clean fidelity signatures and
  legacy trace quarantine.

Acceptance criteria:

- `src/fidelity.rs` contains no direct legacy module imports.
- Public clean API exposes `GameplayEquivalenceSignature`.
- Historical trace tooling remains available under the crate-private
  `legacy_fidelity` root adapter.
- Focused signature tests cover state, gameplay events, sound events, and
  render summaries.

Validation:

```sh
cargo fmt --check
cargo test --lib fidelity::tests
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_accepted_facade
cargo test --lib public_api_tests::legacy_modules_are_crate_private_at_root
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 08:21:38 BST` Started `DC-75`: posted the cycle start update and
  began adding clean frame-equivalence signatures as the first memory-oriented
  oracle retirement slice.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778743298143089`
- `2026-05-14 08:48:24 BST` Completed `DC-75`: added the clean
  `GameplayEquivalenceSignature` contract in `src/fidelity.rs`, root-wired
  historical trace tooling as crate-private `legacy_fidelity`, and added
  focused signature tests against the accepted facade for credited start and
  live control input. README, SPEC, and PLAN now describe clean fidelity
  signatures and leave broad memory-model retirement to `DC-77`. Validation
  passed with formatting, focused fidelity/public API tests, all-target tests,
  clippy, `make fidelity`, live smoke, markdownlint, and `git diff --check`.
  `make fidelity` reported new Rust line coverage `0/0` non-baselined added
  executable lines. Live smoke rendered 239 frames, saw 74 distinct frame
  CRCs, observed attract, credit, and playing states, injected all required
  controls, and exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778744904031159`

### DC-76: Clean Audio Boundary Isolation

Status: `complete`

Goal: remove the clean audio runtime's direct dependency on legacy frame
outputs.

Scope:

- Remove `machine_state::FrameOutput` from `src/audio.rs`.
- Keep live audio submission on clean `GameFrame` and `SoundEvent` contracts.
- Move legacy output-to-clean audio adaptation into `src_legacy/live.rs`.
- Add a public API guard so clean audio cannot re-import legacy frame outputs.
- Update README, SPEC, and PLAN to document the audio boundary.

Acceptance criteria:

- `src/audio.rs` contains no `FrameOutput` or legacy `machine_state` imports.
- Live audio still receives accepted startup sound events.
- Legacy live code owns the only output-to-clean audio adapter.
- Docs describe the clean audio boundary.

Validation:

```sh
cargo fmt --check
cargo test --lib audio::tests
cargo test --lib live::tests::live_core_driver_feeds_sound_events_to_audio_runtime
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_accepted_facade
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
! rg -n "FrameOutput|from_frame_output|submit_frame_output|machine_state" src/audio.rs
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 08:50:28 BST` Started `DC-76`: posted the cycle start update and
  began moving legacy frame-output audio adaptation out of the clean audio
  runtime.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778745028598859`
- `2026-05-14 09:13:39 BST` Completed `DC-76`: removed legacy
  `FrameOutput` and `machine_state` dependencies from clean `src/audio.rs`,
  moved output-to-clean audio adaptation into `src_legacy/live.rs`, and added
  a public API guard to keep clean audio on `GameFrame` and `SoundEvent`
  contracts. README, SPEC, and PLAN now document the clean audio boundary.
  Validation passed with formatting, focused audio/live/public API tests,
  all-target tests, clippy, `make fidelity`, live smoke, the clean audio
  static guard, markdownlint, and `git diff --check`. `make fidelity` reported
  new Rust line coverage `9/9` non-baselined added executable lines. Live
  smoke rendered 239 frames, saw 74 distinct frame signatures, observed attract,
  credit, and playing states, injected all required controls, and exited
  cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778746419261859`

### DC-77: Clean Sound Event Contract Isolation

Status: `complete`

Goal: remove accepted sound command-byte mapping from the clean gameplay domain
and keep that adapter logic at the oracle boundary.

Scope:

- Remove `SoundEvent::from_accepted_command`,
  `SoundEvent::accepted_command`, and `UnmappedAcceptedCommand` from
  `src/game.rs`.
- Add oracle-owned accepted sound command mapping and test-support helpers.
- Route clean fidelity and legacy oracle-equivalence tests through the oracle
  mapping helper.
- Add a public API guard so clean gameplay contracts cannot re-own accepted
  sound command mapping.
- Update README, SPEC, and PLAN to document the sound-event boundary.

Acceptance criteria:

- `src/game.rs` contains no accepted sound command mapping helpers or accepted
  unmapped-command variant.
- `src/oracle.rs` owns the accepted sound command mapping into clean
  `SoundEvent` values.
- Clean fidelity and legacy oracle-equivalence tests compare sound events
  through the oracle boundary.
- Docs describe clean `SoundEvent` as a gameplay contract, not a command-byte
  adapter.

Validation:

```sh
cargo fmt --check
cargo test --lib game::tests
cargo test --lib oracle::tests
cargo test --lib fidelity::tests
cargo test --lib oracle_equivalence_tests::clean_fixture_matches_accepted_oracle_events_and_scene_summaries
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_accepted_facade
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
! rg -n "from_accepted_command|accepted_command|UnmappedAcceptedCommand" src/game.rs
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 09:16:30 BST` Started `DC-77`: posted the cycle start update and
  began moving accepted sound command-byte mapping out of the clean gameplay
  contract and into the oracle boundary.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778746590651139`
- `2026-05-14 09:38:12 BST` Completed `DC-77`: removed accepted command-byte
  conversion helpers from clean `SoundEvent`, renamed the unmapped sound
  surface to neutral `UnmappedSoundCommand`, moved accepted sound command
  mapping into `src/oracle.rs`, and routed clean fidelity plus legacy
  oracle-equivalence checks through the oracle helper. Added a public API guard
  so `src/game.rs` cannot re-own accepted command mapping. README, SPEC, and
  PLAN now document the sound-event boundary and move broad memory-model
  retirement to `DC-78`. Validation passed with focused game/oracle/fidelity,
  oracle-equivalence, and public API tests; formatting; all-target tests;
  clippy; `make fidelity`; live smoke; the static guard; markdownlint; and
  `git diff --check`. `make fidelity` reported new Rust line coverage `9/9`
  non-baselined added executable lines. Live smoke rendered 240 frames, saw 74
  distinct frame signatures, observed attract, credit, and playing states, injected
  all required controls, and exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778747892960589`

### DC-78: Clean Fidelity Reference Probe Isolation

Status: `complete`

Goal: remove clean fidelity tests' direct accepted-facade dependencies by
moving reference-machine probing behind oracle test support.

Scope:

- Add an oracle-owned `ReferenceFrameProbe` for test-only reference frames.
- Remove direct `AcceptedFrame` and `AcceptedGameplayMachine` imports from
  `src/fidelity.rs`.
- Keep `GameplayEquivalenceSignature` tests on clean `GameFrame` values and
  oracle-provided reference frames.
- Add a public API guard so clean fidelity cannot import accepted facade types
  directly.
- Update `README.md`, `SPEC.md`, and `PLAN.md`.

Acceptance criteria:

- `src/fidelity.rs` contains no `crate::accepted::`, `AcceptedFrame`,
  `AcceptedGameplayMachine`, or `adapt_accepted_` references.
- Oracle test support owns reference probing from the accepted implementation.
- Fidelity tests still prove clean signatures match a separate reference probe
  for credited start and controls.
- Documentation describes the reference-probe boundary.

Validation:

```sh
cargo fmt --check
cargo test --lib fidelity::tests
cargo test --lib oracle::tests
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_accepted_facade
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
! rg -n \
  -e "crate::accepted::" \
  -e "AcceptedFrame" \
  -e "AcceptedGameplayMachine" \
  -e "adapt_accepted_" \
  src/fidelity.rs
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 09:40:38 BST` Started `DC-78`: posted the cycle start update and
  began moving clean fidelity reference-machine probing out of `src/fidelity.rs`
  and behind oracle test support while keeping the fidelity contract on clean
  `GameFrame` signatures.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778748038056829`
- `2026-05-14 10:04:58 BST` Completed `DC-78`: added the oracle-owned
  `ReferenceFrameProbe`, removed direct accepted facade imports and adapter
  helpers from `src/fidelity.rs`, strengthened the public API guard against
  reintroducing those references, and updated README, SPEC, and PLAN to
  describe the reference-probe boundary. Broad memory-oriented oracle
  retirement moved to `DC-79`. Validation passed with formatting; focused
  fidelity, oracle, and public API tests; all-target tests; clippy;
  `make fidelity`; live smoke; the static fidelity accepted-facade guard;
  markdownlint; and `git diff --check`. `make fidelity` matched 10 trace
  fixtures covering 15452 frames and reported new Rust line coverage `0/0`
  non-baselined added executable lines. Live smoke rendered 239 frames, saw 74
  distinct frame signatures, observed attract, credit, and playing states, injected
  all required controls, and exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778749498248829`

### DC-79: Clean Game Simulation Shell

Status: `complete`

Goal: create a clean, sprite-first gameplay simulation foothold that does not
depend on the accepted machine or memory-oriented runtime model.

Scope:

- Add a clean `Game` simulation shell that owns `GameState` and emits
  `GameFrame` values.
- Drive credited start, basic playing controls, player motion, smart bomb
  inventory, and projectile launch through clean deterministic systems.
- Add a renderer-owned projectile sprite id and atlas region so clean game
  frames stay sprite-first.
- Keep live runtime and accepted-oracle behavior unchanged.
- Add tests and public API guards so clean game source stays free of legacy
  implementation terminology.
- Update `README.md`, `SPEC.md`, and `PLAN.md`.

Acceptance criteria:

- `Game` implements `GameSimulation` and advances clean frames without accepted
  machine state.
- Clean game frames contain sprite scene data and no temporary raster payload.
- Playing controls exercise clean control, motion, and projectile systems.
- Public API guards reject legacy implementation terminology in `src/game.rs`.
- Documentation describes the clean game shell as the next oracle-retirement
  foothold.

Validation:

```sh
cargo fmt --check
cargo test --lib game::tests
cargo test --lib renderer::tests::texture_atlas_owns_sprite_regions
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_accepted_facade
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
! rg -n \
  -e "crate::accepted::" \
  -e "crate::machine::" \
  -e "crate::machine_state::" \
  -e "crate::red_label::" \
  -e "red_label" \
  -e "RED_LABEL" \
  -e "source routine" \
  -e "assembler" \
  -e "memory" \
  -e "FrameOutput" \
  src/game.rs
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 10:08:58 BST` Started `DC-79`: posted the cycle start update and
  began adding a clean `Game` simulation shell as a bounded first step toward
  retiring the memory-oriented oracle from production gameplay.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778749749081009`
- `2026-05-14 10:48:29 BST` Completed `DC-79`: added the clean `Game`
  simulation shell, wired it to clean control, motion, and projectile systems,
  emitted sprite-first `GameFrame` scenes without raster payloads, added a
  renderer-owned projectile sprite atlas region, exported the clean game type,
  and strengthened the public API guard against legacy terminology in
  `src/game.rs`. The first full fidelity run exposed three uncovered added
  lines in the clean game shell; focused coverage was added for `Default` and
  left-to-right reversal before rerunning the full gate successfully. Validation
  passed with formatting; focused game, renderer, and public API tests; the full
  Rust test suite; clippy with warnings denied; `make fidelity`; live smoke; the
  static terminology guard; markdownlint; and `git diff --check`. The
  `make fidelity` gate matched 10 trace fixtures covering 15452 frames and
  reported new Rust line coverage `134/134` non-baselined added executable
  lines. Live smoke rendered 239 frames, saw 74 distinct frame signatures, observed
  attract, credit, and playing states, injected all required controls, and
  exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778752130012979`

### DC-80: Clean Production Legacy Import Guard

Status: `complete`

Goal: make the first oracle-retirement boundary enforceable by guarding clean
production modules against direct legacy imports and legacy implementation
terminology.

Scope:

- Add a single public API guard that scans clean source modules for direct
  low-level legacy root imports.
- Keep `src/accepted.rs` as the only clean source that may call the
  `accepted_behavior` adapter.
- Keep `src/oracle.rs` and `src/platform.rs` as the only temporary clean callers
  of the accepted facade.
- Remove remaining production-source references to memory-oriented terminology
  outside the parked legacy tree.
- Update `README.md`, `SPEC.md`, and `PLAN.md`.

Acceptance criteria:

- Clean production modules cannot directly import low-level legacy root modules.
- The accepted-behavior bridge remains quarantined behind `src/accepted.rs`.
- Clean gameplay, systems, renderer, platform, audio, fidelity, and oracle
  sources do not expose legacy implementation terminology.
- Runtime behavior and historical fidelity tooling remain unchanged.

Validation:

```sh
cargo fmt --check
cargo test --lib public_api_tests::clean_module_sources_keep_legacy_access_quarantined
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_accepted_facade
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
! rg -n \
  -e "red_label" \
  -e "RED_LABEL" \
  -e "source routine" \
  -e "assembler" \
  -e "memory" \
  -e "FrameOutput" \
  src/accepted.rs src/audio.rs src/fidelity.rs src/game.rs src/main.rs \
  src/oracle.rs src/platform.rs src/renderer.rs src/systems.rs
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 10:52:08 BST` Started `DC-80`: posted the cycle start update and
  began the first memory-oriented oracle retirement slice by adding a
  source-level guard for clean production modules.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778752325584989`
- `2026-05-14 11:14:03 BST` Completed `DC-80`: added
  `public_api_tests::clean_module_sources_keep_legacy_access_quarantined` so
  clean module sources cannot import low-level legacy root modules,
  `src/accepted.rs` remains the only accepted-behavior bridge, and only
  `src/oracle.rs` plus `src/platform.rs` can call the temporary accepted facade.
  Removed remaining clean-source references to memory-oriented terminology,
  documented the guard in `README.md` and `SPEC.md`, and moved the broader
  memory-oriented oracle retirement milestone to `DC-82`. Validation passed with
  formatting; focused public API guards; the full Rust test suite; clippy with
  warnings denied; `make fidelity`; live smoke; the static clean-source
  terminology scan; markdownlint; and `git diff --check`. The `make fidelity`
  gate matched 10 trace fixtures covering 15452 frames and reported new Rust
  line coverage `0/0` non-baselined added executable lines. Live smoke rendered
  239 frames, saw 74 distinct frame signatures, observed attract, credit, and playing
  states, injected all required controls, and exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778753657368129`

### DC-81: Gameplay Oracle Public Surface Retirement

Status: `complete`

Goal: remove the gameplay oracle from the supported public API while keeping it
available internally for fidelity and regression tests.

Scope:

- Make `src/oracle.rs` a crate-private module instead of a supported public
  module.
- Remove the public `GameplayOracle` re-export from `src/lib.rs`.
- Keep internal oracle tests, fidelity signatures, and legacy equivalence tests
  working through crate-private wiring.
- Replace the public oracle contract test with a public clean `Game` simulation
  contract and add a guard that the oracle stays internal.
- Update `README.md`, `SPEC.md`, and `PLAN.md`.

Acceptance criteria:

- Supported public API exposes clean gameplay contracts, not the temporary
  oracle.
- Internal fidelity tooling can still instantiate the oracle for historical
  comparison.
- Runtime behavior and trace fixture fidelity remain unchanged.

Validation:

```sh
cargo fmt --check
cargo test --lib public_api_tests::clean_contracts_have_public_game_simulation
cargo test --lib public_api_tests::gameplay_oracle_is_internal_fidelity_wiring
cargo test --lib oracle::tests
cargo test --lib fidelity::tests::clean_frame_signatures_match_reference_probe_for_start_and_controls
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
! rg -n \
  -e "pub mod oracle;" \
  -e "pub use oracle::GameplayOracle;" \
  -e "crate::GameplayOracle" \
  src
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 11:16:26 BST` Started `DC-81`: posted the cycle start update and
  began retiring the gameplay oracle from the supported public API while keeping
  internal fidelity wiring available.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778753786573509`
- `2026-05-14 11:42:10 BST` Completed `DC-81`: made the oracle module
  crate-private, removed the public `GameplayOracle` export, replaced the public
  API oracle contract with a clean `Game` simulation contract, kept the
  machine-backed oracle as internal fidelity wiring, and documented the updated
  boundary in `README.md`, `SPEC.md`, and this plan. Validation passed with
  formatting; focused public API, oracle, and fidelity-signature tests; the full
  Rust target suite; clippy with warnings denied; `make fidelity`; live smoke;
  the public-oracle static scan; markdownlint; and `git diff --check`. The
  `make fidelity` gate matched 10 trace fixtures covering 15452 frames and
  reported new Rust line coverage `0/0` non-baselined added executable lines.
  Live smoke rendered 239 frames, saw 74 distinct frame signatures, observed attract,
  credit, and playing states, injected all required controls, and exited
  cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778755376066379`

### DC-82: Render Signature Terminology

Status: `complete`

Goal: remove memory-oriented hash/CRC labels from clean render evidence and
live-smoke reporting while preserving the accepted comparison behavior.

Scope:

- Rename clean render-scene evidence from `visual_hash` to
  `visual_signature`.
- Rename live-smoke frame and phase diversity metrics from CRC labels to
  render-signature labels.
- Keep historical CRC trace fixtures and legacy machine evidence quarantined in
  `src_legacy/`.
- Update README, SPEC, and this plan so clean fidelity language describes
  state, event, sound, and render signatures.

Acceptance criteria:

- Clean `src/` APIs no longer expose render hash terminology.
- Supported live-smoke output no longer exposes frame or scene CRC terminology.
- Historical CRC terminology remains only where it describes legacy trace or
  ROM verification evidence.

Validation:

```sh
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
! rg -n \
  -e "visual_hash" \
  -e "distinct_frame_crcs" \
  -e "visual_crcs" \
  -e "frame_crcs" \
  -e "frame CRC" \
  -e "scene CRC" \
  src src_legacy/wgpu_presenter.rs src_legacy/live.rs \
  src_legacy/accepted_behavior.rs src_legacy/oracle_equivalence_tests.rs \
  README.md SPEC.md
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 11:45:41 BST` Started `DC-82`: posted the cycle start update and
  began the first memory-oriented oracle retirement slice by moving clean
  render equivalence and live-smoke evidence from hash/CRC labels to render
  signature terminology.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778755466066549`
- `2026-05-14 12:07:27 BST` Completed `DC-82`: renamed clean render evidence
  from visual hashes to visual signatures, renamed live-smoke diversity metrics
  from frame/visual CRCs to frame/visual signatures, updated the public rewrite
  docs, and kept legacy CRC terminology limited to historical trace and ROM
  evidence. Validation passed with formatting; focused renderer, fidelity, and
  `wgpu` smoke unit tests; the full Rust target suite; clippy with warnings
  denied; `make fidelity`; live smoke; stale render-hash/frame-CRC label scans
  across clean source, supported live-smoke paths, README, and SPEC;
  markdownlint; and `git diff --check`. The `make fidelity` gate matched 10
  trace fixtures covering 15452 frames and reported new Rust line coverage
  `30/30` non-baselined added executable lines. Live smoke rendered 239 frames,
  saw 74 distinct frame signatures, observed attract, credit, and playing
  states, injected all required controls, and exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778756923466539`

### DC-83: Production Model Quarantine

Status: `complete`

Goal: continue retiring the memory-oriented production model by moving the next
remaining legacy-facing runtime dependency behind clean gameplay boundaries.

Scope:

- Move platform launch off the temporary accepted facade and behind a private
  clean runtime bridge.
- Keep `src/runtime.rs` as the only clean launch owner for the current accepted
  runtime adapter.
- Add focused boundary tests so `src/platform.rs` depends on clean runtime
  configuration and does not call accepted or legacy launch functions directly.
- Keep fixture parsers and historical oracle tooling available only where they
  provide review value.
- Preserve accepted gameplay behavior and live `wgpu` behavior.

Acceptance criteria:

- The selected runtime path reads as clean gameplay code at its public boundary.
- Any remaining memory-oriented names are explicitly quarantined in legacy or
  fidelity modules.
- Public API and module names continue moving away from red-label, ROM, source
  routine, and assembler process terminology.

Validation:

```sh
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
cargo test --lib platform::tests::
cargo test --lib runtime::tests::
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_quarantined_adapters
cargo test --lib public_api_tests::clean_module_sources_keep_legacy_access_quarantined
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 12:11:22 BST` Started `DC-83`: posted the cycle start update and
  began quarantining the production launch path by moving `src/platform.rs` off
  the accepted facade and behind a private clean runtime bridge.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778757080478579`
- `2026-05-14 12:50:11 BST` Completed `DC-83`: added a private
  `src/runtime.rs` runtime host, moved `src/platform.rs` to launch through that
  clean bridge, removed runtime launch from `src/accepted.rs`, updated public
  API guards so only the runtime bridge owns the accepted launch adapter, and
  documented the new boundary in README, SPEC, and this plan. Validation passed
  with formatting; focused platform, runtime, and public API tests; `cargo
  check`; `cargo test --all-targets`; clippy with warnings denied;
  `make fidelity`; live smoke; a boundary scan preventing direct accepted/app
  launch calls from clean runtime-facing source; markdownlint; and
  `git diff --check`. The `make fidelity` gate matched 10 trace fixtures
  covering 15452 frames and reported new Rust line coverage `3/3`
  non-baselined added executable lines. Live smoke rendered 239 frames, saw 74
  distinct frame signatures, observed attract, credit, and playing states,
  injected all required controls, and exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778759467260149`

### DC-84: Runtime Config Handoff

Status: `complete`

Goal: make the private runtime bridge consume clean runtime configuration
explicitly for the next supported launch path instead of leaving configuration
hidden behind the accepted adapter.

Scope:

- Identify one live or smoke launch mode that can be selected from clean
  `RuntimeConfig` without relying on legacy CLI parsing.
- Add a clean launch-command adapter inside the private runtime bridge.
- Preserve current default CLI behavior and the accepted adapter while clean
  launch ownership expands.
- Keep `wgpu` live behavior and smoke metrics unchanged.

Acceptance criteria:

- The selected launch mode is driven by clean `RuntimeConfig`.
- The accepted adapter remains private to the runtime bridge.
- Public API and module names continue moving away from red-label, ROM, source
  routine, and assembler process terminology.

Validation:

```sh
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 12:52:55 BST` Started `DC-84`: posted the cycle start update and
  began moving clean `RuntimeConfig` handling into the private runtime bridge,
  with config-driven `wgpu` live smoke as the first direct launch handoff while
  preserving default CLI behavior through the accepted adapter.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778759572099959`
- `2026-05-14 13:15:30 BST` Completed `DC-84`: added a private runtime launch
  command that maps clean smoke configuration to `wgpu` live smoke with clean
  control-profile and CMOS-path handoff, kept default CLI launch on the
  accepted runtime adapter, strengthened runtime/public API coverage for the
  handoff, and documented the boundary in README, SPEC, and this plan.
  Validation passed with formatting; focused runtime, platform, and public API
  tests; `cargo test --all-targets`; clippy with warnings denied;
  `make fidelity`; live smoke; markdownlint; and `git diff --check`.
  `make fidelity` matched 10 trace fixtures covering 15452 frames and reported
  new Rust line coverage `23/23` non-baselined added executable lines. Live
  smoke rendered 240 frames, saw 74 distinct frame signatures, observed
  attract, credit, and playing states, injected all required controls, and
  exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778760958421079`

### DC-85: Configured Interactive Launch Handoff

Status: `complete`

Goal: make configured interactive runtime launches use clean `RuntimeConfig`
for controls, audio, and persistence without changing the default CLI entry
path.

Scope:

- Separate default CLI launch ownership from configured interactive runtime
  launch ownership inside the private runtime bridge.
- Map clean control, audio, and CMOS configuration to the current `wgpu`
  interactive runtime path.
- Preserve `cargo run`, CLI parsing, and accepted-adapter behavior for default
  command-line entry.
- Keep smoke behavior and public API guards intact.

Acceptance criteria:

- `platform::run_with_config(RuntimeConfig::default())` can launch through
  clean configuration instead of CLI argument parsing.
- `platform::run()` preserves current command-line behavior.
- The accepted adapter remains private to the runtime bridge until it is
  replaced by clean gameplay systems.

Validation:

```sh
cargo fmt --check
cargo test --lib runtime::tests::
cargo test --lib platform::tests::
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_quarantined_adapters
cargo test --lib public_api_tests::clean_module_sources_keep_legacy_access_quarantined
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 13:18:01 BST` Started `DC-85`: posted the cycle start update and
  began separating default CLI launch from configured runtime launch so clean
  `RuntimeConfig` can drive interactive `wgpu` controls, audio, and CMOS
  handoff without changing command-line entry behavior.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778761092533899`
- `2026-05-14 13:39:11 BST` Completed `DC-85`: split the private runtime
  bridge so `platform::run()` preserves default command-line behavior through
  the accepted adapter while `platform::run_with_config` maps clean
  interactive controls, audio, and CMOS settings into the `wgpu` live launch
  path. Smoke config remains directly routed to `wgpu` live smoke. Validation
  passed with formatting; focused runtime, platform, and public API tests;
  `cargo test --all-targets`; clippy with warnings denied; `make fidelity`;
  live smoke; markdownlint; and `git diff --check`. `make fidelity` matched
  10 trace fixtures covering 15452 frames and reported new Rust line coverage
  `20/20` non-baselined added executable lines. Live smoke rendered 239 frames,
  saw 74 distinct frame signatures, observed attract, credit, and playing
  states, injected all required controls, and exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778762365968649`

### DC-86: Clean Smoke CLI Handoff

Status: `complete`

Goal: let the clean platform/runtime boundary own the supported smoke-launch
CLI path while preserving the accepted adapter for unsupported historical
commands.

Scope:

- Add a narrow clean CLI handoff for `--live-smoke` and the live configuration
  flags that already correspond to `RuntimeConfig`.
- Keep unsupported or historical commands delegated to the accepted CLI
  adapter.
- Preserve existing help text and command behavior unless the clean parser
  explicitly owns that path.
- Keep `wgpu` live-smoke output and metrics unchanged.

Acceptance criteria:

- `cargo run -- --live-smoke` reaches the config-driven runtime smoke launch
  instead of relying on accepted CLI parsing.
- Unknown or non-runtime historical commands still follow the accepted adapter.
- Public API guards identify the clean CLI-owned runtime path and keep legacy
  launch adapters quarantined.

Validation:

```sh
cargo fmt --check
cargo test --lib runtime::tests::
cargo test --lib platform::tests::
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_quarantined_adapters
cargo test --lib public_api_tests::clean_module_sources_keep_legacy_access_quarantined
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 13:42:20 BST` Started `DC-86`: posted the cycle start update
  and began moving supported `--live-smoke` CLI launches onto the clean
  `RuntimeConfig` path while preserving accepted CLI delegation for default
  live play, help, malformed smoke arguments, and historical commands.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778762555090619`
- `2026-05-14 14:04:00 BST` Completed `DC-86`: clean platform CLI parsing now
  owns valid `--live-smoke` launches and maps supported live configuration
  flags into `RuntimeConfig`, while default live play, help, malformed smoke
  arguments, and historical commands stay delegated to the accepted adapter.
  Public API guards now assert the clean smoke CLI path remains source-visible.
  Validation passed with formatting; focused platform, runtime, and public API
  tests; `cargo test --all-targets`; clippy with warnings denied;
  `make fidelity`; live smoke; markdownlint; and `git diff --check`.
  `make fidelity` matched 10 trace fixtures covering 15452 frames and reported
  new Rust line coverage `37/37` non-baselined added executable lines. Live
  smoke rendered 239 frames, saw 74 distinct frame signatures, observed
  attract, credit, and playing states, injected all required controls, and
  exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778763856173449`

### DC-87: Clean Interactive CLI Handoff

Status: `complete`

Goal: let the clean platform/runtime boundary own the supported default
interactive live-play CLI path while preserving the accepted adapter for help,
malformed live options, and historical commands.

Scope:

- Extend clean CLI handoff to the no-argument live launch and valid
  `--input-profile`, `--mute`, and `--cmos-path` interactive combinations.
- Keep `--help`, malformed live options, unsupported flags, and historical
  commands delegated to the accepted CLI adapter.
- Preserve the current help text and historical command behavior.
- Keep interactive launch mapping on `RuntimeConfig` and the `wgpu` live
  runtime bridge.

Acceptance criteria:

- No-argument launch and valid live configuration flags are classified as clean
  interactive runtime launches.
- Help, malformed live options, and historical commands still follow the
  accepted adapter.
- Public API guards identify the clean-owned interactive CLI path and keep
  root launch adapters quarantined.

Validation:

```sh
cargo fmt --check
cargo test --lib platform::tests::
cargo test --lib runtime::tests::
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_quarantined_adapters
cargo test --lib public_api_tests::clean_module_sources_keep_legacy_access_quarantined
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
cargo run -- --help
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 14:06:16 BST` Started `DC-87`: posted the cycle start update
  and began extending the clean CLI handoff to no-argument interactive live
  play plus valid `--input-profile`, `--mute`, and `--cmos-path`
  combinations, while preserving accepted adapter delegation for help,
  malformed live options, unsupported flags, and historical commands.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778763987144479`
- `2026-05-14 14:27:25 BST` Completed `DC-87`: clean platform CLI parsing now
  owns no-argument interactive live launch plus valid `--input-profile`,
  `--mute`, and `--cmos-path` combinations. `--live-smoke` remains
  clean-owned, while help, malformed live options, unsupported flags, and
  historical commands still delegate to the accepted adapter. Validation
  passed with formatting; focused platform, runtime, and public API tests;
  `cargo test --all-targets`; clippy with warnings denied; `make fidelity`;
  live smoke; help output; markdownlint; and `git diff --check`.
  `make fidelity` matched 10 trace fixtures covering 15452 frames and reported
  new Rust line coverage `3/3` non-baselined added executable lines. Live
  smoke rendered 239 frames, saw 74 distinct frame signatures, observed
  attract, credit, and playing states, injected all required controls, and
  exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778765260010409`

### DC-88: Clean CLI Classification Boundary

Status: `complete`

Goal: make clean CLI ownership explicit and maintainable by separating runtime
classification from launch dispatch while preserving current command behavior.

Scope:

- Extract the platform CLI classifier into a small clean boundary that returns
  either `RuntimeConfig` or accepted-adapter delegation.
- Keep no-argument launch, valid interactive live options, and valid
  `--live-smoke` paths clean-owned.
- Keep help, malformed live options, unsupported flags, and historical commands
  delegated to the accepted adapter.
- Keep public launch dispatch thin and source-visible for API guards.

Acceptance criteria:

- `platform::run()` dispatches through the clean classifier without embedding
  parsing branches in the launch function.
- Supported runtime CLI paths produce `RuntimeConfig` without touching the
  accepted adapter.
- Delegated paths remain behavior-compatible with the accepted CLI parser.
- Public API guards identify the classifier boundary and keep root launch
  adapters quarantined.

Validation:

```sh
cargo fmt --check
cargo test --lib platform::tests::
cargo test --lib runtime::tests::
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_quarantined_adapters
cargo test --lib public_api_tests::clean_module_sources_keep_legacy_access_quarantined
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
cargo run -- --help
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 14:31:38 BST` Started `DC-88`: posted the cycle start update
  and began separating clean CLI classification from runtime launch dispatch
  while keeping no-argument live play, valid live options, and valid
  `--live-smoke` paths clean-owned and preserving accepted adapter delegation
  for help, malformed live options, unsupported flags, and historical commands.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778765521036469`
- `2026-05-14 15:12:20 BST` Completed `DC-88`: platform launch now dispatches
  from an explicit clean CLI classification boundary. The classifier returns
  clean `RuntimeConfig` values for no-argument live play, valid live options,
  and valid `--live-smoke` paths while unsupported, malformed, help, and
  historical command paths remain accepted-adapter delegations. Validation
  passed with formatting; focused platform, runtime, and public API tests;
  `cargo test --all-targets`; clippy with warnings denied; `make fidelity`;
  live smoke; help output; markdownlint; and `git diff --check`.
  `make fidelity` matched 10 trace fixtures covering 15452 frames and reported
  new Rust line coverage `26/26` non-baselined added executable lines. Live
  smoke rendered 239 frames, saw 74 distinct frame signatures, observed
  attract, credit, and playing states, injected all required controls, and
  exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778767934967469`

### DC-89: Clean CLI Help Ownership

Status: `complete`

Goal: move help handling out of accepted-adapter delegation and into the clean
platform/runtime CLI surface while preserving current user-facing help output.

Scope:

- Add a clean help classification or runtime command for `--help` and `-h`.
- Preserve the current help text and command list while moving ownership away
  from the legacy app parser.
- Keep malformed live options, unsupported flags, and historical commands
  delegated to the accepted adapter until clean replacements exist.
- Update public API guards so help ownership is source-visible in the clean
  platform/runtime boundary.

Acceptance criteria:

- `cargo run -- --help` is served by clean runtime/platform code without
  invoking accepted CLI delegation.
- Help output remains behavior-compatible with the current text.
- Accepted-adapter delegation remains covered for malformed and historical
  command paths.
- README, SPEC, and PLAN stay aligned with the clean CLI ownership boundary.

Validation:

```sh
cargo fmt --check
cargo test --lib platform::tests::
cargo test --lib runtime::tests::
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_quarantined_adapters
cargo test --lib public_api_tests::clean_module_sources_keep_legacy_access_quarantined
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --help
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 15:22:47 BST` Started `DC-89`: began moving `--help` and `-h`
  handling into the clean platform/runtime CLI surface while preserving the
  current help text and keeping malformed, unsupported, and historical command
  paths delegated to the accepted adapter. Slack start update attempted twice
  before implementation, but the Slack connector timed out both times before
  returning a message link. A later start/status retry also timed out before
  completion.
- `2026-05-14 15:45:29 BST` Completed `DC-89`: `--help` and `-h` now classify
  as a clean help launch and dispatch through `runtime::run_help()` and
  `RuntimeCommand::Help`, with the existing help text owned by
  `runtime::help_text()`. Valid clean live args remain clean-runtime launches,
  while malformed live options, unsupported flags, and historical command paths
  still delegate to the accepted adapter. Validation passed with formatting;
  focused platform, runtime, and public API tests; `cargo test --all-targets`;
  clippy with warnings denied; `make fidelity`; clean help output;
  markdownlint; and `git diff --check`. `make fidelity` matched 10 trace
  fixtures covering 15452 frames and reported new Rust line coverage `23/23`
  non-baselined added executable lines.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778769990105679`

### DC-90: Clean CLI Error Surface

Status: `complete`

Goal: move malformed clean live option handling into the clean platform/runtime
CLI surface without taking ownership of historical ROM and fidelity commands
before their clean replacements exist.

Scope:

- Add typed clean CLI diagnostics for recognized clean live options with missing
  or invalid values, including `--input-profile` and `--cmos-path`.
- Keep default live play, valid live options, live smoke, and help owned by the
  clean platform/runtime path.
- Preserve accepted-adapter delegation for historical ROM/fidelity commands and
  any unsupported compatibility paths not yet represented in clean code.
- Update source guards and tests so accepted delegation is reserved for legacy
  ownership, not for clean option parse failures.

Acceptance criteria:

- Missing or invalid values for recognized clean live options return stable
  clean CLI errors without invoking accepted CLI delegation.
- Historical ROM/fidelity commands still run through the accepted adapter.
- Unknown compatibility paths remain delegated until the clean command inventory
  explicitly takes ownership.
- CLI exit behavior, diagnostics, and help output are covered by focused tests.

Validation:

```sh
cargo fmt --check
cargo test --lib platform::tests::
cargo test --lib runtime::tests::
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_quarantined_adapters
cargo test --lib public_api_tests::clean_module_sources_keep_legacy_access_quarantined
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --input-profile
cargo run -- --input-profile invalid
cargo run -- --cmos-path
cargo run -- --help
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 18:36:44 BST` Started `DC-90`: began moving malformed
  recognized clean live options onto the clean CLI error surface while keeping
  historical ROM/fidelity commands and unsupported compatibility paths
  delegated to the accepted adapter. Initial targets are missing or invalid
  `--input-profile` values and missing `--cmos-path` values.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778780203089619`
- `2026-05-14 19:19:40 BST` Completed `DC-90`: malformed recognized clean
  live options now stop in clean platform code with typed `CleanCliError`
  diagnostics instead of falling through to the accepted adapter. Missing
  `--input-profile`, unknown `--input-profile`, and missing `--cmos-path`
  keep the previous user-facing messages and exit through the binary error
  path, while historical ROM/fidelity commands and unsupported compatibility
  paths still delegate to the accepted adapter. Validation passed with
  formatting; focused platform, runtime, and public API tests;
  `cargo test --all-targets`; clippy with warnings denied; `make fidelity`;
  clean CLI error probes; help output; markdownlint; and `git diff --check`.
  `make fidelity` matched 10 trace fixtures covering 15452 frames and reported
  new Rust line coverage `14/14` non-baselined added executable lines.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778782792884799`

### DC-91: Clean CLI Legacy Command Inventory

Status: `complete`

Goal: make the remaining accepted-adapter CLI delegation explicit by
inventorying historical ROM/fidelity commands separately from unsupported
compatibility paths.

Scope:

- Add clean classifier structure that distinguishes known historical commands
  from truly unsupported compatibility paths before either branch enters the
  accepted adapter.
- Preserve current runtime behavior for ROM report, ROM verification, fidelity
  trace, fixture, and reference-trace commands.
- Keep default live play, valid live options, live smoke, help, and clean live
  option errors owned by the clean platform/runtime path.
- Update public API guards and focused tests so future cycles can retire
  historical commands one command family at a time.

Acceptance criteria:

- Known historical ROM/fidelity commands are classified explicitly before
  accepted-adapter dispatch.
- Unknown unsupported paths remain delegated only through a separate
  compatibility branch.
- Clean CLI behavior from DC-88 through DC-90 remains unchanged.
- The command inventory is test-covered and source-visible in public API
  guards.

Validation:

```sh
cargo fmt --check
cargo test --lib platform::tests::
cargo test --lib runtime::tests::
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_quarantined_adapters
cargo test --lib public_api_tests::clean_module_sources_keep_legacy_access_quarantined
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --rom-report
cargo run -- --fidelity-list-scenarios
cargo run -- --input-profile invalid
cargo run -- --help
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 19:22:02 BST` Started `DC-91`: separating the known
  historical ROM/fidelity command inventory from unsupported compatibility
  arguments before accepted-adapter dispatch. Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778782922103819`
- `2026-05-14 20:01:24 BST` Completed `DC-91`: added explicit
  `HistoricalCliCommand` classification for ROM report, ROM verification,
  fidelity trace, fixture, scenario, and reference-trace commands; added a
  separate `CompatibilityFallback` path for unsupported and removed
  compatibility arguments; preserved clean live/help/error ownership; and
  added focused platform/API tests plus entrypoint coverage for the
  historical adapter dispatch. Validation passed with `cargo fmt --check`,
  `cargo test --lib platform::tests::`,
  `cargo test --lib runtime::tests::`,
  `cargo test --lib public_api_tests::clean_runtime_and_oracle_use_quarantined_adapters`,
  `cargo test --lib public_api_tests::clean_module_sources_keep_legacy_access_quarantined`,
  `make fidelity` including 10 trace fixtures, 15452 frames, and 29/29
  non-baselined added executable Rust lines covered, plus CLI probes for
  `--rom-report`, `--fidelity-list-scenarios`, `--input-profile invalid`,
  and `--help`. Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778785334701229`

### DC-92: Clean ROM Report Command Ownership

Status: `complete`

Goal: retire the first historical CLI command family from the accepted adapter
by moving ROM report listing into clean runtime ownership while preserving
the current command output and optional verification path.

Scope:

- Add a clean runtime command/config path for `--rom-report` with no path and
  with one optional ROM directory path.
- Keep `--verify-roms` in the historical command inventory for now.
- Preserve current ROM report text, optional path validation, and clean
  live/help/error behavior.
- Update the historical inventory so `--rom-report` no longer enters the
  accepted adapter, while the other ROM/fidelity commands still do.
- Add focused platform/runtime tests and public API guards for the new clean
  ROM report path.

Acceptance criteria:

- `cargo run -- --rom-report` is served by clean runtime code and matches the
  current report text.
- `cargo run -- --rom-report /path/to/roms` preserves current scan behavior
  and error reporting.
- Unsupported extra `--rom-report` arguments remain rejected with the current
  message.
- Historical inventory still explicitly delegates `--verify-roms` and the
  fidelity commands.

Validation:

```sh
cargo fmt --check
cargo test --lib platform::tests::
cargo test --lib runtime::tests::
cargo test --lib rom_report::tests::
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_quarantined_adapters
cargo test --lib public_api_tests::clean_module_sources_keep_legacy_access_quarantined
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --rom-report
cargo run -- --rom-report <empty-temp-dir>
cargo run -- --rom-report --verify-roms
cargo run -- --verify-roms
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 20:03:44 BST` Started `DC-92`: moving `--rom-report`
  listing and optional ROM directory scanning from the historical
  accepted-adapter CLI branch into clean runtime ownership. Slack start
  update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778785424298289`
- `2026-05-14 20:31:06 BST` Completed `DC-92`: `--rom-report` now
  classifies as a clean CLI command, dispatches through `RuntimeCommand`, and
  uses a private `rom_report` facade for current listing/scanning text while
  `--verify-roms` and fidelity commands remain in the historical inventory.
  Public API guards now assert that `runtime.rs` does not reach into legacy
  ROM internals directly, and focused tests cover the clean classifier,
  runtime dispatch, report formatting, malformed arguments, and quarantine
  boundary. Validation passed with the full command set above, including
  `make fidelity` with 10 trace fixtures, 15452 frames, and 38/38
  non-baselined added executable Rust lines covered. Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778787066498619`

### DC-93: Clean Fidelity Scenario Listing Ownership

Status: `complete`

Goal: retire the next low-risk historical CLI command by moving the
read-only fidelity scenario listing command into clean runtime ownership while
leaving trace generation and trace checking in the historical adapter.

Scope:

- Add a clean runtime command/config path for `--fidelity-list-scenarios`.
- Preserve current scenario listing text and ordering.
- Keep fidelity trace generation, trace checking, fixture checking, reference
  checking, and scenario input writing in the historical command inventory.
- Put scenario listing text behind a small clean facade owned by `fidelity` or
  runtime-support code rather than calling the accepted adapter.
- Add focused platform/runtime/API tests for command classification,
  dispatch, and output contract.

Acceptance criteria:

- `cargo run -- --fidelity-list-scenarios` is served by clean runtime code and
  matches the current output.
- Historical inventory still explicitly delegates trace/check/write commands.
- Public API guards make the new ownership boundary visible.

Validation:

```sh
cargo fmt --check
cargo test --lib platform::tests::
cargo test --lib runtime::tests::
cargo test --lib scenario_listing::tests::
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_quarantined_adapters
cargo test --lib public_api_tests::clean_module_sources_keep_legacy_access_quarantined
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --fidelity-list-scenarios
cargo run -- --fidelity-list-scenarios extra
cargo run -- --mute --fidelity-list-scenarios
cargo run -- --fidelity-trace 1
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 20:32:55 BST` Started `DC-93`: moving the read-only
  `--fidelity-list-scenarios` command into clean runtime ownership while
  leaving trace generation, trace checking, fixture checking, reference
  checking, and scenario input writing in the historical inventory. Slack
  start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778787188057879`
- `2026-05-14 20:55:19 BST` Completed `DC-93`: `--fidelity-list-scenarios`
  now classifies as a clean CLI command, dispatches through
  `RuntimeCommand::FidelityScenarioList`, and uses a private
  `scenario_listing` facade that preserves the current scenario manifest
  text and ordering. Public API guards now expose the ownership boundary and
  keep the new facade as the only clean source allowed to read the legacy
  trace scenario manifest. Validation passed with the full command set above,
  including `make fidelity` with 10 trace fixtures, 15452 frames, and 17/17
  non-baselined added executable Rust lines covered. Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778789519436699`

### DC-94: Clean Fidelity Scenario Input Writer Ownership

Status: `complete`

Goal: move the read/write scenario input expansion helper command into clean
runtime ownership while keeping trace generation and trace checking in the
historical adapter.

Scope:

- Add a clean runtime command/config path for
  `--fidelity-write-scenario-inputs <output-dir>`.
- Preserve current directory creation, file naming, expanded input text, and
  completion message.
- Keep `--fidelity-trace`, `--fidelity-trace-inputs`,
  `--fidelity-trace-inputs-file`, `--fidelity-check-trace`,
  `--fidelity-check-trace-dir`, and
  `--fidelity-check-reference-trace-dir` in the historical command inventory.
- Reuse the scenario manifest facade added in DC-93 rather than adding a new
  legacy access path.
- Add focused platform/runtime/API tests for command classification,
  malformed arguments, dispatch, and output/file contract.

Acceptance criteria:

- `cargo run -- --fidelity-write-scenario-inputs <temp-dir>` is served by
  clean runtime code and writes the same input scripts as before.
- Historical inventory still explicitly delegates trace generation and trace
  checking commands.
- Public API guards keep the scenario manifest/input facade private and
  quarantined.

Validation:

```sh
cargo fmt --check
cargo test --lib platform::tests::
cargo test --lib runtime::tests::
cargo test --lib fidelity_scenarios::tests::
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_quarantined_adapters
cargo test --lib public_api_tests::clean_module_sources_keep_legacy_access_quarantined
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --fidelity-write-scenario-inputs <empty-temp-dir>
cargo run -- --fidelity-write-scenario-inputs
cargo run -- --fidelity-write-scenario-inputs <empty-temp-dir> extra
cargo run -- --mute --fidelity-write-scenario-inputs <empty-temp-dir>
cargo run -- --fidelity-trace-inputs 'coin,start_one'
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 21:14:36 BST` Started `DC-94`: moving
  `--fidelity-write-scenario-inputs <output-dir>` into clean runtime
  ownership while preserving directory creation, `*.inputs.txt` file naming,
  expanded input text, and completion output. Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778789688179609`
- `2026-05-14 21:39:12 BST` Completed `DC-94`: scenario listing and scenario
  input writing now share a private `fidelity_scenarios` clean facade.
  `--fidelity-write-scenario-inputs <output-dir>` classifies as a clean CLI
  command, dispatches through
  `RuntimeCommand::FidelityScenarioInputWriter`, and writes the same 12
  expanded scenario input scripts as the historical helper. Trace generation
  and trace checking commands remain in the historical inventory, and public
  API guards expose the new quarantine boundary. Validation passed with the
  full command set above, including `make fidelity` with 10 trace fixtures,
  15452 frames, and 26/26 non-baselined added executable Rust lines covered.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778791204633039`

### DC-95: Clean Verify ROM Command Ownership

Status: `complete`

Goal: retire the remaining ROM-oriented historical CLI command by moving
`--verify-roms /path/to/roms` into clean runtime ownership alongside
`--rom-report`.

Scope:

- Add a clean runtime command/config path for `--verify-roms <rom-dir>`.
- Preserve current required path, extra-argument, live-option conflict,
  failure report, and successful verification output behavior.
- Reuse the existing private ROM command facade instead of adding a new clean
  legacy access path.
- Keep fidelity trace generation and trace checking commands in the historical
  command inventory.
- Add focused platform/runtime/API tests for command classification,
  malformed arguments, dispatch, and output contract.

Acceptance criteria:

- `cargo run -- --verify-roms <rom-dir>` is served by clean runtime code and
  preserves the current verification output.
- Missing paths, extra args, and mixed live options preserve current messages.
- Historical inventory still explicitly delegates fidelity trace generation
  and trace checking commands.
- Public API guards make the new ROM verification ownership boundary visible.

Validation:

```sh
cargo fmt --check
cargo test --lib platform::tests::
cargo test --lib runtime::tests::
cargo test --lib rom_report::tests::
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_quarantined_adapters
cargo test --lib public_api_tests::clean_module_sources_keep_legacy_access_quarantined
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --verify-roms
cargo run -- --verify-roms <rom-dir> extra
cargo run -- --mute --verify-roms <rom-dir>
cargo run -- --verify-roms <empty-rom-dir>
cargo run -- --verify-roms assets/roms/defender
cargo run -- --fidelity-trace 1
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 21:47:30 BST` Started `DC-95`: moving
  `--verify-roms /path/to/roms` into clean platform/runtime ownership while
  preserving argument validation, failure report, and success output behavior.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778791679350619`
- `2026-05-14 22:30:47 BST` Completed `DC-95`: `--verify-roms
  /path/to/roms` now classifies as a clean platform command, dispatches
  through `RuntimeCommand::VerifyRoms`, and reuses the private `rom_report`
  facade for verified-set loading, mapped-image construction, incomplete-set
  reports, and success output. The historical command inventory now only owns
  fidelity trace generation and trace checking commands. Validation passed
  with the focused platform/runtime/rom_report/public API tests, `cargo test
  --all-targets`, `cargo clippy --all-targets -- -D warnings`, `make
  fidelity` with 10 trace fixtures, 15452 frames, and 47/47 non-baselined
  added executable Rust lines covered, CLI probes for missing, extra, mixed
  live-option, incomplete, complete verify fixture, and historical
  `--fidelity-trace 1` behavior, `cargo run -- --live-smoke` with 240
  rendered frames, markdownlint, `cargo fmt --check`, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778794248188319`

### DC-96: Clean Fidelity Trace Command Ownership

Status: `complete`

Goal: move `--fidelity-trace <frames>` into clean platform/runtime ownership
while preserving the current trace output and argument behavior.

Scope:

- Add a clean command/config path for `--fidelity-trace` with the current
  default one-frame trace.
- Preserve live-option conflict, invalid frame-count, zero frame-count, and
  extra-argument error messages.
- Add a private fidelity trace facade that keeps legacy trace generation
  quarantined behind clean runtime ownership.
- Leave trace-input and trace-check commands in the historical inventory for
  later slices.
- Add focused platform/runtime/API tests for classification, malformed
  arguments, dispatch, and output contract.

Acceptance criteria:

- `cargo run -- --fidelity-trace` and `cargo run -- --fidelity-trace 2` are
  served by clean runtime code and preserve the current trace text.
- Malformed `--fidelity-trace` invocations preserve current messages.
- Historical inventory still explicitly delegates trace-input and trace-check
  commands.
- Public API guards make the new trace generation boundary visible.

Validation:

```sh
cargo fmt --check
cargo test --lib platform::tests::
cargo test --lib runtime::tests::
cargo test --lib fidelity_traces::tests::
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_quarantined_adapters
cargo test --lib public_api_tests::clean_module_sources_keep_legacy_access_quarantined
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --fidelity-trace
cargo run -- --fidelity-trace 2
cargo run -- --fidelity-trace 0
cargo run -- --fidelity-trace wat
cargo run -- --fidelity-trace 1 extra
cargo run -- --mute --fidelity-trace 1
cargo run -- --fidelity-trace-inputs none
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 22:32:30 BST` Started `DC-96`: moving
  `--fidelity-trace <frames>` into clean platform/runtime ownership while
  preserving default frame count, malformed-count errors, trace text, and
  historical delegation for trace-input and trace-check commands.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778794361787019`
- `2026-05-14 22:55:51 BST` Completed `DC-96`: `--fidelity-trace
  <frames>` now classifies as a clean platform command, dispatches through
  `RuntimeCommand::FidelityTrace`, and generates the current idle trace through
  a private `fidelity_traces` facade. The facade keeps legacy trace generation
  quarantined while preserving default one-frame output, counted output,
  malformed-count messages, zero-count rejection, extra-argument rejection, and
  live-option conflict behavior. Trace-input and trace-check commands remain
  in the historical inventory for later slices. Validation passed with the
  focused platform/runtime/fidelity trace/public API tests, `cargo test
  --all-targets`, `cargo clippy --all-targets -- -D warnings`, `make
  fidelity` with 10 trace fixtures, 15452 frames, and 40/40 non-baselined
  added executable Rust lines covered, CLI probes for successful and malformed
  `--fidelity-trace` invocations plus historical `--fidelity-trace-inputs
  none`, `cargo run -- --live-smoke` with 240 rendered frames, markdownlint,
  `cargo fmt --check`, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778795765516439`

### DC-97: Clean Fidelity Trace Input Command Ownership

Status: `complete`

Goal: move `--fidelity-trace-inputs <script>` and
`--fidelity-trace-inputs-file <path>` into clean platform/runtime ownership
while preserving current trace output and argument behavior.

Scope:

- Add clean command/config paths for inline trace input scripts and trace input
  script files.
- Preserve live-option conflict, missing argument, extra argument, invalid
  script, and file-read failure behavior.
- Extend the private fidelity trace facade so legacy trace generation remains
  quarantined behind clean runtime ownership.
- Leave trace-check commands in the historical inventory for later slices.
- Add focused platform/runtime/API tests for classification, malformed
  arguments, dispatch, file reads, and output contract.

Acceptance criteria:

- `cargo run -- --fidelity-trace-inputs none` and
  `cargo run -- --fidelity-trace-inputs-file <path>` are served by clean
  runtime code and preserve current trace text.
- Malformed trace-input invocations preserve current messages.
- Historical inventory still explicitly delegates trace-check commands.
- Public API guards make the trace input boundary visible.

Validation:

```sh
cargo fmt --check
cargo test --lib platform::tests::
cargo test --lib runtime::tests::
cargo test --lib fidelity_traces::tests::
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_quarantined_adapters
cargo test --lib public_api_tests::clean_module_sources_keep_legacy_access_quarantined
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --fidelity-trace-inputs none
cargo run -- --fidelity-trace-inputs 'coin,start_one;fire,thrust;none'
cargo run -- --fidelity-trace-inputs
cargo run -- --fidelity-trace-inputs none extra
cargo run -- --mute --fidelity-trace-inputs none
cargo run -- --fidelity-trace-inputs-file <input-script-path>
cargo run -- --fidelity-trace-inputs-file
cargo run -- --fidelity-trace-inputs-file <input-script-path> extra
cargo run -- --mute --fidelity-trace-inputs-file <input-script-path>
cargo run -- --fidelity-check-trace-dir docs/fidelity/fixtures/local/rust-current
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 22:57:49 BST` Started `DC-97`: moving
  `--fidelity-trace-inputs <script>` and
  `--fidelity-trace-inputs-file <path>` into clean platform/runtime ownership
  while preserving argument validation, file-read behavior, trace output, and
  historical delegation for trace-check commands.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778795883407799`
- `2026-05-14 23:24:23 BST` Completed `DC-97`: clean platform/runtime code now
  owns inline and file-based trace input commands, the private fidelity trace
  facade owns inline and file trace text generation, trace-input commands are
  removed from the historical command inventory, and public API guards cover
  the new boundary. Validation passed: focused platform/runtime/fidelity trace
  and public API tests; `cargo test --all-targets`; `cargo clippy --all-targets
  -- -D warnings`; `make fidelity` with 10 trace fixtures, 15452 frames, and
  68/68 added executable Rust lines covered; CLI probes for inline/file
  success, malformed trace-input invocations, file-read failures, live-option
  conflicts, invalid scripts, and historical trace-check delegation;
  `cargo run -- --live-smoke` with 239 rendered frames; `markdownlint`;
  `cargo fmt --check`; and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778797461215109`

### DC-98: Clean Single Trace Check Command Ownership

Status: `complete`

Goal: move `--fidelity-check-trace <inputs> <expected>` into clean
platform/runtime ownership while preserving current trace comparison behavior.

Scope:

- Add a clean command/config path for checking one trace input script against
  one expected TSV trace.
- Preserve live-option conflict, missing argument, extra argument, file-read,
  and trace mismatch behavior.
- Extend the private fidelity trace facade so single trace comparisons remain
  quarantined behind clean runtime ownership.
- Leave fixture directory and reference fixture directory checks in the
  historical inventory for later slices.
- Add focused platform/runtime/API tests for classification, malformed
  arguments, dispatch, file reads, trace mismatch, and output contract.

Acceptance criteria:

- `cargo run -- --fidelity-check-trace <inputs> <expected>` is served by clean
  runtime code and preserves current match/mismatch text.
- Malformed single trace-check invocations preserve current messages.
- Historical inventory still explicitly delegates fixture-directory and
  reference-fixture-directory checks.
- Public API guards make the single trace-check boundary visible.

Validation:

```sh
cargo fmt --check
cargo test --lib platform::tests::
cargo test --lib runtime::tests::
cargo test --lib fidelity_traces::tests::
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_quarantined_adapters
cargo test --lib public_api_tests::clean_module_sources_keep_legacy_access_quarantined
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --fidelity-check-trace <input-script-path> <expected-trace-path>
cargo run -- --fidelity-check-trace
cargo run -- --fidelity-check-trace <input-script-path>
cargo run -- --fidelity-check-trace <input-script-path> <expected-trace-path> extra
cargo run -- --mute --fidelity-check-trace <input-script-path> <expected-trace-path>
cargo run -- --fidelity-check-trace-dir docs/fidelity/fixtures/local/rust-current
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 23:26:35 BST` Started `DC-98`: moving
  `--fidelity-check-trace <inputs> <expected>` into clean platform/runtime
  ownership while preserving argument validation, file-read behavior, trace
  comparison output, and historical delegation for fixture-directory checks.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778797606858679`
- `2026-05-14 23:50:33 BST` Completed `DC-98`: clean platform/runtime code now
  owns single trace comparisons through `RuntimeCommand::FidelityTraceCheck`,
  the private fidelity trace facade compares generated trace text against
  expected TSV files, and single trace checks are removed from the historical
  command inventory while fixture-directory checks remain delegated for later
  slices. Validation passed: focused platform/runtime/fidelity trace and
  public API tests; `cargo test --all-targets`; `cargo clippy --all-targets
  -- -D warnings`; `make fidelity` with 10 trace fixtures, 15452 frames, and
  54/54 added executable Rust lines covered; CLI probes for trace-check match,
  mismatch, malformed arguments, file-read failures, live-option conflict, and
  historical trace fixture directory delegation; `cargo run -- --live-smoke`
  with 239 rendered frames; `markdownlint`; `cargo fmt --check`; and
  `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778799033889449`

### DC-99: Clean Trace Fixture Directory Command Ownership

Status: `complete`

Goal: move `--fidelity-check-trace-dir <fixtures>` into clean platform/runtime
ownership while preserving current Rust-current fixture checking behavior.

Scope:

- Add a clean command/config path for checking a directory of paired
  `*.inputs.txt` and `*.expected.tsv` trace fixtures.
- Preserve live-option conflict, missing argument, extra argument, missing
  directory, empty directory, non-directory, missing pair, fixture ordering,
  mismatch, and summary text behavior.
- Extend the private fidelity trace facade so trace fixture discovery and
  checking remain quarantined behind clean runtime ownership.
- Leave reference fixture directory checks in the historical inventory for a
  later slice.
- Add focused platform/runtime/API tests for classification, malformed
  arguments, dispatch, fixture discovery, ordering, missing pairs, and output
  contract.

Acceptance criteria:

- `cargo run -- --fidelity-check-trace-dir <fixtures>` is served by clean
  runtime code and preserves current matched/skipped/error text.
- Malformed trace fixture directory invocations preserve current messages.
- Historical inventory still explicitly delegates reference fixture directory
  checks.
- Public API guards make the trace fixture directory boundary visible.

Validation:

```sh
cargo fmt --check
cargo test --lib platform::tests::
cargo test --lib runtime::tests::
cargo test --lib fidelity_traces::tests::
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_quarantined_adapters
cargo test --lib public_api_tests::clean_module_sources_keep_legacy_access_quarantined
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --fidelity-check-trace-dir \
  docs/fidelity/fixtures/local/rust-current
cargo run -- --fidelity-check-trace-dir
cargo run -- --fidelity-check-trace-dir \
  docs/fidelity/fixtures/local/rust-current extra
cargo run -- --mute --fidelity-check-trace-dir \
  docs/fidelity/fixtures/local/rust-current
cargo run -- --fidelity-check-reference-trace-dir \
  docs/fidelity/fixtures/local/reference
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 23:52:19 BST` Started `DC-99`: moving
  `--fidelity-check-trace-dir <fixtures>` into clean platform/runtime
  ownership while preserving argument validation, fixture discovery, skipped
  directory behavior, pair validation, manifest ordering, trace comparison, and
  historical delegation for reference fixture directory checks.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778799152129099`
- `2026-05-15 00:38:25 BST` Completed `DC-99`: clean platform/runtime
  now owns `--fidelity-check-trace-dir <fixtures>` through
  `RuntimeCommand::FidelityTraceFixtureDirectory`, with paired fixture
  discovery and trace comparison quarantined in the private fidelity trace
  facade. Historical delegation remains only for
  `--fidelity-check-reference-trace-dir`.
  Validation passed: focused platform/runtime/fidelity trace/public API tests,
  `cargo test --all-targets`, `cargo clippy --all-targets -- -D warnings`,
  `make fidelity`, CLI probes for matched/skipped/malformed/non-directory/
  missing-pair/mismatch cases, historical reference delegation, and
  `cargo run -- --live-smoke`.
  `make fidelity` matched `10` trace fixture(s), `15452` frame(s), and covered
  `146/146` non-baselined added executable Rust line(s). Live smoke rendered
  `239` frames.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778801904026079`

### DC-100: Clean Reference Trace Fixture Directory Command Ownership

Status: `complete`

Goal: move `--fidelity-check-reference-trace-dir <fixtures>` into clean
platform/runtime ownership while preserving local MAME reference fixture
validation behavior.

Scope:

- Add a clean command/config path for checking complete Phase 1 local reference
  trace fixture directories.
- Preserve live-option conflict, missing argument, extra argument,
  non-directory, missing fixture, input-program drift, header drift, frame-count
  drift, required-cell, required-sound-command, required-event, and summary text
  behavior.
- Extend the private fidelity trace facade so reference fixture validation
  remains quarantined behind clean runtime ownership.
- Remove the reference fixture directory command from the historical inventory
  so trace-check CLI paths are no longer delegated to the historical app.
- Add focused platform/runtime/API tests for classification, malformed
  arguments, dispatch, reference fixture validation, evidence deadlines, and
  output contract.

Acceptance criteria:

- `cargo run -- --fidelity-check-reference-trace-dir <fixtures>` is served by
  clean runtime code and preserves current validation and summary text.
- Malformed reference fixture directory invocations preserve current messages.
- Historical command inventory no longer owns trace-check CLI paths.
- Public API guards make the reference fixture directory boundary visible.

Validation:

```sh
cargo fmt --check
cargo test --lib platform::tests::
cargo test --lib runtime::tests::
cargo test --lib fidelity_traces::tests::
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_quarantined_adapters
cargo test --lib public_api_tests::clean_module_sources_keep_legacy_access_quarantined
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --fidelity-check-reference-trace-dir \
  docs/fidelity/fixtures/local/reference
cargo run -- --fidelity-check-reference-trace-dir
cargo run -- --fidelity-check-reference-trace-dir \
  docs/fidelity/fixtures/local/reference extra
cargo run -- --mute --fidelity-check-reference-trace-dir \
  docs/fidelity/fixtures/local/reference
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 00:41:20 BST` Started `DC-100`: moving
  `--fidelity-check-reference-trace-dir <fixtures>` into clean
  platform/runtime ownership while preserving local reference fixture
  validation, malformed argument errors, required evidence checks, and summary
  text.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778802025722099`
- `2026-05-15 01:25:52 BST` Completed `DC-100`: clean platform/runtime
  now owns `--fidelity-check-reference-trace-dir <fixtures>` through
  `RuntimeCommand::FidelityReferenceTraceFixtureDirectory`, with local
  reference fixture validation quarantined in the private fidelity trace
  facade. The clean CLI no longer has a historical command inventory for
  trace-check paths.
  Validation passed: focused platform/runtime/fidelity trace/public API tests,
  `cargo test --all-targets`, `cargo clippy --all-targets -- -D warnings`,
  `make fidelity`, CLI probes for reference fixture match, malformed args,
  live conflict, missing directory, non-directory, missing expected trace,
  input-program drift, and header drift, plus `cargo run -- --live-smoke`.
  `make fidelity` matched `10` Rust-current trace fixture(s), `15452`
  frame(s), and covered `232/232` non-baselined added executable Rust line(s).
  Live smoke rendered `239` frames.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778804751380139`

### DC-101: Clean Unsupported CLI Argument Ownership

Status: `complete`

Goal: remove the accepted CLI fallback from clean platform/runtime dispatch so
unsupported and removed CLI arguments are reported by clean code directly.

Scope:

- Replace the `CompatibilityFallback` classification with clean
  `UnknownArgument` and removed renderer-selection errors.
- Preserve user-facing error text for unknown arguments and removed
  `--renderer` / `--presentation` selection.
- Remove `RuntimeCommand::AcceptedCli`, `RuntimeHost::run_cli`, and the
  `accepted_behavior::run_runtime()` fallback from clean runtime dispatch.
- Update public API guards so clean runtime no longer has an accepted CLI
  escape hatch.
- Add focused platform/runtime/API tests for unsupported arguments, removed
  renderer selection, and absence of accepted runtime fallback wiring.

Acceptance criteria:

- `cargo run -- --unknown` fails through clean platform code with the existing
  `unknown argument: --unknown` text.
- `cargo run -- --renderer wgpu` and `cargo run -- --presentation wgpu` fail
  through clean platform code with the existing `wgpu-only` text.
- Clean platform/runtime no longer calls `crate::runtime::run_cli()` or
  `crate::accepted_behavior::run_runtime()`.
- Public API guards make the removal visible.

Validation:

```sh
cargo fmt --check
cargo test --lib platform::tests::
cargo test --lib runtime::tests::
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_quarantined_adapters
cargo test --lib public_api_tests::clean_module_sources_keep_legacy_access_quarantined
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --unknown
cargo run -- --live-smoke --unknown
cargo run -- --renderer wgpu
cargo run -- --presentation wgpu
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 01:28:41 BST` Started `DC-101`: moving unsupported and
  removed CLI argument handling fully into clean platform ownership, removing
  the accepted CLI fallback from clean runtime dispatch, and preserving
  user-facing unknown-argument and `wgpu-only` renderer-selection errors.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778804917918359`
- `2026-05-15 01:53:21 BST` Completed `DC-101`: removed the clean runtime
  accepted CLI fallback, made unknown and removed renderer-selection arguments
  clean platform errors, deleted the unused accepted runtime helper, and
  removed redundant `Clean` prefixes from internal CLI classification variants.
  Validation passed with focused platform/runtime/API tests,
  `cargo test --all-targets`, `cargo clippy --all-targets -- -D warnings`,
  `make fidelity` (10 local fixtures, 15452 frames, 91/91 new executable Rust
  lines), clean CLI failure probes, `cargo run -- --live-smoke` (239 rendered
  frames), `cargo fmt --check`, markdownlint, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778806424037359`

### DC-102: Clean Sprite Batch Draw Planning

Status: `complete`

Goal: make the clean renderer draw plan resolve sprite instances through the
renderer-owned texture atlas while keeping temporary raster upload separate as
fidelity evidence.

Scope:

- Add a sprite batch representation to `SceneDrawPlan` that records each
  sprite's atlas region, layer, position, size, and tint.
- Resolve scene sprites through `NativeRendererResources::atlas` during
  `NativeSceneRenderer::prepare`.
- Report missing atlas entries as explicit draw-plan evidence instead of
  silently treating every sprite as drawable.
- Preserve the temporary raster upload path as separate from sprite batches.
- Add focused renderer tests for atlas-backed batching, missing sprite regions,
  and mixed raster/sprite plans.
- Update README/SPEC module text for atlas-backed sprite batches and the
  current clean runtime bridge wording.

Acceptance criteria:

- Sprite-first clean scenes produce atlas-backed sprite batches in the native
  renderer plan.
- Missing sprite atlas regions are counted and do not request sprite
  pipelines.
- Raster upload remains an independent temporary fidelity payload.

Validation:

```sh
cargo fmt --check
cargo test --lib renderer::tests::
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 01:56:13 BST` Started `DC-102`: adding atlas-backed sprite
  batch draw-plan evidence to the clean renderer while preserving temporary
  raster upload as a separate fidelity path.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778806586742169`
- `2026-05-15 02:18:57 BST` Completed `DC-102`: added atlas-backed sprite
  draw batches to `SceneDrawPlan`, recorded missing sprite atlas regions,
  kept temporary raster upload separate, added default terrain/star atlas
  entries, exported the new sprite-batch plan types, and refreshed README/SPEC
  module wording. Validation passed with focused renderer tests, a public API
  guard, `cargo test --all-targets`, clippy with warnings denied,
  `make fidelity` (10 local fixtures, 15452 frames, 58/58 new executable Rust
  lines), `cargo run -- --live-smoke` (239 rendered frames),
  `cargo fmt --check`, markdownlint, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778807950561589`

### DC-103: Clean Renderer Viewport Draw Planning

Status: `complete`

Goal: make the clean renderer draw plan describe how native scene coordinates
fit into a target `wgpu` surface without falling back to legacy video geometry.

Scope:

- Add an aspect-preserving viewport layout to the clean renderer plan.
- Keep `NativeSceneRenderer::prepare` as the scene-sized convenience path and
  add a target-surface preparation path for real `wgpu` surfaces.
- Handle empty scene or target surfaces without division-by-zero or misleading
  scale values.
- Use the same viewport calculation for sprite-batch plans and temporary raster
  upload plans.
- Add focused renderer tests for centered scaling, empty surfaces, sprite
  scenes, and raster scenes.
- Update README/SPEC module text for viewport-aware draw planning.

Acceptance criteria:

- Scene draw plans record the native scene surface, target surface, centered
  viewport rectangle, and scale.
- Empty scene or target surfaces produce an empty viewport plan.
- Sprite-only and raster-backed scenes use the same viewport calculation.

Validation:

```sh
cargo fmt --check
cargo test --lib renderer::tests::
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 02:21:12 BST` Started `DC-103`: adding clean viewport layout
  evidence to `SceneDrawPlan` so native scene coordinates can map into a target
  `wgpu` surface without relying on legacy video geometry.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778808084056079`
- `2026-05-15 02:44:19 BST` Completed `DC-103`: added `ViewportLayout` to the
  clean renderer plan, implemented target-aware preparation through
  `NativeSceneRenderer::prepare_for_target`, kept `prepare` as the scene-sized
  convenience path, applied the same viewport fit to sprite and temporary
  raster plans, exported the viewport type, and refreshed README/SPEC wording.
  Validation passed with focused renderer tests, a public API guard,
  `cargo test --all-targets`, clippy with warnings denied, `make fidelity` (10
  local fixtures, 15452 frames, 31/31 new executable Rust lines),
  `cargo run -- --live-smoke` (239 rendered frames), `cargo fmt --check`,
  markdownlint, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778809459870549`
  Slack final validation follow-up:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778810539805979`

### DC-104: Clean Renderer GPU Pass Planning

Status: `complete`

Goal: extend the clean renderer draw plan with GPU-ready pass constants derived
from scene data, so the next native `wgpu` path can bind clear color, viewport,
and scene-coordinate projection without consulting legacy video geometry.

Scope:

- Convert clean `Color` values into normalized `wgpu::Color` clear values.
- Add a viewport command representation that maps `ViewportLayout` to the
  values used by `wgpu::RenderPass::set_viewport`.
- Add scene-to-clip projection constants for native top-left scene coordinates.
- Attach the GPU pass constants to `SceneDrawPlan`.
- Add focused renderer tests for color normalization, empty viewports,
  viewport command values, projection constants, and sprite/raster draw plans.
- Update README/SPEC module text for GPU pass planning.

Acceptance criteria:

- Clean draw plans expose GPU-ready pass constants from clean scene data only.
- Empty scene or target surfaces do not request a viewport command.
- Sprite-only and raster-backed scenes share the same GPU pass constants.

Validation:

```sh
cargo fmt --check
cargo test --lib renderer::tests::
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 03:04:11 BST` Started `DC-104`: adding GPU-ready renderer pass
  constants for clear color, viewport commands, and scene-to-clip projection
  data derived from clean scene draw plans.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778810662415339`
- `2026-05-15 03:28:11 BST` Completed `DC-104`: added GPU-ready pass constants
  to `SceneDrawPlan`, including normalized `wgpu::Color`, optional viewport
  command values, and scene-to-clip projection uniforms for native top-left
  scene coordinates; kept sprite batches and temporary raster upload separate;
  exported the new clean renderer contracts; and refreshed README/SPEC wording.
  Validation passed with focused renderer tests, a public API guard,
  `cargo test --all-targets` (1144 library tests plus binary/example tests),
  clippy with warnings denied, `make fidelity` (10 local fixtures, 15452
  frames, 33/33 new executable Rust lines), `cargo run -- --live-smoke` (239
  rendered frames), `cargo fmt --check`, markdownlint, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778812090473359`

### DC-105: Clean Sprite Instance Buffer Planning

Status: `complete`

Goal: extend clean sprite draw planning with GPU instance-buffer records derived
from renderer-owned atlas batches, so a later native `wgpu` renderer can upload
sprite instance data without consulting legacy video state.

Scope:

- Add GPU instance-buffer records for sprite draw instances.
- Preserve native scene coordinates for sprite rectangles.
- Convert atlas regions into normalized UV origin/size using the
  renderer-owned atlas surface.
- Convert tint colors into normalized RGBA instance data.
- Keep temporary raster upload separate from sprite instance buffers.
- Add focused renderer tests for UV normalization, tint normalization,
  batch-to-buffer conversion, and empty atlas handling.
- Update README/SPEC module text for sprite instance-buffer planning.

Acceptance criteria:

- Clean draw plans expose GPU instance-buffer data alongside logical sprite
  batches.
- Sprite instance-buffer records are derived only from clean scene sprites and
  renderer-owned atlas metadata.
- Raster-backed scenes do not create sprite instance buffers unless they also
  include drawable sprites.

Validation:

```sh
cargo fmt --check
cargo test --lib renderer::tests::
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 03:29:54 BST` Started `DC-105`: adding clean GPU instance-buffer
  records for sprite batches, with native scene rectangles, normalized atlas
  UVs, and normalized tint data.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778812209675969`
- `2026-05-15 03:53:46 BST` Completed `DC-105`: added public
  `SpriteInstanceBuffer` and `SpriteInstanceBufferRecord` draw-plan data
  derived from renderer-owned atlas batches, preserved scene-space sprite
  rectangles, normalized atlas UVs and tint colors for GPU upload, skipped
  instance buffers for empty atlases or raster-only scenes, and documented the
  clean renderer contract in README/SPEC. Validation passed with focused
  renderer tests, the public API guard, `cargo test --all-targets` (1146
  library tests plus binary/example tests), clippy with warnings denied,
  `make fidelity` (10 local fixtures, 15452 frames, 38/38 new executable Rust
  lines), `cargo run -- --live-smoke` (239 rendered frames), `cargo fmt
  --check`, markdownlint, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778813618499279`

### DC-106: Clean Sprite Instance GPU ABI

Status: `complete`

Goal: make clean sprite instance-buffer records directly usable as a stable
`wgpu` instance vertex buffer, so native rendering can upload and bind sprite
instance data without ad hoc conversion at the presenter boundary.

Scope:

- Give `SpriteInstanceBufferRecord` a stable GPU byte layout.
- Expose byte stride and `wgpu` vertex attributes for scene origin, scene size,
  atlas UV origin, atlas UV size, and tint.
- Expose upload-ready bytes from `SpriteInstanceBuffer`.
- Keep logical sprite batches and temporary raster upload separate from the GPU
  instance ABI.
- Add focused renderer tests for byte layout, vertex attributes, and upload
  bytes.
- Update README/SPEC module text for the sprite instance GPU ABI.

Acceptance criteria:

- Sprite instance records can be safely cast to upload bytes.
- The renderer owns the instance-buffer vertex layout used by future `wgpu`
  sprite pipelines.
- Raster-only scenes still do not expose sprite instance upload bytes.

Validation:

```sh
cargo fmt --check
cargo test --lib renderer::tests::
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 03:56:02 BST` Started `DC-106`: adding a stable GPU ABI for
  clean sprite instance buffers, including byte layout, `wgpu` vertex
  attributes, upload bytes, focused renderer tests, and docs.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778813773647089`
- `2026-05-15 04:17:47 BST` Completed `DC-106`: made
  `SpriteInstanceBufferRecord` a stable `repr(C)` bytemuck-safe GPU record,
  exposed its `wgpu` instance vertex-buffer layout, added upload-ready bytes
  for `SpriteInstanceBuffer`, kept raster-only scenes outside the sprite upload
  path, and documented the clean sprite instance GPU ABI in README/SPEC.
  Validation passed with focused renderer tests, the public API guard,
  `cargo test --all-targets` (1148 library tests plus binary/example tests),
  clippy with warnings denied, `make fidelity` (10 local fixtures, 15452
  frames, 9/9 new executable Rust lines), `cargo run -- --live-smoke` (240
  rendered frames), `cargo fmt --check`, markdownlint, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778815079514559`

### DC-107: Clean Sprite Quad GPU ABI

Status: `complete`

Goal: add renderer-owned static quad geometry for instanced sprite draws, so
future native `wgpu` sprite pipelines can bind both vertex and instance data
from clean renderer contracts.

Scope:

- Add a stable bytemuck-safe sprite quad vertex record.
- Expose the quad vertex `wgpu` layout without conflicting with the sprite
  instance attribute locations.
- Expose renderer-owned quad vertices, indices, index format, counts, and
  upload-ready bytes.
- Choose triangle winding that remains front-facing after the clean top-left
  scene projection maps into `wgpu` clip space.
- Add focused renderer tests for layout, upload bytes, index order, and winding.
- Update README/SPEC module text for sprite quad geometry ownership.

Acceptance criteria:

- Sprite quad vertex/index data is owned by the clean renderer contract.
- Sprite quad upload bytes can be used without presenter-side repacking.
- Quad geometry and instance records have distinct vertex attribute locations.

Validation:

```sh
cargo fmt --check
cargo test --lib renderer::tests::
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 04:19:53 BST` Started `DC-107`: adding clean renderer-owned
  sprite quad vertex/index geometry, GPU vertex layout, upload bytes, focused
  renderer tests, and docs.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778815203805089`
- `2026-05-15 04:44:46 BST` Completed `DC-107`: added clean renderer-owned
  `SpriteQuadVertex` and `SpriteQuadGeometry` contracts with bytemuck-safe quad
  vertices, `u16` indices, index format/counts, upload bytes, and a distinct
  `wgpu` vertex-buffer layout; locked the index winding as front-facing after
  the clean top-left scene projection; exported the new renderer types; and
  documented sprite quad geometry ownership in README/SPEC. Validation passed
  with focused renderer tests, the public API guard, `cargo test --all-targets`
  (1151 library tests plus binary/example tests), clippy with warnings denied,
  `make fidelity` (10 local fixtures, 15452 frames, 17/17 new executable Rust
  lines), `cargo run -- --live-smoke` (239 rendered frames), `cargo fmt
  --check`, markdownlint, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778816705196489`

### DC-108: Clean Sprite Draw Command Planning

Status: `complete`

Goal: derive indexed instanced sprite draw commands from clean sprite instance
buffers and renderer-owned quad geometry, so the future native `wgpu` renderer
can bind clean buffers and issue sprite draws without presenter-side planning.

Scope:

- Add draw-command metadata for sprite pipeline, layer, instance range, vertex
  count, index count, index format, and upload byte counts.
- Derive sprite draw commands from `SpriteInstanceBuffer` values after atlas
  validation and resource pipeline filtering.
- Keep logical sprite batches, GPU instance buffers, quad geometry, and
  temporary raster upload as separate renderer-owned contracts.
- Add focused renderer tests for command derivation, multiple command ranges,
  raster-only scenes, and disabled sprite pipelines.
- Update README/SPEC module text for clean sprite draw-command planning.

Acceptance criteria:

- `SceneDrawPlan` exposes sprite draw commands for each drawable instance
  buffer.
- Sprite draw commands reference only clean renderer-owned quad geometry and
  instance-buffer metadata.
- Raster-only scenes and unavailable sprite pipelines do not produce sprite
  draw commands.

Validation:

```sh
cargo fmt --check
cargo test --lib renderer::tests::
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 04:47:02 BST` Started `DC-108`: adding clean sprite
  draw-command metadata derived from renderer-owned quad geometry and sprite
  instance buffers, with focused renderer tests and docs.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778816833640799`
- `2026-05-15 05:29:22 BST` Completed `DC-108`: added `SpriteDrawCommand`
  metadata to `SceneDrawPlan`, derived commands from clean sprite instance
  buffers and renderer-owned quad/index geometry, exported the public renderer
  contract, documented sprite draw-command planning, and covered command
  metadata, cumulative instance ranges, disabled sprite pipelines, missing
  atlas or empty buffers, raster-only scenes, and raster/sprite separation.
  Validation passed with focused renderer tests (33/33), the public API guard,
  `cargo test --all-targets` (1154 library tests plus binary/example tests),
  clippy with warnings denied, `make fidelity` (10 local fixtures, 15452
  frames, 39/39 new executable Rust lines), `cargo run -- --live-smoke` (239
  rendered frames), `cargo fmt --check`, markdownlint, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778819363660199`

### DC-109: Clean Sprite Instance Upload Planning

Status: `complete`

Goal: flatten clean sprite instance-buffer records into one upload-ready
instance stream, so the indexed sprite draw commands have a concrete
renderer-owned buffer target with matching instance ranges and byte spans.

Scope:

- Add a `SpriteInstanceUpload` contract for concatenated sprite instance
  records and upload-ready bytes.
- Derive the upload stream from `SpriteInstanceBuffer` values after atlas
  validation and resource pipeline filtering.
- Wire sprite instance upload data into `SceneDrawPlan` alongside logical
  batches, per-pipeline instance buffers, draw commands, quad geometry, and
  temporary raster upload.
- Keep raster-only scenes, unavailable sprite pipelines, missing atlas regions,
  and empty atlas surfaces outside the sprite instance upload path.
- Add focused renderer tests for upload concatenation, byte lengths, command
  range alignment, raster-only scenes, and disabled sprite pipelines.
- Update README/SPEC module text for clean sprite instance upload planning.

Acceptance criteria:

- `SceneDrawPlan` exposes an upload-ready sprite instance stream whenever
  sprite draw commands exist.
- Sprite draw-command instance ranges and byte spans index into the flattened
  upload stream without presenter-side repacking.
- Raster-only scenes and unavailable sprite pipelines do not produce sprite
  instance upload data.

Validation:

```sh
cargo fmt --check
cargo test --lib renderer::tests::
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 05:31:40 BST` Started `DC-109`: adding clean sprite instance
  upload planning that flattens draw-plan records into one upload-ready stream
  matching `SpriteDrawCommand` instance ranges and byte spans.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778819514207039`
- `2026-05-15 05:53:22 BST` Completed `DC-109`: added
  `SpriteInstanceUpload` as the flattened upload-ready stream for sprite
  instance records, wired it into `SceneDrawPlan`, aligned it with
  `SpriteDrawCommand` ranges and byte spans, kept raster-only and unavailable
  sprite paths outside the upload contract, and documented the renderer
  instance-upload stream in README/SPEC. Validation passed with focused
  renderer tests (34/34), the public API guard, `cargo test --all-targets`
  (1155 library tests plus binary/example tests), clippy with warnings denied,
  `make fidelity` (10 local fixtures, 15452 frames, 17/17 new executable Rust
  lines), `cargo run -- --live-smoke` (239 rendered frames), `cargo fmt
  --check`, markdownlint, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778820801408189`

### DC-110: Clean Sprite WGPU Buffer Upload Planning

Status: `complete`

Goal: describe the `wgpu` buffer uploads needed by clean sprite draw plans, so
future native renderer code can create quad vertex, quad index, and flattened
instance buffers without presenter-side classification.

Scope:

- Add renderer-owned sprite buffer upload metadata for quad vertices, quad
  indices, and flattened sprite instances.
- Use `wgpu::BufferUsages` directly for vertex, index, and copy-destination
  buffer creation intent.
- Wire the sprite buffer upload plan into `SceneDrawPlan` whenever sprite
  instance upload data exists.
- Keep raster-only scenes, unavailable sprite pipelines, missing atlas regions,
  and empty atlas surfaces outside sprite buffer uploads.
- Add focused renderer tests for buffer roles, labels, byte lengths, `wgpu`
  usage flags, upload bytes, and absent sprite paths.
- Update README/SPEC module text for sprite `wgpu` buffer upload planning.

Acceptance criteria:

- `SceneDrawPlan` exposes `wgpu` buffer upload metadata for sprite draw plans.
- Quad vertex/index and instance upload bytes remain renderer-owned and
  presenter-ready.
- Raster-only scenes and unavailable sprite pipelines do not produce sprite
  buffer upload plans.

Validation:

```sh
cargo fmt --check
cargo test --lib renderer::tests::
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 05:55:20 BST` Started `DC-110`: adding clean sprite `wgpu`
  buffer upload metadata for quad vertex, quad index, and flattened instance
  data with `wgpu::BufferUsages`, focused renderer tests, and docs.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778820931672629`
- `2026-05-15 06:17:24 BST` Completed `DC-110`: added
  `SpriteBufferUploadPlan` metadata for quad vertices, quad indices, and
  flattened sprite instances, wired upload plans into `SceneDrawPlan` only
  when sprite instance data exists, kept raster-only/unavailable/missing-atlas
  paths outside the upload contract, exported the public renderer types, and
  documented sprite `wgpu` buffer uploads in README/SPEC. Validation passed
  with focused renderer tests (35/35), the public API guard,
  `cargo test --all-targets` (1156 library tests plus binary/example tests),
  clippy with warnings denied, `make fidelity` (10 local fixtures, 15452
  frames, 33/33 new executable Rust lines), `cargo run -- --live-smoke` (240
  rendered frames), `cargo fmt --check`, markdownlint, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778822243196599`

### DC-111: Clean Sprite Render Pass Planning

Status: `complete`

Goal: derive clean sprite render-pass binding and draw metadata from the sprite
buffer upload plan and indexed draw commands, so future native `wgpu` code can
bind renderer-owned buffers and issue indexed instanced draws without
presenter-side classification.

Scope:

- Add renderer-owned sprite vertex/index buffer binding metadata with stable
  `wgpu` vertex buffer slots.
- Derive indexed draw ranges from existing sprite draw commands.
- Wire an optional sprite render-pass plan into `SceneDrawPlan` whenever sprite
  buffer uploads and draw commands exist.
- Keep raster-only scenes, unavailable sprite pipelines, missing atlas regions,
  empty atlas surfaces, and empty command lists outside sprite render-pass
  planning.
- Add focused renderer tests for buffer slots, byte spans, index format, draw
  ranges, and absent sprite paths.
- Update README/SPEC module text for sprite render-pass planning.

Acceptance criteria:

- `SceneDrawPlan` exposes sprite render-pass metadata only for drawable clean
  sprite plans.
- The pass plan identifies quad vertex, instance vertex, and index buffer
  bindings with `wgpu`-compatible slots and byte spans.
- Indexed draw ranges match the existing sprite draw commands exactly.

Validation:

```sh
cargo fmt --check
cargo test --lib renderer::tests::
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 06:20:38 BST` Started `DC-111`: adding clean sprite render-pass
  planning on top of the sprite `wgpu` upload plan, including stable vertex
  buffer slots, index binding metadata, indexed draw ranges, focused renderer
  tests, and docs.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778822438553379`
- `2026-05-15 06:43:23 BST` Completed `DC-111`: added renderer-owned sprite
  render-pass metadata for quad vertex, instance vertex, and index bindings,
  derived indexed draw ranges from existing sprite draw commands, wired the
  optional pass plan into `SceneDrawPlan` only for drawable sprite plans, kept
  raster-only/unavailable/missing-atlas/empty-atlas paths outside pass
  planning, exported the public renderer types, and documented sprite
  render-pass planning in README/SPEC. Validation passed with focused renderer
  tests (36/36), the public API guard, `cargo test --all-targets` (1157
  library tests plus binary/example tests), clippy with warnings denied,
  `make fidelity` (10 local fixtures, 15452 frames, 52/52 new executable Rust
  lines), `cargo run -- --live-smoke` (239 rendered frames), `cargo fmt
  --check`, markdownlint, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778823804345559`

### DC-112: Clean Sprite Pipeline Planning

Status: `complete`

Goal: describe the `wgpu` shader and render-pipeline state needed by the clean
sprite render-pass plan, so future native sprite rendering can create pipelines
without presenter-side classification.

Scope:

- Add renderer-owned sprite WGSL shader metadata and entry-point names.
- Add sprite vertex-buffer layout metadata for the quad vertex and instance
  streams, using stable pass binding slots.
- Add sprite render-pipeline metadata for primitive topology, color target,
  alpha blending, write mask, and multisample state.
- Wire an optional sprite pipeline plan into `SceneDrawPlan` whenever a sprite
  render-pass plan exists.
- Keep raster-only scenes, unavailable sprite pipelines, missing atlas regions,
  empty atlas surfaces, and empty command lists outside sprite pipeline
  planning.
- Add focused renderer tests for shader descriptors, vertex layouts, color
  target settings, custom texture formats, and absent sprite paths.
- Update README/SPEC module text for sprite pipeline planning.

Acceptance criteria:

- `SceneDrawPlan` exposes sprite pipeline metadata only for drawable clean
  sprite plans.
- The pipeline plan identifies the renderer-owned WGSL shader, vertex entry,
  fragment entry, quad/instance vertex layouts, and `wgpu` color target state.
- Raster-only scenes and unavailable sprite paths do not produce sprite
  pipeline plans.

Validation:

```sh
cargo fmt --check
cargo test --lib renderer::tests::
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 06:46:10 BST` Started `DC-112`: adding clean sprite `wgpu`
  pipeline planning with renderer-owned WGSL shader metadata, quad/instance
  vertex layouts, primitive/color target/multisample state, focused renderer
  tests, and docs.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778823967741159`
- `2026-05-15 07:09:26 BST` Completed `DC-112`: added renderer-owned sprite
  WGSL shader metadata and shader module descriptor data, quad/instance vertex
  layout plans, alpha-blended color target state, primitive state, multisample
  state, settings-driven target texture formats, optional `SceneDrawPlan`
  wiring only for drawable sprite plans, public exports, and README/SPEC
  documentation for sprite pipeline planning. Validation passed with focused
  renderer tests (39/39), the public API guard, `cargo test --all-targets`
  (1160 library tests plus binary/example tests), clippy with warnings denied,
  `make fidelity` (10 local fixtures, 15452 frames, 65/65 new executable Rust
  lines), `cargo run -- --live-smoke` (239 rendered frames), `cargo fmt
  --check`, markdownlint, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778825367225579`

## Ongoing Work

- Keep `README.md`, `SPEC.md`, and `PLAN.md` synchronized with CLI help,
  Makefile targets, workflows, and module boundaries.
- Keep added executable Rust lines covered or explicitly refresh the accepted
  uncovered baseline only when accepting existing debt.
- Keep Slack start and completion notes linked in each dev-cycle work log.
