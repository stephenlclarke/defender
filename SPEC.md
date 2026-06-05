# Defender Current Specification

Last reviewed: `2026-06-05`

## Purpose

This repository implements Williams Defender red-label arcade behavior in Rust.
The goal is deterministic, source-shaped behavior that can be checked against
red-label source material, MAME-observed behavior, local trace fixtures, pixel
fixtures, sound command fixtures, and unit tests.

The runtime is self-contained for normal play. ROM files are optional
verification inputs only.

## Source Of Truth

Use these references when behavior is unclear:

1. Red-label behavior observed under MAME.
2. Red-label source and tables from <https://github.com/mwenge/defender>.
3. MAME Williams driver, video, ROM, memory-map, and PIA behavior.
4. Williams sound-ROM source.
5. Williams operator documentation and cabinet references.
6. External behavior analysis and screenshots.

Current Rust behavior is not authoritative unless it is backed by one of those
sources or by an accepted fixture.

## Current Fidelity Contract

The clean runtime is the production runtime. Normal play uses clean `Game`
frames through clean platform, audio, and `wgpu` renderer modules; the converted
machine remains feature-gated behind `legacy-tools` for developer evidence and
comparison tooling. Current Rust behavior is accepted only when backed by
red-label source/MAME evidence or by the accepted fixture/report gates.

Earlier owner review rejected the clean media for concrete gameplay-facing
fidelity defects: player laser shape, reverse-facing player orientation, and
live sounds. Those known defects have since been repaired against local MAME
red-label evidence. The clean runtime now renders player lasers as sparse
source-byte body/tip/fizzle pixels, projects a distinct left-facing player ship
on reverse, and decodes live-audio events through translated Williams
sound-board command families.

The local reference workflow is the acceptance path:

- `make reference-mame-capture` records local red-label MAME MP4/WAV artifacts
  and trace TSVs under ignored `target/reference-media/` paths.
- `make reference-mame-smoke` runs the short two-second MAME recorder proof
  used by the release gate.
- `make reference-clean-capture` renders clean GIF/WAV/debug candidates from
  the same input-program grammar as the MAME capture path.
- `make reference-media-check` compares bounded MAME and clean windows with
  explicit visual, audio, or all-axis acceptance modes.
- `make reference-report-gate` validates the accepted report manifest,
  semantic coverage requirements including per-facet `min_reports` breadth
  floors, manifest uniqueness for report names/paths and coverage keys,
  explicit matching manifest/report `acceptance_mode` declarations, non-empty
  declared coverage tags compatible with each report's acceptance mode,
  report/media artifact roots under generated `target/reference-media` paths,
  accepted MP4/GIF/WAV media types, non-empty media artifacts, positive
  frame/sample counts, and absence of stale verifier failures on accepted axes.
- `make reference-evidence-package` regenerates the reference-window scan JSON
  files and the deterministic owner-review summary, including each accepted
  report's manifest and local `report.json` acceptance modes plus
  sound/object near-miss diagnostics, per-family object counts, longest
  object-evidence spans, best spans per object family, and terrain-process
  near-miss diagnostics from the current MAME trace corpus.
- `make owner-review-package` regenerates the evidence package, re-runs the
  accepted-report gate, and prints the finite owner-review checklist.

The accepted report gate covers sprite visuals, player laser visual/audio,
reverse orientation, explosion and coalescence visuals, terrain blow,
gameplay audio families, non-lander audio and visual presentation,
playability, rescue/loss, death/respawn, smart bomb, hyperspace, and organic
non-lander presentation. Each coverage facet locks the current accepted report
breadth so future manifest edits cannot reduce broad MAME-vs-clean proof to a
single passing clip without tripping the gate, and duplicate manifest entries
cannot count the same report or coverage row twice. Report coverage tags must
also map to declared objective facets and to an explicitly declared
visual/audio/all acceptance mode that can prove that facet; the local accepted
`report.json` file must declare the same mode. Accepted report JSON stays under
`target/reference-media/`, MAME reference artifacts stay under
`target/reference-media/mame/`, and clean candidate artifacts stay under
`target/reference-media/clean/`. Accepted visual proof uses MP4 MAME reference
clips and clean GIF candidates; accepted audio proof uses WAV reference and
candidate files. The current `make release-gate` path includes the
owner-review package, accepted report gate, MAME doctor, MAME smoke recorder,
README media generation, game smoke, actor smoke gates, live smoke, docs lint,
and diff hygiene.

The accepted gate is still green, but final closure is no longer only an
owner-review decision. A fresh organic smart-bomb/up-thrust search found a
concrete last-human terrain-blow MAME candidate outside the accepted evidence
windows; the current clean candidate is silent and visually divergent. The
probe and review checklist are tracked in
`docs/fidelity/release-closure-audit.md`.

Post-R9 non-rewrite follow-ups are evidence and polish items, not active R9
blockers: exact per-scenario pixel CRC parity, strict long-scenario sprite
count/layer parity, per-scenario offscreen `wgpu` signatures, and optional
local MAME/reference trace refreshes where local ROM inputs are available.

## Implementation Rules

- Implement red-label Defender behavior, not a Defender-like approximation.
- Cite source routines, tables, MAME behavior, or fixtures when adding
  behavior-sensitive code.
- Keep runtime assets in `assets/red-label/` and embed them with
  `include_str!` or `include_bytes!`. Sprite files must stay under
  `assets/sprites/`, new non-legacy sound artifacts must stay under
  `assets/sounds/`, and pre-existing legacy `.wav` cues must remain under
  `assets/arcade/` unless explicitly reclassified with stronger provenance.
- Keep local ROMs and generated MAME reference traces out of normal runtime
  requirements.
- Preserve exact source-visible mutations in tests: RAM, CMOS, video RAM,
  palette RAM, process lists, object lists, shell lists, scheduler state,
  sound commands, and snapshots.
- Keep Planetoid controls and `XYZZY` outside the arcade core as explicit
  compatibility layers.
- Do not guess unknown arcade behavior. Record a gap and add a focused test or
  fixture path before implementation.
- Maintain the 80% project line-coverage floor and keep added executable Rust
  lines covered.

## Current Architecture

The crate is now split between a clean rewrite source tree and a legacy oracle
tree:

- `src/main.rs`: thin CLI entry point that still dispatches to the runtime
  bridge through the clean platform launcher while the rewrite takes over.
- `src/lib.rs`: clean public crate wiring plus explicit `legacy-tools`
  adapters to the legacy oracle tree and a doc-hidden README media facade wired
  from `src_legacy/readme_media.rs`. Default production builds do not compile
  the accepted machine, legacy live core, CMOS storage, or retired `wgpu`
  presenter. Machine process/state contracts, red-label math types, and
  low-level asset, board, memory, ROM, sound, PIA, and video modules must
  remain feature-gated and crate-private. Generated long-trace sample fixtures
  must stay private to the legacy machine oracle instead of being root-wired
  here.
- `src/accepted.rs`: crate-private accepted-behavior contracts and facade that
  isolate clean oracle code from direct legacy imports while carrying neutral
  source-backed high-score, wave-profile, game-over timing, object-list, and
  bounded object-position/detail plus object-picture descriptor and mapped clean
  sprite evidence for the reclassified player/projectile/human/enemy/display,
  reward-picture, null, astronaut-explosion, and terrain-explosion bridge plus
  expanded-object appearance/explosion/score-popup slot evidence, descriptor
  sizes, score-popup source lifetime/value metadata, and expanded-object
  explosion frame/lifetime metadata plus player-death bank-7 pixel-cloud
  color/counter/piece evidence and terrain-blow mutation/process evidence used
  by fidelity checks and bounded sprite projection.
