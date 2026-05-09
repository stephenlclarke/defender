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
Live play now defaults to a windowed `wgpu` backend, while the original Kitty
graphics terminal backend remains available for compatibility checks.

The project is now being rewritten from a clean slate as an exact red-label
arcade implementation. The previous prototype source has been moved to
`oldsrc/` for reference. New code under `src/` is organized around translating
the original red-label assembly and machine behavior into Rust, with a
self-contained runtime so deployment is copying the built binary to a new
machine.

ROMs are treated as reference and verification material only. Runtime assets
and ROM-derived tables live under `assets/` and are embedded with
`include_str!` or `include_bytes!`; a deployed binary does not need a local ROM
or asset directory. The deliberate compatibility features are hidden `xyzzy`
behavior and the BBC Micro `Planetoid` key mapping.

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
- `cargo run -- --renderer wgpu`
- `cargo run -- --renderer kitty`
- `cargo run -- --live-smoke`
- `cargo run -- --rom-report`
- `cargo run -- --rom-report /path/to/roms`
- `cargo run -- --verify-roms /path/to/roms`
- `cargo run -- --fidelity-trace 300`
- `cargo run -- --fidelity-trace-inputs 'coin,start_one;fire,thrust;none'`
- `cargo run -- --fidelity-trace-inputs-file /path/to/inputs.txt`
- `cargo run -- --fidelity-check-trace /path/to/inputs.txt /path/to/expected.tsv`
- `cargo run -- --fidelity-check-trace-dir docs/fidelity/fixtures/local/rust-current`
- `cargo run -- --fidelity-list-scenarios`
- `cargo run -- --fidelity-write-scenario-inputs docs/fidelity/fixtures/local/reference`
- `cargo run -- --fidelity-check-reference-trace-dir docs/fidelity/fixtures/local/reference`
- `make run`
- `make run-wgpu`
- `make run-kitty`
- `make live-wgpu`
- `make live-kitty`
- `make smoke-wgpu`
- `make trace-script-test`
- `make trace-fixtures`
- `make reference-inputs`
- `make reference-traces`
- `make reference-fixtures-check`
- `cargo test`
- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `make ci`
- `make coverage`
- `make coverage-new-code NEW_CODE_COVERAGE_BASE=<git-ref>`
- `make sq-ci`
- `make sq`
- `make readme-media`

By default, live play opens the `wgpu` windowed backend. Use
`cargo run -- --renderer kitty` / `make run-kitty` inside `kitty`, `ghostty`,
`warp`, or another compatible graphics terminal to compare against the Kitty
presentation backend. `cargo run -- --live-smoke` / `make smoke-wgpu` opens the
`wgpu` backend, renders live frames, injects coin/start/control input through
the normal input-profile mapper, verifies attract/credited-start/gameplay
state, and exits cleanly. `--rom-report` remains non-interactive, validates
red-label file sizes and CRC-32 values, and does not require a compatible
graphics terminal. `--verify-roms` performs the same validation and then checks
that the ROM files map into the embedded MAME red-label regions.
`--fidelity-trace` emits deterministic TSV frames from the current Rust core
for local trace fixture work. `--fidelity-trace-inputs` does the same with a
semicolon-separated per-frame cabinet input script. Use
`--fidelity-trace-inputs-file` to read the same script format from a local
fixture file. `--fidelity-check-trace` reads that input script, generates the
current Rust trace, and compares it exactly with an expected TSV fixture.
`make readme-media` rebuilds `docs/start-sequence.gif` from the current
red-label renderer with eight-second Williams and hall-of-fame holds before the
full attract sequence.
`--fidelity-check-trace-dir` checks all local `*.inputs.txt` /
`*.expected.tsv` fixture pairs in a directory, skipping the directory when it is
absent. `--fidelity-list-scenarios` and `--fidelity-write-scenario-inputs`
expose the Phase 1 reference scenario manifest from
`assets/red-label/trace-scenarios.tsv`. `--fidelity-check-reference-trace-dir`
validates that a local reference fixture directory has every required Phase 1
scenario input and expected trace with the checked-in schema header plus the
source-backed `trace-requirements.tsv` evidence markers.
`make trace-script-test` runs the standalone Lua self-test for the local MAME
trace exporter, including the PIA1 port-B sound-command write tap and the local
MAME generator's source CMOS default packing tests.

## Install

Install directly from git with Cargo:

- `cargo install --git https://github.com/stephenlclarke/defender defender`

After installation, run the clean-slate runtime and tooling with:

- `defender`
- `defender --renderer wgpu`
- `defender --renderer kitty`
- `defender --live-smoke`
- `defender --cmos-path ~/.local/state/defender/red-label-cmos.bin`
- `defender --rom-report`
- `defender --rom-report /path/to/roms`
- `defender --verify-roms /path/to/roms`
- `defender --fidelity-trace 300`
- `defender --fidelity-trace-inputs 'coin,start_one;fire,thrust;none'`
- `defender --fidelity-trace-inputs-file /path/to/inputs.txt`
- `defender --fidelity-check-trace /path/to/inputs.txt /path/to/expected.tsv`
- `defender --fidelity-check-trace-dir docs/fidelity/fixtures/local/rust-current`
- `defender --fidelity-list-scenarios`
- `defender --fidelity-write-scenario-inputs docs/fidelity/fixtures/local/reference`
- `defender --fidelity-check-reference-trace-dir docs/fidelity/fixtures/local/reference`

Notes:

- `defender` defaults to the windowed `wgpu` backend. Use
  `defender --renderer kitty` inside `kitty`, `ghostty`, `warp`, or another
  compatible graphics terminal when Kitty compatibility evidence is needed.
- Live CMOS persistence is explicit opt-in. Running without `--cmos-path` uses
  embedded red-label defaults for the session and does not create or update a
  platform default CMOS file. Provide `--cmos-path <file>` when high scores,
  credits, audits, and adjustment CMOS cells should be loaded and saved across
  runs.
- Download Ghostty: <https://ghostty.org/download>
- Download Warp: <https://www.warp.dev/download>
- The `amode1.src` `HALLOF` entry path now runs source `GNCIDE`, `STINIT`,
  credit mirroring, and entry-flag clearing before falling into `HALL1` or
  jumping to `AMODES` during the power-on pass. `HALLOF` / `HOFIN` now writes
  the player label, hall-of-fame instructions, initials, underlines, and
  palette copy into red-label video RAM. Submitted high-score sessions now hand
  off to the source-backed `HALDIS` hall-of-fame table display, including
  headings, underlines, table rows, and the expanded Defender logo. The
  translated player-death game-over handoff now waits through the source
  `PLE2`/`PLE3` 40-tick sleep before `HALLOF`, and non-qualifying sessions wait
  through the source `HALL13` no-entry delay before `HALDIS`; the handoff also
  applies `ATTR`'s map-1 select, and `HALDIS` now performs source `GNCIDE`
  before drawing while preserving coin processes. Source branch-only labels
  `CPR56` and `HALD4` now dispatch through the same translated `HALDIS` and
  `LEDRET` paths, and the live Kitty presenter copies source `PCRAM` into
  hardware palette RAM before scaling each native cabinet frame, then
  double-buffers Kitty image IDs so the terminal is not cleared to black
  between frames. The
  ROM-derived high-score reset copy, packed high-score table
  comparison/insertion helpers, translated death-tail game-over handoff,
  player-one/player-two initials-entry order, today's-greatest qualification
  gate, all-time CMOS insertion, and optional file-backed CMOS persistence
  through `--cmos-path` are modeled. Without `--cmos-path`, live sessions use
  embedded red-label defaults and do not write an implicit CMOS file.

## Controls

Current live controls with the default `planetoid` input profile:

