# Red-Label Embedded Assets

This directory is the checked-in source of truth for extracted Defender
red-label data that the Rust runtime embeds into the final binary.

Rules:

- Gameplay, video, audio, ROM metadata, score, wave, high-score, and trace data
  must live under `assets/` before it is embedded by Rust code.
- Runtime code embeds these files with `include_str!` or `include_bytes!`.
- The deployed game binary must not require a local asset or ROM directory.
- ROM files under `assets/roms/` are verification/reference material, not a
  runtime dependency.
- Future extraction tooling should regenerate these files from the red-label
  source, red-label ROM behavior, or MAME-observed behavior.

Current files:

- `actor-attract.script`: embedded actor-runtime title/attract script that
  drives the Williams reveal, source `ELECV` presents message, Defender
  wordmark coalescence, source-style Hall-of-Fame table rows,
  source-offset scoring/instruction labels, source `CREDV` credits label/count
  text, and the source-shaped scoring scanner surface through the checked
  `AttractScript` parser using source page-start steps for the title messages:
  Williams from step 1, `ELECV` from step 236, the Defender wordmark from step
  365, the high-score/zero-credit Hall-of-Fame page from step 488 for the source
  60-tick stall window, and scoring/instruction labels from step 1088. A
  `credits_nonzero` title-page event suppresses the zero-credit line while
  still showing a real inserted credit. The Hall-of-Fame page also draws
  source-offset `HALLD_*` headings and the source Defender logo; `hall_scores`
  draws Today’s and All-Time table columns from driver scores plus embedded
  red-label seed initials. The scoring/instruction page draws `SCANV`, `LANDV`,
  `MUTV`, `BAITV`, `BOMBV`, `SWRMPV`, and `SWARMV` from checked `messages.tsv`
  rows and source screen addresses, while `scoring_surface` draws the source top
  scanner frame/marker bars and `MTERR` mini-terrain records. It also uses the
  driver-owned high-score table and credit count carried in `StepPrompt`.
  Custom attract scripts can also use checked `messages.tsv` labels through the
  script parser's `message` action with optional visual offsets.
- `actor-behavior.script`: embedded actor-runtime baseline behavior profile
  parsed by `ActorBehaviorScript`, including player, laser, hostile, human,
  explosion, and score-popup timing/tuning defaults.
- `actor-waves.script`: embedded actor-runtime wave progression script parsed
  by `ActorWaveScript`; its `source_waves` directive expands through
  `wave-table.tsv` so the default actor driver is configured from checked text
  while retaining source-table wave values.
- `audit-adjustments.tsv`: red-label `romc8.src` `AUDITG` / `MSGAUD`
  operator audit and adjustment message table, including the source CMOS cell
  offsets and display widths used by `DISAUD`.
- `cmos-defaults.tsv`: red-label `romc8.src` `DEFALT` bytes expanded as
  source-owned CMOS default records. Each byte is written through `CMOSMV` /
  `WCAV` as two 4-bit CMOS cells, most-significant nibble first. The first
  48 bytes also feed the `RHSTD` / `RHSTDS` high-score reset copies into
  all-time CMOS and today's RAM tables.
- `cmos-layout.tsv`: red-label `phr6.src` CMOS cell layout for audit counters,
  all-time high-score slots, credit backup, coinage/operator settings, and
  game-adjust cells.
- `color-ram.tsv`: red-label `defb6.src` `CRTAB` pseudo-color bytes copied by
  the translated `CRINIT` routine into `PCRAM`.
- `color-cycle.tsv`: red-label `defa7.src` `COLTAB` and `defb6.src` `TCTAB`
  pseudo-color cycle tables used by translated `COLR` / `COLRLP`,
  `CBOMB` / `CBMB1`, and `TIECOL` / `TIECL` support processes.
- `defaults.tsv`: red-label default stock, smart bomb, wave, and human counts.
- `high-scores.tsv`: red-label default high-score seed table.
- `input-ports.tsv`: MAME Defender IN0, IN1, and IN2 active-high input bit
  layout and PIA callback mapping.
- `linked-lists.tsv`: red-label RAM list-head symbols and entry link fields for
  active/free processes, free objects, active objects, inactive objects, and the
  SPTR shell-object list.
