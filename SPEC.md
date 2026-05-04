# Defender Exact Arcade Implementation Spec

## Purpose

This document records the current fidelity gaps in this repository and defines
a plan to turn it into an exact Rust implementation of Williams Defender
red-label gameplay, behavior, and appearance.

The target is not just "Defender-like". The arcade core should become
deterministic and trace-equivalent to the red-label arcade game for the same
inputs, random state, object tables, timers, and visible video state. The
project should still keep two deliberate compatibility features:

- `xyzzy` mode, implemented as an explicit non-arcade overlay.
- BBC Micro Planetoid key mappings, implemented as an input profile over the
  original cabinet actions.

## Source Of Truth

The reference target is Defender red-label, not later revisions or home ports.
These are the source materials to use when settling behavior:

- Red-label source and ROM build reference:
  <https://github.com/mwenge/defender>
- Red-label ROM metadata and MAME driver metadata:
  <https://mdk.cab/game/defender>
- MAME Williams driver source for red-label ROM regions and load offsets:
  <https://github.com/mamedev/mame/blob/master/src/mame/midway/williams.cpp>
- Williams sound-ROM source:
  <https://github.com/historicalsource/williams-soundroms>
- Defender setup and operations manual:
  <https://arcade-museum.com/manuals-videogames/D/DefenderSetupBookletUSA.pdf>
- Control behavior analysis:
  <https://www.dougmahugh.com/defender-chapter02/>
- Current repo references listed in `README.md`.

When sources disagree, precedence should be:

1. Red-label assembled ROM behavior under MAME.
2. Red-label source tables and routines.
3. Williams operator documentation.
4. External behavior analysis and screenshots.
5. Current repo behavior only as a non-authoritative clue. Existing Rust or
   archived prototype behavior must not settle arcade behavior unless it is
   traced back to one of the sources above.

## Definition Of Exact

Exact means the arcade core can be validated against red-label ROM behavior, not
only by visual inspection. Acceptance requires:

- Same wave setup, object counts, object tables, and spawn timing.
- Same player movement, inertia, thrust, reverse, vertical control, laser,
  smart bomb, hyperspace, death, respawn, and scoring behavior.
- Same enemy process behavior for Landers, Mutants, Baiters, Bombers, Pods,
  Swarmers, shells, mines, humans, and planet destruction.
- Same high-score, attract, game-over, bonus-stock, and per-player state rules.
- Same visible playfield, scanner, HUD, object pictures, palette behavior, and
  attract sequence within a defined pixel tolerance.
- Same sound-board command behavior and timing within a defined audio tolerance.
- Same behavior with `xyzzy` disabled as the cabinet. `xyzzy` must be isolated
  and testable as an overlay.

## Non-Negotiable Implementation Rule

Build this as a strict red-label Defender translation, not a reinterpretation.

For every gameplay, video, audio, timing, input, scoring, RNG, object, shell,
wave, attract, high-score, and cabinet-state behavior:

- derive it from the red-label source, red-label ROM behavior, or MAME behavior;
- cite the source routine, table, file, MAME field, or trace fixture in comments
  or docs when adding code;
- do not approximate, simplify, tune by feel, interpolate missing constants, or
  invent behavior;
- if exact behavior is unknown, stop and add a documented TODO plus a failing or
  ignored fidelity test instead of guessing;
- keep compatibility features such as `xyzzy` and Planetoid controls outside
  the arcade core as explicit overlays;
- maintain at least 80% unit-test line coverage;
- add golden-trace, pixel, waveform, or command-sequence tests for every
  translated subsystem where possible;
- keep the binary self-contained: no runtime dependency on local ROMs or assets
  except optional verification tooling.
- extract every game asset, ROM-derived table, score/default table, high-score
  seed, video/audio asset, and fidelity schema into `assets/` first, then embed
  it into Rust from that directory with `include_str!` or `include_bytes!`.
  Arcade-core code must not hardcode gameplay asset tables directly.

The most important rule is that unknown exact behavior must be documented as a
gap. Guesswork is not allowed in arcade-core code.

## Current Implementation Summary

The repository is now in a clean-slate rewrite. The previous prototype has been
moved to `oldsrc/` for reference, and the new implementation under `src/`
has moved beyond foundation-only scaffolding into a table-backed red-label
translation with many source-shaped routine slices. It is still incomplete and
now presents translated video RAM through the native cabinet scaler without a
synthetic scaffold fallback; untranslated blank screens remain black:

- `src/red_label.rs` holds translated tiny routines such as `RAND` and `RMAX`,
  and reads defaults, scores, and high-score seed data from embedded assets.
- `src/red_label_memory.rs` reads the embedded MAME CPU memory map, red-label
  SRAM routine metadata, `phr6.src` CMOS cell layout metadata, `romc8.src`
  CMOS default bytes, RAM table layout metadata, and source-owned linked-list
  heads. It exposes source-cited helpers for the CMOS/SRAM byte and word
  packing used by the fixed-bank routines at `0xF813`, `0xF822`, `0xF838`,
  `0xF83A`, and `0xF842`.
- `src/red_label_wave.rs` reads the extracted red-label `WVTAB` records from
  embedded assets and exposes source-order `GETWV` base value copies plus
  `WDELT` intra-wall and inter-wall delta updates.
- `src/machine.rs` defines the deterministic arcade-core scaffold and
  table-backed red-label RAM image. It initializes `PINIT`/`OINIT`-style free
  process, super-process, and object lists from
  `assets/red-label/ram-layout.tsv` and `assets/red-label/linked-lists.tsv`;
  seeds one-player `START` table fields from `romc8.src` CMOS defaults; exposes
  object-table and SPTR-head CRCs to traces; and has source-shaped `MKPROC`,
  `MSPROC`, `SLEEP`, `KILL`, and `DISP` primitives for process creation, delay,
  free-list return, scheduler timer decrement, `CRPROC` update, and due-`PADDR`
  reporting. It translates active/inactive object and shell-list maintenance,
  player/start/death slices, switch dispatch for translated actions, selected
  enemy/support process bodies, terrain-table generation, scoring, collision,
  object-picture output, and IRQ object-band slices while preserving explicit
  gaps for unknown or unverified routines.
- `src/input.rs` maps Planetoid, cabinet, and test input profiles into cabinet
  actions and keeps `xyzzy` as an overlay. It also projects cabinet actions,
  service switches, and coin lines into the MAME-documented active-high
  Defender IN0, IN1, and IN2 port bytes used by PIA0 port A, PIA0 port B, and
  PIA1 port A.
- `src/pia.rs` implements the MAME-modeled Motorola 6821 PIA data/control
  register surface used by the main board: DDR versus data register selection
  through control bit 2, masked control-register writes, DDR-filtered input
  reads, output callbacks on data writes or DDR changes, CA1/CB1 and
  input-mode CA2/CB2 edge IRQ flags, control-register IRQ flag bits, and
  data-port read clearing of those flags. It also models MAME's CA2/CB2
  set/reset output modes and the read/write strobe restore behavior, including
  CB2's delayed CB1-restore path through port-B reads.
- `src/board.rs` exposes the first MAME-documented main-board address
  classifier and memory surface: RAM at `0x0000..=0xbfff`, banked I/O or ROM
  at `0xc000..=0xcfff`, bank-select writes at `0xd000..=0xdfff`, and fixed ROM
  reads at `0xd000..=0xffff`. Raw write-only palette register storage exists
  for the 16 `0xc000..=0xc00f` `BBGGGRRR` entries. The board surface also
  implements the simple MAME-documented handlers for Defender CMOS 4-bit writes,
  video-control cocktail bit 0, watchdog reset recognition on byte `0x39`,
  video-counter reads from the current `vpos`, CPU reads of IN0/IN1/IN2 through
  the main-board PIA data/control registers, and DDR-filtered PIA1 port-B
  output writes that send sound commands to the sound-board latch model. It
  also exposes MAME's older Williams VA11 and COUNT240 video interrupt inputs
  into PIA1 CB1 and CA1. The board harness can now reproduce the visible cell
  effects of `PWRUP`, `CLRAUD`, and `CMINIT`: red-label packed zero writes clear
  audit cells or all CMOS cells, ROM-derived defaults from
  `assets/red-label/cmos-defaults.tsv` are copied into the `CMSDEF` range, and
  the `CMOSCK` / `DIPFLG` power-up branch clears the special-function flag and
  returns source-cited actions for auto-cycle, audit clear, and default reset
  while running the translated visible high-score reset copy for the `0x25`
  special function. It can also return slices for source-owned CMOS fields from
  `assets/red-label/cmos-layout.tsv` and RAM fields from
  `assets/red-label/ram-layout.tsv`, so future traces can checksum
  CMOS/player/object/process bytes without Rust-only structs.
- `src/sound.rs` exposes the first MAME-documented Defender 6808 sound-board
  address classifier and memory surface: internal RAM at `0x0000..=0x007f`,
  PIA IC4 at `0x0400..=0x0403` mirrored at `0x8400..=0x8403`, and sound ROM
  reads at `0xb000..=0xffff` through the embedded red-label ROM image view. It
  also models the MAME-documented main-board sound-command latch handoff: PIA1
  port-B output values are ORed with `0xc0`, latched on sound PIA port B, and
  assert sound CB1 for every byte except `0xff`. The sound CPU can now read the
  latched command through the PIA IC4 port-B data/control register path, and
  sound PIA port-A output writes are captured as the DAC boundary.
- `src/video.rs`, `src/kitty.rs`, and `src/terminal.rs` provide a temporary
  self-contained Kitty presentation path using embedded assets. `src/video.rs`
  also exposes the MAME-documented Williams screen-format helpers for the
  native Defender visible area: absolute screen-to-video-RAM byte offsets,
  high/low pixel nibbles, palette-RAM-index visible frames, and
  resistor-weighted RGBA palette conversion. The renderer can scale and
  letterbox a native RGBA cabinet frame for Kitty presentation.
- `assets/red-label/` now holds embedded TSV source assets for ROM metadata,
  MAME ROM region/load mapping, MAME input-port layout, MAME main-board and
  sound-board memory maps, SRAM routine metadata, red-label CMOS layout and
  default metadata, red-label RAM table layout metadata, linked-list metadata,
  assembled routine entry-point addresses, defaults, score values, high-score
  seeds, trace schema, Phase 1 trace scenarios, and `WVTAB` data. The trace
  schema now records both the current internal input bit packing and the MAME
  IN0/IN1/IN2 bytes needed for later ROM-equivalence fixtures, plus optional
  raw object table, shell table, and native video-frame CRC-32 columns.
- `src/rom.rs` reads embedded ROM metadata, builds verified addressable ROM
  images for the main CPU, banked program ROMs, sound CPU ROM, and decoder
  PROMs for optional local verification and future extraction, and derives the
  24-byte source-shaped `romf8.src` `ROMMAP` descriptor table consumed by
  `CROM0` through `ROM0`/`ROM9` from the embedded MAME load map. It can also
  run the source-shaped `ROM0`/`ROM9` checksum scan over those images and report
  the physical ROM numbers that `CROM0` would display for failures, plus the
  manual/auto ROM-stage outcome for success/failure display intent, `ADVSW` /
  `NEXTST` gate sequence, message-ROM bitmap text transfer including CMOS text
  controls, RAM-test start/failure/no-error visible setup, the RAM2 pattern
  fill/verify pass with page-boundary operator-poll metadata, pass-boundary loop
  dispatch, and CMOS RAM-test write/verify loop plus visible outcomes, and the
  CROM0 color-RAM diagnostic
  heading/bars/palette loop plus audio-test sound-pulse/skip-table behavior,
  switch-test display-table/PIA-scan behavior, monitor-test
  crosshatch/RGB-field/color-bar pattern behavior, and the monitor-test
  audit-branch transfer into the `AUDITG` entry screen.
- `src/app.rs` exposes non-interactive verification commands:
  `--verify-roms PATH` checks a local red-label ROM directory and maps it into
  the embedded MAME ROM regions, while `--fidelity-trace [FRAMES]` emits the
  current deterministic Rust trace TSV for local fixture work.
  `--fidelity-trace-inputs SCRIPT` emits the same TSV while applying
  semicolon-separated per-frame cabinet action sets such as
  `coin,start_one;fire,thrust;none`. `--fidelity-trace-inputs-file PATH`
  reads that same script format from a local fixture file.
  `--fidelity-check-trace INPUTS_PATH EXPECTED_TSV` generates the Rust trace
  from a file-backed input script and compares it exactly with a local expected
  TSV fixture. `--fidelity-check-trace-dir PATH` checks every local
  `*.inputs.txt` / `*.expected.tsv` fixture pair in a directory and skips a
  missing directory so the optional local fixture target is usable before
  golden traces exist. `--fidelity-list-scenarios`,
  `--fidelity-write-scenario-inputs PATH`, and
  `--fidelity-check-reference-trace-dir PATH` expose and validate the Phase 1
  local golden-trace scenario manifest. `tools/generate_reference_traces.py`
  and `tools/mame_defender_trace.lua` provide the local MAME runner that turns
  those scenarios into ignored `*.expected.tsv` reference fixtures.

