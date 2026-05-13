# Defender

[![Quality Gate Status](https://sonarcloud.io/api/project_badges/measure?project=stephenlclarke_defender&metric=alert_status)](https://sonarcloud.io/summary/new_code?id=stephenlclarke_defender)
[![Bugs](https://sonarcloud.io/api/project_badges/measure?project=stephenlclarke_defender&metric=bugs)](https://sonarcloud.io/summary/new_code?id=stephenlclarke_defender)
[![Code Smells](https://sonarcloud.io/api/project_badges/measure?project=stephenlclarke_defender&metric=code_smells)](https://sonarcloud.io/summary/new_code?id=stephenlclarke_defender)
[![Coverage](https://sonarcloud.io/api/project_badges/measure?project=stephenlclarke_defender&metric=coverage)](https://sonarcloud.io/summary/new_code?id=stephenlclarke_defender)
[![Duplicated Lines (%)](https://sonarcloud.io/api/project_badges/measure?project=stephenlclarke_defender&metric=duplicated_lines_density)](https://sonarcloud.io/summary/new_code?id=stephenlclarke_defender)
[![Lines of Code](https://sonarcloud.io/api/project_badges/measure?project=stephenlclarke_defender&metric=ncloc)](https://sonarcloud.io/summary/new_code?id=stephenlclarke_defender)
[![Reliability Rating](https://sonarcloud.io/api/project_badges/measure?project=stephenlclarke_defender&metric=reliability_rating)](https://sonarcloud.io/summary/new_code?id=stephenlclarke_defender)
[![Security Rating](https://sonarcloud.io/api/project_badges/measure?project=stephenlclarke_defender&metric=security_rating)](https://sonarcloud.io/summary/new_code?id=stephenlclarke_defender)
[![Technical Debt](https://sonarcloud.io/api/project_badges/measure?project=stephenlclarke_defender&metric=sqale_index)](https://sonarcloud.io/summary/new_code?id=stephenlclarke_defender)
[![Maintainability Rating](https://sonarcloud.io/api/project_badges/measure?project=stephenlclarke_defender&metric=sqale_rating)](https://sonarcloud.io/summary/new_code?id=stephenlclarke_defender)
[![Vulnerabilities](https://sonarcloud.io/api/project_badges/measure?project=stephenlclarke_defender&metric=vulnerabilities)](https://sonarcloud.io/summary/new_code?id=stephenlclarke_defender)
![Repo Visitors](https://visitor-badge.laobi.icu/badge?page_id=stephenlclarke.defender)

`defender` is a native Rust implementation of Williams Defender red-label
arcade behavior. The runtime is self-contained: red-label tables, ROM metadata,
trace schema, video data, and sound command fixtures are embedded at build
time, so normal play does not need a local ROM or asset directory.

Live play uses a windowed `wgpu` renderer.

![Defender gameplay frame](docs/defender.png)

![Defender attract sequence](docs/start-sequence.gif)

## Install

Install from git with Cargo:

```sh
cargo install --git https://github.com/stephenlclarke/defender defender
```

Then run:

```sh
defender
```

## Run

Common commands:

```sh
cargo run
cargo run -- --input-profile planetoid
cargo run -- --input-profile cabinet
cargo run -- --cmos-path ~/.local/state/defender/red-label-cmos.bin
cargo run -- --live-smoke
cargo run -- --mute
cargo run -- --rom-report
cargo run -- --rom-report /path/to/roms
cargo run -- --verify-roms /path/to/roms
```

Fidelity and trace tooling:

```sh
cargo run -- --fidelity-trace 300
cargo run -- --fidelity-trace-inputs 'coin,start_one;fire,thrust;none'
cargo run -- --fidelity-trace-inputs-file /path/to/inputs.txt
cargo run -- --fidelity-check-trace /path/to/inputs.txt /path/to/expected.tsv
cargo run -- --fidelity-check-trace-dir docs/fidelity/fixtures/local/rust-current
cargo run -- --fidelity-list-scenarios
cargo run -- --fidelity-write-scenario-inputs docs/fidelity/fixtures/local/reference
cargo run -- --fidelity-check-reference-trace-dir docs/fidelity/fixtures/local/reference
```

Make targets:

```sh
make run
make run-wgpu
make live-wgpu
make smoke-wgpu
make ci
make ci-doctor
make trace-doctor
make coverage-doctor
make smoke-doctor
make fidelity
make trace-script-test
make trace-fixtures
make reference-inputs
make reference-traces
make reference-fixtures-check
make coverage
make coverage-new-code NEW_CODE_COVERAGE_BASE=origin/main
make coverage-new-code-baseline NEW_CODE_COVERAGE_BASE=origin/main
make sq-ci
make sq
make readme-media
```

`make readme-media` regenerates `docs/start-sequence.gif` from the current
renderer.

## Controls

The default live input profile is `planetoid`, which maps BBC Micro
Planetoid-style keys onto Defender cabinet actions:

- `5`: left coin slot
- `6`: center coin slot
- `7`: right coin slot
- `1` or `ENTER`: one-player start when credit or free play is available
- `A`: move up
- `Z`: move down
- `SHIFT`: thrust
- `SPACE`: reverse
- `ENTER`: fire while playing
- `TAB`: smart bomb
- `H`: hyperspace
- `F2`: service advance / diagnostics target
- `F3`: reset today's high-score table from ROM defaults
- hold `F4` with `F2`: select the auto/audits target
- `F5`: slam / tilt switch
- `A`-`Z`: enter initials during high-score entry
- `BACKSPACE`: delete the previous initial during high-score entry
- `Q` or `ESC`: quit

Use `--input-profile cabinet` for a MAME-style cabinet profile. In that
profile, `5` inserts a left-slot coin and `1` starts a one-player game;
`ENTER` is not a start key.

To start a normal one-player game from a fresh session, press `5`, then `1`.
The red-label start path intentionally blocks no-credit starts unless the
cabinet is configured for free play.

## Persistence

Live CMOS persistence is opt-in. Without `--cmos-path`, each run starts from
embedded red-label defaults and does not create or update a CMOS file. Provide
`--cmos-path <file>` to persist high scores, credits, audits, and adjustment
CMOS cells across runs.

## XYZZY Overlay

During live play, type `X`, `Y`, `Z`, `Z`, `Y` to toggle `XYZZY` mode.

`XYZZY` is a deliberate compatibility overlay, not Williams red-label arcade
behavior. With it disabled, trace generation and live core stepping stay on the
red-label path.

Current overlay hooks:

- `F`: toggles auto-fire while `XYZZY` is active.
- `TAB`: can emit an overlay smart bomb after red-label inventory reaches zero
  while `XYZZY` is active.
- `G`: toggles the recorded invincibility overlay flag. It is trace-invisible
  until an arcade-facing hook is implemented.

Typing `XYZZY` again disables the overlay and resets overlay toggles.

## Development

Primary validation commands:

```sh
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md
```

`make fidelity` runs formatting, all Rust targets, clippy, Lua trace exporter
self-tests, Python helper tests, local Rust trace fixture comparison, coverage,
and new-Rust-line coverage. The new-line gate compares against
`NEW_CODE_COVERAGE_BASE` when set, otherwise `HEAD` for local dirty worktrees.
`make coverage-new-code` requires an explicit base and subtracts the accepted
uncovered-line baseline in `tools/new_rust_coverage_baseline.txt`; refresh that
baseline only when intentionally accepting existing uncovered debt. `make ci`
adds the `wgpu` live smoke test. GitHub CI runs `make ci-doctor`, then
`make fidelity`, then `xvfb-run -a make smoke-wgpu` so prerequisite, fidelity,
coverage, and smoke failures are separated in the Actions UI. The
`make smoke-doctor` target is Linux CI-oriented and expects `xvfb-run` plus
`vulkaninfo`.
Slack completion notes are a best-effort project protocol outside CI; connector
or token failures should be handled as Slack tooling failures, not Rust
validation failures.

Local SonarCloud support:

```sh
make sq-ci
make sq
```

`make sq` requires `sonar-scanner` and `SONAR_TOKEN`.

## Architecture

The primary source tree is now the clean rewrite under `src/`; the converted
implementation is parked under `src_legacy/` and remains wired through
`src/lib.rs` as the temporary gameplay oracle and compatibility runtime. Clean
modules that still need accepted-behavior evidence reach those adapters through
the doc-hidden `defender::compatibility` namespace instead of taking new direct
dependencies on legacy module names.

Clean rewrite modules:

- `src/game.rs`: gameplay-facing `GameState`, `GameInput`, `GameFrame`,
  `GameEvents`, score, player, direction, and sound-event contracts.
- `src/systems.rs`: deterministic fixed-step timing utilities, clean
  player-control intent/trigger systems, player-motion and projectile
  launch/capacity systems, and the `GameSimulation` trait for future game
  systems.
- `src/renderer.rs`: native `wgpu` scene contracts, surface sizing, sprite
  layers, temporary raster evidence, renderer-owned resources, scene summaries,
  and draw planning.
- `src/platform.rs`: the clean runtime launch boundary plus configuration for
  controls, audio, run mode, and persistence.
- `src/audio.rs`: gameplay-facing sound events, the bounded live-audio runtime,
  no-device backends, and worker diagnostics.
- `src/oracle.rs`: the only clean-tree adapter to the converted implementation,
  returning clean state, event, sound, and scene-summary frames.

Legacy source-shaped modules under `src_legacy/` still own the accepted arcade
behavior, assets, hardware models, ROM verification, rendering, input,
sound-board command evidence, fidelity trace generation and threaded fixture
checks, the threaded live core runtime boundary, `wgpu` window ownership, CMOS
storage, and test helpers. They remain wired as doc-hidden compatibility
modules rather than supported public API. The binary enters through the clean
platform boundary before delegating to the compatibility runtime. The live
worker now wraps accepted visual output as a clean `RenderScene` raster payload
before the presenter draws it. Kitty terminal graphics code remains parked
there as historical compatibility evidence, but it is no longer an active
runtime path.

## Assets And ROMs

Clean runtime data lives under `assets/red-label/` and is embedded in the
binary. The active assets include ROM metadata, MAME memory/input maps,
red-label RAM/CMOS layouts, linked lists, routine addresses, switch tables,
object pictures and images, terrain data, wave data, high-score/default data,
sound command timelines, the live-audio acceptance matrix, and fidelity trace
schemas.

Local ROM files are optional verification inputs for `--rom-report` and
`--verify-roms`. They are not needed for normal play.

Legacy prototype images and sounds under `assets/arcade/` and `assets/sounds/`
are retained only as references unless a module explicitly reclassifies them
with source or ROM provenance.

## Platform Support

- Live backend: `wgpu`, through `cargo run` or `defender`.
- Live audio consumes gameplay-facing `SoundEvent` batches through a bounded
  non-blocking runtime with worker diagnostics. The built-in backend is a null
  backend that opens no audio device; `--mute` disables the runtime path.
  Audible device output is still future work. The accepted implementation
  contract is in
  `docs/fidelity/live-audio.md`.

## References

- Red-label source and ROM build reference:
  <https://github.com/mwenge/defender>
- MAME Williams driver and ROM map:
  <https://github.com/mamedev/mame/blob/master/src/mame/midway/williams.cpp>
- MAME Williams video implementation:
  <https://github.com/mamedev/mame/blob/master/src/mame/midway/williams_v.cpp>
- MAME Motorola 6821 PIA implementation:
  <https://github.com/mamedev/mame/blob/master/src/devices/machine/6821pia.cpp>
- Williams sound-ROM source:
  <https://github.com/historicalsource/williams-soundroms>
- Defender setup and operations manual:
  <https://arcade-museum.com/manuals-videogames/D/DefenderSetupBookletUSA.pdf>
- Williams Defender cabinet reference:
  <https://williamsarcades.com/Defender>
- Red-label ROM metadata reference:
  <https://mdk.cab/game/defender>
- Defender behavior analysis:
  <https://www.dougmahugh.com/defender-chapter02/>
- BBC Micro Planetoid archive entry:
  <https://bbcmicro.co.uk/game.php?id=11>
