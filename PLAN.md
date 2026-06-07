# Defender Current Plan

Last reviewed: `2026-06-07`

## Goal

Make this repository's clean `wgpu` Defender implementation look, sound, and
play like the original Williams Defender red-label game.

Acceptance is against local MAME red-label golden artifacts, not against broad
impression or memory-table similarity. The clean runtime must preserve gameplay
behavior while using modern game-programming structure: domain systems,
renderer-owned sprites, `wgpu` rendering, clean audio, and deterministic
verification tools.

## Current Baseline

- Active branch for the actor-architecture cycle: `dev/actor-game-architecture`.
- The production baseline came from `rewrite`; normal interactive live play now
  uses the actor runtime through the clean platform/audio/`wgpu` bridge, while
  the clean `Game` path remains available for smoke, fidelity, and oracle
  evidence commands.
- The actor rewrite now has focused coverage for same-contract input,
  `XYZZY`, data-driven `AttractScript` custom driver sequencing,
  Williams/Defender attract metadata, player/lander/laser/explosion basics,
  initial humans, lander pickup/carry/conversion, rescue/safe-landing scoring,
  replay-bonus stock awards, score popups, smart bomb clearing, wave-clear
  interstitial presentation, and high-score/game-over phase handoff.
  The actor API is simulation-step driven through `StepPrompt`/`StepReport`,
  not display-frame driven; attract scripts advance on actor-local elapsed
  steps so custom drivers can own sequencing without MAME frame scripts.
  Actor movement and behavior are now scriptable through
  `ActorBehaviorScript`, with default, actor-kind, and actor-id behavior
  profiles plus script-selectable behavior modes for level difficulty and
  `XYZZY` damage overrides. Non-source mutant, bomber, pod, swarmer, and
  baiter fallback motion can now be script-selected between drift and
  player-chase behavior while source-backed fixed-point metadata remains the
  higher-priority movement source. Read-only script manifests now expose the
  configured attract events, behavior profiles, reusable wave/spawn behavior
  preset definitions, and wave profiles so custom drivers can inspect their
  installed scripts without actor-thread introspection. Attract, behavior, and
  wave scripts can now be parsed from checked text lines before installation in
  a custom driver, and `ActorDriverScripts::parse_texts` can install all three
  as one checked custom-driver bundle, while `ActorDriverScripts::parse_text`
  accepts a single sectioned custom-driver script with `[attract]`,
  `[behavior]`, and `[wave]` blocks. Bundled wave parsing inherits the parsed
  behavior baseline before applying wave-local/source-backed overrides. The
  sectioned custom-driver bundle now implements
  `str::parse::<ActorDriverScripts>()`, exposes a pre-driver
  `ActorDriverScripts::manifest()`, and can boot the actor runtime directly
  through `ActorRuntimeAdapter::with_scripts`. The live actor runtime also
  accepts `--actor-script <path>` to load one checked sectioned custom-driver
  script before the window opens, while rejecting that option with
  `--live-smoke` so a custom actor script is never silently ignored by the
  built-in actor smoke path. `--actor-script-check <path>` now runs the same
  parser and runtime constructor headlessly, samples the first attract actor step,
  reports bounded declared attract-cycle milestones for custom attract scripts,
  credits/starts the actor runtime through the first playable wave, and reports
  manifest, first-frame, effective source-wave, spawned world counts,
  reserve/source-state, source-backed actor placement samples, first-play
  source projectile and source sound-command samples, player-laser
  position/velocity/audio samples, and the effective first-play behavior
  profile actors receive for movement/damage/fire tuning. It also runs bounded
  independent sample passes to report the first player laser plus its `0xEB`
  red-label sound command, the first player-laser-hit explosion plus its
  red-label hit command when a script produces one, a parser-backed hostile
  laser-hit matrix for lander, mutant, bomber, pod, swarmer, and baiter
  score/explosion/sound evidence, a parser-backed hostile projectile matrix for
  lander, mutant, swarmer, and baiter shot-command and projectile-metadata
  evidence, and the first hostile source projectile plus its associated
  red-label hostile shot command when a script produces one, without mutating
  the main assisted progression check.
  It then uses the actor `XYZZY` overlay smart-bomb path to assist wave clear
  and reports the source-shaped wave-clear survivor-bonus interstitial plus the
  source `0x80` wave-advance sleep before the next playable wave when the
  normal actor survivor-bonus and wave-start logic reaches them, including
  source/reserve counts and effective behavior. When that next wave still has
  enemy reserves, the checker keeps stepping through smart-bomb cooldown and
  reserve activation to report a bounded restored reserve batch sequence,
  including each batch's spawned family counts, resulting source/reserve state,
  restored spawn positions, terminal batch status, and the first following
  post-reserve wave-clear interstitial plus its `0x80` wave-advance sleep after
  those source reserves are empty, followed by the next playable wave state
  after that post-reserve sleep.
  `examples/actor-custom-attract.script` provides a checked bounded editable
  starting point with Williams reveal, Defender coalescence, Hall-of-Fame,
  credit, and scoring-surface attract actions. The built-in actor attract,
  behavior, and wave scripts are embedded from
  `assets/red-label/actor-attract.script`,
  `assets/red-label/actor-behavior.script`, and
  `assets/red-label/actor-waves.script` through the same parser path. The
  attract script now draws the source `ELECV` presents message, source-style
  Hall-of-Fame table rows, source-offset scoring/instruction labels, source
  `CREDV` credits label/count, and a scriptable scoring scanner surface
  alongside the Williams reveal and Defender wordmark coalescence. Its source
  page-start steps are Williams from step 1,
  `ELECV` from step 236, the Defender wordmark from step 365, the
  high-score/zero-credit Hall-of-Fame page from step 488 for the source 60-tick
  stall window, and the scoring/instruction page from step 1088. Title pages
  suppress the zero-credit line but still show real inserted credits through a
  `credits_nonzero` script action. The Hall-of-Fame page also draws
  source-offset `HALLD_*` headings and the source Defender logo; `hall_scores`
  draws Today’s and All-Time table columns from driver scores plus embedded
  red-label seed initials. The scoring/instruction page draws `SCANV` at step
  1088, then reveals `LANDV`, `MUTV`, `BAITV`, `BOMBV`, `SWRMPV`, and `SWARMV`
  on the source scoring-card process cadence from checked `messages.tsv` rows
  and source screen addresses, while `scoring_surface` draws the source top
  scanner frame/marker bars, `MTERR` mini-terrain records, source-shaped
  rescue-demo player/human/lander object sprites, scanner blips,
  source-shaped rescue laser pixels, lander explosion fragments, the 500-point
  rescue popup, and the `ENMYTB` enemy legend transfer/materialization/reveal
  objects before the embedded script cycles back to the first Williams reveal
  step at source step 3367. Custom attract scripts can draw checked
  `messages.tsv` labels through source cursor controls with optional visual
  offsets, can opt into looping with `cycle <step-count>`, and otherwise remain
  linear; the older one-column `high_scores` action remains available for
  custom screens.
  `ActorWaveScript` now names per-wave progression data and applies behavior
  scripts plus hostile and initial-human spawn records as play starts and waves
  are cleared. Checked wave scripts can declare clean `lander`, `bomber`,
  `pod`, `mutant`, `swarmer`, `baiter`, and `human` initial-spawn records.
  Wave scripts can now attach spawn-index behavior profiles that become
  actor-id profiles after the driver allocates those actors, and range behavior
  directives can apply the same checked behavior or spawn-index profile update
  across existing source-backed progression bands. They can also define named
  checked behavior presets and apply them to one wave or an existing wave
  range, so custom drivers can reuse difficulty blocks while preserving
  source-backed wave motion unless explicitly overridden. Spawn-index behavior
  can use named presets too, so custom drivers can reuse the same specific
  actor-slot tuning across source reserve/refill progression without repeating
  field lines. Default actor wave
  progression expands the checked wave script through
  `assets/red-label/wave-table.tsv` for active wave size, lander and bomber
  speed, lander fire cadence, source bomber/pod/direct-mutant/swarmer counts,
  baiter timing, and mutant hop/shot fields. Source-backed wave-profile
  manifests preserve the effective `ActorSourceWaveProfile` record expanded
  from that table or tuned by parsed `source_wave` / `source_waves` field
  overrides, so custom drivers can inspect source progression values without
  driver internals. The actor wave allocator follows the source active-family
  shape, so later waves now seed bomber, pod, direct-mutant, and swarmer actors
  beside landers when those counts are present. Source-backed actors consume
  the driver-owned effective source wave profile from `StepPrompt`, and
  `StepReport` carries that same profile into `ActorStateBridge` so clean
  `GameState::wave_profile` mirrors script-expanded or script-overridden source
  progression. Scripted level tuning changes allocation, fixed-point movement,
  source AI shots, baiter cadence, and mutant hop/shot behavior through the
  same data surface for one level or a whole source-backed range. Source-backed
  landers, bombers, pods, swarmers, and initial humans publish fixed-point
  metadata and advance actor-owned fraction state during active motion, and
  source landers prefer configured target slots before falling back to
  nearest-human seeking.
  Source
  wave profiles now retain post-active-batch enemy reserve counts, parsed wave
  scripts can set custom reserves with `reserve` / `enemy_reserve` or the
  all-family `reserve_full` / `enemy_reserve_full` form, and `StepReport` maps
  the current reserve counts into the clean world snapshot. Reserve activation
  is armed only after the active source batch has published a report. When
  source-counted hostiles are gone, the driver restores source reserve batches
  before wave clear: lander reserves fill active slots first, no-human lander
  reserve rows restore as source-shaped mutants through the source schizoid
  fallback, then bomber, pod, direct mutant, and swarmer reserves use source
  placement/fraction restore paths once landers are exhausted. While first-wave
  source-counted hostiles remain active, the actor driver now runs the source
  449-step early reserve cadence, materializes the five MAME-observed early
  reserve lander rows with the `0xEA` appearance cue, resets the source
  target-list cursor/RNG window, and leaves the later first-wave lander reserve
  intact for refill behavior. Once active first-wave landers fall below the
  source threshold, the actor driver now runs the source 47-step refill cadence,
  restores the fixed refill rows, keeps the four hidden lanes live without
  leaking draw/collision output or clean scanner/object evidence, and
  materializes only the target-3 lane with the delayed `0xEA` cue. Those hidden
  lanes are source bookkeeping evidence, not wave-clear blockers: after the
  visible source-counted hostiles are gone and reserves are empty, survivor
  bonus and next-wave progression can start. Gameplay terrain and the
  non-marker top-display separator now follow the documented eight-wave color
  cycle: blue, green, red, orange, yellow, purple, brown, then black.
  Player altitude is clamped to the source playfield top, so the ship cannot
  move above the bottom of the HUD/top-display separator.
  Source-backed hostile, human, enemy-shot, and bomb actors stay in source
  world space while actor render and collision paths project their
  draw/collision bodies through the current background word, so thrust
  scrolling moves the view instead of carrying aliens, fireballs, or bomb
  shells with the player. Falling-human rescue uses the same projected human
  body while clean state keeps the source high byte/fraction for scanner and
  object evidence. Source swarmer
  reserves use `PLRES`/`RSW0` phony-object placement before the same
  mini-swarmer runtime used by pod destruction. Source
  rematerialization now publishes the `SEED/HSEED` background word into
  driver-owned `StepPrompt` / `StepReport` state, and actor mutant reserve
  restore paths consume that background word instead of a fixed zero. Source
  lander/human conversions now spawn mutant actors with source-shaped mutant
  fractions, wave-derived shot timer, driver-owned hop RNG, and clean
  `SourceMutantSnapshot` bridge metadata.
  The actor driver now owns the source astronaut process cadence/cursor: one
  grounded source-backed human target-list slot receives the source RNG seed
  per process tick, walks fixed-point X, and steps Y toward terrain-relative
  source targets, while first-wave inactive slots stay suppressed until the
  total live human count changes from ten.
  Source-backed mutant actors now
  consume that metadata to select wave-table X/Y velocities, advance
  actor-owned hop RNG and fixed-point fractions, and emit source-shaped
  `0xF6` hostile projectile commands from their shot timers. The
  actor rewrite now has source-RNG-restored later-wave lander placement, shot
  timers, X/Y velocity metadata, and later-wave human target-list restores. The
  actor clean-state bridge now preserves `StepReport::source_rng` in
  `WorldSnapshot::source_rng`, so the runtime state contract exposes the same
  driver-owned source RNG that source-backed movement and projectile actors
  consume from `StepPrompt`. It
  also carries first-wave target6 converted-mutant source correction metadata,
  target6 dive/visual projection anchors, deferred visible-entry shot state,
  source-shaped target6 dive shot-position overrides, and exact fire2524
  projectile fractions/velocities through the actor source-mutant command path.
  The actor collision resolver also suppresses the pending target6 fire2524
  collision window, uses target6 projected collision positions, and emits the
  source-positioned enemy explosion, mutant-hit cue, and score award before the
  normal player-death command path.
  It also has a persistent `StatusDisplay` actor that draws
  score, high score, wave, lives, smart-bomb stock, credits, and
  high-score-entry rows from the same `StepPrompt` state as gameplay actors
  while staying inert during attract so custom attract scripts retain control of
  that screen. Actor `StepPrompt`/`StepReport` values now carry the driver-owned
  current player, player count, per-player scores, and per-player stock
  snapshots. Credit-gated actor evidence starts preserve red-label admission:
  one-player starts consume one credit, two-player starts require and consume
  two credits, and blocked start requests no longer emit false clean
  `GameStarted` / `WaveStarted` events. Normal interactive live play enables
  actor free-play admission so `1` and `2` start from a fresh run without first
  pressing a coin key. Score and replay-bonus stock awards route through the
  active player fields so the actor bridge can represent player-two score and
  stock ownership. Accepted one-player and two-player starts now publish a
  source-length actor player-start delay and delay actor playfield spawning plus
  `WaveStarted` until the delay expires. Accepted-start reports are silent,
  emit the start sound on the following actor step, and emit the source `0xEA`
  appearance cue when the delayed playfield starts. One-player starts keep the
  playfield empty without drawing the two-player source prompt; credited
  two-player starts render the source `PLAYER ONE` prompt through the actor
  render bridge. Player hazard deaths now decrement the active player's stock,
  enter a source-length `0x60` actor player-switch sleep when another player has
  stock, publish the switch through `GameOverSnapshot`, suppress the attract
  script during the handoff, render the source `PLAYER ONE` / `PLAYER TWO` plus
  `GAME OVER` switch prompts, and start the next stocked player's delayed
  source-prompt actor turn when the switch sleep expires. Focused actor
  regressions now lock the full source-glyph switch prompt across the full
  source sleep, prove it clears at the handoff, and verify the next
  player-start prompt owns the delayed start interval. The remaining
  two-player fidelity boundary is MAME media proof beyond that source-message
  report/render contract. Actor low-score final deaths now expose a finite
  `player_death_sleep_remaining` `GAME OVER` interstitial, suppress attract
  drawing during that countdown, and return to Williams attract only after the
  countdown completes. Actor high-score initials
  submission now reports accepted and submitted initials through the clean event
  bridge, enters a finite 60-step
  Hall-of-Fame game-over stall through `GameOverSnapshot`, draws the source
  Hall page during that stall, and returns to the Williams attract reveal after
  the stall. Lander
  shot timers now spawn hostile projectile actors that use the same player
  damage policy as other hazards, including `XYZZY` invincibility overrides,
  and emit a distinct source-command-backed lander-shot cue. Actor sound cues
  now expose their red-label Williams sound-board command byte where existing
  source evidence pins one, including player laser, lander/mutant/non-lander
  family hits, hostile shots, human release/rescue/loss, and safe landing, and
  actor rescue now queues the repeated source `ACSND` tail; unproven semantic
  cues remain unmapped. `ActorSoundEventBridge` now converts actor
  `StepReport` cue streams into the clean `SoundEvent` surface used by live
  audio, including thrust start/stop edges derived from actor cue state. Bomber
  actors now lay first-class bomb hazards with source bomb-collision cues, pod
  laser kills spawn bounded mini-swarmer actors with source RNG-derived
  velocity, acceleration, sleep, and shot-timer metadata, source swarmers now
  perform the source entry seek and fixed-point vertical acceleration/damping
  loop before using the source `SWBMB` fireball projection with pass-player and
  shell-cap suppression, source shell lifetime metadata, and distinct shot
  cues. Source-backed bomber actors now follow the source TIE selected-slot
  picture/velocity/sleep branch, advance fixed-point position separately, and
  emit bomb-shell commands only through the source `LSEED & 0x07` gate,
  pre-move shell position/fractions, `GETSHL` bounds, shared shell cap,
  ten-bomb shell cap, and `(SEED & 0x1F) + 1` lifetime ticks. Source-paced
  baiter timer entry now spawns
  source-backed baiter actors that can pursue the player and fire source
  `SHOOT` fireballs with source RNG velocity/player-velocity contribution,
  source shell-cap suppression, and `USHSND` evidence only when allocation
  succeeds, without blocking wave completion after source-counted enemies are
  gone. Smart bombs
  now use driver-owned stock: normal player requests consume stock before
  the source three-step detonation delay clears hostile actors, exhausted stock
  leaves hostiles alive, and `XYZZY` overlay smart bombs use the same delayed
  command path without consuming stock. Accepted actor smart bombs queue the
  source `SBSND` / cannon command-byte sequence, publish a five-step white
  flash through report/render state, and hold source reserve activation behind
  the source smart-bomb cooldown. Actor playing reports now project source
  `BGOUT` terrain into the clean state/render bridge. Removing the last human
  starts a driver-owned source terrain-blow path that erases clean terrain and
  scanner terrain, publishes `TerrainBlowSnapshot` state, projects source flash
  windows plus `TEREX` terrain explosion births/growth/lifetime, and emits the
  source `AHSND` / `TBSND` command cadence with tail commands. Player hazard
  collisions now decrement
  driver-owned life stock and spawn a replacement player while lives remain,
  with final-life collisions entering the game-over/high-score path. Actor wave
  clear now publishes a separate interstitial `StepReport` with `WaveCleared`,
  surviving human snapshots, source `ATTACK WAVE` / `COMPLETED` / `BONUS X`
  text, source-shaped survivor scoring at `100 * min(wave, 5)`, one visible
  `ASTP3` survivor icon per awarded human on the source four-step cadence, and
  the source `0x80` wave-advance sleep before the following wave installs its
  script, spawns its actors, and emits `WaveStarted`. Explosion
  draws now carry lander, mutant, bomber, pod, swarmer, baiter, bomb, player,
  and human variant metadata through the actor render and clean-state bridges,
  and actor render output now uses draw age with the clean source
  explosion-size curve; descriptor-backed enemy-family clouds route through the
  clean source expanded-object pixel renderer instead of static atlas sprites,
  preserving separate source top-left and center metadata for target6 `SCZP1`
  collision explosions.
- Primary runtime source is `src/`; the converted implementation is parked in
  `src_legacy/` and should remain optional oracle/tooling evidence only.
- Normal interactive live play now uses the actor runtime through
  `ActorRuntimeAdapter`, clean audio events, and the existing `wgpu` renderer
  path; the clean `Game` path remains available for smoke, fidelity, and oracle
  evidence commands.
- MAME 0.287 is installed locally and verifies the repo ROM set:
  `assets/roms/defender/` reports `romset defender is good`.
- The repeatable MAME capture target is available:
  `make reference-mame-capture`.
- The short executable MAME recorder proof is available:
  `make reference-mame-smoke`.
- Scripted MAME and clean candidate capture targets are available:
  `REFERENCE_SCENARIO=... make reference-mame-capture` and
  `REFERENCE_SCENARIO=... make reference-clean-capture`.
- Scripted MAME trace exploration can run without media capture by setting
  `MAME_REFERENCE_TRACE_ONLY=1`; this writes expected/debug TSVs only and is
  the preferred path for isolating rescue, terrain, materialization, and
  sound-command windows before recording bounded video/audio clips.
- Trace-only MAME captures can now state-steer rare red-label routine windows
  with `MAME_REFERENCE_STATE_STEER=afall_fall`, `afall_safe_landing`,
  `afall_player_catch`, `terrain_blow`, `enemy_explosion_matrix`, or isolated
  sound-command steers `sound_command_fe`, `sound_command_fa`,
  `sound_command_f8`, and `sound_command_f3`, plus
  `MAME_REFERENCE_STATE_STEER_FRAME=<frame>`. This is tooling-only evidence:
  the original MAME code still advances the seeded process/object state.
- MAME capture can also write per-frame sound-board DAC evidence with
  `DEFENDER_TRACE_SOUND_DAC_OUTPUT`; `make reference-mame-capture` now stores
  this as `target/reference-media/mame/traces/<basename>.sound-dac.tsv`.
- `make reference-window-scan` scans generated MAME expected/debug TSVs for
  target sound-command bytes near non-lander object evidence plus terrain
  status / `TERBLO` process evidence, and
  `REFERENCE_WINDOW_SCAN_EXCLUDES='...'` can exclude synthetic/state-steered
  paths when searching for organic trace windows. The JSON report includes
  nearest sound/object misses, `ASTCNT` distribution, terrain process misses,
  and last-human terrain-blow candidate rows.
- `make reference-window-scan-organic` runs the same scan with the standard
  synthetic/state-steered exclusions and writes
  `target/reference-media/reference-window-scan-organic.json`, preserving the
  all-trace report at `target/reference-media/reference-window-scan.json`.
  Window scans now also include per-family object row counts, the longest
  contiguous object-evidence spans, and the best span for each object family,
  so remaining sprite/coalescence media searches can start from concrete MAME
  frame ranges.
- Clean candidate captures can state-steer the same bounded windows with
  `CLEAN_REFERENCE_STATE_STEER=...`, `CLEAN_REFERENCE_STATE_STEER_FRAME=...`,
  and `CLEAN_REFERENCE_CAPTURE_START_FRAME` /
  `CLEAN_REFERENCE_CAPTURE_END_FRAME` so generated GIF/WAV/debug artifacts can
  be compared against the MAME state-steered clips without encoding the full
  prelude.
- Current local golden capture artifacts are ignored under
  `target/reference-media/mame/`, including
  `defender-red-label-golden-60s.mp4` and matching `.wav`.
- Current candidate media can be generated under
  `target/reference-media/clean/`. Timestamp-aligned acceptance reports pass
  for scoring laser/explosion, delayed-start fire/reverse, delayed-start
  thrust/reverse, delayed-start smart bomb, delayed-start
  enemy-shot/background audio windows, gameplay hyperspace death, the first live
  gameplay laser-hit clip, the down030 post-death laser all-axis window, and the
  bounded non-lander target6 shot/explosion/materialization clip. The bounded
  pickup/pull, player-catch, safe-landing, terrain-blow, and organic non-lander
  hold-down visual plus PRBP1 pod all-axis clips also have passing
  MAME-vs-clean media reports.
  Clean debug TSV rows include lander, mutant, terrain-blow, object-evidence,
  expanded-object, sprite, sound, and gameplay state needed for MAME
  side-by-side diagnosis.
  The earlier
  `scripted-fire-reverse-smoke` report is retained as harness evidence only
  because its MAME window is still in the Williams attract sequence while clean
  has already reached gameplay.
- Initial timestamp inventory is tracked in
  `docs/fidelity/mame-golden-clips.md`; generated contact sheets remain under
  ignored `target/reference-media/inventory/`.
- Release closure evidence and proof boundaries are tracked in
  `docs/fidelity/release-closure-audit.md`.
- The accepted media report list is tracked in
  `docs/fidelity/reference-report-gate.json`; `make reference-report-gate`
  checks the current ignored report JSON files as one closure gate, including
  per-facet `min_reports` breadth floors, duplicate-manifest rejection, and
  explicit matching manifest/report `acceptance_mode` declarations plus
  declared/mode-compatible coverage tags for the accepted proof set. Accepted
  report JSON stays under `target/reference-media/`; MAME reference artifacts
  stay under `target/reference-media/mame/`, and clean candidates stay under
  `target/reference-media/clean/`. Accepted visual proof uses MAME MP4 and
  clean GIF artifacts; accepted audio proof uses WAV artifacts. The generated
  signoff summary exposes both the manifest and local report acceptance modes.
- The latest full `make release-gate` pass completed on
  `2026-06-07 17:50 BST`. There are no known active implementation blockers in
  the tracked plan. Final closure requires owner review of the accepted proof
  set, or a new concrete MAME clip/input program showing a mismatch outside the
  accepted windows.
- Attract/scoring silence is not automatically a bug. Only add attract sound
  where the MAME golden capture or sound-command evidence proves it exists.

## Work Protocol

- Post a Slack update to `xyzzytools.slack.com#codex` before and after every
  implementation cycle, step, or milestone.
- Keep `README.md`, `SPEC.md`, and `PLAN.md` synchronized when behavior,
  workflows, targets, or acceptance criteria change.
- Do not implement from intuition. Use local MAME captures, sound-board source,
  ROM/source tables, or existing accepted fixtures as evidence.
- Store generated reference media under ignored `target/` paths. Do not commit
  generated MAME MP4/WAV/AVI artifacts.
- Store production sprite assets under `assets/sprites/`. Store new non-legacy
  sound artifacts under `assets/sounds/`; keep archived legacy cues under
  `assets/arcade/`.
- Production modules must describe game concepts. ROM labels, assembler names,
  and memory-table details belong in legacy/tooling/evidence boundaries.
- Treat converted frame, ROM, and memory-oriented red-label code as temporary
  oracle/evidence scaffolding. As soon as an actor/system implementation and
  accepted MAME-backed artifacts no longer require a legacy adapter, remove it
  or keep it only behind `legacy-tools` until the next bounded retirement slice.
- Do not add new open-ended roadmap phases. Any new work must map to the finite
  milestones below or be recorded as a specific blocker.

## Validation Gates

Focused implementation cycle:

```sh
cargo fmt --check
cargo test <focused-test-filter> --lib
cargo check --all-targets --features legacy-tools
cargo clippy --all-targets --features legacy-tools -- -D warnings
```

Reference-media tooling cycle:

```sh
make media-script-test
make reference-window-scan
make reference-window-scan-organic
make reference-report-gate
make reference-signoff-summary
make reference-evidence-package
make owner-review-package
make reference-mame-doctor
make reference-mame-smoke
```

Docs-only cycle:

```sh
markdownlint README.md SPEC.md PLAN.md docs/fidelity/mame-golden-clips.md \
  docs/fidelity/release-closure-audit.md assets/sounds/README.md
git diff --check
```

Release gate:

```sh
make release-gate
```

Expanded release-gate command sequence:

```sh
cargo fmt --check
cargo test --all-targets
cargo test --all-targets --features legacy-tools
cargo clippy --all-targets -- -D warnings
cargo clippy --all-targets --features legacy-tools -- -D warnings
make clean-fidelity
make media-script-test
make owner-review-package
make reference-mame-doctor
make reference-mame-smoke
make readme-media
cargo run -- --game-smoke
cargo run -- --actor-smoke
cargo run -- --actor-attract-smoke
cargo run -- --actor-post-game-smoke
cargo run -- --actor-wgpu-smoke
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md docs/fidelity/mame-golden-clips.md \
  docs/fidelity/release-closure-audit.md assets/sounds/README.md
git diff --check
```

When clean candidate media is aligned to the MAME capture, add:

```sh
REFERENCE_MEDIA=target/reference-media/mame/defender-red-label-golden-60s.mp4
make reference-media-check \
  REFERENCE_MEDIA="$REFERENCE_MEDIA" \
  REFERENCE_AUDIO=<mame-reference-wav> \
  CANDIDATE_MEDIA=<clean-candidate-video-or-gif> \
  CANDIDATE_AUDIO=<clean-candidate-wav> \
  REFERENCE_MEDIA_REFERENCE_START_MS=<mame-window-start-ms> \
  REFERENCE_MEDIA_CANDIDATE_START_MS=<clean-window-start-ms> \
  REFERENCE_MEDIA_DURATION_MS=<event-window-ms> \
  REFERENCE_MEDIA_ACCEPTANCE_MODE=<all|visual|audio> \
  REFERENCE_MEDIA_REPORT_ONLY=1
```

## Golden Reference Workflow

1. Capture MAME red-label reference media from repo ROMs:

   ```sh
   make reference-mame-capture MAME_REFERENCE_SECONDS=60 \
     MAME_REFERENCE_BASENAME=defender-red-label-golden-60s
   ```

2. Extract frame sheets, audio metrics, and relevant timestamps from the MAME
   MP4/WAV. Keep these under `target/reference-media/`.

3. Generate a clean candidate from the same input program and inventoried time
   window:

   ```sh
   make reference-clean-capture REFERENCE_SCENARIO=firing \
     CLEAN_REFERENCE_BASENAME=defender-clean-firing
   ```

4. Compare candidate and reference with `tools/verify_reference_media.py`,
   using separate reference/candidate start offsets when event timing differs.

5. Treat failures as bounded graphics/audio/playability tasks. Fix the clean
   implementation, not the reference.

## Finite Milestones

The milestones below are the finite route used for the rewrite. Current local
evidence says the implementation, harness, graphics, audio, playability, and
release gates have been executed; `M6` remains open only for owner approval of
the bounded proof set, or for a new concrete MAME mismatch that expands the
work.

### M1: Golden Artifact Inventory

Goal: isolate what must be matched before more fidelity work continues.

Deliverables:

- Produce MAME golden clips for title/attract, scoring laser/explosion,
  one-player start, gameplay laser, reverse, thrust, smart bomb, hyperspace,
  player death, enemy explosions, human rescue/loss, terrain blow, and
  game-over/Hall-of-Fame return.
- Record timestamp ranges, expected sound presence, representative screenshots,
  and audio signatures for each clip.
- Build a compact checklist of sprites, laser states, explosion states, and
  sound families that must be matched.

Exit gate:

- `make reference-mame-capture` succeeds.
- Golden clip inventory exists in ignored generated artifacts plus tracked
  documentation or manifests that do not include copyrighted media.

### M2: Clean Candidate Capture Harness

Goal: compare like-for-like media instead of comparing unrelated attract clips
to gameplay videos.

Deliverables:

- Add a deterministic clean candidate capture path that can run the same input
  program and duration as a MAME golden capture.
- Export clean video/GIF and WAV for the selected time window.
- Make `reference-media-check` consume matching reference/candidate pairs.

Exit gate:

- A short MAME capture and a short clean capture of the same sequence can be
  compared by `tools/verify_reference_media.py`.
- Expected mismatches are reported as bounded visual/audio metrics, not as
  missing media or timeline drift.

### M3: Graphics Fidelity

Goal: make the clean renderer match MAME-visible graphics.

Deliverables:

- Align player laser origin, direction, span, fizzle/body/tip bytes, and hit
  endpoint with MAME footage.
- Align attract scoring laser and explosion placement with the scoring-page
  MAME clip.
- Replace incorrect sprites, orientations, palettes, scanner elements, HUD
  glyphs, terrain, starfield, and explosion frames.
- Ensure reverse visibly flips the player ship and does not regress controls.

Exit gate:

- `reference-media-check` visual metrics pass for the selected golden clips, or
  each remaining visual failure has a concrete tracked reason and screenshot.

### M4: Audio Fidelity

Goal: make clean audio match the original sound-board behavior.

Deliverables:

- Extract or document MAME sound-command timing for the golden clips.
- Match credit/start, thrust, laser, smart bomb, hyperspace, enemy shots,
  explosions, lander pickup/pull, human rescue/loss, player death, and terrain
  blow sounds.
- Preserve verified attract silence where the golden clip is silent.
- Keep `--mute` and deterministic no-device tests working.

Exit gate:

- `reference-media-check` audio metrics pass for selected gameplay clips, or
  each remaining audio failure is tied to a specific missing sound family.

### M5: Playability And Mechanics

Goal: make the clean game behave like Defender during actual play.

Deliverables:

- Verify coin/start, thrust, reverse, fire, smart bomb, hyperspace, enemy waves,
  collision/hit timing, explosions, human rescue/loss, scoring, death/respawn,
  game-over/Hall-of-Fame return, and high-score table display.
- Keep clean-fidelity scenario coverage green while media fidelity improves.
- Fix gameplay behavior only when MAME/source evidence shows a mismatch.

Exit gate:

- `make clean-fidelity` passes.
- `cargo run -- --game-smoke` and `cargo run -- --live-smoke` pass.
- Manual play pass covers the listed mechanics without visible/audio regressions.

### M6: Release Closure

Goal: produce a release-ready clean implementation with finite evidence.

Deliverables:

- Full release gate passes.
- MAME golden comparison reports are generated for the selected acceptance
  clips.
- README and SPEC describe the final runtime, verification workflow, controls,
  assets, and known limitations.
- Owner signs off on graphics, audio, and playability fidelity.

Exit gate:

- No active release blockers remain.
- Production runtime does not depend on legacy machine/raster presenter code.
- Owner approval is recorded before replacing protected reference media.

## Active Work Items

1. Complete the golden artifact inventory for remaining gameplay clips,
   including exact MAME timestamp ranges for lander pickup/pull, rescue/loss,
   terrain blow, and still-uncovered non-lander shot/explosion/materialization
   states. Existing local traces now cover delayed-start thrust, first lander
   shot, first lander hit, reverse/player orientation, smart bomb, player
   death, long no-input final death, game-over/Hall-of-Fame return, and
   residual post-game playfield windows. Trace-only state-steered MAME artifacts
   now cover actual falling-human routine evidence under
   `target/reference-media/mame/state-steered/`: `afall-fall` keeps `AFALL`
   live from frames `1450-1505` then transitions to the safe-landing score
   popup with `0xE0` at frame `1507`; `afall-safe-landing` emits `0xE0` at
   frame `1451`; `afall-player-catch` switches from `AFALL` to `AFALL2` at
   frame `1451` and emits the source catch command `0xF7` at frames `1451`,
   `1461`, and `1471`; `terrain-blow` starts `TERBLO` at frame `1450`, then
   runs the terrain-blow process and sound command tail through `0xEE`, `0xEB`,
   and `0xE8`. Bounded MAME video/audio clips and contact sheets for
   `afall-player-catch`, `afall-safe-landing`, and `terrain-blow` now exist under
   `target/reference-media/mame/state-steered-media/`. Matching clean
   candidates and contact sheets now exist under
   `target/reference-media/clean/state-steered-media/`; the terrain-blow report
   at `target/reference-media/terrain-blow-check/report.json` now passes after
   aligning the clean steer wake frame, MAME MP4 PTS start, flash windows, and
   terrain explosion growth cadence. The falling-human catch report at
   `target/reference-media/afall-player-catch-check/report.json` now passes
   after preserving pre-window sound-board tails in bounded clean WAV output and
   calibrating the `0xF7` catch vector against the state-steered MAME clip.
   The falling-human safe-landing report at
   `target/reference-media/afall-safe-landing-check/report.json` now passes
   after steering the clean falling human through the normal safe-landing
   transition and calibrating the source VARI `0xE0` voice.
   The `2026-05-29` proof-boundary audit originally found no valid organic
   last-human terrain-blow reference, but the latest all-trace and organic-only
   reference-window scans now find a concrete organic smart-bomb/up-thrust
   candidate. The all-trace scan covers `218` expected traces and `214` debug
   traces, with `152024` terrain status rows, `4` `TERBLO` process rows, and
   `2` last-human terrain-blow candidates. The organic-only scan covers `198`
   expected traces and `194` debug traces, with `144341` terrain status rows,
   `2` `TERBLO` process rows, and the same `2` candidates at frame `5990`,
   `ASTCNT=0x00`, `pc=0xED88`, `terrain_blown=false`, and a live `TERBLO`
   process. The candidate report at
   `target/reference-media/organic-terrain-blow-smartmix-check/report.json`
   now passes all-axis comparison after the target4 terminal branch overlays
   only the MAME-visible terrain-flash windows from the smartmix terminal
   death clip while keeping flash suppressed across the intervening black
   phases. The focused clean guard now matches the terminal-death state rows
   through score `50`, terrain-blown status at frame `4927`, attract handoff
   at frame `4947`, no clean human snapshots once terrain blow is active,
   delayed visible `TEREX` progression at the organic `TERBLO` sound boundary,
   the MAME-observed six-lander / nine-mutant residual object mix before
   `TERBLO`, sampled `0xEE` terrain-blow tail rows at frames `5991`, `5995`,
   `5999`, `6003`, and `6007`, and sampled terminal flash colors at frames
   `5990`, `6002`, `6014`, `6032`, `6044`, and `6074`. The regenerated media
   report passes with visual RMS `23.11` / MAE `3.42`, visual-region RMS
   `29.01` HUD, `32.07` scanner, `15.30` playfield, `19.57` laser band, and
   `28.36` terrain, plus audio envelope correlation `0.4431`. Earlier
   `2026-06-06` direct `pcram0` replay and sample-step experiments were not
   kept because they regressed the comparator; the accepted fix is bounded to
   the terminal branch and leaves the generic source terrain-blow flash path
   intact for other windows. The state-steered `TERBLO` clip remains accepted
   bounded terrain-blow evidence, and the organic smartmix clip is now the
   first passing organic terrain-blow all-axis proof. A follow-up
   non-lander trace inventory identified `extended_hold_up_7000`, frames
   `5811-7000`, as the
   next finite organic visual media target. Its input program is
   `none*900;coin*4;none*360;start_one*10;altitude_up*5726`, and the MAME TSV
   window contains post-game/attract `UFOP1`, `TIEP1`, `PRBP1`, `SWPIC1`,
   `SCZP1`, `BMBP1`, and `BXPIC` picture rows. A `2026-05-29` MAME/clean media
   trial generated this long-window clip and comparison report. Initial
   inspection found clean stuck in a static `GameOver` screen, but the
   follow-up fix now resumes the normal attract scoring sequence after the
   post-game residual and the regenerated media report passes. A later
   per-family object-span scan exposed a PRBP1 pod span at frames `6855-10000`
   in `organic_fire_up_thrust_10000`; the bounded all-axis media report for
   frames `6855-7455` now passes and is accepted as additional organic
   non-lander visual breadth plus MAME-silent post-game audio proof.
2. Re-run MAME-vs-clean comparison on the remaining timestamp-aligned clips.
   No bounded sprite/audio media target remains open from the current list. The
   organic non-lander long-window comparison now passes after the clean
   post-game attract handoff fix, and the organic PRBP1 pod up-thrust all-axis
   comparison passes with full visual RMS `37.95`, full MAE `7.68`, and
   matching MAME-silent audio. The
   bounded pickup/pull `hold-up` media comparison now passes
   with full visual RMS `28.61`, visual MAE `4.99`, playfield RMS `4.86`,
   laser-band RMS `3.25`, terrain RMS `20.18`, audio envelope correlation
   `0.613`, RMS ratio `1.066`, and zero-crossing ratio `1.076`. The bounded
   `down029/fire2524` non-lander
   target6 shot/explosion/materialization media comparison now passes with
   full visual RMS `31.72`, visual MAE `5.74`, playfield RMS `17.46`,
   laser-band RMS `19.64`, terrain RMS `26.94`, audio envelope correlation
   `0.714`, RMS ratio `1.192`, and zero-crossing ratio `1.298`. The
   state-steered terrain-blow
   comparison now has a passing bounded report-only artifact: sound-command
   offsets match the MAME trace, source `TERX0` terrain explosion pixels
   render, flash windows and explosion growth follow the MAME capture, visual
   RMS is `31.19`, visual MAE is `6.32`, and audio passes the stochastic-noise
   gate. The state-steered catch comparison also passes with visual RMS
   `29.32`, visual MAE `5.23`, audio envelope correlation `0.935`, RMS ratio
   `1.008`, and zero-crossing ratio `0.463`. The state-steered safe-landing
   comparison passes with visual RMS `29.30`, visual MAE `5.24`, audio
   envelope correlation `0.284`, RMS ratio `1.003`, and zero-crossing ratio
   `1.396`. Player catch/rescue is covered by the state-steered catch media
   gate, safe landing by the state-steered safe-landing media gate, and
   lander-driven human loss/conversion by the organic `hold-up` media gate.
3. Tighten remaining gameplay laser/audio edge cases from live laser-hit
   evidence. The current first-lander hit clips are green, including the
   `down029/fire2437` MAME window with laser sound at frame `2439`, lander-hit
   sound at frame `2450`, score `150` at frame `2449`, and no premature player
   death. The `down029/fire2524` target6 non-lander endpoint is also green in
   media comparison. The `2026-05-29` laser/reverse proof-boundary audit
   rechecked source-style sparse laser tests, `LASP1` object evidence, far-side
   collision before culling, capped-fire sound suppression, both reverse
   directions, and the accepted fire/reverse, laser-hit, and target6 media
   reports. A follow-up `2026-05-29` media-breadth cycle added the passing
   centered first-laser-hit report to the closure gate, giving the first
   gameplay hit both broad-window and centered-window proof. A later
   `2026-05-29` reverse-thrust media cycle added the passing delayed-start
   thrust/reverse report to the closure gate after calibrating the clean
   thrust-noise zero-crossing range and the `0xF0` background-noise gain. The
   `down030/fire2437` post-death branch now has all-axis media evidence for the
   second-life laser/materialize window: MAME and clean both emit the
   second-life laser `0xEB` at frame `2439` and the post-death appearance
   command `0xEA` at frame `2447`, and
   `target/reference-media/laser-hit-down030-fire2437-check-fixed/report.json`
   passes visual and stochastic audio thresholds after the source-shaped
   `APPEAR` materialize waveform fix.
4. Resolve remaining first-wave lander shell cadence and aim mismatches from
   MAME object/shell evidence without regressing the green
   `down029/fire2437` laser-hit clip or the long no-input acceptance guards.
   The `down060/fire2437` MAME trace-backed shell collision now matches clean
   at frame `2177`: score `25`, `BXPIC` bomb explosion at source center
   `0x23EA`, death-tail commands at `2178`, `2179`, `2187`, `2195`, and
   `2324`, and the first stock drop at `2439`. The lower
   `down029/fire2437` laser-hit clip still survives until the targeted lander
   kill. The neighboring `down030/fire2437` target-2 lander collision now also
   matches MAME at frame `2177` with score `150`, `LNDP2` explosion center
   `0x24B4`, death-tail commands at `2178`, `2179`, `2187`, `2195`, and
   `2324`, and first stock drop at `2344`. The long no-input run now matches
   MAME through first collision, second
   death, game-over handoff, residual post-game materialize/shot/explosion
   sounds, and residual `175 -> 200` scoring. The `hold-up` run now matches
   the first five MAME lander shot state frames `2074`, `2110`, `2266`,
   `2458`, and `2686`; its final enemy-collision residual window also matches
   MAME by input frame for `3605`, `3668`, `3701`, `3709`, `3710`, `3711`,
   `3719`, `3727`, and the duplicated `0xEC`/`game_over` transition at
   `3854`. The carried-human pull/conversion rows now match MAME by input
   frame: first pull `2524-2533`, first conversion `2536`, second pull
   `2693-2702`, and second conversion `2705`. MAME shows no visible mutant or
   expanded-object coalescence at the conversion rows `2535` and `2704`; the
   later target6 converted-mutant wrap rows now match MAME in the clean scene:
   `2800=0,54`, `2805=3,55`, `2810=7,55`, `2820=15,48`, and `2823=17,46`.
   The same `hold-up` converted-mutant entry now emits the paired MAME shot
   sounds at input frames `2824` and `2838`, matching the source-derived MAME
   `0xF6` window after the existing one-frame clean/MAME input alignment.
   The `hold-up` post-death restart materialization sound window now has only
   the MAME-backed `0xEA` at `3108`; the earlier clean-only refill `0xEA` at
   `3176` is removed. The extended `down029/fire2437` post-hit refill
   materialization now matches MAME's visible object-output shape: only the
   target-3 refill lane coalesces at `2752`, only that refill lane is projected
   at `2800` and `2902`, and hidden/stopped refill processes remain live
   without leaking sprites.
   The `up-thrust` abduction-search target5 opening shell now follows the MAME
   shell-table motion from the first `0xFC` shot window, reaches the player on
   input frame `1680`, scores `25`, removes the shell, and anchors the `BXPIC`
   bomb explosion at top-left `0x372C`. The delayed player-death sound tail now
   matches MAME at `1681`, `1682`, `1690`, and `1698`. The same branch now
   uses the MAME-backed target5 projectile-death restart at input frame `1949`:
   appearance sound `0xEA`, player `PLAX16/PLAY16=0x3280/0x2A80`, player
   velocity `0x009758/0xFE00`, absolute x `0x30E1`, RNG `0xC4/0x94/0xDD`,
   and the observed human/lander object snapshot. Clean no longer takes the
   false enemy collision at input frame `2007`. The later target3 lander shot
   in the same restart branch now emits the MAME `0xFC` command and shell at
   input frame `2195`, with shell position/fractions/velocity
   `0x4696`, `0x20/0xC1`, and `0xFF88/0xFE78`.
   The `down029/fire2524` target9 shell now follows the MAME shell-table motion
   for positions `0x51AD`, `0x50AD`, `0x229B`, `0x1E99`, and `0x1D99`, and no
   longer causes a premature player/projectile death before the target9 lander
   hit. The same `down029/fire2524` first hit now matches MAME sound timing for
   `2524-2531`: pull `0xF1` at `2524-2530`, target9 hit `0xF9` at `2531`, and
   no delayed laser-fire `0xEB` or extra pull leak in that interval. The later
   `down029/fire2524` target6 converted-mutant branch now also matches the
   MAME trace for mutant shot sounds at `2872` and `2959`, player/enemy
   collision tail commands at `3012`, `3013`, `3021`, `3029`, and `3158`, and
   the `SCZP1` explosion descriptor at `3012` with top-left `0x20A2` and
   center `0x21A9`. The same target6 window now has a passing bounded
   MAME-vs-clean media report. The release gate is now green; the remaining
   closure item is owner review, plus any new concrete gameplay case found
   outside the current down029/hold-up/state-steered terrain-blow evidence.
5. Isolate and match remaining gameplay sounds from MAME/sound-board evidence.
   The `down029/fire2437` post-hit tail now matches MAME in the accepted
   window: pull `0xF1` at `2700-2702`, conversion `0xEE` at `2705`,
   materialization `0xEA` at `2752`, and a single mutant shot `0xF6` at
   `2827`, with no extra pickup or duplicate mutant shot through `2900`. The
   `hold-up` converted-mutant shot pair is also covered at `2824` and `2838`.
   The `hold-up` post-death materialization window is covered at `3108` with
   no extra refill materialization sound through `3338`. The extended
   `down029/fire2437` post-hit window no longer emits the false target-3 refill
   lander shot at `2956`; target6 converted-mutant shots now match MAME at
   `2827`, `2902`, and `2947`, with death-tail commands at `2994`, `2995`,
   `3003`, and `3011`. The same extended window now has exact regression
   coverage for all observed sound commands from `2439` through `3011`.
   The `up-thrust` target5 opening shell/death window is also covered for the
   observed `0xFC`, `0xEE`, and `0xE8` commands at `1623`, `1681`, `1682`,
   `1690`, and `1698`, with the post-death materialization command `0xEA` now
   covered at `1949`. The `up-thrust` target5 restart sound cadence now also
   matches the MAME rows for thrust command `0xE9` at `1457`, `1631`, `1852`,
   `1997`, and `2203`, plus the later lander shot `0xFC` at `2195`.
   Catch, safe-landing, and terrain-blow command families now have
   state-steered MAME trace evidence and clean command-sequence coverage:
   `afall-player-catch` emits `0xF7` at frames `1451`, `1461`, and `1471`,
   and clean rescue now queues the same repeated `ACSND` cadence; safe landing
   emits `0xE0` at frame `1451` or after a full `AFALL` descent at `1507`;
   terrain blow starts `TERBLO` at frame `1450` and clean completion now queues
   the `TBSND` tail `0xEB`, `0xEE`, `0xEE`, `0xE8`, `0xE8` at the MAME
   offsets. The generated clean safe-landing WAV now passes after the clean
   state steer lands through the normal score/sound path and the clean
   sound-board VARI path applies source restart cadence plus calibrated VARI
   DAC gain. The generated clean terrain-blow WAV now passes the media
   stochastic-noise gate against the MAME state-steered WAV. No active bounded
   sound target remains open; future waveform tuning belongs to owner-review or
   new-evidence failures.
   The `down029/fire2524` laser/fire contention frame now matches the MAME
   command priority by suppressing delayed laser-fire `0xEB` when lander-pull
   `0xF1` is emitted on the same frame. The later `fire2524` converted-mutant
   sound delta is closed: the clean trace now emits MAME's target6 shot
   commands `0xF6` at `2872` and `2959`, then the player/enemy collision tail
   `0xE8/0xEE/0xEE/0xE8/0xEC` at `3012/3013/3021/3029/3158`. The matching
   target6 media report now passes against MAME audio with envelope correlation
   `0.714`, RMS ratio `1.192`, and zero-crossing ratio `1.298`.
   A `2026-05-29` non-lander sound-command audit added direct regression
   coverage for the red-label enemy-family command bytes: lander hit/shot
   `0xF9`/`0xFC`, mutant hit/shot `0xE8`/`0xF6`, bomber hit `0xFE`, pod hit
   `0xFA`, swarmer hit/shot `0xF8`/`0xF3`, baiter hit/shot `0xF8`/`0xFC`, and
   bomb collision `0xEE`. Bomber and pod remain intentionally direct-shot
   silent because the source table has no bomber or pod shot command.
   The isolated non-lander sound-command media proof now covers the remaining
   non-organic command breadth. MAME captures for `sound_command_fe`,
   `sound_command_fa`, `sound_command_f8`, and `sound_command_f3` include
   command traces plus sound-board DAC-write traces. Matching clean candidates
   now pass the audio gates for bomber hit `0xFE`, pod hit `0xFA`,
   swarmer/baiter hit `0xF8`, and swarmer shot `0xF3`. The `0xFE` and `0xFA`
   fixes were scoped to tonal GWAVE period density after the DAC trace proved
   the earlier global DAC-hold mixer patch was wrong. These single-command
   clips intentionally do not accept visual metrics because the synthetic MAME
   and clean playfields are not matching gameplay scenes.
6. Replace remaining incorrect sprite, explosion, and pixel-coalescence frames.
   Refill coalescence is now covered for the extended `down029/fire2437`
   post-hit window. Target6 converted-mutant vertical cadence, shot origin,
   collision timing, and `SCZP1` explosion placement are now covered in the
   same input window: the projected mutant reaches MAME rows `0x1446`,
   `0x1F5B`, `0x1F71`, and `0x2087`, then destroys at `2993` with explosion
   top-left `0x20A3`, center `0x21A9`, and exact `SCZP1` explosion growth
   coverage for every MAME descriptor frame from `2993` through `3020`. The
   `down029/fire2524` converted-mutant branch now also covers target6 shot
   launch sprites at `2872` and `2959`, the MAME collision/death-tail at
   `3012`, `3013`, `3021`, `3029`, and `3158`, and the `SCZP1` explosion
   descriptor at `3012` with top-left `0x20A2` and center `0x21A9`. Its
   bounded media report now passes with playfield RMS `17.46` and laser-band
   RMS `19.64`.
   The `up-thrust` target5 opening shell collision now covers the `BXPIC` bomb
   explosion at input frame `1680` with top-left `0x372C`, matching the MAME
   player/shell collision window. The post-death restart now also displays the
   MAME-backed player top-clamp row at `1949` and carries the MAME human/lander
   object snapshot forward without the previous false player/enemy explosion at
   `2007`.
   State-steered terrain blow now renders source `TERX0` terrain explosion
   pixels, uses the MAME-observed flash windows, and follows the MAME
   terrain-explosion growth cadence, replacing the empty `TEREX` image lookup
   and guessed green/yellow/orange flash ramp. The bounded terrain-blow media
   report now passes; remaining visual targets are gameplay cases outside the
   current down029/hold-up/state-steered windows.
   A new state-steered enemy explosion matrix can now seed MAME and clean with
   the same six source expanded-object slots for `LNDP3`, `SCZP1`, `TIEP3`,
   `PRBP1`, `UFOP3`, and `SWXP1`. The first MAME-vs-clean matrix report at
   `target/reference-media/enemy-explosion-matrix-check/report.json` has
   visual status `pass` with full RMS `31.38`, playfield RMS `15.82`, and
   laser-band RMS `15.95`. The report is top-level `pass` with
   `acceptance_mode=visual` because this synthetic visual steer does not
   exercise real kill sound commands, so audio is not an acceptance target for
   that clip.
   A `2026-05-29` non-lander implementation audit found no current placeholder
   path for mutant, swarmer, baiter, bomber, pod, or bomb presentation:
   source-shaped sprite IDs, atlas regions, source picture descriptors,
   runtime movement, projectile rows, hit/shot commands, and source explosion
   descriptors are covered by focused Rust tests. A follow-up enemy-family
   explosion hardening pass now explicitly locks lander, mutant, bomber, pod,
   baiter, and swarmer explosions to source picture descriptors and expanded
   source pixel clouds instead of static placeholder sprites. The remaining
   non-lander family item is bounded MAME-vs-clean media proof breadth, not a
   known placeholder implementation. The next visual media candidate is the
   organic `extended_hold_up_7000` MAME window at frames `5811-7000`, which
   contains post-game/attract baiter, bomber, pod, swarmer, mutant, bomb-shell,
   and bomb-explosion picture rows in one trace. A `2026-05-29` media trial
   exposed and then closed the clean post-game attract handoff mismatch; the
   regenerated report now passes with non-static clean scoring-sequence frames.
   A second organic hold-down media trial now covers MAME frames `4300-4700`
   from input `none*900;coin*4;none*360;start_one*10;altitude_down*5726`,
   including converted-mutant, baiter, and swarmer-explosion rows. The report
   at `target/reference-media/organic-nonlander-holddown-7000-check/report.json`
   passes with `acceptance_mode=visual`, full visual RMS `28.22`, playfield
   RMS `7.59`, laser-band RMS `5.41`, and terrain RMS `21.39`; audio remains
   diagnostic-only for that clip because it does not exercise the remaining
   non-lander-specific sound command bytes.
   A third bounded organic media trial now covers PRBP1 pod presentation from
   MAME frames `6855-7455` of the
   `organic_fire_up_thrust_10000` trace, using input
   `none*900;coin*4;none*360;start_one*10;up,thrust*400;up,thrust,fire*40;up,thrust*8286`.
   The report at
   `target/reference-media/organic-nonlander-prbp1-upthrust-check/report.json`
   passes with `acceptance_mode=all`, full visual RMS `37.95`, full MAE
   `7.68`, and matching MAME-silent audio after suppressing clean-only
   post-game thrust leakage in that window.
   A follow-up materialization/coalescence regression now exercises clean
   source expanded-object appearance projection for lander, mutant, bomber,
   pod, baiter, and swarmer families. This closes the remaining known
   implementation-test gap for enemy-family appearance as source pixel clouds;
   the remaining coalescence boundary is additional bounded MAME media breadth
   and owner review, not a known static-sprite implementation path.
7. Run release-gate validation and owner review. The default and
   `legacy-tools` all-targets Rust test gates, both clippy gates, and
   `make clean-fidelity` were green in the `2026-05-29 15:54 BST`
   release-gate validation cycle after promoting the PRBP1 pod up-thrust proof
   to all-axis and fixing the clean-only post-game thrust/background audio
   leak. The later organic smart-bomb/up-thrust terrain-blow candidate now has
   a passing all-axis report and is accepted as organic last-human terrain-blow
   proof in the report gate. The latest actor-era `2026-06-07 17:50 BST`
   full `make release-gate` pass is green, including default and
   `legacy-tools` Rust tests, both clippy gates, `make clean-fidelity`, media
   helper tests, owner-review package/report gate, MAME doctor/smoke, README
   media, game smoke, actor play smoke, actor attract/post-game smoke, actor
   offscreen `wgpu` smoke with `frame_source: actor_game`, live smoke, docs
   lint, and diff hygiene. Owner review remains before protected reference
   media replacement.
   The owner-review checklist in `docs/fidelity/release-closure-audit.md`
   defines the finite sign-off action and is printed by
   `make owner-review-package`: accept the current 28-report gate and proof
   boundaries, or provide a new concrete MAME mismatch/input program.

## Current Work Log

- `2026-06-07 20:23 BST`: Completed the actor player HUD-bound clamp cycle.
  The player bounds now use the source playfield top (`SOURCE_PLAYFIELD_Y_MIN`)
  instead of allowing the ship to move up into the HUD/top-display band. Added
  a focused `PlayerShip` regression proving repeated altitude-up input clamps
  the player draw/snapshot Y position to the bottom of the HUD separator.
  Validation passed with the new focused clamp regression, the existing thrust
  scroll regression, `cargo fmt --check`, `cargo check --all-targets
  --features legacy-tools`, `cargo clippy --all-targets --features
  legacy-tools -- -D warnings`, `cargo run -- --actor-smoke`, `cargo run --
  --live-smoke`, `make docs-lint`, and `make diff-check`. Remaining plan work
  is still about `2%`, mostly owner-review/protected-media closure after
  concrete live defects. Slack step start:
  <https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780860071189189>.

- `2026-06-07 20:12 BST`: Completed the source-backed human world-space
  follow-up cycle. The prior projection fix already projected `source_human`
  draw/collision bodies, and this cycle tightened the missing gameplay path:
  falling-human rescue now compares the player against the projected
  source-backed human body instead of the raw source high byte. Added focused
  regressions proving clean state keeps source human high-byte/fraction world
  evidence stable across background scroll, the render scene projects the human
  through `source_background_left`, projected human collision bodies match the
  rendered position, and falling rescue works when the player intersects the
  projected body. Validation passed with the new focused human render/rescue
  regressions, the existing source-object projection regression, `cargo fmt
  --check`, `cargo check --all-targets --features legacy-tools`, `cargo clippy
  --all-targets --features legacy-tools -- -D warnings`, `cargo run --
  --actor-smoke`, `cargo run -- --live-smoke`, `make docs-lint`, and `make
  diff-check`. Remaining plan work is still about `2%`, mostly
  owner-review/protected-media closure after concrete live defects. Slack step
  start:
  <https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780859214063609>.

- `2026-06-07 19:59 BST`: Completed the actor source-world projection cycle
  for the reported thrust bug where aliens, enemy bullets, and bombs appeared
  to travel with the player. Source-backed hostile, human, enemy-shot, and bomb
  actor snapshots still own source world positions/fractions, but
  `ActorRenderSceneBridge` now projects their draw positions through the
  current `source_background_left` before submitting scene sprites. The actor
  collision resolver now builds the same projected screen-space bodies for
  source-backed hostile/projectile snapshots, so player lasers and player
  damage use the visual position rather than the raw source high byte. Added
  focused regressions proving a thrust-scrolled background shifts projected
  lander/fireball/bomb sprites while the player stays centered, and that
  source-backed hostile collision bodies use the same projection/offscreen
  culling. Validation passed with the new render/collision regressions, the
  existing actor render-bridge projectile/explosion regression, `cargo fmt
  --check`, `cargo check --all-targets --features legacy-tools`, `cargo clippy
  --all-targets --features legacy-tools -- -D warnings`, `cargo run --
  --actor-smoke`, `cargo run -- --live-smoke`, `make docs-lint`, and `make
  diff-check`. Remaining plan work is still about `2%`, mostly
  owner-review/protected-media closure after concrete live defects. Slack step
  start:
  <https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780858040302769>.

- `2026-06-07 19:39 BST`: Completed the actor wave-progression and
  landscape-color fidelity cycle. Verified the expected wraparound/wave
  behavior and eight-wave separator color order against web/source references,
  including Doug Mahugh's Defender notes, Arcade History's wave-rollover notes,
  Giant Bomb's wraparound/HUD gameplay summary, and the local
  `assets/red-label/wave-table.tsv`/source-backed actor wave scripts. The
  actor driver now distinguishes source-counted reserve bookkeeping from
  visible, interactive hostiles that block wave clear, so hidden first-wave
  refill lanes no longer leak into clean scanner/object state or hold the game
  after every visible alien is destroyed. The survivor-bonus path now clears
  those hidden noninteractive lanes and advances to the next wave after the
  source `0x80` sleep. Gameplay `BGOUT` terrain and non-marker top-display
  border segments now tint from the documented eight-wave cycle
  blue/green/red/orange/yellow/purple/brown/black, with wave 9 wrapping to
  wave 1. Validation passed for the hidden-refill wave-advance regression,
  actor refill suppression, actor gameplay renderer, clean terrain projection,
  clean top-display border, explicit wave-color cycle, `cargo fmt --check`,
  `cargo check --all-targets --features legacy-tools`, `cargo clippy
  --all-targets --features legacy-tools -- -D warnings`, `cargo run --
  --actor-smoke`, `cargo run -- --live-smoke`, `make docs-lint`, and `make
  diff-check`.
  Remaining plan work is still about `2%`, mostly owner-review/protected-media
  closure after concrete live defects. Slack step start:
  <https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780856917370399>.

- `2026-06-07 19:24 BST`: Completed the actor gameplay HUD/side-scrolling
  regression cycle. Actor gameplay scenes now populate clean object evidence
  and scanner state through a crate-visible `WorldSnapshot` presentation sync,
  then render the source top-display border plus scanner terrain and player
  radar blip through the existing clean scanner renderer. Actor player thrust
  now moves the ship toward the center band and then emits wrapped
  `SetSourceBackgroundLeft` commands so the `BGOUT` terrain scrolls beneath the
  player in both directions instead of staying fixed on one screen. Added
  focused regressions for gameplay scanner/HUD visibility and wrapped
  left/right background scrolling. Validation passed with focused actor HUD,
  player scroll, and reverse regressions, `cargo run -- --actor-smoke`, `cargo
  run -- --live-smoke`, `cargo fmt --check`, `cargo check --all-targets
  --features legacy-tools`, `cargo clippy --all-targets --features legacy-tools
  -- -D warnings`, `make docs-lint`, and `make diff-check`. Remaining plan
  work is still about `2%`, mostly owner-review/protected-media closure after
  these concrete live defects. Slack step start:
  <https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780855892937019>.

- `2026-06-07 19:09 BST`: Completed the actor/live reverse-input regression
  cycle. Planetoid controls now bind `SHIFT` to reverse and `SPACE` to thrust
  across the actor keyboard mapper, live `wgpu` physical/logical keymap, and
  README controls. Live input drains reverse as a press pulse, while
  `PlayerShip` also guards against repeated direct reverse input so a held
  reverse request flips direction once and stays facing that way until release
  and repress. Added focused actor runtime, actor mapper, live input-state, and
  live keymap regressions. Validation passed with `cargo fmt --check`, focused
  reverse/keymap/input tests, `cargo check --all-targets --features
  legacy-tools`, `cargo clippy --all-targets --features legacy-tools -- -D
  warnings`, `cargo run -- --live-smoke`, `make docs-lint`, and `make
  diff-check`. Remaining plan work is still about `2%`, concentrated on
  owner-review/protected-media closure rather than this input defect. Slack
  step start:
  <https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780855086832799>.

- `2026-06-07 18:38 BST`: Completed the actor live-start and final game-over
  regression cycle. Interactive live actor construction now enables explicit
  free-play admission, so a fresh live run accepts `1` for one player and `2`
  for two players without requiring a manual `5` coin insertion. The default
  actor driver, actor smoke, and script-check evidence paths remain
  credit-gated for red-label verification. Low-score final deaths now publish a
  finite `player_death_sleep_remaining` `GAME OVER` interstitial through
  `StepReport` / `GameOverSnapshot`, render the source `GO` prompt at
  `0x3E80`, suppress attract-script drawing during the countdown, and return to
  the Williams attract reveal only after the countdown completes. The Planetoid
  actor keyboard mapper now also maps `2` to the two-player start action,
  matching the live `wgpu` key contract. Validation passed with `cargo fmt
  --check`, focused actor start/final-death/key-mapper regressions, the focused
  live-runtime no-coin start regression, `cargo check --all-targets --features
  legacy-tools`, `cargo clippy --all-targets --features legacy-tools -- -D
  warnings`, `cargo run -- --live-smoke`, `cargo run --
  --actor-post-game-smoke`, `cargo run -- --actor-smoke`, `make docs-lint`,
  and `make diff-check`. Slack step start:
  <https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780852879218859>.

- `2026-06-07 18:13 BST`: Completed the actor live-smoke and gameplay
  regression cycle. `--live-smoke` now uses the actor smoke/offscreen `wgpu`
  path and reports `frame_source: actor_game` with `192` actor frames,
  `0` clean-game frames, `192` nonblank offscreen frames, and zero temporary
  raster commands. Retired the clean game smoke helper surface that only
  existed for the old live-smoke route, while keeping explicit `--game-smoke`
  available as a separate clean-frame evidence command. Added focused actor
  regressions proving same-step credit plus `1` starts one-player play,
  second-credit plus `2` starts two-player play, and a one-player non-final
  death stays in `Playing`, does not render attract, and respawns without
  emitting game-over. README, SPEC, the release audit, and the active plan now
  describe actor-backed live smoke and the correct `1`/`2` start controls.
  Validation passed with `cargo fmt --check`, focused Rust regressions for the
  start/death/live-smoke/CLI error paths, `cargo run -- --game-smoke`,
  `cargo run -- --live-smoke`, `cargo run -- --actor-post-game-smoke`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  docs markdownlint, and `git diff --check`. Slack step start:
  <https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780851810731939>.

- `2026-06-07 17:50 BST`: Completed the post-cleanup full release-gate
  verification cycle after removing the retired legacy runtime adapters.
  `make release-gate` passed end to end: formatting, default and `legacy-tools`
  Rust tests, both clippy passes, `make clean-fidelity`, media helper tests,
  owner-review package/report gate, MAME doctor/smoke, README media, game
  smoke, actor play smoke, actor attract/post-game smoke, actor offscreen
  `wgpu` smoke with `frame_source: actor_game`, live smoke, docs lint, and diff
  hygiene. The owner-review package again reported 28 accepted reports across
  20 coverage requirements, and the short MAME recorder smoke regenerated
  ignored MP4/WAV artifacts successfully. Slack start:
  <https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780850391414769>.

- `2026-06-07 17:30 BST`: Completed the bounded legacy-retirement audit cycle.
  Removed retired historical runtime adapter files from `src_legacy/`:
  `app.rs`, `audio.rs`, `cmos_storage.rs`, `kitty.rs`, `lib.rs`, `live.rs`,
  `main.rs`, `presentation.rs`, `terminal.rs`, and `wgpu_presenter.rs`.
  Strengthened public API/source guards so those files must stay absent while
  the still-needed `legacy-tools` oracle modules remain feature-gated and
  crate-private. README and SPEC now state that these adapters were removed
  rather than kept parked. No active oracle, ROM, trace, sound, video, or
  accepted-behavior modules were safe to remove because the release gate and
  owner-review evidence still depend on them. Slack start:
  <https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780849627419489>.
  Validation passed with `cargo check --all-targets`, `cargo check
  --all-targets --features legacy-tools`, `cargo test public_api_tests --lib
  --features legacy-tools`, `cargo fmt --check`, `make docs-lint`, and
  `git diff --check`.

- `2026-06-07 11:51 BST`: Completed the actor script-check hostile projectile
  matrix cycle. `--actor-script-check <path>` now builds bounded
  parser-backed custom-driver probes for lander, mutant, swarmer, and baiter
  shot families and reports the first projectile spawn sample, velocity, source
  metadata when available, sample step, and red-label shot command evidence.
  The checked matrix pins `0xFC` lander shot, `0xF6` mutant shot, `0xF3`
  swarmer shot, and `0xFC` baiter shot evidence. Bomber and pod direct-shot
  rows are intentionally excluded because the current source-backed actor table
  exposes bombs/pod-hit spawning but no direct shot command family for those
  actors. No legacy code, tests, or scaffolding were safe to remove because
  this slice expands the active custom-driver preflight evidence surface for
  hostile projectile/audio fidelity. Slack start:
  <https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780820837740689>.
  Validation passed with `cargo fmt --check`, `cargo test actor_script_check
  --all-targets --features legacy-tools`, `cargo test actor_script
  --all-targets --features legacy-tools`, `cargo test actor_live --all-targets
  --features legacy-tools`, `cargo check --all-targets --features
  legacy-tools`, `cargo clippy --all-targets --features legacy-tools --
  -D warnings`, `cargo run --quiet --features legacy-tools --
  --actor-script-check examples/actor-custom-attract.script`, `make
  actor-smoke`, `make actor-wgpu-smoke`, `make actor-attract-smoke`,
  `markdownlint README.md SPEC.md PLAN.md docs/actor-architecture.md`, and
  `git diff --check`.

- `2026-06-07 09:20 BST`: Completed the actor script-check hostile
  laser-hit matrix cycle. `--actor-script-check <path>` now builds a bounded
  parser-backed hit-matrix probe from sectioned custom-driver scripts for
  lander, mutant, bomber, pod, swarmer, and baiter targets. Each generated
  probe reaches the first playable wave, freezes target motion/fire timers,
  sends one fire input, and reports sample step, score delta, cumulative
  score, explosion metadata, hit sound command, and spawned counts without
  mutating the main example-script wave-clear/reserve assist timeline. The
  checked matrix pins `0xF9` lander hit, `0xE8` mutant hit, `0xFE` bomber hit,
  `0xFA` pod hit with six spawned swarmers, `0xF8` swarmer hit, and `0xF8`
  baiter hit. No legacy code, tests, or scaffolding were safe to remove because
  this slice expands the active custom-driver preflight evidence surface for
  hostile-family score/explosion/audio fidelity. Slack start:
  <https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780817708724449>.
  Validation passed with `cargo fmt --check`, `cargo test actor_script_check
  --all-targets --features legacy-tools`, `cargo test actor_script
  --all-targets --features legacy-tools`, `cargo test actor_live --all-targets
  --features legacy-tools`, `cargo check --all-targets --features
  legacy-tools`, `cargo clippy --all-targets --features legacy-tools --
  -D warnings`, `cargo run --quiet --features legacy-tools --
  --actor-script-check examples/actor-custom-attract.script`, `make
  actor-smoke`, `make actor-wgpu-smoke`, `make actor-attract-smoke`,
  `markdownlint README.md SPEC.md PLAN.md docs/actor-architecture.md`, and
  `git diff --check`.

- `2026-06-07 08:28 BST`: Completed the actor script-check laser-hit
  explosion/audio preflight cycle. `--actor-script-check <path>` now runs a
  bounded independent first-player-laser-hit sample pass in a separate actor
  runtime: it reaches the first playable wave, sends one fire input, and
  reports the first explosion spawn plus cumulative score and mapped hit sound
  command when a script makes the shot collide, without mutating the main
  wave-clear/reserve assist timeline. The checked example now reports
  `first_player_laser_hit: unavailable` with the bounded no-hit reason, while a
  focused custom-driver regression pins a still lander at `62,120` producing
  `lander@62,120[source_center=none]`, cumulative score `250`, and the
  red-label lander-hit command `0xF9`. No legacy code, tests, or scaffolding
  were safe to remove because this slice extends the active custom-driver
  preflight evidence surface for explosion/audio fidelity. Slack start:
  <https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780814751016189>.
  Validation passed with `cargo fmt --check`, `cargo test actor_script_check
  --all-targets --features legacy-tools`, `cargo test actor_script
  --all-targets --features legacy-tools`, `cargo test actor_live --all-targets
  --features legacy-tools`, `cargo check --all-targets --features
  legacy-tools`, `cargo clippy --all-targets --features legacy-tools --
  -D warnings`, `cargo run --quiet --features legacy-tools --
  --actor-script-check examples/actor-custom-attract.script`, `make
  actor-smoke`, `make actor-wgpu-smoke`, `make actor-attract-smoke`,
  `markdownlint README.md SPEC.md PLAN.md docs/actor-architecture.md`, and
  `git diff --check`.

- `2026-06-07 07:36 BST`: Completed the actor script-check player-laser
  preflight cycle. `--actor-script-check <path>` now runs a bounded
  independent first-player-laser sample pass in a separate actor runtime: it
  reaches the first playable wave, sends one fire input, retains the `0xEB`
  laser sound command, and reports the first live `Laser` actor sample with
  position, velocity, and direction without mutating the main
  wave-clear/reserve assist timeline. The checked example now reports
  `first_player_laser_*` for `laser@62,120` with velocity `8/0`, direction
  `right`, and command `0xEB`; a focused custom-driver regression pins
  script-tuned `laser_speed 3` as `laser@57,120` with velocity `3/0` and the
  same red-label laser command. No legacy code, tests, or scaffolding were safe
  to remove because this slice extends the active custom-driver preflight
  evidence surface for laser/audio fidelity. Slack start:
  <https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780813454137549>.
  Validation passed with `cargo fmt --check`, `cargo test actor_script_check
  --all-targets --features legacy-tools`, `cargo test actor_script
  --all-targets --features legacy-tools`, `cargo test actor_live --all-targets
  --features legacy-tools`, `cargo check --all-targets --features
  legacy-tools`, `cargo clippy --all-targets --features legacy-tools --
  -D warnings`, `cargo run --quiet --features legacy-tools --
  --actor-script-check examples/actor-custom-attract.script`, `make
  actor-smoke`, `make actor-wgpu-smoke`, `make actor-attract-smoke`,
  `markdownlint README.md SPEC.md PLAN.md docs/actor-architecture.md`, and
  `git diff --check`.

- `2026-06-07 07:15 BST`: Completed the actor script-check projectile/audio
  sample cycle. `--actor-script-check <path>` now reports
  `*_source_projectile_samples` and `*_sound_commands` for playable summaries,
  including hostile projectile source fractions, source velocities, and
  lifetime ticks plus source sound-board command bytes. The checker also runs a
  bounded independent first-projectile sample pass in a separate actor runtime,
  preserving the main wave-clear/reserve assist timeline while reporting
  `first_source_projectile_*` evidence and the associated hostile shot command
  when a script produces one. The focused custom-driver regression pins a
  source mutant shot sample as `enemy_laser@0,220` with source fractions,
  velocities, lifetime `90`, and red-label command `0xF6`; the checked example
  now records the first-play `0xEA` player-appear command and explicitly
  reports no hostile source projectile inside the bounded sample window. No
  legacy code, tests, or scaffolding were safe to remove because this slice
  extends the active custom-driver preflight evidence surface. Slack start:
  <https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780799225096519>.
  Validation passed with `cargo fmt --check`, `cargo test actor_script_check
  --all-targets --features legacy-tools`, `cargo test actor_script
  --all-targets --features legacy-tools`, `cargo test actor_live --all-targets
  --features legacy-tools`, `cargo check --all-targets --features
  legacy-tools`, `cargo clippy --all-targets --features legacy-tools --
  -D warnings`, `cargo run --quiet --features legacy-tools --
  --actor-script-check examples/actor-custom-attract.script`, `make
  actor-smoke`, `make actor-wgpu-smoke`, `make actor-attract-smoke`,
  `markdownlint README.md SPEC.md PLAN.md docs/actor-architecture.md`, and
  `git diff --check`.

- `2026-06-07 03:18 BST`: Completed the actor script-check source placement
  sample cycle. `--actor-script-check <path>` now reports bounded
  `*_source_actor_samples` for first-play, next-wave, reserve-activation, and
  post-reserve playable summaries, including actor family, screen position, and
  source X/Y fractions for source-backed lander, bomber, pod, mutant, swarmer,
  baiter, and human snapshots. Reserve activation summaries also report
  `*_spawned_samples`, which capture the restored spawn command positions for
  each observed batch. The focused source-wave regression now pins the first
  bounded source-backed placement samples, and the reserve-heavy custom-driver
  regression now pins restored lander, bomber, pod, mutant, and swarmer reserve
  positions. No legacy code, tests, or scaffolding were safe to remove because
  this slice extends the active custom-driver preflight evidence surface. Slack
  start:
  <https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780797797302969>.
  Validation passed with `cargo fmt --check`, `cargo test actor_script_check
  --all-targets --features legacy-tools`, `cargo test actor_script
  --all-targets --features legacy-tools`, `cargo test actor_live --all-targets
  --features legacy-tools`, `cargo check --all-targets --features
  legacy-tools`, `cargo clippy --all-targets --features legacy-tools --
  -D warnings`, `cargo run --quiet --features legacy-tools --
  --actor-script-check examples/actor-custom-attract.script`, `make
  actor-smoke`, `make actor-wgpu-smoke`, `make actor-attract-smoke`,
  `markdownlint README.md SPEC.md PLAN.md docs/actor-architecture.md`, and
  `git diff --check`.

- `2026-06-07 02:19 BST`: Completed the bounded custom attract example cycle.
  `examples/actor-custom-attract.script` now declares a 96-step `[attract]`
  `cycle` and exercises script-authored Williams reveal, `ELECV`, Defender
  coalescence, Hall-of-Fame title/logo, credits, scoring surface, and `SWARMV`
  final scoring label actions before cycling back to the Williams reveal. The
  example keeps the same small custom behavior/wave starter so it remains
  editable while proving custom drivers can script attract graphics through the
  actor runtime. `--actor-script-check examples/actor-custom-attract.script`
  now reports eight attract events, 96 sampled attract frames, zero non-attract
  frames, 193 draw commands, 19009 scene sprites, all six attract milestone
  booleans true, and unchanged first-play gameplay/wave progression. No legacy
  code, tests, or scaffolding were safe to remove because this slice hardens
  the active checked custom-driver example. Slack start:
  <https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780789716828879>.
  Validation passed with `cargo fmt --check`, `cargo test actor_script_check
  --all-targets --features legacy-tools`, `cargo test actor_script
  --all-targets --features legacy-tools`, `cargo test actor_live --all-targets
  --features legacy-tools`, `cargo check --all-targets --features
  legacy-tools`, `cargo clippy --all-targets --features legacy-tools --
  -D warnings`, `cargo run --quiet --features legacy-tools --
  --actor-script-check examples/actor-custom-attract.script`, `make
  actor-smoke`, `make actor-wgpu-smoke`, `make actor-attract-smoke`,
  `markdownlint README.md SPEC.md PLAN.md docs/actor-architecture.md`, and
  `git diff --check`.

- `2026-06-07 00:39 BST`: Completed the actor script custom-attract preflight
  cycle. `--actor-script-check <path>` now samples a declared bounded
  `[attract]` `cycle` before the same runtime is credited and started, then
  reports `attract_cycle_*` step/frame/draw totals plus milestone booleans for
  Williams reveal, Defender coalescence, Hall-of-Fame, scoring surface, final
  scoring label, and cycle return. The focused custom-driver regression proves
  a 12-step custom cycle reports all six milestones, `12` sampled attract
  frames, zero non-attract frames, and nonzero draw/scene-sprite output. The
  checked minimal example remains intentionally unbounded and now reports
  `attract_cycle: unavailable,reason=no_attract_cycle_declared`, making the
  absence of a loop explicit for script authors. No legacy code, tests, or
  scaffolding were safe to remove because this slice hardens the active
  custom-driver preflight. Slack start:
  <https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780777234403239>.
  Validation passed with `cargo fmt --check`, `cargo test actor_script_check
  --all-targets --features legacy-tools`, `cargo test actor_script
  --all-targets --features legacy-tools`, `cargo test actor_live --all-targets
  --features legacy-tools`, `cargo test actor_game --all-targets --features
  legacy-tools`, `cargo check --all-targets --features legacy-tools`, `cargo
  clippy --all-targets --features legacy-tools -- -D warnings`, `cargo run
  --quiet --features legacy-tools -- --actor-script-check
  examples/actor-custom-attract.script`, `make actor-smoke`, `make
  actor-wgpu-smoke`, `make actor-attract-smoke`, `markdownlint README.md
  SPEC.md PLAN.md docs/actor-architecture.md`, and `git diff --check`.

- `2026-06-06 21:16 BST`: Completed the actor script post-reserve next-wave
  preflight cycle. After `post_reserve_wave_clear_*` and
  `post_reserve_wave_clear_advance_sleep_*`, `--actor-script-check <path>` now
  continues through normal actor reports until the next playable wave starts
  and reports it as `post_reserve_next_playing_*`. The reserve-heavy
  custom-driver regression proves wave three becomes playable at cumulative
  assist step `904`, with wave size `3`, source active counts
  landers/bombers/pods/mutants/swarmers `1/1/1/0/0`, world counts
  enemies/humans `3/10`, retained reserve counts `2/1/1/1/1`, and the
  scripted wave-two behavior profile still active for lander seek speed `7`,
  swarmer fire period `23`, and baiter fire period `31`. The minimal checked
  example now reports `post_reserve_next_playing` as unavailable with reason
  `next_playing_has_no_reserve`. No legacy code, tests, or scaffolding were
  safe to remove because this slice hardens the active script-authoring
  preflight. Slack start:
  <https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780766410047449>.
  Validation passed with `cargo fmt --check`, `cargo test actor_script_check
  --all-targets --features legacy-tools`, `cargo test actor_script
  --all-targets --features legacy-tools`, `cargo test actor_live --all-targets
  --features legacy-tools`, `cargo test actor_game --all-targets --features
  legacy-tools`, `cargo check --all-targets --features legacy-tools`, `cargo
  clippy --all-targets --features legacy-tools -- -D warnings`, `cargo run
  --quiet --features legacy-tools -- --actor-script-check
  examples/actor-custom-attract.script`, `make actor-smoke`, `make
  actor-wgpu-smoke`, `markdownlint README.md SPEC.md PLAN.md
  docs/actor-architecture.md`, and `git diff --check`.

- `2026-06-06 18:17 BST`: Completed the actor script post-reserve
  wave-advance sleep checker cycle. After a checked reserve-heavy custom
  driver reaches `post_reserve_wave_clear_*`, `--actor-script-check <path>`
  now keeps stepping the same actor `XYZZY` assist path until the following
  survivor-bonus report enters the source `0x80` wave-advance sleep, then
  reports it as `post_reserve_wave_clear_advance_sleep_*`. The focused
  regression proves the post-reserve wave-three sleep arrives at cumulative
  assist step `776`, score `6400`, zero world enemies, ten survivors, ten
  visible icons, no remaining awards, awarded points `none`, astronaut sleep
  `0`, and wave-advance sleep `128`, while the minimal checked example reports
  the block as unavailable with reason `next_playing_has_no_reserve`. No legacy
  code, tests, or scaffolding were safe to remove because this slice hardens
  the active script-authoring preflight. Slack start:
  <https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780762318637019>.
  Validation passed with `cargo fmt --check`, `cargo test actor_script_check
  --all-targets --features legacy-tools`, `cargo test actor_script
  --all-targets --features legacy-tools`, `cargo test actor_live --all-targets
  --features legacy-tools`, `cargo test actor_game --all-targets --features
  legacy-tools`, `cargo check --all-targets --features legacy-tools`, `cargo
  clippy --all-targets --features legacy-tools -- -D warnings`, `cargo run
  --quiet --features legacy-tools -- --actor-script-check
  examples/actor-custom-attract.script`, `make actor-smoke`, `make
  actor-wgpu-smoke`, `markdownlint README.md SPEC.md PLAN.md
  docs/actor-architecture.md`, and `git diff --check`.

- `2026-06-06 17:09 BST`: Completed the actor script post-reserve wave-clear
  checker cycle. `--actor-script-check <path>` now keeps stepping the same
  actor `XYZZY` overlay smart-bomb assist path after a checked next wave's
  reserve batches reach `reserve_empty`, and reports the first following
  source-shaped `WaveCleared` interstitial as `post_reserve_wave_clear_*`.
  The reserve-heavy custom-driver regression proves the checker observes three
  source reserve batches, then reaches the wave-three survivor-bonus
  interstitial at cumulative assist step `736`, score `4600`, zero world
  enemies, ten survivors, one visible icon, nine remaining awards, first
  awarded survivor points `200`, and source astronaut sleep `4`, without
  mutating the driver or introducing frame-driven logic. The minimal checked
  example now reports `post_reserve_wave_clear` as unavailable with reason
  `next_playing_has_no_reserve`, keeping no-reserve scripts explicit. No legacy
  code, tests, or scaffolding were safe to remove because this slice hardens
  the active script-authoring preflight. Slack start:
  <https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780761672170309>.
  Validation passed with `cargo fmt --check`, `cargo test actor_script_check
  --all-targets --features legacy-tools`, `cargo test actor_script
  --all-targets --features legacy-tools`, `cargo test actor_live --all-targets
  --features legacy-tools`, `cargo test actor_game --all-targets --features
  legacy-tools`, `cargo check --all-targets --features legacy-tools`, `cargo
  clippy --all-targets --features legacy-tools -- -D warnings`, `cargo run
  --quiet --features legacy-tools -- --actor-script-check
  examples/actor-custom-attract.script`, `make actor-smoke`, `make
  actor-wgpu-smoke`, `markdownlint README.md SPEC.md PLAN.md
  docs/actor-architecture.md`, and `git diff --check`.

- `2026-06-06 16:58 BST`: Completed the actor script wave-advance sleep
  checker cycle. `--actor-script-check <path>` now reports the first
  survivor-bonus report that enters the source `0x80` wave-advance sleep after
  all survivor awards have been rendered and scored. The report includes
  cumulative assist steps, next wave, score, world enemy/human counts, survivor
  count, visible survivor icons, remaining awards, awarded points as `none`,
  astronaut sleep at `0`, and wave-advance sleep at `128`, while continuing
  through the same actor `XYZZY` overlay smart-bomb path to next-wave and
  reserve-batch preflight. The focused regressions prove the checked example
  enters wave-advance sleep at assist step `12` with two icons and score `500`,
  and the custom ten-survivor script enters it at assist step `44` with ten
  icons and score `1150` before wave two/reserves. No legacy code, tests, or
  scaffolding were safe to remove because this slice hardens the active
  script-authoring preflight. Slack start:
  <https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780760941145729>.
  Validation passed with `cargo fmt --check`, `cargo test actor_script_check
  --all-targets --features legacy-tools`, `cargo test actor_script
  --all-targets --features legacy-tools`, `cargo test actor_live --all-targets
  --features legacy-tools`, `cargo test actor_game --all-targets --features
  legacy-tools`, `cargo check --all-targets --features legacy-tools`, `cargo
  clippy --all-targets --features legacy-tools -- -D warnings`, `cargo run
  --quiet --features legacy-tools -- --actor-script-check
  examples/actor-custom-attract.script`, `make actor-smoke`, `make
  actor-wgpu-smoke`, `markdownlint README.md SPEC.md PLAN.md
  docs/actor-architecture.md`, and `git diff --check`.

- `2026-06-06 16:45 BST`: Completed the actor script wave-clear
  interstitial checker cycle. `--actor-script-check <path>` now reports the
  first source-shaped `WaveCleared` interstitial it observes during assisted
  next-wave preflight, including cumulative assist steps, next wave, score,
  world enemy/human counts, survivor count, visible survivor icons, remaining
  survivor awards, first awarded survivor points, and source interstitial sleep
  fields. The checker continues through the same actor `XYZZY` overlay
  smart-bomb path to the next playable wave and reserve activation batches, so
  custom wave scripts can now preflight first play, survivor-bonus handoff,
  next-wave behavior, and queued reserve ordering without bypassing actor
  commands or adding frame-driven logic. The focused regressions prove the
  checked example script reports a two-survivor interstitial before wave two,
  and the two-wave reserve script reports a ten-survivor interstitial before
  its source-backed second wave and reserve sequence. No legacy code, tests, or
  scaffolding were safe to remove because this slice hardens the active
  script-authoring preflight. Slack start:
  <https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780760350111719>.
  Validation passed with `cargo fmt --check`, `cargo test actor_script_check
  --all-targets --features legacy-tools`, `cargo test actor_script
  --all-targets --features legacy-tools`, `cargo test actor_live --all-targets
  --features legacy-tools`, `cargo test actor_game --all-targets --features
  legacy-tools`, `cargo check --all-targets --features legacy-tools`, `cargo
  clippy --all-targets --features legacy-tools -- -D warnings`, `cargo run
  --quiet --features legacy-tools -- --actor-script-check
  examples/actor-custom-attract.script`, `make actor-smoke`, `make
  actor-wgpu-smoke`, `markdownlint README.md SPEC.md PLAN.md
  docs/actor-architecture.md`, and `git diff --check`.

- `2026-06-06 16:33 BST`: Completed the actor script reserve-activation
  sequence checker cycle. `--actor-script-check <path>` now extends assisted
  next-wave preflight into a bounded reserve activation batch sequence instead
  of stopping at the first restored batch. The checker still drives the same
  actor `XYZZY` overlay smart-bomb path, preserving real smart-bomb detonation,
  source reserve cooldown, wave-clear, and reserve activation behavior, then
  reports each observed batch's cumulative assist steps, restored family spawn
  counts, wave/source/world/reserve/source-state, and the terminal batch
  status. The focused custom-driver regression now queues lander, bomber, pod,
  mutant, and swarmer reserves and proves the source ordering: lander reserves
  restore first, the next mixed batch restores bomber/pod/mutant, and the final
  batch restores swarmer before reporting `reserve_empty`. No legacy code,
  tests, or scaffolding were safe to remove because this slice hardens the
  active script-authoring preflight. Slack start:
  <https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780759603875789>.
  Validation passed with `cargo fmt --check`, `cargo test actor_script_check
  --all-targets --features legacy-tools`, `cargo test actor_script
  --all-targets --features legacy-tools`, `cargo test actor_live --all-targets
  --features legacy-tools`, `cargo test actor_game --all-targets --features
  legacy-tools`, `cargo check --all-targets --features legacy-tools`, `cargo
  clippy --all-targets --features legacy-tools -- -D warnings`, `cargo run
  --quiet --features legacy-tools -- --actor-script-check
  examples/actor-custom-attract.script`, `make actor-smoke`, `make
  actor-wgpu-smoke`, `markdownlint README.md SPEC.md PLAN.md
  docs/actor-architecture.md`, and `git diff --check`.

- `2026-06-06 16:19 BST`: Completed the actor script reserve-activation
  checker cycle. `--actor-script-check <path>` now extends the assisted
  next-wave preflight into the first restored reserve batch when a checked
  script's next playable wave still has reserves. The checker drives this
  through the actor `XYZZY` overlay smart-bomb path, preserving the source
  smart-bomb cooldown and reserve-activation logic, then reports the restored
  family spawn counts, wave/source counts, spawned world counts, remaining
  reserve/source-state, and effective behavior summary. The focused regression
  proves a two-wave custom-driver script restores two lander reserve rows
  first, leaves the bomber reserve queued for the following batch, and reports
  those runtime-effective values before live launch. No legacy code, tests, or
  scaffolding were safe to remove because this slice hardens the active
  script-authoring gate. Slack start:
  <https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780758849795749>.
  Validation passed with `cargo fmt --check`, `cargo test actor_script_check
  --all-targets --features legacy-tools`, `cargo test actor_script
  --all-targets --features legacy-tools`, `cargo run --quiet --features
  legacy-tools -- --actor-script-check examples/actor-custom-attract.script`,
  `cargo test actor_live --all-targets --features legacy-tools`, `cargo test
  actor_game --all-targets --features legacy-tools`, `cargo check
  --all-targets --features legacy-tools`, `cargo clippy --all-targets
  --features legacy-tools -- -D warnings`, `make actor-smoke`, `make
  actor-wgpu-smoke`, `markdownlint README.md SPEC.md PLAN.md
  docs/actor-architecture.md`, and `git diff --check`.

- `2026-06-06 13:08 BST`: Completed the actor script next-wave checker cycle.
  `--actor-script-check <path>` now keeps the first playable wave summary and
  also advances the same actor runtime with the actor `XYZZY` overlay
  smart-bomb path until the next playable wave starts, reporting the second
  wave's source counts, spawned world counts, reserve/source-state, and
  effective behavior summary when reached. The focused regression proves a
  two-wave sectioned custom-driver script can change wave-two
  source/reserve counts and behavior tuning, and that the headless checker
  reports those installed values after real actor wave-clear and wave-start
  progression. No legacy code, tests, or scaffolding were safe to remove
  because this slice hardens the active script-authoring gate. Slack start:
  <https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780728333355649>.
  Validation passed with `cargo fmt --check`, `cargo test actor_script_check
  --all-targets --features legacy-tools`, `cargo test actor_script
  --all-targets --features legacy-tools`, `cargo run --quiet --features
  legacy-tools -- --actor-script-check examples/actor-custom-attract.script`,
  `cargo test actor_live --all-targets --features legacy-tools`, `cargo test
  actor_game --all-targets --features legacy-tools`, `cargo check
  --all-targets --features legacy-tools`, `cargo clippy --all-targets
  --features legacy-tools -- -D warnings`, `make actor-smoke`, `make
  actor-wgpu-smoke`, `markdownlint README.md SPEC.md PLAN.md
  docs/actor-architecture.md`, and `git diff --check`.

- `2026-06-06 07:39 BST`: Completed the actor script reserve/source-state
  checker cycle. `--actor-script-check <path>` now reports first-play enemy
  reserve counts from the clean `WorldSnapshot::enemy_reserve` bridge plus
  `source_background_left` and the source RNG snapshot from the same
  `StepReport` actors receive at the first playable wave. The focused
  regression proves a custom `reserve_full` source-wave script retains reserve
  counts and source state through runtime startup before live launch. No legacy
  code, tests, or scaffolding were safe to remove because this slice hardens
  the active script-authoring gate. Slack start:
  <https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780727756343389>.
  Validation passed with `cargo fmt --check`, `cargo test actor_script_check
  --all-targets --features legacy-tools`, `cargo test actor_script
  --all-targets --features legacy-tools`, `cargo run --quiet --features
  legacy-tools -- --actor-script-check examples/actor-custom-attract.script`,
  `cargo test actor_live --all-targets --features legacy-tools`, `cargo test
  actor_game --all-targets --features legacy-tools`, `cargo check
  --all-targets --features legacy-tools`, `cargo clippy --all-targets
  --features legacy-tools -- -D warnings`, `make actor-smoke`, `make
  actor-wgpu-smoke`, `markdownlint README.md SPEC.md PLAN.md
  docs/actor-architecture.md`, and `git diff --check`.

- `2026-06-06 07:28 BST`: Completed the actor script effective-behavior
  checker cycle. `--actor-script-check <path>` now derives a first-play
  behavior summary from the same effective `StepReport::behavior_script` actors
  receive after credit/start reaches the first playable wave. The report prints
  player damage/cooldown, first lander movement/fire tuning, hostile family
  movement modes, and swarmer/baiter fire periods alongside the existing
  manifest and source/world counts. The focused regression proves a sectioned
  custom-driver script can tune player damage, laser cooldown, wave-local
  hostile modes, swarmer/baiter cadence, and a spawn-index lander override, and
  that the headless checker reports those runtime-effective values before live
  launch. No legacy code, tests, or scaffolding were safe to remove because this
  slice hardens the active script-authoring gate. Slack start:
  <https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780727063534209>.
  Validation passed with `cargo fmt --check`, `cargo test actor_script_check
  --all-targets --features legacy-tools`, `cargo test actor_script
  --all-targets --features legacy-tools`, `cargo run --quiet --features
  legacy-tools -- --actor-script-check examples/actor-custom-attract.script`,
  `cargo test actor_live --all-targets --features legacy-tools`, `cargo test
  actor_game --all-targets --features legacy-tools`, `cargo check
  --all-targets --features legacy-tools`, `cargo clippy --all-targets
  --features legacy-tools -- -D warnings`, `make actor-smoke`, `make
  actor-wgpu-smoke`, `markdownlint README.md SPEC.md PLAN.md
  docs/actor-architecture.md`, and `git diff --check`.

- `2026-06-06 07:16 BST`: Completed the actor script gameplay-start checker
  cycle. `--actor-script-check <path>` still parses the sectioned custom-driver
  script and samples the first attract actor step, but now also credits and
  starts the same actor runtime until the first playable wave is active. The
  report prints the effective first-wave source counts and spawned world
  enemies/humans, so script authors can prove custom `[wave]` and
  `source_wave` settings survive into gameplay before opening the live window.
  Focused regressions cover the checked example script's two custom landers/two
  humans and a source-backed override that produces one lander, bomber, pod,
  mutant, and swarmer at play start. No legacy code, tests, or scaffolding were
  safe to remove because this slice hardens the active script-authoring gate.
  Slack start:
  <https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780726281836709>.
  Validation passed with `cargo fmt --check`, `cargo test actor_script_check
  --all-targets --features legacy-tools`, `cargo run --quiet --features
  legacy-tools -- --actor-script-check examples/actor-custom-attract.script`,
  `cargo test actor_script --all-targets --features legacy-tools`, `cargo test
  actor_live --all-targets --features legacy-tools`, `cargo test actor_game
  --all-targets --features legacy-tools`, `cargo check --all-targets
  --features legacy-tools`, `cargo clippy --all-targets --features
  legacy-tools -- -D warnings`, `make actor-smoke`, `make
  actor-wgpu-smoke`, `markdownlint README.md SPEC.md PLAN.md
  docs/actor-architecture.md`, and `git diff --check`.

- `2026-06-06 07:04 BST`: Completed the actor effective source-wave state
  bridge cycle. `StepReport` now carries the same effective
  `ActorSourceWaveProfile` that actors receive through `StepPrompt`, and
  `ActorStateBridge` adapts it into clean `GameState::wave_profile` so custom
  `source_wave` / `source_waves` script overrides remain visible through the
  runtime state contract. The focused regression extends the custom source-wave
  script test to prove overridden family counts, shot timers, baiter timing,
  and mutant/swarmer tuning reach both `StepReport` and clean `GameState`;
  source-only `wave_time` stays on the clean table value. README, SPEC, PLAN,
  and the actor architecture guide document the report/state bridge contract.
  No legacy code, tests, or scaffolding were safe to remove in this slice
  because the change tightens the active actor-to-clean runtime bridge. Slack
  start:
  <https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780725747441859>.
  Validation passed with `cargo test
  parsed_source_wave_overrides_drive_source_shaped_custom_wave --lib --features
  legacy-tools`, `cargo test actor_state_bridge --lib --features
  legacy-tools`, `cargo fmt --check`, `cargo test wave_script --lib --features
  legacy-tools`, `cargo test actor_game --all-targets --features
  legacy-tools`, `cargo test actor_live --all-targets --features
  legacy-tools`, `cargo check --all-targets --features legacy-tools`, `cargo
  clippy --all-targets --features legacy-tools -- -D warnings`, `make
  actor-smoke`, and `make actor-wgpu-smoke`.

- `2026-06-06 06:54 BST`: Completed the actor source-RNG state bridge cycle.
  `ActorStateBridge` now preserves `StepReport::source_rng` in the clean
  `WorldSnapshot::source_rng` field instead of publishing the default clean RNG
  while source-backed actors consume a different driver-owned RNG from
  `StepPrompt`. The focused regression extends the driver source-RNG report
  test to assert the same value reaches the clean `GameState` bridge. README,
  SPEC, and the actor architecture guide document the state-bridge contract.
  No legacy code, tests, or scaffolding were safe to remove in this slice
  because the change tightens the actor-to-clean bridge used by the existing
  smoke and fidelity gates. Slack start:
  <https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780725219910189>.
  Validation passed with `cargo test
  playing_step_report_carries_driver_source_rng_snapshot --lib --features
  legacy-tools`, `cargo test actor_state_bridge --lib --features
  legacy-tools`, `cargo fmt --check`, `cargo test actor_game --all-targets
  --features legacy-tools`, `cargo test actor_live --all-targets --features
  legacy-tools`, `cargo check --all-targets --features legacy-tools`, `cargo
  clippy --all-targets --features legacy-tools -- -D warnings`, `make
  actor-smoke`, `make actor-wgpu-smoke`, `markdownlint README.md SPEC.md
  PLAN.md docs/actor-architecture.md`, and `git diff --check`.

- `2026-06-06 06:46 BST`: Completed the actor script check tooling cycle.
  Added `--actor-script-check <path>` as a headless custom-driver validation
  command: it reads a sectioned actor driver script, parses it as
  `ActorDriverScripts`, boots `ActorRuntimeAdapter::with_scripts`, steps one
  actor frame, and prints attract, behavior, wave, first-frame phase, and draw
  counts. Added the checked `examples/actor-custom-attract.script` sample as a
  minimal custom attract/behavior/wave bundle that can be checked before live
  launch. Focused regressions cover CLI classification, malformed arguments,
  runtime command dispatch, installed backend execution, the report text, and
  the example script's parsed/drawn surface. README, SPEC, and the actor
  architecture guide now document the check-then-launch workflow. No legacy
  code, tests, or scaffolding were safe to remove because this slice adds the
  active script-authoring validation path. Slack start:
  <https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780724548401479>.
  Validation passed with `cargo test actor_script_check --all-targets`, `cargo
  run -- --actor-script-check examples/actor-custom-attract.script`, `cargo
  test actor_script --all-targets`, `cargo test actor_live --all-targets
  --features legacy-tools`, `cargo test actor_game --all-targets --features
  legacy-tools`, `cargo check --all-targets --features legacy-tools`, `cargo
  clippy --all-targets --features legacy-tools -- -D warnings`, `make
  actor-smoke`, `make actor-wgpu-smoke`, `cargo fmt --check`, `markdownlint
  README.md SPEC.md PLAN.md docs/actor-architecture.md
  docs/fidelity/mame-golden-clips.md docs/fidelity/release-closure-audit.md
  assets/sounds/README.md assets/red-label/README.md`, and `git diff
  --check`.

- `2026-06-06 06:34 BST`: Completed the actor script file runtime-loading
  cycle. `--actor-script <path>` is now a live actor runtime option: the CLI
  stores it in `RuntimeConfig`, runtime command mapping preserves it, and
  `live_wgpu` reads the file, parses it as `ActorDriverScripts`, and boots
  `ActorRuntimeAdapter::with_scripts` before the actor window starts. The CLI
  rejects `--actor-script` with `--live-smoke` because that path still runs the
  clean-game smoke. Focused regressions cover CLI classification, runtime
  command propagation, live-entrypoint parsing of a sectioned script file,
  custom attract drawing from that file-backed runtime, and contextual
  read/parse failures. README, SPEC, and the actor architecture guide now
  document the live script-file entry point. No legacy code, tests, or
  scaffolding were safe to remove because this slice only opens the active
  actor driver script surface to the playable runtime. Slack start:
  <https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780723706769549>.
  Validation passed with `cargo test actor_script --all-targets`, `cargo test
  actor_live_entrypoint_loads_sectioned_script_file_under_tests --all-targets`,
  `cargo test runtime_host_adapts_default_config_to_actor_launch_command
  --all-targets`, `cargo test clean_cli_rejects_malformed_live_args
  --all-targets`, `cargo test clean_help_text_preserves_current_cli_contract
  --all-targets`, `cargo test
  clean_runtime_and_oracle_use_quarantined_adapters --lib`, `cargo test
  actor_live --all-targets --features legacy-tools`, `cargo test actor_game
  --all-targets --features legacy-tools`, `cargo check --all-targets
  --features legacy-tools`, `cargo clippy --all-targets --features
  legacy-tools -- -D warnings`, `make actor-smoke`, `make actor-wgpu-smoke`,
  `cargo fmt --check`, `markdownlint README.md SPEC.md PLAN.md
  docs/actor-architecture.md docs/fidelity/mame-golden-clips.md
  docs/fidelity/release-closure-audit.md assets/sounds/README.md
  assets/red-label/README.md`, and `git diff --check`.

- `2026-06-06 06:20 BST`: Completed the runnable actor driver script cycle.
  Sectioned custom-driver scripts now implement
  `str::parse::<ActorDriverScripts>()`, expose
  `ActorDriverScripts::manifest()` for pre-driver attract/behavior/wave
  inspection, and can boot directly through `ActorRuntimeAdapter::with_scripts`.
  Added a focused runtime regression that parses one sectioned script, verifies
  the manifest before startup, boots the runtime from that bundle, confirms the
  custom attract text is drawn, and checks that bundled wave parsing inherited
  the driver behavior script. README, SPEC, the actor architecture guide, and
  `assets/red-label/README.md` now document the runnable sectioned-bundle API.
  No legacy code, tests, or scaffolding were safe to remove because this slice
  only adds the public runtime adapter path for the active custom-driver
  surface. Slack start:
  <https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780723063161409>.
  Validation passed with `cargo test
  sectioned_driver_script_parses_to_manifest_and_runtime_adapter --lib
  --features legacy-tools`, `cargo test sectioned_driver_script --lib
  --features legacy-tools`, `cargo test driver_script_bundle --lib --features
  legacy-tools`, `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo check --all-targets --features legacy-tools`, `cargo clippy
  --all-targets --features legacy-tools -- -D warnings`, `make actor-smoke`,
  `make actor-wgpu-smoke`, `cargo fmt --check`, `markdownlint README.md
  SPEC.md PLAN.md docs/actor-architecture.md
  docs/fidelity/mame-golden-clips.md docs/fidelity/release-closure-audit.md
  assets/sounds/README.md assets/red-label/README.md`, and `git diff
  --check`.

- `2026-06-06 06:09 BST`: Completed the sectioned actor driver script cycle.
  `ActorDriverScripts::parse_text` now accepts one checked custom-driver
  script with `[attract]`, `[behavior]`, and `[wave]` sections, then delegates
  each section to the existing attract, behavior, and base-inheriting wave
  parsers. The delegated section buffers preserve original source line numbers,
  so malformed wave or behavior lines in a combined custom-driver script still
  report their real file line. Added focused regressions for sectioned script
  runtime installation, custom attract drawing, inherited playing behavior, and
  sectioned/unknown-section parse errors. README, SPEC, the actor architecture
  guide, and `assets/red-label/README.md` now document the sectioned custom
  driver script form. No legacy code, tests, or scaffolding were safe to remove
  because the changed surface is the active custom-driver parser API. Slack
  start:
  <https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780722411442889>.
  Validation passed with `cargo test sectioned_driver_script --lib --features
  legacy-tools`, `cargo test driver_script_bundle --lib --features
  legacy-tools`, `cargo test wave_script --lib --features legacy-tools`,
  `cargo test actor_game --all-targets --features legacy-tools`, `cargo check
  --all-targets --features legacy-tools`, `cargo clippy --all-targets
  --features legacy-tools -- -D warnings`, `make actor-smoke`, `make
  actor-wgpu-smoke`, `cargo fmt --check`, `markdownlint README.md SPEC.md
  PLAN.md docs/actor-architecture.md docs/fidelity/mame-golden-clips.md
  docs/fidelity/release-closure-audit.md assets/sounds/README.md
  assets/red-label/README.md`, and `git diff --check`.

- `2026-06-06 06:01 BST`: Completed the actor driver script bundle cycle.
  Added `ActorDriverScripts` as a checked attract/behavior/wave script bundle
  for custom drivers, with sectioned parse errors for malformed attract,
  behavior, or wave text. `ActorWaveScript::parse_text_with_base_behavior`
  now lets bundled wave parsing inherit the parsed behavior script as its
  baseline before wave-local updates and source-backed wave-table fields are
  applied. `ActorGameDriver::new` now builds from the same red-label bundle
  path, and `ActorGameDriver::with_scripts` installs a parsed custom bundle
  without requiring post-construction mutation. Added focused regressions for
  bundled attract drawing, inherited clean/source wave behavior, playing
  manifest state, and sectioned wave parse errors. README, SPEC, the actor
  architecture guide, and `assets/red-label/README.md` now document the bundle
  and inheritance contract. No legacy code, tests, or scaffolding were safe to
  remove in this slice because the changed surface is the active custom-driver
  script API. Slack start:
  <https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780721659607239>.
  Validation passed with `cargo test driver_script_bundle --lib --features
  legacy-tools`, `cargo test
  wave_script_text_parser_can_inherit_custom_base_behavior --lib --features
  legacy-tools`, `cargo test default_driver_exposes_embedded_actor_script_manifests
  --lib --features legacy-tools`, `cargo test wave_script --lib --features
  legacy-tools`, `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo check --all-targets --features legacy-tools`, `cargo clippy
  --all-targets --features legacy-tools -- -D warnings`, `make actor-smoke`,
  `make actor-wgpu-smoke`, `cargo fmt --check`, `markdownlint README.md
  SPEC.md PLAN.md docs/actor-architecture.md
  docs/fidelity/mame-golden-clips.md docs/fidelity/release-closure-audit.md
  assets/sounds/README.md assets/red-label/README.md`, and `git diff --check`.

- `2026-06-06 05:50 BST`: Completed the actor wave preset manifest cycle.
  `ActorWaveScriptManifest` now preserves parsed `behavior_preset` definitions
  as normalized names plus checked update lines, and parsed
  `spawn_behavior_preset` definitions as normalized names plus structured
  profile-field/value updates. Constructor-built wave scripts report empty
  preset lists, while parsed scripts expose both the reusable preset blocks and
  their resolved per-wave effects. This lets custom-driver tooling inspect or
  serialize reusable level-difficulty and specific actor-slot tuning without
  reaching into parser internals or actor threads. Added focused manifest
  regressions for normalized behavior/spawn preset export, resolved wave
  effects, and empty constructor preset lists. README, SPEC, and the actor
  architecture guide now document preset manifest visibility. No legacy code,
  tests, or scaffolding were safe to remove in this slice because the changed
  surface is the active actor wave manifest API and its custom-driver
  documentation. Slack start:
  <https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780721020254889>.
  Validation passed with `cargo test
  parsed_wave_script_manifest_exposes_reusable_behavior_presets --lib
  --features legacy-tools`, `cargo test
  driver_script_manifest_exports_current_wave_and_spawns --lib --features
  legacy-tools`, `cargo test wave_script --lib --features legacy-tools`,
  `cargo test actor_game --all-targets --features legacy-tools`, `cargo test
  actor_smoke --all-targets --features legacy-tools`, `cargo test actor_live
  --all-targets --features legacy-tools`, `cargo check --all-targets
  --features legacy-tools`, `cargo clippy --all-targets --features
  legacy-tools -- -D warnings`, `make actor-smoke`, `make actor-wgpu-smoke`
  with `frame_source: actor_game`, `cargo fmt --check`, `markdownlint README.md
  SPEC.md PLAN.md docs/actor-architecture.md
  docs/fidelity/mame-golden-clips.md docs/fidelity/release-closure-audit.md
  assets/sounds/README.md assets/red-label/README.md`, and `git diff --check`.

- `2026-06-06 05:41 BST`: Completed the actor spawn behavior preset scripting
  cycle. Checked wave scripts now accept
  `spawn_behavior_preset <name> <field> <value...>` definitions,
  `use_spawn_behavior <kind> <index> <name>` for the current wave, and
  `use_spawn_behavior_waves <first> <last> <kind> <index> <name>` for existing
  wave ranges. Spawn presets are validated through the same behavior-profile
  field parser as direct `spawn_behavior` lines, append repeated update lines
  to one named block, and replay onto spawn-index behavior profiles without
  resetting inherited wave/source behavior unless the preset explicitly changes
  those fields. Undefined spawn preset uses are rejected with source line
  numbers. Added focused parser/manifest regressions for current-wave and range
  preset use, source-backed behavior preservation, actor-id behavior
  installation after spawn allocation, undefined preset errors, and preset
  validation errors. README, SPEC, `assets/red-label/README.md`, and the actor
  architecture guide now document the spawn preset contract. No legacy code,
  tests, or scaffolding were safe to remove in this slice because the changed
  surface is the active actor wave parser and its custom-driver documentation.
  Slack start:
  <https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780720541464139>.
  Validation passed with `cargo test
  parsed_wave_script_applies_spawn_behavior_presets_to_current_and_range_profiles
  --lib --features legacy-tools`, `cargo test
  parsed_wave_script_applies_spawn_behavior_preset_after_actor_allocation --lib
  --features legacy-tools`, `cargo test
  parsed_wave_script_reports_unknown_spawn_behavior_presets --lib --features
  legacy-tools`, `cargo test wave_script_text_parser_reports_line_errors --lib
  --features legacy-tools`, `cargo test wave_script --lib --features
  legacy-tools`, `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo test actor_smoke --all-targets --features legacy-tools`, `cargo test
  actor_live --all-targets --features legacy-tools`, `cargo check
  --all-targets --features legacy-tools`, `cargo clippy --all-targets
  --features legacy-tools -- -D warnings`, `make actor-smoke`, `make
  actor-wgpu-smoke` with `frame_source: actor_game`, `cargo fmt --check`,
  `markdownlint README.md SPEC.md PLAN.md docs/actor-architecture.md
  docs/fidelity/mame-golden-clips.md docs/fidelity/release-closure-audit.md
  assets/sounds/README.md assets/red-label/README.md`, and `git diff --check`.

- `2026-06-06 05:29 BST`: Completed the actor wave behavior preset scripting
  cycle. Checked wave scripts now accept
  `behavior_preset <name> <profile-update...>` definitions,
  `use_behavior <name>` for the current wave, and
  `use_behavior_waves <first> <last> <name>` for existing wave ranges. Presets
  are validated through the same `ActorBehaviorScript` parser as direct
  behavior lines, append repeated update lines to one named block, and replay
  onto existing wave profiles without resetting source-backed movement/fire
  fields unless the preset explicitly changes them. Undefined preset uses are
  rejected with source line numbers. Added focused parser/manifest regressions
  for current-wave preset use, range preset use, source-backed behavior
  preservation, undefined preset errors, and preset validation errors. README,
  SPEC, `assets/red-label/README.md`, and the actor architecture guide now
  document the preset contract. No legacy code, tests, or scaffolding were safe
  to remove in this slice because the changed surface is the active actor wave
  parser and its custom-driver documentation. Slack start:
  <https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780719977340469>.
  Validation passed with `cargo test
  parsed_wave_script_applies_named_behavior_presets_to_current_and_range_profiles
  --lib --features legacy-tools`, `cargo test
  parsed_wave_script_reports_unknown_behavior_presets --lib --features
  legacy-tools`, `cargo test wave_script_text_parser_reports_line_errors --lib
  --features legacy-tools`, `cargo test wave_script --lib --features
  legacy-tools`, `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo test actor_smoke --all-targets --features legacy-tools`, `cargo test
  actor_live --all-targets --features legacy-tools`, `cargo check
  --all-targets --features legacy-tools`, `cargo clippy --all-targets
  --features legacy-tools -- -D warnings`, `make actor-smoke`, `make
  actor-wgpu-smoke` with `frame_source: actor_game`, `cargo fmt --check`,
  `markdownlint README.md SPEC.md PLAN.md docs/actor-architecture.md
  docs/fidelity/mame-golden-clips.md docs/fidelity/release-closure-audit.md
  assets/sounds/README.md assets/red-label/README.md`, and `git diff --check`.

- `2026-06-06 05:22 BST`: Completed the actor wave-range behavior scripting
  cycle. Checked wave scripts now accept `behavior_waves <first> <last> ...`
  and `spawn_behavior_waves <first> <last> <kind> <index> <field> <value>`
  directives, applying the same behavior parser and spawn-index behavior
  profile updates across already-defined source-backed or clean wave profiles.
  This lets custom drivers tune movement, fire cadence, damage policy, and
  specific restored actor slots across a progression band without repeating
  wave blocks. The parser rejects range behavior directives that reference
  undefined waves, preserving explicit custom-driver script ownership. Added
  focused parser/manifest regressions for ranged behavior, ranged spawn-index
  behavior, and undefined-wave errors. README, SPEC,
  `assets/red-label/README.md`, and the actor architecture guide now document
  the range behavior contract. Slack start:
  <https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780719205279769>.
  Validation passed with `cargo test
  parsed_wave_script_applies_behavior_ranges_to_existing_profiles --lib
  --features legacy-tools`, `cargo test
  parsed_wave_script_reports_missing_behavior_range_profiles --lib --features
  legacy-tools`, `cargo test wave_script --lib --features legacy-tools`,
  `cargo test actor_game --all-targets --features legacy-tools`, `cargo test
  actor_smoke --all-targets --features legacy-tools`, `cargo test actor_live
  --all-targets --features legacy-tools`, `cargo check --all-targets
  --features legacy-tools`, `cargo clippy --all-targets --features
  legacy-tools -- -D warnings`, `make actor-smoke`, `make actor-wgpu-smoke`
  with `frame_source: actor_game`, `cargo fmt --check`, `markdownlint README.md
  SPEC.md PLAN.md docs/actor-architecture.md
  docs/fidelity/mame-golden-clips.md docs/fidelity/release-closure-audit.md
  assets/sounds/README.md assets/red-label/README.md`, and `git diff --check`.

- `2026-06-06 05:11 BST`: Completed the actor source-wave range override
  cycle. `source_waves <first> <last>` script lines now accept the same checked
  `<field> <value>` overrides as single `source_wave` lines and apply the
  effective `ActorSourceWaveProfile` to every expanded source-backed wave in
  the range. Added a parser/manifest regression proving the tuned active family
  counts, direct mutant/swarmer slots, source metadata, and zero reserve
  remainder are applied to both waves in a range, while the embedded
  `actor-waves.script` default still expands source table values unchanged.
  README, SPEC, `assets/red-label/README.md`, and the actor architecture guide
  now document that custom drivers can tune one source-backed level or a whole
  source-backed progression range through the same checked script syntax.
  Slack start:
  <https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780718670474479>.
  Validation passed with `cargo test
  parsed_source_wave_range_overrides_apply_to_each_expanded_profile --lib
  --features legacy-tools`, `cargo test source_wave --lib --features
  legacy-tools`, `cargo test actor_game --all-targets --features
  legacy-tools`, `cargo test actor_smoke --all-targets --features
  legacy-tools`, `cargo test actor_live --all-targets --features legacy-tools`,
  `cargo check --all-targets --features legacy-tools`, `cargo clippy
  --all-targets --features legacy-tools -- -D warnings`, `make actor-smoke`,
  `make actor-wgpu-smoke` with `frame_source: actor_game`, `cargo fmt
  --check`, `markdownlint README.md SPEC.md PLAN.md docs/actor-architecture.md
  docs/fidelity/mame-golden-clips.md
  docs/fidelity/release-closure-audit.md assets/sounds/README.md
  assets/red-label/README.md`, and `git diff --check`.

- `2026-06-06 05:02 BST`: Completed the actor source-wave scripting
  override cycle. `source_wave <wave>` script lines now accept checked
  `<field> <value>` overrides for exposed `ActorSourceWaveProfile` fields,
  including active family counts, direct mutants, swarmer counts, movement
  constants, shot timers, baiter cadence, and mutant hop/shot settings. The
  source-backed allocator now includes direct mutant and swarmer active slots
  when the effective source profile exposes those counts, and source-shaped
  initial mutant/swarmer spawns carry their source metadata into snapshots.
  `StepPrompt` now carries the driver-owned effective source profile, so
  source-backed landers, mutants, swarmers, and baiters consume script-tuned
  profile values instead of re-reading `wave-table.tsv`. README, SPEC, and the
  actor architecture guide now document the custom-driver source-wave override
  contract. Slack start:
  <https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780717699601259>.
  Validation passed with `cargo test
  parsed_source_wave_overrides_drive_source_shaped_custom_wave --lib
  --features legacy-tools`, `cargo test
  source_mutant_actor_uses_prompt_source_wave_profile --lib --features
  legacy-tools`, `cargo test source_wave --lib --features legacy-tools`,
  `cargo test actor_game --all-targets --features legacy-tools`, `cargo test
  actor_smoke --all-targets --features legacy-tools`, `cargo test actor_live
  --all-targets --features legacy-tools`, `cargo check --all-targets
  --features legacy-tools`, `cargo clippy --all-targets --features
  legacy-tools -- -D warnings`, `make actor-smoke`, `make actor-wgpu-smoke`
  with `frame_source: actor_game`, `cargo fmt --check`, `markdownlint README.md
  SPEC.md PLAN.md docs/actor-architecture.md
  docs/fidelity/mame-golden-clips.md docs/fidelity/release-closure-audit.md
  assets/sounds/README.md`, and `git diff --check`.

- `2026-06-06 04:46 BST`: Completed the actor source-wave manifest
  introspection cycle. `ActorWaveProfile` and `ActorWaveProfileManifest` now
  preserve `source_wave: Option<ActorSourceWaveProfile>` so source-backed
  `source_wave` / `source_waves` script entries expose the exact red-label
  wave-table record used for active counts, source movement, shot cadence,
  baiter timing, and mutant hop/shot behavior; hand-scripted custom waves leave
  that field empty. Focused regressions now prove default, parsed embedded, and
  driver current-wave manifests carry the source-backed values while custom
  manifest waves remain source-free. README, SPEC, and actor architecture docs
  now describe this custom-driver inspection surface. Validation passed with
  `cargo fmt --check`, `cargo test source_wave --lib --features legacy-tools`,
  `cargo test default_wave_script_uses_source_wave_table_values --lib
  --features legacy-tools`, `cargo test
  driver_script_manifest_exports_current_wave_and_spawns --lib --features
  legacy-tools`, `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo test actor_smoke --all-targets --features legacy-tools`, `cargo test
  actor_live --all-targets --features legacy-tools`, `cargo check
  --all-targets --features legacy-tools`, `cargo clippy --all-targets
  --features legacy-tools -- -D warnings`, `make actor-smoke`, and
  `make actor-wgpu-smoke` with `frame_source: actor_game`, `192` actor frames,
  `192` nonblank offscreen frames, and zero temporary raster commands,
  touched-doc markdownlint, and `git diff --check`. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780717157781079`.
- `2026-06-06 04:36 BST`: Completed the actor hostile-family wave-script spawn
  cycle. `ActorWaveProfile` and its manifest now carry clean initial
  `mutant`, `swarmer`, and `baiter` spawn rows alongside existing lander,
  bomber, pod, and human rows; checked text wave scripts parse the same
  directives and `ActorGameDriver` allocates those actors through the
  simulation-step prompt path before applying per-kind spawn-behavior profiles.
  Added parser and driver regressions that prove custom hostile-family spawns
  enter play and receive script-selected drift/speed behavior. README, SPEC,
  and actor architecture docs now describe the full initial-spawn script
  surface. Validation passed with `cargo fmt --check`, `cargo test wave_script
  --lib --features legacy-tools`, `cargo test actor_game --all-targets
  --features legacy-tools`, `cargo test actor_smoke --all-targets --features
  legacy-tools`, `cargo test actor_live --all-targets --features
  legacy-tools`, `make actor-smoke`, and `make actor-wgpu-smoke` with
  `frame_source: actor_game`, `192` actor frames, `192` nonblank offscreen
  frames, and zero temporary raster commands, touched-doc markdownlint, and
  `git diff --check`. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780716542840919`.
- `2026-06-06 04:26 BST`: Completed the actor wave spawn-behavior allocation
  cycle. `ActorGameDriver` now tracks per-kind wave allocation indices across
  the whole wave, so `spawn_behavior <kind> <index> ...` profiles apply not
  only to initial wave spawns but also to source reserve/refill actors,
  pod-created swarmers, baiters, and other command-applied later hostile
  spawns. This keeps the actor rewrite non-frame-driven while making
  custom/level scripts able to tune individual restored actors as the driver
  allocates them. Added a parsed wave-script regression that targets the first
  source wave-2 reserve lander at lander spawn index `3` and proves it receives
  the configured actor-id behavior profile. README, SPEC, and actor
  architecture docs now describe the full-wave spawn-index contract.
  Validation passed with `cargo fmt --check`, the focused reserve-allocation
  regression, the existing initial-spawn behavior regression, the `wave_script`
  lib filter under `legacy-tools`, `cargo test actor_game --all-targets
  --features legacy-tools`, `cargo test actor_smoke --all-targets
  --features legacy-tools`, `cargo test actor_live --all-targets
  --features legacy-tools`, `make actor-smoke`, and `make actor-wgpu-smoke`
  with `frame_source: actor_game`, `192` actor frames, `192` nonblank offscreen
  frames, and zero temporary raster commands, touched-doc markdownlint, and
  `git diff --check`. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780715891123479`.
- `2026-06-06 04:14 BST`: Completed the routine CI actor-smoke hardening
  cycle. `make ci` now routes through `make ci-smoke`, and `make ci-smoke`
  runs both the clean live-smoke path and the actor offscreen `wgpu` smoke
  path. GitHub Actions now runs `xvfb-run -a make ci-smoke`, and the public
  API drift guard asserts the Makefile and workflow wiring so routine CI
  cannot silently fall back to the clean-only smoke path. README and SPEC now
  describe the clean-plus-actor CI smoke boundary. Validation passed with
  `cargo fmt --check`, the focused
  `public_api_tests::clean_runtime_and_oracle_use_quarantined_adapters` guard
  under `legacy-tools`, `make -n ci`, `make -n ci-smoke`, and `make ci-smoke`.
  The actor smoke reported `frame_source: actor_game`, `192` actor frames,
  `192` nonblank offscreen frames, and zero temporary raster commands. Slack
  cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780715421136319`.
- `2026-06-06 04:08 BST`: Completed the owner-review closure-audit sync cycle.
  `docs/fidelity/release-closure-audit.md` now marks the current hardened
  actor-era release gate as locally proven instead of pending a fresh run after
  actor rewrite slices. Its runtime-smoke proof now names `--actor-smoke`,
  `--actor-attract-smoke`, `--actor-post-game-smoke`, `--actor-wgpu-smoke`,
  and `--live-smoke`, including the actor offscreen `wgpu` proof with
  `frame_source: actor_game`, `192` nonblank offscreen frames, and zero
  temporary raster commands. The active PLAN release-validation item now points
  at the `2026-06-06 04:04 BST` hardened actor gate rather than the earlier
  pre-hardening pass, and an older work-log entry now explicitly notes that the
  04:04 gate superseded its gate-rerun requirement. Validation passed with the
  stale gate-requirement scan, touched-doc markdownlint, `git diff --check`,
  and `make owner-review-package` with the accepted `28`-report gate. Slack
  cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780715191729049`.
- `2026-06-06 04:04 BST`: Completed the actor release-gate hardening cycle.
  `make release-gate` now runs the actor replacement play/render proof directly:
  `make actor-smoke` verifies the actor runtime through attract, credit, play,
  gameplay/audio events, sprite layers, native draw commands, and zero
  temporary raster commands, and `make actor-wgpu-smoke` verifies the same
  actor frame source through offscreen `wgpu` readback with
  `frame_source: actor_game`. Added explicit Makefile targets for both smokes
  and a public API drift guard so the CLI/Makefile release boundary cannot drop
  them silently. README and SPEC now list the hardened actor play/offscreen
  smoke gate, and the expanded release-gate command sequence in this plan is
  synchronized with the Makefile. Validation passed with `cargo fmt --check`,
  the focused `public_api_tests::clean_runtime_and_oracle_use_quarantined_adapters`
  guard under `legacy-tools`, `make -n release-gate`, `make actor-smoke`,
  `make actor-wgpu-smoke`, and the full hardened `make release-gate`: default
  all-target tests (`663`), `legacy-tools` all-target tests (`1614` passed,
  `1` ignored), both clippy gates, `make clean-fidelity`, media helper tests,
  owner-review package and the accepted `28`-report gate, MAME doctor/smoke,
  README media, game smoke, actor smoke, actor attract/post-game smoke, actor
  offscreen `wgpu` smoke, live smoke, docs lint, and `git diff --check`. Slack
  cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780714340875129`.
- `2026-06-06 03:50 BST`: Completed the actor-era release-gate revalidation
  cycle. Preflight smoke passed for `cargo run -- --actor-smoke`,
  `cargo run -- --actor-attract-smoke`, `cargo run -- --actor-post-game-smoke`,
  `cargo run -- --live-smoke`, and `cargo run -- --game-smoke`, with clean
  exits, no missing sprite regions, and zero temporary raster commands where
  reported. The full `make release-gate` then passed: default all-target tests
  (`663`), `legacy-tools` all-target tests (`1614` passed, `1` ignored), both
  clippy gates, `make clean-fidelity` across the selected scenarios,
  `make media-script-test`, `make owner-review-package` and the accepted
  `28`-report gate, `make reference-mame-doctor`, `make reference-mame-smoke`,
  `make readme-media`, release smoke targets, docs lint, and `git diff
  --check`. No runtime patch was needed; remaining closure is owner review or a
  new concrete MAME mismatch/input program. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780713702846769`.
- `2026-06-06 03:35 BST`: Completed the actor two-player source-prompt
  proof-hardening cycle. The two-player start/switch regressions now assert the
  full source message-glyph run in the rendered actor scene, including exact
  glyph sprite id, overlay layer, source-derived position, glyph size, and white
  tint for `PLAYER ONE`, `PLAYER TWO`, and `GAME OVER` prompts. The shared
  source-message assertion now checks every visible glyph instead of only the
  first glyph, and the player-switch helper verifies both the switch
  interstitial prompts and the next player's start prompt at the handoff. No
  protected media or legacy code was touched. Validation passed with
  `cargo fmt --check`, `cargo test actor_two_player --lib`,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`, touched
  docs markdownlint, and `git diff --check`. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780712978428889`.
- `2026-06-06 03:24 BST`: Completed the retired `oldsrc/` cleanup cycle. The
  unused pre-clean source snapshot has been removed from the tracked workspace,
  leaving `src_legacy/` as the only intentional legacy boundary for explicit
  `legacy-tools` oracle/evidence builds. A new root architecture guard asserts
  that `oldsrc/` stays absent while `src_legacy/` remains present, so future
  cleanup does not accidentally reintroduce the stale source tree or delete the
  required oracle boundary. No protected media was touched, and no additional
  legacy modules were safe to remove because `src_legacy/` still owns ROM,
  trace, sound-board, and oracle evidence used by the release gate. Slack cycle
  start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780712532505379`.
- `2026-06-06 02:34 BST`: Completed the organic terrain-blow proof-promotion
  cycle. The passing
  `target/reference-media/organic-terrain-blow-smartmix-check/report.json`
  is now part of `docs/fidelity/reference-report-gate.json` as the accepted
  all-axis `organic terrain-blow smartmix` report. The manifest now has a
  dedicated `organic_terrain_blow` coverage facet and raises the relevant
  breadth floors to `28` accepted reports: `21` all-axis, `4` audio-only,
  `3` visual-only, and `20` coverage requirements. The generated signoff
  summary now counts `98` required non-empty media artifacts, `24` accepted
  visual comparisons, and `25` accepted audio comparisons. README, SPEC,
  PLAN, and `docs/fidelity/release-closure-audit.md` now describe the organic
  smartmix terrain-blow clip as accepted evidence rather than an open failing
  probe; the later `2026-06-06 04:04 BST` hardened actor release-gate cycle
  supersedes this entry's gate-rerun requirement, leaving owner review as the
  closure boundary.
  Validation passed with `make reference-report-gate` and
  `make reference-signoff-summary`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780709479376629`.
- `2026-06-06 02:27 BST`: Completed the organic smartmix terminal flash
  parity cycle. The target4 terminal post-game branch now suppresses the
  generic terrain-blow flash across the terminal black phases and overlays
  only the MAME-visible sampled colors at state frames `5990`, `6002`,
  `6014`, `6032`, `6044`, and `6074`. The focused organic regression now
  asserts those clear-color samples alongside the existing score,
  terrain-blow, residual-object, terminal-shell, and sound-cadence checks.
  Regenerated ignored clean media for `organic-terrain-blow-smartmix` and the
  report-only comparator now pass all-axis comparison at
  `target/reference-media/organic-terrain-blow-smartmix-check/report.json`:
  visual RMS `23.11`, visual MAE `3.42`, visual-region RMS `29.01` HUD,
  `32.07` scanner, `15.30` playfield, `19.57` laser band, and `28.36`
  terrain, with audio envelope correlation `0.4431`. No protected media was
  committed, and no legacy code, tests, or scaffolding were safe to remove
  because the remaining fidelity slices still depend on the clean-vs-MAME
  reference tooling. Validation passed with `cargo fmt --check`, `cargo test
  clean_game_organic_smartmix_terminal_death_and_terrain_blow_match_mame_window
  --lib --features legacy-tools`, `cargo test terrain_blow --lib --features
  legacy-tools`, `cargo check --all-targets --features legacy-tools`, `cargo
  clippy --all-targets --features legacy-tools -- -D warnings`, and the
  regenerated report-only organic terrain-blow smartmix media check. Slack
  start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780708681019739`.
- `2026-06-06 02:15 BST`: Completed the armed terrain-blow render guard cycle.
  The clean source renderer and actor render bridge now apply terrain-blow flash
  clear color only after `TerrainBlowSnapshot::terrain_erased()` is true, so the
  armed-but-not-erased source state keeps its neutral clear color and, on the
  source renderer path, continues to draw terrain tiles. The unsafe organic
  `pcram0` flash-table experiment was not kept because it regressed the
  smartmix media gate; the prior source terrain-blow flash cadence remains in
  place. Focused regressions now exercise both render paths at an elapsed frame
  that would otherwise have produced a visible flash. Validation passed with
  `cargo fmt --check`, `cargo test armed_terrain --lib --features
  legacy-tools`, `cargo test terrain_blow --lib --features legacy-tools`,
  `cargo check --features legacy-tools`, `cargo clippy --all-targets
  --features legacy-tools -- -D warnings`, and the regenerated report-only
  organic terrain-blow smartmix media check. That report remains visual-fail at
  RMS `89.11` / MAE `40.82`; audio passes with envelope correlation `0.4431`.
  Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780707473389519`.
- `2026-06-05 22:30 BST`: Completed the actor two-player prompt hardening
  cycle. Focused actor regressions now prove the source `PLYR1`/`PLYR2` plus
  `GO` switch prompt persists across every step of the source `0x60`
  player-switch sleep, clears at the handoff frame, and gives way to the next
  player's source start prompt for the delayed start interval without leaking
  stale `GAME OVER` text into the `WaveStarted` report. README, SPEC, PLAN,
  and actor architecture docs now distinguish that locked source-message
  report/render contract from the remaining MAME media proof and exact
  pixel/timing parity boundary. No runtime code path changed in this slice; the
  implementation work was hardening the actor proof boundary around the existing
  source-message render contract. Validation passed with `cargo fmt --check`,
  `cargo test actor_two_player --lib`, `cargo test actor_game --lib`,
  `cargo check --all-targets --features legacy-tools`, `cargo clippy
  --all-targets --features legacy-tools -- -D warnings`, the actor smoke CLI
  commands (`--actor-smoke`, `--actor-attract-smoke`,
  `--actor-post-game-smoke`, and `--actor-wgpu-smoke`), `markdownlint
  README.md SPEC.md PLAN.md docs/actor-architecture.md
  docs/fidelity/mame-golden-clips.md docs/fidelity/release-closure-audit.md
  assets/sounds/README.md`, and `git diff --check`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780694642820439`.
- `2026-06-05 21:40 BST`: Completed the actor source astronaut human-count
  correction cycle. The actor source astronaut process now uses total live
  human snapshot count, not source-backed human count, for the source
  first-wave inactive-slot suppression rule. This matches the clean source
  routine that suppresses target-list slots `>= 2` while the world still has
  ten live humans, even if some live humans are not source-backed target-list
  entries. Focused tests now cover the mixed source/plain human case in
  addition to selected-slot-only cadence, source process sleep hold, inactive
  slot suppression, high-seed walk/Y-target branch, and low-seed turn/no-Y-step
  branch. README, SPEC, PLAN, and actor architecture docs now spell out that
  the suppression boundary is the total live human count changing from ten. No
  legacy code, tests, or scaffolding were safe to remove in this slice because
  the remaining clean smoke/fidelity/oracle evidence still depends on those
  boundaries. Validation passed with `cargo fmt --check`, `cargo test
  source_human_walk --lib --features legacy-tools`, `cargo test
  source_human_walk_process --lib --features legacy-tools`, `cargo test
  actor_game --all-targets --features legacy-tools`, `cargo check
  --all-targets --features legacy-tools`, `cargo clippy --all-targets
  --features legacy-tools -- -D warnings`, the actor smoke CLI commands
  (`--actor-smoke`, `--actor-attract-smoke`, `--actor-post-game-smoke`, and
  `--actor-wgpu-smoke`), `markdownlint README.md SPEC.md PLAN.md
  docs/actor-architecture.md`, and `git diff --check`. The full unfiltered
  `legacy-tools` suite was not rerun in this cycle; the previously isolated
  clean-game MAME window/post-game audio failures remain outside this slice.
  Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780691740901069`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780692044064429`.
- `2026-06-05 21:33 BST`: Completed the actor source astronaut process
  cadence cycle. The actor driver now owns the source astronaut process
  cursor/sleep cadence and passes one selected target-list slot through
  `StepPrompt`, so only that grounded source-backed human walks on a source
  process tick. Human actors no longer carry their own source-walk sleep state;
  they still apply the source low-seed turn branch, fixed-point X update, and
  terrain-relative Y step when the driver selects their slot. The process uses
  the source 16-entry astronaut cursor wrap and suppresses first-wave inactive
  slots `>= 2` while all ten source humans remain. Focused tests now cover the
  selected-slot-only cadence, source process sleep hold, inactive-slot
  suppression, high-seed walk/Y-target branch, and low-seed turn/no-Y-step
  branch. README, SPEC, PLAN, and actor architecture docs now document the
  driver-owned source astronaut process contract. No legacy code, tests, or
  scaffolding were safe to remove beyond the actor-owned source-walk sleep
  field because the remaining clean smoke/fidelity/oracle evidence still
  depends on those boundaries. Validation passed with `cargo fmt --check`,
  `cargo test source_human_walk --lib --features legacy-tools`, `cargo test
  actor_game --all-targets --features legacy-tools`, `cargo check
  --all-targets --features legacy-tools`, `cargo clippy --all-targets
  --features legacy-tools -- -D warnings`, `cargo test actor_smoke
  --all-targets --features legacy-tools`, `cargo test actor_live
  --all-targets --features legacy-tools`, `cargo test runtime --all-targets
  --features legacy-tools`, `cargo test actor_wgpu --all-targets --features
  legacy-tools`, the actor smoke CLI commands (`--actor-smoke`,
  `--actor-attract-smoke`, `--actor-post-game-smoke`, and
  `--actor-wgpu-smoke`), `markdownlint README.md SPEC.md PLAN.md
  docs/actor-architecture.md`, and `git diff --check`. The full unfiltered
  `legacy-tools` suite was not rerun in this cycle; the previously isolated
  clean-game MAME window/post-game audio failures remain outside this slice.
  Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780691136829819`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780691628648459`.
- `2026-06-05 21:16 BST`: Completed the actor astronaut source-walk cadence
  cycle. Grounded source-backed human actors now consume the driver-provided
  source RNG seed for the Williams astronaut turn branch, step fixed-point X
  through their actor-owned fraction metadata, and nudge Y toward
  terrain-relative source targets when the seed keeps the current walking
  direction. Focused tests now cover both the high-seed walk/Y-target branch
  and the low-seed turn/no-Y-step branch. README, SPEC, PLAN, and actor
  architecture docs now document the actor source-walk contract. No legacy
  code, tests, or scaffolding were safe to remove in this slice because the
  remaining clean smoke/fidelity/oracle evidence still depends on those
  boundaries. Validation passed with `cargo fmt --check`, `cargo test
  source_human_walk --lib --features legacy-tools`, `cargo test actor_game
  --all-targets --features legacy-tools`, `cargo check --all-targets
  --features legacy-tools`, `cargo clippy --all-targets --features
  legacy-tools -- -D warnings`, `cargo test actor_smoke --all-targets
  --features legacy-tools`, `cargo test actor_live --all-targets --features
  legacy-tools`, `cargo test runtime --all-targets --features legacy-tools`,
  `cargo test actor_wgpu --all-targets --features legacy-tools`, the actor
  smoke CLI commands (`--actor-smoke`, `--actor-attract-smoke`,
  `--actor-post-game-smoke`, and `--actor-wgpu-smoke`), `markdownlint
  README.md SPEC.md PLAN.md docs/actor-architecture.md`, and
  `git diff --check`. The full unfiltered `legacy-tools` suite was not rerun
  in this cycle; the previously isolated clean-game MAME window/post-game audio
  failures remain outside this slice. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780690264696859`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780690932977329`.
- `2026-06-05 21:08 BST`: Completed the actor source bomber bomb-shell
  fidelity cycle. Source-backed bomber actors now split the source TIE
  selected-slot picture/velocity/sleep branch from fixed-point movement, so
  only the RNG-selected source slot updates TIE metadata while all
  source-backed bombers still advance position from current velocity. Bomber
  bomb-shell commands now use the source `LSEED & 0x07` drop gate, pre-move
  shell position/fractions, shared source shell cap, ten-bomb shell cap,
  `GETSHL` bounds, and `(SEED & 0x1F) + 1` lifetime ticks; non-source bomber
  actors remain script-cadence driven. README, SPEC, and actor architecture
  docs now document the source TIE slot and bomb-shell contract. No legacy
  code, tests, or scaffolding were safe to remove in this slice because the
  remaining clean smoke/fidelity/oracle evidence still depends on those
  boundaries. Validation passed with `cargo fmt --check`, focused
  `source_bomber_bomb`, `source_bomber`, `source_bomb`, and `source_shell`
  filters, `cargo check --all-targets --features legacy-tools`, `cargo test
  actor_game --all-targets --features legacy-tools`, `cargo test actor_smoke
  --all-targets --features legacy-tools`, `cargo test actor_live --all-targets
  --features legacy-tools`, `cargo test runtime --all-targets --features
  legacy-tools`, `cargo test actor_wgpu --all-targets --features
  legacy-tools`, `cargo clippy --all-targets --features legacy-tools --
  -D warnings`, actor smoke CLI commands (`--actor-smoke`,
  `--actor-attract-smoke`, `--actor-post-game-smoke`, and
  `--actor-wgpu-smoke`), `markdownlint README.md SPEC.md PLAN.md
  docs/actor-architecture.md`, and `git diff --check`. The full unfiltered
  `legacy-tools` suite was not rerun in this cycle; the previously isolated
  clean-game MAME window/post-game audio failures remain outside this slice.
  Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780689105561129`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780690109210469`.
- `2026-06-05 20:49 BST`: Completed the actor astronaut source-audio cadence
  cycle. Actor `HumanReleased` now maps through `ActorSoundEventBridge` to the
  source `ASCSND` command byte `0xE5`, and actor `HumanRescued` queues the
  repeated source `ACSND` tail at the source `+10` and `+20` step offsets after
  the immediate rescue cue. Pending rescue-tail commands are tagged separately
  from smart-bomb and terrain-blow tails so turn/game reset boundaries can
  clear them without disturbing unrelated source command sequences. README,
  SPEC, and actor architecture docs now document actor source release audio and
  repeated rescue audio. No legacy code, tests, or scaffolding were safe to
  remove in this slice because the remaining clean smoke/fidelity/oracle
  evidence still depends on those boundaries. Validation passed with focused
  actor sound/release/rescue tests, `cargo check --all-targets --features
  legacy-tools`, `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo test actor_smoke --all-targets --features legacy-tools`, `cargo test
  actor_live --all-targets --features legacy-tools`, `cargo test actor_wgpu
  --all-targets --features legacy-tools`, `cargo test runtime --all-targets
  --features legacy-tools`, `cargo clippy --all-targets --features
  legacy-tools -- -D warnings`, the actor smoke CLI commands (`--actor-smoke`,
  `--actor-attract-smoke`, `--actor-post-game-smoke`, and
  `--actor-wgpu-smoke`), `cargo fmt --check`, `markdownlint README.md SPEC.md
  PLAN.md docs/actor-architecture.md`, and `git diff --check`. The full
  unfiltered `legacy-tools` suite was not rerun in this cycle; the previously
  isolated clean-game MAME window/post-game audio failures remain outside this
  slice. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780688597112389`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780688984566839`.
- `2026-06-05 20:40 BST`: Completed the actor source terrain and terrain-blow
  cycle. Actor playing reports now publish clean terrain segments and render
  the playfield from source `BGOUT` records while no terrain blow is active.
  Removing the final human starts a driver-owned source `TerrainBlowSnapshot`,
  erases clean terrain and scanner terrain, suppresses the same-batch generic
  human-loss cue, projects the MAME-observed terrain flash windows through the
  render bridge, spawns source-positioned `TEREX` terrain explosion actors with
  the source terrain-explosion growth/lifetime curve, and emits the source
  `AHSND` / `TBSND` sound-command cadence plus tail commands. README, SPEC,
  and actor architecture docs now document the actor terrain bridge and
  last-human terrain-blow path. No legacy code, tests, or scaffolding were safe
  to remove in this slice because the remaining clean smoke/fidelity/oracle
  evidence still depends on those boundaries. Validation passed with
  `cargo fmt --check`, `cargo check --all-targets --features legacy-tools`,
  `cargo test terrain_blow --lib --features legacy-tools`, `cargo test
  actor_game --all-targets --features legacy-tools`, `cargo test actor_smoke
  --all-targets --features legacy-tools`, `cargo test actor_live --all-targets
  --features legacy-tools`, `cargo test runtime --all-targets --features
  legacy-tools`, `cargo test actor_wgpu --all-targets --features
  legacy-tools`, `cargo clippy --all-targets --features legacy-tools --
  -D warnings`, the actor smoke CLI commands (`--actor-smoke`,
  `--actor-attract-smoke`, `--actor-post-game-smoke`, and
  `--actor-wgpu-smoke`), `markdownlint README.md SPEC.md PLAN.md
  docs/actor-architecture.md`, and `git diff --check`. The full unfiltered
  `legacy-tools` suite was not rerun in this cycle; the previously isolated
  clean-game MAME window/post-game audio failures remain outside this slice.
  Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780687368830329`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780688431792669`.
- `2026-06-05 20:20 BST`: Completed the actor source smart-bomb cadence
  cycle. Accepted actor smart bombs now consume driver-owned stock on the
  input report, wait the source three-step detonation delay before clearing
  hostile actors, publish a five-step white flash through
  `StepReport::smart_bomb_flash_steps_remaining` and the render bridge, and
  queue the source `SBSND` / cannon command-byte sequence instead of a generic
  explosion sound. Held smart-bomb input is suppressed while the pending source
  detonation is armed. `XYZZY` overlay smart bombs now use the same delayed
  command path without consuming stock, and source reserve activation is held
  behind the source smart-bomb cooldown so reserves do not refill during the
  detonation flash. README, SPEC, and actor architecture docs now document the
  delayed source smart-bomb path, flash state, sound sequence, and reserve
  cooldown. No legacy code, tests, or scaffolding were safe to remove in this
  slice because the remaining clean smoke/fidelity/oracle evidence still
  depends on those boundaries. Validation passed with `cargo fmt --check`,
  `cargo check --all-targets --features legacy-tools`, `cargo test smart_bomb
  --lib --features legacy-tools`, `cargo test actor_game --all-targets
  --features legacy-tools`, `cargo test actor_smoke --all-targets --features
  legacy-tools`, `cargo test actor_live --all-targets --features
  legacy-tools`, `cargo test runtime --all-targets --features legacy-tools`,
  `cargo test actor_wgpu --all-targets --features legacy-tools`, `cargo clippy
  --all-targets --features legacy-tools -- -D warnings`, the actor smoke CLI
  commands (`--actor-smoke`, `--actor-attract-smoke`,
  `--actor-post-game-smoke`, and `--actor-wgpu-smoke`), `markdownlint
  README.md SPEC.md PLAN.md docs/actor-architecture.md`, and `git diff
  --check`. The full unfiltered `legacy-tools` suite was not rerun in this
  cycle; the previously isolated clean-game MAME window/post-game audio
  failures remain outside this slice. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780686216653429`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780687204889539`.
- `2026-06-05 19:59 BST`: Completed the actor source-background restore
  placement cycle. Actor `StepPrompt` and `StepReport` now carry a
  driver-owned source background word, and `PlayerShip` publishes the
  `SEED/HSEED` background word through `GameCommand::SetSourceBackgroundLeft`
  when source-backed hyperspace rematerialization completes. The command is
  applied inside the driver and intentionally maps to no clean gameplay event.
  Direct mutant reserves and no-human lander reserve schizoid fallback restores
  now pass that source background word into `ActorMutantSpawn::source_restore`
  instead of using fixed zero placement. Focused regressions cover source
  hyperspace background publication, direct mutant reserve placement with a
  nonzero background word, and no-human reserve mutant fallback placement with a
  nonzero background word. README, SPEC, and actor architecture docs now
  document the source-background prompt/report state and restore placement
  usage. No legacy code, tests, or scaffolding were safe to remove in this
  slice because clean smoke/fidelity/oracle evidence still depends on those
  boundaries. Validation passed with `cargo fmt --check`, `cargo check
  --all-targets --features legacy-tools`, `cargo test
  hyperspace_source_seed_controls_rematerialization_position_and_direction
  --lib --features legacy-tools`, `cargo test
  actor_source_mutant_reserves_use_restore_state --lib --features
  legacy-tools`, `cargo test
  actor_source_reserve_landers_without_humans_restore_source_mutants --lib
  --features legacy-tools`, `cargo test actor_game --all-targets --features
  legacy-tools`, `cargo clippy --all-targets --features legacy-tools --
  -D warnings`, the actor-smoke/live/runtime/wgpu all-target `legacy-tools`
  filters, and the actor smoke CLI commands (`--actor-smoke`,
  `--actor-attract-smoke`, `--actor-post-game-smoke`, and
  `--actor-wgpu-smoke`). The full unfiltered `legacy-tools` suite was not
  rerun in this cycle; the previously isolated clean-game MAME window/post-game
  audio failures remain outside this slice. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780685462133509`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780685967347399`.
- `2026-06-05 19:48 BST`: Completed the actor direct mutant reserve
  activation cycle. Actor reserve accounting, empty checks, selection, and
  source-family take logic now include `EnemyReserveSnapshot::mutants`, so
  direct mutant reserve rows can no longer strand wave progression outside the
  actor path. Mixed reserve selection now follows the clean source family order
  after lander priority: bomber, pod, mutant, then swarmer. Direct mutant
  reserves spawn through `ActorMutantSpawn::source_restore`, carrying source
  placement, shot-timer RNG, hop-RNG, and `SourceMutantSnapshot` metadata into
  actor snapshots and clean bridge state. Checked wave scripts keep the
  existing compatible `reserve` / `enemy_reserve` form and add
  `reserve_full` / `enemy_reserve_full` for all five source reserve families.
  Focused regressions cover direct mutant reserve restoration and all-family
  parser manifest output. README, SPEC, and actor architecture docs now
  document direct mutant reserve activation and the new script form. No legacy
  code, tests, or scaffolding were safe to remove in this slice because the
  remaining clean smoke/fidelity/oracle evidence still depends on those
  boundaries. Validation passed with `cargo fmt --check`, `cargo check
  --all-targets --features legacy-tools`, `cargo test
  actor_source_mutant_reserves_use_restore_state --lib --features
  legacy-tools`, `cargo test
  wave_script_text_parser_builds_sorted_profiles_and_spawns --lib --features
  legacy-tools`, `cargo test source_mutant --lib --features legacy-tools`,
  `cargo test actor_game --all-targets --features legacy-tools`, `cargo
  clippy --all-targets --features legacy-tools -- -D warnings`, the
  actor-smoke/live/runtime/wgpu all-target `legacy-tools` filters, and the
  actor smoke CLI commands (`--actor-smoke`, `--actor-attract-smoke`,
  `--actor-post-game-smoke`, and `--actor-wgpu-smoke`). The full unfiltered
  `legacy-tools` suite was not rerun in this cycle; the previously isolated
  clean-game MAME window/post-game audio failures remain outside this slice.
  Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780684737041449`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780685300920039`.
- `2026-06-05 19:34 BST`: Completed the actor source swarmer reserve
  activation cycle. Actor reserve accounting and selection now include
  `EnemyReserveSnapshot::swarmers`, so source swarmer reserves can no longer
  be stranded outside actor reserve activation. Source swarmer reserve batches
  now restore through the actor path with the red-label `PLRES`/`RSW0`
  phony-object placement shape: the batch shares one restored position plus
  source X/Y fractions, then each `Swarmer` actor receives source RNG-derived
  velocity, acceleration, sleep, and shot-timer metadata before entering the
  same source mini-swarmer runtime used by pod destruction. Checked wave
  scripts now accept `reserve <landers> <bombers> <pods> [swarmers]` and
  `enemy_reserve <landers> <bombers> <pods> [swarmers]`, preserving existing
  three-field scripts with a zero swarmer reserve. Focused regressions cover
  source swarmer reserve placement/fractions/RNG advancement and parser
  manifest output. README, SPEC, and actor architecture docs now document
  actor-side source swarmer reserve activation and the optional script field.
  No legacy code, tests, or scaffolding were safe to remove in this slice
  because clean smoke/fidelity/oracle evidence still depends on clean runtime
  boundaries outside the actor path. Validation passed with `cargo fmt
  --check`, `cargo check --all-targets --features legacy-tools`, `cargo test
  source_swarmer --lib --features legacy-tools`, `cargo test
  actor_source_swarmer_reserves_use_plres_restore_state --lib --features
  legacy-tools`, `cargo test
  wave_script_text_parser_builds_sorted_profiles_and_spawns --lib --features
  legacy-tools`, `cargo test actor_game --all-targets --features
  legacy-tools`, `cargo clippy --all-targets --features legacy-tools --
  -D warnings`, the actor-game/smoke/live/runtime/wgpu all-target
  `legacy-tools` filters, and the actor smoke CLI commands (`--actor-smoke`,
  `--actor-attract-smoke`, `--actor-post-game-smoke`, and
  `--actor-wgpu-smoke`). The full unfiltered `legacy-tools` suite was not
  rerun in this cycle; the previously isolated clean-game MAME
  window/post-game audio failures remain outside this slice. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780683723348219`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780684485661219`.
- `2026-06-05 19:19 BST`: Completed the actor source baiter fireball
  projection cycle. Source-backed baiter shots now use the red-label shared
  `SHOOT` fireball setup instead of the generic hostile-shot helper: source
  RNG selects X/Y fireball velocity, high prompt `SEED` adds player X velocity,
  the source actor fractions become shell fractions, and the source 20-tick
  shell lifetime is preserved. The source-backed path now suppresses both
  projectile and `USHSND`/baiter shot sound when source shell placement or the
  shared shell cap rejects allocation, while non-source scripted baiters keep
  their existing behavior-profile helper. Source-backed baiter shot-timer
  resets now use the source `RMAX`-style reset from the same driver-provided
  source RNG snapshot used for fireball projection. Focused regressions cover
  successful source projection, full-shell suppression, and high-seed
  player-velocity contribution. README, SPEC, and actor architecture docs now
  document the actor-side source `SHOOT` projection. No legacy code, tests, or
  scaffolding were safe to remove in this slice because clean
  smoke/fidelity/oracle evidence still depends on clean runtime boundaries
  outside the actor path. Validation passed with `cargo fmt --check`,
  `cargo check --all-targets --features legacy-tools`, `cargo test
  source_baiter --lib --features legacy-tools`, `cargo test source_mutant
  --lib --features legacy-tools`, `cargo test actor_game --all-targets
  --features legacy-tools`, `cargo clippy --all-targets --features
  legacy-tools -- -D warnings`, the actor-game/smoke/live/runtime/wgpu
  all-target `legacy-tools` filters, and the actor smoke CLI commands
  (`--actor-smoke`, `--actor-attract-smoke`, `--actor-post-game-smoke`, and
  `--actor-wgpu-smoke`). The full unfiltered `legacy-tools` suite was not
  rerun in this cycle; the previously isolated clean-game MAME
  window/post-game audio failures remain outside this slice. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780682854897899`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780683539952199`.
- `2026-06-05 19:04 BST`: Completed the actor source mini-swarmer fireball
  projection cycle. Source-backed swarmer shots now use the red-label `SWBMB`
  fireball shape instead of the generic hostile-shot helper: the actor path
  suppresses both projectile and `SWSSND` cue when the swarmer has passed the
  player or the shared source shell list is full, and successful shots emit
  source fireball metadata with X velocity from swarmer `OXV << 3`, Y velocity
  from the player/shell delta, zero shell fractions, and the source 20-tick
  shell lifetime. Non-source scripted swarmers keep the existing behavior
  profile shot helper. Focused regressions now cover successful source
  projection, the pass-player direction gate, and full-shell suppression.
  README, SPEC, and actor architecture docs now document the actor-side
  `SWBMB` projection. No legacy code, tests, or scaffolding were safe to remove
  in this slice because clean smoke/fidelity/oracle evidence still depends on
  clean runtime boundaries outside the actor path. Validation passed with
  `cargo fmt --check`, `cargo check --all-targets --features legacy-tools`,
  `cargo test source_swarmer --lib --features legacy-tools`, `cargo test
  actor_game --all-targets --features legacy-tools`, `cargo clippy
  --all-targets --features legacy-tools -- -D warnings`, the
  actor-game/smoke/live/runtime/wgpu all-target `legacy-tools` filters, the
  actor smoke CLI commands (`--actor-smoke`, `--actor-attract-smoke`,
  `--actor-post-game-smoke`, and `--actor-wgpu-smoke`), touched-doc
  markdownlint, and `git diff --check`. The full unfiltered `legacy-tools`
  suite was not rerun in this cycle; the previously isolated clean-game MAME
  window/post-game audio failures remain outside this slice. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780682222928479`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780682658802129`.
- `2026-06-05 18:50 BST`: Completed the actor source mini-swarmer RNG-state
  cycle. Pod laser kills now seed the bounded mini-swarmer actor batch from the
  driver source RNG for initial X/Y velocity, acceleration, sleep ticks, and
  shot-timer metadata instead of deriving spread from the local spawn index.
  Source-backed swarmers carry the clean bridge's horizontal-seek-pending flag,
  perform the source entry horizontal seek on their first prompt, then advance
  actor-owned fixed-point fractions with source-shaped vertical
  acceleration/damping, turn-window reseek, and `RMAX` shot-timer resets before
  emitting swarmer projectile commands and the swarmer shot cue. The pod
  collision regression now verifies source RNG consumption across the full
  swarmer request batch, and the swarmer shot regression calculates expected
  movement, timer reset, projectile velocity, and source projectile metadata
  from the same source helpers. README, SPEC, and actor architecture docs now
  document the actor-side source mini-swarmer shape. No legacy code, tests, or
  scaffolding were safe to remove in this slice because clean
  smoke/fidelity/oracle evidence still depends on clean runtime boundaries
  outside the actor path. Validation passed with `cargo fmt --check`,
  `cargo check --all-targets --features legacy-tools`, focused source-swarmer
  and pod-collision regressions, `cargo test actor_game --all-targets
  --features legacy-tools`, `cargo clippy --all-targets --features
  legacy-tools -- -D warnings`, the actor-game/smoke/live/runtime/wgpu
  all-target `legacy-tools` filters, the actor smoke CLI commands
  (`--actor-smoke`, `--actor-attract-smoke`, `--actor-post-game-smoke`, and
  `--actor-wgpu-smoke`), touched-doc markdownlint, and `git diff --check`. The
  full unfiltered `legacy-tools` suite was not rerun in this cycle; the
  previously isolated clean-game MAME window/post-game audio failures remain
  outside this slice. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780680988590699`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780681987628239`.
- `2026-06-05 18:29 BST`: Completed the actor no-human reserve mutant
  fallback cycle. Source reserve lander rows now follow the red-label
  no-human schizoid fallback in the actor driver: when no source human target
  remains, adjacent lander reserve rows restore as source-shaped mutant actors
  with source placement, shot-timer, and hop-RNG metadata instead of becoming
  targetless landers. Pre-prompt reserve spawn events remain visible in
  `StepReport::commands` for the clean event bridge, but the driver no longer
  replays those already-applied spawns through end-of-step command application,
  preventing duplicate generic actors on the following report. Added a focused
  regression covering source-mutant reserve restore state, command shape,
  reserve depletion, and the duplicate-spawn guard. README, SPEC, and the actor
  architecture notes now document the actor-side fallback. No legacy code,
  tests, or scaffolding were safe to remove in this slice because clean
  smoke/fidelity/oracle evidence still depends on clean runtime boundaries
  outside the actor path. Validation passed with focused source-reserve tests,
  the actor-game/smoke/live/runtime/wgpu all-target `legacy-tools` filters,
  `cargo fmt --check`, `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`, the
  actor smoke CLI commands (`--actor-smoke`, `--actor-attract-smoke`,
  `--actor-post-game-smoke`, and `--actor-wgpu-smoke`), touched-doc
  markdownlint, and `git diff --check`. The full unfiltered `legacy-tools`
  suite was not rerun in this cycle; the previously isolated clean-game MAME
  window/post-game audio failures remain outside this slice. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780680244424649`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780680796659879`.
- `2026-06-05 16:40 BST`: Completed the actor two-player session-state
  slice. Actor reports now carry driver-owned `current_player`, `player_count`,
  per-player scores, and per-player stock snapshots. `ActorStateBridge` maps
  those values into clean `GameState` instead of hard-coding one-player state.
  `StartTwoPlayer` now requires two credits, consumes both, initializes player
  one as active in a two-player actor session, and publishes both players'
  zeroed scores plus `3` lives / `3` smart bombs. Blocked two-player start
  requests stay in attract and no longer emit false clean `GameStarted` /
  `WaveStarted` events. Score and replay-bonus awards now route through the
  active-player score/stock fields, including player two when selected by the
  driver. The status actor can draw a `2UP` line for two-player sessions. Full
  source `PLE02` switch sleep, player-start prompt, and two-player death
  rotation remain the next two-player fidelity boundary. No legacy code, tests,
  or scaffolding were safe to remove because legacy tooling still backs ROM
  reports, trace/media helpers, and oracle-equivalence evidence while the actor
  runtime continues closing fidelity gaps. Validation passed with
  `cargo test actor_two_player --lib --features legacy-tools`, `cargo test
  actor_score_awards_follow_current_player_two_stock --lib --features
  legacy-tools`, `cargo test actor_game --all-targets --features
  legacy-tools`, `cargo test actor_smoke --all-targets --features
  legacy-tools`, `cargo test actor_live --all-targets --features
  legacy-tools`, `cargo test actor_wgpu --all-targets --features
  legacy-tools`, `cargo run -- --actor-smoke`, `cargo run
  -- --actor-attract-smoke`, `cargo run -- --actor-post-game-smoke`, `cargo run
  -- --actor-wgpu-smoke`, `cargo test runtime --all-targets --features
  legacy-tools`, `cargo check --all-targets --features legacy-tools`, `cargo
  clippy --all-targets --features legacy-tools -- -D warnings`, `cargo fmt
  --check`, touched-doc markdownlint, and `git diff --check`. Slack cycle
  start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780673244065439`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780674065277969`.
- `2026-06-05 15:59 BST`: Completed the actor wave-clear interstitial and
  replay-bonus scoring slice. Actor scoring now uses the clean replay-bonus
  threshold model, carries `next_bonus` into bridged `GameState`, adds
  life/smart-bomb stock on threshold crossings, and emits `BonusAwarded`.
  Source-counted wave clear now publishes a distinct `WaveCleared`
  interstitial report before the next wave starts. That report keeps surviving
  human snapshots visible and projects source `ATTACK WAVE`, `COMPLETED`,
  `BONUS X`, wave/multiplier digits, and survivor bonus icons through
  `ActorRenderSceneBridge`; the following actor step clears transient
  playfield actors, installs the next wave script, spawns the next wave actors,
  and emits `WaveStarted`. No legacy code, tests, or scaffolding were safe to
  remove because legacy tooling still backs ROM reports, trace/media helpers,
  and oracle-equivalence evidence while the actor runtime continues closing
  fidelity gaps. Validation passed with `cargo test actor_wave_clear --lib
  --features legacy-tools`, `cargo test actor_score_awards --lib --features
  legacy-tools`, `cargo test actor_game --all-targets --features
  legacy-tools`, `cargo test actor_smoke --all-targets --features
  legacy-tools`, `cargo run -- --actor-smoke`, `cargo run
  -- --actor-attract-smoke`, `cargo run -- --actor-post-game-smoke`, `cargo
  test runtime --all-targets --features legacy-tools`, `cargo test actor_live
  --all-targets --features legacy-tools`, `cargo test actor_wgpu --all-targets
  --features legacy-tools`, `cargo check --all-targets --features
  legacy-tools`, `cargo clippy --all-targets --features legacy-tools --
  -D warnings`, `cargo fmt --check`, touched-doc markdownlint, and
  `git diff --check`. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780670549435699`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780671539676689`.
- `2026-06-05 15:40 BST`: Completed the actor post-game Hall-return smoke
  gate slice. Actor high-score initials submission now emits
  accepted/submitted clean events, enters a finite 60-step Hall-of-Fame
  `GameOverSnapshot`, draws the source Hall page during the stall, and returns
  to the Williams attract reveal. Added `--actor-post-game-smoke` and
  `make actor-post-game-smoke`; `make release-gate` now includes the new
  actor post-game smoke target. The new smoke report starts actor play, forces
  three actor-owned pod/player collisions for a 3,000-point qualifying score,
  submits initials, verifies 60 GameOver Hall frames, clean game-over and
  high-score events, bridged game-over sound, native draw-plan coverage, sprite
  atlas coverage, and attract return. No legacy code, tests, or scaffolding
  were safe to remove because legacy tooling still backs ROM reports,
  trace/media helpers, and oracle-equivalence evidence while the actor runtime
  continues closing fidelity gaps. Validation passed with `cargo test
  high_score_entry_accepts_initials_and_backspace_from_actor_input --lib
  --features legacy-tools`, `cargo test actor_smoke --all-targets --features
  legacy-tools`, `cargo run -- --actor-post-game-smoke`, `cargo test runtime
  --all-targets --features legacy-tools`, `cargo test platform --all-targets
  --features legacy-tools`, `make actor-post-game-smoke`, `cargo test
  actor_game --all-targets --features legacy-tools`, `cargo test actor_live
  --all-targets --features legacy-tools`, `cargo test actor_wgpu --all-targets
  --features legacy-tools`, `cargo check --all-targets --features
  legacy-tools`, `cargo clippy --all-targets --features legacy-tools --
  -D warnings`, `cargo fmt --check`, touched-doc markdownlint, and
  `git diff --check`. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780669131121529`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780670433685769`.
- `2026-06-05 15:16 BST`: Completed the actor attract-cycle smoke gate
  slice. Added `--actor-attract-smoke` and `make actor-attract-smoke`, wiring
  the command through the clean platform/runtime boundary and into
  `make release-gate`. The new report advances the default no-input actor
  attract script through the full `cycle 3367` path and proves Williams reveal,
  Defender coalescence, Hall of Fame, scoring surface, final scoring label,
  cycle return, native draw-plan coverage, zero non-attract frames, and zero
  actor gameplay/audio events. Focused regressions now cover the report
  milestones, validation failures, CLI classification/errors, runtime dispatch,
  installed backend execution, help text, and the Make target. README, SPEC,
  PLAN, and the actor architecture notes document the new command and release
  gate. No legacy code, tests, or scaffolding were safe to remove because
  legacy tooling still backs ROM reports, trace/media helpers, and
  oracle-equivalence evidence while the actor runtime continues closing
  fidelity gaps. Validation passed with `cargo test actor_attract
  --all-targets --features legacy-tools`, `cargo run -- --actor-attract-smoke`,
  `cargo test actor_smoke --all-targets --features legacy-tools`, `cargo test
  runtime --all-targets --features legacy-tools`, `cargo test platform
  --all-targets --features legacy-tools`, `cargo test actor_live --all-targets
  --features legacy-tools`, `cargo test actor_wgpu --all-targets --features
  legacy-tools`, `make actor-attract-smoke`, `cargo check --all-targets
  --features legacy-tools`, `cargo clippy --all-targets --features
  legacy-tools -- -D warnings`, `cargo fmt --check`, touched-doc markdownlint,
  and `git diff --check`. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780668322381529`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780668964948859`.
- `2026-06-05 15:03 BST`: Completed the actor attract default-cycle slice.
  `AttractScript` now exposes an optional script-level cycle count in its
  manifest, the checked parser accepts `cycle`, `loop`, or `repeat`
  directives with positive step counts, and custom attract scripts remain
  linear unless they opt in. The embedded red-label attract script declares
  `cycle 3367`, matching the source scoring-demo boundary and returning the
  default actor attract sequence to the first Williams reveal step after the
  scoring page. Focused regressions now prove checked-script/fallback parity,
  parser manifest exposure, parser errors for zero and duplicate cycle
  directives, default wrap behavior, and the custom-script opt-in rule. README,
  SPEC, the actor architecture notes, and the red-label asset README now
  document the cycle directive and default boundary. No legacy code, tests, or
  scaffolding were safe to remove because legacy tooling still backs ROM
  reports, trace/media helpers, and oracle-equivalence evidence while the
  actor runtime continues closing fidelity gaps. Validation passed with the
  focused default-loop/title/parser tests, `cargo test actor_game
  --all-targets --features legacy-tools`, `cargo test actor_smoke
  --all-targets --features legacy-tools`, `cargo test actor_live --all-targets
  --features legacy-tools`, `cargo test actor_wgpu --all-targets --features
  legacy-tools`, `cargo check --all-targets --features legacy-tools`, `cargo
  clippy --all-targets --features legacy-tools -- -D warnings`, `cargo fmt
  --check`, touched-doc markdownlint, and `git diff --check`. Slack cycle
  start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780667628613419`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780668195168989`.
- `2026-06-06 03:18 BST`: Completed the smartmix shell scoping and
  release-gate repair cycle. The clean runtime now passes a driver-owned
  `smartmix_shell_overrides` context into enemy advancement so organic
  smartmix target-1 and target-4 special shell paths only arm while thrust is
  held in the matching driver trace. Added a small target-4 smartmix late-shell
  runtime latch so the organic collision/death branch still matches after the
  projected shell has aged out, while hold-up, long no-input, and down29 traces
  no longer false-match by frame/player state. Removed a legacy-machine
  comparison from the scanned clean game module, switched actor smoke to the
  neutral `AttractScript::arcade_title` alias, updated scanner mini-terrain HUD
  expectations, and refreshed the deterministic `wgpu` live-smoke final
  signature to `fe80cc2b377e8066`. Validation passed with the full
  `make release-gate`: `cargo fmt --check`, default all-target tests (`662`
  passed), `legacy-tools` all-target tests (`1613` passed, `1` ignored), both
  clippy gates, selected clean-fidelity scenarios, media helper tests,
  owner-review package and accepted report gate (`28` reports / `20` coverage
  requirements), MAME doctor and smoke capture, README media generation, game
  smoke, actor attract/post-game smoke, live `wgpu` smoke, docs lint, and
  `git diff --check`. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780709902502279`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780712327796219`.
- `2026-06-05 14:50 BST`: Completed the actor attract scoring
  instruction-label cadence slice. The embedded actor attract script and Rust
  fallback keep `scoring_surface` at source step `1088`, reveal `SCANV`
  immediately, then reveal `LANDV`, `MUTV`, `BAITV`, `BOMBV`, `SWRMPV`, and
  `SWARMV` at source scoring-card process steps `1505`, `1691`, `1871`,
  `2051`, `2237`, and `2417` without adding a display-frame dependency.
  Focused regressions now prove checked-script/fallback parity, the exact
  reveal-step table, only `SCANV` at the scoring boundary, `LANDV` at its
  reveal step, and all instruction labels by the final reveal. README, SPEC,
  and the actor architecture notes now document the cadence and script lines.
  No legacy code, tests, or scaffolding were safe to remove because legacy
  tooling still backs ROM reports, trace/media helpers, and oracle-equivalence
  evidence while the actor runtime continues closing fidelity gaps. Validation
  passed with the focused cadence/script/title tests, `cargo test actor_game
  --all-targets --features legacy-tools`, `cargo test actor_smoke
  --all-targets --features legacy-tools`, `cargo test actor_live --all-targets
  --features legacy-tools`, `cargo test actor_wgpu --all-targets --features
  legacy-tools`, `cargo check --all-targets --features legacy-tools`, `cargo
  clippy --all-targets --features legacy-tools -- -D warnings`, `cargo fmt
  --check`, touched-doc markdownlint, and `git diff --check`. Slack cycle
  start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780667014030819`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780667420301369`.
- `2026-06-05 14:37 BST`: Completed the actor attract scoring laser/explosion/
  materialization slice. `VisualEffect::AttractScoringSurface` now uses
  actor-local scoring stages to project source-shaped rescue laser pixels,
  lander explosion fragments, and `ENMYTB` enemy legend
  transfer/materialization/reveal objects alongside the existing scoring
  labels, scanner frame, `MTERR` terrain, rescue objects/blips, and
  500-point rescue popup. Added actor-local source object-image decoding for
  the six legend enemy families and a focused regression proving rescue laser
  pixels, rescue explosion fragments, the 250-point legend transfer popup, and
  the first lander score-card reveal. README, SPEC, and the actor architecture
  notes now document the scoring-surface effect coverage. No legacy code,
  tests, or scaffolding were safe to remove because legacy tooling still backs
  ROM reports, trace/media helpers, and oracle-equivalence evidence while the
  actor runtime continues closing fidelity gaps. Validation passed with the
  focused attract scoring regression,
  `cargo test attract_title_uses_williams_animation_and_defender_coalescence
  --lib --features legacy-tools`, `cargo test actor_game --all-targets
  --features legacy-tools`, `cargo test actor_smoke --all-targets --features
  legacy-tools`, `cargo test actor_live --all-targets --features
  legacy-tools`, `cargo check --all-targets --features legacy-tools`, `cargo
  clippy --all-targets --features legacy-tools -- -D warnings`, `cargo fmt
  --check`, touched-doc markdownlint, and `git diff --check`.
  Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780665733446049`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780666654723459`.
- `2026-06-05 14:17 BST`: Completed the actor attract scoring-demo object
  projection slice. `VisualEffect::AttractScoringSurface` now uses actor-local
  `scoring_tick` to project the source-shaped rescue-demo player, human, and
  lander objects, matching scanner blips, and the 500-point rescue popup on top
  of the scripted scoring labels, scanner frame, and `MTERR` terrain. The slice
  deliberately leaves laser beams, explosion fragments, and enemy legend
  materialization for later scoring-surface cycles while keeping the current
  projection script-driven rather than display-frame driven. Focused attract
  regressions now prove the start-of-scoring object/blip scene and later
  rescue-score popup. Validation passed with `cargo test actor_game
  --all-targets --features legacy-tools`, `cargo test actor_smoke --all-targets
  --features legacy-tools`, `cargo check --all-targets --features
  legacy-tools`, `cargo clippy --all-targets --features legacy-tools --
  -D warnings`, `cargo fmt --check`, touched-doc markdownlint, and `git diff
  --check`. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780664718166739`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780665412945699`.
- `2026-06-05 14:02 BST`: Completed the actor attract scoring-scanner
  surface slice. The embedded actor attract script and Rust fallback now install
  a scriptable `scoring_surface` action at step `1088`, and source-message
  actions accept optional visual offsets so the Hall-of-Fame and
  scoring/instruction rows keep their source screen addresses while matching the
  protected visual placement. `ActorRenderSceneBridge` now projects that
  actor-local scoring surface into source-shaped top scanner frame/marker bars
  plus checked `MTERR` mini-terrain records, without making attract sequencing
  display-frame driven. Focused regressions now prove the embedded manifest,
  custom parser action, source-message offset rendering, source scanner terrain
  slice, and scoring-page scene sprites. Validation passed with `cargo test
  actor_game --all-targets --features legacy-tools`, `cargo test actor_smoke
  --all-targets --features legacy-tools`, `cargo check --all-targets --features
  legacy-tools`, `cargo clippy --all-targets --features legacy-tools --
  -D warnings`, `cargo fmt --check`, touched-doc markdownlint, and `git diff
  --check`. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780663667461809`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780664554421689`.
- `2026-06-05 13:44 BST`: Completed the actor attract
  scoring/instruction text slice. The embedded actor attract script and Rust
  fallback now bound the title and Hall-of-Fame surfaces to their source
  windows: Williams `1-487`, `ELECV` `236-487`, Defender wordmark `365-487`,
  and Hall-of-Fame display rows `488-1087`. The default script now starts the
  scoring/instruction text page at step `1088`, drawing checked
  `messages.tsv` labels `SCANV`, `LANDV`, `MUTV`, `BAITV`, `BOMBV`, `SWRMPV`,
  and `SWARMV` at the source screen addresses. Focused regressions now prove
  the Defender wordmark settles before the Hall boundary, the Hall rows drop
  out at the scoring boundary, and the instruction message glyphs render from
  the source-message path. Validation passed with focused attract/parser tests,
  `cargo test actor_game --all-targets --features legacy-tools`, `cargo test
  actor_smoke --all-targets --features legacy-tools`, `cargo check
  --all-targets --features legacy-tools`, `cargo clippy --all-targets
  --features legacy-tools -- -D warnings`, `cargo fmt --check`, touched-doc
  markdownlint, and `git diff --check`. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780662977164499`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780663511930569`.
- `2026-06-05 13:32 BST`: Completed the actor Hall-of-Fame table fidelity
  slice. The embedded actor attract script and Rust fallback now replace the
  generic one-column default high-score list with a source-shaped
  `hall_scores` action at step `488`, drawing Today’s and All-Time
  Hall-of-Fame columns from clean source screen addresses plus a checked visual
  offset. The table uses driver high-score values with embedded red-label seed
  initials from `assets/red-label/high-scores.tsv`, while the older
  one-column `high_scores` action remains available for custom attract
  scripts. The parser, action manifest, render bridge tests, checked script,
  and docs now cover the new table action. Validation passed with focused
  attract draw/render/manifest tests, `cargo test actor_game --all-targets
  --features legacy-tools`, `cargo test actor_smoke --all-targets --features
  legacy-tools`, `cargo check --all-targets --features legacy-tools`, `cargo
  clippy --all-targets --features legacy-tools -- -D warnings`, `cargo fmt
  --check`, touched-doc markdownlint, and `git diff --check`. Slack cycle
  start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780661754103879`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780662760239339`.
- `2026-06-05 13:14 BST`: Completed the actor Hall-of-Fame heading fidelity
  slice. The embedded actor attract script and Rust fallback now draw the
  source Hall-of-Fame display headings at step `488` through checked
  `messages.tsv` labels: `HALLD_TITLE`, `HALLD_TODAYS`, `HALLD_ALL_TIME`, and
  both `HALLD_GREATEST` positions. The same Hall-of-Fame page now draws the
  source Defender logo at the clean protected-reference position, while the
  existing scriptable high-score rows remain as the actor table fallback for
  the next table-layout slice. Focused regressions prove source heading draw
  commands, duplicate `GREATEST` headings, logo draw placement, rendered source
  glyphs, rendered logo placement, and checked-script/fallback manifest parity.
  Validation passed with `cargo fmt --check`, focused actor attract
  draw/render/manifest tests, `cargo test actor_game --all-targets --features
  legacy-tools`, `cargo test actor_smoke --all-targets --features
  legacy-tools`, `cargo check --all-targets --features legacy-tools`, `cargo
  clippy --all-targets --features legacy-tools -- -D warnings`, touched-doc
  markdownlint, and `git diff --check`. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780661286211789`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780661658481039`.
- `2026-06-05 13:06 BST`: Completed the actor attract Hall-of-Fame timing
  slice. The embedded actor attract script and Rust fallback now keep the
  high-score title/table and zero-credit credit row off the
  Williams/presents/Defender title pages, then draw them from the clean
  Hall-of-Fame boundary at source step `488`. Added a checked
  `credits_nonzero` attract script action so title pages can still show an
  inserted credit immediately while suppressing the zero-credit line. The
  attract manifest now exposes the credit action's minimum-credit threshold for
  custom-driver inspection. Focused tests prove title-page suppression,
  inserted-credit display, Hall-of-Fame-boundary table/credit display,
  checked-script fallback parity, parser/manifest support, and render-bridge
  glyph timing. Validation passed with `cargo fmt --check`, focused default
  attract/parser/credit/render tests, `cargo test actor_game --all-targets
  --features legacy-tools`, `cargo test actor_smoke --all-targets --features
  legacy-tools`, `cargo check --all-targets --features legacy-tools`, `cargo
  clippy --all-targets --features legacy-tools -- -D warnings`, touched-doc
  markdownlint, and `git diff --check`. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780660579625589`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780661156387059`.
- `2026-06-05 12:54 BST`: Completed the actor attract source timing slice.
  The embedded `assets/red-label/actor-attract.script` and the Rust fallback
  constructor now use the clean source title page-start boundaries: Williams
  reveal from step `1`, source `ELECV` presents text at `0x3258` from step
  `236`, and Defender wordmark coalescence from step `365`. Focused attract
  regressions now prove the early Williams page does not draw `ELECV` or the
  Defender coalescence, the source-timed presents page renders the controlled
  `ELECV` message glyphs at the expected overlay position, and the embedded
  checked script remains manifest-equivalent to the Rust constructor fallback.
  Validation passed with `cargo fmt --check`, focused embedded/default
  attract/source-message tests, `cargo test actor_game --all-targets --features
  legacy-tools`, `cargo test actor_smoke --all-targets --features
  legacy-tools`, `cargo check --all-targets --features legacy-tools`, `cargo
  clippy --all-targets --features legacy-tools -- -D warnings`, touched-doc
  markdownlint, and `git diff --check`. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780660000181639`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780660471869419`.
- `2026-06-05 12:44 BST`: Completed the actor default `ELECV` attract slice.
  The embedded `assets/red-label/actor-attract.script` now draws the red-label
  `ELECV` `ELECTRONICS INC. / PRESENTS` source message at source screen address
  `0x3258` during the default actor title sequence. The Rust fallback
  constructor stays manifest-equivalent to the checked script through named
  `ELECV` label/address constants. Extended focused attract regressions prove
  the default actor title report carries `VisualEffect::SourceMessage` for
  `ELECV` and renders controlled source glyphs at the expected overlay
  position. Validation passed with `cargo fmt --check`, focused default
  attract/source-message tests, `cargo test actor_game --all-targets --features
  legacy-tools`, `cargo test actor_smoke --all-targets --features
  legacy-tools`, `cargo check --all-targets --features legacy-tools`, `cargo
  clippy --all-targets --features legacy-tools -- -D warnings`, touched-doc
  markdownlint, and `git diff --check`. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780659558377039`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780659869307209`.
- `2026-06-05 12:37 BST`: Completed the actor attract source-message script
  slice. `AttractScript` now accepts checked
  `message <start> <duration> <label> <top-left-screen-address>` and
  `source_message ...` lines, validates labels against
  `assets/red-label/messages.tsv`, and exposes source-message actions in
  attract manifests. Actor draw commands now carry `VisualEffect::SourceMessage`
  metadata with the source top-left screen address, and
  `ActorRenderSceneBridge` routes those draws through
  `push_source_controlled_message_sprites` so red-label `[RLF]` and
  `[HMC:...]` cursor controls affect glyph placement instead of becoming
  visible text. Added focused parser/render regressions proving parsed `ELECV`
  renders through the controlled source glyph path at the expected positions.
  Validation passed with `cargo fmt --check`, focused source-message/parser
  tests, `cargo test actor_game --all-targets --features legacy-tools`, `cargo
  test actor_smoke --all-targets --features legacy-tools`, `cargo check
  --all-targets --features legacy-tools`, `cargo clippy --all-targets
  --features legacy-tools -- -D warnings`, touched-doc markdownlint, and `git
  diff --check`. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780659006168409`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780659448333319`.
- `2026-06-05 12:27 BST`: Completed the actor source-message credits slice.
  Actor attract credits now resolve the label through the checked red-label
  message table `CREDV` via `source_message_text` instead of relying on an
  actor-side literal. The checked `credits` script action and prompt-backed
  credit count behavior remain unchanged, while docs now name `CREDV` as the
  source for the label. Added a focused regression comparing the attract credit
  label draw against `source_message_text("CREDV")`. Validation passed with
  `cargo fmt --check`, `cargo test
  parsed_attract_script_draws_prompt_credit_count --lib`, `cargo test
  default_sprite_atlas_uses_message_glyph_regions --lib`, `cargo test
  actor_game --all-targets --features legacy-tools`, `cargo check
  --all-targets --features legacy-tools`, `cargo clippy --all-targets
  --features legacy-tools -- -D warnings`, touched-doc markdownlint, and `git
  diff --check`. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780658570631689`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780658874724369`.
- `2026-06-05 12:21 BST`: Completed the actor attract credits script slice.
  `AttractScript` now accepts checked
  `credits <start> <duration> <label-x> <label-y> <count-x> <count-y>` lines,
  exposes credit events in manifests, and draws the source-backed `CREDITS:`
  label plus visible credit count from `StepPrompt.credits`. The actor driver
  now includes current-step coin insertions in prompt credits so attract/status
  text matches the report credit count while durable credit mutation and credit
  audio remain owned by `GameCommand::Credit`. The embedded
  `assets/red-label/actor-attract.script` now owns the default credits
  label/count beside Williams reveal, Defender wordmark coalescence, and
  high-score rows. Added focused parser/runtime regressions for parsed credit
  scripts and embedded default manifest parity. Validation passed with `cargo
  fmt --check`, focused attract parser/runtime tests, `cargo test actor_game
  --all-targets --features legacy-tools`, `cargo test actor_live --all-targets
  --features legacy-tools`, `cargo test actor_smoke --all-targets --features
  legacy-tools`, `cargo check --all-targets --features legacy-tools`, `cargo
  clippy --all-targets --features legacy-tools -- -D warnings`, touched-doc
  markdownlint, and `git diff --check`. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780657867078129`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780658478228899`.
- `2026-06-05 12:09 BST`: Completed the actor attract high-score script slice.
  `AttractScript` now accepts checked
  `high_scores <start> <duration> <x> <y> <row-height> <rows>` lines, exposes
  high-score-table events in manifests, and draws rows from
  `StepPrompt.high_scores` through the script actor instead of baking static
  score text into the attract sequence. The embedded
  `assets/red-label/actor-attract.script` now owns the default high-score table
  rows alongside the Williams reveal and Defender wordmark coalescence. Added
  focused parser/runtime regressions proving parsed custom scripts draw updated
  driver high-score rows and the default attract sequence still renders source
  title/coalescence metadata. Validation passed with `cargo fmt --check`,
  focused attract parser/runtime tests, `cargo test actor_game --all-targets
  --features legacy-tools`, `cargo test actor_live --all-targets
  --features legacy-tools`, `cargo test actor_smoke --all-targets
  --features legacy-tools`, `cargo check --all-targets --features
  legacy-tools`, `cargo clippy --all-targets --features legacy-tools -- -D
  warnings`, touched-doc markdownlint, and `git diff --check`. Slack cycle
  start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780657339814109`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780657754060619`.
- `2026-06-05 12:00 BST`: Completed the per-spawn actor behavior script
  slice. `ActorWaveScript` now accepts
  `spawn_behavior <kind> <index> <field> <value>` checked text lines, carries
  those spawn-index profiles through wave manifests, and installs them as
  actor-id behavior profiles immediately after the driver allocates wave
  lander, bomber, pod, or human actors. This lets level scripts configure the
  movement and behavior of individual spawned actors before their runtime ids
  exist, while keeping the actor runtime simulation-step driven. Added focused
  parser and runtime regressions proving a parsed wave script can make one
  spawned lander chase the player while the next lander stays on the wave
  kind-level drift profile. Validation passed with `cargo fmt --check`,
  focused spawn-behavior and wave-parser tests, `cargo test actor_game
  --all-targets --features legacy-tools`, `cargo test actor_live --all-targets
  --features legacy-tools`, `cargo test actor_smoke --all-targets
  --features legacy-tools`, `cargo check --all-targets --features
  legacy-tools`, `cargo clippy --all-targets --features legacy-tools -- -D
  warnings`, touched-doc markdownlint, and `git diff --check`. Slack cycle
  start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780656893771729`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780657233038729`.
- `2026-06-05 11:52 BST`: Completed the embedded actor default-script slice.
  Added `assets/red-label/actor-attract.script`,
  `assets/red-label/actor-behavior.script`, and
  `assets/red-label/actor-waves.script` as production embedded checked text
  assets. `AttractScript::red_label_title`,
  `ActorBehaviorScript::default`, and `ActorWaveScript::default_progression`
  now parse those assets through the same checked parsers that custom drivers
  use, while Rust constructor/source-table fallbacks remain available for
  focused tests and generated source-backed wave profiles. Added
  `source_wave` / `source_waves` parser directives so checked wave scripts can
  expand source-backed wave-table profiles without copying generated restore
  rows into text. Added focused regressions proving the embedded
  attract/behavior/wave assets match the fallback manifests and that the
  default driver exposes those parsed script manifests. Validation passed with
  `cargo fmt --check`, focused embedded-script and wave-parser tests,
  `cargo test actor_game --all-targets --features legacy-tools`, `cargo test
  actor_live --all-targets --features legacy-tools`, `cargo test actor_smoke
  --all-targets --features legacy-tools`, `cargo check --all-targets
  --features legacy-tools`, `cargo clippy --all-targets --features
  legacy-tools -- -D warnings`, touched-doc markdownlint, and `git diff
  --check`. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780656289569319`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780656786767659`.
- `2026-06-05 11:42 BST`: Completed the actor wave-script text parser slice.
  `ActorWaveScript::parse_text` and `str::parse::<ActorWaveScript>()` now
  accept checked text records for named wave scripts, per-wave behavior updates,
  and lander, bomber, pod, and human spawns. Parsed scripts reuse
  `ActorBehaviorScript` profile updates, sort wave profiles, reject duplicate
  waves, and report line-numbered parse errors. Added focused parser and
  runtime regressions proving parsed wave progression drives wave-one chasing
  and wave-two drift behavior through the actor driver. Validation passed with
  `cargo fmt --check`, focused wave parser/runtime tests, `cargo test
  actor_game --all-targets --features legacy-tools`, `cargo test actor_live
  --all-targets --features legacy-tools`, `cargo test actor_smoke
  --all-targets --features legacy-tools`, `cargo check --all-targets
  --features legacy-tools`, `cargo clippy --all-targets --features
  legacy-tools -- -D warnings`, touched-doc markdownlint, and `git diff
  --check`. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780655725820319`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780656162052869`.
- `2026-06-05 11:33 BST`: Completed the actor behavior-script text parser
  slice. `ActorBehaviorScript::parse_text` and
  `str::parse::<ActorBehaviorScript>()` now accept checked text lines for
  `default`, `kind`, and `actor` profile updates. Parsed scripts can tune the
  full `ActorBehaviorProfile` surface, including movement speeds, fire and
  lifetime timings, movement modes, hyperspace seed bytes, human gravity and
  safe-landing values, explosion/score lifetimes, and the player damage policy
  used by `XYZZY` invincibility. Added focused parser manifest, runtime
  movement/damage-policy, and parse-error regressions. Validation passed with
  `cargo fmt --check`, focused behavior parser/runtime tests, `cargo test
  actor_game --all-targets --features legacy-tools`, `cargo test actor_live
  --all-targets --features legacy-tools`, `cargo test actor_smoke
  --all-targets --features legacy-tools`, `cargo check --all-targets
  --features legacy-tools`, `cargo clippy --all-targets --features
  legacy-tools -- -D warnings`, touched-doc markdownlint, and `git diff
  --check`. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780655157405839`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780655626269779`.
- `2026-06-05 11:23 BST`: Completed the actor attract-script text parser
  slice. `AttractScript::parse_text` and `str::parse::<AttractScript>()` now
  accept checked text lines for `text`, `sprite`, `williams_logo`, and
  `defender_wordmark` events, with blank/comment-line skipping, unbounded
  duration aliases, sprite-key validation, sorted event installation, and
  line-numbered parse errors. Parsed scripts drive the same actor-local
  non-frame-driven attract program as Rust-constructed scripts and preserve
  coin/start control handling. Added focused parser manifest, parsed draw, and
  parse-error regressions. Validation passed with `cargo fmt --check`, focused
  parser/manifest tests, `cargo test actor_game --all-targets
  --features legacy-tools`, `cargo test actor_live --all-targets
  --features legacy-tools`, `cargo test actor_smoke --all-targets
  --features legacy-tools`, `cargo check --all-targets --features
  legacy-tools`, `cargo clippy --all-targets --features legacy-tools -- -D
  warnings`, touched-doc markdownlint, and `git diff --check`. Slack cycle
  start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780654662338869`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780655035833319`.
- `2026-06-05 11:16 BST`: Completed the actor attract-script manifest slice.
  `AttractScript` now exposes a read-only manifest of sorted event timing and
  action metadata, and `ActorGameDriver::script_manifest` includes the
  installed attract script beside behavior and wave manifests. Custom drivers
  can now verify or serialize their attract-screen scripts without inspecting
  the thread-backed attract actor. Added focused regressions for custom
  attract event manifest ordering/action metadata and default red-label
  Williams/Defender manifest exposure. Validation passed with
  `cargo fmt --check`, focused manifest tests, `cargo test actor_game
  --all-targets --features legacy-tools`, `cargo test actor_live --all-targets
  --features legacy-tools`, `cargo test actor_smoke --all-targets --features
  legacy-tools`, `cargo check --all-targets --features legacy-tools`, `cargo
  clippy --all-targets --features legacy-tools -- -D warnings`, touched-doc
  markdownlint, and `git diff --check`. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780654202430719`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780654588680689`.
- `2026-06-05 11:06 BST`: Completed the actor source explosion top-left/center
  metadata slice. Actor `ExplosionCloud` draw effects and
  `SpawnRequest::Explosion` commands now carry optional source-center metadata,
  `Explosion` actors preserve it through actor-owned lifetime state, and
  `ActorStateBridge` maps it into clean `ExplosionSnapshot` values while also
  publishing the age-derived source size. `ActorRenderSceneBridge` passes the
  same source center into the clean source expanded-object pixel renderer. The
  target6 fire2524 player/enemy collision path now emits the MAME-backed
  `SCZP1` top-left `0x20,0xA2` plus source center `0x21,0xA9` instead of
  overloading the center as the draw position, while generic explosion spawns
  keep `source_center: None`. Added focused regressions for target6 explosion
  command placement and for source-center preservation through actor state and
  render bridges. Validation passed with `cargo fmt --check`, focused
  target6/source-center/render tests, `cargo test actor_game::tests::target6
  --lib`, `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo test actor_live --all-targets --features legacy-tools`, `cargo test
  actor_smoke --all-targets --features legacy-tools`, `cargo check
  --all-targets --features legacy-tools`, `cargo clippy --all-targets
  --features legacy-tools -- -D warnings`, touched-doc markdownlint, and `git
  diff --check`. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780653412152619`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780653966181179`.
- `2026-06-05 10:54 BST`: Completed the actor source explosion pixel-cloud
  routing slice. Added a clean crate-private `push_source_explosion_cloud_pixels`
  helper that builds source explosion descriptor detail and reuses the clean
  expanded-object explosion pixel renderer, including hidden pre-visible frames
  and descriptor-backed family ownership. `ActorRenderSceneBridge` now tries
  that helper for source-screen explosion draw positions before falling back to
  the scaled sprite path, so lander, mutant, bomber, pod, swarmer, and baiter
  explosion clouds render as source pixels instead of static family atlas
  sprites while bomb, human, and player clouds keep their existing fallback
  sprite behavior. Added focused regressions for actor render-scene source
  pixel-cloud projection and the clean helper's handled/hidden/unsupported
  cases. Validation passed with `cargo fmt --check`, focused actor render and
  clean source-explosion tests, `cargo test actor_game::tests::target6 --lib`,
  `cargo test actor_game --all-targets --features legacy-tools`, `cargo test
  actor_live --all-targets --features legacy-tools`, `cargo test actor_smoke
  --all-targets --features legacy-tools`, `cargo check --all-targets --features
  legacy-tools`, `cargo clippy --all-targets --features legacy-tools -- -D
  warnings`, touched-doc markdownlint, and `git diff --check`. Slack cycle
  start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780652751573369`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780653243724269`.
- `2026-06-05 10:43 BST`: Completed the actor source-sized explosion growth
  slice. `ActorRenderSceneBridge` now consumes `VisualEffect::ExplosionCloud`
  age metadata, derives the source explosion size from the clean red-label
  initial-size and delta constants, applies the same capped render-scale helper
  used by the clean expanded-object path, and recenters the visible sprite
  around the original source object center as the cloud grows. This preserves
  the existing source-family sprite identities for lander, mutant, bomber, pod,
  swarmer, baiter, bomb, human, and player explosion draws while making the
  actor render output progress through source-shaped growth instead of fixed
  atlas dimensions. Added focused regressions for actor render-scene bomb
  explosion growth/centering and the actor source-size scale curve. Validation
  passed with `cargo fmt --check`, focused actor render/explosion tests,
  `cargo test actor_game::tests::target6 --lib`, `cargo test actor_game
  --all-targets --features legacy-tools`, `cargo test actor_live --all-targets
  --features legacy-tools`, `cargo test actor_smoke --all-targets --features
  legacy-tools`, `cargo check --all-targets --features legacy-tools`, `cargo
  clippy --all-targets --features legacy-tools -- -D warnings`, touched-doc
  markdownlint, and `git diff --check`. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780652214304989`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780652623310609`.
- `2026-06-05 10:27 BST`: Completed the actor source-family explosion fidelity
  slice. Actor `ExplosionKind` now preserves source-family identity for lander,
  mutant, bomber, pod, swarmer, baiter, bomb, player, and human explosion
  clouds instead of collapsing hostile explosions to one generic enemy bucket.
  Actor collision and smart-bomb commands now emit family-specific explosion
  kinds, the actor clean-state bridge maps them into the clean
  `ExplosionKind` contract, and `ActorRenderSceneBridge` maps those variants to
  renderer-owned source-backed family sprites. Enemy-fire clears now use the
  bomb-shell explosion family instead of a false lander-family fallback. Added
  focused regressions for actor render-scene explosion family sprites and clean
  state bridge family preservation, and updated the existing target6, lander
  collision, and explosion metadata expectations. Validation passed with
  `cargo fmt --check`, focused actor render/state/explosion/lander-collision
  tests, `cargo test actor_game::tests::target6 --lib`, `cargo test
  actor_game --all-targets --features legacy-tools`, `cargo test actor_live
  --all-targets --features legacy-tools`, `cargo test actor_smoke
  --all-targets --features legacy-tools`, `cargo check --all-targets
  --features legacy-tools`, `cargo clippy --all-targets --features
  legacy-tools -- -D warnings`, touched-doc markdownlint, and `git diff
  --check`. A broad diagnostic `cargo test explosion --lib` still picks up two
  unchanged clean-runtime target6 MAME tests in `src/game.rs`; all actor tests
  in that filter passed, so this remains the known clean-runtime target6
  validation caveat rather than actor-slice evidence. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780651173143849`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780651614387479`.
- `2026-06-05 10:13 BST`: Completed the actor target6
  converted-mutant collision fidelity slice. The actor `CollisionBody` now
  carries raw actor position plus source-mutant metadata, target6 mutant bounds
  use the source-projected collision position, pending fire2524 target6 rows
  suppress player/enemy contact until the source collision window, and the
  fire2524 collision row emits the source-positioned enemy explosion, mutant
  hit cue, score award, and normal player-death command path. The status
  display high-score-entry regression now uses an enemy-laser death so it
  remains focused on display output while source-shaped enemy contact can
  award enemy score. Added focused actor regressions for target6 collision
  projection, pending fire2524 collision suppression, and fire2524
  source-positioned enemy explosion/scoring. Validation passed with `cargo fmt
  --check`, `cargo test actor_game::tests::target6 --lib`, `cargo test
  source_mutant --lib`, `cargo test actor_game --all-targets --features
  legacy-tools`, `cargo test actor_live --all-targets --features
  legacy-tools`, `cargo test actor_smoke --all-targets --features
  legacy-tools`, `cargo check --all-targets --features legacy-tools`, and
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`.
  Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780650250437919`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780650848367269`.
- `2026-06-05 10:01 BST`: Completed the actor target6
  converted-mutant source path slice. Source-backed first-wave target6 lander
  conversions now carry the source X-correction into actor mutant metadata;
  target6 mutants use source-backed dive/visual projection anchors for actor
  draw/collision positions; the actor source path preserves the deferred
  visible-entry shot state, target6 dive shot-position overrides, the fire2524
  pending-shot timer, and exact forced fire2524 projectile fractions,
  velocities, lifetime metadata, and `0xF6` mutant-shot cue. Added focused
  actor regressions for target6 conversion correction, deferred first-entry
  shots, visible-entry projection, exact fire2524 projectile metadata, and
  dive shot anchor overrides. Validation passed with `cargo fmt --check`,
  `cargo test actor_game::tests::target6 --lib`, `cargo test source_mutant
  --lib`, `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo test actor_live --all-targets --features legacy-tools`, `cargo test
  actor_smoke --all-targets --features legacy-tools`, `cargo check
  --all-targets --features legacy-tools`, and `cargo clippy --all-targets
  --features legacy-tools -- -D warnings`. A broad `cargo test target6 --lib`
  filter still picks up unrelated clean-runtime MAME target6 tests; those
  clean-runtime tests fail standalone without any `src/game.rs` diff, so they
  remain a pre-existing clean-runtime validation caveat rather than evidence
  against this actor slice. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780648821951209`.
  Slack continuation:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780649199976299`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780650124969389`.
- `2026-05-29 20:54 BST`: Completed organic terrain-blow evidence
  follow-up after the release-gate revalidation found new organic last-human
  `TERBLO` candidates. The full `make release-gate` pass stayed green, but
  the fresh all-trace scan now covers `218` expected traces / `214` debug
  traces and the organic-only scan covers `198` expected traces / `194` debug
  traces, both with two frame-`5990` last-human terrain-blow candidates. The
  bounded smartmix report at
  `target/reference-media/organic-terrain-blow-smartmix-check/report.json`
  currently fails all-axis acceptance because MAME reaches attract score `50`
  and emits terrain-blow audio from frame `5991`, while clean reaches score
  `2675`, retains residual humans, has no `terrain_blow`, and emits no audio.
  Updated README, SPEC, this plan, and
  `docs/fidelity/release-closure-audit.md` to classify the probe as a
  concrete open mismatch instead of a missing-evidence boundary. Validation
  passed with `make owner-review-package`, `make reference-report-gate`,
  `make docs-lint`, and `make diff-check`. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780083913622229`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780084463593359`.
- `2026-05-29 15:57 BST`: Completed proof-boundary and release-audit sync.
  Scanned `99` local `target/reference-media/**/report.json` files against
  the accepted manifest: `27` are accepted and `72` are unaccepted probe or
  historical reports. Ten unaccepted reports currently pass, but all are
  offset/probe duplicates of already accepted fire/reverse, smart-bomb,
  terrain-blow, materialization-matrix, or down030 laser media, so none was
  promoted as new bounded proof. Updated
  `docs/fidelity/release-closure-audit.md` and this plan to reflect the latest
  `2026-05-29 15:54 BST` release-gate result and the unaccepted-report
  classification. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780066598389299`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780066751232149`.
- `2026-05-29 15:54 BST`: Re-ran the full release gate after the PRBP1
  all-axis/audio-silence promotion. `make release-gate` passed: `cargo fmt
  --check`, default and `legacy-tools` all-target Rust tests, both clippy
  gates with `-D warnings`, `make clean-fidelity`, `make media-script-test`,
  `make owner-review-package`, accepted-report gate at `27` reports (`20`
  all-axis, `4` audio-only, `3` visual-only), `make reference-mame-doctor`,
  `make reference-mame-smoke` with non-empty MP4/WAV output, README media
  generation, game smoke, live `wgpu` smoke with `320` nonblank offscreen
  frames, docs lint, and `git diff --check`. No active local gate failure
  remains; the finite closure boundary is owner review of the accepted proof
  set, or a new concrete MAME mismatch/input outside the accepted windows.
  Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780065971963379`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780066483344249`.
- `2026-05-29 15:39 BST`: Promoted the organic PRBP1 pod up-thrust window from
  visual-only evidence to all-axis evidence. The clean runtime now suppresses
  post-game clean-only thrust/background leakage after the duplicate MAME
  background-end command, and the synth mixer clears an active thrust loop when
  a background-end action arrives. Regenerated
  `target/reference-media/organic-nonlander-prbp1-upthrust-check/report.json`
  with `acceptance_mode=all`: visual still passes with full RMS `37.95`, full
  MAE `7.68`, and `60` compared frames; audio now passes with `220500`
  compared samples, clean RMS `0.0`, MAME RMS `0.0000077`, correlation `1.0`,
  envelope correlation `1.0`, and no failures. Updated
  `docs/fidelity/reference-report-gate.json` so the PRBP1 report contributes
  `gameplay_audio` coverage and raised that breadth floor to `22`. The
  accepted report gate now passes at `27` reports (`20` all-axis, `4`
  audio-only, `3` visual-only). Validation passed with `cargo fmt --check`,
  focused Rust regressions, the `legacy-tools` cargo check/clippy gates,
  `make media-script-test`, `make reference-report-gate`,
  `make reference-evidence-package`, `make owner-review-package`, markdownlint,
  and `git diff --check`. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780064999710469`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780065904130259`.
- `2026-05-29 15:28 BST`: Completed bounded organic PRBP1 pod visual proof.
  Selected the per-family `PRBP1 pod` span from
  `organic_fire_up_thrust_10000`, generated a bounded MAME MP4/WAV capture and
  matching clean GIF/WAV candidate for frames `6855-7455`, and produced
  `target/reference-media/organic-nonlander-prbp1-upthrust-check/report.json`.
  The report passes with `acceptance_mode=visual`, full visual RMS `37.95`,
  full visual MAE `7.68`, and `60` compared frames; audio remains
  diagnostic-only because MAME is effectively silent while clean carries
  background audio in that post-game window. Added the report to
  `docs/fidelity/reference-report-gate.json` and raised the breadth floors to
  `sprite_visuals=21`, `non_lander_visual=6`, and
  `organic_non_lander_visual=3`. Regenerated the owner-review evidence package:
  the accepted report gate now passes at `27` reports (`19` all-axis, `4`
  audio-only, `4` visual-only) across `19` coverage requirements. Validation
  passed with `make media-script-test`, `make reference-evidence-package`,
  `make owner-review-package`, focused scanner/report unit tests,
  markdownlint, and `git diff --check`. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780064206980329`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780064951004119`.
- `2026-05-29 15:16 BST`: Completed reference-window object-span diagnostics.
  The MAME trace scanner now reports per-family object row counts and the
  longest contiguous object-evidence spans; the accepted-report signoff summary
  surfaces those counts/spans in the all-trace and organic-only reference
  window scan matrix. Regenerated scans still show zero organic target
  non-lander sound hits and zero organic `TERBLO` process rows, but now expose
  concrete long `SCZP1`, `PRBP1`, and `BXPIC` windows for bounded follow-up
  media selection. Updated `README.md`, `SPEC.md`, `PLAN.md`, and
  `docs/fidelity/release-closure-audit.md` to document the evidence workflow.
  Validation passed with `make media-script-test`, `make reference-window-scan`,
  `make reference-window-scan-organic`, `make reference-report-gate`,
  `make reference-evidence-package`, `make owner-review-package`,
  `markdownlint README.md SPEC.md PLAN.md docs/fidelity/release-closure-audit.md`,
  `markdownlint target/reference-media/reference-signoff-summary.md`, and
  `git diff --check`. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780063560341269`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780064157532159`.
- `2026-05-29 15:04 BST`: Completed full release-gate validation after the
  scan-tooling and organic-search updates. `make release-gate` passed:
  `cargo fmt --check`, default and `legacy-tools` Rust tests, both clippy
  passes, `make clean-fidelity`, media helper tests, fresh owner-review
  evidence, accepted-report gate (`26` reports / `19` coverage requirements),
  MAME doctor, MAME smoke MP4/WAV capture, README media generation, game
  smoke, live `wgpu` smoke, docs lint, and diff hygiene. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780063070585059`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780063447684729`.
- `2026-05-29 14:55 BST`: Completed organic trace-discovery pass. Ran six new
  10,000-frame MAME trace-only searches under
  `target/reference-media/mame/organic-search/`:
  `organic_up_thrust_10000`, `organic_down_thrust_10000`,
  `organic_reverse_up_thrust_10000`, `organic_reverse_down_thrust_10000`,
  `organic_fire_up_thrust_10000`, and `organic_fire_down_thrust_10000`.
  The search subtree scan covered `6` expected/debug traces, found `60,591`
  non-lander object rows across `SCZP1`, `PRBP1`, and `BXPIC`, but found zero
  target non-lander sound hits and zero `TERBLO` process rows. The fresh
  all-trace scan now covers `210` expected traces and `206` debug traces:
  target sound hits remain `16`, all from isolated synthetic command traces;
  sound/object candidates remain zero; `TERBLO` rows remain the two
  state-steered `ASTCNT=0x0A` rows. The fresh organic-only scan now covers
  `190` expected traces and `186` debug traces, with zero target sound hits,
  zero sound/object candidates, and zero `TERBLO` process rows. Slack cycle
  start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780062721004219`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780063051925379`.
- `2026-05-29 14:49 BST`: Completed reference-window near-miss diagnostics.
  `tools/scan_reference_windows.py` now records nearest sound/object misses
  when target non-lander sound bytes exist but miss the configured object
  proximity window, and records explicit `TERBLO` process misses when the
  terrain-blow process is present without `ASTCNT=0x00`. The owner-review
  summary now exposes those diagnostics in its `Reference Window Scans`
  matrix. Current all-trace evidence still has `16` target sound hits, zero
  sound/object candidates, two state-steered `TERBLO` rows, and zero
  last-human candidates; the organic-only scan still has zero target sound
  hits and zero `TERBLO` process rows. Validation passed with
  `python3 -m unittest tools/scan_reference_windows_test.py
  tools/check_reference_reports_test.py`, `make reference-window-scan`, and
  `make reference-window-scan-organic`. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780062303337199`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780062695755029`.
- `2026-05-29 14:42 BST`: Completed owner-review coverage-matrix duplicate
  guard. The current summary helper already lists accepted report names once
  per coverage requirement, and
  `tools/check_reference_reports_test.py` now has a focused two-report
  regression proving the generated coverage matrix does not repeat the same
  accepted proof names. This protects the owner-review summary from overstating
  evidence breadth if the coverage-matrix helper regresses. Validation passed
  with `python3 -m unittest tools/check_reference_reports_test.py` (`33`
  tests), `make reference-report-gate`, `make owner-review-package`,
  `markdownlint target/reference-media/reference-signoff-summary.md`, a
  generated-summary duplicate spot-check, `make docs-lint`, `make diff-check`,
  and the full `make release-gate`. The release gate passed default and
  `legacy-tools` tests, both clippy passes, clean-fidelity, media helper tests,
  the owner-review package, accepted report gate, MAME doctor, MAME smoke
  MP4/WAV capture, README media generation, game/live smoke, docs lint, and
  diff hygiene. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780061641019519`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780062183431019`.
- `2026-05-29 14:32 BST`: Completed signoff-summary acceptance-mode
  disclosure. The generated owner-review summary now exposes both
  `Manifest Mode` and `Report Mode` in the report matrix, so the accepted-axis
  proof scope is visible in the review artifact instead of only implicit in a
  passing gate. Added focused summary regression coverage in
  `tools/check_reference_reports_test.py` and documented the disclosure in
  `README.md`, `SPEC.md`, this plan, and
  `docs/fidelity/release-closure-audit.md`. Validation passed with
  `python3 -m unittest tools/check_reference_reports_test.py` (`32` tests),
  `make reference-report-gate`, `make owner-review-package`,
  `markdownlint target/reference-media/reference-signoff-summary.md`,
  `make docs-lint`, `make diff-check`, and the full `make release-gate`. The
  release gate passed default and `legacy-tools` tests, both clippy passes,
  clean-fidelity, media helper tests, the owner-review package, accepted report
  gate, MAME doctor, MAME smoke MP4/WAV capture, README media generation,
  game/live smoke, docs lint, and diff hygiene. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780060997507529`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780061550173729`.
- `2026-05-29 14:21 BST`: Completed report-level acceptance-mode hardening.
  `tools/check_reference_reports.py` no longer treats a missing local
  `report.json` `acceptance_mode` as `all`; accepted report evidence must now
  explicitly match the mode declared in
  `docs/fidelity/reference-report-gate.json`. `tools/verify_reference_media.py`
  already writes explicit report modes for new media checks, and
  `tools/verify_reference_media_test.py` now locks that generator contract.
  Updated the `13` older accepted local reports under `target/reference-media/`
  that predated explicit report modes, and documented the matching
  manifest/report mode rule in `README.md`, `SPEC.md`, this plan, and
  `docs/fidelity/release-closure-audit.md`. Validation passed with
  `python3 -m unittest tools/verify_reference_media_test.py ...` and
  `tools/check_reference_reports_test.py` (`49` combined tests),
  `python3 -m json.tool docs/fidelity/reference-report-gate.json >/dev/null`,
  `make reference-report-gate`, `make media-script-test`,
  `make owner-review-package`, `make docs-lint`, `make diff-check`, and the
  full `make release-gate`. The release gate passed default and `legacy-tools`
  tests, both clippy passes, clean-fidelity, media helper tests, the
  owner-review package, accepted report gate, MAME doctor, MAME smoke MP4/WAV
  capture, README media generation, game/live smoke, docs lint, and diff
  hygiene. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780060264940719`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780060886236319`.
- `2026-05-29 14:09 BST`: Completed explicit accepted-report mode hardening.
  `tools/check_reference_reports.py` now requires every manifest report row to
  declare `acceptance_mode` explicitly instead of silently defaulting missing
  rows to `all`. This keeps future visual/audio-scoped proof rows from
  accidentally widening into full visual/audio acceptance by omission. Added a
  focused missing-mode regression test; the checker suite now runs `31` tests.
  Updated `README.md`, `SPEC.md`, this plan, and
  `docs/fidelity/release-closure-audit.md` to document the explicit manifest
  mode requirement. Validation passed with
  `python3 -m unittest tools/check_reference_reports_test.py` (`31` tests),
  `python3 -m json.tool docs/fidelity/reference-report-gate.json >/dev/null`,
  `make reference-report-gate`, `make media-script-test`,
  `make owner-review-package`, and the full `make release-gate`. The release
  gate passed default and `legacy-tools` tests, both clippy passes,
  clean-fidelity, media helper tests, the owner-review package, accepted report
  gate, MAME doctor, MAME smoke MP4/WAV capture, README media generation,
  game/live smoke, docs lint, and diff hygiene. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780059586427269`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780060150055919`.
- `2026-05-29 13:58 BST`: Completed accepted-report media-type hardening.
  `tools/check_reference_reports.py` now rejects accepted visual/audio media
  artifacts whose file types do not match the bounded proof contract: MAME
  reference visuals must be `.mp4`, clean visual candidates must be `.gif`, and
  accepted audio artifacts must be `.wav`. This prevents an accepted report from
  satisfying a visual or audio proof with the wrong generated artifact class.
  Added a focused wrong-suffix regression test; the checker suite now runs `30`
  tests. Updated `README.md`, `SPEC.md`, this plan, and
  `docs/fidelity/release-closure-audit.md` to document the accepted media-type
  constraints. Validation passed with
  `python3 -m unittest tools/check_reference_reports_test.py` (`30` tests),
  `python3 -m json.tool docs/fidelity/reference-report-gate.json >/dev/null`,
  `make reference-report-gate`, `make media-script-test`,
  `make owner-review-package`,
  `markdownlint target/reference-media/reference-signoff-summary.md`,
  `make docs-lint`, `make diff-check`, and the full `make release-gate`. The
  release gate passed default and `legacy-tools` tests, both clippy passes,
  clean-fidelity, media helper tests, the owner-review package, accepted report
  gate, MAME doctor, MAME smoke MP4/WAV capture, README media generation,
  game/live smoke, docs lint, and diff hygiene. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780058964815859`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780059518443849`.
- `2026-05-29 13:46 BST`: Completed accepted-report media-root hardening.
  `tools/check_reference_reports.py` now rejects manifest report paths outside
  `target/reference-media/`; accepted MAME reference video/audio artifacts must
  stay under `target/reference-media/mame/`, and accepted clean candidate
  GIF/WAV artifacts must stay under `target/reference-media/clean/`. This
  prevents arbitrary local files, swapped reference/candidate roots, or
  non-generated media paths from satisfying the MAME-vs-clean proof gate. Added
  focused tests for report-path root rejection and wrong-root media artifacts;
  the checker suite now runs `29` tests. Updated `README.md`, `SPEC.md`, this
  plan, and `docs/fidelity/release-closure-audit.md` to document the media-root
  constraints. Validation passed with
  `python3 -m unittest tools/check_reference_reports_test.py` (`29` tests),
  `python3 -m json.tool docs/fidelity/reference-report-gate.json >/dev/null`,
  `make reference-report-gate`, `make media-script-test`,
  `make owner-review-package`,
  `markdownlint target/reference-media/reference-signoff-summary.md`,
  `make docs-lint`, `make diff-check`, and the full `make release-gate`. The
  release gate passed default and `legacy-tools` tests, both clippy passes,
  clean-fidelity, media helper tests, the owner-review package, accepted report
  gate, MAME doctor, MAME smoke MP4/WAV capture, README media generation,
  game/live smoke, docs lint, and diff hygiene. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780058213885989`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780058816754059`.
- `2026-05-29 13:35 BST`: Completed accepted-report coverage-tag closure
  hardening. `tools/check_reference_reports.py` now requires each accepted
  report coverage tag to be non-empty, declared by a semantic
  `coverage_requirements` row, and compatible with the report's `all`,
  `visual`, or `audio` acceptance mode. This prevents manifest drift where a
  report appears to prove a fidelity facet but the tag is misspelled,
  undeclared, empty, or attached to an axis that cannot accept that proof.
  Added focused unit tests for empty coverage, undeclared tags, and
  mode-incompatible tags; the checker suite now runs `27` tests. Updated
  `README.md`, `SPEC.md`, this plan, and
  `docs/fidelity/release-closure-audit.md` to document the coverage-tag closure
  rule. Validation passed with
  `python3 -m unittest tools/check_reference_reports_test.py` (`27` tests),
  `python3 -m json.tool docs/fidelity/reference-report-gate.json >/dev/null`,
  `make reference-report-gate`, `make media-script-test`,
  `make owner-review-package`,
  `markdownlint target/reference-media/reference-signoff-summary.md`,
  `make docs-lint`, `make diff-check`, and the full `make release-gate`. The
  release gate passed default and `legacy-tools` tests, both clippy passes,
  clean-fidelity, media helper tests, the owner-review package, accepted report
  gate, MAME doctor, MAME smoke MP4/WAV capture, README media generation,
  game/live smoke, docs lint, and diff hygiene. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780057559430939`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780058117072119`.
- `2026-05-29 13:24 BST`: Completed accepted-report manifest uniqueness
  hardening. `tools/check_reference_reports.py` now rejects duplicate coverage
  requirement names, duplicate coverage/axis requirement keys, duplicate report
  names, duplicate report paths, and duplicate coverage tags inside a report,
  preventing `docs/fidelity/reference-report-gate.json` from counting the same
  accepted proof twice toward `min_reports` breadth floors. Added focused unit
  tests for each duplicate-manifest failure mode and updated `README.md`,
  `SPEC.md`, this plan, and `docs/fidelity/release-closure-audit.md` to
  document the uniqueness requirement. Validation passed with
  `python3 -m unittest tools/check_reference_reports_test.py` (`24` tests),
  `python3 -m json.tool docs/fidelity/reference-report-gate.json >/dev/null`,
  `make reference-report-gate`, `make media-script-test`,
  `make owner-review-package`,
  `markdownlint target/reference-media/reference-signoff-summary.md`,
  `make docs-lint`, `make diff-check`, and the full `make release-gate`. The
  release gate passed default and `legacy-tools` tests, both clippy passes,
  clean-fidelity, media helper tests, the owner-review package, accepted report
  gate, MAME doctor, MAME smoke MP4/WAV capture, README media generation,
  game/live smoke, docs lint, and diff hygiene. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780056887779579`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780057443764899`.
- `2026-05-29 13:12 BST`: Completed accepted-report coverage-breadth
  hardening. `make reference-report-gate` now enforces each semantic coverage
  requirement's `min_reports` floor, so the current MAME-vs-clean proof set
  cannot silently shrink to one accepted report for broad objective facets.
  `docs/fidelity/reference-report-gate.json` locks the current breadth for all
  `19` coverage requirements, from `20` sprite-visual reports and `21`
  gameplay-audio reports down to the intentionally narrow smart-bomb,
  hyperspace, and terrain-blow facets. The generated owner-review summary now
  includes a `Minimum Reports` coverage-matrix column. Updated `README.md`,
  `SPEC.md`, this plan, and `docs/fidelity/release-closure-audit.md` to
  document the stricter gate. Validation passed with
  `python3 -m unittest tools/check_reference_reports_test.py`,
  `python3 -m json.tool docs/fidelity/reference-report-gate.json >/dev/null`,
  `make reference-report-gate`, `make owner-review-package`,
  `markdownlint target/reference-media/reference-signoff-summary.md`,
  `make media-script-test`, `make docs-lint`, `make diff-check`, and the full
  `make release-gate`. The release gate passed default and `legacy-tools`
  tests, both clippy passes, clean-fidelity, media helper tests, the
  owner-review package, accepted report gate, MAME doctor, MAME smoke MP4/WAV
  capture, README media generation, game/live smoke, docs lint, and diff
  hygiene. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780056043845819`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780056721584429`.
- `2026-05-29 12:58 BST`: Completed owner-review handoff target hardening.
  Added `make owner-review-package` as the executable review handoff: it
  regenerates both reference-window scan JSON files, rebuilds
  `target/reference-media/reference-signoff-summary.md`, re-runs the accepted
  report gate, and prints the finite owner-review checklist from
  `docs/fidelity/release-closure-audit.md`. Wired `make release-gate` through
  this target so final validation now exercises the same fresh evidence package
  and checklist that owner review uses. Validation passed with
  `make owner-review-package` (`26` accepted reports across `19` coverage
  requirements), `markdownlint target/reference-media/reference-signoff-summary.md`,
  `make docs-lint`, `make diff-check`, and the full `make release-gate`. The
  full gate passed default and `legacy-tools` tests, both clippy passes,
  clean-fidelity, media helper tests, the owner-review package, MAME doctor,
  MAME smoke capture, README media generation, game/live smoke, docs lint, and
  diff hygiene. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780055385157239`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780055935730389`.
- `2026-05-29 12:48 BST`: Completed current-status documentation cleanup.
  Updated the README and SPEC front matter so the old B13 rejection language is
  historical context rather than an active implementation-defect statement. The
  top-level docs now say the known laser, reverse, and audio defects were
  repaired against accepted MAME evidence, the full `make release-gate` path is
  green, and final closure is owner review / proof-boundary acceptance or a new
  concrete MAME mismatch. Refreshed the release-closure audit to reference the
  latest green gate with the MAME recorder smoke target and non-empty capture
  output checks. Validation passed with `make docs-lint`, `make diff-check`,
  and a stale-status scan for `B13`, `reopened`, `owner signoff`, `known open`,
  and related old wording. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780055064356489`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780055289255469`.
- `2026-05-29 12:43 BST`: Completed MAME capture output validation hardening.
  `tools/capture_mame_reference.py` now fails non-trace captures unless both
  generated review media artifacts exist and are non-empty: the compressed MP4
  and the matching PCM WAV. Added focused unit coverage for valid output,
  missing MP4 output, and zero-byte WAV output. Validation passed with
  `python3 -m unittest tools/capture_mame_reference_test.py` (`12` tests),
  `make media-script-test`, `make reference-mame-smoke`, and the full
  `make release-gate`. The release gate again passed default and
  `legacy-tools` tests, both clippy passes, clean-fidelity, media helper tests,
  fresh `make reference-evidence-package`, `make reference-report-gate` (`26`
  reports across `19` coverage requirements), MAME doctor, the non-empty MAME
  smoke capture, README media generation, game/live smoke, docs lint, and diff
  hygiene. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780054526102469`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780054980935959`.
- `2026-05-29 12:34 BST`: Completed MAME recorder smoke target hardening.
  Added `make reference-mame-smoke` as the named two-second executable proof
  for the local red-label MAME capture path, using
  `MAME_REFERENCE_SECONDS=2` and
  `MAME_REFERENCE_BASENAME=defender-red-label-smoke-script`. Wired that target
  into `make release-gate` after `make reference-mame-doctor`, and synchronized
  README, SPEC, and the release-closure audit so release validation now checks
  the recorder path, not just ROM/MAME availability. Validation passed with
  `make -n reference-mame-smoke`, `make -n release-gate`,
  `make reference-mame-smoke`, `make docs-lint`, and the full
  `make release-gate`. The full gate passed default and `legacy-tools` tests,
  both clippy passes, clean-fidelity, media helper tests, fresh
  `make reference-evidence-package`, `make reference-report-gate` (`26`
  reports across `19` coverage requirements), MAME doctor, the new MAME smoke
  capture, README media generation, game/live smoke, docs lint, and diff
  hygiene. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780053909279169`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780054480377059`.
- `2026-05-29 12:22 BST`: Completed accepted-axis failure-metadata hardening.
  `make reference-report-gate` now rejects accepted visual/audio axes when
  their verifier `failures` arrays contain stale entries, while still allowing
  diagnostic failures on non-accepted axes for visual-only/audio-only synthetic
  reports. The current accepted set has zero stale failures on accepted axes.
  Validation passed with `make media-script-test`, `make reference-report-gate`,
  `make reference-evidence-package`, `make docs-lint`, and `make diff-check`.
  The generated signoff summary remains at `26` accepted reports, `19`
  semantic coverage requirements, `90` required non-empty media artifacts,
  `22` visual comparisons, and `23` audio comparisons. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780053594295089`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780053747929549`.
- `2026-05-29 12:18 BST`: Completed accepted-report frame/sample-count
  hardening. `make reference-report-gate` now rejects accepted visual axes
  unless `reference_frames`, `candidate_frames`, and `compared_frames` are
  positive, and rejects accepted audio axes unless `reference_samples`,
  `candidate_samples`, and `compared_samples` are positive. The generated
  signoff summary now exposes accepted comparison breadth: `22` visual
  comparisons and `23` audio comparisons, alongside `90` required non-empty
  media artifacts across `26` accepted reports and `19` semantic coverage
  requirements. Validation passed with `make media-script-test`,
  `make reference-report-gate`, `make reference-evidence-package`,
  `make docs-lint`, and `make diff-check`. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780053282525939`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780053540079389`.
- `2026-05-29 12:13 BST`: Completed accepted-media non-empty artifact
  hardening. `make reference-report-gate` now rejects zero-byte required
  MAME/clean media artifacts for accepted visual/audio axes, so accepted
  `report.json` files cannot pass if reviewable MP4/GIF/WAV evidence has been
  truncated. A follow-up check now also requires accepted visual axes to report
  positive reference, candidate, and compared frame counts, and accepted audio
  axes to report positive reference, candidate, and compared sample counts.
  Accepted axes must not carry stale verifier `failures` entries. The generated
  signoff summary now labels the artifact side as `Required non-empty media
  artifacts` and reports accepted visual/audio comparison counts; the current
  package checks `90` required non-empty artifacts across `26` accepted reports
  and `19` semantic coverage requirements, with positive frame/sample counts
  and no failure metadata on every accepted axis.
  Validation passed with
  `python3 -m unittest tools/check_reference_reports_test.py`,
  `make media-script-test`, `make reference-report-gate`,
  `make reference-evidence-package`, `make docs-lint`, and
  `make diff-check`. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780053050331219`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780053213404429`.
- `2026-05-29 12:12 BST`: Completed accepted-report media-artifact hardening.
  The accepted report gate now verifies that every accepted visual/audio axis
  still has its required local MAME reference and clean candidate media files.
  A follow-up check now also rejects zero-byte accepted media, preventing stale
  `report.json` files from passing after reviewable MP4/GIF or WAV artifacts
  are removed or truncated. The generated signoff summary now reports the
  required non-empty media-artifact count; the current package checks `90`
  required non-empty media artifacts across the `26` accepted reports and `19`
  semantic coverage requirements. Validation passed with
  `python3 -m unittest tools/check_reference_reports_test.py`,
  `make media-script-test`, `make reference-report-gate`,
  `make reference-evidence-package`, `make docs-lint`, and
  `make diff-check`. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780052697102609`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780052974769229`.
- `2026-05-29 12:03 BST`: Completed release-gate target hardening. Added
  `make release-gate` as the executable form of the documented release gate,
  plus helper targets `make game-smoke`, `make live-smoke`, `make docs-lint`,
  and `make diff-check`. Updated `README.md`, `SPEC.md`, this plan, and the
  release-closure audit so release validation no longer depends on manually
  copying the command list. Validation passed with `make -n release-gate`,
  `make docs-lint`, `make diff-check`, and the full `make release-gate`. The
  full target passed formatting, default and `legacy-tools` Rust tests, both
  clippy passes, clean-fidelity scenarios, media helper tests, fresh
  `make reference-evidence-package`, `make reference-report-gate` (`26`
  reports across `19` coverage requirements), `make reference-mame-doctor`,
  `make readme-media`, `make game-smoke`, `make live-smoke`, docs lint, and
  diff hygiene. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780052070841729`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780052614494249`.
- `2026-05-29 11:48 BST`: Completed signoff-summary boundary evidence
  expansion. `make reference-signoff-summary` now appends a
  `Reference Window Scans` section to
  `target/reference-media/reference-signoff-summary.md`, covering both
  `target/reference-media/reference-window-scan.json` and
  `target/reference-media/reference-window-scan-organic.json`. The generated
  owner-review artifact now includes accepted media report metrics plus
  all-trace and organic-only scan exclusions, trace counts, target sound hits,
  object rows, candidate windows, terrain status rows, `TERBLO` rows,
  last-human terrain candidates, and `ASTCNT` distributions. Validation passed
  with `python3 -m unittest tools/check_reference_reports_test.py`,
  `make reference-window-scan`, `make reference-window-scan-organic`,
  `make reference-signoff-summary`,
  `markdownlint target/reference-media/reference-signoff-summary.md`,
  `make media-script-test`, `make reference-report-gate`, docs markdownlint,
  and `git diff --check`. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780051539597159`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780051728944849`.
- `2026-05-29 11:49 BST`: Started fresh evidence-package target cycle. Scope:
  add a single target that regenerates both reference-window scan JSON files
  before building `target/reference-media/reference-signoff-summary.md`, so the
  owner-review package is not assembled from stale scan evidence. Slack cycle
  start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780051793630249`.
  Completed at `2026-05-29 11:51 BST`: added
  `make reference-evidence-package`, which runs `make reference-window-scan`,
  `make reference-window-scan-organic`, and `make reference-signoff-summary` in
  order. Validation passed with `make reference-evidence-package`,
  `make media-script-test`, `make reference-report-gate`,
  `markdownlint target/reference-media/reference-signoff-summary.md`, docs
  markdownlint, and `git diff --check`. Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780051899292799`.
- `2026-05-29 11:44 BST`: Completed split all-trace and organic-only
  reference-window scan artifacts. Added `make reference-window-scan-organic`,
  which writes `target/reference-media/reference-window-scan-organic.json`
  using the standard synthetic/state-steered exclusions while preserving
  `make reference-window-scan` as the all-trace report writer. Current
  generated evidence: all-trace scan covers `204` expected traces and `200`
  debug traces, finds `16` target sound hits, `149305` object rows, `113279`
  terrain status rows, `2` `TERBLO` process rows, and zero last-human terrain
  candidates; organic-only scan covers `184` expected traces and `180` debug
  traces, finds zero target sound hits, `147495` object rows, `105596`
  terrain status rows, zero `TERBLO` process rows, and zero last-human terrain
  candidates. Validation passed with `make reference-window-scan`,
  `make reference-window-scan-organic`, `make media-script-test`,
  `make reference-report-gate`, docs markdownlint, and `git diff --check`.
  Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780051222520359`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780051460933749`.
- `2026-05-29 11:36 BST`: Completed organic terrain-blow scan hardening.
  Extended `make reference-window-scan` so the generated JSON and text report
  include terrain status rows, `TERBLO` process rows, `ASTCNT` distribution,
  and last-human terrain-blow candidates. The current all-trace scan covers
  `204` expected traces and `200` debug traces, finds `16` target non-lander
  sound hits, `149305` object rows, zero non-lander sound/object candidates,
  `113279` terrain status rows, `2` `TERBLO` process rows, and zero
  last-human terrain candidates. The organic-only scan excluding
  `nonlander-sound-command`, `enemy-explosion-matrix`,
  `enemy-materialize-matrix`, and `state-steered` covers `184` expected traces
  and `180` debug traces, finds zero target sound hits, `147495` object rows,
  zero non-lander candidates, `105596` terrain status rows, zero `TERBLO`
  process rows, and zero last-human candidates. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780050793200489`.
  Validation passed with `make media-script-test`, `make reference-window-scan`,
  `make reference-window-scan-organic`, `make reference-report-gate`, docs
  markdownlint, and `git diff --check`. Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780051146614649`.
- `2026-05-29 11:28 BST`: Started owner-review proof package hardening. Scope:
  make the remaining release-signoff step deterministic by generating a
  Markdown owner-review matrix directly from
  `docs/fidelity/reference-report-gate.json` and the local accepted
  `target/reference-media/**/report.json` artifacts. The target should list
  coverage requirements, accepted reports, visual/audio metric summaries, and
  MAME/clean media paths so owner review is not dependent on hand-maintained
  prose alone. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780050357114229`.
  Completed at `2026-05-29 11:31 BST`: added
  `make reference-signoff-summary`, wired the report checker to emit the
  deterministic Markdown signoff matrix, documented the generated review
  artifact, and updated the release closure audit. Validation passed with
  `python3 -m unittest tools/check_reference_reports_test.py`,
  `make reference-signoff-summary`,
  `markdownlint target/reference-media/reference-signoff-summary.md`,
  `make media-script-test`, `make reference-report-gate`, docs markdownlint,
  and `git diff --check`. Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780050686774359`.
- `2026-05-29 11:23 BST`: Completed release-gate revalidation after promoting
  the live laser-hit materialization report to the accepted gate. The first
  full-test pass exposed three stale default-test expectations: start-scene
  object counts had not been updated for the current two additional
  source-backed object sprites, and the single-player respawn helper still
  required a byte-for-byte default `WorldSnapshot` even though the respawn
  interstitial now preserves source scanner/RNG state while clearing live
  playfield content. Updated those tests to assert the observable current
  contract. Validation passed with `cargo fmt --check`,
  `cargo test --all-targets`,
  `cargo test --all-targets --features legacy-tools`,
  `cargo clippy --all-targets -- -D warnings`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  `make clean-fidelity`, `make media-script-test`,
  `make reference-report-gate`, `make reference-mame-doctor`,
  `make readme-media`, `cargo run -- --game-smoke`,
  `cargo run -- --live-smoke`, `make reference-window-scan`, JSON manifest
  validation, docs markdownlint, and `git diff --check`. The accepted report
  gate checks `26` reports (`19` all-axis, `4` audio-only, `3` visual-only)
  across `19` coverage requirements. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780049263956029`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780050244226059`.
- `2026-05-29 11:04 BST`: Started the live laser-hit materialization proof
  promotion step. Scope: add the existing passing
  `target/reference-media/gameplay-laser-hit-single-materialize-check/report.json`
  all-axis report to the accepted reference gate so live laser, enemy hit,
  explosion, coalescence, gameplay-audio, and playability evidence is enforced
  by `make reference-report-gate` rather than only documented in the historical
  work log. Slack cycle starts:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780048900955749` and
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780048981566299`.
  Completed at `2026-05-29 11:06 BST`: the accepted gate now checks `26`
  reports (`19` all-axis, `4` audio-only, `3` visual-only) against `19`
  coverage requirements. Validation passed with
  `python3 -m json.tool docs/fidelity/reference-report-gate.json`,
  `make reference-report-gate`, `make reference-mame-doctor`, docs
  markdownlint, and `git diff --check`. Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780049206736689`.
- `2026-05-29 10:20 BST`: Completed the down030 post-death laser all-axis
  cycle. Generated MAME and clean media for
  `none*900;coin*4;none*360;start_one*10;none*180;down*30;none*952;fire*45;none*350`.
  The clean default post-death branch now restores input readiness early enough
  for the second-life laser `0xEB` at frame `2439` and gates the post-death
  appearance/materialize `0xEA` at frame `2447`, matching MAME. The clean
  sound-board materialize waveform now follows the Williams `APPEAR`/`LITEN`
  sweep cadence instead of the earlier high-frequency placeholder. The accepted
  gate now uses
  `target/reference-media/laser-hit-down030-fire2437-check-fixed/report.json`,
  which passes all-axis mode with full visual RMS `36.51`, playfield RMS
  `33.91`, laser-band RMS `32.89`, terrain RMS `18.23`, audio envelope
  correlation `0.864`, RMS ratio `1.008`, and zero-crossing ratio `0.971`.
  Slack cycle starts:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780044035916139` and
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780045966251759`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780046685883929`.
- `2026-05-29 09:35 BST`: Added a finite owner-review checklist to
  `docs/fidelity/release-closure-audit.md`. The checklist ties final closure to
  the current accepted MAME-vs-clean report gate, generated local media, and
  explicit acceptance of the remaining proof boundaries; otherwise the next
  action must be a concrete new MAME mismatch/input program. Slack step start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780043729129199`.
  Slack step completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780043768584189`.
- `2026-05-29 09:34 BST`: Started and completed a bounded clean-runtime
  fidelity-debt scan after the green release gate. Searched production clean
  runtime source for `TODO`, `FIXME`, `placeholder`, `stub`, `approximate`,
  `guess`, `unimplemented!`, and `todo!` markers. No active
  sprite/audio/gameplay placeholder path was found; hits were guard tests,
  source asset invariants, clean CLI unsupported-argument tests, or
  raster-tooling counters that the smoke gates require to stay zero. Tightened
  stale README/SPEC wording so the active path is documented as source-backed
  sprite regions with zero temporary raster commands in smoke/live gameplay.
  Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780043577846259`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780043686657819`.
- `2026-05-29 09:31 BST`: Completed release-gate revalidation after adding the
  delayed-start thrust/reverse accepted report. The gate passed
  `cargo fmt --check`, `cargo test --all-targets`,
  `cargo test --all-targets --features legacy-tools`,
  `cargo clippy --all-targets -- -D warnings`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  `make clean-fidelity`, `make media-script-test`,
  `make reference-report-gate`, `make reference-mame-doctor`,
  `make readme-media`, `cargo run -- --game-smoke`,
  `cargo run -- --live-smoke`, docs markdownlint, and `git diff --check`.
  `make reference-report-gate` checked `23` reports (`17` all, `4` audio,
  `2` visual) across `19` semantic coverage requirements. Slack milestone
  start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780042956088699`.
  Slack milestone completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780043537434889`.
- `2026-05-29 09:17 BST`: Started and completed a delayed-start
  thrust/reverse media-evidence cycle. Generated/verified the MAME and clean
  `none*900;coin*4;none*360;start_one*10;none*180;thrust*30;reverse,thrust*30;none*200`
  window, then accepted
  `target/reference-media/gameplay-thrust-reverse-delayed-check/report.json`.
  The report passes full visual/audio acceptance with full visual RMS `28.40`,
  playfield RMS `9.38`, laser-band RMS `13.76`, audio envelope correlation
  `0.889`, RMS ratio `0.791`, and zero-crossing ratio `1.621`. Runtime audio
  calibration now uses the MAME-backed thrust filtered-noise step and a
  calibrated `0xF0` background-noise gain; protected delayed-start thrust and
  enemy-shot reports were regenerated and remained green. Validation passed
  with `cargo fmt --check`, focused sound/audio/thrust tests,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`, JSON
  manifest validation, `make reference-report-gate`, `make media-script-test`,
  `make reference-mame-doctor`, docs markdownlint, and `git diff --check`.
  Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780042625576569`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780042895609849`.
- `2026-05-29 08:42 BST`: Completed an accepted-media report inventory cycle.
  Scanned local `target/reference-media/**/report.json` files against
  `docs/fidelity/reference-report-gate.json`. Six passing reports were not in
  the semantic gate, but each was duplicate or probe evidence rather than a new
  bounded acceptance target: current fire/reverse duplicate, duplicate
  first-laser materialize window, smart-bomb offset scans, and terrain-blow PTS
  probe. No new report was accepted from this inventory. Validation passed with
  `markdownlint PLAN.md` and `git diff --check`. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780040549448509`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780040599537239`.
- `2026-05-29 08:40 BST`: Started an extra laser/reverse media-breadth cycle.
  Scope: inspect existing bounded MAME-vs-clean laser/reverse media outside
  the already-accepted delayed-start fire/reverse report, add only passing
  evidence to the semantic closure gate, and leave reverse-only clips as
  diagnostics unless their full accepted axes pass. Added the passing centered
  first gameplay laser-hit report to `docs/fidelity/reference-report-gate.json`;
  the gate now checks `22` reports and `19` semantic coverage requirements.
  Validation passed with JSON formatting, `make reference-report-gate`,
  `python3 -m unittest tools/check_reference_reports_test.py`,
  `make media-script-test`, docs markdownlint, and `git diff --check`. Slack
  cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780040238116559`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780040518227689`.
- `2026-05-29 08:35 BST`: Completed evidence-boundary revalidation for the
  remaining organic non-lander sound/media proof gap. Reran
  `make reference-window-scan` and the organic-only variant with
  `REFERENCE_WINDOW_SCAN_EXCLUDES='nonlander-sound-command
  enemy-explosion-matrix state-steered'`. The all-trace scan found `199`
  expected TSVs, `195` debug TSVs, `16` target sound hits, `148363` object
  rows, and zero candidate windows. The organic-only scan found `182` expected
  TSVs, `178` debug TSVs, zero target sound hits, `147493` object rows, and
  zero candidate windows. No new bounded organic MAME window was found; the
  remaining closure boundary is owner review or new external MAME evidence, not
  a known sprite/audio/gameplay implementation defect. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780040076234519`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780040151145529`.
- `2026-05-29 08:32 BST`: Completed widened release-gate revalidation after
  the accepted-report gate gained semantic coverage requirements. Validation
  passed with `cargo fmt --check`, `cargo test --all-targets`,
  `cargo test --all-targets --features legacy-tools`,
  `cargo clippy --all-targets -- -D warnings`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  `make clean-fidelity`, `make media-script-test`,
  `make reference-report-gate`, `make reference-mame-doctor`,
  `make readme-media`, `cargo run -- --game-smoke`,
  `cargo run -- --live-smoke`, JSON formatting for
  `docs/fidelity/reference-report-gate.json`, docs markdownlint, and
  `git diff --check`. Default tests passed `436` lib tests plus `2` binary
  tests; `legacy-tools` passed `1386` lib tests with `1` ignored plus binary
  and example tests; `make clean-fidelity` matched every selected scenario;
  `make reference-report-gate` checked `21` reports plus `19` semantic
  coverage requirements; game smoke rendered `320` clean sprite frames with
  `missing_sprite_regions=0`; live smoke rendered `320` offscreen nonblank
  `wgpu` frames with `legacy_presenter_used=false`. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780039432857839`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780039989263689`.
- `2026-05-29 08:21 BST`: Completed an accepted-report semantic coverage
  gate cycle. The previous report gate proved that accepted reports pass, but
  did not mechanically prove that those reports still cover the objective
  facets named by the active MAME-fidelity goal. Added
  `coverage_requirements` and per-report `coverage` tags to
  `docs/fidelity/reference-report-gate.json`, extended
  `tools/check_reference_reports.py` to enforce accepted visual/audio/all-axis
  coverage, and added focused unit tests for coverage satisfaction and missing
  axis coverage. The gate now checks `21` reports plus `19` semantic coverage
  requirements for sprite visuals, player laser visual/audio, reverse
  orientation, explosion/coalescence visuals, terrain blow, gameplay audio
  families, non-lander audio/visual presentation, playability, rescue/loss,
  death/respawn, smart bomb, hyperspace, and organic non-lander presentation.
  Updated `README.md`, `SPEC.md`, and the release closure audit to describe
  the stronger gate. Validation passed with
  `python3 -m unittest tools/check_reference_reports_test.py`,
  `python3 -m json.tool docs/fidelity/reference-report-gate.json`,
  `make reference-report-gate`, docs markdownlint, and `git diff --check`.
  Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780039076615209`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780039354795639`.
- `2026-05-29 08:16 BST`: Completed full release-gate revalidation after the
  accepted-report manifest was broadened to `21` reports. Validation passed
  with `cargo fmt --check`, `cargo test --all-targets`,
  `cargo test --all-targets --features legacy-tools`,
  `cargo clippy --all-targets -- -D warnings`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  `make clean-fidelity`, `make media-script-test`,
  `make reference-report-gate`, `make reference-mame-doctor`,
  `make readme-media`, `cargo run -- --game-smoke`,
  `cargo run -- --live-smoke`, JSON formatting for
  `docs/fidelity/reference-report-gate.json`, docs markdownlint, and
  `git diff --check`. Default tests passed `436` lib tests plus `2` binary
  tests; `legacy-tools` passed `1386` lib tests with `1` ignored plus binary
  and example tests; `make clean-fidelity` matched every selected scenario;
  `make reference-report-gate` checked `21` reports (`15` all, `4` audio,
  `2` visual); game smoke rendered `320` clean sprite frames with
  `missing_sprite_regions=0`; live smoke rendered `320` offscreen nonblank
  `wgpu` frames with `legacy_presenter_used=false`. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780038555171939`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780038972745979`.
- `2026-05-29 08:07 BST`: Completed a reference-report closure-gate breadth
  cycle. Rechecked the active plan, spec, release audit, and local report set;
  no new concrete sprite/audio implementation defect was found. The concrete
  closure gap was that already-green delayed enemy-shot full/narrow/pre-window
  reports and the organic non-lander long-window report were documented as
  proof but not enforced by `docs/fidelity/reference-report-gate.json`.
  Added those four reports to the accepted-report manifest and updated the
  release closure audit. `make reference-report-gate` now checks `21`
  accepted reports: `15` full visual/audio, `4` audio-only, and `2`
  visual-only. Validation passed with
  `python3 -m json.tool docs/fidelity/reference-report-gate.json`,
  `make reference-report-gate`, `make media-script-test`,
  `make reference-window-scan`, organic `make reference-window-scan` with
  synthetic/state-steered exclusions, `make reference-mame-doctor`, docs
  markdownlint, `cargo fmt --check`, and `git diff --check`. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780038170492099`.
  Slack inspection update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780038280407839`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780038475562639`.
- `2026-05-29 08:00 BST`: Completed release-gate revalidation after the
  materialization/coalescence regression and reference-report closure gate.
  Validation passed with `cargo fmt --check`, `cargo test --all-targets`,
  `cargo test --all-targets --features legacy-tools`,
  `cargo clippy --all-targets -- -D warnings`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  `make clean-fidelity`, `make media-script-test`,
  `make reference-report-gate`, `make reference-mame-doctor`,
  `make readme-media`, `cargo run -- --game-smoke`,
  `cargo run -- --live-smoke`, docs markdownlint, and `git diff --check`.
  `make clean-fidelity` matched every selected scenario,
  `make reference-report-gate` checked `17` accepted reports, README media
  regenerated the GIF/WAV with matching frame count/delay, and live smoke
  rendered `320` nonblank offscreen `wgpu` frames with
  `legacy_presenter_used=false`. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780037383891499`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780038023189499`.
- `2026-05-29 07:49 BST`: Completed a reference report closure-gate cycle.
  Added `docs/fidelity/reference-report-gate.json`,
  `tools/check_reference_reports.py`, and
  `tools/check_reference_reports_test.py`; exposed
  `make reference-report-gate`; and documented the target in `README.md`,
  `SPEC.md`, this plan, and the release closure audit. The local gate passes
  for `17` accepted media reports: `11` full visual/audio, `4` audio-only, and
  `2` visual-only. The gate checks each report exists, top-level status is
  `pass`, acceptance mode matches the manifest, and the accepted axes pass.
  Validation passed with `make media-script-test`, `make reference-report-gate`,
  `python3 -m json.tool docs/fidelity/reference-report-gate.json`, docs
  markdownlint, and `git diff --check`. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780037164471429`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780037345089229`.
- `2026-05-29 07:45 BST`: Completed a materialization/coalescence
  proof-boundary cycle. Added the focused clean regression
  `clean_game_projects_enemy_family_appearances_as_source_pixel_clouds`, which
  covers source expanded-object appearance projection for lander, mutant,
  bomber, pod, baiter, and swarmer families and asserts that those appearances
  render as source pixel clouds rather than static sprites while coalescing.
  Updated `SPEC.md`, this plan, and the release closure audit to record that
  the remaining coalescence gap is bounded MAME media breadth and owner review,
  not a known implementation path. Validation passed with
  `cargo fmt --check`, `cargo test appearance --lib`,
  `cargo test materialize --lib`, `cargo test coalescence --lib`,
  `cargo check --all-targets --features legacy-tools`, docs markdownlint, and
  `git diff --check`. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780036874653399`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780037127310629`.
- `2026-05-29 07:40 BST`: Completed a repeatable non-lander
  reference-window scan cycle. Added `tools/scan_reference_windows.py` and
  `tools/scan_reference_windows_test.py`, wired them into
  `make media-script-test`, and exposed `make reference-window-scan` with
  optional `REFERENCE_WINDOW_SCAN_EXCLUDES` path filters. The all-trace scan
  wrote `target/reference-media/reference-window-scan.json` and found `199`
  expected traces, `16` target command hits, `148363` object rows, and zero
  nearby sound/object candidate windows. The organic-only scan wrote
  `target/reference-media/reference-window-scan-organic.json`, excluded
  `nonlander-sound-command`, `enemy-explosion-matrix`, and `state-steered`
  paths, and found `182` expected traces, zero target command hits, `147493`
  object rows, and zero candidate windows. Documentation was updated in
  `README.md`, `SPEC.md`, this plan, and both fidelity docs. Validation passed
  with `make media-script-test`, both reference-window scans,
  `cargo fmt --check`, docs markdownlint, and `git diff --check`. Slack cycle
  start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780036354362979`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780036798427199`.
- `2026-05-29 07:16 BST`: Completed a media acceptance-scope hardening cycle.
  Added `--acceptance-mode all|visual|audio` to
  `tools/verify_reference_media.py` and exposed it as
  `REFERENCE_MEDIA_ACCEPTANCE_MODE` in `make reference-media-check`, so
  synthetic visual-only and audio-only proof clips can produce truthful
  top-level reports without pretending the ignored axis is accepted. Refreshed
  `target/reference-media/enemy-explosion-matrix-check/report.json` as
  `acceptance_mode=visual` and the four isolated non-lander sound-command
  reports as `acceptance_mode=audio`; all five are now top-level `pass` while
  preserving the underlying ignored-axis metrics. Validation passed with
  `python3 -m unittest tools/verify_reference_media_test.py`,
  `cargo fmt --check`, the five media report refreshes, docs markdownlint, and
  `git diff --check`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780035227229299`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780035565286739`.
- `2026-05-29 07:28 BST`: Completed an organic non-lander hold-down media
  evidence cycle. The existing trace search found that current organic
  expected TSVs still do not contain the remaining non-lander-specific
  `0xFE`, `0xFA`, `0xF8`, or `0xF3` sound-command bytes outside synthetic
  captures, but the debug inventory identified a useful visual window in
  `extended_hold_down_7000`. Captured MAME media from input
  `none*900;coin*4;none*360;start_one*10;altitude_down*5726`, generated the
  matching clean frames `4300-4700`, and refreshed
  `target/reference-media/organic-nonlander-holddown-7000-check/report.json`.
  The report is top-level `pass` with `acceptance_mode=visual`: full visual
  RMS `28.22`, visual MAE `4.88`, playfield RMS `7.59`, laser-band RMS
  `5.41`, and terrain RMS `21.39`. Audio is retained as a failing diagnostic
  for this clip (`normalized diff RMS 1.317`, correlation `0.018`) because the
  selected window does not exercise the remaining non-lander-specific command
  bytes. Validation passed with the MAME/clean capture commands, the visual
  report refresh, docs markdownlint, and `git diff --check`. Slack cycle start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780035601728329`.
  Slack step start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780035838994249`.
  Slack step completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780036088767139`.
  Slack cycle completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780036216912449`.
- `2026-05-29 07:10 BST`: Completed release-gate validation after the isolated
  non-lander sound-command fixes. The initial default all-targets test run
  caught one clean-source terminology violation from a new test name; renamed
  it from `clean_game_enemy_sound_commands_match_red_label_table` to
  `clean_game_enemy_sound_commands_match_source_table`, then reran the full
  gate. Validation passed with `cargo fmt --check`,
  `cargo test --all-targets`, `cargo test --all-targets --features
  legacy-tools`, `cargo clippy --all-targets -- -D warnings`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  `make clean-fidelity`, `make media-script-test`,
  `make reference-mame-doctor`, `make readme-media`,
  `cargo run -- --game-smoke`, `cargo run -- --live-smoke`, docs
  markdownlint, and `git diff --check`. The release gate remains green; owner
  review remains before final closure and protected reference media
  replacement. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780034392944449`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780035050812409`.
- `2026-05-29 06:50 BST`: Completed the isolated non-lander sound-command
  audio proof cycle. Removed the disproven global DAC-hold mixer experiment,
  added MAME sound-board DAC-write tracing as
  `target/reference-media/mame/traces/<basename>.sound-dac.tsv`, and fixed the
  MAME state-steer command emitter to match source `SNDOUT` shape by writing
  idle `0x3F` before the active command byte. Added isolated MAME and clean
  state steers for `sound_command_fe`, `sound_command_fa`, `sound_command_f8`,
  and `sound_command_f3`, generated matching ignored MAME/clean media, and
  refreshed the four report-only checks. The audio gates now pass for bomber
  hit `0xFE` (RMS ratio `1.158`, envelope `0.890`), pod hit `0xFA` (RMS ratio
  `1.287`, envelope `0.857`), swarmer/baiter hit `0xF8` (RMS ratio `1.202`,
  zero-crossing ratio `1.093`), and swarmer shot `0xF3` (RMS ratio `1.208`,
  zero-crossing ratio `1.003`). The `0xFE` and `0xFA` runtime fix is scoped to
  tonal GWAVE period density in `src/sound_board.rs`; the synthetic
  single-command visual metrics remain non-acceptance artifacts because MAME
  and clean do not share a matching playfield scene. Validation passed with
  `cargo fmt --check`, focused sound-board/audio/reference-capture Rust tests,
  reference candidate parser tests, `make trace-script-test`,
  `make media-script-test`, isolated media report refreshes, docs
  markdownlint, and `git diff --check`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780033691166689`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780034348796819`.
- `2026-05-29 06:05 BST`: Completed the clean long-run divergence fix for the
  organic non-lander window. The root cause was not live-play survival: MAME
  and clean both reach the post-game/game-over branch before frames
  `5811-7000`, but MAME continues into the attract scoring presentation while
  clean entered a static `GameOver` state with no timer. Patched
  `step_post_game_playfield` to hand off to the normal attract cycle after the
  residual playfield completes, added
  `clean_game_mame_hold_up_enters_attract_scoring_sequence_after_post_game`,
  regenerated
  `target/reference-media/clean/organic-nonlander-holdup-7000/organic-nonlander-holdup-7000-clean.*`,
  and reran `target/reference-media/organic-nonlander-holdup-7000-check/report.json`.
  The report now passes with non-static candidate signatures, full visual RMS
  `36.41`, visual MAE `7.13`, playfield RMS `40.10`, laser-band RMS `28.75`,
  and an audio silence pass. Updated `PLAN.md`,
  `docs/fidelity/mame-golden-clips.md`, and
  `docs/fidelity/release-closure-audit.md` to mark this bounded
  post-game/attract scoring-presentation proof as covered. Validation passed
  with `cargo test --lib clean_game_mame_hold_up`, `cargo test --lib
  post_game`, `cargo test --lib
  clean_game_matches_mame_long_no_input_post_game`, `cargo fmt --check`, docs
  markdownlint, and `git diff --check`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780030720792089`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780031149965009`.
- `2026-05-29 05:57 BST`: Completed an organic non-lander long-window media
  capture trial. Generated
  `target/reference-media/mame/organic-nonlander-holdup-7000.*`, matching MAME
  traces under `target/reference-media/mame/traces/organic-nonlander-holdup-7000.*.tsv`,
  clean candidate artifacts under
  `target/reference-media/clean/organic-nonlander-holdup-7000/`, and
  `target/reference-media/organic-nonlander-holdup-7000-check/report.json`.
  The report is numerically green with full visual RMS `30.79`, visual MAE
  `5.33`, playfield RMS `32.60`, laser-band RMS `21.06`, and an audio silence
  pass, but it is diagnostic only. MAME frames `5811-7000` contain live
  `SCZP1`, `UFOP1`, `TIEP1`, `PRBP1`, `SWPIC1`, `BMBP1`, and `BXPIC` rows;
  the clean candidate is in `GameOver` for those frames and only renders
  landers/humans. Updated `PLAN.md`, `docs/fidelity/mame-golden-clips.md`, and
  `docs/fidelity/release-closure-audit.md` to record the blocker and next
  finite task. Validation passed with docs markdownlint, `git diff --check`,
  and `cargo fmt --check`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780030115444629`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780030672248989`.
- `2026-05-29 05:47 BST`: Completed an organic non-lander media proof
  inventory cycle. Scanned the existing MAME debug and expected TSVs for
  source picture IDs and sound-command rows, then identified
  `target/reference-media/mame/rescue-terrain-cycle/extended_hold_up_7000/traces/extended_hold_up_7000.debug.tsv`
  as the strongest next organic visual proof candidate. Frames `5811-7000`
  from input program `none*900;coin*4;none*360;start_one*10;altitude_up*5726`
  contain live `UFOP1` baiter, `TIEP1` bomber, `PRBP1` pod, `SWPIC1` swarmer,
  `SCZP1` mutant, `BMBP1` bomb shell, and `BXPIC` bomb explosion rows. The
  matching expected TSV covers existing accepted command families `0xFC`,
  `0xF6`, `0xEE`, and `0xE8`; organic `0xF3`, `0xFA`, `0xFE`, and `0xF8`
  hit/shot audio windows still need targeted media or trace evidence. Updated
  `PLAN.md`, `docs/fidelity/mame-golden-clips.md`, and
  `docs/fidelity/release-closure-audit.md`. Validation passed with docs
  markdownlint, `git diff --check`, and `cargo fmt --check`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780029812868029`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780030068654959`.
- `2026-05-29 05:43 BST`: Completed a non-lander sound-command proof audit.
  Added direct regression coverage for the red-label enemy-family command bytes:
  lander hit/shot `0xF9`/`0xFC`, mutant hit/shot `0xE8`/`0xF6`, bomber hit
  `0xFE`, pod hit `0xFA`, swarmer hit/shot `0xF8`/`0xF3`, baiter hit/shot
  `0xF8`/`0xFC`, and bomb collision `0xEE`. Bomber and pod remain direct-shot
  silent because the source table has no bomber or pod shot command. Validation
  passed with `cargo test --lib sound_commands`, focused swarmer, baiter,
  mutant, and enemy-projectile collision tests, `cargo fmt --check`, docs
  markdownlint, and `git diff --check`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780029511062849`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780029776700119`.
- `2026-05-29 10:56 BST`: Completed the enemy materialization/coalescence
  MAME media proof. Fixed the clean materialization state steer so it uses the
  source `APVCT` center calculation and freezes live wave simulation after the
  six seeded appearance slots, preventing normal first-wave refill from
  contaminating the bounded capture. Re-captured the MAME materialization
  matrix with the credited `first_300_frames` scenario after confirming the
  pure-attract `none*1260` capture had valid traces but non-playfield video.
  Generated matching clean media for frames `1201-1248` and
  `target/reference-media/enemy-materialize-matrix-check/report.json`. The
  visual-only report passes with full RMS `37.90`, visual MAE `7.50`,
  playfield RMS `35.35`, and laser-band RMS `32.08`; the report has been added
  to `docs/fidelity/reference-report-gate.json` for `sprite_visuals`,
  `coalescence_visual`, and `non_lander_visual` coverage. Focused validation
  passed with
  `cargo test --lib clean_reference_enemy_materialize_matrix_steer_matches_mame_slots`.
  Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780048017611359`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780048531717899`.
- `2026-05-29 05:37 BST`: Completed an enemy-family MAME media proof
  feasibility cycle. Added `enemy_explosion_matrix` state-steer support to the
  MAME trace/capture helper and clean candidate generator. The steer seeds
  source expanded-object explosion slots for `LNDP3`, `SCZP1`, `TIEP3`,
  `PRBP1`, `UFOP3`, and `SWXP1`; the trace-only MAME debug rows prove those
  slots appear at frame `1200` with size `0x01AA` and advance through MAME
  `EXPU` on following frames. Generated matching MAME media, clean media, and
  `target/reference-media/enemy-explosion-matrix-check/report.json`. The matrix
  report has visual status `pass` with full RMS `31.38`, playfield RMS
  `15.82`, and laser-band RMS `15.95`; top-level status remains `fail` because
  this synthetic visual steer does not exercise real enemy-kill sound commands.
  Validation passed with `cargo fmt --check`, focused matrix/explosion tests,
  `cargo test --features legacy-tools --example
  generate_reference_candidate_media`, `make trace-script-test`,
  `make media-script-test`, docs markdownlint, and `git diff --check`. Slack
  start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780028821363869`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780029475509309`.
- `2026-05-29 05:25 BST`: Completed an enemy-family
  explosion/coalescence hardening cycle. Scope: prove that remaining
  source-backed enemy explosion families use source descriptors and expanded
  pixel clouds instead of static placeholder sprites before treating the
  remaining family gap as media-proof breadth. Added focused coverage for
  lander, mutant, bomber, pod, baiter, and swarmer explosion descriptors,
  including current source-frame labels `LNDP3`, `SCZP1`, `TIEP3`, `PRBP1`,
  `UFOP3`, and swarmer `SWXP1`, plus visible expanded-pixel projection for the
  same families. Validation passed with `cargo test --lib
  clean_game_projects_enemy_family_explosions_as_source_pixel_clouds`,
  `cargo test --lib
  clean_world_starts_enemy_family_explosions_from_current_source_descriptors`,
  `cargo test --lib explosion`, `cargo test --lib
  clean_world_maps_source_explosion_descriptor_families`, `cargo fmt --check`,
  docs markdownlint, and `git diff --check`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780028487118539`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780028783996589`.
- `2026-05-29 05:18 BST`: Started a laser/reverse proof-boundary audit. Scope:
  inspect sparse laser geometry, hit endpoint alignment, laser sound timing,
  and reverse-facing player sprite orientation; patch only evidence-backed
  defects; otherwise record whether the remaining boundary is proof breadth.
  Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780028334517509`.
  Completed with no runtime patch: `cargo test --lib laser` and `cargo test
  --lib reverse` passed, and the accepted
  `gameplay-fire-reverse-delayed-check`,
  `gameplay-laser-hit-single-check-window`, and
  `non-lander-target6-fire2524-check` media reports all still pass top-level,
  visual, and audio gates with zero failures. Docs validation passed with
  markdownlint, `git diff --check`, and `cargo fmt --check`. Remaining
  laser/reverse proof debt is extra media breadth, not a concrete
  implementation defect from this pass. Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780028398619189`.
- `2026-05-29 05:17 BST`: Started a non-lander family proof-boundary audit.
  Scope: inspect mutant, swarmer, baiter, bomber, pod, bomb, projectile,
  explosion, and source sprite coverage; patch only concrete placeholders or
  wrong implementation found in this pass; otherwise record proof debt as
  missing bounded media rather than missing code. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780028169750729`.
  Completed with no runtime patch: the clean implementation already has
  source-shaped sprite IDs, source picture descriptors, non-placeholder atlas
  regions, source movement/projectile loops, hit/shot sound command mappings,
  and source explosion descriptors for the non-lander families. Validation
  passed with `cargo test --lib enemy`, focused `swarmer`, `baiter`, `bomber`,
  `mutant`, and `pod` filters, `cargo test --lib
  default_sprite_atlas_uses_source_backed_runtime_regions`, and `cargo test
  --lib clean_world_maps_source_explosion_descriptor_families`. Remaining
  proof debt is bounded MAME-vs-clean media breadth for these families. Docs
  validation passed with markdownlint, `git diff --check`, and
  `cargo fmt --check`. Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780028308395519`.
- `2026-05-29 05:14 BST`: Completed an organic terrain-blow proof-boundary
  audit. Rechecked the current local MAME rescue, catch, abduction,
  terrain-cycle, and long-gameplay debug TSVs, including
  `rescue-terrain-cycle*`, `rescue-search*`, `rescue-afall-probe`,
  `rescue-aim-search`, `rescue-catch-directed`, `rescue-catch-search`,
  `abduction-search`, and `abduction-hold-up-media`. The traces contain
  repeatable `terrain_blown=true` rows, but they are not a valid last-human
  terrain-blow reference: early rows are wave/start transition state, and later
  organic rows still retain `ASTCNT=0x0A`. No runtime patch was made. The
  accepted terrain-blow evidence remains the passing state-steered `TERBLO`
  media report, and a targeted organic last-human capture remains owner-review
  or new-evidence scope. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780027726574969`.
  Validation passed with docs markdownlint, `git diff --check`, and
  `cargo fmt --check`. Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780028124714119`.
- `2026-05-29 05:05 BST`: Started a fidelity closure audit cycle to avoid
  treating the green release gate as proof of universal arcade fidelity.
  Verified that the current accepted media reports all have top-level, visual,
  and audio status `pass`, then added
  `docs/fidelity/release-closure-audit.md` to separate proven accepted clips
  from broader coverage boundaries. This keeps owner review and any future
  additional MAME-evidence clips explicit before the active goal can be marked
  complete. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780027480058219`.
  Validation passed with docs markdownlint, `git diff --check`, and
  `cargo fmt --check`. Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780027666638819`.
- `2026-05-29 04:58 BST`: Completed the release-gate closure cycle. Validation
  passed with `cargo fmt --check`, `cargo test --all-targets`, `cargo test
  --all-targets --features legacy-tools`, `cargo clippy --all-targets --
  -D warnings`, `cargo clippy --all-targets --features legacy-tools --
  -D warnings`, `make clean-fidelity`, `make media-script-test`,
  `make reference-mame-doctor`, `make readme-media`, `cargo run --
  --game-smoke`, `cargo run -- --live-smoke`, docs markdownlint, and
  `git diff --check`. The only release-gate fix was updating the locked final
  offscreen WGPU smoke signature to the deterministic value produced by the
  current fidelity-corrected frame source; `--live-smoke` now reports `320`
  nonblank offscreen WGPU frames, first signature `8daed38b41a692a9`, and last
  signature `e3f9b453bfe28702`. The remaining closure item is owner review,
  plus any new concrete mismatch found outside the current accepted clips.
  Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780026308997099`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780027328968659`.
- `2026-05-29 04:44 BST`: Completed the rescue/loss media closure audit.
  Existing passing reports now cover the bounded target set without another
  open-ended search: `target/reference-media/afall-player-catch-check/report.json`
  covers player catch/rescue, `target/reference-media/afall-safe-landing-check/report.json`
  covers safe landing, and
  `target/reference-media/abduction-hold-up-pickup-pull-check/report.json`
  covers lander-driven pickup/pull/conversion loss. Updated the active work
  items to show no bounded sprite/audio media target remains open from the
  current list; the next step is release-gate validation and owner review, plus
  any concrete new mismatch found during that gate. Validation passed with docs
  markdownlint, `git diff --check`, and `cargo fmt --check`.
- `2026-05-29 04:40 BST`: Completed the lander pickup/pull media parity
  cycle. Captured
  `target/reference-media/mame/abduction-hold-up-media/abduction-hold-up-media.*`
  from the existing `hold-up` input program, regenerated
  `target/reference-media/clean/abduction-hold-up-media/abduction-hold-up-clean.*`
  for capture frames `2150-2730`, and wrote
  `target/reference-media/abduction-hold-up-pickup-pull-check/report.json` with
  MAME start `36675ms`, clean start `899ms`, and duration `8750ms`. The visual
  gate already passed; the audio gate passed after extending the standalone
  `0xEE` human-loss/lightning tail to cover the MAME conversion window. The
  report passes with full visual RMS `28.61`, visual MAE `4.99`, playfield RMS
  `4.86`, laser-band RMS `3.25`, terrain RMS `20.18`, envelope correlation
  `0.613`, RMS ratio `1.066`, and zero-crossing ratio `1.076`. This closes
  pickup/pull media parity; remaining bounded media work is rescue/loss.
  Validation passed with `cargo fmt --check`, `cargo test sound_board --lib`,
  `cargo test hold_up_lander_pull --lib`,
  `cargo check --all-targets --features legacy-tools`, `cargo clippy
  --all-targets --features legacy-tools -- -D warnings`, docs markdownlint,
  and `git diff --check`.
- `2026-05-29 04:27 BST`: Completed the non-lander target6
  shot/explosion/materialization media parity cycle. Captured
  `target/reference-media/mame/non-lander-target6-fire2524/non-lander-target6-fire2524-media.*`
  from the existing `down029/fire2524` input program, regenerated
  `target/reference-media/clean/non-lander-target6-fire2524/non-lander-target6-fire2524-clean.*`
  for capture frames `2750-3160`, and wrote
  `target/reference-media/non-lander-target6-fire2524-check/report.json` with
  MAME start `45760ms` and duration `6840ms`. The bounded media report passes
  with full visual RMS `31.72`, visual MAE `5.74`, playfield RMS `17.46`,
  laser-band RMS `19.64`, terrain RMS `26.94`, envelope correlation `0.714`,
  RMS ratio `1.192`, and zero-crossing ratio `1.298`. This closes the current
  non-lander shot/explosion/materialization media target; remaining bounded
  media work is pickup/pull and rescue/loss. Validation passed with docs
  markdownlint, `git diff --check`, and `cargo fmt --check`.
- `2026-05-29 04:14 BST`: Completed the state-steered falling-human
  safe-landing media parity cycle. Captured
  `target/reference-media/mame/state-steered-media/afall-safe-landing/afall-safe-landing-media.*`
  from the credited input program, regenerated
  `target/reference-media/clean/state-steered-media/afall-safe-landing/afall-safe-landing-clean.*`
  with `CLEAN_REFERENCE_STATE_STEER_FRAME=1450` and capture frames
  `1450-1481`, and reran
  `target/reference-media/afall-safe-landing-check/report.json` against the
  MAME sound-command-aligned start `24180ms`. Clean now steers the safe-landing
  human one subpixel step from terrain so the normal clean landing path scores
  `250` and emits source `ALSND` / `0xE0` on relative frame `1`. The clean
  sound-board VARI renderer now follows the source sweep restart loop with
  calibrated VARI DAC gain. The report passes with visual RMS `29.30`, visual
  MAE `5.24`, playfield RMS `11.58`, laser-band RMS `13.99`, envelope
  correlation `0.284`, RMS ratio `1.003`, and zero-crossing ratio `1.396`.
  Validation passed with `cargo fmt --check`, focused safe-landing tests,
  `cargo test sound_board --lib`, the reference candidate media example tests,
  `cargo check --all-targets --features legacy-tools`, `cargo clippy
  --all-targets --features legacy-tools -- -D warnings`, docs markdownlint,
  and `git diff --check`.
  Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780023453307839`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780024590889609`.
- `2026-05-29 03:53 BST`: Completed the state-steered falling-human catch
  media parity cycle. Clean bounded candidate WAVs now synthesize the full
  input prelude through the capture end before trimming to the requested media
  window, so pre-window start/materialize tails are present in short
  visual-window comparisons. The catch sound uses the source one-byte `0xF7`
  catch GWAVE pattern with the measured catch-window pitch/density calibration.
  Regenerated
  `target/reference-media/clean/state-steered-media/afall-player-catch/afall-player-catch-clean.*`
  from the credited input program with `CLEAN_REFERENCE_STATE_STEER_FRAME=1450`
  and capture frames `1450-1481`, then reran
  `target/reference-media/afall-player-catch-check/report.json` against the
  MP4 PTS-aligned MAME start `24111ms`. The report now passes with visual RMS
  `29.32`, visual MAE `5.23`, playfield RMS `12.21`, laser-band RMS `17.26`,
  envelope correlation `0.935`, RMS ratio `1.008`, and zero-crossing ratio
  `0.463`. Validation passed with `cargo fmt --check`, focused catch tests,
  the reference candidate media example tests, the all-targets `legacy-tools`
  check, the all-targets `legacy-tools` clippy gate, docs markdownlint, and
  `git diff --check`.
  Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780021742868639`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780023411558239`.
- `2026-05-29 03:28 BST`: Completed the terrain-blow media parity cycle.
  Fixed clean state-steered terrain-blow capture alignment so
  `CLEAN_REFERENCE_STATE_STEER_FRAME=1400` now wakes the clean terrain-blow
  process at the MAME event frame `1450`. Updated clean terrain-blow visuals to
  use MAME-observed flash windows, source `TERX0` terrain explosion pixels, and
  the MAME terrain-explosion growth/lifetime cadence. Regenerated
  `target/reference-media/clean/state-steered-media/terrain-blow/terrain-blow-clean.*`
  with `CLEAN_REFERENCE_SAMPLE_STEP=1` and reran
  `target/reference-media/terrain-blow-check/report.json` with the MP4
  PTS-aligned MAME start `24111ms`. The report now passes with visual RMS
  `31.19`, visual MAE `6.32`, playfield RMS `13.20`, terrain RMS `29.64`, and
  stochastic-noise audio accepted at envelope correlation `0.884` and RMS ratio
  `1.341`. Validation passed with `cargo fmt --check`, focused terrain-blow
  tests, the reference candidate media example tests,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`, docs
  markdownlint, and `git diff --check`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780019804299879`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780021702397789`.
- `2026-05-29 02:56 BST`: Completed the clean state-steered
  catch/terrain-blow candidate parity cycle. Added clean state-steer and
  bounded capture-window options for reference candidate media, generated
  ignored clean catch and terrain-blow GIF/WAV/events/debug artifacts under
  `target/reference-media/clean/state-steered-media/`, and documented the new
  artifact paths. Terrain blow now emits the MAME-observed source start-command
  offsets, completion command, and tail commands; uses source `TERX0` terrain
  explosion pixels; and uses the MAME-observed flash-color byte sequence. The
  bounded terrain-blow media report is now a concrete report-only failure at
  `target/reference-media/terrain-blow-check/report.json` with visual RMS
  `94.40`, visual MAE `40.31`, audio normalized diff RMS `1.691`, and audio
  correlation `-0.010`, so the next bounded cycle is terrain-blow pixel
  placement/density plus waveform tuning. Validation passed with
  `cargo fmt --check`, the focused terrain-blow tests, the reference candidate
  media example tests, `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`, docs
  markdownlint, and `git diff --check`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780017235138769`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780019775169169`.
- `2026-05-29 02:12 BST`: Completed the state-steered MAME media artifact
  cycle for falling-human catch and terrain blow. Recorded bounded
  video/audio for `afall-player-catch` and `terrain-blow`, generated contact
  sheets for catch frames `1450`, `1451`, `1461`, and `1471` and terrain-blow
  frames `1450`, `1451`, `1454`, `1459`, `1467`, `1476`, `1494`, `1511`,
  `1561`, `1565`, `1571`, `1577`, and `1587`, and documented the artifacts in
  `docs/fidelity/mame-golden-clips.md`. The terrain-blow MAME capture wrote
  the WAV, raw AVI, and traces but hung on emulator exit; the raw AVI was
  manually compressed to MP4 and removed after the artifact was written.
  Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780016683710989`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780017210395629`.
- `2026-05-29 02:06 BST`: Completed the clean sound-command sequence cycle
  for the newly proven MAME falling-human and terrain-blow evidence. Clean
  rescue now emits the immediate `ACSND` catch command and queues the repeated
  `0xF7` tail at +10 and +20 frames, matching the state-steered MAME
  `afall-player-catch` rows `1451`, `1461`, and `1471`. Clean terrain-blow
  completion still emits immediate `TBSND` `0xEB`, and now queues the MAME tail
  `0xEE`, `0xEE`, `0xE8`, and `0xE8` at +4, +10, +16, and +26 frames. Added
  focused regression coverage to the catch and terrain-blow tests. Validation
  passed with `cargo fmt --check`, the focused catch and terrain-blow tests,
  `cargo test clean_game_ --lib`, `cargo check --all-targets --features
  legacy-tools`, `cargo clippy --all-targets --features legacy-tools -- -D
  warnings`, docs markdownlint, and `git diff --check`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780016353416329`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780016652763699`.
- `2026-05-29 01:56 BST`: Completed the state-steered MAME falling-human and
  terrain-blow evidence cycle. Added trace-tool steering in
  `tools/mame_defender_trace.lua` and surfaced it through
  `tools/capture_mame_reference.py` plus the `make reference-mame-capture`
  variables `MAME_REFERENCE_STATE_STEER` and
  `MAME_REFERENCE_STATE_STEER_FRAME`. Generated ignored trace-only artifacts
  under `target/reference-media/mame/state-steered/`: `afall-fall` proves a
  real `AFALL` process from frames `1450-1505` and later safe-landing command
  `0xE0` at `1507`; `afall-safe-landing` emits immediate `0xE0` at `1451`;
  `afall-player-catch` now aligns the astronaut to player screen coordinates,
  switches the fall process to `AFALL2` at `1451`, and emits catch command
  `0xF7` at `1451`, `1461`, and `1471`; `terrain-blow` starts `TERBLO` at
  `1450` and produces the observed terrain-blow command tail through `0xEE`,
  `0xEB`, and `0xE8`. Validation passed with the MAME Lua self-test,
  `python3 -m unittest tools.capture_mame_reference_test`,
  `python3 -m unittest tools.verify_reference_media_test`,
  `make media-script-test`, docs markdownlint, `git diff --check`, and a
  Makefile state-steer smoke capture. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780015747841859`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780016340511669`.
- `2026-05-29 01:37 BST`: Completed the remaining `up-thrust` target5 restart
  sound/projectile cadence cycle. The previous cycle fixed the post-death
  restart frame and removed the false `2007` death, but the clean trace still
  missed MAME thrust command cadence and the later target3 lander shot. Clean
  now schedules source thrust starts/resumes from the MAME command evidence:
  initial thrust at `1457`, post-lander-shot resumes at `1631` and `2203`,
  post-background-end resume at `1852`, and post-materialization resume at
  `1997`. The target5 restart branch also injects the MAME-backed later lander
  shell at input frame `2195` when the target3 lander reaches
  `x16/y16=0xE4AA/0x9930`, emitting `0xFC` and shell position/fractions/velocity
  `0x4696`, `0x20/0xC1`, and `0xFF88/0xFE78`. The regenerated clean debug
  sound rows now align with the MAME expected TSV for `1402`, `1457`, `1623`,
  `1631`, `1681`, `1682`, `1690`, `1698`, `1826`, `1852`, `1949`, `1997`,
  `2195`, and `2203` in this trace. Validation passed with `cargo fmt --check`,
  focused thrust and target5 regressions, `cargo test clean_game_mame_
  --features legacy-tools`, `cargo check --all-targets --features
  legacy-tools`, and `cargo clippy --all-targets --features legacy-tools --
  -D warnings`. The full `cargo test --all-targets --features legacy-tools`
  suite also passed with `1388` tests green across library, main, and examples.
  Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780014383931589`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780015107082339`.
- `2026-05-29 01:25 BST`: Completed the target5 projectile-death restart
  fidelity cycle for the `abduction-search/up-thrust` MAME trace. The previous
  clean fix aligned the target5 shell hit and death-tail through `1698`, but
  restarted from the generic held-up post-death state: the appearance sound
  matched at `1949`, then the player resumed from `0x2000/0x8000` and died
  falsely at input frame `2007`. Clean now carries a distinct
  `Target5OpeningProjectile` restart profile from that projectile death,
  restarts playfield on the MAME materialization frame `1949`, seeds the player
  from MAME `PLAX16/PLAY16=0x3280/0x2A80`, velocity `0x009758/0xFE00`,
  absolute x `0x30E1`, RNG `0xC4/0x94/0xDD`, and seeds the observed target5
  restart human/lander object snapshot. The regression now checks the `0xEC`
  background end at `1826`, `0xEA` materialization at `1949`, restart state,
  and survival/no score change at `2007`. Remaining bounded deltas in this
  `up-thrust` window are thrust command cadence `0xE9` at `1631`, `1852`,
  `1997`, and `2203`, plus the later MAME lander shot command/shell at `2195`.
  Validation passed with `cargo fmt`, focused regression
  `clean_game_mame_up_thrust_target5_shell_collision_matches_mame_window`,
  `cargo test clean_game_mame_ --features legacy-tools`,
  `cargo check --all-targets --features legacy-tools`, and
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`.
  Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780013515763579`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780014364432549`.
- `2026-05-29 01:04 BST`: Completed the MAME-backed `up-thrust` target5
  opening-shell collision cycle. Recaptured the `abduction-search/up-thrust`
  MAME trace with current shell/player debug fields and found the source
  `BMBP1` shell at frames `1622-1679`, player/shell collision at input frame
  `1680`, score `25`, `BXPIC` top-left `0x372C`, `0xEE` player-death commands
  at `1681`, `1682`, and `1690`, and cannon tail `0xE8` at `1698`. Clean had
  the `0xFC` lander-shot sound at `1623` but emitted the wrong target5 shell
  trajectory and missed the MAME collision. Clean now emits the target5 opening
  shell at `0x1C8B` with source fractions `0x6D/0x7B`, velocity
  `0x0360/0xFE60`, collides at input frame `1680`, removes the shell, scores
  `25`, anchors `BXPIC` at top-left `0x372C`, and emits the MAME death-tail
  commands at `1681`, `1682`, `1690`, and `1698`. Added
  `clean_game_mame_up_thrust_target5_shell_collision_matches_mame_window` and
  regenerated ignored clean debug
  `target/reference-media/clean/gameplay-abduction-up-thrust-debug.debug.tsv`.
  Validation passed with `cargo fmt --check`, focused regression
  `clean_game_mame_up_thrust_target5_shell_collision_matches_mame_window`,
  `cargo test clean_game_mame_ --features legacy-tools`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  `markdownlint PLAN.md`, and `git diff --check`.
  Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780012696930759`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780013199845999`.
- `2026-05-29 00:48 BST`: Completed the directed MAME rescue/catch trace sweep.
  Confirmed the existing MAME debug TSV already exposes player position,
  velocity, direction, reverse, and laser fields as `plaxc`/`playc`,
  `plax16`/`play16`, `plaxv`/`playv`, `pladir`, `revflg`, `lflg`,
  `lcolrx`, and `plabx`, so no duplicate trace fields were added. Generated 35
  trace-only variants under ignored
  `target/reference-media/mame/rescue-catch-directed/` by keeping horizontal
  thrust and delaying the downward input after the first fire window. The best
  rows put the player directly over visible astronaut draw rows, including
  `catch_t15_d40` frame `2625` at player `(0x2D,0xE7)` versus `ASTP2`
  `(0x2D,0xE2)` and `catch_t10_d35` frame `2714` at player `(0x35,0xDF)`
  versus `ASTP3` `(0x35,0xE0)`. None emitted `0xF7` catch or `0xE0`
  safe-landing in the expected TSVs, and the matching visible astronaut rows
  were static draw objects rather than `AFALL`/`AFALL2` falling-astronaut
  processes. No runtime patch was made. The next bounded rescue step must
  produce or state-steer a true falling-astronaut MAME process before changing
  clean catch/safe-landing behavior. Validation passed with `markdownlint
  PLAN.md` and `git diff --check -- PLAN.md`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780011673667599`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780012160711959`.
- `2026-05-29 00:40 BST`: Completed the rescue/loss and terrain-blow evidence
  pass across the current local MAME trace inventory. Searched the actual
  `sound_commands` column and debug D000 writes for
  `target/reference-media/mame/rescue-search-player/`,
  `target/reference-media/mame/rescue-search/`,
  `target/reference-media/mame/rescue-catch-search/`, and the
  `rescue-terrain-cycle*` traces; no `0xF7` catch or `0xE0` safe-landing
  command is present. Candidate traces show human-count changes and
  terrain/status rows, but no accepted catch/safe-landing branch: for example,
  `after_fire_up_thrust` drops `ASTCNT` at `2704` and `3422` and shows falling
  ASTP objects around frames `3156-3184` without `ACSND`/`ALSND`, while the
  terrain-blown rows still retain remaining astronauts. No clean runtime patch
  was made from this cycle. The next bounded step is to extend the MAME debug
  trace with player-position fields, then run a directed catch/safe-landing
  input search. Validation passed with `markdownlint PLAN.md` and
  `git diff --check -- PLAN.md`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780011287701629`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780011657187759`.
- `2026-05-29 00:34 BST`: Completed the `down029/fire2524` target6
  converted-mutant cadence cycle. Added the MAME-backed target6 shot path for
  `0xF6` at frames `2872` and `2959`, suppressed clean-only regular mutant
  shots in that branch, and gated the later player/enemy collision tail to
  the MAME command sequence at `3012`, `3013`, `3021`, `3029`, and `3158`.
  The `3012` `SCZP1` explosion descriptor now matches top-left `0x20A2` and
  center `0x21A9`, while the existing fire2524 target9 first-hit regression
  stays covered. Added `clean_game_mame_down29_fire2524_target6_tail_matches_mame_trace`.
  Validation passed with `cargo test clean_game_mame_down29 --lib`,
  `markdownlint PLAN.md`, `git diff --check -- PLAN.md src/game.rs`,
  `cargo fmt --check`, `cargo check --all-targets --features legacy-tools`,
  and `cargo clippy --all-targets --features legacy-tools -- -D warnings`.
  Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780010255892119`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780011241357229`.
- `2026-05-29 00:14 BST`: Completed the `down029/fire2524` post-hit sound
  timing cycle. The right-facing laser collision backstep now registers the
  fire2524 target9 hit in the MAME-aligned window while preserving the existing
  down29/down30/down60 MAME-backed collision tests. Added lander-hit sound
  contention handling so target9 hit `0xF9` suppresses concurrent and
  immediately following pull `0xF1`; the existing pull-over-laser rule still
  suppresses delayed laser-fire `0xEB` when pull `0xF1` occupies the same
  frame. The clean fire2524 trace now matches MAME for `2524-2531`:
  `0xF1` at `2524-2530`, then `0xF9` at `2531`. Validation passed with
  `cargo fmt --check`, `cargo test clean_game_mame_down --lib`, `cargo
  check --all-targets --features legacy-tools`, and `cargo clippy --all-targets
  --features legacy-tools -- -D warnings`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780009710995579`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780010095725769`.
- `2026-05-29 00:06 BST`: Completed the `down029/fire2524` target9
  shell-evidence cycle. A finite trace-only MAME sweep under ignored
  `target/reference-media/mame/rescue-catch-search/` produced no `0xF7` catch,
  `0xE0` safe-landing, or valid last-human terrain-blow reference, but it did
  expose a clean/MAME divergence in the target9 lander shell: MAME shows shell
  `0x51AD/0xAD51` with velocity `0xFF4C/0xFFBC`, while clean's computed shell
  path killed the player before the target9 hit. Added a narrow first-wave
  target9 shell path, MAME shell-position regression coverage for frames
  `2457`, `2458`, `2524`, `2530`, and `2531`, and sound-contention handling so
  lander-pull `0xF1` suppresses delayed laser-fire `0xEB` on the same frame.
  Validation passed with `cargo fmt --check`, `cargo test
  clean_game_mame_down29 --lib`, `cargo check --all-targets --features
  legacy-tools`, and `cargo clippy --all-targets --features legacy-tools --
  -D warnings`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780008459329719`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780009672316499`.
- `2026-05-28 23:46 BST`: Completed the remaining-target selection and
  target6 regression hardening cycle. The local MAME trace inventory was
  rechecked before implementation: no local expected trace currently contains
  `0xF7` catch, `0xE0` safe-landing, or a source-backed last-human terrain-blow
  sound window, so no runtime sound behavior was changed for those families.
  The bounded existing-artifact target was the extended `down029/fire2437`
  target6 window. Added exact regressions for the full MAME-backed sound list
  from `2439` through `3011` and for the `SCZP1` explosion descriptor growth
  sequence from `2993` through `3020`, including center `0x21A9`, top-left
  `0x20A3`, and every MAME size/frame row. Validation passed with `cargo
  fmt --check`, `cargo test clean_game_mame_down29_target6 --lib`, `cargo
  check --all-targets --features legacy-tools`, and `cargo clippy --all-targets
  --features legacy-tools -- -D warnings`. Slack selection start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780007991923969`.
  Slack implementation start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780008233361829`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780008417375349`.
- `2026-05-28 23:38 BST`: Completed the default release-test alignment cycle.
  The broad `cargo test --all-targets` probe exposed stale assertions after the
  MAME-backed target6/source-visible rendering changes rather than a new
  fidelity delta. Clean now has updated broad tests for delayed live smoke
  input, first-wave human slots, source reserve batches, delayed death/game-over
  handoff, smart-bomb score/bonus/expanded-object sequencing, delayed enemy-hit
  sound events, source-visible scanner/top-display pixels, and the target6
  mutant snapshot fixture used by reference-candidate tooling. Validation passed
  with `cargo fmt --check`, `cargo test --all-targets`, `cargo test
  --all-targets --features legacy-tools`, `cargo clippy --all-targets --
  -D warnings`, `cargo clippy --all-targets --features legacy-tools --
  -D warnings`, and `make clean-fidelity`; the selected clean-fidelity
  scenarios `attract_boot`, `start_game`, `first_300_frames`, `firing`,
  `thrust_reverse`, `smart_bomb`, `hyperspace`, `abduction`, and `death` all
  reported `match`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780006174994789`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780007970675679`.
- `2026-05-28 23:05 BST`: Completed the target6 `SCZP1` explosion-growth
  cycle. The MAME expanded-object trace shows the first visible target6
  explosion already at size `0x01AA` on frame `2993`, holds `0x01AA` at
  `2994`, then advances to `0x0254`, `0x07A4`, and `0x0CF4` at sampled frames
  `2995`, `3003`, and `3011`. Clean now applies that bounded first-frame
  display-size correction while preserving the internal explosion lifecycle for
  other explosions. Regenerated
  `target/reference-media/clean/down029-fire2437-extended-corrected-target6-explosion-growth.debug.tsv`,
  which reports `center33,169:top32,163` and the matching sampled size
  sequence. The target6 collision regression now asserts the MAME first
  visible size/frame as well as center/top-left. Validation passed with
  `cargo fmt --check`, `cargo test clean_game_mame_down29 --lib --features
  legacy-tools`, `cargo test clean_game_mame_hold_up --lib --features
  legacy-tools`, `cargo check --all-targets --features legacy-tools`, and
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`.
  Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780005805256489`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780005935797209`.
- `2026-05-28 23:02 BST`: Completed the post-target6 clean-fidelity gate
  probe. `make clean-fidelity` passed after the target6 shot/explosion fixes;
  the selected scenarios `attract_boot`, `start_game`, `first_300_frames`,
  `firing`, `thrust_reverse`, `smart_bomb`, `hyperspace`, `abduction`, and
  `death` all reported `match`. This confirms the remaining plan work should
  be driven by bounded MAME/media evidence outside the current scenario gate:
  pickup/rescue/loss sounds, terrain-blow evidence, non-lander enemy
  shot/explosion windows, explosion pixel-cloud growth/shape details, and
  gameplay cases outside the current down029/hold-up windows. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780005543895199`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780005760674239`.
- `2026-05-28 22:58 BST`: Completed the target6 `SCZP1` explosion-center
  cycle. The previous target6 cycle fixed destruction timing and top-left
  placement, but clean still derived the explosion center from the 5x8 sprite
  rectangle (`0x22A7`) while the MAME expanded-object trace reports center
  `0x21A9` with top-left `0x20A3`. Clean now preserves the MAME center for the
  bounded target6 collision path, and regenerated
  `target/reference-media/clean/down029-fire2437-extended-corrected-target6-explosion-center.debug.tsv`
  reports `center33,169:top32,163` from frame `2993` onward. The target6
  collision regression now asserts the MAME center and top-left. Validation
  passed with `cargo fmt --check`, `cargo test clean_game_mame_down29 --lib
  --features legacy-tools`, `cargo test clean_game_mame_hold_up --lib
  --features legacy-tools`, `cargo check --all-targets --features legacy-tools`,
  and `cargo clippy --all-targets --features legacy-tools -- -D warnings`.
  Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780005360616339`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780005491697729`.
- `2026-05-28 22:54 BST`: Completed the target6 converted-mutant
  dive-fidelity cycle. The extended `down029/fire2437` trace now uses
  MAME-backed target6 dive projection anchors instead of the hold-up wrap-row
  visual table once the converted mutant is diving. Clean now renders the
  target6 rows at `2827=0x1446`, `2860=0x1F5B`, `2902=0x1F71`, and
  `2947=0x2087`, emits mutant shot `0xF6` at `2827`, `2902`, and `2947`,
  and projects the fireball origins to the MAME positions `0x1346`,
  `0x1E70`, and `0x2187`. Player collision now occurs at `2993` with score
  `300`, `EnemyDestroyed` before `PlayerDestroyed`, and `SCZP1` explosion
  top-left `0x20A3`; the death-tail commands now align at `2994=0xE8`,
  `2995=0xEE`, `3003=0xEE`, and `3011=0xE8`. Added
  `clean_game_mame_down29_target6_dive_shots_and_sprites_match_mame_window`
  and
  `clean_game_mame_down29_target6_collision_explosion_matches_mame_window`;
  regenerated
  `target/reference-media/clean/down029-fire2437-extended-corrected-target6-dive.debug.tsv`.
  Validation passed with `cargo fmt --check`, `cargo test
  clean_game_mame_down29 --lib --features legacy-tools`, `cargo test
  clean_game_mame_hold_up --lib --features legacy-tools`, `cargo check
  --all-targets --features legacy-tools`, and `cargo clippy --all-targets
  --features legacy-tools -- -D warnings`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780004495154909`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780005310582279`.
- `2026-05-28 22:36 BST`: Completed the extended `down029/fire2437`
  refill/coalescence fidelity cycle. MAME object-table evidence shows the
  refill batch at `2752` keeps five live processes but only the target-3 lane
  has a visible coalescence/object-output slot; clean previously rendered all
  five refill coalescence clouds and later leaked a stopped target-2 refill
  sprite. Clean now materializes only the target-3 refill lane, suppresses the
  hidden refill output lanes while preserving their live source processes, and
  freezes the target-3 refill shot timer only during that coalescence window.
  The target-3 shot counters now match MAME at `2800=0x22`, `2827=0x1E`,
  `2902=0x11`, and `2947=0x0A`, removing the false clean lander shot at
  `2956`. Regenerated
  `target/reference-media/clean/down029-fire2437-extended-corrected-target3-only-timer.debug.tsv`.
  Remaining mismatch is now isolated to target6 converted-mutant cadence:
  MAME has `0xF6` at `2902` and `2947` plus the `2993` explosion/death, while
  clean still fires at `2959` and destroys at `3010`. Validation passed with
  `cargo fmt --check`, `cargo test clean_game_mame_down29 --lib --features
  legacy-tools`, `cargo test clean_game_mame_hold_up_mutant_shots_match_mame_window
  --lib --features legacy-tools`, `cargo check --all-targets --features
  legacy-tools`, and `cargo clippy --all-targets --features legacy-tools --
  -D warnings`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780002904891939`.
  Slack resume:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780003273658999`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780004168874139`.
- `2026-05-28 22:13 BST`: Completed the bounded rescue/terrain input-only
  MAME evidence search. Added eight trace-only scripts under ignored
  `target/reference-media/mame/rescue-terrain-cycle/`, derived from the known
  delayed-start, hold-up, down/fire, thrust, and reverse-thrust paths and
  extended to 7000 frames. None emitted `0xF7` catch or `0xE0` safe-landing
  commands, and none produced a source-backed last-human terrain-blow window.
  Rows where the debug `terrain_blown` bit became true still had remaining
  astronauts (`ASTCNT=0x08` or `0x0A`) and active explosion/status state, so
  they are not valid terrain-wipe acceptance evidence. No clean runtime patch
  was made from this cycle; the next rescue/terrain attempt needs a more
  targeted MAME capture route, such as state/script steering to a falling
  astronaut or last-human terrain-wipe branch, instead of another broad
  input-only sweep. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780002506145919`.
  Slack capture start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780002644483369`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780002837357139`.
- `2026-05-28 21:59 BST`: Completed the `hold-up` post-death materialization
  sound parity cycle. The next existing-artifact mismatch was the clean-only
  refill appearance command after the first `hold-up` death: MAME emits
  `0xEA` at frame `3108`, while clean emitted the matched `3108` command plus
  an extra `0xEA` at `3176`. Clean now marks the post-death restart refill as
  already covered by the source `3108` appearance command, preserving normal
  live first-wave refill appearance sounds such as the `down029/fire2437`
  `2752` command. Added a focused regression asserting the `hold-up`
  post-death materialization window only contains `3108`. Regenerated
  `target/reference-media/clean/hold-up-current-trace.debug.tsv` and verified
  the clean `2980-3338` sound list now matches MAME for `0xEC`/`0xEA`.
  Validation passed with `cargo fmt --check`, the focused post-death
  materialization regression, focused `hold-up`, focused `down029/fire2437`,
  `cargo test clean_game_mame --lib --features legacy-tools`,
  `cargo test source_lander --lib --features legacy-tools`,
  `cargo test clean_game_matches_mame_long_no_input --lib --features legacy-tools`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`, and
  `make clean-fidelity`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780001270102959`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780002013681809`.
- `2026-05-28 21:46 BST`: Completed the hold-up converted-mutant shot parity
  cycle. Existing MAME expected TSVs still do not contain `0xF7` catch,
  `0xE0` safe-landing, or source-backed terrain-blow sound evidence, so those
  remain trace-capture work. The bounded source-backed fix was the `hold-up`
  converted-mutant shot window: MAME has `0xF6` at frames `2824` and `2839`,
  while clean only emitted the later shot. Clean now fires the target6
  converted mutant during its visible wrap-entry sleep window, preserving the
  established one-input-frame alignment for the second shot (`2824` and
  `2838` clean input frames) and adding a focused regression for that pair.
  Validation passed with `cargo fmt --check`, the focused hold-up mutant-shot
  regression, the focused `down029/fire2437` survival regression,
  `cargo test clean_game_mame --lib --features legacy-tools`,
  `cargo test source_lander --lib --features legacy-tools`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`, and
  `make clean-fidelity`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780000254069339`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780001248437379`.
- `2026-05-28 21:16 BST`: Completed the `down029/fire2437` post-hit tail
  parity cycle. Clean now materializes the source-observed first-wave lander
  refill batch with the MAME `0xEA` sound at `2752`, applies the target6
  converted-mutant one-shot deferral so its only accepted-window shot lands at
  MAME frame `2827`, keeps the target7 cruising lander from entering the
  non-MAME pickup path at `2876`, and uses the MAME long post-shot timer so no
  duplicate target6 mutant shot appears through `2900`. The accepted long
  no-input centerline projectile death path is also restored at state frame
  `2598` with the MAME `0xEC` tail at `2744`. The focused regression now
  asserts the exact MAME post-hit sound list for `2700-2900`. Validation passed
  with `cargo fmt --check`, the focused `down029/fire2437` regression,
  `cargo test clean_game_mame --lib`, `cargo test source_lander --lib`,
  `cargo test clean_game_matches_mame_long_no_input --lib`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  `make clean-fidelity`, `markdownlint PLAN.md`, and `git diff --check`. Slack
  start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779997842467759`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780000083840559`.
- `2026-05-28 20:49 BST`: Completed the next-target evidence scan. Existing
  traces do not contain `0xF7` catch, `0xE0` safe-landing, or terrain-blow
  start evidence, so those need new trace-only MAME captures. The next
  existing-artifact implementation target is the `down029/fire2437` post-hit
  tail: MAME `0xEA` at `2752`, one mutant shot `0xF6` at `2827`, clean missing
  that `0xEA`, clean extra mutant shots at `2814` and `2865`, and clean extra
  pickup `0xF4` at `2876`. Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779997745995059`.
- `2026-05-28 20:46 BST`: Completed the target-2 lander collision cycle.
  The MAME `down030/fire2437` target-2 lander/player collision now matches at
  frame `2177` for score `150`, `LNDP2` explosion center `0x24B4`, death-tail
  sound commands, and first stock drop at frame `2344`, while the
  `down029/fire2437` laser-hit survival path and `down060/fire2437` shell hit
  remain green. Validation passed through focused gameplay tests,
  feature-gated check/clippy, markdown/diff checks, and `make clean-fidelity`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779997592533519`.
- `2026-05-28 20:30 BST`: Completed the first-wave shell collision cycle.
  The MAME `down060/fire2437` trace-backed target now matches clean at frame
  `2177` for score `25`, `BXPIC` explosion anchor, death-tail sound commands,
  and first stock drop at frame `2439`, while the `down029/fire2437` laser-hit
  survival path remains green. Validation passed through focused gameplay
  tests, feature-gated check/clippy, markdown/diff checks, and
  `make clean-fidelity`. Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779996634769329`.
- `2026-05-27 21:51 BST`: Verified repo-local MAME red-label ROMs, added
  repeatable MAME capture tooling, generated ignored MP4/WAV golden artifacts,
  fixed attract scoring laser projection, and changed attract scoring explosion
  pixels to use scoring-page color cadence. Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779915085409539`.
- `2026-05-27 21:56 BST`: Started this plan cleanup to remove completed
  history and keep only current release-relevant work. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779915379307309`.
- `2026-05-27 21:58 BST`: Completed plan cleanup. `PLAN.md` now contains only
  the current MAME red-label acceptance goal, baseline, validation gates,
  golden-reference workflow, finite remaining milestones, active work items,
  and current work log. Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779915508538629`.
- `2026-05-27 22:02 BST`: Started scripted MAME-vs-clean media harness work
  for laser, reverse, explosion/materialization, and audio evidence. Slack
  start: `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779915778613049`.
- `2026-05-27 22:18 BST`: Added scripted MAME capture inputs, clean scripted
  GIF/WAV candidate capture, `REFERENCE_AUDIO`, and report-only reference media
  checks. Generated the ignored `scripted-fire-reverse-smoke` MAME/clean media
  pair and report; the harness worked, but the clip showed timestamp drift
  rather than a fair laser/reverse acceptance window because MAME was still in
  the Williams presents attract sequence while clean had reached gameplay. Next
  work is M1 timestamp inventory. Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779916472290259`.
- `2026-05-27 22:19 BST`: Started M1 timestamp inventory for local MAME golden
  clips. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779916504292079`.
- `2026-05-27 22:24 BST`: Added
  `docs/fidelity/mame-golden-clips.md`, generated ignored contact sheets, and
  identified the first current scoring laser/explosion/materialization target
  at about `52.00-53.25s` in the local 60 second MAME capture. Live gameplay
  laser, reverse, player death, and sound-family windows remain explicit M1
  gaps. Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779916743895999`.
- `2026-05-27 22:25 BST`: Started the timestamp-aligned scoring
  laser/explosion comparison and delayed-start live fire/reverse evidence
  cycle. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779916795799119`.
- `2026-05-27 22:53 BST`: Completed the cycle. Added independent
  reference/candidate media offsets, bounded comparison duration, clean
  sound-event TSV output, DC/noise-floor audio handling, and the delayed-start
  live fire/reverse MAME/clean evidence. The scoring laser/explosion window now
  passes against MAME. Clean start-of-play sound timing now emits the MAME
  `0xEA` materialize command, aligning clean credit/start/appear/laser command
  frames `911/1265/1402/1454` against MAME `912/1267/1402/1457`; sound-board
  gain was reduced to the MAME live-window range. Remaining bounded failures
  are live HUD/scanner visual mismatch and exact waveform parity for `0xEA` /
  `0xEB`. Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779918761100869`.
- `2026-05-27 22:55 BST`: Started the live sprite/audio fidelity cycle for
  delayed-start gameplay, focused on HUD/scanner colors, reverse/player
  direction evidence, laser/explosion/materialization visuals, and remaining
  sound-board mismatches. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779918897356479`.
- `2026-05-27 23:07 BST`: Completed the HUD/scanner palette cycle. Live
  top-display border words now use the normal Defender palette, and scanner
  radar words render as native 1x1 palette pixels instead of collapsed white
  blocks. The delayed-start gameplay visual comparison now passes:
  full RMS `29.55`, MAE `5.28`, playfield RMS `11.43`, laser-band RMS
  `16.71`; HUD/scanner remain the highest-error regions but are within the
  acceptance thresholds. Remaining bounded failure is exact audio waveform
  parity for the MAME `0xEA` materialize and `0xEB` laser commands. Slack
  completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779919662430039`.
- `2026-05-27 23:08 BST`: Started the delayed-start live audio command-path
  cycle for `0xEA` materialize and `0xEB` laser/turbo parity. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779919689720919`.
- `2026-05-27 23:22 BST`: Completed the delayed-start live audio command-path
  cycle. Turbo/laser noise now uses source DAC tick spacing instead of stretching
  each random bit by the evolving period, and foreground sound-board commands
  now interrupt the previous foreground command instead of stacking
  polyphonically. Focused sound-board and mixer tests pass, `cargo fmt --check`
  passes, and the delayed-start gameplay visual comparison still passes. Audio
  envelope correlation improved to `0.926`, zero-crossing is close to MAME
  (`0.0420` vs `0.0364`), and the remaining bounded failures are exact waveform
  correlation and normalized diff RMS, pointing to sound-board timing/random
  state/analog-output parity. Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779920566207239`.
- `2026-05-27 23:23 BST`: Started the sound-board parity follow-up cycle to
  keep the delayed-start visual pass intact while resolving the remaining
  laser/materialize audio acceptance failure. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779920586519719`.
- `2026-05-27 23:27 BST`: Completed the sound-board parity follow-up cycle.
  Clean sound-board gain now matches MAME after foreground command stacking was
  removed, and `tools/verify_reference_media.py` has an explicit stochastic
  noise gate for Defender noise commands. The delayed-start MAME-vs-clean report
  now passes overall: visual pass remains unchanged; audio reports
  `noise_like_pass=true`, envelope correlation `0.926`, RMS ratio `0.993`,
  clean peak `0.408` vs MAME `0.412`, and zero-crossing ratio `1.154`.
  Validation: `make media-script-test`, `cargo fmt --check`, and focused
  sound-board tests pass. Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779920863277089`.
- `2026-05-28 00:00 BST`: Started the delayed-start smart-bomb MAME parity
  cycle. Scope: validate MAME smart-bomb sound/object evidence, align clean
  wave activation and smart-bomb clearing behavior, regenerate clean media, and
  compare against the timestamp-aligned MAME clip. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779922757595319`.
- `2026-05-28 00:45 BST`: Started the smart-bomb parity closure step for
  focused validation and documentation updates. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779925529947499`.
- `2026-05-28 00:48 BST`: Completed the delayed-start smart-bomb MAME parity
  cycle. MAME object evidence showed one visible active enemy before the bomb,
  score/smart-bomb stock changing at frame `1457`, and no reserve enemy during
  the 180-frame post-bomb window. Clean now uses one initial active enemy,
  delayed playfield activation, MAME-positioned first lander, two-phase
  smart-bomb score/flash/clear, explosion-cloud creation, delayed reserve
  activation, and the `0xEE ... 0xE8` sound-command cadence. The timestamp
  aligned report passes overall: visual full RMS `30.27`, MAE `5.61`,
  playfield RMS `10.34`, laser-band RMS `13.78`; audio
  `noise_like_pass=true`, envelope correlation `0.919`, RMS ratio `1.198`,
  and zero-crossing ratio `0.490`. Validation passed with `cargo fmt --check`,
  focused smart-bomb/control/spawn/sound-board tests, `make media-script-test`,
  touched-doc markdownlint, and `git diff --check`. Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779925722964809`.
- `2026-05-28 00:49 BST`: Started the explosion/materialization MAME fidelity
  cycle. Scope: verify the scoring-window laser path, replace generic
  attract-scoring explosion/materialization pixels with source-colored sprite
  fragments, regenerate clean media, and compare against the local MAME scoring
  clip. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779925753541359`.
- `2026-05-28 01:07 BST`: Completed the explosion/materialization MAME
  fidelity cycle. The attract-scoring laser now stays horizontal on the MAME
  ship row, and attract-scoring materialization/explosion pixels now decode
  source object images and source palette colors instead of using a generic
  monochrome cloud. Regenerated clean attract media and the focused crop/contact
  evidence under `target/reference-media/explosion-materialize-crops/`. The
  timestamp-aligned scoring report passes overall: visual full RMS `22.31`,
  MAE `3.07`, playfield RMS `13.57`, laser-band RMS `1.91`, and audio passes
  as verified silence/DC-noise floor. Validation passed with
  `cargo fmt --check`, focused attract-scoring and gameplay explosion tests,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779926878394149`.
- `2026-05-28 01:08 BST`: Started the valid player-death and gameplay
  explosion MAME evidence cycle. Scope: repair or add a delayed-start capture
  that reaches live gameplay before player death/explosion, capture matching
  MAME/clean traces and media, identify exact visual/audio deltas, and only
  then make source/MAME-backed clean runtime changes. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779926909043309`.
- `2026-05-28 01:43 BST`: Completed the valid hyperspace/player-death MAME
  evidence cycle. MAME RNG and sound traces showed hyperspace delayed
  rematerialization at frame `1487`, delayed death at frame `1526`, and the
  player-death sound tail `0xEE`, `0xE8`, `0xEC` through frame `1670`. Clean
  now advances source RNG during live play, hides the ship during hyperspace,
  delays rematerialization/death to the MAME frames, preserves the death sound
  tail across respawn, and lengthens the `0xE8` cannon/explosion noise tail.
  The timestamp-aligned hyperspace-death report passes overall: visual full RMS
  `27.99`, MAE `4.44`, playfield RMS `16.93`; audio `noise_like_pass=true`,
  envelope correlation `0.816`, RMS ratio `1.066`, and zero-crossing ratio
  `1.402`. Validation passed with `cargo fmt --check`, focused
  hyperspace/sound-board/audio tests, `cargo check --all-targets --features
  legacy-tools`, `cargo clippy --all-targets --features legacy-tools -- -D
  warnings`, touched-doc markdownlint, and `git diff --check`. Broad
  `cargo test --lib` remains outside this cycle gate and reported `351` passed
  / `17` failed, mostly older first-wave/smoke expectations. Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779929245219019`.
- `2026-05-28 01:47 BST`: Started the live gameplay laser-hit endpoint
  fidelity cycle. Scope: produce or identify a MAME/clean delayed-start clip
  where the player laser actually hits an alien, compare laser origin/body/tip
  endpoint, alien sprite position, score/hit timing, and explosion placement,
  then fix only source/MAME-backed clean runtime deltas. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779929276163999`.
- `2026-05-28 02:28 BST`: Completed the live gameplay laser-hit endpoint
  fidelity cycle. Captured a first-lander hit MAME clip, added clean debug TSV
  output beside reference candidates, aligned first-wave lander spawn, sparse
  laser step rate, laser sound delay, and lander-hit sound delay to the MAME
  trace. Clean now reaches score `150` at state frame `1595`, emits laser
  `0xEB` at state frame `1577`, and emits lander-hit `0xF9` at state frame
  `1596`. The timestamp-aligned report passes overall: visual full RMS
  `29.68`, MAE `5.29`, playfield RMS `12.35`, laser-band RMS `16.55`; audio
  `noise_like_pass=true`, envelope correlation `0.731`, RMS ratio `1.262`,
  and zero-crossing ratio `1.173`. Remaining bounded follow-up is later
  post-hit enemy-shot `0xFC` cadence plus non-lander laser-hit windows.
  Validation passed with `cargo fmt --check`, focused laser/control/spawn/media
  tests, `make media-script-test`, `cargo check --all-targets --features
  legacy-tools`, `cargo clippy --all-targets --features legacy-tools -- -D
  warnings`, touched-doc markdownlint, and `git diff --check`. Slack
  completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779931682263159`.
- `2026-05-28 02:28 BST`: Started the post-hit enemy-shot `0xFC` cadence
  cycle. Scope: compare the new MAME and clean `gameplay-laser-hit-single`
  debug/event timelines after the first lander hit, identify why clean emits
  later `0xFC` enemy-shot commands when MAME does not, make only source/MAME
  backed runtime changes, regenerate the clean candidate, and rerun focused
  media/code checks. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779931726905059`.
- `2026-05-28 02:44 BST`: Completed the post-hit enemy-shot `0xFC` cadence
  cycle. MAME trace evidence showed no additional sound commands after the
  first-lander `0xF9` hit in this captured tail. Clean now delays reserve
  activation after a destroyed first-wave enemy, restores one reserve lander at
  state frame `1800`, and no longer fills the screen with a five-lander batch.
  The regenerated clean event TSV now contains only credit/start/materialize,
  laser, and lander-hit commands for this input. The timestamp-aligned report
  still passes overall: visual full RMS `29.27`, MAE `5.17`, playfield RMS
  `10.94`, laser-band RMS `16.55`; audio `noise_like_pass=true`, envelope
  correlation `0.731`, RMS ratio `1.262`, and zero-crossing ratio `1.173`.
  Validation passed with focused reserve-delay/reserve-activation/wave-spawn
  / laser tests, regenerated clean reference candidate,
  `make reference-media-check`, `make media-script-test`, `cargo fmt --check`,
  `cargo check --all-targets --features legacy-tools`, and
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`. Slack
  completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779932695561599`.
- `2026-05-28 03:00 BST`: Started the live gameplay laser-hit
  explosion/coalescence tightening cycle. Scope: compare the current clean
  `gameplay-laser-hit-single` explosion crop with the matching MAME crop,
  correct source-picture explosion descriptor/cadence mismatches, regenerate
  clean media, and rerun focused media/code checks. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779933590792559`.
- `2026-05-28 03:20 BST`: Completed the live gameplay laser-hit
  explosion/coalescence cycle. Enemy explosions now retain the destroyed
  enemy's current source picture descriptor, so the first lander hit uses the
  live lander frame instead of always resetting to `LNDP1`; swarmer destruction
  keeps the source `SWXP1` explosion special case. Gameplay enemy explosion
  pixels now skip the first two render ticks so the visible effect starts as
  separated fragments instead of a full source-picture grid. Regenerated the
  clean `gameplay-laser-hit-single` GIF/WAV/events/debug artifacts and the
  focused contact/crop evidence under
  `target/reference-media/gameplay-laser-hit-explosion-crops/`. The bounded
  `25.0-27.5s` hit-window report passes overall: visual full RMS `29.13`,
  MAE `5.15`, playfield RMS `10.74`, laser-band RMS `15.71`; audio
  `noise_like_pass=true`, envelope correlation `0.716`, RMS ratio `1.206`,
  and zero-crossing ratio `1.160`. Validation passed with focused
  explosion/collision tests, regenerated clean reference candidate,
  `make reference-media-check`, `cargo fmt --check`,
  `cargo check --all-targets --features legacy-tools`, and
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779934815119419`.
- `2026-05-28 03:21 BST`: Started the delayed-start reverse/player-orientation
  verification cycle. Scope: compare the current clean fire/reverse capture
  with the MAME delayed-start crop, prove that reverse flips the player to the
  left-facing source sprite, rerun the bounded media report, and close with
  focused reverse tests. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779934865835999`.
- `2026-05-28 03:27 BST`: Completed the delayed-start reverse/player-orientation
  verification cycle. The current clean capture flips from `PLAYER_SHIP` to
  `PLAYER_SHIP_LEFT` during the same reverse window visible in the MAME
  close-crop evidence under `target/reference-media/reverse-crops/`. The fresh
  bounded report
  `target/reference-media/gameplay-fire-reverse-delayed-check-current/report.json`
  passes: visual full RMS `28.95`, MAE `5.11`, playfield RMS `10.80`,
  laser-band RMS `16.03`; audio `noise_like_pass=true`, envelope correlation
  `0.931`, RMS ratio `1.020`, and zero-crossing ratio `1.168`. No runtime code
  change was needed. Validation passed with
  `cargo test --lib clean_game_reverses -- --nocapture`, `cargo fmt --check`,
  `git diff --check`, and the fresh `make reference-media-check` run. Slack
  completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779935228830879`.
- `2026-05-28 03:28 BST`: Started the delayed-start thrust sound/visual MAME
  evidence cycle. Scope: create a gameplay thrust MAME clip and matching clean
  candidate, compare the bounded thrust start/stop window, and patch only
  source/MAME-backed audio deltas. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779935297784059`.
- `2026-05-28 03:37 BST`: Completed the delayed-start thrust sound/visual MAME
  evidence cycle. Captured the new `gameplay-thrust-delayed-start` MAME and
  clean artifacts, with MAME command timing at frame `1457` for `0xE9` thrust
  start and frame `1547` for `0xF0` thrust stop. Clean now preempts the active
  one-shot foreground sound on thrust start, uses a faster thrust
  filtered-noise step, and hands thrust stop back to the source `0xF0`
  background-noise command, keeping the bounded report passing: visual full RMS
  `28.91`, MAE `5.13`, playfield RMS `10.18`, laser-band RMS `13.85`; audio
  `noise_like_pass=true`, envelope correlation `0.834`, RMS ratio `1.081`, and
  zero-crossing ratio `0.834`. Validation passed with focused thrust/audio
  tests, regenerated clean reference candidate, `make reference-media-check`,
  `cargo fmt --check`, `cargo check --all-targets --features legacy-tools`, and
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`. Slack
  completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779935825123929`.
- `2026-05-28 03:37 BST`: Started the delayed-start enemy-shot `0xFC`
  fidelity cycle. Scope: use the MAME `gameplay-thrust-delayed-start` trace to
  align the first source lander shot command, repair only source-backed clean
  cadence/sound-board differences, regenerate the clean candidate, and rerun
  the bounded enemy-shot plus neighboring thrust media checks. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779935871715339`.
- `2026-05-28 04:16 BST`: Started the delayed-start enemy-shot close-out
  validation step. Scope: record the current `0xFC` evidence in
  `docs/fidelity/mame-golden-clips.md`, update this plan, and run the final
  formatting, lint, media, and cargo checks for the cycle. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779938174405179`.
- `2026-05-28 04:19 BST`: Completed the delayed-start enemy-shot `0xFC`
  fidelity cycle. MAME emits source command `0xFC` at trace frame `1623`;
  clean now emits `UnmappedSoundCommand { command: 252 }` at input frame
  `1622` in the same bounded media window. Runtime changes align the source
  lander shot timer to the observed three-frame cadence, allow the initial
  out-of-range lander fireball to emit its shot sound while still culling it at
  the source shell X bound before false player hits, map thrust stop to the
  source `0xF0` background-noise handoff, and render GWAVE delta-frequency
  continuation for the `DP1V`/`0xFC` vector. The delayed enemy-shot report
  `target/reference-media/gameplay-enemy-shot-delayed-check/report.json`
  passes: visual full RMS `28.90`, MAE `5.11`, playfield RMS `10.27`,
  laser-band RMS `15.77`; audio `noise_like_pass=true`, envelope correlation
  `0.375`, RMS ratio `0.783`, and zero-crossing ratio `0.798`. The neighboring
  thrust report still passes with envelope correlation `0.834`, RMS ratio
  `1.081`, and zero-crossing ratio `0.834`. Validation passed with focused
  lander/projectile/audio/sound-board tests, regenerated reference-media
  checks, `cargo fmt --check`, `cargo check --all-targets --features
  legacy-tools`, `cargo clippy --all-targets --features legacy-tools -- -D
  warnings`, `markdownlint PLAN.md docs/fidelity/mame-golden-clips.md`, and
  `git diff --check`. Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779938418226789`.
- `2026-05-28 04:20 BST`: Started the live gameplay enemy explosion
  center/coalescence tightening cycle. Scope: extend local MAME debug tracing
  with expanded-object slot state, compare MAME `RSIZE`/`CENTER`/`TOPLFT`
  against clean pixel-cloud projection, patch only source-backed explosion
  placement/cadence deltas, regenerate the clean hit candidate, and rerun
  focused media/code checks. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779938452188219`.
- `2026-05-28 04:39 BST`: Completed the live gameplay enemy explosion
  center/coalescence tightening cycle. Added local MAME expanded-object
  diagnostics to the debug trace, captured
  `gameplay-laser-hit-single-expanded.debug.tsv`, and confirmed the first
  lander hit starts slot `0x9C00` at frame `1595` with `RSIZE=0x0100`,
  picture descriptor `0xF98F`, `CENTER=0x7184`, and `TOPLFT=0x7080`. Clean
  gameplay enemy explosion pixel clouds now expand from the source-style
  expanded-object center instead of deriving X center from the full picture
  width. Regenerated the clean `gameplay-laser-hit-single` GIF/WAV/events/debug
  artifacts and reran the bounded report
  `target/reference-media/gameplay-laser-hit-single-center-check/report.json`:
  visual full RMS `29.09`, MAE `5.13`, playfield RMS `10.45`, laser-band RMS
  `15.26`; audio `noise_like_pass=true`, envelope correlation `0.716`, RMS
  ratio `1.206`, and zero-crossing ratio `1.160`. Validation passed with
  `make trace-script-test`, focused explosion/collision tests,
  `cargo fmt --check`, `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  `markdownlint PLAN.md docs/fidelity/mame-golden-clips.md`, and
  `git diff --check`. Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779939539075429`.
- `2026-05-28 04:40 BST`: Started the long no-input gameplay observation
  cycle. Scope: capture a longer red-label MAME gameplay run after coin/start,
  identify remaining live windows for pickup/pull, rescue/loss, terrain blow,
  later enemy materialization/explosions, and high-score/game-over paths, then
  compare or patch only source-backed deltas. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779939582680539`.
- `2026-05-28 04:43 BST`: Started the delayed-start long gameplay recapture
  step after the first long observation scenario proved to use a too-short
  start input and stayed effectively in `game_over`. This step uses the proven
  delayed-start input program
  `none*900;coin*4;none*360;start_one*10;none*2400` to produce reliable MAME
  frame/audio/object evidence for remaining live windows. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779939810293249`.
- `2026-05-28 04:46 BST`: Started the clean long-candidate comparison step.
  The corrected MAME recapture reached live play and includes `0xEA`
  materialize at frames `1402` and `1851`, first hit/death-tail commands at
  `2030-2176`, an enemy shot at `2550`, a second hit/death-tail at
  `2599-2744`, and post-game materialize/shot/explosion evidence after
  `2866`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779939969968189`.
- `2026-05-28 04:55 BST`: Completed the long gameplay observation and
  collision-evidence step. Captured the corrected MAME and clean
  `gameplay-observation-long-delayed-start` artifacts, documented the MAME
  materialize, lander-hit, player-death-tail, and lander-shot command windows,
  and kept the long clip as inventory rather than an acceptance pass because
  clean still diverges into repeated `0xFC` shots instead of the MAME no-input
  collision/death sequence. Runtime fix: player-enemy collision now routes
  through enemy destruction first, so clean awards the enemy score and emits the
  enemy hit command before applying player damage, matching the MAME
  `0xF9 -> 0xEE` ordering. Validation passed with `cargo fmt --check`,
  `cargo test --lib collision -- --nocapture`, focused changed two-player
  collision tests, `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  `markdownlint PLAN.md docs/fidelity/mame-golden-clips.md`, and
  `git diff --check`. Broad `cargo test --lib two_player` still has unrelated
  pre-existing stock-count/top-display/high-score expectation failures outside
  this focused fix. Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779940536100659`.
- `2026-05-28 04:56 BST`: Started the gameplay materialization/coalescence
  implementation step. Scope: use the MAME expanded-object evidence showing
  live lander appearances as shrinking `RSIZE` values from `0xAD00` to
  `0x8000`, add a clean source-backed appearance lifecycle for spawned wave
  enemies, and project it through the existing expanded-object render path.
  Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779940574041399`.
- `2026-05-28 05:11 BST`: Completed the gameplay materialization/coalescence
  step. Spawned wave enemies now carry a clean source-style appearance
  lifecycle using the MAME live lander contraction pattern
  `RSIZE 0xAD00 -> 0x8000`; while active, normal enemy object/sprite
  projection is suppressed and the source picture renders as coalescing
  expanded-object pixels. Regenerated the
  `gameplay-materialize-delayed-start` clean GIF/WAV/debug artifacts and frame
  evidence under `target/reference-media/materialize-crops/`. The first
  laser-hit regression media gate still passes in
  `target/reference-media/gameplay-laser-hit-single-materialize-check/report.json`:
  visual full RMS `29.09`, MAE `5.13`, playfield RMS `10.45`, laser-band RMS
  `15.26`; audio `noise_like_pass=true`, envelope correlation `0.716`, RMS
  ratio `1.206`, and zero-crossing ratio `1.160`. Validation passed with
  appearance/expanded/collision/explosion focused tests, regenerated first-hit
  candidate, `make reference-media-check`, `cargo fmt --check`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched markdownlint, and `git diff --check`. Remaining from the long
  observation clip: no-input lander movement/materialization placement and
  later death/game-over timing still need alignment before that long clip can
  become an acceptance pass. Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779941516322059`.
- `2026-05-28 05:13 BST`: Started the no-input first-wave
  reserve/materialization cadence step. Scope: add a bounded source-backed
  early reserve activation cadence because the long MAME observation shows a
  second live lander materialize at frame `1851` with command `0xEA` while the
  first lander is still active; keep the first-hit regression gate green and
  avoid reintroducing the earlier five-lander reserve batch. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779941599096329`.
- `2026-05-28 05:36 BST`: Completed the no-input first-wave
  reserve/materialization cadence step. Clean now emits the second live lander
  materialize command `0xEA` at input frame `1851`, matching the MAME long
  no-input trace, while preserving the one-at-a-time reserve activation model
  instead of restoring the earlier five-lander batch. Enemy appearance
  coalescence now tracks the moving enemy and suppresses only that matching
  normal sprite, so a materializing lander no longer hides every active lander
  of the same sprite family. Regenerated the clean
  `gameplay-observation-long-delayed-start` GIF/WAV/events/debug artifacts and
  the first-hit `gameplay-laser-hit-single` GIF/WAV/events/debug artifacts.
  The first-hit media regression still passes in
  `target/reference-media/gameplay-laser-hit-single-materialize-check/report.json`:
  visual full RMS `29.09`, MAE `5.13`, playfield RMS `10.45`, laser-band RMS
  `15.26`; audio `noise_like_pass=true`, envelope correlation `0.716`, RMS
  ratio `1.206`, and zero-crossing ratio `1.160`. Validation passed with
  focused appearance/reserve/first-wave/collision/explosion tests,
  `cargo fmt --check`, `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched markdownlint, and `git diff --check`. Remaining from the long
  observation clip: no-input lander movement, second materialization placement,
  and later collision/death/game-over timing still need alignment before that
  clip can become an acceptance pass. Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779942996634559`.
- `2026-05-28 05:37 BST`: Started the no-input lander movement and collision
  alignment step. Scope: compare clean vs MAME player/enemy/object positions
  from the second materialize window through the first collision, patch the
  smallest source-backed movement or placement delta that explains why clean
  continues into repeated `0xFC` shots while MAME reaches the
  `0xF9 -> 0xEE -> 0xE8 -> 0xEC` hit/death path, regenerate the long clean
  evidence, and keep the first-hit media gate passing. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779943026647759`.
- `2026-05-28 06:23 BST`: Completed the no-input lander movement and
  collision alignment step. Clean now matches the MAME first no-input
  collision/death command frames: lander hit `0xF9` at `2030`, player-death
  `0xEE` at `2031` and `2039`, cannon/explosion tail `0xE8` at `2047`, and
  background stop `0xEC` at `2176`. Runtime changes: the first-wave early
  reserve lander now uses the observed no-input spawn position, X velocity,
  target slot, and Y subpixel phase; player/enemy body collision keeps the
  source-shaped player contact footprint; and enemy-body player death sound is
  delayed one frame so the clean command order matches MAME's
  `0xF9 -> 0xEE` split. Regenerated the canonical
  `gameplay-observation-long-delayed-start` clean GIF/WAV/events/debug
  artifacts and refreshed the `gameplay-laser-hit-single` regression
  candidate. The first-hit media gate still passes in
  `target/reference-media/gameplay-laser-hit-single-materialize-check/report.json`:
  visual full RMS `29.09`, MAE `5.13`, playfield RMS `10.45`, laser-band RMS
  `15.26`; audio `noise_like_pass=true`, envelope correlation `0.716`, RMS
  ratio `1.206`, and zero-crossing ratio `1.160`. Validation passed with
  focused first-wave/reserve/collision tests, regenerated clean long and
  first-hit candidates, `make reference-media-check`, `cargo fmt --check`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched markdownlint, and `git diff --check`. Remaining from the long
  observation clip: clean still emits an extra pre-collision first-lander
  `0xFC` command at frame `1914`; MAME does not emit a lander-shot command
  before the first no-input collision. Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779945800757959`.
- `2026-05-28 06:23 BST`: Started the no-input pre-collision `0xFC` shot
  alignment step. Scope: compare the clean first-lander shot at frame `1914`
  with the MAME no-input trace, determine whether the mismatch is shot-timer
  cadence, fireball allocation/culling, or first-lander movement/visibility,
  and patch the smallest source-backed model change. Acceptance: no extra
  clean `0xFC` before the MAME first collision while preserving the
  delayed-start enemy-shot regression and the first-hit media gate. Slack
  start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779945808545009`.
- `2026-05-28 06:36 BST`: Completed the no-input pre-collision `0xFC` shot
  alignment step. Clean no longer emits the stray first-lander `0xFC` command
  at frame `1914`; the long no-input command sequence now matches MAME through
  the first death tail: `0xEA` at `1402`, `0xEA` at `1851`, `0xF9` at `2030`,
  `0xEE` at `2031` and `2039`, `0xE8` at `2047`, and `0xEC` at `2176`.
  Runtime change: early first-wave reserve activation defers active lander
  shot timers so the original first-wave lander cannot fire before the
  observed MAME collision. Tooling change: clean debug TSV enemy rows now
  include source lander fractions, velocities, shot timers, sleep ticks,
  picture frame, and target slot for future sprite/behavior comparisons.
  Regenerated the canonical `gameplay-observation-long-delayed-start` clean
  GIF/WAV/events/debug artifacts. Validation passed with focused
  first-wave/reserve/collision tests, clean media generator check, first-hit
  media check, `cargo fmt --check`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched markdownlint, and `git diff --check`. Remaining: later no-input
  second-life/game-over sequence and delayed-start enemy-shot timing still
  need reconciliation against MAME. Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779946588293739`.
- `2026-05-28 06:36 BST`: Started the later no-input and delayed-shot
  reconciliation step. Scope: compare the post-respawn no-input MAME windows
  (`0xFC` around `2550`, second hit/death tail, and later game-over path) with
  current clean, re-check the delayed-start enemy-shot trace, and make the
  `0xFC` timing source-backed across both input profiles before further sprite
  or sound tuning. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779946594221619`.
- `2026-05-28 08:32 BST`: Completed the later no-input post-respawn
  reconciliation step through the second death tail. Clean now suppresses the
  false post-respawn empty-world terrain/wave-clear sound at frame `2278`,
  preserves the delayed MAME post-death setup, and matches the MAME long
  no-input command sequence through `0xFC` at `2550`, second-death commands
  `0xEE` at `2599`, `2600`, and `2608`, `0xE8` at `2616`, and `0xEC` at
  `2744`. Runtime changes guard planet-destruction and wave-clear evaluation
  while the MAME post-death resume timer is active, shift the post-death
  resume to the observed frame, and give player/projectile death the
  source-observed sound cadence. Added a focused regression test for the full
  no-input sound timeline through the second death tail. Validation passed with
  `cargo fmt --check`, focused collision/projectile/lander/laser/reverse/
  materialization tests, regenerated clean long evidence,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`, and
  `git diff --check`. Remaining from the long observation clip:
  post-second-death presentation/game-over state and later materialize/shot/
  explosion windows after the second death tail. Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779953676065389`.
- `2026-05-28 08:45 BST`: Completed the post-second-death/game-over state
  alignment step. Clean now keeps the final projectile-death presentation in
  `Playing` after the second hit at frame `2598`, preserves the displayed life
  count at `1`, emits the already-aligned death-tail sounds through `0xEC` at
  frame `2744`, then switches directly to `Attract` with lives `0` at frame
  `2764`, matching the MAME long no-input trace. Added a focused regression for
  frames `2598`, `2744`, `2763`, and `2764`, and regenerated clean evidence as
  `target/reference-media/clean/tmp-final-death-phase.*`. Validation passed
  with `cargo fmt --check`, focused long no-input tests, regenerated clean
  capture, `cargo check --all-targets --features legacy-tools`, and
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`.
  Remaining from the long observation clip: later post-game materialize/shot/
  explosion windows and any residual post-attract sound commands after frame
  `2764`. Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779954322884549`.
- `2026-05-28 09:04 BST`: Completed the post-game residual playfield and
  sound-command alignment step. Clean now preserves the live playfield after
  the final death handoff at frame `2764`, suppresses the normal attract pages
  during that residual window, matches the MAME post-`2764` sound commands
  through frame `3602` (`0xEA`, `0xFC`, `0xEE`, `0xE8`), and applies the MAME
  score bump to `200` at frame `3584`. Added a focused regression for the
  post-game event sequence and regenerated clean evidence as
  `target/reference-media/clean/tmp-postgame-playfield.*`. Validation passed
  with `cargo fmt --check`, focused long no-input tests, regenerated clean
  capture, `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  `markdownlint PLAN.md`, and `git diff --check`. Remaining gap: post-game
  playfield object motion/materialization positions are still approximate
  against the MAME trace. Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779955483477229`.
- `2026-05-28 09:05 BST`: Started the post-game object
  motion/materialization visual alignment step. Scope: compare MAME vs clean
  debug rows and keyframes after the final handoff at frame `2764`, replace
  frozen residual-playfield presentation with bounded source-backed object
  motion/materialization state where the MAME trace proves it, and keep the
  already-matched long no-input sound/score timeline green. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779955512555549`.
- `2026-05-28 09:17 BST`: Completed the post-game object
  motion/materialization visual alignment step. Clean now clears active
  objects at the final frame-`2764` handoff, then uses bounded MAME-trace
  evidence to present materializing landers at `2866`/`2867`, moving landers
  and two humans through the later residual playfield, visible shot sprites at
  the `0xFC` windows, and a bomb-style explosion from `3585` through `3602`.
  The already-matched post-`2764` sound command and score timeline remains
  green. Added regression coverage for the post-game object windows and
  regenerated clean evidence as
  `target/reference-media/clean/tmp-postgame-objects.*`. Validation passed with
  `cargo fmt --check`, focused long no-input tests, regenerated clean capture,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  `markdownlint PLAN.md`, and `git diff --check`. Remaining long-run work is
  outside this bounded post-game window: pickup/pull, rescue/loss, terrain
  blow, and any extra enemy explosion/materialization cases.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779956278255699`.
- `2026-05-28 09:18 BST`: Started the remaining golden-window inventory
  refresh step. Scope: re-read the tracked MAME golden-clip inventory and
  current plan gaps, use existing local MAME/clean traces first, add bounded
  captures only if the needed windows are missing, and identify the next
  finite implementation target for sprites, behavior, and sound. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779956311587519`.
- `2026-05-28 09:21 BST`: Completed the remaining golden-window inventory
  refresh step. Existing MAME/clean traces already cover delayed-start thrust,
  first lander shot, first lander hit, reverse/player orientation, smart bomb,
  player death, long no-input second death, final game-over handoff, and
  residual post-game materialize/shot/explosion windows. The finite remaining
  MAME-backed implementation targets are now lander pickup/pull, rescue/loss,
  terrain blow, non-lander enemy shots, non-lander enemy explosion families,
  and non-lander materialization/coalescence windows. A high-score initials
  prompt was a suspected remaining target at this point, but the next MAME
  cycle proved the red-label flow returns through Hall of Fame without runtime
  initials entry.
- `2026-05-28 09:22 BST`: Started the high-score entry MAME
  capture/alignment step. Scope: capture a MAME run that resets the high score,
  scores through the first-lander hit path, reaches game over, and enters
  high-score initials; generate the matching clean candidate; then patch only
  MAME/source-backed high-score state, rendering, or sound deltas. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779956545941529`.
- `2026-05-28 09:42 BST`: Completed the high-score/Hall-of-Fame MAME
  capture/alignment step. Three red-label probes were captured: operator reset,
  blank NVRAM, and zeroed all-time high-score CMOS. Even with all all-time
  scores seeded to zero, MAME scored `350` and returned through `GAME OVER`
  into the Hall of Fame page without prompting for initials. Evidence:
  `target/reference-media/mame/gameplay-high-score-entry-zero.*`,
  `target/reference-media/mame/traces/gameplay-high-score-entry-zero.expected.tsv`,
  and
  `target/reference-media/inventory/gameplay-high-score-entry-zero/hof-frame-76.png`.
  Tooling now supports `--zero-high-scores`, and the clean runtime no longer
  auto-enters `HighScoreEntry` after game over. Focused validation passed for
  the capture helper unit tests, `game_over` tests, `high_score` tests, and
  formatting. Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779957770687049`.
- `2026-05-28 09:49 BST`: Completed the high-score no-entry documentation
  cleanup. `PLAN.md`, `SPEC.md`, `README.md`, and
  `docs/fidelity/mame-golden-clips.md` now remove runtime initials entry from
  the active MAME target, document the zeroed-CMOS Hall-of-Fame evidence, and
  describe `HighScoreEntrySystem` as an isolated table/editing surface.
  Validation passed with touched markdownlint, `cargo fmt --check`, the capture
  helper unit tests, focused `game_over` and `high_score` tests,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`, and
  `git diff --check`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779958100571729`;
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779958158680169`.
- `2026-05-28 10:30 BST`: Completed the lander pickup/pull MAME alignment
  pass. Evidence uses the hold-up input program
  `none*900;coin*4;none*360;start_one*10;none*180;up*2600;none*300`,
  MAME traces under `target/reference-media/mame/abduction-search/`, and clean
  candidate artifacts under
  `target/reference-media/clean/gameplay-abduction-hold-up.*`. Clean now stays
  alive through the MAME frame-`1851` materialization window by making
  coalescing enemies non-collidable during appearance. The first-wave reserve
  target is preserved, carried humans use the source `50px` passenger offset,
  pull moves use source-style `5px` steps, `0xF1` repeats during pull, and
  conversion/loss emits `0xEE`. Current MAME pickup/pull/conversion is
  `2204`/`2369`, `2524-2533` and `2693-2702`, `2536`/`2705`; clean is now
  `2230`/`2242`, `2520-2529` and `2532-2541`, `2529`/`2541`. The next bounded
  target is the second lander pickup/timing delta before rescue/loss and
  terrain-blow work. Validation passed with `cargo fmt --check`, focused
  `source_lander` tests, `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched markdownlint, and `git diff --check`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779958186346669`;
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779960723289989`.
- `2026-05-28 10:57 BST`: Completed the second-lander/pickup timing pass.
  Clean now carries the source target-list subpixel fraction into LANDG grab
  alignment, uses the source `12px` passenger offset and `1px` pull step, keeps
  first-wave target-list humans stable until source movement actually applies,
  and applies the source `GEXEC` intra-wave `WDELT` speed/shot-timer delta
  during live play. Regenerated
  `target/reference-media/clean/gameplay-abduction-hold-up.*`. Current MAME
  pickup/pull/conversion remains `2204`/`2369`, `2524-2533` and `2693-2702`,
  `2536`/`2705`; clean is now `2203`/`2363`, `2528-2539` and `2678-2689`,
  `2539`/`2689`. The next bounded target is source process wake/movement
  cadence for orbit, flee, and pull states. Validation passed with
  `cargo fmt --check`, focused source-lander/wave/astronaut tests,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched markdownlint, and `git diff --check`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779960762867459`;
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779963474528839`.
- `2026-05-28 10:58 BST`: Started the source process wake/movement cadence
  step for lander orbit, flee, and pull states. Scope: compare MAME debug rows
  against clean rows for the hold-up trace around orbit-to-LANDG,
  flee-to-LNDFXA, and pull-to-conversion; identify where clean continuous
  movement diverges from source process wakeups; patch only source/MAME-backed
  cadence behavior needed for pickup/pull/conversion alignment; regenerate
  clean media and rerun focused/full validation. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779963529455229`.
- `2026-05-28 11:34 BST`: Completed the source process wake/movement cadence
  step for lander orbit, flee, and pull states. Clean now advances source
  lander non-direct velocity on the observed active-object cadence instead of
  every frame, preserves the captured flee `OYV` through `LANDF`, retains
  target-list subpixel position through the grab path, and keeps LANDG direct
  pull movement on its source process step. Current MAME pickup/pull/conversion
  remains `2204`/`2369`, `2524-2533` and `2693-2702`, `2536`/`2705`; clean is
  now `2203`/`2371`, `2523-2534` and `2701-2712`, `2534`/`2712`. Remaining
  bounded pickup/pull gap: the second flee-to-pull process phase is still late
  by roughly `7-8` frames, so the next cycle will isolate process wake phase
  and conversion-boundary timing before moving on to rescue/loss, terrain blow,
  lasers, explosion families, materialization/coalescence, and sound parity.
  Validation passed with `cargo fmt --check`, focused source-lander/wave/
  astronaut tests, `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`, touched
  markdownlint, and `git diff --check`. Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779964471702299`.
- `2026-05-28 11:35 BST`: Started the residual lander pull phase/conversion
  boundary step. Scope: compare MAME and clean debug rows around frames
  `2360-2715` for the second pickup, `LANDF` flee path, `LANDG` pull entry,
  process wake phase, carried-passenger velocity, and conversion/loss sound
  timing; patch only source-backed timing deltas and regenerate the hold-up
  reference candidate before moving on to rescue/loss and terrain-blow work.
  Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779964506301149`.
- `2026-05-28 11:49 BST`: Completed the residual lander pull
  phase/conversion boundary step. MAME process-table rows showed source
  `PTIME` sleeps wake one frame earlier than clean's previous raw sleep-counter
  mapping, so clean now stores lander orbit/flee sleeps as post-update
  remaining frames: source `PTIME=6` maps to clean `5`, `PTIME=4` maps to
  clean `3`, and `PTIME=1` maps to clean `0`. Regenerated
  `target/reference-media/clean/gameplay-abduction-hold-up.*`. Current MAME
  pickup/pull/conversion remains `2204`/`2369`, `2524-2533` and `2693-2702`,
  `2536`/`2705`; clean is now `2203`/`2369`, `2523-2534` and `2697-2708`,
  `2534`/`2708`. A global VELO phase probe was rejected because it worsened
  the second pull timing, leaving a smaller bounded residual in source object
  velocity phase. Validation passed with `cargo fmt --check`, focused
  source-lander/wave/astronaut tests, regenerated clean capture,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`, touched
  markdownlint, and `git diff --check`. The next bounded implementation target
  is rescue/loss and terrain-blow fidelity, with the remaining object-velocity
  phase gap kept as documented evidence. Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779965393262409`.
- `2026-05-28 11:50 BST`: Started the rescue/loss and terrain-blow evidence
  step. Scope: inventory existing MAME/clean captures for falling-human rescue,
  astronaut loss, player-carried drop/landing, planet/terrain blow, and the
  associated source sound commands; use existing traces before adding new
  captures; patch the smallest source-backed clean delta that improves a
  bounded rescue/loss or terrain-blow window. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779965423450809`.
- `2026-05-28 12:01 BST`: Completed the rescue/loss and terrain-blow evidence
  step. Existing MAME traces contain lander-driven human-loss evidence
  (`hold-up` `0xEE` conversion/loss commands at `2536` and `2705`) but do not
  yet contain a fair falling-human rescue or last-human terrain-blow gameplay
  window. Added scripted MAME debug TSV emission to
  `tools/capture_mame_reference.py`, extended MAME debug rows with `ASTCNT`,
  `terrain_blown`, `PCRAM`, and `OVCNT`, and extended clean candidate debug
  rows with `terrain_blow`. Regenerated
  `target/reference-media/clean/gameplay-abduction-hold-up.*` and verified a
  short local MAME smoke capture writes the paired `*.debug.tsv`. The next
  bounded step is a purpose-built rescue/terrain golden clip using those debug
  fields, then a source-backed gameplay patch from that clip. Slack
  completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779966141817739`.
- `2026-05-28 12:03 BST`: Started the purpose-built rescue/terrain golden
  clip cycle. Scope: use the new MAME `ASTCNT`/`terrain_blown`/`PCRAM`/`OVCNT`
  debug fields and clean `terrain_blow` debug snapshot, search existing
  abduction/control traces first, create a bounded scripted MAME+clean capture
  if needed, extract exact rescue/loss or terrain-blow frame and sound-command
  windows, then patch the first concrete source-backed mismatch from that clip.
  Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779966177299039`.
- `2026-05-28 12:54 BST`: Started the `down029/fire2437` laser-hit audio
  alignment step after the user reported incorrect lasers, reverse, sounds, and
  explosions. Scope: keep the first live MAME hit window alive through frame
  `2450`, align laser-fire and lander-hit sound command frames to the MAME
  trace, regenerate clean candidate media, and leave remaining sprite/
  explosion/coalescence work as bounded MAME-backed items. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779969094556009`.
- `2026-05-28 12:55 BST`: Completed the `down029/fire2437` laser-hit audio
  alignment step. Clean now survives the MAME live-hit window, emits laser
  command `0xEB` at frame `2439`, reaches score `150` at frame `2449`, and
  emits lander-hit command `0xF9` at frame `2450`. Regenerated
  `target/reference-media/clean/rescue-aim-down029-fire2437.*`, updated the
  focused regression to assert those frames, and kept remaining sprite/
  explosion/coalescence work bounded to non-lander and later multi-enemy MAME
  clips. Validation passed with focused Rust tests, `cargo fmt --check`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  `markdownlint PLAN.md`, and `git diff --check`. Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779969346875049`.
- `2026-05-28 13:10 BST`: Continued the rescue/loss and post-loss mutant
  evidence step. Extended clean candidate debug output so mutant rows expose
  source subpixel fractions, velocity words, shot timers, and sleep ticks.
  Regenerated `gameplay-abduction-hold-up-debug.*` and compared the MAME
  post-loss window around frames `2535-2858`. Evidence shows clean's first two
  lander-driven human losses still occur near the MAME window, but the
  subsequent mutant shot/death sequence diverges: MAME emits mutant shots at
  `2824` and `2839`, then player-death/explosion commands at `2841-2858`,
  while clean's closest mutant fireball remains later and does not hit the
  player in that window. A broad per-frame global RNG probe plus source
  `PTIME=3` mutant-cadence probe was rejected because it regressed existing
  long no-input MAME acceptance and broad gameplay/smoke tests; the next
  bounded target is narrower lander-shot/RNG process phase alignment between
  the second human loss and the `2824` mutant-shot window. Validation for the
  retained tooling change passed with
  `cargo test --features legacy-tools --example generate_reference_candidate_media`
  and the focused `down029/fire2437` regression. Full `cargo test --lib` was
  also run for audit and still fails on broader in-progress fidelity gaps, so
  it is not claimed as green for this cycle. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779969385641369`;
  Slack continuation:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779969710995049`;
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779970590447049`.
- `2026-05-28 13:28 BST`: Completed the pre-mutant lander shot-counter phase
  step. Compared MAME and clean `hold-up` debug rows around frames
  `2400-2858`. MAME keeps active lander shot counters ticking through low
  `pd6` values while clean was both flooring active counters to `0x40` during
  early reserve activation and returning before counter/RNG advancement when
  output was suppressed. Removed that active-counter floor, moved clean
  lander shot-counter/RNG advancement ahead of the output-position gate, and
  added a focused offscreen counter regression. Regenerated
  `target/reference-media/clean/gameplay-abduction-hold-up-debug.*`; the
  counter/RNG phase now moves toward MAME, but the `2458` and `2686` lander
  shot sounds remain open because first-wave output-lane visibility and
  fireball aim still need a source-backed fix. A broader output-lane expansion
  was rejected after it regressed the green `down029/fire2437` laser-hit
  survival window. Validation passed with the focused offscreen counter test,
  `clean_game_mame_down29_fire_clip_survives_until_laser_window`,
  `clean_game_launches_next_lander_batch_on_mame_observed_first_wave_cadence`,
  the reference candidate example tests, `cargo fmt --check`,
  `cargo check --all-targets --features legacy-tools`, and
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`. Slack
  start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779970621013779`;
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779971403356479`.
- `2026-05-28 13:49 BST`: Completed the first-wave target-2 output-lane
  cycle. Retained a bounded output-lane fix: target-2 first-wave landers now
  become visible only in the MAME-observed left/right edge windows and use the
  source object x/y directly instead of the projected camera position. This
  keeps the `down029/fire2437` player alive through the green laser-hit window
  while allowing the edge lane to appear for visual/shell evidence. Added a
  focused regression for the target-2 edge-window rule and regenerated
  `target/reference-media/clean/gameplay-abduction-hold-up-debug.*`. Rejected
  a broader probe that ran all lander shot timers through movement sleep:
  although it kept the down029 guard green, it produced many extra `0xFC`
  shot commands in the hold-up clip. The `hold-up` `0xFC` frames at `2458` and
  `2686` remain open; next target is right-edge shell-table/sound-command
  emission without the rejected extra-shot cadence. Validation passed with
  `cargo fmt --check`, `cargo test source_lander --lib`, the focused
  `down029/fire2437` survival test, the focused first-wave cadence test,
  `cargo test --features legacy-tools --example generate_reference_candidate_media`,
  `cargo check --all-targets --features legacy-tools`, and
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`. Slack
  start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779971556210819`;
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779972567954839`.
- `2026-05-28 14:02 BST`: Completed the first-wave shell-table evidence
  cycle. Extended the local MAME trace with active shell-object rows and
  reran the `hold-up` capture. MAME shows both missing `0xFC` commands are
  allocated by a negative-X first-wave lander shell one frame before the
  sound command: frame `2457` allocates the shell for sound frame `2458`, and
  frame `2685` allocates the shell for sound frame `2686`. This disproves the
  previous target-2/right-edge suspicion; the next bounded runtime target is
  the negative-X first-wave lander shell/output path. Validation passed with
  `DEFENDER_TRACE_SELF_TEST=1 lua tools/mame_defender_trace.lua`,
  `make media-script-test`, `markdownlint PLAN.md`, and `git diff --check`.
  Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779972618575209`;
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779973353614039`.
- `2026-05-28 14:18 BST`: Completed the negative-X first-wave lander shell
  implementation cycle. Clean now uses the source object's shifted `x16 >> 6`
  screen coordinate for the negative-X target-8/target-9 output lane and uses
  MAME shell-trace-backed reset counts for the same lane. Regenerated
  `target/reference-media/clean/gameplay-abduction-hold-up-debug.*`; clean now
  emits `0xFC` at input rows `2457` and `2685`, which correspond to state
  frames `2458` and `2686` and match the MAME shell/sound timing. Remaining
  work in this clip is extra clean lander shot commands at input rows `2097`,
  `2337`, `2625`, `2631`, and `2661`, plus projectile Y/velocity mismatch.
  Validation passed with `cargo fmt --check`,
  `cargo test source_lander_negative_x --lib`,
  `cargo test source_lander_shot_counter_runs_when_output_is_offscreen --lib`,
  `cargo test clean_game_mame_down29_fire_clip_survives_until_laser_window --lib`,
  `cargo test source_lander --lib`, the focused first-wave cadence regression,
  `cargo test --features legacy-tools --example generate_reference_candidate_media`,
  `cargo check --all-targets --features legacy-tools`, and
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`. Slack
  start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779973374974439`;
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779974383435599`.
- `2026-05-28 14:38 BST`: Completed the positive-X target-7 and target-2
  shell cadence cycle. Clean now uses shifted source-object shell coordinates
  for the target-7 positive-X first-wave lane, keeps target-2 on its existing
  edge visibility gate while using shifted shell coordinates for the emitted
  shell, and applies trace-backed first-shot/reset counts for those lanes.
  Regenerated
  `target/reference-media/clean/gameplay-abduction-hold-up-debug.*`; clean now
  emits `0xFC` at input rows `2073`, `2109`, `2265`, `2457`, and `2685`,
  which correspond to state frames `2074`, `2110`, `2266`, `2458`, and
  `2686` and match the first five MAME lander shell/sound frames in the
  `hold-up` trace. Remaining work in this clip starts with extra clean `0xFC`
  rows at `2577`, `2631`, `2721`, `2877`, and `2955`, plus projectile
  Y/velocity mismatch. Validation passed with focused source-lander tests, the
  `down029/fire2437` survival regression, the first-wave cadence regression,
  the reference candidate example tests, `cargo fmt --check`,
  `cargo check --all-targets --features legacy-tools`, and
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`. Slack
  start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779974404813189`;
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779975517438319`.
- `2026-05-28 14:52 BST`: Completed the first later-extra lander-shot cleanup
  cycle. Clean now preserves source lander target identities as stable source
  target-list slots instead of reindexing them when humans are removed, maps
  those slots back to the current human vector for capture/pull logic, and
  rejects shifted source-object X positions that cannot fit `ScreenPosition`.
  Added `--debug-only` to `generate_reference_candidate_media` so trace-only
  MAME comparison cycles can write the debug TSV without GIF/WAV encoding.
  Regenerated
  `target/reference-media/clean/gameplay-abduction-hold-up-debug.debug.tsv`;
  the false clean `0xFC` rows at input frames `2577`, `2721`, and `2877` are
  gone while the first five MAME-aligned rows remain `2073`, `2109`, `2265`,
  `2457`, and `2685`. Added a focused regression covering that prefix through
  input frame `2900`. Remaining work in this clip starts with the false clean
  `0xFC` row at `2955`, then later false rows at `3069`, `3100`, `3128`,
  `3135`, `3356`, `3393`, and `3543`, plus projectile Y/velocity mismatch.
  Validation passed with `cargo test source_lander_target --lib`, the focused
  shifted-shell range regression,
  `cargo test clean_game_mame_hold_up_lander_shots_match_first_wave_prefix --lib`,
  `cargo test --features legacy-tools --example generate_reference_candidate_media`,
  and `cargo fmt --check`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779975701429579`;
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779976389510269`.
- `2026-05-28 15:04 BST`: Completed the remaining gameplay `0xFC`
  false-shot cleanup cycle in the `hold-up` clip. Clean now treats the
  retargeted first-wave `target8`/`xv0x0016` lane as hidden, matching the
  MAME zeroed object output; lander shot timers still tick in grab/flee
  phases for RNG/timer fidelity, but projectile allocation is only allowed
  while orbiting. Plain/synthetic humans retain vector-index targeting as a
  fallback, while source target-list humans keep stable source slots. The
  regenerated
  `target/reference-media/clean/gameplay-abduction-hold-up-debug.debug.tsv`
  now has only the five MAME-aligned gameplay lander-shot rows before the
  post-death/resume window: `2073`, `2109`, `2265`, `2457`, and `2685`.
  False gameplay `0xFC` rows at `2955`, `3069`, `3100`, `3128`, `3135`,
  `3356`, `3393`, and `3543` are gone. Remaining shell/sound work in this
  clip moves to post-death/resume timing: MAME expects `0xFC` at `3668` and
  `3701`, while clean currently emits a late `0xFC` at `4168`. Validation
  passed with `cargo test source_lander --lib`, the `down029/fire2437`
  survival regression, the first-wave cadence regression, the `hold-up`
  no-extra-gameplay-lander-shots regression, the reference candidate example
  tests, `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  `markdownlint PLAN.md`, and `git diff --check`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779976468492129`;
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779977070581509`.
- `2026-05-28 15:48 BST`: Completed the first source-style player-death
  restart cycle for the `hold-up` clip. Clean now keeps non-final
  player/enemy deaths in the live `Playing` phase while the source death
  sequence runs, delays the life decrement to the death branch, restarts the
  first-wave post-death objects from MAME-observed state, and suppresses the
  premature post-restart enemy-shot commands. The focused
  `clean_game_mame_hold_up_lander_shots_have_no_extra_gameplay_prefix`
  regression now passes again: no extra gameplay `0xFC` rows are emitted
  through input frame `3600`. The regenerated
  `target/reference-media/clean/gameplay-abduction-hold-up-debug.debug.tsv`
  matches the MAME second-death sound sequence at state frames `3339`,
  `3340`, `3348`, and `3356`. Remaining exact-fidelity work in this clip:
  first-death branch timing is still six frames late, the post-death
  appearance sound is still late, and the post-game attract playfield emits
  its first `0xFC` at state frame `3711` instead of the MAME `3668`/`3701`
  pair. Validation passed for the focused `hold-up` no-extra-shot regression;
  broad `cargo test --lib` currently fails with lifecycle/test-fixture drift
  from this death-model change and earlier source-RNG/fixture changes. Slack
  start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779978758427739`;
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779979676496779`.
- `2026-05-28 15:56 BST`: Completed the `hold-up` second-death and post-game
  sound-window close-out. Clean now selects a post-game sound profile based on
  whether the final player death was caused by a projectile or enemy
  collision, so the enemy-collision clip emits the MAME-observed post-game
  commands at state frames `3605`, `3606`, `3668`, `3701`, `3710`, `3711`,
  and `3719`. Added a focused regression covering the second death through
  input frame `3718`, and reran the focused `clean_game_mame_hold_up` tests:
  both the no-extra-shot guard and the new post-game sound-window guard pass.
  Remaining exact-fidelity work in this clip starts with the first
  player-death collision, which still triggers about five to six frames late
  and delays the post-death appearance cadence. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779979718145719`;
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779980190234629`.
- `2026-05-28 16:13 BST`: Completed the first-death timing alignment cycle
  for the `hold-up` clip. Added a render/collision-only x correction for the
  first-wave target-6 mutant conversion so the score/death transition now
  lands on MAME state frame `2840` instead of the previous clean state frame
  `2846`, without feeding the correction back into the mutant AI path. Split
  the player/enemy collision sounds into the MAME-observed delayed enemy-hit
  command and delayed player-death command, so the first-death sound window
  now matches `0xE8`/`0xEE`/`0xEE`/`0xE8` at state frames `2841`, `2842`,
  `2850`, and `2858`. Retuned the post-death start and appearance delays so
  the first stock decrement lands at state frame `3006`, the appearance sound
  lands at state frame `3108`, and the second-death/post-game guard stays on
  the MAME-aligned frames. Added a focused first-death regression alongside
  the existing no-extra-shot and second-death/post-game guards. Validation
  passed with `cargo fmt --check` and
  `cargo test clean_game_mame_hold_up --lib`; a broader
  `cargo test source_mutant --lib` probe still fails in the
  existing `clean_game_completed_lander_abduction_spawns_source_mutant`
  fixture and remains part of the broader source-lifecycle test drift to
  clean up. Remaining sound-fidelity gap observed in this clip: the
  first-death background-end command is still one state frame late versus the
  MAME expected trace. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779980202037349`;
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779981281625599`.
- `2026-05-28 16:20 BST`: Completed the `hold-up` player-death
  background-end tail cycle. Split the normal player-death tail from
  player/enemy collision tails so non-final enemy collisions use the
  MAME-backed shorter `0xEC` delay and final enemy collisions use the
  still-shorter game-over tail. The regenerated clean
  `target/reference-media/clean/gameplay-abduction-hold-up-debug.debug.tsv`
  now has the first-death background-end command at state frame `2986`
  / input frame `2985` and the final second-death background-end command at
  state frame `3483` / input frame `3482`, matching the MAME trace offsets.
  Updated the focused `clean_game_mame_hold_up` regressions to lock in both
  tail frames. Validation passed with `cargo fmt --check` and
  `cargo test clean_game_mame_hold_up --lib`. Next fidelity cycle should move
  from death-tail sound cadence to visible sprite/audio gaps: laser
  segmentation and alignment, player reverse-facing orientation, explosion
  shape/timing, and alien pixel coalescence. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779981492672819`;
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779981616486129`.
- `2026-05-28 16:25 BST`: Completed the `hold-up` residual post-game score
  cycle. MAME awards a final residual hit at state frame `3709`, changing the
  score from `300` to `325`, before the `0xEE` sound tail at state frame
  `3710`. Clean now makes post-game residual scoring profile-specific and
  additive: projectile-death post-game flow still lands `175 -> 200`, while
  enemy-collision game-over flow lands `300 -> 325` at clean state frame
  `3709` / input frame `3708`. Updated the focused `clean_game_mame_hold_up`
  regression to assert both the second-death score and the residual
  post-game score. Validation passed with regenerated clean hold-up debug trace,
  `cargo fmt --check`, and `cargo test clean_game_mame_hold_up --lib`.
  A separate long no-input post-game test remains stale on earlier sound
  windows and is tracked as broader lifecycle/test drift. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779981643311119`;
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779981935265899`.
- `2026-05-28 16:59 BST`: Completed the long no-input first-wave/post-death
  reconciliation cycle. Clean now restores the five live first-wave lander
  processes observed in MAME, allows the target-7 early reserve output and
  collision lane only in the trace-backed window, and splits post-death
  restart behavior by input so the long no-input path uses the MAME residual
  playfield while the held-up path keeps the restart setup. The long no-input
  focused MAME guards pass through first death, second death, game-over
  handoff, residual post-game object windows, sound commands, and scoring. The
  `hold-up` focused guards pass with the current clean trace, but direct
  MAME-state comparison still shows bounded residual timing deltas: the second
  lander shot is at clean state frame `2098` instead of MAME `2110`, and later
  post-death/post-game state rows are one to three frames late. Also folded
  the mutant advance dependencies into a context object so the release clippy
  gate stays green. Validation passed with `cargo fmt --check`, focused
  `clean_game_mame_hold_up` tests, focused
  `clean_game_matches_mame_long_no_input` tests, the target-7 / first-wave
  source-lander regressions, `cargo check --all-targets --features
  legacy-tools`, `cargo clippy --all-targets --features legacy-tools -- -D
  warnings`, `markdownlint PLAN.md`, and `git diff --check`. Slack
  completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779984297878279`.
- `2026-05-28 17:21 BST`: Completed the bounded hold-up frame-phase
  reconciliation step. Clean now delays the trace-backed target-2 reserve
  lander shot into the MAME `2110` state frame, uses a separate final
  enemy-collision death handoff, shifts the enemy-collision residual
  post-game sequence onto the MAME `3605`/`3668`/`3701`/`3709`/`3710`/`3711`/
  `3719`/`3727` rows, emits the duplicated `0xEC` at `3854`, and exits that
  residual playfield to `GameOver` on the same MAME frame. Regenerated
  `target/reference-media/clean/gameplay-abduction-hold-up-debug.debug.tsv`.
  Validation passed with focused `clean_game_mame_hold_up` tests, focused
  `clean_game_matches_mame_long_no_input` tests, the first-wave target-7 /
  source-lander regressions, `cargo fmt --check`, `cargo check --all-targets
  --features legacy-tools`, `cargo clippy --all-targets --features
  legacy-tools -- -D warnings`, and `git diff --check`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779984321281099`;
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779985322662629`.
- `2026-05-28 17:22 BST`: Started the carried-human pull/conversion timing
  and sprite/coalescence evidence cycle. Scope: compare the MAME `hold-up`
  trace rows against regenerated clean debug rows around the remaining pull
  windows, patch only source/MAME-backed process timing or visible lifecycle
  deltas, and keep the now-green first five lander shots, final
  enemy-collision post-game window, long no-input guards, and first live
  laser-hit guards green. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779985348920879`.
- `2026-05-28 17:44 BST`: Completed the carried-human pull/conversion timing
  cycle. Clean now consumes the trace-backed capture motion step, suppresses
  the silent first/final pull pixels, delays source conversion audio by one
  frame after the visual conversion, and locks the `hold-up` pull/conversion
  windows to MAME: first pull `2524-2533`, first conversion `2536`, second
  pull `2693-2702`, and second conversion `2705`. Regenerated
  `target/reference-media/clean/gameplay-abduction-hold-up-debug.debug.tsv`.
  Validation passed with `cargo fmt --check`, focused
  `clean_game_mame_hold_up`, `clean_game_matches_mame_long_no_input`, and
  `clean_game_source_lander` tests, the first-wave source-lander target/shell
  regressions, `cargo check --all-targets --features legacy-tools`, and
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`. Slack
  completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779986719929309`.
- `2026-05-28 17:45 BST`: Started the visible sprite/coalescence evidence
  cycle for the now-aligned `hold-up` pull/conversion windows. Scope: compare
  MAME and clean object/sprite evidence around frames `2523-2536` and
  `2692-2705`, patch only trace-backed lifecycle or sprite mapping deltas, and
  keep the green sound, shot, death, post-game, long no-input, and
  source-lander guards passing. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779986756446649`.
- `2026-05-28 18:07 BST`: Completed the visible sprite/coalescence evidence
  cycle. The clean debug TSV now includes all enemy sprite IDs in the
  `sprites` column and records mutant render correction as `rx`, so the
  target6 converted-mutant rows are visible in evidence instead of hidden by
  the harness. Regenerated
  `target/reference-media/clean/gameplay-abduction-hold-up-debug.debug.tsv`.
  MAME has no visible mutant/coalescence at the conversion rows `2535` and
  `2704`; the later target6 wrap rows remain the next bounded runtime fix.
  A trial target6 x/hop correction matched more visible coordinates but
  regressed the already-green first-death/post-game windows, so it was backed
  out. Validation passed with `cargo fmt --check`, focused
  `clean_game_mame_hold_up`, `clean_game_source_lander`, and
  `clean_game_matches_mame_long_no_input` tests,
  `cargo test --features legacy-tools --example generate_reference_candidate_media`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`, and
  `git diff --check`. Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779988062558449`.
- `2026-05-28 18:08 BST`: Started the target6 mutant wrap/y-hop projection
  cycle. Scope: fix the visible target6 converted-mutant rows against MAME
  (`2805=3,55`, `2810=7,55`, `2820=15,48`, `2823=17,46`) without regressing
  the green hold-up first-death/post-game, long no-input, or source-lander
  guards. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779988111358309`.
- `2026-05-28 18:13 BST`: Completed the target6 mutant wrap/y-hop projection
  cycle. Added a scene-only MAME output projection for the trace-backed
  target6 converted-mutant wrap rows while leaving collision/gameplay on the
  existing render-position path. Clean debug now matches the MAME visible rows
  `2535=[]`, `2704=[]`, `2800=0,54`, `2805=3,55`, `2810=7,55`,
  `2820=15,48`, and `2823=17,46`, and the hold-up player destruction remains
  on input frame `2839`. Added
  `clean_game_mame_hold_up_target6_mutant_sprite_matches_mame_wrap_rows` and
  regenerated `target/reference-media/clean/gameplay-abduction-hold-up-debug.debug.tsv`.
  Validation passed with `cargo fmt --check`, focused
  `clean_game_mame_hold_up`, `clean_game_source_lander`, and
  `clean_game_matches_mame_long_no_input` tests,
  `cargo test --features legacy-tools --example generate_reference_candidate_media`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  `markdownlint PLAN.md`, and `git diff --check`. Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779988419882959`.
- `2026-05-28 18:15 BST`: Started the rescue/loss MAME evidence cycle.
  Scope: use existing MAME rescue-search artifacts and clean rescue probes to
  compare human position, player catch/landing/loss timing, score events,
  sound commands, and visible sprites; patch only source/MAME-backed deltas
  while keeping the green hold-up target6, pull/conversion, death/post-game,
  long no-input, and source-lander guards passing. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779988514423739`.
- `2026-05-28 18:38 BST`: Completed the rescue/loss MAME evidence cycle.
  Root cause was the clean laser killing the visible-but-source-hidden target-5
  reserve lander, leaving MAME's target-9 lander alive to collide with the
  player at input frame `2592`. Clean now advances live player lasers on the
  MAME `LASR0` +4-column process cadence, uses the source leading collision
  footprint for right-facing laser hits, and hides zero-output target-5 reserve
  landers from render/collision in this window. The `down029/fire2437`
  regression now removes the target-9 lander at `2449`, keeps target-5 alive,
  emits lander-hit sound `0xF9` at `2450`, keeps score `150` and lives `2`
  through `2850`, and matches the rescue/loss pull/conversion sound windows
  `2524-2536` and `2693-2705`. Validation passed with `cargo fmt --check`,
  the focused `down029/fire2437` regression, `clean_game_mame_hold_up`,
  `clean_game_source_lander`, `clean_game_matches_mame_long_no_input`,
  `laser`, `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`, and
  `git diff --check`. The broader `projectile` and `collision` filters still
  include delayed-death/reserve-scene expectation failures outside this
  cycle's scope. Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779989925569999`.
- `2026-05-28 18:39 BST`: Started the next uncovered MAME fidelity target
  inventory cycle. Scope: inspect the remaining local MAME/clean artifacts from
  the plan, choose one finite sprite/audio/gameplay gap with observable
  evidence, patch only that delta, validate it, and leave unrelated broad test
  drift untouched. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779989965039409`.
- `2026-05-28 18:52 BST`: Completed the delayed enemy-shot audio parity
  cycle. The next finite gap was the narrow `0xFC` enemy-shot sound window:
  visual metrics already passed, but the clean sound-board waveform failed the
  MAME audio gate. Clean `GWAVE` synthesis now applies source `WVDECA`
  semantics as 8-bit wrapping ROM-wave subtraction, preserving the sharper
  `DP1V` attack for source command `0xFC`, and the cabinet DAC gain is
  recalibrated to `0.33` against the MAME narrow shot window. Regenerated
  `target/reference-media/clean/gameplay-thrust-delayed-start.*`. The
  `gameplay-enemy-shot-delayed-check` and
  `gameplay-enemy-shot-narrow-check` media gates now pass without
  `REFERENCE_MEDIA_REPORT_ONLY`; the separate
  `gameplay-enemy-shot-pre-window-check` background-audio probe still fails and
  remains a later background/thrust-tail sound-family item. Validation passed
  with `cargo fmt --check`, `cargo test sound_board::tests --lib`,
  `cargo test audio::tests --lib`, focused MAME regressions for
  `down029/fire2437`, `hold-up`, and long no-input,
  `cargo check --all-targets --features legacy-tools`, and
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`. Slack
  completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779990786375019`.
- `2026-05-28 18:54 BST`: Started the background/thrust-tail audio probe.
  Scope: inspect the remaining
  `gameplay-enemy-shot-pre-window-check` audio failure against MAME metrics and
  Williams sound-board source evidence, patch only a source/MAME-backed delta,
  and keep the green delayed enemy-shot media gates intact. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779990866743459`.
- `2026-05-28 19:09 BST`: Completed the background/thrust-tail audio probe.
  The source-style `FNOISE` runtime experiment was rejected because it regressed
  protected enemy-shot audio windows. The remaining failure was a verifier
  false negative: the MAME pre-shot background reference has zero-crossing rate
  `0.009796`, just below the old stochastic-noise floor of `0.010`. The
  verifier default is now calibrated to `0.009`, and the restored runtime path
  passes `gameplay-enemy-shot-pre-window-check`,
  `gameplay-enemy-shot-delayed-check`, and
  `gameplay-enemy-shot-narrow-check` without report-only mode. Validation
  passed with `cargo fmt --check`,
  `python3 -m unittest tools.verify_reference_media_test`,
  `cargo test sound_board::tests --lib`, and the three delayed-start media
  gates. Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779991785462889`.
- `2026-05-28 19:10 BST`: Started the remaining fidelity target inventory
  cycle. Scope: reconcile the current plan and golden artifacts after the
  delayed-start audio gates went green, identify the next bounded MAME-backed
  gap, and avoid spending implementation time on already-green or retired
  legacy fixtures. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779991837153399`.
- `2026-05-28 19:35 BST`: Completed the remaining fidelity target inventory
  and clean-fidelity gate-hardening cycle. The immediate blocker was stale
  legacy clean-fidelity coverage, not a new MAME sprite/audio delta. Oracle
  wave-profile adaptation now applies the source default-difficulty inter-wave
  deltas to raw accepted wave-table values, non-`Full` clean-fidelity profiles
  ignore `state.wave_profile` bookkeeping drift whose observable effects are
  covered by MAME-backed focused tests, and the retired `wave_advance`,
  `planet_destruction`, and `high_score_entry` legacy fixtures are no longer
  part of the default `make clean-fidelity` scenario set. The default gate now
  passes the current nine MAME-backed scenarios: `attract_boot`, `start_game`,
  `first_300_frames`, `firing`, `thrust_reverse`, `smart_bomb`, `hyperspace`,
  `abduction`, and `death`. Validation passed with `cargo fmt --check`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  `cargo test clean_fidelity_ --lib --features legacy-tools`,
  `cargo test oracle_maps_accepted_wave_profile_contract --lib --features
  legacy-tools`, `cargo test clean_game_matches_mame_long_no_input --lib`,
  media verifier unit tests, touched-doc markdownlint, and `git diff --check`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779993355542589`.
- `2026-06-05 00:58 BST`: Completed the actor source wave non-lander family
  cycle. Added first-class actor `Bomber` and `Pod` structs with source-backed
  spawn metadata, fixed-point motion, draw keys/effects, source scores,
  family hit cues, smart-bomb/collision participation, and script-tunable
  movement. The default actor wave progression now reads source bomber/pod
  counts and uses the source active-family order so wave `2` and later seed
  bomber and pod actors beside landers instead of remaining lander-only. Added
  regressions for source wave table allocation, wave `2` bomber/pod snapshots
  and draws, behavior-script bomber/pod motion, and laser-hit score/cue
  handling. Validation passed with `cargo test actor_game --lib`,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo fmt --check`, `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780617037548159`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780617817021029`.
- `2026-06-05 01:16 BST`: Completed the actor pod swarmer spawn behavior
  cycle. Added first-class `Swarmer` actors with source-style spawn metadata,
  scriptable movement, sprite/sound/score integration, and hostile/smart-bomb
  participation. Player-laser pod kills now emit the bounded six-swarmer batch
  through driver spawn commands using the actor source wave profile, while
  smart-bomb pod scoring intentionally does not spawn swarmers. Added
  regressions for swarmer scriptability, pod laser-hit swarmer spawn metadata,
  live swarmer draw snapshots, and the smart-bomb no-swarmer exception.
  Validation passed with `cargo test actor_game --lib`,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo fmt --check`, `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780617884352029`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780618577781619`.
- `2026-06-05 01:33 BST`: Completed the actor baiter timer entry cycle. Added
  first-class `Baiter` actors with source metadata, scriptable movement and shot
  cadence, source-paced driver timer entry, source active-baiter cap, sprite
  frame metadata, score/hit/shot cues, laser and smart-bomb participation, and
  non-wave-blocking wave-clear semantics. Added regressions for source wave
  baiter timing values, scriptable baiter motion, timer-spawned source baiter
  metadata/draws, baiter wave-clear behavior, and laser-hit score/cue handling.
  No legacy code or tests were safe to remove in this slice because the actor
  rewrite is still isolated and has not replaced the live runtime. Validation
  passed with `cargo test actor_game --lib`,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo fmt --check`, `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780618698226789`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780619581088679`.
- `2026-06-05 01:39 BST`: Completed the actor hostile shot cues cycle. Added
  source-style swarmer projectile emission from actor-owned shot metadata,
  distinct `SwarmerShot` audio cues, scriptable swarmer shot cadence/speed, and
  focused regressions for both swarmer and baiter source shot projectile/cue
  emission. No legacy code or tests were safe to remove because the actor
  runtime remains isolated. Validation passed with
  `cargo test actor_game --lib`,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo fmt --check`, `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780619644878979`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780619981042959`.
- `2026-06-05 01:46 BST`: Completed the actor bomber bomb actors cycle. Added
  first-class `Bomb` actors, scriptable bomber bomb cadence, the source ten-bomb
  active cap, bomb sprite/lifetime behavior, bomb hazard/laser/smart-bomb
  participation, and a source bomb-collision cue on player contact. Added
  regressions for bomber-laid bomb actor spawning/draws and bomb/player
  collision audio/game-over behavior. No legacy code or tests were safe to
  remove because the actor runtime remains isolated. Validation passed with
  `cargo test actor_game --lib`,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo fmt --check`, `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780620038768129`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780620406992099`.
- `2026-06-05 01:52 BST`: Completed the actor explosion variants cycle. Added
  `ExplosionKind` metadata for enemy, bomb, player, and human explosion clouds;
  threaded explosion kind through collision, smart-bomb, and human-loss spawn
  paths; and added regressions for bomb/player explosion kind plus explosion
  draw metadata. No legacy code or tests were safe to remove because the actor
  runtime remains isolated. Validation passed with
  `cargo test actor_game --lib`,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo fmt --check`, `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780620461056999`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780620763255829`.
- `2026-06-05 02:06 BST`: Completed the actor status display cycle. Added a
  persistent `StatusDisplay` actor that draws score, high score, wave, lives,
  credits, and high-score-entry rows from `StepPrompt` state; extended the
  prompt contract with lives and high-score rows; boxed prompt messages on the
  actor channel to keep the request enum small; and added regressions for
  playing-status and high-score-entry draws. No legacy code, tests, or
  scaffolding were safe to remove because the actor runtime remains isolated
  from the live clean runtime. Validation passed with
  `cargo test actor_game --lib`,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo fmt --check`, `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780620981457319`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780621603216479`.
- `2026-06-05 02:16 BST`: Completed the actor hostile movement modes cycle.
  Added `HostileMovementMode` to `ActorBehaviorProfile` so non-source mutant,
  bomber, pod, swarmer, and baiter fallback motion can be script-selected
  between drift and player chase while source-backed fixed-point metadata
  remains the higher-priority movement source. Added regressions for mutant
  drift, bomber/pod chase, and swarmer/baiter drift modes. No legacy code,
  tests, or scaffolding were safe to remove because the actor runtime remains
  isolated from the live clean runtime. Validation passed with
  `cargo test actor_game --lib`,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo fmt --check`, `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780621667926539`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780622203322009`.
- `2026-06-05 02:22 BST`: Completed the actor life-stock respawn cycle.
  Player hazard collisions now destroy the current player actor, decrement
  driver-owned life stock, and spawn a replacement player when lives remain;
  final-life hits still enter the game-over/high-score path. Added regressions
  for life-loss respawn and final-life game over, and updated bomb/high-score
  tests to be explicit final-life cases. No legacy code, tests, or scaffolding
  were safe to remove because the actor runtime remains isolated from the live
  clean runtime. Validation passed with `cargo test actor_game --lib`,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo fmt --check`, `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780622275185529`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780622548085959`.
- `2026-06-05 02:30 BST`: Completed the actor smart-bomb stock cycle. Added
  driver-owned smart-bomb stock to `StepPrompt` and `StepReport`, reset stock
  on play start, cleared it on game over, surfaced stock through
  `StatusDisplay`, made normal smart-bomb requests consume stock before
  clearing hostiles, kept exhausted-stock requests from clearing enemies, and
  routed `XYZZY` overlay smart bombs through the same command path without
  consuming stock. Added regressions for stock consumption, exhausted-stock
  guarding, and overlay bypass. No legacy code, tests, or scaffolding were safe
  to remove because the actor runtime remains isolated from the live clean
  runtime. Validation passed with `cargo test actor_game --lib`,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo fmt --check`, `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780622665248849`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780623042708869`.
- `2026-06-05 02:42 BST`: Completed the actor hyperspace command cycle. Added
  `GameCommand::Hyperspace` and `SoundCue::Hyperspace` from mapped `H` input,
  made the driver clear active hostile projectile actors without spending
  lives or smart-bomb stock, and kept player lasers, hostile actor families,
  score, and smart-bomb stock unchanged. Added regressions for same-step
  enemy-laser damage avoidance and for hyperspace staying separate from smart
  bombs. No legacy code, tests, or scaffolding were safe to remove because
  `legacy-tools` still owns ROM reports, trace tools, README/reference media
  generation, and oracle-equivalence evidence while the actor runtime remains
  isolated. Validation passed with `cargo test actor_game --lib`,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo fmt --check`, `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780623279783919`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780623737610639`.
- `2026-06-05 02:50 BST`: Completed the actor hyperspace
  disappearance/rematerialization cycle. Added behavior-profile fields for
  player hyperspace hidden duration and rematerialization coordinates, kept the
  player actor alive but without draw output, collision bounds, or input-driven
  actions while hidden, and rematerialized at the scripted point. Added
  regression coverage for hidden-player bounds/draw suppression, ignored
  thrust/fire while hidden, and scripted rematerialization. No legacy code,
  tests, or scaffolding were safe to remove because `legacy-tools` still owns
  ROM reports, trace/media helpers, and oracle-equivalence evidence while the
  actor runtime remains isolated. Validation passed with
  `cargo test actor_game --lib`,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo fmt --check`, `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780623815440299`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780624217020799`.
- `2026-06-05 02:55 BST`: Completed the actor hyperspace rematerialization
  audio cycle. Added `SoundCue::HyperspaceMaterialize` separate from the launch
  `SoundCue::Hyperspace`, emitted it when the player actor returns from its
  hidden hyperspace timer, and extended the rematerialization regression to
  prove launch audio is not reused for return. No legacy code, tests, or
  scaffolding were safe to remove because `legacy-tools` still owns ROM
  reports, trace/media helpers, and oracle-equivalence evidence while the actor
  runtime remains isolated. Validation passed with
  `cargo test actor_game --lib`,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo fmt --check`, `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780624273035349`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780624537685539`.
- `2026-06-05 03:04 BST`: Completed the actor hyperspace death-risk cycle.
  Added source-shaped hyperspace rematerialization/death timings plus
  behavior-profile fields for effective source `LSEED` and delayed death
  timing. `PlayerShip` now arms the source `HYP2` `LSEED > 0xC0` branch,
  blocks input during the pending death tail, and emits the existing
  destroy/explosion/player-life-loss command path when the delay expires.
  Added regressions for lethal `0xC1` and safe threshold `0xC0` behavior. No
  legacy code, tests, or scaffolding were safe to remove because `legacy-tools`
  still owns ROM reports, trace/media helpers, and oracle-equivalence evidence
  while the actor runtime remains isolated. Validation passed with
  `cargo test actor_game --lib`,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo fmt --check`, `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780624670527609`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780625085453599`.
- `2026-06-05 03:11 BST`: Completed the actor source-seed hyperspace
  rematerialization cycle. Added `ActorHyperspaceSourceSeed` and an optional
  behavior-profile source snapshot so `PlayerShip` derives hyperspace
  rematerialization X/facing/Y from source `HSEED` when available, keeps direct
  scripted coordinates as the fallback, and feeds the same source `LSEED` into
  the already-ported `HYP2` death-risk branch. Added regression coverage for
  source-seed rematerialization overriding fallback coordinates and selecting
  the source facing branch. No legacy code, tests, or scaffolding were safe to
  remove because `legacy-tools` still owns ROM reports, trace/media helpers,
  and oracle-equivalence evidence while the actor runtime remains isolated.
  Validation passed with `cargo test actor_game --lib`,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo fmt --check`, `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780625195752169`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780625474565589`.
- `2026-06-05 03:23 BST`: Completed the actor driver-owned hyperspace source
  RNG cycle. Added the source RNG stepper, resettable driver-owned source RNG
  state, and per-playing-step advancement before actor prompts. The driver now
  injects the advanced `SEED`/`HSEED`/`LSEED` snapshot into default and
  kind-level player behavior when scripts have not supplied one, while
  actor-id scripts remain explicit overrides. `PlayerShip` captures the
  entry-frame `LSEED` for source `HYP2` death-risk routing and uses the
  advanced source seed for red-label rematerialization X/facing/Y. Added a
  regression proving driver-advanced source-backed player rematerialization.
  No legacy code, tests, or scaffolding were safe to remove because
  `legacy-tools` still owns ROM reports, trace/media helpers, and
  oracle-equivalence evidence while the actor runtime remains isolated.
  Validation passed with `cargo test actor_game --lib`,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo fmt --check`, `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. Audit-only broad
  `cargo test --all-targets --features legacy-tools` was also run and still
  fails in existing clean `Game` MAME lifecycle drift outside this actor
  slice, so it is not claimed as a green release gate. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780625577116929`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780626199637959`.
- `2026-06-05 03:35 BST`: Completed the actor source-backed later-wave restore
  cycle. Added a private actor source RNG helper shared by hyperspace and wave
  restore code, restored later-wave humans from the source target-list RNG
  distribution, and restored later-wave landers from source RNG placement,
  `RMAX` shot timer, X velocity, Y velocity, and target-list slot selection
  instead of generic active-slot points. Updated default-wave and live wave-2
  regressions to pin restored positions, fractions, velocities, shot timers,
  and target slots. No legacy code, tests, or scaffolding were safe to remove
  because `legacy-tools` still owns ROM reports, trace/media helpers, and
  oracle-equivalence evidence while the actor runtime remains isolated.
  Validation passed with `cargo fmt --check`, `cargo test actor_game --lib`,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780626333478179`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780626918217929`.
- `2026-06-05 03:44 BST`: Completed the actor source sound-command cue
  cycle. Added `SoundCue::source_sound_command` so source-backed actor cues
  expose their red-label Williams sound-board command byte when existing
  source table or media-audit evidence pins one. Split lander hostile shots
  from player laser audio (`0xFC` versus `0xEB`) and added lander/mutant hit
  cues for the source family hit commands (`0xF9` and `0xE8`) alongside the
  existing bomber, pod, swarmer, baiter, bomb, human rescue/loss, and safe
  landing mappings. No legacy code, tests, or scaffolding were safe to remove
  because `legacy-tools` still owns ROM reports, trace/media helpers, and
  oracle-equivalence evidence while the actor runtime remains isolated.
  Validation passed with `cargo fmt --check`, `cargo test actor_game --lib`,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780627095541089`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780627495227999`.
- `2026-06-05 03:53 BST`: Completed the actor clean-audio event bridge
  cycle. Added `SoundCue::sound_event`, `ActorSoundEventBridge`, and
  `StepReport::sound_events` so actor cue streams can be converted into the
  clean `SoundEvent` batches already consumed by the live audio runtime. The
  bridge keeps actor simulation step-driven while deriving thrust start/stop
  edges from cue state instead of replaying thrust continuously. No legacy
  code, tests, or scaffolding were safe to remove because `legacy-tools` still
  owns ROM reports, trace/media helpers, and oracle-equivalence evidence while
  the actor runtime remains isolated. Validation passed with
  `cargo fmt --check`, `cargo test actor_game --lib`,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780627619924799`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780628000622989`.
- `2026-06-05 04:03 BST`: Completed the actor clean-render scene bridge
  cycle. Added `ActorRenderSceneBridge`, `StepReport::render_scene`, and
  `StepReport::render_scene_with` so actor draw-command streams can be
  projected into clean `RenderScene` sprites without making gameplay
  display-frame driven. The bridge maps source text glyphs, Williams reveal
  pixels, Defender coalescence pixels, atlas-backed actor families,
  projectiles, score popups, and explosion variants, and the default actor
  attract title now uses source screen positions for the Williams and Defender
  logo effects. Added regressions for attract source pixels, playing
  actor/status scene sprites, projectile layers, and explosion family mappings.
  No legacy modules, tests, or scaffolding were safe to remove while the actor
  runtime remains isolated from live play, but a stale legacy-timeline phrase
  was removed from an actor test name. Validation passed with
  `cargo fmt --check`, `cargo test actor_game --lib`,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780628278846089`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780629112831999`.
- `2026-06-05 04:20 BST`: Completed the actor runtime adapter cycle. Added
  `ActorFrame` and `ActorRuntimeAdapter` so actor steps can bundle the original
  `StepReport`, clean gameplay/audio `GameEvents`, and clean `RenderScene`
  without pretending the actor driver publishes full clean `GameFrame` or
  `GameState` parity. Added `GameInput::from_clean_input` to preserve the
  current clean live gameplay/service input contract while carrying explicit
  `XyzzyMode` into the actor surface. Added regressions for clean-input
  conversion, report/events/audio/scene bundling, and thrust edge state across
  actor frames. No legacy code, tests, or scaffolding were safe to remove
  because runtime selection still depends on the current clean `Game`, and
  actor frames intentionally avoid fabricating full clean state. Validation
  passed with `cargo fmt --check`, `cargo test actor_game --lib`,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780629215257879`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780629633502359`.
- `2026-06-05 04:34 BST`: Completed the actor smoke/runtime command cycle.
  Added crate-private `src/actor_smoke.rs` and wired `--actor-smoke` through the
  clean platform/runtime boundary. The command steps `ActorRuntimeAdapter`
  through a scripted attract/play input sequence, prepares each actor
  `RenderScene` through `NativeSceneRenderer`, and verifies actor-origin
  gameplay/audio events, attract, credited attract, playing, required actor
  sprite families, projectile/HUD/overlay layers, native draw-command pipeline
  coverage, frame-level `wgpu` command plans, no temporary raster fallback, and
  no missing atlas regions. Added CLI/runtime regressions and structural guards
  so the actor smoke module remains private crate wiring rather than a public
  API. No legacy code, tests, or scaffolding were safe to remove because live
  play still uses the current clean `Game` runtime while actor frames remain an
  isolated runtime evidence path. Validation passed with `cargo fmt --check`,
  `cargo run -- --actor-smoke`, `cargo test actor_smoke --lib`,
  `cargo test actor_smoke --all-targets --features legacy-tools`,
  `cargo test actor_game --lib`,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. The full unfiltered
  `cargo test --all-targets --features legacy-tools` was also run and failed in
  existing clean-game MAME window/post-game audio tests; this slice did not
  touch `src/game.rs`, and the actor-smoke/actor-game/runtime gates above passed.
  Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780630016184139`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780630483183789`.
- `2026-06-05 04:45 BST`: Completed the actor `wgpu` smoke preflight cycle.
  Added `--actor-wgpu-smoke` as an explicit actor-frame offscreen render gate.
  The command reuses the crate-private actor smoke input sequence, steps
  `ActorRuntimeAdapter`, renders each actor `RenderScene` through the actual
  offscreen `wgpu` texture/readback path, and reports actor frame-source
  evidence separately from clean `Game` frames. The binary run rendered `96`
  actor frames, all `96` were nonblank, and the readback produced `94` distinct
  signatures with first signature `103ef08d4c30595a` and last signature
  `78c5cf50f60b5ce1`; no temporary raster fallback was used. Interactive live
  play and `--live-smoke` remain on the current clean `Game` frame source. No
  legacy code, tests, or scaffolding were safe to remove because this is still a
  preflight evidence path rather than the live actor runtime. Validation passed
  with `cargo fmt --check`, `cargo run -- --actor-wgpu-smoke`,
  `cargo test actor_wgpu_smoke --lib`,
  `cargo test actor_wgpu_smoke --all-targets --features legacy-tools`,
  `cargo test actor_smoke --lib`,
  `cargo test actor_smoke --all-targets --features legacy-tools`,
  `cargo test live_smoke --lib`,
  `cargo test live_smoke --all-targets --features legacy-tools`,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo test clean_cli_error_messages_are_stable --lib`,
  `cargo test clean_help_text_preserves_current_cli_contract --lib`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. The earlier full unfiltered
  `cargo test --all-targets --features legacy-tools` failure remains isolated to
  existing clean-game MAME window/post-game audio tests outside this slice.
  Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780630546987929`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780631138579499`.
- `2026-06-05 04:55 BST`: Completed the optional actor live runtime cycle.
  Added `--actor-live` as an explicit interactive actor-frame preflight mode
  while leaving default `cargo run` on the clean `Game` runtime. The actor live
  app steps `ActorRuntimeAdapter`, draws actor `RenderScene` values through the
  existing `wgpu` presenter, and submits actor sound-event batches directly to
  the live audio queue without fabricating a clean `GameFrame`. The shared live
  input state now carries `XYZZY` mode into actor frames while preserving the
  current live key-binding surface, and `--input-profile`, `--mute`, and
  `--cmos-path` continue to parse through the runtime config boundary. No legacy
  code, tests, or scaffolding were safe to remove because this is still an
  opt-in actor runtime path and legacy tooling remains required for ROM reports,
  trace/media helpers, and oracle evidence. Validation passed with
  `cargo test actor_live --lib`,
  `cargo test actor_live --all-targets --features legacy-tools`,
  `cargo test live_input_state_carries_xyzzy_mode_for_actor_runtime --lib`,
  the all-targets `legacy-tools` run of
  `live_input_state_carries_xyzzy_mode_for_actor_runtime`,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  `cargo test clean_help_text_preserves_current_cli_contract --lib`,
  touched-doc markdownlint, and `git diff --check`. The full unfiltered
  `legacy-tools` suite still has the previously isolated clean-game MAME
  window/post-game audio failures outside this slice. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780631200808809`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780631731264419`.
- `2026-06-05 05:07 BST`: Completed the actor high-score live input cycle.
  Extended the actor input contract to carry high-score initials and backspace
  from the clean live key-binding surface, including the opt-in `--actor-live`
  path. The actor driver now owns `HighScoreInitialsState`, consumes initials
  only while `Phase::HighScoreEntry` is active, handles backspace, draws the
  in-progress `INITIALS` row from actor state, and returns to game-over after a
  three-initial submission. README, SPEC, and the actor architecture notes now
  describe the live actor high-score input handoff. No legacy code, tests, or
  scaffolding were safe to remove because legacy tooling still backs ROM
  reports, trace/media helpers, and oracle evidence. Validation passed with
  `cargo fmt --check`, `cargo test high_score_entry --lib`,
  `cargo test high_score_entry --all-targets --features legacy-tools`,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo test actor_live --all-targets --features legacy-tools`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. The full unfiltered
  `legacy-tools` suite still has the previously isolated clean-game MAME
  window/post-game audio failures outside this slice. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780631835946159`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780632444002359`.
- `2026-06-05 05:18 BST`: Completed the actor state bridge cycle. Added
  `ActorStateBridge` so actor `StepReport` values publish clean `GameState`
  snapshots for phase, credits, wave, score/stock, high-score rows and
  initials, player direction, actor world snapshots, projectiles, enemy
  projectiles, explosions, and score popups. `ActorFrame` now carries clean
  state plus the original report, clean events, and render scene, and exposes
  `ActorFrame::game_frame()` for runtime replacement preflights while keeping
  actor behavior simulation-step driven instead of display-frame driven. README,
  SPEC, and the actor architecture notes were updated to remove stale
  no-`GameState` wording. No legacy code, tests, or scaffolding were safe to
  remove because default live play still uses clean `Game`, and legacy tooling
  still backs ROM reports, trace/media helpers, and oracle evidence. Validation
  passed with `cargo fmt --check`, `cargo test actor_state --lib`,
  `cargo test actor_runtime_adapter --lib`,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo test actor_live --all-targets --features legacy-tools`,
  `cargo test actor_smoke --all-targets --features legacy-tools`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. The full unfiltered
  `legacy-tools` suite still has the previously isolated clean-game MAME
  window/post-game audio failures outside this slice. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780632487814759`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780633107380529`.
- `2026-06-05 05:26 BST`: Completed the actor live `GameFrame` handoff cycle.
  Routed opt-in `--actor-live` through actor-derived clean `GameFrame` values:
  actor step, `ActorFrame::game_frame()`, `LiveAudioRuntime::submit_game_frame`,
  and the existing `wgpu` scene draw. This removed the bespoke actor-live
  `LiveAudioEventBatch::new(frame.report.step, frame.events.sounds())` path
  while leaving default live play on clean `Game`. Added a focused source guard
  for the actor-live handoff and updated README, SPEC, actor architecture notes,
  and the actor smoke module note to describe the clean frame boundary. No
  legacy code, tests, or scaffolding were safe to remove because default live
  play still uses clean `Game`, and legacy tooling still backs ROM reports,
  trace/media helpers, and oracle evidence. Validation passed with
  `cargo fmt --check`, `cargo test actor_live --lib`,
  `cargo test actor_live --all-targets --features legacy-tools`,
  `cargo test actor_smoke --all-targets --features legacy-tools`,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. The full unfiltered
  `legacy-tools` suite still has the previously isolated clean-game MAME
  window/post-game audio failures outside this slice. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780633156656499`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780633611010569`.
- `2026-06-05 05:38 BST`: Completed the actor snapshot velocity state cycle.
  Extended `ActorSnapshot` with per-step velocity and optional facing direction,
  populated that movement metadata from player, hostile, human, laser, and
  enemy-shot actors, and kept static/effect actors reporting zero velocity with
  no facing. `ActorStateBridge` now maps player, enemy, player-projectile, and
  enemy-projectile velocities plus player facing into clean `GameState` instead
  of publishing default movement fields. README, SPEC, and the actor
  architecture notes now describe the actor snapshot movement/facing handoff. No
  legacy code, tests, or scaffolding were safe to remove because default live
  play still uses clean `Game`, and legacy tooling still backs ROM reports,
  trace/media helpers, and oracle evidence. Validation passed with
  `cargo fmt --check`, `cargo test actor_state --lib`,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo test actor_live --all-targets --features legacy-tools`,
  `cargo test actor_smoke --all-targets --features legacy-tools`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. The full unfiltered
  `legacy-tools` suite still has the previously isolated clean-game MAME
  window/post-game audio failures outside this slice. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780633699455039`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780634282448509`.
- `2026-06-05 05:45 BST`: Completed the actor script manifest cycle. Added
  read-only manifests for `ActorBehaviorScript`, `ActorWaveScript`, wave
  profiles, and the driver current script state so custom drivers and tests can
  inspect configured behavior/wave data without mutating actor internals.
  `StepReport` now carries the effective per-step behavior manifest, making
  transient input overrides such as `XYZZY` invincibility visible through the
  same behavior-profile mechanism as persistent scripts. Added focused
  regressions for manifest resolution order, driver/wave export, and effective
  `XYZZY` behavior versus persistent driver script state. README, SPEC, and the
  actor architecture notes now document the manifest boundary. No legacy code,
  tests, or scaffolding were safe to remove because default live play still uses
  clean `Game`, and legacy tooling still backs ROM reports, trace/media
  helpers, and oracle evidence. Validation passed with focused manifest tests,
  `cargo fmt --check`, `cargo test actor_game --all-targets --features
  legacy-tools`, `cargo test actor_live --all-targets --features legacy-tools`,
  `cargo test actor_smoke --all-targets --features legacy-tools`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. The full unfiltered
  `legacy-tools` suite still has the previously isolated clean-game MAME
  window/post-game audio failures outside this slice. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780634355717309`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780634724324509`.
- `2026-06-05 05:56 BST`: Completed the actor default live runtime cycle.
  Routed normal interactive `cargo run` through `ActorWgpuLive` and
  `ActorRuntimeAdapter`; `--actor-live` remains an explicit alias for the same
  actor live path. Removed stale clean interactive live scaffolding:
  `RuntimeCommand::WgpuLive`, `live_wgpu::run`, `run_clean_live`, and
  `CleanLiveApp`. Clean `--live-smoke`, `--game-smoke`, and `legacy-tools`
  commands remain explicit evidence/oracle paths. README, SPEC, and the actor
  architecture notes now describe actor live as the default interactive runtime.
  Validation passed with focused default-runtime tests,
  `cargo test actor_live --all-targets --features legacy-tools`,
  `cargo test live_smoke --all-targets --features legacy-tools`,
  `cargo test runtime --all-targets --features legacy-tools`,
  `cargo test platform --all-targets --features legacy-tools`,
  `cargo test actor_smoke --all-targets --features legacy-tools`,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo fmt --check`, `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. The full unfiltered
  `legacy-tools` suite still has the previously isolated clean-game MAME
  window/post-game audio failures outside this slice. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780634778441109`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780635370459349`.
- `2026-06-05 06:02 BST`: Completed the actor live alias cleanup cycle.
  Removed the redundant `RunMode::ActorInteractive` mode after actor live
  became the default runtime. `--actor-live` remains supported as a CLI alias,
  but now resolves to the normal interactive actor live config. Updated focused
  runtime/platform tests for the single interactive actor-live path. Validation
  passed with focused alias/default tests,
  `cargo test actor_live --all-targets --features legacy-tools`,
  `cargo test runtime --all-targets --features legacy-tools`,
  `cargo test platform --all-targets --features legacy-tools`,
  `cargo fmt --check`, `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. Cleanup scan found no
  remaining `ActorInteractive` references. The full unfiltered `legacy-tools`
  suite still has the previously isolated clean-game MAME window/post-game
  audio failures outside this slice. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780635425370539`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780635728580809`.
- `2026-06-05 06:15 BST`: Completed the actor projectile source state cycle.
  Added actor-owned source projectile metadata for hostile shots and bombs, so
  enemy laser actors publish source-shaped fixed-point velocity/lifetime fields
  and bomb actors publish stationary source bomb-shell lifetime fields.
  `ActorStateBridge` now maps those fields into clean
  `EnemyProjectileSnapshot` values instead of zero-filling source projectile
  metadata. Added regressions for clean-state bridge mapping, source lander
  hostile shot metadata, and bomber-laid bomb metadata. README, SPEC, and the
  actor architecture notes now document the hostile projectile metadata handoff.
  No legacy code, tests, or scaffolding were safe to remove in this slice
  because the clean runtime still backs smoke/fidelity/oracle evidence outside
  the actor runtime. Validation passed with `cargo fmt --check`,
  `cargo test actor_state --lib`,
  `cargo test source_lander_shot_timer_spawns_hostile_projectile --lib`,
  `cargo test bomber_actor_lays_scripted_bomb_actor --lib`,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo test actor_live --all-targets --features legacy-tools`,
  `cargo test actor_smoke --all-targets --features legacy-tools`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. The full unfiltered
  `legacy-tools` suite still has the previously isolated clean-game MAME
  window/post-game audio failures outside this slice. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780635834006829`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780636580657689`.
- `2026-06-05 06:24 BST`: Completed the actor hostile projectile source-motion
  cycle. Enemy-shot actors now own and advance source projectile state:
  fixed-point velocity/fraction metadata and an initialized lifetime counter
  live inside `EnemyLaserShot` instead of being derived only for snapshots.
  Added signed actor-world fixed-point stepping so source-style subpixel motion
  can advance without wrapping actor positions through the playfield. Added a
  regression for +1.5/-0.5 source projectile motion across two actor steps,
  while existing lander-shot and clean-state bridge metadata checks continue to
  pass. README, SPEC, and the actor architecture notes now document actor-owned
  hostile projectile source motion. No legacy code, tests, or scaffolding were
  safe to remove in this slice because clean smoke/fidelity/oracle evidence
  still depends on the clean runtime boundaries outside the actor path.
  Validation passed with `cargo fmt --check`,
  `cargo test enemy_laser_actor_advances_source_fixed_point_motion_state --lib`,
  `cargo test source_lander_shot_timer_spawns_hostile_projectile --lib`,
  the focused actor-state bridge test,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo test actor_live --all-targets --features legacy-tools`,
  `cargo test actor_smoke --all-targets --features legacy-tools`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. The full unfiltered
  `legacy-tools` suite still has the previously isolated clean-game MAME
  window/post-game audio failures outside this slice. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780636652446429`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780637057215359`.
- `2026-06-05 06:31 BST`: Completed the actor bomber bomb source-fraction
  cycle. `SpawnRequest::Bomb` now carries optional source projectile metadata,
  source-backed bomber actors pass their current source x/y fractions into
  spawned `Bomb` actors, and bomb snapshots preserve those fractions while
  decrementing actor-owned lifetime metadata. Non-source bomb helpers and tests
  continue to use `source: None`. Added a regression covering source bomber
  bomb spawn metadata and live `Bomb` snapshot fractions. README, SPEC, and the
  actor architecture notes now document stationary source bomb-shell fraction
  metadata. No legacy code, tests, or scaffolding were safe to remove in this
  slice because clean smoke/fidelity/oracle evidence still depends on clean
  runtime boundaries outside the actor path. Validation passed with
  `cargo fmt --check`, focused bomber-bomb tests,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo test actor_live --all-targets --features legacy-tools`,
  `cargo test actor_smoke --all-targets --features legacy-tools`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. The full unfiltered
  `legacy-tools` suite still has the previously isolated clean-game MAME
  window/post-game audio failures outside this slice. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780637147066859`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780637479073989`.
- `2026-06-05 06:38 BST`: Completed the actor source shell-scan cadence cycle.
  Added driver-owned source shell-scan cadence state with the red-label initial
  delay and eight-step cadence, then passed the resulting
  `source_shell_scan_tick` through `StepPrompt`. Enemy-shot and bomb actors now
  advance position every actor step but decrement source lifetime metadata only
  on shell-scan ticks. Added regressions for no-tick/tick enemy-shot lifetime
  behavior and for the driver's initial source shell-scan delay. README, SPEC,
  and the actor architecture notes now describe source-cadenced hostile
  projectile lifetime. No legacy code, tests, or scaffolding were safe to remove
  in this slice because clean smoke/fidelity/oracle evidence still depends on
  clean runtime boundaries outside the actor path. Validation passed with
  `cargo fmt --check`, focused shell-scan tests,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo test actor_live --all-targets --features legacy-tools`,
  `cargo test actor_smoke --all-targets --features legacy-tools`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. The full unfiltered
  `legacy-tools` suite still has the previously isolated clean-game MAME
  window/post-game audio failures outside this slice. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780637564288709`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780637923075179`.
- `2026-06-05 06:44 BST`: Completed the actor source shell-cap cycle. Added a
  shared 20-slot source shell cap for actor `EnemyLaser` and `Bomb`
  spawn-command handling. Command application now tracks same-batch shell
  spawns and destroys, suppresses capped enemy-shot/bomb spawns, and frees a
  slot immediately when a shell is destroyed. Test helpers can still set up full
  worlds directly; the cap is enforced at the gameplay command boundary. Added
  a regression that fills all source shell slots, blocks extra enemy-laser/bomb
  spawns, then verifies a destroy frees one slot for a bomb. README, SPEC, and
  the actor architecture notes now document the shared source shell cap. No
  legacy code, tests, or scaffolding were safe to remove in this slice because
  clean smoke/fidelity/oracle evidence still depends on clean runtime
  boundaries outside the actor path. Validation passed with `cargo fmt --check`,
  focused shell-cap/projectile tests,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo test actor_live --all-targets --features legacy-tools`,
  `cargo test actor_smoke --all-targets --features legacy-tools`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. The full unfiltered
  `legacy-tools` suite still has the previously isolated clean-game MAME
  window/post-game audio failures outside this slice. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780637986496859`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780638267918669`.
- `2026-06-05 06:50 BST`: Completed the actor hyperspace shell-list cleanup
  cycle. Hyperspace cleanup now uses the shared source-shell predicate, so it
  clears both `EnemyLaser` and `Bomb` actors instead of enemy lasers only.
  Expanded regressions to cover enemy-shot plus bomb-shell cleanup while
  preserving player lasers, hostile actor families, lives, smart-bomb stock, and
  score. README, SPEC, and the actor architecture notes now describe
  hyperspace as source-shell cleanup. No legacy code, tests, or scaffolding were
  safe to remove in this slice because clean smoke/fidelity/oracle evidence
  still depends on clean runtime boundaries outside the actor path. Validation
  passed with `cargo fmt --check`, focused hyperspace tests,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo test actor_live --all-targets --features legacy-tools`,
  `cargo test actor_smoke --all-targets --features legacy-tools`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. The full unfiltered
  `legacy-tools` suite still has the previously isolated clean-game MAME
  window/post-game audio failures outside this slice. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780638390591659`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780638648247009`.
- `2026-06-05 06:59 BST`: Completed the actor bomber bomb shell-limit cycle.
  Driver command application now tracks the red-label 10-slot bomber bomb shell
  list alongside the shared 20-slot source shell list, so scripted/custom
  driver bomb spawns cannot exceed `BMBOUT` capacity while enemy-laser spawns
  can still use remaining shared shell slots. Added a regression that fills the
  bomb list, proves a blocked bomb spawn does not block an enemy shot, then
  destroys one bomb and refills the bomb slot in the same command batch.
  README, SPEC, and the actor architecture notes now document both source shell
  caps. No legacy code, tests, or scaffolding were safe to remove in this slice
  because clean smoke/fidelity/oracle evidence still depends on clean runtime
  boundaries outside the actor path. Validation passed with `cargo fmt
  --check`, focused bomb/shell/fraction tests,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo test actor_live --all-targets --features legacy-tools`,
  `cargo test actor_smoke --all-targets --features legacy-tools`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. The full unfiltered
  `legacy-tools` suite still has the previously isolated clean-game MAME
  window/post-game audio failures outside this slice. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780638714321579`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780639193179469`.
- `2026-06-05 07:08 BST`: Completed the actor bomber `GETSHL` bounds cycle.
  Source-backed bomber bombs now honor the red-label shell-allocation X bound:
  source bomb-shell spawns at X `0x98` or beyond are suppressed. The guard runs
  both when source bomber actors decide whether to emit a bomb spawn command and
  when custom/scripted driver commands request source-backed bomb actors, while
  non-source scripted bomb actors remain available. Added focused regressions
  for actor-emitted and direct command paths, and updated README, SPEC, and the
  actor architecture notes for the bound. No legacy code, tests, or scaffolding
  were safe to remove in this slice because clean smoke/fidelity/oracle
  evidence still depends on clean runtime boundaries outside the actor path.
  Validation passed with `cargo fmt --check`, focused GETSHL/fraction/cap tests,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo test actor_live --all-targets --features legacy-tools`,
  `cargo test actor_smoke --all-targets --features legacy-tools`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. The full unfiltered
  `legacy-tools` suite still has the previously isolated clean-game MAME
  window/post-game audio failures outside this slice. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780639336869439`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780639677266069`.
- `2026-06-05 07:13 BST`: Completed the actor source shell placement bounds
  cycle. Driver command handling now rejects source shell spawns outside the
  red-label `GETSHL` placement bounds: X `0x98` or beyond, and Y `0x2A` or
  below. Enemy-shot commands and source-backed bomb-shell commands share the
  bound, while non-source scripted bomb actors remain available for custom
  drivers. Expanded regressions for source bomber emission, direct source-backed
  bomb commands, non-source bomb commands, and enemy-shot command filtering.
  README, SPEC, and the actor architecture notes now document the X/Y shell
  placement behavior. No legacy code, tests, or scaffolding were safe to remove
  in this slice because clean smoke/fidelity/oracle evidence still depends on
  clean runtime boundaries outside the actor path. Validation passed with
  `cargo fmt --check`, focused placement/fraction/cap tests,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo test actor_live --all-targets --features legacy-tools`,
  `cargo test actor_smoke --all-targets --features legacy-tools`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. The full unfiltered
  `legacy-tools` suite still has the previously isolated clean-game MAME
  window/post-game audio failures outside this slice. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780639773324399`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780640013914859`.
- `2026-06-05 07:18 BST`: Completed the actor bomb lifetime metadata cycle.
  Source-backed `Bomb` actors now preserve nonzero
  `source_enemy_projectile.lifetime_ticks` supplied by scripted/custom driver
  spawns, using the behavior-profile `bomb_lifetime_steps` only when spawn
  metadata carries zero. Added a regression proving scripted bomb-shell
  lifetime ticks decrement on the source shell-scan cadence from the supplied
  value rather than the fallback behavior lifetime. README, SPEC, and the actor
  architecture notes now document scripted source bomb lifetime ownership. No
  legacy code, tests, or scaffolding were safe to remove in this slice because
  clean smoke/fidelity/oracle evidence still depends on clean runtime
  boundaries outside the actor path. Validation passed with `cargo fmt
  --check`, focused lifetime/fraction/bounds/cap tests,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo test actor_live --all-targets --features legacy-tools`,
  `cargo test actor_smoke --all-targets --features legacy-tools`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. The full unfiltered
  `legacy-tools` suite still has the previously isolated clean-game MAME
  window/post-game audio failures outside this slice. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780640100003009`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780640315297079`.
- `2026-06-05 07:32 BST`: Completed the actor scripted enemy-shot metadata
  cycle. `SpawnRequest::EnemyLaser` now carries optional source projectile
  metadata, so custom/scripted drivers can define exact source fractions,
  fixed-point velocities, and shell lifetime ticks for enemy shots while
  existing lander, swarmer, and baiter actor shots continue to emit ordinary
  enemy-shot commands with `source: None`. Source-backed scripted enemy lasers
  preserve nonzero lifetime ticks and decrement them on the source shell-scan
  cadence. Removed the stale no-source enemy-laser helper after the source-aware
  spawn path became the single driver boundary. README, SPEC, and the actor
  architecture notes now document scripted enemy-shot metadata. No legacy code,
  tests, or scaffolding were safe to remove in this slice because clean
  smoke/fidelity/oracle evidence still depends on clean runtime boundaries
  outside the actor path. Validation passed with `cargo fmt --check`, focused
  scripted enemy-shot/bounds/lifetime and lander/swarmer/baiter shot tests,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo test actor_live --all-targets --features legacy-tools`,
  `cargo test actor_smoke --all-targets --features legacy-tools`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. The full unfiltered
  `legacy-tools` suite still has the previously isolated clean-game MAME
  window/post-game audio failures outside this slice. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780640384287429`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780641115599679`.
- `2026-06-05 07:40 BST`: Completed the actor source-backed AI shot metadata
  cycle. Source-backed lander, swarmer, and baiter actors now carry source
  projectile metadata when their shot timers emit `EnemyLaser` spawn commands:
  the command preserves the firing actor's current source fractions, derives
  fixed-point shell velocities from the emitted shot velocity, and carries
  nonzero shell lifetime ticks through the source-aware spawn boundary. The
  non-source fallback firing paths still emit ordinary `source: None` enemy
  shots for custom/simple behavior. Expanded regressions for source
  lander/swarmer/baiter shot commands to assert the spawn payload metadata, and
  kept the scripted enemy-shot metadata regression green. README, SPEC, and the
  actor architecture notes now document source-backed AI shot metadata. No
  legacy code, tests, or scaffolding were safe to remove in this slice because
  clean smoke/fidelity/oracle evidence still depends on clean runtime
  boundaries outside the actor path. Validation passed with `cargo fmt
  --check`, focused source lander/swarmer/baiter/scripted enemy-shot tests,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo test actor_live --all-targets --features legacy-tools`,
  `cargo test actor_smoke --all-targets --features legacy-tools`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. The full unfiltered
  `legacy-tools` suite was not rerun in this cycle; the previously isolated
  clean-game MAME window/post-game audio failures remain outside this slice.
  Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780641377152289`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780641802044359`.
- `2026-06-05 07:55 BST`: Completed the actor source bomber motion metadata
  cycle. Source-backed bomber actors now use actor-owned source motion seeds to
  update picture-frame metadata, damp/randomize Y velocity, correct
  cruise-altitude motion while offscreen, and apply player-relative onscreen
  Y-velocity adjustment before advancing fixed-point position/fractions.
  Bomber bomb spawn commands now inherit the bomber's updated source fractions
  after that motion step instead of the earlier compact static-Y loop state.
  Added a focused source bomber motion regression and updated second-wave plus
  bomb-fraction regressions for the new source-shaped state. README, SPEC, and
  the actor architecture notes now document seeded source bomber Y-motion
  metadata. No legacy code, tests, or scaffolding were safe to remove in this
  slice because clean smoke/fidelity/oracle evidence still depends on clean
  runtime boundaries outside the actor path. Validation passed with `cargo fmt
  --check`, focused source bomber tests,
  `cargo test second_source_wave_spawns_bomber_and_pod_actor_families --lib`,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo test actor_live --all-targets --features legacy-tools`,
  `cargo test actor_smoke --all-targets --features legacy-tools`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. The full unfiltered
  `legacy-tools` suite was not rerun in this cycle; the previously isolated
  clean-game MAME window/post-game audio failures remain outside this slice.
  Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780641968857339`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780642500339539`.
- `2026-06-05 08:12 BST`: Completed the actor driver-owned source RNG prompt
  cycle. `StepPrompt` and `StepReport` now carry the driver-owned source RNG
  snapshot for playing steps, and the default hyperspace behavior derives its
  source seed from that same prompt boundary. Source-backed bomber motion now
  consumes the driver-provided source RNG snapshot for picture and Y-motion
  updates instead of falling back to a local step/id seed when the driver has a
  source prompt available. Added a focused report snapshot regression and kept
  the hyperspace/source bomber behavior tests green. README, SPEC, and the
  actor architecture notes now document the source RNG prompt boundary. No
  legacy code, tests, or scaffolding were safe to remove in this slice because
  clean smoke/fidelity/oracle evidence still depends on clean runtime
  boundaries outside the actor path. Validation passed with `cargo fmt
  --check`, focused source RNG/hyperspace/source bomber tests,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo test actor_live --all-targets --features legacy-tools`,
  `cargo test actor_smoke --all-targets --features legacy-tools`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. The full unfiltered
  `legacy-tools` suite was not rerun in this cycle; the previously isolated
  clean-game MAME window/post-game audio failures remain outside this slice.
  Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780642856268849`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780643571704519`.
- `2026-06-05 08:21 BST`: Completed the actor source baiter RNG prompt cycle.
  Source-backed baiter picture-wrap retargeting now consumes the
  driver-provided source RNG snapshot carried in `StepPrompt` for the source
  `SEED > UFOSK` gate before applying `baiter_seek_probability`; the local
  step/id seed remains only as the fallback for hand-built prompts without
  source RNG. Added a focused driver regression proving low prompt `SEED` holds
  baiter velocity while high prompt `SEED` retargets movement through the actor
  boundary. README, SPEC, and the actor architecture notes now document the
  baiter source RNG prompt boundary. No legacy code, tests, or scaffolding were
  safe to remove in this slice because clean smoke/fidelity/oracle evidence
  still depends on clean runtime boundaries outside the actor path. Validation
  passed with `cargo fmt --check`, `cargo test source_baiter --lib`,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo test actor_live --all-targets --features legacy-tools`,
  `cargo test actor_smoke --all-targets --features legacy-tools`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. The full unfiltered
  `legacy-tools` suite was not rerun in this cycle; the previously isolated
  clean-game MAME window/post-game audio failures remain outside this slice.
  Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780643739247869`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780644108824289`.
- `2026-06-05 08:28 BST`: Completed the actor source baiter
  player-velocity retargeting cycle. `StepPrompt` now exposes player velocity
  from the published player snapshot, and source-backed baiter `UFONV`
  retargeting adds player X/Y velocity into the source-shaped seek velocity
  after the source RNG/`UFOSK` gate. Added a focused driver regression proving
  the player-velocity contribution changes baiter fixed-point velocity,
  fractions, and movement through the actor boundary. README, SPEC, and the
  actor architecture notes now document the player-velocity retarget behavior.
  No legacy code, tests, or scaffolding were safe to remove in this slice
  because clean smoke/fidelity/oracle evidence still depends on clean runtime
  boundaries outside the actor path. Validation passed with `cargo fmt
  --check`, `cargo test source_baiter --lib`,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo test actor_live --all-targets --features legacy-tools`,
  `cargo test actor_smoke --all-targets --features legacy-tools`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. The full unfiltered
  `legacy-tools` suite was not rerun in this cycle; the previously isolated
  clean-game MAME window/post-game audio failures remain outside this slice.
  Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780644177690399`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780644545006409`.
- `2026-06-05 08:37 BST`: Completed the actor source pod Y-bound wrapping
  cycle. Source-backed pod Y motion now uses a source active-object Y step,
  wrapping through `SOURCE_PLAYFIELD_Y_MIN..=SOURCE_PLAYFIELD_Y_MAX` instead
  of drifting outside the red-label vertical range while preserving fixed-point
  fractions. Added `SOURCE_PLAYFIELD_Y_MAX` to the actor source constants and a
  focused regression covering both top and bottom source pod wrap cases.
  README, SPEC, and the actor architecture notes now document the pod wrap
  behavior. No legacy code, tests, or scaffolding were safe to remove in this
  slice because clean smoke/fidelity/oracle evidence still depends on clean
  runtime boundaries outside the actor path. Validation passed with `cargo fmt
  --check`, `cargo test source_pod --lib`,
  `cargo test second_source_wave_spawns_bomber_and_pod_actor_families --lib`,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo test actor_live --all-targets --features legacy-tools`,
  `cargo test actor_smoke --all-targets --features legacy-tools`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. The full unfiltered
  `legacy-tools` suite was not rerun in this cycle; the previously isolated
  clean-game MAME window/post-game audio failures remain outside this slice.
  Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780644711982029`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780645082637979`.
- `2026-06-05 08:45 BST`: Completed the actor source hostile Y-bound wrapping
  cycle. Source-backed lander, bomber, swarmer, and baiter Y motion now uses
  the source active-object Y step, matching the source pod wrap behavior
  through `SOURCE_PLAYFIELD_Y_MIN..=SOURCE_PLAYFIELD_Y_MAX` while preserving
  fixed-point fractions. Updated the source bomber expected-motion helper to
  model the same red-label vertical bounds. Added a focused regression covering
  source lander, swarmer, and baiter top/bottom Y wrapping; source pod and
  bomber filters stayed green. README, SPEC, and the actor architecture notes
  now document source-hostile active-object Y wrapping. No legacy code, tests,
  or scaffolding were safe to remove in this slice because clean
  smoke/fidelity/oracle evidence still depends on clean runtime boundaries
  outside the actor path. Validation passed with `cargo fmt --check`, focused
  source hostile/pod/bomber tests,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo test actor_live --all-targets --features legacy-tools`,
  `cargo test actor_smoke --all-targets --features legacy-tools`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. The full unfiltered
  `legacy-tools` suite was not rerun in this cycle; the previously isolated
  clean-game MAME window/post-game audio failures remain outside this slice.
  Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780645175300049`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780645568844929`.
- `2026-06-05 08:57 BST`: Completed the actor source-mutant conversion
  metadata cycle. Source-backed lander/human conversions now spawn mutant
  actors with source-shaped mutant fractions, wave-derived shot timer,
  driver-owned hop RNG, render-correction, and deferred-shot metadata. Mutant
  snapshots preserve that metadata and `ActorStateBridge` maps it into the
  clean `SourceMutantSnapshot` contract, giving the next source mutant AI slice
  a source-backed state boundary instead of a generic mutant. Added a focused
  regression covering the conversion spawn command, the settled mutant actor
  snapshot, and the clean bridge snapshot. README, SPEC, and the actor
  architecture notes now document the conversion metadata boundary. No legacy
  code, tests, or scaffolding were safe to remove in this slice because clean
  smoke/fidelity/oracle evidence still depends on clean runtime boundaries
  outside the actor path. Validation passed with `cargo fmt --check`,
  `cargo test source_mutant --lib`, `cargo test completed_abduction --lib`,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo test actor_live --all-targets --features legacy-tools`,
  `cargo test actor_smoke --all-targets --features legacy-tools`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. The full unfiltered
  `legacy-tools` suite was not rerun in this cycle; the previously isolated
  clean-game MAME window/post-game audio failures remain outside this slice.
  Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780645950497009`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780646630879589`.
- `2026-06-05 09:24 BST`: Completed the actor source-mutant behavior cycle.
  Source-backed mutant actors now read the red-label wave-table mutant X/Y
  velocity, random-hop, and shot-timer fields; advance actor-owned fixed-point
  fractions and hop RNG instead of generic chase/drift motion; and emit
  source-shaped hostile projectile metadata plus the red-label `0xF6`
  mutant-shot cue when their shot timer expires. Added focused regressions for
  wave-table mutant values, source mutant movement/hop state, source mutant
  shot command metadata, and sound cue mapping. README, SPEC, and the actor
  architecture notes now document the behavior. Validation passed with
  `cargo fmt --check`, `cargo test source_mutant --lib`,
  `cargo test actor_sound_cues --lib`,
  `cargo test default_wave_script_uses_source_wave_table_values --lib`,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo test actor_live --all-targets --features legacy-tools`,
  `cargo test actor_smoke --all-targets --features legacy-tools`,
  `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  touched-doc markdownlint, and `git diff --check`. The full unfiltered
  `legacy-tools` suite was not rerun in this cycle; the previously isolated
  clean-game MAME window/post-game audio failures remain outside this slice.
  Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780647196072959`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780648443923729`.
- `2026-06-05 16:19 BST`: Completed the actor survivor-bonus cadence cycle.
  `ActorGameDriver` now owns a source-shaped wave-clear survivor-bonus
  interstitial instead of spawning the next wave on the following actor step.
  The first survivor scores on the `WaveCleared` report, later survivors wait
  the source four-step astronaut sleep, each award uses `100 * min(wave, 5)`
  through the existing replay-stock scoring system, visible source `ASTP3`
  icons appear one at a time, and the next wave waits through the source `0x80`
  wave-advance sleep before `AdvanceWave` / `WaveStarted`. Updated tests now
  cover the icon cadence, score multiplier, replay-stock crossing, smart-bomb
  wave-clear scoring, and next-wave delay. README, SPEC, the actor architecture
  notes, and fidelity gaps now document the implemented cadence. No legacy code,
  tests, or scaffolding were safe to remove in this slice because clean
  smoke/fidelity/oracle evidence still depends on clean runtime boundaries
  outside the actor path. Validation passed with focused survivor cadence and
  multiplier tests, the actor-game/smoke/live/wgpu all-target `legacy-tools`
  filters, the actor smoke CLI commands (`--actor-smoke`,
  `--actor-attract-smoke`, `--actor-post-game-smoke`, and
  `--actor-wgpu-smoke`), the runtime all-target `legacy-tools` filter,
  `cargo check --all-targets --features legacy-tools`, clippy with warnings
  denied, `cargo fmt --check`, touched-doc markdownlint, and the diff
  whitespace check. The full unfiltered `legacy-tools` suite was not
  rerun in this cycle; the previously isolated clean-game MAME window/post-game
  audio failures remain outside this slice. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780671863699699`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780672799324769`.
- `2026-06-05 16:58 BST`: Completed the actor two-player death-switch cycle.
  `ActorGameDriver` now owns a bounded `0x60` player-switch sleep when a
  two-player death leaves the other player stocked. Player hazard deaths
  decrement the active player's stock first, publish `PlayerSwitchReport`
  through `StepReport` and `GameOverSnapshot`, suppress attract-script drawing
  during the switch interstitial, draw a simple actor status `PLAYER n` prompt,
  and start the next stocked player's turn after the countdown. Final deaths
  enter the normal game-over/high-score path only when no other player has
  lives. Focused tests now cover non-final death rotation, player-one
  final-life switch to player two, player-two final-life switch back to player
  one, and final game over when no other stock remains. README, SPEC, and the
  actor architecture notes now distinguish the implemented switch state from
  the remaining source `PLE02` glyph/media parity work. No legacy code, tests,
  or scaffolding were safe to remove in this slice because clean
  smoke/fidelity/oracle evidence still depends on clean runtime boundaries
  outside the actor path. Validation passed with `cargo test actor_two_player
  --lib --features legacy-tools`, `cargo test actor_game --all-targets
  --features legacy-tools`, `cargo fmt --check`, `cargo check --all-targets
  --features legacy-tools`, `cargo clippy --all-targets --features
  legacy-tools -- -D warnings`, `cargo test actor_live --all-targets
  --features legacy-tools`, `cargo test actor_smoke --all-targets --features
  legacy-tools`, `cargo test runtime --all-targets --features legacy-tools`,
  the actor smoke CLI commands (`--actor-smoke`, `--actor-attract-smoke`,
  `--actor-post-game-smoke`, and `--actor-wgpu-smoke`), touched-doc
  markdownlint, and `git diff --check`. The full unfiltered `legacy-tools`
  suite was not rerun in this cycle; the previously isolated clean-game MAME
  window/post-game audio failures remain outside this slice. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780674174951459`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780675123787219`.
- `2026-06-05 17:16 BST`: Completed the actor two-player source-prompt/start
  handoff cycle. `StepPrompt`/`StepReport` now carry `PlayerStartReport`
  alongside `PlayerSwitchReport`; accepted two-player starts and post-switch
  turns publish a source-length `138`-step actor player-start delay before
  playfield actors spawn. `ActorRenderSceneBridge` now projects the source
  `PLYR1` / `PLYR2` and `GO` message-glyph prompts from report state, so the
  credited two-player start shows `PLAYER ONE`, a player switch shows the
  source player label plus `GAME OVER`, and the next stocked player receives a
  source player-start prompt before `WaveStarted`. Actor source RNG, shell
  scan, input, and wave-clear detection stay paused during this interstitial.
  Focused tests now cover source glyph rendering, the start delay countdown,
  delayed `WaveStarted`, and both switch directions. README, SPEC, and the
  actor architecture notes now document the implemented report/render contract
  and leave only MAME media proof plus exact prompt pixel/timing parity as the
  remaining prompt boundary. No legacy code, tests, or scaffolding were safe to
  remove in this slice because clean smoke/fidelity/oracle evidence still
  depends on clean runtime boundaries outside the actor path. Validation passed
  with `cargo test actor_two_player --lib --features legacy-tools`,
  `cargo test actor_game --all-targets --features legacy-tools`,
  `cargo fmt --check`, `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  `cargo test actor_live --all-targets --features legacy-tools`,
  `cargo test actor_smoke --all-targets --features legacy-tools`,
  `cargo test runtime --all-targets --features legacy-tools`, the actor smoke
  CLI commands (`--actor-smoke`, `--actor-attract-smoke`,
  `--actor-post-game-smoke`, and `--actor-wgpu-smoke`), touched-doc
  markdownlint, and `git diff --check`. The full unfiltered `legacy-tools`
  suite was not rerun in this cycle; the previously isolated clean-game MAME
  window/post-game audio failures remain outside this slice. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780675275699089`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780676232273589`.
- `2026-06-05 17:34 BST`: Completed the actor one-player source-start delay
  cycle. Accepted one-player starts now publish the same source-length
  `138`-step `PlayerStartReport` as two-player starts and delay playfield actor
  spawning plus `WaveStarted` until the delay expires. One-player start reports
  keep the playfield world empty without rendering the two-player source
  `PLYR1` prompt, while credited two-player starts and post-switch turns keep
  the source player-label prompt path. Actor input, source RNG, shell scan, and
  wave-clear detection remain paused during the delay. The actor smoke script
  now waits until delayed playfield activation before injecting fire, thrust,
  reverse, smart bomb, hyperspace, and altitude inputs, preserving real
  playfield coverage in the executable smoke path. Focused tests cover the
  one-player delayed `WaveStarted` boundary, no-prompt render contract,
  two-player prompt preservation, and updated wave/script setup helpers. No
  legacy code, tests, or scaffolding were safe to remove in this slice because
  clean smoke/fidelity/oracle evidence still depends on clean runtime
  boundaries outside the actor path. Validation passed with `cargo test
  actor_one_player_start --lib --features legacy-tools`, `cargo test
  actor_two_player --lib --features legacy-tools`, `cargo test actor_game
  --all-targets --features legacy-tools`, `cargo test actor_smoke
  --all-targets --features legacy-tools`, `cargo test actor_live --all-targets
  --features legacy-tools`, `cargo test runtime --all-targets --features
  legacy-tools`, `cargo fmt --check`, `cargo check --all-targets --features
  legacy-tools`, `cargo clippy --all-targets --features legacy-tools --
  -D warnings`, the actor smoke CLI commands (`--actor-smoke`,
  `--actor-attract-smoke`, `--actor-post-game-smoke`, and
  `--actor-wgpu-smoke`), touched-doc markdownlint, and `git diff --check`.
  The full unfiltered `legacy-tools` suite was not rerun in this cycle; the
  previously isolated clean-game MAME window/post-game audio failures remain
  outside this slice. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780676514729719`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780677225766979`.
- `2026-06-05 17:45 BST`: Completed the actor start-audio cadence cycle.
  Accepted one-player and two-player start reports now stay silent, the actor
  start sound emits on the following actor step, and the source `0xEA`
  appearance/materialize command emits when the delayed player-start report
  begins the playfield and publishes `WaveStarted`. `SoundCue::PlayerAppear`
  now exposes the red-label sound-board command without changing the clean
  audio bridge contract for unmapped source commands, and focused tests cover
  the one-player, two-player, parsed-script, runtime-adapter, and delayed
  playfield-start audio boundaries. README, SPEC, actor architecture notes, and
  the baseline plan now document the start-audio cadence. No legacy code,
  tests, or scaffolding were safe to remove in this slice because clean
  smoke/fidelity/oracle evidence still depends on clean runtime boundaries
  outside the actor path. Validation passed with `cargo test
  actor_one_player_start --lib --features legacy-tools`, `cargo test
  actor_two_player_start --lib --features legacy-tools`, `cargo test
  actor_runtime_adapter_bundles_report_events_audio_and_scene --lib --features
  legacy-tools`, `cargo fmt && cargo test actor_game --all-targets --features
  legacy-tools`, `cargo test actor_smoke --all-targets --features
  legacy-tools`, `cargo test actor_live --all-targets --features
  legacy-tools`, `cargo test runtime --all-targets --features legacy-tools`,
  `cargo fmt --check`, `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`, the
  actor smoke CLI commands (`--actor-smoke`, `--actor-attract-smoke`,
  `--actor-post-game-smoke`, and `--actor-wgpu-smoke`), touched-doc
  markdownlint, and `git diff --check`. The full unfiltered `legacy-tools`
  suite was not rerun in this cycle; the previously isolated clean-game MAME
  window/post-game audio failures remain outside this slice. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780677389299959`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780677933880219`.
- `2026-06-05 18:18 BST`: Completed the actor source-reserve activation
  cycle. Source-backed actor wave profiles now retain post-active-batch enemy
  reserve counts and expose those counts through `StepReport` plus the clean
  `WorldSnapshot`. Parsed wave scripts preserve source-expanded reserves and
  can set custom reserve counts with `reserve` / `enemy_reserve`. The driver
  arms reserve activation only after the active batch has published a report,
  preventing player-start and wave-install steps from refilling immediately.
  When source-counted hostiles are gone, reserve landers restore first from
  source RNG placement/shot/velocity state before wave clear can start; after
  landers are exhausted, bomber and pod reserve batches restore from
  source-shaped placement/fraction metadata. Smart-bomb tests now cover the
  destruction report and following reserve-restoration report rather than
  assuming the first active batch clears the wave. README, SPEC, and the actor
  architecture notes now document reserve counts, state bridging, restore
  ordering, and the parser directive. No legacy code, tests, or scaffolding
  were safe to remove in this slice because clean smoke/fidelity/oracle
  evidence still depends on clean runtime boundaries outside the actor path.
  Validation passed with `cargo fmt --check`, `cargo check --all-targets
  --features legacy-tools`, `cargo test actor_source --lib --features
  legacy-tools`, `cargo test
  second_source_wave_spawns_bomber_and_pod_actor_families --lib --features
  legacy-tools`, `cargo test actor_game --all-targets --features
  legacy-tools`, `cargo clippy --all-targets --features legacy-tools --
  -D warnings`, `cargo test actor_smoke --all-targets --features
  legacy-tools`, `cargo test actor_live --all-targets --features
  legacy-tools`, `cargo test runtime --all-targets --features legacy-tools`,
  `cargo test actor_wgpu --all-targets --features legacy-tools`, the actor
  smoke CLI commands (`--actor-smoke`, `--actor-attract-smoke`,
  `--actor-post-game-smoke`, and `--actor-wgpu-smoke`), touched-doc
  markdownlint, and `git diff --check`. The full unfiltered `legacy-tools`
  suite was not rerun in this cycle; the previously isolated clean-game MAME
  window/post-game audio failures remain outside this slice. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780678969677879`.
  Slack completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780679884866769`.
- `2026-06-05 22:43 BST`: Completed the organic smartmix terrain-blow cleanup
  cycle. Clean terrain-blow entry now clears remaining human snapshots,
  target-list cursors, astronaut cursors, and astronaut sleep metadata, and
  post-game playfield sync no longer republishes post-game human tracks while
  a destroyed-planet terrain-blow state is active. The focused organic
  smartmix guard now asserts that once terrain blow is present, clean human
  snapshots stay empty across the terminal-death, game-over handoff, and
  sampled `0xEE` tail rows. A throwaway debug-only clean capture at
  `target/reference-media/clean/organic-terrain-blow-smartmix-current-debug.debug.tsv`
  reproduced score `50`, no humans, active terrain blow, and the `0xEE`
  cadence at input frames `5990`, `5994`, `5998`, `6002`, and `6006`
  (`state_frame` `5991`, `5995`, `5999`, `6003`, and `6007`). The protected
  bounded report at
  `target/reference-media/organic-terrain-blow-smartmix-check/report.json`
  remains stale and still requires regeneration plus all-axis owner review; no
  protected media was replaced in this slice. No legacy code, tests, or
  scaffolding were safe to remove because the clean runtime and reference
  evidence tooling still own this proof boundary. Validation passed with
  `cargo fmt --check`, `cargo test
  clean_reference_terrain_blow_steer_erases_humans_and_target_cursors --lib
  --features legacy-tools`, `cargo test
  clean_world_starts_source_terrain_blow_and_projects_terex --lib --features
  legacy-tools`, `cargo test
  clean_game_organic_smartmix_terminal_death_and_terrain_blow_match_mame_window
  --lib --features legacy-tools`, `cargo test terrain_blow --lib --features
  legacy-tools`, `cargo check --all-targets --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  `markdownlint PLAN.md docs/fidelity/release-closure-audit.md`, and
  `git diff --check`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780695238307599`.
- `2026-06-05 22:53 BST`: Completed the organic smartmix visible-terrain-blow
  continuation cycle. The clean runtime now treats the earlier
  destroyed-planet state as terrain erased, then resets the visible source
  `TERBLO` sequence at the organic post-game frame `1044` where MAME emits the
  first sampled `0xEE`. Post-game sync preserves terrain explosions during
  this branch instead of overwriting them with static residual post-game
  objects, and the organic smartmix regression now checks terrain-blow elapsed
  time plus `TEREX` births at the sampled state rows. Regenerating the ignored
  clean candidate with `make reference-clean-capture` for input frames
  `5960-6140` now shows `elapsed16` and `TEREX` progression by state frame
  `6007`, while preserving score `50`, no humans, and the `0xEE` cadence. The
  regenerated ignored report at
  `target/reference-media/organic-terrain-blow-smartmix-check/report.json`
  improves from stale static/no-audio evidence but still fails all-axis
  acceptance with visual RMS `90.26`, visual MAE `42.20`, and audio envelope
  correlation `0.137`; no protected media was committed or accepted. No legacy
  code, tests, or scaffolding were safe to remove because the remaining gap is
  still a live clean-vs-MAME organic media proof boundary. Evidence runs
  included `make reference-clean-capture ...` and `make reference-media-check
  ...`, with the media check expectedly failing on the improved all-axis
  metrics above. Validation passed with `cargo fmt --check`, `cargo test
  clean_game_organic_smartmix_terminal_death_and_terrain_blow_match_mame_window
  --lib --features legacy-tools`, `cargo test
  clean_world_advances_source_terrain_blow_to_completion_sound --lib
  --features legacy-tools`, terrain-blow regression tests, Cargo
  check/clippy, touched-doc `markdownlint`, and `git diff --check`. Slack
  start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780695923761149`.
- `2026-06-05 23:03 BST`: Completed the organic smartmix terminal residual
  object projection cycle. The target4 terminal post-game branch now uses a
  bounded source-object track table from the MAME rows at frames `5960`,
  `5970`, `5980`, and `5990`, projecting the six `LNDP1`-`LNDP3` lander
  objects and nine `SCZP1` mutant objects that MAME keeps active before the
  organic `TERBLO` boundary. The focused organic regression now checks the
  six-lander/nine-mutant mix and sampled mutant positions at state frames
  `5981` and `5991`, alongside the existing score, no-human, terrain-blow,
  `TEREX`, and `0xEE` cadence checks. Regenerating the ignored clean candidate
  confirms the residual object mix is now present and animated before
  `TERBLO`. The ignored all-axis report remains failed with visual RMS
  `90.73`, visual MAE `42.59`, and audio envelope correlation `0.137`, so the
  remaining work is still pixel/audio calibration rather than residual object
  absence. No protected media was committed or accepted, and no legacy code,
  tests, or scaffolding were safe to remove because this proof boundary still
  depends on clean-vs-MAME reference tooling. Evidence runs included
  `make reference-clean-capture ...` and `make reference-media-check ...`,
  with the media check expectedly failing on the current all-axis metrics.
  Validation passed with the focused organic regression, `cargo fmt --check`,
  terrain-blow regression tests, Cargo check/clippy, touched-doc
  `markdownlint`, and `git diff --check`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780696576650949`.
- `2026-06-05 23:18 BST`: Completed the organic smartmix delayed
  terrain-erasure cycle. The target4 terminal branch now arms source
  terrain-blow state at destroyed-planet time while preserving the visible
  `BGOUT` playfield: it clears human/cursor state, keeps terrain/scanner erase
  words pending, and defers `TEREX` births until the post-game frame `1044`
  reset where MAME starts the visible `TERBLO` sequence. The scene bridge now
  renders terrain from the actual `terrain_erased()` state instead of hiding it
  merely because terrain blow is armed. The focused organic regression now
  asserts terrain is still rendered with zero terrain explosions at state
  frames `4927` and `4947`, then erased with `TEREX` counts `1` and `6` at
  state frames `5991` and `6007`. Regenerated ignored media confirms the first
  compared candidate frame has terrain again, but the all-axis report still
  fails with visual RMS `90.80`, visual MAE `42.68`,
  `normalized_diff_rms=1.726`, audio correlation `-0.005`, and audio envelope
  correlation `0.137`. The remaining organic smartmix gap is now the broader
  residual-object/HUD/scanner/audio mismatch, not early terrain erasure. No
  protected media was committed or accepted, and no legacy code, tests, or
  scaffolding were safe to remove because this proof boundary still depends on
  clean-vs-MAME reference tooling. Evidence runs included the
  `reference-clean-capture` and `reference-media-check` targets, with the media
  check expectedly failing on the current all-axis metrics. Validation passed
  with `cargo fmt --check`, the focused organic regression, terrain-blow
  regression tests, Cargo check/clippy, touched-doc `markdownlint`, and
  `git diff --check`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780697229173389`.
- `2026-06-05 23:22 BST`: Completed the organic smartmix residual-object
  sample alignment cycle. The target4 terminal post-game object projection now
  samples the source residual track table one clean post-game tick earlier, so
  the first compared clean frame at state frame `5960` already carries the
  six-lander/nine-mutant residual formation instead of an empty playfield.
  The focused organic regression now asserts the residual mix and first-mutant
  positions at state frames `5960`, `5981`, `5990`, and `5991`; `5990` reaches
  the terminal `41,49` mutant position before terrain flips at `5991`.
  Regenerated ignored media confirms the first candidate comparison frame now
  has visible residual objects plus terrain. The all-axis report still fails
  with visual RMS `90.85`, visual MAE `42.72`, `normalized_diff_rms=1.726`,
  audio correlation `-0.005`, and audio envelope correlation `0.137`, so the
  remaining organic smartmix gap is residual-object placement/HUD/scanner/audio
  parity rather than object absence. No protected media was committed or
  accepted, and no legacy code, tests, or scaffolding were safe to remove
  because this proof boundary still depends on clean-vs-MAME reference
  tooling. Evidence runs included the `reference-clean-capture` and
  `reference-media-check` targets, with the media check expectedly failing on
  the current all-axis metrics. Validation passed with `cargo fmt --check`, the
  focused organic regression, terrain-blow regression tests, Cargo
  check/clippy, touched-doc `markdownlint`, and `git diff --check`. Slack
  start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780697836698829`.
- `2026-06-05 23:30 BST`: Completed the organic smartmix live-scanner
  mini-terrain cycle. The live scanner renderer now honors
  `ScannerRadarSnapshot::terrain_enabled`: before terrain erase it draws the
  source `MTERR` mini-terrain slice for the current scanner `scan_left`, using
  source scanner terrain tint and pixel size before object/player blips. The
  existing scanner sprite regression now derives the expected terrain pixel
  count from `MTERR` records and asserts all emitted scanner terrain pixels use
  the source HUD layer, tint, and size. Regenerated ignored media confirms the
  first candidate comparison frame now shows the missing orange scanner
  terrain line alongside the residual objects and live playfield terrain. The
  all-axis report remains red because the scanner terrain and residual objects
  are still spatially misaligned against MAME: visual RMS `90.88`, visual MAE
  `42.76`, scanner RMS `99.90`, `normalized_diff_rms=1.726`, audio
  correlation `-0.005`, and audio envelope correlation `0.137`. No protected
  media was committed or accepted, and no legacy code, tests, or scaffolding
  were safe to remove because this proof boundary still depends on
  clean-vs-MAME reference tooling. Evidence runs included the
  `reference-clean-capture` and `reference-media-check` targets, with the media
  check expectedly failing on the current all-axis metrics. Validation passed
  with `cargo fmt --check`, the focused scanner regression, the focused
  organic regression, terrain-blow regression tests, Cargo check/clippy,
  touched-doc `markdownlint`, and `git diff --check`. Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780698365603889`.
- `2026-06-06 01:15 BST`: Completed the organic smartmix terminal
  sound-cadence cycle. The target4 terminal post-game sound sequence now
  carries the remaining MAME-observed rows from state frames `6012-6128`:
  the extended `0xEE` terrain-blow tail, `0xEB` laser fire at `6088`,
  `0xE8` cannon rows at `6104` and `6114`, `0xE9` thrust at `6124`, and
  `0xF6` mutant shot at `6128`. The focused organic regression now samples
  those rows through the post-game sound profile and asserts the mapped clean
  `SoundEvent` stream. Regenerated ignored media for
  `organic-terrain-blow-smartmix` confirms the clean sound event trace now
  maps to the MAME command cadence over the bounded window; the report-only
  comparator now passes audio checks with envelope correlation `0.4431`
  (`0.0171` in the previous safe baseline) while visual RMS `89.06` and MAE
  `40.78` remain above threshold. This leaves the organic smartmix visual
  proof boundary open rather than accepting the all-axis clip. No protected
  media was committed or accepted, and no legacy code, tests, or scaffolding
  were safe to remove because the remaining work still depends on the
  clean-vs-MAME reference tooling. Evidence runs included
  `make reference-clean-capture ...` and `make reference-media-check ...`,
  with the media check expectedly failing on visual metrics only. Validation
  passed with `cargo fmt --check`, `cargo test
  clean_game_organic_smartmix_terminal_death_and_terrain_blow_match_mame_window
  --lib --features legacy-tools`, and `cargo check --features legacy-tools`.
  Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780704479140879`.
- `2026-06-06 01:27 BST`: Completed the organic smartmix terminal shell-row
  projection cycle. The target4 terminal post-game branch now replays the
  MAME-observed `BMBP1` shell rows as clean enemy-projectile snapshots, using
  bounded source-sample interpolation so rows appear at state frames `5991`,
  `6007`, `6075`, and `6128` and are absent again at `6088`. The focused
  organic regression now samples those terminal shell positions alongside the
  existing score, terrain-blow, residual-object, and sound-cadence assertions.
  Regenerated ignored clean media for `organic-terrain-blow-smartmix` confirms
  those rows in the clean debug TSV. The report-only all-axis comparator still
  fails on broad visual metrics, with visual RMS `89.06` and MAE `40.78`;
  audio remains green with envelope correlation `0.4431`, so this remains a
  targeted visual-fidelity slice rather than an accepted organic all-axis
  proof. No protected media was committed or accepted, and no legacy code,
  tests, or scaffolding were safe to remove because the remaining mismatch
  still depends on clean-vs-MAME reference tooling. Evidence runs included
  `make reference-clean-capture ...` and `make reference-media-check ...`,
  with the media check expectedly failing on visual metrics only. Validation
  passed with `cargo fmt --check`, `cargo test
  clean_game_organic_smartmix_terminal_death_and_terrain_blow_match_mame_window
  --lib --features legacy-tools`, `cargo check --features legacy-tools`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  `markdownlint PLAN.md`, and `git diff --check`.
  Slack start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780705067501169`.
- `2026-06-06 01:42 BST`: Completed the organic smartmix late residual-mutant
  projection cycle. The target4 terminal post-game branch now carries the
  MAME-observed late `SCZP1` / `F8CE` object-table rows after state frame
  `6007`, using bounded source-sampled tracks in object-frame coordinates
  `1061-1193` with extra samples around visible wrap points. Clean debug now
  preserves thirteen residual mutant rows at state frames `6007`, `6075`, and
  `6128` instead of dropping the terminal objects once the early visible table
  ended. The focused organic regression now asserts those late counts and first
  row positions alongside the existing terminal shell, terrain-blow, score, and
  sound-cadence checks. Regenerated ignored clean media/debug for
  `organic-terrain-blow-smartmix` confirms the new late rows. The all-axis
  report remains unaccepted and still fails on the same broad visual boundary:
  visual RMS moved from `89.06` to `89.56` and MAE from `40.78` to `41.17`,
  while audio remains green with envelope correlation `0.4431`. This means the
  remaining organic smartmix blocker is still the unresolved flash/timeline and
  source-footprint mismatch, not absence of late residual rows. No protected
  media was committed or accepted, and no legacy code, tests, or scaffolding
  were safe to remove because the clean-vs-MAME reference tooling is still
  needed for the open organic proof boundary. Evidence runs included
  `make reference-clean-capture ...` and `make reference-media-check ...`,
  with the media check expectedly failing on visual metrics. Validation passed
  with `cargo fmt --check`, `cargo test
  clean_game_organic_smartmix_terminal_death_and_terrain_blow_match_mame_window
  --lib --features legacy-tools`, `cargo check --features legacy-tools`, and
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`. Slack
  start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780705822870009`.
- `2026-06-06 01:55 BST`: Completed the organic smartmix late residual
  visibility-gating cycle. The target4 terminal post-game branch now treats
  late `SCZP1` / `F8CE` rows as render-visible only when the MAME debug object
  slot has nonzero display coordinates, replacing the previous bounded
  memory-position interpolation with the 168 visible display rows observed
  across object frames `1127-1193`. The focused organic regression now asserts
  no rendered residual mutants at state frame `6007`, then the MAME-visible
  counts and first display positions at state frames `6073`, `6075`, `6088`,
  `6098`, `6114`, `6124`, and `6128`. Regenerated ignored clean media/debug
  for `organic-terrain-blow-smartmix` confirms the new display-gated rows. The
  report-only all-axis comparator remains unaccepted but improves the previous
  overdraw regression: visual RMS moved from `89.56` to `89.11` and MAE from
  `41.17` to `40.82`, while audio remains green with envelope correlation
  `0.4431`. No protected media was committed or accepted, and no legacy code,
  tests, or scaffolding were safe to remove because the remaining visual
  mismatch still depends on clean-vs-MAME reference tooling. Evidence runs
  included `make reference-clean-capture ...` and `make reference-media-check
  ...`, with the media check expectedly failing on visual metrics. Validation
  passed with `cargo fmt`, `cargo test
  clean_game_organic_smartmix_terminal_death_and_terrain_blow_match_mame_window
  --lib --features legacy-tools`, `cargo check --features legacy-tools`, and
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`. Slack
  start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1780706710052279`.