- `src/game.rs`: gameplay-facing `Game`, `GameState`, `GameInput`,
  `GameFrame`, `GameEvents`, world, terrain, starfield, enemy, human, score,
  source-backed wave profile, high-score table/session state, isolated
  initials-entry surfaces, game-over return timing, neutral object evidence,
  clean object categories, mapped clean sprite evidence, source-backed
  human/enemy/projectile object-picture descriptors,
  expanded-object lifecycle/presentation evidence, source-backed score-popup
  lifecycle snapshots, source-backed expanded-object explosion lifecycle
  snapshots, source-backed terrain-blow snapshots, projectile, player,
  direction, two-player start
  admission, source-backed score digit scene sprites, player-count stock
  HUD scene sprites, current-player score/bonus stock synchronization,
  two-player final-life switch/respawn
  state, two-player start prompt, player-switch, final game-over,
  player-death pixel-cloud snapshots, and attract presentation page-frame state
  for the Williams, presents, Defender wordmark,
  copyright wait, and instruction surfaces, attract credit/presents/instruction
  message-glyph scene sprites with source message row-feed/horizontal-cursor
  controls, wave-completion status message-glyph and survivor bonus icon scene
  sprites, isolated high-score-entry prompt message-glyph scene sprites,
  hall-of-fame display heading/table scene sprites, and sound-event contracts,
  including source-backed per-family enemy hit command bytes surfaced through
  `UnmappedSoundCommand`.
  The clean
  `Game` shell emits sprite-first scene frames without touching the accepted
  machine adapter.
- `src/actor_game.rs`: isolated actor-oriented rewrite surface with
  thread-backed assets, `StepPrompt`/`StepReport` simulation turns,
  scriptable behavior and wave profiles, two-player admission/session
  snapshots, bounded source start-playfield delay reports, source-cadenced
  start/appearance sound cues, two-player player-switch sleep reports,
  draw/effect descriptions, and `SoundCue::source_sound_command` metadata for
  red-label Williams sound-board command bytes where source evidence exists.
  `ActorSoundEventBridge` adapts
  report sound cues into clean `SoundEvent` batches for the existing audio
  runtime contract. `ActorRenderSceneBridge` adapts report draw commands into
  clean `RenderScene` sprites for the existing native renderer contract, and
  `ActorStateBridge` adapts report phase, current player, player count,
  per-player scores/stocks, high-score state, and actor snapshots with
  velocity/facing plus hostile-projectile source metadata into clean
  `GameState` values. Source-backed mutant actors consume
  source-shaped conversion metadata, wave-table mutant velocity/random-hop
  rows, driver-provided source RNG snapshots, and actor-owned shot timers to
  emit `0xF6` source-shaped hostile projectile commands. First-wave target6
  converted-mutant metadata now preserves the source conversion X correction,
  projected draw/collision anchors, deferred first-shot state, target6 dive
  shot-position overrides, and exact fire2524 projectile fractions/velocities
  in the actor source path. Actor player/enemy collision now also preserves the
  target6 fire2524 wait window, projected collision position,
  source-positioned enemy explosion, hit cue, and score award before routing
  through the normal player death command path.
  `ActorRuntimeAdapter` bundles reports, clean `GameState`, clean `GameEvents`,
  and clean `RenderScene` values into actor `ActorFrame` values. Normal
  interactive play uses the actor live runtime, with explicit actor smoke and
  WGPU smoke commands retained as evidence gates.
- `src/actor_smoke.rs`: the crate-private actor smoke command that steps
  `ActorRuntimeAdapter` through scripted attract/play inputs, verifies
  actor-origin clean gameplay/audio events, required actor sprite coverage,
  projectile/HUD/overlay layers, native draw-command pipeline coverage, and
  frame-level `wgpu` command plans. It also owns `--actor-attract-smoke`, which
  advances the default no-input actor attract script through Williams reveal,
  Defender coalescence, Hall of Fame, scoring surface, final scoring label, and
  the `cycle 3367` return while verifying native draw plans and no attract
  gameplay/audio events. It also owns `--actor-post-game-smoke`, which drives
  actor play through three real pod/player collisions, a qualifying high-score
  initials submission, the 60-step Hall-of-Fame game-over stall, and return to
  the Williams attract reveal while verifying actor events, bridged sound,
  native draw plans, and sprite atlas coverage.
- `src/live_wgpu.rs`: also owns `--actor-wgpu-smoke`, which reuses the actor
  smoke input sequence, renders actor `RenderScene` frames through the offscreen
  `wgpu` texture/readback path, and checks nonblank dynamic readback evidence
  for the actor runtime. It also owns default interactive actor live play and
  the explicit `--actor-live` alias, which step `ActorRuntimeAdapter`,
  submits actor-derived clean `GameFrame` values to the live audio runtime, and
  draws actor scenes with the existing `wgpu` presenter. The actor live input
  path carries initials/backspace into actor high-score entry instead of
  dropping those keys.
- `src/game_smoke.rs`: the crate-private clean game smoke command that steps
  `Game` through scripted controls, verifies sprite plus native pipeline and
  draw-instance coverage, verifies sprite buffer upload-plan, render-pass plan,
  and frame-command evidence, and prepares emitted scenes with the native
  renderer draw planner.
- `src/systems.rs`: deterministic fixed-step timing utilities, clean
  player-control intent/trigger systems, operator trigger handling,
  player-motion, enemy-motion, projectile launch/capacity/motion systems,
  projectile/enemy collision detection, player/enemy damage resolution,
  smart-bomb resolution, wave-completion evaluation, score/bonus awards,
  high-score table helpers, isolated initials-entry handling, and the
  `GameSimulation` trait used by the clean game and internal oracle
  implementations.
- `src/renderer.rs`: native `wgpu` scene contracts, surface sizing, sprite
  layers, temporary raster evidence, renderer-owned resources, atlas-backed
  sprite batches, sprite quad geometry, sprite instance buffers, the sprite
  instance GPU ABI, sprite instance upload streams, sprite draw commands,
  sprite `wgpu` buffer upload plans, sprite render-pass plans, sprite pipeline
  plans, sprite resource binding plans, sprite atlas texture upload plans,
  sprite pipeline layout plans, sprite render pipeline descriptor plans,
  sprite render-pass encoder command plans, frame-level GPU command plans,
  viewport layout, GPU pass planning, scene summaries, render signatures, and
  draw planning.
- `src/platform.rs`: the clean runtime launch boundary plus configuration for
  controls, audio, run mode, and persistence.
- `src/runtime.rs`: the crate-private launch bridge that translates clean
  runtime configuration into launch commands and routes config-driven `wgpu`
  live and smoke launches.
- `src/live_wgpu.rs`: the crate-private WGPU live launcher that owns the
  `winit` event loop, `wgpu` surface/device lifecycle, clean input mapping,
  clean `Game` stepping, clean audio event submission, native sprite draw-plan
  execution, and the clean `Game` frame source plus offscreen `wgpu`
  render/readback evidence for `--live-smoke`.
