# Actor Architecture Rewrite

`src/actor_game.rs` is the first isolated slice of the Rust actor-oriented
rewrite. It now owns the default interactive live runtime, while the clean
`Game` path remains available through explicit smoke, fidelity, and oracle
evidence commands.

## Structure

- `ActorGameDriver` owns game phase, score, credits, lives, high scores, actor
  ids, actor threads, and the latest snapshots.
- Each asset is a Rust struct implementing `AssetActor`. The current slice has
  `AttractDirector`, `ScriptedAttractProgram`, `StatusDisplay`, `PlayerShip`,
  `Lander`, `Mutant`, `Bomber`, `Bomb`, `Pod`, `Swarmer`, `Baiter`, `Human`,
  `LaserShot`, `EnemyLaserShot`, `Explosion`, and `ScorePopup`.
- `ThreadedAsset` runs one actor on one Rust thread. The driver sends one
  `StepPrompt` per simulation step and waits for one `ActorReply`, keeping
  behavior deterministic while still matching the requested thread-per-asset
  shape.
- Actors do not mutate the world directly. They return snapshots, draw
  commands, sound cues, and gameplay commands. The driver resolves collisions
  and applies world rules in stable actor-id order.
- Rendering is described by `DrawCommand`, `SpriteKey`, and `VisualEffect`
  values. The status display is also an actor: it draws score, high score, wave,
  lives, credits, and high-score-entry rows from `StepPrompt` state. Explosion
  draws carry `ExplosionKind` metadata for lander, mutant, bomber, pod,
  swarmer, baiter, bomb, player, and human clouds plus optional source-center
  metadata, so the actor render and clean-state bridges preserve source family
  identity and source top-left/center placement. The render bridge also routes
  descriptor-backed enemy-family explosions through the clean source
  expanded-object pixel-cloud renderer and uses the draw age to apply the clean
  source explosion-size curve. Audio is
  described by `SoundCue`; source-backed cues expose their red-label Williams
  sound-board command byte through `SoundCue::source_sound_command`.
  `ActorSoundEventBridge` converts a stream of `StepReport` sound cues into the
  clean `SoundEvent` values consumed by the live audio runtime, including
  thrust start/stop edges derived from actor cue state.
- Rendering is adapted after actor behavior resolves. `StepReport::render_scene`
  and `ActorRenderSceneBridge` read actor draw commands and emit clean
  `RenderScene` sprites for source text glyphs, Williams logo reveal pixels,
  Defender coalescence pixels, atlas-backed actor families, projectile layers,
  score popups, and explosion variants. Actors do not own renderer resources or
  display cadence.
- `ActorRuntimeAdapter` combines the sound and render bridges into an
  actor-specific `ActorFrame`: the original `StepReport`, an actor-derived
  clean `GameState`, a clean gameplay/audio `GameEvents` batch, and the clean
  `RenderScene`. `ActorStateBridge` maps actor phase, score, stock,
  high-score-entry state, and published actor snapshots with velocity/facing
  metadata plus hostile-projectile source metadata into the clean state
  contract without making the actor simulation display-frame driven.
  `StepPrompt` and `StepReport` also carry the driver-owned source RNG snapshot
  for playing steps, so source actors can make seeded movement decisions from
  shared world state instead of local frame counters.
- `src/actor_smoke.rs` exercises `ActorRuntimeAdapter` through a scripted
  attract/play sequence and the native draw planner. The smoke report verifies
  attract, credited attract, playing actor frames, clean gameplay/audio events,
  required actor sprite families, projectiles, HUD text, overlays, native
  draw-command pipelines, and frame-level `wgpu` command plans for actor live
  runtime coverage.
- `--actor-wgpu-smoke` reuses that actor smoke sequence but sends the resulting
  actor `RenderScene` values through the offscreen `wgpu` texture/readback path.
  It requires every actor frame to render nonblank pixels and produce dynamic
  readback signatures for the actor runtime.
- Normal `cargo run` and the explicit `--actor-live` alias now use the
  interactive actor runtime. The actor window steps `ActorRuntimeAdapter`,
  converts each actor frame into a clean `GameFrame`, submits that frame to the
  live audio runtime, and draws actor scenes with the existing `wgpu` presenter.
  The shared live input state carries the same key bindings and `XYZZY` mode
  into actor steps. Actor high-score entry now consumes initials/backspace from
  that input surface, updates driver-owned initials state, and returns to
  game-over after a three-letter entry is submitted.

