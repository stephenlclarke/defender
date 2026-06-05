# Release Closure Audit

Last reviewed: `2026-06-05`

This audit maps the active fidelity goal to current evidence. It is deliberately
stricter than the release gate: green tests and media reports prove the covered
clips, but they do not prove universal arcade fidelity for every possible game
state.

## Goal Scope

The clean `wgpu` implementation must match Williams Defender red-label MAME
from these perspectives:

- sprite shapes, colors, orientation, scanner/HUD elements, terrain, and
  starfield presentation
- laser origin, span, sparse body/tip/fizzle pixels, and hit endpoint
- explosion placement, growth, fragmentation, and pixel coalescence
- audio command timing and waveform shape for gameplay sound families
- gameplay behavior for controls, waves, enemy fire, collisions, rescue/loss,
  scoring, death/respawn, game-over, and Hall of Fame return
- clean runtime architecture using clean domain systems, clean audio, and
  `wgpu` sprite rendering, with legacy code kept as optional evidence/tooling

## Current Proof

- MAME/clean media harness:
  `make reference-mame-capture`, `make reference-clean-capture`,
  `make reference-mame-smoke`, `make reference-media-check`, and
  `make media-script-test` are present. MAME capture now rejects missing or
  empty MP4/WAV outputs. Status: proven for local generated artifacts.
- Red-label ROM/MAME path:
  `make reference-mame-doctor` passed in the release gate.
  Status: proven locally.
- Core gameplay equivalence scenarios:
  `make clean-fidelity` passed for attract, start, first gameplay, firing,
  thrust/reverse, smart bomb, hyperspace, abduction, and death.
  Status: proven for the embedded scenario set.
- Runtime smoke paths:
  `cargo run -- --game-smoke` and `cargo run -- --live-smoke` passed. Live
  smoke rendered `320` nonblank offscreen `wgpu` frames.
  Status: proven for the smoke path.
- Clean runtime fidelity-debt scan:
  a `2026-05-29` bounded scan checked production clean runtime source for
  `TODO`, `FIXME`, `placeholder`, `stub`, `approximate`, `guess`,
  `unimplemented!`, and `todo!` markers. No active sprite/audio/gameplay
  placeholder path was found. The remaining hits are guard tests, source asset
  invariants, clean CLI unsupported-argument tests, or raster-tooling counters
  that the smoke gates require to remain zero in the active gameplay path.
  Status: proven as source hygiene for current clean runtime debt markers.
- Release validation:
  the release gate in `PLAN.md` passed on `2026-05-29 15:54 BST` after the
  PRBP1 pod up-thrust report was promoted to all-axis evidence and the
  clean-only post-game thrust/background audio leak was fixed. The current gate
  now also includes actor smoke checks, so full current-gate status requires a
  fresh run after the actor rewrite slices. The gate includes default and
  `legacy-tools` Rust tests, both clippy passes, clean fidelity, media script
  tests, owner-review package generation, the accepted-report gate, MAME
  doctor, MAME smoke recording, README media, game smoke, actor smoke gates,
  live smoke, docs lint, and diff hygiene.
  Status: previously proven for the 2026-05-29 gate; current expanded gate
  pending fresh full run.
