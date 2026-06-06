# Actor Architecture Rewrite

`src/actor_game.rs` is the first isolated slice of the Rust actor-oriented
rewrite. It now owns the default interactive live runtime, while the clean
`Game` path remains available through explicit smoke, fidelity, and oracle
evidence commands.

## Structure

- `ActorGameDriver` owns game phase, current player, player count, per-player
  scores/stocks, credits, high scores, actor ids, actor threads, and the latest
  snapshots.
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
  values. The status display is also an actor: it draws player-one/player-two
  scores, high score, wave, lives, credits, and high-score-entry rows from
  `StepPrompt` state. Playing reports also expose the source `BGOUT` terrain
  through the state and render bridges while no terrain blow is active.
  Explosion
  draws carry `ExplosionKind` metadata for lander, mutant, bomber, pod,
  swarmer, baiter, bomb, player, human, and terrain clouds plus optional
  source-center metadata, so the actor render and clean-state bridges preserve
  source family identity and source top-left/center placement. The render
  bridge also routes descriptor-backed enemy-family explosions through the
  clean source expanded-object pixel-cloud renderer, uses the draw age to apply
  the clean source explosion-size curve, and uses the source terrain-explosion
  growth curve for terrain clouds. Audio is
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
  high-score-entry state, the effective source-wave profile, the driver-owned
  source RNG snapshot, and published actor snapshots with velocity/facing
  metadata plus hostile-projectile source metadata into the clean state
  contract without making the actor simulation display-frame driven.
  `StepPrompt` and `StepReport` also carry the driver-owned source RNG snapshot
  for playing steps, so source actors can make seeded movement decisions from
  shared world state instead of local frame counters. Submitted high-score
  initials now enter a finite 60-step Hall-of-Fame game-over stall through the
  clean `GameOverSnapshot` contract before returning to the normal Williams
  attract reveal.
- `src/actor_smoke.rs` exercises `ActorRuntimeAdapter` through a scripted
  attract/play sequence and the native draw planner. The smoke report verifies
  attract, credited attract, playing actor frames, clean gameplay/audio events,
  required actor sprite families, projectiles, HUD text, overlays, native
  draw-command pipelines, and frame-level `wgpu` command plans for actor live
  runtime coverage. The same module owns `--actor-attract-smoke`, a no-input
  full-cycle gate that advances the default checked attract script through
  Williams reveal, Defender coalescence, Hall of Fame, scoring surface, final
  scoring label, and the `cycle 3367` return while checking native draw plans
  and absence of attract gameplay/audio events. It also owns
  `--actor-post-game-smoke`, which starts actor play, forces three actor-owned
  pod/player collisions, submits high-score initials, checks the 60-step
  Hall-of-Fame game-over stall, and verifies return to the Williams attract
  reveal with actor event, sound, draw-plan, and atlas coverage.
- `--actor-wgpu-smoke` reuses that actor smoke sequence but sends the resulting
  actor `RenderScene` values through the offscreen `wgpu` texture/readback path.
  It requires every actor frame to render nonblank pixels and produce dynamic
  readback signatures for the actor runtime.
- Normal `cargo run` and the explicit `--actor-live` alias now use the
  interactive actor runtime. The actor window steps `ActorRuntimeAdapter`,
  converts each actor frame into a clean `GameFrame`, submits that frame to the
  live audio runtime, and draws actor scenes with the existing `wgpu` presenter.
  `--actor-script <path>` parses one checked sectioned actor driver script and
  boots the same live runtime through `ActorRuntimeAdapter::with_scripts`; it
  is not accepted with `--live-smoke` because that command remains the clean
  game smoke path. `--actor-script-check <path>` uses the same parser and
  runtime constructor headlessly, samples the first attract actor step, then
  credits/starts the actor runtime through the first playable wave and prints
  attract/behavior/wave manifest counts plus first-frame, effective source-wave,
  spawned world, reserve/source-state, and first-play behavior summaries. The
  checker then uses the actor `XYZZY` overlay smart-bomb path to assist
  progression and reports the next playable wave when real wave-clear/wave-start
  logic reaches it, plus a bounded reserve activation batch sequence when
  reserve activation emits restored actor spawns. The checked
  `examples/actor-custom-attract.script` file is the editable smoke-tested
  example. The shared live input state carries the same key
  bindings and `XYZZY` mode into actor steps. Actor high-score entry now
  consumes initials/backspace from
  that input surface, updates driver-owned initials state, enters the 60-step
  Hall-of-Fame game-over stall after a three-letter entry is submitted, and
  returns to attract after the stall.

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

