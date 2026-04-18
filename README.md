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

---

This repository is the first native Rust pass at Williams' `Defender`.

The current milestone focuses on a solid native-Rust foundation for the start
logo, attract sequence, high-score presentation, ROM-set auditing, CI, and test
coverage before a richer live arcade loop lands. The game logic is native Rust;
ROMs are treated as reference material only, and the current sound cues are
synthesized in-process so the app does not depend on external audio files.

![Defender](docs/defender.png)

<!-- markdownlint-disable MD033 -->
<p align="center">
  <img
    src="docs/start-sequence.gif"
    alt="Defender logo, attract panel, and high-score sequence"
  />
</p>
<!-- markdownlint-enable MD033 -->

Run targets:

- `cargo run`
- `cargo run -- --audio-demo`
- `cargo run -- --play-demo`
- `cargo run -- --play-demo --mute --no-sleep`
- `cargo run -- --play-attract`
- `cargo run -- --play-attract --mute --no-sleep`
- `cargo run -- --play-live`
- `cargo run -- --play-live --mute`
- `cargo run -- --scene logo`
- `cargo run -- --scene attract`
- `cargo run -- --scene high-score`
- `cargo run -- --frames 8`
- `cargo run -- --rom-report assets/roms/defender`
- `make live`
- `cargo test`
- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo run --example generate_readme_media`
- `make ci`

## Install

Install directly from git with Cargo:

- `cargo install --git https://github.com/stephenlclarke/defender defender`

After installation, run the prototype with:

- `defender`
- `defender --audio-demo`
- `defender --play-demo`
- `defender --play-attract`
- `defender --play-live`
- `defender --scene attract`
- `defender --scene high-score`
- `defender --frames 8`

## Controls

Live mode controls:

- `Enter` or `1`: start from the title screen, or restart after game over
- `A` / `D` or `Left` / `Right`: move horizontally
- `W` / `S` or `Up` / `Down`: move vertically
- `Space`: fire
- `q` or `Esc`: quit

## Current Notes

- The app now has explicit `logo`, `attract`, and `high-score` scenes so the
  README screenshot and animated preview are generated from real application
  output rather than mocked assets.
- All current sounds are embedded in the app via synthesized `rodio` cues,
  following the same self-contained runtime principle used in `../battlezone`.
- `defender --play-attract` now runs the logo, attract, and high-score sequence
  as a real terminal playback path; use `--mute --no-sleep` for deterministic
  test or capture runs.
- `defender --play-demo` now runs a scripted gameplay showcase with score,
  wave, life, enemy, and human-count changes through the same playback/audio
  path.
- `defender --play-live` now runs a real text-mode play loop with keyboard
  input, title/start flow, player shots, incoming enemy fire, enemy hits, wave
  progression, human losses, game-over handling, and restart support on top of
  the same native Rust world model.
- The current renderer is still deliberately text-first so the repo can
  establish game-state, ROM-reference, CI, and test coverage foundations before
  adding a fuller terminal graphics path.
- `defender --rom-report` checks the expected Williams ROM filenames and reports
  missing or unexpected files from a local directory.
- Local ROM material under `assets/roms/` remains development-only reference
  data and is ignored by git.
- CI runs formatting, tests, clippy, Sonar coverage, and Miri-based leak checks
  on both Linux and macOS.
- `examples/generate_readme_media.rs` regenerates `docs/defender.png` and
  `docs/start-sequence.gif` from the same shared attract-cycle definition used
  by the app.

## Reference Repos

- `../battlezone`: the primary local template for crate layout, CI, SonarCloud,
  README structure, and self-contained synthesized audio for these terminal
  arcade rewrites.
- `../pacman`: secondary local reference for README/media conventions and
  workflow shape across the sibling Rust arcade repos, including an
  embedded-audio path based on bundled sound assets.
- <https://github.com/mwenge/defender>: external Defender rewrite used to
  compare canonical ROM naming and overall project direction.

## Source Materials

These references were used for reverse engineering, rules verification, attract
screen reconstruction, and extraction of historical arcade data while keeping
the final runtime self-contained:

- <https://github.com/mwenge/defender>: reference implementation and ROM layout
  comparison point, especially for the red-label `defend.*` program ROM names.
- <https://www.thedefenderproject.com/defender-rom-versions-the-history/>:
  revision history and ROM-set background for Williams Defender releases.
- <https://www.andysarcade.net/personal/defcolours/index.htm>: cabinet colour
  and palette reference for later presentation work.
- <https://mdk.cab/game/defender>: artwork, screenshots, and general cabinet
  reference material for start-screen and attract-sequence planning.

## Platform Support

The current build stays in plain ANSI terminal output rather than Kitty
graphics, so the attract/demo modes should run anywhere a recent Rust toolchain
is available.

The new live loop uses `crossterm` raw-mode keyboard input, so `--play-live`
must be launched from a real interactive terminal session rather than piped or
captured stdout. The long-term target is still a richer terminal arcade
presentation similar to the sibling Rust repos in this directory, but the
current milestone keeps the renderer text-first so CI, leak detection, input
handling, and ROM validation can land before a fuller graphics path.
