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

This repository is a native Rust reimplementation of Williams' `Defender`.

The current milestone focuses on a solid native-Rust foundation for the start
logo, attract sequence, high-score presentation, ROM-set auditing, CI, and test
coverage before a richer live arcade loop lands. The game logic is native Rust;
ROMs are treated as reference material only, and the current sound cues are
synthesized in-process so the app does not depend on external audio files.
The target is a faithful recreation of the original arcade game, with hidden
`xyzzy` extras as the deliberate behavior outside the original cabinet rules.

![Defender gameplay frame](docs/defender.png)

<!-- markdownlint-disable MD033 -->
<p align="center">
  <img
    src="docs/start-sequence.gif"
    alt="Defender attract sequence with logo, attract panel, and high-score screen"
  />
</p>
<!-- markdownlint-enable MD033 -->

Run targets:

- `cargo run`
- `cargo run -- --mute`
- `cargo run -- --rom-report assets/roms/defender`
- `make run`
- `make run-muted`
- `cargo test`
- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `make ci`
- `make coverage`
- `make sq-ci`
- `make sq`
- `make readme-media`

## Install

Install directly from git with Cargo:

- `cargo install --git https://github.com/stephenlclarke/defender defender`

After installation, run the prototype with:

- `defender`
- `defender --mute`
- `defender --rom-report assets/roms/defender`

## Controls

Live mode controls:

- `ENTER` or `1`: start from the title screen, or restart after game over
- `A`: move up
- `Z`: move down
- `SHIFT`: thrust forward
- `SPACE`: flip the ship's facing direction
- `ENTER`: fire the laser bolt
- `TAB`: detonate a smart bomb
- `H`: trigger hyperspace
- `BACKSPACE`: delete the previous letter during high-score initials entry
- `Q` or `ESC`: quit

Letter-key controls accept either upper- or lower-case input.

The live key layout is currently modelled on the BBC Micro `Planetoid`
control scheme from Acornsoft's 1982 release of its `Defender` variant.

## XYZZY Mode

During a live session, type `X`, `Y`, `Z`, `Z`, `Y` to toggle hidden `XYZZY`
mode on or off.

Typing `XYZZY` a second time turns the mode off and resets the hidden
invincibility and auto-fire toggles back to their default state.

Extra keys and game behaviour while `xyzzy` mode is active:

- smart bombs become unlimited automatically while `xyzzy` mode is enabled,
  and `xyzzy` smart bombs also clear bullets and mines on the main screen.
- `F`: toggle fully automatic firing. While active, the ship fires
  automatically when a current laser shot would destroy an alien directly ahead
  in the active firing lane.
- `G`: toggle god mode. While active, enemy fire and mines cannot kill the
  player, and ramming an enemy destroys it and awards its normal score instead
  of costing a ship.
- `H`: hyperspace becomes safe and cannot destroy the player ship while
  `xyzzy` mode is active, and the rematerialization point is chosen as far away
  from enemies as possible.

## Current Notes

- The public CLI is now intentionally narrow: `cargo run` launches the live
  game, `--mute` disables audio, and `--rom-report` stays as the one
  non-interactive inspection path.
- All current sounds are embedded in the app via synthesized `rodio` cues,
  following the same self-contained runtime principle used in `../battlezone`.