## Start Delay And Two-Player Handoff

The driver owns two-player session state instead of giving one actor authority
over another player's turn. `StepPrompt` and `StepReport` carry current player,
player count, per-player scores, per-player stocks, optional
`PlayerSwitchReport`, and optional `PlayerStartReport` values. When an
accepted one-player or two-player start enters play, the driver publishes a
source-length player-start delay before spawning playfield actors or emitting
`WaveStarted`. Start audio is also driver-owned: the accepted-start report is
silent, the start cue is emitted on the following actor step, and
`SoundCue::PlayerAppear` carries the source `0xEA` appearance command on the
same report that starts the delayed playfield. The render bridge projects the
source player-start prompt only for two-player sessions, so a credited
two-player start draws `PLAYER ONE` while a one-player start keeps the
playfield empty without drawing `PLYR1`. When a player hazard collision reaches
`GameCommand::PlayerKilled`, the driver decrements the active player's stock.
If another player still has lives, it enters a source-length `0x60` switch
sleep, publishes that sleep through `GameOverSnapshot`, suppresses attract
script output during the handoff, and starts the next stocked player's actor
start delay after the countdown. The render bridge projects the source
`PLAYER ONE` / `PLAYER TWO` plus `GAME OVER` switch prompt, then the next
player's source start prompt. If no other player has stock, the normal
game-over / high-score path runs instead. Actor regressions prove the switch
prompt persists for every step of the source `0x60` sleep, clears when the
handoff frame starts the next player's delay, and does not leak into the
eventual `WaveStarted` report.

The current actor handoff deliberately keeps the remaining visual gap explicit:
the bounded switch/start state, source-message glyph projection, and exact
source-glyph prompt placement are locked, but full MAME media proof remains a
separate fidelity boundary.

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
`ActorDriverScripts::parse_texts` checks attract, behavior, and wave text as
one custom-driver configuration. `ActorDriverScripts::parse_text` accepts a
single sectioned script using `[attract]`, `[behavior]`, and `[wave]` headers:

```text
[attract]
text 1 forever 12 20 CUSTOM DRIVER
[behavior]
kind lander lander_mode drift
[wave]
name custom opening
wave 1
lander 80 214
human 100 214
```

The parsed behavior script becomes the base used while parsing the bundled wave
script, so clean wave records inherit driver-wide tuning and source-backed wave
records preserve that tuning unless the red-label wave table owns the specific
family field. Sectioned parser errors preserve the original source line number
when a delegated attract, behavior, or wave line is malformed. The same
sectioned text implements `str::parse::<ActorDriverScripts>()`, can be
inspected through `ActorDriverScripts::manifest()` before a driver is created,
and can be launched directly with `ActorRuntimeAdapter::with_scripts` for
custom-driver runtime smoke tests or through `--actor-script <path>` in the
interactive actor live runtime. `--actor-script-check <path>` runs the same
file-backed path without opening the window and reports the parsed script
surface, first attract actor frame, first playable wave counts, and effective
first-play reserve/source-state plus the behavior profile that actors receive
through `StepReport`. It also samples the next playable wave with an actor
`XYZZY` overlay smart-bomb assist loop, so progressive custom-driver wave
profiles can be checked before live launch without bypassing actor commands.
When that next wave still has enemy reserves, the checker keeps stepping the
same actor path through smart-bomb cooldown and reserve activation, then reports
each observed restored batch's spawned family counts, resulting source state,
and terminal batch status.

