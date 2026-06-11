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

---

This repository is a native Rust reimplementation of Williams' `Defender`
arcade game.

The live game uses a clean actor runtime, `wgpu` rendering, `winit` windowing,
and synthesized audio driven by Williams sound-command families. Runtime
tables, sprites, attract scripts, wave scripts, glyphs, and sound metadata are
embedded in the binary, so normal play does not require external asset files or
an asset directory.

The goal is faithful arcade behaviour: visuals, audio, sprite
behaviour, lasers, explosions, reverse direction, attract mode, side scrolling,
waves, humans, and playability should match accepted arcade reference behavior
as closely as practical while remaining a clean actor implementation.

![Defender gameplay scene](docs/defender.png)

<!-- markdownlint-disable MD033 -->
<p align="center">
  <img
    src="docs/start-sequence.gif"
    alt="
      Defender attract sequence with Williams logo page, hall of fame, and
      instruction page
    "
  />
</p>
<!-- markdownlint-enable MD033 -->

## Install

Install directly from git with Cargo:

```sh
cargo install --git https://github.com/stephenlclarke/defender defender
```

After installation, run the game with:

```sh
defender
```

Useful launch options:

- `defender --input-profile cabinet`
- `defender --input-profile planetoid`
- `defender --mute`
- `defender --cmos-path ~/.local/state/defender/arcade-cmos.bin`
- `defender --actor-script /path/to/driver.script`

## Build

Build and validate the current runtime with:

```sh
cargo build
cargo test --all-targets
cargo fmt --check
cargo clippy --all-targets -- -D warnings
make release-gate
```

Important Makefile targets:

- `make` or `make help`: print the supported target list.
- `make run`: launch the playable `wgpu` game.
- `make fmt`: check Rust formatting.
- `make test`: run all Rust tests.
- `make clippy`: run clippy with warnings denied.
- `make smoke`: run the local actor and `wgpu` smoke suite.
- `make live-smoke`: run the headless live runtime smoke path.
- `make actor-smoke`: run the accepted actor gameplay smoke path.
- `make actor-attract-smoke`: run the accepted attract sequence smoke path.
- `make actor-post-game-smoke`: run the game-over return smoke path.
- `make actor-wgpu-smoke`: run actor-to-`wgpu` renderer smoke validation.
- `make readme-gameplay-image`: regenerate `docs/defender.png`.
- `make readme-attract-sequence`: regenerate `docs/start-sequence.gif`.
- `make readme-media`: regenerate both committed README media files.
- `make readme-media-check`: verify committed README media is current.
- `make ci`: run non-graphical CI checks and coverage.
- `make ci-smoke`: run CI smoke checks; Linux CI wraps this with `xvfb-run`.
- `make ci-doctor`: check CI coverage and smoke prerequisites.
- `make coverage`: generate lcov and Cobertura coverage reports.
- `make docs-lint`: lint `README.md`.
- `make diff-check`: run whitespace hygiene over the working tree diff.
- `make clean`: remove Cargo build outputs and local scanner artifacts.
- `make release-gate`: run the local release checks and verify README media.

The README media targets render the accepted actor runtime through the same
sprite atlas path used by the native renderer. They do not depend on the
conversion tooling.

## Play

Launch the game:

```sh
cargo run
```

The default keyboard profile is `planetoid`.

Planetoid controls:

- `1`: start one-player game
- `2`: start two-player game
- `5`: insert coin
- `A`: move up
- `Z`: move down
- `SHIFT`: reverse ship direction
- `SPACE`: thrust in the current facing direction
- `ENTER`: fire
- `TAB`: smart bomb
- `H`: hyperspace
- `BACKSPACE`: delete the previous high-score initial
- `ESC`: quit

Cabinet-style controls are available with:

```sh
cargo run -- --input-profile cabinet
```

Cabinet controls:

- `1`: start one-player game
- `2`: start two-player game
- `5`: insert coin
- `UP`: move up
- `DOWN`: move down
- `R`: reverse ship direction
- `T`: thrust
- `F`: fire
- `B`: smart bomb
- `H`: hyperspace
- `BACKSPACE`: delete the previous high-score initial
- `ESC`: quit

Normal interactive live play enables free-play admission, so `1` and `2` can
start a fresh run without pressing `5` first. Coin input remains available for
credit-gated test and evidence flows.

## XYZZY Mode

During a live session, type `X`, `Y`, `Z`, `Z`, `Y` to toggle hidden `XYZZY`
mode on or off.

Extra keys and behaviour while `XYZZY` mode is active:

