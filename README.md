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

This repository is a native Rust reimplementation of Williams' `Defender`,
rendered through the Kitty graphics protocol.

The game logic is native Rust; ROMs are treated as reference material only, and
the app now embeds its gameplay object art as PNGs from `assets/arcade/`, with
the red-label text font bundled as `assets/arcade/font-sheet.png`, plus
ROM-derived branding art in `assets/arcade/logo-page.png` and
`assets/arcade/defender-logo.png`. Live audio is now synthesized in Rust from
Williams sound-ROM routines translated out of `VSNDRM1.SRC`, so compile and
runtime do not depend on a local ROM or sound directory. The target is a
faithful recreation of the original red-label arcade game, with hidden `xyzzy`
extras as the deliberate behavior outside the original cabinet rules.

![Defender gameplay frame](docs/defender.png)

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

Run targets:

- `cargo run`
- `cargo run -- --mute`
- `cargo run -- --rom-report`
- `cargo run -- --rom-report /path/to/roms`
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

Run the live game inside `kitty`, `ghostty`, `warp`, or another terminal that
supports the Kitty graphics protocol. `--rom-report` remains non-interactive
and does not require a compatible graphics terminal.

## Install

Install directly from git with Cargo:

- `cargo install --git https://github.com/stephenlclarke/defender defender`

After installation, run the prototype with:

- `defender`
- `defender --mute`
- `defender --rom-report`
- `defender --rom-report /path/to/roms`

Notes:

- Run `defender` inside `kitty`, `ghostty`, `warp`, or another terminal that
  supports the Kitty graphics protocol.
- Download Ghostty: <https://ghostty.org/download>
- Download Warp: <https://www.warp.dev/download>
- High scores persist between live runs in `~/.xyzzy/defender/high_scores.txt`;
  set `DEFENDER_DATA_DIR` to redirect that file for local experiments or tests.

## Controls

Live mode controls:

- `ENTER` or `1`: start from the attract sequence, or restart after game over
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

- the normal four-shot arcade laser cap is removed while `xyzzy` mode is
  active.
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
- falling humanoids always survive the landing while `xyzzy` mode is active,
  even after a full-height drop.

## Current Notes

- The public CLI is now intentionally narrow: `cargo run` launches the live
  game, `--mute` disables audio, and `--rom-report` stays as the one
  non-interactive inspection path.
- The executable does not require `assets/roms/` at compile time or runtime.
  `--rom-report` now prints the embedded canonical red-label filename list when
  no directory is supplied, and only touches the filesystem when you pass an
  explicit ROM path.
- Gameplay object art now loads from embedded `assets/arcade/*.png` sprites,
  following the same bundled-asset layout used in `../pacman`.
- UI and attract text now render through the embedded
  `assets/arcade/font-sheet.png` sheet built from the red-label `mess0.src`
  character tables instead of the old generic bitmap font.
- The first attract page and the shared `DEFENDER` wordmark now come from
  ROM-derived assets generated from the red-label `amode1.src` `LGOTAB`,
  `DEFDAT`, and `CPRTAB` tables, embedded as
  `assets/arcade/logo-page.png` and `assets/arcade/defender-logo.png`.
- The live attract renderer now also decodes the red-label `LGOTAB` and
  `DEFDAT` tables directly in Rust, so the opening page traces the Williams
  script and materializes the `DEFENDER` wordmark from ROM-derived data before
  falling back to the exact embedded page asset for the held frame. The title
  page now also preserves the ROM's short full-logo hold before the copyright
  line appears, rather than jumping straight from the reverse explosion to the
  final page.
- The instruction/demo page now uses the ROM-timed `LEDRET` rescue/scoring
  sequence in native fixed-point attract coordinates rather than routing those
  poses through the normal gameplay world update loop.
- The first attract-page and hall-of-fame screens now keep a live
  elapsed-time colour cycle running during the first `16` seconds of the
  cabinet sequence, matching the flashing/cycling behaviour visible in the
  original attract-mode capture instead of holding a fixed palette per beat.