- `5`: insert a left-slot coin
- `6`: insert a center-slot coin
- `7`: insert a right-slot coin
- `ENTER` or `1`: start a one-player game when credit or free play is available
- `A`: move up
- `Z`: move down
- `SHIFT`: thrust forward
- `SPACE`: flip the ship's facing direction
- `ENTER`: fire while playing
- `TAB`: emit a smart-bomb event while playing
- `H`: emit a hyperspace event while playing
- `F2`: service advance / diagnostics target
- `F3`: reset today's high-score table from ROM defaults
- hold `F4` with `F2`: select the auto/audits target
- `F5`: slam/tilt switch
- `A`-`Z`: enter initials while high-score entry is active
- `BACKSPACE`: delete the previous initial while high-score entry is active
- `Q` or `ESC`: quit

Letter-key controls accept either upper- or lower-case input.

After the Williams logo page, the source attract loop should continue
automatically into the later Defender, instruction, and hall-of-fame screens;
no keyboard input is required to advance attract mode. A fresh release run may
start on the Williams/Defender title page, but it is already in attract mode.

To start a normal one-player game from a fresh live session, press `5` to add a
credit, then press `1`. `ENTER` also starts a one-player game in the default
`planetoid` profile, but it maps to fire after play begins, so `1` is the least
ambiguous start key. The red-label start path intentionally blocks no-credit
starts unless the cabinet is configured for free play.

The live key layout is currently modelled on the BBC Micro `Planetoid`
control scheme from Acornsoft's 1982 release of its `Defender` variant.
That layout is an input profile only: terminal keys are translated into
Defender cabinet actions before they reach the arcade core. Use
`--input-profile cabinet` for a MAME-style keyboard profile. In the `cabinet`
profile, `5` inserts a left-slot coin and `1` starts a one-player game; `ENTER`
is not a start key in that profile.

## XYZZY Overlay

During a live session, type `X`, `Y`, `Z`, `Z`, `Y` to toggle hidden `XYZZY`
mode on or off.

`XYZZY` is a deliberate compatibility overlay, not Williams red-label arcade
behavior. With the overlay disabled, trace generation and live core stepping
stay on the red-label path.

Typing `XYZZY` a second time turns the mode off and resets the hidden
invincibility and auto-fire toggles back to their default state.

Implemented overlay behavior while `xyzzy` mode is active:

- `auto_fire`: `F` toggles this hook; when enabled it emits the live fire event
  while playing without changing the raw cabinet input bits.
- `unlimited_smart_bombs`: active `xyzzy` can emit an overlay smart-bomb event
  after red-label inventory reaches zero; disabled-`xyzzy` zero-inventory
  behavior remains the arcade path.
- `invincibility`: `G` toggles the compatibility flag. It is currently recorded
  as overlay state only and is trace-invisible until a source-facing arcade hook
  is implemented.

Future `xyzzy` effects are reserved as explicit overlay hook names:
`shot_cap_override`, `bullet_mine_clear`, `safe_hyperspace`,
`collision_death_override`, and `falling_humanoid_survival`. They are inactive
until implemented with paired tests proving disabled arcade behavior and enabled
compatibility behavior.

## Current Status

- Live mode now presents translated red-label video RAM through the
  native `292x240` cabinet-frame scaler, including the `HALLOF` / `HOFIN`
  initials-entry screen, submitted-score `HALDIS` hall-of-fame table, and the
  gameplay `PLE2` `GAME OVER` text write. The two-player `PLSTR5` player-start
  prompt and `PDTH5R` player-one/player-two game-over handoff also write their
  source labels into video RAM before their source sleeps. The translated
  attract setup now covers source `SCINIT` through object-list reset,
  `SCRCLR`, zeroed `BGL`/`BGLX`, `BGI`, `CRTAB`, final `STATUS=$DB`, and
  `PLAXC=$1030`; `PLE3` applies `ATTR`'s map-1 select before the attract
  vector, `HALDIS` runs source `GNCIDE` before drawing, and source `CREDS`
  refreshes the attract credits label/number while maintaining `OCRED` and
  `ICREDF`; source `AMODES` now prepares the Williams-page state, and source
  `LOGO` expands the `DEFENDER` data, walks `LGOTAB`, draws the Williams-logo
  pixels, switches to the fast pass, and creates `PRES`; `PRES` / `PRES1`
  writes the source `ELECV` two-line `ELECTRONICS INC.` / `PRESENTS` block and
  starts `DEFEND`; `DEF51`, `CPR56`, `HALL12`, and `HALD4` now dispatch
  through their source `DEF50`, `HALDIS`, `HALL1` / `HALL13`, and `LEDRET`
  branch targets; `LEDRET` now drives the source
  instruction-page sequence
  through `AMODE1` / `AMODE2`, `AMODE3` / `AMODE4` / `AMODE5`, `AMODE7` /
  `AMODE8`, the `AMOD12` / `AMOD10` / `AMOD11` enemy table loop,
  `BMODE2` / `BMODE3`, `AMOD13`, `LASRS` / `LAS0`, direct `LASR` / `LASL`
  setup labels, and `TEXTP` / `TEXTP2` with source `TEXTAB` / `TENT` message
  vectors; `HOFST`, `HOFBL`,
  `HOFUD` / `HOFUD1`, `HALL3A` / `HALL4` / `HALL5` / `HALL6`, and `HALD3`
  now perform the source initials-entry stall, blink, up/down, fire-switch
  debounce/advance, initials submission, and hall-of-fame wait ticks. Blank or
  untranslated
  boot/attract frames stay native black instead of falling back to a synthetic
  scaffold frame. Its pacing
  is derived from the core red-label `FRAME_RATE_MILLIHZ` constant, advances any
  due core frames before each terminal draw, and no longer lets terminal draw
  cadence decide core frame count.
- Known live fidelity gap history: a `2026-05-05` live-review pass reported
  that the app did not advance beyond the initial Williams/`DEFENDER` screen,
  and that the title graphic was corrupted into large red/purple blocky bands.
  `DC-16.5` added a core smoke test proving live attract progresses into later
  attract, credit, and start-ready states. `DC-20.1` added MAME-derived visible
  pixel-nibble CRC capture to local reference traces, `DC-24` made stale
  `video_crc32=-` fixtures fail before exact comparison, and `DC-25` fixed the
  MAME-derived cold-boot/title/initial-attract pixel drift enough to promote
  `local_reference_attract_boot_matches_red_label` as a normal passing test.
  `DC-27`/`DC-30` then extended the exact local reference gate through all 12
  Phase 1 scenarios and added live-render/core-input and release-runtime
  evidence, so the old title corruption and Williams-screen stall are no
  longer active blockers. Later `DC-30.11` live evidence repaired the remaining
  attract/action visual reports by proving enemies as main-screen sprites
  rather than scanner/HUD-only state, clearing stale active-object and
  expanded-object footprints before redraw, bounding the reported ship/laser
  trails, and presenting live gameplay fire as a continuous beam instead of a
  small bolt. `DC-30.12` then repaired the Williams startup-screen
  `DEFENDER` wordmark blink, preserving the coalesced wordmark until the normal
  whole-wordmark refresh lands.
- `DC-22` closes the hardware/asset audit by naming which edge
  cases are fixture-backed versus deferred. Fixed main CPU ROM, selected
  banked program ROM, sound CPU ROM, decoder PROM image views, `CROM0`
  `ROMMAP` descriptors, MAME-observed power-on fill boundaries, watchdog
  reset-byte recognition, no-process `SWTAB` entries, translated collision
  dispatch, and archived prototype asset isolation all have focused tests.
  CPU/IRQ cycle ownership, physical advance/lamp timing, full watchdog timeout
  side effects, complete decoder PROM hardware behavior, and remaining
  scheduler/golden-trace equivalence are still explicit fidelity gaps.
- `DC-23` freezes the API and future-refactor validation contract in
  `docs/fidelity/refactor-freeze.md`. The large module-split refactor is
  deferred until the game is fully ROM-complete and playable. `ArcadeMachine`
  now explicitly exposes `new`, `reset`, `step`, `snapshot`, `restore`, and
  full `save_state` / `restore_state` behavior for that later refactor, with a
  focused reset/API unit test proving reset returns to the same observable
  state as `new`.
