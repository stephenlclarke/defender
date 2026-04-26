# Known Fidelity Gaps

This file records behavior that must not be guessed in arcade-core code.

## Trace Harness

- Local MAME-driven reference trace generation tooling exists under `tools/`,
  and Phase 1 scenario definitions are embedded under
  `assets/red-label/trace-scenarios.tsv`. Golden traces remain local ignored
  artifacts because they are emulator outputs and require user-supplied ROMs
  plus a local MAME install. The generator runs each scenario with its own
  freshly cleared MAME `cfg`/`nvram` directories under the ignored local fixture
  tree so scenario traces do not share emulator state.
- On `2026-04-26`, Homebrew MAME `0.287` at `/opt/homebrew/bin/mame` generated
  all 12 Phase 1 local reference traces against the verified local
  `assets/roms/defender` red-label ROM set. `make reference-fixtures-check`
  reported 12 complete fixtures and 9,600 frames. Rust trace generation now
  starts from cold-boot object RAM, while the regular live/test constructor
  keeps the pre-initialized translated-routine harness. With that split,
  `make trace-fixtures` now matches all 12 local Phase 1 fixtures and 9,600
  frames for the current trace columns. The first boot-time drift was resolved
  by modeling the `defb6.src` power-up RAM-fill routine and the MAME-observed
  frame boundaries for its first two visible passes. A local
  `DEFENDER_TRACE_DEBUG` run showed MAME executing fixed-ROM addresses in the
  `0xF66x`-`0xF69F` range across the first visible RAM mutations. Those program
  counters map to the `defb6.src` reset path and power-up RAM test: `RESET`
  setup at lines 1463-1489, `RAM2` seed save at line 1498, `RAM3` RNG update
  at line 1500, and `RAM6` RAM-fill writes at line 1515:
  <https://github.com/mwenge/defender/blob/master/src/defb6.src#L1463-L1515>.
  The remaining acceptance work is to keep replacing trace-only scheduling with
  source-shaped reset, ROM-test, interrupt, and translated gameplay execution
  rather than treating the current trace-column match as a complete emulator.
- The first Rust trace schema exists in `src/fidelity.rs` and includes raw
  internal input bits, MAME IN0/IN1/IN2 input bytes, optional raw object-table,
  shell-table, and native video-frame CRC-32 columns, and a raw sound-command
  column.
- `assets/red-label/trace-schema.tsv` is the single checked-in trace schema
  source. The old duplicate `docs/fidelity/trace-schema.tsv` was removed to
  prevent fixture documentation drift.
- The trace format can carry object and shell table checksums. The first
  `phr6.src` RAM layouts and linked-list heads are embedded under
  `assets/red-label/ram-layout.tsv` and `assets/red-label/linked-lists.tsv`,
  the current Rust trace emits object-table and SPTR-head CRCs from the
  table-backed runtime memory, and the local MAME runner can emit Phase 1
  fixture rows. The checked-in Rust code still cannot prove those CRCs against
  red-label behavior unless local expected fixtures have been generated.
- `--fidelity-trace [FRAMES]` can emit deterministic Rust TSV frames for local
  fixture work, but it is not a MAME/source golden-trace generator.
- `--fidelity-trace-inputs SCRIPT` can apply scripted per-frame cabinet inputs
  to Rust traces, but it still uses the current scaffold core until translated
  routines replace it.
- `--fidelity-trace-inputs-file PATH` can read those scripted cabinet inputs
  from local fixture files.
- `--fidelity-check-trace INPUTS_PATH EXPECTED_TSV` can compare the generated
  Rust trace against a local expected TSV fixture exactly, but the expected
  fixture still has to be created outside this repo.
- `--fidelity-check-trace-dir PATH` can check every paired `*.inputs.txt` /
  `*.expected.tsv` fixture in an ignored local directory and skips a missing
  fixture directory. The local fixture layout exists under
  `docs/fidelity/fixtures/`.
- `--fidelity-list-scenarios`, `--fidelity-write-scenario-inputs PATH`, and
  `--fidelity-check-reference-trace-dir PATH` expose and validate the complete
  Phase 1 scenario manifest without checking generated golden traces into the
  repository.

## Core State

