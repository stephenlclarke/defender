# Defender Current Specification

Last reviewed: `2026-05-21`

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
  source-backed wave profile, high-score entry/session/table, game-over return
  timing, neutral object evidence, clean object categories, mapped clean sprite
  evidence, source-backed human/enemy/projectile object-picture descriptors,
  expanded-object lifecycle/presentation evidence, source-backed score-popup
  lifecycle snapshots, source-backed expanded-object explosion lifecycle
  snapshots, source-backed terrain-blow snapshots, projectile, player,
  direction, two-player start
  admission, source-backed score digit scene sprites, player-count stock
  HUD scene sprites, two-player final-life switch/respawn
  state, two-player start prompt, player-switch, final game-over,
  player-death pixel-cloud snapshots, and attract presentation page-frame state
  for the Williams, presents, Defender wordmark,
  copyright wait, and instruction surfaces, attract credit/presents/instruction
  message-glyph scene sprites with source message row-feed/horizontal-cursor
  controls, wave-completion status message-glyph and survivor bonus icon scene
  sprites, high-score entry prompt
  message-glyph scene sprites,
  hall-of-fame display heading/table scene sprites, and sound-event contracts,
  including source-backed per-family enemy hit command bytes surfaced through
  `UnmappedSoundCommand`.
  The clean
  `Game` shell emits sprite-first scene frames without touching the accepted
  machine adapter.
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
  high-score entry qualification and initials handling, and the
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
direction-derived velocity, advance through `ProjectileMotionSystem`, and are
culled through gameplay state before rendering. Clean collision boxes resolve
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
game-exec pacing cadence, accelerates the timer when the remaining enemy total
is low, and respects the source active-baiter cap. Spawned baiters
retain source shot-timer, picture-cycle, sleep, and velocity state, pursue the
player through source seek rules, fire source-shaped fireball shells with
source `USHSND` command evidence, and those enemy projectiles use source
`SHSCAN` lifetime decrement/wrap behavior, scroll-adjusted fixed-point motion,
offscreen culling, collision scoring, and player-damage handling.
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
or die on over-speed impact with an astronaut explosion and the existing
last-human planet-loss handoff; falling humans caught by the player enter the
clean player-carried state, award the source-backed 500-point rescue score,
emit source `ACSND` command evidence, and start the existing `P500` score-popup
lifecycle; player-carried humans settle on terrain when the player-carried
offset reaches the local terrain line.
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
`GETSHL` placement bounds. Bomber picture/Y/bomb state updates now honor the
source `TIE`
`SEED & 0x06` squad-slot selection, leaving empty selected slots sleeping while
active bomber positions continue through source velocity. Reserve bombers now
use source `TIEST` player-relative squad placement and alternating X velocity
before entering the source bomber runtime. Enemy projectile evidence now
carries source `BMBP1` shell descriptor fields for the standalone
mine/source-shell fixture, and
active clean enemy evidence now carries source object-picture descriptor
labels, addresses, dimensions, and primary/alternate image pointers for the
current lander, baiter, bomber, mutant, pod, and swarmer presentations. Clean
human evidence carries per-human source astronaut picture descriptors: default
`ASTP1` rows and source-restored `ASTP3` rows selected from the `PLRES`
`LSEED` low bit, with restored `LSEED` X low bytes retained as source X
fractions for world-position evidence. Clean worlds also carry source `ASTRO`
process cursor/sleep state that walks one restored, uncarried target-list human
per source cadence, applies source fixed-point X motion, steps Y toward
terrain-relative source targets, and cycles evidence from `ASTP1` through
`ASTP4`. Clean player projectile evidence now carries the source `LASP1`
descriptor label, address, 8x1 size, and primary image pointer while the direct
runtime projectile renderer remains unchanged.
Clean enemy, human, player-projectile, and enemy-projectile object evidence
also carries source-style 8.8 world-position words, velocity words, and
deterministic source object-table identity evidence from the clean source
fixed-point state and source layout: addresses from `0xA23C` plus `0x17` per
slot, source slot numbers, and neutral `OTYP` `0x00`. Runtime scene sprites
remain on the direct clean render path.
Remaining per-family movement/projectile behavior and focused source ecology
fixtures remain later object-ecology work.
Clean
smart bombs consume player stock, clear active enemies through
`SmartBombSystem`, route score through the same scoring system, and leave
destroyed active enemy sprites absent from the scene while source reserves can
enter as the next active batch. One-player credited starts consume one credit,
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
residuals, live top-display scheduling, and exact hardware palette/RGB
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
missing-size and transparent null-object rows stay evidence-only. Score-popup
rows additionally carry source 50-tick lifetime and 250/500 value metadata;
explosion rows carry source `EXST`/`EXPU` frame/lifetime metadata and scale
their descriptor size from the source `RSIZE` high byte until the source kill
threshold. Player death starts a source-backed bank-7 pixel-cloud snapshot
from `PXVCT`/`PX1A` state, carrying source color/counter metadata and visible
piece positions into clean/oracle scenes. Planet destruction starts a
source-backed terrain-blow snapshot: terrain rows are removed from the clean
scene, scanner terrain is disabled, `TEREX` explosions are projected through
expanded-object evidence, and the source status bit, iteration, erase-table
counts, pseudo color, and overload counter are carried for fidelity checks.
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
the final game-over return path. During that final game-over sleep, scenes draw
source-backed `GO` message glyphs at the translated `PLE2` screen address
`0x3E80`. The player-death pixel cloud is cleared before high-score entry
handoff so high-score scenes remain prompt/table-only.
Qualifying final scores are routed through `HighScoreEntrySystem` into
`HighScoreEntry` with `HighScoreEntryStarted` output. High-score entry accepts
alphabetic initials through clean input, normalizes them to uppercase, supports
backspace, emits `HighScoreInitialAccepted`, and emits `HighScoreSubmitted`
when the third initial enters the source-shaped hall-of-fame display stall
before the clean game returns to attract. Normal attract scenes draw the
source-backed `CREDV` credits label at `0x28E5` and the visible credit count
digits at `0x48E5`. The title program is gated by
`AttractPresentationSnapshot`: the source `LGOTAB` Williams logo appears first
at `0x363C`, `ELECV` presents copy appears at `0x3258`/`0x3E6C`, the
source-expanded Defender wordmark appears at `0x3090`, the source copyright
strip appears at `0x3BD0` during the copyright wait gate, and the
instruction-page `SCANV`, `LANDV`, `MUTV`, `BAITV`, `BOMBV`, `SWRMPV`, and
`SWARMV` labels appear at `0x4330`, `0x1C70`, `0x3C70`, `0x5F70`, `0x1CA8`,
`0x40A8`, and `0x5CA8`. This projection is suppressed during the hall-of-fame
display stall. During a pending two-player start handoff, scenes
draw the source-backed `PLYR1`/`PLYR2` player label at `0x3C80`. On the
existing clean wave-cleared frame, scenes draw source-backed `ATWV`, `COMPV`,
and `BONSX` status text at `0x3850`, `0x3D60`, and `0x3C90`, plus the wave
number at `0x6550` and multiplier digit at `0x5890`; surviving humans are
also projected as source `ASTP3` bonus icons from `0x3CA0` with the source
`+0x0400` step. While `HighScoreEntry` is active,
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
palette-to-RGB rendering remains separate render-parity work. Enemy exhaustion
is reported through `WaveSystem`,
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
sprite atlas decodes the reclassified temporary R2 PNG inputs into nonblank
renderer-owned regions plus the `wgpu` texture format, usage, extent, and copy
layout needed to populate it. Sprite pipeline layout plans then order those
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
and an ordered sprite-only stream predicate plus temporary raster evidence into
one ordered scene command stream, while the current live
path still carries a temporary raster payload for visual
equivalence. Kitty graphics and terminal-session code remain
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
- Runtime renderer selection has been removed.
- `--input-profile planetoid` is the default input profile.
- `--input-profile cabinet` exposes a MAME-style cabinet keyboard profile.
- `--mute` disables the live audio event runtime path.
- `--cmos-path <file>` opts into file-backed CMOS persistence.
- `--rom-report` and `--verify-roms` validate optional local red-label ROM
  files against embedded metadata when built with `--features legacy-tools`.