The new code builds and keeps unit-test line coverage above 80%, but it is not
yet an exact arcade game. Remaining work is concentrated in trace-proving the
translated slices, completing scheduler/frame integration, replacing live
scaffold presentation with video-RAM output, and translating the remaining
session, sound, operator, wave-launch, and object-process behavior.

## Drift Review

This section records drift found during the repository review on
`2026-04-25`.

### Aligned Areas

- `src/app.rs`, README run targets, and `docs/fidelity/README.md` agree on the
  current non-interactive tooling: ROM reporting, ROM verification, deterministic
  trace emission, scripted input traces, file-backed input traces, and exact
  local TSV trace comparison.
- `src/assets.rs` embeds the active red-label TSV assets from `assets/red-label/`
  and the current runtime does not need a local ROM or asset directory.
- `assets/red-label/memory-map.tsv`, `src/red_label_memory.rs`, and the
  board/sound classifier tests now agree on the MAME-documented CPU address
  ranges and mirrors.
- `assets/red-label/sram-routines.tsv`, `src/red_label_memory.rs`, and the
  main-board CMOS tests now agree on the red-label SRAM byte/word packing
  contract: the first 4-bit cell is the most significant nibble, followed by
  decreasingly significant cells.
- `assets/red-label/cmos-layout.tsv`, `src/red_label_memory.rs`, and the
  board harness now agree on the source-owned `phr6.src` CMOS audit,
  high-score, credit-backup, coinage/operator, and game-adjust cell ranges.
- `assets/red-label/cmos-defaults.tsv`, `src/red_label_memory.rs`, and the
  board harness now agree on the `romc8.src` `DEFALT` bytes copied by
  `CMOSMV` / `CMINIT`, expanded as two 4-bit CMOS cells per byte.
- `assets/red-label/audit-adjustments.tsv`, `src/red_label_memory.rs`, and the
  board harness now agree on the `romc8.src` `AUDITG` / `MSGAUD` audit and
  operator-adjustment messages, CMOS offsets, and packed display widths.
- `src/board.rs` now models the source-visible `AUDITG` game-adjust entry
  screens: `VINS15` title transfer, `IAUD1` initial instructions, 1.5-second
  delay intent, and active `IAUD2` prompt state.
- `src/board.rs` now models the source-visible `DISAUD` audit row stack buffer:
  row numbers, packed CMOS values, the replay row's dummy trailing zeroes, and
  `MSGAUD` messages land in the same 31 visible character columns; the row can
  also transfer to video RAM at `0x1080` while erasing the previous visible row.
- `src/board.rs` now models the source-visible `AUDITG` post-display
  delay/debounce registers: the first 100-tick scan delay, the six-tick repeat
  scan cadence, and the `BITB #$0A` release shift register.
- `src/board.rs` now ties the source-visible `AUDITG` row step, `DISAUD`
  display-line formatting/video transfer/previous-row erasure, and
  post-display debounce gate into one deterministic audit cycle helper.
- `src/board.rs` now models the post-`PWRUP` `AUDITG` outer frame step: the
  first frame records the source dispatch target and transfers the entry
  screen, then later frames advance the audit display/debounce cycle until
  auto/up after row 28 returns to the caller.
- `src/board.rs` now models the source-visible `ALTER` / `HYSCRE` operator
  adjustment mutation rules: audit rows stay read-only, coin multiplier rows
  respect `COINSL`, `DIPSW` changes set `DIPFLG`, and byte/replay-level values
  use the `BMPNUP` / `BMPNDN` packed-BCD update paths.
- `src/board.rs` now models the source-visible `AUDITG` operator row step:
  service advance moves through `MSGAUD` rows, auto/up exits after row 28, and
  high-score reset applies the current row's adjustment direction.
- `src/board.rs` now models the visible CMOS cell effects of `CLRAUD` and
  `CMINIT`: `CLRAUD` performs 14 packed zero writes from `CMOS`, while `CMINIT`
  clears the visible CMOS image and applies the embedded `DEFALT` bytes.
- `src/board.rs` now models the CMOS-visible `PWRUP` branch around `CMOSCK`,
  `DIPFLG`, and `DIPSW`: bad check bytes and the `0x45` default function run
  `CMINIT`, `0x35` runs `CLRAUD`, `0x15` returns the auto-cycle ROM-test
  action, `0x25` runs the translated high-score reset copy, and `AuditGate`
  dispatch transfers the `AUDITG` entry screen.
- `src/board.rs` now models the visible `RHSTD` / `RHSTDS` reset copy:
  the first 48 `DEFALT` bytes are copied back into the all-time `CRHSTD` CMOS
  range and into the `THSTAB` "today's greatest" RAM table using the same
  two-cell packed format.
- `src/board.rs` now models packed high-score table comparison/insertion for
  the all-time `CRHSTD` CMOS table and the `THSTAB` "today's greatest" RAM
  table: six-digit BCD scores plus three ASCII initials decode from the same
  two-cell format, qualifying scores shift lower entries, and the dropped entry
  is reported.
- `assets/red-label/ram-layout.tsv`, `assets/red-label/linked-lists.tsv`,
  `src/red_label_memory.rs`, and the board harness now agree on the source-owned
  `phr6.src` player table, object cell, process cell, super-process cell, and
  list-head addresses. Shells are now recorded as SPTR-linked object cells
  rather than as an invented separate Rust table.
- `src/machine.rs` now translates the visible `defa7.src` object-list and
  shell-list primitives `GETOB`, `OBINIT`, `KILLOB`, `KILSHL`, and `GETSHL`
  over the embedded `phr6.src` object cells, base-page state, and list heads.
  It translates `OSCAN` / `ISCAN` active/inactive object scanner maintenance
  over `OPTR`, `IPTR`, `BGL`, `XTEMP`, `OX16`, `OY16`, `OXV`, and `OYV`. It
  also translates `SHSCAN` shell timer cleanup over `SPTR`, `ODATA`, `OFREE`,
  and `BMBCNT`, the `SHELL` movement/death/dead-erase front half over
  `STATUS`, `BGL`, `BGLX`, `SHTEMP`, `OX16`, `OY16`, `OXV`, and `OYV`, and
  `OPROC` active-object descriptor erase/write banding over `OPTR`, `OBJX`,
  `OBJY`, `OPICT`, `OX16`, `OY16`, and `BGL`. It also translates `VELO`
  active-object velocity addition and Y wrap over `OPTR`, `OX16`, `OY16`,
  `OXV`, `OYV`, `YMIN`, and `YMAX`, plus source-ordered normal and inverted
  `IRQ` object-band tails over `XXX1` / `XXX2` / `XXX3` that sequence the
  already translated `PRDISP`, `OPROC`, `SHELL`, and `VELO` slices.
  The `BMBOUT` and `FBOUT` shell output callbacks are translated as raw
  video-memory byte writes over `BAX`, `FBX`, `OBJX`, `OBJY`, and `OX16+1`;
  dispatch from `OBJCOL` uses the assembled entry points in
  `assets/red-label/routine-addresses.tsv`, and the ROM-resident bomb image
  bytes are embedded from `assets/red-label/shell-images.tsv`. The first
  source-shaped scoring/collision slice embeds complete and short-form
  `defb6.src` object-picture metadata and object image bytes; translates
  `SCORE` including visible score-digit refresh and replay stock-icon redraw,
  the visible RAM effects of `SNDLD`, `BKIL`, `LFIRE` entry,
  `LASR0` / `LASL0` drawing/fizzle/erase/sleep loops, `LASD` tail, the
  RAM-visible part of `LCOL`, source-ordered `CRINIT` / `FISS` / `STINIT` /
  `OINIT` / `FBINIT` / `THINIT` boot table initialization, `STOUT` star output
  including `SBLNK` color blink and hyperspace/RAM-blast side effects, exact
  `COLIDE` / `COL0` non-zero picture-byte intersection for those descriptors,
  source-shaped `CWRIT` / `COFF` generic picture writes/erases plus descriptor
  ON/OFF picture dispatch helpers, `OPROC` active-object display banding, the
  source `VELO` active-object velocity update, the visible `COLCHK`
  player-collision flag/status/death-process writes, the `REV` reverse
  debounce path, and the `SBOMB` entry/flash/debounce tail; and routes
  translated object
  collisions through `OCVECT`, currently including source no-op `NOKILL`.
- `make fidelity` exists and runs formatting, tests, clippy, the MAME trace
  script self-test, local trace-fixture checks, and the 80% line coverage gate.
  This is no longer a placeholder.
- The live path now feeds translated video RAM into the native
  `render_cabinet_frame` presenter without a synthetic fallback; blank or
  untranslated screens remain native black.

### Resolved Drift Items

- `docs/fidelity/trace-schema.tsv` was a stale duplicate of
  `assets/red-label/trace-schema.tsv`. It has been removed, and
  `src/assets.rs` now has a unit test that fails if the stale docs duplicate is
  reintroduced. The single checked-in trace schema source is
  `assets/red-label/trace-schema.tsv`.
- README old-prototype drift has been cleaned up. It now describes the current
  scaffold controls, implemented `xyzzy` overlay flags, active
  `assets/red-label/` files, and legacy prototype assets without claiming that
  unported high-score, hyperspace, humanoid, enemy, or sound behavior is already
  implemented.
- The `PDTHL`, `PDTH2`, `PDTH4`, and `PDTH5` entries in
  `assets/red-label/routine-addresses.tsv` previously pointed at the preceding
  sleep setup bytes. They now use ROM-confirmed label addresses, and the table
  also records `PXVCT`, `PX1A`, `PDTH5R`, `PLE02`, `PLE3`, `PLSTRT`, and
  `ATTR` for the translated death-tail dispatch.
- The bank-7 terrain vectors are recorded from `phr6.src`: `BGINIT` is
  `0xC000`, `BGOUT` is `0xC003`, `ALINIT` is `0xC006`, and `BGERAS` is
  `0xC009`. The earlier `ALINIT`/`BGINIT` drift has been corrected.
- The fixed-ROM active-screen clear and bonus process entries are now recorded
  from ROM call sites: `SCLR1` is `0xF5F1` and `BONUS` is `0xDDEC`. The
  active-screen clear has been split from `SCRCLR`: translated `SCLR1` callers
  now clear only rows `Y >= 42` on each video page, matching the original stack
  clear loop. The `BONUS` continuation labels and the return site back into
  `PDTH5` are also ROM-confirmed: `BC1` `0xDE66`, `BC2` `0xDE6C`, `BC3`
  `0xDE79`, `GETWV` `0xDE7C`, and `PDTH5SCLR` `0xDADB`. The `GEX0`
  post-`BONUS` return site is recorded as `GEXBON` `0xDD02`; it is an
  assembled return address for the Rust dispatcher, not a standalone source
  label.
- The base-page scratch RAM layout was corrected from the source allocation:
  `CURSER` is recorded at `0xA050`, `ITEMP` at `0xA06F`, and `ITEMP2` at
  `0xA071`. The earlier `ITEMP` / `ITEMP2` entries incorrectly overlapped the
  message workspace and have been moved behind `SPTR` where `phr6.src` places
  them.

### Drift And Cleanup Items

- `assets/arcade/*.png`, `assets/arcade/arcade-rules.txt`, and
  `assets/sounds/*.wav` remain from the prototype. `assets/arcade/README.md`
  and `assets/sounds/README.md` classify them as archived prototype references.
  Only `assets/arcade/logo-page.png` is currently embedded by the clean-slate
  runtime, and it is explicitly temporary until video RAM output is proven.
  Remaining work: regenerate any needed visual/audio asset from red-label
  source/ROM data or remove it from the shipped runtime path.