- Verified ROM image views exist for fixed main-CPU ROM, banked program ROMs,
  sound CPU ROM, and decoder PROM bytes, and the source-shaped `CROM0` `ROMMAP`
  descriptor table is derived from the embedded MAME load map. The
  source-shaped `ROM0`/`ROM9` checksum scan can report the physical ROM numbers
  that `CROM0` would display for failures, and the ROM-stage outcome records
  the manual/auto success/failure display intent, `ADVSW` / `NEXTST` gate
  sequence, message-ROM bitmap text transfer including CMOS text controls,
  RAM-test start/failure/no-error visible setup, the RAM2 pattern fill/verify
  pass with page-boundary operator-poll metadata, pass-boundary loop dispatch,
  and CMOS RAM-test write/verify loop plus visible outcomes, plus the CROM0
  color-RAM diagnostic heading/bars/palette loop, audio-test
  sound-pulse/skip-table behavior, switch-test
  display-table/PIA-scan behavior, and monitor-test
  crosshatch/RGB-field/color-bar pattern behavior.
  The MAME-documented main-board and sound-board memory maps are
  embedded under
  `assets/red-label/memory-map.tsv` and checked against the Rust address
  classifiers. Red-label fixed-bank SRAM routine metadata is embedded under
  `assets/red-label/sram-routines.tsv`,
  source-owned CMOS cell metadata is embedded under
  `assets/red-label/cmos-layout.tsv`, `romc8.src` CMOS default bytes are
  embedded under `assets/red-label/cmos-defaults.tsv`, `defb6.src` bomb shell
  image bytes are embedded under `assets/red-label/shell-images.tsv`,
  `defb6.src` complete and short-form object-picture metadata and image bytes
  are embedded under `assets/red-label/object-pictures.tsv` and
  `assets/red-label/object-images.tsv`, `defa7.src`
  shell/scoring/sound/collision/hyperspace/player-display/scanner/start-support
  routine entry points plus object-picture ON/OFF and `CWRIT` / `COFF` entry
  points are embedded under
  `assets/red-label/routine-addresses.tsv`,
  red-label sound table bytes are embedded under
  `assets/red-label/sound-tables.tsv`, the
  complete `defb6.src`
  `SWTAB` switch bit table is embedded under
  `assets/red-label/switch-table.tsv`, and the first source-owned RAM table and
  linked-list metadata is embedded under `assets/red-label/ram-layout.tsv` and
  `assets/red-label/linked-lists.tsv`,
  including the switch-history/queue bytes used by `SSCAN`/`SWP`, the `TEMP48`
  scanline bound used by `PRDISP`, the `SMAP`, `THTAB`, and `FBTAB` table
  bytes, and the split player-position/velocity plus RNG seed and `ASTCNT`
  fields touched by `PLSTR5` and `HYPER`.
  The board harness can now read/write packed CMOS/SRAM bytes and words using
  the documented most-significant-nibble-first order, can apply the ROM-derived
  CMOS defaults to its CMOS cell array through the visible `CMINIT` clear/copy
  sequence, can model the visible `CLRAUD` packed zero writes, can route the
  CMOS-visible `PWRUP` branch around `CMOSCK`/`DIPFLG`/`DIPSW`, can run the
  visible `RHSTD` / `RHSTDS` all-time and today's high-score reset copy, can
  read `AUDITG` / `MSGAUD` audit and operator-adjustment rows from their
  source CMOS offsets, can transfer the source-visible `AUDITG` entry screens
  and `DISAUD` row text/erasure, can apply the source-visible `ALTER` /
  `HYSCRE` mutation rules, can step the source-visible `AUDITG` row navigation
  from IN2 service inputs, can model the post-display `AUDITG` debounce
  countdown, can run those pieces as one deterministic audit cycle with
  previous-row erasure, can step the post-`PWRUP` `AUDITG` outer frame path to
  return-to-caller, can compare/insert packed high-score entries in the all-time
  and today's tables, and can snapshot source-labeled CMOS and main-RAM fields.
  A main-board address classifier exists for RAM, banked I/O,
  selected banked program ROM, bank-select writes, and fixed ROM reads. Main
  RAM bytes can now be read and written through a deterministic harness
  surface, and raw write-only palette register bytes are stored. CMOS 4-bit
  writes store `data | 0xf0`,
  video-control writes update
  the cocktail bit, and watchdog writes only count reset recognition for byte
  `0x39`. Video-counter reads expose the MAME `vpos & 0xfc` behavior with the
  `vpos >= 0x100` clamp, driven by a deterministic harness value rather than a
  screen scheduler. Cabinet input projection now exposes the MAME
  active-high Defender IN0, IN1, and IN2 port bytes, including service and coin
  lines, and the main-board now routes CPU reads and writes through the
  MAME-modeled 6821 PIA data/control and data-direction register behavior. This
  lets ROM code consume IN0/IN1/IN2 bytes and drive DDR-filtered sound-command
  writes through PIA1 port B. CA1/CB1 and input-mode CA2/CB2 edge IRQ flags are
  modeled, including control-register flag bits and data-port read clearing.
  CA2/CB2 set/reset output modes and read/write strobe restore behavior are
  also modeled from MAME, including CB2's delayed CB1-restore path through
  port-B reads.
  MAME's older Williams VA11/COUNT240 video interrupt inputs can now drive PIA1
  CB1/CA1, and the main PIA IRQ line state is visible to deterministic tests.
  The main board can expose native visible palette-index and RGBA frames from
  its video RAM and palette RAM. A sound-board address classifier exists for
  Defender 6808 internal RAM, PIA IC4 mirrors, sound ROM reads, and the
  main-board sound-command latch handoff onto sound PIA port B/CB1. The sound
  CPU can read that latched command through PIA IC4 port B after selecting the
  data register, PIA IC4 port A writes are captured as the DAC callback
  boundary, and command CB1 drives the sound PIA IRQ state. The board layer can
  report the `romc0.src` target reached by each `PWRUP` action decision,
  dispatch `AuditGate` into the source-visible `AUDITG` entry screen, and
  step/read/format/mutate and video-transfer the source `AUDITG` / `MSGAUD`
  table rows and its post-display debounce countdown as one deterministic cycle
  plus the post-`PWRUP` outer frame path, but CPU IRQ scheduling, exact Williams
  power-on RAM contents, physical advance-switch timing, physical lamp timing,
  default live CMOS path policy,
  screen scanline scheduling, watchdog timing/reset side effects,
  palette/rendering timing side effects, decoder PROM behavior, and DAC sample
  generation are not modeled.