`AttractScript::manifest`, `ActorBehaviorScript::manifest`,
`ActorWaveScript::manifest`, `ActorDriverScripts::manifest`, and
`ActorGameDriver::script_manifest` expose read-only snapshots of configured
attract events, driver behavior, and wave scripts for custom drivers and test
tooling. Wave manifests also expose parsed `behavior_preset` and
`spawn_behavior_preset` definitions by normalized name, so tooling can verify
or serialize reusable level-difficulty blocks instead of only inspecting their
resolved per-wave effects. Source-backed wave profiles carry the exact
`ActorSourceWaveProfile` expanded from
`assets/red-label/wave-table.tsv` in those manifests; hand-scripted custom
waves leave that field empty.
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
byte, and `SEED/HSEED` publishes the source background word through the driver
prompt/report state, while the entry-frame `LSEED` drives the death-risk
threshold. Without a seed snapshot the actor uses the direct scripted
rematerialization coordinates and effective `LSEED` byte without changing the
source background word. Values above `0xC0` arm the source `HYP2` death-risk
branch and route through the normal player death/life-stock path
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

Wave clear enters a driver-owned survivor-bonus interstitial instead of
spawning the next wave immediately. `StepReport::survivor_bonus` exposes the
active source-shaped cadence: the first surviving human is scored on the
`WaveCleared` report, each later survivor waits four actor steps, every survivor
awards `100 * min(wave, 5)` points through the same replay-stock scoring system,
and the next wave waits through the source `0x80` wave-advance sleep before the
driver emits `AdvanceWave`/`WaveStarted`. The render bridge draws only the
source `ASTP3` icons whose awards have become visible, so rendering remains a
consumer of actor reports rather than the source of the timing.

The default actor progression is the embedded checked script
`assets/red-label/actor-waves.script`, whose `source_waves` directive expands
through `assets/red-label/wave-table.tsv` via an actor-owned adapter. The
current actor mapping uses source-backed
`wave_size`, `lander_x_velocity`, `bomber_x_velocity`, `lander_shot_time`,
`mutant_random_y`, `mutant_y_velocity_msb`, `mutant_y_velocity_lsb`,
`mutant_x_velocity`, `mutant_shot_time`, `baiter_time`, `baiter_shot_time`,
`baiter_seek_probability`, `bombers`, `pods`, `mutants`, and `swarmers` to set
active family allocation, movement speed, fire cadence, mutant hop/shot
behavior, and baiter entry pacing. The effective source profile is preserved in
each wave manifest so custom drivers can inspect the same source progression
values without reaching into driver internals; parsed `source_wave` overrides
and `source_waves` range overrides preserve the tuned profile rather than the
table default. Wave `1` remains
lander-only; later source waves seed one lander, one bomber, one pod, one direct
mutant, and one swarmer when those reserve counts are available before filling
the rest of the active batch, matching the clean source allocator's family
order. Each source-backed wave profile also carries the source enemy reserve
counts left after that active batch. The driver publishes those counts through
`StepReport::enemy_reserve` and the clean `WorldSnapshot`, arms reserve
activation only after the current active batch has reported once, and restores
the next source batch before survivor-bonus wave clear can start. Lander
reserves fill active slots first; once no landers remain, bomber, pod, direct
mutant, and swarmer reserves restore in the source wave-size batch shape.
Direct mutant reserve rows use source placement, shot-timer, hop-RNG metadata,
and the driver-owned source background word. Source swarmer reserves use the
source `PLRES`/`RSW0` phony-object placement/fraction state before entering the
same mini-swarmer runtime used by pod destruction. If no source human target
remains while lander reserves are selected, the actor driver follows the source
schizoid fallback and restores those rows as source-shaped mutants with source
placement, shot-timer, hop-RNG metadata, and the same source background word.
Wave `1` uses the source first-wave
lander restore metadata from the existing clean evidence, including fixed-point
fractions, velocities, shot timer, sleep ticks, picture frame, and target-human
index. Later source waves restore landers from the source RNG placement, `RMAX`
shot timer, X velocity, and Y velocity path, then assign target-list slots from
the restored human distribution. Source-backed bombers
restore from player-relative source squad placement, and source-backed pods
restore from source RNG placement/velocity state before entering normal
source-motion updates. Source-backed landers, bombers, pods, and baiters
publish their metadata in snapshots and advance active motion by updating their
own fixed-point position/fraction state. Source-backed hostile actors wrap Y
motion through the source active-object playfield bounds. Source-backed bomber
actors also update
seeded picture-frame and Y-velocity metadata, including cruise-altitude and
player-relative Y adjustments, from the driver-provided source RNG snapshot.
Source-backed baiter actors use that same source RNG snapshot to gate
picture-wrap retargeting against the wave's `baiter_seek_probability` and add
player velocity into the source-shaped seek velocity.