- `ArcadeMachine::step` returns post-frame main-board and sound-board snapshots
  on `FrameOutput`. Tests now lock the input-port bytes, main RAM/CMOS CRCs,
  palette RAM, hardware map, watchdog count, video-counter sample, sound latch,
  CB1 IRQ state, and latch write count at the frame boundary. A save/restore
  replay test also proves those frame-output surfaces survive dirty
  intermediate mutations without needing additional trace TSV columns.
- The runtime embeds red-label defaults, score values, high-score seeds, CMOS
  layout/default metadata, input port metadata, RAM table metadata, ROM
  metadata, ROM-region/load maps with derived `CROM0` `ROMMAP` descriptors and
  checksum/stage reports, shell image bytes, object-picture descriptors, object
  image bytes, diagnostic LED output behavior, CROM0 diagnostic screen text
  transfer including operator instructions, RAM-test start/failure/no-error
  visible outcomes, the RAM2 pattern fill/verify pass with page-boundary
  operator-poll metadata, pass-boundary loop dispatch, CMOS RAM-test
  write/verify loop and visible outcomes, the CROM0
  color-RAM diagnostic heading/bars/palette loop, CROM0 audio-test
  heading/sound-pulse/skip-table behavior, CROM0 switch-test
  heading/display-table/PIA-scan behavior, CROM0 monitor-test
  heading/crosshatch/RGB-field/color-bar pattern behavior and audit-branch
  handoff, `AUDITG` entry screens and `DISAUD` row video transfer/erasure, the
  post-`PWRUP` `AUDITG` frame-step loop, packed high-score table
  comparison/insertion helpers, deterministic initials-entry submission to both
  live high-score tables,
  optional file-backed CMOS persistence, and advance-gate intent,
  sound-table bytes, the
  complete `SWTAB` switch table, the trace schema, and `WVTAB` wave records from
  `assets/red-label/`.
