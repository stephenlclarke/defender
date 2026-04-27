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
- `cargo run -- --mute`
- `cargo run -- --rom-report`
- `cargo run -- --rom-report /path/to/roms`
- `cargo run -- --verify-roms /path/to/roms`
- `cargo run -- --fidelity-trace 300`
- `cargo run -- --fidelity-trace-inputs 'coin,start_one;fire,thrust;none'`
- `cargo run -- --fidelity-trace-inputs-file /path/to/inputs.txt`
- `cargo run -- --fidelity-check-trace /path/to/inputs.txt /path/to/expected.tsv`
- `cargo run -- --fidelity-check-trace-dir docs/fidelity/fixtures/local`
- `cargo run -- --fidelity-list-scenarios`
- `cargo run -- --fidelity-write-scenario-inputs docs/fidelity/fixtures/local`
- `cargo run -- --fidelity-check-reference-trace-dir docs/fidelity/fixtures/local`
- `make run`
- `make run-muted`
- `make trace-fixtures`
- `make reference-inputs`
- `make reference-traces`
- `make reference-fixtures-check`
- `cargo test`
- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `make ci`
- `make coverage`
- `make sq-ci`
- `make sq`
- `make readme-media` (currently archived with the old prototype)

Run the live game inside `kitty`, `ghostty`, `warp`, or another terminal that
supports the Kitty graphics protocol. `--rom-report` remains non-interactive,
validates red-label file sizes and CRC-32 values, and does not require a
compatible graphics terminal. `--verify-roms` performs the same validation and
then checks that the ROM files map into the embedded MAME red-label regions.
`--fidelity-trace` emits deterministic TSV frames from the current Rust core
for local trace fixture work. `--fidelity-trace-inputs` does the same with a
semicolon-separated per-frame cabinet input script. Use
`--fidelity-trace-inputs-file` to read the same script format from a local
fixture file. `--fidelity-check-trace` reads that input script, generates the
current Rust trace, and compares it exactly with an expected TSV fixture.
`--fidelity-check-trace-dir` checks all local `*.inputs.txt` /
`*.expected.tsv` fixture pairs in a directory, skipping the directory when it is
absent. `--fidelity-list-scenarios` and `--fidelity-write-scenario-inputs`
expose the Phase 1 reference scenario manifest from
`assets/red-label/trace-scenarios.tsv`. `--fidelity-check-reference-trace-dir`
validates that a local reference fixture directory has every required Phase 1
scenario input and expected trace with the checked-in schema header.

## Install

Install directly from git with Cargo:

- `cargo install --git https://github.com/stephenlclarke/defender defender`

After installation, run the clean-slate runtime and tooling with:

- `defender`
- `defender --mute`
- `defender --cmos-path ~/.local/state/defender/red-label-cmos.bin`
- `defender --rom-report`
- `defender --rom-report /path/to/roms`
- `defender --verify-roms /path/to/roms`
- `defender --fidelity-trace 300`
- `defender --fidelity-trace-inputs 'coin,start_one;fire,thrust;none'`
- `defender --fidelity-trace-inputs-file /path/to/inputs.txt`
- `defender --fidelity-check-trace /path/to/inputs.txt /path/to/expected.tsv`
- `defender --fidelity-check-trace-dir docs/fidelity/fixtures/local`
- `defender --fidelity-list-scenarios`
- `defender --fidelity-write-scenario-inputs docs/fidelity/fixtures/local`
- `defender --fidelity-check-reference-trace-dir docs/fidelity/fixtures/local`

Notes:

- Run `defender` inside `kitty`, `ghostty`, `warp`, or another terminal that
  supports the Kitty graphics protocol.
- Download Ghostty: <https://ghostty.org/download>
- Download Warp: <https://www.warp.dev/download>
- Exact high-score screen rendering and the full game-over-to-attract screen
  flow are not implemented in the clean-slate core yet. The ROM-derived
  high-score reset copy, packed high-score table comparison/insertion helpers,
  translated death-tail game-over handoff into deterministic initials-entry
  submission, and optional file-backed CMOS persistence through `--cmos-path`
  are modeled.

## Controls

Current live controls with the default `planetoid` input profile:

- `5`: insert a left-slot coin
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
- `A`-`Z`: enter initials while high-score entry is active
- `BACKSPACE`: delete the previous initial while high-score entry is active
- `Q` or `ESC`: quit