- `memory-map.tsv`: MAME-documented main-board and sound-board address ranges,
  bank behavior, mirrors, and handlers used by the Rust address classifiers.
- `message-glyphs.tsv`: source `mess0.src` character image bytes consumed by
  the translated `BONUS` `MESS` text calls and CROM0 diagnostic headline,
  bad-ROM-row, operator-instruction, RAM-test start/failure/no-error, and
  CMOS/color/audio/switch-test/monitor-test/audit text transfer plus the
  `HALLOF` / `HOFIN` initials-entry screen.
- `messages.tsv`: source `mess0.src` message vectors, word lists, and
  text-control tokens consumed by
  the translated `BONUS` screen text and CROM0 ROM-test diagnostic headline,
  bad-ROM-row, operator-instruction, RAM-test start/failure/no-error, and
  CMOS/color/audio/switch-test/monitor-test/audit messages, including the
  `IGAMEO` / `ICOLDO` `VINS3` and `VINS8` operator vectors, plus the
  high-score initials-entry player label, instruction lines, gameplay `GO`
  text, attract `CREDV` credits text, and the `TEXTAB` / `TENT`
  instruction-page AMODE message vectors.
- `object-images.tsv`: red-label object image bytes currently needed by
  `COLIDE` / `COL0` picture-mask intersection, `PRDISP` / `ON86` player
  picture writes, and translated `CWRIT` / `COFF` plus descriptor ON/OFF
  output helpers including `OPROC` active-object band writes, plus short-form
  laser, mini-player, and smart-bomb image data.
- `object-pictures.tsv`: red-label complete and short-form object-picture
  metadata currently needed by `BKIL`, `COLIDE` / `COL0`, visible `COLCHK`
  player-collision checks, translated generic picture output/erase, and
  translated `OPROC` object display banding and player-picture output.
- `ram-layout.tsv`: red-label `phr6.src` RAM table bases, entry sizes, counts,
  and field offsets for base-page state, text cursor workspace, runtime
  pointers, laser fizzle bytes, star-map bytes, thrust table bytes, fireball
  table bytes, player data, object
  cells, regular process cells, super-process cells, bank-7 terrain runtime
  bytes, `ALTTBL`, `TERTF0` / `TERTF1`, and `STBL`, including `PLSTR5`
  player-start fields, `SEED` / `HSEED` / `LSEED`, `ASTCNT`, `GTIME`, `PDFLG`,
  `TIMER`, `IRQHK`, `IFLG`, `LSEXPL`, `XSTART`, `DATPTR`, the `ITEMP` /
  `ITEMP2`, `XTEMP` / `XTEMP2`, `TEMP48`, and `XXX1`-`XXX3` scratch bytes
  consumed by the translated IRQ object-phase gate, `PRDISP`, `PLAYER`, and
  object-band tails, plus `PWRFLG` / `WCURS` bytes consumed by `STOUT` /
  `SBLNK`, the
  `CREDIT`, `CUNITS`, and `BUNITS` start/coin accounting bytes consumed by
  `FPLAY` and `START`,
  the full source-order `ELIST` reserve/runtime parameter block through
  `UFOSK`, `ECNTS` active counts through `UFOCNT` used by `PLSAV` and enemy
  kill vectors, the `MONOTB` player-death monochrome descriptor, scanner-object
  `SETAB` / `SETEND` bytes consumed by
  `SCNRV`, `OVCNT` and scanner-terrain `STETAB` bytes consumed by the
  translated `TERBLO` terrain-blow process and `SCNRV`, bank-7
  player-explosion state/table bytes used by `PXVCT` / `PX1A`, the `RAMALS`
  appearance RAM fields used by `APVCT` / `APST` / `EXST` / `EXPU`, and the
  `SSCAN`/`SWP`
  `PIA21`, `PIA22`, `PIA31`,
  and `SWPROC` switch bytes plus `TLIST` / `TPTR` target-list metadata used by
  `PLSTR5`, and `PLAYC` / `NPLAYC` player screen-byte aliases consumed by
  `PRDISP` and `PLAYER`.