- `ArcadeMachine` now owns a table-backed main-RAM image for the red-label core
  scaffold. It initializes `PINIT`/`OINIT`-style process, super-process, and
  object free lists, sets `CRPROC` to the active-process head, clears active
  object/inactive object/shell heads, seeds one-player `START` fields from the
  `NSHIP` and `REPLAY` CMOS defaults, exposes source-shaped `MKPROC`,
  `MSPROC`, `SLEEP`, `KILL`, and `DISP` process-list primitives, exposes
  source-shaped `GETOB`, `OBINIT`, `KILLOB`, `KILSHL`, `OSCAN`, `ISCAN`,
  `GETSHL`, `SHSCAN`, and `SHELL` movement/death/dead-erase primitives,
  translates `SCORE`, the visible RAM effects of `SNDLD`, `BKIL`, the `REV`
  reverse debounce path, and the `SBOMB` entry/flash/debounce tail, routes
  collision dispatch through `OCVECT` for translated vectors, and emits
  object-table and SPTR-head CRCs in trace rows.
- `--verify-roms PATH` can validate and map a local red-label ROM set into the
  embedded MAME region layout. ROM execution, golden traces, and generated
  derived assets remain future work.
- Object cell layout, player table layout, regular process cell layout,
  super-process cell layout, and runtime list-head addresses are now embedded
  from `phr6.src`. The `GETOB`, `OBINIT`, `KILLOB`, `KILSHL`, `GETSHL`,
  `OSCAN`, `ISCAN`, `SHSCAN`, and `SHELL` movement/death/dead-erase cell/list
  mutations are translated over those bytes. The `BMBOUT` and `FBOUT` shell
  output byte writes are translated, with ROM-resident bomb image bytes embedded
  under `assets/red-label/shell-images.tsv`, and `OBJCOL` dispatch reaches
  those translated callbacks through `assets/red-label/routine-addresses.tsv`.
  `SCORE`, the visible `SNDLD` state updates, `BKIL`, `LFIRE` entry,
  `LASR0` / `LASL0` drawing/fizzle/erase/sleep loops, `LASD` tail,
  RAM-visible `LCOL`, source-ordered `FISS` / `STINIT` / `FBINIT` / `THINIT`
  boot table initialization, source-addressed `OCVECT` dispatch to translated
  vectors including `NOKILL`,
  `COLIDE` / `COL0` picture-mask intersection for complete and short-form
  object-picture descriptors, visible `COLCHK` player-collision side effects,
  `OPROC` active-object descriptor erase/write banding, `VELO` active-object
  velocity addition and Y wrap, source-ordered normal and inverted `IRQ`
  object-band tails over `XXX1` / `XXX2` / `XXX3`, the `REV` reverse debounce
  path, the `SBOMB` entry/flash/debounce tail, and exact source `LFIRE`
  fall-through into the first laser loop are translated.
  The `STOUT` IRQ star-output path is now translated through its source
  fall-through `SBLNK` blink/hyperspace side effects over `SMAP`, `ITEMP`,
  `ITEMP2`, `SEED`, `HSEED`, `LSEED`, `STATUS`, and `WCURS`.
  The `HYPER` entry guard/status/`SCLR1`/sleep sequence is translated through
  the assembled `HYPER` and `HYP02` addresses. The visible `HYP02`
  rematerialization slice now clears `SPTR` shells, seeds `BGL`/`BGLX`,
  updates player position/direction RAM, creates the phony player object, runs
  the source `APVCT`/`APST` appearance-start RAM updates, loads `APSND`, and
  sleeps to `HYP2`. The visible `HYP2` tail now kills the phony object, resets
  `STATUS` through `STCHK`, suicides on the safe path, and exposes the `PLEND`
  branch when `LSEED > 192`. The RAM-visible `EXST` / `EXPU`
  appearance/explosion path now initializes explosion slots, advances `RAMALS`
  slots, restores object pictures when appearances finish/offscreen, clears old
  erase-list blocks, and runs the translated `EWRITE` expanded-object writer
  from embedded picture assets. `KILOFF` now unlinks objects and clears the
  current picture footprint through `OFSHIT`'s map-2 erase path. The
  `PLEND`/`PDEATH` entry and glow loop are translated through `PDTHL`, `PDTH2`,
  and `PDTH4`, including `PLSAV`, `MONO`, `PDSND`, `PXCTB`, and pseudo-color
  RAM side effects. `PDTH5` now clears `PCRAM`, runs exact `GNCIDE`, enters
  the bank-7 `PXVCT`/`PX1A` player-explosion setup and frame loop from
  `blk71.src` using the extracted `PXCOL` color table, resumes at the
  ROM-confirmed `PDTH5R` address, runs `WVCHK`, and translates the non-wave-end
  respawn, player-switch, and game-over branch decisions through `PLE02`,
  `PLE3`, `PLSTRT`, and `ATTR`. `SCPROC` now runs the source-shaped `ISCAN`,
  `OSCAN`, `SHSCAN`, and bank-1 `SCNRV` scanner-raster stages and sleeps
  through the exact `SCP1`, `SCP2`, and `SCPROC` resume addresses. `PLSTRT`
  now covers the RAM-visible
  entry handoff, `PINIT` process-list reset, `PLSTR3` / `PLSTR5`
  current-player runtime bytes, bank-7 `ALINIT` / `BGALT` altitude-table
  generation from `TDATA`, bank-7 `BGINIT` mirrored `TERTF0` / `TERTF1`
  terrain-table generation from `TDATA`, support-process creation,
  `PLS01` status/sleep,
  `PLS1` `PLRES` astronaut-process/target-list/enemy-runtime restore,
  schizoid-reserve `SCZST` restore from copied `SCZRES`, the `SCZ0`
  movement/shot-timer process slice through shared `SHOOT`,
  probe-reserve `PRBST` restore from copied `PRBRES`, tie-fighter reserve
  `TIEST` restore from copied `TIERES`, `STCHK`/`PDFLG` tail, and machine-level
  `INIT20` `CRINIT` / `FISS` / `STINIT` / `OINIT` / `FBINIT` / `THINIT`
  refresh. `BGOUT` now rolls the terrain tables to `BGL` and writes selected
  terrain flavor records through `STBL` when the caller supplies the live 6809
  stack pointer, and `BGERAS` erases terrain screen words through the source
  `STBL` table. `COLR` / `COLRLP`, `FLPUP` / `FLP2`, `CBOMB` / `CBMB1`, and
  `TIECOL` / `TIECL` now run as translated support-process bodies using
  embedded `COLTAB` / `TCTAB` assets. Translated `PLSTRT` runtime dispatch now
  syncs the live snapshot's current player, wave, lives, smart bombs, and
  player motion from red-label RAM. The zero-enemy `BONUS` death-tail path now
  clears through `SCLR1`, writes the source `MESS` / `WNBV` wave-complete text
  and numbers from embedded `mess0.src` assets, scores survivor icons through
  `BC1`, refreshes wave parameters through `GETWV`, and returns through `BC3`
  to `PDTH5SCLR`. The scheduled `ASTRO` process now walks `TLIST` targets
  against `ALTTBL`, toggles `ASTP1`-`ASTP4`, and sleeps back to `ASTRO`.
  `ASTKIL` now runs `ASTCLR`, `KILOFF`, `ASXP1`, `XSVCT`, and `AHSND`, and
  creates and dispatches the source `TERBLO` terrain-blow process when the
  final astronaut is removed, including `BGERAS`, scanner-terrain `STETAB`
  erase, `TEREX` explosion passes, `TBL3` / `TBL4` sleeps, `OVCNT`, `COLTAB`,
  `AHSND`, and final `TBSND` / `SUCIDE`. The `PLRES` schizoid reserve path
  now restores copied
  `SCZRES` through source-shaped `SCZST` / `SCZ0` process/object setup,
  including the second `RMAX` RNG advance and `APVCT` appearance entry; the
  scheduled `SCZ0` body now runs its source-shaped X seek, Y seek/avoid, random
  Y hop, shot timer, shared `SHOOT` shell setup, and `SSHSND` load. `UFOST`
  now enters through translated process dispatch and runs the source UFO
  process/object start; `UFOLP` runs the shot timer, shared `SHOOT` shell
  setup, `UFOP1`-`UFOP3` image cycle, `UFONV` velocity update, and `USHSND`
  shot sound. `SCZKIL`
  now runs the source `KILP` score/explosion/sound path with `SCHSND`,
  `UFOKIL` decrements `UFOCNT` and runs the source `KILP` path with
  `UFHSND`, normal `LKILL` decrements `LNDCNT` and runs the source `KILP`
  path with `LHSND`, `LANDST` creates source-shaped landers or falls through
  to `SCZS0`, `GTARG` selects target slots, `LANDS0` orbits and fires through
  `LSHOT`, `LANDG` captures passengers, `LANDF` flees, `LNDFXA` pulls the
  passenger in or gives up, and `SCZ00` converts completed landers to
  schizoids. Kidnapping `LKIL1` starts the source `AFALL` passenger-release
  process, loads `ASCSND`, then falls through `LKILL`, while captured
  astronaut `AKIL1` handles player catch into `AFALL2` / `P500` with `ACSND`
  and shot kills through `ASTK1` / `KILLOP`. `AFALL` / `AFALL2` now run the
  falling/caught astronaut path, safe/fatal landing decisions, `ALSND`, and
  the `P250` / `P500` / `P503` rescue-score popup lifecycle,
  `PRBKIL` runs the source `KILO` / `RMAX` / `MMSW` probe-hit mini-swarmer
  spawn path with `PRHSND`, `MSWKIL` runs the source `KILOFF` / `KILLOP` /
  `XSVCT` score/sound path with `SWHSND`, `MSWM` / `MSWLP` now runs the
  mini-swarmer acceleration/damping/turnback loop and `SWBMB` `GETSHL` fireball
  shell path with `SWSSND`, `TIE` runs the source image/vertical/cruise slice
  and `BOMBST` bomb-shell lifetime path, and `TIEKIL` runs the source `KILO`
  score/explosion/sound path with `TIHSND` plus squad-slot and final
  super-process cleanup. The
  remaining `PLRES` swarmer respawn path, whose phony-object X low byte depends
  on the CPU B register carried into `PLRES`, enemy kill/collision vectors
  beyond `BKIL` / `NOKILL` / `ASTKIL` / `AKIL1` / `MSWKIL` / `PRBKIL` /
  `SCZKIL` / `UFOKIL` / `LKILL` / `LKIL1` / `TIEKIL`, live terrain scheduling,
  full live respawn orchestration beyond the runtime snapshot handoff, and
  golden-trace proof for the translated death tail remain open. The
  assembled/ROM-confirmed addresses for `BONUS`, `BC1`, `BC2`, `BC3`, `GETWV`,
  `PDTH5SCLR`, `SCLR1`, `PLRES`, `ASTST`, `ASTRO`, `ASTKIL`, `PRBST`, `PRBKIL`,
  `MMSW`, `MSWM`, `MSWLP`, `SWBMB`, `MSWKIL`, `SHOOT`, `SCZST`, `SCZ0`,
  `SCZKIL`, `UFOST`, `UFOLP`, `UFOKIL`, `LANDST`, `LANDS0`, `LANDG`, `LANDF`,
  `LNDFXA`, `SCZ00`, `AKIL1`, `LKIL1`, `LKILL`, `AFALL`, `AFALL2`, `P250`,
  `P500`, `P503`, `BGI`, `TERBLO`, `TBL3`, `TBL4`, `TIEST`, `TIE`, and
  `TIEKIL` bodies are embedded, alongside the
  translated `BGINIT`, `BGOUT`, and
  support-process entry points.
  Translated `SCLR1` callers now
  use the original active-screen clear shape, clearing rows `Y >= 42` on each
  video page rather than the whole `SCRCLR` range.
  `ram-layout.tsv` now also records the source text cursor at `A050` and keeps
  `ITEMP` / `ITEMP2` at their source positions `A06F` / `A071`, behind
  `SPTR`, so translated star/player routines no longer overlap message RAM.
  One-player start now applies the visible `PLSTR5` player runtime bytes, and
  live fire/smart-bomb/hyperspace/reverse now enters `LFIRE`/`SBOMB`/`HYPER`/
  `REV` through the asset-backed red-label `SWTAB`, `SSCAN` switch history,
  `SWPROC` queue, and `SWP` status gate. The scanner records all eight source
  table bits, and the translated IRQ `PLAYER` motion slice now consumes the
  thrust and altitude bits for source-shaped horizontal damping, thrust,
  X/scroll correction, absolute-X, and altitude velocity updates. The
  source-shaped `PRDISP` / `POUT` player-picture slice now stores the scanline
  bound in `TEMP48`, gates on `STATUS` / `PLAYC`, clears the old 8x6 footprint
  through `OFF86`, copies `NPLAD` / `NPLAXC`, and draws `PLAPIC` / `PLBPIC`
  through embedded `ON86` image bytes. The same translated display slice now
  writes and clears the adjacent `THOUT` / `THOFF` thrust-flame bytes from the
  extracted `THTAB`, while `THPROC` advances the fireball/thrust table
  pointers. `OPROC` now walks active objects in a caller-supplied scanline band,
  clears old descriptor footprints, and redraws descriptor pictures with the
  ROM BGL-relative X and alternate-flavor rules. `VELO` now advances active
  object positions by source velocities and wraps Y through `YMIN` / `YMAX`.
  The `IRQ` / `IRQB` object-band tails now run already translated `PRDISP`,
  `OPROC`, `SHELL`, and `VELO` slices in source order with scanline pairs read
  from `XXX1` / `XXX2` / `XXX3`. The scanline object-phase gate now applies the
  source `VERTCT` thresholds, `IFLG` latch, `TIMER` increment, normal/flipped
  watchdog data byte, palette-copy due tests, and `XXX2` calculations before
  entering those tails, and it runs translated `PLAYER` / `STOUT` pre-tail work
  on the source branches that call them. When the caller supplies the live 6809
  stack pointer, the terrain branch also runs translated `BGOUT`; otherwise it
  records that `BGOUT` is due. The remaining `SNDSEQ`, `CSCAN`, palette copy
  side effects, live stack-context wiring, and hardware-map restoration still
  need full scheduler integration.
  The start-flow foundation now covers source-shaped `FPLAY` credit
  seeding from the core CMOS image, the RAM-visible `START` power-page
  gate/player table reset/`PLSTRT` process creation, source `SCRCLR` video-RAM
  clear, and the `START2` BCD credit/`PLRCNT` RAM updates plus `WCMOSA CREDST`
  packed CMOS credit backup. The source `TDISP` top-of-screen redraw now covers
  `BLKCLR`, `BORDER`, `LDISP`, `SBDISP`, and `SCRTR0` visible writes from
  extracted score digit, mini-ship, and smart-bomb image assets. `ST1` and `ST2`
  are now queued from `SWTAB` and
  dispatched through their translated source order, including status/credit
  gates, start sounds, one/two `START` calls, and final `DIE` cleanup. Object
  behavior,
  per-field semantics beyond those routines, live scanline object rendering
  around the translated picture helpers, and process ownership rules are not
  translated.
