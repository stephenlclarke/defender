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
keeps live presentation explicitly scaffolded until video RAM output is proven
against fixtures:

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
- `src/rom.rs` reads embedded ROM metadata and builds verified addressable ROM
  images for the main CPU, banked program ROMs, sound CPU ROM, and decoder PROMs
  for optional local verification and future extraction.
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
- `src/board.rs` now models the source-visible `DISAUD` audit row stack buffer:
  row numbers, packed CMOS values, the replay row's dummy trailing zeroes, and
  `MSGAUD` messages land in the same 31 visible character columns.
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
  action, and `0x25` runs the translated high-score reset copy.
- `src/board.rs` now models the visible `RHSTD` / `RHSTDS` reset copy:
  the first 48 `DEFALT` bytes are copied back into the all-time `CRHSTD` CMOS
  range and into the `THSTAB` "today's greatest" RAM table using the same
  two-cell packed format.
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
  `SCORE`, the visible RAM effects of `SNDLD`, `BKIL`, `LFIRE` entry,
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
- `make fidelity` exists and runs formatting, tests, clippy, and the 80% line
  coverage gate. This is no longer a placeholder.
- The live path still calls `render_scaffold`, which matches the documented
  status that visible gameplay is not yet red-label video RAM output.

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
  `0xDE79`, `GETWV` `0xDE7C`, and `PDTH5SCLR` `0xDADB`.
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
- `assets/red-label/high-scores.tsv` is parsed as a seed table and
  `assets/red-label/cmos-defaults.tsv` records the matching ROM default CMOS
  bytes, but there is no persistence model, initials entry, or today's-greatest
  flow yet. Remaining work: trace the red-label high-score and CMOS routines
  before implementing persistence.
- The current deterministic trace compares Rust output to local expected TSV,
  `docs/fidelity/fixtures/` defines the ignored local fixture layout, and
  Phase 1 now has a local MAME/source trace runner plus a complete scenario
  manifest. Generated golden TSV files remain local artifacts, not checked-in
  source.
- `src/machine.rs` now backs the core trace CRC columns with red-label RAM
  bytes instead of placeholder `None` values: object-table CRCs come from the
  initialized object cells, and shell CRCs come from the SPTR shell-list head.
- The board and sound surfaces model useful MAME-documented boundaries, but
  they are not yet integrated into `ArcadeMachine::step`. The game core still
  advances most scaffold state directly instead of executing translated
  red-label routines against cabinet memory.

### Next Work Order

1. Run the Phase 1 local MAME runner against a user-supplied red-label ROM set
   to populate `docs/fidelity/fixtures/local/`, then use those fixtures as the
   acceptance gate for each translated subsystem.
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
5. Drive native video RAM from translated routines and switch live rendering
   from `render_scaffold` to `render_cabinet_frame` only after trace or pixel
   fixtures prove the frame source.