- `assets/red-label/high-scores.tsv` is parsed as a seed table,
  `assets/red-label/cmos-defaults.tsv` records the matching ROM default CMOS
  bytes, and the board can compare/insert entries in both all-time and today's
  packed high-score tables. Live mode can load/save the 256-cell CMOS image
  through a file-backed storage trait when `--cmos-path` is provided, and the
  core can collect three initials for a qualifying game-over score before
  submitting it to the all-time CMOS table and `THSTAB` today's-greatest RAM
  table. Translated player-death game-over sleeps now hold the clean core in
  `GameOver` for the source `PLE2`/`PLE3` 40-tick delay before `HALLOF`;
  non-qualifying sessions also sleep through the source `HALL13` no-entry delay
  before `HALDIS`. The `PLE3` handoff now applies source `ATTR`'s map-1
  select, and `HALLOF` runs source `GNCIDE`, `STINIT`, credit mirroring, and
  entry-flag clearing before falling into `HALL1` or jumping to `AMODES` during
  the power-on pass. `HALDIS` runs source `GNCIDE` before drawing while
  preserving coin processes. Source branch-only labels `CPR56` and `HALD4`
  now dispatch through the same translated `HALDIS` and `LEDRET` paths, and
  live rendering copies source `PCRAM` into hardware palette RAM before
  scaling each native cabinet frame. The
  `HALL1` qualification/body path now drives the source-backed initials-entry
  setup and falls through to `HALL3A`. The
  `HALLOF` / `HOFIN` initials-entry UI now writes source-backed video RAM, and
  submitted high-score sessions now render the source-backed `HALDIS`
  hall-of-fame table.
- The current deterministic trace compares Rust output to local expected TSV,
  `docs/fidelity/fixtures/` defines the ignored local fixture layout, and
  Phase 1 now has a local MAME/source trace runner plus a complete scenario
  manifest. Generated golden TSV files remain local artifacts, not checked-in
  source. The MAME generator now seeds source CMOS defaults by default, uses a
  MAME-observed ready/coin/start sequence, and validates `0xE6` /
  `credit_added` plus `0xF5` / `game_started` evidence before gameplay-named
  reference traces are accepted.
- `src/machine.rs` now backs the core trace CRC columns with red-label RAM
  bytes instead of placeholder `None` values: object-table CRCs come from the
  initialized object cells, and shell CRCs come from the SPTR shell-list head.
- The board and sound surfaces model useful MAME-documented boundaries, but
  they are not yet integrated into `ArcadeMachine::step`. The game core still
  advances most scaffold state directly instead of executing translated
  red-label routines against cabinet memory.

### Next Work Order

1. Use the new MAME/source Phase 1 reference oracle as the acceptance gate for
   each translated subsystem as Rust-current exact-match fixtures are promoted.
2. Continue the table-backed core path by translating the remaining reset,
   boot/default initialization beyond the translated `CRINIT` / `FISS` /
   `STINIT` / `OINIT` / `FBINIT` / `THINIT` table initializers, live scanline
   object rendering ownership around the translated picture helpers, remaining
   shell/object collision callbacks, and process body dispatch around the
   initialized RAM tables and process/object primitives.
3. Continue player control and firing routines next: remaining reverse
   player-motion integration, scanline player display scheduling, laser
   allocation, remaining smart-bomb movement/world integration, hyperspace,
   death, respawn, and carried-human state, each checked against local golden
   traces.
4. Continue defining the raw red-label cabinet memory contract beyond the
   embedded MAME address map, SRAM packing helpers, CMOS layout, and first
   `phr6.src` RAM table assets: video RAM ownership, field semantics inside
   object/process cells, and the exact per-routine bytes that must be traced.
5. Extend trace or pixel fixtures around the native video-RAM presenter, then
   keep replacing untranslated blank screens with translated cabinet screens.
6. Continue porting the Williams sound-board path from `VSNDRM1.SRC`: setup,
   IRQ command decoding, the normal `IRQ1` `GWAVE` / `VARI` command runner,
   the `SP1` / `BG1` / `BG2INC` / `LITE` / `BON2` / `BGEND` / `APPEAR` /
   `TURBO` / `THRUST` / `CANNON` / `RADIO` / `HYPER` / `SCREAM` /
   `ORGANT` / `ORGANN` special command runners,
   the `GWLD` / `VARILD` table loaders, the `SP1` / `BON2` / `BG2` pre-loop
   setup paths, and the shared
   `GEND` / `GEND40` / `GEND50` / `GEND60` / `GEND61` echo and
   frequency-window updates plus the `GWAVE` / `GPLAY`
   per-period, `VARI` / `VSWEEP` per-sweep, `LITE` /
   `APPEAR` shared `LITEN` random-complement, `TURBO` / `NOISE` noise-decay,
   `BG1` / `THRUST` first-window `FNOISE`, `CANNON` / `FNOISE`
   filtered-noise decay, `RADIO` / `RADSND` timer-table, `HYPER` phase-edge,
   `SCREAM` echo-cascade, `ORGANN` / `ORGNN1` first-duration `ORGAN` note,
   `ORGNT1` / `ORGTAB` organ tune DAC byte streams, the source `IRQ3`
   background handoff, and the source IRQ organ-continuation gate are modeled,
   while
   cycle-accurate waveform/DAC routine scheduling and command-sequence fixtures
   remain.
7. Keep `xyzzy` and Planetoid behavior outside the arcade core; add paired
   off/on tests for each future `xyzzy` hook.

## Current Fidelity Gaps

### Core Runtime And Timing

- The previous coarse `World` model is archived under `oldsrc/`; the new
  `ArcadeMachine` now owns a table-backed red-label RAM image and initializes
  source-owned player, player-start, switch-history, switch-queue, object,
  process, super-process, and shell-list heads for the first table-backed state
  step. It still does not execute translated
  object/shell/process bodies or full cabinet memory state; only the first
  source-shaped `COLIDE` / `COL0` scan over extracted complete and short-form
  object-picture descriptors, `LFIRE` entry, `LASR0` / `LASL0`
  drawing/fizzle/erase/sleep loops, `LASD` tail, the RAM-visible `LCOL` laser
  collision setup, `OCVECT` dispatch to translated vectors, `REV` reverse
  debounce path, and source-shaped `SBOMB` entry/flash/debounce tail are
  translated.
- The live loop now uses a cabinet-rate frame duration and advances any due
  core frames before each terminal draw, but the core does not yet execute
  red-label process timing or CPU-cycle-level behavior.
- `RAND` is translated, but the whole program does not yet run the same sequence
  of RNG reads as the red-label program. Exactness requires preserving call
  order, not just the formula.
