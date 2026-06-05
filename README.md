# Defender

[![Quality Gate Status](https://sonarcloud.io/api/project_badges/measure?project=stephenlclarke_defender&metric=alert_status)](https://sonarcloud.io/summary/new_code?id=stephenlclarke_defender)
[![Bugs](https://sonarcloud.io/api/project_badges/measure?project=stephenlclarke_defender&metric=bugs)](https://sonarcloud.io/summary/new_code?id=stephenlclarke_defender)
[![Code Smells](https://sonarcloud.io/api/project_badges/measure?project=stephenlclarke_defender&metric=code_smells)](https://sonarcloud.io/summary/new_code?id=stephenlclarke_defender)
[![Coverage](https://sonarcloud.io/api/project_badges/measure?project=stephenlclarke_defender&metric=coverage)](https://sonarcloud.io/summary/new_code?id=stephenlclarke_defender)
[![Duplicated Lines (%)](https://sonarcloud.io/api/project_badges/measure?project=stephenlclarke_defender&metric=duplicated_lines_density)](https://sonarcloud.io/summary/new_code?id=stephenlclarke_defender)
[![Lines of Code](https://sonarcloud.io/api/project_badges/measure?project=stephenlclarke_defender&metric=ncloc)](https://sonarcloud.io/summary/new_code?id=stephenlclarke_defender)
[![Reliability Rating](https://sonarcloud.io/api/project_badges/measure?project=stephenlclarke_defender&metric=reliability_rating)](https://sonarcloud.io/summary/new_code?id=stephenlclarke_defender)
[![Security Rating](https://sonarcloud.io/api/project_badges/measure?project=stephenlclarke_defender&metric=security_rating)](https://sonarcloud.io/summary/new_code?id=stephenlclarke_defender)
[![Technical Debt](https://sonarcloud.io/api/project_badges/measure?project=stephenlclarke_defender&metric=sqale_index)](https://sonarcloud.io/summary/new_code?id=stephenlclarke_defender)
[![Maintainability Rating](https://sonarcloud.io/api/project_badges/measure?project=stephenlclarke_defender&metric=sqale_rating)](https://sonarcloud.io/summary/new_code?id=stephenlclarke_defender)
[![Vulnerabilities](https://sonarcloud.io/api/project_badges/measure?project=stephenlclarke_defender&metric=vulnerabilities)](https://sonarcloud.io/summary/new_code?id=stephenlclarke_defender)
![Repo Visitors](https://visitor-badge.laobi.icu/badge?page_id=stephenlclarke.defender)

`defender` is a native Rust implementation of Williams Defender red-label
arcade behavior. The runtime is self-contained: red-label tables, ROM metadata,
trace schema, video data, and sound command fixtures are embedded at build
time, so normal play does not need a local ROM or asset directory.

Live play uses a clean windowed `wgpu` renderer: it steps clean `Game` frames,
submits clean audio events, and executes `NativeSceneRenderer` sprite draw
plans. `--live-smoke` is the clean runtime smoke path and reports
sprite/temporary-raster evidence plus offscreen `wgpu` render/readback
signatures, including checked first/last frame signatures, without using the
legacy live presenter for frame generation.

## Current Fidelity Status

The clean runtime is the production runtime. Normal play steps clean `Game`
frames through clean platform, audio, and `wgpu` renderer modules, while the
converted machine remains feature-gated behind `legacy-tools` for developer
evidence and comparison tooling.

Earlier owner review rejected the clean media for concrete gameplay-facing
fidelity defects: laser shape, reverse-facing player orientation, and live
sound. Those known defects have since been repaired against local MAME
red-label evidence. The clean runtime now renders player lasers as sparse
source-byte body/tip/fizzle pixels, flips reverse-facing play to a distinct
left-facing player ship sprite, and decodes live audio through translated
Williams sound-board command families instead of the rejected guessed cue
mapping.

The accepted MAME-vs-clean report gate now covers sprite visuals, player laser
visual/audio, reverse orientation, explosion/coalescence visuals, terrain blow,
gameplay audio families, non-lander audio/visual presentation, playability,
rescue/loss, death/respawn, smart bomb, hyperspace, and organic non-lander
presentation. The full `make release-gate` path is green as of 2026-05-29 and
includes the accepted report gate, the owner-review evidence package, MAME
doctor, a short MAME recorder smoke capture, README media generation, game
smoke, live `wgpu` smoke, docs lint, and diff hygiene.

The accepted gate is still green, but final closure is no longer only an
owner-review decision. A fresh organic smart-bomb/up-thrust search found a
concrete last-human terrain-blow MAME candidate outside the accepted evidence
windows; the current clean candidate is silent and visually divergent. That
probe and the owner-review checklist are tracked in
`docs/fidelity/release-closure-audit.md`.

## Actor-Oriented Rewrite Branch

`src/actor_game.rs` starts an isolated actor-oriented Rust rewrite on the
`dev/actor-game-architecture` branch. It keeps the current runtime available
for fidelity evidence while introducing a driver-owned prompt/reply model where
each asset owns its motion, draw description, sound cues, and interactions.
This branch is not display-frame driven: `ActorGameDriver::step` advances a
simulation turn and emits a `StepReport`, while renderers are expected to draw
the latest report at whatever cadence they own.
`StepReport::render_scene` and `ActorRenderSceneBridge` now project actor draw
commands into the clean `RenderScene` contract using renderer-owned source
text, Williams logo, Defender coalescence, sprite atlas, projectile, and
explosion assets; this keeps actor behavior separate from render cadence while
making the actor report consumable by the native renderer path.
`ActorRuntimeAdapter` bundles each actor step into an `ActorFrame` containing
the original report, an actor-derived clean `GameState`, clean gameplay/audio
`GameEvents`, and the clean `RenderScene`. The state bridge maps actor phase,
score, stock, high-score state, and published actor snapshots with movement
velocity/facing metadata plus hostile-projectile source metadata into the clean
runtime contract without making actor behavior display-frame driven.
`--actor-smoke` exercises that actor frame path through a
scripted attract/play input sequence and the native draw planner, proving actor
events, audio, sprites, projectiles, HUD text, overlays, and `wgpu` command-plan
coverage. `--actor-wgpu-smoke` uses the same actor input sequence and renders
actor `RenderScene` frames through the offscreen `wgpu` readback path, proving
nonblank dynamic WGPU output. Normal `cargo run` now opens the actor live
runtime through `ActorRuntimeAdapter`, actor-derived clean `GameFrame` values,
the existing `wgpu` presenter, live audio runtime, and the same live key
bindings/XYZZY input state. `--actor-live` remains as an explicit alias for
that actor live path. Actor high-score entry now accepts live initials/backspace
through the actor input surface and draws the in-progress initials row from
actor state.
The current slice includes same-contract keyboard profiles, `XYZZY` overlay
state, thread-backed actors, a data-driven `AttractScript` for custom attract
drivers, `ActorBehaviorScript` profiles for level-wide and per-actor movement
and behavior tuning, script-selectable lander behavior modes, script-selectable
non-source hostile drift/chase modes, and named `ActorWaveScript` progression
backed by `assets/red-label/wave-table.tsv` for active wave size, lander and
bomber movement speed, lander fire cadence, baiter entry/shot/seek timing, and
scripted hostile/human spawn records. Read-only script manifests expose the
configured attract events, driver behavior, and wave profiles, while each
`StepReport` carries the effective per-step behavior manifest after transient
input overrides such as `XYZZY` invincibility. Behavior scripts can also be
parsed from checked text lines that update default, actor-kind, or actor-id
profiles. Wave scripts can be parsed from checked text lines that name wave
profiles, per-wave behavior updates, and lander/bomber/pod/human spawn
records, including spawn-index behavior profiles that become actor-id profiles
after the driver allocates those wave actors. The built-in actor attract,
behavior, and wave scripts are embedded from
`assets/red-label/actor-attract.script`,
`assets/red-label/actor-behavior.script`, and
`assets/red-label/actor-waves.script`, so the default driver and custom drivers
share the same checked script parser path. The attract script now includes
the source `ELECV` presents message, source-style Hall-of-Fame table rows,
source-offset scoring/instruction labels, the source `CREDV` credits
label/count, and a scriptable scoring scanner surface alongside Williams reveal
and Defender wordmark coalescence. Its source
page-start steps are Williams from step 1, `ELECV` from step 236, the Defender
wordmark from step 365, the high-score/zero-credit Hall-of-Fame page from step
488 for the source 60-tick stall window, and the scoring/instruction page from
step 1088. Title pages suppress the zero-credit line but still show a real
inserted credit through a `credits_nonzero` script action. The Hall-of-Fame
page also draws source-offset `HALLD_*` headings and the source Defender logo;
`hall_scores` draws Today’s and All-Time table columns from driver scores plus
embedded red-label seed initials. The scoring/instruction page draws `SCANV`
at step 1088, then reveals `LANDV`, `MUTV`, `BAITV`, `BOMBV`, `SWRMPV`, and
`SWARMV` on the source scoring-card process cadence from checked
`messages.tsv` rows and source screen addresses, while `scoring_surface` draws
the source top scanner frame/marker bars, `MTERR` mini-terrain records,
source-shaped rescue-demo player/human/lander object sprites, scanner blips,
source-shaped rescue laser pixels, lander explosion fragments, the 500-point
rescue popup, and the `ENMYTB` enemy legend transfer/materialization/reveal
objects before the default script cycles back to the first Williams reveal step
at source step 3367. Custom attract scripts can draw checked `messages.tsv`
labels through source cursor controls with optional visual offsets, can opt
into looping with `cycle <step-count>`, and otherwise remain unbounded linear
scripts; the older one-column `high_scores` action remains available for custom
screens.
The default actor wave allocator now
uses the source active-family
shape, so later waves can introduce bomber and pod actor families alongside
landers instead of remaining lander-only. Later-wave
actor landers now restore from source RNG placement/shot/velocity state, and
later-wave humans restore from the source target-list distribution.
Custom attract drivers can construct scripts in Rust or parse checked text
lines for text, static sprites, Williams reveal, Defender wordmark coalescence,
red-label message-table labels, high-score rows, and credit-count events before
passing the script to `ActorGameDriver`.
Source-backed landers, bombers, pods, swarmers, baiters, and humans publish
their fixed-point metadata through snapshots, publish per-step movement/facing
metadata for the clean state bridge, and advance fraction state during active
motion. Source-backed hostile actors now wrap Y motion through the source
active-object playfield bounds instead of drifting outside the red-label
vertical range.
Source-backed bombers now update seeded picture-frame and Y-velocity
metadata, including cruise-altitude and player-relative Y adjustments, during
active source motion from the driver-provided source RNG snapshot carried in
`StepPrompt`/`StepReport`, and source-backed baiters gate their picture-wrap
retargeting through that same source RNG snapshot before applying
`baiter_seek_probability` and adding player velocity to the source-shaped seek
velocity. Hostile projectile actors also publish source-shaped shell metadata:
enemy lasers own and advance fixed-point source velocity, fraction, and
lifetime state with the red-label shell-scan cadence, and enemy-shot spawn
commands can carry source fractions, velocities, and lifetime ticks from
scripted drivers and source-backed lander, swarmer, and baiter AI shots.
Source-backed bomber bomb actors carry stationary source bomb-shell fraction
and source-cadenced lifetime state into the clean `EnemyProjectileSnapshot`
contract, with nonzero scripted source lifetime ticks preserved at spawn. The
driver enforces the shared 20-slot source shell cap across
enemy-shot and bomb spawn commands plus the red-label 10-slot bomber bomb shell
cap. Source-backed bomb-shell spawn commands also honor the source `GETSHL`
X/Y placement bounds at X `0x98` and Y `0x2A`, while non-source scripted bombs
remain available. Enemy-shot spawn commands use the same source shell placement
bounds. Source-backed lander/human conversions now spawn mutant actors with
source-shaped mutant fractions, shot timer, hop RNG, and clean-state bridge
metadata. Source-backed mutant actors now read the red-label mutant wave
motion rows, advance actor-owned fixed-point velocity and hop state, and emit
source-shaped `0xF6` hostile shots. First-wave target6 conversions carry the
source X-correction into actor metadata, project target6 mutant draw/collision
positions through source-backed dive/visual anchors, defer the visible entry
shot when MAME does, and emit the exact source-shaped target6 fire2524
projectile metadata for the forced converted-mutant shots. Actor collision now
also honors the target6 fire2524 wait window, projected collision position,
source-positioned enemy explosion, hit sound, and scoring on player contact.
It also includes Williams title animation
metadata, the coalescing Defender wordmark surface, lander target-slot seeking,
an actor-owned
status display for score, high score, wave, lives, smart-bomb stock, credits,
and high-score-entry rows, pickup/carry/conversion, source-shaped
converted-mutant metadata,
falling-human rescue and safe landing scores, score popups, hostile projectile
actors for lander shots, source-command-backed sound cues for player laser,
lander and mutant hits, non-lander family hits, hostile shots, human
rescue/loss, and safe landing, bomber-laid bomb actors with source
bomb-collision cues and seeded source Y-motion metadata, pod laser-hit swarmer
spawning, swarmer scoring and shot cues, source-paced baiter timer entry,
baiter shots/scoring/hit cues, and stock-backed smart-bomb hostile clearing.
`ActorSoundEventBridge` now converts
actor `StepReport` sound cues into the clean `SoundEvent` surface consumed by
the live audio runtime, deriving thrust start/stop edges from the actor cue
stream. `ActorRenderSceneBridge` likewise converts report draw commands into
clean scene sprites for renderer-owned source text, attract graphics,
projectiles, explosions, and sprite atlas families. `ActorRuntimeAdapter`
combines those bridges into actor `ActorFrame` values with clean event batches
for future runtime selection while keeping the actor game separate from the
full clean `GameState` contract. Player
actor hyperspace now emits separate launch/materialization cues and clears
active source shell actors, including enemy shots and bomb shells, without
spending lives, smart-bomb stock, scoring points, or clearing hostile actor
families; the player actor owns the hidden hyperspace interval and
rematerializes at behavior-scripted coordinates with no draw, collision bounds,
or input actions while hidden. The driver owns
and advances the source hyperspace RNG during play, injects that
`SEED`/`HSEED`/`LSEED` snapshot into default/kind player behavior, and still
lets actor-specific scripts override the snapshot; rematerialization uses the
red-label X/facing/Y branch and the source `HYP2` `LSEED > 0xC0` delayed
death-risk path. Player hazard collisions now decrement driver-owned life stock
and respawn a replacement player when lives remain, while final-life hits still
enter game-over/high-score flow. Baiters do not block wave
completion once source-counted wave enemies are gone, and explosion draws carry
lander, mutant, bomber, pod, swarmer, baiter, bomb, player, and human variant
metadata through the actor render and clean-state bridges. Actor explosion
rendering now uses draw age with the clean source explosion-size curve, keeping
the source-family cloud centered as it expands, and descriptor-backed
enemy-family clouds now render through clean source expanded-object pixels with
source top-left and center metadata preserved where MAME evidence requires it.
`XYZZY` invincibility and overlay smart bombs are applied through the same
driver-controlled behavior and command paths used by scripts. See
`docs/actor-architecture.md` for the C++-to-Rust mapping and migration shape.

![Defender gameplay frame](docs/defender.png)

![Defender attract sequence](docs/start-sequence.gif)

## Install

Install from git with Cargo:

```sh
cargo install --git https://github.com/stephenlclarke/defender defender
```

Then run:

```sh
defender
```

## Run

Common commands:

```sh
cargo run
cargo run -- --actor-live # explicit actor-live alias
cargo run -- --input-profile planetoid
cargo run -- --input-profile cabinet
cargo run -- --cmos-path ~/.local/state/defender/red-label-cmos.bin
cargo run -- --live-smoke
cargo run -- --game-smoke
cargo run -- --actor-smoke
cargo run -- --actor-wgpu-smoke
cargo run -- --mute
cargo run --features legacy-tools -- --rom-report
cargo run --features legacy-tools -- --rom-report /path/to/roms
cargo run --features legacy-tools -- --verify-roms /path/to/roms
```

Fidelity and trace tooling:

```sh
cargo run --features legacy-tools -- --fidelity-trace 300
cargo run --features legacy-tools -- --fidelity-trace-inputs 'coin,start_one;fire,thrust;none'
cargo run --features legacy-tools -- --fidelity-trace-inputs-file /path/to/inputs.txt
cargo run --features legacy-tools -- --fidelity-check-trace \
  /path/to/inputs.txt /path/to/expected.tsv
cargo run --features legacy-tools -- --fidelity-check-trace-dir \
  docs/fidelity/fixtures/local/rust-current
cargo run --features legacy-tools -- --fidelity-list-scenarios
cargo run --features legacy-tools -- --fidelity-write-scenario-inputs \
  docs/fidelity/fixtures/local/reference
cargo run --features legacy-tools -- --fidelity-check-reference-trace-dir \
  docs/fidelity/fixtures/local/reference
```

Make targets:

```sh
make run
make run-wgpu
make live-wgpu
make smoke-wgpu
make game-smoke
make live-smoke
make ci
make ci-doctor
make trace-doctor
make coverage-doctor
make smoke-doctor
make reference-media-doctor
make reference-media-fetch
make reference-mame-capture
make reference-mame-smoke
make reference-clean-capture
make reference-window-scan
make reference-window-scan-organic
make reference-report-gate
make reference-signoff-summary
make reference-evidence-package
make owner-review-package
make release-gate
make fidelity
make clean-fidelity
make trace-script-test
make media-script-test
make trace-fixtures
make reference-inputs
make reference-traces
make reference-fixtures-check
make reference-media-check REFERENCE_MEDIA=/path/to/mame-defender.mp4 \
  REFERENCE_AUDIO=/path/to/mame-defender.wav \
  CANDIDATE_MEDIA=target/readme-media/start-sequence-candidate.gif \
  CANDIDATE_AUDIO=/path/to/clean-candidate.wav \
  REFERENCE_MEDIA_ACCEPTANCE_MODE=all \
  REFERENCE_MEDIA_REPORT_ONLY=1
make coverage
make coverage-new-code NEW_CODE_COVERAGE_BASE=origin/main
make coverage-new-code-baseline NEW_CODE_COVERAGE_BASE=origin/main
make sq-ci
make sq
make docs-lint
make diff-check
make readme-media
```

`make readme-media` builds the explicit `legacy-tools` tooling path, steps
clean `Game` frames, rasterizes the clean sprite scene/atlas media path, and
writes `target/readme-media/start-sequence-candidate.gif` plus
`target/readme-media/start-sequence-candidate.wav`. The WAV is rendered from
the clean `SoundEvent` timeline through the same source-derived sound-board
synth used by live audio. The target then compares the GIF candidate against
the protected original `docs/start-sequence.gif` with sampled frame, timing,
title, Hall of Fame, numeric-glyph, sprite, terrain, and scoring metrics. Until
the owner approves a replacement, `docs/start-sequence.gif` must not be
overwritten; set `DEFENDER_ALLOW_REFERENCE_MEDIA_OVERWRITE=1` only after that
approval.

`make reference-media-fetch` materializes the supplied MAME reference video,
`https://youtu.be/gss3lxeqCok`, into the ignored local path
`target/reference-media/sources/defender-red-label-mame-gss3lxeqCok.mp4`.
It uses `yt-dlp` when available and falls back to `youtubedr`.

The current local MAME clip inventory is tracked in
`docs/fidelity/mame-golden-clips.md`. It records timestamp windows and generated
artifact paths without committing copyrighted media.

`make reference-mame-capture` records directly from local MAME red-label ROMs.
It verifies the `defender` set under `assets/roms`, seeds the Williams CMOS
defaults used by the trace harness, records MAME's native video/audio, compresses
the capture to `target/reference-media/mame/*.mp4`, and writes a matching PCM
WAV. Set `MAME_REFERENCE_SECONDS=...` or `MAME_REFERENCE_BASENAME=...` for
bounded captures. Set `REFERENCE_SCENARIO=...` or
`REFERENCE_INPUT_PROGRAM='none*900;coin*4;...'` to drive MAME with the same
script grammar used by the trace fixtures. Set
`MAME_REFERENCE_TRACE_ONLY=1` with a scenario or explicit input program to skip
AVI/WAV/ffmpeg output and write only the MAME expected/debug TSVs; this is the
fast path for isolating gameplay state, sprite, and sound-command windows.
For rare branches that are hard to reach by input search alone, set
`MAME_REFERENCE_STATE_STEER=afall_player_catch`, `afall_safe_landing`,
`afall_fall`, `terrain_blow`, `enemy_explosion_matrix`, or
`enemy_materialize_matrix` with
`MAME_REFERENCE_STATE_STEER_FRAME=<frame>` to seed a small red-label RAM state
inside the MAME trace tool and let the original routines produce the resulting
sprite/process/sound-command evidence.
`make reference-mame-smoke` is the short executable recorder proof used by the
release gate. It runs `make reference-mame-capture` with
`MAME_REFERENCE_SECONDS=2` and
`MAME_REFERENCE_BASENAME=defender-red-label-smoke-script`, writing ignored
MP4/WAV artifacts under `target/reference-media/mame/`.

`make reference-window-scan` scans generated MAME expected/debug TSVs for
bounded non-lander sound/object windows and terrain-blow boundary evidence,
then writes `target/reference-media/reference-window-scan.json`.
`make reference-window-scan-organic` runs the same scan while excluding
synthetic/state-steered evidence and writes
`target/reference-media/reference-window-scan-organic.json`. Both reports
include nearest sound/object misses when target sound bytes are not close
enough to object evidence, per-family object row counts, the longest
contiguous object-evidence spans plus the best span for each object family,
`TERBLO` process rows, `ASTCNT`
distribution, terrain process misses, and any last-human terrain-blow
candidates found in the scanned corpus.

`make reference-report-gate` checks the tracked accepted-report manifest in
`docs/fidelity/reference-report-gate.json` against the local ignored report
JSON files under `target/reference-media/`. It fails when a report is missing,
not top-level `pass`, has a different acceptance mode than the manifest, or no
passing accepted report covers a required fidelity facet such as laser visuals,
reverse orientation, explosion/coalescence visuals, gameplay audio, rescue/loss,
terrain blow, or non-lander presentation. Each semantic coverage requirement
also carries a `min_reports` breadth floor, so the current accepted proof set
cannot silently shrink to one passing clip for a broad objective facet. The
manifest loader rejects duplicate report names, duplicate report paths,
duplicate coverage tags inside a report, and duplicate coverage/axis
requirements, so manifest edits cannot count the same proof twice. Every
manifest report row and local accepted `report.json` file must declare an
explicit matching `acceptance_mode`, and every report coverage tag must be
non-empty, declared by a semantic requirement, and compatible with that report's
acceptance mode. Manifest report paths must stay under
`target/reference-media/`; accepted MAME reference artifacts must stay under
`target/reference-media/mame/`, and accepted clean candidate artifacts must stay
under `target/reference-media/clean/`. Accepted visual reference artifacts must
be MAME MP4 files, accepted visual candidates must be clean GIF files, and
accepted audio artifacts must be WAV files. The gate also verifies that the
local reference and clean candidate media artifacts required by each accepted
visual or audio axis still exist and are non-empty, so stale report JSON cannot
pass after review media has been removed or truncated. Accepted visual axes must
also report positive reference, candidate, and compared frame counts; accepted
audio axes must report positive reference, candidate, and compared sample
counts. Accepted axes must not carry stale verifier `failures` entries.

`make reference-signoff-summary` runs the same manifest/report validation and
writes `target/reference-media/reference-signoff-summary.md`, a deterministic
owner-review matrix listing each semantic coverage requirement, the accepted
reports that prove it, the visual/audio metrics and media paths for each
accepted report, both the manifest and local `report.json` acceptance modes,
the required minimum report count per coverage facet, and the current all-trace
/ organic-only reference-window scan boundary metrics, including per-family
object counts and best object-family spans for selecting the next bounded
media windows. The summary is generated evidence and is not committed.
`make reference-evidence-package` regenerates both reference-window scan JSON
reports before writing that signoff summary; the summary includes the count of
required non-empty media artifacts and accepted visual/audio comparisons checked
by the report gate.
`make owner-review-package` regenerates the same evidence package, re-runs the
accepted-report gate, and prints the finite owner-review checklist from
`docs/fidelity/release-closure-audit.md`.

`make release-gate` runs the full local release validation path in order,
including `make owner-review-package`, so the accepted MAME-vs-clean proof set,
generated owner-review evidence, checklist, and the short local MAME recorder
smoke are checked from fresh local artifacts.

`make reference-clean-capture` renders the clean `Game` and sprite atlas with
the same `REFERENCE_SCENARIO` or `REFERENCE_INPUT_PROGRAM` grammar. It writes a
GIF, WAV, sound-event TSV, and clean debug TSV under
`target/reference-media/clean/`; set `CLEAN_REFERENCE_BASENAME=...` and
`CLEAN_REFERENCE_SAMPLE_STEP=...` for bounded candidate clips. Set
`CLEAN_REFERENCE_STATE_STEER=afall_player_catch`, `afall_safe_landing`,
`afall_fall`, `terrain_blow`, or `enemy_explosion_matrix` with
`CLEAN_REFERENCE_STATE_STEER_FRAME=<frame>` when generating tooling-only clean
windows for the matching MAME state-steered evidence. For `terrain_blow`, the
clean capture treats the requested steer frame as the MAME seed frame and wakes
the terrain-blow process 50 frames later, so the default steer frame `1400`
matches the MAME event at frame `1450`. Set
`CLEAN_REFERENCE_CAPTURE_START_FRAME=<frame>` and
`CLEAN_REFERENCE_CAPTURE_END_FRAME=<frame>` to encode only the comparison
window while still running the full input prelude. Candidate WAV output is
rendered from the start of the input program through the capture end, then
trimmed to the requested window, so sound tails that begin before the encoded
video window are still present in the bounded audio comparison.

`make reference-media-check` compares clean candidate media against a local
captured reference video such as a MAME recording. It uses `ffmpeg` to extract
deterministic visual frames and mono float audio, then writes
`target/reference-media/report.json`, `visual_metrics.tsv`, and
`audio_metrics.tsv`. The harness has no YouTube runtime dependency: download or
capture reference videos locally, pass the file as `REFERENCE_MEDIA`, pass the
matching MAME WAV as `REFERENCE_AUDIO` when available, and pass clean audio with
`CANDIDATE_AUDIO` when the visual candidate container has no audio track. Set
`REFERENCE_MEDIA_REFERENCE_START_MS` and `REFERENCE_MEDIA_CANDIDATE_START_MS`
when the MAME and clean timelines need different offsets for the same event.
Set `REFERENCE_MEDIA_DURATION_MS` for a bounded event window and
`REFERENCE_MEDIA_REPORT_ONLY=1` while inventorying known mismatches.
`REFERENCE_MEDIA_ACCEPTANCE_MODE` defaults to `all`; set it to `visual` for
synthetic sprite/explosion clips where sound is not part of the proof, or
`audio` for isolated sound-command clips where the visual scene is
intentionally not aligned. Defaults point at the fetched MAME video and the
generated candidate GIF/WAV. Stochastic Defender noise effects such as
laser/materialize are accepted by envelope, RMS ratio, peak level, and
zero-crossing behavior rather than exact random sample phase. On this
workstation `ffmpeg` is available on PATH at
`/opt/homebrew/bin/ffmpeg`.

## Controls

The default live input profile is `planetoid`, which maps BBC Micro
Planetoid-style keys onto Defender cabinet actions:

- `5`: left coin slot
- `6`: center coin slot
- `7`: right coin slot
- `1` or `ENTER`: one-player start when credit or free play is available
- `A`: move up
- `Z`: move down
- `SHIFT`: thrust
- `SPACE`: reverse
- `ENTER`: fire while playing
- `TAB`: smart bomb
- `H`: hyperspace
- `F2`: service advance / diagnostics target
- `F3`: reset today's high-score table from ROM defaults
- hold `F4` with `F2`: select the auto/audits target
- `F5`: slam / tilt switch
- `Q` or `ESC`: quit

The current MAME-backed red-label runtime evidence returns through
`GAME OVER` into the Hall of Fame page without a player initials prompt, even
when all all-time high-score slots are seeded to zero.

Use `--input-profile cabinet` for a MAME-style cabinet profile. In that
profile, `5` inserts a left-slot coin and `1` starts a one-player game;
`ENTER` is not a start key.

To start a normal one-player game from a fresh session, press `5`, then `1`.
The red-label start path intentionally blocks no-credit starts unless the
cabinet is configured for free play.

## Persistence

Live CMOS persistence is opt-in. Without `--cmos-path`, each run starts from
embedded red-label defaults and does not create or update a CMOS file. Provide
`--cmos-path <file>` to persist high scores, credits, audits, and adjustment
CMOS cells across runs.

## XYZZY Overlay

During live play, type `X`, `Y`, `Z`, `Z`, `Y` to toggle `XYZZY` mode.

`XYZZY` is a deliberate compatibility overlay, not Williams red-label arcade
behavior. With it disabled, trace generation and live core stepping stay on the
red-label path.

Current overlay hooks:

- `F`: toggles auto-fire while `XYZZY` is active.
- `TAB`: can emit an overlay smart bomb after red-label inventory reaches zero
  while `XYZZY` is active.
- `G`: toggles the recorded invincibility overlay flag. It is trace-invisible
  until an arcade-facing hook is implemented.

Typing `XYZZY` again disables the overlay and resets overlay toggles.

## Development

Primary validation commands:

```sh
make release-gate
```

`make release-gate` is the executable version of the full local release gate:
formatting, default and `legacy-tools` Rust tests, both clippy passes,
clean-fidelity, reference-media helper tests, fresh MAME evidence package,
accepted report gate and owner-review checklist, MAME doctor, short MAME
recorder smoke, README media generation, game/live smoke, docs lint, and diff
hygiene.

`make fidelity` runs formatting, default Rust targets, default clippy,
`legacy-tools` Rust targets, `legacy-tools` clippy, Lua trace exporter
self-tests, Python helper tests, local Rust trace fixture comparison, coverage,
and new-Rust-line coverage. The default Rust target set excludes the parked
legacy oracle/tooling adapters; the explicit `legacy-tools` pass validates
those developer tools. The new-line gate compares against
`NEW_CODE_COVERAGE_BASE` when set, otherwise `HEAD` for local dirty worktrees.
`make clean-fidelity` builds with `legacy-tools` and runs the clean rewrite
harness against the current MAME-backed embedded scenario set by default. It
compares the real clean `Game` to the accepted oracle and prints a TSV
first-divergence report. The oracle now carries source-backed high-score,
game-over timing, wave-profile, neutral object-list counts, bounded source
object-position/detail evidence, red-label object-picture descriptor metadata,
explicit clean sprite targets, source object-detail sprite projection,
expanded-object appearance/explosion/score-popup slot evidence plus sprite
projection, source-backed score-popup lifetime/value/position evidence, and
source-backed expanded-object appearance contraction evidence, source-backed
expanded-object explosion frame/lifetime/size-scale evidence,
source-backed player-death bank-7 pixel-cloud color/counter/piece evidence,
source-backed terrain-blow mutation/process evidence with `TEREX` presentation,
source `TERBLO` / `AHSND` entry command evidence, and source `TBSND`
completion command evidence, MAME-observed terrain flash windows, and
source-shaped terrain explosion growth cadence, source-backed per-family enemy
hit sound-command evidence, source-backed
enemy-projectile collision sound-command evidence, and source `BORDER`
top-display frame geometry. R9 now treats exact per-scenario pixel/offscreen
render parity, strict long-scenario sprite count/layer parity,
and wave-completion survivor-bonus loop/cadence beyond current presentation as
post-R9 audit follow-ups rather than active B01-B12 blockers. The former B13
visual rejection is now handled through the current owner-review checklist and
accepted MAME-vs-clean media gate rather than as an open implementation defect.
Player-one/player-two score digits, life/smart-bomb stock drawing, two-player
start admission/top-display initialization, the two-player player-start
prompt, and the two-player final-life `PLE02` switch/respawn handoff plus its
source message-glyph prompt are now clean-owned. Non-final player deaths now
pause active play through the source-backed player-death cloud, then respawn the
next stocked player through the existing player-start handoff; score and replay
bonus awards sync the active player's public stock snapshot so player-one and
player-two stock ownership stays isolated after rotation. Second-player
final-life switch-back follows the same player-start handoff and stock decrement
path for player one, and final two-player game-over uses the same no-entry
game-over/Hall-of-Fame return when no other player has stock. The final
player-death game-over sleep also draws the source `GAME OVER` prompt at the
translated `PLE2` screen position. Isolated high-score-entry scenes remain
available for table/editing tests and draw the source hall-of-fame player
label, instructions, entered initials, and source-shaped active/inactive
underline words at their translated screen positions.
Normal attract scenes draw the source-backed `CREDITS:` label and visible
credit count during Hall of Fame / scoring contexts and whenever a real credit
is present; the zero-credit line is suppressed on the Williams title pages.
During the Hall of Fame page, the credit line follows the same
protected-reference `COLTAB` `0x47` display magenta and protected-reference
display offset as the table text.
The title program now carries a source-backed page scheduler: Williams logo,
presents copy, Defender wordmark, copyright wait, and instruction labels are
gated by clean attract page frames and source wait constants while the
hall-of-fame display stall stays suppressed.
Wave-cleared scenes draw the source-backed `ATTACK WAVE`, `COMPLETED`, and
`BONUS X` status text with source-shaped wave and multiplier digits, plus
source-positioned survivor bonus icons for the clean survivors currently
remaining on that frame.
Playing scenes also project the source `BORDER` top-display frame as clean HUD
sprites: the lower display line, scanner side/top boundaries, and scanner
marker bars at translated source screen positions. Playing scenes also project
source-backed scanner/radar object and player blips from the scanner sleep
cadence, scan-left calculation, object erase table, player blip bytes, and
source `OBJCOL` scanner color words. Scanner terrain-raster residuals for
normal playing and hardware palette/RGB render audit residuals remain later
render-parity work.
Playing scenes also project bounded source object-detail rows that carry
`screen_position`, `picture_size`, and a mapped clean sprite: active rows draw
on the object layer, projectile rows draw on the projectile layer, and inactive
or transparent null-object rows remain evidence-only.
Playing scenes also project bounded source expanded-object detail rows that
carry `top_left`, source descriptor size, and a mapped clean sprite onto the
object layer. Live enemy appearance rows now render source-picture coalescence
pixels while their `RSIZE` contracts toward the final sprite, and the normal
enemy sprite is hidden during that active appearance window. Missing-size rows
and transparent null-object rows remain evidence-only.
Hall-of-fame display scenes also draw source-backed headings, the expanded
Defender logo, underline bars, and both visible high-score tables as rank,
initials, and score text. Pass `SCENARIOS="attract_boot start_game"` to narrow
the scenario set during focused work.
`cargo run -- --live-smoke` runs the clean live-smoke frame source and reports
`frame_source: clean_game`, `legacy_presenter_used: false`, sprite counts,
temporary-raster counts, and offscreen `wgpu` frame readback counts with
checked first/last frame signatures.
`make coverage-new-code` requires an explicit base and subtracts the accepted
uncovered-line baseline in `tools/new_rust_coverage_baseline.txt`; refresh that
baseline only when intentionally accepting existing uncovered debt. `make ci`
adds the `wgpu` live smoke test. GitHub CI runs `make ci-doctor`, then
`make fidelity`, then `xvfb-run -a make smoke-wgpu` so prerequisite, fidelity,
coverage, and smoke failures are separated in the Actions UI. The
`make smoke-doctor` target is Linux CI-oriented and expects `xvfb-run` plus
`vulkaninfo`.
Slack completion notes are a best-effort project protocol outside CI; connector
or token failures should be handled as Slack tooling failures, not Rust
validation failures.

Local SonarCloud support:

```sh
make sq-ci
make sq
```

`make sq` requires `sonar-scanner` and `SONAR_TOKEN`.

## Architecture

The primary source tree is now the clean rewrite under `src/`; the converted
implementation is parked under `src_legacy/` and is compiled only when the
explicit `legacy-tools` feature is enabled. Clean runtime launch goes through
the private `src/runtime.rs` bridge without compiling the accepted machine,
legacy live core, CMOS storage, or retired raster presenter in default builds.
The internal oracle reaches accepted behavior through the crate-private
`src/accepted.rs` facade before `src/oracle.rs` adapts it to gameplay state
when `legacy-tools` is enabled.
The actual legacy machine bridge lives in `src_legacy/accepted_behavior.rs`,
keeping legacy imports out of the clean accepted-behavior surface. The root
legacy adapters that remain active are feature-gated and crate-private; README
media generation uses the doc-hidden, feature-gated `defender::readme_media`
facade instead of low-level machine, input, or video exports. Machine
process/state contracts, red-label math types, and low-level asset, board,
memory, ROM, sound, PIA, and video modules stay crate-private. Generated
long-trace sample fixtures are private to the legacy machine oracle, not
root-wired through the clean crate.

Clean rewrite modules:

- `src/accepted.rs`: crate-private accepted-behavior contracts and facade over
  the temporary adapter.
- `src/game.rs`: gameplay-facing `Game`, `GameState`, `GameInput`,
  `GameFrame`, `GameEvents`, world, terrain, starfield, enemy, human, score,
  source-backed wave profile, high-score table/session state, isolated
  initials-entry surfaces, game-over return timing, player-switch/final
  game-over scene sprites, hall-of-fame display heading/table scene sprites,
  projectile, player, direction, and sound-event contracts without accepted
  command-byte mapping.
  The clean `Game` shell emits sprite-first scene frames without touching the
  accepted machine adapter.
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
  execution, and the clean `Game` frame source for `--live-smoke`.
- `src/roms.rs`: the crate-private optional ROM verification facade that owns
  the temporary ROM metadata, scan, and loader bridge.
- `src/audio.rs`: gameplay-facing sound events, the bounded live-audio runtime,
  no-device backends, and worker diagnostics. It consumes clean `GameFrame`
  and `SoundEvent` contracts, not legacy frame outputs.
- `src/sound_board.rs`: translated Williams sound-board synthesis and command
  decoding for source GWAVE vectors, VARI sweeps, special routines, organ
  routines, and thrust-loop playback.
- `src/fidelity.rs`: clean frame-equivalence signatures over gameplay state,
  gameplay events, sound events, and render summaries. Clean fidelity tests use
  oracle-owned reference probes instead of importing accepted facade types
  directly.
- `src/clean_fidelity.rs`: test-only clean rewrite harness that steps the real
  clean `Game` against the accepted oracle and emits first-divergence TSV
  reports for selected Phase 1 scenario input streams.
- `src/fidelity_manifest.rs`: the crate-private fidelity scenario manifest
  facade that owns temporary scenario metadata and input expansion.
- `src/fidelity_trace_engine.rs`: the crate-private fidelity trace engine
  facade that owns temporary trace generation, comparison, and schema access.
- `src/oracle.rs`: the crate-private gameplay oracle, returning clean state,
  event, sound, and scene-summary frames from the accepted-behavior facade for
  internal fidelity comparison.

Legacy source-shaped modules under `src_legacy/` still own the accepted arcade
behavior, assets, hardware models, ROM verification, rendering, input,
sound-board command evidence, legacy fidelity trace generation and threaded
fixture checks, plus parked historical live, CMOS, and presenter code that is
no longer compiled from `src/lib.rs`. `src_legacy/accepted_behavior.rs` owns
the temporary accepted-machine adapter for the internal oracle.
`src/live_wgpu.rs` owns clean config-driven interactive `wgpu` launches and
routes `--live-smoke` through clean `Game` smoke frames. `src/roms.rs` owns the
temporary ROM metadata, scan, and loader bridge for optional verification
commands.
`src/fidelity_manifest.rs` owns the temporary scenario manifest and input
expansion bridge for fidelity scenario commands. `src/fidelity_trace_engine.rs`
owns the temporary trace generation, comparison, and schema bridge for fidelity
trace commands.
These ROM, scenario, trace, and oracle modules are explicit `legacy-tools`
developer tooling rather than default production runtime wiring.
Legacy-specific clean equivalence regressions are also wired from `src_legacy/`
under that feature so `src/accepted.rs` and `src/oracle.rs` stay focused on
clean gameplay contracts. They remain wired as doc-hidden legacy bridge modules
rather than supported public API.
Internal clean equivalence regressions use crate-private oracle wiring, while
clean frame-signature gates live under `src/fidelity.rs` and compare clean
render signatures rather than exposing memory-oriented CRC labels. README media
tooling uses the narrow doc-hidden `defender::readme_media` facade only through
`legacy-tools`, but that facade now captures clean `Game` frames and the clean
sprite scene/atlas media path. The binary enters through the clean platform
boundary before delegating to the runtime bridge. `--game-smoke` steps the clean
game through scripted controls, verifies required gameplay sprite layers,
sprite IDs, native draw-command pipeline and instance coverage, sprite buffer
upload-plan coverage, render-pass plan coverage, and frame-command sprite
command/draw/instance plus ordered sprite-only begin-pass, viewport, and
projection upload coverage, and prepares sprite-only native draw plans plus
frame-level `wgpu` command, resource bind-group, pipeline-layout bind-group,
pipeline descriptor shape, encoder command-shape, and upload plans without
entering the legacy live presenter. `--live-smoke` still reuses that clean frame
source, renders the smoke frames through an offscreen `wgpu` target, reads back
pixel signatures, checks the selected first/last signatures, and reports
live-smoke evidence with `legacy_presenter_used: false`. `--actor-smoke` is the
actor-runtime counterpart: it steps `ActorRuntimeAdapter`, verifies attract,
credited attract, playing, clean gameplay/audio events, required actor sprite
families, projectile/HUD/overlay layers, native sprite draw commands, native
pipelines, and frame-level `wgpu` command plans without entering the legacy
presenter. `--actor-wgpu-smoke` then renders actor scenes through the actual
offscreen `wgpu` texture/readback path and requires every actor smoke frame to
produce nonblank output plus dynamic readback signatures. Normal interactive
play and the explicit `--actor-live` alias now open an actor-frame window
through the same `wgpu` presenter and submit actor-derived clean `GameFrame`
values to live audio; actor high-score entry accepts the same
initials/backspace keys through that path. Actor frames publish a clean state
snapshot alongside events and scene data for the live runtime.
The clean `Game` world seeds terrain, starfield, source-profile active enemy
batches, human, and projectile snapshots for playing waves and renders them as
atlas-backed scene sprites. Operator controls are
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
wave batch now uses that profile to spawn lander, bomber, and pod families when
the source wave exposes them, with family-specific sprite, collision-size, and
score mappings. Initial wave landers retain deterministic source fixed-point
fractions, shot timers, picture frame, and X/Y velocity, then advance through
the same bounded source `LANDS0` orbit/shot loop as source-restored landers.
Initial active pods retain deterministic source fixed-point fractions and
bounded signed X velocity, then advance through the same source fixed-point
X/Y motion as source-restored pods.
Remaining source-profile enemies stay in
`EnemyReserveSnapshot`, flow into inactive object-evidence counts plus bounded
inactive source-detail rows, and activate as the next clean batch before
`WaveCleared`; those inactive rows carry the reserved family category, source
object-picture descriptor, deterministic source object-table identity, mapped
clean sprite, and source scanner color while leaving position and velocity
empty until activation. Reserve landers now use source `LANDST` placement,
fixed-point fractions, shot-timer RNG, velocity bytes, and then advance through
a bounded source `LANDS0` orbit/shot loop with picture cycling and
source-shaped fireball projection. When no humans remain, the reserve lander
path follows the source `LANDST` schizoid fallback and restores source-shaped
mutants directly. Reserve pods now use source `PRBST`/`PRBRES` placement,
fixed-point fractions, and signed velocity bytes before entering source
fixed-point X/Y motion. Active source enemy Y motion now uses the source
`VELO` `YMIN`/`YMAX` wrap for landers, pods, bombers, mutants, swarmers, and
baiters. Destroyed pods now spawn a
deterministic clean mini-swarmer batch using the source request bound and
active-swarmer cap across projectile and smart-bomb kills. Spawned
mini-swarmers carry source RNG-derived velocity, acceleration, sleep, and
shot-timer state, and reserve mini-swarmer activation now uses source
`PLRES`/`RSW0` phony-object placement before the same source swarmer runtime.
Mini-swarmers advance through the source entry seek, fixed-point loop, vertical
acceleration/damping, turnback, and enemy-bomb projection shape while sharing
the source shell free-list cap with the other fireball paths, including source
`RMAX` RNG consumption on shot-timer resets when allocation fails, and emit
source `SWSSND` command evidence when allocation succeeds. Clean baiter entry
now advances on the source game-exec pacing cadence, accelerates when the
remaining source wave-enemy total is low, excludes active baiters from that
source `WVCHK` count, and respects the source active-baiter cap. Active baiters
therefore do not block reserve activation or wave clear when no source-counted
enemies remain.
Spawned baiters retain source shot-timer, picture-cycle, sleep, and velocity
state, pursue the player through source seek rules, fire source-shaped
fireball shells with source `USHSND` command evidence, and those enemy
projectiles use source `SHSCAN` lifetime decrement/wrap behavior,
scroll-adjusted fixed-point motion, offscreen culling, collision scoring, and
player-damage handling with source `BKIL` / `AHSND` command evidence when a
shell hits the player plus source `PLEND` / `PDSND` command evidence when the
player-hit path starts.
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
`AHSND` command evidence, and the existing last-human planet-loss handoff
that starts source `TERBLO` / `AHSND` terrain-blow command evidence before
advancing to source `TBSND` completion evidence; falling
humans caught by the player enter the clean player-carried state, award the
source-backed 500-point rescue score, emit source `ACSND` command evidence,
and start the existing `P500` score-popup lifecycle; player-carried
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
bomber positions continue through source velocity.
Pod reserve activation now uses source `PRBST`/`PRBRES` placement, fixed-point
fractions, and velocity bytes before entering source fixed-point X/Y motion
with the same source `VELO` Y-bound wrap, and reserve bombers now use source
`TIEST` four-slot player-relative squad placement and alternating X velocity
before entering the source bomber runtime. Enemy projectile evidence now
distinguishes source `FBOUT` fireballs from source `BMBOUT` bomber bomb shells
and carries source `BMBP1` shell descriptor fields for the standalone
mine/source-shell fixture. Enemy-projectile/player collision uses the source
`BMBP1` 2x3 footprint while the direct runtime projectile renderer keeps the
existing 4x6 bomb sprite.
Active clean enemy evidence now carries source object-picture descriptor labels,
addresses, dimensions, and primary/alternate image pointers for the current
lander, baiter, bomber, mutant, pod, and swarmer presentations, and clean
projectile/enemy plus player/enemy collision uses those source enemy picture
sizes while direct runtime enemy rendering keeps the current clean sprite
sizes. Clean hostile player collision uses the source `PLAPIC` / `PLBPIC` 8x6
player picture footprint while the direct runtime player renderer keeps the
current 16x6 ship sprite; falling-human rescue collision uses that player
footprint plus source `ASTP1`-`ASTP4` 2x8 astronaut footprints while direct
runtime human rendering keeps the current 4x8 sprite. Clean
player projectile evidence now carries the source `LASP1` descriptor label,
address, 8x1 size, and primary image pointer while the direct runtime
projectile renderer projects each live shot as a red-label tail-to-tip laser
span. Clean player projectiles advance through the source `LASR0` / `LASL0`
five-column loop step across the full clean playfield, keep the trailing beam
moving at the source one-column erase cadence, and use the leading 16x1
`LASP1` hit span for enemy collisions. Clean enemy, human,
player-projectile, and
enemy-projectile object evidence also carries source-style 8.8
world-position words, velocity words, and deterministic source object-table
identity evidence from the clean source fixed-point state and source layout:
addresses from `0xA23C` plus `0x17` per slot, source slot numbers, and neutral
`OTYP` `0x00`. Runtime scene sprites remain on the direct clean render path.
The R9-C4 residual ecology audit now classifies the per-family
movement/projectile runtime surfaces as covered by the current clean runtime
and focused unit tests, and targeted source ecology fixture hardening matches
the current `start_game`, `smart_bomb`, `hyperspace`, `abduction`, and `death`
clean-fidelity scenarios. The R9-C4.5
closure gate closes Step 50/B08 without exposing drift, so R9-C4.2 stayed
unused. Step 51/R9-D1 now closes B09 two-player flow with focused fixtures for
final-life switch in both directions, non-final death rotation, stock and score
ownership, and current-player final game-over/Hall-of-Fame routing.
Clean
smart bombs consume player stock, clear active enemies through
`SmartBombSystem`, route score through the same scoring system, and leave
destroyed active enemy sprites absent from the scene while source reserves can
enter as the next active batch. Successful clean laser launches emit source
`LASSND` command evidence, and accepted smart-bomb inputs emit the first source
`SBSND` command before enemy destruction sounds; accepted thrust inputs emit
the source start/stop sound events on press and release. Accepted clean
hyperspace inputs clear active enemy projectiles through the source `HYP02` / `KILSHL`
shell-list cleanup while leaving player projectiles outside that shell-object
list. They then reload source rematerialization state from the current clean
source `SEED`/`HSEED`: the clean camera/background word, player X/facing branch,
player Y high byte, cleared velocity, and source `APSND` appearance command.
Their clean `HYP2` tail follows the source `LSEED > 0xC0` death-risk branch
into the existing player damage path with source `PDSND` command evidence,
while `0xC0` and below complete safely.
Playing scenes draw
current-player
life-stock and smart-bomb-stock HUD sprites with source-backed display caps,
positions, and the reclassified stock sprite targets. Enemy contact with the
player is resolved through clean collision and `PlayerDamageSystem`,
decrementing lives, removing the colliding enemy, and entering `GameOver` on
the final life. The player-death pixel cloud is cleared before the final
attract handoff so Hall of Fame scenes remain table-only.
MAME red-label evidence does not show automatic initials entry after a
qualifying final score, even when all all-time high-score slots are seeded to
zero. The clean runtime therefore returns through the no-entry
Hall-of-Fame/game-over path after final death. `HighScoreEntrySystem` remains
covered as an isolated table/editing surface: it accepts alphabetic initials
through clean input, normalizes them to uppercase, supports backspace, emits
`HighScoreInitialAccepted`, and emits `HighScoreSubmitted` when the third
initial enters the source-shaped hall-of-fame display stall before the clean
game returns to attract. Manual submissions insert into both all-time and
today's-greatest tables by score rank while preserving the current-player
submission metadata. Isolated high-score-entry scenes draw the source-backed
player label, four hall-of-fame instruction lines, and entered initials with
message glyph sprites plus source-shaped active/inactive underline words.
During the hall-of-fame display stall, scenes draw the
source-backed display headings plus both visible high-score tables with rank
digits, initials, score fields, and source-shaped underline bars. Enemy
exhaustion is reported through `WaveSystem`, keeping the last-hit frame empty
and spawning the next clean wave on the following playing frame. Native draw
planning
resolves scene sprites
through
renderer-owned atlas regions into sprite batches and records GPU
instance-buffer data with native scene rectangles, normalized atlas UVs,
normalized tint, stable record counts and upload bytes, and the `wgpu` vertex
layout for the instance buffer. It flattens those
per-batch records into one upload-ready
instance stream. The renderer also owns unit quad vertices, `u16` indices,
record counts, upload bytes, and the `wgpu` vertex layout used to draw
instanced sprites, then derives indexed instanced sprite draw commands with
quad/index counts, instance ranges, and upload byte spans into that stream.
Sprite draw plans also include
`wgpu::BufferUsages` metadata and upload bytes for the quad vertex, quad index,
and instance buffers, plus a sprite render-pass plan with stable vertex buffer
slots, index-buffer metadata, indexed instance draw ranges, draw counts, and
instance counts. Sprite pipeline plans describe the renderer-owned WGSL shader,
vertex and fragment entry points, quad and instance vertex layouts,
alpha-blended color target, primitive state, and multisample state for the
target texture format. Sprite resource
binding plans describe the scene-projection uniform upload, projection bind
group layout, atlas texture binding, atlas sampler binding, atlas texture
upload metadata, and expected bind-group and binding-entry totals used by that
shader. The default clean sprite atlas decodes the source-backed runtime
regions, source table pixels, and reclassified PNG inputs into nonblank
renderer-owned regions plus the `wgpu` texture format, usage, extent, and copy
layout needed to populate it. Sprite pipeline layout
plans then order those projection and atlas bind groups for `wgpu`
`PipelineLayoutDescriptor` creation and expose the expected bind-group and
binding-entry totals carried into that layout. Sprite render pipeline
descriptor plans combine that layout with shader entries, vertex buffers,
primitive state, color target, and multisample state for `wgpu` render pipeline
creation, and expose the layout bind-group, vertex-buffer, and color-target
totals carried into the descriptor. Sprite
render-pass encoder command plans then order the pipeline, bind groups, vertex
buffers, index buffer, and indexed draw calls for `wgpu::RenderPass`
execution, and expose the set-pipeline, set-bind-group, set-vertex-buffer, and
set-index-buffer command totals carried into the encoder.
Frame-level GPU command plans combine begin-pass clear state, viewport command
presence, scene-projection upload presence, optional sprite execution with
command, draw, and instance totals, an ordered sprite-only stream predicate,
and separate raster-tooling evidence into one ordered scene command stream.
The clean smoke and live smoke gates require zero temporary raster commands for
the active gameplay path. It also records the centered viewport layout plus GPU-ready
clear color, viewport command, and scene-projection constants for the target
surface. Live presentation now steps clean gameplay frames directly; parked
Kitty graphics, terminal-session code, legacy live code, CMOS storage, and the
old `wgpu` presenter remain historical compatibility evidence outside default
crate wiring. The legacy video renderer owns its remaining `TerminalGeometry`
value type directly so it does not pull terminal session setup into active
builds. Generated long-trace sample data is nested under the legacy machine
oracle because it is historical fixture evidence, not a clean root adapter. A
public API guard scans clean module sources so new production code cannot
import low-level legacy root modules, bypass the accepted-behavior facade, or
reintroduce legacy implementation terminology.

## Assets And ROMs

Clean runtime data lives under `assets/red-label/` and is embedded in the
binary. The active assets include ROM metadata, MAME memory/input maps,
red-label RAM/CMOS layouts, linked lists, routine addresses, switch tables,
object pictures and images, terrain data, wave data, high-score/default data,
actor attract/behavior/wave scripts, including prompt-backed attract
high-score rows, sound command timelines, the live-audio acceptance matrix, and
fidelity trace schemas.

Local ROM files are optional verification inputs for `--rom-report` and
`--verify-roms` when built with `--features legacy-tools`. They are not needed
for normal play.

Legacy prototype cue WAVs under `assets/arcade/` are retained only as
references unless a module explicitly reclassifies them with source or ROM
provenance. Sprite and sprite-sheet PNGs live under `assets/sprites/`. `DC-156`
temporarily reclassifies `ship1.png`, `lander1.png`, `humanoid1.png`,
`player-shot.png`, and `font-sheet.png` as R2 clean sprite-atlas inputs; they
are transitional art inputs, not authoritative gameplay evidence. `DC-164`
maps matching red-label picture labels for those inputs into clean sprite
evidence and extends that bridge to the existing enemy-family prototype sprites
`mutant1.png`, `baiter1.png`, `bomber1.png`, `pod1.png`, and `swarmer1.png`.
It also maps bounded bomb, explosion, score-popup, life-stock, and smart-bomb
stock picture labels to existing transitional sprites. The residual `ASXP1`,
`NULOB`, and `TEREX` picture labels are atlas-backed from
`assets/red-label/object-images.tsv` bytes because no PNG has been reclassified
for them. The accepted oracle also exposes source object-detail rows as sprite
presentation evidence, expanded-object appearance/explosion slots as sprite
presentation evidence, and source `BORDER` top-display frame geometry, but
clean gameplay lifecycle behavior outside the source-backed score-popup
surface, source expanded-object explosion timing, and source-backed
player-death pixel-cloud and terrain-blow surfaces, and wave-completion
survivor-bonus loop/cadence beyond current presentation remain source-backed
audit residuals. R9-E1 records exact per-scenario pixel/offscreen render parity
as audit evidence rather than another clean runtime surface.
Red-label message glyphs now back the
two-player player-start `PLAYER ONE` / `PLAYER TWO` prompt, player-switch
`PLAYER ONE` / `PLAYER TWO` plus `GAME OVER` prompt, the ordinary final
`GAME OVER` prompt, attract `CREDITS:` text, attract
`ELECTRONICS INC.` / `PRESENTS` text through source message row-feed and
horizontal-cursor controls, attract `SCANNER` and enemy score labels,
wave-completion `ATTACK WAVE` / `COMPLETED` / `BONUS X` status text, active
isolated high-score-entry player label/instruction/initial glyphs, and
hall-of-fame display heading/table text; survivor bonus icons use the source
`ASTP3`
shape at the translated wave-cleared frame positions. The entry and display
underline words use a small atlas-backed clean sprite at the source-shaped
word positions. The
hall-of-fame Defender logo and normal attract Defender wordmark are generated
from the compressed source logo bytes into the clean sprite atlas. The normal
attract title program is scheduled through clean page-frame gates backed by
source wait constants, now ordering the title/copyright surfaces before the
Hall of Fame display and source scoring/action sequence. The scoring/action
sequence replays the source instruction-page rescue demo, scanner blips,
laser/explosion/500-point rescue beat, and `ENMYTB` enemy score-card reveal
before cycling back to the Williams page. The clean scoring page starts that
display from the protected reference's post-setup rescue-fall phase and uses
the protected GIF's sampled `COLTAB` cadence for scoring text and credits while
preserving the source scoring-page duration. The attract scoring scanner uses
the source `MTERR` mini-terrain slice consumed by `SCNRV`, applies the
protected-reference display offset to scoring text/credits/scanner/terrain
surfaces, and uses the protected-reference purple border tint for that scoring
page. The lower scoring-page terrain is drawn from source `BGOUT` records as a
single source scanline instead of the earlier two-scanline placeholder height.
The `ENMYTB` score-card table objects use protected-reference placement for the
six enemy entries while preserving the source sprite atlas and clean scoring
cadence. The Williams logo is generated from the source `LGOTAB` final pixel
pattern and early title frames draw a source-ordered handwritten reveal with
source `TCTAB` Williams palette cycling;
the reveal cadence follows source `LOGO0` table operations instead of a linear
pixel ramp. For protected README media, the first title sample stays blank and
the Williams/logo title colors follow sampled protected-reference cadence.
Title text uses the Williams resistor palette with source `COLTAB` cycling. The
protected GIF's Defender appearance now uses source `APVCT`
row-pair coalescence projected as sprite-first clean pixels instead of
temporary rasters.
The normal attract Defender wordmark now uses the 15 source `DEFENS` 4-byte by
12-row `APVCT` appearance slots and descriptor addresses during the appearance
window before returning to the full `0x3C` by `0x18` source-expanded wordmark
for settled frames. Source numeric digits decode the `NUMBR0`-`NUMBR9` image
records in column-major byte order for score, credit, Hall of Fame, and
scoring/action text surfaces. The normal attract copyright strip is generated
from the source `CPRTAB` bitmap bytes.
The playing top-display border uses a small atlas-backed clean border word
sprite projected from the source `BORDER` geometry: bottom line, scanner side
boundaries, top scanner boundary, and scanner marker bars.
Scanner/radar object and player blips use small atlas-backed HUD sprites at the
source scanner screen positions. Object blips are derived from bounded
active/inactive object evidence and source scanner color words; projectile rows
remain non-scanner rows.
Clean human object-detail rows now carry per-human source astronaut descriptor
evidence: default `ASTP1` rows and source-restored `ASTP3` rows selected from
the `PLRES` `LSEED` low bit, including descriptor label, address, 2x8 picture
size, primary/alternate image pointers, and mapped clean human sprite evidence
while the runtime playfield keeps drawing the clean 4x8 astronaut sprite.
Source-restored clean humans also retain the `PLRES` `LSEED` X low byte as the
source X fraction used in object-detail world-position evidence. Clean
worlds carry a separate source `ASTRO` process cursor/sleep state that walks
one restored, uncarried target-list human per source cadence, updates source
fixed-point X motion and terrain-relative Y position, and cycles evidence from
`ASTP1` through `ASTP4`. Clean object-detail rows also carry source-layout object
addresses, slots, and neutral `OTYP` evidence while the clean scene path skips
those source-detail rows to avoid duplicate runtime sprites.
HUD, attract title, top-display border, Hall of Fame logo/text, underline, and
blink-adjacent surfaces now share a source visual-state contract for the
source PCRAM/color indices, border words, underline words, Williams restore
rates, and Hall of Fame blink sleep/color evidence. The Williams title path now
uses the source-rate cadence. The default runtime atlas decodes the
early-gameplay player, enemy, projectile, bomb, explosion, reward, stock, and
smart-bomb sprites from `assets/red-label/object-images.tsv` with source
palette overrides instead of the transitional prototype sprite PNGs.
Mapped active and projectile source object-detail rows are projected from their
source screen positions and descriptor sizes into the object/projectile layers;
inactive and transparent null-object rows remain comparison evidence only.
Mapped source expanded-object appearance/explosion detail rows are projected
from their source top-left positions into the object layer. Appearance rows use
descriptor sizes directly unless they carry a live high-bit `RSIZE`, in which
case they draw as source-picture coalescence pixels until the final sprite
size is reached. Live enemy appearances track the moving clean enemy while
suppressing only that matching normal sprite, and the first wave now
materializes its second live lander at the MAME-observed `0xEA` cadence without
restoring the old five-lander reserve batch. Enemy-family explosion rows retain
source `RSIZE` frame/lifetime metadata and draw as sparse source-style pixel
clouds around the mapped sprite center until the source kill threshold is
crossed.
Missing-size and transparent null-object rows remain comparison evidence only.
Player-one and player-two score digits and stock drawing now use
source-backed clean scene
sprites, two-player credited-start admission initializes the two-player top
display, and final-life two-player handoff follows the source `PLE02` switch
sleep before respawning the other player. Non-final deaths now wait through the
player-death cloud and rotate to the next stocked player before the clean
player-start handoff. Two-player stock, score, switch, and final game-over
routing is covered by focused B09 fixtures. New sprite
files should stay under `assets/sprites/`, new non-legacy sound artifacts should
stay under `assets/sounds/`, and pre-existing legacy `.wav` cues should remain
under
`assets/arcade/`.

## Platform Support

- Live backend: `wgpu`, through `cargo run` or `defender`.
- Live audio consumes gameplay-facing `SoundEvent` batches through a bounded
  non-blocking runtime with worker diagnostics. Normal interactive play
  attempts a synthesized device backend and falls back to the no-device null
  backend if host output is unavailable; `--mute` disables the runtime path.
  Foreground sound-board commands are single-DAC and interrupt the previous
  foreground command; thrust remains a looped clean backend voice. Smoke mode
  remains no-device and deterministic. The accepted implementation contract is
  in `docs/fidelity/live-audio.md`.

## References

- Red-label source and ROM build reference:
  <https://github.com/mwenge/defender>
- MAME Williams driver and ROM map:
  <https://github.com/mamedev/mame/blob/master/src/mame/midway/williams.cpp>
- MAME Williams video implementation:
  <https://github.com/mamedev/mame/blob/master/src/mame/midway/williams_v.cpp>
- MAME Motorola 6821 PIA implementation:
  <https://github.com/mamedev/mame/blob/master/src/devices/machine/6821pia.cpp>
- Williams sound-ROM source:
  <https://github.com/historicalsource/williams-soundroms>
- Defender setup and operations manual:
  <https://arcade-museum.com/manuals-videogames/D/DefenderSetupBookletUSA.pdf>
- Williams Defender cabinet reference:
  <https://williamsarcades.com/Defender>
- Red-label ROM metadata reference:
  <https://mdk.cab/game/defender>
- Defender behavior analysis:
  <https://www.dougmahugh.com/defender-chapter02/>
- BBC Micro Planetoid archive entry:
  <https://bbcmicro.co.uk/game.php?id=11>