6. Port the Williams sound-board interrupt dispatch and DAC path from
   `VSNDRM1.SRC`, then add command-sequence and waveform fixtures.
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
- The live loop now uses a cabinet-rate frame duration, but the core does not
  yet execute red-label process timing or CPU-cycle-level behavior.
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
  normal/flipped watchdog data byte, palette-copy due tests, and `XXX2`
  calculations before entering those tails, and it runs translated `PLAYER` /
  `STOUT` pre-tail work on the source branches that call them. When the caller
  supplies the live 6809 stack pointer, the terrain branch also runs translated
  `BGOUT`; otherwise it records that `BGOUT` is due. The remaining `SNDSEQ`,
  `CSCAN`, palette copy side effects, live stack-context wiring, and hardware-map
  restoration still need full scheduler integration.
  The `GEXEC` tail slice
  now restores `STRCNT` toward 16, advances `GTIME` through the source
  audit-meter wrap, decrements the process `PD` counter, and applies the
  source `WDELT` intra-wall update to `ELIST` every 40 passes. The `GETWV`
  wave parameter path now increments `PWAV`, refreshes `PENEMY` from
  source-order `WVTAB`, applies CMOS difficulty/ceiling inter-wall `WDELT`
  updates, and restores `PTARG` on the `GA1+6` restore-wave cadence. The
  start-flow foundation now extracts the `CREDIT`,
  `CUNITS`, and `BUNITS` base-page fields, translates exact `FPLAY` free-play
  credit seeding from the core CMOS image, and translates the RAM-visible
  `START` power-page gate, first-game player table reset/copy, `PLSTRT`
  process creation, source `SCRCLR` video-RAM clear, and `START2` `PLRCNT`
  increment plus `ADDA #$99` / `DAA` BCD credit decrement and `WCMOSA CREDST`
  packed CMOS credit backup. The `START2` helper now runs the source `TDISP`
  side effect when `PLRCNT` advances past one, using extracted score digit,
  mini-ship, and smart-bomb image assets for `BORDER`, `LDISP`, `SBDISP`, and
  `SCRTR0` visible RAM writes. `ST1` and `ST2` now execute through the
  translated process dispatcher in the source order, including status/credit
  gates, start sounds, one/two `START` calls, and the final `DIE` process
  cleanup.
  Generic/untranslated process body dispatch,
  remaining object callbacks, live respawn integration after `PLSTRT`,
  full trace proof for the translated `BONUS` wave-clear death-tail path,
  `SWTAB`
  cabinet-session integration beyond the translated start-switch process
  bodies, scanline scheduling, live video presentation, and full frame/cycle
  integration remain gaps.
  The board harness can now read/write packed CMOS/SRAM bytes and words using
  the documented most-significant-nibble-first order, can apply the ROM-derived
  CMOS defaults to its CMOS cell array through the visible `CMINIT` clear/copy
  sequence, can model the visible `CLRAUD` packed zero writes, can route the
  CMOS-visible `PWRUP` branch around `CMOSCK`/`DIPFLG`/`DIPSW`, can run the
  visible `RHSTD` / `RHSTDS` all-time and today's high-score reset copy, can
  report the `romc0.src` target reached by each `PWRUP` action decision, can
  read `AUDITG` / `MSGAUD` audit and operator-adjustment rows from their
  source CMOS offsets, can format the source-visible `DISAUD` line buffer for
  those rows, can apply the source-visible `ALTER` / `HYSCRE` mutation rules,
  can step the source-visible `AUDITG` row navigation from IN2 service inputs,
  and can snapshot source-labeled CMOS and RAM fields. A main-board address
  classifier exists for RAM, banked I/O,
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
  native visible palette-index and RGBA frames from its video RAM and palette
  RAM, but CPU interrupt scheduling remains a gap. LED segment side effects and
  sample generation also remain gaps.
  Sound-board PIA IC4 data/control behavior exists for port-B command reads and
  port-A DAC writes, and command CB1 updates the PIA IRQ state. There is still
  no exact power-on RAM state, translated `AUDITG` live text transfer/debounce
  timing or `CROM0` diagnostics after the now-identified `PWRUP` action
  decision, CMOS persistence, screen scanline scheduler, watchdog timing/reset
  side effects, rendering timing side effects, decoder PROM behavior, DAC
  sample output, CPU IRQ scheduling, or translated `VSNDRM1.SRC` routines.

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
  debounce timing, but exact reverse integration with player movement and
  rendering still needs the surrounding player routines. The `SBOMB` switch
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
  `PLSTRT` live respawn integration, human-carry behavior, and full IRQ
  scanline/live rendering integration remain open.
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
- Wave setup has `WVTAB` data available, but launch processes, spawn positions,
  object slots, and process order are not implemented.
- Planet destruction, humanoid restoration, human abduction/rescue, and
  two-player session state are not implemented.

### Enemy Behavior

- Schizoid, UFO, lander, tie-fighter, mini-swarmer, astronaut,
  falling-astronaut, and score-popup process slices are translated into the new
  arcade core.