- Verified ROM image views exist for fixed main-CPU ROM, banked program ROMs,
  sound CPU ROM, and decoder PROM bytes. The MAME-documented main-board and
  sound-board memory maps are embedded under `assets/red-label/memory-map.tsv`
  and checked against the Rust address classifiers. Red-label fixed-bank SRAM
  routine metadata is embedded under `assets/red-label/sram-routines.tsv`,
  `phr6.src` CMOS cell metadata is embedded under
  `assets/red-label/cmos-layout.tsv`, `romc8.src` CMOS default bytes are
  embedded under `assets/red-label/cmos-defaults.tsv`, `defb6.src` bomb shell
  image bytes are embedded under `assets/red-label/shell-images.tsv`,
  `defb6.src` complete and short-form object-picture metadata and image bytes
  are embedded under `assets/red-label/object-pictures.tsv` and
  `assets/red-label/object-images.tsv`, `defa7.src`
  shell/scoring/sound/collision/hyperspace/player-display/table-process
  callback entry points plus object-picture ON/OFF and `CWRIT` / `COFF` entry
  points are embedded under
  `assets/red-label/routine-addresses.tsv`, red-label sound table bytes
  including `APSND`, `ACSND`, `PRHSND`, `SCHSND`, `SWHSND`, `TIHSND`, `LPKSND`,
  `LSKSND`, `LGSND`, `LSHSND`, and `SSHSND` are embedded under
  `assets/red-label/sound-tables.tsv`, the
  complete `defb6.src`
  `SWTAB` switch bit table is embedded under
  `assets/red-label/switch-table.tsv`, and `phr6.src` RAM table/list metadata
  is embedded under `assets/red-label/ram-layout.tsv` and
  `assets/red-label/linked-lists.tsv`, including the RNG seed bytes,
  star-map/thrust/fireball tables, and `ASTCNT` needed by the hyperspace tail.
  The machine runtime initializes those
  RAM tables from the embedded metadata, seeds one-player `START` fields from
  `NSHIP` and `REPLAY`, applies the visible `PLSTR5` player runtime bytes
  (`NPLAD`, `NPLAXC`, `PLAX16`, `PLAY16`, `PLAS`, `PCRAM+5`, and related
  flags), and emits object-table and SPTR-head CRCs in trace rows. It also has
  source-shaped `MKPROC`, `MSPROC`, `SLEEP`, `KILL`, asset-backed `SSCAN`
  switch history over all eight `SWTAB` IN0 bits, translated
  fire/smart-bomb/reverse switch queueing, `SWP` switch-process dispatch, and
  `DISP` primitives for active/free process-list mutation and process timer
  updates, plus source-shaped `OSCAN` / `ISCAN` active/inactive object scanner
  maintenance, source-shaped `BMBOUT` / `FBOUT` shell output writes, and
  source-addressed `OBJCOL` dispatch for those translated callbacks.
  It also translates `SCORE`, the visible RAM effects of `SNDLD`, `BKIL`,
  `LFIRE` entry, `LASR0` / `LASL0` drawing/fizzle/erase/sleep loops, `LASD`
  tail, the RAM-visible part of `LCOL`, source-ordered `FISS` / `STINIT` /
  `FBINIT` / `THINIT` boot table initialization, `COLIDE` / `COL0`
  picture-mask intersection for complete and short-form object-picture
  descriptors, visible `COLCHK` player-collision side effects, the `REV`
  reverse debounce path, the `SBOMB`
  entry/flash/debounce tail, and source-addressed `OCVECT` dispatch to
  translated collision vectors, including `NOKILL`.
  The process dispatcher can now run translated `SBOMB` entry/resume addresses
  and `LFIRE` / `LASR0` / `LASL0` / `HYPER` / `HYP02` / `REV` / `REV2` /
  `REVX1` / `REVX` from `PADDR`, including the guarded `SBMBX2`/`SUCIDE` path
  that does not clear an already-active `SBFLG`, the source `LFIRE`
  fall-through into the first laser loop, the `HYPER`
  entry guard/status/`SCLR1`/sleep sequence, and the visible `HYP02` rematerialization
  slice. `HYP02` now clears `SPTR` shells, copies `SEED`/`HSEED` to
  `BGL`/`BGLX`, updates player direction/position RAM, creates the phony
  player object, starts `APVCT`/`APST` appearance RAM, loads `APSND`, and
  sleeps to `HYP2`. The visible `HYP2` tail now kills the phony object, resets
  `STATUS` through `STCHK`, suicides on the safe path, and exposes the `PLEND`
  branch when `LSEED > 192`. The RAM-visible `EXST` / `EXPU`
  appearance/explosion path now initializes explosion slots, advances `RAMALS`
  slots, restores object pictures when appearances finish/offscreen, clears old
  erase-list blocks, and runs the translated `EWRITE` expanded-object writer
  from embedded picture assets. `KILOFF` now unlinks objects and clears the
  current picture footprint through `OFSHIT`'s map-2 erase path. The
  `PLEND`/`PDEATH` entry and glow-loop slice is now translated through
  `PDTHL`, `PDTH2`, and `PDTH4`, including `PLSAV`, `PDSND`, `MONO` into
  `MONOTB`, `PXCTB` glow color consumption, pseudo-color RAM writes, and the
  sleep into `PDTH5`. The `PDTH5` entry now clears `PCRAM`, runs exact
  `GNCIDE`, enters the bank-7 `PXVCT`/`PX1A` player-explosion setup and frame
  loop from `blk71.src` using the extracted `PXCOL` table, and resumes at the
  ROM-confirmed `PDTH5R` address. The non-wave-end respawn, player-switch, and
  game-over branch decisions now use exact `WVCHK`, `PLE02`, `PLE3`, `PLSTRT`,
  and `ATTR` addresses. The zero-enemy branch now enters the translated
  `BONUS` long-sleep path: it clears active screen RAM through `SCLR1`, writes
  the source `MESS` / `WNBV` wave-complete text and numbers from embedded
  `mess0.src` assets, scores each surviving astronaut, sleeps through `BC1`,
  refreshes wave parameters through `GETWV`, and returns through `BC3` to the
  ROM-confirmed post-bonus `SCLR1` call site.
  The IRQ `PLAYER` motion slice now applies exact 24-bit horizontal damping,
  thrust acceleration from `PIA21`, X/scroll correction, absolute-X
  calculation, and altitude up/down velocity from `PIA31` / `PIA21`.
  The source-shaped `PRDISP` / `POUT` player-picture slice now stores the
  scanline bound in `TEMP48`, gates on `STATUS` / `PLAYC`, clears the old 8x6
  footprint through `OFF86`, copies `NPLAD` / `NPLAXC`, and draws `PLAPIC` /
  `PLBPIC` through embedded `ON86` image bytes. The same source-shaped display
  slice now performs the adjacent `THOUT` / `THOFF` thrust-flame byte writes,
  while `THPROC` advances the fireball/thrust table pointers. `OPROC` now walks
  active objects in a caller-supplied scanline band, clears old descriptor
  footprints, and redraws descriptor pictures with the ROM BGL-relative X and
  alternate-flavor rules. `VELO` now advances active-object positions by
  source velocities and wraps Y through `YMIN` / `YMAX`. The normal and
  inverted IRQ object-band tails now read `LDD XXX1` / `LDD XXX2` source
  bounds from RAM and run `PRDISP`, `OPROC`, `SHELL`, and `VELO` in the exact
  visible order for those tail slices. The scanline object-phase gate now
  applies the source `VERTCT` thresholds, `IFLG` latch, `TIMER` increment,
  normal/flipped watchdog data byte, palette-copy thresholds, the source `PSHU`
  color-mapping copy from `PCRAM` into hardware color RAM, and `XXX2`
  calculations before entering those tails, and it runs translated `PLAYER` /
  `STOUT` pre-tail work on the source branches that call them. Native red-label
  frame rendering now reads the modeled hardware color RAM after that copy.
  When the caller supplies the source IRQ context or an explicit 6809 stack
  pointer, the terrain branch also runs translated `BGOUT`; otherwise it
  records that `BGOUT` is due. The
  pre-tail `SNDSEQ`
  sound-table sequencer now advances `SNDX` / `SNDPRI` / `SNDTMR` / `SNDREP`,
  emits source-shaped main-board sound commands, and handles the thrust sound
  gate. Full frame output and fidelity traces now include the resulting raw
  command bytes. The source `CSCAN` branch now keeps the `PIA01` / `PIA02`
  coin-door history, masks IN2 through `ANDB #$3F`, double-checks the sample,
  and queues the first surviving `SWTAB1` coin/admin switch process. The queued
  coin process path now translates `LCOIN` / `RCOIN` / `CCOIN` debounce/sleep
  handling, `CN1` coin sound loading, and the fixed-bank BCD
  coinage/audit/credit updates. The queued admin switch path now translates
  `HSRES` today's-high-score reset and `ADVSW` manual diagnostics/audit target
  selection, with the diagnostic/audit mode handoff still explicit. The IRQ
  scheduler now records the source `MAPC` clear/select/restore write sequence
  and leaves hardware map selection restored from `MAPCR`. The source IRQ
  `BGOUT` context now derives the saved `SSTACK` value from `HSTK=$BFFF`, the
  12-byte 6809 IRQ frame, and the `JSR` return address. Full frame-level IRQ
  scheduling still needs integration. Initialized live machines now apply
  source `P1SW`, and the source `P1SW` / `P2SW` screen-switch path now writes
  the RAM-visible `IRQHK` hook for upright and cocktail flipped player starts
  before the live renderer chooses the IRQ pass.
  The source `EXEC0` / `EXEC1` pre-`DISP` slice now clears `TIMER`, updates
  `OVCNT` / `STRCNT`, demotes the first overload-eligible active object through
  the source `OFSHIT` / `IPTR` path, selects map 2, runs translated `COLCHK`,
  calls the translated `XUVCT` / `EXPU` expanded-object update, advances
  `RAND`, and drains queued `SWPROC` entries through `SWP`. A source-shaped
  executive iteration wrapper now resets `CRPROC` to `#ACTIVE`, runs that
  pre-dispatch slice, then performs one translated `DISP` scheduler/dispatch
  pass without changing the live trace cadence.
  The `GEXEC` / `GEX0` game-executive slice now initializes `PD`, `UFOTMR`,
  and `WAVTMR`, runs the source `WVCHK` gate, accelerates and decrements
  `UFOTMR`, dispatches `UFOST` and increments `UFOCNT` when the baiter timer
  expires, launches `LANDST` squads from `WAVTMR` / `LNDRES` / `WAVSIZ`, then
  restores `STRCNT` toward 16, advances `GTIME` through the source audit-meter
  wrap, decrements the process `PD` counter, applies the source `WDELT`
  intra-wall update to `ELIST` every 40 passes, and sleeps back to `GEX0`. The
  wave-clear path now sets `STATUS=$77`, runs `GNCIDE` / `PLSAV`, enters the
  shared `BONUS` body with the assembled `GEXBON` return site, increments
  `PLAS`, and restarts through the translated `PLSTR0` handoff. The `GETWV`
  wave parameter path now increments `PWAV`, refreshes `PENEMY` from
  source-order `WVTAB`, applies CMOS difficulty/ceiling inter-wall `WDELT`
  updates, and restores `PTARG` on the `GA1+6` restore-wave cadence. The
  start-flow foundation now extracts the `CREDIT`,
  `CUNITS`, and `BUNITS` base-page fields, translates exact `FPLAY` free-play
  credit seeding from the core CMOS image, and translates the RAM-visible
  `START` power-page gate, first-game player table reset/copy, `PLSTRT`
  process creation, source `SCRCLR` video-RAM clear, and `START2` `PLRCNT`
  increment plus `ADDA #$99` / `DAA` BCD credit decrement and `WCMOSA CREDST`
  packed CMOS credit backup. The source `TDISP` HUD/top-display path now
  dispatches directly and the `START2` helper runs the same side effect when
  `PLRCNT` advances past one, using extracted score digit, mini-ship, and
  smart-bomb image assets for `BORDER`, `LDISP`, `SBDISP`, and `SCRTR0`
  visible RAM writes. The source `SCORES` player-score redraw used by `HALDIS`
  and `LEDRET` now routes through the same score-transfer helper. The source
  `SCORE` tail now reuses `SCRTRN`, `LDISP`, and `SBDISP` so score and replay
  events refresh the live HUD from the same source assets. `ST1` and `ST2`
  now execute through the translated process dispatcher in the source order,
  including status/credit gates, start sounds, one/two `START` calls, and the
  final `DIE` process cleanup. The generic `SUCIDE` / `HYPX` tail now runs
  through the translated
  dispatcher, unlinks the current active process, returns it to the correct
  free list, and rewinds `CRPROC` to the previous link cell.
  Generic/untranslated process body dispatch,
  remaining object callbacks, live respawn orchestration beyond the `PLSTRT`
  runtime snapshot handoff,
  full trace proof for the translated `BONUS` wave-clear death-tail path,
  source-exact boot/start-ready state, broader `SWTAB` cabinet-session
  integration, scanline scheduling, live video presentation, and full
  frame/cycle integration remain gaps. Live coin/admin input now enters through
  translated `CSCAN` / `SWTAB1` / `SWP`, ignores the source auto/manual
  selector for queue priority while preserving it for `ADVSW`, ticks the source
  slam/coin debounce counters from live tilt/coin input, sleeps through
  `LCOIN` / `RCOIN` / `CCOIN`, and awards credit from `CN1` with `CNSND`,
  slot audits, paid-credit audit,
  `CUNITS`/`BUNITS`, and CMOS-backed `CREDIT` / `CREDST` updates. Live
  high-score reset now runs through `HSRES`, live service advance reports the
  translated diagnostics/audits target, and credited/free-play live one- and
  two-player start buttons now scan `SWTAB`, dispatch `ST1`/`ST2`, and consume
  credits through the translated `START2` tail. Live player controls stay gated
  while the active `PLSTRT` / `PLSTR3` / `PLS01` / `PLS1` start handoff
  advances.
  The board harness can now read/write packed CMOS/SRAM bytes and words using
  the documented most-significant-nibble-first order, can apply the ROM-derived
  CMOS defaults to its CMOS cell array through the visible `CMINIT` clear/copy
  sequence, can model the visible `CLRAUD` packed zero writes, can route the
  CMOS-visible `PWRUP` branch around `CMOSCK`/`DIPFLG`/`DIPSW`, can run the
  visible `RHSTD` / `RHSTDS` all-time and today's high-score reset copy, can
  report the `romc0.src` target reached by each `PWRUP` action decision and
  dispatch `AuditGate` into the source-visible `AUDITG` entry screen, can read
  `AUDITG` / `MSGAUD` audit and operator-adjustment rows from their source CMOS
  offsets, can transfer the source-visible `AUDITG` entry screens and `DISAUD`
  row text/erasure, can apply the source-visible `ALTER` /
  `HYSCRE` mutation rules, can step the source-visible `AUDITG` row navigation
  from IN2 service inputs, can model the post-display `AUDITG` debounce
  countdown, can run those pieces as one deterministic audit cycle with
  previous-row erasure, can step the post-`PWRUP` `AUDITG` outer frame path to
  return-to-caller, can compare/insert packed high-score entries, and can
  snapshot source-labeled CMOS and RAM fields. A
  main-board address classifier exists for RAM, banked I/O,
  selected banked program ROM, bank-select writes, and fixed ROM reads. Main
  RAM bytes can now be read and written through a deterministic harness
  surface. Raw write-only palette register bytes, CMOS 4-bit write/read bytes,
  video-control cocktail state, and watchdog reset
  recognition are stored. Video-counter reads expose the MAME `vpos & 0xfc`
  behavior with the `vpos >= 0x100` clamp. The main-board PIA data/control
  register path is modeled for IN0/IN1/IN2 reads and sound-command output
  writes, including DDR filtering and CA/CB edge IRQ flags. MAME's older
  Williams VA11/COUNT240 lines can drive PIA1 CB1/CA1, and the resulting PIA
  IRQ line state is visible to the deterministic harness. The board can expose
  native visible pixel-nibble, palette-index, and RGBA frames from its video
  RAM and palette RAM, and can record source-shaped diagnostic LED output bytes,
  `FLASHL`
  events, CROM0 diagnostic text/palette intent, bitmap
  headline/bad-ROM-row/operator-instruction transfer, and `ADVSW` / `NEXTST`
  gate metadata. It can also render the source-visible RAM-test start,
  failure, and no-error screens reached from the CROM0 handoff, execute the
  source RAM2 pattern fill/verify pass with page-boundary operator-poll
  metadata, route pass-boundary continue/failure/operator-abort loop targets,
  and run the CMOS RAM-test backup/write/verify/restore loop with
  OK/failure/operator-abort routing and source text-control visible outcomes. It
  can also transfer the CROM0
  color-RAM test heading/instructions, draw the source `RAMBAR` vertical bars,
  step the `COLRMT` palette loop, model the CROM0 audio-test heading,
  `PLAYB` pulse, skip-sound table, BCD sound-number display, CROM0
  switch-test heading/display table/PIA scan, CROM0 monitor-test
  heading/crosshatch/RGB-field/color-bar pattern displays, monitor-to-`AUDITG`
  entry transfer, and `AUDITG` title/prompt/row text transfers. CPU interrupt
  scheduling, physical lamp timing, and cycle-accurate sound sample scheduling
  remain gaps.
  Sound-board PIA IC4 data/control behavior exists for port-B command reads and
  port-A DAC writes, and command CB1 updates the PIA IRQ state. There is still
  no exact power-on RAM state, physical advance-switch timing, physical lamp
  timing, default live CMOS path policy, screen scanline scheduler, watchdog
  timing/reset side effects, rendering timing side effects, decoder PROM
  behavior, complete DAC sample output, CPU IRQ scheduling, or all translated
  `VSNDRM1.SRC` waveform routines.

### Player And Controls

- The live player state now mirrors the translated IRQ `PLAYER` motion slice
  for horizontal damping, thrust, X/scroll correction, absolute-X, and altitude
  velocity, plus the translated `PRDISP` / `POUT` player-picture and
  `THOUT` / `THOFF` thrust-flame byte output slice. The translated `OPROC`
  slice can erase and redraw active-object descriptors in caller scanline
  bands. The source `STOUT` / `SBLNK` star-output path is translated over
  `SMAP` and video RAM, but not yet wired into a cycle/scanline scheduler. The
  remaining `defa7.src` /
  `defb6.src` player path still needs scanline scheduling, full live video
  presentation, respawn/start bodies, and carried-human behavior.