- The live gameplay renderer now uses a cabinet-style open black playfield with
  amber terrain, clipped sprite drawing, a centered scanner block with score
  side pods, a sparser mixed-colour starfield, colour-cycling attract `500`
  bonus text, a segmented cycling attract/gameplay laser beam, and larger
  hall-of-fame score rows, so the README screenshot and attract GIF are closer
  to the original cabinet framing and effects.
- Gameplay sprite phasing now follows ROM-informed display rules more closely:
  the player ship uses left/right cabinet art with horizontal display-phase
  selection instead of a made-up tick animation, shared enemy animation now
  follows a single cabinet-style cadence instead of per-entity position seeds,
  the HUD uses the little-ship stock icon, and the attract rescue/scoring page
  now draws its `250` / `500` bonuses from embedded score art in
  `assets/arcade/`.
- Live audio now comes from `src/audio_rom.rs`, which translates the Williams
  `VSNDRM1.SRC` radio, filtered-noise, scream, organ, and GWAVE routines into
  Rust sample generation instead of decoding pre-rendered WAV cue files.
- `cargo run` / `defender` now launch the real Kitty-graphics play loop with keyboard
  input, title/start flow, player shots, incoming enemy fire, smart bombs,
  hyperspace, enemy hits, wave progression, human abductions, falling-human
  recovery, grounded-human pickup and redeploy, Defender-style rescue scoring
  with `500` catch bonuses, `500` return-to-ground bonuses, and `250`
  low-altitude safe-fall saves, wave-end humanoid survivor bonuses, default
  `10,000`-point extra ship and smart-bomb awards, a ten-humanoid population
  that now restores on every fifth wave, mutant conversion after successful
  abductions, full lander-to-mutant conversion when the last humanoid is lost,
  and mutant replacement waves until the next fifth-wave planet restoration,
  persistent eight-entry high-score tracking with initials entry, active lander
  pursuit of free humans, mutant pursuit of the player, and an arcade-style
  facing model where `Space` flips the ship direction while `Shift` thrust and
  `Enter` fire follow the current heading, with horizontal momentum preserved
  until you counter-thrust, with the arcade four-shot laser cap restored in
  normal play and player shots now expiring at the main-screen edge instead of
  wandering the whole wrapped world, and risky hyperspace behavior outside
  `xyzzy` mode, while `xyzzy` hyperspace deliberately relocates the ship away
  from enemy clusters, with automatic unlimited smart bombs in `xyzzy` mode,
  the normal shot cap removed in `xyzzy`, a separate `G` invincibility toggle,
  an `F` auto-fire toggle that only shoots when the current firing lane has a
  direct alien kill, and secret-mode full-height humanoid fall survival, plus
  enemy fire now limited to visible main-screen threats, with lander/mutant/
  baiter volleys using the arcade-style lob-versus-chaser split and swarmer
  shots using the quarter-screen focal-point model from the original game, plus
  pod waves that now follow the red-label `WVTAB` counts from `blk71.src`,
  release deterministic five-to-seven swarmer bursts when shot, and preserve
  the classic follow-from-behind swarmer counterplay, with lander reserve
  counts and reinforcement timing now coming from that same `WVTAB` data, so
  wave one still opens as three five-ship lander groups while wave two and
  later use the ROM-recorded twenty-lander/four-group opener before the
  recorded Bomber/TIE and Pod/PROBE counts layer on top, while destroyed-planet
  mutant phases follow the same staged reinforcement pattern, a
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
- Live startup now goes straight into the cabinet attract sequence instead of a
  separate modern title page: the ROM-ordered Williams / Presents logo page
  leads into the split `TODAYS GREATEST` / `ALL TIME GREATEST` hall-of-fame
  page and then the instruction/demo page, with the volatile per-run left table
  and persistent right table matching the original attract flow more closely,
  and the staged logo/scoring cadence now cross-checked against the original
  arcade attract video reference.
- The live game-over state now uses a sparse arcade-style overlay instead of
  the earlier restart panel: a thin top strip with score and scanner, the open
  playfield below it, amber terrain, and a centered `GAME OVER` message.
- The live/bootstrap world now uses a deterministic scrolling terrain profile
  instead of a flat floor, so demo and live frames show a moving landscape and
  projectiles are clipped by terrain. When the last humanoid is lost, the live
  renderer now drops the terrain line and marks the HUD as `DEEP SPACE` until a
  fifth-wave restoration brings the planet and all ten humanoids back.