- The core now initializes source-owned process, super-process, object, player,
  player-start, switch-history, switch-queue, and shell-list RAM bytes from
  embedded red-label metadata and emits object-table/SPTR-head CRCs in fidelity
  traces. It also has source-shaped
  process allocation, super-process allocation, sleep, kill, scheduler, object
  allocation, object initialization, object kill, shell allocation, and
  active/inactive object scanner, shell-list cleanup/movement, dead-shell erase,
  active-object `OPROC` descriptor erase/write banding, active-object `VELO`
  position updates, source-ordered `IRQ` / `IRQB` object-band tails over
  `XXX1` / `XXX2` / `XXX3`, and bomb/fireball shell output primitives over
  those bytes. Snapshot restore now writes the observable score, credit,
  current-player, player runtime, motion, facing, and RNG fields back into
  red-label RAM, and full save-state restore round-trips RAM, CMOS, palette,
  hardware-map, and trace scheduler state.
  The translated
  `SCPROC` process now runs the source `ISCAN`, `OSCAN`, and `SHSCAN`
  maintenance stages, bank-1 `SCNRV` scanner raster writes, and sleeps through
  the exact `SCP1`, `SCP2`, and `SCPROC` resume addresses. Shell output
  dispatch now uses the assembled red-label `OBJCOL` entry-point addresses
  embedded under `assets/red-label/routine-addresses.tsv`. The first
  source-shaped scoring and collision slice translates `SCORE`, including the
  visible `SCRTRN` score-digit refresh and replay-award stock-icon redraw, the
  visible RAM effects of `SNDLD`, `BKIL`, `LFIRE` entry, `LASR0` / `LASL0`
  drawing/fizzle/erase/sleep loops, `LASD` tail, the RAM-visible part of
  `LCOL`, source-ordered `FISS` / `STINIT` / `FBINIT` / `THINIT` boot table
  initialization, `STOUT` star output including its fall-through `SBLNK` color
  blink/hyperspace RAM side effects, exact source `LFIRE` fall-through into the
  first laser loop, and `COLIDE` / `COL0` picture-mask intersection for
  extracted complete and short-form object-picture
  descriptors; `COLCHK` now performs the visible player-collision
  flag/status/death-process writes.
  Collision dispatch reads `OCVECT` and routes only translated vectors. The
  translated vectors currently include `BKIL`, source no-op `NOKILL`, `MSWKIL`,
  `PRBKIL`, `ASTKIL`, captured-astronaut `AKIL1`, `SCZKIL`, `UFOKIL`, normal
  `LKILL`, kidnapping `LKIL1`, and `TIEKIL`. The
  `SBOMB` path now consumes
  the current player's `PSBC`, loads `SBSND`, walks active objects through
  `OCVECT`, toggles `PCRAM` through the source `NAP` resume points, debounces
  `PIA21`, clears `SBFLG`, and suicides the smart-bomb process. The dispatcher
  also preserves the source guard path that jumps to `SBMBX2`/`SUCIDE` without
  clearing an already-active `SBFLG`. One-player start now applies the visible
  `PLSTR5` player RAM initialization. The `HYPER` entry now translates the
  exact `STATUS & $FD` guard, `STATUS=$77` write, source-shaped `SCLR1`
  active-screen clear for rows `Y >= 42`, and `NAP 15,HYP02` process sleep.
  The visible `HYP02` rematerialization slice
  now kills `SPTR` shells, copies `SEED`/`HSEED` into `BGL`/`BGLX`, seeds the
  phony player object, runs the source `APVCT`/`APST` appearance-start RAM
  updates with the extracted `APSND` table, and sleeps to `HYP2`. The visible
  `HYP2` tail now kills the
  phony object, resets `STATUS` through `STCHK`, suicides on the safe path, and
  exposes the `PLEND` branch when `LSEED > 192`. The RAM-visible `EXST` /
  `EXPU` expansion path now initializes explosion slots, advances `RAMALS`
  appearance/explosion slots, restores object pictures when appearances
  finish/offscreen, clears old erase-list blocks, and runs the translated
  `EWRITE` expanded-object writer from embedded picture assets. `KILOFF` now
  unlinks objects and clears the current picture footprint through `OFSHIT`'s
  map-2 erase path. The player-death entry and glow loop now translate
  `PLEND` / `PDTHL` / `PDTH2` / `PDTH4`: `PLSAV`, `PDSND`, `MONO` into
  `MONOTB`, `PXCTB` glow colors, pseudo-color RAM writes, and the sleep into
  `PDTH5` are source-shaped. The `PDTH5` entry now clears `PCRAM`, runs exact
  `GNCIDE`, enters the bank-7 `PXVCT`/`PX1A` player-explosion loop using the
  extracted `PXCOL` color table, and resumes into the ROM-confirmed `PDTH5R`
  continuation. The non-wave-end respawn, player-switch, and game-over branch
  decisions are translated through `WVCHK`, `PLE02`, `PLE3`, `PLSTRT`, and
  `ATTR` addresses, including the source two-player `PLAYER ONE/TWO` plus
  `GAME OVER` video RAM writes before switching players. The wave-clear
  `BONUS` long-sleep path now writes the
  source `MESS` / `WNBV` bonus text and numbers from embedded `mess0.src`
  assets, scores each surviving astronaut, sleeps through `BC1`, refreshes the
  next wave through `GETWV`, and returns through `BC3` to the ROM-confirmed
  post-bonus `SCLR1` address. The `PLSTRT` respawn path now
  translates the RAM-visible entry
  handoff, source `PINIT` process-list reset, `PLSTR3` / `PLSTR5` current-player
  runtime initialization, bank-7 `ALINIT` / `BGALT` altitude-table generation
  from `TDATA`, bank-7 `BGINIT` `TERTF0` / `TERTF1` terrain-table generation
  from `TDATA`, bank-7 `BGOUT` terrain-table rolling and `STBL` screen-word
  output with an explicit 6809 stack pointer, support-process creation,
  `PLS01` status/sleep,
  `PLS1` `PLRES` astronaut-process/target-list/enemy-runtime restore,
  the scheduled `ASTRO` target-list walker against `ALTTBL`, the `ASTKIL`
  `ASTCLR` / `KILOFF` / `XSVCT` astronaut explosion path, and the source
  `TERBLO` terrain-blow process when the final astronaut is removed, including
  `BGERAS`, scanner-terrain `STETAB` erase, `TEREX` explosion passes, `TBL3` /
  `TBL4` sleeps, `OVCNT`, `COLTAB`, `AHSND`, and final `TBSND` / `SUCIDE`,
  schizoid-reserve `SCZST` restore from copied `SCZRES`,
  the `SCZ0` movement/shot-timer process slice through shared `SHOOT`,
  UFO `UFOST` dispatcher process/object start plus `UFOLP` shot timer,
  `UFOP1`-`UFOP3` image cycle, `UFONV` velocity update, and `USHSND` shot sound,
  the `SCZKIL`, `UFOKIL`, normal `LKILL`, `LANDST` lander start/`GTARG`
  target selection, `LANDS0` orbit/`LSHOT`, `LANDG` capture, `LANDF` flee,
  `LNDFXA` pull-in, `SCZ00` mutant conversion, and kidnapping `LKIL1` /
  `AKIL1` / `AFALL` / `AFALL2` passenger-release, catch, and
  falling-astronaut paths, including `P250` / `P500` rescue score popups and
  `P503` cleanup,
  probe-reserve `PRBST` restore from copied `PRBRES`, the `PRBKIL` `KILO` /
  `RMAX` / `MMSW` mini-swarmer spawn path, the `MSWKIL` `KILOFF` / `KILLOP` /
  `XSVCT` score/sound path, the `MSWM` / `MSWLP` mini-swarmer
  acceleration/damping/turnback process loop, `SWBMB` firing through `GETSHL`,
  and the `SWSSND` swarmer-shot sound load, tie-fighter reserve `TIEST`
  restore from copied `TIERES`, the `TIE` process image/vertical/cruise slice,
  `BOMBST` bomb-shell allocation and lifetime path, the `TIEKIL` `KILO`
  squad-slot and super-process cleanup path, `STCHK`/`PDFLG` tail, and
  machine-level
  `INIT20` `CRINIT` / `FISS` / `STINIT` / `OINIT` / `FBINIT` / `THINIT`
  refresh. `BGERAS` now erases terrain screen words through the source `STBL`
  screen-address table. `COLR` / `COLRLP`, `FLPUP` / `FLP2`, `CBOMB` /
  `CBMB1`, and `TIECOL` / `TIECL` now run as translated support-process bodies
  using embedded `COLTAB` / `TCTAB` assets. Translated `PLSTRT` runtime
  dispatch now syncs the live snapshot's current player, wave, lives, smart
  bombs, and player motion from red-label RAM. Public snapshots now read those
  player/session fields, scores, high score, credits, and RNG from the
  source-owned red-label RAM/CMOS tables whenever the tables are initialized,
  leaving cached fields only as cold-boot/compatibility control state. `PLRES`
  mini-swarmer reserve
  restore now runs the source `RSW0` phony-object placement, source `PLS1`
  entry B=`0x07` for targetless reserve restore, target-list B-register X-low
  byte, `MMSW` six-at-a-time batching, `SWMRES` decrement, and `OFREE`
  return/reuse path. Live playing frames now schedule translated
  `BGOUT` with the source-derived IRQ stack pointer, so the cabinet terrain
  screen table and video RAM advance through the red-label terrain output path.
  The live `PLS1` tail now hands the active start process to `GEXEC`, matching
  the source jump so credited starts release the gameplay frame path and render
  both lower terrain/ground and visible enemy object/appearance state. The
  translated lander target picker now skips source `GTARG` overrun words from
  `FISTAB` that are not object-table pointers, matching the hardware behavior
  of treating those reads as non-astronaut targets instead of crashing the live
  loop; reverse pulses are covered by the same live terrain/enemy regression.
  DC-08 proof fixtures now cover reverse rendering, laser lifecycle,
  smart-bomb world effects, hyperspace branches, human rescue, and
  death/respawn/wave/game-over branch decisions.
  The IRQ `PLAYER` motion slice now applies source-shaped 24-bit horizontal
  damping, thrust acceleration, X/scroll correction, absolute-X calculation,
  and altitude up/down velocity from `PIA21` / `PIA31`. The `PRDISP` player
  picture slice now stores the source scanline bound in `TEMP48`, gates on
  `STATUS` / `PLAYC`, erases the old 8x6 `OFF86` footprint, copies `NPLAD` /
  `NPLAXC`, and draws `PLAPIC` / `PLBPIC` through embedded `ON86` image bytes;
  the same slice now performs the adjacent `THOUT` / `THOFF` thrust-flame byte
  writes from the extracted `THTAB` data. `THPROC` now advances the
  fireball/thrust table pointers from source-shaped RAM. Live player frames now
  run translated `PLAYER` plus translated `PRDISP` over the current player
  display band, so source `REV` updates to `NPLAD` are visible in rendered and
  facing `PLADIR` state. `OPROC` now erases and
  redraws active-object descriptor pictures in caller scanline bands with the
  ROM BGL-relative X and alternate-flavor checks, and `VELO` now advances
  active-object `OX16` / `OY16` by source velocities with Y wrap through
  `YMIN` / `YMAX`. The normal and inverted IRQ object-band tails now load the
  source scanline pairs from `XXX1` or `XXX2` and run already translated
  `PRDISP`, `OPROC`, `SHELL`, and `VELO` slices in the ROM order. The
  scanline object-phase gate now applies the source `VERTCT` thresholds, `IFLG`
  latch, `TIMER` increment, normal/flipped watchdog data byte, palette-copy
  thresholds, the source `PSHU` color-mapping copy from `PCRAM` into hardware
  color RAM, and `XXX2` calculations before entering those tails, and it runs
  translated `PLAYER` / `STOUT` pre-tail work on the source branches that call
  them. Native red-label frame rendering now reads the modeled hardware color
  RAM after that copy. When the caller supplies the source IRQ context or an
  explicit 6809 stack pointer, the terrain branch also runs translated `BGOUT`;
  otherwise it records that `BGOUT` is due. The
  pre-tail `SNDSEQ` sound-table sequencer now advances `SNDX` / `SNDPRI` /
  `SNDTMR` / `SNDREP`, with `SNDLD` preserving the source equal-priority
  interrupt and pre-priority `THFLG` clear. All embedded source sound tables can
  now be expanded into exact repeated command plans, an embedded
  `sound-table-timelines.tsv` fixture with table labels, addresses, `SNDSEQ`
  tick anchors, terminator pointers, and sequence-end ticks, and a
  source-derived `sound-table-command-sequences.tsv` fixture that expands each
  table command through `SNDOUT` into an idle write and complemented command
  write. The source thrust gate has a matching
  `sound-thrust-command-sequences.tsv` fixture for the `SNDS01` start and
  `SNDS00` stop branches, and the direct `PDTH5`, `PLE2`, and `LNDFX0`
  `SNDOUT` callsites are covered by `sound-direct-command-sequences.tsv`.
  Fixture checks compare the embedded rows against the generated data and count
  timeline command/sequence-end rows plus command-sequence idle/command writes.
  `SNDSEQ` models the source `SNDOUT` idle write before the complemented
  sound-number command and handles the thrust sound gate. Full frame output and
  fidelity traces now include the resulting asserted raw command bytes and a
  native visible-video CRC computed from red-label video RAM. The
  `DC-21.2` local MAME `start_game` recheck matched the trace-required sound
  command/event rows at frames 731 (`0xC0`), 912 (`0xE6`, `credit_added`), and
  1027 (`0xF5`, `game_started`). Phase 10 acceptance now uses exact
  source-visible command traces and deterministic DAC byte signatures as the
  documented audio tolerance rather than checked-in external waveform files or
  hardware-cycle DAC reconstruction. The
  source `CSCAN` branch now keeps
  the `PIA01` / `PIA02` coin-door history, masks IN2 through `ANDB #$3F`,
  double-checks the sample, and queues the first surviving `SWTAB1`
  coin/admin switch process. The queued coin process path now translates
  `LCOIN` / `RCOIN` / `CCOIN` debounce/sleep handling, `CN1` coin sound
  loading, and the fixed-bank BCD coinage/audit/credit updates. The queued
  admin switch path now translates `HSRES` today's-high-score reset and `ADVSW`
  manual diagnostics/audit target selection, with the diagnostic/audit mode
  handoff still explicit. The IRQ scheduler now records the source `MAPC`
  clear/select/restore write sequence and leaves hardware map selection
  restored from `MAPCR`. The source IRQ `BGOUT` context now derives the
  `SSTACK` value from `HSTK=$BFFF`, the 12-byte 6809 IRQ frame, and the `JSR`
  return address instead of accepting a magic test pointer. Full frame-level
  IRQ scheduling still needs integration.
  The source `EXEC0` / `EXEC1` pre-`DISP` slice now clears `TIMER`, updates
  `OVCNT` / `STRCNT`, demotes the first overload-eligible active object through
  the source `OFSHIT` / `IPTR` path, selects map 2, runs translated `COLCHK`,
  calls the translated `XUVCT` / `EXPU` expanded-object update, advances
  `RAND`, and drains queued `SWPROC` entries through `SWP`. A source-shaped
  executive iteration wrapper now resets `CRPROC` to `#ACTIVE`, runs that
  pre-dispatch slice, then keeps walking the translated `DISP` scheduler in the
  same pass after process `SLEEP` and `SUCIDE` tails resume through `DISP2`,
  without changing the live trace cadence.
  Live playing frames now enter the source `VERTCT` / `IFLG` scheduler for
  upright `IRQ` and `IRQHK`-selected flipped `IRQB` video passes, carrying the
  source map writes, timer/watchdog side effects, palette copy, translated
  `PLAYER`, `STOUT`, upper/lower `OPROC` / `PRDISP` bands, `SHELL`, IRQ
  `BGOUT`, and `VELO` instead of the old player-plus-terrain shortcut. The
  initialized core now applies translated `P1SW`, and the `PLSTR3` cocktail
  path now runs translated `P1SW` / `P2SW` IRQ-hook selection before live
  frames read `IRQHK`. Sound-IRQ ownership and remaining non-gameplay
  presentation paths remain open.
  The `GEXEC` / `GEX0` game-executive slice now initializes `PD`, `UFOTMR`,
  and `WAVTMR`, runs the source `WVCHK` gate, accelerates and decrements
  `UFOTMR`, dispatches `UFOST` and increments `UFOCNT` when the baiter timer
  expires, launches `LANDST` squads from `WAVTMR` / `LNDRES` / `WAVSIZ`, then
  restores `STRCNT` after star overflow, advances `GTIME` through the source
  audit-meter wrap, decrements the process `PD` counter, applies source
  `WDELT` intra-wall deltas to `ELIST` every 40 passes, and sleeps back to
  `GEX0`. The wave-clear path now sets `STATUS=$77`, runs `GNCIDE` / `PLSAV`,
  enters the shared `BONUS` body with the assembled `GEXBON` return site,
  increments `PLAS`, and runs the translated `PLSTRT` / `PLSTR3` handoff. The
  `GETWV` wave parameter path now increments `PWAV`, refreshes `PENEMY` from
  source-order `WVTAB`, applies CMOS difficulty/ceiling inter-wall `WDELT`
  updates, and restores `PTARG` on the `GA1+6` restore-wave cadence. Focused
  fixtures now lock the wave launch, terrain/scanner, `TERBLO`, `GETWV` /
  `WDELT`, survivor-bonus, and wave-to-wave RAM/list mutations for later
  refactors; end-to-end MAME trace proof remains a later fidelity phase. The
  start-flow
  foundation now extracts `CREDIT`,
  `CUNITS`, and `BUNITS`; implements exact `FPLAY` free-play credit seeding;
  and translates the RAM-visible `START` power-page gate, first-game player
  table reset/copy, `PLSTRT` process creation, source `SCRCLR` video-RAM clear,
  and `START2` BCD credit/`PLRCNT` tail plus `WCMOSA CREDST` packed CMOS credit
  backup. The source `TDISP` top-of-screen redraw now dispatches through the
  translated scheduler path and runs through extracted score digit, mini-ship,
  and smart-bomb image assets, including `BLKCLR`, `BORDER`, `LDISP`, `SBDISP`,
  and `SCRTR0` visible RAM writes. The source `SCORES` player-score redraw
  used by `HALDIS` and `LEDRET` now routes through the same helper. The source
  `SCORE` tail now reuses those same `SCRTRN`, `LDISP`, and `SBDISP` paths
  when score and replay events update the live HUD. `ST1` and `ST2` now
  execute through the translated process dispatcher in the source order,
  including status/credit gates, start sounds, one/two `START` calls, and the
  final `DIE` process cleanup. The generic `SUCIDE` / `HYPX` process tail now
  unlinks the current process and rewinds `CRPROC` through the translated
  dispatcher. Live frames now present translated video RAM through the native
  cabinet scaler without a synthetic fallback. The source `SCINIT` attract
  setup, `CREDS` credits display, `AMODES` Williams-page setup, `LOGO` table
  drawing through the first-pass `PRES` handoff, `PRES` / `PRES1` `ELECV`
  redraw, `DEFEND` / `DEFENS` wordmark appearances, `DEF33` / `DEF50` whole
  wordmark refresh, `DEF44` / `COPYRT` copyright wait, `WILLIR` / `WILR1`
  fast-logo restore, `LEDRET` instruction-page setup/rescue/enemy-table/text
  sequence, `HOFST` / `HOFBL` / `HOFUD` / `HOFUD1`,
  `ATTR` / `HALLOF` entry setup, `HALL1` qualification/screen setup,
  `HALL3A` / `HALL4` / `HALL5` / `HALL6` initials-entry support ticks,
  `HALD3` hall-of-fame wait, `PLE3`/`ATTR` map handoff, `HALDIS`
  entry `GNCIDE`, and live `PCRAM` palette presentation are translated. Idle
  live attract mode now creates the source `ATTR` process and follows the
  immediate source `ATTR` -> `AMODES` -> `LOGO` jump chain before the logo
  sleeps to `LOGO0`; a core smoke test now proves it reaches later attract
  processes, accepts credit, and starts play. Remaining boot/attract handoff
  video and full frame/cycle ownership remain gaps.
  Live fire, smart bomb, hyperspace, reverse, and credited/free-play one- and
  two-player start buttons now enter through an asset-backed red-label `SWTAB`,
  `SSCAN` switch history, `SWPROC` queue, `SWP` status gate, and translated
  `LFIRE` / `SBOMB` / `HYPER` / `REV` / `ST1` / `ST2` scheduler paths. The
  scanner records all eight IN0 switch bits and queues every translated source
  switch process. Live coin/admin input now enters through the translated
  `CSCAN` / `SWTAB1` / `SWP` scheduler path, ignores the source auto/manual
  selector for queue priority while preserving it for `ADVSW`, ticks the source
  slam/coin debounce counters from live tilt/coin input, sleeps through
  `LCOIN` / `RCOIN` / `CCOIN`, and awards credit from `CN1` with `CNSND`,
  slot audits, paid-credit audit,
  `CUNITS`/`BUNITS`, and CMOS-backed `CREDIT` / `CREDST` updates. Live
  high-score reset now runs through `HSRES`, and live service advance reports
  the translated diagnostics/audits target. No-credit one-player starts now
  enter translated `ST1` and die at the source credit gate instead of using the
  old compatibility shortcut. After a translated start, live player controls
  stay gated while the active `PLSTRT` / `PLSTR3` / `PLS01` / `PLS1`
  player-start handoff advances, then the `PLS1` tail schedules `GEXEC` so
  live terrain/ground output and enemy object/appearance rendering resume from
  the native cabinet frame path. Live attract rendering is regression-covered
  through visible post-title frame changes before start.
  `REV` sets
  `REVFLG`, negates `PLADIR` into `NPLAD`, debounces `PIA21`, clears the flag,
  and returns the process to the free list.
