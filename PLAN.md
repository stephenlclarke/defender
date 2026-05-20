# Defender Current Plan

Last reviewed: `2026-05-17`

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
- Clean rewrite equivalence gaps are tracked in `docs/fidelity/gaps.md` while
  R0-R9 retire the temporary accepted oracle and legacy runtime dependencies.

## Current Validation Gate

Use this gate for behavior, architecture, or release-facing changes:

```sh
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
make clean-fidelity
cargo run -- --game-smoke
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
`make clean-fidelity` covers clean `Game` versus accepted-oracle first
divergence reporting for all 12 embedded Phase 1 scenario input streams by
default; set `SCENARIOS="..."` for targeted implementation checks.

Validation ladder going forward:

- During a step, run focused checks for the touched code and `cargo fmt --check`
  when Rust changed.
- At step completion, add directly affected smoke or doc checks.
- At dev-cycle close, run focused tests, touched-document lint, smoke commands,
  and clippy when Rust behavior changed.
- At milestone close, run the full validation gate above.

## Work Protocol

- Keep `README.md`, `SPEC.md`, and `PLAN.md` aligned with the current code.
- Use focused tests for material code changes.
- `2026-05-16 19:01:48 BST`: Remove dead code as it is found, including tests
  that only protect removed APIs, unused helpers, or retired behavior. Keep or
  replace tests only when they still guard supported contracts.
- Preserve source-visible mutation checks for arcade-core behavior.
- Preserve gameplay behavior while the rewrite is underway. `XYZZY` and
  Planetoid controls remain compatibility features unless explicitly removed.
- Kitty removal is now intentional rewrite scope; do not add new terminal
  renderer abstractions.
- Use Conventional Commits for committed work.
- Do not use `codex` in branch names, commit messages, or PR titles.
- Store sprite files under `assets/sprites/`. Reuse existing sprite PNGs there
  when they fit a documented transitional runtime need, and record the
  reclassification/provenance before embedding. Store new non-legacy sound
  artifacts under `assets/sounds/`; keep pre-existing legacy `.wav` cues under
  `assets/arcade/`.
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

## Definitive Rewrite Completion Plan

This section is the terminal plan for the rewrite. Work after `DC-152` must map
to one of these milestones. Do not add more open-ended evidence cycles unless a
new source-backed fidelity gap is accepted in `docs/fidelity/gaps.md`.

### Source Audit Summary

- The clean gameplay model is real but incomplete. `src/game.rs` and
  `src/systems.rs` cover credits, start, source-profile active wave spawning
  for landers plus source-exposed bomber/pod families, player movement,
  projectiles, smart bombs, scoring, bonus stock, player damage, wave advance,
  high-score initials, bounded lander abduction/carry/release, source-shaped
  falling-human acceleration, player-catch rescue scoring, AFALL2 carried
  landing, safe-landing scoring, fatal landing/human-loss handoff,
  pod-triggered source mini-swarmer spawn/motion/bomb projection, and
  source-shaped baiter movement/fireball shell behavior, source-shaped mutant
  runtime conversion/movement/fireballs plus reserve restore placement and
  shot-timer fixtures, and source-shaped bomber image/vertical/cruise movement
  with `BOMBST` bomb shells plus source `TIEST` reserve squad placement,
  source-shaped initial lander `LANDS0` runtime plus `LANDST` reserve
  placement/velocity bytes, source `PRBST`/`PRBRES` pod reserve restore
  placement/velocity bytes, plus
  source-shell/mine descriptor fixtures for `BMBP1` projectile evidence. They
  do not yet cover the full attract program, full two-player flow,
  tilt/service behavior, or source-faithful
  death/respawn cadence.
- The clean renderer owns sprite scene contracts and detailed `wgpu` planning.
  Interactive live presentation now uses `src/live_wgpu.rs` to step clean
  `Game` frames, submit clean audio events, and execute native sprite draw
  plans through `wgpu`. `DC-156` replaces the default atlas solid fills for the
  current clean smoke-path sprite IDs with temporary PNG-backed regions.
- The default clean atlas now decodes selected reclassified prototype PNGs from
  `assets/sprites/`. These are transitional R2 atlas inputs, not authoritative
  red-label art, and later cycles must replace or extend them with source-cited
  red-label, ROM/MAME-derived, or generated assets with stronger provenance.
- The audio boundary is clean and normal interactive play attempts the
  synthesized device backend with null fallback; smoke mode remains no-device
  and deterministic. The clean game still emits only the semantic sound subset
  currently covered by accepted event timing.
- The clean-fidelity harness compares the real clean `Game` against the
  accepted oracle across all 12 embedded Phase 1 scenarios under milestone
  profiles. The profiled gate passes, but strict R9 final acceptance is still
  blocked by the remaining source-backed lifecycle and acceptance surfaces:
  later two-player turn/session sequencing and high-score ordering,
  score-popup lifecycle, explosion timing, terrain-blow presentation, clean
  object spawning/physics, final render presentation parity, validation
  stabilization, and owner acceptance of the final R9 contract.
- `DC-163` gates remaining oracle, ROM, trace, and README media adapters behind
  the explicit `legacy-tools` feature. Default production builds no longer
  compile the accepted machine, legacy live core, CMOS storage, or retired
  `wgpu` presenter; optional developer tooling still uses feature-gated
  `src_legacy/` evidence while clean equivalence remains under validation.

### Definition Of Done

The rewrite is complete only when all of these are true:

- `cargo run` plays the clean `Game` through clean `platform`, `audio`, and
  `renderer` modules, with no non-test runtime dependency on
  `src_legacy/live.rs`, `src_legacy/wgpu_presenter.rs`, or the legacy
  `ArcadeMachine`.
- The active live renderer is sprite/atlas/instanced `wgpu`, not a hidden
  full-frame raster presenter. Temporary raster upload remains available only
  for oracle tooling or explicit debug comparison.
- The clean game passes source-backed equivalence gates for the accepted
  gameplay surface: state, events, sound timing, and rendered output for the
  12 Phase 1 scenarios plus focused mechanics fixtures added below.
- The clean game uses renderer-owned arcade assets for player, enemies, humans,
  projectiles, explosions, terrain, scanner/HUD text, and title/attract
  presentation. Solid placeholder atlas regions are gone from normal runtime.
- Real audio output exists behind the clean audio backend, with deterministic
  tests for event-to-clip mapping, mixing/queueing, shutdown, and dropped-event
  accounting. `--mute` still disables audio.
- Playability is validated through live smoke plus a bounded manual play pass:
  coin/start, thrust, reverse, fire, smart bomb, hyperspace, human rescue or
  loss, enemy waves, death/respawn, scoring, high score entry, pause-free
  windowed rendering, and clean shutdown.
- `src_legacy/` is retained only as historical source, optional oracle tooling,
  or removable archived code. Public APIs and production modules expose no
  red-label routine names, memory-table names, assembler labels, or ROM labels.

### Milestone R0: Final Gates And Oracle Harness

Goal: make progress measurable before more implementation work.

Deliverables:

- Add a clean-vs-accepted scenario runner that steps `Game` and the accepted
  oracle with the same `GameInput` streams.
- Compare clean `Game` output, not `GameplayOracle` output, against accepted
  state, events, sound events, and render evidence.
- Emit a machine-readable gap report for each failed scenario, with the first
  divergent frame and divergent fields.
- Extend the accepted facade only with neutral domain contracts needed for
  comparison, not memory or routine names.
- Add a `make clean-fidelity` target and document whether it uses embedded
  Rust-current fixtures, optional local MAME references, or both.

Current implementation:

- `DC-153` adds the first test-owned clean-vs-accepted runner. It uses the
  embedded Phase 1 scenario manifest and input expansion, does not require local
  ROMs or MAME reference traces, and emits TSV first-divergence rows from the
  real clean `Game` to the accepted oracle.

Exit gate:

```sh
make clean-fidelity
cargo run -- --game-smoke
```

R0 is complete when the clean runner exists and every known clean-vs-accepted
failure is either fixed or recorded as a bounded gap in `docs/fidelity/gaps.md`.

### Milestone R1: Clean Runtime Takes Over Live Play

Goal: make the product runtime execute the clean game, even before every
mechanic is complete.

Deliverables:

- Move the live fixed-step loop, input mapper ownership, window lifecycle, and
  resize/device lifecycle into clean `platform` and `runtime` code.
- Feed `Game::step` frames into the clean renderer and audio runtime.
- Replace `src/live_wgpu.rs` as a temporary presenter facade with a clean
  launcher that owns `winit` and `wgpu` directly.
- Keep legacy live play available only behind a test/dev oracle path if still
  needed.

Current implementation:

- `DC-154` moves the `--live-smoke` frame source to the clean `Game` and
  `NativeSceneRenderer`. The smoke report now records `frame_source:
  clean_game`, `legacy_presenter_used: false`, sprite counts, and
  temporary-raster counts.
- `DC-155` moves normal interactive `cargo run` to a clean `winit`/`wgpu`
  launcher. It owns window/device lifecycle, fixed-step clean `Game` stepping,
  clean input state mapping, clean audio event submission, and indexed
  instanced sprite draws from `NativeSceneRenderer` plans.
- `2026-05-16 18:05:40 BST` R1 verification passed. `cargo run --
  --live-smoke` reported `frame_source: clean_game`,
  `legacy_presenter_used: false`, 24 clean game frames, 24 sprite frames, 290
  sprite instances, 92 sprite draw commands, and zero temporary raster frames
  or commands. `cargo run -- --game-smoke` reported the same sprite coverage
  and zero raster frames. Focused tests passed with `cargo test --lib live_wgpu
  -- --nocapture`,
  `cargo test --lib clean_runtime_and_oracle_use_quarantined_adapters --
  --nocapture`, and
  `cargo test --lib clean_module_sources_keep_legacy_access_quarantined --
  --nocapture`. `cargo check` passed. A targeted source scan of
  `src/live_wgpu.rs`, `src/runtime.rs`, and `src/game_smoke.rs` found no
  `wgpu_presenter`, `crate::live::`, `ArcadeMachine`, or `src_legacy`
  dependency in the R1 live path; the only `legacy_presenter` matches are the
  false-valued smoke report fields and tests.

Exit gate:

```sh
cargo run -- --live-smoke
cargo run -- --game-smoke
```

R1 is complete when `--live-smoke` proves sprite-rendered clean frames and no
longer depends on `src_legacy/wgpu_presenter.rs` or a legacy raster scene.

### Milestone R2: Production Sprite Assets And WGPU Execution

Status: `complete`

Completed: `2026-05-16 19:55:20 BST`

Goal: replace renderer evidence plans and placeholder art with actual
production rendering.

Deliverables:

- Load or embed the arcade PNG assets from `assets/sprites/` into a production
  texture atlas.
- Replace solid default atlas regions in normal runtime with real sprite and
  glyph pixels.
- Turn `SceneDrawPlan` into actual `wgpu` resource creation, upload, pipeline,
  bind-group, render-pass, and draw execution code.
- Add an offscreen render smoke gate that compares selected frames to checked
  visual signatures.

Exit gate:

```sh
cargo run -- --game-smoke
cargo run -- --live-smoke
```

R2 is complete when the smoke reports are backed by real asset pixels and
offscreen `wgpu` output, not only plan metadata.

Completion evidence:

- `DC-156` moved the default clean sprite atlas from solid placeholder regions
  to the reclassified `assets/sprites/` PNG inputs.
- `DC-157` moved `--live-smoke` from draw-plan metadata to actual offscreen
  `wgpu` texture rendering, readback, nonblank checks, and checked first/last
  frame signatures.
- `2026-05-16 19:55:20 BST` R2 validation passed. `make fidelity` passed
  during the milestone gate. After coverage-only dead-code/import cleanup,
  focused checks passed with `cargo fmt --check`,
  `RUSTFLAGS='--cfg coverage' cargo check --lib`,
  `cargo test --lib live_wgpu::tests -- --nocapture`,
  `cargo clippy --all-targets -- -D warnings`,
  `markdownlint README.md SPEC.md PLAN.md docs/fidelity/gaps.md
  assets/arcade/README.md assets/sprites/README.md assets/sounds/README.md`,
  and `git diff --check`. `make clean-fidelity` passed. The R2 exit smokes
  passed with `cargo run -- --game-smoke` and
  `cargo run -- --live-smoke`; final live smoke reported 24 rendered frames,
  24 offscreen `wgpu` frames, 24 nonblank readbacks, 22 distinct offscreen
  signatures, first signature `72f0f2beddc5084e`, last signature
  `262b08d50efc12c2`, zero temporary raster frames/commands, and clean exit.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778957757426449`.