- Shells are now identified as SPTR-linked object cells from `GETSHL`, not as a
  separate invented table. `GETSHL` allocation and `KILSHL` cleanup are
  translated, `SHSCAN` lifetime cleanup is translated, and the `SHELL`
  movement/death/dead-erase front half is translated. `BMBOUT` and `FBOUT`
  output byte writes and `OBJCOL` dispatch for those callbacks are translated.
  The first visible bomb shell collision path is translated through `OCVECT` to
  `BKIL`, including `SCORE`, `KILSHL`, shell-footprint erase, explosion-picture
  handoff, and `SNDLD(AHSND)`. `COLIDE` / `COL0` now performs exact non-zero
  picture-byte intersection for complete and short-form object-picture
  descriptors and writes `CENTMP` before dispatch. Full ROM/bank memory
  integration, live object rendering ownership, and remaining
  collision callbacks remain gaps.
- Process table layout is now embedded, and source-shaped `MKPROC`, `MSPROC`,
  `SLEEP`, `KILL`, and `DISP` primitives can allocate regular/super processes,
  splice new cells through `[CRPROC]`, delay the current process, unlink killed
  cells to the correct free list based on `PCOD`, walk the active-process list,
  decrement `PTIME`, update `CRPROC`, and return `PADDR` for a due process.
  The translated process dispatcher now runs the `SBOMB` entry/resume
  addresses, live-created `LFIRE` and `SBOMB` processes, `REV` resume
  addresses, `ASTRO` and `MSWM` / `MSWLP` enemy process resumes, and the
  `COLR` / `FLPUP` / `CBOMB` / `TIECOL` support-process resumes from `PADDR`,
  including guarded `SBMBX2`/`SUCIDE`.
  Live coin input now increments the red-label `CREDIT` byte as BCD and mirrors
  `CREDST`, but source-exact coinage/debounce is still not translated.
  Generic/untranslated process bodies, broader suicide resume semantics, the
  remaining `SWTAB` routine bodies and no-process input effects, exact
  frame/cycle integration, and golden-trace equivalence are not translated.