- `cargo run` / `defender` now launch the real text-mode play loop with keyboard
  input, title/start flow, player shots, incoming enemy fire, smart bombs,
  hyperspace, enemy hits, wave progression, human abductions, falling-human
  recovery, grounded-human pickup and redeploy, Defender-style rescue scoring
  with `500` catch bonuses, `500` return-to-ground bonuses, and `250`
  low-altitude safe-fall saves, wave-end humanoid survivor bonuses, default
  `10,000`-point extra ship and smart-bomb awards, a ten-humanoid population
  that now restores on every fifth wave, mutant conversion after successful
  abductions, full lander-to-mutant conversion when the last humanoid is lost,
  and mutant replacement waves until the next fifth-wave planet restoration,
  persistent five-entry high-score tracking with initials entry, active lander
  pursuit of free humans, mutant pursuit of the player, and an arcade-style
  facing model where `Space` flips the ship direction while `Shift` thrust and
  `Enter` fire follow the current heading, with horizontal momentum preserved
  until you counter-thrust, and risky hyperspace behavior outside `xyzzy` mode,
  while `xyzzy` hyperspace deliberately relocates the ship away from enemy
  clusters, with automatic unlimited smart bombs in `xyzzy` mode, a separate
  `G` invincibility toggle, and an `F` auto-fire toggle that only shoots when
  the current firing lane has a direct alien kill, plus pod waves that now
  start on wave two and ramp up to four ships by wave four, release
  deterministic five-to-seven swarmer bursts when shot, and preserve the
  classic follow-from-behind swarmer counterplay, with 15-enemy attack waves
  now opening with five landers and then delivering two later five-ship
  reinforcement groups before later-wave bombers and pods layer on top, while
  destroyed-planet mutant phases follow the same staged reinforcement pattern, a
  full-width radar scanner strip that compresses the wrapped world into the top
  HUD,
  bomber waves that arrive from wave two onward and leave persistent mine
  trails that normal smart bombs do not clear, smart bombs that now only
  destroy enemies on the main screen instead of globally in normal play, with
  `xyzzy` mode extending the bomb to clear on-screen bullets and mines too, and
  baiters that beam in when a wave drags on while landers remain, then clear as
  soon as the last lander is gone, plus arcade-style collision scoring where
  ramming enemies still awards their
  normal value and dying on bullets or mines still grants `25` points, while
  `G` god mode blocks bullet and mine deaths and turns direct enemy crashes
  into safe kills that still pay their normal value,
  game-over handling, restart support, a wrapped scrolling camera, and an
  `xyzzy`/`g` secret-mode path on top of the same native Rust world model.
- The live/bootstrap world now uses a deterministic scrolling terrain profile
  instead of a flat floor, so demo and live frames show a moving landscape and
  projectiles are clipped by terrain. When the last humanoid is lost, the live
  renderer now drops the terrain line and marks the HUD as `DEEP SPACE` until a
  fifth-wave restoration brings the planet and all ten humanoids back.
- Defender’s gameplay-fidelity and AI tuning values now live in
  `assets/arcade/arcade-rules.txt`, loaded through `src/arcade.rs` in the same
  embedded-text-table style used by `../pacman` and `../battlezone`, so future
  arcade-rule adjustments do not add more hard-coded tuning constants to
  `src/game.rs`.
- Gameplay work is being prioritized toward faithful Williams-arcade behavior in
  Rust first; hidden `xyzzy` options remain the only intentional rules
  extension outside that baseline.
- The current renderer is still deliberately text-first so the repo can
  establish game-state, ROM-reference, CI, and test coverage foundations before
  adding a fuller terminal graphics path.
- `defender --rom-report` checks the expected Williams ROM filenames and reports
  missing or unexpected files from a local directory.
- Local ROM material under `assets/roms/` remains development-only reference
  data and is ignored by git.
- CI runs formatting, tests, clippy, Sonar coverage, and Miri-based leak checks
  on both Linux and macOS.
- Local SonarQube wiring is exposed through `make sq`, which generates the same
  coverage report as CI and then runs `sonar-scanner` when `SONAR_TOKEN` is set.
- `examples/generate_readme_media.rs` regenerates `docs/defender.png` from the
  gameplay demo path and `docs/start-sequence.gif` from the shared attract
  builders used by the app and README media pipeline.
- High scores persist between live runs in `~/.xyzzy/defender/high_scores.txt`;
  set `DEFENDER_DATA_DIR` to redirect that file for local experiments or tests.

## SonarQube

- `make sq-ci` generates the Cobertura coverage report used by the SonarCloud
  workflow in CI.
- `make sq` runs the same coverage step locally and then invokes
  `sonar-scanner`.
- Local SonarQube scans require `cargo-llvm-cov`, `sonar-scanner`, and a
  `SONAR_TOKEN` environment variable.

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

- <https://github.com/mwenge/defender>: Motorola 6809 assembly language for the
  'Red Label' version of the game. Used for reference implementation and ROM
  layout comparison point, especially for the red-label `defend.*` program ROM
  names.