### Milestone R3: Core Cabinet Flow

Goal: make clean boot, attract, credits, start, two-player state, operator
controls, high scores, and CMOS persistence source-faithful.

Deliverables:

- Implement clean attract/title/instruction state as domain systems.
- Implement coin slots, one-player and two-player start, current-player
  switching, tilt, diagnostics, audits, high-score reset, and CMOS-backed
  persistence through clean modules.
- Preserve Planetoid and `XYZZY` as compatibility layers outside arcade
  mechanics.

Exit gate:

```sh
make clean-fidelity SCENARIOS="attract_boot start_game"
cargo run -- --live-smoke --input-profile cabinet
```

R3 is complete when clean output matches accepted evidence for attract/start
flow and live cabinet controls remain playable.

### Milestone R4: Player, Terrain, Scanner, And Projectiles

Goal: make the player-facing control loop feel and behave like Defender.

Deliverables:

- Finish source-faithful player acceleration, damping, reverse, vertical
  bounds, hyperspace, smart-bomb stock, laser/shot cadence, and projectile
  lifetime.
- Implement scrolling terrain, starfield, scanner/radar presentation, viewport
  behavior, and scene ordering as clean systems.
- Replace simplified HUD placeholders with clean score, lives, bombs, wave,
  scanner, and status text/glyph rendering.

Exit gate:

```sh
make clean-fidelity \
  SCENARIOS="first_300_frames firing thrust_reverse smart_bomb hyperspace"
cargo run -- --game-smoke
```

R4 is complete when player control, projectile, HUD, scanner, and frame timing
match accepted evidence for those focused scenarios.

### Milestone R5: Enemy Ecology And Human Rules

Goal: implement the complete Defender playfield model.

Deliverables:

- Add clean enemy kinds for lander, mutant, baiter, bomber, pod, swarmer, mine,
  and enemy shots.
- Implement wave composition, RNG, spawning, movement, targeting, collisions,
  explosions, scoring, and baiter pressure.
- Implement humanoid standing, abduction, carrying, falling, catching, rescue,
  death, mutation, planet survival, and planet destruction.

Exit gate:

```sh
make clean-fidelity SCENARIOS="abduction death wave_advance planet_destruction"
cargo run -- --live-smoke
```

R5 is complete when the long playfield scenarios match accepted state, event,
sound, and visual evidence.

### Milestone R6: Death, Game Over, And High Score Completion

Goal: finish the session loop.

Deliverables:

- Implement source-faithful player death, explosion, respawn, invulnerability or
  restart timing, remaining-stock handling, game over, attract return, and high
  score qualification.
- Complete high-score entry visuals, initials editing, submission, persistence,
  and post-entry return behavior.

Exit gate:

```sh
make clean-fidelity SCENARIOS="death high_score_entry"
cargo run -- --live-smoke
```

R6 is complete when death and high-score scenarios are clean-owned and
source-faithful.

### Milestone R7: Real Audio Backend

Goal: make audio faithful and playable, not only represented by events.

Deliverables:

- Map accepted sound commands to clean semantic sound events with exact frame
  timing.
- Add a real backend that plays embedded/generated sound assets or synthesized
  equivalents through a non-blocking mixer.
- Cover startup, credit, start, thrust loop, laser, explosions, smart bomb,
  enemy movement, rescue, high score, and game-over sounds.
- Keep deterministic null backend tests and add a bounded audio smoke report.

Exit gate:

```sh
make clean-fidelity
cargo run -- --live-smoke
cargo test --lib audio::tests
```

R7 is complete when sound events match accepted timing and live play produces
audible output unless `--mute` is set.

### Milestone R8: Legacy Retirement

Goal: remove converted implementation dependencies from production.

Deliverables:

- Remove non-test `#[path = "../src_legacy/..."]` adapters from `src/lib.rs`.
- Keep optional ROM verification and MAME/reference tooling in explicit tool or
  dev-only modules, not in the production runtime path.
- Delete or archive temporary raster-presenter code after offscreen and live
  sprite rendering cover the final behavior.
- Remove public guard tests that only protect temporary quarantine paths and
  replace them with guards that production code cannot import legacy modules.

Exit gate:

```sh
cargo tree
cargo test --all-targets
make fidelity
make clean-fidelity
cargo run -- --game-smoke
cargo run -- --live-smoke
```

R8 is complete when production builds and live play no longer compile through
the legacy machine, memory model, or raster presenter.

### Milestone R9: Final Acceptance

Goal: close the rewrite with finite acceptance evidence.

Deliverables:

- Run the full validation gate, clean-fidelity gate, reference fixture checks
  where local ROM/MAME inputs are available, offscreen render checks, audio
  smoke, and live smoke.
- Record final visual, audio, and playability evidence in `README.md`,
  `SPEC.md`, and `docs/fidelity/`.
- Update `PLAN.md` to state the rewrite is complete and remove the old
  completed-cycle narrative from active planning.

Exit gate:

```sh
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
make clean-fidelity
cargo run -- --game-smoke
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md \
  docs/fidelity/live-audio.md docs/fidelity/gaps.md
git diff --check
```

R9 is complete when there are no active rewrite gaps and the clean runtime is
the only production runtime.

### Work Sequencing Rules

- Work in vertical scenario slices. A cycle should either make a named scenario
  pass, retire a named legacy runtime dependency, or replace a named placeholder
  asset/audio path.
- Avoid further renderer-evidence micro-cycles unless they directly enable R1
  or R2. The next renderer work should execute real `wgpu`, not only expose
  more command counts.
- Do not implement arcade behavior from intuition. If source/MAME evidence is
  missing, add a gap and a fixture path first.
- The plan stops at R9. Adding R10 or extending the goal requires an explicit
  owner decision.

### Repo Guidance Assessment

- The shared guidance does not block the rewrite. Conventional Commits,
  Markdown consistency, Rust module boundaries, focused tests, and the coverage
  floor all support a disciplined migration.
- The shared `agents_repo-contexts.md` contains historical `battlezone` Kitty
  terminal notes. They are not applicable to this repository's current
  `wgpu`-only Defender rewrite. For this repo, `PLAN.md`, `SPEC.md`, and local
  source tests are the operative guidance.
- The current spec rule to preserve source-visible mutations should apply to
  legacy oracle and fixture evidence, not to clean production architecture. The
  clean game should prove equivalent behavior through domain state, events,
  sound, and rendered output rather than recreating memory tables.
- `make fidelity` is intentionally broad and slow. Keep it as the phase gate
  for behavior and release-facing changes, but add narrower `make
  clean-fidelity` scenario gates so implementation cycles are not forced into
  all-or-nothing validation for every edit.
- Slack start/completion posts remain mandatory only for planned dev-cycles.
  Analysis-only and docs-only planning updates do not require new implementation
  cycles unless they are explicitly tracked as one.

## Active Development Cycle

### DC-164: R9 Final Acceptance Vertical Slice

Status: `in progress`

Milestone: `R9: Final Acceptance`

Started: `2026-05-16 21:58:34 BST`

Goal: close the rewrite with finite acceptance evidence, refreshed docs, and the
full R9 validation gate after confirming the clean runtime is the only
production runtime path.

Scope:

- Reconcile the R9 exit criteria against current clean runtime, accepted traces,
  reclassified assets, live/offscreen smoke evidence, and docs.
- Remove dead code or stale tests only if the final audit proves they guard
  retired behavior rather than supported contracts.
- Do not close the milestone until the full R9 gate passes or a bounded,
  timestamped blocker is recorded.

Completion roadmap after Step 40:

- Baseline: `make clean-fidelity` currently matches all 12 embedded Phase 1
  scenarios under the profiled comparison surface. Future work must either
  tighten that surface with source-backed evidence or retire a remaining
  source-backed blocker directly.
- Each implementation cycle below maps to a future work-log step. Post Slack
  start/completion updates for implementation cycles, update README/SPEC/gaps
  when behavior changes, and keep focused validation green before closing a
  cycle. Run full clean-fidelity and the broader R9 gate at phase boundaries,
  shared-contract changes, broad-risk changes, or milestone closeout.
- If a cycle discovers missing source/MAME evidence, stop that cycle with a
  timestamped blocker and fixture/source path instead of guessing behavior.

Phase 1: final contract freeze.

- Step 41 / Cycle R9-A1: strict blocker matrix and acceptance contract.
  Deliver a table in `PLAN.md` or `docs/fidelity/gaps.md` that maps every
  remaining blocker to one of: accepted-surface gap, clean-game behavior gap,
  render-presentation gap, docs/evidence gap, or owner-decision gap. Exit when
  no blocker is described only as broad "presentation parity" or "remaining
  behavior".
- Step 42 / Cycle R9-A2: accepted-surface audit. Inspect whether the accepted
  facade already exposes enough neutral evidence for score popups, explosions,
  terrain blow, scanner/radar, two-player turns, high-score ordering, and
  attract waits. Add only narrow source-backed fields needed by later cycles,
  with focused adapter/oracle tests.

Phase 2: presentation program closure.

- Step 43 / Cycle R9-B1: title/status text sweep. Inventory source text
  outside covered prompt, attract, top-display-border, wave-completion,
  high-score-entry, and hall-of-fame surfaces; add bounded message-glyph
  projections or record exact excluded cases.
- Step 44 / Cycle R9-B2: Williams/copyright attract waits and page scheduler.
  Implement or expose the remaining source timing gates for Williams/logo,
  copyright, presents, Defender wordmark, and instruction pages without
  changing gameplay lifecycle behavior.
- Step 45 / Cycle R9-B3: palette, blink, and color contract. Define the clean
  render contract for logo, underline, border, HUD, and attract color/blink
  behavior. Implement only source-backed visual state that can be tested
  without full-frame raster dependence.
- Step 46 / Cycle R9-B4: scanner/radar animation. Add source-backed scanner
  and radar state, scene sprites, and focused clean/oracle tests. Keep enemy
  gameplay spawning and physics out unless required by the source scanner
  contract.

Phase 3: lifecycle and world closure.

- Step 47 / Cycle R9-C1: score-popup lifecycle. Carry source-backed score-popup
  spawn, duration, position, and sprite identity through clean/oracle scenes;
  keep scoring arithmetic unchanged except where source evidence requires it.
- Step 48 / Cycle R9-C2: explosion timing. Close player/enemy/bomb/swarmer/
  astronaut explosion timing and sprite progression with source-backed frame
  evidence and focused collision/death tests.
- Step 49 / Cycle R9-C3: terrain-blow presentation. Implement terrain
  explosion presentation and terrain mutation evidence for planet-destruction
  paths, bounded to source-backed terrain-blow behavior.
- Step 50 / Cycle R9-C4: clean object spawning and physics. Replace remaining
  simplified clean object ecology with source-backed spawning, active/inactive
  object transitions, projectile limits, enemy-family movement, abduction, and
  rescue/loss behavior needed for strict R9 acceptance.

Phase 4: session and high-score closure.

- Step 51 / Cycle R9-D1: two-player turn/session sequencing. Close all
  remaining player-one/player-two turn transitions, respawn cadence, stock
  accounting, score ownership, and game-over routing beyond the covered
  final-life switch/respawn slice.
- Step 52 / Cycle R9-D2: high-score ordering and post-entry return. Prove
  source-backed table ordering, initials insertion, today's-greatest behavior,
  post-entry display timing, and return-to-attract behavior across one-player
  and two-player sessions.

Phase 5: final render parity and acceptance.

- Step 53 / Cycle R9-E1: final render-presentation parity audit. Compare clean
  sprite scenes, offscreen `wgpu` signatures, and accepted visual evidence for
  the 12 Phase 1 scenarios plus any focused fixtures added above. Record only
  source-backed residual differences.