- `src/roms.rs`: the crate-private optional ROM verification facade that owns
  the temporary ROM metadata, scan, and loader bridge.
- `src/audio.rs`: gameplay-facing `SoundEvent` batches, the live audio worker
  boundary, disabled/null no-device modes, and runtime diagnostics. It consumes
  clean `GameFrame` and `SoundEvent` contracts, not legacy frame outputs.
- `src/sound_board.rs`: translated Williams sound-board synthesis and command
  decoding for source GWAVE vectors, VARI sweeps, special routines, organ
  routines, and thrust-loop playback.
- `src/fidelity.rs`: clean frame-equivalence signatures for gameplay state,
  gameplay events, sound events, and render summaries. Clean fidelity tests use
  oracle-owned reference probes instead of importing accepted facade types
  directly.
- `src/clean_fidelity.rs`: test-only clean rewrite harness that steps the real
  clean `Game` and accepted oracle with the same scenario input streams, then
  emits first-divergence TSV reports for R0 and later scenario gates.
- `src/fidelity_manifest.rs`: the crate-private fidelity scenario manifest
  facade that owns temporary scenario metadata and input expansion.
- `src/fidelity_trace_engine.rs`: the crate-private fidelity trace engine
  facade that owns temporary trace generation, comparison, and schema access.
- `src/oracle.rs`: the crate-private gameplay oracle, including clean state,
  event, sound, source object-list/object-position/object-picture evidence,
  expanded-object lifecycle/presentation evidence including score-popup
  lifetime/value metadata, and scene-summary frames from the accepted-behavior
  facade for internal fidelity comparison.