- Defender’s gameplay-fidelity, AI tuning, and the ROM-derived red-label
  wave/fire records now all live in `assets/arcade/arcade-rules.txt`, so the
  live game no longer depends on hand-authored attack-wave counts or
  reinforcement timing constants in `src/game.rs`.
- Gameplay work is being prioritized toward faithful Williams-arcade behavior in
  Rust first; hidden `xyzzy` options remain the only intentional rules
  extension outside that baseline.
- `defender --rom-report` works without a ROM directory and prints the embedded
  expected Williams ROM filenames; `defender --rom-report /path/to/roms`
  compares a local directory and reports missing or unexpected files.
- Local ROM material under `assets/roms/` remains development-only reference
  data and is ignored by git.
- CI runs formatting, tests, clippy, Sonar coverage, and Miri-based leak checks
  on both Linux and macOS.
- Local SonarQube wiring is exposed through `make sq`, which generates the same
  coverage report as CI and then runs `sonar-scanner` when `SONAR_TOKEN` is set.
- `examples/generate_readme_media.rs` now regenerates `docs/defender.png` and
  `docs/start-sequence.gif` from the same Kitty-graphics renderer used by the
  live game and attract flow, with the README GIF sampled at a denser interval
  so the Williams handwriting, reverse-explosion title reveal, and opening
  colour cycling stay smooth while preserving the cabinet sequence timing.
- `tools/extract_rom_branding_assets.py` regenerates the embedded Williams
  logo page, Defender wordmark, and copyright art directly from the red-label
  source tables.
- `tools/extract_rom_attract_data.py` regenerates the Rust tables used by the
  live attract-page Williams trace and `DEFENDER` materialization code.
- `tools/extract_rom_wave_table.py` regenerates the red-label `WVTAB` block
  inside `assets/arcade/arcade-rules.txt` from the source `blk71.src` records.
- The title, attract legend, and hall-of-fame seed data now use the red-label
  ROM message/default tables instead of the earlier placeholder prototype
  strings.
- High scores persist between live runs in `~/.xyzzy/defender/high_scores.txt`;
  set `DEFENDER_DATA_DIR` to redirect that file for local experiments or tests.
  The attract-mode `TODAYS GREATEST` table is intentionally volatile and resets
  from the red-label ROM defaults each new app launch.

## SonarQube

- `make sq-ci` generates the Cobertura coverage report used by the SonarCloud
  workflow in CI.
- `make sq` runs the same coverage step locally and then invokes
  `sonar-scanner`.
- Local SonarQube scans require `cargo-llvm-cov`, `sonar-scanner`, and a
  `SONAR_TOKEN` environment variable.

## Reference Repos

- `../battlezone`: the primary local template for crate layout, CI, SonarCloud,
  README structure, and Kitty-graphics terminal workflow for these arcade
  rewrites.
- `../pacman`: secondary local reference for README/media conventions and
  workflow shape across the sibling Rust arcade repos, including the direct
  `assets/arcade/*.png` asset layout and bundled-media documentation pattern.
- <https://github.com/mwenge/defender>: external Defender rewrite used to
  compare canonical ROM naming and overall project direction.

## Source Materials

These references were used for reverse engineering, rules verification, attract
screen reconstruction, and extraction of historical arcade data while keeping
the final runtime self-contained:

- <https://github.com/mwenge/defender>: Motorola 6809 assembly language for the
  'Red Label' version of the game. Used for reference implementation and ROM
  layout comparison point, especially for the red-label `defend.*` program ROM
  names, the `mess0.src` `CHRTBL` / `CHARACTERS` font tables used to build the
  embedded `assets/arcade/font-sheet.png`, and the `amode1.src` `LGOTAB`,
  `DEFDAT`, and `CPRTAB` tables used to reconstruct the embedded attract-logo
  page and `DEFENDER` wordmark assets, plus the `blk71.src` `WVTAB` records
  used to reconstruct the ROM-derived wave/fire block in
  `assets/arcade/arcade-rules.txt`.