Letter-key controls accept either upper- or lower-case input.

The live key layout is currently modelled on the BBC Micro `Planetoid`
control scheme from Acornsoft's 1982 release of its `Defender` variant.

## XYZZY Overlay

During a live session, type `X`, `Y`, `Z`, `Z`, `Y` to toggle hidden `XYZZY`
mode on or off.

Typing `XYZZY` a second time turns the mode off and resets the hidden
invincibility and auto-fire toggles back to their default state.

Implemented clean-slate scaffold behavior while `xyzzy` mode is active:

- `F`: toggles the auto-fire compatibility flag.
- `G`: toggles the invincibility compatibility flag.
- Auto-fire currently emits the scaffold fire event while playing.
- Smart bombs can still emit a scaffold smart-bomb event after inventory reaches
  zero.

Future `xyzzy` effects such as the arcade laser cap override, bullet and mine
clearing, safe hyperspace, collision overrides, and falling-humanoid survival
must be added as explicit overlay hooks with paired arcade-off tests.

## Current Status

- Live mode renders an explicitly named scaffold frame, not yet the red-label
  video RAM output. Its pacing is derived from the core red-label
  `FRAME_RATE_MILLIHZ` constant, advances any due core frames before each
  terminal draw, and no longer lets terminal draw cadence decide core frame
  count.
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
  those bytes.
  The translated
  `SCPROC` process now runs the source `ISCAN`, `OSCAN`, and `SHSCAN`
  maintenance stages, bank-1 `SCNRV` scanner raster writes, and sleeps through
  the exact `SCP1`, `SCP2`, and `SCPROC` resume addresses. Shell output
  dispatch now uses the assembled red-label `OBJCOL` entry-point addresses
  embedded under `assets/red-label/routine-addresses.tsv`. The first
  source-shaped scoring and collision slice translates `SCORE`, the visible
  RAM effects of `SNDLD`, `BKIL`, `LFIRE` entry, `LASR0` / `LASL0`
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
  `ATTR` addresses. The wave-clear `BONUS` long-sleep path now writes the
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
  bombs, and player motion from red-label RAM. The remaining `PLRES` swarmer
  respawn path, whose `PLRES` phony-object X low byte needs register-aware
  scheduler tracing for the entry B register, live terrain scheduling, and full
  live respawn orchestration beyond that snapshot handoff remain documented
  gaps.
  The IRQ `PLAYER` motion slice now applies source-shaped 24-bit horizontal
  damping, thrust acceleration, X/scroll correction, absolute-X calculation,
  and altitude up/down velocity from `PIA21` / `PIA31`. The `PRDISP` player
  picture slice now stores the source scanline bound in `TEMP48`, gates on
  `STATUS` / `PLAYC`, erases the old 8x6 `OFF86` footprint, copies `NPLAD` /
  `NPLAXC`, and draws `PLAPIC` / `PLBPIC` through embedded `ON86` image bytes;
  the same slice now performs the adjacent `THOUT` / `THOFF` thrust-flame byte
  writes from the extracted `THTAB` data. `THPROC` now advances the
  fireball/thrust table pointers from source-shaped RAM. `OPROC` now erases and
  redraws active-object descriptor pictures in caller scanline bands with the
  ROM BGL-relative X and alternate-flavor checks, and `VELO` now advances
  active-object `OX16` / `OY16` by source velocities with Y wrap through
  `YMIN` / `YMAX`. The normal and inverted IRQ object-band tails now load the
  source scanline pairs from `XXX1` or `XXX2` and run already translated
  `PRDISP`, `OPROC`, `SHELL`, and `VELO` slices in the ROM order. The
  scanline object-phase gate now applies the source `VERTCT` thresholds, `IFLG`
  latch, `TIMER` increment, normal/flipped watchdog data byte, palette-copy due
  conditions, and `XXX2` calculations before entering those tails, and it runs
  translated `PLAYER` / `STOUT` pre-tail work on the source branches that call
  them. When the caller supplies the live 6809 stack pointer, the terrain branch
  also runs translated `BGOUT`; otherwise it records that `BGOUT` is due. The
  pre-tail `SNDSEQ` sound-table sequencer now advances `SNDX` / `SNDPRI` /
  `SNDTMR` / `SNDREP`, emits source-shaped main-board sound commands, and
  handles the thrust sound gate. Full frame output and fidelity traces now
  include the resulting raw command bytes. The source `CSCAN` branch now keeps
  the `PIA01` / `PIA02` coin-door history, masks IN2 through `ANDB #$3F`,
  double-checks the sample, and queues the first surviving `SWTAB1`
  coin/admin switch process. The queued coin process path now translates
  `LCOIN` / `RCOIN` / `CCOIN` debounce/sleep handling, `CN1` coin sound
  loading, and the fixed-bank BCD coinage/audit/credit updates. The queued
  admin switch path now translates `HSRES` today's-high-score reset and `ADVSW`
  manual diagnostics/audit target selection, with the diagnostic/audit mode
  handoff still explicit. Palette copy side effects, live stack-context wiring,
  and hardware-map restoration still need full scheduler integration.
  The `GEXEC` tail slice now
  restores `STRCNT` after star
  overflow, advances `GTIME` through the source
  audit-meter wrap, decrements the process `PD` counter, and applies source
  `WDELT` intra-wall deltas to `ELIST` every 40 passes. The `GETWV` wave
  parameter path now increments `PWAV`, refreshes `PENEMY` from source-order
  `WVTAB`, applies CMOS difficulty/ceiling inter-wall `WDELT` updates, and
  restores `PTARG` on the `GA1+6` restore-wave cadence. The start-flow
  foundation now extracts `CREDIT`,
  `CUNITS`, and `BUNITS`; implements exact `FPLAY` free-play credit seeding;
  and translates the RAM-visible `START` power-page gate, first-game player
  table reset/copy, `PLSTRT` process creation, source `SCRCLR` video-RAM clear,
  and `START2` BCD credit/`PLRCNT` tail plus `WCMOSA CREDST` packed CMOS credit
  backup. The source `TDISP` top-of-screen redraw now runs through extracted
  score digit, mini-ship, and smart-bomb image assets, including `BLKCLR`,
  `BORDER`, `LDISP`, `SBDISP`, and `SCRTR0` visible RAM writes. `ST1` and `ST2`
  now
  execute through the translated process dispatcher in the source order,
  including status/credit gates, start sounds, one/two `START` calls, and the
  final `DIE` process cleanup. Scanline scheduling, remaining `PLRES` swarmer
  object spawning, and live video presentation remain gaps.
  Live fire, smart bomb, hyperspace, reverse, and credited/free-play one- and
  two-player start buttons now enter through an asset-backed red-label `SWTAB`,
  `SSCAN` switch history, `SWPROC` queue, `SWP` status gate, and translated
  `LFIRE` / `SBOMB` / `HYPER` / `REV` / `ST1` / `ST2` scheduler paths. The
  scanner records all eight IN0 switch bits and queues every translated source
  switch process. Live coin/admin input now enters through the translated
  `CSCAN` / `SWTAB1` / `SWP` scheduler path, ignores the source auto/manual
  selector for queue priority while preserving it for `ADVSW`, ticks the source
  slam/coin debounce counters, sleeps through `LCOIN` / `RCOIN` / `CCOIN`, and
  awards credit from `CN1` with `CNSND`, slot audits, paid-credit audit,
  `CUNITS`/`BUNITS`, and CMOS-backed `CREDIT` / `CREDST` updates. Live
  high-score reset now runs through `HSRES`, and live service advance reports
  the translated diagnostics/audits target. No-credit one-player starts now
  enter translated `ST1` and die at the source credit gate instead of using the
  old compatibility shortcut. After a translated start, live player controls
  stay gated while the active `PLSTRT` / `PLSTR3` / `PLS01` / `PLS1`
  player-start handoff advances.
  `REV` sets
  `REVFLG`, negates `PLADIR` into `NPLAD`, debounces `PIA21`, clears the flag,
  and returns the process to the free list.