- Object state must preserve raw `phr6.src` object cell fields such as position,
  velocity, picture, color, process state, counters, and shell ownership.
  The translated `GETOB`, `OBINIT`, `KILLOB`, `KILSHL`, `GETSHL`, `OSCAN`, and
  `ISCAN` list/scanner primitives are the first step; shells must remain
  SPTR-linked object cells rather than a separate gameplay-only collection.
- Baiter, broader bomber/pod wave launch, full minefield behavior, full swarmer
  respawn, wave launch setup, and remaining object/process behavior must be
  translated from red-label routines and verified with traces.

### Video, Appearance, And Attract Mode

- The MAME Williams screen-memory byte layout, native visible-area
  palette-index extraction, RGB palette resistor conversion, and an
  aspect-preserving native RGBA cabinet-frame scaler are implemented.
- The current live loop still renders an explicitly named scaffold frame until
  the arcade core produces real video RAM frames. `STOUT` / `SBLNK`, `PRDISP` /
  `POUT`, `OPROC`, shell output, `EWRITE`, and player-explosion routines can
  mutate video RAM directly, but full scanner/terrain/object scheduling, color
  walkers, text, attract mode, game-over, high-score screens, and scanline
  ownership still need exact translation.
- Embedded PNGs are acceptable only as generated/self-contained deployment
  assets. The verified renderer must be derived from red-label ROM/source data
  and fixture-checked.

### Audio

- The clean-slate tree has raw sound-command trace plumbing and a
  MAME-documented sound-board RAM/PIA/ROM memory surface plus the
  main-board-to-sound-board command latch handoff and sound PIA port-A DAC
  callback boundary. Command CB1 now sets the sound PIA IRQ state, but the tree
  does not yet include sound-routine execution, CPU IRQ scheduling, or sample
  generation.
- Sound must be rebuilt as command writes into a translated sound-board state
  machine from `VSNDRM1.SRC`, not as high-level gameplay cues.

### Session, Cabinet State, And Operator Behavior

- Credits and one-player start are only a scaffold.
- CMOS-backed high-score reset copies all-time and today's tables from
  `DEFALT`, but high-score comparison, initials entry, persistence,
  two-player state, service switches, diagnostics, audits, and adjustments are
  absent.
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
- There is no regression suite for two-player state, coin/start flow, operator
  settings, or cabinet input profiles.

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
- Local high-score persistence mirrors CMOS-like storage through a trait.
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
The runtime must switch from `render_scaffold` to that path only after video RAM
is driven by translated arcade routines or trace-verified ROM-derived state.

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
9. Keep unit-test line coverage at or above 80% at all times.

### Phase 1: Reference Build And Golden Trace Harness

Status: complete for checked-in harness code. The generated golden traces are
local ignored artifacts produced with user-supplied ROMs and MAME.

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
   - `make reference-inputs` writes the expanded `*.inputs.txt` files.
   - `make reference-traces` invokes MAME through
     `tools/generate_reference_traces.py` and `tools/mame_defender_trace.lua`
     to produce local ignored `*.expected.tsv` files.
   - `make reference-fixtures-check` validates that every required expected
     trace uses the checked-in schema and has the expected frame count.
5. Add tests that compare Rust trace output to golden traces when local
   fixtures are available, using `--fidelity-check-trace` for exact TSV
   comparison where practical, and `--fidelity-check-trace-dir` for local
   fixture directories that skip gracefully when fixtures are absent.
6. Add ignored failing tests for known unknowns before translating each
   subsystem. Unignore each test when the exact source behavior is implemented.

### Phase 2: Core State Replacement

Status: in progress. The first table-backed RAM image now initializes
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
including guarded `SBMBX2`/`SUCIDE`. Remaining work is to move scaffold state
onto those bytes, integrate the translated IRQ object-phase gate with the
remaining sound, coin-scan, terrain, palette-copy, hardware-map, and video
scheduler side effects, and translate the remaining process bodies.

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