- Reference report closure gate:
  `docs/fidelity/reference-report-gate.json` lists the accepted media reports
  and expected acceptance modes; `make reference-report-gate` currently passes
  for `27` local reports: `20` full visual/audio reports, `4` audio-only
  reports, and `3` visual-only reports. The same manifest now enforces `19`
  semantic coverage requirements for the objective facets named in this audit:
  sprite visuals, player laser visual/audio, reverse orientation, explosion and
  coalescence visuals, terrain blow, gameplay audio families, non-lander audio
  and visuals, playability, rescue/loss, death/respawn, smart bomb,
  hyperspace, and organic non-lander presentation. Each coverage requirement
  now includes a `min_reports` breadth floor matching the current accepted
  proof set for that facet, and the manifest loader rejects duplicate report
  names, duplicate report paths, duplicate per-report coverage tags, and
  duplicate coverage/axis requirement rows. Every manifest report row and local
  accepted `report.json` file must declare an explicit matching
  `acceptance_mode`; each report coverage tag must be non-empty, declared by a
  semantic requirement, and compatible with the report's visual/audio/all
  acceptance mode. Manifest report paths must stay under
  `target/reference-media/`; accepted MAME reference artifacts must stay under
  `target/reference-media/mame/`, and accepted clean candidate artifacts must
  stay under `target/reference-media/clean/`. Accepted visual reference
  artifacts must be MP4 MAME captures, accepted visual candidate artifacts must
  be clean GIF captures, and accepted audio artifacts must be WAV files. The
  gate also verifies the local reference and clean candidate media files
  required by each accepted visual or audio axis; the current signoff summary
  counts `94` required media artifacts and requires each one to be non-empty.
  Accepted visual axes must also report positive reference, candidate, and
  compared frame counts, and accepted audio axes must report positive
  reference, candidate, and compared sample counts. Accepted axes must not
  carry stale verifier `failures` entries.
  A `2026-05-29 15:57 BST` report-inventory triage checked `99` local
  `target/reference-media/**/report.json` files: `27` are accepted by the
  manifest and `72` are unaccepted probe or historical reports. Ten unaccepted
  reports currently have top-level `pass` status, but each is an offset/probe
  duplicate of already accepted fire/reverse, smart-bomb, terrain-blow,
  materialization-matrix, or down030 laser media. None was promoted because it
  would not add a new bounded proof boundary beyond the accepted report set.
  Status: proven locally for the current accepted report and coverage set.
- Owner-review proof summary:
  `make reference-signoff-summary` validates the same accepted report manifest
  and writes `target/reference-media/reference-signoff-summary.md`, a generated
  Markdown matrix of each coverage requirement, accepted reports, visual/audio
  metric summaries, local MAME/clean media paths, manifest and local
  `report.json` acceptance modes, the minimum report count per coverage facet,
  the required non-empty media-artifact count, accepted visual/audio comparison
  counts, and the current all-trace / organic-only reference-window scan
  boundary metrics, including per-family object row counts and the longest
  contiguous object-evidence spans plus the best span for each object family.
  `make reference-evidence-package` regenerates both scan JSON reports before
  writing the summary.
  `make owner-review-package` regenerates that evidence package, re-runs the
  accepted-report gate, and prints this audit's owner-review checklist.
  Status: available as deterministic review evidence for the current accepted
  report set.
- Documentation sync:
  `README.md`, `SPEC.md`, `PLAN.md`, this inventory, and sound docs are linted
  together.
  Status: proven for current docs.
- Non-lander implementation coverage:
  the clean runtime has source-shaped sprite IDs, atlas regions, object
  evidence, movement, projectile, hit-sound, appearance, and
  explosion-lifecycle tests for mutant, swarmer, baiter, bomber, pod, and bomb
  families. A follow-up hardening pass explicitly proves lander, mutant,
  bomber, pod, baiter, and swarmer explosions project as source expanded pixel
  clouds rather than static placeholder sprites, and a later
  materialization/coalescence regression proves those enemy families also use
  source expanded-object appearance pixels instead of static sprites while
  appearing. A `2026-05-29` sound-command audit also pins the clean
  enemy-family mappings to the red-label sound table bytes: hit commands use
  lander `0xF9`, mutant `0xE8`, bomber `0xFE`, pod `0xFA`, swarmer `0xF8`,
  and baiter `0xF8`; shot commands use lander `0xFC`, mutant `0xF6`, swarmer
  `0xF3`, baiter `0xFC`, and no direct shot command for bomber or pod; bomb
  collision remains `0xEE`.
  Status: proven as implementation coverage, but not a substitute for bounded
  MAME-vs-clean media proof for every family.
- Enemy-family visual matrix coverage:
  the MAME and clean capture tools now support `enemy_explosion_matrix`, which
  seeds matching expanded-object explosion slots for `LNDP3`, `SCZP1`,
  `TIEP3`, `PRBP1`, `UFOP3`, and `SWXP1`. The generated matrix report has
  top-level status `pass` with `acceptance_mode=visual`.
  Status: proven as visual media coverage for source expanded-object families;
  audio is intentionally not accepted from this synthetic clip because it does
  not exercise real enemy-kill sound commands.