`ActorWaveScript::parse_text(...)`,
`ActorWaveScript::parse_text_with_base_behavior(...)`, and
`str::parse::<ActorWaveScript>()` accept checked text level scripts:

```text
name hard opening
behavior_preset hard_lander kind lander lander_mode chase_player
behavior_preset hard_lander kind lander lander_seek_speed 7
spawn_behavior_preset fast_slot lander_seek_speed 9
wave 1
use_behavior hard_lander
use_spawn_behavior lander 0 fast_slot
lander 80 96
human 40 214
wave 2
behavior kind lander lander_mode drift
behavior kind lander lander_drift_speed 5
lander 100 100
bomber 120 80
pod 160 88
mutant 140 90
swarmer 150 96
baiter 170 104
reserve 4 1 0
source_waves 3 16
use_behavior_waves 3 16 hard_lander
behavior_waves 3 16 kind baiter baiter_fire_period_steps 30
use_spawn_behavior_waves 3 16 lander 0 fast_slot
spawn_behavior_waves 3 16 baiter 0 baiter_fire_period_steps 24
source_wave 17 wave_size 5 landers 1 bombers 1 pods 1 mutants 1 swarmers 1
source_waves 18 20 wave_size 5 landers 1 bombers 1 pods 1 mutants 1 swarmers 1
```

`behavior` lines reuse the `ActorBehaviorScript` parser for the current wave;
when parsed through `parse_text_with_base_behavior` or
`ActorDriverScripts::parse_text` / `parse_texts`, each new clean wave starts
from the supplied base behavior script instead of the arcade fallback.
`behavior_preset <name> ...` stores one checked behavior update in a reusable
named block, and repeated lines append to that block. `use_behavior <name>`
replays the named updates onto the current wave profile, preserving existing
source-backed behavior fields unless the preset changes them.
`use_behavior_waves <first> <last> <name>` replays the same preset onto every
existing wave profile in the range.
`behavior_waves <first> <last> ...` applies that same checked behavior update
to every existing wave profile in the range. The parser rejects range behavior
updates that reference undefined waves and preset uses that reference undefined
presets, which keeps custom scripts explicit about which source-backed or clean
profiles they are tuning.
`spawn_behavior <kind> <index> <field> <value>` lines configure one spawned
actor before the driver has allocated its runtime actor id. They inherit the
current wave/kind behavior at the point where the line is parsed and are then
installed as actor-id behavior profiles when that spawn index is allocated.
`spawn_behavior_preset <name> <field> <value>` stores a reusable spawn-index
profile-field update, and repeated lines append to that preset.
`use_spawn_behavior <kind> <index> <name>` applies that preset to one
spawn-index profile in the current wave.
Indices are per actor kind and advance across the whole wave allocation stream:
initial wave spawns consume the first indices, then source reserve/refill
spawns, pod-created swarmers, baiters, and other command-applied later hostile
spawns consume subsequent same-kind indices. This lets custom progression
scripts tune a specific restored actor without changing the actor struct.
`spawn_behavior_waves <first> <last> <kind> <index> <field> <value>` applies
the same direct spawn-index profile update across an existing wave range.
`use_spawn_behavior_waves <first> <last> <kind> <index> <name>` applies a
spawn behavior preset across an existing wave range.
`lander`, `bomber`, `pod`, `mutant`, `swarmer`, `baiter`, and `human` lines add
clean scripted spawn records; humans default to `grounded` and can also be
declared as `falling <velocity>` or `carried <actor-id>`. The compatible
`reserve` and `enemy_reserve` forms take
`<landers> <bombers> <pods> [swarmers]`; `reserve_full` and
`enemy_reserve_full` take `<landers> <bombers> <pods> <mutants> <swarmers>` and
set all five reserve families explicitly. `source_wave <wave>` and
`source_waves <first> <last>` expand source-backed wave-table profiles,
including source reserve counts and source wave-table metadata, into the same
checked script, so the production default and custom level scripts use the same
parser surface. Either form can also append `<field> <value>` pairs for any
exposed `ActorSourceWaveProfile` field. Single-wave overrides tune one level;
range overrides apply the same source-shaped allocation, restore metadata,
fixed-point movement, AI shots, baiter cadence, and mutant hop/shot constants
to every expanded wave in the range. The driver copies the effective current
profile into `StepPrompt`, so source-backed actors consume these overrides
directly instead of re-reading `wave-table.tsv`. The parser sorts wave profiles
by wave number, rejects duplicate waves, and reports malformed lines with
source line numbers.