- Step 54 / Cycle R9-E2: full validation stabilization. Run and fix the full
  R9 gate: formatting, all Rust targets, clippy, `make fidelity`, full
  `make clean-fidelity`, game/live smoke, markdownlint, and diff hygiene.
- Step 55 / Cycle R9-E3: owner acceptance and milestone closeout. Update
  README, SPEC, `docs/fidelity/gaps.md`, and this plan to state the final R9
  contract, remaining non-rewrite follow-ups if any, validation evidence, and
  owner acceptance status.

Strict R9 blocker matrix after Step 41:

| ID | Area | Type | Step | Done when |
| --- | --- | --- | --- | --- |
| B01 | Text/status | render | 43 | exact cases covered/excluded |
| B02 | Attract waits | accepted/clean | 44 | source timing tested |
| B03 | Palette/blink | render | 45 | visual-state contract tested |
| B04 | Scanner/radar | accepted/render | 46 | state and sprites tested |
| B05 | Score popups | clean behavior | 47 | spawn/duration/position tested |
| B06 | Explosions | clean behavior | 48 | frame progression tested |
| B07 | Terrain blow | clean/render | 49 | mutation/presentation tested |
| B08 | Object ecology | clean behavior | 50 | source ecology fixtures pass |
| B09 | Two-player flow | clean behavior | 51 | turn/stock routing tested |
| B10 | High-score return | clean/accepted | 52 | tables and return tested |
| B11 | Render audit | docs/evidence | 53 | residuals source-backed |
| B12 | Full validation | docs/evidence | 54 | full R9 gate passes |
| B13 | Owner signoff | owner decision | 55 | final contract accepted |

Step 41 acceptance contract:

- The phrase "final render presentation parity" now means only the residual
  visual comparison left after B01-B10 are complete; it is no longer a
  standalone implementation blocker.
- The phrase "broader title/status text" now maps to B01 and must be closed by
  an inventory of exact included and intentionally excluded source text cases.
- Lifecycle wording is split across B05-B08 so score popups, explosions,
  terrain blow, and object ecology can be implemented and tested separately.
- Session wording is split across B09-B10 so two-player runtime flow and
  high-score table/return behavior can close independently.
- B12 is the first mandatory broad-gate cycle unless an earlier cycle changes a
  shared public contract, alters broad clean-fidelity behavior, or introduces a
  risk that focused tests cannot cover.

Accepted-surface audit after Step 42:

| ID | Current accepted evidence | Later need |
| --- | --- | --- |
| B01 | phase, frame, source glyphs | remaining text inventory |
| B02 | phase/frame only | attract page and wait state |
| B03 | source visual state | palette/RGB render audit residuals |
| B04 | object/scanner evidence, border/scanner sprites | closed by Step 46 |
| B05 | object/expanded sprite rows | popup lifecycle and duration |
| B06 | expanded explosion rows | explosion frame/timer state |
| B07 | mapped terrain explosion sprite | terrain mutation/blow state |
| B08 | bounded object rows, wave profile | full topology and transitions |
| B09 | current player, stocks, switch timers | full turn/session fixtures |
| B10 | entry, submission, tables, stalls | ordering and return fixtures |
| B11 | visual signature, clean scene | post-B01-B10 visual residuals |
| B12 | validation commands only | full R9 gate run |
| B13 | none | owner decision |

Step 42 acceptance contract:

- Add accepted fields only when a later cycle has a focused fixture that needs
  them. Do not widen the accepted facade speculatively.
- B02-B04 require new neutral evidence before implementation can be proven
  without relying on visual CRC alone.
- B05-B08 can start from existing object and expanded-object rows, but each
  needs explicit lifecycle state before it can close strictly.
- B09-B10 have enough high-level session and high-score fields for focused
  tests, but the next cycles must add fixtures that prove ordering and return
  behavior instead of depending on broad scenario matches.

Title/status text sweep after Step 43:

- Included source-message projection cases:
  - `GO`: final game-over prompt and player-switch game-over line.
  - `PLYR1`, `PLYR2`: player start, player switch, and high-score entry
    player label.
  - `CREDV`: attract credits label, paired with the bounded credit digits.
  - `ELECV`: attract electronics/presents page text.
  - `SCANV`, `LANDV`, `MUTV`, `BAITV`, `BOMBV`, `SWRMPV`, `SWARMV`: attract
    instruction and enemy-score page text.
  - `ATWV`, `COMPV`, `BONSX`: wave-completion status text, wave number,
    bonus multiplier, and survivor bonus row.
  - `HOFV1`, `HOFV2`, `HOFV3`, `HOFV4`: high-score entry instructions.
  - `HALLD_TITLE`, `HALLD_TODAYS`, `HALLD_ALL_TIME`, `HALLD_GREATEST`:
    hall-of-fame headings, with table rank, initials, and score glyph rows.
- Deferred source-backed presentation cases:
  - B02 owns attract page scheduling and wait-state timing for the Williams
    logo, Defender wordmark, copyright strip, presents page, and instruction
    page.
  - B03 owns palette, blink, and color state for the projected text, border,
    logo, underline, HUD, and attract surfaces.
- Excluded R9 B01 source-message cases:
  - Service diagnostics: `VWROM`, `VWRAM`, `VROMFL`, `VRAMFL`, `VALROM`,
    `VRAMTS`, `VNORAM`, `VCMSFL`, `VCMSOK`, `VCMSAB`, `VCOLTS`, `VAUDTS`,
    `VSWTTS`, and `VMONTS`.
  - Switch-test labels: the `VSW*` labels from `VSW0` through `VSW17`,
    including `VSWA` through `VSWF`.
  - Test and adjustment instructions: `VINS1` through `VINS17`.
  - These labels are service, audit, switch-test, or adjustment-mode text,
    not final gameplay presentation. They stay outside strict R9 unless the
    owner explicitly expands the milestone beyond clean runtime acceptance.

Step 43 acceptance contract:

- B01 is closed as an exact source-text inventory. Future title/status work
  must name the specific label, state gate, and blocker ID it belongs to.
- Every gameplay, attract, high-score, and hall-of-fame source-message label in
  `assets/red-label/messages.tsv` now has an included, deferred, or excluded
  owner.
- Diagnostics, switch-test, and adjustment text are excluded from R9 final
  gameplay presentation and should be tracked as a later service-mode feature
  if they become product scope.

Attract scheduler after Step 44:

- `GameState` now carries `AttractPresentationSnapshot` with the current
  normal-attract page frame, source page, source sleep ticks, and copyright
  stall ticks.
- The clean and oracle scenes gate the Williams logo, presents copy, Defender
  wordmark, copyright strip, and instruction labels from that snapshot. Credits
  keep their existing projection path, and hall-of-fame display stalls continue
  to suppress ordinary attract title-program surfaces.
- The source-backed page-frame thresholds are: presents at frame 236, Defender
  wordmark at frame 285, copyright wait at frame 419, and instruction labels
  at frame 441. The exposed wait constants are `LOGO0` 2 ticks, `PRES1` 5
  ticks, `DEFEND` 0x30 ticks, `CPR55` 10 ticks with a 60-tick stall, and
  instruction entry 0xE6 ticks.

Step 44 acceptance contract:

- B02 is closed for the clean runtime title-program scheduler. Future attract
  work must target a specific remaining presentation blocker instead of
  reopening broad Williams/copyright wait timing.
- This step does not claim live Williams `LGOTAB` table-walker animation or
  exact palette/blink/color parity; those remain later visual-state and
  render-audit work.
- The source-backed scheduler is proven by focused clean/oracle unit tests and
  targeted `attract_boot` plus `start_game` clean-fidelity scenarios, not by a
  full all-scenario gate.

Palette/color contract after Step 45:

- `SOURCE_VISUAL_STATE` now records the source-owned color and blink evidence
  used by the clean scene projection: Williams status `0xFB`, Williams logo
  PCRAM color `0x3F`, copyright Williams color index `0x0F`, Williams restore
  rates `0xFF` and `10`, instruction color words `0x6666`, `0x0000`, and
  `0x4433`, Hall of Fame display/entry letter color indices `0x00` and
  `0x85`, Hall of Fame blink color `0x85` with 15-tick sleep, Hall of Fame
  underline words `0x1111` and `0xDDDD`, and top-display border/scanner-marker
  words `0x5555` and `0x9999`.
- Clean and oracle scenes route HUD score/stock tints, attract title surfaces,
  Hall of Fame text/logo/underline tints, and top-display border/marker tints
  through that source visual-state contract without introducing full-frame
  raster dependence.
- The current contract preserves the existing white/gray clean sprite output.
  Hardware palette-to-RGB conversion, live Williams table-walker animation, and
  final render-audit residuals remain later B11/final-audit work rather than
  reopening B03.

Step 45 acceptance contract:

- B03 is closed as a source visual-state contract. Future palette work must
  name a specific source index/word/rate or a final render-audit residual.
- This step does not claim a hardware palette resistor model or per-pixel
  source palette replay in the clean renderer.
- The contract is proven by focused clean/oracle unit tests and compile checks;
  broad all-scenario clean-fidelity is deferred until Phase 2 closes, a later
  shared contract change, broad-risk change, or R9 finalization.

Scanner/radar contract after Step 46:

- `WorldSnapshot` now carries `ScannerRadarSnapshot`, a source-shaped scanner
  state with the `SCPROC`/`SCP1`/`SCP2` sleep cadence `[2, 2, 4]`, selected
  hardware map `1`, source scan-left calculation, terrain-enabled flag,
  object erase-table addresses starting at `0xB05D`, source `SETEND`, object
  blips, and the player blip bytes `0x9099`, `0x90`, and `0x09`.
- Source object scanner colors now flow through the machine snapshot, accepted
  facade, oracle adapter, and clean object evidence. Clean lander and human
  evidence uses the source-backed scanner color words `0x4433` and `0x6666`;
  projectile rows remain non-scanner rows.
- Clean and oracle playing scenes project scanner object and player blips as
  HUD sprites using source scanner screen-address formulas. This closes B04
  without adding enemy spawning, enemy physics, or source terrain mutation.

Step 46 acceptance contract:

- B04 is closed for source-backed scanner/radar state and scene sprites. Future
  scanner work must name a specific source terrain-raster residual, final
  render-audit residual, or object-ecology dependency instead of reopening broad
  scanner/radar animation.
- This step intentionally does not claim clean enemy spawning/physics,
  terrain-blow mutation, or exact hardware palette/RGB replay.
- Because Step 46 closes Phase 2 and changed shared public state, the next gate
  is the Phase 2 broad validation pass instead of another narrow-only cycle.

Score-popup lifecycle contract after Step 47:

- `ExpandedObjectKind` now has a `ScorePopup` kind. Machine snapshots, the
  accepted facade, the oracle adapter, and clean snapshots carry source-backed
  popup lifetime and value fields for the `P250` / `P500` / `P503` rescue-score
  surface.
- Source `C25P1` and `C5P1` expanded-object rows are classified as score
  popups with a 50-tick source lifetime, 250/500 values, 6x6 descriptor size,
  and mapped `SCORE_POPUP_250` / `SCORE_POPUP_500` sprite identity.
- Clean `WorldSnapshot` can spawn score popups at a source top-left position,
  projects them through the same expanded-object scene path as the oracle, and
  removes them when the 50-frame source lifetime expires. This keeps scoring
  arithmetic unchanged; rescue/abduction entry points that decide when to award
  those values remain part of the later object-ecology cycle.

Step 47 acceptance contract:

- B05 is closed for score-popup spawn state, source lifetime, value, position,
  sprite identity, and clean/oracle scene projection.
- This step does not claim full astronaut rescue/loss object ecology, enemy
  spawning/physics, explosion timing, or terrain-blow behavior.
- The contract is proven by focused clean lifecycle tests, oracle adapter/scene
  tests, source-machine metadata tests, compile checks, and diff hygiene. Broad
  all-scenario clean-fidelity, full fidelity, all-target tests, and clippy stay
  deferred until Phase 3 closes, a broader shared-contract risk appears, or R9
  finalization begins.

Explosion timing contract after Step 48:

- Source `EXST`/`EXPU` expanded-object explosion rows now carry frame/lifetime
  metadata from `RSIZE = 0x0100`, the per-update `+0x00AA` size delta, and the
  source `> 0x30` high-byte kill threshold through the machine snapshot,
  accepted facade, oracle adapter, and clean snapshot surface.
