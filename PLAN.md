# Defender Current Plan

Last reviewed: `2026-06-07`

## Goal

Ship the accepted clean Defender rewrite as the normal repository state.

The game should match the original Williams Defender red-label arcade machine
as closely as practical for visuals, audio, sprite behaviour, laser behaviour,
explosions, reverse direction, attract mode, side scrolling, waves, humans, and
playability, while remaining a clean actor implementation backed by modern Rust
systems and `wgpu`.

## Current State

- The gameplay implementation has been accepted by the owner.
- Normal play uses the actor runtime, `wgpu` renderer, `winit` windowing, and
  synthesized Williams sound-command audio.
- The retired ROM/frame/oracle conversion tree is being removed from the
  production crate and default CLI.
- README media is generated from the accepted actor runtime, not from the
  retired frame-oriented tooling.

## Active Cleanup Slice

This slice is the final repository tidy-up after implementation acceptance.

- [x] Retire old source modules, test harnesses, and stale documentation that
  only supported the assembler-conversion/oracle phase.
- [x] Remove default CLI paths for ROM reports and trace/fidelity commands.
- [x] Keep live play, smoke checks, actor script checks, and `wgpu` smoke checks
  as the supported developer command surface.
- [x] Add Makefile targets for README media:
  - `make readme-gameplay-image`
  - `make readme-attract-sequence`
  - `make readme-media`
- [x] Generate README media through accepted actor runtime scene rendering.
- [x] Regenerate `docs/defender.png`.
- [x] Regenerate `docs/start-sequence.gif`.
- [x] Run formatting, tests, clippy, docs lint, and diff hygiene.
- [x] Commit this cleanup slice with a Conventional Commit message.

## Validation

Run these before closing the slice:

```sh
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make readme-media
make docs-lint
make diff-check
```

For release-style validation, run:

```sh
make release-gate
```

## Slack And Commit Rules

- Post Slack updates at the start and end of each dev cycle, step, or
  milestone.
- Include the approximate percentage of repo-finalisation work remaining in
  Slack updates.
- Commit each completed slice using a Conventional Commit message.
- Do not push unless explicitly asked.

## Remaining Work

After the active cleanup slice is validated and committed, the plan is complete
unless new owner feedback identifies a concrete mismatch against the original
arcade behaviour.