- `roms.tsv`: expected red-label ROM filenames, byte sizes, and CRC-32 values.
- `rom-regions.tsv`: MAME ROM region sizes used by verification tooling.
- `rom-map.tsv`: MAME `ROM_LOAD` offsets for red-label main CPU, banked
  program ROM, sound CPU, and decoder PROM images; the `defend.*` rows also
  derive the source-shaped `CROM0` `ROMMAP` descriptor bytes used by
  `ROM0`/`ROM9` checksum and stage reports.
- `routine-addresses.tsv`: assembled red-label routine entry points for the
  translated `SCORE`, `HSRES`, `ADVSW`, `LCOIN`, `RCOIN`, `CCOIN`, `CN1`,
  `SNDLD`, `SHELL`, `BKIL`, `LFIRE`, `LCOL`, `LASR` / `LASR0`, `LASL` / `LASL0`,
  `LASD`, `COLIDE`, `COL0`, `COLCHK`, `REV`, `PLEND` / `PDTHL` / `PDTH2` /
  `PDTH4` / `PDTH5`, `PXVCT` / `PX1A`, `PDTH5R`, `PLE02`, `PLE3`, `PLSTRT`,
  `PLST1A`, `PLSTR3`, `PLS01`, `PLS1`,
  `STCHK`, `ATTR` / `HALLOF`, `AMODES`, `LOGO` / `LOGO0`, `PRES` / `PRES1`,
  `DEFEND` / `DEFENS`, `DEF33`, `DEF44` / `COPYRT` / `CPR55` / `CPR56`,
  `DEF50` / `DEF51`, `WILLIR` / `WILR1`, `LEDRET`,
  `HOFST` / `HOFBL` / `HOFUD` / `HOFUD1`,
  `HALL1` / `HALL3A` / `HALL4` / `HALL5` / `HALL6` / `HALL12` / `HALL13`,
  `HALDIS` / `HALD3` / `HALD4`,
  `CREDS`,
  `FPLAY`, `ST1`, `ST2`, `START`, `START2`,
  `TDISP`,
  `ON28` / `OFF28`, `ON48` / `OFF48`, `ON58` / `OFF58`, `ON34` / `OFF34`,
  `ON23` / `OFF23`, `ON64` / `OFF64`, `ON86` / `OFF86`, `ON66` / `OFF66`,
  `DRTS`, `CWRIT`, `COFF`, `BLKCLR`, `SCRCLR`, `WCMOSA`, `SBOMB`,
  smart-bomb tail resume, `HYPER` /
  `HYP02` / `HYP2` entry/resume labels, `PRDISP`, `PLAYER`, `THPROC`,
  `SCPROC`, `SCP1`, `SCP2`, `OSCAN`, `ISCAN`, `SHSCAN`, `SCNRV`, `BGOUT`,
  `ALINIT`, `BGINIT`, `BGERAS`, `SCLR1`, `PLRES`, `BONUS`, `BC1`, `BC2`,
  `BC3`, `GETWV`, `PDTH5SCLR`, the assembled `GEXBON` return site, `ASTST`,
  `ASTRO`, `ASTKIL`, `TERBLO`, `TBL3`,
  `TBL4`, `COLR`, `COLRLP`, `FLPUP`, `FLP2`, `CBOMB`, `CBMB1`, `TIECOL`,
  `TIECL`, `PRBST`, `PRBKIL`, `RANDV`, `MMSW`, `MSWM`, `MSWLP`, `SWBMB`,
  `MSWKIL`, `SHOOT`, `SCZST`, `SCZ0`, `SCZKIL`, `UFOST`, `UFOLP`, `BGI`,
  `TIEST`, `TIE`, `UFOKIL`, `LKIL1`, `LKILL`, `AFALL`, `AFALL2`, `P250`,
  `P500`, `P503`, `TIEKIL`, `OBJCOL`, and `OCVECT` paths.
- `player-death.tsv`: `PXCTB` glow color table consumed by the translated
  `PDTH2` player-death glow loop and `PXCOL` player-explosion color table
  consumed by the bank-7 `PX1A` loop.