- Clean `WorldSnapshot` can spawn source-shaped expanded-object explosions for
  the mapped `LNDP1`, `BXPIC`, `SWXP1`, `ASXP1`, and `PLAPIC` descriptor
  families. Clean projectile/lander collision now leaves a timed lander
  explosion row and sprite; clean/oracle expanded-object scenes scale explosion
  sprites from the source `RSIZE` high byte.
- Source `PXVCT`/`PX1A` player-death pixel-cloud state now flows through the
  machine snapshot, accepted facade, oracle adapter, clean `WorldSnapshot`, and
  clean/oracle scenes. The clean runtime starts the bank-7-shaped cloud on
  player/enemy contact, preserves the source color table and counter cadence,
  advances visible pieces from the source seed/velocity rules, and renders the
  split-piece 4x2 variant when the source low X byte crosses bit 7.
- B06 is closed for source-backed explosion timing and sprite progression.
  Future object-ecology work may add more gameplay entry points that start the
  already mapped non-lander explosion families, but that belongs to B08 rather
  than reopening the explosion timing contract.

Terrain-blow contract after Step 49:

- Source `TERBLO` terrain-blow state now flows through the machine snapshot,
  accepted facade, oracle adapter, clean `WorldSnapshot`, and clean-fidelity
  comparison. The carried evidence includes the source terrain-blown status
  bit, stage, iteration, sleep, pseudo-color, overload counter,
  terrain/scanner erase entry counts, remaining nonzero terrain words, and
  two-explosions-per-pass contract.
- Clean planet destruction clears clean terrain segments, disables scanner
  terrain, starts source-shaped terrain-blow evidence, and projects `TEREX`
  terrain explosions through the expanded-object sprite path.
- B07 is closed for terrain mutation and presentation evidence. Full
  rescue/abduction object ecology and the live gameplay entry points that
  remove humans remain B08, not terrain-blow presentation.

Object-ecology progress after R9-C4 slices:

- Clean wave spawning no longer uses the old `wave`-number lander shortcut.
  `WorldSnapshot::for_wave` now derives the active enemy batch from the
  source-backed `WaveProfileSnapshot`: active wave size is capped by the
  source wave-size field, wave 1 starts with five active source-state landers,
  and later source-exposed bomber/pod families are admitted into the active
  clean batch with deterministic positions and source-profile-derived
  velocities.
- Clean enemy kind, object-evidence category, sprite, collision-size, score,
  explosion-entry, scanner-color, smart-bomb, projectile-collision, and scene
  rendering paths now know the lander, mutant, bomber, pod, swarmer, and baiter
  families. Active clean enemy object-evidence rows now carry source object
  picture descriptor labels, addresses, sizes, and primary/alternate image
  pointers for the static mutant/pod/swarmer descriptors and the current
  `LNDP`/`UFOP`/`TIEP` frame-cycled lander, baiter, and bomber descriptors. The
  clean human row now carries the source `ASTP1` descriptor label, address,
  2x8 picture size, primary/alternate image pointers, and mapped human sprite
  while the direct runtime renderer keeps the existing clean 6x8 astronaut
  sprite. The clean player projectile row now carries the source `LASP1`
  descriptor label, address, 8x1 picture size, and primary image pointer while
  the direct runtime renderer keeps the existing clean projectile sprite size.
  Clean enemy, human, player-projectile, and enemy-projectile rows now also
  carry source-style 8.8 world-position words, velocity words, and deterministic
  source object-table identity evidence derived from their existing source
  fixed-point state and source layout: addresses from `0xA23C` plus `0x17` per
  slot, source slot numbers, and neutral `OTYP` `0x00`. The wave-profile table
  currently exposes landers, bombers, and pods as initial active wave families,
  while mutant runtime entry now comes from source-shaped clean lander mutation.
- Clean `WorldSnapshot` now carries `EnemyReserveSnapshot` for source-profile
  enemies not in the active batch. Reserve totals flow into inactive object
  evidence counts plus bounded inactive source-detail rows carrying reserved
  family categories, source object-picture descriptors, deterministic source
  object-table identity, mapped clean sprites, and source scanner colors while
  position and velocity remain empty until activation. Smart bombs and
  projectile/player collision remove only the active batch, and the next
  reserve batch activates before `WaveCleared`.
  Reserve lander activation now mirrors source `LANDST` placement, fixed-point
  fractions, shot-timer RNG consumption, and X/Y velocity bytes before restored
  landers continue through a bounded source `LANDS0` orbit/shot loop with
  picture cycling and fireball projection; when no humans remain, the reserve
  lander path follows the source `LANDST` schizoid fallback and restores
  source-shaped mutants directly. Initial clean wave landers now carry
  deterministic source fixed-point fractions, shot timers, picture frame, and
  X/Y velocity into the same bounded `LANDS0` runtime while preserving the
  current active wave count/order. Initial active pods now carry deterministic
  source fixed-point fractions and bounded signed X velocity into the same
  source fixed-point X/Y motion as source-restored pods while preserving the
  current active wave count/order.
- Clean pod destruction now enters a bounded mini-swarmer transition:
  projectile and smart-bomb pod kills share the clean enemy-destroy path, spawn
  the pod explosion, award the source-backed pod score, and append a
  deterministic swarmer batch capped by the source request bound and active
  swarmer limit. Spawned mini-swarmers carry source RNG-derived velocity,
  acceleration, sleep, and shot-timer state, then advance through the source
  entry seek, fixed-point loop, vertical acceleration/damping, turnback, and
  `SWBMB` enemy-bomb projection shape with the same source shell free-list cap
  as other fireball paths, including source `RMAX` RNG consumption on
  shot-timer resets when allocation fails. Reserve mini-swarmer activation now
  mirrors the source `PLRES`/`RSW0` phony-object placement and carries the
  source placement fractions into the same source swarmer runtime. Pod reserve
  activation now mirrors the source `PRBST`/`PRBRES` restore placement and
  velocity bytes, then carries restored pods through source fixed-point
  X/Y motion.
- Clean bomber runtime now carries source-shaped state for active bombers.
  Spawned clean bombers retain source fixed-point fractions, X velocity,
  vertical velocity, picture frame, cruise altitude, and sleep state, then
  advance through the source `TIE` image cycle, random vertical drift/damping,
  on-screen player-Y steering, off-screen cruise-altitude steering, and bounded
  `BOMBST` bomb-shell projection state with source `GETSHL` placement bounds.
  Reserve bomber activation now mirrors
  source `TIEST` squad placement: up to three bombers per squad, X positions
  spaced from the current player X, fixed cruise altitude, and alternating
  source X velocity per restored squad.
- Clean baiter runtime entry now follows the source game-exec pacing shape:
  a 15-frame cadence, low-enemy timer acceleration, zero-enemy wave-clear
  guard, deterministic clean spawn, and source active-baiter cap. Spawned
  baiters retain source shot-timer, picture-cycle, sleep, and velocity state,
  pursue the player through the source `UFONV` seek rules, fire source-shaped
  `SHOOT` fireball shells, and those enemy projectiles now use source lifetime,
  scroll-adjusted fixed-point motion, offscreen culling, collision scoring,
  player-damage handling, and source `BMBP1` shell descriptor evidence.
- Clean mutant runtime now carries source-shaped state for active mutants.
  Completed carried-lander abductions consume the passenger and convert that
  lander into a mutant, no-target/no-human landers convert through the same
  path, and active mutants retain source shot-timer, sleep, fixed-point
  position fractions, source X seek, vertical seek/avoid, random Y hop, and
  shared `SHOOT` fireball projection state. Mutant reserve activation now
  restores active mutants through source-shaped placement fractions and
  shot-timer RNG state.
- Clean lander abduction now covers the first carry transition: aligned humans
  enter the carried state, move with the fleeing lander, and are released when
  that lander is destroyed.
- Released, uncarried humans above terrain now follow source-shaped `AFALL`
  fixed-point acceleration. Safe landings at or below the source velocity
  threshold award the source-backed 250-point score and start the existing
  `P250` score-popup lifecycle; over-speed impacts remove the human, spawn the
  astronaut explosion lifecycle, and feed the existing last-human planet-loss
  handoff.
- Falling humans caught by the player now enter the clean player-carried state,
  award the source-backed 500-point rescue score, and start the existing `P500`
  score-popup lifecycle from the caught astronaut position.
- Player-carried humans now follow the source AFALL2 carried offset and settle
  on terrain when the player-carried position reaches the local terrain line.
- This is a R9-C4 progress slice, not full B08 closure. Remaining Step 50 work
  is remaining per-family movement/projectile behavior and focused source
  ecology fixtures for those transitions.

Work log:

- `2026-05-17 16:03:07 BST` Planning update. Added the post-Step-40
  completion roadmap above so the remaining R9 work is split into explicit
  phases, future steps, and implementation cycles instead of a single broad
  blocker list. This was a docs-only planning update, not an implementation
  cycle.
- `2026-05-17 16:14:12 BST` Archive update. Moved the detailed
  completed DC-164 Step 1-Step 40 implementation history to
  [Completed Plan Step Archive](docs/fidelity/plan-completed-steps-archive.md).
  The active plan now keeps the R9 roadmap, current work-log notes, and the
  next executable steps visible while preserving closed step evidence in the
  archive. Latest archived implementation step: Step 40
  `expanded-object slot sprite presentation slice`, completed with the full
  all-12-scenario `make clean-fidelity` gate matching in 359.57s.
- `2026-05-17 16:10:18 BST` Completed Step 41 / Cycle R9-A1
  `strict blocker matrix and acceptance contract`. Added the blocker matrix
  and acceptance contract above, decomposing broad R9 wording into B01-B13 and
  clarifying that broad validation is deferred to phase boundaries,
  shared-contract changes, broad-risk changes, or R9 closeout. This was a
  docs-only contract-freeze step; no Slack update or Rust validation was
  required.
- `2026-05-17 16:12:14 BST` Completed Step 42 / Cycle R9-A2
  `accepted-surface audit`. Inspected the accepted facade, oracle adapter, and
  source-machine snapshot surfaces, then added the audit table above. The audit
  records which blockers already have neutral evidence and which need
  just-in-time accepted fields or focused fixtures in later cycles. This closes
  Phase 1 as docs-only planning/audit work; no Slack update or Rust validation
  was required.
- `2026-05-17 16:15:15 BST` Completed Step 43 / Cycle R9-B1
  `title/status text sweep`. Inventoried the source-message labels in
  `assets/red-label/messages.tsv` against the clean/oracle projection surfaces
  and recorded the exact included, deferred, and excluded cases above. This was
  a docs-only inventory step; no Slack update or Rust validation was required.
- `2026-05-17 16:31:14 BST` Completed Step 44 / Cycle R9-B2
  `Williams/copyright attract waits and page scheduler`. Added
  `AttractPresentationSnapshot`, gated clean/oracle Williams logo, presents,
  Defender wordmark, copyright, and instruction title-program surfaces from
  source-backed page frames and wait constants, and updated README, SPEC, and
  gaps docs. Focused validation passed: `cargo fmt --check`, `cargo check`,
  focused attract/oracle unit tests, targeted `make clean-fidelity
  SCENARIOS="attract_boot start_game"`, `cargo test --all-targets`,
  markdownlint for touched docs, and `git diff --check` for touched files. Full
  `make fidelity`, full all-scenario `make clean-fidelity`, and clippy were
  deferred until Phase 2 close, a broad-risk/shared-contract change, or R9
  finalization. Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779030996418839`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779031870880909`.
- `2026-05-17 16:47:14 BST` Completed Step 45 / Cycle R9-B3
  `palette, blink, and color contract`. Added `SOURCE_VISUAL_STATE` for the
  source color indices, underline/border words, Williams restore rates, and
  Hall of Fame blink sleep/color evidence; routed clean/oracle HUD, attract,
  top-display-border, and Hall of Fame tints through that contract without
  changing current sprite output; and updated README, SPEC, and gaps docs.
  Focused validation passed: `cargo fmt --check`, `cargo check`,
  `cargo test source_visual_state --lib`, `cargo test
  clean_scene_uses_source_visual_state_tints --lib`, and focused oracle tests
  for attract credits, top-display border, score digits, high-score entry, and
  hall-of-fame display under `--features legacy-tools`. Full
  `cargo test --all-targets`, full `make fidelity`, full all-scenario
  `make clean-fidelity`, and clippy were deferred because this slice preserved
  sprite output and did not close Phase 2 yet. Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779031959663589`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779032827604179`.
