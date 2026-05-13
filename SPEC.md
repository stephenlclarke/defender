# Defender Current Specification

Last reviewed: `2026-05-13`

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

- `src/main.rs`: thin CLI entry point that still dispatches to the compatibility
  app while the rewrite takes over.
- `src/lib.rs`: clean public crate wiring plus explicit `#[path]` adapters to
  the legacy oracle tree.
- `src/game.rs`: gameplay-facing `GameState`, `GameInput`, `GameFrame`,
  `GameEvents`, score, player, direction, and sound-event contracts.
- `src/systems.rs`: deterministic fixed-step timing utilities, clean
  player-control intent/trigger systems, player-motion and projectile
  launch/capacity systems, and the `GameSimulation` trait for future game
  systems.
- `src/renderer.rs`: native `wgpu` scene contracts, surface sizing, sprite
  layers, temporary raster evidence, renderer-owned resources, scene summaries,
  and draw planning.
- `src/platform.rs`: runtime configuration for controls, audio, run mode, and
  persistence.
- `src/audio.rs`: gameplay-facing `SoundEvent` batches, the live audio worker
  boundary, disabled/null no-device modes, and runtime diagnostics.
- `src/oracle.rs`: the explicit adapter from clean gameplay contracts to the
  current accepted implementation, including clean state, event, sound, and
  scene-summary frames.

The converted implementation is parked under `src_legacy/`. It still owns the
accepted arcade behavior, hardware models, ROM verification, rendering, input,
sound-board command evidence, fidelity trace generation, the threaded live core
runtime boundary, `wgpu` window ownership, CMOS storage, and test helpers until
clean systems replace those responsibilities. Those compatibility modules are
doc-hidden from the supported public API surface. Live presentation receives
clean `RenderScene` data, currently with a temporary raster payload for visual
equivalence. Kitty terminal graphics code remains parked there as historical
compatibility evidence, but it is not part of the active runtime surface.

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

- Live audio currently maps accepted `FrameOutput::sound_commands()` timing to
  clean `SoundEvent` batches and delivers them to a bounded non-blocking
  backend trait. The built-in backend is a null backend that opens no audio
  device; audible device output remains future work. The accepted
  implementation contract is documented in
  `docs/fidelity/live-audio.md`.
- Local MAME reference generation is intentionally local tooling; generated
  reference traces are not part of the normal runtime.
- New callers should prefer canonical `machine_state::...` and
  `machine_process::...` contract paths; existing `machine::...` compatibility
  imports stay available until a narrow migration proves callers no longer need
  them.
