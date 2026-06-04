# Actor Architecture Rewrite

`src/actor_game.rs` is the first isolated slice of the Rust actor-oriented
rewrite. It does not replace the live `Game` runtime yet. The current runtime
remains available for fidelity evidence while the actor model is built out and
verified.

## Structure

- `ActorGameDriver` owns game phase, score, credits, lives, high scores, actor
  ids, actor threads, and the latest snapshots.
- Each asset is a Rust struct implementing `AssetActor`. The current slice has
  `AttractDirector`, `ScriptedAttractProgram`, `PlayerShip`, `Lander`,
  `Mutant`, `Human`, `LaserShot`, `Explosion`, and `ScorePopup`.
- `ThreadedAsset` runs one actor on one Rust thread. The driver sends one
  `StepPrompt` per simulation step and waits for one `ActorReply`, keeping
  behavior deterministic while still matching the requested thread-per-asset
  shape.
- Actors do not mutate the world directly. They return snapshots, draw
  commands, sound cues, and gameplay commands. The driver resolves collisions
  and applies world rules in stable actor-id order.
- Rendering is described by `DrawCommand`, `SpriteKey`, and `VisualEffect`
  values. Audio is described by `SoundCue`. The future renderer/audio bridge can
  translate those descriptions into `wgpu` sprites and Williams sound-board
  commands.

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
cooldown, laser speed and lifetime, lander seek/carry/fire behavior, mutant
pursuit, human fall/landing behavior, and timed effect lifetimes. It also holds
behavior modes such as `LanderBehaviorMode`, allowing scripts to choose whether
a lander seeks humans, chases the player, or simply drifts.

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
and chase the player.

`XYZZY` invincibility uses the same mechanism. When invincibility is active,
the driver applies a temporary player behavior override that disables enemy
collision damage for that simulation step; collision handling reads that
profile rather than branching on a separate god-mode flag.

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

- Starts seed the playfield with ten human actors in target-list-like ground
  positions.
- Landers seek the nearest grounded human, attach it through an
  `AttachHuman` command, carry it upward, and convert into a mutant when the
  carried human reaches the upper conversion band.
- Carried humans follow their lander. If the carrier disappears, the human
  falls under a simple acceleration model and emits the release sound cue.
- Falling humans caught by the player award 500 points, emit the rescue sound
  cue, and spawn a 500-point popup actor.
- Slow falling humans settle safely on the terrain line for 250 points and a
  250-point popup. Fast impacts destroy the human and spawn an explosion.
- Smart bomb is now a real driver command: it removes active hostile actors,
  awards enemy scores, and spawns explosions while preserving human actors.

These mechanics are still intentionally compact. The next fidelity slices
should replace the baseline behavior profiles with MAME-backed tables and bind
the draw/sound descriptions to the source sprite and Williams sound-board
assets.