- `score-digits.tsv`: message-ROM `NUMBR0` through `NUMBR9` bitmap bytes used
  by the translated `TDISP` / `SCRTR0` score display path.
- `scores.tsv`: red-label score-card and bonus-stock values.
- `shell-images.tsv`: red-label bomb shell image bytes used by `BMBOUT` when
  `BAX` points at the ROM-resident `BMBD10` / `BMBD11` / `BMBD20` / `BMBD21`
  records.
- `sound-tables.tsv`: red-label sound table bytes currently used by `SNDLD` and
  `SNDSEQ` for coin, replay, player-death, one-player start, two-player start,
  terrain-blow, smart-bomb, bomb-hit, laser, appearance, probe-hit,
  schizoid-hit, swarmer-hit, swarmer-shot, UFO-hit, lander-hit, lander-pickup,
  lander-suck, lander-grab, lander-shot, astronaut-catch, astronaut-scream,
  astronaut-land, UFO-shot, tie-hit, and schizoid-shot sound loads. The Rust
  timeline helper now exposes every embedded table with its label, address,
  priority, repeated command cadence, terminator pointer, and `SNDSEQ`
  sequence-end tick as deterministic TSV rows.
- `sound-direct-command-sequences.tsv`: source-derived `SNDOUT` write rows for
  the direct `PDTH5`, `PLE2`, and `LNDFX0` command callsites.
- `sound-table-command-sequences.tsv`: source-derived `SNDOUT` write rows
  generated from `sound-table-timelines.tsv`, with each table command expanded
  into idle and complemented command writes. The asset tests also verify that
  MAME-observed trace-required start/credit commands from
  `trace-requirements.tsv` are present in these command-sequence fixtures.
- `sound-table-timelines.tsv`: source-derived command and sequence-end rows
  generated from `sound-tables.tsv`, `SNDSEQ`, and `SNDOUT`; the Rust timeline
  helper is tested against this embedded fixture, and the fixture validator
  counts command versus sequence-end rows after the exact comparison.
- `sound-thrust-command-sequences.tsv`: source-derived `SNDOUT` write rows for
  the `SNDSEQ` thrust start and stop gate branches.
- `sram-routines.tsv`: red-label SRAM routine metadata for the 4-bit cell
  packing used by CMOS/high-score reads and writes.
- `switch-table.tsv`: red-label `defb6.src` `SWTAB` switch bit table for
  `SSCAN` / `SWP`, including fire, thrust, smart bomb, hyperspace, start
  buttons, reverse, and altitude-down.
- `terrain-data.tsv`: red-label `blk71.src` `TDATA` terrain bitstream consumed
  by the translated `ALINIT` / `BGALT` altitude-table generator and `BGINIT`
  terrain flavor-table generator, plus `amode1.src` `MTERR` mini-terrain bytes
  consumed by `SCNRV`.
- `trace-scenarios.tsv`: Phase 1 trace scenario names, frame counts, and
  compact cabinet input programs, including the source-CMOS boot wait and
  MAME-observed credited-start prefix used by gameplay reference fixtures.
- `trace-requirements.tsv`: Phase 1 reference-fixture evidence requirements,
  including the MAME-observed credited-start sound commands and events.
- `trace-schema.tsv`: current TSV trace header for fidelity fixtures, including
  internal input bits, MAME IN0/IN1/IN2 input bytes, and optional raw
  object-table, process-table, super-process-table, shell-table, and native
  video-frame CRC-32 values.
- `wave-table.tsv`: extracted source-order `blk71.src` `WVTAB` wave records
  consumed by translated `GETWV` base copies and `WDELT` intra/inter-wall
  updates.