The converted implementation is parked under `src_legacy/`. It still owns the
accepted arcade behavior, hardware models, ROM verification, rendering, input,
sound-board command evidence, legacy fidelity trace generation, and test
helpers for explicit `legacy-tools` builds. Historical live, CMOS, and retired
`wgpu` presenter code remain parked there but are no longer compiled from
`src/lib.rs`. Those root adapters are feature-gated and crate-private. Clean
runtime launch goes through the private `runtime` bridge, while the internal
oracle uses the crate-private `accepted` facade when `legacy-tools` is enabled.
`src_legacy/accepted_behavior.rs` performs the current legacy-machine
adaptation into neutral accepted-behavior contracts before the public clean
gameplay types see it. `src/live_wgpu.rs` owns clean config-driven interactive
`wgpu` launches and routes `--live-smoke` through clean `Game` smoke frames.
`src/roms.rs` owns the temporary ROM metadata, scan, and loader bridge for
optional verification commands. `src/fidelity_manifest.rs` owns the temporary
scenario manifest and input expansion bridge for fidelity scenario commands.
`src/fidelity_trace_engine.rs` owns the temporary trace generation, comparison,
and schema bridge for fidelity trace commands. ROM, scenario, trace, and
oracle modules are explicit `legacy-tools` developer tooling rather than
default production runtime wiring. Legacy-specific clean equivalence
regressions are also wired from `src_legacy/` under that feature so clean
accepted/oracle source stays focused on gameplay contracts. Internal clean
equivalence regressions use crate-private oracle wiring. Clean frame-signature
gates live under `src/fidelity.rs`, compare clean render signatures rather than
exposing memory-oriented CRC labels, and use oracle-owned reference probes for
accepted behavior comparison when `legacy-tools` is enabled. Temporary README
media tooling uses the doc-hidden `defender::readme_media` facade rather than
low-level legacy module exports, and only through `legacy-tools`. Machine
process/state contracts remain crate-private oracle wiring. Live presentation
receives clean `RenderScene` data. Native draw
planning resolves scene sprites through renderer-owned atlas regions into
sprite batches and records GPU instance-buffer data with native scene
rectangles, normalized atlas UVs, normalized tint, stable record counts and
upload bytes, and the `wgpu` vertex layout for the instance buffer.
`--game-smoke` steps the clean game through scripted controls, verifies
required gameplay sprite layers, sprite IDs, native draw-command pipeline and
instance coverage, sprite buffer upload-plan coverage, render-pass plan
coverage, and frame-command sprite command/draw/instance plus ordered
sprite-only begin-pass, viewport, and projection upload coverage, and prepares
sprite-only native draw plans plus frame-level `wgpu` command, resource
bind-group, pipeline-layout bind-group, pipeline descriptor shape, encoder
command-shape, and upload plans without entering the legacy live presenter. The
interactive live path executes clean sprite draw plans through `wgpu` buffers,
bind groups, and indexed draws.
clean `Game` world seeds terrain, starfield, source-profile active enemy
batches, human, and projectile snapshots for playing waves and renders them as
atlas-backed scene sprites.
`--live-smoke` reuses that clean frame source, renders it through an offscreen
`wgpu` target, reads back pixel signatures, checks selected first/last
signatures, and reports
`frame_source: clean_game`, `legacy_presenter_used: false`, sprite counts,
temporary-raster counts, and offscreen render evidence.
Operator controls are
sampled through `OperatorControlSystem`, emitting diagnostics, audits, and
high-score reset gameplay events on button edges while preserving current
player scores and bonus thresholds during high-score reset. Clean enemy
snapshots carry gameplay-domain velocity and advance through
`EnemyMotionSystem` during playing frames. Clean projectile snapshots carry
source `LASR0` / `LASL0` laser-loop motion: five source screen columns per
step with source edge-stop bounds before rendering. Clean collision boxes resolve
projectile/enemy hits through `CollisionSystem`, remove the hit entities from
world state, and award score through `ScoreSystem` before rendering. Crossing
the clean bonus threshold updates player stock and emits `BonusAwarded`. Clean
wave state carries source-backed enemy counts, active wave size, velocities,
shot timers, and baiter timing from the checked-in wave table. The clean active
wave batch uses that profile to spawn source-exposed lander, bomber, and pod
families with family-specific sprite, collision-size, and score mappings;
initial wave landers retain deterministic source fixed-point fractions, shot
timers, picture frame, and X/Y velocity, then advance through a bounded source
`LANDS0` orbit/shot loop with picture cycling and source-shaped fireball
projection; initial active pods retain deterministic source fixed-point
fractions and bounded signed X velocity before entering source fixed-point X/Y
motion. Active source enemy Y motion now uses the source `VELO` `YMIN`/`YMAX`
wrap for landers, pods, bombers, mutants, swarmers, and baiters. Remaining
source-profile enemies stay in `EnemyReserveSnapshot`, flow
into inactive object-evidence counts plus bounded inactive source-detail rows,
and activate as the next clean batch before `WaveCleared`. The inactive rows
carry the reserved family category, source object-picture descriptor,
deterministic source object-table identity, mapped clean sprite, and source
scanner color while position and velocity remain empty until activation.
Reserve landers now use source `LANDST` placement, fixed-point fractions,
shot-timer RNG, and velocity bytes, then enter the same bounded `LANDS0`
runtime; when no humans remain, the same reserve lander path follows the
source `LANDST` schizoid fallback and restores source-shaped mutants directly.
Destroyed pods now spawn a deterministic clean
mini-swarmer batch using the source spawn request bound and active-swarmer cap,
including projectile and smart-bomb destruction paths. Spawned mini-swarmers
carry source RNG-derived velocity, acceleration, sleep, and shot-timer state,
and reserve mini-swarmer activation now uses source `PLRES`/`RSW0`
phony-object placement before the same source swarmer runtime. Reserve pod
activation now uses source `PRBST`/`PRBRES` placement, fixed-point fractions,
and velocity bytes before entering source fixed-point X/Y motion with the same
source `VELO` Y-bound wrap. Mini-swarmers advance through the source entry
seek, fixed-point loop, vertical
acceleration/damping, turnback, and enemy-bomb projection shape while sharing
the source shell free-list cap with the other fireball paths, including source
`RMAX` RNG consumption on shot-timer resets when allocation fails, and emit
source `SWSSND` command evidence when allocation succeeds. Clean baiter entry
now advances on the source
game-exec pacing cadence, accelerates the timer when the remaining source
wave-enemy total is low, excludes active baiters from that source `WVCHK`
count, and respects the source active-baiter cap. Active baiters therefore do
not block reserve activation or wave clear when no source-counted enemies
remain. Spawned baiters
retain source shot-timer, picture-cycle, sleep, and velocity state, pursue the
player through source seek rules, fire source-shaped fireball shells with
source `USHSND` command evidence, and those enemy projectiles use source
`SHSCAN` lifetime decrement/wrap behavior, scroll-adjusted fixed-point motion,
offscreen culling, collision scoring, and player-damage handling with source
`BKIL` / `AHSND` command evidence when a shell hits the player plus source
`PLEND` / `PDSND` command evidence when the player-hit path starts. Accepted
clean hyperspace inputs clear active enemy projectiles through the source
`HYP02` / `KILSHL` shell-list cleanup while leaving player projectiles outside
that shell-object list. They then reload source rematerialization state from
the current clean source `SEED`/`HSEED`: the clean camera/background word,
player X/facing branch, player Y high byte, cleared velocity, and source
`APSND` appearance command. Their clean `HYP2` tail follows the source
`LSEED > 0xC0` death-risk branch into the existing player damage path with
source `PDSND` command evidence, while `0xC0` and below complete safely.
Clean landers now abduct aligned humans, can carry explicit selected-human
target state for source-shaped landers, enter the source `LANDG`
target-approach step only when that selected target passes the source `LANDS0`
close-X check, seed source-shaped `LANDG` flee vector/sleep state on capture,
keep carried passengers associated with the lander that captured them while it
flees, pull the passenger upward through the source `LANDF` / `LNDFXA`
top-edge shape before conversion, and release the passenger when that lander is
destroyed, including source `ASCSND` command evidence when the passenger is
released. Pickup now emits source `LPKSND` command evidence, and the first
top-edge pull-in frame emits source `LSKSND` command evidence. Source landers
already in the pull phase give up and return to reserve if the passenger
target is cleared. Initial clean humans now restore the source `PLRES` /
`TLIST` startup shape: ten target-list humans are placed
through the source grouping rules and retain slot addresses from the `0xA11A`
target-list base with a two-byte stride. Clean source lander spawns now advance
the source `TPTR`-shaped cursor through those `TLIST` slots for initial,
reserve, and retarget selection while preserving the separate source enemy RNG
cadence. Source lander fireball allocation now emits `LSHSND` command
evidence.
Released, uncarried humans
above terrain now use source-shaped `AFALL` fixed-point acceleration, settle
safely at or below the source threshold with the 250-point safe-landing score
and existing `P250` score-popup lifecycle plus source `ALSND` command evidence,
or die on over-speed impact with an astronaut explosion, source `ASTKIL` /
`AHSND` command evidence, and the existing last-human planet-loss handoff that
starts source `TERBLO` / `AHSND` terrain-blow command evidence;
falling humans caught by the player enter the clean player-carried state,
award the source-backed 500-point rescue score, emit source `ACSND` command
evidence, and start the existing `P500` score-popup lifecycle; player-carried
humans settle on terrain when the player-carried offset reaches the local
terrain line.
Completed carried-lander abductions now consume the pulled-in passenger and
convert the lander into a source-shaped mutant. No-target/no-human landers
enter the same mutation path, and active clean mutants retain source
shot-timer, sleep,
fixed-point fractions, X seek, vertical seek/avoid, random Y hop, and shared
fireball projection state; reserve mutants now restore through source-shaped
placement fractions, shot-timer RNG state, and source `SSHSND` command
evidence when shared `SHOOT` shell allocation succeeds. Clean bombers now
retain source fixed-point fractions, X velocity, vertical velocity, picture
frame, cruise altitude, and sleep state, then advance through source `TIE`
image cycling, random vertical drift/damping, player-Y steering, off-screen
cruise steering, and bounded silent `BOMBST` bomb-shell projection with source
`GETSHL` placement bounds, the separate `BMBCNT` ten-bomb cap, and the total
20-cell source shell-list cap. Bomber picture/Y/bomb state updates now honor
the source `TIE` `SEED & 0x06` four-slot squad selection from persistent
source slots, leaving killed or empty selected slots sleeping while active
bomber positions continue through source velocity. Reserve bombers now use
source `TIEST` four-slot player-relative squad placement and alternating X
velocity before entering the source bomber runtime. Enemy projectile evidence
now distinguishes source `FBOUT` fireballs from source `BMBOUT` bomber bomb
shells and carries source `BMBP1` shell descriptor fields for the standalone
mine/source-shell fixture. Enemy-projectile/player collision uses the source
`BMBP1` 2x3 footprint while the direct runtime projectile renderer keeps the
existing 4x6 bomb sprite. Active clean enemy evidence now carries source
object-picture descriptor labels, addresses, dimensions, and primary/alternate
image pointers for the current lander, baiter, bomber, mutant, pod, and
swarmer presentations, and clean projectile/enemy plus player/enemy collision
uses those source enemy picture sizes while direct runtime enemy rendering
keeps the current clean sprite sizes. Clean hostile player collision uses the
source `PLAPIC` / `PLBPIC` 8x6 player picture footprint while the direct
runtime player renderer keeps the current 16x6 ship sprite; falling-human
rescue collision uses that player footprint plus source `ASTP1`-`ASTP4` 2x8
astronaut footprints while direct runtime human rendering keeps the current 4x8
sprite. Clean
human evidence carries per-human source astronaut picture descriptors: default
`ASTP1` rows and source-restored `ASTP3` rows selected from the `PLRES`
`LSEED` low bit, with restored `LSEED` X low bytes retained as source X
fractions for world-position evidence. Clean worlds also carry source `ASTRO`
process cursor/sleep state that walks one restored, uncarried target-list human
per source cadence, applies source fixed-point X motion, steps Y toward
terrain-relative source targets, and cycles evidence from `ASTP1` through
`ASTP4`. Clean player projectile evidence now carries the source `LASP1`
descriptor label, address, 8x1 size, and primary image pointer while the direct
runtime projectile renderer projects each live shot as a red-label
tail-to-tip laser span. Clean player projectiles advance through the source
`LASR0` / `LASL0` five-column loop step across the full clean playfield, keep
the trailing beam moving at the source one-column erase cadence, and use the
leading 16x1 `LASP1` hit span for enemy collisions.
Clean enemy, human, player-projectile, and enemy-projectile object evidence
also carries source-style 8.8 world-position words, velocity words, and
deterministic source object-table identity evidence from the clean source
fixed-point state and source layout: addresses from `0xA23C` plus `0x17` per
slot, source slot numbers, and neutral `OTYP` `0x00`. Runtime scene sprites
remain on the direct clean render path.
The R9-C4 residual ecology audit now classifies the per-family
movement/projectile runtime surfaces as covered by the current clean runtime
and focused unit tests, and targeted source ecology fixture hardening matches
the current `start_game`, `smart_bomb`, `hyperspace`, `abduction`, and `death`
clean-fidelity scenarios. The R9-C4.5
closure gate closes Step 50/B08 without exposing drift, so R9-C4.2 stayed
unused. Step 51/R9-D1 closes B09 two-player flow with final-life switch,
non-final death-rotation, stock/score ownership, and current-player final
game-over/Hall-of-Fame routing fixtures.
Clean
smart bombs consume player stock, clear active enemies through
`SmartBombSystem`, route score through the same scoring system, and leave
destroyed active enemy sprites absent from the scene while source reserves can
enter as the next active batch. Successful clean laser launches emit source
`LASSND` command evidence, and accepted smart-bomb inputs emit the first source
`SBSND` command before enemy destruction sounds; accepted thrust inputs emit
the source start/stop sound events on press and release. One-player credited
starts consume one credit,
while two-player starts require and consume two credits, set
`player_count` to two, enter play as player one, and expose the player-one and
player-two top-display fields immediately. Scenes draw player-one and player-two
score fields from source-backed digit sprites with six-position leading-zero
blanking, and playing scenes draw player-one and player-two life-stock and
smart-bomb-stock HUD sprites with source-backed display caps, positions, and
the reclassified stock sprite targets. Playing scenes also draw the source
`BORDER` top-display frame geometry as clean HUD sprites: the bottom display
line from `0x0028`, scanner side boundaries from `0x2F08` and `0x7008`, the
top scanner boundary from `0x2F07`, and scanner marker bars from `0x4C07` and
`0x4C28`. Playing scenes also draw scanner/radar object and player blips from
source-backed scanner state: `SCPROC`/`SCP1`/`SCP2` cadence, selected scanner
map `1`, scan-left calculation, object erase-table addresses, source `SETEND`,
player blip bytes, and `OBJCOL` scanner colors. Scanner terrain-raster
residuals for normal playing, live top-display scheduling, and exact hardware
palette/RGB
rendering remain separate render-parity work. HUD and border tints are routed
through the source visual-state contract that records the `0x5555` border words
and `0x9999` scanner-marker words while preserving the current clean sprite
output. Bounded source object-detail rows that carry
`screen_position`, `picture_size`, and a mapped clean sprite are projected into
scenes: active rows draw on the object layer, projectile rows draw on the
projectile layer, and inactive or transparent null-object rows stay
evidence-only. Clean reserve counts and refills are represented in clean world
state; abduction/rescue transitions and object lifecycle timing remain separate
work. Bounded source expanded-object
appearance/explosion/score-popup detail rows that carry `top_left`, descriptor
size, and a mapped clean sprite are projected onto the object layer;
missing-size and transparent null-object rows stay evidence-only. Live spawned
enemy appearance rows can render as source-picture coalescence pixels while the
appearance size contracts from the MAME-observed high-bit `RSIZE` range toward
the final sprite size, and the normal enemy sprite is hidden during that active
appearance window. Score-popup
rows additionally carry source 50-tick lifetime and 250/500 value metadata;
enemy-family explosion rows carry source `EXST`/`EXPU` frame/lifetime metadata
and draw as sparse source-style pixel clouds around their mapped sprite center
until the source kill threshold. Player death starts a source-backed bank-7
pixel-cloud snapshot
from `PXVCT`/`PX1A` state, carrying source color/counter metadata and visible
piece positions into clean/oracle scenes. Planet destruction starts a
source-backed terrain-blow snapshot: terrain rows are removed from the clean
scene, scanner terrain is disabled, `TEREX` explosions are projected through
expanded-object evidence with the MAME-observed terrain flash windows and
terrain-explosion growth cadence, and the source status bit, iteration,
erase-table counts, pseudo color, overload counter, source `TERBLO` / `AHSND`
entry command evidence, and source `TBSND` completion command evidence are
carried for fidelity checks.
Enemy contact with the player is
resolved through clean collision and
`PlayerDamageSystem`,
decrementing lives, removing the colliding enemy, and entering `GameOver` on
the final one-player life. In two-player games, a final-life death for the
active player enters the source-backed `PLE02` `0x60`-tick switch sleep when the
other player still has stock, then hands off to that player and starts the clean
playfield entry path. During that switch sleep, scenes draw source-backed
`PLYR1`/`PLYR2` and `GO` message glyphs at the red-label screen addresses
`0x3C78` and `0x3E88`; if no player has remaining stock, the clean game enters
the final game-over return path. Non-final deaths with remaining stock enter a
death-cloud pause and then respawn the next stocked player through the same
clean player-start path; two-player games rotate to the other stocked player per
the source `PLE02` loop, and one-player games wrap back to player one. After
rotation, score and replay bonus awards sync the active player's public stock
snapshot, so player-one and player-two scores, lives, and smart-bomb stocks stay
owned by the active player. The second-player final-life switch-back path follows
the same player-start cadence for player one, and final two-player game-over
uses the same no-entry game-over/Hall-of-Fame return when no other player has
stock. During that final game-over sleep, scenes draw
source-backed `GO` message glyphs at the translated `PLE2` screen address
`0x3E80`. The player-death pixel cloud is cleared before the final attract
handoff so Hall of Fame scenes remain table-only.
MAME red-label evidence does not show automatic initials entry after a
qualifying final score, even when all all-time high-score slots are seeded to
zero. The clean runtime therefore returns through the no-entry
Hall-of-Fame/game-over path after final death. `HighScoreEntrySystem` remains
covered as an isolated table/editing surface: it accepts alphabetic initials,
normalizes them to uppercase, supports backspace, emits
`HighScoreInitialAccepted`, and emits `HighScoreSubmitted` when the third
initial enters the source-shaped hall-of-fame display stall before returning
to attract. Manual submissions insert into both all-time and
today's-greatest tables by score rank while preserving the current-player
submission metadata. Normal attract scenes draw the
source-backed `CREDV` credits label at `0x28E5` and the visible credit count
digits at `0x48E5` during Hall of Fame / scoring contexts and whenever a real
credit exists; the zero-credit line is suppressed on the Williams title pages.
The title program is gated by
`AttractPresentationSnapshot`: the source `LGOTAB` Williams logo appears first
at `0x363C`, `ELECV` presents copy appears at `0x3258`/`0x3E6C`, the Defender
wordmark coalesces through the 15 source `DEFENS` 4-byte by 12-row `APVCT`
appearance slots starting at object table `0xB304`, picture table `0xB3D6`,
data `0xB412`, and screen address `0x3090` before the full `0x3C` by `0x18`
wordmark descriptor refreshes, the source copyright strip appears at `0x3BD0`
during the copyright wait gate, and the
instruction-page labels use the source scoring-card cadence: `SCANV` appears
first at `0x4330`, then `LANDV`, `MUTV`, `BAITV`, `BOMBV`, `SWRMPV`, and
`SWARMV` reveal at `0x1C70`, `0x3C70`, `0x5F70`, `0x1CA8`, `0x40A8`, and
`0x5CA8`. After the hall-of-fame display stall, the
scoring/action page replays the source instruction-page rescue sequence,
scanner blips, player laser, 500-point rescue popup, and `ENMYTB` enemy
score-card reveal before cycling back to the Williams page. This projection is
suppressed during the hall-of-fame display stall. README media captures now
preserve sampled frame cadence and match the protected reference duration and
frame count. The Williams handwriting reveal advances by source `LOGO0`
table-operation slices and uses source `TCTAB`/Williams resistor palette title
colors; `ELECV` title text uses source `COLTAB` cycling. The Defender wordmark
appearance now uses source `APVCT` row-pair coalescence over the `DEFENS`
chunks while staying sprite-first in the clean scene. Hall of Fame display and
credit text now use the protected-reference `COLTAB` `0x47` magenta and
protected-reference display offset while entry/blink evidence stays on `0x85`.
The attract scoring page applies that protected-reference offset to
scoring text/credits/scanner/terrain surfaces, projects source `MTERR`
mini-terrain records inside the top scanner box, and uses the
protected-reference purple scanner border tint. The clean scoring action starts
from the protected reference's post-setup rescue-fall phase and scoring
text/credits use the protected GIF's sampled `COLTAB` cadence while the source
scoring-page duration is preserved. The clean title segment keeps the first
sampled protected frame blank and uses sampled protected-reference cadence for
Williams/logo title colors. Runtime sprite regions now use source
object-picture dimensions, player lasers render as source-style tail-to-tip
spans, explosion presentation uses sparse source-style pixel clouds, and
score-card object placement has been corrected against the protected scoring
frames. Replacement of `docs/start-sequence.gif` still requires owner
acceptance of the candidate media.
Accepted actor starts publish the source-length start-playfield delay before
spawning actors or emitting `WaveStarted`; their accepted-start report is
silent, the actor start sound appears on the following step, and the source
`0xEA` appearance cue appears on the delayed playfield-start report. During a
pending two-player start handoff, scenes draw the source-backed `PLYR1`/`PLYR2`
player label at `0x3C80`; one-player starts use the same delay but suppress
that label. On the
actor wave-cleared interstitial, scenes draw source-backed `ATWV`, `COMPV`,
and `BONSX` status text at `0x3850`, `0x3D60`, and `0x3C90`, plus the wave
number at `0x6550` and multiplier digit at `0x5890`; source numeric glyphs
use the `NUMBR0`-`NUMBR9` column-major image records for score, credit,
Hall of Fame, wave, multiplier, and scoring/action digits. Surviving humans are
projected as source `ASTP3` bonus icons from `0x3CA0` with the source
`+0x0400` step as each survivor is scored on the source four-step cadence,
then next-wave spawning waits through the source `0x80` wave-advance sleep.
While the isolated `HighScoreEntry` surface is active,
scenes draw the source-backed `PLYR1`/`PLYR2` player label at `0x3E38`,
`HOFV1`-`HOFV4` instruction message lines from `0x1458` with source vertical
offsets, entered initials from `0x46AC` with source horizontal offsets, and
the source-shaped `HOFUL` underline words from `0x45B7` using the `0x0800`
initial step and `[0x0400,0x0300,0x0200,0x0100]` word offsets. The clean scene
marks the active cursor underline separately from inactive underline words.
During the hall-of-fame display stall, scenes draw the source-backed display
headings from `0x3854`, `0x2268`, `0x6068`, `0x1E72`, and `0x5F72`; they also
draw the today's-greatest and all-time table rows from `0x1886` and `0x5986`
with source row step, initials offset, score offset, and leading-blank score
rules, and draw the expanded Defender logo at `0x3038` using the source
`0x3C` by `0x18` logo image shape. The display underline bars use the source
left base `0x1E7B`, with the two source-shaped offset segments `0x5F..0x41`
and `0x1E..0x00`. Logo/underline palette/blink/color source evidence is
captured as a visual-state contract for color indices, underline words,
Williams restore rates, and Hall of Fame blink sleep/color evidence; hardware
palette-to-RGB rendering remains a post-acceptance render audit follow-up
rather than a closed clean gameplay contract. Enemy exhaustion is reported through
`WaveSystem`,
keeping the last-hit frame empty and spawning the next clean wave on the
following playing frame. It flattens those per-batch records into one
upload-ready
instance stream. The renderer also owns unit quad vertices, `u16` indices,
record counts, upload bytes, and the `wgpu` vertex layout used to draw
instanced sprites, then derives indexed instanced sprite draw commands with
quad/index counts, instance ranges, and upload byte spans into that stream. It
also records the centered
viewport layout plus GPU-ready clear color, viewport command, and
scene-projection constants for the target surface.
Sprite draw plans also include `wgpu::BufferUsages` metadata and upload bytes
for the quad vertex, quad index, and instance buffers, plus a sprite render-pass
plan with stable vertex buffer slots, index-buffer metadata, indexed
instance draw ranges, draw counts, and instance counts. Sprite render-pass
plans prove native draw command ranges are ready to encode without presenter
side regrouping. Sprite pipeline plans describe the renderer-owned WGSL shader,
vertex and fragment entry points, quad and instance vertex layouts,
alpha-blended color target, primitive state, and multisample state for the
target texture format. Sprite resource binding plans describe the
scene-projection uniform upload, projection bind group layout, atlas texture
binding, atlas sampler binding, atlas texture upload metadata, and expected
bind-group and binding-entry totals used by that shader. The default clean
sprite atlas decodes source-backed runtime regions, source table pixels, and
reclassified PNG inputs into nonblank renderer-owned regions plus the `wgpu`
texture format, usage, extent, and copy layout needed to populate it. Sprite
pipeline layout plans then order those
projection and atlas bind groups for `wgpu` `PipelineLayoutDescriptor` creation
and expose the expected bind-group and binding-entry totals carried into that
layout. Sprite
render pipeline descriptor plans combine that layout with shader entries,
vertex buffers, primitive state, color target, and multisample state for `wgpu`
render pipeline creation, and expose the layout bind-group, vertex-buffer, and
color-target totals carried into the descriptor. Sprite render-pass encoder
command plans then order the pipeline, bind groups, vertex buffers, index
buffer, and indexed draw calls for `wgpu::RenderPass` execution, and expose
the set-pipeline, set-bind-group, set-vertex-buffer, and set-index-buffer
command totals carried into the encoder. Frame-level GPU command plans combine
begin-pass clear state, viewport command presence, scene-projection upload
presence, optional sprite execution with command, draw, and instance totals,
and an ordered sprite-only stream predicate plus separate raster-tooling
evidence into one ordered scene command stream. The clean smoke and live smoke
gates require zero temporary raster commands for the active gameplay path.
Kitty graphics and terminal-session code remain
parked there as historical compatibility evidence, but they are not part of the
active runtime or compatibility API surface. The legacy video renderer owns its
remaining `TerminalGeometry` value type directly instead of importing terminal
session setup. Generated long-trace sample data is nested under the legacy
machine oracle because it is historical fixture evidence, not a clean root
adapter.
Public API tests scan clean module sources so production code cannot import
low-level legacy root modules, bypass the accepted-behavior facade, or
reintroduce legacy implementation terminology.