Status: started. Source-shaped `MKPROC`, `MSPROC`, `SLEEP`, `KILL`, and
`DISP` primitives can allocate regular/super processes, splice them through
`CRPROC`, delay the current process, return killed cells to the right free list,
walk the active-process list, decrement `PTIME`, write `CRPROC`, and return
`PADDR` for the due process. Body dispatch exists for the translated
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
`KILP` score/explosion/sound path, `UFOST` / `UFOLP` UFO start/process loop,
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
`THINIT` refresh. The remaining `PLRES` swarmer respawn path remains an
explicit gap because its phony-object `OX16` low byte depends on the CPU B
register carried into `PLRES`; implement it only after register-aware scheduler
tracing proves that entry state. Broader body dispatch, suicide resume semantics,
cycle/frame scheduling, and golden-trace verification are still open.

1. Replace the 90ms gameplay tick with cabinet frame stepping.
2. Decouple terminal draw cadence from core stepping.
3. Port or model red-label process scheduling and delay semantics.
4. Verify object process order against golden traces.
5. Preserve compatibility frame pacing only in the renderer/application layer.
6. Do not convert cycle/frame values by feel. Use source constants, MAME driver
   metadata, or measured trace timing.

### Phase 4: Player, Inputs, And `xyzzy`

Status: next implementation phase after golden local traces and the remaining
table-backed initialization/scheduler work. `LFIRE` entry/fall-through,
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
`THPROC` table advance translated. `OPROC` active-object band erase/write is
translated for descriptor pictures, Y-band gating, BGL-relative X bounds, and
alternate image flavor selection. `VELO` active-object position updates are
translated for velocity addition and Y wrap. The normal/inverted IRQ
object-band tails are translated for the source ordering around `PRDISP`,
`OPROC`, `SHELL`, and `VELO`, using `XXX1`/`XXX2`/`XXX3` bounds from RAM. The
IRQ object-phase gate now applies source `VERTCT` thresholds, `IFLG`, `TIMER`,
watchdog data-byte reporting, palette-copy due tests, and the normal/flipped
`XXX2` calculations, and runs translated `PLAYER` / `STOUT` pre-tail work on
the branches that call them. It can also run translated `BGOUT` when the caller
supplies the live 6809 stack pointer. Full hardware IRQ integration still needs
`SNDSEQ`, `CSCAN`, palette copy side effects, live stack-context wiring, and
hardware map restoration. The
`SCPROC` scanner maintenance
process is translated through `ISCAN`, `OSCAN`, `SHSCAN`, and `SCNRV` with exact
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
`KILP` score/explosion/sound path, `UFOST` / `UFOLP` UFO start/process loop,
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
support-process bodies using embedded `COLTAB` / `TCTAB` assets. The `BONUS`
wave-clear death tail reached through `PDTH5` now writes the `MESS` / `WNBV`
bonus text and numbers, scores survivor icons through `BC1`, refreshes the next
wave through `GETWV`, and returns through `BC3` to `PDTH5SCLR`. Full reverse
movement/rendering integration, remaining enemy kill/collision vectors beyond
`BKIL` / `NOKILL` / `ASTKIL` / `MSWKIL` / `PRBKIL` / `SCZKIL` / `UFOKIL` /
`LKILL` / `LKIL1` / `TIEKIL`, the remaining `PLRES` swarmer respawn path,
full IRQ scanline/live video integration, full frame/cycle integration, and
golden-trace proof for the translated death tail remain open.
The assembled/ROM-confirmed addresses for the remaining `PLSTRT` / `PLRES`
companion labels (`SCLR1`, `ASTST`, `ASTRO`, `ASTKIL`, `PRBST`, `PRBKIL`,
`MMSW`, `MSWM`, `MSWLP`, `SWBMB`, `MSWKIL`, `SHOOT`, `SCZST`, `SCZ0`,
`SCZKIL`, `UFOST`, `UFOLP`, `UFOKIL`, `LKIL1`, `LKILL`, `AFALL`, `AFALL2`,
`P250`, `P500`, `P503`, `TERBLO`, `TBL3`, `TBL4`, `TIEST`, `TIE`, `TIEKIL`)
are embedded under
`assets/red-label/routine-addresses.tsv`; `BONUS`
continuation labels
(`BC1`, `BC2`, `BC3`, `GETWV`) and `PDTH5SCLR` are embedded there as well.
The next `PLSTRT`
work is to
translate the remaining routine bodies from source or ROM traces, not to infer
their behavior from labels.