- ROM files are optional verification inputs only. The deployed runtime remains
  self-contained.
- MAME/source reference-trace tooling is in place for local Phase 1 fixtures,
  but checked-in golden traces are intentionally absent because they are local
  emulator outputs. The Phase 1 MAME generator now seeds `romc8.src` `DEFALT`
  CMOS defaults by default, uses a MAME-observed 900-frame ready wait followed
  by held coin and delayed held one-player start, and validates the credited
  start through observed `0xE6` / `0xF5` sound commands plus
  `credit_added` / `game_started` trace events. All 12 Phase 1 local MAME
  reference scenarios now pass as normal `local_reference_*_matches_red_label`
  tests when the local fixture directory is present; `cargo test --all-targets
  -- --ignored` should report zero ignored tests.
- Phase 11 final acceptance has passed for the accepted red-label target: the
  full fidelity gate, all promoted local reference fixtures, README media
  generation, release build, ROM report/verification, short fidelity trace, and
  forced Kitty-compatible live smoke all pass. The next planned work is the
  post-acceptance `wgpu` presentation backend; the broad module-split refactor
  stays deferred until after that renderer path is accepted.

## SonarQube

- `make sq-ci` generates the Cobertura coverage report used by the SonarCloud
  workflow in CI.
- `make sq` runs the same coverage step locally and then invokes
  `sonar-scanner`.
