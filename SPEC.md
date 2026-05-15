# Defender Current Specification

Last reviewed: `2026-05-14`

## Purpose

This repository implements Williams Defender red-label arcade behavior in Rust.
The goal is deterministic, source-shaped behavior that can be checked against
red-label source material, MAME-observed behavior, local trace fixtures, pixel
fixtures, sound command fixtures, and unit tests.

The runtime is self-contained for normal play. ROM files are optional
verification inputs only.

## Source Of Truth

Use these references when behavior is unclear:

1. Red-label behavior observed under MAME.
2. Red-label source and tables from <https://github.com/mwenge/defender>.
3. MAME Williams driver, video, ROM, memory-map, and PIA behavior.
4. Williams sound-ROM source.
5. Williams operator documentation and cabinet references.
6. External behavior analysis and screenshots.

Current Rust behavior is not authoritative unless it is backed by one of those
sources or by an accepted fixture.

## Implementation Rules

- Implement red-label Defender behavior, not a Defender-like approximation.
- Cite source routines, tables, MAME behavior, or fixtures when adding
  behavior-sensitive code.
- Keep runtime assets in `assets/red-label/` and embed them with
  `include_str!` or `include_bytes!`.
- Keep local ROMs and generated MAME reference traces out of normal runtime
  requirements.
- Preserve exact source-visible mutations in tests: RAM, CMOS, video RAM,
  palette RAM, process lists, object lists, shell lists, scheduler state,
  sound commands, and snapshots.
- Keep Planetoid controls and `XYZZY` outside the arcade core as explicit
  compatibility layers.
- Do not guess unknown arcade behavior. Record a gap and add a focused test or
  fixture path before implementation.
- Maintain the 80% project line-coverage floor and keep added executable Rust
  lines covered.

## Current Architecture

The crate is now split between a clean rewrite source tree and a legacy oracle
tree:

- `src/main.rs`: thin CLI entry point that still dispatches to the runtime
  bridge through the clean platform launcher while the rewrite takes over.
- `src/lib.rs`: clean public crate wiring plus crate-private explicit
  `#[path]` adapters to the legacy oracle tree and a doc-hidden README media
  facade wired from `src_legacy/readme_media.rs`. Machine process/state
  contracts, red-label math types, and low-level asset, board, memory, ROM,
  sound, live, PIA, and `wgpu` modules must remain crate-private. Generated
  long-trace sample fixtures must stay private to the legacy machine oracle
  instead of being root-wired here.
- `src/accepted.rs`: crate-private accepted-behavior contracts and facade that
  isolate clean oracle code from direct legacy imports.
- `src/game.rs`: gameplay-facing `Game`, `GameState`, `GameInput`,
  `GameFrame`, `GameEvents`, world, terrain, starfield, enemy, human, score,
  projectile, player, direction, and sound-event contracts without accepted
  command-byte mapping. The clean `Game` shell emits sprite-first scene frames
  without touching the accepted machine adapter.
- `src/systems.rs`: deterministic fixed-step timing utilities, clean
  player-control intent/trigger systems, operator trigger handling,
  player-motion, enemy-motion, projectile launch/capacity/motion systems,
  projectile/enemy collision detection, player/enemy damage resolution,
  smart-bomb resolution, wave-completion evaluation, score/bonus awards,
  high-score entry qualification and initials handling, and the
  `GameSimulation` trait used by the clean game and internal oracle
  implementations.
- `src/renderer.rs`: native `wgpu` scene contracts, surface sizing, sprite
  layers, temporary raster evidence, renderer-owned resources, atlas-backed
  sprite batches, sprite quad geometry, sprite instance buffers, the sprite
  instance GPU ABI, sprite instance upload streams, sprite draw commands,
  sprite `wgpu` buffer upload plans, sprite render-pass plans, sprite pipeline
  plans, sprite resource binding plans, sprite atlas texture upload plans,
  sprite pipeline layout plans, sprite render pipeline descriptor plans,
  sprite render-pass encoder command plans, frame-level GPU command plans,
  viewport layout, GPU pass planning, scene summaries, render signatures, and
  draw planning.
- `src/platform.rs`: the clean runtime launch boundary plus configuration for
  controls, audio, run mode, and persistence.
- `src/runtime.rs`: the crate-private launch bridge that translates clean
  runtime configuration into launch commands and routes config-driven `wgpu`
  live and smoke launches.
- `src/live_wgpu.rs`: the crate-private WGPU live launch facade that owns the
  temporary presenter/input-profile bridge for interactive and smoke runs.
- `src/roms.rs`: the crate-private optional ROM verification facade that owns
  the temporary ROM metadata, scan, and loader bridge.
