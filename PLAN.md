# Defender Current Plan

Last reviewed: `2026-05-10`

## Current Baseline

- Active branch: `red-label-refactor`.
- Latest accepted implementation commit: `7f7b6a1`.
- Phase 13 is complete. The post-`wgpu` machine core is split into
  source-shaped child modules while preserving existing `machine::...` public
  imports.
- Live play defaults to the `wgpu` backend. Kitty remains a compatibility
  backend.
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
- Keep compatibility behavior (`XYZZY`, Planetoid controls, Kitty renderer)
  outside the red-label arcade contract unless paired tests prove the boundary.
- Use Conventional Commits for committed work.
- Do not use `codex` in branch names, commit messages, or PR titles.
- Post completion notes to `xyzzytools.slack.com#codex` when a planned
  dev-cycle closes.

## Active Cycle

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

## Next Work

- Keep docs synchronized with the CLI help, Makefile targets, and current
  module boundaries.
- If live audio output becomes a target, add a new cycle with source-backed
  acceptance criteria before implementation.
- If public API cleanup is desired, migrate `machine::...` re-exports in a
  narrow cycle with caller checks and focused tests.