- ROM files are optional verification inputs only. The deployed runtime remains
  self-contained.
- MAME/source reference-trace tooling is in place for local Phase 1 fixtures,
  but checked-in golden traces are intentionally absent because they are local
  emulator outputs.
- Exact high-score screen rendering, full game-over-to-attract timing,
  two-player sessions, operator settings, untranslated object/shell/process
  bodies, generic body dispatch/frame scheduling, and sound-routine execution
  are still fidelity gaps tracked in `SPEC.md` and `docs/fidelity/gaps.md`.

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

These references are used for reverse engineering, rules verification, attract
screen reconstruction, extraction of historical arcade data, and future
translation work while keeping the final runtime self-contained. A reference
listed here does not imply the clean-slate runtime has implemented that behavior
yet; current implementation status is tracked in `SPEC.md` and
`docs/fidelity/gaps.md`.

- <https://github.com/mwenge/defender>: Motorola 6809 assembly language for the
  'Red Label' version of the game. Used for reference implementation and ROM
  layout comparison point, especially for the red-label `defend.*` program ROM
  names, the `mess0.src` `CHRTBL` / `CHARACTERS` font tables used to build the
  embedded `assets/arcade/font-sheet.png`, and the `amode1.src` `LGOTAB`,
  `DEFDAT`, `CPRTAB`, `TEXTAB`, `TENT`, `ENMYTB`, `PICTS`, `XS`, and `BLIPS`
  tables used to reconstruct the embedded attract-logo page, `DEFENDER`
  wordmark assets, and the instruction-page rescue/legend sequence, plus the
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
  player-explosion and non-wave-end branch path, plus the `defb6.src`
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
  Defender visible-area crop, palette-RAM-index frame extraction, and palette
  resistor conversion. MAME's `src/emu/video/resnet.cpp` provides the resistor
  weight algorithm used for the RGBA palette lookup.