## C++ to Rust Mapping

| C++ shape | Rust actor rewrite |
| --- | --- |
| Base sprite class with virtual methods | `AssetActor` trait |
| Sprite subclasses | concrete actor structs |
| Game/world driver class | `ActorGameDriver` |
| Per-object update method | `AssetActor::update(&StepPrompt)` |
| Draw and sound side effects | `DrawCommand` and `SoundCue` replies |
| Direct object pointers | stable `ActorId` plus snapshots |
| Thread per object | `ThreadedAsset` prompt/reply channel |

## Controls and XYZZY

The actor rewrite owns a fresh input mapper instead of depending on the
existing runtime mapper. `KeyboardMapper` supports the current `planetoid` and
`cabinet` profiles:

- Planetoid keeps `5`/`6`/`7` coins, `1` or `Enter` start, `A`/`Z` altitude,
  shift thrust, space reverse, `Enter` fire, `Tab` smart bomb, `H`
  hyperspace, `F2` service advance, `F3` high-score reset, `F4` auto/audits
  hold, `F5` tilt, and `Q`/`Esc` quit.
- Cabinet keeps `5`/`6`/`7` coins, `1` one-player start, `2` two-player start,
  arrows for altitude, `R` reverse, `T` thrust, `F` fire, `B` smart bomb, `H`
  hyperspace, and the same service/reset/audit/tilt keys.
- Typing `X`, `Y`, `Z`, `Z`, `Y` toggles `XYZZY`. While active, `F` toggles
  auto-fire, `G` toggles invincibility, and `Tab` can mark an overlay smart
  bomb request.

## Not Frame Driven

The actor rewrite is not a MAME frame-script runner. `ActorGameDriver::step`
advances the simulation once, prompts each actor once, and lets actor-owned
state decide what changes. A renderer may draw the latest `StepReport` at any
display cadence, but rendering cadence is not the source of gameplay behavior.

Attract scripting follows the same rule. `ScriptedAttractProgram` keeps its own
elapsed script step and evaluates relative `AttractScriptEvent` durations. It
does not branch on the global driver step or on protected-reference frame
numbers.

## Behavior Scripts

Actor movement and behavior are configurable data owned by the driver.
`ActorBehaviorProfile` holds tunable attributes for player movement and laser
cooldown, player hyperspace hiding/rematerialization including optional source
seed snapshots, laser speed and lifetime, lander seek/carry/fire behavior,
mutant movement, bomber movement/bomb cadence, pod movement, swarmer
movement/fire behavior, baiter movement/fire behavior, bomb lifetime, human
fall/landing behavior, and timed effect lifetimes. It also holds behavior modes
such as
`LanderBehaviorMode` and `HostileMovementMode`, allowing scripts to choose
whether a lander seeks humans, chases the player, or simply drifts, and whether
non-source mutant, bomber, pod, swarmer, and baiter actors drift or chase the
player.

`ActorBehaviorScript` resolves those profiles in this order:

- Actor-id profile, for one specific asset instance.
- Actor-kind profile, for a level-wide group such as all landers.
- Default profile, for the baseline arcade behavior.

The resolved script is carried in every `StepPrompt`, so each actor still owns
its state and movement code while the game/world/driver can tune the attributes
that code uses. `ActorGameDriver::set_default_behavior`,
`ActorGameDriver::set_kind_behavior`, and `ActorGameDriver::set_actor_behavior`
are the current script API. A level script can use those calls to make later
waves faster, shorten laser cooldowns, change human fall gravity, or alter one
specific actor without changing the actor struct. Scripts can also choose
movement behavior modes, for example making a later-wave lander ignore humans
and chase the player, or making a specific bomber chase the player instead of
using its fallback drift.
`ActorBehaviorScript::parse_text(...)` and
`str::parse::<ActorBehaviorScript>()` accept checked text profile updates using
the same resolution model:

```text
# scope target field value
default player_speed 5
kind lander lander_mode chase_player
kind lander lander_seek_speed 6
actor 42 lander_drift_speed 7
kind player player_takes_enemy_collision_damage false
```