- The `REV` switch process is translated through the asset-backed `SWTAB`,
  `SSCAN`/`SWPROC`/`SWP`, and updates `REVFLG`/`NPLAD` with the source
  debounce timing. Live reverse now reaches the translated `PLAYER` and
  `PRDISP` slices so `NPLAD` is committed to rendered/facing `PLADIR` during
  player frames. The `SBOMB` switch
  process also enters through `SWTAB`/`SSCAN`/`SWPROC`/`SWP`. The scanner now
  records fire, thrust, smart bomb, hyperspace, start-one, start-two, reverse,
  and altitude-down bits from the source table, but only fully translated
  live-safe `LFIRE`, `SBOMB`, `HYPER`, and `REV` routine bodies are queued. The
  `PLAYER` IRQ motion slice consumes the recorded thrust and altitude bits for
  source-shaped movement and scroll updates. The `HYPER`
  entry guard/status/`SCLR1`/sleep sequence and visible `HYP02`/`HYP2`
  rematerialization tail are translated for direct process dispatch, and
  `EXPU` now reaches translated `EWRITE` video writes. `KILOFF` object unlink
  and footprint erase is translated. `PLEND`/`PDEATH` is translated through the
  `PDTHL`/`PDTH2` glow loop, `PDTH4` final flash, `PDTH5` entry, bank-7
  `PXVCT`/`PX1A` explosion loop, and non-wave-end respawn/game-over branch
  decisions. `PRDISP` / `POUT` player-picture output now uses embedded
  red-label image and thrust table bytes, and `OPROC` active-object band
  rendering now erases and redraws descriptor pictures with the source Y-band,
  BGL-relative X, width, and alternate-flavor checks. `VELO` active-object
  velocity addition is translated for direct dispatch. `BGI` now selects
  bank/map 7 and runs the translated `BGINIT` terrain-table generator.
  `PLSTRT` live respawn orchestration beyond runtime snapshot sync,
  human-carry behavior, and full IRQ scanline/live rendering integration
  remain open.
- The live keyboard profiles expose operator cabinet bits with `F2` for
  service advance, `F3` for high-score reset, and held `F4` for the
  auto/up selector used with service advance, plus `F5` for the slam/tilt
  switch. They also expose the three coin slots with `5` for left, `6` for
  center, and `7` for right.
- Input profiles and MAME IN0/IN1/IN2 port projection exist, and the main-board
  CPU can now read those bytes through the PIA data/control register path once
  the ROM selects data registers. Cabinet control behavior and initials entry
  still need exact red-label flow.

### Gameplay And Scoring

- Mutant score is corrected to `150` in the new red-label score table, and the
  generic `SCORE` BCD/replay RAM update path is translated. Most gameplay
  callers and full scoring coverage are not implemented.
- Player lasers, enemy shells, mines, smart bombs, Pods, Swarmers, collisions,
  wave-end bonuses, extra stocks, and most side effects still need exact
  translation. The current laser/boot-table slice covers `CRINIT`, `FISS`,
  `STINIT`, `OINIT`, `FBINIT`, and `THINIT` initialization, `LFIRE`
  cap/process-data entry setup, `LASR0` / `LASL0`
  drawing/fizzle/erase/sleep loops, `LASD` `LFLG` decrement plus `SUCIDE`,
  RAM-visible `LCOL` collision setup, exact source `LFIRE` fall-through into the
  first loop, source-shaped `FPLAY` and RAM-visible `START` / `START2`
  credit/free-play/table/process effects, translated `ST1` / `ST2` process
  dispatch, and live fire, smart-bomb, hyperspace, reverse, and start-switch
  paths through red-label `SWTAB`/`SSCAN`/`SWPROC` / `SWP` into the translated
  scheduler. The current
  hyperspace slice covers `HYPER` entry guard/status/`SCLR1`/sleep side effects
  and the visible `HYP02`/`HYP2` rematerialization, `APVCT` appearance-start, and
  RAM-visible `EXST` / `EXPU` appearance/explosion side effects, including
  translated `EWRITE` erase-table and video writes and `KILOFF` footprint erase,
  up to the `PLEND` branch. The `PLAYER` IRQ motion slice now covers
  source-shaped horizontal damping, thrust, X/scroll correction, absolute-X,
  and altitude up/down velocity, and the `PRDISP` / `POUT` player-picture slice
  now covers the 8x6 ship body and `THOUT` / `THOFF` thrust-flame bytes through
  embedded image/table data. The `OPROC` active-object display slice now covers
  old-picture erase, new descriptor picture output, the ROM Y-band rules,
  BGL-relative X bounds, width clipping, and alternate image flavor selection.
  The `VELO` active-object velocity slice now covers source X addition and
  Y wrap through cabinet bounds. Remaining `PLSTRT` live respawn integration,
  golden-trace proof for the
  `BONUS`/`SCLR1` wave-clear tail, full IRQ scanline/live rendering integration,
  and full frame/cycle integration are still scaffold/open. The
  current smart-bomb slice covers the `SBOMB` entry/flash/debounce tail, and
  the current player-death slice covers `PLEND`/`PDEATH`, `PLSAV`, `MONO`,
  `PDTHL`, `PDTH2`, `PXCTB` glow color writes, `PDTH4`, `PDTH5`,
  `PXVCT`/`PX1A`, `PXCOL`, `GNCIDE`, `WVCHK`, non-wave-end respawn or
  game-over decision points, and the translated `BONUS` / `BC1` / `BC2` /
  `BC3` wave-clear tail. The current collision slice covers
  `COLIDE` / `COL0` over complete and short-form object-picture descriptors,
  visible `COLCHK` side effects, and dispatch to translated `OCVECT` routines
  such as `BKIL` and `NOKILL`.
- Wave setup uses source-order `WVTAB` / `WDELT` data and translated `GEXEC` /
  `GEX0` pacing to launch `UFOST` baiters and `LANDST` squads from red-label
  reserve counters. Two-player session state remains a later cabinet-session
  phase.
- Planet destruction, humanoid restoration, human abduction/rescue, and falling
  humanoid/catch behavior are translated as source-owned process slices; live
  whole-game trace proof remains a later fidelity phase.

### Enemy Behavior

- Schizoid/mutant, UFO/baiter, lander, tie-fighter/bomber, probe/pod,
  mini-swarmer, astronaut, falling-astronaut, score-popup, and terrain-blow
  process slices are translated into the new arcade core.
- Object state must preserve raw `phr6.src` object cell fields such as position,
  velocity, picture, color, process state, counters, and shell ownership.
  The translated `GETOB`, `OBINIT`, `KILLOB`, `KILSHL`, `GETSHL`, `OSCAN`, and
  `ISCAN` list/scanner primitives are the first step; shells must remain
  SPTR-linked object cells rather than a separate gameplay-only collection.
- Focused source-backed unit coverage exists for the translated enemy/process
  families. End-to-end MAME trace proof for their interaction remains a later
  fidelity phase once full-frame IRQ scheduling is promoted into the live core.

### Video, Appearance, And Attract Mode

- The MAME Williams screen-memory byte layout, native visible-area pixel-nibble
  and palette-index extraction, RGB palette resistor conversion, and an
  aspect-preserving native RGBA cabinet-frame scaler are implemented.
- The current live loop presents translated video RAM through the native
  cabinet-frame scaler without a synthetic fallback. Untranslated screens stay
  black until source-backed video RAM writes exist. It advances core frames
  from the red-label frame clock before drawing the latest snapshot, so
  terminal rendering no longer controls core frame count. Live playing frames
  now run a source-ordered upright IRQ video slice over translated `PLAYER`,
  `STOUT`, upper `OPROC` / `PRDISP` / `SHELL`, IRQ `BGOUT`, and lower
  `PRDISP` / `OPROC` / `VELO`; `PRDISP` / `POUT`, `OPROC`, shell output,
  `EWRITE`, and player-explosion routines can mutate video RAM directly, and
  `HALLOF` / `HOFIN` now renders the source initials-entry player label,
  hall-of-fame instructions, initials, underlines, and palette copy into video
  RAM. The gameplay `PLE2` game-over
  branch now writes source `GO` text at `0x3E80`; the two-player `PDTH5R`
  handoff writes the current player label at `0x3C78` plus `GO` at `0x3E88`;
  `PLSTR5` writes the two-player current-player start prompt at `0x3C80`;
  source `SCINIT` clears video, resets object lists, zeroes `BGL`/`BGLX`, runs
  `BGI`, reloads `CRTAB`, and sets `STATUS=$DB` plus `PLAXC=$1030`; `PLE3`
  waits through the source handoff before `HALLOF` while applying `ATTR`'s
  map-1 select, and submitted-score sessions render the source-backed
  `HALDIS` hall-of-fame table after source `GNCIDE`; source `CREDS` writes the
  attract credits label and BCD number while maintaining `OCRED` / `ICREDF`.
  Source `AMODES` prepares the Williams-page state, starts `COLR` / `TIECOL`,
  and source `LOGO` expands `DEFENDER`, walks `LGOTAB`, draws Williams-logo
  pixels, switches to the fast pass, and creates `PRES`; `PRES` / `PRES1`
  writes the source `ELECV` `ELECTRONICS INC.` / `PRESENTS` block and starts
  `DEFEND`. `DEFEND` / `DEFENS` now drives the source wordmark appearances,
  `DEF33` / `DEF50` refreshes the whole wordmark, `DEF44` starts `CBOMB`
  before the source `COPYRT` copyright strip and wait gates, direct `COPYRT`
  redraws without starting `CBOMB`, and `WILLIR` / `WILR1` restores the logo
  rate. `DEF51`, `CPR56`, `HALL12`, and `HALD4` now dispatch through their
  source `DEF50`, `HALDIS`, `HALL1` / `HALL13`, and `LEDRET` branch targets.
  `LEDRET` now drives the source instruction-page setup, rescue, enemy table
  loop, laser process, direct `LASR` / `LASL` setup labels, and `TEXTP` /
  `TEXTP2` text cadence from `TEXTAB` / `TENT`; `HOFST`, `HOFBL`, `HOFUD` /
  `HOFUD1`,
  `HALL1` / `HALL3A` / `HALL4` / `HALL5` / `HALL6`, and `HALD3` now tick the
  source initials-entry qualification, screen setup, stall, blink, up/down,
  fire-switch debounce/advance, initials submission, hall-of-fame wait state,
  and live `PCRAM` palette presentation. Live playing frames now select the
  upright `IRQ` or flipped `IRQB` video path from `IRQHK` and run those passes
  through the translated source `VERTCT` / `IFLG` scheduler, including map
  writes, timer/watchdog side effects, palette copy, translated `PLAYER`,
  `STOUT`, upper/lower `OPROC` / `PRDISP` bands, `SHELL`, IRQ `BGOUT`, and
  `VELO`. Cocktail player-start screen switching now runs translated `P1SW` /
  `P2SW` hook setup before that `IRQHK` selection. Idle live attract now starts
  source `ATTR` and follows the immediate power-on `AMODES` / `LOGO` handoff
  before sleeping to `LOGO0`. Full
  hardware scanline cadence, sound-IRQ ownership, color walkers, general text
  beyond the translated message vectors, remaining game-over presentation, and
  exact scanline ownership still need exact translation.
- Embedded PNGs are acceptable only as generated/self-contained deployment
  assets. The verified renderer must be derived from red-label ROM/source data
  and fixture-checked.

### Audio

- The clean-slate tree has raw sound-command trace plumbing and a
  MAME-documented sound-board RAM/PIA/ROM memory surface plus the
  main-board-to-sound-board command latch handoff and sound PIA port-A DAC
  callback boundary. The main-board `SNDSEQ` table sequencer is translated for
  source-shaped `SNDLD` priority/equal-interrupt loading, `SNDOUT`
  idle-plus-command writes, and the thrust sound gate. All embedded sound-table
  bytes can also be expanded into repeated command plans, the embedded
  `sound-table-timelines.tsv` fixture with table labels, addresses, `SNDSEQ`
  tick anchors, terminator pointers, and sequence-end ticks, and the
  source-derived `sound-table-command-sequences.tsv` fixture with each table
  command expanded through `SNDOUT` into idle and complemented command writes,
  plus the source-derived `sound-thrust-command-sequences.tsv` fixture for the
  `SNDS01` thrust-start and `SNDS00` thrust-stop branches, plus
  `sound-direct-command-sequences.tsv` for the direct `PDTH5`, `PLE2`, and
  `LNDFX0` `SNDOUT` callsites. The fixture validators compare those rows
  against the generated data and count timeline command/sequence-end rows plus
  command-sequence idle/command writes. Command CB1 now sets the sound PIA IRQ
  state, and
  the `VSNDRM1.SRC` `SETUP`
  plus IRQ command decoder classify raw command bytes into GWAVE, jump-table
  special, and VARI routine targets while applying the source-visible
  background/spinner/bonus flag gates; normal `IRQ1` GWAVE/VARI commands can
  now run their translated loader and first waveform window from that dispatch
  result, `SP1` can run
  its `CABSHK` `VARILD` setup and first translated `VARI` sweep, `BON2` can
  run its first `BONV` `GWAVE` window or `GEND50` continuation, `LITE` /
  `APPEAR` special commands can run the shared translated `LITEN` stream,
  `BG2INC` can run its source flag update and first `BG2` setup, `BGEND`
  can report its source flag-only result, `BG1` / `THRUST` special commands
  can run their first translated
  `FNOISE` windows, `TURBO` / `CANNON` / `RADIO` / `HYPER` / `SCREAM`
  special commands can run their translated finite DAC streams, `ORGANT` /
  `ORGANN` can report their source organ-flag setup states, and the source
  `IRQ3` background handoff can kill `B2FLG` and run the first translated
  `BG1` / `BG2` continuation step when background flags are active. The `GWLD`
  loader now reads `SVTAB` / `GWVTAB` / `GFRTAB` from the loaded sound ROM and
  populates direct-page GWAVE state through `WVDECA`
  predecay, `VARILD` reads `VVECT` into the VARI direct-page parameters, and
  the `SP1`, `BON2`, and `BG2` pre-loop setup paths, the `GWAVE` / `GPLAY`
  per-period DAC byte stream, the `VARI` / `VSWEEP` per-sweep DAC byte stream,
  the `LITE` / `APPEAR` shared `LITEN` random-complement byte stream, the
  `TURBO` / `NOISE` noise-decay byte stream, the `HYPER` phase-edge DAC byte
  stream, the `BG1` / `THRUST` first-window `FNOISE` byte streams, the
  `CANNON` / `FNOISE` filtered-noise decay byte stream, the `RADIO` / `RADSND`
  timer-table byte stream, the `SCREAM` echo-cascade byte stream, the
  `ORGANN` / `ORGNN1` first-duration `ORGAN` note byte stream, the `ORGNT1` /
  `ORGTAB` organ tune byte streams, the source `IRQ` PIA
  command-read/CB1-clear prelude, the source `IRQ3` background handoff,
  command return/readiness classification, source-shaped `IRQ1`
  command-to-`IRQ3` background step, the top-level source IRQ organ gate, the
  source IRQ organ-continuation gate, the source IRQ prelude-to-flow cycle,
  plus the shared `GEND` / `GEND40` / `GEND50` / `GEND60` / `GEND61` echo and
  frequency-window updates and the source NMI diagnostic checksum-to-VARI
  branch are translated. The tree does not yet include cycle-accurate waveform routine
  scheduling, CPU IRQ scheduling, or golden command-sequence and waveform
  fixtures.