## Current Behavior Surface

- Live play uses the clean windowed `wgpu` backend, steps clean `Game` frames,
  submits clean audio events, and executes native sprite draw plans.
- `--live-smoke` steps clean `Game` frames through `NativeSceneRenderer` and
  renders them through offscreen `wgpu` texture/readback evidence without using
  the legacy live presenter for smoke frame generation. The smoke gate checks
  selected first/last offscreen frame signatures.
- `--game-smoke` runs a clean game, gameplay sprite coverage, native
  draw-command pipeline and instance coverage, native draw-plan, `wgpu`
  frame-plan ordered sprite-only begin-pass/viewport/projection and sprite
  command/draw/instance evidence, and GPU resource-plan smoke without the
  legacy live presenter.
- `--actor-smoke` runs the actor runtime adapter through attract, credited
  attract, and playing actor steps, verifies clean gameplay/audio events,
  actor sprite families, projectile/HUD/overlay layers, native draw-command
  pipeline coverage, and frame-level `wgpu` command plans.
- `--actor-attract-smoke` runs the default no-input actor attract cycle through
  Williams reveal, Defender coalescence, Hall of Fame, scoring surface, final
  scoring label, and the `cycle 3367` return while verifying native draw-plan
  coverage and attract silence.
- `--actor-post-game-smoke` runs the actor runtime through final-life
  post-game play: three actor-owned pod/player collisions, high-score initials
  entry, a 60-step Hall-of-Fame game-over stall, and return to the Williams
  attract reveal, with actor event, sound, draw-plan, and sprite atlas checks.