- Fidelity commands emit and compare deterministic TSV traces from the Rust
  core and local fixture directories when built with `--features legacy-tools`.
- README media is generated from the current native renderer with
  `make readme-media`, which uses the explicit `legacy-tools` tooling path.

## Compatibility Features

`XYZZY` is intentionally non-arcade behavior. It must stay outside the
red-label trace contract unless paired tests prove disabled arcade behavior and
enabled overlay behavior.

Planetoid key mapping is an input profile only. The arcade core receives
Defender cabinet actions, not Planetoid-specific semantics.

## Validation

Required local gates for behavior or architecture changes:

```sh
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
make clean-fidelity
cargo run -- --game-smoke
cargo run -- --live-smoke
```

Docs-only changes should at least run:

```sh
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md
git diff --check
```

`make fidelity` runs the broad gate: formatting, default Rust targets, default
clippy, explicit `legacy-tools` Rust targets, `legacy-tools` clippy, trace
exporter self-tests, Python helper tests, local trace fixture comparison,
coverage, and added-line coverage.
`make clean-fidelity` builds with `legacy-tools`, runs the clean rewrite
equivalence gate against all 12 embedded Phase 1 scenario input programs by
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
  backend if host output is unavailable; smoke mode stays no-device and
  deterministic. The accepted implementation contract is documented in
  `docs/fidelity/live-audio.md`.
- Local MAME reference generation is intentionally local tooling; generated
  reference traces are not part of the normal runtime.
- New clean production callers must avoid direct legacy module imports. Code
  that still needs accepted-behavior evidence should use the crate-private
  `accepted` facade under `legacy-tools` until the clean system replaces that
  responsibility. Temporary README media tooling may use the doc-hidden
  `readme_media` facade under `legacy-tools`; root legacy modules must remain
  feature-gated and crate-private. This boundary is guarded by source-level
  public API tests.