- Sound must be rebuilt as command writes into a translated sound-board state
  machine from `VSNDRM1.SRC`, not as high-level gameplay cues.

### Session, Cabinet State, And Operator Behavior

- Live coin/admin input and one-/two-player start buttons now enter through the
  translated red-label coin/admin and start switch paths. No-credit one-player
  starts are blocked by the source `ST1` credit gate.
- CMOS-backed high-score reset copies all-time and today's tables from
  `DEFALT`, packed table comparison/insertion is modeled, and qualifying
  game-over scores follow the `amode1.src` player-one-then-player-two
  initials-entry order. Today's-greatest qualification gates entry, submitted
  initials write `THSTAB`, all-time CMOS insertion runs when that table also
  qualifies, translated player-death game-over dispatches hand live mode into
  that flow, and live CMOS persistence remains explicit through `--cmos-path`.
  Exact `HALLOF` / `HOFIN` initials screen rendering is backed by red-label
  video RAM, and the submitted-score `HALDIS` hall-of-fame table is now
  source-backed. The source `PLE2`/`PLE3` 40-tick game-over sleep and `HALL13`
  no-entry delay are translated; operator-screen rendering and remaining
  attract presentation remain Phase 7/video-scheduler work.
- The CLI now has `--input-profile`, ROM metadata reporting with CRC-32
  validation, local ROM-set mapping verification, and deterministic Rust trace
  emission, exact local TSV fixture comparison, and an ignored local trace
  fixture directory convention. MAME/source golden trace generation and
  optional SHA file validation still need to be implemented.

### Tests And Verification

- Tests currently validate the new scaffold and translated constants/routines,
  but most do not yet assert red-label equivalence.
- The CLI can compare generated Rust traces with local expected TSV fixtures
  one file at a time or across an ignored local fixture directory. It can also
  list/write/check the required Phase 1 scenario fixtures, and the local MAME
  runner can generate the ignored expected TSV files when user-supplied ROMs
  and MAME are available.
- There are no object-table, shell-table, or pixel-frame golden tests against
  attract/gameplay references, though the trace format can now carry raw table
  and native frame CRC-32 values.
- There are no audio waveform or command-sequence golden tests.
- There is no end-to-end MAME golden regression suite for two-player state,
  coin/start flow, operator settings, or cabinet input profiles.

## Required Compatibility Features

### `xyzzy` Mode

`xyzzy` is not arcade behavior, but it is a supported project feature. It must
be implemented as an explicit overlay so exact arcade behavior remains clean.

Requirements:

- With `xyzzy` disabled, the arcade core must be trace-equivalent to red-label.
- Typing `X`, `Y`, `Z`, `Z`, `Y` toggles `xyzzy`.
- `F` toggles auto-fire only while `xyzzy` is active.
- `G` toggles invincibility only while `xyzzy` is active.
- `xyzzy` may override shot cap, smart bomb inventory, hyperspace failure,
  collision death, and falling-human death only through explicit hooks.
- Every hook must have paired tests: arcade behavior with `xyzzy` off,
  compatibility behavior with `xyzzy` on.

### BBC Micro Planetoid Key Mapping

The Planetoid layout is a keyboard compatibility profile, not game logic.

Required default profile:

- `ENTER` or `1`: start/restart.
- `A`: altitude up.
- `Z`: altitude down.
- `SHIFT`: thrust.
- `SPACE`: reverse.
- `ENTER`: fire during play.
- `TAB`: smart bomb.
- `H`: hyperspace.
- `BACKSPACE`: compatibility delete during high-score entry.
- `Q` or `ESC`: quit application.

Additional profile requirements:

- The core should expose cabinet actions, not key names.
- Add an input profile abstraction, for example `planetoid`, `cabinet`, and
  `test`.
- Planetoid profile remains the default unless the project intentionally
  changes CLI defaults.
- Cabinet profile should map cleanly to the original action set for arcade
  panels or MAME-style keyboards.
- Input tests should assert both profile mapping and arcade action behavior.

## Target Architecture

### Arcade Core

Create a deterministic arcade core independent of rendering and terminal input.
It should model:

- Red-label machine configuration.
- Frame/tick scheduler.
- Main game state, player tables, object tables, shell tables, score tables,
  high-score tables, and process state.
- Fixed-point coordinates and velocities.
- Red-label RNG state and exact read order.
- Cabinet actions as input bits.
- Sound command output.
- Video/sprite/palette output state.

The core should expose a small public API:

- `ArcadeCore::new(config)`.
- `ArcadeCore::reset()`.
- `ArcadeCore::step(input_bits) -> FrameOutputs`.
- `ArcadeCore::snapshot()`.
- `ArcadeCore::restore(snapshot)`.

### Compatibility Layer

Build compatibility features around the arcade core:

- Input profiles map terminal keys into cabinet action bits.
- `xyzzy` modifies input and selected arcade events through explicit overlay
  hooks.
- Local CMOS persistence uses a file-backed storage trait; deterministic
  initials-entry submissions update the all-time CMOS image and the live
  today's-greatest RAM table.
- Terminal rendering consumes arcade video output but does not alter gameplay.

### Renderer

The renderer should consume cabinet video state and produce:

- Native visible frame buffer for verification.
- Scaled Kitty frame for live terminal play.
- Optional debug overlays outside the verified cabinet frame.

The verified cabinet frame should not contain control help, synthetic glyphs,
or non-arcade UI.

The Kitty presentation layer already has a native cabinet-frame handoff that
scales and letterboxes `292x240` Defender RGBA frames without changing pixels.
The runtime now uses that path for translated video RAM and leaves blank or
untranslated screens black instead of substituting scaffold artwork.

### Assets

All extracted data and media used by the shipped game must live under
`assets/` before it is compiled into Rust:

- red-label ROM metadata and CRC expectations;
- gameplay defaults, score tables, high-score seed tables, wave tables, terrain
  bitstreams/tables, object pictures, character/font data, palette data, and
  attract data;
- sound command tables, sound ROM-derived tables, and generated audio fixtures;
- generated PNG, WAV, frame, trace, and schema assets used by the runtime or
  fidelity tooling.

Runtime code should consume these assets through `src/assets.rs`. Verification
tools may inspect local ROM directories, but normal deployment remains copying
the built binary to a new machine.

### Sound

Sound should be modeled as:

- Main-board sound command writes.
- Sound-board state machine.
- Generated mono sample stream.
- Optional live mixer for terminal play.

The gameplay core should not directly play named high-level cues.

## Implementation Plan

### Phase 0: Baseline And Guardrails

1. Keep this spec current as implementation details change.
2. Maintain `docs/fidelity/` for source notes, trace formats, drift logs, and
   test-fixture workflow notes that do not contain copyrighted ROM payloads.
3. Keep `make fidelity` as the local gate for formatting, tests, clippy, local
   trace-fixture checks, and coverage; extend it with more equivalence checks
   as they become available.
4. Document that ROMs are user-supplied and not redistributed.
5. Keep all extracted game data under `assets/` and embed it from there. Do not
   introduce new hardcoded gameplay asset tables in Rust modules.
6. Keep the new game runnable while replacing scaffold behavior with exact
   translations behind stable interfaces.
7. Add a source-citation rule to code review: every arcade-core routine must
   cite a red-label routine/table/trace.
8. Add an `UNKNOWN_BEHAVIOR.md` or `docs/fidelity/gaps.md` log. Unknowns must be
   recorded there rather than guessed in code.
9. Keep whole-project unit-test line coverage at or above 80% at all times, and
   use the new-code coverage gate with a review base ref to require 100%
   coverage for added executable Rust lines.

### Phase 1: Reference Build And Golden Trace Harness

Status: complete for checked-in harness code and the local MAME/source Phase 1
reference oracle. Generated golden traces are local ignored artifacts produced
with user-supplied ROMs and MAME. The current Rust core does not yet exactly
match the gameplay reference traces; exact Rust-current comparisons are kept in
a separate optional fixture directory while later subsystems are translated.

1. Add tooling instructions to build red-label ROMs from `mwenge/defender`.
2. Add optional local configuration for MAME path and ROM directory.
3. Keep the trace schema under `assets/red-label/trace-schema.tsv` as the source
   of truth. Duplicate docs schemas are not checked in; fixtures must either
   embed this header exactly or be generated from it. The schema must be able to
   record:
   - frame number;
   - input bits;
   - MAME IN0, IN1, and IN2 input bytes;
   - RNG bytes;
   - player state;
   - object table;
   - shell table;
   - score/lives/smart bombs;
   - sound commands;
   - video checksum.
4. Produce golden traces for attract boot, start game, first 300 frames,
   firing, thrust/reverse, smart bomb, hyperspace, abduction, death, wave
   advance, planet destruction, and high-score entry.
   - The required scenario names, frame counts, and compact input programs live
     in `assets/red-label/trace-scenarios.tsv`.
   - The gameplay scenarios seed MAME NVRAM from `romc8.src` `DEFALT`, wait for
     the source-shaped boot/attract window, then hold coin and delayed
     one-player start long enough for the MAME-observed `CSCAN` / `SSCAN` /
     `ST1` path.
   - `make reference-inputs` writes the expanded `*.inputs.txt` files.
   - `make reference-traces` invokes MAME through
     `tools/generate_reference_traces.py` and `tools/mame_defender_trace.lua`
     to produce local ignored `*.expected.tsv` files.
   - `make reference-fixtures-check` validates that every required expected
     trace uses the checked-in schema, has the expected frame count, and
     contains the source-backed evidence markers from
     `assets/red-label/trace-requirements.tsv`.
5. Add tests that compare Rust trace output to golden traces when local
   fixtures are available, using `--fidelity-check-trace` for exact TSV
   comparison where practical, and `--fidelity-check-trace-dir` for local
   fixture directories that skip gracefully when fixtures are absent.
6. Add ignored failing tests for known unknowns before translating each
   subsystem. Unignore each test when the exact source behavior is implemented.

### Phase 2: Core State Replacement