- Enemy-family isolated sound-command coverage:
  the MAME and clean capture tools now support isolated single-command steers
  for `0xFE`, `0xFA`, `0xF8`, and `0xF3`, and MAME captures now include
  sound-board DAC-write TSVs. The generated reports pass audio gates for
  bomber hit, pod hit, swarmer/baiter hit, and swarmer shot after calibrating
  tonal GWAVE period density for `0xFE` and `0xFA`; each report has top-level
  status `pass` with `acceptance_mode=audio`.
  Status: proven as audio media coverage for the remaining non-lander command
  bytes; visual metrics are intentionally not accepted from these synthetic
  single-command clips.
- Organic non-lander long-run trace inventory:
  the existing `extended_hold_up_7000` MAME debug TSV contains
  post-game/attract rows for `SCZP1`, `UFOP1`, `TIEP1`, `PRBP1`, `SWPIC1`,
  `BMBP1`, and `BXPIC`. The candidate bounded media window is frames
  `5811-7000` from input
  program `none*900;coin*4;none*360;start_one*10;altitude_up*5726`. A
  `2026-05-29` MAME/clean media trial generated the long-window artifacts,
  exposed a static clean `GameOver` handoff defect, then passed after clean
  resumed the normal attract scoring sequence after the post-game residual.
  A follow-up hold-down organic media trial captured frames `4300-4700` from
  input `none*900;coin*4;none*360;start_one*10;altitude_down*5726`; that MAME
  trace contains converted-mutant, baiter, and swarmer-explosion rows, and the
  visual-only report passes with full RMS `28.22` and playfield RMS `7.59`.
  A later PRBP1 pod up-thrust media trial captured frames `6855-7455` from
  input
  `none*900;coin*4;none*360;start_one*10;up,thrust*400;up,thrust,fire*40;up,thrust*8286`;
  that MAME trace contains a long organic pod span, and the all-axis report
  passes with full RMS `37.95`, full MAE `7.68`, and matching MAME-silent
  audio.
  Status: proven as bounded post-game/attract non-lander
  scoring-presentation media coverage plus one additional organic visual
  non-lander-family window and one additional organic all-axis pod window; not
  proof of organic live-gameplay hit/shot waveform breadth.
- Reference-window scan:
  `make reference-window-scan` and `make reference-window-scan-organic` now
  scan generated MAME expected/debug TSVs for target non-lander sound bytes
  near non-lander object evidence and terrain status / `TERBLO` process
  evidence. The reports include nearest sound/object misses, per-family object
  row counts, longest contiguous object-evidence spans, best spans for each
  object family, and explicit terrain process misses, so failed candidate
  searches show the closest known bounded windows instead of only a zero
  candidate count. The targets write separate all-trace and organic-only JSON
  reports so the organic run does not overwrite the all-trace evidence. A
  `2026-05-29` revalidation reran both scans against the current local MAME
  trace corpus after additional organic long-control trace-only searches and
  the bounded PRBP1 pod media capture. The all-trace scan now covers `218`
  expected traces and `214` debug traces, finds `16` target sound hits,
  `276644` object rows, zero sound/object candidates, `152024` terrain status
  rows, `4` `TERBLO` process rows, and `2` last-human terrain-blow candidates.
  The organic-only scan excluding `nonlander-sound-command`,
  `enemy-explosion-matrix`, `enemy-materialize-matrix`, and `state-steered`
  covers `198` expected traces and `194` debug traces, finds zero target sound
  hits, `274834` object rows, zero sound/object candidates, `144341` terrain
  status rows, `2` `TERBLO` process rows, and the same `2` last-human
  terrain-blow candidates. Both candidates are frame `5990`,
  `ASTCNT=0x00`, `pc=0xED88`, `terrain_blown=false`, with a live `TERBLO`
  process in `organic_fire_smartbomb_mix_12000.debug.tsv` and
  `organic-terrain-blow-smartmix.debug.tsv`.
  Status: the scan is now a positive organic terrain-blow evidence source.
  The accepted state-steered terrain-blow report remains green, but the
  organic smartmix probe is still unaccepted. The current clean candidate now
  reaches score `50`, keeps human snapshots empty, arms the destroyed-planet
  state while preserving visible terrain, projects the six-lander/nine-mutant
  residual object formation from the first compared frame, starts visible
  terrain-blow explosions only at the organic post-game `TERBLO` boundary, and
  emits the sampled `0xEE` cadence, but the regenerated all-axis report still
  fails visual placement and audio waveform thresholds.