`rom-regions.tsv`, `rom-map.tsv`, `input-ports.tsv`, and `memory-map.tsv`
mirror the red-label ROM, input, and CPU map declarations from the MAME
Williams driver:
<https://github.com/mamedev/mame/blob/master/src/mame/midway/williams.cpp>.
The `rom-map.tsv` main-program rows also derive the 24-byte `romf8.src`
`ROMMAP` descriptor table, checksum reports, and manual/auto ROM-stage outcomes
consumed through `ROM0`/`ROM9` by `romc0.src` `CROM0`:
<https://github.com/mwenge/defender/blob/master/src/romf8.src#L233-L282> and
<https://github.com/mwenge/defender/blob/master/src/romf8.src#L615-L641>.
The CPU address view is cross-checked against the Computer Archeology Defender
RAM-use notes: <https://computerarcheology.com/Arcade/Defender/RAMUse.html>.
`sram-routines.tsv` is derived from the same RAM-use notes for the fixed-bank
`SRAMRead`, `RdSRAMbyte`, `SRAMWordRd`, `SRAMByteRd`, and `SRAMWrite`
routine metadata.
`cmos-layout.tsv`, `ram-layout.tsv`, and `linked-lists.tsv` are derived from
`phr6.src` RAM declarations in the red-label source tree:
<https://github.com/mwenge/defender/blob/master/src/phr6.src>. The shell list
row is cross-checked against `GETSHL` and `SHSCAN` in `defa7.src`:
<https://github.com/mwenge/defender/blob/master/src/defa7.src>.
`cmos-defaults.tsv` is derived from `romc8.src` `DEFALT`, `CMOSMV`, `CMINIT`,
`RHSTD`, and `RHSTDS`:
<https://github.com/mwenge/defender/blob/master/src/romc8.src>.
`audit-adjustments.tsv` is derived from `romc8.src` `AUDITG`, `DISAUD`, and
`MSGAUD`:
<https://github.com/mwenge/defender/blob/master/src/romc8.src#L5-L173>
and
<https://github.com/mwenge/defender/blob/master/src/romc8.src#L716-L774>.
`color-ram.tsv` is derived from the `defb6.src` `CRTAB` records copied by
`defa7.src` `CRINIT` into `PCRAM`:
<https://github.com/mwenge/defender/blob/master/src/defb6.src#L1874-L1890>.
`terrain-data.tsv` is derived from the bank-7 `blk71.src` `TDATA` records
consumed by `BGALT` / `ALINIT` and `BGINIT`, plus the bank-1 `amode1.src`
`MTERR` records consumed by `SCNRV`:
<https://github.com/mwenge/defender/blob/master/src/blk71.src#L375-L399>
and
<https://github.com/mwenge/defender/blob/master/src/blk71.src#L95-L149>
and
<https://github.com/mwenge/defender/blob/master/src/blk71.src#L510-L525>
and
<https://github.com/mwenge/defender/blob/master/src/amode1.src#L1182-L1311>.
`shell-images.tsv` is derived from the `defb6.src` bomb picture records used by
the `defa7.src` `BMBOUT` shell output callback:
<https://github.com/mwenge/defender/blob/master/src/defb6.src#L2072-L2077>.
`object-pictures.tsv` and `object-images.tsv` are derived from the `defb6.src`
complete four-word object-picture descriptors from `SCZP1` through `TEREX`,
the short-form `LASP1`, `PLAMIN`, and `SBPIC` descriptors, and their image
data:
<https://github.com/mwenge/defender/blob/master/src/defb6.src#L1931-L1964>
and
<https://github.com/mwenge/defender/blob/master/src/defb6.src#L2067-L2175>.
`sound-tables.tsv` is derived from the `defa7.src` `RPSND`, `PDSND`, `TBSND`,
`SBSND`, `ALSND`, `AHSND`, `ASCSND`, `APSND`, `LASSND`, `PRHSND`, `SCHSND`,
`UFHSND`, `LHSND`, `SWHSND`, `TIHSND`, `SSHSND`, `USHSND`, and `SWSSND` sound
table records loaded through `SNDLD`:
<https://github.com/mwenge/defender/blob/master/src/defa7.src#L665-L691>.
`switch-table.tsv` is derived from the `defb6.src` `SWTAB` switch bit table
used by `SSCAN` and `SWP`:
<https://github.com/mwenge/defender/blob/master/src/defb6.src#L1843-L1860>.
`color-cycle.tsv` is derived from `defa7.src` `COLTAB` and `defb6.src`
`TCTAB`, which drive the laser, bomb, and tie-fighter pseudo-color cycling
support processes:
<https://github.com/mwenge/defender/blob/master/src/defa7.src#L3035-L3042>.
<https://github.com/mwenge/defender/blob/master/src/defb6.src#L1206-L1209>.
`message-glyphs.tsv` and `messages.tsv` are derived from `mess0.src`
`GO`, `LANDV`, `MUTV`, `SWRMPV`, `BOMBV`, `SWARMV`, `BAITV`, `CREDV`,
`SCANV`, `ELECV`, `BONSX`, `ATWV`, `COMPV`, `PLYR1`, `PLYR2`, `HOFV`,
`HALLD`, the CROM0 diagnostic/RAM-test/CMOS/color/audio/switch/monitor-test
and audit vectors, word records, text-control bytes, `CHRTBL`, and the
character image records consumed by the `BONUS` routine's `MESS` / `WNBV`
calls, the `HALLOF` / `HOFIN` initials-entry screen, the `HALDIS`
hall-of-fame table, and the instruction-page `TEXTP` / `TEXTP2` process:
<https://github.com/mwenge/defender/blob/master/src/mess0.src#L91-L99>.
<https://github.com/mwenge/defender/blob/master/src/mess0.src#L136-L153>.
<https://github.com/mwenge/defender/blob/master/src/mess0.src#L188-L196>.
<https://github.com/mwenge/defender/blob/master/src/mess0.src#L241-L262>.
<https://github.com/mwenge/defender/blob/master/src/mess0.src#L170-L174>.
<https://github.com/mwenge/defender/blob/master/src/mess0.src#L293-L296>.
<https://github.com/mwenge/defender/blob/master/src/mess0.src#L168-L181>.
<https://github.com/mwenge/defender/blob/master/src/mess0.src#L291-L322>.
<https://github.com/mwenge/defender/blob/master/src/mess0.src#L448-L646>.
`routine-addresses.tsv` is derived by assembling `phr6.src`, `defa7.src`,
`defb6.src`, and `amode1.src` with the upstream red-label build recipe, then
recording the `SCORE`, `SNDLD`, `SHELL`, `BMBOUT`, `FBOUT`, `BKIL`, `LFIRE`,
`LCOL`, `LASR` / `LASR0`, `LASL` / `LASL0`, `LASD`, `COLIDE`, `COL0`,
`COLCHK`, `HSRES`, `ADVSW`, `LCOIN`, `RCOIN`, `CCOIN`, `CN1`, `REV`, `PLEND` /
`PDTHL` / `PDTH2` / `PDTH4` / `PDTH5`, `PXVCT` / `PX1A`, `PDTH5R`, `PLE02`,
`PLE3`, `HALL13`, `PLSTRT`, `PLST1A`, `PLSTR3`, `PLS01`, `PLS1`, `STCHK`,
`ASTST`, `ATTR` / `HALLOF`, `AMODES`, `LOGO` / `LOGO0`, `PRES` / `PRES1`,
`DEFEND` / `DEFENS`, `DEF33`, `DEF44` / `COPYRT` / `CPR55` / `CPR56`,
`DEF50` / `DEF51`, `WILLIR` / `WILR1`, `LEDRET`,
`HOFST` / `HOFBL` / `HOFUD` / `HOFUD1`,
`HALL1` / `HALL3A` / `HALL4` / `HALL5` / `HALL6` / `HALL12` / `HALL13`,
`HALDIS` / `HALD3` / `HALD4`,
`CREDS`,
`GEXEC` / `GEX0` / `GEXBON`, `RMAX`, `WVCHK`,
`SBOMB`, smart-bomb tail,
`HYPER` / `HYP02` / `HYP2`, `PRDISP`,
`PLAYER`, `THPROC`, `SCPROC`, `SCP1`, `SCP2`,
`OSCAN`, `ISCAN`, `SHSCAN`, `SCNRV`, `BGOUT`, `ALINIT`, `BGINIT`, `BGERAS`,
`SCLR1`, `PLRES`, `BONUS`, `BC1`, `BC2`, `BC3`, `GETWV`, `PDTH5SCLR`,
`GEXBON`, `ASTRO`, `ASTKIL`, `TERBLO`, `TBL3`, `TBL4`, `COLR`, `COLRLP`,
`FLPUP`,
`FLP2`, `CBOMB`, `CBMB1`, `TIECOL`, `TIECL`, object-picture ON/OFF routines,
`DRTS`, `CWRIT`, `COFF`, `PRBST`, `PRBKIL`, `RANDV`,
`MMSW`, `MSWM`, `MSWLP`, `SWBMB`, `MSWKIL`, `SHOOT`, `SCZST`, `SCZ0`,
`SCZKIL`, `UFOST`, `UFOLP`, `UFOKIL`, `LANDST`, `LANDS0`, `LANDG`, `LANDF`,
`LNDFXA`, `SCZ00`, `AKIL1`, `LKIL1`, `LKILL`, `AFALL`, `AFALL2`, `P250`,
`P500`, `P503`, `BGI`, `TIEST`, `TIE`, `TIEKIL`, and `APVCT` / `APST` /
`EXST` / `EXPU` / `EWRITE`
labels
from the generated listing and ROM byte checks. The
  `PDTH*` labels are recorded for the player-death glow loop and death tail, and