Status: complete for the core-state replacement layer. The table-backed RAM
image now initializes
`PINIT`/`OINIT`-style process, super-process, and object free lists,
one-player `START` table fields, trace-visible object/shell CRCs, and
source-shaped `MKPROC`/`MSPROC`/`SLEEP`/`KILL` process-list mutation,
`GETOB`/`OBINIT`/`KILLOB`/`KILSHL` object-list mutation, `OSCAN` / `ISCAN`
active/inactive object scanner maintenance, and `GETSHL` shell-list allocation
plus `SHSCAN` shell timer cleanup and the `SHELL` movement/death/dead-erase
front half plus `BMBOUT` / `FBOUT` shell output byte writes and `OBJCOL`
dispatch for those translated callbacks, plus `SCORE`, the visible RAM effects
of `SNDLD`, `BKIL`, `LFIRE` entry, `LASR0` / `LASL0`
drawing/fizzle/erase/sleep loops, `LASD` tail, RAM-visible `LCOL`, `COLIDE` /
`COL0` picture-mask intersection for complete and short-form object-picture
descriptors, visible `COLCHK` player-collision side effects, `SBOMB`
entry/flash/debounce behavior, `REV` reverse debounce behavior, and `OCVECT`
dispatch to translated collision vectors, plus source-ordered `FISS` /
`STINIT` / `FBINIT` / `THINIT` boot table initialization. The player-death tail
now reaches `PDTH5`, exact `GNCIDE`,
the bank-7 `PXVCT` / `PX1A` explosion loop, `PXCOL`, `PDTH5R`, and non-wave-end
respawn/game-over branch decisions. The dispatcher can now run translated
`SBOMB` entry/resume addresses, `LFIRE` / `LASR0` / `LASL0` / `REV` resume
addresses, and the translated player-death resume addresses from `PADDR`,
including guarded `SBMBX2`/`SUCIDE`. Deterministic restore now writes the
observable snapshot state back into the source-owned red-label RAM fields for
credits, current player pointer, player scores, wave/lives/smart bombs, player
motion, facing, and RNG seeds, and full save-state restore now round-trips the
RAM/CMOS/palette/hardware-map image plus trace scheduler state. Remaining work
after this phase is scheduler, video, player, enemy, session, and audio
integration tracked by the later phases rather than more core-state
replacement.

1. Continue expanding `src/machine.rs` or split it into `arcade_core` modules
   once table/process size justifies it.
2. Build table-backed state accessors over `assets/red-label/ram-layout.tsv`
   and `assets/red-label/linked-lists.tsv`; any typed wrappers must preserve the
   exact underlying red-label bytes and offsets.
3. Port red-label state initialization exactly, including default high scores,
   players, credits, wave, humans, terrain, and timers.
4. Port `RAND` and prove byte-for-byte sequence equivalence.
5. Replace coarse entity storage with table-backed state in the new core.
6. Add snapshot/restore for deterministic tests.
7. For every copied constant, include its red-label source table or ROM trace.
8. Remove any scaffold state once the exact table-backed field exists.

### Phase 3: Timing And Scheduler

Status: complete for the translated red-label scheduler layer. Source-shaped
`MKPROC`, `MSPROC`, `SLEEP`, `KILL`, and
`DISP` primitives can allocate regular/super processes, splice them through
`CRPROC`, delay the current process, return killed cells to the right free list,
walk the active-process list, decrement `PTIME`, write `CRPROC`, and return
`PADDR` for the due process. The executive wrapper now runs the source
`EXEC0`/`EXEC1` pre-dispatch slice and keeps walking `DISP` in the same pass
after translated `SLEEP` and `SUCIDE` tails resume through the ROM's `DISP2`
link, preserving same-frame process order for sleeping and killed cells. Body
dispatch exists for the translated
`LFIRE`/laser-loop, `SBOMB`, `REV`, `SCPROC`/`SCP1`/`SCP2`,
`PLSTRT`/`PLST1A`/`PLSTR3`/`PLS01`/`PLS1`, `COLR`/`COLRLP`,
`FLPUP`/`FLP2`, `CBOMB`/`CBMB1`, `TIECOL`/`TIECL`, and
player-death/`PX1A` slices. The scanner process now runs the translated
`ISCAN`, `OSCAN`, `SHSCAN`, and bank-1 `SCNRV` scanner-raster stages with
exact source `NAP` resume addresses. The player-start process now runs the
RAM-visible `PINIT`, `PLSTR5`,
support-process creation, `PLS01`, and `PLS1` `PLRES`
astronaut-process/target-list/enemy-runtime restore plus the
scheduled `ASTRO` target-list walker, the `ASTKIL` astronaut explosion path,
and source `TERBLO` terrain-blow process on the final astronaut, including
`BGERAS`, scanner-terrain `STETAB` erase, `TEREX` explosion passes, `TBL3` /
`TBL4` sleeps, `OVCNT`, `COLTAB`, `AHSND`, and final `TBSND` / `SUCIDE`,
schizoid-reserve `SCZST` restore from copied `SCZRES`, the `SCZ0`
movement/shot-timer process slice through shared `SHOOT`, the `SCZKIL`
`KILP` score/explosion/sound path, `UFOST` dispatcher UFO start / `UFOLP`
process loop,
`UFOKIL`, normal `LKILL`, `LANDST` lander start/`GTARG` target selection,
`LANDS0` orbit/`LSHOT`, `LANDG` capture, `LANDF` flee, `LNDFXA` pull-in,
`SCZ00` mutant conversion, and kidnapping `LKIL1` / `AKIL1` / `AFALL` /
`AFALL2` passenger-release, catch, and falling/caught-astronaut paths,
including `P250` / `P500` rescue score popups and `P503` cleanup,
probe-reserve `PRBST` restore from copied `PRBRES`, the `PRBKIL` `KILO` /
`RMAX` / `MMSW` mini-swarmer spawn path,
the `MSWKIL` `KILOFF` / `KILLOP` / `XSVCT` score/sound path, the
`MSWM` / `MSWLP` mini-swarmer process loop including vertical
acceleration/damping, delayed horizontal turnback, `SWBMB` fireball shell
allocation, and `SWSSND` sound loading, tie-fighter reserve `TIEST` restore from
copied `TIERES`, the `TIE` process image/vertical/cruise slice, `BOMBST`
bomb-shell allocation and lifetime path, plus the status/PDFLG path, with
machine-level `INIT20` `CRINIT` / `FISS` / `STINIT` / `OINIT` / `FBINIT` /
`THINIT` refresh. `PLRES` mini-swarmer reserve restore now models the source
`RSW0` phony-object placement, B-register `OX16` low-byte flow, `MMSW` batches
of six, `SWMRES` decrement, and `OFREE` return/reuse path. Broader process body
coverage, full hardware IRQ scanline/frame integration, and MAME golden-trace
equivalence remain tracked by the later gameplay, video, and fidelity phases.

1. Done: keep terminal presentation outside the verified cabinet frame
   while the live loop advances due core frames from `FRAME_RATE_MILLIHZ`
   independently of draw calls.
2. Done: port red-label process scheduling and delay semantics, including
   same-pass `DISP2` continuation after `SLEEP` and `SUCIDE`.
3. Done at unit level for translated scheduler order; MAME process/object CRC
   equivalence remains a later full-fidelity gate once more process bodies and
   frame IRQ integration are in place.
4. Done: preserve compatibility frame pacing only in the renderer/application
   layer.
5. Done: do not convert cycle/frame values by feel. Use source constants, MAME driver
   metadata, or measured trace timing.

### Phase 4: Player, Inputs, And `xyzzy`

Status: complete for the player/input/`xyzzy` phase boundary; remaining world,
full-frame IRQ, and golden-trace acceptance work is tracked by later phases.
`LFIRE`
entry/fall-through,
`LASR0` / `LASL0`, `LASD`, RAM-visible `LCOL`, `CRINIT`, `FISS`, `STINIT`,
`OINIT`, `FBINIT`, `THINIT`, `STOUT` / `SBLNK`, the `GEXEC`
`STRCNT`/`GTIME`/`WDELT` tail
slice, the `GETWV` wave-parameter path, the `PLSTR5` player start RAM slice,
and the live fire, smart-bomb, and reverse paths through asset-backed
`SWTAB`/`SSCAN`/`SWPROC`/`SWP` are translated. `PLEND` / `PDEATH` is translated
through the `PDTHL`/`PDTH2` glow loop, `PDTH4`, `PDTH5`, the `PXVCT` / `PX1A`
explosion loop, and the non-wave-end respawn/game-over branch decisions. The
`PLAYER` IRQ motion slice is translated for horizontal damping, thrust,
X/scroll correction, absolute-X, and altitude up/down velocity. The `PRDISP` /
`POUT` player-picture slice is translated for the 8x6 ship body and `THOUT` /
`THOFF` thrust-flame byte writes, with source boot table initialization and
`THPROC` table advance translated. The live player frame now runs translated
`PLAYER` and translated `PRDISP` over the current player display band, so
source `REV` changes to `NPLAD` are committed into rendered/facing `PLADIR`
during live play. `OPROC` active-object band erase/write is
translated for descriptor pictures, Y-band gating, BGL-relative X bounds, and
alternate image flavor selection. `VELO` active-object position updates are
translated for velocity addition and Y wrap. The normal/inverted IRQ
object-band tails are translated for the source ordering around `PRDISP`,
`OPROC`, `SHELL`, and `VELO`, using `XXX1`/`XXX2`/`XXX3` bounds from RAM. The
IRQ object-phase gate now applies source `VERTCT` thresholds, `IFLG`, `TIMER`,
watchdog data-byte reporting, palette-copy thresholds, the source `PSHU`
color-mapping copy from `PCRAM` into hardware color RAM, and the normal/flipped
`XXX2` calculations, and runs translated `PLAYER` / `STOUT` pre-tail work on
the branches that call them. Native red-label frame rendering uses that modeled
hardware color RAM. It can also run translated `BGOUT` when the caller supplies
the live 6809 stack pointer. The source `CSCAN` branch now keeps the `PIA01` /
`PIA02` history and queues source `SWTAB1` coin/admin switch processes from
caller-supplied IN2. The queued coin process path translates
`LCOIN` / `RCOIN` / `CCOIN`, `CN1`, `CNSND`, fixed-bank coinage, slot audits,
paid-credit audit, and `CUNITS`/`BUNITS`/`CREDIT` updates. The queued admin
switch path translates `HSRES` today's-high-score reset and `ADVSW` manual
diagnostics/audit target selection, with the diagnostic/audit mode handoff
still explicit. The IRQ scheduler records the source `MAPC`
clear/select/restore write sequence and restores hardware map selection from
`MAPCR`. The source IRQ `BGOUT` context now derives the saved `SSTACK` value
from `HSTK=$BFFF`, the 12-byte 6809 IRQ frame, and the `JSR` return address.
Full hardware IRQ integration still needs full frame-level IRQ scheduling. The `SCPROC`
scanner
maintenance process is translated through `ISCAN`, `OSCAN`, `SHSCAN`, and
`SCNRV` with exact
`SCP1`/`SCP2` sleep cadence, including `SETAB` / `SETEND` object/player blip
erase state, `STETAB` terrain erase state, and embedded `MTERR` mini-terrain
records. `PLSTRT` is translated through the RAM-visible process/list
handoff, current-player runtime initialization, support-process creation,
`PLS01`, `PLS1` `PLRES` astronaut-process/target-list/enemy-runtime restore,
the scheduled `ASTRO` target-list walker against `ALTTBL`, the `ASTKIL`
`ASTCLR` / `KILOFF` / `XSVCT` astronaut explosion path, and the source `TERBLO`
terrain-blow process when the final astronaut is removed, schizoid-reserve `SCZST`
restore from copied `SCZRES`, the `SCZ0`
movement/shot-timer process slice through shared `SHOOT`, the `SCZKIL`
`KILP` score/explosion/sound path, `UFOST` dispatcher UFO start / `UFOLP`
process loop,
`UFOKIL`, normal `LKILL`, `LANDST` lander start/`GTARG` target selection,
`LANDS0` orbit/`LSHOT`, `LANDG` capture, `LANDF` flee, `LNDFXA` pull-in,
`SCZ00` mutant conversion, and kidnapping `LKIL1` / `AKIL1` / `AFALL` /
`AFALL2` passenger-release, catch, and falling/caught-astronaut paths,
including `P250` / `P500` rescue score popups and `P503` cleanup,
probe-reserve `PRBST` restore from copied `PRBRES`, the `PRBKIL` / `MMSW`
mini-swarmer spawn path, the `MSWM` / `MSWLP`
mini-swarmer process loop and `SWBMB` shot path, tie-fighter reserve `TIEST`
restore from copied `TIERES`, the `TIE` process image/vertical/cruise slice,
`BOMBST` bomb-shell allocation and lifetime path, the `TIEKIL` `KILO`
squad-slot and super-process cleanup path, `STCHK`/`PDFLG` tail, and
machine-level
`INIT20` `CRINIT` / `FISS` / `STINIT` / `OINIT` / `FBINIT` / `THINIT`
refresh. `PLSTR5` now selects bank 7, translates `ALINIT` / `BGALT` by
expanding the ROM `TDATA` bitstream into the source `ALTTBL` altitude table,
and translates `BGINIT` by filling the mirrored `TERTF0` / `TERTF1` terrain
tables from `TDATA` while zeroing `STBL`. `BGOUT` now rolls those terrain
tables to `BGL` and draws the selected flavor through `STBL` when the caller
supplies the live 6809 stack pointer, and `BGERAS` erases terrain screen words
through the source `STBL` table. `COLR` / `COLRLP`, `FLPUP` / `FLP2`,
`CBOMB` / `CBMB1`, and `TIECOL` / `TIECL` now run as translated
support-process bodies using embedded `COLTAB` / `TCTAB` assets. Translated
`PLSTRT` runtime dispatch now syncs the live snapshot's current player, wave,
lives, smart bombs, and player motion from red-label RAM. The `BONUS`
wave-clear death tail reached through `PDTH5` now writes the `MESS` / `WNBV`
bonus text and numbers, scores survivor icons through `BC1`, refreshes the
next wave through `GETWV`, and returns through `BC3` to `PDTH5SCLR`. Full
`GEX0` wave-clear completion uses the same `BONUS` body with the assembled
post-call `GEXBON` return site, then increments `PLAS` and jumps through the
translated `PLSTR0` restart. Remaining enemy kill/collision vectors
beyond
`BKIL` / `NOKILL` / `ASTKIL` / `MSWKIL` / `PRBKIL` / `SCZKIL` / `UFOKIL` /
`LKILL` / `LKIL1` / `TIEKIL`, full IRQ scanline/live video integration, full
frame/cycle integration, and golden-trace proof for the translated death tail
remain open.
The assembled/ROM-confirmed addresses for the remaining `PLSTRT` / `PLRES`
companion labels (`SCLR1`, `ASTST`, `ASTRO`, `ASTKIL`, `PRBST`, `PRBKIL`,
`MMSW`, `MSWM`, `MSWLP`, `SWBMB`, `MSWKIL`, `SHOOT`, `SCZST`, `SCZ0`,
`SCZKIL`, `UFOST`, `UFOLP`, `UFOKIL`, `LKIL1`, `LKILL`, `AFALL`, `AFALL2`,
`P250`, `P500`, `P503`, `TERBLO`, `TBL3`, `TBL4`, `TIEST`, `TIE`, `TIEKIL`)
are embedded under
`assets/red-label/routine-addresses.tsv`; `BONUS`
continuation labels
(`BC1`, `BC2`, `BC3`, `GETWV`), `PDTH5SCLR`, and the assembled `GEXBON`
post-call return site are embedded there as well.
The next `PLSTRT`
work is to
translate the remaining routine bodies from source or ROM traces, not to infer
their behavior from labels.