- CMOS layout, ROM default bytes, 4-bit cell writes, `CLRAUD`/`CMINIT` visible
  cell effects, the CMOS-visible `PWRUP` branch and source dispatch target,
  `RHSTD`/`RHSTDS` reset copies, `AUDITG` / `MSGAUD` message-offset rows and
  source-visible entry-screen transfer, row navigation, display-line formatting
  and video transfer/erasure, adjustment mutations, the post-display debounce
  countdown, and red-label packed byte/word helper behavior are modeled, and
  the CROM0 ROM stage now carries
  diagnostic text/palette intent, bitmap
  headline/bad-ROM-row/operator-instruction transfer, and `ADVSW` / `NEXTST`
  gate metadata plus the RAM-test start/failure/no-error visible setup, RAM2
  pattern fill/verify pass with page-boundary operator-poll metadata,
  pass-boundary loop dispatch, and CMOS RAM-test write/verify loop plus visible
  outcomes and the CROM0 color-RAM diagnostic heading/bars/palette loop plus
  audio-test sound-pulse/skip-table behavior,
  switch-test display-table/PIA-scan behavior, monitor-test
  crosshatch/RGB-field/color-bar pattern behavior, monitor-to-`AUDITG` entry
  transfer, post-`PWRUP` `AuditGate` entry transfer, `AUDITG` outer frame-step
  dispatch, packed high-score table comparison/insertion, and optional
  file-backed CMOS persistence. Deterministic initials-entry submission to the
  all-time CMOS and today's-greatest RAM tables is modeled, and translated
  player-death game-over sleeps can enter live `GameOver` for that path. Exact
  high-score screen rendering and full game-over-to-attract timing are not
  translated.