- `make coverage` keeps the whole-project 80% line-coverage floor, writes
  `target/coverage/lcov.info`, and requires 100% coverage for added executable
  Rust lines. `NEW_CODE_COVERAGE_BASE` defaults to `HEAD` for local dirty
  worktrees; set it to another review base such as `origin/main` when needed.
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

These references are used for reverse engineering, rules verification, attract
screen reconstruction, extraction of historical arcade data, and future
translation work while keeping the final runtime self-contained. A reference
listed here does not imply the clean-slate runtime has implemented that behavior
yet; current implementation status is tracked in `SPEC.md` and
`docs/fidelity/gaps.md`.

- <https://github.com/mwenge/defender>: Motorola 6809 assembly language for the
  'Red Label' version of the game. Used for reference implementation and ROM
  layout comparison point, especially for the red-label `defend.*` program ROM
  names, the `mess0.src` `CHRTBL` / `CHARACTERS` font tables that supersede the
  archived prototype `assets/arcade/font-sheet.png`, and the `amode1.src`
  `LGOTAB`, `DEFDAT`, `CPRTAB`, `TEXTAB`, `TENT`, `ENMYTB`, `PICTS`, `XS`, and
  `BLIPS` tables used to reconstruct the attract-logo page, `DEFENDER`
  wordmark, instruction-page rescue/legend sequence, and source
  `TEXTP` / `TEXTP2` text cadence, plus the
  source-ordered `blk71.src` `WVTAB` records used to reconstruct the compiled
  red-label wave/fire tables, `GETWV` base values, and `WDELT` updates now
  embedded from `assets/red-label/wave-table.tsv`, and the `blk71.src`
  `TDATA` bitstream used by `ALINIT` / `BGALT` to generate the bank-7
  altitude table and by `BGINIT` to generate the mirrored terrain flavor
  tables, plus the `amode1.src` `MTERR` mini-terrain bytes used by `SCNRV`, are
  now embedded from `assets/red-label/terrain-data.tsv`. The
  `romc0.src` `PWRUP`, `romf8.src` `ROMMAP`/`ROM0`/`ROM9`, and `romc8.src`
  `DEFALT`, `CMOSMV`, `CMINIT`, `RHSTD`, `RHSTDS`, `AUDITG`, `LEDS`, `FLASHL`,
  `ADVSW`, and `NEXTST` routines provide the ROM-derived CMOS defaults,
  power-up CMOS branch/source dispatch target including `AUDITG` entry
  dispatch, source-shaped `CROM0` ROM descriptor bytes and checksum failure
  reports plus the ROM-stage manual/auto
  outcome, diagnostic LED output bytes, CROM0 diagnostic text/palette intent and
  bitmap headline/bad-ROM-row/operator-instruction transfer plus advance gates,
  RAM-test start/failure/no-error visible setup, RAM2 pattern fill/verify pass
  with page-boundary operator-poll metadata, pass-boundary loop dispatch, CMOS
  RAM-test write/verify loop and visible
  outcomes, CROM0 color-RAM diagnostic heading/bars/palette-loop behavior,
  CROM0 audio-test heading/sound-pulse/skip-table behavior, CROM0 switch-test
  heading/display-table/PIA-scan behavior, CROM0 monitor-test
  heading/crosshatch/RGB-field/color-bar pattern behavior and audit-branch
  handoff, high-score reset copy, operator audit/adjustment table, `AUDITG`
  game-adjust entry screens, and `DISAUD` display-line formatting/video
  transfer/erasure, row navigation, post-display debounce cadence,
  deterministic audit cycle with previous-row erasure, post-`PWRUP` outer
  frame-step dispatch, packed high-score table comparison/insertion,
  deterministic initials-entry submission to both live high-score tables,
  optional file-backed CMOS persistence, and CMOS mutation rules now embedded
  under `assets/red-label/`.
  The
  `defb6.src` `CRTAB` bytes now feed
  the `CRINIT` pseudo-color RAM reset embedded in
  `assets/red-label/color-ram.tsv`. The `defa7.src` `SCORE`, `SNDLD`, `SHELL`,
  `BMBOUT`, `FBOUT`, `BKIL`, `LFIRE`, `LCOL`, `LASR` / `LASR0`, `LASL` /
  `LASL0`, `LASD`, `COLIDE`, `COL0`, `COLCHK`, `REV`, `PLEND` / `PDTHL` /
  `PDTH2` / `PDTH4` / `PDTH5`, `PXVCT` / `PX1A`, `PDTH5R`, `PLE02`, `PLE3`,
  `PLSTRT`, `PLST1A`, `PLSTR3`, `PLS01`, `PLS1`, `STCHK`, `ATTR`, `PLAYER`,
  `THPROC`, `SCPROC` / `SCP1` / `SCP2`, `OSCAN`, `ISCAN`, `SHSCAN`, `SCNRV`,
  `BGOUT`, `ALINIT`, `BGINIT`, `BGERAS`, `BGI`, `UFOST` / `UFOLP`, `SBOMB`,
  and `CN1` assembled entry points and smart-bomb resume labels provide the
  first routine-address asset. The
  `defa7.src` `PLSTR5`, `SSCAN`, `CSCAN`, `SWP`,
  `REV`, and `SBOMB` paths, with the complete `defb6.src` `SWTAB` and
  `SWTAB1`, provide the first live player-fire, smart-bomb, reverse, and
  coin-door switch process wiring. The
  `defa7.src` `BMBOUT` / `FBOUT` shell output callbacks, `SCORE` BCD/replay
  path, `SNDLD` sound-table loader, `BKIL` bomb-collision routine, `LFIRE`
  cap/process-data entry path, RAM-visible `LCOL` collision setup, source
  `FISS` / `STINIT` / `FBINIT` / `THINIT` table initialization, bank-7
  `ALINIT` altitude-table generation from `TDATA`, `BGINIT` terrain-table
  generation from `TDATA`, `BGERAS` terrain erase via `STBL`, source `LFIRE`
  fall-through into `LASR` / `LASL`, `LASD` process
  tail, `COLIDE` / `COL0`
  picture-intersection routine, visible `COLCHK` player-collision side effects,
  `REV` reverse debounce path,
  `SBOMB` smart-bomb entry/tail path, `COLR` laser-color cycle, `FLPUP`
  score-flash process, `CBOMB` bomb-color/image cycle, `TIECOL` tie-fighter
  color cycle, `PLEND` / `PDEATH` / `MONO` /
  `PLSAV` player-death entry/glow-loop path, and the `PDTH5` / `PXVCT`
  player-explosion and non-wave-end branch path, live reverse
  movement/rendering through translated `PLAYER` / `PRDISP`, plus the
  `defb6.src`
  object-picture descriptors and image bytes, source-shaped `CWRIT` / `COFF`
  generic picture writes/erases, and descriptor ON/OFF picture dispatch helpers
  plus `OPROC` active-object display banding and `VELO` active-object movement
  provide the first source-shaped shell output, laser, scoring, switch,
  collision, drawing, and death primitives. The
  remaining `defa7.src` and `defb6.src` gameplay routines remain the next
  translation targets; old prototype extractions are archived under `oldsrc/`.
