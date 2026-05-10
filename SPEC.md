# Defender Current Specification

Last reviewed: `2026-05-10`

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

The crate is split around the current accepted module boundaries:

- `src/main.rs`: thin CLI entry point.
- `src/app.rs`: CLI parsing, command dispatch, help text, and user-facing
  command output.
- `src/lib.rs`: public crate module wiring.
- `src/machine.rs`: shared red-label contracts, source asset parsers, and
  stable `machine::...` re-exports.
- `src/machine_state.rs`: data-only arcade state, snapshots, events, and
  frame-output contracts.
- `src/machine_process.rs`: scheduler data contracts:
  `RedLabelCpuRegisters` and `RedLabelScheduledProcess`.
- `src/machine_memory.rs`: source-visible runtime memory and translated
  source mutations.
- `src/machine_session.rs`: `ArcadeMachine` public API, live stepping,
  session flow, snapshots, save/restore, high-score, operator, and
  compatibility orchestration.
- `src/machine_scheduler.rs`: source-entry register overrides and
  live-prioritized routine sets.
- `src/machine_sound.rs`: red-label sound command contracts and fixture
  helpers.
- `src/machine_video.rs`: reusable laser, star, terrain, and video helper
  primitives.
- `src/machine_player.rs`: reusable player, projectile, object, and signed
  arithmetic helpers.
- `src/machine_world.rs`: wave/world and BCD helper primitives.
- `src/board.rs`, `src/pia.rs`, and `src/rom.rs`: hardware, memory-map,
  PIA, ROM metadata, and verification surfaces.
- `src/sound.rs`: sound-board model, command latch, PIA, ROM, IRQ, and DAC
  signature behavior.
- `src/video.rs`, `src/live.rs`, `src/wgpu_presenter.rs`, `src/kitty.rs`, and
  `src/terminal.rs`: native frame extraction, live presentation, input loop,
  `wgpu`, Kitty, and terminal support.
- `src/input.rs`: Planetoid, cabinet, and test input profiles plus `XYZZY`
  overlay input detection.
- `src/fidelity.rs`: trace schema and fixture comparison support.
- `src/cmos_storage.rs`: optional file-backed CMOS persistence.

## Current Behavior Surface

- Live play defaults to the windowed `wgpu` backend.
- `--renderer kitty` keeps the Kitty graphics backend available for
  compatibility evidence.
- `--input-profile planetoid` is the default input profile.
- `--input-profile cabinet` exposes a MAME-style cabinet keyboard profile.
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

## Active Constraints

- Live audio output is not implemented. Sound fidelity is currently covered by
  command traces, sound-board state, and deterministic DAC byte signatures.
- Local MAME reference generation is intentionally local tooling; generated
  reference traces are not part of the normal runtime.
- Any future public API cleanup must preserve existing `machine::...` imports
  until a narrow migration proves callers no longer need them.