## Player

- The visible `PLSTR5` player-start RAM bytes are source-owned. The surrounding
  live movement/rendering path still uses scaffold state until the exact player
  process is translated.
- The `REV` switch process and debounce tail are translated, including
  `REVFLG`, `NPLAD`, `PIA21`, and process free-list side effects. The `SBOMB`
  switch process now starts from asset-backed `SWTAB`/`SSCAN`/`SWPROC`/`SWP`
  instead of scaffold inventory mutation. The visible `HYP02`/`HYP2`
  hyperspace rematerialization tail is translated up to the `PLEND` death
  branch, `PLEND`/`PDEATH` is translated through the glow-loop sleep into
  `PDTH5`, the `PXVCT`/`PX1A` explosion loop, and non-wave-end
  respawn/game-over branch decisions, and `KILOFF` object unlink/footprint
  erase is translated. Exact `PLAYER` motion now covers horizontal damping,
  thrust, X/scroll correction, absolute-X, and altitude velocity, and
  `PRDISP` / `POUT` now covers the 8x6 player body and `THOUT` / `THOFF`
  thrust-flame write/erase paths. `OPROC` now covers active-object descriptor
  erase/redraw inside caller scanline bands, and `VELO` now covers
  active-object velocity addition and Y wrap. The normal/inverted IRQ
  object-band tails now cover the ROM ordering around `PRDISP`, `OPROC`,
  `SHELL`, and `VELO` for `XXX1`/`XXX2` scanline pairs. The `GEXEC` tail now
  restores
  `STRCNT` after star overflow, advances `GTIME` through the source wrap,
  decrements the process `PD` counter, and applies source `WDELT` intra-wall
  deltas to `ELIST` every 40 passes. `GETWV` now increments `PWAV`, refreshes
  `PENEMY` from
  source-order `WVTAB`, applies CMOS difficulty/ceiling inter-wall `WDELT`
  updates, and restores `PTARG` on the `GA1+6` restore-wave cadence. `BGI`
  now selects bank/map 7 and runs the translated `BGINIT` terrain-table
  generator. `PLSTRT` live respawn orchestration beyond runtime snapshot sync,
  golden-trace proof for the translated `BONUS` / `SCLR1` wave-clear death
  tail, respawn, human-carry routines, full IRQ scanline/live rendering
  integration, and full reverse movement integration remain open.