Hostile projectile actors publish source-shaped shell metadata too: enemy
lasers own and advance fixed-point velocity, fraction, and lifetime values,
with lifetime decrementing on the source shell-scan cadence. Enemy-shot spawn
commands can also carry source fractions, velocities, and lifetime ticks from
custom drivers and source-backed lander, swarmer, and baiter AI shots.
Source-backed baiter shots use the shared source fireball projection helper,
so their source fractions, X/Y velocities, 20-tick lifetime metadata, and
`USHSND` cue are emitted only after source shell bounds/cap allocation passes.
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
and source target-list slot metadata. The driver owns the source astronaut
process cadence/cursor and prompts one grounded source human target-list slot
per process tick; the selected human applies the driver source RNG seed,
updates picture frame, steps fixed-point X, and steps Y toward
terrain-relative source targets. First-wave inactive source human slots `>= 2`
stay suppressed while the total live human count remains ten.
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
source `ELECV` presents, bounded Hall-of-Fame rows, source-offset
scoring/instruction labels, the scoring scanner surface, and credits opening
sequence from checked text. The default Williams reveal,
`ELECV`, and Defender wordmark events use the same source page-start steps as
the clean attract scheduler: Williams from step 1, `ELECV` from step 236 at
screen address `0x3258`, and the Defender wordmark from step 365. The default
high-score title/table and zero-credit credit line start at the Hall-of-Fame
boundary, step 488; title pages use a `credits_nonzero` event so an inserted
credit can still be shown immediately without drawing a zero-credit line. The
same Hall-of-Fame boundary draws source-offset `HALLD_*` headings, the source
Defender logo, and a `hall_scores` two-column table for the source 60-tick
stall window. After that bounded Hall window, the script starts
`scoring_surface` at step 1088, draws `SCANV` immediately, then reveals
`LANDV`, `MUTV`, `BAITV`, `BOMBV`, `SWRMPV`, and `SWARMV` on the source
scoring-card process cadence from checked message rows. The scoring surface
draws the source top scanner frame/marker bars and the source `MTERR`
mini-terrain records through an actor-local visual effect.
It also projects the source-shaped rescue-demo player, human, and lander object
sprites, matching scanner blips, source-shaped rescue laser pixels, lander
explosion fragments, the 500-point rescue popup, and the `ENMYTB` enemy legend
transfer/materialization/reveal objects from the surface's actor-local
`scoring_tick`. The default checked script declares `cycle 3367`, so the
sequence returns to the first Williams reveal step after the source scoring
demo; custom scripts stay linear unless they declare their own cycle. The older
Rust event constructor remains available as a fallback. Custom drivers can pass
their own parsed or
constructed sequence through
`ActorGameDriver::with_attract_script(...)` without replacing coin/start
control handling.
`ActorGameDriver::script_manifest()` includes the immutable attract-event
manifest so custom drivers can verify or serialize the installed sequence
without inspecting the thread-backed attract actor.
`AttractScript::parse_text(...)` and `str::parse::<AttractScript>()` accept the
same event model from checked text script lines:

```text
# action start duration x y ...
cycle 3367
text 1 forever 10 10 PRESS START
williams_logo 1 487 108 60
message 236 252 ELECV 0x3258
defender_wordmark 365 123 96 144
sprite 2 forever defender_logo 40 44
credits_nonzero 1 487 176 226 248 226
credits 488 forever 176 226 248 226
message 488 600 HALLD_TITLE 0x3854 -11 -6
message 488 600 HALLD_TODAYS 0x2268 -11 -6
message 488 600 HALLD_ALL_TIME 0x6068 -11 -6
message 488 600 HALLD_GREATEST 0x1E72 -11 -6
message 488 600 HALLD_GREATEST 0x5F72 -11 -6
sprite 488 600 defender_logo 85 50
hall_scores 488 600 0x1886 0x5986 -11 -6
scoring_surface 1088 forever
message 1088 forever SCANV 0x4330 -11 -7
message 1505 forever LANDV 0x1C70 -11 -7
message 1691 forever MUTV 0x3C70 -11 -7
message 1871 forever BAITV 0x5F70 -11 -7
message 2051 forever BOMBV 0x1CA8 -11 -7
message 2237 forever SWRMPV 0x40A8 -11 -7
message 2417 forever SWARMV 0x5CA8 -11 -7
```