`default` lines update the baseline profile, `kind` lines update all actors of
an `ActorKind`, and `actor` lines update one actor id. Numeric values accept
decimal or `0x` hex notation, mode fields accept names such as `drift` and
`chase_player`, and parser errors include the source line number.
The built-in baseline profile is the embedded checked script
`assets/red-label/actor-behavior.script`; `ActorBehaviorScript::default()`
parses that asset at startup and the raw arcade profile remains available as
the parser fallback.

`AttractScript::manifest`, `ActorBehaviorScript::manifest`,
`ActorWaveScript::manifest`, and `ActorGameDriver::script_manifest` expose
read-only snapshots of configured attract events, driver behavior, and wave
scripts for custom drivers and test tooling.
`StepReport::behavior_script` carries the effective behavior manifest used for
that simulation step, after transient input overrides such as `XYZZY`
invincibility have been applied.

Hyperspace is represented as a driver-applied gameplay command rather than a
render frame effect. The player actor emits `GameCommand::Hyperspace` and
`SoundCue::Hyperspace` from the mapped `H` input; the driver clears active
source shell actors, including enemy shots and bomb shells, while leaving player
lasers, hostile actor families, score, lives, and smart-bomb stock unchanged.
`PlayerShip` then owns the hidden interval: while its hyperspace timer is active
it remains alive but publishes no collision bounds, no player draw, and no
input-driven actions.
`ActorBehaviorProfile` configures the hidden step count and fallback
rematerialization coordinates, and the actor emits
`SoundCue::HyperspaceMaterialize` when it returns. `ActorGameDriver` owns the
source hyperspace RNG, advances it once per playing step, and injects that
`ActorHyperspaceSourceSeed` into default and kind-level player behavior when a
script has not provided a seed. Actor-id profiles remain explicit overrides. In
the source-backed path `HSEED` selects the source X/facing branch and Y high
byte, while the entry-frame `LSEED` drives the death-risk threshold. Without a
seed snapshot the actor uses the direct scripted rematerialization coordinates
and effective `LSEED` byte. Values above `0xC0` arm the source `HYP2`
death-risk branch and route through the normal player death/life-stock path
after the delay.

`XYZZY` invincibility uses the same mechanism. When invincibility is active,
the driver applies a temporary player behavior override that disables enemy
collision damage for that simulation step; collision handling reads that
profile rather than branching on a separate god-mode flag.

## Wave Progression

`ActorWaveScript` is the driver-owned level progression script. It has a stable
name plus ordered `ActorWaveProfile` records. Each wave profile supplies the
`ActorBehaviorScript` for that wave plus hostile and initial-human spawn
records. The driver applies wave `1` when play starts, carries the wave number
in `StepPrompt` and `StepReport`, and advances to the next configured profile
when the current hostile snapshots are cleared.

The default actor progression is the embedded checked script
`assets/red-label/actor-waves.script`, whose `source_waves` directive expands
through `assets/red-label/wave-table.tsv` via an actor-owned adapter. The
current actor mapping uses source-backed
`wave_size`, `lander_x_velocity`, `bomber_x_velocity`, `lander_shot_time`,
`mutant_random_y`, `mutant_y_velocity_msb`, `mutant_y_velocity_lsb`,
`mutant_x_velocity`, `mutant_shot_time`, `baiter_time`, `baiter_shot_time`,
`baiter_seek_probability`, `bombers`, and `pods` to set active family
allocation, movement speed, fire cadence, mutant hop/shot behavior, and baiter
entry pacing. Wave `1` remains lander-only; later source waves seed one
lander, one bomber, and one pod when those reserve counts are available before
filling the rest of the active batch, matching the clean source allocator's
family order. Wave `1` uses the source first-wave lander restore metadata from
the existing clean evidence, including fixed-point fractions, velocities, shot
timer, sleep ticks, picture frame, and target-human index. Later source waves
restore landers from the source RNG placement, `RMAX` shot timer, X velocity,
and Y velocity path, then assign target-list slots from the restored human
distribution. Source-backed landers, bombers, pods, and baiters publish their
metadata in snapshots and advance active motion by updating their own
fixed-point position/fraction state. Source-backed hostile actors wrap Y motion
through the source active-object playfield bounds. Source-backed bomber actors
also update
seeded picture-frame and Y-velocity metadata, including cruise-altitude and
player-relative Y adjustments, from the driver-provided source RNG snapshot.
Source-backed baiter actors use that same source RNG snapshot to gate
picture-wrap retargeting against the wave's `baiter_seek_probability` and add
player velocity into the source-shaped seek velocity.