- `--actor-wgpu-smoke` renders the same actor smoke frames through the actual
  offscreen `wgpu` readback path and verifies nonblank dynamic frame signatures.
- Normal `cargo run` and the explicit `--actor-live` alias open an interactive
  actor-frame window using the existing `wgpu` presenter and actor-derived
  clean `GameFrame` handoff to live audio. They carry high-score
  initials/backspace through actor input for the actor-owned high-score-entry
  phase.
- Runtime renderer selection has been removed.
- `--input-profile planetoid` is the default input profile.
- `--input-profile cabinet` exposes a MAME-style cabinet keyboard profile.
- `--mute` disables the live audio event runtime path.
- `--cmos-path <file>` opts into file-backed CMOS persistence.
- `--rom-report` and `--verify-roms` validate optional local red-label ROM
  files against embedded metadata when built with `--features legacy-tools`.
- Fidelity commands emit and compare deterministic TSV traces from the Rust
  core and local fixture directories when built with `--features legacy-tools`.
- README media is generated from clean `Game` frames and the clean sprite
  scene/atlas media path with `make readme-media`, which still uses the
  explicit `legacy-tools` tooling path.
- `src/actor_game.rs` is an isolated actor-oriented rewrite slice. It exposes a
  driver-owned `StepPrompt`/`StepReport` model, thread-backed asset actors,
  same-contract keyboard profiles, `XYZZY` overlay state, data-driven
  `AttractScript` support for custom attract drivers, Williams title reveal
  metadata, and Defender wordmark coalescence metadata. `AttractScript` can be
  built from Rust event constructors or checked text script lines before being
  installed in a custom driver. `StepReport` can now project into the clean
  `RenderScene` contract through
  `ActorRenderSceneBridge`, mapping actor draw commands to renderer-owned
  source text glyphs, Williams logo reveal pixels, Defender appearance pixels,
  sprite atlas families, projectile layers, and explosion families. The actor
  slice also exposes `ActorRuntimeAdapter`, which steps the actor driver and
  returns `ActorFrame` values containing the original report, an actor-derived
  clean `GameState`, a clean gameplay/audio `GameEvents` batch, and the clean
  render scene. It is simulation-step driven rather than display-frame driven:
  render cadence is outside actor behavior, and attract scripts use actor-local
  elapsed steps.
  Actor movement and behavior are scriptable through `ActorBehaviorScript`,
  which resolves default, actor-kind, and actor-id profiles into each prompt;
  level scripts can tune speeds, fire cadence, pickup/conversion bands,
  gravity, timed effects, damage policy, and behavior modes such as human
  seeking versus player chasing or non-source hostile drift versus player chase
  without rewriting actor structs. `ActorBehaviorScript` can be built from Rust
  profile constructors or checked text profile updates. The built-in baseline
  behavior is embedded from `assets/red-label/actor-behavior.script`.
  Read-only script manifests expose persistent attract event, driver behavior,
  and wave-profile configuration, and every `StepReport` includes the effective
  behavior manifest after transient input overrides such as `XYZZY`
  invincibility.
  `ActorWaveScript` names a driver-owned progression script whose wave profiles
  apply behavior scripts plus hostile and initial-human spawn records when play
  starts and when hostile snapshots are cleared. `ActorWaveScript` can be built
  from Rust profile constructors or checked text wave/spawn records, including
  spawn-index behavior profiles that the driver installs as actor-id profiles
  immediately after allocating wave actors. The built-in actor attract,
  behavior, and wave scripts are embedded from
  `assets/red-label/actor-attract.script`,
  `assets/red-label/actor-behavior.script`, and
  `assets/red-label/actor-waves.script`. The attract script can draw
  the source `ELECV` presents message, source-style Hall-of-Fame table rows,
  source-offset scoring/instruction labels, source `CREDV` credits label/count,
  and a scriptable scoring scanner surface in addition to Williams reveal and
  Defender wordmark coalescence. Its source
  page-start steps are Williams from step 1, `ELECV` from step 236, the
  Defender wordmark from step 365, the high-score/zero-credit Hall-of-Fame page
  from step 488 for the source 60-tick stall window, and the
  scoring/instruction page from step 1088. Title pages suppress the zero-credit
  line but still show real inserted credits through a `credits_nonzero` script
  action. The Hall-of-Fame page also draws source-offset `HALLD_*` headings and
  the source Defender logo; `hall_scores` draws Today’s and All-Time table
  columns from driver scores plus embedded red-label seed initials. The
  scoring/instruction page draws `SCANV` at step 1088, then reveals `LANDV`,
  `MUTV`, `BAITV`, `BOMBV`, `SWRMPV`, and `SWARMV` on the source scoring-card
  process cadence from checked `messages.tsv` rows and source screen
  addresses, while `scoring_surface` draws the source
  top scanner frame/marker bars, `MTERR` mini-terrain records, source-shaped
  rescue-demo player/human/lander object sprites, scanner blips, and the
  source-shaped rescue laser, lander explosion fragments, 500-point rescue
  popup, and `ENMYTB` enemy legend transfer/materialization/reveal objects
  before the embedded script cycles back to the first Williams reveal step at
  source step 3367. Custom attract scripts can opt into looping with
  `cycle <step-count>` and otherwise stay linear.
  Custom attract scripts can draw checked `messages.tsv` labels through source
  cursor controls with optional visual offsets, and the older one-column
  `high_scores` action remains available for custom screens. The
  default actor wave progression expands
  that wave script through `assets/red-label/wave-table.tsv` for active
  wave size, lander and bomber movement speed, lander fire cadence, baiter
  entry/shot/seek timing, and source bomber/pod counts. The actor allocator
  follows the source active-family
  shape, so wave `1` remains lander-only while later waves can seed bomber and
  pod actors beside source-RNG-restored landers. Source-backed wave profiles
  retain the post-active-batch enemy reserve counts, script wave profiles can
  set those counts with `reserve` / `enemy_reserve`, and
  `StepReport::enemy_reserve` maps them into the clean world snapshot. Reserve
  activation is armed only after the active batch has published a report; when
  source-counted hostiles are gone, the driver restores a source-shaped reserve
  batch before allowing the survivor-bonus `WaveCleared` path. Lander reserves
  fill active slots first, then bomber and pod reserves restore from their
  source placement/fraction state once landers are exhausted. Later-wave humans
  restore from the source target-list distribution instead of reusing
  first-wave starts.
  A persistent `StatusDisplay` actor draws score, high score, wave, lives,
  credits, and high-score-entry rows from `StepPrompt` state while staying
  inert during attract so custom attract scripts remain in control.
  Source-backed landers, bombers, pods, swarmers, baiters, and humans publish
  fixed-point metadata plus movement/facing metadata in snapshots and advance
  their actor-owned fraction state during active motion. Source-backed hostile
  actors now wrap Y motion through the source active-object playfield bounds
  instead of drifting outside the red-label vertical range. Source-backed
  bomber actors now update seeded picture-frame and Y-velocity metadata,
  including
  cruise-altitude and player-relative Y adjustments, during active source motion
  from the driver-provided source RNG snapshot carried in
  `StepPrompt`/`StepReport`; source-backed baiters use that same source RNG
  snapshot to gate picture-wrap retargeting against `baiter_seek_probability`
  and add player velocity into the source-shaped seek velocity. Hostile
  projectile
  snapshots now carry source-shaped shell velocity/lifetime metadata into the
  clean state bridge; enemy-shot actors advance their own fixed-point fraction
  state every actor step while decrementing lifetime on the source shell-scan
  cadence, and enemy-shot spawn commands can carry source fractions,
  velocities, and lifetime ticks from scripted drivers and source-backed
  lander, swarmer, and baiter AI shots. Source-backed bomber bomb actors publish
  stationary bomb-shell fraction plus source-cadenced lifetime state,
  preserving nonzero scripted source lifetime ticks at spawn. Actor spawn
  command handling
  enforces the shared 20-slot source shell cap for enemy shots and bombs plus
  the red-label 10-slot bomber bomb shell cap, and source-backed bomb-shell
  plus enemy-shot spawns honor the source `GETSHL` X/Y placement bounds at
  X `0x98` and Y `0x2A`.
  `XYZZY` invincibility is represented as the same
  temporary player behavior override. The slice now also models source landers
  preferring their configured human target slot, lander pickup/carry/conversion,
  source-shaped converted-mutant fractions, shot timer, hop RNG, and clean
  `SourceMutantSnapshot` bridge metadata,
  falling-human rescue and safe landing scoring, score popups, hostile
  projectile actors for lander shots, bomber/pod hit scores and cues,
  bomber-laid bomb actors with source bomb-collision cues and seeded source
  Y-motion metadata, pod laser-hit swarmer spawning with the source request
  bound, swarmer shot timer projectile/cue emission, smart-bomb pod scoring
  without swarmer spawning, source-paced baiter timer entry with
  non-wave-blocking baiters, baiter shots/scores/hit cues,
  stock-backed smart-bomb hostile clearing with exhausted-stock guards and a
  non-consuming `XYZZY` overlay path, hyperspace source-shell cleanup for enemy
  shots and bomb shells, separate launch/materialization cue emission,
  actor-owned hidden hyperspace
  interval plus behavior-scripted rematerialization point or source
  `SEED`/`HSEED`/`LSEED` snapshot, driver-owned source hyperspace RNG
  advancement and default/kind player-behavior injection, life-stock decrement
  and replacement-player respawn on non-final player hazard collisions, source
  `HYP2` `LSEED > 0xC0` delayed death-risk routing through the same player
  death path, final-life game-over/high-score handoff, clean replay-bonus stock
  awards through actor scoring, and an
  actor wave-cleared interstitial report. On that report, the driver emits
  `WaveCleared`, keeps surviving humans visible for the source `ATWV` /
  `COMPV` / `BONSX` status, awards one survivor at a time for
  `100 * min(wave, 5)`, projects each source `ASTP3` icon as it is awarded, and
  delays the next wave's actor spawns plus `WaveStarted` event until the source
  four-step astronaut cadence and final `0x80` wave-advance sleep complete.
  variant draw metadata for hostile families, bomb, player, and human clouds,
  age-based source explosion-size scaling in the actor render bridge,
  descriptor-backed enemy-family source explosion pixel clouds, source
  top-left/center placement metadata for target6 `SCZP1` collisions, and
  mutant spawn handoff. Default
  interactive play now reaches this actor path through `ActorRuntimeAdapter`.