- <https://github.com/mamedev/mame/blob/master/src/mame/midway/williams.cpp>:
  MAME Williams driver source used for the red-label `ROM_REGION` and
  `ROM_LOAD` region mapping now embedded under `assets/red-label/`, the
  main-board memory/PIA callback map, and the active-high Defender IN0/IN1/IN2
  cabinet input bit layout.
- <https://github.com/mamedev/mame/blob/master/src/mame/midway/williams_v.cpp>:
  MAME Williams video source used for the 4-bit bitmap screen-memory layout,
  Defender visible-area crop, native pixel-nibble and palette-RAM-index frame
  extraction, and palette resistor conversion. MAME's
  `src/emu/video/resnet.cpp` provides the resistor weight algorithm used for
  the RGBA palette lookup.
- <https://github.com/mamedev/mame/blob/master/src/devices/machine/6821pia.cpp>:
  MAME Motorola 6821 PIA device source used for the main-board PIA data/control
  register behavior, data-direction filtering, CA/CB IRQ flags, CA2/CB2 strobe
  behavior, and output-callback timing.
- <https://github.com/historicalsource/williams-soundroms>: original Williams
  sound-ROM source reference for the `VSNDRM1.SRC` translation. The setup, IRQ
  command decoder, normal `IRQ1` `GWAVE` / `VARI` command runner, `SP1` /
  `BG1` / `BG2INC` / `LITE` / `BON2` / `BGEND` / `APPEAR` / `TURBO` /
  `THRUST` / `CANNON` / `RADIO` / `HYPER` / `SCREAM` / `ORGANT` /
  `ORGANN` special command runners, `GWLD`
  loader for `SVTAB` / `GWVTAB` / `GFRTAB`, and `VARILD` loader for `VVECT`
  plus the `SP1`, `BON2`, and `BG2` pre-loop setup
  paths, the `GWAVE` / `GPLAY` per-period DAC byte stream, `VARI` / `VSWEEP`
  per-sweep DAC byte stream, `LITE` / `APPEAR` shared `LITEN` random-complement
  byte stream, `TURBO` / `NOISE` noise-decay byte stream, `HYPER` phase-edge
  DAC byte stream, `BG1` / `THRUST` first-window `FNOISE` byte streams,
  `CANNON` / `FNOISE` filtered-noise decay byte stream, `RADIO` / `RADSND`
  timer-table byte stream, `SCREAM` echo-cascade byte stream, `ORGANN` /
  `ORGNN1` first-duration `ORGAN` note byte stream, `ORGNT1` / `ORGTAB`
  organ tune byte streams, the source `IRQ` PIA command-read/CB1-clear prelude,
  the source `IRQ3` background handoff, command return/readiness
  classification, the source-shaped `IRQ1` command-to-`IRQ3` background step,
  the top-level source IRQ organ gate, the source IRQ organ-continuation gate,
  the source IRQ prelude-to-flow cycle, main-board `SNDOUT` idle-plus-command
  output shape, and shared `GEND` / `GEND40` / `GEND50` / `GEND60` / `GEND61`
  echo and frequency-window updates, plus the source NMI diagnostic
  checksum-to-VARI branch, are now modeled;
  cycle-accurate `GWAVE` / `VARI` / `LITEN` / `NOISE` / `FNOISE` / `RADIO` /
  `HYPER` / `SCREAM` / `ORGAN` scheduling and sound CPU IRQ cadence remain
  future work.
  Legacy cue assets are kept only as archived prototype references under
  `assets/sounds/`.
- <https://seanriddle.com/ripper.html>: Williams graphics-ripper reference used
  to confirm Defender's screen-format sprite layout for the future red-label
  `defb6.src` picture-table translation.
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
- <https://www.dougmahugh.com/defender-chapter03/>: lander-fire reference for
  the translated shared `SHOOT` path, random broad-arc jitter through `SEED` /
  `LSEED`, and the rule that enemies only fire while on the main screen.
- <https://www.dougmahugh.com/defender-chapter04/>: mutant-behaviour reference
  for future translation of the more aggressive mutant fire path and its shared
  cabinet `SHOOT` routine.
- <https://www.dougmahugh.com/defender-chapter05/>: swarmer-behavior reference
  for the translated `MSWM` vertical acceleration/damping/turnback path and
  `SWBMB` swarmer-shot focal point, plus future checks of pod bursts and
  surrounding wave behavior.
- <https://www.dougmahugh.com/defender-chapter06/>: bomber-behavior reference
  for future translation of wave-two bomber introduction, the `TIEXV`-backed
  horizontal cruise path, and persistent mine trails.
- <https://www.dougmahugh.com/defender-chapter07/>: baiter-behavior reference
  for future translation of wave-delay baiter pressure, the `UFOST`
  visible-band spawn shape, and relative pursuit behavior.
- <https://www.arcade-history.com/?id=614&n=defender&page=detail>: scoring and
  gameplay reference for future checks of humanoid rescue values, safe-fall
  saves, and wave bonus behavior.
- <https://strategywiki.org/wiki/Defender/Gameplay>: gameplay reference for
  future checks of enemy and humanoid behavior against the original arcade
  rules, including opening five-lander attack waves, later five-ship
  reinforcement groups, later pod scheduling, and the destroyed-planet
  mutant-wave cycle until the next fifth-round restore.
- <https://strategywiki.org/wiki/Defender/Walkthrough>: rescue-strategy and
  scoring reference for future humanoid rescue implementation.
- <https://en.wikipedia.org/wiki/Defender_%281981_video_game%29>: general rules
  reference for future checks of the default `10,000`-point extra ship and
  smart bomb award behavior.
- <https://www.digitpress.com/reviews/defender.htm>: secondary gameplay
  reference for future checks of Defender's reverse-with-inertia handling, where
  the ship keeps its current momentum until thrust changes it.
- <https://bbcmicro.co.uk/game.php?id=11>: BBC Micro `Planetoid` archive entry
  used to anchor the current keyboard layout to the Acornsoft 1982 home-port
  control scheme.
- <https://www.youtube.com/watch?v=6w2cKBWx2Uc>: original arcade attract-mode
  capture used to verify the full attract-sequence order, Williams logo-page
  composition, and the scoring legend reveal shown around `0:26`.

## Red-Label Assets

Clean-slate runtime data lives under `assets/red-label/` before it is embedded
into Rust. The active embedded assets currently include:

- `defaults.tsv`: stock, smart bomb, starting wave, and human-count defaults.
- `high-scores.tsv`: red-label high-score seed initials and scores.
- `cmos-defaults.tsv`: `romc8.src` `DEFALT` bytes used by `CMOSMV`, `CMINIT`,
  `RHSTD`, and `RHSTDS` for high-score reset, replay, coinage, and
  game-adjust defaults.
- `cmos-layout.tsv`: source-owned `phr6.src` CMOS cell layout for audits,
  high-score slots, credit backup, coinage/operator settings, and game-adjust
  cells.
- `color-ram.tsv`: source-owned `defb6.src` `CRTAB` bytes copied by `CRINIT`
  into `PCRAM`.
- `input-ports.tsv`: MAME IN0, IN1, and IN2 cabinet input bit layout.
- `linked-lists.tsv`: red-label RAM linked-list heads for active/free
  processes, object lists, and the shell list.
- `memory-map.tsv`: MAME main-board and sound-board address ranges used by the
  Rust address classifiers.
- `message-glyphs.tsv`: source `mess0.src` text glyph bytes consumed by the
  translated `BONUS` `MESS` calls and CROM0
  diagnostic/RAM-test/CMOS/color/audio/switch-test/monitor-test/audit text
  transfers.
- `messages.tsv`: source `mess0.src` message vectors, words, and text-control
  tokens consumed by the translated `BONUS` screen text and CROM0 diagnostic,
  RAM-test, CMOS-test, color-test, audio-test, switch-test, monitor-test, and
  audit/game-adjust messages.