`ActorWaveScript::parse_text(...)` and
`str::parse::<ActorWaveScript>()` accept checked text level scripts:

```text
name hard opening
wave 1
behavior kind lander lander_mode chase_player
behavior kind lander lander_seek_speed 6
spawn_behavior lander 0 lander_seek_speed 8
lander 80 96
human 40 214
wave 2
behavior kind lander lander_mode drift
behavior kind lander lander_drift_speed 5
lander 100 100
bomber 120 80
pod 160 88
source_waves 3 16
```

`behavior` lines reuse the `ActorBehaviorScript` parser for the current wave.
`spawn_behavior <kind> <index> <field> <value>` lines configure one spawned
actor before the driver has allocated its runtime actor id. They inherit the
current wave/kind behavior at the point where the line is parsed and are then
installed as actor-id behavior profiles when that spawn index is allocated.
`lander`, `bomber`, `pod`, and `human` lines add clean scripted spawn records;
humans default to `grounded` and can also be declared as `falling <velocity>` or
`carried <actor-id>`. `source_wave <wave>` and `source_waves <first> <last>`
expand source-backed wave-table profiles into the same checked script, so the
production default and custom level scripts use the same parser surface. The
parser sorts wave profiles by wave number, rejects duplicate waves, and reports
malformed lines with source line numbers.

Hostile projectile actors publish source-shaped shell metadata too: enemy
lasers own and advance fixed-point velocity, fraction, and lifetime values,
with lifetime decrementing on the source shell-scan cadence. Enemy-shot spawn
commands can also carry source fractions, velocities, and lifetime ticks from
custom drivers and source-backed lander, swarmer, and baiter AI shots.
Source-backed bomber bomb actors carry stationary bomb-shell fraction and
source-cadenced lifetime values into the clean state bridge, preserving nonzero
scripted source lifetime ticks as the initial actor shell-scan lifetime. Driver
spawn-command handling
enforces the shared 20-slot source shell cap across enemy shots and bombs plus
the red-label 10-slot bomber bomb shell cap. Source-backed bomb-shell spawn
commands and enemy-shot spawn commands also honor the source `GETSHL` X/Y
placement bounds at X `0x98` and Y `0x2A`; non-source scripted bomb actors are
still available for custom drivers.

Initial humans are source-backed: wave `1` uses the captured first-wave starts,
while later source waves restore humans through the source target-list RNG
distribution. Their actor spawns carry fixed-point fractions, picture frame,
and source target-list slot metadata. Grounded source humans advance their own
walk cadence, picture frame, and fixed-point X fraction after the initial
source sleep ticks.
Source-backed landers ask the prompt for their configured target slot first and
only fall back to nearest-human seeking when that target is unavailable.
When a source-backed lander completes a human conversion, the spawn command for
the new mutant carries source-shaped mutant fractions, wave-derived shot timer,
driver-provided hop RNG, render correction, and deferred-shot metadata into the
mutant actor snapshot and clean `SourceMutantSnapshot` bridge. Source-backed
mutant actors use that state as their movement source: they select X/Y velocity
from the wave table and player position, advance their own hop RNG and
fixed-point fractions, decrement shot timers, and emit source-shaped hostile
projectile metadata with the red-label `0xF6` mutant-shot cue. First-wave
target6 converted mutants also carry the source conversion X correction through
actor metadata, use source-backed dive/visual projection anchors for
draw/collision positions, preserve the deferred first-shot flag, and emit the
exact source-shaped fire2524 projectile metadata for the forced target6 shot
rows. The actor driver also uses that metadata during player/enemy collision:
pending fire2524 target6 rows suppress contact, collision boxes use the
projected target6 position, and the fire2524 collision row produces a
source-positioned enemy explosion, mutant-hit cue, and score before the normal
player death command path.

## Attract Graphics