- Laser/reverse implementation coverage:
  focused tests cover source-style sparse laser sprites, `LASP1` object
  evidence, far-side enemy collision before culling, capped-fire sound
  suppression, and both reverse directions with `PLAYER_SHIP_LEFT`.
  Status: proven as implementation coverage; the accepted MAME-vs-clean media
  reports prove the delayed-start fire/reverse window, delayed-start
  thrust/reverse window, first laser-hit window, first laser-hit centered
  window, the down030 post-death laser all-axis window, and target6 laser-band
  window. The down030 all-axis report now proves the default post-death
  second-life laser `0xEB` at frame `2439`, appearance/materialize `0xEA` at
  frame `2447`, matching visual laser/materialize presentation, and
  source-shaped stochastic audio.

## Passing Media Reports

These reports currently have top-level status `pass`. Unless noted, visual and
audio status are both `pass`; axis-specific synthetic reports use
`acceptance_mode=visual` or `acceptance_mode=audio` so the ignored axis remains
visible without failing the accepted proof. The same list is encoded in
`docs/fidelity/reference-report-gate.json` and checked by
`make reference-report-gate`.

- Scoring laser/explosion:
  `target/reference-media/scoring-laser-explosion-check/report.json`
  proves the attract scoring laser/explosion comparison window.
- Delayed-start fire/reverse:
  `target/reference-media/gameplay-fire-reverse-delayed-check/report.json`
  proves sparse player laser, reverse-facing sprite, materialize audio, and
  laser audio in the delayed-start window.
- Delayed-start thrust:
  `target/reference-media/gameplay-thrust-delayed-check/report.json`
  proves thrust start/stop command timing and the noise-shape gate.
- Delayed-start thrust/reverse:
  `target/reference-media/gameplay-thrust-reverse-delayed-check/report.json`
  proves reverse-facing player orientation during a thrust/reverse window plus
  thrust/background audio after the sound-board noise calibration.
- Delayed-start enemy shot and background:
  `target/reference-media/gameplay-enemy-shot-delayed-check/report.json`,
  `target/reference-media/gameplay-enemy-shot-narrow-check/report.json`, and
  `target/reference-media/gameplay-enemy-shot-pre-window-check/report.json`
  prove the first lander-shot `0xFC` command, the narrow shot waveform window,
  and the pre-shot background/thrust-tail audio window.
- Delayed-start smart bomb:
  `target/reference-media/gameplay-smart-bomb-delayed-check/report.json`
  proves smart-bomb scoring, flash, sound command burst, and cannon tail.
- Hyperspace death:
  `target/reference-media/gameplay-hyperspace-death-probe-check/report.json`
  proves hyperspace rematerialization, death-risk branch, death tail, and
  respawn tail.
- First gameplay laser hit:
  `target/reference-media/gameplay-laser-hit-single-check-window/report.json`
  proves first lander hit timing, laser endpoint, score, hit sound, and enemy
  explosion placement. The additional centered-window report at
  `target/reference-media/gameplay-laser-hit-single-center-check/report.json`
  proves the same first-hit sequence with the tighter playfield/laser-band
  alignment window.
- Down030 post-death laser all-axis window:
  `target/reference-media/laser-hit-down030-fire2437-check-fixed/report.json`
  proves a bounded second-life player laser and materialization window after
  the target2 collision branch, including the MAME-matched `0xEB` laser at
  frame `2439`, `0xEA` appearance/materialize at frame `2447`, visual
  laser/materialize thresholds, and stochastic audio thresholds.
- Non-lander target6:
  `target/reference-media/non-lander-target6-fire2524-check/report.json`
  proves the converted-mutant shot sequence, player/enemy collision tail, and
  `SCZP1` explosion window.
- Lander pickup/pull/loss:
  `target/reference-media/abduction-hold-up-pickup-pull-check/report.json`
  proves both pickup commands, both pull bursts, both conversions, and the
  human-loss/lightning tail.
- Falling-human catch:
  `target/reference-media/afall-player-catch-check/report.json`
  proves player catch/rescue command cadence and the bounded visual/audio
  window.
- Safe landing:
  `target/reference-media/afall-safe-landing-check/report.json`
  proves the safe-landing `0xE0` command and VARI voice window.
- Terrain blow:
  `target/reference-media/terrain-blow-check/report.json`
  proves terrain-blow flash windows, terrain explosion growth, and the
  stochastic-noise audio gate.