`PX1A` / `PXCOL`
are cross-checked against the bank-7 player explosion source:
<https://github.com/mwenge/defender/blob/master/src/defa7.src#L1328-L1434>.
<https://github.com/mwenge/defender/blob/master/src/blk71.src#L564-L672>.
The source ranges for `SCORE`, `HSRES` / `ADVSW` / `LCOIN` / `RCOIN` /
`CCOIN` / `CN1`, `SNDLD`, `LFIRE` / `LCOL` / `LASR` / `LASL` /
`LASD` / `CRINIT` / `FISS`, `STINIT`, `COLIDE` / `COLCHK`, `PLSTR5`, `SSCAN` /
`SWP`, `SWTAB`, `REV`, `SBOMB`, `HYPER`, `HYP02`, `THINIT` / `THOUT` / `THOFF`,
`FBINIT`, `THPROC`, `SCPROC` / `SCP1` / `SCP2`, `OSCAN`, `ISCAN`, `SHSCAN`,
`SCNRV`, `PRDISP`, `PLAYER`, `GETWV`, `WDELT`, and `APVCT` / `APST` / `EXST` /
`EXPU` / `EWRITE` are:
<https://github.com/mwenge/defender/blob/master/src/defa7.src#L474-L535> and
<https://github.com/mwenge/defender/blob/master/src/defa7.src#L602-L656> and
<https://github.com/mwenge/defender/blob/master/src/defa7.src#L706-L721> and
<https://github.com/mwenge/defender/blob/master/src/defa7.src#L2761-L2902> and
<https://github.com/mwenge/defender/blob/master/src/defa7.src#L2904-L3153> and
<https://github.com/mwenge/defender/blob/master/src/defa7.src#L1241-L1283> and
<https://github.com/mwenge/defender/blob/master/src/defa7.src#L2304-L2476> and
<https://github.com/mwenge/defender/blob/master/src/defa7.src#L759-L805> and
<https://github.com/mwenge/defender/blob/master/src/defb6.src#L1843-L1860> and
<https://github.com/mwenge/defender/blob/master/src/defa7.src#L3155-L3171> and
<https://github.com/mwenge/defender/blob/master/src/defa7.src#L3173-L3209> and
<https://github.com/mwenge/defender/blob/master/src/defa7.src#L3213-L3278> and
<https://github.com/mwenge/defender/blob/master/src/defa7.src#L2073-L2094> and
<https://github.com/mwenge/defender/blob/master/src/defa7.src#L2201-L2302> and
<https://github.com/mwenge/defender/blob/master/src/defa7.src#L2722-L2738> and
<https://github.com/mwenge/defender/blob/master/src/defa7.src#L3280-L3294> and
<https://github.com/mwenge/defender/blob/master/src/defa7.src#L3298-L3373> and
<https://github.com/mwenge/defender/blob/master/src/defa7.src#L2742-L2756> and
<https://github.com/mwenge/defender/blob/master/src/defa7.src#L1846-L1924> and
<https://github.com/mwenge/defender/blob/master/src/samexap7.src#L19-L392>.