The attract screen is data-driven. `AttractScript` contains ordered
`AttractScriptEvent` records, and `ScriptedAttractProgram` turns the active
events for its own current script step into draw commands. The default
`AttractScript::red_label_title()` parses
`assets/red-label/actor-attract.script`, recreating the current Williams/logo,
high-score, and credits opening sequence from checked text while the older Rust
event constructor remains available as a fallback. Custom drivers can pass
their own parsed or constructed sequence through
`ActorGameDriver::with_attract_script(...)` without replacing coin/start
control handling.
`ActorGameDriver::script_manifest()` includes the immutable attract-event
manifest so custom drivers can verify or serialize the installed sequence
without inspecting the thread-backed attract actor.
`AttractScript::parse_text(...)` and `str::parse::<AttractScript>()` accept the
same event model from checked text script lines:

```text
# action start duration x y ...
text 1 forever 10 10 PRESS START
sprite 2 forever defender_logo 40 44
high_scores 1 forever 82 188 10 5
credits 1 forever 176 226 248 226
williams_logo 5 - 108 60
defender_wordmark 72 - 96 144
```

Blank lines and `#` comments are ignored. `duration` can be a step count or
`-`, `none`, `forever`, or `infinite` for an unbounded event. Parser errors
include the source line number so custom driver tooling can reject malformed
scripts before the actor runtime starts. `high_scores` lines use
`x y row-height rows` after the timing fields and draw rows from the
driver-owned high-score table carried in `StepPrompt`. `credits` lines use
`label-x label-y count-x count-y` after the timing fields and draw the
source-backed `CREDV` / `CREDITS:` label plus the visible credit count from
`StepPrompt.credits`.

Script actions currently cover:

- `Text`, for scripted title/status lines.
- `Sprite`, for static sprite placement.
- `HighScores`, for prompt-backed high-score table rows in attract/game-over.
- `Credits`, for the prompt-backed credit label/count in attract/game-over.
- `WilliamsLogo`, which emits `SpriteKey::WilliamsLogo` with
  `VisualEffect::WilliamsReveal` metadata for handwriting reveal and title
  color phase.
- `DefenderWordmark`, which emits `SpriteKey::DefenderCoalescence` with slot
  and row-pair metadata until the wordmark settles to
  `SpriteKey::DefenderWordmark`.

`ActorRenderSceneBridge` maps the Williams reveal metadata through the
renderer-owned source pixel walk and maps Defender coalescence metadata through
the renderer-owned source appearance pixels. Static attract sprites still honor
their script positions, so custom attract drivers can place their own text and
non-source sprite events while the default red-label title uses source screen
positions.

`GameInput::from_clean_input` preserves the current clean live gameplay,
coin/start, service advance/reset, tilt, and auto-up key contract while
accepting an explicit `XyzzyMode`. High-score initials/backspace remain outside
the actor input conversion until the actor high-score-entry surface owns those
text-editing controls.

This is intentionally a Rust data script first. A later text parser or
MAME-table translator can target the same `AttractScript` API without changing
actor ownership.

## Gameplay Slice

The actor driver now owns a first Defender gameplay loop:

- Starts seed the playfield with ten source first-wave human actors carrying
  target-list slot and picture-frame metadata. Source-backed grounded humans
  update their own walk frame and fixed-point X fraction.
- A persistent `StatusDisplay` actor emits play-state text for score, high
  score, wave, lives, smart-bomb stock, credits, and high-score-entry rows from
  the same simulation prompt as every other actor. It stays inert during attract
  so a custom attract script can own that screen.
- Landers seek the nearest grounded human, attach it through an
  `AttachHuman` command, carry it upward, and convert into a mutant when the
  carried human reaches the upper conversion band. Source-backed landers prefer
  their configured target-human slot before falling back to nearest-human
  seeking, and source-backed conversions preserve source mutant metadata for
  the clean state bridge.
- Source-backed mutant actors advance through actor-owned source velocity,
  random-hop, sleep, and shot-timer state. Mutant shots emit source-shaped enemy
  projectile metadata and the red-label `0xF6` cue through the same driver
  command boundary as other hostile shots. The first-wave target6 branch now
  applies the MAME-backed conversion correction, projected draw/collision
  anchors, deferred visible-entry shot, dive-shot anchor overrides, and exact
  fire2524 projectile fractions/velocities inside the actor source path. The
  driver-side collision resolver also suppresses the pending fire2524 target6
  collision interval and preserves the eventual target6 `SCZP1` explosion
  top-left plus source center.
