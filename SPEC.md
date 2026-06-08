# Defender Current Specification

Last reviewed: `2026-06-07`

## Purpose

This repository implements Williams Defender red-label arcade behaviour in
Rust. The runtime is a clean actor implementation, not an assembler conversion
or memory-frame replay.

## Accepted Runtime

The production runtime is self-contained:

- `src/actor_game.rs` owns actor simulation, attract sequencing, wave
  progression, world-space objects, player state, humanoids, scoring, and
  actor sound/draw reports.
- `src/game.rs` owns the public game state contract and source-shaped data
  structures used by the actor bridge.
- `src/renderer.rs` owns scene sprites, source-derived sprite atlas data,
  CPU README rasterization, and native `wgpu` render planning.
- `src/live_wgpu.rs` owns the native windowed runtime, live input mapping,
  live smoke checks, actor script checks, and `wgpu` smoke validation.
- `src/audio.rs` and `src/sound_board.rs` own live synthesized audio from
  Williams sound-command families.
- `src/platform.rs` and `src/runtime.rs` own the supported CLI and launch
  boundaries.

Normal play must not require ROM files, MAME, local media artifacts, or the
retired frame-oriented oracle tree.

## Behaviour Contract

The implementation should match the original MAME red-label game as closely as
practical for:

- attract-mode page order, coalescence effects, scoring legend, and silence.
- side-scrolling world-space gameplay.
- terrain, scanner, HUD, score, credits, lives, and smart-bomb display.
- player thrust, reverse orientation, laser shape, shot cap, smart bomb, and
  hyperspace.
- landers, mutants, bombers, pods, swarmers, baiters, bombs, humans, rescues,
  captures, conversions, wave clear, and terrain colour cycling.
- player death, explosions, game-over timing, high-score entry, and return to
  attract.
- synthesized arcade sound families and command timing where source evidence is
  available.

Unknown arcade behaviour should be recorded and tested before changing the
runtime. Do not add Defender-like approximations when source/MAME evidence is
available.

## Supported CLI

Supported commands:

- `cargo run`
- `cargo run -- --actor-live`
- `cargo run -- --actor-script /path/to/driver.script`
- `cargo run -- --actor-script-check /path/to/driver.script`
- `cargo run -- --live-smoke`
- `cargo run -- --game-smoke`
- `cargo run -- --actor-smoke`
- `cargo run -- --actor-attract-smoke`
- `cargo run -- --actor-post-game-smoke`
- `cargo run -- --actor-wgpu-smoke`
- `cargo run -- --mute`
- `cargo run -- --input-profile planetoid`
- `cargo run -- --input-profile cabinet`
- `cargo run -- --cmos-path ~/.local/state/defender/red-label-cmos.bin`

Retired ROM report, trace, and legacy fidelity CLI commands are intentionally
not supported by the accepted runtime.

## README Media

Committed README media lives at:

- `docs/defender.png`
- `docs/start-sequence.gif`

Media generation is part of the maintained workflow:

- `make readme-gameplay-image` regenerates the gameplay screenshot.
- `make readme-attract-sequence` regenerates the attract GIF.
- `make readme-media` regenerates both.

These targets render accepted actor runtime scenes through the source-derived
sprite atlas and CPU raster path in `src/renderer.rs`.

## Validation

The normal local validation path is:

```sh
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make readme-media
make docs-lint
make diff-check
```

The release-style path is:

```sh
make release-gate
```

## Source Materials

Primary references are the red-label source, MAME red-label behaviour,
Williams sound ROM source, Williams operator documentation, and accepted owner
review media. External references are listed in `README.md`.