Blank lines and `#` comments are ignored. `duration` can be a step count or
`-`, `none`, `forever`, or `infinite` for an unbounded event. Parser errors
include the source line number so custom driver tooling can reject malformed
scripts before the actor runtime starts. `cycle`, `loop`, or `repeat` declares
an optional positive script step count that wraps draw selection back to the
first visible actor script step at the boundary; scripts without it do not
loop. `high_scores` lines use
`x y row-height rows` after the timing fields and draw rows from the
driver-owned high-score table carried in `StepPrompt`. `hall_scores` lines use
`todays-screen-address all-time-screen-address offset-x offset-y` and draw
source-shaped Today’s and All-Time table columns with ranks, red-label seed
initials from `assets/red-label/high-scores.tsv`, and current actor score
values when available. `credits` and `credits_nonzero` lines use
`label-x label-y count-x count-y` after the timing fields and draw the
source-backed `CREDV` / `CREDITS:` label plus the visible credit count from
`StepPrompt.credits`; `credits_nonzero` suppresses the draw until at least one
credit exists. `message` / `source_message` lines use
`label top-left-screen-address [offset-x offset-y]` after the timing fields,
validate `label` against `assets/red-label/messages.tsv`, and render through
the source controlled-message glyph path so red-label cursor tokens such as
`[RLF]` and `[HMC:...]` affect layout instead of becoming visible text. When an
offset is present, the renderer applies it after source cursor layout so
scripts can preserve source screen addresses while matching the protected
visual page placement. `scoring_surface` lines take no extra fields and draw
the source-shaped top scanner border plus `MTERR` mini-terrain on the scoring
page.

Script actions currently cover:

- `Text`, for scripted title/status lines.
- `SourceMessage`, for checked red-label message-table labels rendered through
  source cursor controls.
- `Sprite`, for static sprite placement.
- `HighScores`, for prompt-backed one-column high-score table rows in custom
  attract/game-over scripts.
- `HallScores`, for source-shaped Today’s and All-Time Hall-of-Fame table rows.
- `ScoringSurface`, for the source top scanner frame and `MTERR` mini-terrain
  on the scoring page.
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
  target-list slot and picture-frame metadata. The driver prompts one
  source-backed grounded human target-list slot per source astronaut process
  tick; that actor updates walk frame, fixed-point X fraction, source-seeded
  turn branch, and terrain-relative Y step.
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
  and Y-velocity updates from the driver-provided source RNG snapshot only when
  `(SEED & 0x06) >> 1` selects their source slot; all source-backed bomber
  actors still advance fixed-point position from their current velocity.
- Bomber actors can lay first-class `Bomb` actors on a scriptable cadence when
  they are not source-backed. Source-backed bombers instead use the source TIE
  slot selection, `LSEED & 0x07` drop gate, source ten-bomb active cap, shared
  20-shell cap, and source `GETSHL` placement bounds. Bombs draw their own
  sprite, expire through actor-owned lifetime state, act as player hazards, and
  emit a bomb collision cue when they hit the player. Source-backed bomber
  bombs snapshot the bomber's pre-move source fractions as stationary
  bomb-shell fraction metadata, start with `(SEED & 0x1F) + 1` source lifetime
  ticks, and decrement actor-owned lifetime metadata on the source shell-scan
  cadence. Custom source bomb spawns can provide nonzero source lifetime ticks
  directly; otherwise the behavior profile supplies the fallback lifetime.
- Projectile-killed pods spawn a bounded mini-swarmer actor batch using the
  source request count. Those actor spawns now consume the driver source RNG for
  initial X/Y velocity, acceleration, sleep, and shot-timer metadata instead of
  deriving spread from the local spawn index. Source-backed swarmers carry an
  entry horizontal-seek flag, then advance actor-owned fixed-point fractions
  with source-shaped vertical acceleration/damping, turn-window reseek, and
  source `RMAX` shot-timer reset. Source-backed swarmer shots now use the
  source `SWBMB` fireball projection: they suppress fire and sound after the
  swarmer has passed the player or the source shell list is full, otherwise
  they emit X velocity from swarmer `OXV << 3`, Y velocity from the
  player/shell delta, source shell lifetime metadata, and the distinct swarmer
  shot cue. Smart-bomb pod scoring intentionally does not spawn swarmers,
  matching the clean source behavior.