- `object-images.tsv`: red-label object image bytes currently used by
  `COLIDE` / `COL0` picture-mask intersection, `PRDISP` / `ON86` player
  picture writes, `CWRIT` / `COFF` generic object-picture writes/erases, and
  descriptor ON/OFF output helpers including `OPROC` active-object band
  writes, plus short-form laser, mini-player, and smart-bomb image data.
- `object-pictures.tsv`: red-label complete and short-form object-picture
  metadata currently used by the generic picture-collision path, translated
  generic picture output/erase helpers, translated `OPROC` object display
  banding, and the translated player-picture output slice.
- `ram-layout.tsv`: source-owned `phr6.src` RAM table bases, strides, counts,
  and field offsets for base-page state, text cursor workspace, `TIMER`,
  `IRQHK`, `IFLG`, RNG seed bytes, `ASTCNT`, runtime pointers, `ITEMP` /
  `ITEMP2`, `XTEMP` / `XTEMP2`, `TEMP48`, and `XXX1`-`XXX3` scratch bytes,
  star-map bytes,
  laser fizzle bytes, thrust/fireball table bytes, player data, object cells,
  process cells, super-process cells, bank-7 terrain runtime bytes, `ALTTBL`,
  `TERTF0` / `TERTF1`, `STBL`, and the source-order `ELIST` enemy
  reserve/runtime parameter fields through `UFOSK` plus `ECNTS` active counts
  through `UFOCNT`.
- `roms.tsv`: red-label ROM filenames, byte sizes, and CRC-32 values.
- `rom-regions.tsv`: MAME ROM region sizes.
- `rom-map.tsv`: MAME `ROM_LOAD` mapping for fixed, banked, sound, and PROM
  views; the `defend.*` rows also derive the source-shaped `CROM0` `ROMMAP`
  descriptor bytes and checksum/stage reports.
- `routine-addresses.tsv`: assembled red-label `defa7.src` routine entry
  points currently used for `SCORE`, `SNDLD`, `SHELL`, `BKIL`, `LFIRE`,
  `LCOL`, `LASR` / `LASR0`, `LASL` / `LASL0`, `LASD`, `COLIDE`, `COL0`,
  `COLCHK`, `REV`, `PLEND` / `PDTHL` / `PDTH2` / `PDTH4` / `PDTH5`,
  `PXVCT` / `PX1A`, `PDTH5R`, `PDTH5SCLR`, `PLE02`, `PLE3`, `PLSTRT`, `ATTR`,
  `GEXEC` / `GEX0`, the assembled `GEXBON` post-`BONUS` return site, `SBOMB`,
  `BONUS`, `BC1`, `BC2`, `BC3`, `GETWV`, smart-bomb tail resume points,
  `HYPER` entry/resume labels,
  `SCLR1`, `PRDISP`, `PLAYER`, `THPROC`, `PRBKIL`, `MMSW`, `MSWM`, `MSWLP`,
  `SWBMB`, `MSWKIL`, `SHOOT`, `SCZST` / `SCZ0` / `SCZKIL`, `UFOST` /
  `UFOLP` / `UFOKIL`, `LKIL1`, `LKILL`, `AFALL` / `AFALL2`, `P250`, `P500`,
  `P503`, `BGI`, `TERBLO`, `TBL3`, `TBL4`, `TIEST`, `TIE`, `TIEKIL`,
  `OBJCOL`, and `OCVECT` dispatch.
- `player-death.tsv`: ROM-derived `PXCTB` player-death glow table and `PXCOL`
  player-explosion color table consumed by the translated death path.
- `scores.tsv`: score-card values currently parsed by `src/red_label.rs`.
- `shell-images.tsv`: `defb6.src` bomb shell image bytes consumed by the
  translated `BMBOUT` callback.
- `sound-tables.tsv`: red-label sound table bytes currently used by the
  translated `SNDLD` loader for replay, player-death, start, smart-bomb,
  bomb-hit, appearance, laser, probe-hit, schizoid-hit, UFO-hit, lander-hit,
  lander-pickup, lander-suck, lander-grab, lander-shot, astronaut-catch,
  astronaut-scream, astronaut-land, swarmer-hit, swarmer-shot, UFO-shot,
  terrain-blow, tie-hit, and schizoid-shot sounds.
- `sound-direct-command-sequences.tsv`: source-derived `SNDOUT` write rows for
  the direct `PDTH5`, `PLE2`, and `LNDFX0` sound commands, with each callsite
  expanded into idle and complemented command port writes.
- `sound-table-command-sequences.tsv`: source-derived `SNDOUT` write rows for
  every sound-table command, with each row pair covering the idle port write and
  the complemented command port write.
- `sound-table-timelines.tsv`: source-derived command and sequence-end rows for
  every embedded `SNDLD` table consumed by the Rust `SNDSEQ` timeline helper
  and checked by the timeline fixture validator.
- `sound-thrust-command-sequences.tsv`: source-derived `SNDOUT` write rows for
  the `SNDSEQ` thrust start and stop gate branches.
- `sram-routines.tsv`: red-label SRAM byte/word packing routine metadata for
  CMOS/high-score work.
- `switch-table.tsv`: complete red-label `defb6.src` `SWTAB` switch bit table
  used by the translated `SSCAN` / `SWP` scanner path.
- `terrain-data.tsv`: `blk71.src` `TDATA` terrain bitstream consumed by the
  translated `ALINIT` / `BGALT` altitude-table generator and `BGINIT`
  terrain flavor-table generator, plus `amode1.src` `MTERR` mini-terrain bytes
  consumed by `SCNRV`.
- `trace-scenarios.tsv`: Phase 1 trace scenario names, frame counts, and
  compact cabinet input programs, including the source-CMOS boot wait and
  MAME-observed credited-start prefix used by gameplay reference fixtures.
- `trace-requirements.tsv`: required per-scenario evidence markers for local
  MAME reference fixtures, including the observed credited-start sound commands
  and events.
- `trace-schema.tsv`: the single checked-in TSV trace schema source.
- `wave-table.tsv`: extracted source-order `blk71.src` `WVTAB` wave records.

`assets/arcade/arcade-rules.txt`, the PNG files under `assets/arcade/`, and the
WAV files under `assets/sounds/` are legacy prototype references unless a
clean-slate module explicitly reclassifies them with source/ROM provenance.
The runtime no longer embeds any `assets/arcade/*.png` file. The live renderer
no longer uses a temporary scaffold frame; it scales native red-label video RAM
directly, and source-backed score/replay, attract-logo, boot, start, gameplay,
death, high-score, and operator/AUDITG frames now have native video fixture
signatures. Newly regenerated local MAME reference traces also populate
`video_crc32` with the visible pixel-nibble CRC so those source-native
signatures can be checked against MAME-derived pixel evidence.

## Platform Support

The preferred live loop uses the windowed `wgpu` backend and does not require a
graphics terminal. The compatibility Kitty backend still depends on the Kitty
graphics protocol: launch `cargo run -- --renderer kitty` / `defender
--renderer kitty` from a real interactive terminal session inside `kitty`,
`ghostty`, `warp`, or a compatible emulator. If your terminal supports the
protocol but is not recognised by name, set `DEFENDER_FORCE_KITTY=1` to bypass
the terminal-name guard.

Live audio output is not implemented yet. Sound fidelity is currently covered by
source-visible command traces and deterministic DAC byte signatures, so the
runtime does not expose audio or mute controls.

Non-interactive tooling paths such as `--rom-report`, `--verify-roms`,
`--fidelity-trace`, `--fidelity-trace-inputs-file`, and
`--fidelity-check-trace` / `--fidelity-check-trace-dir` remain usable anywhere
a recent Rust toolchain is available.

The Kitty live session requests terminal keyboard-enhancement reporting so
standalone `Shift` thrust input can be captured in terminals that support the
extended key protocol. The `wgpu` backend receives keyboard input through
`winit` and maps it through the same cabinet input-profile layer.