## World And Enemies

- Terrain `TDATA`, `ALINIT` altitude-table generation, `BGINIT` terrain-table
  generation, RAM-level `BGOUT` terrain output, and `BGERAS` screen-table erase
  are translated. Live terrain scheduling, mini-terrain, and destroyed-planet
  behavior are not translated.
- Wave launch process uses `WVTAB` data but not exact object/process setup.
- Schizoid, UFO, lander, tie-fighter, mini-swarmer, astronaut,
  falling-astronaut, and score-popup process slices are translated. Baiter,
  broader bomber/pod wave launch, full minefield behavior, full swarmer respawn,
  wave launch setup, and remaining object/process bodies are still open.
- Collision detection is translated for the currently extracted complete and
  short-form picture descriptors/images: landers, humanoids, tie fighters,
  probes, swarmers, UFOs, player pictures, laser, mini-player, smart bomb, bomb
  shells, score popups, and explosions. Remaining object-specific callbacks and
  process rules still need translation before `COLIDE` can cover the full game.

## Video

- The MAME Williams screen-memory byte layout, native Defender visible-area
  palette-index extraction, RGB palette resistor conversion, and native RGBA
  cabinet-frame scaler are implemented. The translated `PRDISP` / `POUT`
  player-picture slice can write and erase the 8x6 ship body and
  thrust-flame bytes in video RAM. `OPROC` can erase and redraw active-object
  descriptor pictures in the caller's scanline band. `STOUT` / `SBLNK` can
  write, clear, blink, and hyperspace-shift the source star-map bytes in video
  RAM. The `BONUS`
  `MESS` / `WNBV` calls can write the wave-complete text and numbers used by
  the survivor bonus screen. `SCNRV` now writes scanner terrain, object, and
  player blips through the source `SETAB` / `STETAB` erase tables. Full
  HUD/video presentation, live terrain scheduling, full IRQ object-rendering
  integration, color presentation, general text, attract, game-over, and
  high-score screens are not translated.