- `2026-05-17 17:35:09 BST` Completed Step 46 / Cycle R9-B4
  `scanner/radar animation`. Added source-backed scanner/radar state to
  `WorldSnapshot`, carried source `OBJCOL` scanner colors through the accepted
  facade and oracle adapter, projected scanner object/player blips in clean and
  oracle scenes, added atlas-backed scanner blip sprites, and updated the
  checked live-smoke first/last offscreen signatures for the new renderer
  output. Focused validation passed: `cargo fmt --check`, `cargo check`,
  `cargo check --features legacy-tools`, scanner/radar clean unit tests,
  scanner atlas unit test, focused oracle scanner/object-evidence tests under
  `--features legacy-tools`, focused clean-fidelity object-evidence test under
  `--features legacy-tools`, source scanner raster regression,
  `cargo run -- --game-smoke`, and `cargo run -- --live-smoke`. Phase 2 broad
  validation passed with full all-12-scenario `make clean-fidelity`,
  `make fidelity`, touched-doc markdownlint, and `git diff --check`. The phase
  gate refreshed `tools/new_rust_coverage_baseline.txt` with 77 accepted
  uncovered added executable lines from the current dirty branch state after
  the new-line coverage check exposed existing uncovered R9 branch debt. Slack
  start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779032976703089`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779035751570959`.
- `2026-05-17 17:53:26 BST` Completed Step 47 / Cycle R9-C1
  `score-popup lifecycle`. Added source-backed score-popup metadata to
  expanded-object snapshots across the machine state, accepted facade, oracle
  adapter, and clean state; classified `C25P1` / `C5P1` rows as
  `ScorePopup` with source 50-tick lifetime and 250/500 values; added clean
  score-popup spawn/projection/expiry through the expanded-object scene path;
  and updated README, SPEC, and gaps docs. Focused validation passed:
  `cargo fmt --check`, `cargo check`, `cargo check --features legacy-tools`,
  `cargo test score_popup --features legacy-tools --lib`, `cargo test
  p500_score_sprite_scores_positions_object_and_sleeps_to_cleanup --features
  legacy-tools --lib`, `cargo test expanded_object --features legacy-tools
  --lib`, touched-doc markdownlint, and `git diff --check`. Full
  `cargo test --all-targets`, clippy, `make fidelity`, and full all-scenario
  `make clean-fidelity` were deferred because this slice did not change
  scenario input behavior; the next full gate should run at Phase 3 close,
  broader shared-contract risk, or R9 finalization. Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779035871257009`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779036859089229`.
- `2026-05-17 18:10:59 BST` R9-C2 progress slice
  `source expanded-object explosion timing`. Added source-backed explosion
  frame/lifetime metadata to expanded-object snapshots across the machine
  state, accepted facade, oracle adapter, and clean state; scaled
  expanded-object explosion sprites from the source `RSIZE` high byte; added
  clean explosion lifecycle projection/expiry; and changed projectile/lander
  collision to leave a timed lander explosion sprite. Focused validation
  passed: `cargo fmt --check`, `cargo check`, `cargo check --features
  legacy-tools`, `cargo test explosion --features legacy-tools --lib`,
  `cargo test expanded_object --features legacy-tools --lib`, and `cargo test
  clean_game_resolves_projectile_enemy_collision_and_scores --lib`. Full
  `cargo test --all-targets`, clippy, `make fidelity`, and full all-scenario
  `make clean-fidelity` were deferred because this slice is bounded to the
  source expanded-object explosion surface and does not close Phase 3. The next
  full gate should run at Phase 3 close, broader shared-contract risk, or R9
  finalization. Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779036945983079`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779038105875769`.
- `2026-05-17 18:33:00 BST` Completed Step 48 / Cycle R9-C2
  `explosion timing`. Added source-backed player-death `PXVCT`/`PX1A`
  pixel-cloud snapshots across the machine state, accepted facade, oracle
  adapter, and clean state; started the clean player cloud from player/enemy
  contact; projected visible 4x1 and split 4x2 player-explosion pixels in
  clean/oracle scenes; and updated README, SPEC, and gaps docs. Focused
  validation passed: formatting, default and `legacy-tools` compile checks,
  player-explosion tests under `legacy-tools`, the focused player/enemy
  collision, clean-fidelity comparator, and frame-signature tests, and the
  targeted `death` clean-fidelity scenario. Full
  `cargo test --all-targets`, clippy, `make fidelity`, and full all-scenario
  `make clean-fidelity` were deferred because this closes B06 inside Phase 3
  but does not close the phase or add broad object-ecology risk. The next full
  gate should run at Phase 3 close, broader shared-contract risk, or R9
  finalization. Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779038196326969`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779039329878999`.
- `2026-05-17 18:49:07 BST` Completed Step 49 / Cycle R9-C3
  `terrain-blow presentation`. Added source-backed `TERBLO` terrain-blow
  snapshots across the machine state, accepted facade, oracle adapter, clean
  state, and clean-fidelity comparison; clean planet destruction now clears
  terrain, disables scanner terrain, and projects two `TEREX` terrain
  explosions through expanded-object sprites; and updated README, SPEC, and
  gaps docs. Focused validation passed: formatting, default and
  `legacy-tools` compile checks, terrain-blow tests under `legacy-tools`, the
  focused oracle snapshot adapter test, and targeted `planet_destruction`
  clean-fidelity. Full `cargo test --all-targets`, clippy, `make fidelity`,
  and full all-scenario `make clean-fidelity` were deferred because this
  closes B07 inside Phase 3 but does not close Phase 3. The next full gate
  should run at Phase 3 close, broader shared-contract risk, or R9
  finalization. Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779039405418739`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779040291389669`.
- `2026-05-17 19:06:15 BST` R9-C4 progress slice
  `source-profile active enemy families`. Replaced the clean wave-number
  lander shortcut with source-profile active wave spawning, added clean enemy
  family mappings for lander, mutant, bomber, pod, swarmer, and baiter across
  object evidence, scanner colors, sprites, collision sizes, scores, and
  explosion entry points, cleared the player-death pixel cloud before
  high-score entry handoff, and updated clean start/wave-advance scene
  expectations. Focused validation passed: `cargo fmt --check`, `cargo check`,
  `cargo check --features legacy-tools`, `cargo test
  clean_wave_spawns_source_profile_active_enemy_batch --lib`, `cargo test
  clean_enemy_families_use_source_message_scores_and_sprites --lib`, `cargo
  test clean_game --lib`, and targeted `make clean-fidelity
  SCENARIOS="start_game wave_advance"`. Full `cargo test --all-targets`,
  clippy, `make fidelity`, and full all-scenario `make clean-fidelity` remain
  deferred because this is a bounded B08 progress slice rather than Phase 3
  closure. The next full gate should run when Step 50 closes, a broader
  shared-contract risk appears, or R9 finalization begins. Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779040440745669`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779041310370949`.
- `2026-05-17 19:18:07 BST` R9-C4 progress slice
  `source-profile enemy reserves`. Added `EnemyReserveSnapshot` to clean world
  state, moved non-active source-profile enemies into reserve counts, reported
  those reserves through inactive object evidence, and activated the next
  reserve batch before declaring `WaveCleared`. Focused validation passed:
  `cargo fmt --check`, `cargo check`, `cargo check --features legacy-tools`,
  `cargo test clean_wave_spawns_source_profile_active_enemy_batch --lib`,
  `cargo test clean_game_activates_source_reserve_batch_before_wave_clear
  --lib`, `cargo test clean_game --lib`, and targeted `make clean-fidelity
  SCENARIOS="start_game smart_bomb wave_advance"`. Full
  `cargo test --all-targets`, clippy, `make fidelity`, and full all-scenario
  `make clean-fidelity` remain deferred because this is still bounded B08 work
  rather than Step 50/Phase 3 closure. The next full gate should run when Step
  50 closes, a broader shared-contract risk appears, or R9 finalization begins.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779041484212419`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779041967361759`.
- `2026-05-17 19:28:49 BST` R9-C4 progress slice
  `pod-to-swarmer destruction transition`. Added a shared clean
  enemy-destroy path for projectile and smart-bomb kills, and taught pod
  destruction to append deterministic mini-swarmers capped at the source
  request bound of six and active swarmer limit of twenty. Focused validation
  passed: `cargo fmt --check`, `cargo check`, `cargo check --features
  legacy-tools`, `cargo test clean_game_pod --lib`, `cargo test
  clean_game_smart_bomb_pod_spawns_swarmers_after_destroyed_batch --lib`,
  `cargo test clean_game --lib`, and targeted `make clean-fidelity
  SCENARIOS="smart_bomb wave_advance"`. Full `cargo test --all-targets`,
  clippy, `make fidelity`, and full all-scenario `make clean-fidelity` remain
  deferred because this is a bounded B08 transition slice rather than Step
  50/Phase 3 closure. The next full gate should run when Step 50 closes, a
  broader shared-contract risk appears, or R9 finalization begins. Slack start
  update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779042056730599`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779042622615929`.
- `2026-05-17 19:37:32 BST` R9-C4 progress slice
  `baiter runtime entry`. Added clean source-shaped baiter pacing with the
  15-frame game-exec cadence, low-enemy timer acceleration, zero-enemy
  wave-clear guard, deterministic baiter spawn, and active baiter cap of
  twelve. Focused validation passed: `cargo fmt --check`, `cargo check`,
  `cargo check --features legacy-tools`, `cargo test clean_game_baiter --lib`,
  `cargo test clean_game --lib`, and targeted `make clean-fidelity
  SCENARIOS="start_game wave_advance"`. Full `cargo test --all-targets`,
  clippy, `make fidelity`, and full all-scenario `make clean-fidelity` remain
  deferred because this is a bounded B08 transition slice rather than Step
  50/Phase 3 closure. The next full gate should run when Step 50 closes, a
  broader shared-contract risk appears, or R9 finalization begins. Slack start
  update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779042800321979`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779043169756889`.
- `2026-05-17 19:45:09 BST` R9-C4 progress slice
  `lander abduction/carry/release`. Added clean source-shaped lander capture
  and carry behavior from `LANDG` / `LKIL1` evidence: aligned humans enter the
  carried state, follow the fleeing lander, and are released when the carrying
  lander is destroyed. Focused validation passed: `cargo fmt --check`,
  `cargo check`, `cargo check --features legacy-tools`, `cargo test
  clean_game_lander --lib`, `cargo test
  clean_game_killed_carrying_lander_releases_human --lib`, `cargo test
  clean_game --lib`, and targeted `make clean-fidelity SCENARIOS="abduction"`.
  Full `cargo test --all-targets`, clippy, `make fidelity`, and full
  all-scenario `make clean-fidelity` remain deferred because this is a bounded
  B08 transition slice rather than Step 50/Phase 3 closure. The next full gate
  should run when Step 50 closes, a broader shared-contract risk appears, or
  R9 finalization begins. Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779043290461549`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779043580382719`.
- `2026-05-17 19:52:48 BST` R9-C4 progress slice
  `falling human motion`. Added a bounded clean `AFALL`-shaped falling pass:
  released, uncarried humans above terrain descend each clean frame until they
  reach the local terrain line, while standing humans on terrain remain stable.
  Player catch, rescue scoring, safe/fatal landing, human-loss behavior, and
  exact source falling acceleration remain later B08 work. Focused validation
  passed: `cargo fmt --check`, `cargo check`, `cargo check --features
  legacy-tools`, `cargo test
  clean_game_released_human_falls_until_terrain_landing --lib`, `cargo test
  clean_game_standing_humans_do_not_fall --lib`, `cargo test clean_game
  --lib`, targeted `make clean-fidelity SCENARIOS="abduction"`, `markdownlint
  README.md SPEC.md PLAN.md docs/fidelity/gaps.md`, and `git diff --check`.
  Full `cargo test --all-targets`, clippy, `make fidelity`, and full
  all-scenario `make clean-fidelity` remain deferred because this is a bounded
  B08 transition slice rather than Step 50/Phase 3 closure. The next full gate
  should run when Step 50 closes, a broader shared-contract risk appears, or
  R9 finalization begins. Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779043784742469`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779044027628599`.