- <https://github.com/historicalsource/williams-soundroms>: original Williams
  sound-ROM source reference used to translate `VSNDRM1.SRC` routines and
  tables into `src/audio_rom.rs`, including the `IRQ` dispatch path, `RADSND`,
  `SVTAB`, `GFRTAB`, `GWVTAB`, `SCREAM`, and organ-note/tune logic.
- <https://seanriddle.com/ripper.html>: Williams graphics-ripper reference used
  to confirm Defender's screen-format sprite layout and the red-label object
  sprite list/rip used to build the embedded `assets/arcade/*.png` object art.
- <https://www.thedefenderproject.com/defender-rom-versions-the-history/>:
  revision history and ROM-set background for Williams Defender releases.
- <https://www.mamechannel.it/files_free/arcade_manuals_unpacked/defenderw.pdf>:
  Defender operations manual used to confirm that `Reverse` flips ship
  direction while `Thrust` controls forward movement.
- <https://williamsarcades.com/Defender>: original cabinet and control-panel
  reference used to keep the Rust controls and cabinet assumptions aligned with
  Williams' arcade hardware.
- <https://mdk.cab/game/defender>: artwork, screenshots, and general cabinet
  reference material for start-screen and attract-sequence planning.
- <https://www.andysarcade.net/personal/defcolours/index.htm>: cabinet colour
  and palette reference for later presentation work.
- <https://www.dougmahugh.com/defender-chapter01/>: general arcade-rules
  reference used for bomber wave timing, bomber-mine danger, radar behavior,
  ten-humanoid/fifth-wave restoration rules, the three-group 15-enemy attack
  wave structure, mutant-only post-extinction wave behavior, and the scoring
  rule that collision deaths on bullets or mines still award `25` points while
  ramming enemies still scores their normal value.
- <https://www.dougmahugh.com/defender-chapter02/>: control-analysis reference
  used for hyperspace risk, stopped re-entry speed, direction changes on
  rematerialization, the four-shot laser cap, the rule that shots only remain
  active until they outrun the main screen, and the rules that smart bombs only
  destroy enemies on the main screen while leaving bullets and bomber
  minefields alone.
- <https://www.dougmahugh.com/defender-chapter03/>: lander-fire reference used
  for the broad-arc shot model, the alternating chaser/lob split shared with
  mutants and baiters, and the requirement that enemies only fire while they
  are on the main screen.
- <https://www.dougmahugh.com/defender-chapter04/>: mutant-behaviour reference
  used to keep the more aggressive mutant fire path aligned with the same
  broad-arc/chaser model as the original game.
- <https://www.dougmahugh.com/defender-chapter05/>: swarmer-behavior reference
  used to model pod bursts, delayed swarmer turnback, the follow-from-behind
  movement pattern, and the quarter-screen-ahead swarmer firing focal point.
- <https://www.dougmahugh.com/defender-chapter06/>: bomber-behavior reference
  used to model wave-two bomber introduction, altitude-triggered speed boosts,
  and the persistent mine trails they leave behind.
- <https://www.dougmahugh.com/defender-chapter07/>: baiter-behavior reference
  used to model wave-delay baiter pressure and relative pursuit behavior.
- <https://www.arcade-history.com/?id=614&n=defender&page=detail>: scoring and
  gameplay reference used for humanoid rescue values, safe-fall saves, and wave
  bonus behavior.
- <https://strategywiki.org/wiki/Defender/Gameplay>: gameplay reference used to
  cross-check live enemy and humanoid behavior against the original arcade
  rules, including opening five-lander attack waves, later five-ship
  reinforcement groups, later pod scheduling, and the destroyed-planet mutant-
  wave cycle until the next fifth-round restore.
- <https://strategywiki.org/wiki/Defender/Walkthrough>: rescue-strategy and
  scoring reference used for the current `500` catch / `500` return humanoid
  rescue implementation.
- <https://en.wikipedia.org/wiki/Defender_%281981_video_game%29>: general rules
  reference used to cross-check the default `10,000`-point extra ship and smart
  bomb award behavior.