- Current live renderer remains explicitly named scaffolding until translated
  video RAM can feed the native cabinet-frame presenter. Live pacing is derived
  from the core red-label `FRAME_RATE_MILLIHZ` constant, and the live loop
  advances any due core frames before each terminal draw so presentation cadence
  no longer decides core frame count.

## Audio

- Gameplay routines that write main-board sound command values are not
  translated.
- The core has a raw sound-command output path, but no gameplay routine emits
  commands until exact command values are source-cited or fixture-verified.
- Sound-board RAM/PIA/ROM address classification exists, and the MAME-documented
  main-board command latch byte/CB1 handoff is modeled from the PIA1 port-B
  output callback boundary. Sound CPU PIA IC4 register reads can consume the
  latched command byte, and PIA port-A writes are captured at the DAC boundary.
  Command CB1 drives the sound PIA IRQ state. DAC sample generation, CPU IRQ
  scheduling, and `VSNDRM1.SRC` routine dispatch are not translated.
- No waveform or command-sequence fixtures exist yet.

## Compatibility

- `xyzzy` exists as an overlay, but each future hook needs a paired test proving
  disabled-`xyzzy` arcade behavior remains unchanged.
- Planetoid controls exist as an input profile and must remain outside the
  arcade core. Cabinet action projection to red-label IN0/IN1/IN2 bytes exists;
  Planetoid and test keyboard choices remain compatibility inputs layered over
  those cabinet actions.