- `2026-05-17 20:06:06 BST` R9-C4 progress slice
  `player-catch rescue scoring`. Added the bounded source `AKIL1` / `P500`
  catch path: falling humans that overlap the player enter a clean
  player-carried state, award the source-backed 500-point rescue score through
  `ScoreSystem`, and start the existing `P500` score-popup lifecycle from the
  caught astronaut position. Grounded humans remain uncaught. Focused
  validation passed: `cargo fmt --check`, `cargo check`, `cargo check
  --features legacy-tools`, `cargo test
  clean_game_player_catches_falling_human_scores_and_starts_p500_popup --lib`,
  `cargo test clean_game_player_does_not_catch_grounded_human --lib`,
  `cargo test clean_game_released_lander_passenger_falls_on_following_frame
  --lib`, `cargo test clean_game_released_human_falls_until_terrain_landing
  --lib`, `cargo test clean_game --lib`, `cargo test
  oracle_scene_projects_wave_completion_status_sprites --features legacy-tools
  --lib`, targeted `make clean-fidelity SCENARIOS="abduction"`, touched-doc
  markdownlint, `git diff --check`, and the broader public-contract check
  `cargo test --all-targets`. Clippy, `make fidelity`, and full all-scenario
  `make clean-fidelity` remain deferred because this is a bounded B08
  transition slice rather than Step 50/Phase 3 closure. The next full gate
  should run when Step 50 closes, a broader shared-contract risk appears, or
  R9 finalization begins. Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779044512076799`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779044929473269`.
  Supplemental broad-validation update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779044996769709`.
- `2026-05-17 20:11:43 BST` R9-C4 progress slice
  `AFALL2 player-carried landing`. Added the bounded source-shaped
  player-carried landing transition: humans caught by the player continue to
  follow the clean AFALL2 carried offset and settle onto terrain when that
  carried position reaches the local terrain line, without creating a second
  rescue score popup. Focused validation passed: `cargo fmt --check`, `cargo
  check`, `cargo check --features legacy-tools`, `cargo test
  clean_game_player_carried_human_lands_when_carried_to_terrain --lib`,
  `cargo test
  clean_game_player_catches_falling_human_scores_and_starts_p500_popup --lib`,
  `cargo test clean_game_player_does_not_catch_grounded_human --lib`, `cargo
  test clean_game --lib`, targeted `make clean-fidelity
  SCENARIOS="abduction"`, touched-doc markdownlint, and `git diff --check`.
  Full `cargo test --all-targets`, clippy, `make fidelity`, and full
  all-scenario `make clean-fidelity` remain deferred because this is a bounded
  B08 transition slice rather than Step 50/Phase 3 closure. The next full gate
  should run when Step 50 closes, a broader shared-contract risk appears, or
  R9 finalization begins. Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779045053880489`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779045198181889`.

- `2026-05-17 20:17:56 BST` R9-C4 progress slice
  `AFALL safe-landing scoring`. Added the bounded source `AFALL` safe-landing
  score transition: released, uncarried humans that settle on terrain now award
  the source-backed 250-point safe-landing score through `ScoreSystem` and
  start the existing `P250` score-popup lifecycle from the settled astronaut
  position. Focused validation passed: `cargo fmt --check`, `cargo check`,
  `cargo check --features legacy-tools`, `cargo test
  clean_game_released_human_falls_until_terrain_landing --lib`, `cargo test
  clean_game_standing_humans_do_not_fall --lib`, `cargo test clean_game --lib`,
  targeted `make clean-fidelity SCENARIOS="abduction"`, touched-doc
  markdownlint, and `git diff --check`. Full `cargo test --all-targets`,
  clippy, `make fidelity`, and full all-scenario `make clean-fidelity` remain
  deferred because this is a bounded B08 transition slice rather than Step
  50/Phase 3 closure. The next full gate should run when Step 50 closes, a
  broader shared-contract risk appears, or R9 finalization begins. Slack start
  update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779045246819539`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779045475425419`.
- `2026-05-17 20:26:24 BST` R9-C4 progress slice
  `AFALL falling acceleration and fatal landing`. Added source-shaped clean
  falling-human velocity state: uncarried falling humans now accelerate by
  `0x0008`, preserve the source max-velocity clamp before `0x0300`, safe-land
  at or below `0x00E0` through the existing `P250` score path, and remove
  over-speed impacts with an astronaut explosion plus the existing last-human
  terrain-blow handoff. Focused validation passed: `cargo fmt --check`, `cargo
  check`, `cargo check --features legacy-tools`, `cargo test
  clean_game_released_lander_passenger_falls_on_following_frame --lib`, `cargo
  test clean_game_released_human_falls_until_terrain_landing --lib`, `cargo test
  clean_game_fatal_falling_human_impact_removes_human_and_starts_human_loss
  --lib`, `cargo test
  clean_game_player_catches_falling_human_scores_and_starts_p500_popup --lib`,
  `cargo test clean_game --lib`, targeted `make clean-fidelity
  SCENARIOS="abduction"`, touched-doc markdownlint, and `git diff --check`.
  Broad `cargo test --all-targets` also passed because `HumanSnapshot` gained
  public clean-state fields. Clippy, `make fidelity`, and full all-scenario
  `make clean-fidelity` remain deferred because this is a bounded B08 transition
  slice and the shared-contract risk was covered by the all-target Rust test
  pass. The next full gate should run when Step 50 closes, another broader
  shared-contract risk appears, or R9 finalization begins. Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779045515879429`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779045984811539`.
- `2026-05-17 21:59:16 BST` R9-C4 progress slice
  `source mini-swarmer runtime`. Added source-backed clean mini-swarmer state:
  pod-triggered spawns now retain RNG-derived velocity, acceleration, sleep,
  and shot-timer fields; active source swarmers advance through the entry seek,
  fixed-point position/fraction updates, loop sleep cadence, vertical
  acceleration/damping, turnback reseek, and bounded enemy-bomb projection.
  Clean fidelity now compares enemy projectiles. Focused validation passed:
  `cargo fmt --check`, `cargo check`, `cargo check --features legacy-tools`,
  `cargo test
  clean_game_pod_projectile_collision_spawns_source_bounded_swarmers --lib`,
  `cargo test clean_game_pod_swarmer_spawn_respects_source_active_limit --lib`,
  `cargo test clean_game_mini_swarmer --lib`, `cargo test clean_game --lib`,
  targeted `make clean-fidelity SCENARIOS="smart_bomb"`, touched-doc
  markdownlint, and `git diff --check`. Broad `cargo test --all-targets` also
  passed because `EnemySnapshot`, `WorldSnapshot`, and clean-fidelity state
  gained public clean-state fields. Clippy, `make fidelity`, and full
  all-scenario `make clean-fidelity` remain deferred because this is a bounded
  B08 transition slice and the public-contract risk was covered by the
  all-target Rust test pass. The next full gate should run when Step 50 closes,
  another broader shared-contract risk appears, or R9 finalization begins.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779046194681939`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779051554385829`.
- `2026-05-17 22:11:08 BST` R9-C4 progress slice
  `source baiter movement and fireballs`. Added source-backed clean baiter
  state: spawned baiters now retain source shot-timer, picture-cycle, sleep,
  and velocity fields; active baiters pursue through the source `UFONV` seek
  rules and fire source-shaped `SHOOT` fireball shells. Enemy projectiles now
  carry source shell lifetime, source offscreen culling, 25-point collision
  scoring, bomb explosion entry, and player-damage handling. Focused validation
  passed: `cargo fmt --check`, `cargo check`, `cargo check --features
  legacy-tools`, `cargo test clean_game_baiter --lib`, `cargo test
  clean_game_enemy_projectile --lib`, `cargo test clean_game --lib`, targeted
  `make clean-fidelity SCENARIOS="wave_advance"`, touched-doc markdownlint,
  and `git diff --check`. Broad `cargo test --all-targets` also passed because
  `EnemySnapshot`, `SourceBaiterSnapshot`, and `EnemyProjectileSnapshot` gained
  public clean-state fields. Clippy, `make fidelity`, and full all-scenario
  `make clean-fidelity` remain deferred because this is a bounded B08
  transition slice and the public-contract risk was covered by the all-target
  Rust test pass. The next full gate should run when Step 50 closes, another
  broader shared-contract risk appears, or R9 finalization begins. Slack start
  update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779051619228369`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779052266782509`.
- `2026-05-17 22:22:22 BST` R9-C4 progress slice
  `source mutant runtime and lander conversion`. Added clean source-mutant
  state for active mutants. Completed carried-lander abductions now consume the
  passenger and convert that lander into a source-shaped mutant, no-target and
  no-human landers enter the same mutation path, and active mutants retain
  source shot-timer, sleep, fixed-point fractions, X seek, vertical seek/avoid,
  random Y hop, and shared `SHOOT` fireball projection state. Focused
  validation passed: `cargo fmt --check`, `cargo check`, `cargo check
  --features legacy-tools`, `cargo test clean_game_mutant --lib`, `cargo test
  mutant --lib`, `cargo test clean_game --lib`, targeted `make
  clean-fidelity SCENARIOS="abduction wave_advance"`, touched-doc
  markdownlint, and `git diff --check`. Broad `cargo test --all-targets` also
  passed because `EnemySnapshot` gained a public clean-state field. Clippy,
  `make fidelity`, and full all-scenario `make clean-fidelity` remain deferred
  because this is a bounded B08 transition slice and the public-contract risk
  was covered by the all-target Rust test pass. The next full gate should run
  when Step 50 closes, another broader shared-contract risk appears, or R9
  finalization begins. Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779052381477969`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779052941208549`.
- `2026-05-17 22:31:24 BST` R9-C4 progress slice
  `source bomber movement and bomb shells`. Added clean source-bomber state:
  wave-spawned bombers now retain source fixed-point fractions, X velocity,
  vertical velocity, picture frame, cruise altitude, and sleep fields; active
  bombers follow the source `TIE` picture cycle, random vertical drift/damping,
  on-screen player-Y steering, off-screen cruise-altitude steering, and
  bounded `BOMBST` bomb-shell projection. Focused validation passed: `cargo
  fmt --check`, `cargo check`, `cargo check --features legacy-tools`, `cargo
  test clean_game_bomber --lib`, `cargo test clean_game --lib`, targeted `make
  clean-fidelity SCENARIOS="wave_advance smart_bomb"`, touched-doc
  markdownlint, and `git diff --check`. Broad `cargo test --all-targets` also
  passed because `EnemySnapshot` gained another public clean-state field.
  Clippy, `make fidelity`, and full all-scenario `make clean-fidelity` remain
  deferred because this is a bounded B08 transition slice and the
  public-contract risk was covered by the all-target Rust test pass. The next
  full gate should run when Step 50 closes, another broader shared-contract
  risk appears, or R9 finalization begins. Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779053066041699`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779053483585039`.
- `2026-05-17 22:39:26 BST` R9-C4 progress slice
  `mutant reserve restore fixture`. Clean reserve mutant activation now uses
  source-shaped placement fractions and shot-timer RNG state, carrying restored
  mutants into the existing source-shaped mutant runtime loop. Added focused
  clean-game coverage for the restored mutant position, velocity, source state,
  RNG consumption, and reserve drain. Focused validation passed: `cargo fmt
  --check`, `cargo check`, `cargo check --features legacy-tools`, `cargo test
  clean_game_mutant --lib`, `cargo test clean_game --lib`, targeted `make
  clean-fidelity SCENARIOS="wave_advance"`, touched-doc markdownlint, and `git
  diff --check`. Broad `cargo test --all-targets`, clippy, `make fidelity`, and
  full all-scenario `make clean-fidelity` remain deferred because this is a
  bounded B08 transition slice with no public API change. The next full gate
  should run when Step 50 closes, another broader shared-contract risk appears,
  or R9 finalization begins. Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779053705176209`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779053965306839`.
- `2026-05-17 22:47:50 BST` R9-C4 progress slice
  `source-shell/mine descriptor fixture`. Clean enemy projectile evidence now
  carries the source `BMBP1` shell descriptor address, 2x3 picture size,
  primary/alternate embedded image addresses, and `ENEMY_BOMB` sprite mapping,
  while direct clean projectile rendering still uses the existing 4x6 runtime
  bomb sprite without duplicating clean evidence rows. Focused validation
  passed: `cargo fmt --check`, `cargo check`, `cargo check --features
  legacy-tools`, `cargo test clean_game_enemy_projectile --lib`, `cargo test
  clean_game --lib`, touched-doc markdownlint, and `git diff --check`.
  Targeted clean-fidelity, broad `cargo test --all-targets`, clippy, `make
  fidelity`, and full all-scenario `make clean-fidelity` remain deferred
  because this is a bounded evidence/fixture slice with no scenario input
  behavior or public API change. The next full gate should run when Step 50
  closes, another broader shared-contract risk appears, or R9 finalization
  begins. Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779054048626429`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779054469441199`.
- `2026-05-17 22:51:15 BST` R9-C4 progress slice
  `pod reserve restore fixture`. Clean reserve pod activation now uses the
  source `PRBST`/`PRBRES` restore placement and signed velocity bytes for each
  restored pod while leaving initial wave pod placement and public snapshots
  unchanged. Focused validation passed: `cargo fmt --check`, `cargo check`,
  `cargo check --features legacy-tools`, `cargo test clean_game_pod --lib`,
  `cargo test clean_game --lib`, targeted `make clean-fidelity
  SCENARIOS="wave_advance"`, touched-doc markdownlint, and `git diff --check`.
  Broad `cargo test --all-targets`, clippy, `make fidelity`, and full
  all-scenario `make clean-fidelity` remain deferred because this is a bounded
  reserve-restore fixture with no public API change. The next full gate should
  run when Step 50 closes, another broader shared-contract risk appears, or R9
  finalization begins. Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779054616891509`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779054897742059`.
- `2026-05-17 22:57:30 BST` R9-C4 progress slice
  `mini-swarmer reserve restore fixture`. Clean reserve swarmer activation now
  groups selected reserve mini-swarmers behind the source `PLRES`/`RSW0`
  phony-object placement shape, preserves the targetless low X byte as source
  placement fraction state, and carries each restored swarmer into the existing
  source swarmer runtime. Focused validation passed: `cargo fmt --check`,
  `cargo check`, `cargo check --features legacy-tools`, `cargo test
  clean_game_swarmer_reserve_batch_uses_source_plres_restore_state --lib`,
  `cargo test clean_game_mini_swarmer --lib`, `cargo test clean_game_swarmer
  --lib`, `cargo test clean_game --lib`, targeted `make clean-fidelity
  SCENARIOS="wave_advance"`, touched-doc markdownlint, and `git diff --check`.
  Broad `cargo test --all-targets`, clippy, `make fidelity`, and full
  all-scenario `make clean-fidelity` remain deferred because this is a bounded
  reserve-restore fixture with no public API change. The next full gate should
  run when Step 50 closes, another broader shared-contract risk appears, or R9
  finalization begins. Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779055046636789`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779055264360129`.
- `2026-05-17 23:02:58 BST` R9-C4 progress slice
  `lander reserve LANDST fixture`. Clean reserve lander activation now uses the
  source `LANDST` restore shape for placement, shot-timer RNG consumption, and
  signed X/Y velocity bytes while leaving initial wave lander placement and
  public snapshots unchanged. Focused validation passed: `cargo fmt --check`,
  `cargo check`, `cargo check --features legacy-tools`, `cargo test
  clean_game_lander_reserve_batch_uses_source_landst_spawn_state --lib`,
  `cargo test clean_game_lander --lib`, `cargo test clean_game --lib`,
  targeted `make clean-fidelity SCENARIOS="wave_advance"`, touched-doc
  markdownlint, and `git diff --check`. Broad `cargo test --all-targets`,
  clippy, `make fidelity`, and full all-scenario `make clean-fidelity` remain
  deferred because this is a bounded reserve-activation fixture with no public
  API change. The next full gate should run when Step 50 closes, another
  broader shared-contract risk appears, or R9 finalization begins. Slack start
  update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779055366320629`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779055578253799`.
- `2026-05-17 23:11:17 BST` R9-C4 progress slice
  `bomber reserve TIEST fixture`. Clean reserve bomber activation now uses the
  source `TIEST` restore shape for player-relative squad placement, fixed
  cruise altitude, and alternating signed X velocity per restored squad while
  carrying each restored bomber into the existing source bomber runtime.
  Focused validation passed: `cargo fmt --check`, `cargo check`, `cargo check
  --features legacy-tools`, `cargo test
  clean_game_bomber_reserve_batch_uses_source_tiest_restore_state --lib`,
  `cargo test clean_game_bomber --lib`, `cargo test clean_game --lib`,
  targeted `make clean-fidelity SCENARIOS="wave_advance"`, touched-doc
  markdownlint, and `git diff --check`. Broad `cargo test --all-targets`,
  clippy, `make fidelity`, and full all-scenario `make clean-fidelity` remain
  deferred because this is a bounded reserve-activation fixture with no public
  API change. The next full gate should run when Step 50 closes, another
  broader shared-contract risk appears, or R9 finalization begins. Slack start
  update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779055701950759`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779056053054829`.
- `2026-05-17 23:21:18 BST` R9-C4 progress slice
  `source lander runtime`. Clean reserve landers now carry public
  `SourceLanderSnapshot` state from the source `LANDST` restore path:
  fixed-point fractions, shot timer, sleep ticks, picture frame, and X/Y
  velocity. Restored landers advance through a bounded source `LANDS0`
  orbit/shot loop with terrain-relative Y velocity, `LSHOT` fireball
  projection, `LNDP1`-`LNDP3` picture cycling, and a source-shaped flee
  velocity when a passenger is already carried. Focused validation passed:
  `cargo fmt --check`, `cargo check`, `cargo check --features legacy-tools`,
  `cargo test clean_game_lander_source_loop_orbits_cycles_and_fires --lib`,
  `cargo test clean_game_lander_reserve_batch_uses_source_landst_spawn_state
  --lib`, `cargo test clean_game_lander --lib`, `cargo test clean_game --lib`,
  targeted `make clean-fidelity SCENARIOS="wave_advance"`, touched-doc
  markdownlint, and `git diff --check`. Because the clean snapshot contract
  changed, broad `cargo test --all-targets` also passed. Broad clippy,
  `make fidelity`, and full all-scenario `make clean-fidelity` remain deferred
  because this is still a bounded Step 50 slice rather than Step 50/Phase 3
  closure. The next full gate should run when Step 50 closes, another broader
  shared-contract risk appears, or R9 finalization begins. Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779056244093689`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779056645123769`.
- `2026-05-17 23:28:01 BST` R9-C4 progress slice
  `source pod fixed-point runtime`. Clean reserve pods now carry public
  `SourcePodSnapshot` state from the source `PRBST`/`PRBRES` restore path:
  fixed-point fractions plus signed X/Y velocity bytes. Restored pods now
  advance through source fixed-point object motion instead of the previous
  pixel-velocity projection, while initial clean wave pods stay on the current
  clean placement path. Focused validation passed: `cargo fmt --check`,
  `cargo check`, `cargo check --features legacy-tools`, `cargo test
  clean_game_pod_source_motion_uses_fixed_point_velocity --lib`, `cargo test
  clean_game_pod --lib`, `cargo test clean_game --lib`, targeted `make
  clean-fidelity SCENARIOS="wave_advance"`, touched-doc markdownlint, and
  `git diff --check`. Because the clean snapshot contract changed, broad
  `cargo test --all-targets` also passed. Broad clippy, `make fidelity`, and
  full all-scenario `make clean-fidelity` remain deferred because this is still
  a bounded Step 50 slice rather than Step 50/Phase 3 closure. The next full
  gate should run when Step 50 closes, another broader shared-contract risk
  appears, or R9 finalization begins. Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779056772315609`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779057029455259`.
- `2026-05-17 23:36:48 BST` R9-C4 progress slice
  `initial source lander runtime`. Initial active wave landers now carry
  deterministic source `SourceLanderSnapshot` state: fixed-point fractions,
  shot timer, sleep ticks, picture frame, and X/Y velocity. They enter the same
  bounded source `LANDS0` orbit/shot loop used by `LANDST`-restored landers
  while preserving active wave count/order. Focused validation passed:
  `cargo fmt --check`, `cargo check`, `cargo check --features legacy-tools`,
  `cargo test clean_wave_spawns_source_profile_active_enemy_batch --lib`,
  `cargo test clean_game_lander --lib`, `cargo test clean_game --lib`,
  targeted `make clean-fidelity SCENARIOS="start_game abduction
  wave_advance"`, touched-doc markdownlint, and `git diff --check`. Broad
  `cargo test --all-targets`, clippy, `make fidelity`, and full all-scenario
  `make clean-fidelity` remain deferred because this slice changes behavior but
  does not change public API, close Step 50, or introduce a broad-risk contract
  change. The next full gate should run when Step 50 closes, another broader
  shared-contract risk appears, or R9 finalization begins. Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779057126727139`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779057486087689`.