- <https://www.thedefenderproject.com/defender-rom-versions-the-history/>:
  revision history and ROM-set background for Williams Defender releases.
- <https://www.dougmahugh.com/defender-chapter01/>: general arcade-rules
  reference used for bomber wave timing, bomber-mine danger, radar behavior,
  ten-humanoid/fifth-wave restoration rules, the three-group 15-enemy attack
  wave structure, mutant-only post-extinction wave behavior, and the scoring
  rule that collision deaths on bullets or mines still award `25` points while
  ramming enemies still scores their normal value.
- <https://williamsarcades.com/Defender>: original cabinet and control-panel
  reference used to keep the Rust controls and cabinet assumptions aligned with
  Williams' arcade hardware.
- <https://www.arcade-history.com/?id=614&n=defender&page=detail>: scoring and
  gameplay reference used for humanoid rescue values, safe-fall saves, and wave
  bonus behavior.
- <https://en.wikipedia.org/wiki/Defender_%281981_video_game%29>: general rules
  reference used to cross-check the default `10,000`-point extra ship and smart
  bomb award behavior.
- <https://strategywiki.org/wiki/Defender/Gameplay>: gameplay reference used to
  cross-check live enemy and humanoid behavior against the original arcade
  rules, including opening five-lander attack waves, later five-ship
  reinforcement groups, later pod scheduling, and the destroyed-planet mutant-
  wave cycle until the next fifth-round restore.
- <https://strategywiki.org/wiki/Defender/Walkthrough>: rescue-strategy and
  scoring reference used for the current `500` catch / `500` return humanoid
  rescue implementation.
- <https://www.mamechannel.it/files_free/arcade_manuals_unpacked/defenderw.pdf>:
  Defender operations manual used to confirm that `Reverse` flips ship
  direction while `Thrust` controls forward movement.
- <https://www.digitpress.com/reviews/defender.htm>: secondary gameplay
  reference used to model Defender's reverse-with-inertia handling, where the
  ship keeps its current momentum until thrust changes it.
- <https://www.dougmahugh.com/defender-chapter02/>: control-analysis reference
  used for hyperspace risk, stopped re-entry speed, direction changes on
  rematerialization, and the rules that smart bombs only destroy enemies on the
  main screen while leaving bullets and bomber minefields alone.
- <https://www.dougmahugh.com/defender-chapter05/>: swarmer-behavior reference
  used to model pod bursts, delayed swarmer turnback, and the follow-from-
  behind movement pattern.
- <https://www.dougmahugh.com/defender-chapter06/>: bomber-behavior reference
  used to model wave-two bomber introduction, altitude-triggered speed boosts,
  and the persistent mine trails they leave behind.
- <https://www.dougmahugh.com/defender-chapter07/>: baiter-behavior reference
  used to model wave-delay baiter pressure and relative pursuit behavior.
- <https://www.andysarcade.net/personal/defcolours/index.htm>: cabinet colour
  and palette reference for later presentation work.
- <https://mdk.cab/game/defender>: artwork, screenshots, and general cabinet
  reference material for start-screen and attract-sequence planning.
- <https://bbcmicro.co.uk/game.php?id=11>: BBC Micro `Planetoid` archive entry
  used to anchor the current keyboard layout to the Acornsoft 1982 home-port
  control scheme.

## Platform Support

The current build stays in plain ANSI terminal output rather than Kitty
graphics, so the non-interactive tooling paths such as `--rom-report` and the
README media generator should run anywhere a recent Rust toolchain is
available.

The live loop uses `crossterm` raw-mode keyboard input, so `cargo run` /
`defender` must be launched from a real interactive terminal session rather
than piped or captured stdout. The long-term target is still a richer terminal
arcade presentation similar to the sibling Rust repos in this directory, but
the current milestone keeps the renderer text-first so CI, leak detection,
input handling, and ROM validation can land before a fuller graphics path.

The live session now requests terminal keyboard-enhancement reporting so
standalone `Shift` thrust input can be captured in terminals that support the
extended key protocol.