- <https://www.digitpress.com/reviews/defender.htm>: secondary gameplay
  reference used to model Defender's reverse-with-inertia handling, where the
  ship keeps its current momentum until thrust changes it.
- <https://bbcmicro.co.uk/game.php?id=11>: BBC Micro `Planetoid` archive entry
  used to anchor the current keyboard layout to the Acornsoft 1982 home-port
  control scheme.
- <https://www.youtube.com/watch?v=6w2cKBWx2Uc>: original arcade attract-mode
  capture used to verify the full attract-sequence order, Williams logo-page
  composition, and the scoring legend reveal shown around `0:26`.

## Customisation

Arcade tuning defaults and the ROM-derived red-label wave/fire records ship in
`assets/arcade/arcade-rules.txt`. To override them locally, copy that file to
`~/.xyzzy/defender/` and only include the keys you want to change. Omitted keys
keep their embedded defaults. If `DEFENDER_DATA_DIR` is set, the game reads
that file from the override directory instead.

### `arcade-rules.txt`

`safe_fall_height`
Default: `2`
Meaning: maximum drop height that still counts as a safe humanoid landing.

`safe_fall_score`
Default: `250`
Meaning: rescue score awarded when an uncaught humanoid survives a safe fall.

`human_catch_score`
Default: `500`
Meaning: score awarded when the player catches a falling humanoid.

`human_landing_score`
Default: `500`
Meaning: score awarded when a carried humanoid is returned safely to the
ground.

`hazard_collision_score`
Default: `25`
Meaning: score awarded when a bullet or mine kills the player, matching the
arcade hazard rule.

`bonus_stock_score`
Default: `10000`
Meaning: score interval that awards an extra ship and smart bomb.

`max_wave_humanoid_bonus`
Default: `500`
Meaning: upper cap on the end-of-wave surviving-humanoid bonus.

`player_max_speed`
Default: `1`
Meaning: maximum horizontal thrust increment applied to the player ship.

`player_shot_limit`
Default: `4`
Meaning: maximum number of player laser shots that can be active at once
outside `xyzzy` mode.

`player_shot_speed`
Default: `2`
Meaning: horizontal speed of the player's laser burst tip.

`enemy_shot_limit`
Default: `6`
Meaning: maximum number of enemy shots allowed to remain active at once.

`enemy_fire_base_delay`
Default: `5`
Meaning: shared firing cadence for landers, mutants, and baiters.

`enemy_fire_chaser_cycle`
Default: `2`
Meaning: every Nth non-swarmer firing volley becomes a chaser that adds the
player's horizontal motion to the shot.

`swarmer_fire_delay`
Default: `3`
Meaning: faster firing cadence used by swarmers.

`swarmer_fire_lead_divisor`
Default: `4`
Meaning: quarter-screen lead factor used for the swarmer focal-point shot
model.

`swarmer_speed`
Default: `2`
Meaning: horizontal speed used by swarmer enemies.

`baiter_speed`
Default: `2`
Meaning: horizontal speed used by baiter enemies.

`bomber_base_speed`
Default: `1`
Meaning: normal horizontal speed used by bomber enemies.

`bomber_evasive_speed`
Default: `2`
Meaning: faster bomber speed used when they cross the player altitude.

`max_baiters`
Default: `4`
Meaning: maximum number of baiters allowed on-screen at once.

`baiter_repeat_delay`
Default: `20`
Meaning: delay between additional baiter spawns while the wave stalls.

`pod_swarmer_burst_min`
Default: `5`
Meaning: minimum number of swarmers released when a pod is destroyed.

`pod_swarmer_burst_range`
Default: `3`
Meaning: additional swarmer burst range above the minimum.

`max_swarmers`
Default: `20`
Meaning: maximum number of live swarmers allowed at one time.

`bomber_mine_drop_delay`
Default: `3`
Meaning: tick delay between mine drops from a bomber.

`max_mines`
Default: `24`
Meaning: maximum number of live bomber mines allowed at once.

`default_human_world_xs`
Default: `8,26,44,62,80,98,116,134,152,170`
Meaning: default world X positions for the ten starting humanoids.

Record format for the ROM-derived wave/fire keys:
`max,min,intra_delta,inter_delta|wave1,wave2,wave3,wave4`