- Lander shot timers emit the source `0xFC` lander-shot cue and an `EnemyLaser`
  actor.
  Enemy lasers are player hazards, smart-bomb targets with no score value, and
  respect the same player damage behavior profile used by `XYZZY` invincibility.
  Their snapshots expose source-shaped shell velocity, fraction, and lifetime
  metadata advanced by the actor.
- Later source waves seed bomber and pod actor families when the wave table
  exposes those counts. Bombers and pods draw their own sprites, move through
  actor-owned source fixed-point metadata when source-backed, publish per-step
  movement/facing metadata, and remain script-tunable through their behavior
  profiles. Source-backed hostile actors wrap Y motion through the source
  active-object playfield bounds. Source-backed bombers derive picture-frame
  and Y-velocity updates from the driver-provided source RNG snapshot, including
  cruise-altitude and player-relative Y adjustments.
- Bomber actors can lay first-class `Bomb` actors on a scriptable cadence with
  the source ten-bomb active cap and source `GETSHL` placement bounds. Bombs
  draw their own sprite, expire through actor-owned lifetime state, act as
  player hazards, and emit a bomb collision cue when they hit the player.
  Source-backed bomber bombs snapshot the bomber source fractions as stationary
  bomb-shell fraction metadata and decrement actor-owned lifetime metadata on
  the source shell-scan cadence. Custom source bomb spawns can provide nonzero
  source lifetime ticks directly; otherwise the behavior profile supplies the
  fallback lifetime.
- Projectile-killed pods spawn a bounded mini-swarmer actor batch using the
  source request count. Source-backed swarmers decrement their actor-owned shot
  timer into hostile projectile commands with a distinct swarmer shot cue.
  Smart-bomb pod scoring intentionally does not spawn swarmers, matching the
  clean source behavior.
- The driver advances a source-paced baiter timer while source-counted wave
  enemies remain. Expired timers spawn source-backed baiter actors up to the
  source active cap. Baiters pursue and shoot through actor-owned metadata,
  publish three-frame source animation state, gate picture-wrap retargeting
  through the driver-provided source RNG snapshot, fold player velocity into
  source-shaped seek velocity, score 200 points on laser hit, and do not block
  wave completion once lander/bomber/pod/swarmer snapshots are gone.
- Carried humans follow their lander. If the carrier disappears, the human
  falls under a simple acceleration model and emits the release sound cue.
- Falling humans caught by the player award 500 points, emit the rescue sound
  cue, and spawn a 500-point popup actor.
- Slow falling humans settle safely on the terrain line for 250 points and a
  250-point popup. Fast impacts destroy the human and spawn an explosion.
- Explosion actors publish family-specific variant metadata for lander, mutant,
  bomber, pod, swarmer, baiter, bomb, player, and human clouds while retaining
  actor-owned lifetime state. `ActorRenderSceneBridge` maps that age through
  the clean source explosion-size curve, caps the render scale the same way as
  the clean expanded-object path, and projects descriptor-backed lander,
  mutant, bomber, pod, swarmer, and baiter clouds through source pixels with
  optional source-center metadata instead of static atlas sprites.
- Smart bomb is now a real driver command: normal player requests consume the
  driver-owned stock before clearing active hostile actors, awarding enemy
  scores, and spawning explosions while preserving human actors. Exhausted stock
  leaves hostiles alive. The `XYZZY` overlay smart bomb uses the same command
  path without consuming stock.
- Player hazard collisions destroy the current player actor, decrement the
  driver-owned life stock, and spawn a replacement player when lives remain.
  Final-life collisions enter the game-over/high-score path.
- Player laser hits now resolve lander, mutant, bomber, pod, swarmer, and baiter
  targets through the driver, awarding source scores and family hit cues for
  bomber, pod, swarmer, and baiter families.
- Wave scripts now apply behavior profiles when play starts and when all
  hostile snapshots are cleared, allowing level difficulty to progress through
  driver-owned data.

These mechanics are still intentionally compact. The next fidelity slices
should port the remaining source restore positions against the actor state,
render, and audio bridges now used by the default live runtime, then retire
isolated scaffolding as parity improves.