- Enemy explosion matrix:
  `target/reference-media/enemy-explosion-matrix-check/report.json`
  proves source expanded-object visual families for lander, converted mutant,
  bomber, pod, baiter, and swarmer. This report has
  `acceptance_mode=visual`; audio is intentionally ignored for the synthetic
  seed window.
- Enemy materialization matrix:
  `target/reference-media/enemy-materialize-matrix-check/report.json`
  proves source expanded-object appearance/coalescence visuals for lander,
  converted mutant, bomber, pod, baiter, and swarmer. This report has
  `acceptance_mode=visual`; audio is intentionally ignored for the synthetic
  appearance-only seed window.
- Isolated non-lander sound commands:
  `target/reference-media/nonlander-sound-command-fe-check/report.json`,
  `target/reference-media/nonlander-sound-command-fa-check/report.json`,
  `target/reference-media/nonlander-sound-command-f8-check/report.json`, and
  `target/reference-media/nonlander-sound-command-f3-check/report.json`
  prove the remaining bomber hit, pod hit, swarmer/baiter hit, and swarmer
  shot audio commands. These reports have `acceptance_mode=audio`; visual
  metrics are intentionally ignored for the synthetic single-command windows.
- Organic non-lander long-window:
  `target/reference-media/organic-nonlander-holdup-7000-check/report.json`
  proves the bounded post-game/attract non-lander scoring-presentation window
  after normal play hands back into the attract scoring cycle.
- Organic non-lander hold-down visual window:
  `target/reference-media/organic-nonlander-holddown-7000-check/report.json`
  proves a bounded organic converted-mutant, baiter, and swarmer-explosion
  visual window. This report has `acceptance_mode=visual`; audio is retained
  as a failing diagnostic because this window does not exercise the remaining
  non-lander-specific command bytes.
- Organic PRBP1 pod up-thrust all-axis window:
  `target/reference-media/organic-nonlander-prbp1-upthrust-check/report.json`
  proves a bounded organic pod visual window selected from the per-family
  object-span scan plus the matching post-game audio-silence boundary. This
  report has `acceptance_mode=all`; both visual and audio axes pass.

## Coverage Boundaries

The current evidence proves the accepted clips above. It does not prove every
possible arcade state. These boundaries prevent marking the active goal
complete without owner review or more MAME evidence:

- Additional gameplay laser and hit endpoint windows outside the current
  first-lander, first-lander centered, down030 all-axis, and target6 cases.
  The `2026-05-29` down030 MAME-vs-clean cycle closed the previously observed
  default post-death restart timing defect, so remaining laser work is proof
  breadth unless a new concrete MAME mismatch is found.
- Additional reverse-facing player windows outside the delayed-start
  fire/reverse and thrust/reverse passes. A `2026-05-29` audit rechecked both
  reverse-direction tests and the accepted delayed-start media reports; no
  current implementation defect was found, so the remaining gap is extra media
  breadth.
- Enemy explosion families outside scoring, first-lander hit, player death,
  target6 `SCZP1`, terrain-blow windows, and the state-steered visual matrix.
- Mutant, swarmer, baiter, bomber, pod, and other non-lander shot variants not
  yet represented by organic bounded gameplay media reports. A `2026-05-29`
  source audit found
  no placeholder implementation for these families: sprites, source picture
  descriptors, source movement loops, enemy projectile rows, hit sounds, and
  source explosion descriptors are covered by focused Rust tests. A follow-up
  `2026-05-29` hardening cycle added explicit source-pixel-cloud regression
  coverage for lander, mutant, bomber, pod, baiter, and swarmer explosions.
  Another `2026-05-29` cycle added a state-steered visual matrix report for
  the same source descriptor families. A sound-command audit then added a
  direct regression for the red-label hit/shot/bomb command bytes, and an
  isolated MAME-vs-clean audio cycle added passing single-command reports for
  the remaining `0xFE`, `0xFA`, `0xF8`, and `0xF3` command families. A long-run
  MAME trace inventory identified frames `5811-7000` of `extended_hold_up_7000`
  as the next bounded organic media candidate for live baiter, bomber, pod,
  swarmer, mutant, bomb-shell, and bomb-explosion presentation. The generated
  media comparison now passes after repairing the clean post-game attract
  handoff. A second organic hold-down window now passes visual-only acceptance
  for converted mutant, baiter, and swarmer explosion rows. A third bounded
  organic up-thrust window now passes all-axis acceptance for PRBP1 pod
  presentation and the MAME-silent post-game audio boundary selected from the
  per-family object-span scan. The repeatable reference-window scan found no
  current organic trace with remaining target non-lander sound bytes near
  non-lander object evidence. The remaining gap is organic live-gameplay
  hit/shot waveform breadth and owner review, not an identified missing sprite
  implementation.