`landers`
Default: `20,0,0,0|15,20,20,20`
Meaning: red-label lander reserve count per wave, including the four-group
twenty-lander opener used from wave two onward.

`bombers`
Default: `3,0,0,0|0,3,4,5`
Meaning: red-label TIE/Bomber count per wave.

`pods`
Default: `6,0,0,0|0,1,3,4`
Meaning: red-label PROBE/Pod count per wave.

`mutants`
Default: `10,0,0,0|0,0,0,0`
Meaning: red-label SCHITZO/Mutant baseline count record.

`swarmers`
Default: `10,0,0,0|0,0,0,0`
Meaning: red-label baseline swarmer reserve record.

`wave_time`
Default: `30,0,0,0|30,25,20,16`
Meaning: red-label delay between successive lander squad launches.

`wave_size`
Default: `5,0,0,0|5,5,5,5`
Meaning: red-label size of each launched lander squad.

`lander_x_velocity`
Default: `96,0,3,2|22,30,38,46`
Meaning: red-label lander horizontal-speed record from `WVTAB`.

`lander_y_velocity_msb`
Default: `1,0,0,0|0,0,1,1`
Meaning: red-label lander vertical-speed MSB record from `WVTAB`.

`lander_y_velocity_lsb`
Default: `255,0,16,0|112,176,0,0`
Meaning: red-label lander vertical-speed LSB record from `WVTAB`.

`lander_shot_time`
Default: `128,16,-4,-2|74,58,42,42`
Meaning: red-label lander shot-timer record from `WVTAB`.

`bomber_x_velocity`
Default: `48,0,0,0|32,40,44,48`
Meaning: red-label bomber horizontal-speed record from `WVTAB`.

`mutant_random_y`
Default: `2,0,0,0|1,1,2,2`
Meaning: red-label mutant vertical-randomness record from `WVTAB`.

`mutant_y_velocity_msb`
Default: `1,0,0,0|0,0,1,1`
Meaning: red-label mutant vertical-speed MSB record from `WVTAB`.

`mutant_y_velocity_lsb`
Default: `255,0,8,6|98,224,2,18`
Meaning: red-label mutant vertical-speed LSB record from `WVTAB`.

`mutant_x_velocity`
Default: `96,0,8,4|12,28,36,40`
Meaning: red-label mutant horizontal-speed record from `WVTAB`.

`mutant_shot_time`
Default: `255,8,-2,-2|42,34,30,28`
Meaning: red-label mutant shot-timer record from `WVTAB`.

`swarmer_x_velocity`
Default: `96,0,8,2|22,30,32,34`
Meaning: red-label swarmer horizontal-speed record from `WVTAB`.

`swarmer_shot_time`
Default: `40,10,-2,-1|25,25,25,25`
Meaning: red-label swarmer shot-timer record from `WVTAB`.

`swarmer_acceleration_mask`
Default: `63,0,0,0|31,31,31,63`
Meaning: red-label swarmer acceleration-mask record from `WVTAB`.

`baiter_time`
Default: `192,24,-12,-4|212,196,164,148`
Meaning: red-label UFO/Baiter spoiler timer used as the first-spawn delay for
the live baiter-pressure path.

`baiter_shot_time`
Default: `10,3,-1,-1|15,13,12,10`
Meaning: red-label baiter shot-timer record from `WVTAB`.

`baiter_seek_probability`
Default: `200,40,-12,-8|240,220,200,200`
Meaning: red-label baiter seek-probability record from `WVTAB`.

## Platform Support

The live loop now depends on a terminal that supports the Kitty graphics
protocol. `cargo run` / `defender` should be launched from a real interactive
terminal session inside `kitty`, `ghostty`, `warp`, or a compatible emulator.
If your terminal supports the protocol but is not recognised by name, set
`DEFENDER_FORCE_KITTY=1` to bypass the terminal-name guard.

Non-interactive tooling paths such as `--rom-report` and
`cargo run --example generate_readme_media` remain usable anywhere a recent
Rust toolchain is available.

The live session now requests terminal keyboard-enhancement reporting so
standalone `Shift` thrust input can be captured in terminals that support the
extended key protocol.