- The driver advances a source-paced baiter timer while source-counted wave
  enemies remain. Expired timers spawn source-backed baiter actors up to the
  source active cap. Baiters pursue and shoot through actor-owned metadata,
  publish three-frame source animation state, gate picture-wrap retargeting
  through the driver-provided source RNG snapshot, fold player velocity into
  source-shaped seek velocity, project source `SHOOT` fireball shells from
  source RNG X/Y deltas plus high-seed player velocity, suppress projectile and
  `USHSND` cue when the shared source shell list is full, score 200 points on
  laser hit, and do not block wave completion once lander/bomber/pod/swarmer
  snapshots are gone.
- Carried humans follow their lander. If the carrier disappears, the human
  falls under a simple acceleration model and emits the source `ASCSND`
  release command.
- Falling humans caught by the player award 500 points, emit the rescue sound
  cue, queue the source repeated `ACSND` tail, and spawn a 500-point popup
  actor.
- Slow falling humans settle safely on the terrain line for 250 points and a
  250-point popup. Fast impacts destroy the human and spawn an explosion.
- When the last human is removed, the driver starts a source
  `TerrainBlowSnapshot`, erases clean terrain and scanner terrain, suppresses
  the normal human-loss cue for that batch, emits the source terrain-blow flash
  and `AHSND` / `TBSND` command cadence, and spawns `TEREX` terrain explosion
  actors at the source birth positions.
- Explosion actors publish family-specific variant metadata for lander, mutant,
  bomber, pod, swarmer, baiter, bomb, player, human, and terrain clouds while
  retaining actor-owned lifetime state. `ActorRenderSceneBridge` maps that age
  through the clean source explosion-size curve, caps the render scale the same
  way as the clean expanded-object path, projects descriptor-backed lander,
  mutant, bomber, pod, swarmer, and baiter clouds through source pixels with
  optional source-center metadata instead of static atlas sprites, and maps
  terrain explosions through the source terrain-explosion growth/lifetime
  curve.
- Smart bomb is now a real driver command: normal player requests consume the
  driver-owned stock, then the driver waits the source three-step detonation
  delay before clearing active hostile actors, awarding enemy scores, and
  spawning explosions while preserving human actors. Accepted smart bombs queue
  the source `SBSND` / cannon command-byte sequence, publish a five-step flash
  countdown that the render bridge consumes as a white clear color, and hold
  source reserve activation behind the source smart-bomb cooldown. Exhausted
  stock leaves hostiles alive. The `XYZZY` overlay smart bomb uses the same
  delayed command path without consuming stock.
- Actor scoring uses the clean replay-bonus threshold model. Enemy, rescue, and
  safe-landing awards update driver-owned life/smart-bomb stock when a threshold
  is crossed, carry the new `next_bonus` into `GameState`, and emit
  `BonusAwarded` through the actor event bridge.
- Player hazard collisions destroy the current player actor, decrement the
  driver-owned life stock, and spawn a replacement player when lives remain.
  Final-life collisions enter the game-over/high-score path.
- Player laser hits now resolve lander, mutant, bomber, pod, swarmer, and baiter
  targets through the driver, awarding source scores and family hit cues for
  bomber, pod, swarmer, and baiter families.
- Wave scripts now apply behavior profiles when play starts and after a
  wave-cleared interstitial report, allowing level difficulty to progress
  through driver-owned data. The clear report keeps the current wave number,
  surviving human snapshots, `WaveCleared`, and the source `ATTACK WAVE` /
  `COMPLETED` / `BONUS X` plus survivor-icon overlay. The following actor step
  clears transient playfield actors, installs the next wave script, spawns the
  next wave's actors, and emits `WaveStarted`.

These mechanics are still intentionally compact. The next fidelity slices
should port the remaining source restore positions against the actor state,
render, and audio bridges now used by the default live runtime, then retire
isolated scaffolding as parity improves.