- `src/audio.rs`: gameplay-facing `SoundEvent` batches, the live audio worker
  boundary, disabled/null no-device modes, and runtime diagnostics. It consumes
  clean `GameFrame` and `SoundEvent` contracts, not legacy frame outputs.
- `src/fidelity.rs`: clean frame-equivalence signatures for gameplay state,
  gameplay events, sound events, and render summaries. Clean fidelity tests use
  oracle-owned reference probes instead of importing accepted facade types
  directly.
- `src/fidelity_manifest.rs`: the crate-private fidelity scenario manifest
  facade that owns temporary scenario metadata and input expansion.
- `src/fidelity_trace_engine.rs`: the crate-private fidelity trace engine
  facade that owns temporary trace generation, comparison, and schema access.
- `src/oracle.rs`: the crate-private gameplay oracle, including clean state,
  event, sound, and scene-summary frames from the accepted-behavior facade for
  internal fidelity comparison.

The converted implementation is parked under `src_legacy/`. It still owns the
accepted arcade behavior, hardware models, ROM verification, rendering, input,
sound-board command evidence, legacy fidelity trace generation, the threaded
live core runtime boundary, `wgpu` window ownership, CMOS storage, and test
helpers until clean systems replace those responsibilities. Those root adapters
are crate-private. Clean runtime launch goes through the private `runtime`
bridge, while the internal oracle uses the crate-private `accepted` facade.
`src_legacy/accepted_behavior.rs` performs the current legacy-machine
adaptation into neutral accepted-behavior contracts before the public clean
gameplay types see it. `src/live_wgpu.rs` owns the temporary presenter and
input-profile bridge for config-driven `wgpu` live and smoke launches used by
the clean runtime API. `src/roms.rs` owns the temporary ROM metadata, scan, and
loader bridge for optional verification commands. `src/fidelity_manifest.rs`
owns the temporary scenario manifest and input expansion bridge for fidelity
scenario commands. `src/fidelity_trace_engine.rs` owns the temporary trace
generation, comparison, and schema bridge for fidelity trace commands.
Legacy-specific clean equivalence regressions are also wired from `src_legacy/`
so clean
accepted/oracle source stays focused on gameplay contracts. Internal clean
equivalence regressions use crate-private oracle wiring. Clean frame-signature
gates live under
`src/fidelity.rs`, compare clean render signatures rather than exposing
memory-oriented CRC labels, and use oracle-owned reference probes for accepted
behavior comparison, while legacy trace generation is root-wired as
`legacy_fidelity`.
Legacy live code adapts frame outputs into clean audio
frames before submitting to `src/audio.rs`. Temporary README media tooling uses
the doc-hidden `defender::readme_media` facade rather than low-level legacy
module exports. Machine process/state contracts remain crate-private oracle
wiring. Live presentation receives clean `RenderScene` data. Native draw
planning resolves scene sprites through renderer-owned atlas regions into
sprite batches and records GPU instance-buffer data with native scene
rectangles, normalized atlas UVs, normalized tint, stable upload bytes, and the
`wgpu` vertex layout for the instance buffer. The clean `Game` world seeds
terrain, starfield, enemy, human, and projectile snapshots for the first playing
wave and renders them as atlas-backed scene sprites. Operator controls are
sampled through `OperatorControlSystem`, emitting diagnostics, audits, and
high-score reset gameplay events on button edges while preserving current
player scores and bonus thresholds during high-score reset. Clean enemy
snapshots carry gameplay-domain velocity and advance through
`EnemyMotionSystem` during playing frames. Clean projectile snapshots carry
direction-derived velocity, advance through `ProjectileMotionSystem`, and are
culled through gameplay state before rendering. Clean collision boxes resolve
projectile/enemy hits through `CollisionSystem`, remove the hit entities from
world state, and award score through `ScoreSystem` before rendering. Crossing
the clean bonus threshold updates player stock and emits `BonusAwarded`. Clean
smart bombs consume player stock, clear active enemies through
`SmartBombSystem`, route score through the same scoring system, and leave
cleared enemies absent from the scene. Enemy contact with the player is
resolved through clean collision and `PlayerDamageSystem`, decrementing lives,
removing the colliding enemy, and entering `GameOver` on the final life.
Qualifying final scores are routed through `HighScoreEntrySystem` into
`HighScoreEntry` with `HighScoreEntryStarted` output. High-score entry accepts
alphabetic initials through clean input, normalizes them to uppercase, supports
backspace, emits `HighScoreInitialAccepted`, and emits `HighScoreSubmitted`
when the third initial returns the clean game to attract. Enemy exhaustion is
reported through `WaveSystem`, keeping the last-hit frame empty and spawning
the next clean wave on the following playing frame. It flattens those per-batch
records into one upload-ready
instance stream. The renderer also owns unit quad vertices, `u16` indices,
upload bytes, and the `wgpu` vertex layout used to draw instanced sprites, then
derives indexed instanced sprite draw commands with quad/index counts, instance
ranges, and upload byte spans into that stream. It also records the centered
viewport layout plus GPU-ready clear color, viewport command, and
scene-projection constants for the target surface.
Sprite draw plans also include `wgpu::BufferUsages` metadata and upload bytes
for the quad vertex, quad index, and instance buffers, plus a sprite render-pass
plan with stable vertex buffer slots, index-buffer metadata, and indexed
instance draw ranges. Sprite pipeline plans describe the renderer-owned WGSL
shader, vertex and fragment entry points, quad and instance vertex layouts,
alpha-blended color target, primitive state, and multisample state for the
target texture format. Sprite resource binding plans describe the
scene-projection uniform upload, projection bind group layout, atlas texture
binding, atlas sampler binding, and atlas texture upload metadata used by that
shader. The default clean sprite atlas owns deterministic nonblank RGBA pixels
plus the `wgpu` texture format, usage, extent, and copy layout needed to
populate it. Sprite pipeline layout plans then order those projection and atlas
bind groups for `wgpu` `PipelineLayoutDescriptor` creation, and sprite render
pipeline descriptor plans combine that layout with shader entries, vertex
buffers, primitive state, color target, and multisample state for `wgpu` render
pipeline creation. Sprite render-pass encoder command plans then order the
pipeline, bind groups, vertex buffers, index buffer, and indexed draw calls for
`wgpu::RenderPass` execution. Frame-level GPU command plans combine pass clear
state, viewport, scene-projection upload presence, optional sprite execution,
and temporary raster evidence into one ordered scene command stream, while the
current live path still carries a temporary raster payload for visual
equivalence. Kitty graphics and terminal-session code remain
parked there as historical compatibility evidence, but they are not part of the
active runtime or compatibility API surface. The legacy video renderer owns its
remaining `TerminalGeometry` value type directly instead of importing terminal
session setup. Generated long-trace sample data is nested under the legacy
machine oracle because it is historical fixture evidence, not a clean root
adapter.
Public API tests scan clean module sources so production code cannot import
low-level legacy root modules, bypass the accepted-behavior facade, or
reintroduce legacy implementation terminology.