1. Define cabinet input bits and action names.
2. Add input profiles:
   - `planetoid`, preserving current keyboard mapping;
   - `cabinet`, mapping original arcade actions;
   - `test`, deterministic and easy for fixtures.
3. Finish full reverse movement/rendering integration, respawn, scanline
   display scheduling, live video presentation, and carry-human behavior.
4. Fix Mutant score to `150` in the arcade core.
5. Reimplement `xyzzy` as an overlay with explicit hooks and tests.
6. Keep current high-level `SessionState` only as orchestration around the core.
7. Keep Planetoid key mapping in input-profile code only. It must never alter
   arcade-core semantics.
8. Any `xyzzy` change must have a paired test proving the arcade path is
   unchanged when `xyzzy` is disabled.

### Phase 5: World, Waves, Humans, And Enemies

1. Port remaining exact terrain state: live `BGOUT` scheduling, destroyed-planet
   terrain, and mini-terrain behavior. `TDATA`, `ALINIT` altitude-table
   generation, `BGINIT` terrain-table generation, RAM-level `BGOUT` terrain
   output, and `BGERAS` screen-table erase are already source-derived.
2. Port wave setup and launch processes from red-label source instead of
   hand-placed opener slots.
3. Port Landers and human abduction.
4. Port Mutants, including planet-destroyed behavior.
5. Port remaining bomber/pod wave-launch and minefield behavior.
6. Port Pods and Swarmer burst behavior.
7. Port Baiter spawn, pursuit, firing, and lifetime behavior.
8. Port remaining shell rendering/output and collision behavior.
9. Verify each enemy subsystem against focused traces.
10. Do not port old `oldsrc/src/game.rs` behavior unless it is first traced back
    to a red-label routine/table.

### Phase 6: Scoring, Players, And Cabinet Session

1. Implement one-player and two-player start/session flow.
2. Implement credits and start buttons.
3. Implement P1/P2 scores, lives, smart bombs, current player, and alternating
   turns.
4. Implement exact scoring for enemies, bullets, mines, humans, rescued humans,
   Pods, Swarmers, wave-end humanoid bonuses, extra ships, and smart bombs.
5. Implement red-label high-score and today's-greatest behavior.
6. Add a CMOS-like storage trait with file-backed local persistence.
7. Add operator/default settings needed for red-label exactness.
8. Add negative tests for every corrected score so old prototype mistakes, such
   as Mutant `250`, cannot return.

### Phase 7: Video And Appearance

1. Define the native cabinet frame buffer and visible crop used by verification.
2. Feed translated video RAM into the native cabinet-frame renderer.
3. Replace synthetic live terrain drawing with the cabinet terrain path.
4. Generate or decode every gameplay object from red-label data.
5. Preserve per-object `OPICT`, animation, palette, and color state.
6. Continue integrating translated `SCNRV` scanner drawing into full-frame live
   video presentation and pixel fixtures.
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

1. Replace high-level gameplay cue dispatch with sound command writes.
2. Model the sound board state, command latch, and routine dispatch.
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
- Unit-test line coverage is at least 80%.
- Every arcade-core routine cites a red-label source/trace, or is listed as an
  unknown gap with an ignored/failing fidelity test.
- `README.md` and this spec agree on current behavior and remaining gaps.