- the normal four-shot arcade laser cap is removed.
- smart bombs become unlimited and also clear bullets and mines.
- `F`: toggle fully automatic firing.
- `G`: toggle god mode.
- `H`: hyperspace becomes safe where possible.
- falling humanoids always survive the landing.

`XYZZY` is a deliberate compatibility/extra mode outside the standard cabinet
rules.

## Runtime Notes

- Live CMOS persistence is opt-in. Without `--cmos-path`, each run starts from
  embedded arcade CMOS defaults.
- Actor scripts can be supplied with
  `cargo run -- --actor-script /path/to/driver.script`.
- The checked custom-driver starting point is
  `examples/actor-custom-attract.script`.
- The retained actor scripts and tables live under `assets/arcade-scripts/`.
- The retained sprite source image is `assets/sprites/font-sheet.png`; object
  and terrain sprites are generated from embedded typed asset tables.
- Committed README media lives at `docs/defender.png` and
  `docs/start-sequence.gif`.

## SonarQube

- `make sq-ci` generates the Cobertura coverage report used by the SonarCloud
  workflow in CI.
- `make sq` runs the same coverage step locally and then invokes
  `sonar-scanner`.
- Coverage runs under the `stable` Rust toolchain by default; override with
  `COVERAGE_TOOLCHAIN=<toolchain>` when needed.
- Local SonarQube scans require `cargo-llvm-cov`, `llvm-tools-preview` for the
  coverage toolchain, `sonar-scanner`, and a `SONAR_TOKEN` environment
  variable.

## Platform Support

The live runtime uses `wgpu`, `winit`, and `cpal`.

The primary supported desktop path is macOS on the local development machine.
The code is structured for portable native `wgpu` support, and CI-style smoke
targets include headless/offscreen `wgpu` readback where the platform provides
the required graphics stack. Linux headless validation may require Mesa/Vulkan,
`xvfb-run`, and related packages documented by the Makefile doctor targets.

## Source Materials

These references were used for rules verification, media comparison, attract
reconstruction, graphics reconstruction, and audio validation while keeping the
normal runtime self-contained:

- <https://seanriddle.com/ripper.html>: Williams graphics-ripper notes.
- <https://www.mamechannel.it/files_free/arcade_manuals_unpacked/defenderw.pdf>:
  Williams Defender operations manual.
- <https://williamsarcades.com/Defender>: cabinet and control-panel reference.
- <https://mdk.cab/game/defender>: artwork, screenshots, and cabinet material.
- <https://www.andysarcade.net/personal/defcolours/index.htm>: cabinet colour
  and palette reference.
- <https://thekingofgrabs.com/2018/02/15/defender-arcade/>: gameplay, laser,
  explosion, sprite, and attract-screen image reference.
- <https://www.youtube.com/watch?v=uOI7x1zWuLA>: Defender sound reference.
- <https://www.youtube.com/watch?v=kOAidQcQcNI>: Defender sound reference.
- <https://www.youtube.com/watch?v=6w2cKBWx2Uc>: arcade attract-mode capture.
- <https://www.dougmahugh.com/defender-chapter01/>: bomber timing, radar,
  wave, humanoid, and scoring rules.
- <https://www.dougmahugh.com/defender-chapter02/>: controls, hyperspace,
  reverse inertia, shot limit, and smart-bomb behaviour.
- <https://www.dougmahugh.com/defender-chapter03/>: lander fire behaviour.
- <https://www.dougmahugh.com/defender-chapter04/>: mutant behaviour.
- <https://www.dougmahugh.com/defender-chapter05/>: swarmer and pod behaviour.
- <https://www.dougmahugh.com/defender-chapter06/>: bomber behaviour.
- <https://www.dougmahugh.com/defender-chapter07/>: baiter behaviour.
- <https://www.arcade-history.com/?id=614&n=defender&page=detail>: scoring and
  gameplay reference.
- <https://strategywiki.org/wiki/Defender/Gameplay>: gameplay, wave, enemy,
  and humanoid behaviour cross-check.
- <https://strategywiki.org/wiki/Defender/Walkthrough>: rescue strategy and
  scoring cross-check.
- <https://en.wikipedia.org/wiki/Defender_%281981_video_game%29>: general
  rules and historical reference.
- <https://www.digitpress.com/reviews/defender.htm>: reverse-with-inertia
  gameplay reference.
- <https://bbcmicro.co.uk/game.php?id=11>: BBC Micro `Planetoid` archive entry
  used to anchor the default keyboard profile.