- Pixel coalescence/materialization placement media breadth for enemy families
  outside the current first-wave lander and target6 converted-mutant evidence.
  Clean implementation coverage now proves family-wide source appearance pixel
  clouds for lander, mutant, bomber, pod, baiter, and swarmer; the remaining
  gap is bounded MAME media breadth, not a known static-sprite implementation
  path.
- Organic last-human terrain-blow gameplay. The state-steered terrain blow
  proves the routine window, and the latest scan now has organic evidence for
  the last-human `TERBLO` branch. The bounded organic smartmix report at
  `target/reference-media/organic-terrain-blow-smartmix-check/report.json`
  still fails all-axis acceptance. The stale clean state mismatch is repaired:
  current clean reaches score `50`, enters terrain blow, keeps clean human
  snapshots empty once the destroyed-planet branch is active, preserves the
  visible terrain while the source `TERBLO` process is armed, starts visible
  `TEREX` terrain explosions at post-game frame `1044`, projects the
  MAME-observed six-lander/nine-mutant residual object mix from state frame
  `5960` through the sampled terminal rows, and emits the sampled `0xEE` tail.
  The remaining implementation gap is all-axis media fidelity for that organic
  branch: current regenerated metrics are visual RMS `90.85` / MAE `42.72`
  against thresholds `38.00` / `28.00`, and audio still fails waveform
  correlation despite nonzero clean terrain-blow audio
  (`normalized_diff_rms=1.726`, `correlation=-0.005`,
  `envelope_correlation=0.137`).
- Owner review of graphics, audio, and playability. `PLAN.md` still requires
  owner signoff before protected reference media replacement and final closure.

## Owner Review Checklist

The release gate and accepted reports are green, but final closure still
requires owner review of the bounded proof set plus regeneration and review of
the organic smartmix terrain-blow report. Use this checklist for that review:

- Review the current accepted media report set in
  `docs/fidelity/reference-report-gate.json`; it is the bounded MAME-vs-clean
  evidence set for sprite visuals, lasers, explosions, coalescence, audio
  families, controls, rescue/loss, death/respawn, smart bomb, hyperspace, and
  non-lander coverage.
- Generate and review `target/reference-media/reference-signoff-summary.md`
  with `make owner-review-package`; it is the deterministic coverage and
  metric matrix derived from the accepted manifest, local report JSON files,
  and freshly regenerated reference-window scan JSON files. The same target
  prints this checklist so review is anchored to the latest local evidence.
- Run `make release-gate` before final closure so the full local validation
  path, fresh evidence package, accepted report gate, smoke checks, docs lint,
  short MAME recorder smoke, and diff hygiene are exercised from one
  reproducible target.
- Review the generated local media under `target/reference-media/`, especially
  the delayed-start fire/reverse, delayed-start thrust/reverse, first
  laser-hit, centered first laser-hit, non-lander target6, pickup/pull,
  catch, safe-landing, terrain-blow, enemy-explosion matrix,
  enemy-materialization matrix, live laser-hit materialization, isolated
  non-lander sound-command, and organic non-lander reports.
- Confirm that the remaining proof boundaries below are acceptable for release,
  or provide a new concrete MAME clip/input program showing a mismatch outside
  the accepted windows.
- If accepted, record owner approval in this audit with the date, reviewer, and
  the accepted report gate count. Only then should protected reference media be
  replaced or the active goal be marked complete.

## Current Conclusion

The implementation is release-gate green and all accepted bounded media targets
pass, including the down030 post-death all-axis laser/materialize window. The
enemy materialization matrix now also passes as a bounded visual proof for
source coalescence across lander, mutant, bomber, pod, baiter, and swarmer
families. The active goal is not yet proven complete because proof boundaries
and owner review remain. New work should either address another coverage
boundary with a bounded MAME-vs-clean clip or record owner acceptance that the
current accepted clips are sufficient for release closure.