## Current Behavior Surface

- Live play uses the windowed `wgpu` backend.
- Runtime renderer selection has been removed.
- `--input-profile planetoid` is the default input profile.
- `--input-profile cabinet` exposes a MAME-style cabinet keyboard profile.
- `--mute` disables the live audio event runtime path.
- `--cmos-path <file>` opts into file-backed CMOS persistence.
- `--rom-report` and `--verify-roms` validate optional local red-label ROM
  files against embedded metadata.
- Fidelity commands emit and compare deterministic TSV traces from the Rust
  core and local fixture directories.
- README media is generated from the current native renderer with
  `make readme-media`.

## Compatibility Features

`XYZZY` is intentionally non-arcade behavior. It must stay outside the
red-label trace contract unless paired tests prove disabled arcade behavior and
enabled overlay behavior.

Planetoid key mapping is an input profile only. The arcade core receives
Defender cabinet actions, not Planetoid-specific semantics.

## Validation

Required local gates for behavior or architecture changes:

```sh
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
```

Docs-only changes should at least run:

```sh
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md
git diff --check
```

`make fidelity` runs the broad gate: formatting, all Rust targets, clippy,
trace exporter self-tests, Python helper tests, local trace fixture comparison,
coverage, and added-line coverage.

GitHub CI keeps the expensive gates split by subsystem: `make ci-doctor`
checks Lua, Python, coverage, and Linux smoke prerequisites; `make fidelity`
runs the Rust, trace, and coverage gate; and `xvfb-run -a make smoke-wgpu`
runs the no-device live smoke path. Coverage baseline refreshes must use the
explicit `make coverage-new-code-baseline NEW_CODE_COVERAGE_BASE=...` command.

## Active Constraints

- Live audio consumes clean `GameFrame` and `SoundEvent` batches and delivers
  them to a bounded non-blocking backend trait. Legacy frame-output sound
  timing is adapted before it reaches `src/audio.rs`. The built-in backend is
  a null backend that opens no audio device; audible device output remains
  future work. The accepted implementation contract is documented in
  `docs/fidelity/live-audio.md`.
- Local MAME reference generation is intentionally local tooling; generated
  reference traces are not part of the normal runtime.
- New clean production callers must avoid direct legacy module imports. Code
  that still needs accepted-behavior evidence should use the crate-private
  `accepted` facade until the clean system replaces that responsibility.
  Temporary README media tooling may use the doc-hidden `readme_media` facade;
  root legacy modules must remain crate-private. This boundary is guarded by
  source-level public API tests.