## Compatibility Features

`XYZZY` is intentionally non-arcade behavior. It must stay outside the
red-label trace contract unless paired tests prove disabled arcade behavior and
enabled overlay behavior.

Planetoid key mapping is an input profile only. The arcade core receives
Defender cabinet actions, not Planetoid-specific semantics.

## Validation

Required local gates for behavior or architecture changes:

```sh
make release-gate
```

Docs-only changes should at least run:

```sh
make docs-lint
make diff-check
```

`make release-gate` runs the full local release validation path in order:
formatting, default and `legacy-tools` Rust tests, both clippy passes,
clean-fidelity, reference-media helper tests, fresh owner-review evidence
package, accepted report gate, MAME doctor, short MAME recorder smoke, README
media generation, game smoke, actor attract/post-game smoke, live smoke, docs
lint, and diff hygiene.

`make fidelity` runs the broad gate: formatting, default Rust targets, default
clippy, explicit `legacy-tools` Rust targets, `legacy-tools` clippy, trace
exporter self-tests, Python helper tests, local trace fixture comparison,
coverage, and added-line coverage.
`make clean-fidelity` builds with `legacy-tools`, runs the clean rewrite
equivalence gate against the current MAME-backed embedded scenario set by
default, and prints TSV first-divergence reports from the real clean `Game` to
the accepted oracle. It does not require local ROMs or MAME reference traces.
Set `SCENARIOS="..."` to narrow the gate during focused implementation steps.