- <https://github.com/mamedev/mame/blob/master/src/devices/machine/6821pia.cpp>:
  MAME Motorola 6821 PIA device source used for the main-board PIA data/control
  register behavior, data-direction filtering, CA/CB IRQ flags, CA2/CB2 strobe
  behavior, and output-callback timing.
- <https://github.com/historicalsource/williams-soundroms>: original Williams
  sound-ROM source reference for the future `VSNDRM1.SRC` translation,
  including the `IRQ` dispatch path, `RADSND`, `SVTAB`, `GFRTAB`, `GWVTAB`,
  `SCREAM`, and organ-note/tune logic. Legacy cue assets are kept only as
  archived prototype references under `assets/sounds/`.
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
  `SBOMB`, `BONUS`, `BC1`, `BC2`, `BC3`, `GETWV`, smart-bomb tail resume points,
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
- `sram-routines.tsv`: red-label SRAM byte/word packing routine metadata for
  CMOS/high-score work.
- `switch-table.tsv`: complete red-label `defb6.src` `SWTAB` switch bit table
  used by the translated `SSCAN` / `SWP` scanner path.
- `terrain-data.tsv`: `blk71.src` `TDATA` terrain bitstream consumed by the
  translated `ALINIT` / `BGALT` altitude-table generator and `BGINIT`
  terrain flavor-table generator, plus `amode1.src` `MTERR` mini-terrain bytes
  consumed by `SCNRV`.
- `trace-scenarios.tsv`: Phase 1 golden-trace scenario names, frame counts, and
  compact cabinet input programs.
- `trace-schema.tsv`: the single checked-in TSV trace schema source.
- `wave-table.tsv`: extracted source-order `blk71.src` `WVTAB` wave records.

`assets/arcade/arcade-rules.txt`, the PNG files under `assets/arcade/`, and the
WAV files under `assets/sounds/` are legacy prototype references unless a
clean-slate module explicitly embeds them through `src/assets.rs`. At present,
the clean-slate renderer embeds only `assets/arcade/logo-page.png` for the
temporary scaffold frame.

## Platform Support

The live loop now depends on a terminal that supports the Kitty graphics
protocol. `cargo run` / `defender` should be launched from a real interactive
terminal session inside `kitty`, `ghostty`, `warp`, or a compatible emulator.
If your terminal supports the protocol but is not recognised by name, set
`DEFENDER_FORCE_KITTY=1` to bypass the terminal-name guard.

Non-interactive tooling paths such as `--rom-report`, `--verify-roms`,
`--fidelity-trace`, `--fidelity-trace-inputs-file`, and
`--fidelity-check-trace` / `--fidelity-check-trace-dir` remain usable anywhere
a recent Rust toolchain is available.

The live session now requests terminal keyboard-enhancement reporting so
standalone `Shift` thrust input can be captured in terminals that support the
extended key protocol.