- `2026-05-17 23:44:55 BST` R9-C4 progress slice
  `initial source pod runtime`. Initial active wave pods now carry
  deterministic source `SourcePodSnapshot` state: fixed-point fractions,
  bounded signed X velocity, and zero Y velocity. They enter the same source
  fixed-point X/Y motion already used by `PRBST`/`PRBRES`-restored pods while
  preserving active wave count/order. Focused validation passed:
  `cargo fmt --check`, `cargo check`, `cargo check --features legacy-tools`,
  `cargo test clean_wave_spawns_source_profile_active_enemy_batch --lib`,
  `cargo test clean_game_pod --lib`, `cargo test clean_game --lib`, targeted
  `make clean-fidelity SCENARIOS="wave_advance"`, touched-doc markdownlint,
  and `git diff --check`. Broad `cargo test --all-targets`, clippy,
  `make fidelity`, and full all-scenario `make clean-fidelity` remain deferred
  because this slice changes behavior but does not change public API, close
  Step 50, or introduce a broad-risk contract change. The next full gate should
  run when Step 50 closes, another broader shared-contract risk appears, or R9
  finalization begins. Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779057628043729`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779057894434819`.
- `2026-05-17 23:53:19 BST` R9-C4 progress slice
  `LANDST no-human fallback`. Clean reserve lander activation now mirrors the
  source no-astronaut `LANDST` fallback by restoring source-shaped mutants
  directly through the `SCZS0`/`SCZST` placement and shot-timer RNG path. Mutant
  reserve placement now uses the current clean background/camera word for the
  source avoid window. Focused validation passed: `cargo fmt --check`,
  `cargo check`, `cargo check --features legacy-tools`, `cargo test
  clean_game_lander_reserve --lib`, `cargo test
  clean_game_mutant_reserve_batch_uses_source_restore_state --lib`, `cargo test
  clean_game --lib`, targeted `make clean-fidelity
  SCENARIOS="planet_destruction wave_advance"`, touched-doc markdownlint, and
  `git diff --check`. Broad `cargo test --all-targets`, clippy,
  `make fidelity`, and full all-scenario `make clean-fidelity` remain deferred
  because this slice changes behavior but does not change public API, close
  Step 50, or introduce a broad-risk contract change. The next full gate should
  run when Step 50 closes, another broader shared-contract risk appears, or R9
  finalization begins. Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779058118394679`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779058514140589`.
- `2026-05-18 00:02:23 BST` R9-C4 progress slice
  `active enemy source picture evidence`. Clean active enemy object-evidence
  rows now carry source object-picture descriptor labels, addresses, sizes, and
  primary/alternate image pointers for static mutant/pod/swarmer descriptors
  plus current lander, baiter, and bomber frame-cycled descriptors. This keeps
  rendering behavior on the clean sprite path while preserving source
  descriptor evidence for object ecology comparisons. Focused validation
  passed: `cargo fmt --check`, `cargo check`, `cargo check --features
  legacy-tools`, `cargo test clean_world_maps_active_enemy_source_picture_descriptors
  --lib`, `cargo test clean_game --lib`, targeted `make clean-fidelity
  SCENARIOS="wave_advance"`, touched-doc markdownlint, and `git diff --check`.
  Broad `cargo test --all-targets`, clippy, `make fidelity`, and full
  all-scenario `make clean-fidelity` remain deferred because this slice changes
  clean object evidence but does not change public API, close Step 50, or
  introduce a broad-risk contract change. The next full gate should run when
  Step 50 closes, another broader shared-contract risk appears, or R9
  finalization begins. Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779058757810019`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779059117750519`.