GitHub CI keeps the expensive gates split by subsystem: `make ci-doctor`
checks Lua, Python, coverage, and Linux smoke prerequisites; `make fidelity`
runs the Rust, trace, and coverage gate; and `xvfb-run -a make smoke-wgpu`
runs the no-device live smoke path. Coverage baseline refreshes must use the
explicit `make coverage-new-code-baseline NEW_CODE_COVERAGE_BASE=...` command.

## Active Constraints

- Live audio consumes clean `GameFrame` and `SoundEvent` batches and delivers
  them to a bounded non-blocking backend trait. Legacy frame-output sound
  timing is adapted before it reaches `src/audio.rs`. Normal interactive play
  attempts a synthesized device backend and falls back to the no-device null
  backend if host output is unavailable; foreground sound-board commands
  interrupt the previous foreground command to match the single-DAC Williams
  board behavior; smoke mode stays no-device and deterministic. The accepted
  implementation contract is documented in `docs/fidelity/live-audio.md`.
- Local MAME and clean candidate reference generation is intentionally local
  tooling; generated media and reference traces are not part of the normal
  runtime.
- New clean production callers must avoid direct legacy module imports. Code
  that still needs accepted-behavior evidence should use the crate-private
  `accepted` facade under `legacy-tools` until the clean system replaces that
  responsibility. Temporary README media tooling may use the doc-hidden
  `readme_media` facade under `legacy-tools`, but that facade must capture
  clean `Game` frames and clean sprite scene/atlas media rather than the
  accepted machine. Root legacy modules must remain feature-gated and
  crate-private. This boundary is guarded by source-level public API tests.
