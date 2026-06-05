# Actor Architecture Rewrite

`src/actor_game.rs` is the first isolated slice of the Rust actor-oriented
rewrite. It does not replace the live `Game` runtime yet. The current runtime
remains available for fidelity evidence while the actor model is built out and
verified.

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
  draws carry `ExplosionKind` metadata so bomb, enemy, player, and human
  explosions can map to separate source sprite families later. Audio is
  described by `SoundCue`. The future renderer/audio bridge can translate those
  descriptions into `wgpu` sprites and Williams sound-board commands.

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
cooldown, player hyperspace hiding/rematerialization, laser speed and lifetime,
lander seek/carry/fire behavior, mutant movement, bomber movement/bomb cadence,
pod movement, swarmer movement/fire behavior, baiter movement/fire behavior,
bomb lifetime, human fall/landing behavior, and timed effect lifetimes. It also
holds behavior modes such as
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

Hyperspace is represented as a driver-applied gameplay command rather than a
render frame effect. The player actor emits `GameCommand::Hyperspace` and
`SoundCue::Hyperspace` from the mapped `H` input; the driver clears active
`EnemyLaser` projectile actors while leaving player lasers, hostile actor
families, score, lives, and smart-bomb stock unchanged. `PlayerShip` then owns
the hidden interval: while its hyperspace timer is active it remains alive but
publishes no collision bounds, no player draw, and no input-driven actions.
`ActorBehaviorProfile` configures the hidden step count and rematerialization
coordinates. The later MAME source RNG and death-risk branch remains a separate
porting slice.

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

The default actor progression reads `assets/red-label/wave-table.tsv` through
an actor-owned adapter. The current actor mapping uses source-backed
`wave_size`, `lander_x_velocity`, `bomber_x_velocity`, `lander_shot_time`,
`baiter_time`, `baiter_shot_time`, `baiter_seek_probability`, `bombers`, and
`pods` to set active family allocation, movement speed, fire cadence, and baiter
entry pacing. Wave `1` remains lander-only; later source waves seed one
lander, one bomber, and one pod when those reserve counts are available before
filling the rest of the active batch, matching the clean source allocator's
family order. Wave `1` uses the source first-wave lander restore metadata from
the existing clean evidence, including fixed-point fractions, velocities, shot
timer, sleep ticks, picture frame, and target-human index. Source-backed
landers, bombers, pods, and baiters publish their metadata in snapshots and advance
active motion by updating their own fixed-point position/fraction state.

Initial humans are source-backed for wave `1` as well. Their actor spawns carry
fixed-point fractions, picture frame, and source target-list slot metadata.
Grounded source humans advance their own walk cadence, picture frame, and
fixed-point X fraction after the initial source sleep ticks.
Source-backed landers ask the prompt for their configured target slot first and
only fall back to nearest-human seeking when that target is unavailable.

## Attract Graphics

The attract screen is data-driven. `AttractScript` contains ordered
`AttractScriptEvent` records, and `ScriptedAttractProgram` turns the active
events for its own current script step into draw commands. The default
`AttractScript::red_label_title()` recreates the current Williams/logo/high
score opening sequence, while `ActorGameDriver::with_attract_script(...)` lets
a custom driver provide its own sequence without replacing coin/start control
handling.

Script actions currently cover:

- `Text`, for scripted title/status lines.
- `Sprite`, for static sprite placement.
- `WilliamsLogo`, which emits `SpriteKey::WilliamsLogo` with
  `VisualEffect::WilliamsReveal` metadata for handwriting reveal and title
  color phase.
- `DefenderWordmark`, which emits `SpriteKey::DefenderCoalescence` with slot
  and row-pair metadata until the wordmark settles to
  `SpriteKey::DefenderWordmark`.

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
  seeking.
- Lander shot timers emit both the laser sound cue and an `EnemyLaser` actor.
  Enemy lasers are player hazards, smart-bomb targets with no score value, and
  respect the same player damage behavior profile used by `XYZZY` invincibility.
- Later source waves seed bomber and pod actor families when the wave table
  exposes those counts. Bombers and pods draw their own sprites, move through
  actor-owned source fixed-point metadata when source-backed, and remain
  script-tunable through their behavior profiles.
- Bomber actors can lay first-class `Bomb` actors on a scriptable cadence with
  the source ten-bomb active cap. Bombs draw their own sprite, expire through
  actor-owned lifetime state, act as player hazards, and emit a bomb collision
  cue when they hit the player.
- Projectile-killed pods spawn a bounded mini-swarmer actor batch using the
  source request count. Source-backed swarmers decrement their actor-owned shot
  timer into hostile projectile commands with a distinct swarmer shot cue.
  Smart-bomb pod scoring intentionally does not spawn swarmers, matching the
  clean source behavior.
- The driver advances a source-paced baiter timer while source-counted wave
  enemies remain. Expired timers spawn source-backed baiter actors up to the
  source active cap. Baiters pursue and shoot through actor-owned metadata,
  publish three-frame source animation state, score 200 points on laser hit, and
  do not block wave completion once lander/bomber/pod/swarmer snapshots are
  gone.
- Carried humans follow their lander. If the carrier disappears, the human
  falls under a simple acceleration model and emits the release sound cue.
- Falling humans caught by the player award 500 points, emit the rescue sound
  cue, and spawn a 500-point popup actor.
- Slow falling humans settle safely on the terrain line for 250 points and a
  250-point popup. Fast impacts destroy the human and spawn an explosion.
- Explosion actors publish variant metadata for enemy, bomb, player, and human
  explosion clouds while retaining actor-owned lifetime state.
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
should port the remaining source restore positions and bind the draw/sound
descriptions to the source sprite and Williams sound-board assets.