1. Done: define cabinet input bits and action names.
2. Done: add `planetoid`, `cabinet`, and deterministic `test` input profiles.
3. Done for Phase 4: live reverse movement/rendering now reaches translated
   `PLAYER` and `PRDISP`; respawn, human-carry behavior, and full scanline
   frame scheduling remain in the later world/video/fidelity phases.
4. Done: Mutant score is `150` in the arcade core.
5. Done: `xyzzy` is an overlay with explicit state and tests.
6. Done: high-level `SessionState` remains orchestration around the core.
7. Done: Planetoid key mapping remains in input-profile code only.
8. Done for current hooks: `xyzzy` smart-bomb/auto-fire behavior has paired
   disabled/enabled coverage; future hooks must keep that paired-test rule.

### Phase 5: World, Waves, Humans, And Enemies

Status: complete for the world/wave/human/enemy translation boundary; full
frame-level IRQ scheduling, complete cabinet session flow, and end-to-end MAME
trace acceptance remain in later phases.

1. Done: exact terrain state now covers live `BGOUT` scheduling, destroyed-planet
   `TERBLO`, scanner mini-terrain `MTERR`, `TDATA`, `ALINIT`, `BGINIT`,
   RAM-level `BGOUT`, and `BGERAS`.
2. Done: wave setup and launch use red-label `WVTAB`, `WDELT`, `GEXEC` /
   `GEX0`, `UFOST`, and `LANDST` paths instead of hand-placed opener slots.
3. Done: Landers and human abduction/carry/release/catch/fall behavior are
   translated.
4. Done: Mutants are represented by the translated `SCZ` process family,
   including conversion after failed lander capture and planet-destroyed flow.
5. Done: bomber/tie wave-launch, bomb-shell allocation, minefield-like bomb
   shell output, and collision behavior are translated.
6. Done: Pods/probes and Swarmer burst behavior are translated through
   `PRBST`, `PRBKIL`, `MMSW`, `MSWM`, `MSWLP`, `SWBMB`, and `MSWKIL`.
7. Done: Baiter/UFO spawn, pursuit velocity, firing, picture cycling, and
   lifetime process behavior are translated through `UFOST` / `UFOLP`.
8. Done: shell rendering/output and collision behavior for the translated
   families uses source shell cells and `BMBOUT` / `FBOUT` / `BKIL`.
9. Done at unit level for the translated enemy subsystems; end-to-end MAME
   trace proof remains in the final fidelity phase.
10. Done: Phase 5 uses red-label routines/tables and does not port archived
    `oldsrc/src/game.rs` behavior.

### Phase 6: Scoring, Players, And Cabinet Session

1. Done: one-player and two-player start/session flow enters the translated
   `ST1` / `ST2`, `START`, `START2`, `PLSTRT`, `PLE02`, and `PLE3` paths.
2. Done: credits and start buttons run through translated coin/start switch
   paths with source BCD credit and CMOS backup behavior.
3. Done: P1/P2 scores, lives, smart bombs, current player, and alternating
   turns are RAM-backed and covered by translated session tests.
4. Done: exact scoring for translated enemies, bullets, mines, humans, rescued
   humans, Pods, Swarmers, wave-end humanoid bonuses, extra ships, and smart
   bombs is source-table backed with regression tests.
5. Done for cabinet-session logic: high-score entry follows `amode1.src`
   player-one/player-two order, today's-greatest qualification, all-time CMOS
   insertion, deterministic initials submission, and source-backed
   `HALLOF` / `HOFIN` initials-entry video RAM writes, gameplay `PLE2`
   `GAME OVER` text write, two-player `PDTH5R` player/game-over labels, and
   two-player `PLSTR5` start prompt. The submitted-score `HALDIS`
   hall-of-fame table is source-backed, and the source `PLE3` / `HALL13`
   handoff delays are modeled with `ATTR` map selection and `HALDIS` entry
   `GNCIDE`; source `CREDS` attract credit display, `AMODES` Williams-page
   setup, `LOGO` table drawing through the first-pass `PRES` handoff,
   `PRES` / `PRES1` `ELECV` redraw, `DEFEND` / `DEFENS` wordmark
   appearances, `DEF33` / `DEF50` whole-wordmark refresh, source-shaped
   `DEF44` / direct `COPYRT` copyright wait, `DEF51`, `WILLIR` / `WILR1`
   fast-logo restore, `LEDRET`
   instruction-page setup/rescue/enemy-table/text flow through direct `LASR` /
   `LASL` and `TEXTP2`,
   `HOFST` / `HOFBL` / `HOFUD` / `HOFUD1`, `ATTR` / `HALLOF` entry setup,
   `HALL1` / `HALL3A` / `HALL4` / `HALL5` / `HALL6` initials-entry support
   ticks, and `HALD3` hall-of-fame wait are translated. Remaining
   boot/game-over
   presentation stays in Phase 7.
6. Done: live CMOS persistence remains explicit via `--cmos-path`; there is no
   implicit platform default path.
7. Done at data/model level: red-label defaults, audit adjustments, coinage,
   free play, replay, and game adjustments are embedded and used by translated
   paths; full operator-screen presentation remains in Phase 7/final scheduler
   work.
8. Done: corrected score regressions include negative coverage so old prototype
   mistakes, such as Mutant `250`, cannot return.

### Phase 7: Video And Appearance

1. Define the native cabinet frame buffer and visible crop used by verification.
2. Extend translated video-RAM presenter coverage with pixel fixtures.
3. Replace synthetic live terrain drawing with the cabinet terrain path.
4. Generate or decode every gameplay object from red-label data.
5. Preserve per-object `OPICT`, animation, palette, and color state.
6. Continue integrating translated `SCNRV` scanner drawing and IRQ scheduler
   timing into full-frame live video presentation and pixel fixtures.
7. Implement HUD, scores, lives, smart bomb icons, player-two fields, game-over,
   high-score, and initials screens from cabinet routines.
8. Run the red-label attract process instead of a reconstructed beat script, or
   port it with trace equivalence.
9. Keep the Kitty renderer as a scaler/presenter over the native verified
   frame buffer.
10. Add pixel checksum and perceptual diff tests for key frames.
11. Do not ship hand-placed visual approximations inside the verified cabinet
    frame. Temporary visuals must be labeled as scaffolding.

### Phase 8: Audio

1. Continue replacing high-level gameplay cue dispatch with source-shaped sound
   command writes.
2. Extend the sound-board state beyond setup, IRQ command decoding, normal
   `IRQ1` GWAVE/VARI and
   `LITE`/`APPEAR`/`TURBO`/`CANNON`/`RADIO`/`HYPER`/`SCREAM` command-flow
   execution, `GWLD` / `VARILD` table loading, `SP1` / `BON2` / `BG2` setup,
   `GWAVE` / `GPLAY`
   period-byte extraction, `VARI` / `VSWEEP` sweep-byte extraction, `TURBO` /
   `NOISE` noise-decay
   extraction, `BG1` / `THRUST` first-window `FNOISE` extraction, `CANNON` /
   `FNOISE` filtered-noise decay extraction, `RADIO` / `RADSND` timer-table
   extraction, `HYPER` phase-edge extraction, `SCREAM` echo-cascade
   extraction, `ORGANN` / `ORGNN1` first-duration `ORGAN` note extraction,
   `ORGNT1` / `ORGTAB` organ tune extraction, the source `IRQ3` background
   handoff, and the source IRQ organ-continuation gate into cycle-scheduled
   translated routine execution.
3. Port remaining `VSNDRM1.SRC` routines needed by Defender.
4. Verify command sequences against red-label traces.
5. Add waveform tests with tolerance for deterministic generated buffers.
6. Keep `--mute` as an output-layer switch only.
7. Do not trigger sound from semantic Rust events unless that event is proven to
   correspond to a red-label sound command write.

### Phase 9: CLI, Tooling, And Developer Workflow

1. Keep these CLI options tested and documented:
   - `--input-profile planetoid|cabinet|test`;
   - `--mute`;
   - `--rom-report [PATH]`;
   - `--verify-roms PATH`;
   - `--fidelity-trace [FRAMES]` for local development;
   - `--fidelity-trace-inputs SCRIPT` for scripted cabinet input traces;
   - `--fidelity-trace-inputs-file PATH` for file-backed input scripts;
   - `--fidelity-check-trace INPUTS_PATH EXPECTED_TSV` for exact local TSV
     fixture comparison;
   - `--fidelity-check-trace-dir PATH` for exact local TSV fixture-directory
     comparison.
2. Keep ROM reporting validating CRC-32 for the red-label set; add SHA
   validation only if a source fixture requires it.
3. Keep `cargo run` useful for live terminal play.
4. Keep `make fidelity` green; add `make frames` and `make audio-fixtures` once
   the corresponding harnesses exist.
5. Document all required external tools and optional local ROM paths.
6. Make verification tools optional. The deployed game binary must remain
   self-contained.

### Phase 10: Decommission Prototype Paths

1. Remove or quarantine coarse-grid gameplay once the arcade core is complete.
2. Keep legacy renderer/demo helpers only if they consume the arcade core.
3. Remove synthetic cabinet-screen text and overlays from verified frames.
4. Ensure `xyzzy` and Planetoid controls live only in compatibility modules.
5. Update `README.md` to describe exact arcade mode, compatibility features,
   and verification status.
6. Delete or mark obsolete any scaffold routine that cannot cite red-label
   source behavior.

## Acceptance Checklist

- `xyzzy` disabled: Rust traces match red-label traces for the accepted golden
  scenarios.
- `xyzzy` enabled: only documented overlay behavior differs.
- Planetoid profile maps to cabinet actions without changing arcade core
  semantics.
- Mutant score is `150` in live gameplay.
- Gameplay runs at cabinet cadence internally.
- Native video frames match reference frames within accepted tolerance.
- Sound command traces and generated audio pass fixture tests.
- One-player and two-player flows work.
- Coin/start/high-score/session behavior matches red-label defaults.
- `cargo test --all-targets` passes.
- `cargo clippy --all-targets -- -D warnings` passes.
- Whole-project unit-test line coverage is at least 80%, and `make coverage`
  enforces coverage for added executable Rust lines against
  `NEW_CODE_COVERAGE_BASE`, which defaults to `HEAD` for local dirty worktrees.
- Every arcade-core routine cites a red-label source/trace, or is listed as an
  unknown gap with an ignored/failing fidelity test.
- `README.md` and this spec agree on current behavior and remaining gaps.