- `2026-05-18 00:09:48 BST` R9-C4 progress slice
  `player projectile source picture evidence`. Clean player projectile
  object-evidence rows now carry the source `LASP1` descriptor label, address,
  8x1 picture size, and primary image pointer while the direct clean runtime
  projectile renderer keeps the existing 8x2 sprite. Focused validation
  passed: `cargo fmt --check`, `cargo check`, `cargo check --features
  legacy-tools`, `cargo test
  clean_game_player_projectile_evidence_uses_source_laser_picture --lib`,
  `cargo test clean_game_enemy_projectile_evidence_uses_source_shell_picture
  --lib`, `cargo test clean_game --lib`, touched-doc markdownlint, and
  `git diff --check`. Targeted clean-fidelity scenarios were not run because
  this slice changes source descriptor evidence only, not scenario behavior.
  Broad `cargo test --all-targets`, clippy, `make fidelity`, and full
  all-scenario `make clean-fidelity` remain deferred because this slice changes
  clean object evidence but does not change public API, close Step 50, or
  introduce a broad-risk contract change. The next full gate should run when
  Step 50 closes, another broader shared-contract risk appears, or R9
  finalization begins. Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779059281878429`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779059468093189`.
- `2026-05-18 00:16:08 BST` R9-C4 progress slice
  `astronaut source picture evidence`. Clean human object-evidence rows now
  carry the source `ASTP1` descriptor label, address, 2x8 picture size,
  primary/alternate image pointers, mapped human sprite evidence, and source
  scanner color while the direct clean runtime astronaut renderer keeps the
  existing 6x8 sprite. Focused validation passed: `cargo fmt --check`, `cargo
  check`, `cargo check --features legacy-tools`, `cargo test
  clean_game_human_evidence_uses_source_astronaut_picture --lib`, `cargo test
  clean_game --lib`, touched-doc markdownlint, and `git diff --check`.
  Targeted clean-fidelity scenarios were not run because this slice changes
  source descriptor evidence only, not scenario behavior. Broad `cargo test
  --all-targets`, clippy, `make fidelity`, and full all-scenario `make
  clean-fidelity` remain deferred because this slice changes clean object
  evidence but does not change public API, close Step 50, or introduce a
  broad-risk contract change. The next full gate should run when Step 50
  closes, another broader shared-contract risk appears, or R9 finalization
  begins. Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779059627067949`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779059841972379`.
- `2026-05-18 00:25:55 BST` R9-C4 progress slice
  `source motion object evidence`. Clean enemy, human, player-projectile, and
  enemy-projectile object-evidence rows now carry source-style 8.8
  world-position words and velocity words from their existing clean source
  fixed-point state while leaving runtime scenes and scenario behavior
  unchanged. Focused validation passed: `cargo fmt --check`, `cargo check`,
  `cargo check --features legacy-tools`, `cargo test
  clean_world_object_evidence_carries_source_motion_words --lib`, `cargo test
  clean_game_human_evidence_uses_source_astronaut_picture --lib`, `cargo test
  clean_game --lib`, touched-doc markdownlint, and `git diff --check`.
  Targeted clean-fidelity scenarios were not run because this slice changes
  source evidence fields only, not scenario behavior. Broad `cargo test
  --all-targets`, clippy, `make fidelity`, and full all-scenario `make
  clean-fidelity` remain deferred because this slice changes clean object
  evidence but does not change public API, close Step 50, or introduce a
  broad-risk contract change. The next full gate should run when Step 50
  closes, another broader shared-contract risk appears, or R9 finalization
  begins. Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779060171604429`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779060444641219`.
- `2026-05-20 20:29:31 BST` R9-C4 progress slice
  `source object-table identity evidence`. Clean enemy, human,
  player-projectile, and enemy-projectile object-evidence rows now carry
  deterministic source-layout addresses from `0xA23C + 0x17 * slot`, source
  slot numbers, and neutral `OTYP` `0x00` while clean categorized source-detail
  rows remain skipped by the direct clean scene projection to avoid duplicate
  runtime sprites. Focused validation passed: `cargo fmt --check`, `cargo
  check`, `cargo check --features legacy-tools`, `cargo test
  clean_world_object_evidence_carries_source_motion_words --lib`, `cargo test
  clean_world_maps_active_enemy_source_picture_descriptors --lib`, `cargo test
  clean_game_human_evidence_uses_source_astronaut_picture --lib`, `cargo test
  clean_game_player_projectile_evidence_uses_source_laser_picture --lib`,
  `cargo test clean_game_enemy_projectile_evidence_uses_source_shell_picture
  --lib`, `cargo test clean_game --lib`, touched-doc markdownlint, and `git
  diff --check`. Targeted clean-fidelity scenarios were not run because this
  slice changes source evidence fields only, not scenario behavior. Broad
  `cargo test --all-targets`, clippy, `make fidelity`, and full all-scenario
  `make clean-fidelity` remain deferred because this slice changes clean object
  evidence but does not change public API, close Step 50, or introduce a
  broad-risk contract change. The next full gate should run when Step 50
  closes, another broader shared-contract risk appears, or R9 finalization
  begins. Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779305212541739`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779305464868959`.
- `2026-05-20 20:35:45 BST` R9-C4 progress slice
  `reserve inactive object-evidence details`. Clean reserve enemy totals now
  expand into bounded `ObjectEvidenceList::Inactive` detail rows after the
  active/projectile detail rows. The inactive reserve rows carry reserved
  family categories, source object-picture descriptors, deterministic
  source-layout addresses from `0xA23C + 0x17 * slot`, source slot numbers,
  neutral `OTYP` `0x00`, mapped clean sprites, and source scanner colors while
  leaving screen/world position and velocity empty until activation. Focused
  validation passed: `cargo fmt --check`, `cargo check`, `cargo check
  --features legacy-tools`, `cargo test
  clean_world_object_evidence_carries_reserve_inactive_source_details --lib`,
  `cargo test object_evidence --lib`, `cargo test clean_game --lib`, `cargo
  test clean_wave_spawns_source_profile_active_enemy_batch --lib`, touched-doc
  markdownlint, and `git diff --check`. Targeted clean-fidelity scenarios were
  not run because this slice changes source evidence fields only, not scenario
  behavior. Broad `cargo test --all-targets`, clippy, `make fidelity`, and full
  all-scenario `make clean-fidelity` remain deferred because this slice changes
  clean object evidence but does not close Step 50 or introduce a broad-risk
  contract change. The next full gate should run when Step 50 closes, another
  broader shared-contract risk appears, or R9 finalization begins. Slack start
  update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779305752169999`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779306028776519`.
- `2026-05-20 20:44:12 BST` R9-C4 progress slice
  `mini-swarmer source shell cap`. The clean `SWBMB` fireball path now receives
  the current enemy-projectile count and refuses to append another shell when
  the source shell free-list cap is already full. This preserves the existing
  direction gate, shot-timer reset, and source RNG behavior while closing the
  full-shell edge case for mini-swarmer bombs. Focused validation passed:
  `cargo fmt --check`, `cargo check`, `cargo check --features legacy-tools`,
  `cargo test clean_game_mini_swarmer --lib`, `cargo test clean_game --lib`,
  targeted `make clean-fidelity SCENARIOS="wave_advance"`, touched-doc
  markdownlint, and `git diff --check`. Broad `cargo test --all-targets`,
  clippy, `make fidelity`, and full all-scenario `make clean-fidelity` remain
  deferred because this is a bounded Step 50 projectile-cap slice, not Step 50
  or Phase 3 closure. The next full gate should run when Step 50 closes,
  another broader shared-contract risk appears, or R9 finalization begins.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779306259354109`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779307987920709`.
- `2026-05-20 21:22:01 BST` R9-C4 progress slice
  `mini-swarmer full-shell RMAX parity`. The clean `SWBMB` fireball attempt
  now resets the mini-swarmer shot timer through `source_advance_rmax`, so
  source RNG is consumed on every shot reset path, including when the shell
  free-list is full and no fireball cell is allocated. The bomb helper now
  only decides whether a projectile is allocated; the caller owns the source
  `RMAX` reseed just like the source fall-through path. Focused validation
  passed: `cargo fmt --check`, `cargo check`, `cargo check --features
  legacy-tools`, `cargo test clean_game_mini_swarmer --lib`, `cargo test
  clean_game --lib`, targeted `make clean-fidelity SCENARIOS="wave_advance"`,
  touched-doc markdownlint, and `git diff --check`. Broad `cargo test
  --all-targets`, clippy, `make fidelity`, and full all-scenario `make
  clean-fidelity` remain deferred because this is a bounded Step 50
  mini-swarmer RNG edge slice, not Step 50 or Phase 3 closure. The next full
  gate should run when Step 50 closes, another broader shared-contract risk
  appears, or R9 finalization begins. Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779308272642439`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779308516667089`.
- `2026-05-20 21:27:46 BST` R9-C4 progress slice
  `bomber BOMBST GETSHL bounds`. Clean bomber `BOMBST` shell allocation now
  applies the source `GETSHL` placement bounds as well as the existing
  ten-shell cap: bomb shells are refused when the firing bomber is outside the
  source shell screen range or at/above the source playfield top. Added focused
  clean-game coverage for the out-of-range bomber allocation edge. Focused
  validation passed: `cargo fmt --check`, `cargo check`, `cargo check
  --features legacy-tools`, `cargo test clean_game_bomber --lib`, `cargo test
  clean_game --lib`, targeted `make clean-fidelity SCENARIOS="wave_advance"`,
  touched-doc markdownlint, and `git diff --check`. Broad `cargo test
  --all-targets`, clippy, `make fidelity`, and full all-scenario `make
  clean-fidelity` remain deferred because this is a bounded Step 50 bomber
  shell-allocation edge slice, not Step 50 or Phase 3 closure. The next full
  gate should run when Step 50 closes, another broader shared-contract risk
  appears, or R9 finalization begins. Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779308693691909`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779308862431699`.
- `2026-05-20 21:34:41 BST` R9-C4 progress slice
  `source SHELL scroll delta`. Clean hostile shell movement now applies the
  source `SHELL` X scroll term, adding `(previous BGL - current BGL) << 2` to
  fixed-point X motion when the clean camera/background moves. Added focused
  clean-game coverage for camera-scroll shell movement while preserving source
  lifetime, Y/X culling, collision, and shell evidence behavior. Focused
  validation passed: `cargo fmt --check`, `cargo check`, `cargo check
  --features legacy-tools`, `cargo test clean_game_enemy_projectile --lib`,
  `cargo test clean_game --lib`, targeted `make clean-fidelity
  SCENARIOS="wave_advance"`, touched-doc markdownlint, and `git diff --check`.
  Broad `cargo test --all-targets`, clippy, `make fidelity`, and full
  all-scenario `make clean-fidelity` remain deferred because this is a bounded
  Step 50 hostile-shell motion slice, not Step 50 or Phase 3 closure. The next
  full gate should run when Step 50 closes, another broader shared-contract
  risk appears, or R9 finalization begins. Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779309053099619`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779309277857069`.

## Archived Completed History

Detailed closed-step and completed-cycle history has been moved to
[Completed Plan Step Archive](docs/fidelity/plan-completed-steps-archive.md).
Keep `PLAN.md` focused on current baseline, rewrite target, active R9 roadmap,
and ongoing work. Add active-cycle notes above; move them to the archive only
after they are closed and no longer needed for day-to-day planning.

## Ongoing Work

- Keep `README.md`, `SPEC.md`, and `PLAN.md` synchronized with CLI help,
  Makefile targets, workflows, and module boundaries.
- Keep added executable Rust lines covered or explicitly refresh the accepted
  uncovered baseline only when accepting existing debt.
- Keep Slack start and completion notes linked in each dev-cycle work log.
