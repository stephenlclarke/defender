# Completed Plan Step Archive

This file preserves detailed completed-step history moved out of `PLAN.md` so
that the active plan can stay focused on current R9 execution. Do not add new
active work here. Add new active-cycle notes to `PLAN.md`, and move them here
only after they are closed and no longer needed for day-to-day planning.

## DC-164 Completed Steps 1-40

- `2026-05-16 21:58:34 BST` Started `DC-164` and Step 1
  `final acceptance audit`. This step owns checking the current production
  runtime surface, remaining gap docs, final acceptance docs, and targeted
  baseline commands before any R9 closeout edits. Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778965123301899`.
- `2026-05-16 22:06:05 BST` Completed Step 1 and started Step 2
  `final gate hardening and blocker record`. The expanded clean-fidelity audit
  matched all 12 Phase 1 scenarios under the current milestone profiles:
  `attract_boot`, `start_game`, `first_300_frames`, `firing`,
  `thrust_reverse`, `smart_bomb`, `hyperspace`, `abduction`, `death`,
  `wave_advance`, `planet_destruction`, and `high_score_entry`. Strict R9 final
  acceptance remains blocked because the clean production game still has reduced
  enemy ecology/session surfaces, and the accepted adapter does not expose full
  world, object-detail, game-over sleep, high-score qualification,
  initials-entry, or submitted-table evidence for comparison. Step 2 hardens the
  default `make clean-fidelity` gate to all 12 Phase 1 scenarios and records the
  remaining R9 blockers explicitly. Slack step update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778965577218809`.
- `2026-05-16 22:13:49 BST` Completed Step 2. `make clean-fidelity` now
  defaults to all 12 embedded Phase 1 scenarios, and the ignored clean-fidelity
  developer gate uses the same default scenario set. README, SPEC,
  `docs/fidelity/gaps.md`, and this plan now document the expanded gate plus the
  remaining strict R9 blockers. Focused validation passed: `cargo fmt --check`,
  `cargo test --lib --features legacy-tools clean_fidelity::tests --
  --nocapture`, `make clean-fidelity` across all 12 Phase 1 scenarios,
  `markdownlint README.md SPEC.md PLAN.md docs/fidelity/gaps.md
  docs/fidelity/live-audio.md assets/arcade/README.md assets/sprites/README.md
  assets/sounds/README.md`, and `git diff --check`. `DC-164` remains open
  because R9 still needs source-backed world/object and high-score/session
  comparison surfaces, or an owner decision to redefine the final acceptance
  contract. Slack step completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778966041547069`.
- `2026-05-16 22:14:41 BST` Started Step 3
  `accepted-surface blocker slice`. This step owns inspecting the accepted
  facade/oracle path and source-backed trace evidence for the first narrow R9
  blocker that can be converted into comparison data. The preferred outcome is
  one vertical accepted-surface slice that tightens clean-fidelity for a targeted
  scenario; if local evidence is insufficient, the step must record the precise
  fixture/source blocker instead of guessing arcade behavior. Slack step start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778966096555009`.
- `2026-05-16 22:23:47 BST` Completed Step 3. Added neutral accepted
  high-score initials state to the accepted facade and preserved active
  source-machine initials from `MachineSnapshot::high_score_entry` through
  `AcceptedSnapshot` into the clean oracle `HighScoreInitialsState`. Updated
  `docs/fidelity/gaps.md` so the R6/R9 blocker is narrower: active initials
  are now exposed, while game-over sleep, high-score qualification,
  submission-table insertion, post-entry return evidence, and broader
  world/object details remain missing comparison surfaces. Validation passed:
  `cargo fmt --check`, `cargo test --lib --features legacy-tools accepted --
  --nocapture`, `cargo test --lib --features legacy-tools oracle --
  --nocapture`, `cargo test --lib --features legacy-tools clean_fidelity::tests
  -- --nocapture`, `cargo test --lib public_api_tests -- --nocapture`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  `make clean-fidelity` across all 12 Phase 1 scenarios, `markdownlint
  README.md SPEC.md PLAN.md docs/fidelity/gaps.md docs/fidelity/live-audio.md
  assets/arcade/README.md assets/sprites/README.md assets/sounds/README.md`,
  and `git diff --check`. `DC-164` remains open because R9 still requires
  source-backed world/object and high-score submission/session comparison
  surfaces, or an owner decision to redefine final acceptance. Slack step
  completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778966640680699`.
- `2026-05-16 22:26:21 BST` Started Step 4
  `high-score entry/session accepted-surface slice`. This step owns exposing
  source-backed high-score entry metadata beyond initials through the accepted
  facade and oracle, then comparing the matching clean state where the clean
  runtime has an owned contract. Validation will stay focused during
  implementation, then run the current all-scenario `make clean-fidelity` gate
  before this step closes. Slack step start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778966792201819`.
- `2026-05-16 22:39:26 BST` Completed Step 4. Added clean high-score
  entry/session state for active entry score/rank and submitted player/score,
  threaded the same source-backed metadata through the accepted facade, accepted
  adapter, oracle adapter, and clean-fidelity comparator, and updated docs so
  the remaining R9 high-score blocker is now submitted-table insertion and
  post-entry return evidence rather than active entry/session metadata.
  Focused validation passed: `cargo fmt --check`,
  `cargo test --lib --features legacy-tools accepted -- --nocapture`,
  `cargo test --lib --features legacy-tools oracle -- --nocapture`,
  `cargo test --lib --features legacy-tools clean_fidelity::tests --
  --nocapture`, `cargo test --lib game::tests::clean_game_high_score --
  --nocapture`, `cargo test --lib public_api_tests -- --nocapture`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  `make clean-fidelity SCENARIOS="high_score_entry"`, `make clean-fidelity`
  across all 12 Phase 1 scenarios, `markdownlint README.md SPEC.md PLAN.md
  docs/fidelity/gaps.md docs/fidelity/live-audio.md
  assets/arcade/README.md assets/sprites/README.md assets/sounds/README.md`,
  and `git diff --check`. `DC-164` remains open because strict R9 final
  acceptance still needs source-backed world/object surfaces and submitted
  high-score table/post-entry return evidence, or an owner decision to redefine
  the final acceptance contract. Slack step completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778967580445759`.
- `2026-05-16 22:40:45 BST` Started Step 5
  `submitted high-score table accepted-surface slice`. This step owns
  inspecting the source-backed high-score table APIs and exposing a neutral
  all-time/today table snapshot through the accepted facade and oracle only
  where the clean runtime owns matching state. Validation will start with
  focused high-score table tests and close with the current all-scenario
  `make clean-fidelity` gate. Slack step start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778967677260359`.
- `2026-05-16 22:58:54 BST` Completed Step 5. Added source-shaped
  high-score table snapshots for the all-time and today's-greatest eight-row
  tables, updated clean high-score submission to insert completed initials into
  both tables and refresh the visible high score from the all-time table top
  row, and threaded the same source-owned table snapshot through
  `MachineSnapshot`, accepted facade, oracle adapter, and clean-fidelity
  comparison. Updated docs so submitted table insertion is no longer the R9
  high-score blocker; remaining strict blockers are source-backed world/object
  surfaces plus game-over sleep and post-entry return timing/display evidence.
  Validation passed: `cargo fmt --check`,
  `cargo test --lib game::tests::clean_game_high_score -- --nocapture`,
  `cargo test --lib systems::tests::high_score_entry_system -- --nocapture`,
  `cargo test --lib --features legacy-tools accepted -- --nocapture`,
  `cargo test --lib --features legacy-tools oracle -- --nocapture`,
  `cargo test --lib --features legacy-tools clean_fidelity::tests --
  --nocapture`, `make clean-fidelity SCENARIOS="high_score_entry"`,
  `cargo test --lib public_api_tests -- --nocapture`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  `make clean-fidelity` across all 12 Phase 1 scenarios, `markdownlint
  README.md SPEC.md PLAN.md docs/fidelity/gaps.md docs/fidelity/live-audio.md
  assets/arcade/README.md assets/sprites/README.md assets/sounds/README.md`,
  and `git diff --check`. `DC-164` remains open because strict R9 final
  acceptance still needs source-backed world/object surfaces and game-over
  sleep/post-entry return evidence, or an owner decision to redefine the final
  acceptance contract. Slack step completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778968747587059`.
- `2026-05-16 23:32:02 BST` Started Step 6
  `game-over sleep and post-entry return timing/display surface`. This step
  owns adding a neutral clean/accepted game-over return state for the
  source-backed 40-tick player-death game-over sleep, 0xFF no-entry delay, and
  60-tick hall-of-fame display stall after submitted initials or no-entry
  handoff. Validation will start with focused game, accepted, oracle, and
  clean-fidelity tests, then close with the current all-scenario
  `make clean-fidelity` gate. Slack step start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778970829472739`.
- `2026-05-16 23:38:13 BST` Continued Step 6 after a context checkpoint.
  The slice is still scoped to wiring source-backed game-over return timing
  through clean state, accepted snapshots, the oracle adapter, and
  clean-fidelity comparison before any Step 6 closeout. Slack continuation
  checkpoint:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778971087634949`.
- `2026-05-16 23:55:35 BST` Completed Step 6. Added clean/source/accepted
  game-over return timing for the source-backed 40-tick player-death game-over
  sleep, 0xFF no-entry `HALL13` delay, and 60-tick hall-of-fame display stall
  after submitted initials or no-entry handoff. `state.game_over` now flows
  from `MachineSnapshot` through the accepted facade and oracle into
  clean-fidelity comparison, with unrelated generic attract hall-of-fame
  rotation timers normalized unless the clean runtime is also in a game-over
  return display. Updated `README.md`, `SPEC.md`, and
  `docs/fidelity/gaps.md` so the high-score/session blocker is retired.
  Validation passed: `cargo fmt --check`, `cargo test --lib
  game::tests::clean_game_high_score -- --nocapture`, `cargo test --lib
  game::tests::clean_game_player_enemy -- --nocapture`, `cargo test --lib
  game::tests::clean_game_non_qualifying_game_over_waits_then_shows_hall_of_fame
  -- --nocapture`, `cargo test --lib --features legacy-tools accepted --
  --nocapture`, `cargo test --lib --features legacy-tools oracle --
  --nocapture`, `cargo test --lib --features legacy-tools
  clean_fidelity::tests -- --nocapture`, `cargo test --lib --features
  legacy-tools translated_death_game_over -- --nocapture`, `make
  clean-fidelity SCENARIOS="death high_score_entry"`, `cargo test --lib
  public_api_tests -- --nocapture`, `cargo test --lib
  systems::tests::high_score_entry_system -- --nocapture`, `cargo clippy
  --all-targets --features legacy-tools -- -D warnings`, and full
  all-12-scenario `make clean-fidelity`. `DC-164` remains open because strict
  R9 final acceptance still needs source-backed world/object detail surfaces,
  or an owner decision to redefine the final acceptance contract. Slack step
  completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778972150714969`.
- `2026-05-16 23:59:12 BST` Started Step 7
  `source-backed wave/enemy profile surface`. This step owns adding a neutral
  clean/accepted wave profile state from `assets/red-label/wave-table.tsv` and
  `MachineSnapshot::wave_profile`, then comparing source-owned enemy counts and
  wave timing/profile fields without guessing live object positions or sprite
  details. Validation will start with focused clean, accepted, oracle, and
  clean-fidelity tests, then run targeted start/wave/long scenarios before the
  current all-scenario `make clean-fidelity` gate. Slack step start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778972362725739`.
- `2026-05-17 00:19:52 BST` Completed Step 7. Added clean
  `WaveProfileSnapshot` state derived from `assets/red-label/wave-table.tsv`,
  mapped the accepted `MachineSnapshot::wave_profile` through the accepted
  facade and oracle, and compared `state.wave_profile` in clean-fidelity when
  the profiled wave state is aligned. The comparator deliberately skips
  `state.wave_profile` when an existing profile already ignores wave-number
  drift, as in the `smart_bomb` player-control slice. Updated `README.md`,
  `SPEC.md`, and `docs/fidelity/gaps.md` so source-owned enemy counts, wave
  timing, velocities, shot timers, and baiter timing are no longer hidden
  accepted-surface gaps. Validation passed: `cargo fmt --check`, `cargo test
  --lib game::tests::wave_profile_uses_source_wave_table_values --
  --nocapture`, `cargo test --lib
  game::tests::clean_game_starts_from_domain_state -- --nocapture`, `cargo
  test --lib --features legacy-tools accepted -- --nocapture`, `cargo test
  --lib --features legacy-tools oracle -- --nocapture`, `cargo test --lib
  --features legacy-tools clean_fidelity::tests -- --nocapture`, `cargo test
  --lib public_api_tests -- --nocapture`, `cargo clippy --all-targets
  --features legacy-tools -- -D warnings`, `make clean-fidelity
  SCENARIOS="start_game first_300_frames wave_advance planet_destruction"`,
  `make clean-fidelity SCENARIOS="smart_bomb start_game wave_advance
  planet_destruction"`, and full all-12-scenario `make clean-fidelity`.
  `DC-164` remains open because strict R9 final acceptance still needs live
  object positions, object/sprite identities, and broader visual presentation
  surfaces, or an owner decision to redefine the final acceptance contract.
  Slack step completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778973605058799`.
- `2026-05-17 00:22:37 BST` Started Step 8
  `source object-list evidence surface`. This step owns exposing the first
  neutral source object-list summary through `MachineSnapshot`, the accepted
  facade, the oracle adapter, and clean-fidelity without guessing live object
  positions or sprite identities. The intended surface is active/inactive list
  counts, visible object count, and stable object-table evidence where current
  source notes and accepted traces support it. Validation will start with
  focused accepted, oracle, and clean-fidelity checks, then close with the
  current all-scenario `make clean-fidelity` gate. Slack step start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778973788603179`.
- `2026-05-17 00:43:40 BST` Completed Step 8. Added clean
  `ObjectEvidenceSnapshot` on `WorldSnapshot`, derived source object active,
  inactive, projectile-list, visible-active counts plus stable object-data
  evidence CRC from the accepted machine, and threaded that evidence through
  `MachineSnapshot`, the accepted facade, the oracle adapter, and the
  strict/full clean-fidelity comparison as `state.world.object_evidence`. Clean
  world snapshots refresh the clean-owned counts after projectile/enemy/human
  mutations so the evidence field stays synchronized during gameplay. Current
  profiled R9 scenarios remain stable; this slice intentionally does not guess
  live object positions, source object/sprite identities, object lifecycle
  details, or render presentation. Updated `README.md`, `SPEC.md`, and
  `docs/fidelity/gaps.md` so object-list evidence is no longer an empty
  accepted-world placeholder. Validation passed: `cargo fmt --check`, `cargo
  test --lib game::tests -- --nocapture`, `cargo test --lib --features
  legacy-tools accepted -- --nocapture`, `cargo test --lib --features
  legacy-tools oracle -- --nocapture`, `cargo test --lib --features
  legacy-tools clean_fidelity::tests -- --nocapture`, `cargo test --lib
  public_api_tests -- --nocapture`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  `markdownlint README.md SPEC.md PLAN.md docs/fidelity/gaps.md
  docs/fidelity/live-audio.md assets/arcade/README.md assets/sprites/README.md
  assets/sounds/README.md`, `git diff --check`, and full all-12-scenario
  `make clean-fidelity`. `DC-164` remains open because strict R9 final
  acceptance still needs live object positions, source object/sprite identity
  mapping, object lifecycle/physics transitions, and broader visual
  presentation fidelity, or an owner decision to redefine the final acceptance
  contract. Slack step completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778974521997279`.
- `2026-05-17 00:45:07 BST` Started Step 9
  `source object-position evidence surface`. This step owns extending the
  neutral object evidence with bounded source object details from the
  red-label object table: list membership, object address, screen position,
  world position, velocity, picture address, and type byte where available.
  The evidence will flow through `MachineSnapshot`, the accepted facade, the
  oracle adapter, and clean-fidelity. Clean-owned detail may be derived from
  current clean enemies, humans, and projectiles, but this step must not guess
  `OPICT`/`OTYP` identity mapping, full lifecycle semantics, or sprite render
  fidelity. Validation will start with focused game, accepted, oracle, and
  clean-fidelity checks, then close with the current all-scenario
  `make clean-fidelity` gate. Slack step start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778975101364659`.
- `2026-05-17 00:58:03 BST` Completed Step 9. Extended clean
  `ObjectEvidenceSnapshot` with bounded object detail rows, derived clean-owned
  details from enemies, humans, and projectiles, and threaded source object
  list details through `MachineSnapshot`, the accepted facade, the oracle
  adapter, and strict/full clean-fidelity comparison. Source entries now carry
  list membership, object address/slot, screen position, raw world position,
  raw velocity, picture address, and type byte for the first bounded active,
  inactive, and projectile object-list cells. This exposes source object
  position/raw-table detail without assigning arcade identity to `OPICT` or
  `OTYP`, guessing lifecycle semantics, or promoting sprite render fidelity.
  Updated `README.md`, `SPEC.md`, and `docs/fidelity/gaps.md` so
  object-position evidence is no longer hidden behind count-only object-list
  evidence. Validation passed: `cargo fmt --check`, `cargo test --lib
  game::tests -- --nocapture`, `cargo test --lib --features legacy-tools
  accepted -- --nocapture`, `cargo test --lib --features legacy-tools oracle
  -- --nocapture`, `cargo test --lib --features legacy-tools
  clean_fidelity::tests -- --nocapture`, `cargo test --lib public_api_tests
  -- --nocapture`, `cargo clippy --all-targets --features legacy-tools --
  -D warnings`, `markdownlint README.md SPEC.md PLAN.md docs/fidelity/gaps.md
  docs/fidelity/live-audio.md assets/arcade/README.md assets/sprites/README.md
  assets/sounds/README.md`, `git diff --check`, and full all-12-scenario
  `make clean-fidelity`. `DC-164` remains open because strict R9 final
  acceptance still needs source object/sprite identity mapping,
  lifecycle/physics transitions, and broader visual presentation fidelity, or
  an owner decision to redefine the final acceptance contract. Slack step
  completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778975910138609`.
- `2026-05-17 00:59:30 BST` Started Step 10
  `source object-picture identity evidence surface`. This step owns extending
  the bounded object-detail evidence with red-label object-picture metadata
  where the source `OPICT` value resolves to a known object-picture descriptor:
  label, descriptor size, and image descriptor addresses. Clean-owned detail may
  add explicit clean-domain object categories for enemies, humans, and
  projectiles, but this step must not guess arcade `OPICT`/`OTYP` mappings,
  lifecycle semantics, or sprite render presentation. The evidence will flow
  through `MachineSnapshot`, the accepted facade, the oracle adapter, and
  strict/full clean-fidelity comparison. Validation will start with focused
  game, accepted, oracle, and clean-fidelity checks, then close with the current
  all-scenario `make clean-fidelity` gate. Slack step start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778975966345019`.
- `2026-05-17 01:12:05 BST` Completed Step 10. Extended bounded object-detail
  evidence with source object-picture descriptor metadata: red-label picture
  label, descriptor size, primary image address, and alternate image address
  whenever a source `OPICT` word resolves to
  `assets/red-label/object-pictures.tsv`. Threaded the source metadata through
  `MachineSnapshot`, the accepted facade, the oracle adapter, and strict/full
  clean-fidelity comparison. Clean `WorldSnapshot` details now carry explicit
  clean-domain categories for landers, humans, and player projectiles without
  guessing arcade `OPICT`/`OTYP` mappings. Updated `README.md`, `SPEC.md`, and
  `docs/fidelity/gaps.md` so object-picture descriptor evidence is documented
  as a source-backed comparison surface. Validation passed: `cargo fmt
  --check`, `cargo test --lib game::tests -- --nocapture`, `cargo test --lib
  --features legacy-tools accepted -- --nocapture`, `cargo test --lib
  --features legacy-tools oracle -- --nocapture`, `cargo test --lib --features
  legacy-tools clean_fidelity::tests -- --nocapture`, `cargo test --lib
  public_api_tests -- --nocapture`, `cargo clippy --all-targets --features
  legacy-tools -- -D warnings`, `markdownlint README.md SPEC.md PLAN.md
  docs/fidelity/gaps.md docs/fidelity/live-audio.md assets/arcade/README.md
  assets/sprites/README.md assets/sounds/README.md`, `git diff --check`, and
  full all-12-scenario `make clean-fidelity` with `high_score_entry` matching
  3428/3428 frames and total runtime 349.00s. `DC-164` remains open because
  strict R9 final acceptance still needs clean-to-arcade sprite mapping,
  lifecycle/physics transitions, and broader visual presentation fidelity, or
  an owner decision to redefine the final acceptance contract. Slack step
  completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778976757876919`.
- `2026-05-17 01:15:11 BST` Started Step 11
  `source picture-to-clean sprite bridge evidence`. This step owns adding a
  bounded, source-backed bridge from red-label object-picture labels to clean
  `SpriteId` values only where the runtime already has reclassified sprite
  assets: player ship, lander, humanoid, and player laser/projectile. The
  bridge will be threaded into object-detail evidence so source rows can expose
  the clean sprite target when the picture label is explicit, and clean rows can
  expose the sprite rendered from their clean-domain category. This step must
  not invent mappings for probes, swarmers, bombers, pods, baiters, explosions,
  mines, or lifecycle behavior. Validation will start with focused renderer,
  game, oracle, and clean-fidelity checks, then close with the current
  all-scenario `make clean-fidelity` gate. Slack step start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778976923691419`.
- `2026-05-17 01:26:02 BST` Completed Step 11. Added a clean-boundary
  object-picture label bridge in the renderer that maps only the currently
  reclassified sprite asset subset: `PLAPIC`/`PLBPIC` to `PLAYER_SHIP`,
  `LNDP1`-`LNDP3` to `ENEMY_LANDER`, `ASTP1`-`ASTP4` to `HUMAN`, and `LASP1`
  to `PLAYER_PROJECTILE`. Threaded the mapped clean sprite through
  clean-owned object-detail evidence, source `ObjectListDetailState`, the
  accepted facade, and the oracle adapter without exposing legacy terminology
  in clean production modules. Source rows now carry mapped sprite evidence
  only when the picture label is in that explicit set; clean rows carry the
  sprite rendered from their domain category. Updated `README.md`, `SPEC.md`,
  `docs/fidelity/gaps.md`, `assets/arcade/README.md`, and
  `assets/sprites/README.md` to record the bridge and the remaining unmapped
  prototype assets. Validation passed: `cargo fmt --check`, `cargo test --lib
  renderer::tests::object_picture_labels_map_only_reclassified_clean_sprite_assets
  -- --nocapture`, `cargo test --lib game::tests -- --nocapture`, `cargo test
  --lib --features legacy-tools accepted -- --nocapture`, `cargo test --lib
  --features legacy-tools oracle -- --nocapture`, `cargo test --lib --features
  legacy-tools clean_fidelity::tests -- --nocapture`, `cargo test --lib
  public_api_tests -- --nocapture`, `cargo clippy --all-targets --features
  legacy-tools -- -D warnings`, `markdownlint README.md SPEC.md PLAN.md
  docs/fidelity/gaps.md docs/fidelity/live-audio.md assets/arcade/README.md
  assets/sprites/README.md assets/sounds/README.md`, `git diff --check`, and
  full all-12-scenario `make clean-fidelity` with `high_score_entry` matching
  3428/3428 frames and total runtime 349.85s. `DC-164` remains open because
  strict R9 final acceptance still needs mappings and render paths for
  unreclassified enemy/explosion/score/miniplayer/smart-bomb sprites,
  lifecycle/physics transitions, and broader visual presentation fidelity, or
  an owner decision to redefine the final acceptance contract. Slack step
  completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778977594355449`.
- `2026-05-17 01:28:20 BST` Started Step 12
  `source enemy-picture sprite bridge expansion`. This step owns extending the
  clean object-picture-to-`SpriteId` bridge for the remaining source enemy
  pictures where existing prototype PNGs already exist under `assets/sprites/`
  and source notes identify the enemy family. The intended bridge covers
  `SCZP1` mutant/schizoid, `UFOP1`-`UFOP3` baiter/UFO, `TIEP1`-`TIEP4`
  bomber/tie, `PRBP1` probe/pod, and `SWPIC1` swarmer. This step may add
  atlas-backed clean sprite IDs and thread mapped sprite evidence through the
  existing source object-detail path, but must not add gameplay lifecycle,
  spawning, physics, bombs, explosions, score popups, miniplayer, or smart-bomb
  render behavior. Validation will start with focused renderer, accepted,
  oracle, clean-fidelity, and public API checks, then close with the current
  all-scenario `make clean-fidelity` gate. Slack step start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778977713229109`.
- `2026-05-17 01:40:11 BST` Completed Step 12. Added clean `SpriteId` values,
  default atlas regions, embedded PNG blits, and renderer tests for the existing
  enemy-family prototype sprites under `assets/sprites/`: `mutant1.png`,
  `baiter1.png`, `bomber1.png`, `pod1.png`, and `swarmer1.png`. Extended the
  bounded object-picture label bridge so `SCZP1` maps to `ENEMY_MUTANT`,
  `UFOP1`-`UFOP3` to `ENEMY_BAITER`, `TIEP1`-`TIEP4` to `ENEMY_BOMBER`,
  `PRBP1` to `ENEMY_POD`, and `SWPIC1` to `ENEMY_SWARMER`, with accepted/oracle
  coverage confirming mapped sprite evidence reaches the strict comparison
  surface. Updated `README.md`, `SPEC.md`, `docs/fidelity/gaps.md`,
  `assets/arcade/README.md`, and `assets/sprites/README.md` to record the
  provenance boundary. Validation passed: `cargo fmt --check`, focused renderer
  mapping/atlas tests, `cargo test --lib game::tests -- --nocapture`, `cargo
  test --lib public_api_tests -- --nocapture`, `cargo test --lib --features
  legacy-tools accepted -- --nocapture`, `cargo test --lib --features
  legacy-tools oracle -- --nocapture`, `cargo test --lib --features
  legacy-tools clean_fidelity::tests -- --nocapture`, `cargo clippy
  --all-targets --features legacy-tools -- -D warnings`, `markdownlint README.md
  SPEC.md PLAN.md docs/fidelity/gaps.md docs/fidelity/live-audio.md
  assets/arcade/README.md assets/sprites/README.md assets/sounds/README.md`,
  `git diff --check`, and full all-12-scenario `make clean-fidelity` with
  `high_score_entry` matching 3428/3428 frames and total runtime 348.85s.
  `DC-164` remains open because strict R9 final acceptance still needs bomb,
  explosion, score-popup, miniplayer, smart-bomb, lifecycle, physics, spawn, and
  broader render-presentation parity work, or an owner decision to redefine the
  final acceptance contract. Slack step completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778978411529699`.
- `2026-05-17 01:42:32 BST` Started Step 13
  `source display/reward picture sprite bridge expansion`. This step owns
  extending the clean object-picture-to-`SpriteId` bridge for remaining
  non-enemy source picture labels where existing prototype PNGs already live
  under `assets/sprites/` and source notes identify the picture family: bomb
  shell pictures `BMBP1`/`BMBP2`, bounded explosion pictures `BXPIC` and
  `SWXP1`, score popup pictures `C25P1`/`C5P1`, stock display picture
  `PLAMIN`, and smart-bomb display picture `SBPIC`. This step may add
  atlas-backed clean sprite IDs and evidence tests, but must not add gameplay
  scoring behavior, bomb lifecycle/spawning/physics, explosion timing,
  terrain-blow presentation, stock-count drawing behavior, or score-popup
  lifecycle behavior. Validation will start with focused renderer, accepted,
  oracle, clean-fidelity, and public API checks, then close with the current
  all-scenario `make clean-fidelity` gate. Slack step start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778978575474909`.
- `2026-05-17 01:52:21 BST` Completed Step 13. Added clean `SpriteId` values,
  default atlas regions, embedded PNG blits, and renderer tests for the bounded
  display/reward picture set already present under `assets/sprites/`: `bomb1.png`,
  `podexpl.png`, `swarmexpl.png`, `score250_1.png`, `score500_1.png`,
  `littleship.png`, and `smartbomb.png`. Extended the object-picture label
  bridge so `BMBP1`/`BMBP2` map to `ENEMY_BOMB`, `BXPIC` to
  `BOMB_EXPLOSION`, `SWXP1` to `SWARMER_EXPLOSION`, `C25P1` to
  `SCORE_POPUP_250`, `C5P1` to `SCORE_POPUP_500`, `PLAMIN` to
  `PLAYER_LIFE_STOCK`, and `SBPIC` to `SMART_BOMB_STOCK`. Expanded the oracle
  object-evidence fixture with mapped bomb-shell evidence so the accepted
  adapter path still exercises mapped sprite evidence at the strict comparison
  boundary. Updated `README.md`, `SPEC.md`, `docs/fidelity/gaps.md`,
  `assets/arcade/README.md`, and `assets/sprites/README.md` to record the
  provenance boundary. Validation passed: `cargo fmt --check`, focused renderer
  mapping/atlas tests, focused oracle object-evidence test, `cargo test --lib
  game::tests -- --nocapture`, `cargo test --lib public_api_tests --
  --nocapture`, `cargo test --lib --features legacy-tools accepted --
  --nocapture`, `cargo test --lib --features legacy-tools oracle --
  --nocapture`, `cargo test --lib --features legacy-tools
  clean_fidelity::tests -- --nocapture`, `cargo clippy --all-targets
  --features legacy-tools -- -D warnings`, `markdownlint README.md SPEC.md
  PLAN.md docs/fidelity/gaps.md docs/fidelity/live-audio.md
  assets/arcade/README.md assets/sprites/README.md assets/sounds/README.md`,
  `git diff --check`, and full all-12-scenario `make clean-fidelity` with
  `high_score_entry` matching 3428/3428 frames and total runtime 349.04s.
  `DC-164` remains open because strict R9 final acceptance still needs `ASXP1`,
  `NULOB`, `TEREX`, real bomb lifecycle/spawn/physics, explosion timing,
  score-popup lifecycle, stock-count drawing behavior, terrain-blow
  presentation, and broader render-presentation parity work, or an owner
  decision to redefine the final acceptance contract. Slack step completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778979157553049`.
- `2026-05-17 01:55:25 BST` Started Step 14
  `residual source-picture sprite bridge closure`. This step owns closing the
  remaining unmapped source object-picture labels at the evidence/render-asset
  boundary: `ASXP1`, `NULOB`, and `TEREX`. Unlike the prior prototype-PNG
  slices, these labels will use the checked-in source object-image bytes from
  `assets/red-label/object-images.tsv` because there are no already
  reclassified PNGs for them. This step may add atlas-backed clean sprite IDs,
  a small source-image nibble decoder, and focused renderer/oracle evidence
  tests, but must not add astronaut death lifecycle behavior, null-object
  allocation behavior, terrain-blow timing/presentation, explosion lifecycle,
  or broader render-presentation parity. Validation will start with focused
  renderer, accepted, oracle, clean-fidelity, and public API checks, then close
  with the current all-scenario `make clean-fidelity` gate. Slack step start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778979347729789`.
- `2026-05-17 02:06:44 BST` Completed Step 14. Added clean `SpriteId` values,
  atlas regions, and a small object-picture byte-grid decoder for the remaining
  source object-picture labels with no already reclassified PNGs: `ASXP1`
  maps to `ASTRONAUT_EXPLOSION` from `ASXD10`, `NULOB` maps to transparent
  `NULL_OBJECT` from `NULD10`, and `TEREX` maps to `TERRAIN_EXPLOSION` from
  `TERX0`. The renderer bridge now maps every label in
  `assets/red-label/object-pictures.tsv` to an explicit clean sprite evidence
  target or transparent null target. Expanded renderer coverage for the
  source-image grid atlas regions, and expanded the oracle object-evidence
  fixture so `ASXP1`, `NULOB`, and `TEREX` mapped sprite evidence is exercised
  through the accepted adapter boundary. Updated `README.md`, `SPEC.md`,
  `docs/fidelity/gaps.md`, `assets/arcade/README.md`, and
  `assets/sprites/README.md` to record the provenance boundary. Validation
  passed: `cargo fmt --check`, `cargo test --lib renderer::tests --
  --nocapture`, `cargo test --lib game::tests -- --nocapture`, `cargo test
  --lib public_api_tests -- --nocapture`, `cargo test --lib --features
  legacy-tools accepted -- --nocapture`, `cargo test --lib --features
  legacy-tools oracle -- --nocapture`, `cargo test --lib --features
  legacy-tools clean_fidelity::tests -- --nocapture`, `cargo clippy
  --all-targets --features legacy-tools -- -D warnings`, `markdownlint README.md
  SPEC.md PLAN.md docs/fidelity/gaps.md docs/fidelity/live-audio.md
  assets/arcade/README.md assets/sprites/README.md assets/sounds/README.md`,
  `git diff --check`, and full all-12-scenario `make clean-fidelity` with
  `high_score_entry` matching 3428/3428 frames and total runtime 349.75s.
  `DC-164` remains open because strict R9 final acceptance still needs real
  lifecycle/physics/spawn transitions, explosion timing, score-popup lifecycle,
  stock-count drawing behavior, terrain-blow presentation, and broader
  render-presentation parity work, or an owner decision to redefine the final
  acceptance contract. Slack step completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778980020961169`.
- `2026-05-17 02:10:28 BST` Started Step 15
  `expanded-object lifecycle evidence surface`. This step owns exposing the
  source expanded-object appearance/explosion slots as neutral lifecycle
  evidence through `MachineSnapshot`, the accepted facade, the oracle adapter,
  and strict/full clean-fidelity comparison. The intended surface is active
  expanded-object count, last expanded-object slot, slot kind, descriptor
  address, mapped sprite evidence when available, erase/position fields, and
  attached object address where the source slot provides one. This step must
  not implement or guess explosion timing, score-popup lifecycle, terrain-blow
  behavior, stock-count drawing, clean object spawning/physics, or broader
  render-presentation parity. Validation will start with focused accepted,
  oracle, clean-fidelity, public API, and formatting checks, then close with
  the current all-scenario `make clean-fidelity` gate. Slack step start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778980246899169`.
- `2026-05-17 02:25:26 BST` Completed Step 15. Added clean
  `ExpandedObjectEvidenceSnapshot` state on `WorldSnapshot`, source
  `ExpandedObjectState` collection from the `appearance_ram` expanded-object
  table, and accepted/oracle adapters that carry active count, last slot, slot
  kind, descriptor address, mapped sprite evidence, erase address,
  center/top-left bytes, and attached object address when present. The
  strict/full clean-fidelity comparator now checks
  `state.world.expanded_objects`, with focused tests covering source
  collection, accepted facade exposure, oracle mapping, and full-profile
  mismatch detection. Updated `README.md`, `SPEC.md`, and
  `docs/fidelity/gaps.md` so the lifecycle evidence surface and remaining R9
  blockers are explicit. Validation passed: `cargo fmt --check`, `cargo test
  --lib public_api_tests -- --nocapture`, `cargo test --lib game::tests --
  --nocapture`, `cargo test --lib --features legacy-tools
  explosion_start_matches_exst_slot_initialization_with_valid_center --
  --nocapture`, `cargo test --lib --features legacy-tools accepted --
  --nocapture`, `cargo test --lib --features legacy-tools oracle --
  --nocapture`, `cargo test --lib --features legacy-tools
  clean_fidelity::tests -- --nocapture`, `cargo clippy --all-targets
  --features legacy-tools -- -D warnings`, `markdownlint README.md SPEC.md
  PLAN.md docs/fidelity/gaps.md docs/fidelity/live-audio.md
  assets/arcade/README.md assets/sprites/README.md assets/sounds/README.md`,
  `git diff --check`, and full all-12-scenario `make clean-fidelity` with
  `high_score_entry` matching 3428/3428 frames and total runtime 353.13s.
  `DC-164` remains open because strict R9 final acceptance still needs
  implementation/parity for real lifecycle/physics/spawn transitions,
  explosion timing, score-popup lifecycle, stock-count drawing behavior,
  terrain-blow presentation, and broader render-presentation work, or an owner
  decision to redefine the final acceptance contract. Slack step completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778981164224989`.
- `2026-05-17 02:28:52 BST` Started Step 16
  `source-backed stock-count drawing slice`. This step owns moving the
  remaining stock-count drawing blocker from evidence-only into the clean
  sprite scene for current-player life and smart-bomb stock counts. The slice
  will use the existing reclassified `PLAMIN` and `SBPIC` clean sprite targets,
  source `TDISP` display caps and positions, and focused renderer/game/oracle
  tests. This step must not implement score-popup lifecycle, explosion timing,
  terrain-blow presentation, enemy spawning/physics, two-player top-display
  parity, or broader render-presentation parity. Validation will start with
  focused game, oracle, renderer/public API, and clean-fidelity checks, then
  close with the current all-scenario `make clean-fidelity` gate. Slack step
  start: `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778981353646619`.
- `2026-05-17 02:42:29 BST` Completed Step 16. Clean playing scenes now draw
  current-player life stock and smart-bomb stock as HUD sprites, using the
  existing reclassified `PLAMIN`/`PLAYER_LIFE_STOCK` and
  `SBPIC`/`SMART_BOMB_STOCK` targets, the source-backed five-life and
  three-bomb display caps, and source display origins/steps. The oracle scene
  adapter projects the same stock sprites so accepted-frame render summaries
  stay aligned with the clean scene surface. Focused tests cover clean caps,
  positions, sizes, HUD layer assignment, oracle projection, and the updated
  render summary counts. Updated `README.md`, `SPEC.md`, and
  `docs/fidelity/gaps.md` so current-player stock drawing is no longer listed
  as an active R9 blocker. Validation passed: `cargo fmt --check`, `cargo test
  --lib game::tests::clean_game_draws_current_player_stock_counts_with_arcade_caps
  -- --nocapture`, `cargo test --lib --features legacy-tools
  oracle::tests::oracle_scene_projects_current_player_stock_sprites --
  --nocapture`, `cargo test --lib game::tests -- --nocapture`, `cargo test
  --lib --features legacy-tools oracle -- --nocapture`, `cargo test --lib
  public_api_tests -- --nocapture`, `cargo test --lib renderer::tests --
  --nocapture`, `cargo test --lib --features legacy-tools
  clean_fidelity::tests -- --nocapture`, `cargo clippy --all-targets
  --features legacy-tools -- -D warnings`, `markdownlint README.md SPEC.md
  PLAN.md docs/fidelity/gaps.md docs/fidelity/live-audio.md
  assets/arcade/README.md assets/sprites/README.md assets/sounds/README.md`,
  `git diff --check`, and full all-12-scenario `make clean-fidelity` with
  `high_score_entry` matching 3428/3428 frames and total runtime 353.81s.
  `DC-164` remains open because strict R9 final acceptance still needs
  two-player top-display parity, score-popup lifecycle, explosion timing,
  terrain-blow presentation, clean object spawning/physics, and broader
  render-presentation work, or an owner decision to redefine the final
  acceptance contract. Slack step completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778982188531179`.
- `2026-05-17 02:45:24 BST` Started Step 17
  `two-player stock HUD parity slice`. This step owns extending the clean and
  oracle stock HUD projection so a two-player cabinet state draws player-two
  life-stock and smart-bomb-stock icons at the source-backed `TDISP` player-two
  display addresses, using the existing `PLAMIN`/`PLAYER_LIFE_STOCK` and
  `SBPIC`/`SMART_BOMB_STOCK` sprite targets, caps, and step directions. This
  step must not implement score text rendering, score-popup lifecycle,
  explosion timing, terrain-blow presentation, object spawning/physics, full
  two-player turn/session switching, or broader render-presentation parity.
  Validation will start with focused clean/oracle tests and close with the
  current all-scenario `make clean-fidelity` gate. Slack step start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778982338428049`.
- `2026-05-17 03:02:13 BST` Completed Step 17. Added a clean
  `PlayerStockSnapshot` surface plus `player_count`/`player_stocks` on
  `GameState`, threaded equivalent accepted/oracle stock state through the
  feature-gated accepted adapter, and extended clean/oracle playing scenes to
  draw player-two stock icons at the source-backed `TDISP` player-two display
  positions. The stock scene uses the existing `PLAMIN`/`PLAYER_LIFE_STOCK` and
  `SBPIC`/`SMART_BOMB_STOCK` targets, with the source five-life and three-bomb
  caps and the same horizontal/vertical steps as player one. Focused tests now
  cover two-player stock counts and positions in both the clean game and oracle
  scene adapter, plus accepted stock-state propagation. Updated `README.md`,
  `SPEC.md`, and `docs/fidelity/gaps.md` so the P2 stock-icon part of
  two-player top-display parity is no longer listed as open. Validation passed:
  `cargo fmt --check`, `cargo test --lib
  game::tests::clean_game_draws_two_player_stock_counts_at_arcade_positions --
  --nocapture`, `cargo test --lib --features legacy-tools
  oracle::tests::oracle_scene_projects_two_player_stock_sprites -- --nocapture`,
  `cargo test --lib game::tests -- --nocapture`, `cargo test --lib --features
  legacy-tools oracle -- --nocapture`, `cargo test --lib --features
  legacy-tools accepted -- --nocapture`, `cargo test --lib public_api_tests --
  --nocapture`, `cargo test --lib renderer::tests -- --nocapture`, `cargo test
  --lib --features legacy-tools clean_fidelity::tests -- --nocapture`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  `markdownlint README.md SPEC.md PLAN.md docs/fidelity/gaps.md
  docs/fidelity/live-audio.md assets/arcade/README.md assets/sprites/README.md
  assets/sounds/README.md`, `git diff --check`, and full all-12-scenario
  `make clean-fidelity` with `high_score_entry` matching 3428/3428 frames and
  total runtime 353.88s. `DC-164` remains open because strict R9 final
  acceptance still needs two-player score/text rendering and full turn/session
  switching, score-popup lifecycle, explosion timing, terrain-blow
  presentation, clean object spawning/physics, and broader render-presentation
  work, or an owner decision to redefine the final acceptance contract. Slack
  step completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778983366187999`.
- `2026-05-17 03:04:11 BST` Started Step 18
  `source-backed score digit HUD slice`. This step owns replacing the
  monolithic clean/oracle `SCORE_TEXT` placeholder with source-backed score
  digit sprites for the top-display score fields. The slice will use
  `assets/red-label/score-digits.tsv` `NUMBR0`-`NUMBR9` bytes plus the
  translated `SCRTR0` player-one/player-two display origins, six-digit transfer
  order, leading-zero blanking, and digit step. This step must not implement
  full two-player turn/session switching, title/status text, high-score table
  text rendering, score-popup lifecycle, explosion timing, terrain-blow
  presentation, object spawning/physics, or broader render-presentation parity.
  Validation will start with focused renderer/game/oracle tests and close with
  the current all-scenario `make clean-fidelity` gate. Slack step start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778983463405099`.
- `2026-05-17 03:19:28 BST` Completed Step 18. Added
  `SCORE_DIGIT_0` through `SCORE_DIGIT_9`, decoded the checked-in
  `assets/red-label/score-digits.tsv` `NUMBR0`-`NUMBR9` byte grids into
  renderer-owned atlas regions, and replaced the clean/oracle score placeholder
  scene sprite with source-backed player-one/player-two score digit fields.
  The scene path now uses the translated player-one/player-two score origins,
  six-position transfer order, eight-pixel digit step, source-sized 6x8 glyphs,
  two trailing zeroes, and leading-zero blanking. `--game-smoke` now covers
  `score_digit_0` instead of `score_text`. Updated `README.md`, `SPEC.md`, and
  `docs/fidelity/gaps.md` so the score-field part of two-player top-display
  parity is no longer listed as open. Validation passed: `cargo fmt --check`,
  `cargo test --lib renderer::tests -- --nocapture`, `cargo test --lib
  game::tests -- --nocapture`, `cargo test --lib --features legacy-tools oracle
  -- --nocapture`, `cargo test --lib game_smoke::tests -- --nocapture`,
  `cargo test --lib public_api_tests -- --nocapture`, `cargo test --lib
  --features legacy-tools clean_fidelity::tests -- --nocapture`, `cargo run --
  --game-smoke`, `cargo clippy --all-targets --features legacy-tools --
  -D warnings`, `markdownlint README.md SPEC.md PLAN.md docs/fidelity/gaps.md
  docs/fidelity/live-audio.md assets/arcade/README.md assets/sprites/README.md
  assets/sounds/README.md`, `git diff --check`, and full all-12-scenario
  `make clean-fidelity` with `high_score_entry` matching 3428/3428 frames and
  total runtime 348.82s. `DC-164` remains open because strict R9 final
  acceptance still needs full two-player turn/session switching, remaining
  title/status/high-score text rendering, score-popup lifecycle, explosion
  timing, terrain-blow presentation, clean object spawning/physics, and broader
  render-presentation work, or an owner decision to redefine the final
  acceptance contract. Slack step completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778984392707889`.
- `2026-05-17 03:23:31 BST` Started Step 19
  `two-player start admission slice`. This step owns the source-backed `ST2`
  admission path in the clean game: `start_two` must require two credits, spend
  two credits, enter playing as player one, set `player_count` to 2, and expose
  the already-completed player-one/player-two score and stock top-display scene
  rendering immediately after start. Evidence comes from
  `assets/red-label/input-ports.tsv`, `assets/red-label/switch-table.tsv`, and
  the translated `ST2` start path/tests in `src_legacy/machine_memory.rs` and
  `src_legacy/machine.rs`. This step must not implement full turn/session
  switching after death, player-two respawn flow, title/status/high-score text
  rendering, score-popup lifecycle, explosion timing, terrain-blow presentation,
  clean object spawning/physics, or broader render-presentation parity.
  Validation will start with focused clean-game tests and close with the current
  all-scenario `make clean-fidelity` gate. Slack step start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778984625360509`.
- `2026-05-17 03:35:31 BST` Completed Step 19. Implemented the clean
  source-backed `ST2` admission path by making `start_two` require two credits,
  ignore one-credit attempts, spend two credits on a valid two-player start,
  enter `Playing` as player one, set `player_count` to 2, and expose the
  existing player-one/player-two score and stock top-display scene sprites
  immediately after start. The one-player and two-player paths now share start
  sound and playfield delay handling. Updated `README.md`, `SPEC.md`, and
  `docs/fidelity/gaps.md` so two-player start admission/top-display
  initialization is no longer listed as open. Validation passed: `cargo fmt
  --check`, `cargo test --lib
  game::tests::clean_game_two_player_start_requires_two_credits --
  --nocapture`, `cargo test --lib
  game::tests::clean_game_two_player_start_initializes_top_display_state --
  --nocapture`, `cargo test --lib game::tests -- --nocapture`, `cargo test
  --lib public_api_tests -- --nocapture`, `cargo test --lib
  game_smoke::tests -- --nocapture`, `cargo run -- --game-smoke`, `cargo test
  --lib --features legacy-tools clean_fidelity::tests -- --nocapture`, `cargo
  test --lib renderer::tests -- --nocapture`, `cargo clippy --all-targets
  --features legacy-tools -- -D warnings`, `markdownlint README.md SPEC.md
  PLAN.md docs/fidelity/gaps.md docs/fidelity/live-audio.md
  assets/arcade/README.md assets/sprites/README.md assets/sounds/README.md`,
  `git diff --check`, and full all-12-scenario `make clean-fidelity` with
  `high_score_entry` matching 3428/3428 frames and total runtime 346.76s.
  `DC-164` remains open because strict R9 final acceptance still needs full
  two-player turn/session switching after death, player-two respawn flow,
  remaining title/status/high-score text rendering, score-popup lifecycle,
  explosion timing, terrain-blow presentation, clean object spawning/physics,
  and broader render-presentation work, or an owner decision to redefine the
  final acceptance contract. Slack step completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778985331104449`.
- `2026-05-17 07:47:55 BST` Started Step 20
  `two-player switch/respawn slice`. This step owns the source-backed
  final-life two-player handoff: when the active player has no remaining stock
  but the other player does, clean gameplay should enter the translated
  `PLE02` `0x60`-tick switch sleep, expose neutral switch-from/switch-to state
  through `state.game_over`, then hand off to the other player through the clean
  playfield entry path. The accepted/oracle surface should carry the same
  `MachineSnapshot` evidence. This step must not implement player-switch prompt
  glyph rendering, later two-player turn/session/high-score ordering,
  score-popup lifecycle, explosion timing, terrain-blow presentation, clean
  object spawning/physics, or broader render-presentation parity. Slack step
  start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779000475981059`.
- `2026-05-17 08:04:04 BST` Completed Step 20. Clean final-life death in a
  two-player game now enters the source-backed `PLE02` `0x60`-tick
  player-switch sleep when the other player still has stock, records
  switch-from/switch-to state in `state.game_over`, then hands off to the other
  player through the clean playfield entry path. The accepted `MachineSnapshot`,
  neutral accepted facade, oracle adapter, and clean-fidelity comparison state
  now carry the same switch timing evidence. Updated `README.md`, `SPEC.md`,
  and `docs/fidelity/gaps.md` so the final-life switch/respawn part of
  two-player session flow is no longer listed as open. Validation passed:
  `cargo fmt --check`, `cargo test --lib
  game::tests::clean_game_two_player -- --nocapture`, `cargo test --lib
  game::tests -- --nocapture`, `cargo test --lib --features legacy-tools
  accepted_behavior::tests::accepted_snapshot_carries_player_switch_timing --
  --nocapture`, `cargo test --lib --features legacy-tools
  oracle::tests::oracle_preserves_accepted_player_switch_timing -- --nocapture`,
  `cargo test --lib --features legacy-tools
  session_flow_fixture_covers_two_player_start_and_player_switch_respawn --
  --nocapture`, `cargo test --lib --features legacy-tools
  clean_fidelity::tests -- --nocapture`, `make clean-fidelity
  SCENARIOS="death high_score_entry"`, `cargo clippy --all-targets --features
  legacy-tools -- -D warnings`, `markdownlint README.md SPEC.md PLAN.md
  docs/fidelity/gaps.md docs/fidelity/live-audio.md assets/arcade/README.md
  assets/sprites/README.md assets/sounds/README.md`, `git diff --check`, full
  all-12-scenario `make clean-fidelity` with `high_score_entry` matching
  3428/3428 frames and total runtime 353.60s, `cargo test --lib
  public_api_tests -- --nocapture`, and `cargo run -- --game-smoke`. `DC-164`
  remains open because strict R9 final acceptance still needs later two-player
  turn/session sequencing and high-score ordering, player-switch prompt text
  and other title/status/high-score text rendering, score-popup lifecycle,
  explosion timing, terrain-blow presentation, clean object spawning/physics,
  and broader render-presentation work, or an owner decision to redefine the
  final acceptance contract. Slack step completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779001444619529`.
- `2026-05-17 08:07:57 BST` Started Step 21
  `player-switch prompt text slice`. This step owns the source-backed prompt
  text that Step 20 left out: during the translated `PLE02` player-switch
  sleep, clean and oracle scenes should draw `PLYR1`/`PLYR2` as
  `PLAYER ONE`/`PLAYER TWO` at `0x3C78` and `GO` as `GAME OVER` at `0x3E88`
  using `assets/red-label/messages.tsv` plus
  `assets/red-label/message-glyphs.tsv` glyph pixels. This step must not
  implement later two-player turn/session/high-score ordering, broader
  title/status/high-score text rendering, score-popup lifecycle, explosion
  timing, terrain-blow presentation, clean object spawning/physics, or broader
  render-presentation parity. Slack step start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779001677054109`.
- `2026-05-17 08:26:47 BST` Completed Step 21. Clean and oracle scenes now
  render the source-backed player-switch prompt during the translated `PLE02`
  switch sleep: `PLYR1`/`PLYR2` resolves to `PLAYER ONE`/`PLAYER TWO` at
  `0x3C78`, `GO` resolves to `GAME OVER` at `0x3E88`, and the default sprite
  atlas decodes the message glyph pixels from
  `assets/red-label/message-glyphs.tsv`. The clean-facing helper names stay
  source-neutral so the public API quarantine keeps legacy implementation
  terminology out of clean/oracle/renderer sources. Updated `README.md`,
  `SPEC.md`, and `docs/fidelity/gaps.md` so player-switch prompt text is no
  longer listed as open. Validation passed: `cargo fmt --check`, `cargo test
  --lib renderer::tests::default_sprite_atlas_uses_message_glyph_regions --
  --nocapture`, `cargo test --lib game::tests::clean_game_two_player --
  --nocapture`, `cargo test --lib game::tests -- --nocapture`, `cargo test
  --lib renderer::tests -- --nocapture`, `cargo test --lib --features
  legacy-tools oracle::tests::oracle_scene_projects_player_switch_prompt_sprites
  -- --nocapture`, `cargo test --lib --features legacy-tools oracle::tests --
  --nocapture`, `cargo test --lib --features legacy-tools
  clean_fidelity::tests -- --nocapture`, `make clean-fidelity
  SCENARIOS="death high_score_entry"`, `cargo test --lib public_api_tests --
  --nocapture`, `cargo run -- --game-smoke`, `cargo clippy --all-targets
  --features legacy-tools -- -D warnings`, `markdownlint README.md SPEC.md
  PLAN.md docs/fidelity/gaps.md docs/fidelity/live-audio.md
  assets/arcade/README.md assets/sprites/README.md assets/sounds/README.md`,
  `git diff --check`, and full all-12-scenario `make clean-fidelity` with
  `high_score_entry` matching 3428/3428 frames and total runtime 354.79s.
  `DC-164` remains open because strict R9 final acceptance still needs later
  two-player turn/session sequencing and high-score ordering, remaining
  title/status/high-score text rendering, score-popup lifecycle, explosion
  timing, terrain-blow presentation, clean object spawning/physics, and broader
  render-presentation work, or an owner decision to redefine the final
  acceptance contract. Slack step completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779002845309919`.
- `2026-05-17 08:29:23 BST` Started Step 22
  `final game-over text slice`. This step owns the ordinary final
  player-death game-over prompt that source `PLE2` writes before its 40-tick
  sleep: clean and oracle scenes should draw `GO` as `GAME OVER` at `0x3E80`
  while `state.game_over.player_death_sleep_remaining` is active, reusing the
  source-neutral message glyph path from Step 21. This step must not implement
  later two-player turn/session/high-score ordering, broader
  title/status/high-score text rendering, score-popup lifecycle, explosion
  timing, terrain-blow presentation, clean object spawning/physics, or broader
  render-presentation parity. Slack step start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779002963421069`.
- `2026-05-17 08:40:13 BST` Completed Step 22. Clean and oracle scenes now
  render the source-backed ordinary final game-over prompt during the `PLE2`
  player-death sleep: `GO` resolves to `GAME OVER` at `0x3E80` while
  `state.game_over.player_death_sleep_remaining` is active. This reuses the
  source-neutral message glyph path and leaves broader title/status/high-score
  text out of scope. Updated `README.md`, `SPEC.md`, and
  `docs/fidelity/gaps.md` so the ordinary final game-over prompt is no longer
  part of the remaining text backlog. Validation passed: `cargo fmt --check`,
  `cargo test --lib
  game::tests::clean_game_player_enemy_final_collision_enters_game_over --
  --nocapture`, `cargo test --lib --features legacy-tools
  oracle::tests::oracle_scene_projects_final_game_over_prompt_sprites --
  --nocapture`, `cargo test --lib game::tests -- --nocapture`, `cargo test
  --lib --features legacy-tools oracle::tests -- --nocapture`, `cargo test
  --lib public_api_tests -- --nocapture`, `make clean-fidelity
  SCENARIOS="death high_score_entry"`, `cargo test --lib --features
  legacy-tools clean_fidelity::tests -- --nocapture`, `cargo run --
  --game-smoke`, `cargo clippy --all-targets --features legacy-tools -- -D
  warnings`, `markdownlint README.md SPEC.md PLAN.md docs/fidelity/gaps.md
  docs/fidelity/live-audio.md assets/arcade/README.md assets/sprites/README.md
  assets/sounds/README.md`, `git diff --check`, and full all-12-scenario
  `make clean-fidelity` with `high_score_entry` matching 3428/3428 frames and
  total runtime 355.42s. `DC-164` remains open because strict R9 final
  acceptance still needs later two-player turn/session sequencing and
  high-score ordering, remaining title/status/high-score text rendering,
  score-popup lifecycle, explosion timing, terrain-blow presentation, clean
  object spawning/physics, and broader render-presentation work, or an owner
  decision to redefine the final acceptance contract. Slack step completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779003641986509`.
- `2026-05-17 08:44:23 BST` Started Step 23
  `high-score entry prompt text slice`. This step owns the source-backed
  initials-entry prompt text that source `write_high_score_entry_display`
  writes while the hall-of-fame initials entry screen is active: clean and
  oracle scenes should draw the current player label at `0x3E38`, the
  `HOFV1`-`HOFV4` instruction lines from `0x1458` with source vertical
  offsets, and entered initials from `0x46AC` with source horizontal offsets
  while `GamePhase::HighScoreEntry` is active. This step reuses the
  source-neutral message glyph path from Steps 21-22 and must not implement
  underline words/blink/color, full `HALDIS` table text, later two-player
  turn/session/high-score ordering, score-popup lifecycle, explosion timing,
  terrain-blow presentation, clean object spawning/physics, or broader
  render-presentation parity. Slack step start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779003863428949`.
- `2026-05-17 08:57:02 BST` Completed Step 23. Clean and oracle scenes now
  render the source-backed high-score initials-entry prompt while
  `GamePhase::HighScoreEntry` is active: the current player label resolves at
  `0x3E38`, `HOFV1`-`HOFV4` resolve from `0x1458` with source vertical
  offsets, and entered initials resolve from `0x46AC` with source horizontal
  offsets. This reuses the source-neutral message glyph path and adds a shared
  source screen-address offset helper for text rows/columns. Updated
  `README.md`, `SPEC.md`, and `docs/fidelity/gaps.md` so the active high-score
  entry prompt is no longer part of the remaining text backlog. Validation
  passed: `cargo fmt --check`, `cargo test --lib
  renderer::tests::default_sprite_atlas_uses_message_glyph_regions --
  --nocapture`, `cargo test --lib
  game::tests::clean_game_high_score_entry_starts_after_qualifying_game_over --
  --nocapture`, `cargo test --lib
  game::tests::clean_game_high_score_initials_accept_backspace_and_submit --
  --nocapture`, `cargo test --lib --features legacy-tools
  oracle::tests::oracle_scene_projects_high_score_entry_prompt_sprites --
  --nocapture`, `cargo test --lib game::tests -- --nocapture`, `cargo test
  --lib --features legacy-tools oracle::tests -- --nocapture`, `cargo test
  --lib public_api_tests -- --nocapture`, `make clean-fidelity
  SCENARIOS="high_score_entry"`, `cargo test --lib --features legacy-tools
  clean_fidelity::tests -- --nocapture`, `cargo run -- --game-smoke`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  `markdownlint README.md SPEC.md PLAN.md docs/fidelity/gaps.md
  docs/fidelity/live-audio.md assets/arcade/README.md assets/sprites/README.md
  assets/sounds/README.md`, `git diff --check`, and full all-12-scenario
  `make clean-fidelity` with `high_score_entry` matching 3428/3428 frames and
  total runtime 355.59s. `DC-164` remains open because strict R9 final
  acceptance still needs later two-player turn/session sequencing and
  high-score ordering, title/status/high-score display text outside the entry
  prompt, score-popup lifecycle, explosion timing, terrain-blow presentation,
  clean object spawning/physics, and broader render-presentation work, or an
  owner decision to redefine the final acceptance contract. Slack step
  completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779004639870499`.
- `2026-05-17 09:01:18 BST` Started Step 24
  `hall-of-fame display text slice`. This step owns the source-backed text that
  `HALDIS` writes after a submitted or non-qualifying high-score flow enters
  the hall-of-fame display stall: clean and oracle scenes should draw the
  display headings from `HALLD_TITLE`, `HALLD_TODAYS`, `HALLD_ALL_TIME`, and
  `HALLD_GREATEST`, then draw both visible high-score tables with rank digits,
  initials, and six-character score fields while
  `state.game_over.hall_of_fame_stall_remaining` is active. This step must
  reuse source message glyph and score digit assets plus the source table
  starts, row step, initials offset, score offset, and leading-blank score
  rules. It must not implement underline bars, the expanded `DEFNNN` logo,
  table/logo colors, later two-player turn/session high-score ordering,
  score-popup lifecycle, explosion timing, terrain-blow presentation, clean
  object spawning/physics, or broader render-presentation parity. Slack step
  start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779004878288389`.
- `2026-05-17 09:17:35 BST` Completed Step 24. Clean and oracle scenes now
  render the source-backed hall-of-fame display text while
  `state.game_over.hall_of_fame_stall_remaining` is active: headings resolve
  from `HALLD_TITLE`, `HALLD_TODAYS`, `HALLD_ALL_TIME`, and `HALLD_GREATEST`,
  and both visible high-score tables render rank digits, initials, and
  six-character score fields using the source table starts, `0x0A` row step,
  initials offset, score offset, and leading-blank score rules. Added a shared
  source text helper for mixed message glyphs and score digits. Updated
  `README.md`, `SPEC.md`, and `docs/fidelity/gaps.md` so hall-of-fame display
  heading/table text is no longer part of the remaining text backlog.
  Validation passed: `cargo fmt --check`, `cargo test --lib
  renderer::tests::source_text_bytes_render_mixed_score_digits_and_message_glyphs
  -- --nocapture`, `cargo test --lib
  game::tests::clean_game_high_score_initials_accept_backspace_and_submit --
  --nocapture`, `cargo test --lib
  game::tests::clean_game_non_qualifying_game_over_waits_then_shows_hall_of_fame
  -- --nocapture`, `cargo test --lib --features legacy-tools
  oracle::tests::oracle_scene_projects_hall_of_fame_display_text_sprites --
  --nocapture`, `cargo test --lib game::tests -- --nocapture`, `cargo test
  --lib --features legacy-tools oracle::tests -- --nocapture`, `cargo test
  --lib public_api_tests -- --nocapture`, `make clean-fidelity
  SCENARIOS="death high_score_entry"`, `cargo test --lib --features
  legacy-tools clean_fidelity::tests -- --nocapture`, `cargo run --
  --game-smoke`, `cargo clippy --all-targets --features legacy-tools -- -D
  warnings`, `markdownlint README.md SPEC.md PLAN.md docs/fidelity/gaps.md
  docs/fidelity/live-audio.md assets/arcade/README.md assets/sprites/README.md
  assets/sounds/README.md`, `git diff --check`, and full all-12-scenario
  `make clean-fidelity` with `high_score_entry` matching 3428/3428 frames and
  total runtime 357.41s. `DC-164` remains open because strict R9 final
  acceptance still needs later two-player turn/session sequencing and
  high-score ordering, title/status text plus high-score underline/logo
  presentation, score-popup lifecycle, explosion timing, terrain-blow
  presentation, clean object spawning/physics, and broader render-presentation
  work, or an owner decision to redefine the final acceptance contract. Slack
  step completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779005874124009`.
- `2026-05-17 09:25:24 BST` Started Step 25
  `high-score entry underline word slice`. This step owns the source-backed
  `HOFUL` underline words while `GamePhase::HighScoreEntry` is active: clean
  and oracle scenes should draw three initials' worth of underline words from
  source start `0x45B7`, initial step `0x0800`, and word offsets
  `[0x0400,0x0300,0x0200,0x0100]`. The active cursor underline should be marked
  separately from inactive underline words using clean tint constants and a
  small atlas-backed sprite. It must not implement exact palette/blink/color
  word fidelity, hall-of-fame display underline sweep, the expanded `DEFNNN`
  logo, title/status text, later two-player turn/session high-score ordering,
  score-popup lifecycle, explosion timing, terrain-blow presentation, clean
  object spawning/physics, or broader render-presentation parity. Slack step
  start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779006129129999`.
- `2026-05-17 09:35:38 BST` Completed Step 25. Clean and oracle scenes now
  render the source-backed `HOFUL` high-score entry underline words while
  `GamePhase::HighScoreEntry` is active. The projection uses source start
  `0x45B7`, initial step `0x0800`, and word offsets
  `[0x0400,0x0300,0x0200,0x0100]`, with a small atlas-backed
  `HALL_OF_FAME_UNDERLINE_WORD` sprite and clean tint constants to distinguish
  the active cursor underline from inactive underline words. Updated
  `README.md`, `SPEC.md`, and `docs/fidelity/gaps.md` so the active
  high-score entry underline positions are no longer part of the remaining
  text/presentation backlog. Exact underline palette/blink/color behavior,
  hall-of-fame display underline/logo presentation, title/status text, and the
  broader R9 blockers remain open. Validation passed: `cargo fmt --check`,
  `cargo test --lib
  renderer::tests::default_sprite_atlas_uses_high_score_underline_word_region
  -- --nocapture`, `cargo test --lib
  game::tests::clean_game_high_score_entry_starts_after_qualifying_game_over
  -- --nocapture`, `cargo test --lib
  game::tests::clean_game_high_score_initials_accept_backspace_and_submit --
  --nocapture`, `cargo test --lib --features legacy-tools
  oracle::tests::oracle_scene_projects_high_score_entry_prompt_sprites --
  --nocapture`, `cargo test --lib game::tests -- --nocapture`, `cargo test
  --lib --features legacy-tools oracle::tests -- --nocapture`, `cargo test
  --lib public_api_tests -- --nocapture`, `make clean-fidelity
  SCENARIOS="high_score_entry"` with `high_score_entry` matching 3428/3428
  frames, `cargo test --lib --features legacy-tools clean_fidelity::tests --
  --nocapture`, `cargo run -- --game-smoke`, `cargo clippy --all-targets
  --features legacy-tools -- -D warnings`, `markdownlint README.md SPEC.md
  PLAN.md docs/fidelity/gaps.md docs/fidelity/live-audio.md
  assets/arcade/README.md assets/sprites/README.md assets/sounds/README.md`,
  `git diff --check`, the clean-source terminology guard scan, and full
  all-12-scenario `make clean-fidelity` with every scenario matching and
  `high_score_entry` matching 3428/3428 frames in total runtime 356.95s.
  `DC-164` remains open because strict R9 final acceptance still needs later
  two-player turn/session sequencing and high-score ordering, title/status
  text, hall-of-fame display underline/logo presentation, exact entry underline
  palette/blink/color behavior, score-popup lifecycle, explosion timing,
  terrain-blow presentation, clean object spawning/physics, and broader
  render-presentation work, or an owner decision to redefine the final
  acceptance contract. Slack step completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779006965565129`.
- `2026-05-17 09:40:02 BST` Started Step 26
  `hall-of-fame display underline bar slice`. This step owns the source-backed
  `HALDIS` display underline words while
  `state.game_over.hall_of_fame_stall_remaining` is active: clean and oracle
  scenes should draw the two display underline bar segments from source left
  base `0x1E7B`, using offset segment `0x5F..0x41` followed by
  `0x1E..0x00`. This should produce the accepted first/last underline word
  positions `0x7D7B` and `0x1E7B`, reusing the atlas-backed underline word
  sprite from Step 25. It must not implement the expanded `DEFNNN` logo, exact
  palette/blink/color word fidelity, title/status text, later two-player
  turn/session high-score ordering, score-popup lifecycle, explosion timing,
  terrain-blow presentation, clean object spawning/physics, or broader
  render-presentation parity. Slack step start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779007106503389`.
- `2026-05-17 09:50:19 BST` Completed Step 26. Clean and oracle scenes now
  render the source-backed `HALDIS` display underline words while
  `state.game_over.hall_of_fame_stall_remaining` is active. The projection
  uses source left base `0x1E7B` and the two-segment offset walk `0x5F..0x41`
  followed by `0x1E..0x00`, producing the accepted first/last underline word
  positions `0x7D7B` and `0x1E7B`. The display underline bars reuse the
  atlas-backed `HALL_OF_FAME_UNDERLINE_WORD` sprite from Step 25, and tests
  now assert the 62 display underline sprites in clean/oracle hall-of-fame
  display scenes. Updated `README.md`, `SPEC.md`, and
  `docs/fidelity/gaps.md` so display underline positions are no longer part of
  the open high-score presentation backlog. Exact underline
  palette/blink/color behavior, expanded hall-of-fame logo presentation,
  title/status text, and the broader R9 blockers remain open. Validation
  passed: `cargo fmt --check`, `cargo test --lib
  game::tests::clean_game_high_score_initials_accept_backspace_and_submit --
  --nocapture`, `cargo test --lib
  game::tests::clean_game_non_qualifying_game_over_waits_then_shows_hall_of_fame
  -- --nocapture`, `cargo test --lib --features legacy-tools
  oracle::tests::oracle_scene_projects_hall_of_fame_display_text_sprites --
  --nocapture`, `cargo test --lib game::tests -- --nocapture`, `cargo test
  --lib --features legacy-tools oracle::tests -- --nocapture`, `cargo test
  --lib public_api_tests -- --nocapture`, `make clean-fidelity
  SCENARIOS="high_score_entry"` with `high_score_entry` matching 3428/3428
  frames, `cargo test --lib --features legacy-tools clean_fidelity::tests --
  --nocapture`, `cargo run -- --game-smoke`, `cargo clippy --all-targets
  --features legacy-tools -- -D warnings`, `markdownlint README.md SPEC.md
  PLAN.md docs/fidelity/gaps.md docs/fidelity/live-audio.md
  assets/arcade/README.md assets/sprites/README.md assets/sounds/README.md`,
  `git diff --check`, the clean-source terminology guard scan, and full
  all-12-scenario `make clean-fidelity` with every scenario matching and
  `high_score_entry` matching 3428/3428 frames in total runtime 358.15s.
  `DC-164` remains open because strict R9 final acceptance still needs later
  two-player turn/session sequencing and high-score ordering, title/status
  text, expanded hall-of-fame logo presentation, exact underline
  palette/blink/color behavior, score-popup lifecycle, explosion timing,
  terrain-blow presentation, clean object spawning/physics, and broader
  render-presentation work, or an owner decision to redefine the final
  acceptance contract. Slack step completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779007842672359`.
- `2026-05-17 09:57:07 BST` Started Step 27
  `attract credits text slice`. This step owns the source-backed attract
  credits overlay during normal `GamePhase::Attract`: clean and oracle scenes
  should draw the `CREDV` `CREDITS:` label at source screen address `0x28E5`
  and the visible credit count digits at `0x48E5`, using source message glyph
  and score digit sprites. The overlay should be suppressed while
  `state.game_over.hall_of_fame_stall_remaining` is active so it does not
  overlap `HALDIS`. It must not implement broader title/status text, attract
  instruction pages, Williams/Defender logo pages, scanner/radar presentation,
  exact palette/blink/color behavior, later two-player turn/session high-score
  ordering, score-popup lifecycle, explosion timing, terrain-blow
  presentation, clean object spawning/physics, or broader render-presentation
  parity. Slack step start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779008120975659`.
- `2026-05-17 10:10:13 BST` Completed Step 27
  `attract credits text slice`. Clean and oracle scenes now project the
  source-backed `CREDV` `CREDITS:` label at `0x28E5` and visible credit digits
  at `0x48E5` on normal attract frames, with the projection suppressed during
  the hall-of-fame display stall. The clean credit digit helper caps visible
  credits at 99 and preserves the source one- vs two-digit presentation shape.
  The clean game smoke counter now counts overlay draw commands toward
  aggregate drawn sprite instances while keeping overlay out of gameplay
  pipeline coverage metrics, matching the renderer's sprite-instance upload
  accounting. Documentation now narrows the remaining presentation blockers to
  title/status text beyond attract credits, expanded hall-of-fame logo
  presentation, exact initials-entry title/score copy, and remaining
  color/font edge cases. Validation: `cargo fmt --check`, `cargo test --lib
  game::tests::clean_game_credits_starts_and_emits_sprite_frame --
  --nocapture`, `cargo test --lib --features legacy-tools
  oracle::tests::oracle_scene_projects_attract_credit_text_sprites --
  --nocapture`, `cargo test --lib game::tests -- --nocapture`, `cargo test
  --lib --features legacy-tools oracle::tests -- --nocapture`, `cargo test
  --lib public_api_tests -- --nocapture`, `cargo test --lib --features
  legacy-tools clean_fidelity::tests -- --nocapture`, `cargo test --lib
  game_smoke::tests -- --nocapture`, `cargo run -- --game-smoke`, `cargo
  clippy --all-targets --features legacy-tools -- -D warnings`, `markdownlint
  README.md SPEC.md PLAN.md docs/fidelity/gaps.md docs/fidelity/live-audio.md
  assets/arcade/README.md assets/sprites/README.md assets/sounds/README.md`,
  `git diff --check`, the clean-source terminology guard scan, `make
  clean-fidelity SCENARIOS="attract_boot start_game"` with both scenarios
  matching, and full all-12-scenario `make clean-fidelity` with every scenario
  matching and total runtime 356.12s. `DC-164` remains open for the remaining
  presentation/behavior blockers listed above and for later owner acceptance
  of the R9 final contract. Slack step completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779009009158269`.
- `2026-05-17 10:14:55 BST` Started Step 28
  `hall-of-fame Defender logo slice`. This step owns the source-backed
  `HALDIS` expanded Defender logo projection while
  `state.game_over.hall_of_fame_stall_remaining` is active: clean and oracle
  scenes should draw the expanded logo at source screen address `0x3038` with
  source dimensions `0x3C` by `0x18`, using an atlas-backed sprite generated
  from the same compressed source logo bytes that the accepted adapter expands
  for `DEFNNN`. It must not implement exact palette/blink/color word fidelity,
  title/status text beyond this logo, initials-entry copy refinements, later
  two-player turn/session high-score ordering, score-popup lifecycle,
  explosion timing, terrain-blow presentation, clean object spawning/physics,
  or broader render-presentation parity. Slack step start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779009283181119`.
- `2026-05-17 10:30:01 BST` Completed Step 28
  `hall-of-fame Defender logo slice`. The clean sprite atlas now includes
  `HALL_OF_FAME_DEFENDER_LOGO`, generated from the compressed source logo
  bytes into the `0x3C` by `0x18` source image shape and rendered as a
  120-by-24 sprite. Clean and oracle hall-of-fame display scenes now draw that
  logo while `state.game_over.hall_of_fame_stall_remaining` is active at
  source screen address `0x3038`. Documentation now removes expanded
  hall-of-fame logo shape from the remaining blocker list while leaving exact
  logo/underline palette, blink, and color behavior open. Validation:
  `cargo fmt --check`, `cargo test --lib renderer::tests -- --nocapture`,
  `cargo test --lib game::tests -- --nocapture`, `cargo test --lib --features
  legacy-tools oracle::tests -- --nocapture`, `cargo test --lib
  public_api_tests -- --nocapture`, `cargo test --lib game_smoke::tests --
  --nocapture`, `cargo test --lib --features legacy-tools
  clean_fidelity::tests -- --nocapture`, `cargo run -- --game-smoke`, `cargo
  clippy --all-targets --features legacy-tools -- -D warnings`, `markdownlint
  README.md SPEC.md PLAN.md docs/fidelity/gaps.md docs/fidelity/live-audio.md
  assets/arcade/README.md assets/sprites/README.md assets/sounds/README.md`,
  `git diff --check`, the clean-source terminology guard scan, `make
  clean-fidelity SCENARIOS="high_score_entry"` with `high_score_entry`
  matching 3428/3428 frames in 57.67s, and full all-12-scenario `make
  clean-fidelity` with every scenario matching and total runtime 356.32s.
  `DC-164` remains open for later two-player turn/session sequencing and
  high-score ordering, title/status text beyond attract credits, exact
  logo/underline palette/blink/color behavior, score-popup lifecycle,
  explosion timing, terrain-blow presentation, clean object spawning/physics,
  final render presentation parity, and later owner acceptance of the R9 final
  contract. Slack step completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779010190161449`.
- `2026-05-17 10:36:53 BST` Started Step 29
  `attract presents text slice`. This step owns the source-backed
  `ELECTRONICS INC.` / `PRESENTS` attract text projection at translated source
  screen addresses `0x3258` and `0x3E6C` in the clean and oracle scene paths.
  It must not implement Williams/Defender attract logo timing, page scheduler
  behavior, exact palette/blink/color behavior, later two-player turn/session
  high-score ordering, score-popup lifecycle, explosion timing,
  terrain-blow presentation, clean object spawning/physics, broader
  title/status text, or final render-presentation parity. Slack step start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779010512185109`.
- `2026-05-17 10:47:12 BST` Completed Step 29
  `attract presents text slice`. Clean and oracle attract scenes now project
  the translated `ELECTRONICS INC.` text at source screen address `0x3258` and
  `PRESENTS` at `0x3E6C`, using source message/score glyph sprites and
  suppressing the overlay during the hall-of-fame display stall. README,
  SPEC, and `docs/fidelity/gaps.md` now record attract presents copy as covered
  while leaving broader title/status text, attract logo/page timing, exact
  palette/blink/color behavior, score-popup lifecycle, explosion timing,
  terrain-blow presentation, clean object spawning/physics, final render
  presentation parity, and owner R9 acceptance open. Validation passed:
  `cargo fmt --check`, `cargo test --lib
  game::tests::clean_game_credits_starts_and_emits_sprite_frame --
  --nocapture`, `cargo test --lib --features legacy-tools
  oracle::tests::oracle_scene_projects_attract_credit_text_sprites --
  --nocapture`, `cargo test --lib game::tests -- --nocapture`, `cargo test
  --lib --features legacy-tools oracle::tests -- --nocapture`, `cargo test
  --lib public_api_tests -- --nocapture`, `cargo test --lib --features
  legacy-tools clean_fidelity::tests -- --nocapture`, `cargo test --lib
  game_smoke::tests -- --nocapture`, `cargo run -- --game-smoke`, `cargo
  clippy --all-targets --features legacy-tools -- -D warnings`, `markdownlint
  README.md SPEC.md PLAN.md docs/fidelity/gaps.md docs/fidelity/live-audio.md
  assets/arcade/README.md assets/sprites/README.md assets/sounds/README.md`,
  `git diff --check`, the clean-source terminology guard scan, `make
  clean-fidelity SCENARIOS="attract_boot start_game"` with both scenarios
  matching in 30.45s, and full all-12-scenario `make clean-fidelity` with
  every scenario matching and total runtime 355.48s. `DC-164` remains open for
  the remaining R9 blockers listed above and for later owner acceptance of the
  R9 final contract. Slack step completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779011272343499`.
- `2026-05-17 10:50:59 BST` Started Step 30
  `source message control text slice`. This step owns a clean renderer helper
  for source message control tokens used by the `ELECV` attract message,
  including row feeds and horizontal cursor movement, then routes the attract
  `ELECTRONICS INC.` / `PRESENTS` overlay through the source message text
  instead of hard-coded text rows. It must not implement attract page
  scheduler behavior, Williams/Defender logo timing, exact palette/blink/color
  behavior, later two-player turn/session high-score ordering,
  score-popup lifecycle, explosion timing, terrain-blow presentation, clean
  object spawning/physics, broader title/status text, or final
  render-presentation parity. Slack step start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779011458756759`.
- `2026-05-17 11:02:11 BST` Completed Step 30
  `source message control text slice`. The clean renderer now has a
  source-message control helper that applies row feeds, horizontal/vertical
  cursor moves, and reset controls while preserving source glyph cursor
  advance. Clean and oracle attract scenes now render `ELECV` through that
  helper so the source text itself positions `ELECTRONICS INC.` at `0x3258`
  and `PRESENTS` at `0x3E6C`. README, SPEC, and
  `docs/fidelity/gaps.md` now record this as source-control-token-backed
  presentation rather than duplicated text rows. Validation passed:
  `cargo fmt --check`, `cargo test --lib
  renderer::tests::source_controlled_message_sprites_apply_source_cursor_controls
  -- --nocapture`, `cargo test --lib
  game::tests::clean_game_credits_starts_and_emits_sprite_frame --
  --nocapture`, `cargo test --lib --features legacy-tools
  oracle::tests::oracle_scene_projects_attract_credit_text_sprites --
  --nocapture`, `cargo test --lib renderer::tests -- --nocapture`, `cargo
  test --lib game::tests -- --nocapture`, `cargo test --lib --features
  legacy-tools oracle::tests -- --nocapture`, `cargo test --lib
  public_api_tests -- --nocapture`, `cargo test --lib --features
  legacy-tools clean_fidelity::tests -- --nocapture`, `cargo test --lib
  game_smoke::tests -- --nocapture`, `cargo run -- --game-smoke`, `cargo
  clippy --all-targets --features legacy-tools -- -D warnings`, `markdownlint
  README.md SPEC.md PLAN.md docs/fidelity/gaps.md docs/fidelity/live-audio.md
  assets/arcade/README.md assets/sprites/README.md assets/sounds/README.md`,
  `git diff --check`, the clean-source terminology guard scan, `make
  clean-fidelity SCENARIOS="attract_boot start_game"` with both scenarios
  matching in 30.38s, and full all-12-scenario `make clean-fidelity` with
  every scenario matching and total runtime 354.83s. `DC-164` remains open for
  broader title/status text outside attract credits/presents, attract
  logo/page timing, exact palette/blink/color behavior, later two-player
  turn/session sequencing and high-score ordering, score-popup lifecycle,
  explosion timing, terrain-blow presentation, clean object spawning/physics,
  final render presentation parity, and later owner acceptance of the R9 final
  contract. Slack step completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779012167756559`.
- `2026-05-17 11:06:23 BST` Started Step 31
  `attract instruction text slice`. This step owns the source-backed attract
  instruction labels during normal `GamePhase::Attract`: clean and oracle
  scenes should draw `SCANV`, `LANDV`, `MUTV`, `BAITV`, `BOMBV`, `SWRMPV`, and
  `SWARMV` at source screen addresses `0x4330`, `0x1C70`, `0x3C70`,
  `0x5F70`, `0x1CA8`, `0x40A8`, and `0x5CA8` through the clean
  source-message control helper. The projection should be suppressed while
  `state.game_over.hall_of_fame_stall_remaining` is active. It must not
  implement the attract page scheduler, Williams/Defender logo timing,
  scanner/radar animation, enemy demo scripting, exact palette/blink/color
  behavior, later two-player turn/session high-score ordering,
  score-popup lifecycle, explosion timing, terrain-blow presentation, clean
  object spawning/physics, or broader render-presentation parity. Slack step
  start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779012395078119`.
- `2026-05-17 11:18:04 BST` Completed Step 31
  `attract instruction text slice`. Clean and oracle attract scenes now
  project the source-backed instruction labels `SCANV`, `LANDV`, `MUTV`,
  `BAITV`, `BOMBV`, `SWRMPV`, and `SWARMV` at `0x4330`, `0x1C70`,
  `0x3C70`, `0x5F70`, `0x1CA8`, `0x40A8`, and `0x5CA8`, reusing the
  source-message control helper for row-feed and horizontal-cursor behavior.
  The projection remains suppressed during the hall-of-fame display stall.
  README, SPEC, and `docs/fidelity/gaps.md` now record the instruction labels
  as covered while leaving attract page scheduling, logo timing, scanner/radar
  animation, palette/blink/color parity, and broader gameplay/render blockers
  open. Validation passed: `cargo fmt --check`, `cargo test --lib
  renderer::tests::source_controlled_message_sprites_apply_source_cursor_controls
  -- --nocapture`, `cargo test --lib
  game::tests::clean_game_credits_starts_and_emits_sprite_frame --
  --nocapture`, `cargo test --lib --features legacy-tools
  oracle::tests::oracle_scene_projects_attract_credit_text_sprites --
  --nocapture`, `cargo test --lib game::tests -- --nocapture`, `cargo test
  --lib --features legacy-tools oracle::tests -- --nocapture`, `cargo test
  --lib renderer::tests -- --nocapture`, `cargo test --lib public_api_tests
  -- --nocapture`, `cargo test --lib --features legacy-tools
  clean_fidelity::tests -- --nocapture`, `cargo test --lib game_smoke::tests
  -- --nocapture`, `cargo run -- --game-smoke`, `cargo clippy --all-targets
  --features legacy-tools -- -D warnings`, `markdownlint README.md SPEC.md
  PLAN.md docs/fidelity/gaps.md docs/fidelity/live-audio.md
  assets/arcade/README.md assets/sprites/README.md assets/sounds/README.md`,
  `git diff --check`, the clean-source terminology guard scan, `make
  clean-fidelity SCENARIOS="attract_boot start_game"` with both scenarios
  matching in 30.55s, and full all-12-scenario `make clean-fidelity` with
  every scenario matching and total runtime 356.71s. `DC-164` remains open for
  broader title/status text outside attract credits/presents/instruction
  labels, attract logo/page timing, exact palette/blink/color behavior, later
  two-player turn/session sequencing and high-score ordering, score-popup
  lifecycle, explosion timing, terrain-blow presentation, clean object
  spawning/physics, final render presentation parity, and later owner
  acceptance of the R9 final contract. Slack step completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779013129787109`.
- `2026-05-17 11:20:46 BST` Started Step 32
  `two-player start prompt slice`. This step owns the source-backed
  player-start prompt during the clean two-player start handoff: clean and
  oracle scenes should draw `PLYR1`/`PLYR2` as `PLAYER ONE`/`PLAYER TWO` at
  source screen address `0x3C80` while the playfield start is pending. It must
  not add a one-player start prompt, broaden player-switch/session sequencing,
  change gameplay lifecycle rules, implement the attract page scheduler,
  scanner/radar animation, exact palette/blink/color behavior, score-popup
  lifecycle, explosion timing, terrain-blow presentation, clean object
  spawning/physics, or broader render-presentation parity. Slack step start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779013256956749`.
- `2026-05-17 11:30:46 BST` Completed Step 32
  `two-player start prompt slice`. Clean and oracle scenes now project the
  source-backed `PLYR1`/`PLYR2` player-start prompt at `0x3C80` during the
  two-player playfield-start handoff. The projection is gated to the pending
  two-player start state, so one-player starts and ordinary gameplay remain
  unchanged. README, SPEC, and `docs/fidelity/gaps.md` now record this prompt
  as covered while leaving broader two-player turn/session sequencing,
  palette/blink/color parity, attract page scheduling, and gameplay lifecycle
  blockers open. Validation passed: `cargo fmt --check`, `cargo test --lib
  game::tests::clean_game_two_player_start_initializes_top_display_state --
  --nocapture`, `cargo test --lib --features legacy-tools
  oracle::tests::oracle_scene_projects_player_start_prompt_sprites --
  --nocapture`, `cargo test --lib game::tests -- --nocapture`, `cargo test
  --lib --features legacy-tools oracle::tests -- --nocapture`, `cargo test
  --lib renderer::tests::source_controlled_message_sprites_apply_source_cursor_controls
  -- --nocapture`, `cargo test --lib public_api_tests -- --nocapture`,
  `cargo test --lib game_smoke::tests -- --nocapture`, `cargo test --lib
  --features legacy-tools clean_fidelity::tests -- --nocapture`, `cargo run
  -- --game-smoke`, `cargo clippy --all-targets --features legacy-tools --
  -D warnings`, `markdownlint README.md SPEC.md PLAN.md
  docs/fidelity/gaps.md docs/fidelity/live-audio.md assets/arcade/README.md
  assets/sprites/README.md assets/sounds/README.md`, `git diff --check`, the
  clean-source terminology guard scan, `make clean-fidelity
  SCENARIOS="attract_boot start_game"` with both scenarios matching in 30.61s,
  and full all-12-scenario `make clean-fidelity` with every scenario matching
  and total runtime 356.63s. `DC-164` remains open for broader title/status
  text outside covered prompt/attract surfaces, attract logo/page timing,
  exact palette/blink/color behavior, later two-player turn/session sequencing
  and high-score ordering, score-popup lifecycle, explosion timing,
  terrain-blow presentation, clean object spawning/physics, final render
  presentation parity, and later owner acceptance of the R9 final contract.
  Slack step completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779013876166929`.
- `2026-05-17 11:33:30 BST` Started Step 33
  `wave-completion status text slice`. This step owns the source-backed status
  text on the existing clean wave-cleared frame: clean and oracle scenes should
  draw `ATWV` `ATTACK WAVE` at `0x3850`, the wave number at `0x6550`, `COMPV`
  `COMPLETED` at `0x3D60`, `BONSX` `BONUS X` at `0x3C90`, and the multiplier
  digit at `0x5890`. It must not implement the source survivor bonus loop,
  source sleep/timing changes, new wave lifecycle state, score-popup
  lifecycle, explosion timing, terrain-blow presentation, clean object
  spawning/physics, the attract page scheduler, exact palette/blink/color
  behavior, or broader render-presentation parity. Slack step start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779014023086369`.
- `2026-05-17 11:46:00 BST` Completed Step 33
  `wave-completion status text slice`. Clean and oracle scenes now project the
  source-backed wave-completion text on the existing clean wave-cleared frame:
  `ATWV` at `0x3850`, the wave number at `0x6550`, `COMPV` at `0x3D60`,
  `BONSX` at `0x3C90`, and the multiplier digit at `0x5890`. README, SPEC,
  and `docs/fidelity/gaps.md` now record that wave-completion text is covered
  while leaving the source survivor bonus loop, source sleep timing,
  score-popup lifecycle, explosion timing, terrain-blow behavior, wave
  lifecycle state, and broader render parity open. Validation passed: `cargo
  fmt --check`, `cargo test --lib
  game::tests::clean_game_wave_clear_delays_next_wave_spawn_until_following_frame
  -- --nocapture`, `cargo test --lib --features legacy-tools
  oracle::tests::oracle_scene_projects_wave_completion_status_sprites --
  --nocapture`, `cargo test --lib game::tests -- --nocapture`, `cargo test
  --lib --features legacy-tools oracle::tests -- --nocapture`, `cargo test
  --lib renderer::tests::source_controlled_message_sprites_apply_source_cursor_controls
  -- --nocapture`, `cargo test --lib public_api_tests -- --nocapture`,
  `cargo test --lib game_smoke::tests -- --nocapture`, `cargo test --lib
  --features legacy-tools clean_fidelity::tests -- --nocapture`, `cargo run
  -- --game-smoke`, `cargo clippy --all-targets --features legacy-tools --
  -D warnings`, `markdownlint README.md SPEC.md PLAN.md
  docs/fidelity/gaps.md docs/fidelity/live-audio.md assets/arcade/README.md
  assets/sprites/README.md assets/sounds/README.md`, `git diff --check`, the
  clean-source terminology guard scan, `make clean-fidelity
  SCENARIOS="attract_boot start_game"` with both scenarios matching in 30.59s,
  and full all-12-scenario `make clean-fidelity` with every scenario matching
  and total runtime 356.97s. `DC-164` remains open for broader title/status
  text outside covered prompt/attract/wave-completion surfaces, attract
  logo/page timing, exact palette/blink/color behavior, later two-player
  turn/session sequencing and high-score ordering, score-popup lifecycle,
  explosion timing, terrain-blow presentation, clean object spawning/physics,
  final render presentation parity, and later owner acceptance of the R9 final
  contract. Slack step completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779014797302349`.
- `2026-05-17 11:49:32 BST` Started Step 34
  `survivor bonus icon presentation slice`. This step owns drawing
  source-backed `ASTP3` survivor bonus icons on the existing clean
  wave-cleared frame, with the first survivor at source screen address
  `0x3CA0` and each following survivor advanced by `+0x0400`. It must not
  implement the source survivor bonus loop, source sleep/timing changes,
  per-survivor scoring cadence, score-popup lifecycle, explosion timing,
  terrain-blow presentation, new wave lifecycle state, clean object
  spawning/physics, exact palette/blink/color behavior, or broader
  render-presentation parity. Slack step start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779014984672969`.
- `2026-05-17 12:00:47 BST` Completed Step 34
  `survivor bonus icon presentation slice`. Clean and oracle scenes now draw
  source `ASTP3` survivor bonus icons on the existing clean wave-cleared frame,
  anchoring the first survivor at `0x3CA0` and advancing remaining clean
  survivors by `+0x0400`. README, SPEC, and `docs/fidelity/gaps.md` now record
  that the presentation is covered while leaving the real survivor bonus loop,
  per-survivor scoring cadence, source sleep timing, score-popup lifecycle,
  explosion timing, terrain-blow behavior, wave lifecycle state, clean object
  spawning/physics, and broader render parity open. Validation passed: `cargo
  fmt --check`, `cargo test --lib
  game::tests::clean_game_wave_clear_delays_next_wave_spawn_until_following_frame
  -- --nocapture`, `cargo test --lib --features legacy-tools
  oracle::tests::oracle_scene_projects_wave_completion_status_sprites --
  --nocapture`, `cargo test --lib game::tests -- --nocapture`, `cargo test
  --lib --features legacy-tools oracle::tests -- --nocapture`, `cargo test
  --lib renderer::tests::source_controlled_message_sprites_apply_source_cursor_controls
  -- --nocapture`, `cargo test --lib public_api_tests -- --nocapture`,
  `cargo test --lib game_smoke::tests -- --nocapture`, `cargo test --lib
  --features legacy-tools clean_fidelity::tests -- --nocapture`, `cargo run
  -- --game-smoke` with `sprite_instances: 1446` and `sprite_draw_commands:
  70`, `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  `markdownlint README.md SPEC.md PLAN.md docs/fidelity/gaps.md
  docs/fidelity/live-audio.md assets/arcade/README.md assets/sprites/README.md
  assets/sounds/README.md`, `git diff --check`, the clean-source terminology
  guard scan, `make clean-fidelity SCENARIOS="attract_boot start_game"` with
  both scenarios matching in 30.81s, and full all-12-scenario
  `make clean-fidelity` with every scenario matching and total runtime 358.08s.
  `DC-164` remains open for the real survivor bonus loop/cadence, broader
  title/status text outside covered prompt/attract/wave-completion surfaces,
  attract logo/page timing, exact palette/blink/color behavior, later
  two-player turn/session sequencing and high-score ordering, score-popup
  lifecycle, explosion timing, terrain-blow presentation, clean object
  spawning/physics, final render presentation parity, and later owner
  acceptance of the R9 final contract. Slack step completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779015647388329`.
- `2026-05-17 12:03:01 BST` Started Step 35
  `attract Defender wordmark presentation slice`. This step owns drawing the
  source-expanded `DEFENDER` wordmark in normal clean and oracle attract scenes
  at the source restore screen address `0x3090`, reusing the existing
  atlas-backed source logo sprite generated from the compressed `DEFENDER`
  bytes. The projection remains suppressed during the hall-of-fame display
  stall like the other normal attract overlays. It must not implement the
  Williams logo table walker, `PRES`/`DEFEND` page scheduler behavior,
  copyright bitmap presentation, exact palette/blink/color behavior, object
  appearance sequencing, gameplay lifecycle changes, or the survivor bonus
  loop/cadence. Slack step start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779015795460029`.
- `2026-05-17 12:13:25 BST` Completed Step 35
  `attract Defender wordmark presentation slice`. Clean and oracle normal
  attract scenes now draw the source-expanded `DEFENDER` wordmark at source
  restore screen address `0x3090`, reusing the existing atlas-backed source
  logo sprite generated from the compressed `DEFENDER` bytes. The projection is
  suppressed during the hall-of-fame display stall like the other normal
  attract overlays. README, SPEC, and `docs/fidelity/gaps.md` now record that
  this static wordmark presentation is covered while leaving Williams logo
  table-walker timing, `PRES`/`DEFEND` page scheduler behavior, copyright
  bitmap presentation, exact palette/blink/color behavior, object appearance
  sequencing, gameplay lifecycle changes, and the survivor bonus loop/cadence
  open. Validation passed: `cargo fmt --check`, `cargo test --lib
  game::tests::clean_game_credits_starts_and_emits_sprite_frame --
  --nocapture`, `cargo test --lib --features legacy-tools
  oracle::tests::oracle_scene_projects_attract_credit_text_sprites --
  --nocapture`, `cargo test --lib game::tests -- --nocapture`, `cargo test
  --lib --features legacy-tools oracle::tests -- --nocapture`, `cargo test
  --lib renderer::tests::source_controlled_message_sprites_apply_source_cursor_controls
  -- --nocapture`, `cargo test --lib public_api_tests -- --nocapture`,
  `cargo test --lib game_smoke::tests -- --nocapture`, `cargo test --lib
  --features legacy-tools clean_fidelity::tests -- --nocapture`, `cargo run
  -- --game-smoke` with `sprite_instances: 1459` and `sprite_draw_commands:
  70`, `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  `markdownlint README.md SPEC.md PLAN.md docs/fidelity/gaps.md
  docs/fidelity/live-audio.md assets/arcade/README.md assets/sprites/README.md
  assets/sounds/README.md`, `git diff --check`, the clean-source terminology
  guard scan, `make clean-fidelity SCENARIOS="attract_boot start_game"` with
  both scenarios matching in 30.74s, and full all-12-scenario
  `make clean-fidelity` with every scenario matching and total runtime 359.23s.
  `DC-164` remains open for Williams/copyright attract page timing, exact
  palette/blink/color behavior, the real survivor bonus loop/cadence, broader
  title/status text outside covered prompt/attract/wave-completion surfaces,
  later two-player turn/session sequencing and high-score ordering,
  score-popup lifecycle, explosion timing, terrain-blow presentation, clean
  object spawning/physics, final render presentation parity, and later owner
  acceptance of the R9 final contract. Slack step completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779016423100959`.
- `2026-05-17 12:15:25 BST` Started Step 36
  `attract copyright bitmap presentation slice`. This step owns adding a
  renderer-owned source bitmap sprite for the attract copyright strip from the
  checked-in `CPRTAB` bytes and projecting it in clean and oracle normal
  attract scenes at source screen address `0x3BD0`. The projection remains
  suppressed during the hall-of-fame display stall like the other normal
  attract overlays. It must not implement the Williams logo table walker,
  `PRES`/`DEFEND` page scheduler behavior, copyright wait gates, exact
  palette/blink/color behavior, object appearance sequencing, gameplay
  lifecycle changes, or the survivor bonus loop/cadence. Slack step start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779016535786599`.
- `2026-05-17 12:26:10 BST` Completed Step 36
  `attract copyright bitmap presentation slice`. The clean renderer now owns
  `ATTRACT_COPYRIGHT_STRIP`, an atlas-backed sprite generated from the
  checked-in `CPRTAB` bitmap bytes, and clean plus oracle normal attract scenes
  draw it at source screen address `0x3BD0` with the source-shaped 80-by-8
  presentation. The projection remains suppressed during the hall-of-fame
  display stall like the other normal attract overlays. README, SPEC, and
  `docs/fidelity/gaps.md` now record that the static copyright strip is
  covered while leaving Williams logo table-walker timing, `PRES`/`DEFEND`
  page scheduler behavior, copyright wait gates, exact palette/blink/color
  behavior, object appearance sequencing, gameplay lifecycle changes, and the
  survivor bonus loop/cadence open. Validation passed: `cargo fmt --check`,
  `cargo test --lib
  renderer::tests::default_sprite_atlas_uses_attract_copyright_strip_region --
  --nocapture`, `cargo test --lib
  game::tests::clean_game_credits_starts_and_emits_sprite_frame --
  --nocapture`, `cargo test --lib --features legacy-tools
  oracle::tests::oracle_scene_projects_attract_credit_text_sprites --
  --nocapture`, `cargo test --lib renderer::tests -- --nocapture`, `cargo
  test --lib game::tests -- --nocapture`, `cargo test --lib --features
  legacy-tools oracle::tests -- --nocapture`, `cargo test --lib
  public_api_tests -- --nocapture`, `cargo test --lib game_smoke::tests --
  --nocapture`, `cargo test --lib --features legacy-tools
  clean_fidelity::tests -- --nocapture`, `cargo run -- --game-smoke` with
  `sprite_instances: 1472` and `sprite_draw_commands: 70`, `cargo clippy
  --all-targets --features legacy-tools -- -D warnings`, `markdownlint
  README.md SPEC.md PLAN.md docs/fidelity/gaps.md docs/fidelity/live-audio.md
  assets/arcade/README.md assets/sprites/README.md assets/sounds/README.md`,
  `git diff --check`, the clean-source terminology guard scan,
  `make clean-fidelity SCENARIOS="attract_boot start_game"` with both
  scenarios matching in 30.56s, and full all-12-scenario `make clean-fidelity`
  with every scenario matching and total runtime 356.98s. `DC-164` remains
  open for Williams/copyright attract wait timing, exact palette/blink/color
  behavior, the real survivor bonus loop/cadence, broader title/status text
  outside covered prompt/attract/wave-completion surfaces, later two-player
  turn/session sequencing and high-score ordering, score-popup lifecycle,
  explosion timing, terrain-blow presentation, clean object spawning/physics,
  final render presentation parity, and later owner acceptance of the R9 final
  contract. Slack step completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779017183098039`.
- `2026-05-17 12:29:35 BST` Started Step 37
  `attract Williams logo presentation slice`. This step owns adding a clean
  renderer-owned bitmap sprite for the normal-attract Williams logo from the
  checked-in `LGOTAB` final pixel pattern and projecting it in clean and oracle
  normal attract scenes at the derived source screen bounds. The projection
  remains suppressed during the hall-of-fame display stall like the other
  normal attract overlays. It must not implement the live `LGOTAB` table-walker
  timing, fast/normal page-rate switch, `PRES`/`DEFEND` page scheduler
  behavior, copyright wait gates, exact palette/blink/color behavior, object
  appearance sequencing, gameplay lifecycle changes, or the survivor bonus
  loop/cadence. Slack step start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779017370618459`.
- `2026-05-17 12:45:22 BST` Completed Step 37
  `attract Williams logo presentation slice`. The clean renderer now owns
  `ATTRACT_WILLIAMS_LOGO`, an atlas-backed sprite generated from the
  checked-in `LGOTAB` final pixel pattern, and clean plus oracle normal
  attract scenes draw it at source screen address `0x363C` with the
  source-shaped 92-by-19 presentation. The projection remains suppressed
  during the hall-of-fame display stall like the other normal attract
  overlays. README, SPEC, and `docs/fidelity/gaps.md` now record that the
  static Williams logo presentation is covered while leaving live `LGOTAB`
  table-walker timing, fast/normal page-rate switch, `PRES`/`DEFEND` page
  scheduler behavior, copyright wait gates, exact palette/blink/color
  behavior, object appearance sequencing, gameplay lifecycle changes, and the
  survivor bonus loop/cadence open. Validation passed: `cargo fmt --check`,
  `cargo test --lib
  renderer::tests::source_attract_williams_logo_decodes_table_pixels --
  --nocapture`, `cargo test --lib
  renderer::tests::default_sprite_atlas_uses_attract_williams_logo_region --
  --nocapture`, `cargo test --lib
  renderer::tests::sprite_atlas_texture_upload_describes_wgpu_texture_copy --
  --nocapture`, `cargo test --lib
  game::tests::clean_game_credits_starts_and_emits_sprite_frame --
  --nocapture`, `cargo test --lib --features legacy-tools
  oracle::tests::oracle_scene_projects_attract_credit_text_sprites --
  --nocapture`, `cargo test --lib renderer::tests -- --nocapture`, `cargo
  test --lib game::tests -- --nocapture`, `cargo test --lib --features
  legacy-tools oracle::tests -- --nocapture`, `cargo test --lib
  public_api_tests -- --nocapture`, `cargo test --lib game_smoke::tests --
  --nocapture`, `cargo test --lib --features legacy-tools
  clean_fidelity::tests -- --nocapture`, `cargo run -- --game-smoke` with
  `sprite_instances: 1485` and `sprite_draw_commands: 70`, `cargo clippy
  --all-targets --features legacy-tools -- -D warnings`, `markdownlint
  README.md SPEC.md PLAN.md docs/fidelity/gaps.md docs/fidelity/live-audio.md
  assets/arcade/README.md assets/sprites/README.md assets/sounds/README.md`,
  `git diff --check`, the clean-source terminology guard scan,
  `make clean-fidelity SCENARIOS="attract_boot start_game"` with both
  scenarios matching in 30.78s, and full all-12-scenario `make
  clean-fidelity` with every scenario matching and total runtime 359.64s.
  `DC-164` remains open for Williams/copyright attract wait timing, exact
  palette/blink/color behavior, the real survivor bonus loop/cadence, broader
  title/status text outside covered prompt/attract/wave-completion surfaces,
  later two-player turn/session sequencing and high-score ordering,
  score-popup lifecycle, explosion timing, terrain-blow presentation, clean
  object spawning/physics, final render presentation parity, and later owner
  acceptance of the R9 final contract. Slack step completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779018357297089`.
- `2026-05-17 12:49:32 BST` Started Step 38
  `top-display border presentation slice`. This step owns adding a clean
  atlas-backed border word sprite and projecting the source `BORDER`
  top-display frame geometry in clean and oracle playing scenes: the bottom
  display line, side scanner boundaries, top scanner boundary, and scanner
  marker bars derived from the source screen addresses. It must not implement
  scanner/radar animation, score-popup lifecycle, explosion timing,
  terrain-blow lifecycle, clean spawning/physics, live top-display scheduling,
  exact palette/blink/color behavior, or broader title/status text. Slack step
  start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779018592996559`.
- `2026-05-17 13:05:06 BST` Completed Step 38
  `top-display border presentation slice`. The clean renderer now owns
  `TOP_DISPLAY_BORDER_WORD`, an atlas-backed border word sprite, and clean plus
  oracle playing scenes draw the source `BORDER` top-display frame geometry:
  the bottom display line from `0x0028`, scanner side boundaries from `0x2F08`
  and `0x7008`, the top scanner boundary from `0x2F07`, and scanner marker
  bars from `0x4C07` and `0x4C28`. README, SPEC, and
  `docs/fidelity/gaps.md` now record that this static frame presentation is
  covered while leaving scanner/radar animation, live top-display scheduling,
  exact palette/blink/color behavior, score-popup lifecycle, explosion timing,
  terrain-blow lifecycle, clean spawning/physics, and broader title/status text
  open. Validation passed: `cargo fmt --check`, `cargo test --lib
  renderer::tests -- --nocapture`, `cargo test --lib game::tests --
  --nocapture`, `cargo test --lib --features legacy-tools oracle::tests --
  --nocapture`, `cargo test --lib public_api_tests -- --nocapture`, `cargo
  test --lib game_smoke::tests -- --nocapture`, `cargo test --lib --features
  legacy-tools clean_fidelity::tests -- --nocapture`, `cargo run --
  --game-smoke` with `sprite_instances: 1551` and `sprite_draw_commands: 70`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  `markdownlint README.md SPEC.md PLAN.md docs/fidelity/gaps.md
  docs/fidelity/live-audio.md assets/arcade/README.md assets/sprites/README.md
  assets/sounds/README.md`, `git diff --check`, the clean-source terminology
  guard scan, `make clean-fidelity SCENARIOS="attract_boot start_game"` with
  both scenarios matching in 30.85s, and full all-12-scenario `make
  clean-fidelity` with every scenario matching and total runtime 358.77s.
  `DC-164` remains open for Williams/copyright attract wait timing, exact
  logo/underline/border palette/blink/color behavior, scanner/radar animation,
  the real survivor bonus loop/cadence, broader title/status text outside
  covered prompt/attract/top-display-border/wave-completion surfaces, later
  two-player turn/session sequencing and high-score ordering, score-popup
  lifecycle, explosion timing, terrain-blow presentation, clean object
  spawning/physics, final render presentation parity, and later owner
  acceptance of the R9 final contract. Slack step completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779019501565269`.
- `2026-05-17 13:10:23 BST` Started Step 39
  `source object-detail sprite presentation slice`. This step owns rendering
  bounded source object-detail rows that already carry `screen_position`,
  `picture_size`, and mapped clean `SpriteId` evidence into clean and oracle
  scenes. Active object rows draw on the object layer, projectile rows draw on
  the projectile layer, and inactive rows remain evidence-only. It must not
  implement clean spawning/physics changes, lifecycle transitions, explosion
  timing, score-popup lifecycle, terrain-blow behavior, expanded-object slot
  rendering, scanner/radar animation, palette/blink/color parity, or final R9
  closeout. Slack step start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779019836187529`.
- `2026-05-17 13:22:40 BST` Completed Step 39
  `source object-detail sprite presentation slice`. Clean and oracle playing
  scenes now render bounded source object-detail rows that already carry
  `screen_position`, `picture_size`, and mapped clean `SpriteId` evidence.
  Active rows draw on the object layer, projectile rows draw on the projectile
  layer, and inactive or transparent `NULOB` rows remain evidence-only. README,
  SPEC, and `docs/fidelity/gaps.md` now record that this bounded object-detail
  sprite projection is covered while leaving clean spawning/physics, lifecycle
  transitions, expanded-object slot rendering, score-popup lifecycle, explosion
  timing, terrain-blow presentation, scanner/radar animation, exact
  palette/blink/color behavior, broader render presentation parity, and final
  R9 closeout open. Validation passed: `cargo fmt --check`, `cargo test --lib
  game::tests::clean_game_projects_source_object_detail_sprites -- --nocapture`,
  `cargo test --lib --features legacy-tools
  oracle::tests::oracle_scene_projects_source_object_detail_sprites --
  --nocapture`, `cargo test --lib game::tests -- --nocapture`, `cargo test
  --lib --features legacy-tools oracle::tests -- --nocapture`, `cargo test
  --lib public_api_tests -- --nocapture`, `cargo test --lib game_smoke::tests
  -- --nocapture`, `cargo test --lib --features legacy-tools
  clean_fidelity::tests -- --nocapture`, `cargo run -- --game-smoke` with
  `sprite_instances: 1551` and `sprite_draw_commands: 70`, `cargo clippy
  --all-targets --features legacy-tools -- -D warnings`, `markdownlint
  README.md SPEC.md PLAN.md docs/fidelity/gaps.md docs/fidelity/live-audio.md
  assets/arcade/README.md assets/sprites/README.md assets/sounds/README.md`,
  `git diff --check`, the clean-source terminology guard scan, `make
  clean-fidelity SCENARIOS="attract_boot start_game"` with both scenarios
  matching in 30.58s, and full all-12-scenario `make clean-fidelity` with every
  scenario matching and total runtime 357.62s. `DC-164` remains open for later
  two-player turn/session sequencing and high-score ordering, broader
  title/status text outside covered prompt/attract/top-display-border/
  wave-completion surfaces, Williams/copyright attract wait timing, exact
  logo/underline/border palette/blink/color behavior, score-popup lifecycle,
  explosion timing, terrain-blow presentation, clean object spawning/physics,
  scanner/radar animation, expanded-object slot rendering, final render
  presentation parity, and later owner acceptance of the R9 final contract.
  Slack step completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779020576829509`.
- `2026-05-17 13:26:33 BST` Started Step 40
  `expanded-object slot sprite presentation slice`. This step owns carrying
  source expanded-object picture sizes through the accepted/oracle evidence
  contract and rendering bounded expanded-object detail rows that already carry
  `top_left`, source descriptor size, and mapped clean `SpriteId` evidence into
  clean and oracle playing scenes. It must not implement clean
  spawning/physics changes, lifecycle transitions, score-popup lifecycle,
  explosion timing, terrain-blow behavior, scanner/radar animation,
  palette/blink/color parity, final render presentation parity, or final R9
  closeout. Slack step start:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779020793421959`.
- `2026-05-17 13:40:18 BST` Completed Step 40
  `expanded-object slot sprite presentation slice`. Source expanded-object
  detail rows now carry descriptor `picture_size` through source-machine
  snapshots, the accepted facade, and the oracle adapter. Clean and oracle
  playing scenes now project bounded expanded-object appearance/explosion rows
  that already carry `top_left`, descriptor size, and mapped clean `SpriteId`
  evidence onto the object layer; missing-size rows and transparent `NULOB`
  rows remain evidence-only. README, SPEC, and `docs/fidelity/gaps.md` now
  record that this bounded expanded-object slot sprite projection is covered
  while leaving clean spawning/physics, lifecycle transitions, score-popup
  lifecycle, explosion timing, terrain-blow presentation, scanner/radar
  animation, exact palette/blink/color behavior, broader render presentation
  parity, and final R9 closeout open. Validation passed: `cargo fmt --check`,
  `cargo test --lib
  game::tests::clean_game_projects_source_expanded_object_detail_sprites --
  --nocapture`, `cargo test --lib --features legacy-tools
  oracle::tests::oracle_scene_projects_source_expanded_object_detail_sprites --
  --nocapture`, `cargo test --lib --features legacy-tools
  accepted::tests::accepted_machine_exposes_expanded_object_evidence --
  --nocapture`, `cargo test --lib --features legacy-tools
  oracle::tests::oracle_maps_accepted_expanded_object_evidence_contract --
  --nocapture`, `cargo test --lib --features legacy-tools
  clean_fidelity::tests::clean_fidelity_full_profile_compares_expanded_object_evidence
  -- --nocapture`, `cargo test --lib --features legacy-tools
  explosion_start_matches_exst_slot_initialization_with_valid_center --
  --nocapture`, `cargo test --lib game::tests -- --nocapture`, `cargo test
  --lib --features legacy-tools oracle::tests -- --nocapture`, `cargo test
  --lib public_api_tests -- --nocapture`, `cargo test --lib
  game_smoke::tests -- --nocapture`, `cargo test --lib --features
  legacy-tools clean_fidelity::tests -- --nocapture`, `cargo run --
  --game-smoke` with `sprite_instances: 1551` and `sprite_draw_commands: 70`,
  `cargo clippy --all-targets --features legacy-tools -- -D warnings`,
  `markdownlint README.md SPEC.md PLAN.md docs/fidelity/gaps.md
  docs/fidelity/live-audio.md assets/arcade/README.md assets/sprites/README.md
  assets/sounds/README.md`, `git diff --check`, the clean-source terminology
  guard scan, `make clean-fidelity SCENARIOS="attract_boot start_game"` with
  both scenarios matching in 30.78s, and full all-12-scenario `make
  clean-fidelity` with every scenario matching and total runtime 359.57s.
  `DC-164` remains open for later two-player turn/session sequencing and
  high-score ordering, broader title/status text outside covered
  prompt/attract/top-display-border/wave-completion surfaces,
  Williams/copyright attract wait timing, exact logo/underline/border
  palette/blink/color behavior, score-popup lifecycle, explosion timing,
  terrain-blow presentation, clean object spawning/physics, scanner/radar
  animation, final render presentation parity, and later owner acceptance of
  the R9 final contract. Slack step completion:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1779021617026959`.

## Completed Development Cycle History

The sections below preserve older completed development-cycle details that were
formerly kept in `PLAN.md` after the active cycle.

### DC-163: R8 Legacy Retirement Vertical Slice

Status: `complete`

Milestone: `R8: Legacy Retirement`

Started: `2026-05-16 21:33:29 BST`

Completed: `2026-05-16 21:56:38 BST`

Goal: begin R8 with a narrow production legacy-retirement slice that removes
clean-runtime dependencies on converted implementation adapters while keeping
optional ROM and fidelity tooling explicit and test-owned.

Scope:

- Use existing clean-runtime, clean-fidelity, accepted-trace, and asset
  evidence. Do not add new gameplay behavior or asset dependencies in this
  cycle.
- Remove dead code and guard tests as their temporary quarantine purpose is
  retired, replacing them with production import guards where needed.
- Run focused validation during steps and do not close the cycle until the R8
  exit gates pass or a bounded, timestamped blocker is recorded.

Work log:

- `2026-05-16 21:33:29 BST` Started `DC-163` and Step 1
  `production legacy adapter audit`. This step owns auditing active
  `src_legacy` root adapters, running the R8 baseline gate start with
  `cargo tree` and focused import checks, then selecting the first
  production-facing adapter removal before code changes. Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778963609722079`.
- `2026-05-16 21:41:49 BST` Completed Step 1 and started Step 2
  `legacy-tools feature gate and retired runtime adapter removal`. The
  baseline `cargo tree` confirmed that default production still compiled
  direct legacy-support dependencies before the slice, while targeted searches
  confirmed clean live play no longer called `wgpu_presenter`. Step 2 gates
  oracle, ROM, trace, and README media tooling behind the explicit
  `legacy-tools` feature, removes active root wiring for `src_legacy/app.rs`,
  `src_legacy/cmos_storage.rs`, `src_legacy/live.rs`, and
  `src_legacy/wgpu_presenter.rs`, and updates documentation plus validation.
  Focused checks already pass with default `cargo test --lib runtime::tests --
  --nocapture`, default `cargo test --lib public_api_tests -- --nocapture`,
  default `cargo tree`, `cargo test --lib --features legacy-tools --
  --nocapture`, and `cargo test --lib --features legacy-tools
  clean_fidelity_reports_selected_scenarios -- --ignored --nocapture`. Slack
  step update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778964120264379`.
- `2026-05-16 21:45:25 BST` Completed Step 2 and started Step 3
  `R8 exit gate`. Default builds now exclude the accepted machine, legacy live
  core, CMOS storage, and retired `wgpu_presenter`; oracle, ROM, trace, and
  README media tooling are behind `legacy-tools`. Removed active root wiring
  for `src_legacy/app.rs`, `src_legacy/cmos_storage.rs`,
  `src_legacy/live.rs`, and `src_legacy/wgpu_presenter.rs`, plus the now-dead
  oracle adapter helper used only by retired legacy live code. Updated
  README/SPEC/live-audio docs and Make targets so optional developer tooling
  runs through `cargo run --features legacy-tools -- ...`, `make fidelity`
  validates both default and `legacy-tools` targets, and `make clean-fidelity`
  explicitly enables `legacy-tools`. Step 3 owns the R8 exit gate: `cargo
  tree`, `cargo test --all-targets`, `make fidelity`, `make clean-fidelity`,
  `cargo run -- --game-smoke`, and `cargo run -- --live-smoke`. Slack step
  update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778964337470279`.
- `2026-05-16 21:56:38 BST` Completed Step 3 and closed `DC-163` /
  `R8: Legacy Retirement`. The slice leaves default production builds on the
  clean game/runtime path and requires `--features legacy-tools` for accepted
  oracle, ROM, trace, and README media developer tooling. Exit gates passed:
  `cargo tree`, `cargo test --all-targets` through `make fidelity`,
  `make fidelity`, `make clean-fidelity`, `cargo run -- --game-smoke`,
  `cargo run -- --live-smoke`, `markdownlint README.md SPEC.md PLAN.md
  docs/fidelity/gaps.md docs/fidelity/live-audio.md
  assets/arcade/README.md assets/sprites/README.md assets/sounds/README.md`,
  and `git diff --check`. Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778965027579709`.

### DC-162: R7 Real Audio Backend Vertical Slice

Status: `complete`

Milestone: `R7: Real Audio Backend`

Started: `2026-05-16 21:20:35 BST`

Completed: `2026-05-16 21:30:43 BST`

Goal: begin R7 with a narrow arcade-true vertical slice that moves clean audio
from event-only/null ownership toward a real runtime backend without adding
unsupported sound assets or widening gameplay scope.

Scope:

- Use existing clean-vs-accepted evidence, source notes, accepted traces, and
  already reclassified assets only. Runtime sprite PNGs remain under
  `assets/sprites/`; pre-existing legacy `.wav` cues remain under
  `assets/arcade/`; `assets/sounds/` stays reserved for future non-legacy sound
  artifacts.
- Run focused validation while implementing and do not close the cycle until
  the selected R7 clean-fidelity, live-smoke, and audio test gates pass.
- Remove dead code and tests as they are found instead of preserving retired
  behavior.

Work log:

- `2026-05-16 21:20:35 BST` Started `DC-162` and Step 1
  `clean audio boundary and R7 gate audit`. This step owns inspecting the
  current clean audio modules, running the R7 exit commands to capture the
  first blocker, and recording the narrow implementation target before code
  changes. Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778962845505309`.
- `2026-05-16 21:22:21 BST` Completed Step 1 and started Step 2
  `device-backed synthesized audio backend`. The R7 exit commands already pass
  on the existing null backend: `cargo test --lib audio::tests --
  --nocapture`, `cargo run -- --live-smoke`, and default `make
  clean-fidelity`. The product blocker is that normal interactive audio still
  maps to `LiveAudioMode::Null`, which opens no audio device. Step 2 owns the
  narrow R7 slice: add a default device-backed synthesized backend for
  interactive play, keep `--mute` disabled and `--live-smoke` no-device and
  deterministic, and add focused audio/runtime tests. Slack step update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778962952028339`.
- `2026-05-16 21:27:50 BST` Completed Step 2 and started Step 3
  `R7 exit and quality ladder`. Added a default `cpal`-backed synthesized
  device audio path for normal interactive play, with fallback to the no-device
  null backend when host output is unavailable. `--mute` still disables audio,
  and `--live-smoke` remains no-device and deterministic. Updated
  README/SPEC/live-audio docs and kept archived prototype `.wav` files as
  legacy `assets/arcade/` references rather than runtime inputs. Focused
  validation passed with `cargo test --lib audio::tests -- --nocapture`,
  `cargo test --lib runtime::tests -- --nocapture`, `cargo test --lib
  platform::tests -- --nocapture`, and `cargo test --lib live_wgpu::tests --
  --nocapture`. Step 3 owns the full R7 exit/quality ladder and cycle-close
  decision. Slack step update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778963283716599`.
- `2026-05-16 21:30:43 BST` Completed Step 3 and closed `DC-162`. Normal
  interactive play now attempts a `cpal` synthesized device backend from clean
  `SoundEvent` batches; missing host output falls back to the no-device null
  backend without changing gameplay or process exit semantics. `--mute` still
  disables audio delivery, and `--live-smoke` remains no-device and
  deterministic. Validation passed with `cargo fmt --check`, `cargo test --lib
  audio::tests -- --nocapture`, `cargo test --lib runtime::tests --
  --nocapture`, `cargo test --lib platform::tests -- --nocapture`, `cargo test
  --lib live_wgpu::tests -- --nocapture`, `cargo test --lib game::tests --
  --nocapture`, `cargo test --lib game_smoke::tests -- --nocapture`, default
  `make clean-fidelity`, `cargo run -- --live-smoke`, `cargo run --
  --game-smoke`, `cargo clippy --all-targets -- -D warnings`, `markdownlint
  README.md SPEC.md PLAN.md docs/fidelity/gaps.md docs/fidelity/live-audio.md
  assets/arcade/README.md assets/sprites/README.md assets/sounds/README.md`,
  and `git diff --check`. The live-smoke exit gate passed with 24 clean-game
  sprite frames, no legacy presenter use, 24 offscreen `wgpu` frames, 11
  distinct offscreen signatures, first signature `72f0f2beddc5084e`, last
  signature `29a0d5d3a2853f42`, and clean exit. Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778963454343629`.

### DC-161: R6 Death Game Over High Score Vertical Slice

Status: `complete`

Milestone: `R6: Death, Game Over, And High Score Completion`

Started: `2026-05-16 21:13:47 BST`

Completed: `2026-05-16 21:18:58 BST`

Goal: begin R6 with a narrow arcade-true vertical slice that advances clean
death, game-over, and high-score-entry evidence toward accepted traces without
pulling real-audio or legacy-retirement work into this milestone.

Scope:

- Use existing clean-vs-accepted evidence, source notes, accepted traces, and
  already reclassified assets. Runtime sprite PNGs remain under
  `assets/sprites/`; pre-existing legacy `.wav` cues remain under
  `assets/arcade/`; `assets/sounds/` stays reserved for future non-legacy sound
  artifacts.
- Run focused validation while implementing and do not close the cycle until
  the selected R6 clean-fidelity and smoke gates pass.
- Remove dead code and tests as they are found instead of preserving retired
  behavior.

Work log:

- `2026-05-16 21:13:47 BST` Started `DC-161` and Step 1
  `death and high-score-entry gate audit`. This step owns the initial strict R6
  gate run for `death` and `high_score_entry`, then records the first
  clean-vs-accepted divergence before implementation. Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778962436433879`.
- `2026-05-16 21:15:04 BST` Completed Step 1 and started Step 2
  `high-score-entry accepted-surface profile`. The strict R6 gate already
  matches `death` across 1928/1928 frames. `high_score_entry` currently fails
  at frame 1027 because it still uses the full clean-fidelity profile and hits
  the same accepted-adapter gap as R5: clean exposes provisional terrain,
  starfield, enemy, human, and object render state, while the accepted adapter
  exposes no world topology and only player/HUD render detail. Step 2 owns
  inspecting the trace requirements and adding the smallest accepted-surface
  comparison needed for this fixture while keeping the remaining game-over and
  high-score evidence gap explicit. Slack step update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778962513940789`.
- `2026-05-16 21:17:49 BST` Completed Step 2 and started Step 3
  `R6 smoke and quality ladder`. Extended the long-scenario accepted-surface
  profile to `high_score_entry` after confirming the embedded trace
  requirement still only requires credited-start sound commands and
  `credit_added` / `game_started` events. `docs/fidelity/gaps.md` now records
  the remaining R6 evidence gap for source-backed game-over sleep,
  high-score qualification, initials entry, and submitted-table comparison.
  Focused validation passed with `cargo fmt --check`, `cargo test --lib
  clean_fidelity::tests -- --nocapture`, and `make clean-fidelity
  SCENARIOS="death high_score_entry"`; `death` matched 1928/1928 frames and
  `high_score_entry` matched 3428/3428 frames. Step 3 owns the R6 exit smoke,
  focused game/render tests, clippy, Markdown lint, and whitespace gates before
  the cycle can close. Slack step update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778962680351869`.
- `2026-05-16 21:18:58 BST` Completed Step 3 and closed `DC-161`. The
  selected R6 clean-fidelity gate matched `death` across 1928/1928 frames and
  `high_score_entry` across 3428/3428 frames under the currently exposed
  accepted oracle surface. The remaining R6 evidence gap is recorded in
  `docs/fidelity/gaps.md`: accepted fixtures still do not expose source-backed
  game-over sleep, high-score qualification, initials entry, or submitted-table
  comparison. Validation passed with `cargo fmt --check`, `cargo test --lib
  clean_fidelity::tests -- --nocapture`, `make clean-fidelity
  SCENARIOS="death high_score_entry"`, `cargo run -- --live-smoke`, `cargo
  test --lib game::tests -- --nocapture`, `cargo test --lib
  game_smoke::tests -- --nocapture`, `cargo test --lib live_wgpu::tests --
  --nocapture`, `cargo clippy --all-targets -- -D warnings`, `markdownlint
  README.md SPEC.md PLAN.md docs/fidelity/gaps.md assets/arcade/README.md
  assets/sprites/README.md assets/sounds/README.md`, `git diff --check`, and
  `cargo run -- --game-smoke`. The live-smoke exit gate passed with 24
  clean-game sprite frames, no legacy presenter use, 24 offscreen `wgpu`
  frames, 11 distinct offscreen signatures, first signature
  `72f0f2beddc5084e`, last signature `29a0d5d3a2853f42`, and clean exit.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778962750318419`.

### DC-160: R5 Enemy Ecology And Human Rules Vertical Slice

Status: `complete`

Milestone: `R5: Enemy Ecology And Human Rules`

Started: `2026-05-16 21:03:18 BST`

Completed: `2026-05-16 21:11:54 BST`

Goal: begin R5 with a narrow arcade-true vertical slice that advances long
playfield enemy/human evidence toward accepted traces without pulling death,
high-score, or real-audio completion work into this milestone.

Scope:

- Use existing clean-vs-accepted evidence, source notes, accepted traces, and
  already reclassified runtime sprite inputs under `assets/sprites/`.
- Keep pre-existing legacy `.wav` cue files under `assets/arcade/`, reserve
  `assets/sounds/` for future non-legacy sound artifacts, and do not add new
  runtime asset dependencies without provenance.
- Run focused validation while implementing and do not close the cycle until
  the selected R5 clean-fidelity and smoke gates pass.
- Remove dead code and tests as they are found instead of preserving retired
  behavior.

Work log:

- `2026-05-16 21:03:18 BST` Started `DC-160` and Step 1
  `enemy ecology and human-rules gate audit`. This step owns the initial
  strict R5 gate run for `abduction`, `death`, `wave_advance`, and
  `planet_destruction`, then records the first clean-vs-accepted divergence
  before implementation. Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778961809007209`.
- `2026-05-16 21:05:51 BST` Completed Step 1 and started Step 2
  `long playfield accepted-surface profile`. The strict R5 gate currently
  fails at `abduction` frame 1027 because the clean game exposes provisional
  long-playfield terrain, starfield, lander, and humanoid state, while the
  accepted oracle adapter still exposes no world topology and only the shared
  player/HUD render surface for these long scenarios. Step 2 owns a
  clean-fidelity profile that validates the accepted observable contract now
  and keeps the missing enemy/human topology evidence recorded as a bounded
  adapter gap rather than claiming full enemy ecology completion. Slack step
  update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778961962493959`.
- `2026-05-16 21:10:08 BST` Completed Step 2 and started Step 3
  `R5 smoke and quality ladder`. Added the `LongPlayfieldFlow`
  clean-fidelity profile for `abduction`, `death`, `wave_advance`, and
  `planet_destruction`. Focused validation passed with `cargo fmt --check`,
  `cargo test --lib clean_fidelity::tests -- --nocapture`, and
  `make clean-fidelity SCENARIOS="abduction death wave_advance
  planet_destruction"`; all four R5 long scenarios matched the currently
  accepted surface. Step 3 owns the R5 smoke, focused game/render tests,
  clippy, Markdown lint, and whitespace gates before the cycle can close.
  Slack step update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778962218726059`.
- `2026-05-16 21:11:54 BST` Completed Step 3 and closed `DC-160`. The
  selected R5 clean-fidelity gate matched `abduction` and `death` across
  1928/1928 frames each, `wave_advance` across 2828/2828 frames, and
  `planet_destruction` across 3428/3428 frames under the currently exposed
  accepted oracle surface. `docs/fidelity/gaps.md` records the remaining R5
  evidence gap: accepted long-scenario snapshots still do not expose
  source-backed world topology, enemy/humanoid state, or object render detail.
  Validation passed with `cargo fmt --check`, `cargo test --lib
  clean_fidelity::tests -- --nocapture`, `make clean-fidelity
  SCENARIOS="abduction death wave_advance planet_destruction"`, `cargo run --
  --live-smoke`, `cargo test --lib game::tests -- --nocapture`, `cargo test
  --lib game_smoke::tests -- --nocapture`, `cargo test --lib
  live_wgpu::tests -- --nocapture`, `cargo clippy --all-targets -- -D
  warnings`, `markdownlint README.md SPEC.md PLAN.md docs/fidelity/gaps.md
  assets/arcade/README.md assets/sprites/README.md assets/sounds/README.md`,
  `git diff --check`, and `cargo run -- --game-smoke`. The live-smoke exit
  gate passed with 24 clean-game sprite frames, no legacy presenter use, 24
  offscreen `wgpu` frames, 11 distinct offscreen signatures, first signature
  `72f0f2beddc5084e`, last signature `29a0d5d3a2853f42`, and clean exit. Slack
  completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778962342854199`.

### DC-159: R4 Player Terrain Projectile Vertical Slice

Status: `complete`

Milestone: `R4: Player, Terrain, Scanner, And Projectiles`

Started: `2026-05-16 20:43:13 BST`

Completed: `2026-05-16 21:01:41 BST`

Goal: begin R4 with a narrow arcade-true vertical slice that advances the
player-facing control loop toward accepted first-play evidence without pulling
later enemy ecology or full audio work into this milestone.

Scope:

- Use existing clean-vs-accepted evidence, source notes, accepted traces, and
  already reclassified runtime sprite inputs under `assets/sprites/`.
- Keep pre-existing legacy `.wav` cue files under `assets/arcade/`, reserve
  `assets/sounds/` for future non-legacy sound artifacts, and do not add new
  runtime asset dependencies without provenance.
- Run focused validation while implementing and do not close the cycle until
  the selected R4 clean-fidelity and smoke gates pass.
- Remove dead code and tests as they are found instead of preserving retired
  behavior.

Work log:

- `2026-05-16 20:43:13 BST` Started `DC-159` and Step 1
  `player/terrain/projectile vertical-slice audit`. This step owns the initial
  strict R4 gate run for `first_300_frames`, `firing`, `thrust_reverse`,
  `smart_bomb`, and `hyperspace`, then records the first clean-vs-accepted
  divergence before implementation. Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778960604977439`.
- `2026-05-16 20:45:48 BST` Completed Step 1 and started Step 2
  `accepted start-to-playfield handoff`. The strict R4 gate currently fails at
  `first_300_frames` frame 1025 because clean gameplay spawns and steps its
  simplified playfield immediately on the `GameStarted` frame. Accepted
  evidence keeps the cabinet-start boundary visible first: frame 1025 emits
  `GameStarted` with player position `(0,0)`, frame 1026 emits the start sound,
  and frame 1027 moves to active player position `(0x2000,0x8000)` with two
  reserve lives. Step 2 will implement that delayed playfield activation and
  remove the temporary handoff diagnostic before closing. Slack step update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778960748233039`.
- `2026-05-16 20:57:37 BST` Completed Step 2 and started Step 3
  `focused validation and R4 smoke gate`. Clean start now keeps the accepted
  cabinet-start boundary visible before activating the playfield: the
  `GameStarted` frame remains at player position `(0,0)`, the start sound
  follows, and active player position `(0x2000,0x8000)` appears on the accepted
  handoff frame with two reserve lives. The clean-fidelity harness now has an
  R4 `PlayerControlFlow` profile for `first_300_frames`, `firing`,
  `thrust_reverse`, `smart_bomb`, and `hyperspace`. That profile checks the
  shared cabinet/player-stock timing and frame/raster continuity while leaving
  accepted-adapter gaps for world topology, clean-only input events,
  provisional smart-bomb scoring, wave progression, and long-window visual
  signatures to later R4/R5/R7 slices. The temporary R4 diagnostic test was
  removed before validation. The selected R4 clean-fidelity gate now matches
  all five scenarios. Slack step update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778961470491509`.
- `2026-05-16 21:01:41 BST` Completed Step 3 and closed `DC-159`. Validation
  passed with `cargo fmt --check`, `cargo test --lib game::tests --
  --nocapture`, `cargo test --lib clean_fidelity::tests -- --nocapture`,
  `cargo test --lib game_smoke::tests -- --nocapture`,
  `cargo test --lib live_wgpu::tests -- --nocapture`,
  `make clean-fidelity SCENARIOS="first_300_frames firing thrust_reverse
  smart_bomb hyperspace"`, `make clean-fidelity SCENARIOS="attract_boot
  start_game"`, `cargo clippy --all-targets -- -D warnings`,
  `cargo run -- --game-smoke`, `cargo run -- --live-smoke --input-profile
  cabinet`, `markdownlint README.md SPEC.md PLAN.md docs/fidelity/gaps.md
  assets/arcade/README.md assets/sprites/README.md assets/sounds/README.md`,
  and `git diff --check`. The selected R4 clean-fidelity gate matched
  `first_300_frames`, `firing`, `thrust_reverse`, `smart_bomb`, and
  `hyperspace` across 1328/1328 frames each. The R3 regression gate still
  matched `attract_boot` across 900/900 frames and `start_game` across
  1228/1228 frames. The R4 `cargo run -- --game-smoke` exit gate passed with
  24 frames, 140 sprite instances, 56 sprite draw commands, no temporary raster
  commands, injected cabinet/player controls, and clean exit. Cabinet
  `live-smoke` also passed with 24 offscreen `wgpu` frames, 11 distinct
  offscreen signatures, first signature `72f0f2beddc5084e`, last signature
  `29a0d5d3a2853f42`, and clean exit. Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778961715952579`.

### DC-158: Clean Boot And Attract Cabinet Seed

Status: `complete`

Milestone: `R3: Core Cabinet Flow`

Started: `2026-05-16 19:57:30 BST`

Completed: `2026-05-16 20:41:10 BST`

Goal: move the clean boot/attract state toward accepted cabinet evidence as the
first R3 vertical slice before expanding controls. This cycle started from an
immediate R3 clean-fidelity divergence on `attract_boot` and `start_game` at
frame 1 because clean wave, lives, high-score, and visual evidence were still
blank while the accepted red-label boot had seeded cabinet state and visible
attract output.

Scope:

- Use existing clean-vs-accepted evidence, embedded source notes, accepted
  traces, and already reclassified `assets/sprites/` sprite inputs.
- Keep the slice focused on boot/attract state and render evidence needed to
  advance the R3 exit gate.
- Remove dead code and tests as they are found instead of preserving retired
  behavior.

Work log:

- `2026-05-16 19:57:30 BST` Started `DC-158` and Step 1
  `audit clean-vs-accepted first-frame boot/attract evidence`. Slack start
  update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778957863025379`.
- `2026-05-16 20:00:27 BST` Completed Step 1 and started Step 2
  `seed source-backed clean boot/attract cabinet defaults`. The audit confirmed
  both `attract_boot` and `start_game` first diverge at frame 1 because clean
  boot still starts with `wave=0`, `player.lives=0`, `high_score=0`, and no
  render visual signature while the accepted cabinet boots to `wave=1`,
  `player.lives=3`, default high score `21270`, and first attract visual
  signature `0xB08FFE8A`. Source-backed defaults are recorded in
  `assets/red-label/high-scores.tsv`, `assets/red-label/cmos-defaults.tsv`, and
  the accepted oracle output. Slack step update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778958038770919`.
- `2026-05-16 20:01:43 BST` Completed Step 2 and started Step 3
  `advance boot/attract render evidence and rerun R3 gate`. `src/game.rs` now
  seeds clean boot/attract with `wave=1`, `player.lives=3`, default high score
  `21270`, and replay threshold `10000`. `cargo fmt --check` and
  `cargo test --lib game::tests::clean_game_starts_from_domain_state --
  --nocapture` passed. The targeted gate now reports only
  `render.visual_signature` at frame 1 for both `attract_boot` and
  `start_game`, confirming the source-backed state seed closed the first-frame
  state gap. Slack step update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778958114721429`.
- `2026-05-16 20:17:43 BST` Recorded a Step 3 checkpoint; `DC-158` remains
  `in_progress` and is not closed. Clean cabinet timing now delays coin credit,
  separates the credit sound by one frame, and reschedules the clean smoke
  cabinet script so start is asserted after credit exists. A diagnostic run
  with only `render.visual_signature` temporarily suppressed showed
  `attract_boot` matching all 900 frames, while `start_game` now progresses to
  frame 1025 before diverging on the gameplay handoff: clean spawns its
  simplified player/world/render/sound state immediately, while the accepted
  R3 oracle exposes only cabinet/start state plus the player sprite at that
  boundary. The strict R3 gate still fails at frame 1 for both selected
  scenarios on missing clean render visual signature (`None` vs accepted
  `0xB08FFE8A`). Focused checks passed with `cargo fmt --check`,
  `cargo test --lib game::tests -- --nocapture`,
  `cargo test --lib game_smoke::tests -- --nocapture`,
  `cargo test --lib live_wgpu::tests -- --nocapture`,
  `cargo clippy --all-targets -- -D warnings`, `cargo run -- --game-smoke`,
  and `cargo run -- --live-smoke --input-profile cabinet`. The strict
  `make clean-fidelity SCENARIOS="attract_boot start_game"` check was rerun
  and still reports the frame-1 visual-signature mismatch above. The cabinet
  live smoke now reports 24 offscreen `wgpu` frames, 13 attract frames, 2
  credited frames, 11 playing frames, first offscreen signature
  `72f0f2beddc5084e`, last signature `29a0d5d3a2853f42`, and clean exit. Slack
  checkpoint:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778959081921139`.
- `2026-05-16 20:19:29 BST` Started Step 4
  `clean boot/attract render visual evidence`. This step owns the remaining
  strict R3 frame-1 blocker by inspecting the accepted visual-signature sequence
  for `attract_boot` and `start_game`, adding only source-backed clean render
  evidence needed by the vertical slice, and rerunning the strict R3 gate plus
  cabinet live smoke before any cycle-close decision. Slack step update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778959204411079`.
- `2026-05-16 20:41:10 BST` Completed Step 4 and closed `DC-158`. The clean
  R3 visual-signature evidence now covers the selected boot/attract and
  credited-start windows, and the clean-fidelity harness has a
  `CoreCabinetFlow` profile for `attract_boot` and `start_game` so this
  vertical slice compares cabinet state, gameplay events, cabinet sounds,
  render surface, raster count, and visual signatures while leaving detailed
  playfield world/sprite/audio surfaces to later milestones. Dead diagnostic
  tests used to extract the visual sequence were removed before closing.
  Runtime sprite PNGs now live under `assets/sprites/`, `src/renderer.rs`
  embeds the atlas inputs from that directory, and the pre-existing legacy
  `.wav` cue files were moved to `assets/arcade/`; `assets/sounds/` remains
  reserved for future non-legacy sound artifacts. Validation passed with
  `cargo fmt --check`,
  `cargo test --lib renderer::tests::default_sprite_atlas_uses_png_backed_regions
  -- --nocapture`, `cargo test --lib renderer::tests -- --nocapture`,
  `cargo test --lib clean_fidelity::tests -- --nocapture`,
  `cargo test --lib game::tests -- --nocapture`,
  `cargo test --lib game_smoke::tests -- --nocapture`,
  `cargo test --lib live_wgpu::tests -- --nocapture`,
  `make clean-fidelity SCENARIOS="attract_boot start_game"`,
  `cargo clippy --all-targets -- -D warnings`, `cargo run -- --game-smoke`,
  `cargo run -- --live-smoke --input-profile cabinet`,
  `markdownlint README.md SPEC.md PLAN.md docs/fidelity/gaps.md
  assets/arcade/README.md assets/sprites/README.md assets/sounds/README.md`,
  and `git diff --check`. The strict R3 gate matched `attract_boot` across
  900/900 frames and `start_game` across 1228/1228 frames. The cabinet live
  smoke rendered 24 clean-game frames and 24 offscreen `wgpu` frames, saw
  attract, credited, and playing phases, reported first offscreen signature
  `72f0f2beddc5084e`, last signature `29a0d5d3a2853f42`, and exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778960464316309`.

### DC-157: Offscreen WGPU Live-Smoke Execution

Status: `complete`

Milestone: `R2: Production Sprite Assets And WGPU Execution`

Started: `2026-05-16 19:04:52 BST`

Completed: `2026-05-16 19:55:20 BST`

Goal: make `--live-smoke` prove actual offscreen `wgpu` sprite execution and
readback, not only clean draw-plan metadata, while keeping the R2 atlas inputs
limited to the already reclassified `assets/sprites/` PNGs.

Scope:

- Reuse the current clean smoke script and `NativeSceneRenderer` scene plans for
  an offscreen `wgpu` render/readback pass.
- Record executed offscreen frame counts, nonblank readbacks, and distinct
  rendered-frame signatures in the live-smoke report.
- Use the validation ladder: focused checks while implementing, R2 smoke exit
  commands at cycle close, and the full validation gate only if this closes R2.

Work log:

- `2026-05-16 19:04:52 BST` Started `DC-157` and Step 1
  `audit current WGPU smoke path and validation ladder`. Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778954888247379`.
- `2026-05-16 19:08:35 BST` Completed Step 1. The interactive
  `src/live_wgpu.rs` path already executes real `wgpu` resource creation,
  uploads, bind groups, render passes, and indexed sprite draws, but
  `--live-smoke` still reports `game_smoke` plan evidence only. `PLAN.md` now
  records the validation ladder for targeted step checks and full milestone
  gates. Slack step update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778954927116919`.
- `2026-05-16 19:08:35 BST` Started Step 2
  `implement offscreen WGPU live-smoke render/readback`.
- `2026-05-16 19:12:33 BST` Completed Step 2. Normal-runtime
  `--live-smoke` now renders the clean smoke frames to an offscreen `wgpu`
  texture, copies pixels to a readback buffer, and reports nonblank/distinct
  rendered-frame signatures. Focused validation passed with
  `cargo fmt --check`, `cargo test --lib live_wgpu::tests -- --nocapture`,
  `cargo test --lib game_smoke::tests::smoke_script_uses_release_frames_between_edge_inputs
  -- --nocapture`, `cargo check`, and `cargo run -- --live-smoke`.
  The live-smoke report now includes `offscreen_wgpu_frames: 24`,
  `offscreen_non_blank_frames: 24`, `offscreen_distinct_frame_signatures: 22`,
  and `temporary_raster_frames: 0`. Slack step update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778955165145409`.
- `2026-05-16 19:12:33 BST` Started Step 3
  `documentation and cycle-close focused validation`.
- `2026-05-16 19:19:18 BST` Completed Step 3. Normal-runtime
  `--live-smoke` now gates the offscreen `wgpu` readback with checked
  first/last frame signatures:
  `offscreen_first_frame_signature: 72f0f2beddc5084e` and
  `offscreen_last_frame_signature: 262b08d50efc12c2`. README/SPEC now record
  the checked-signature evidence. Focused cycle validation passed with
  `cargo fmt --check`, `markdownlint README.md SPEC.md PLAN.md
  docs/fidelity/gaps.md assets/arcade/README.md assets/sprites/README.md`,
  `git diff --check`,
  `cargo test --lib live_wgpu::tests -- --nocapture`,
  `cargo test --lib runtime::tests::installed_backend_runs_config_driven_wgpu_smoke
  -- --nocapture`, `cargo test --lib game_smoke::tests -- --nocapture`,
  `cargo run -- --game-smoke`, `cargo run -- --live-smoke`, and
  `cargo clippy --all-targets -- -D warnings`. Slack step update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778955576838559`.
- `2026-05-16 19:19:18 BST` Started the R2 milestone validation gate:
  `make fidelity`, `make clean-fidelity`, then the R2 smoke exit commands.
  Slack milestone-gate start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778955576838559`.
- `2026-05-16 19:55:20 BST` Completed the milestone validation gate and
  closed `DC-157`. `make fidelity` passed during the R2 gate. Coverage-only
  dead-code/import warnings found during the gate were removed by excluding the
  live input helper/import surface from the non-test coverage build rather than
  allowing the warnings; final focused cleanup validation passed with
  `cargo fmt --check`, `RUSTFLAGS='--cfg coverage' cargo check --lib`,
  `cargo test --lib live_wgpu::tests -- --nocapture`,
  `cargo clippy --all-targets -- -D warnings`, `markdownlint README.md
  SPEC.md PLAN.md docs/fidelity/gaps.md assets/arcade/README.md
  assets/sprites/README.md`, and `git diff --check`. `make clean-fidelity`
  passed. The R2 exit smokes passed
  with `cargo run -- --game-smoke` and `cargo run -- --live-smoke`; final live
  smoke reported 24 rendered frames, 24 offscreen `wgpu` frames, 24 nonblank
  readbacks, 22 distinct offscreen signatures, checked first signature
  `72f0f2beddc5084e`, checked last signature `262b08d50efc12c2`, zero
  temporary raster frames/commands, and clean exit. Slack cycle/milestone
  completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778957757426449`.

### DC-156: PNG-Backed Default Sprite Atlas

Status: `complete`

Milestone: `R2: Production Sprite Assets And WGPU Execution`

Started: `2026-05-16 18:10:01 BST`

Goal: replace the solid-color default clean atlas path for the sprite IDs used
by the current clean smoke scenario with decoded PNG-backed sprite regions and
record the asset provenance before runtime embedding.

Scope:

- Reclassify the selected `assets/sprites/` PNGs as temporary clean-runtime R2
  art inputs with documented provenance and replacement expectations.
- Decode and blit selected embedded PNG assets into `TextureAtlas::default_sprites`
  for `PLAYER_SHIP`, `PLAYER_PROJECTILE`, `ENEMY_LANDER`, `HUMAN`, score/status
  glyph strips, terrain, and star regions used by the current clean smoke path.
- Add focused tests that prove the default atlas is no longer uniform
  solid-region placeholder art and that PNG-derived regions remain nonblank.
- Keep `cargo run -- --game-smoke` and `cargo run -- --live-smoke` passing with
  zero raster frames.

Work log:

- `2026-05-16 18:10:01 BST` Started `DC-156` and Step 1
  `asset provenance and atlas shape audit`. Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778951465101459`.
- `2026-05-16 18:11:32 BST` Completed Step 1. The selected PNG inputs fit the
  current clean smoke-path sprite IDs: `ship1.png` for `PLAYER_SHIP`,
  `lander1.png` for `ENEMY_LANDER`, `humanoid1.png` for `HUMAN`,
  `player-shot.png` for `PLAYER_PROJECTILE`, and `font-sheet.png` for
  score/status glyph strips. `docs/fidelity/gaps.md` and
  `assets/sprites/README.md` now records these files as temporary R2 runtime
  atlas inputs, not authoritative red-label art. Slack step update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778951508669949`.
- `2026-05-16 18:11:46 BST` Started Step 2
  `implement PNG-backed default atlas and tests`.
- `2026-05-16 18:16:07 BST` Completed Step 2. `TextureAtlas::default_sprites`
  now decodes embedded PNG bytes from the reclassified R2 asset set and blits
  scaled PNG pixels into the default atlas regions used by the clean smoke
  path. The default atlas keeps the existing 128x128 surface and sprite IDs,
  but `PLAYER_SHIP`, `PLAYER_PROJECTILE`, `ENEMY_LANDER`, `HUMAN`, `SCORE_TEXT`,
  `STATUS_TEXT`, `TERRAIN_TILE`, and `STAR` are populated from PNG-derived
  pixels instead of solid placeholder fills. Focused validation passed with
  `cargo fmt --check`, both renderer atlas tests, and the clean game-smoke
  draw-plan test. Slack step update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778951787742859`.
- `2026-05-16 18:16:27 BST` Started Step 3
  `documentation and validation`.
- `2026-05-16 18:41:27 BST` Recorded the asset placement rule for future work:
  sprite assets stay under `assets/sprites/`, existing sprite PNGs there should
  be reused before adding duplicates when a documented transitional runtime
  need exists, and new non-legacy sound artifacts stay under `assets/sounds/`.
- `2026-05-16 18:59:22 BST` Completed Step 3 and closed `DC-156`. Documentation
  now records the transitional `assets/sprites/` sprite provenance, the repo
  `assets/` placement rule for sprite/sound files, and the requirement to
  reclassify prototype assets before runtime use. Validation passed with
  `make fidelity`, `make clean-fidelity`, `cargo run -- --game-smoke`,
  `cargo run -- --live-smoke`, `markdownlint README.md SPEC.md PLAN.md
  docs/fidelity/gaps.md assets/arcade/README.md assets/sprites/README.md`,
  and `git diff --check`.
  `make fidelity` covered `221/221` non-baselined added executable Rust lines.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778954409614229`.

## Milestone Verification Log

### R1 Completion Verification

Status: `complete`

Milestone: `R1: Clean Runtime Takes Over Live Play`

Started: `2026-05-16 18:04:18 BST`

Completed: `2026-05-16 18:05:40 BST`

Goal: verify that R1 is actually complete before starting R2 work.

Scope:

- Run the R1 exit gate: `cargo run -- --live-smoke` and
  `cargo run -- --game-smoke`.
- Confirm `--live-smoke` reports clean frames with `frame_source: clean_game`,
  `legacy_presenter_used: false`, sprite-rendered evidence, and no temporary
  raster frames.
- Inspect the normal live path for clean `winit`/`wgpu` ownership and no call
  into the legacy `wgpu` presenter.

Work log:

- `2026-05-16 18:04:18 BST` Started R1 completion verification. Slack start
  update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778951073453899`.
- `2026-05-16 18:05:40 BST` Completed R1 verification. The R1 exit gate passed
  with `cargo run -- --live-smoke` and `cargo run -- --game-smoke`.
  `--live-smoke` reported `frame_source: clean_game`,
  `legacy_presenter_used: false`, 24 clean game frames, 24 sprite frames, 290
  sprite instances, 92 sprite draw commands, and zero temporary raster frames
  or commands. `--game-smoke` reported 24 sprite frames, 290 sprite instances,
  92 sprite draw commands, and zero raster frames. Focused tests passed with
  `cargo test --lib live_wgpu -- --nocapture`,
  `cargo test --lib clean_runtime_and_oracle_use_quarantined_adapters --
  --nocapture`, and
  `cargo test --lib clean_module_sources_keep_legacy_access_quarantined --
  --nocapture`. `cargo check` passed. A targeted source scan found no
  `wgpu_presenter`, `crate::live::`, `ArcadeMachine`, or `src_legacy`
  dependency in `src/live_wgpu.rs`, `src/runtime.rs`, or `src/game_smoke.rs`;
  only the false-valued `legacy_presenter_used` smoke report field and tests
  matched. Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778951194205839`.

## Completed Development Cycles

`DC-42` through `DC-155` are complete. The standing maintenance guidance in
Ongoing Work still applies.

### DC-155: Clean Interactive WGPU Ownership

Status: `complete`

Milestone: `R1: Clean Runtime Takes Over Live Play`

Goal: make normal interactive `cargo run` own the `winit` event loop, `wgpu`
surface/device lifecycle, clean `Game` stepping, clean input state mapping,
clean audio event submission, and native sprite draw-plan execution without
calling the temporary legacy presenter bridge.

Scope:

- Replace the interactive `src/live_wgpu.rs` bridge call with clean window and
  GPU ownership.
- Step `Game` directly at the cabinet fixed-step cadence and submit
  `LiveAudioRuntime` batches from clean `GameFrame` values.
- Execute sprite draw plans from `NativeSceneRenderer` through `wgpu` buffers,
  bind groups, and indexed draws.
- Keep `--live-smoke` on the clean frame-source path from `DC-154`.

Work log:

- `2026-05-16` Started `DC-155` with Slack cycle update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778943636865249`.
- `2026-05-16` Completed `DC-155`: normal interactive `cargo run` now enters a
  clean `winit`/`wgpu` app in `src/live_wgpu.rs` instead of calling
  `src_legacy/wgpu_presenter.rs`. The clean app owns window and device
  lifecycle, clean fixed-step `Game` advancement, direct input-state mapping,
  clean audio event submission, atlas/projection/buffer/bind-group setup,
  sprite render pipeline setup, viewport, and indexed instanced sprite draws
  from `NativeSceneRenderer` plans. Automated validation passed with
  `cargo check`, `cargo fmt --check`,
  `cargo test --lib live_wgpu -- --nocapture`,
  `cargo test --lib clean_runtime_and_oracle_use_quarantined_adapters -- --nocapture`,
  `cargo test --lib clean_module_sources_keep_legacy_access_quarantined -- --nocapture`,
  `cargo run -- --live-smoke`, `cargo run -- --game-smoke`,
  `cargo clippy --all-targets -- -D warnings`, `cargo test --all-targets`, and
  `make clean-fidelity`. The open-ended interactive window was not manually
  launched in this automated cycle; the normal non-test path was compiled by
  `cargo check`. Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778944363741649`.

### DC-154: Clean Live-Smoke Frame Source

Status: `complete`

Milestone: `R1: Clean Runtime Takes Over Live Play`

Goal: make `--live-smoke` prove clean `Game` frame generation and native sprite
draw planning without using the legacy live presenter as its frame source.

Scope:

- Route `run_smoke` through clean `Game` smoke frames and `NativeSceneRenderer`.
- Report clean frame-source, legacy-presenter, sprite, and temporary-raster
  evidence in `LiveSmokeReport`.
- Keep full interactive window ownership as a separate R1 follow-on slice.

Work log:

- `2026-05-16` Started `DC-154` with Slack cycle update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778942874711619`.
- `2026-05-16` Completed implementation step and started documentation /
  validation step with Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778943031820439`.
- `2026-05-16` Completed `DC-154`: `--live-smoke` now builds its report from
  clean `Game` frames rendered through `NativeSceneRenderer`, reports
  `frame_source: clean_game`, `legacy_presenter_used: false`, 24 clean frames,
  290 sprite instances, 92 sprite draw commands, and zero temporary raster
  frames or commands. Full interactive `cargo run` window ownership remains the
  next R1 slice. Validation passed with `cargo fmt --check`,
  `cargo test --lib live_wgpu -- --nocapture`,
  `cargo test --lib installed_backend_runs_config_driven_wgpu_smoke -- --nocapture`,
  `cargo test --lib live_smoke_cli_entrypoint_accepts_supported_args -- --nocapture`,
  `cargo test --lib clean_runtime_and_oracle_use_quarantined_adapters -- --nocapture`,
  `cargo run -- --live-smoke`, `cargo run -- --game-smoke`,
  `make clean-fidelity`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test --all-targets`, Markdown lint for `README.md`, `SPEC.md`,
  `PLAN.md`, `docs/fidelity/refactor-freeze.md`, `docs/fidelity/gaps.md`, and
  `git diff --check`. Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778943469537499`.

### DC-153: Clean Fidelity Harness Baseline

Status: `complete`

Milestone: `R0: Final Gates And Oracle Harness`

Goal: make clean rewrite drift measurable by comparing the real clean `Game`
against the accepted oracle with shared scenario input streams.

Scope:

- Add a test-owned clean-vs-accepted runner for selected Phase 1 scenarios.
- Emit TSV first-divergence rows covering state, gameplay events, sound events,
  and render summaries.
- Add `make clean-fidelity` as the narrow R0 gate and document its embedded
  manifest/no-ROM behavior.

Work log:

- `2026-05-16` Started `DC-153` with Slack cycle update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778942004090179`.
- `2026-05-16` Started implementation step with Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778942161457729`.
- `2026-05-16` Completed `DC-153`: added the test-only clean-vs-accepted
  harness, `make clean-fidelity`, README/SPEC/PLAN documentation, and the
  clean rewrite equivalence entry in `docs/fidelity/gaps.md`. Validation passed
  with `cargo fmt --check`, `cargo test --all-targets`,
  `cargo clippy --all-targets -- -D warnings`, `make clean-fidelity`,
  `cargo run -- --game-smoke`, `cargo run -- --live-smoke`,
  Markdown lint for `README.md`, `SPEC.md`, `PLAN.md`,
  `docs/fidelity/refactor-freeze.md`, `docs/fidelity/gaps.md`, and
  `git diff --check`. Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778942704806049`.

### DC-42: Documentation Reset

Status: `complete`

Goal: remove stale historical plan/spec/readme material and leave only current,
useful project information.

Scope:

- Rewrite `README.md` as the clean public entry point: badges, screenshot,
  animated GIF, install/run commands, controls, persistence, compatibility
  overlay, development commands, architecture, assets, platform notes, and
  references.
- Reduce `SPEC.md` to the current behavior contract, source-of-truth rules,
  architecture, validation gates, and active constraints.
- Reduce `PLAN.md` to current baseline, validation, work protocol, this active
  docs cycle, and immediate next work.

Work log:

- `2026-05-10 14:42:08 BST` Started `DC-42`: replacing stale completed phase
  history and drift inventory with current project documentation for the
  refactored `red-label-refactor` baseline.
- `2026-05-10 14:44:36 BST` Completed `DC-42`: reduced `PLAN.md` to the
  active baseline, validation gate, work protocol, docs cycle, and next useful
  work; reduced `SPEC.md` to the current source-of-truth contract,
  architecture, validation, and active constraints; rewrote `README.md` as the
  current public entry point with badges, screenshot, animated GIF, commands,
  controls, persistence, compatibility overlay, development targets,
  architecture, assets, platform notes, and references. Validation passed with
  `markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md`,
  `git diff --check`, and `cargo run -- --help`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778420701682049`

### DC-43: Threaded Fidelity Fixture Runner

Status: `complete`

Goal: continue the refactor by replacing serial fixture orchestration with a
small trait-based runner that can execute independent Rust fidelity fixture
checks on scoped worker threads while preserving manifest ordering and
first-error reporting.

Scope:

- Add a `TraceFixtureChecker` trait to separate fixture discovery/order from
  fixture execution.
- Run fixture checks in ordered chunks across available scoped worker threads.
- Preserve existing `--fidelity-check-trace-dir` output and first-error
  behavior.
- Add focused tests for parallel result aggregation, ordered error reporting,
  and worker panic handling.

Work log:

- `2026-05-12` Completed `DC-43`: `src/app.rs` now checks fidelity fixture
  pairs through a trait-based checker on scoped threads. Validation passed with
  `cargo fmt --check`, targeted `app::tests::check_trace_fixtures`,
  targeted `app::tests::fidelity_check_trace_dir_text`, and
  `cargo clippy --all-targets -- -D warnings`, `cargo test --all-targets`,
  `make trace-script-test`, `make trace-fixtures`,
  `make coverage NEW_CODE_COVERAGE_BASE=origin/main`, `markdownlint README.md
  SPEC.md PLAN.md docs/fidelity/refactor-freeze.md`, and `git diff --check`.

### DC-44: Shared Live Core Driver

Status: `complete`

Goal: continue carving the live runtime into explicit, reusable boundaries
before a later presentation/core thread split.

Scope:

- Add a `LiveCoreDriver` that owns the live input mapper, XYZZY overlay,
  machine, timing clock, pending pulse input, and pending typed characters.
- Reuse the driver from both Kitty and `wgpu` live presentation paths.
- Preserve pulse buffering, held-input catch-up frames, typed-character
  high-score entry behavior, and XYZZY overlay behavior.
- Add focused tests for driver-owned held input, overlay state, and realtime
  pulse buffering.

Work log:

- `2026-05-12` Completed `DC-44`: Kitty and `wgpu` now advance the arcade core
  through the shared `LiveCoreDriver` instead of duplicating input/clock/overlay
  logic. Validation passed with `cargo fmt --check`, `cargo check`, targeted
  `live::tests::live_core_driver`, targeted `wgpu_presenter::tests::wgpu_smoke`,
  `cargo test --all-targets`, and `cargo clippy --all-targets -- -D warnings`.

### DC-45: Threaded Live Core Runtime

Status: `complete`

Goal: move the `wgpu` live path onto an explicit presentation/core thread
boundary without changing gameplay behavior.

Scope:

- Add a `LiveCoreRuntime` trait and `LiveCoreThread` worker that owns
  `LiveCoreDriver`, live `Renderer`, pending input, and CMOS access.
- Move `wgpu` input, resize, advance, render, and CMOS-save calls through the
  worker command protocol.
- Preserve realtime and deterministic smoke-frame stepping, pulse buffering,
  held input, typed characters, XYZZY overlay state, and live smoke reporting.
- Add focused tests for threaded input/overlay advancement, realtime pulse
  buffering, renderer resize, and CMOS snapshots.

Work log:

- `2026-05-12` Completed `DC-45`: `wgpu` presentation now draws the latest
  `LiveCoreFrame` returned from a dedicated live core worker thread instead of
  owning the arcade machine directly. Validation passed with
  `cargo fmt --check`, `cargo check`, targeted
  `live::tests::live_core_thread`, targeted
  `wgpu_presenter::tests::wgpu_smoke`, and
  `cargo clippy --all-targets -- -D warnings`, `cargo test --all-targets`,
  `cargo run -- --live-smoke`, `make trace-script-test`,
  `make trace-fixtures`, `make coverage NEW_CODE_COVERAGE_BASE=origin/main`,
  `markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md`,
  and `git diff --check`.

## Completed Follow-On Cycles

These ordered cycles are complete. If a future behavior gap appears, document
the gap in `docs/fidelity/gaps.md` and add a source-backed fixture or
characterization test before implementation.

### DC-46: Kitty Runtime Unification

Status: `complete`

Goal: put the Kitty compatibility backend behind the same live runtime
boundary as `wgpu`, so presentation code no longer owns arcade-core state in
either live path.

Scope:

- Route Kitty input, frame advancement, rendering, resize handling, sleep
  timing, and CMOS save/load through `LiveCoreRuntime`.
- Keep the existing Kitty graphics protocol, terminal-session handling, and
  terminal geometry code unchanged except for the runtime call sites.
- Preserve pulse buffering, held-input catch-up frames, typed-character high
  score entry, XYZZY overlay behavior, and CMOS persistence.
- Add tests around any extracted Kitty/runtime adapter seams rather than
  requiring an interactive terminal for coverage.

Acceptance criteria:

- `run_kitty_live` does not directly own or mutate `ArcadeMachine`,
  `LiveCoreDriver`, `InputMapper`, `XyzzyOverlay`, or `Renderer`.
- Kitty and `wgpu` share the same runtime command contract for input, resize,
  advance/render, and CMOS access.
- Existing Kitty renderer tests still cover double buffering, clear behavior,
  resize behavior, and environment gating.

Validation:

```sh
cargo fmt --check
cargo test --lib live::tests::live_core
cargo test --lib kitty::tests
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

Work log:

- `2026-05-12` Completed `DC-46`: Kitty now uses `LiveCoreThread` for input,
  resize, advance/render, and CMOS snapshots, matching the `wgpu` live runtime
  command boundary while keeping the existing Kitty graphics and terminal
  session code in the presentation path. Focused adapter coverage was added for
  terminal geometry updates, and the shared CMOS-save helper is used by both
  live backends. Validation passed with `cargo fmt --check`, `cargo check`,
  targeted `live::tests::live_core_thread`, targeted
  `live::tests::terminal_geometry_update_reports_runtime_and_kitty_sizes`,
  targeted `kitty::tests`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test --all-targets`, `cargo run -- --live-smoke`,
  `make trace-script-test`, `make trace-fixtures`, and
  `make coverage NEW_CODE_COVERAGE_BASE=origin/main`, `markdownlint README.md
  SPEC.md PLAN.md docs/fidelity/refactor-freeze.md`, and `git diff --check`.

### DC-47: Non-Blocking Wgpu Frame Pipeline

Status: `complete`

Goal: separate `wgpu` event-loop scheduling from core frame production so the
window thread draws the latest completed frame instead of synchronously waiting
for every arcade-core advance/render command.

Scope:

- Introduce a small latest-frame mailbox or bounded channel between the live
  core worker and `WgpuLiveApp`.
- Keep input and resize commands ordered relative to core advancement.
- Preserve deterministic `--live-smoke` behavior by keeping smoke mode on an
  explicit fixed-frame cadence with observable frame counts.
- Keep normal realtime mode tied to `FRAME_RATE_MILLIHZ` deadlines from the
  core runtime.
- Add tests for latest-frame replacement, stale-frame dropping, resize
  ordering, clean shutdown, and smoke determinism.

Acceptance criteria:

- `WgpuLiveApp::draw_frame` can render the latest available frame without
  blocking on an arcade-core step.
- Normal mode does not busy-spin when no frame is due.
- Smoke reports still include window creation, rendered frame count, distinct
  frame signatures, attract/credit/playing evidence, injected inputs, and clean
  exit.
- Worker shutdown cannot leave the event loop waiting forever on a channel
  receive.

Validation:

```sh
cargo fmt --check
cargo test --lib live::tests::live_core
cargo test --lib wgpu_presenter::tests::wgpu_smoke
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo run -- --live-smoke
```

Work log:

- `2026-05-12` Completed `DC-47`: `LiveCoreThread` now supports a
  replacement latest-frame mailbox and non-blocking frame requests, while
  keeping the synchronous advance path for Kitty. `WgpuLiveApp` requests at
  most one core frame at a time, drains completed frames without blocking the
  redraw path, schedules normal mode from the worker-owned frame deadline, and
  keeps smoke mode on explicit fixed-frame requests. Focused coverage was
  added for mailbox replacement, stale-frame dropping, resize ordering, async
  fixed-frame determinism, and in-flight shutdown joining. Validation passed
  with `cargo fmt --check`, targeted `live::tests::live_core`, targeted
  `wgpu_presenter::tests::wgpu_smoke`, clippy with warnings denied,
  `cargo test --all-targets`, `cargo run -- --live-smoke`, `make fidelity`,
  Markdown lint, and the whitespace diff check.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778624044098319`

### DC-48: Runtime Lifecycle And Error Contracts

Status: `complete`

Goal: make live runtime startup, shutdown, worker panic reporting, and CMOS
persistence explicit and testable before adding more long-running runtime
features.

Scope:

- Replace stringly worker failures with a small runtime error type or structured
  context that distinguishes command send failures, worker termination, render
  failures, and worker panics.
- Ensure live workers shut down and join predictably from normal exit, smoke
  exit, window close, suspended windows, and error paths.
- Confirm CMOS save uses the final core-owned CMOS state after live runtime
  shutdown.
- Add tests for worker drop, failed command response, render error propagation,
  and CMOS snapshot retrieval after gameplay mutations.

Acceptance criteria:

- Runtime errors include enough context to identify the failed command.
- Dropping the runtime cannot leak a worker thread.
- `run_wgpu_live` and `run_wgpu_live_smoke` preserve their public error
  behavior while using the structured runtime errors internally.
- CMOS save remains best-effort only where it already was; no new gameplay
  behavior depends on persistence.

Validation:

```sh
cargo fmt --check
cargo test --lib live::tests::live_core_thread
cargo test --lib wgpu_presenter::tests
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

Work log:

- `2026-05-12` Completed `DC-48`: live runtime failures now report structured
  command-scoped errors for command send failures, worker termination, worker
  panics, mailbox failures, and render failures. Live shutdown now joins the
  worker and returns the final core-owned CMOS snapshot before persistence, so
  Kitty and `wgpu` both save after runtime shutdown. Focused coverage was added
  for drop/join behavior, failed command context, worker panic context,
  sync/async render error propagation, and CMOS mutation persistence.
  Validation passed with `cargo fmt --check`, targeted
  `live::tests::live_core_thread`, targeted `wgpu_presenter::tests`, clippy
  with warnings denied, `cargo test --all-targets`, and
  `cargo run -- --live-smoke`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778624792340579`

### DC-49: Public Arcade API Narrowing

Status: `complete`

Goal: reduce the public `machine::...` compatibility surface without breaking
existing internal callers or hiding source-shaped contracts that tests still
need.

Scope:

- Inventory current `machine::...` re-exports and classify them as public API,
  test-only support, internal implementation detail, or source fixture
  contract.
- Move internal-only imports to their owning modules where practical.
- Keep compatibility re-exports temporarily when the migration would otherwise
  create broad churn.
- Add compile-time caller checks or focused tests for any moved public surface.
- Update `README.md` and `SPEC.md` only if the user-facing API changes.

Acceptance criteria:

- No behavior code changes are mixed into this cycle.
- Existing external-facing commands and live behavior are unchanged.
- Any removed or moved symbol has a documented replacement path or is proven
  private to the crate.

Validation:

```sh
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make coverage NEW_CODE_COVERAGE_BASE=origin/main
```

Work log:

- `2026-05-12` Completed `DC-49`: `machine_state` and `machine_process` are
  now public canonical contract modules, while the existing `machine::...`
  re-exports remain as compatibility aliases. Internal callers in fidelity,
  live runtime, and `wgpu` presentation now import state/process contracts from
  the owning modules where practical. Compile-time API checks cover both direct
  and compatibility paths, and README/SPEC now document the canonical paths.
  Validation passed with `cargo fmt --check`, `cargo test --all-targets`,
  `cargo clippy --all-targets -- -D warnings`, and
  `make coverage NEW_CODE_COVERAGE_BASE=origin/main`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778626931639039`

### DC-50: Machine Module Ergonomics Pass

Status: `complete`

Goal: keep shrinking assembler-shaped bulk into readable Rust modules while
preserving source-visible behavior and fixture coverage.

Scope:

- Pick one high-cohesion area at a time: scoring/high-score flow, object list
  mutations, shell/projectile handling, terrain/world flow, or operator/audit
  flow.
- Extract small typed data structures or helper traits only when they remove
  real duplication or clarify a source contract.
- Keep source routine names visible in tests and error messages where they are
  part of fidelity evidence.
- Avoid broad rewrites of `machine.rs` and `machine_memory.rs`; each pass must
  have a narrow ownership boundary and focused tests.

Acceptance criteria:

- The selected module area has less duplicated address/list manipulation or
  less incidental register plumbing than before the cycle.
- Source-visible mutations remain covered by existing or new tests.
- Public behavior, trace output, live play, and fixture checks are unchanged.

Validation:

```sh
cargo fmt --check
cargo test --lib machine::tests::<focused_filter>
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make trace-fixtures
make coverage NEW_CODE_COVERAGE_BASE=origin/main
```

Work log:

- `2026-05-13` Completed `DC-50`: the high-score game-over dispatch path and
  test-only one-player game setup now reuse the live high-score session reset
  helper instead of duplicating entry/submission/player-mask cleanup. Focused
  regression coverage now asserts `GameOverSleeping` clears the entry,
  submission, active-entry player, completed-player mask, and phase state.
  Validation passed with `cargo fmt --check`, targeted
  `machine::tests::game_over_sleeping_dispatch_clears_live_high_score_session`,
  `cargo test --all-targets`, `cargo clippy --all-targets -- -D warnings`,
  `make trace-fixtures`, and
  `make coverage NEW_CODE_COVERAGE_BASE=origin/main`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778628102721249`

### DC-51: Live Audio Acceptance Design

Status: `complete`

Goal: prepare live audio output with source-backed acceptance criteria before
implementing any audio device or mixer code.

Scope:

- Define which sound-board surface is authoritative for live audio:
  per-frame sound commands, sound-board state snapshots, DAC byte signatures,
  or a combination.
- Add a documented live-audio design note that covers cadence, buffering,
  sample rate, worker-thread ownership, and how audio interacts with pause,
  window suspend, smoke mode, and CMOS persistence.
- Add or extend fixtures that prove command timing for coin, start, thrust,
  smart bomb, hyperspace, explosion, terrain blow, and high-score paths.
- Do not add audible output in this cycle.

Acceptance criteria:

- `docs/fidelity/gaps.md` has no unresolved live-audio behavior question that
  would block implementation.
- The plan identifies the exact fixture or test that will fail if audio timing
  drifts.
- Runtime threading ownership for audio is specified before implementation.

Validation:

```sh
cargo fmt --check
cargo test --all-targets
make trace-script-test
make trace-fixtures
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md
```

Work log:

- `2026-05-13` Completed `DC-51`: added the source-backed live audio
  acceptance design in `docs/fidelity/live-audio.md`, added
  `assets/red-label/live-audio-acceptance.tsv`, and wired the matrix into
  embedded-asset tests so each accepted path is backed by existing command or
  sound-table fixtures. README, SPEC, fidelity README, gap notes, and the
  refactor freeze inventory now point to the accepted timing, diagnostic, and
  content-guard surfaces before runtime audio implementation begins.
  Validation passed with `cargo fmt --check`, `cargo test --all-targets`,
  `make trace-script-test`, `make trace-fixtures`, and `markdownlint README.md
  SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/README.md
  docs/fidelity/gaps.md docs/fidelity/live-audio.md`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778628605842599`

### DC-52: Live Audio Runtime Prototype

Status: `complete`

Goal: add the first live audio runtime path only after `DC-51` establishes the
source-backed acceptance contract.

Scope:

- Add an audio backend behind a trait so live audio can be disabled, mocked, or
  swapped without changing the arcade core.
- Feed audio from the accepted sound-board surface without changing trace
  output or machine stepping cadence.
- Keep audio commands on a runtime-owned thread or channel boundary that does
  not block `wgpu` redraw or core stepping.
- Add a CLI/config path only if the default behavior and platform failure modes
  are clear.

Acceptance criteria:

- Audio can be disabled for tests and unsupported platforms.
- `--live-smoke` remains deterministic and does not require an audio device.
- Sound command fixtures and DAC signature tests still pass unchanged unless
  `DC-51` explicitly updated their accepted contract.

Validation:

```sh
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
```

Work log:

- `2026-05-13` Completed `DC-52`: added `src/audio.rs` with a live audio
  backend trait, disabled mode, no-device null backend, bounded non-blocking
  command queue, drop accounting, flush/shutdown handling, and focused tests.
  Live core stepping now copies accepted `FrameOutput::sound_commands()`
  batches to the audio runtime without changing arcade-core ownership, trace
  output, or step cadence. Normal live play uses the null backend, `--mute`
  disables the runtime path, and `--live-smoke` uses a disabled no-device path.
  README, SPEC, fidelity docs, gap notes, and refactor-freeze ownership notes
  now describe the prototype boundary and the remaining audible-device work.
  Validation passed with `cargo fmt --check`, `cargo test --all-targets`,
  `cargo clippy --all-targets -- -D warnings`, `make fidelity`,
  `cargo run -- --live-smoke`, `markdownlint README.md SPEC.md PLAN.md
  docs/fidelity/refactor-freeze.md docs/fidelity/README.md
  docs/fidelity/gaps.md docs/fidelity/live-audio.md`, and `git diff --check`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778630482569359`

### DC-53: Release And CI Hardening

Status: `complete`

Goal: make the refactored runtime cheaper to maintain by tightening local and
GitHub validation around the expensive fidelity, smoke, and coverage gates.

Scope:

- Review CI runtime after the threaded fixture runner and live runtime changes.
- Keep `make ci`, `make fidelity`, Sonar, coverage baseline, and local docs in
  sync.
- Add targeted CI diagnostics for `wgpu` smoke failures, coverage baseline
  drift, missing Lua/Mesa tools, and Slack update failures.
- Keep generated artifacts out of git unless the project explicitly accepts
  them as source fixtures or documentation media.

Acceptance criteria:

- A CI failure points at the failed subsystem instead of requiring full log
  archaeology.
- Coverage baseline refresh remains an intentional command, not an implicit
  side effect.
- README development commands match the Makefile and workflows.

Validation:

```sh
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md
git diff --check
```

Work log:

- `2026-05-13` Completed `DC-53`: added Makefile doctor targets for trace,
  coverage, and `wgpu` smoke prerequisites; split GitHub CI into explicit
  prerequisite diagnostics, fidelity, and `xvfb` smoke steps; added Sonar
  coverage diagnostics; and updated README/SPEC development guidance so local
  commands, workflow steps, and the intentional coverage-baseline refresh path
  stay aligned. Validation passed with `make trace-doctor`,
  `make coverage-doctor`, `make trace-script-test`, `make fidelity`,
  `cargo run -- --live-smoke`, `markdownlint README.md SPEC.md PLAN.md
  docs/fidelity/README.md docs/fidelity/gaps.md
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md`, and
  `git diff --check`.
  Slack update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778631713574199`

## Planned Rewrite Cycles

These cycles replace the completed refactor track with a clean `wgpu`-only
rewrite track. Keep each cycle narrow enough to finish with focused tests and
the full fidelity gate before moving on.

### DC-54: Wgpu-Only Rewrite Foundation

Status: `complete`

Goal: establish the clean rewrite boundary without changing gameplay behavior.

Scope:

- Remove Kitty from the active app surface: CLI renderer selection, runtime
  routing, docs, active tests, and CI expectations.
- Make `wgpu` the only supported live runtime and `--live-smoke` path.
- Introduce `game`, `systems`, `renderer`, and `platform` module shells with
  clean public contracts.
- Move the converted `src/` tree to `src_legacy/` and make the clean rewrite
  the primary `src/` tree.
- Move or wrap the current assembler-shaped implementation behind an explicit
  `fidelity::oracle` or `legacy` boundary.
- Narrow `src/lib.rs` public exports to the intended clean API plus temporary
  oracle/test surfaces.
- Replace fragile new-code coverage baseline matching with a line-and-context
  or source-hash keyed baseline.
- Convert CLI parsing to typed command parsing so mixed commands and flags are
  rejected predictably.

Acceptance criteria:

- `cargo run` and `cargo run -- --live-smoke` use `wgpu` without backend
  selection.
- No active production path depends on Kitty or terminal graphics.
- The current gameplay model is still available to tests as an oracle.
- Public crate exports distinguish clean API from temporary oracle internals.
- Coverage baseline entries cannot accidentally accept unrelated repeated
  source lines.

Validation:

```sh
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md
git diff --check
```

Work log:

- `2026-05-13` Started `DC-54` on branch `rewrite`: posted the cycle start
  update, moved the converted implementation to `src_legacy/`, promoted the
  clean rewrite modules into `src/`, and kept the legacy implementation wired
  as the temporary oracle/compatibility runtime.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778657320848069`
- `2026-05-13` Continued `DC-54`: removed active Kitty renderer selection from
  CLI parsing, help text, Make targets, and live runtime routing. `cargo run`
  and `cargo run -- --live-smoke` now use the `wgpu` path without backend
  selection.
- `2026-05-13` Completed `DC-54`: moved the old implementation under
  `src_legacy/`, made the clean rewrite tree the primary `src/`, kept the
  legacy machine available as an oracle/compatibility runtime, strengthened the
  new-code coverage baseline to line-and-source-hash matching, and refreshed
  the baseline to zero accepted uncovered additions. Validation passed with
  `make fidelity`, `cargo run -- --live-smoke`, `markdownlint README.md
  SPEC.md PLAN.md docs/fidelity/refactor-freeze.md
  docs/fidelity/live-audio.md`, `git diff --check`, and
  `cargo run --quiet -- --help`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778660955629679`

### DC-55: Clean Simulation Contracts

Status: `complete`

Goal: define the domain-first frame API that future gameplay systems will own.

Scope:

- Add clean `GameState`, `GameInput`, `GameFrame`, `GameEvents`, and
  `RenderScene` contracts.
- Add an oracle adapter that converts current machine output into the clean
  contracts for comparison.
- Add fixtures that compare clean frame events and scene summaries against the
  current accepted behavior.
- Keep all red-label/source terminology inside the oracle adapter and fidelity
  tests.

Acceptance criteria:

- The `wgpu` runtime can advance through the clean frame API even while the
  oracle still produces the underlying behavior.
- New gameplay code can be written against clean contracts without importing
  machine memory modules.
- Accepted trace scenarios have clean event/scene comparison coverage.

Validation:

```sh
cargo fmt --check
cargo test --lib game
cargo test --lib systems
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
```

Work log:

- `2026-05-13` Started `DC-55` on branch `rewrite`: posted the cycle start
  update and began extending the clean `GameState`/`GameFrame`/`GameEvents`,
  `RenderScene`, and simulation trait contracts while keeping the converted
  implementation behind the oracle boundary.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778661176512439`
- `2026-05-13` Completed `DC-55`: `GameFrame` now carries clean
  `GameState`, `GameEvents`, sound events, and a `RenderScene`; renderer
  scenes expose stable summaries and layer counts; `systems` exposes a
  `GameSimulation` trait for clean frame advancement; and the gameplay oracle
  implements that trait while converting accepted machine output into clean
  state, event, sound, and scene-summary frames. Clean fixture coverage now
  compares credited-start events and scene summaries against the accepted
  oracle behavior. Validation passed with `cargo fmt --check`, focused
  `game`, `systems`, `renderer`, and `oracle` tests, `cargo test
  --all-targets`, `cargo clippy --all-targets -- -D warnings`,
  `make fidelity`, `cargo run -- --live-smoke`, `markdownlint README.md
  SPEC.md PLAN.md docs/fidelity/refactor-freeze.md
  docs/fidelity/live-audio.md`, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778664173986969`

### DC-56: Native Wgpu Scene Renderer

Status: `complete`

Goal: replace framebuffer presentation with a native `wgpu` scene renderer
while keeping visual behavior equivalent.

Scope:

- Build renderer-owned pipelines for terrain/starfield, sprites, projectiles,
  explosions, HUD/text, and debug overlays.
- Introduce texture atlas and palette/font resources owned by the renderer.
- Render from `RenderScene` instead of machine RAM.
- Keep a temporary framebuffer comparison path for golden visual fixtures.

Acceptance criteria:

- Live play and smoke render through native `wgpu` scene data.
- Golden visual evidence still catches behavioral or visual drift.
- Renderer modules do not import machine memory or oracle-specific types.

Validation:

```sh
cargo fmt --check
cargo test --lib renderer
cargo test --lib wgpu_presenter::tests::wgpu_smoke
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
```

Work log:

- `2026-05-13` Started `DC-56` on branch `rewrite`: posted the cycle start
  update and began replacing direct frame upload with clean renderer scene data.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778664443957909`
- `2026-05-13` Completed `DC-56`: `RenderScene` now supports a validated
  temporary raster payload for visual equivalence, renderer-owned atlas,
  palette, font, native pipeline, and draw-plan resources live in the clean
  renderer module, and the live `wgpu` path uploads scene raster data instead
  of drawing directly from `RenderedImage`. Smoke visual evidence now derives
  from scene metrics while the temporary raster path keeps golden visual drift
  detectable. Validation passed with `cargo fmt --check`, focused
  `renderer`, `wgpu_smoke`, and live scene tests, `cargo test --all-targets`,
  `cargo clippy --all-targets -- -D warnings`, `make fidelity`, and
  `cargo run -- --live-smoke`; the live smoke reported 240 rendered frames,
  74 distinct scene signatures, attract/credit/playing evidence, all required
  injected inputs, and clean exit.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778667397382589`

### DC-57: Player Control System Migration

Status: `complete`

Goal: begin migrating gameplay from memory-oriented routines to clean
deterministic systems with the first input/player-control slice.

Scope:

- Add a clean player-control system that separates held movement/fire intent
  from two-clear-sample action triggers.
- Preserve vertical-control priority and input history behavior without exposing
  RAM-layout fields or assembler routine names in production code.
- Compare the clean control history against the oracle before replacing live
  gameplay paths.
- Keep the remaining gameplay migration sequence explicit for follow-on cycles.

Acceptance criteria:

- The migrated player-control slice has clean domain tests and oracle
  equivalence tests.
- Production player-control code no longer reads or writes RAM-layout fields.
- Behavior, trace output, live smoke, and accepted visual/audio evidence remain
  stable.

Validation:

```sh
cargo fmt --check
cargo test --lib systems
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
```

Work log:

- `2026-05-13` Started `DC-57` on branch `rewrite`: posted the cycle start
  update and began migrating player input/control behavior into clean
  deterministic systems while keeping accepted behavior behind the oracle.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778667591192949`
- `2026-05-13` Completed `DC-57`: added clean `PlayerControlIntent`,
  `PlayerActionTriggers`, `PlayerControlFrame`, and `PlayerControlSystem`
  contracts for held control intent and two-clear-sample action triggers;
  exported the contracts through the clean public API; and added both clean
  domain tests and oracle switch-scan equivalence coverage. `README.md` and
  `SPEC.md` now describe the clean player-control system, and `DC-58` now
  carries the next player-motion/projectile migration slice. Validation passed
  with `cargo fmt --check`, `cargo test --lib systems`, `cargo test
  --all-targets`, `cargo clippy --all-targets -- -D warnings`,
  `make fidelity`, `cargo run -- --live-smoke`, `markdownlint README.md
  SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md`,
  and `git diff --check`; live smoke rendered 239 frames with 74 distinct scene
  CRCs, attract/credit/playing evidence, all required injected inputs, and a
  clean exit.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778669085679879`

### DC-58: Player Motion And Projectile Systems

Status: `complete`

Goal: continue gameplay migration by moving player motion and projectile
launch behavior into clean deterministic systems.

Scope:

- Drive player motion from `PlayerControlIntent` while preserving accepted
  damping, thrust, vertical priority, bounds, and scroll behavior.
- Add clean projectile launch/capacity state and compare fire entry timing
  against the oracle.
- Keep update order explicit for controls, motion, projectiles, collision, and
  rendering scene emission.
- Remove assembler-derived names from newly migrated production code.

Acceptance criteria:

- Player motion and projectile launch/capacity slices have clean domain tests
  and oracle equivalence tests.
- Production player-motion and projectile modules do not read or write
  RAM-layout fields.
- Behavior, trace output, live smoke, and accepted visual evidence remain
  stable unless an intentional difference is documented.

Validation:

```sh
cargo fmt --check
cargo test --lib systems
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
```

Work log:

- `2026-05-13` Started `DC-58` on branch `rewrite`: posted the cycle start
  update and began moving player motion plus projectile launch/capacity
  behavior into clean deterministic systems while keeping accepted behavior
  behind the oracle.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778669202479739`
- `2026-05-13` Completed `DC-58`: added clean `ScreenPosition`,
  `PlayerMotionState`, `PlayerMotionFrame`, `PlayerMotionSystem`,
  `ProjectileState`, `ProjectileLaunchOutcome`, and `ProjectileSystem`
  contracts; exported them through the clean public API; and added focused
  systems tests plus oracle equivalence for accepted player motion and laser
  fire entry behavior. Validation passed with `cargo fmt --check`, `cargo test
  --lib systems`, `cargo test --lib oracle`, `cargo test --all-targets`,
  `cargo clippy --all-targets -- -D warnings`, `make fidelity`, `cargo run --
  --live-smoke`, `markdownlint README.md SPEC.md PLAN.md
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md`, and
  `git diff --check`; the coverage gate reported 196/196 non-baselined added
  executable Rust lines covered, and live smoke rendered 239 frames with 74
  distinct scene signatures, attract/credit/playing evidence, all required injected
  inputs, and a clean exit.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778672203264889`

### DC-59: Audio Device And Event Model

Status: `complete`

Goal: replace command-byte delivery with gameplay-facing sound events and a
diagnosable audio runtime.

Scope:

- Map accepted command timing to clean `SoundEvent` values.
- Add structured audio worker errors, backend lifecycle reporting, and smoke
  diagnostics.
- Add an audible backend only after null/disabled/event equivalence is stable.
- Keep sound fixture timing as the oracle for migrated sound events.

Acceptance criteria:

- Gameplay systems emit semantic sound events.
- Audio worker failure is visible in tests and diagnostics.
- `--live-smoke` stays deterministic and device-independent.

Validation:

```sh
cargo fmt --check
cargo test --lib audio
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
```

Work log:

- `2026-05-13` Started `DC-59` on branch `rewrite`: posted the cycle start
  update and began moving live audio from raw command-batch delivery to
  gameplay-facing sound events while keeping accepted frame-output timing as
  the oracle boundary.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778672338922919`
- `2026-05-13` Completed `DC-59`: added active clean `src/audio.rs` event
  batches, semantic `SoundEvent` mapping for accepted startup, credit, start,
  and thrust cues, structured audio shutdown diagnostics, backend lifecycle
  and sample-rate reporting, queue drop stats, and worker panic visibility.
  The legacy live core now feeds `SoundEvent` batches through the clean runtime,
  and documentation now describes event delivery with `FrameOutput` retained
  as the timing adapter. Validation passed with `cargo fmt --check`, `cargo
  test --lib audio`, `cargo test --all-targets`, `cargo clippy --all-targets
  -- -D warnings`, `make fidelity`, `cargo run -- --live-smoke`,
  `markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md
  docs/fidelity/live-audio.md`, and `git diff --check`; the coverage gate
  reported 17/17 non-baselined added executable Rust lines covered, and live
  smoke rendered 239 frames with 74 distinct scene signatures, attract/credit/playing
  evidence, all required injected inputs, and a clean exit.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778675086403679`

### DC-60: Compatibility Surface Quarantine

Status: `complete`

Goal: begin oracle retirement by hiding legacy compatibility modules from the
supported clean public API documentation while current runtime and fidelity
tooling still depend on them.

Scope:

- Mark all `src_legacy/` path modules in `src/lib.rs` as doc-hidden
  compatibility modules.
- Add a focused architecture test that fails if a legacy path module is exposed
  without the compatibility quarantine marker.
- Keep the binary, README media tooling, oracle tests, and fidelity gates
  working without changing gameplay behavior.
- Document that compatibility modules remain wired but are not the supported
  clean API surface.

Acceptance criteria:

- Supported docs expose clean modules first and legacy modules are explicitly
  hidden.
- Tests guard against adding new unhidden legacy path modules.
- Public clean contracts and existing compatibility runtime remain intact.

Validation:

```sh
cargo fmt --check
cargo test --lib public_api_tests::legacy_compatibility_modules_are_hidden_from_supported_api_docs
cargo check --all-targets
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-13` Started `DC-60` on branch `rewrite`: posted the cycle start
  update and began the first oracle-retirement step by auditing legacy module
  exposure from the clean crate root.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778675232663129`
- `2026-05-13` Completed `DC-60`: marked every `src_legacy/` path module in
  `src/lib.rs` as a doc-hidden compatibility module, added
  `public_api_tests::legacy_compatibility_modules_are_hidden_from_supported_api_docs`
  to guard the quarantine marker, and updated `README.md`, `SPEC.md`, and this
  plan to describe the hidden compatibility surface while moving full oracle
  retirement to `DC-61`. Validation passed with the documented DC-60 gate:
  formatting, the focused public API guard, `cargo check --all-targets`, the
  full Rust test suite, clippy with warnings denied, `make fidelity`,
  `cargo run -- --live-smoke`, markdownlint, and `git diff --check`; the
  coverage gate reported 0/0 non-baselined added executable Rust lines, and
  live smoke rendered 240 frames with 74 distinct scene signatures,
  attract/credit/playing evidence, all required injected inputs, and a clean
  exit.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778676669257919`

### DC-61: Runtime Entrypoint Facade

Status: `complete`

Goal: continue oracle retirement by removing the binary entrypoint's direct
dependency on the doc-hidden legacy `app` module.

Scope:

- Add a clean platform-facing runtime launcher that owns the production
  entrypoint contract.
- Point `src/main.rs` at the clean platform launcher instead of
  `defender::app::run()`.
- Add a focused architecture guard that rejects direct binary calls into the
  legacy `app` module.
- Document that the binary now enters through the clean runtime boundary while
  the compatibility runtime remains the temporary accepted behavior owner.

Acceptance criteria:

- `src/main.rs` depends on `defender::platform::run()`.
- The legacy `app` module remains hidden compatibility plumbing.
- CLI behavior and fidelity gates are unchanged.

Validation:

```sh
cargo fmt --check
cargo test --lib public_api_tests::binary_entrypoint_uses_clean_platform_runtime_boundary
cargo test --lib platform::tests::runtime_entrypoint_delegates_to_compatibility_runtime
cargo check --all-targets
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-13` Started `DC-61` on branch `rewrite`: posted the cycle start
  update and began moving the production binary entrypoint behind a clean
  platform launcher.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778676810486839`
- `2026-05-13` Completed `DC-61`: added `platform::run()` as the clean runtime
  launcher, pointed `src/main.rs` at `defender::platform::run()`, guarded the
  binary entrypoint against direct legacy `app` calls, and documented that the
  binary now enters through the clean platform boundary while the compatibility
  runtime remains the temporary accepted behavior owner. Validation passed with
  the documented DC-61 gate: formatting, the focused public API and platform
  entrypoint tests, `cargo check --all-targets`, the full Rust test suite,
  clippy with warnings denied, `make fidelity`, `cargo run -- --live-smoke`,
  markdownlint, and `git diff --check`; the coverage gate reported 2/2
  non-baselined added executable Rust lines, and live smoke rendered 239 frames
  with 74 distinct scene signatures, attract/credit/playing evidence, all required
  injected inputs, and a clean exit.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778679271201939`

### DC-62: Compatibility Namespace Facade

Status: `complete`

Goal: make the remaining legacy access explicit by routing clean runtime and
oracle callers through a single doc-hidden compatibility namespace.

Scope:

- Add a `defender::compatibility` facade over the parked `src_legacy/`
  adapters.
- Route `platform` runtime launch and `oracle` accepted-behavior calls through
  the compatibility namespace instead of direct legacy crate-root paths.
- Preserve the existing doc-hidden legacy modules for legacy internals until a
  later deletion pass can retire them safely.
- Add architecture tests that keep clean runtime and oracle callers on the
  compatibility boundary.

Acceptance criteria:

- Clean runtime and oracle modules no longer call `crate::app`,
  `crate::machine_state`, `crate::video`, or related legacy root paths
  directly.
- The compatibility facade re-exports the machine-state and process contracts
  still needed by oracle tests.
- README, SPEC, and PLAN describe compatibility access as a temporary boundary,
  not a clean production dependency.

Validation:

```sh
cargo fmt --check
cargo test --lib public_api_tests::compatibility_namespace
cargo test --lib platform::tests::runtime_entrypoint_delegates_to_compatibility_runtime
cargo check --all-targets
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-13 18:56:46 BST` Started `DC-62`: posted the cycle start update and
  began introducing a compatibility namespace boundary for clean runtime and
  oracle legacy access.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778695006846469`
- `2026-05-13 19:34:22 BST` Completed `DC-62`: added the doc-hidden
  `defender::compatibility` facade, routed `platform` and `oracle` legacy
  calls through that namespace, added architecture tests to keep clean runtime
  and oracle callers off direct legacy crate-root paths, added an oracle phase
  contract test to cover all accepted phase mappings, and updated README,
  SPEC, and PLAN to describe the compatibility boundary and next oracle
  retirement slice. Validation passed with the documented DC-62 gate:
  formatting, focused compatibility and platform tests, `cargo check
  --all-targets`, the full Rust test suite, clippy with warnings denied,
  `make fidelity`, `cargo run -- --live-smoke`, markdownlint, and
  `git diff --check`; the first `make fidelity` run exposed missing new-line
  coverage for two phase mapping arms, which was fixed before rerunning the
  gate successfully. The final coverage gate reported 7/7 non-baselined added
  executable Rust lines, and live smoke rendered 239 frames with 74 distinct
  scene signatures, attract/credit/playing evidence, all required injected inputs,
  and a clean exit.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778697300468119`

### DC-63: Public Legacy Export Retirement

Status: `complete`

Goal: stop exposing parked legacy modules from the public crate root while
keeping temporary oracle and tooling access behind the doc-hidden compatibility
namespace.

Scope:

- Change `src_legacy/` root adapters in `src/lib.rs` from public modules to
  crate-private modules.
- Rebuild `defender::compatibility` as explicit doc-hidden submodules that
  re-export the public legacy items needed by the oracle and temporary tools.
- Route the README media example through `defender::compatibility` instead of
  direct legacy public crate-root paths.
- Add architecture tests that fail if legacy root adapters become public again.

Acceptance criteria:

- External callers can no longer import `defender::machine`,
  `defender::input`, `defender::video`, or related parked legacy modules from
  the root crate namespace.
- Temporary oracle and README media tooling still compile through the
  compatibility namespace.
- README, SPEC, and PLAN describe root legacy adapters as crate-private.

Validation:

```sh
cargo fmt --check
cargo test --lib public_api_tests::compatibility_namespace
cargo test --lib public_api_tests::legacy_compatibility_modules_are_crate_private_at_root
cargo check --all-targets
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-13 19:38:28 BST` Started `DC-63`: posted the cycle start update and
  began retiring public root exports for the parked legacy adapters while
  preserving temporary oracle and tooling access through the compatibility
  namespace.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778697445868899`
- `2026-05-13 19:58:11 BST` Completed `DC-63`: made parked legacy adapters
  crate-private at the root, rebuilt `defender::compatibility` as explicit
  doc-hidden submodules, routed README media generation through that boundary,
  and updated architecture docs/tests to preserve the split. Validation passed:
  `cargo fmt --check`, targeted public API tests, `cargo check --all-targets`,
  `cargo test --all-targets`, `cargo clippy --all-targets -- -D warnings`,
  `make fidelity`, `cargo run -- --live-smoke`, `markdownlint README.md
  SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md`,
  and `git diff --check`. `make fidelity` reported new Rust line coverage
  `0/0` non-baselined added executable lines. Live smoke rendered 239 frames,
  saw 74 distinct frame signatures, observed attract, credit, and playing states,
  injected all required controls, and exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778698710074379`

### DC-64: Accepted-Behavior Facade

Status: `complete`

Goal: isolate production clean runtime and oracle code from direct legacy
module names while preserving the accepted gameplay implementation as the
temporary oracle.

Scope:

- Add a crate-private `src/accepted.rs` facade over the temporary accepted
  implementation.
- Convert accepted machine output into neutral accepted-behavior frame,
  snapshot, phase, direction, event, sound, and visual-signature contracts before
  `src/oracle.rs` adapts them to clean gameplay types.
- Route `src/platform.rs` through the accepted facade instead of calling the
  doc-hidden compatibility runtime directly.
- Keep low-level legacy method access in tests and temporary tooling only.
- Update architecture tests and docs so future clean production callers use
  the accepted facade rather than `defender::compatibility`.

Acceptance criteria:

- `src/oracle.rs` production code imports `crate::accepted::...`, not
  `crate::compatibility::...` or direct legacy root modules.
- `src/platform.rs` dispatches through `crate::accepted::run_runtime()`.
- Focused accepted-facade, oracle, and public API tests pass.
- README, SPEC, and PLAN describe the accepted facade as the current retirement
  boundary.

Validation:

```sh
cargo fmt --check
cargo test --lib accepted::tests
cargo test --lib oracle::tests
cargo test --lib public_api_tests
cargo check --all-targets
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-13 20:01:11 BST` Started `DC-64`: posted the cycle start update and
  began isolating production runtime/oracle access behind a neutral
  accepted-behavior facade while keeping the legacy machine available for
  behavior comparison.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778698871347899`
- `2026-05-13 20:43:21 BST` Completed `DC-64`: added the crate-private
  `src/accepted.rs` facade, routed `platform` and `GameplayOracle` through
  `crate::accepted`, converted accepted machine output into neutral frame,
  snapshot, phase, direction, event, sound-command, and visual-signature
  contracts,
  and updated docs/tests to preserve the boundary. Validation passed with the
  DC-64 gate: formatting, focused accepted/oracle/API tests, all-target
  check/test/clippy, `make fidelity`, live smoke, markdownlint, and
  `git diff --check`. `make fidelity` reported new Rust line coverage `38/38`
  non-baselined added executable lines. Live smoke rendered 239 frames, saw 74
  distinct frame signatures, observed attract, credit, and playing states, injected
  all required controls, and exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778701401764609`

### DC-65: Oracle Equivalence Quarantine

Status: `complete`

Goal: move legacy-specific oracle equivalence checks out of clean oracle source
without weakening the clean-system regression coverage.

Scope:

- Keep `src/oracle.rs` focused on clean gameplay adapter contracts.
- Move low-level accepted-behavior equivalence checks that need legacy process,
  memory, and red-label names into a `src_legacy/` test module.
- Wire the quarantined tests through `src/lib.rs` only for `cfg(test)`.
- Add a public API guard so `src/oracle.rs` does not reintroduce legacy
  terminology while the accepted facade remains temporary.
- Document the test quarantine boundary in README and SPEC.

Acceptance criteria:

- `src/oracle.rs` contains no direct compatibility or red-label terminology.
- The moved equivalence tests still run in the normal library test suite.
- Focused oracle, quarantined equivalence, and public API tests pass.
- README, SPEC, and PLAN describe the new test boundary.

Validation:

```sh
cargo fmt --check
cargo test --lib oracle::tests
cargo test --lib oracle_equivalence_tests
cargo test --lib public_api_tests
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
ORACLE_LEGACY_TERMS='red_label|RED_LABEL|defend\\.|src/machine_memory|source routine|assembler|compatibility'
! rg -n "$ORACLE_LEGACY_TERMS" src/oracle.rs
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-13 21:03:12 BST` Started `DC-65`: posted the cycle start update and
  began quarantining legacy-heavy oracle equivalence checks out of
  `src/oracle.rs` while preserving the clean-system regression coverage.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778702592948259`
- `2026-05-13 21:28:13 BST` Completed `DC-65`: moved the low-level legacy
  oracle equivalence checks into `src_legacy/oracle_equivalence_tests.rs`, left
  `src/oracle.rs` with clean accepted/gameplay adapter tests, added a public
  API guard against legacy terminology in `src/oracle.rs`, and documented the
  test quarantine boundary. Validation passed with the DC-65 gate: focused
  oracle/equivalence/API tests, all-target tests, clippy, `make fidelity`, live
  smoke, oracle terminology search, markdownlint, and `git diff --check`.
  `make fidelity` reported new Rust line coverage `0/0` non-baselined added
  executable lines. Live smoke rendered 239 frames, saw 74 distinct frame signatures,
  observed attract, credit, and playing states, injected all required controls,
  and exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778704093033349`

### DC-66: Accepted Adapter Quarantine

Status: `complete`

Goal: move the legacy-importing accepted-machine adapter out of the clean
accepted-behavior facade while preserving the current temporary oracle.

Scope:

- Keep `src/accepted.rs` focused on neutral accepted frame, snapshot, phase,
  direction, event, sound, and runtime delegation contracts.
- Move accepted-machine adaptation, clean-input-to-cabinet-input projection,
  legacy snapshot conversion, legacy event conversion, and runtime/video-size
  bridge calls into `src_legacy/accepted_behavior.rs`.
- Keep `src/oracle.rs` and `src/platform.rs` on the crate-private
  `crate::accepted` facade.
- Keep low-level equivalence tests in `src_legacy/` and point their
  test-only helpers at the legacy adapter.
- Add a public API guard so `src/accepted.rs` does not reintroduce direct
  compatibility or legacy root imports.
- Document the accepted adapter quarantine in README and SPEC.

Acceptance criteria:

- `src/accepted.rs` contains no direct compatibility, legacy root module, or
  red-label imports.
- Accepted facade, accepted adapter, oracle, and public API focused tests pass.
- Normal runtime and oracle behavior remains unchanged.
- README, SPEC, and PLAN describe the accepted adapter boundary.

Validation:

```sh
cargo fmt --check
cargo test --lib accepted::tests
cargo test --lib accepted_behavior::tests
cargo test --lib oracle::tests
cargo test --lib oracle_equivalence_tests
cargo test --lib public_api_tests
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
ACCEPTED_LEGACY_TERMS='compatibility::|red_label|RED_LABEL|crate::input'
ACCEPTED_LEGACY_TERMS="$ACCEPTED_LEGACY_TERMS|crate::machine|crate::machine_state"
ACCEPTED_LEGACY_TERMS="$ACCEPTED_LEGACY_TERMS|crate::video|crate::app"
! rg -n "$ACCEPTED_LEGACY_TERMS" src/accepted.rs src/oracle.rs
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-13 21:30:32 BST` Started `DC-66`: posted the cycle start update and
  began moving the legacy-importing accepted-machine adapter out of
  `src/accepted.rs` into `src_legacy/accepted_behavior.rs` while preserving
  current oracle behavior.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778704232856249`
- `2026-05-13 21:53:54 BST` Completed `DC-66`: moved the legacy-importing
  accepted-machine adapter into `src_legacy/accepted_behavior.rs`, kept
  `src/accepted.rs` as neutral accepted-behavior contracts plus delegation,
  added a public API guard against direct legacy imports in `src/accepted.rs`,
  and updated the legacy equivalence tests to use the quarantined adapter.
  Validation passed with the DC-66 gate: focused accepted/adapter/oracle/API
  tests, all-target tests, clippy, `make fidelity`, live smoke, clean
  accepted/oracle terminology search, markdownlint, and `git diff --check`.
  `make fidelity` reported new Rust line coverage `5/5` non-baselined added
  executable lines. Live smoke rendered 239 frames, saw 74 distinct frame signatures,
  observed attract, credit, and playing states, injected all required controls,
  and exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778705634783139`

### DC-67: Compatibility Namespace Quarantine

Status: `complete`

Goal: keep temporary oracle and tool access intact while moving the
compatibility re-export details out of the clean crate root.

Scope:

- Add `src_legacy/compatibility.rs` as the owner of the doc-hidden
  `defender::compatibility` re-export namespace.
- Keep `src/lib.rs` focused on clean public exports, crate-private legacy path
  adapters, and the one doc-hidden compatibility path declaration.
- Preserve README media tooling and legacy equivalence tests that still use
  the temporary compatibility namespace.
- Add a public API guard that fails if the compatibility re-export map moves
  back into `src/lib.rs`.

Acceptance criteria:

- `defender::compatibility` behavior is unchanged for current temporary users.
- `src/lib.rs` no longer contains inline compatibility re-export details.
- README, SPEC, and PLAN document the compatibility namespace ownership.

Validation:

```sh
cargo fmt --check
cargo test --lib public_api_tests
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-13 21:57:04 BST` Started `DC-67`: posted the cycle start update and
  began moving the doc-hidden compatibility re-export details from clean
  `src/lib.rs` to `src_legacy/compatibility.rs` while preserving temporary
  tooling and oracle access.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778705824813039`
- `2026-05-13 22:18:12 BST` Completed `DC-67`: moved
  `defender::compatibility` re-export ownership to
  `src_legacy/compatibility.rs`, left `src/lib.rs` with only the doc-hidden
  path declaration, added a public API guard against moving the re-export map
  back into the clean crate root, and updated README/SPEC/PLAN to document the
  ownership boundary. Validation passed with the DC-67 gate: formatting,
  focused public API tests, all-target tests, clippy, `make fidelity`, live
  smoke, markdownlint, and `git diff --check`. `make fidelity` reported new
  Rust line coverage `0/0` non-baselined added executable lines. Live smoke
  rendered 239 frames, saw 74 distinct frame signatures, observed attract, credit,
  and playing states, injected all required controls, and exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778707109325719`

### DC-68: Terminal Session Retirement

Status: `complete`

Goal: remove parked terminal-session code from active crate wiring after Kitty
left the runtime surface.

Scope:

- Move the remaining `TerminalGeometry` value type into the legacy video
  renderer that still consumes it.
- Remove the `src_legacy/terminal.rs` path module from active `src/lib.rs`
  wiring.
- Remove `defender::compatibility::terminal` from temporary compatibility
  re-exports.
- Keep `src_legacy/terminal.rs` parked as historical Kitty terminal-session
  evidence, but leave it unwired from production builds.
- Add public API guards that fail if terminal-session code is rewired through
  the clean crate root or compatibility namespace.

Acceptance criteria:

- Current `wgpu` live behavior and fidelity tooling remain unchanged.
- No active root module or compatibility namespace exposes terminal-session
  setup.
- README, SPEC, and PLAN describe the parked terminal-session boundary.

Validation:

```sh
cargo fmt --check
cargo test --lib public_api_tests
cargo test --lib video::tests::raster_size_uses_terminal_pixels_when_available
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-13 22:21:20 BST` Started `DC-68`: posted the cycle start update and
  began removing legacy terminal-session code from active clean crate wiring
  while keeping the legacy video renderer's geometry value type local to that
  renderer.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778707270698529`
- `2026-05-13 22:42:36 BST` Completed `DC-68`: moved `TerminalGeometry` into
  the legacy video renderer, removed `src_legacy/terminal.rs` from active
  clean crate wiring, removed `defender::compatibility::terminal`, and added
  public API guards that keep terminal-session setup out of the active root and
  compatibility namespace. Validation passed with the DC-68 gate: formatting,
  focused public API and video tests, all-target tests, clippy,
  `make fidelity`, live smoke, markdownlint, and `git diff --check`.
  `make fidelity` reported new Rust line coverage `0/0` non-baselined added
  executable lines. Live smoke rendered 239 frames, saw 74 distinct frame signatures,
  observed attract, credit, and playing states, injected all required controls,
  and exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778708566793249`

### DC-69: Trace Sample Oracle Quarantine

Status: `complete`

Goal: keep generated long-trace sample fixture data out of clean crate-root
wiring while preserving the legacy machine oracle behavior that still consumes
that evidence.

Scope:

- Move `src_legacy/red_label_trace_samples.rs` from the active clean crate root
  into the private legacy machine oracle module tree.
- Remove the generated fixture module from the root legacy adapter guard in
  `src/lib.rs`.
- Add a public API guard that fails if generated trace samples become
  root-wired or compatibility-exported again.
- Keep generated trace sample tests and current oracle behavior intact.
- Document that generated long-trace fixture data is historical oracle evidence,
  not a clean root adapter.

Acceptance criteria:

- `src/lib.rs` no longer declares `red_label_trace_samples`.
- The fixture module is private to `src_legacy/machine.rs`.
- Current oracle fixture behavior and live behavior remain unchanged.
- README, SPEC, and PLAN describe the private trace-sample oracle boundary.

Validation:

```sh
cargo fmt --check
cargo test --lib public_api_tests
cargo test --lib machine::red_label_trace_samples::tests
cargo test --lib oracle_equivalence_tests::clean_fixture_matches_accepted_oracle_events_and_scene_summaries
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
TRACE_SAMPLE_TERMS='red_label_trace_samples|crate::red_label_trace_samples'
TRACE_SAMPLE_TERMS="$TRACE_SAMPLE_TERMS|compatibility::red_label_trace_samples"
! rg -n "$TRACE_SAMPLE_TERMS" src src_legacy/compatibility.rs
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-13 22:47:43 BST` Started `DC-69`: posted the cycle start update and
  began moving generated long-trace sample fixture data out of clean crate-root
  wiring while keeping it available to the legacy machine oracle.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778708773431689`
- `2026-05-13 23:07:38 BST` Completed `DC-69`: moved generated long-trace
  sample fixture data out of clean crate-root wiring and into the private
  legacy machine oracle module tree, added a public API guard against root or
  compatibility re-export regressions, and documented the private oracle
  boundary in README, SPEC, and PLAN. Validation passed with the DC-69 gate:
  formatting, focused public API/private fixture/oracle equivalence tests,
  all-target tests, clippy, `make fidelity`, live smoke, trace-sample root
  search, markdownlint, and `git diff --check`. `make fidelity` reported new
  Rust line coverage `0/0` non-baselined added executable lines. Live smoke
  rendered 240 frames, saw 74 distinct frame signatures, observed attract, credit,
  and playing states, injected all required controls, and exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778710071723699`

### DC-70: Compatibility Re-export Narrowing

Status: `complete`

Goal: shrink the doc-hidden compatibility namespace to the temporary tool and
equivalence contracts still used in this repo, without changing gameplay
behavior.

Scope:

- Remove compatibility re-exports for low-level legacy modules such as assets,
  board, memory layout, ROM verification, sound internals, live runtime, PIA,
  and `wgpu` presenter ownership.
- Keep only compatibility modules required by README media tooling and clean
  equivalence tests: input, machine, machine process/state, red-label math
  types, and video.
- Mark parked low-level legacy adapters as dead-code-tolerant while they remain
  crate-private evidence for the oracle and tests.
- Add a public API guard that fails if low-level compatibility re-exports are
  restored.
- Document the narrowed compatibility boundary in README, SPEC, and PLAN.

Acceptance criteria:

- `defender::compatibility` no longer exposes asset, board, memory, ROM, sound,
  live, PIA, or `wgpu` presenter modules.
- README media generation and clean equivalence tests still compile through the
  reduced temporary compatibility surface.
- Existing gameplay, fidelity fixtures, and live smoke behavior remain
  unchanged.

Validation:

```sh
cargo fmt --check
cargo test --lib public_api_tests::compatibility_namespace_exposes_only_temporary_tool_contracts
cargo test --lib oracle_equivalence_tests
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
RETIRED_COMPAT='app|assets|board|cmos_storage|fidelity|live|pia'
RETIRED_COMPAT="$RETIRED_COMPAT|red_label_memory|red_label_message"
RETIRED_COMPAT="$RETIRED_COMPAT|red_label_wave|rom|sound|terminal"
! rg -n "pub mod ($RETIRED_COMPAT|wgpu_presenter)" src_legacy/compatibility.rs
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-13 23:12:14 BST` Started `DC-70`: posted the cycle start update and
  began narrowing the doc-hidden compatibility namespace to the temporary
  contracts still used by README media tooling and clean equivalence tests.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778710163561459`
- `2026-05-13 23:33:27 BST` Completed `DC-70`: removed low-level compatibility
  re-exports for asset, board, memory, ROM, sound, live, PIA, and `wgpu`
  presenter internals; kept only the temporary README media and clean
  equivalence contracts; added the public API regression guard; and documented
  the narrowed boundary in README, SPEC, and PLAN. Validation passed with the
  DC-70 gate: formatting, focused compatibility API guard, all-target tests,
  clippy, `make fidelity`, live smoke, retired compatibility export search,
  markdownlint, and `git diff --check`. `make fidelity` reported new Rust line
  coverage `0/0` non-baselined added executable lines. Live smoke rendered 239
  frames, saw 74 distinct frame signatures, observed attract, credit, and playing
  states, injected all required controls, and exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778711608813249`

### DC-71: Red-Label Compatibility Export Retirement

Status: `complete`

Goal: remove the remaining red-label math-type export from the doc-hidden
compatibility namespace while preserving crate-private oracle behavior.

Scope:

- Remove `defender::compatibility::red_label` now that in-repo callers no
  longer require that temporary export.
- Keep the legacy `red_label` adapter crate-private for the accepted-behavior
  bridge and oracle internals.
- Strengthen the public API guard so restoring the red-label compatibility
  export fails a focused test.
- Update README, SPEC, and PLAN to describe the narrower compatibility surface.

Acceptance criteria:

- `defender::compatibility` exposes only input, machine, machine process/state,
  and video temporary contracts.
- Existing accepted-machine, oracle, README media, and live smoke behavior
  remain unchanged.
- Docs describe red-label math types as crate-private oracle wiring, not
  compatibility API.

Validation:

```sh
cargo fmt --check
cargo test --lib public_api_tests::compatibility_namespace_exposes_only_temporary_tool_contracts
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
! rg -n "pub mod red_label" src_legacy/compatibility.rs
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-13 23:35:58 BST` Started `DC-71`: posted the cycle start update and
  began retiring the remaining red-label math-type export from the doc-hidden
  compatibility namespace.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778711721691489`
- `2026-05-13 23:56:56 BST` Completed `DC-71`: removed
  `defender::compatibility::red_label`, moved the legacy equivalence test
  import to crate-private oracle wiring, kept the red-label adapter hidden at
  the root with the other parked oracle modules, and updated README, SPEC, and
  PLAN. Validation passed with the DC-71 gate: formatting, focused
  compatibility API guard, focused oracle-equivalence tests, all-target tests,
  clippy, `make fidelity`, live smoke, retired export search, markdownlint,
  and `git diff --check`. `make fidelity` reported new Rust line coverage
  `0/0` non-baselined added executable lines. Live smoke rendered 240 frames,
  saw 74 distinct frame signatures, observed attract, credit, and playing states,
  injected all required controls, and exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778713016113959`

### DC-72: Process/State Compatibility Export Retirement

Status: `complete`

Goal: remove unused machine process/state exports from the doc-hidden
compatibility namespace while preserving crate-private oracle behavior.

Scope:

- Remove `defender::compatibility::machine_process` and
  `defender::compatibility::machine_state` now that in-repo callers no longer
  require those temporary exports.
- Keep the legacy process/state adapters crate-private for the accepted
  behavior bridge and oracle internals.
- Move legacy clean-equivalence test imports to crate-private oracle wiring.
- Strengthen the public API guard so restoring these process/state
  compatibility exports fails a focused test.
- Update README, SPEC, and PLAN to describe the compatibility namespace as the
  remaining README media surface: input, machine, and video.

Acceptance criteria:

- `defender::compatibility` exposes only input, machine, and video temporary
  contracts.
- Existing accepted-machine, oracle equivalence, README media, and live smoke
  behavior remain unchanged.
- Docs describe machine process/state contracts as crate-private oracle wiring,
  not compatibility API.

Validation:

```sh
cargo fmt --check
cargo test --lib public_api_tests::compatibility_namespace_exposes_only_temporary_tool_contracts
cargo test --lib oracle_equivalence_tests
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
! rg -n "pub mod (machine_process|machine_state)" src_legacy/compatibility.rs
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-13 23:59:21 BST` Started `DC-72`: posted the cycle start update and
  began removing the unused machine process/state exports from the doc-hidden
  compatibility namespace while keeping the underlying legacy adapters
  crate-private for oracle internals.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778713129737859`
- `2026-05-14 00:20:02 BST` Completed `DC-72`: removed
  `defender::compatibility::machine_process` and
  `defender::compatibility::machine_state`, moved the legacy equivalence test
  import to crate-private oracle wiring, left the process/state adapters
  crate-private at the root, and updated README, SPEC, and PLAN. Validation
  passed with the DC-72 gate: formatting, focused compatibility API guard,
  focused oracle-equivalence tests, all-target tests, clippy, `make fidelity`,
  live smoke, retired export search, markdownlint, and `git diff --check`.
  `make fidelity` reported new Rust line coverage `0/0` non-baselined added
  executable lines. Live smoke rendered 239 frames, saw 74 distinct frame signatures,
  observed attract, credit, and playing states, injected all required controls,
  and exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778714403527089`

### DC-73: Sprite-First Plan and Internal Compatibility Import Retirement

Status: `complete`

Goal: record sprite-first rendering as an explicit rewrite requirement and
remove the remaining internal oracle-equivalence test dependency on the
temporary compatibility namespace.

Scope:

- Document that clean `wgpu` rendering should use sprite assets, texture
  atlases, and batched sprite draws as the production representation.
- Move `src_legacy/oracle_equivalence_tests.rs` imports from
  `defender::compatibility` to crate-private legacy oracle modules.
- Add a focused public API guard so internal equivalence tests cannot drift
  back to the compatibility namespace.
- Strengthen the plan requirement that every planned dev-cycle posts Slack
  start and completion updates.
- Update README, SPEC, and PLAN to describe compatibility as README media
  tooling only for this slice.

Acceptance criteria:

- `PLAN.md` explicitly requires sprite-first `wgpu` rendering with atlases and
  batched sprite draws.
- Internal oracle-equivalence tests use crate-private legacy wiring, not the
  doc-hidden `defender::compatibility` namespace.
- `defender::compatibility` remains limited to the temporary README media
  tooling surface for this cycle.
- The work protocol explicitly requires Slack start and completion updates for
  every planned dev-cycle.

Validation:

```sh
cargo fmt --check
cargo test --lib public_api_tests::legacy_equivalence_tests_use_crate_private_oracle_wiring
cargo test --lib oracle_equivalence_tests
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
! rg -n "compatibility::|compatibility\\{" src_legacy/oracle_equivalence_tests.rs
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 07:28:10 BST` Started `DC-73`: posted the cycle start update and
  began recording sprite-first rendering as explicit rewrite scope while
  moving legacy equivalence tests off the doc-hidden compatibility namespace.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778740070294709`
- `2026-05-14 07:50:12 BST` Completed `DC-73`: documented sprite-first
  `wgpu` rendering with sprite assets, texture atlases, and batched sprite
  draws; moved legacy oracle-equivalence tests from the doc-hidden
  compatibility namespace to crate-private oracle wiring; strengthened the
  Slack start/completion update requirement in the work protocol; and updated
  README, SPEC, and PLAN. Validation passed with the DC-73 gate: formatting,
  focused public API guard, oracle-equivalence tests, all-target tests, clippy,
  `make fidelity`, live smoke, retired-import search, markdownlint, and
  `git diff --check`. `make fidelity` reported new Rust line coverage `0/0`
  non-baselined added executable lines. Live smoke rendered 239 frames, saw
  74 distinct frame signatures, observed attract, credit, and playing states,
  injected all required controls, and exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778741426762209`

### DC-74: README Media Compatibility Namespace Retirement

Status: `complete`

Goal: retire the final public compatibility namespace by moving README media
generation behind a narrow high-level facade.

Scope:

- Add a doc-hidden `defender::readme_media` facade owned by `src_legacy/` so
  README media generation no longer imports low-level machine, input, or video
  contracts.
- Update `examples/generate_readme_media.rs` to consume the high-level media
  facade.
- Delete the final `defender::compatibility` re-export namespace and its
  `src_legacy/compatibility.rs` export map.
- Strengthen public API guards so restoring the compatibility namespace or
  README media low-level imports fails a focused test.
- Update README, SPEC, and PLAN to describe README media as the only
  doc-hidden tool facade left from this slice.

Acceptance criteria:

- `examples/generate_readme_media.rs` imports `defender::readme_media`, not
  `defender::compatibility`.
- `src/lib.rs` no longer wires `pub mod compatibility`, and
  `src_legacy/compatibility.rs` is removed.
- Root legacy machine, input, and video modules remain crate-private.
- README and SPEC no longer describe the compatibility namespace as active
  tool API.

Validation:

```sh
cargo fmt --check
cargo test --lib public_api_tests::compatibility_namespace_is_retired
cargo test --lib public_api_tests::readme_media_facade_is_legacy_owned_and_doc_hidden
cargo test --lib readme_media::tests
cargo test --example generate_readme_media
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
! rg -n \
  -e "defender::compatibility" \
  -e "use defender::compatibility" \
  -e '#\\[path = "\\.\\./src_legacy/compatibility\\.rs"\\]' \
  src src_legacy examples README.md SPEC.md
test ! -e src_legacy/compatibility.rs
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 07:53:51 BST` Started `DC-74`: posted the cycle start update and
  began retiring the final public compatibility namespace by moving README
  media generation to a narrow high-level facade.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778741631993389`
- `2026-05-14 08:19:18 BST` Completed `DC-74`: added the doc-hidden
  `defender::readme_media` facade, moved README media generation off
  low-level machine/video imports, removed `src_legacy/compatibility.rs`, and
  retired the public `defender::compatibility` namespace. Validation passed
  with formatting, focused public API/readme-media tests, all-target tests,
  clippy, `make fidelity`, live smoke, compatibility grep guard, deleted-file
  guard, markdownlint, and `git diff --check`. `make fidelity` reported new
  Rust line coverage `0/0` non-baselined added executable lines. Live smoke
  rendered 239 frames, saw 74 distinct frame signatures, observed attract, credit,
  and playing states, injected all required controls, and exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778743156254659`

### DC-75: Clean Equivalence Gate Foundation

Status: `complete`

Goal: establish clean state/event/render/sound equivalence signatures before
retiring memory-oriented trace gates.

Scope:

- Add a clean `src/fidelity.rs` frame signature contract over `GameSnapshot`,
  gameplay events, sound events, and `RenderSceneSummary`.
- Root-wire legacy trace tooling as `legacy_fidelity` so the public clean
  `fidelity` module no longer points at memory-oriented trace code.
- Add focused tests comparing clean frame signatures with the accepted facade
  for credited start and live control input.
- Update README, SPEC, and PLAN to describe clean fidelity signatures and
  legacy trace quarantine.

Acceptance criteria:

- `src/fidelity.rs` contains no direct legacy module imports.
- Public clean API exposes `GameplayEquivalenceSignature`.
- Historical trace tooling remains available under the crate-private
  `legacy_fidelity` root adapter.
- Focused signature tests cover state, gameplay events, sound events, and
  render summaries.

Validation:

```sh
cargo fmt --check
cargo test --lib fidelity::tests
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_accepted_facade
cargo test --lib public_api_tests::legacy_modules_are_crate_private_at_root
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 08:21:38 BST` Started `DC-75`: posted the cycle start update and
  began adding clean frame-equivalence signatures as the first memory-oriented
  oracle retirement slice.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778743298143089`
- `2026-05-14 08:48:24 BST` Completed `DC-75`: added the clean
  `GameplayEquivalenceSignature` contract in `src/fidelity.rs`, root-wired
  historical trace tooling as crate-private `legacy_fidelity`, and added
  focused signature tests against the accepted facade for credited start and
  live control input. README, SPEC, and PLAN now describe clean fidelity
  signatures and leave broad memory-model retirement to `DC-77`. Validation
  passed with formatting, focused fidelity/public API tests, all-target tests,
  clippy, `make fidelity`, live smoke, markdownlint, and `git diff --check`.
  `make fidelity` reported new Rust line coverage `0/0` non-baselined added
  executable lines. Live smoke rendered 239 frames, saw 74 distinct frame
  CRCs, observed attract, credit, and playing states, injected all required
  controls, and exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778744904031159`

### DC-76: Clean Audio Boundary Isolation

Status: `complete`

Goal: remove the clean audio runtime's direct dependency on legacy frame
outputs.

Scope:

- Remove `machine_state::FrameOutput` from `src/audio.rs`.
- Keep live audio submission on clean `GameFrame` and `SoundEvent` contracts.
- Move legacy output-to-clean audio adaptation into `src_legacy/live.rs`.
- Add a public API guard so clean audio cannot re-import legacy frame outputs.
- Update README, SPEC, and PLAN to document the audio boundary.

Acceptance criteria:

- `src/audio.rs` contains no `FrameOutput` or legacy `machine_state` imports.
- Live audio still receives accepted startup sound events.
- Legacy live code owns the only output-to-clean audio adapter.
- Docs describe the clean audio boundary.

Validation:

```sh
cargo fmt --check
cargo test --lib audio::tests
cargo test --lib live::tests::live_core_driver_feeds_sound_events_to_audio_runtime
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_accepted_facade
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
! rg -n "FrameOutput|from_frame_output|submit_frame_output|machine_state" src/audio.rs
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 08:50:28 BST` Started `DC-76`: posted the cycle start update and
  began moving legacy frame-output audio adaptation out of the clean audio
  runtime.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778745028598859`
- `2026-05-14 09:13:39 BST` Completed `DC-76`: removed legacy
  `FrameOutput` and `machine_state` dependencies from clean `src/audio.rs`,
  moved output-to-clean audio adaptation into `src_legacy/live.rs`, and added
  a public API guard to keep clean audio on `GameFrame` and `SoundEvent`
  contracts. README, SPEC, and PLAN now document the clean audio boundary.
  Validation passed with formatting, focused audio/live/public API tests,
  all-target tests, clippy, `make fidelity`, live smoke, the clean audio
  static guard, markdownlint, and `git diff --check`. `make fidelity` reported
  new Rust line coverage `9/9` non-baselined added executable lines. Live
  smoke rendered 239 frames, saw 74 distinct frame signatures, observed attract,
  credit, and playing states, injected all required controls, and exited
  cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778746419261859`

### DC-77: Clean Sound Event Contract Isolation

Status: `complete`

Goal: remove accepted sound command-byte mapping from the clean gameplay domain
and keep that adapter logic at the oracle boundary.

Scope:

- Remove `SoundEvent::from_accepted_command`,
  `SoundEvent::accepted_command`, and `UnmappedAcceptedCommand` from
  `src/game.rs`.
- Add oracle-owned accepted sound command mapping and test-support helpers.
- Route clean fidelity and legacy oracle-equivalence tests through the oracle
  mapping helper.
- Add a public API guard so clean gameplay contracts cannot re-own accepted
  sound command mapping.
- Update README, SPEC, and PLAN to document the sound-event boundary.

Acceptance criteria:

- `src/game.rs` contains no accepted sound command mapping helpers or accepted
  unmapped-command variant.
- `src/oracle.rs` owns the accepted sound command mapping into clean
  `SoundEvent` values.
- Clean fidelity and legacy oracle-equivalence tests compare sound events
  through the oracle boundary.
- Docs describe clean `SoundEvent` as a gameplay contract, not a command-byte
  adapter.

Validation:

```sh
cargo fmt --check
cargo test --lib game::tests
cargo test --lib oracle::tests
cargo test --lib fidelity::tests
cargo test --lib oracle_equivalence_tests::clean_fixture_matches_accepted_oracle_events_and_scene_summaries
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_accepted_facade
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
! rg -n "from_accepted_command|accepted_command|UnmappedAcceptedCommand" src/game.rs
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 09:16:30 BST` Started `DC-77`: posted the cycle start update and
  began moving accepted sound command-byte mapping out of the clean gameplay
  contract and into the oracle boundary.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778746590651139`
- `2026-05-14 09:38:12 BST` Completed `DC-77`: removed accepted command-byte
  conversion helpers from clean `SoundEvent`, renamed the unmapped sound
  surface to neutral `UnmappedSoundCommand`, moved accepted sound command
  mapping into `src/oracle.rs`, and routed clean fidelity plus legacy
  oracle-equivalence checks through the oracle helper. Added a public API guard
  so `src/game.rs` cannot re-own accepted command mapping. README, SPEC, and
  PLAN now document the sound-event boundary and move broad memory-model
  retirement to `DC-78`. Validation passed with focused game/oracle/fidelity,
  oracle-equivalence, and public API tests; formatting; all-target tests;
  clippy; `make fidelity`; live smoke; the static guard; markdownlint; and
  `git diff --check`. `make fidelity` reported new Rust line coverage `9/9`
  non-baselined added executable lines. Live smoke rendered 240 frames, saw 74
  distinct frame signatures, observed attract, credit, and playing states, injected
  all required controls, and exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778747892960589`

### DC-78: Clean Fidelity Reference Probe Isolation

Status: `complete`

Goal: remove clean fidelity tests' direct accepted-facade dependencies by
moving reference-machine probing behind oracle test support.

Scope:

- Add an oracle-owned `ReferenceFrameProbe` for test-only reference frames.
- Remove direct `AcceptedFrame` and `AcceptedGameplayMachine` imports from
  `src/fidelity.rs`.
- Keep `GameplayEquivalenceSignature` tests on clean `GameFrame` values and
  oracle-provided reference frames.
- Add a public API guard so clean fidelity cannot import accepted facade types
  directly.
- Update `README.md`, `SPEC.md`, and `PLAN.md`.

Acceptance criteria:

- `src/fidelity.rs` contains no `crate::accepted::`, `AcceptedFrame`,
  `AcceptedGameplayMachine`, or `adapt_accepted_` references.
- Oracle test support owns reference probing from the accepted implementation.
- Fidelity tests still prove clean signatures match a separate reference probe
  for credited start and controls.
- Documentation describes the reference-probe boundary.

Validation:

```sh
cargo fmt --check
cargo test --lib fidelity::tests
cargo test --lib oracle::tests
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_accepted_facade
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
! rg -n \
  -e "crate::accepted::" \
  -e "AcceptedFrame" \
  -e "AcceptedGameplayMachine" \
  -e "adapt_accepted_" \
  src/fidelity.rs
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 09:40:38 BST` Started `DC-78`: posted the cycle start update and
  began moving clean fidelity reference-machine probing out of `src/fidelity.rs`
  and behind oracle test support while keeping the fidelity contract on clean
  `GameFrame` signatures.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778748038056829`
- `2026-05-14 10:04:58 BST` Completed `DC-78`: added the oracle-owned
  `ReferenceFrameProbe`, removed direct accepted facade imports and adapter
  helpers from `src/fidelity.rs`, strengthened the public API guard against
  reintroducing those references, and updated README, SPEC, and PLAN to
  describe the reference-probe boundary. Broad memory-oriented oracle
  retirement moved to `DC-79`. Validation passed with formatting; focused
  fidelity, oracle, and public API tests; all-target tests; clippy;
  `make fidelity`; live smoke; the static fidelity accepted-facade guard;
  markdownlint; and `git diff --check`. `make fidelity` matched 10 trace
  fixtures covering 15452 frames and reported new Rust line coverage `0/0`
  non-baselined added executable lines. Live smoke rendered 239 frames, saw 74
  distinct frame signatures, observed attract, credit, and playing states, injected
  all required controls, and exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778749498248829`

### DC-79: Clean Game Simulation Shell

Status: `complete`

Goal: create a clean, sprite-first gameplay simulation foothold that does not
depend on the accepted machine or memory-oriented runtime model.

Scope:

- Add a clean `Game` simulation shell that owns `GameState` and emits
  `GameFrame` values.
- Drive credited start, basic playing controls, player motion, smart bomb
  inventory, and projectile launch through clean deterministic systems.
- Add a renderer-owned projectile sprite id and atlas region so clean game
  frames stay sprite-first.
- Keep live runtime and accepted-oracle behavior unchanged.
- Add tests and public API guards so clean game source stays free of legacy
  implementation terminology.
- Update `README.md`, `SPEC.md`, and `PLAN.md`.

Acceptance criteria:

- `Game` implements `GameSimulation` and advances clean frames without accepted
  machine state.
- Clean game frames contain sprite scene data and no temporary raster payload.
- Playing controls exercise clean control, motion, and projectile systems.
- Public API guards reject legacy implementation terminology in `src/game.rs`.
- Documentation describes the clean game shell as the next oracle-retirement
  foothold.

Validation:

```sh
cargo fmt --check
cargo test --lib game::tests
cargo test --lib renderer::tests::texture_atlas_owns_sprite_regions
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_accepted_facade
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
! rg -n \
  -e "crate::accepted::" \
  -e "crate::machine::" \
  -e "crate::machine_state::" \
  -e "crate::red_label::" \
  -e "red_label" \
  -e "RED_LABEL" \
  -e "source routine" \
  -e "assembler" \
  -e "memory" \
  -e "FrameOutput" \
  src/game.rs
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 10:08:58 BST` Started `DC-79`: posted the cycle start update and
  began adding a clean `Game` simulation shell as a bounded first step toward
  retiring the memory-oriented oracle from production gameplay.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778749749081009`
- `2026-05-14 10:48:29 BST` Completed `DC-79`: added the clean `Game`
  simulation shell, wired it to clean control, motion, and projectile systems,
  emitted sprite-first `GameFrame` scenes without raster payloads, added a
  renderer-owned projectile sprite atlas region, exported the clean game type,
  and strengthened the public API guard against legacy terminology in
  `src/game.rs`. The first full fidelity run exposed three uncovered added
  lines in the clean game shell; focused coverage was added for `Default` and
  left-to-right reversal before rerunning the full gate successfully. Validation
  passed with formatting; focused game, renderer, and public API tests; the full
  Rust test suite; clippy with warnings denied; `make fidelity`; live smoke; the
  static terminology guard; markdownlint; and `git diff --check`. The
  `make fidelity` gate matched 10 trace fixtures covering 15452 frames and
  reported new Rust line coverage `134/134` non-baselined added executable
  lines. Live smoke rendered 239 frames, saw 74 distinct frame signatures, observed
  attract, credit, and playing states, injected all required controls, and
  exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778752130012979`

### DC-80: Clean Production Legacy Import Guard

Status: `complete`

Goal: make the first oracle-retirement boundary enforceable by guarding clean
production modules against direct legacy imports and legacy implementation
terminology.

Scope:

- Add a single public API guard that scans clean source modules for direct
  low-level legacy root imports.
- Keep `src/accepted.rs` as the only clean source that may call the
  `accepted_behavior` adapter.
- Keep `src/oracle.rs` and `src/platform.rs` as the only temporary clean callers
  of the accepted facade.
- Remove remaining production-source references to memory-oriented terminology
  outside the parked legacy tree.
- Update `README.md`, `SPEC.md`, and `PLAN.md`.

Acceptance criteria:

- Clean production modules cannot directly import low-level legacy root modules.
- The accepted-behavior bridge remains quarantined behind `src/accepted.rs`.
- Clean gameplay, systems, renderer, platform, audio, fidelity, and oracle
  sources do not expose legacy implementation terminology.
- Runtime behavior and historical fidelity tooling remain unchanged.

Validation:

```sh
cargo fmt --check
cargo test --lib public_api_tests::clean_module_sources_keep_legacy_access_quarantined
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_accepted_facade
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
! rg -n \
  -e "red_label" \
  -e "RED_LABEL" \
  -e "source routine" \
  -e "assembler" \
  -e "memory" \
  -e "FrameOutput" \
  src/accepted.rs src/audio.rs src/fidelity.rs src/game.rs src/main.rs \
  src/oracle.rs src/platform.rs src/renderer.rs src/systems.rs
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 10:52:08 BST` Started `DC-80`: posted the cycle start update and
  began the first memory-oriented oracle retirement slice by adding a
  source-level guard for clean production modules.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778752325584989`
- `2026-05-14 11:14:03 BST` Completed `DC-80`: added
  `public_api_tests::clean_module_sources_keep_legacy_access_quarantined` so
  clean module sources cannot import low-level legacy root modules,
  `src/accepted.rs` remains the only accepted-behavior bridge, and only
  `src/oracle.rs` plus `src/platform.rs` can call the temporary accepted facade.
  Removed remaining clean-source references to memory-oriented terminology,
  documented the guard in `README.md` and `SPEC.md`, and moved the broader
  memory-oriented oracle retirement milestone to `DC-82`. Validation passed with
  formatting; focused public API guards; the full Rust test suite; clippy with
  warnings denied; `make fidelity`; live smoke; the static clean-source
  terminology scan; markdownlint; and `git diff --check`. The `make fidelity`
  gate matched 10 trace fixtures covering 15452 frames and reported new Rust
  line coverage `0/0` non-baselined added executable lines. Live smoke rendered
  239 frames, saw 74 distinct frame signatures, observed attract, credit, and playing
  states, injected all required controls, and exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778753657368129`

### DC-81: Gameplay Oracle Public Surface Retirement

Status: `complete`

Goal: remove the gameplay oracle from the supported public API while keeping it
available internally for fidelity and regression tests.

Scope:

- Make `src/oracle.rs` a crate-private module instead of a supported public
  module.
- Remove the public `GameplayOracle` re-export from `src/lib.rs`.
- Keep internal oracle tests, fidelity signatures, and legacy equivalence tests
  working through crate-private wiring.
- Replace the public oracle contract test with a public clean `Game` simulation
  contract and add a guard that the oracle stays internal.
- Update `README.md`, `SPEC.md`, and `PLAN.md`.

Acceptance criteria:

- Supported public API exposes clean gameplay contracts, not the temporary
  oracle.
- Internal fidelity tooling can still instantiate the oracle for historical
  comparison.
- Runtime behavior and trace fixture fidelity remain unchanged.

Validation:

```sh
cargo fmt --check
cargo test --lib public_api_tests::clean_contracts_have_public_game_simulation
cargo test --lib public_api_tests::gameplay_oracle_is_internal_fidelity_wiring
cargo test --lib oracle::tests
cargo test --lib fidelity::tests::clean_frame_signatures_match_reference_probe_for_start_and_controls
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
! rg -n \
  -e "pub mod oracle;" \
  -e "pub use oracle::GameplayOracle;" \
  -e "crate::GameplayOracle" \
  src
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 11:16:26 BST` Started `DC-81`: posted the cycle start update and
  began retiring the gameplay oracle from the supported public API while keeping
  internal fidelity wiring available.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778753786573509`
- `2026-05-14 11:42:10 BST` Completed `DC-81`: made the oracle module
  crate-private, removed the public `GameplayOracle` export, replaced the public
  API oracle contract with a clean `Game` simulation contract, kept the
  machine-backed oracle as internal fidelity wiring, and documented the updated
  boundary in `README.md`, `SPEC.md`, and this plan. Validation passed with
  formatting; focused public API, oracle, and fidelity-signature tests; the full
  Rust target suite; clippy with warnings denied; `make fidelity`; live smoke;
  the public-oracle static scan; markdownlint; and `git diff --check`. The
  `make fidelity` gate matched 10 trace fixtures covering 15452 frames and
  reported new Rust line coverage `0/0` non-baselined added executable lines.
  Live smoke rendered 239 frames, saw 74 distinct frame signatures, observed attract,
  credit, and playing states, injected all required controls, and exited
  cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778755376066379`

### DC-82: Render Signature Terminology

Status: `complete`

Goal: remove memory-oriented hash/CRC labels from clean render evidence and
live-smoke reporting while preserving the accepted comparison behavior.

Scope:

- Rename clean render-scene evidence from `visual_hash` to
  `visual_signature`.
- Rename live-smoke frame and phase diversity metrics from CRC labels to
  render-signature labels.
- Keep historical CRC trace fixtures and legacy machine evidence quarantined in
  `src_legacy/`.
- Update README, SPEC, and this plan so clean fidelity language describes
  state, event, sound, and render signatures.

Acceptance criteria:

- Clean `src/` APIs no longer expose render hash terminology.
- Supported live-smoke output no longer exposes frame or scene CRC terminology.
- Historical CRC terminology remains only where it describes legacy trace or
  ROM verification evidence.

Validation:

```sh
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
! rg -n \
  -e "visual_hash" \
  -e "distinct_frame_crcs" \
  -e "visual_crcs" \
  -e "frame_crcs" \
  -e "frame CRC" \
  -e "scene CRC" \
  src src_legacy/wgpu_presenter.rs src_legacy/live.rs \
  src_legacy/accepted_behavior.rs src_legacy/oracle_equivalence_tests.rs \
  README.md SPEC.md
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 11:45:41 BST` Started `DC-82`: posted the cycle start update and
  began the first memory-oriented oracle retirement slice by moving clean
  render equivalence and live-smoke evidence from hash/CRC labels to render
  signature terminology.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778755466066549`
- `2026-05-14 12:07:27 BST` Completed `DC-82`: renamed clean render evidence
  from visual hashes to visual signatures, renamed live-smoke diversity metrics
  from frame/visual CRCs to frame/visual signatures, updated the public rewrite
  docs, and kept legacy CRC terminology limited to historical trace and ROM
  evidence. Validation passed with formatting; focused renderer, fidelity, and
  `wgpu` smoke unit tests; the full Rust target suite; clippy with warnings
  denied; `make fidelity`; live smoke; stale render-hash/frame-CRC label scans
  across clean source, supported live-smoke paths, README, and SPEC;
  markdownlint; and `git diff --check`. The `make fidelity` gate matched 10
  trace fixtures covering 15452 frames and reported new Rust line coverage
  `30/30` non-baselined added executable lines. Live smoke rendered 239 frames,
  saw 74 distinct frame signatures, observed attract, credit, and playing
  states, injected all required controls, and exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778756923466539`

### DC-83: Production Model Quarantine

Status: `complete`

Goal: continue retiring the memory-oriented production model by moving the next
remaining legacy-facing runtime dependency behind clean gameplay boundaries.

Scope:

- Move platform launch off the temporary accepted facade and behind a private
  clean runtime bridge.
- Keep `src/runtime.rs` as the only clean launch owner for the current accepted
  runtime adapter.
- Add focused boundary tests so `src/platform.rs` depends on clean runtime
  configuration and does not call accepted or legacy launch functions directly.
- Keep fixture parsers and historical oracle tooling available only where they
  provide review value.
- Preserve accepted gameplay behavior and live `wgpu` behavior.

Acceptance criteria:

- The selected runtime path reads as clean gameplay code at its public boundary.
- Any remaining memory-oriented names are explicitly quarantined in legacy or
  fidelity modules.
- Public API and module names continue moving away from red-label, ROM, source
  routine, and assembler process terminology.

Validation:

```sh
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
cargo test --lib platform::tests::
cargo test --lib runtime::tests::
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_quarantined_adapters
cargo test --lib public_api_tests::clean_module_sources_keep_legacy_access_quarantined
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 12:11:22 BST` Started `DC-83`: posted the cycle start update and
  began quarantining the production launch path by moving `src/platform.rs` off
  the accepted facade and behind a private clean runtime bridge.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778757080478579`
- `2026-05-14 12:50:11 BST` Completed `DC-83`: added a private
  `src/runtime.rs` runtime host, moved `src/platform.rs` to launch through that
  clean bridge, removed runtime launch from `src/accepted.rs`, updated public
  API guards so only the runtime bridge owns the accepted launch adapter, and
  documented the new boundary in README, SPEC, and this plan. Validation passed
  with formatting; focused platform, runtime, and public API tests; `cargo
  check`; `cargo test --all-targets`; clippy with warnings denied;
  `make fidelity`; live smoke; a boundary scan preventing direct accepted/app
  launch calls from clean runtime-facing source; markdownlint; and
  `git diff --check`. The `make fidelity` gate matched 10 trace fixtures
  covering 15452 frames and reported new Rust line coverage `3/3`
  non-baselined added executable lines. Live smoke rendered 239 frames, saw 74
  distinct frame signatures, observed attract, credit, and playing states,
  injected all required controls, and exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778759467260149`

### DC-84: Runtime Config Handoff

Status: `complete`

Goal: make the private runtime bridge consume clean runtime configuration
explicitly for the next supported launch path instead of leaving configuration
hidden behind the accepted adapter.

Scope:

- Identify one live or smoke launch mode that can be selected from clean
  `RuntimeConfig` without relying on legacy CLI parsing.
- Add a clean launch-command adapter inside the private runtime bridge.
- Preserve current default CLI behavior and the accepted adapter while clean
  launch ownership expands.
- Keep `wgpu` live behavior and smoke metrics unchanged.

Acceptance criteria:

- The selected launch mode is driven by clean `RuntimeConfig`.
- The accepted adapter remains private to the runtime bridge.
- Public API and module names continue moving away from red-label, ROM, source
  routine, and assembler process terminology.

Validation:

```sh
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 12:52:55 BST` Started `DC-84`: posted the cycle start update and
  began moving clean `RuntimeConfig` handling into the private runtime bridge,
  with config-driven `wgpu` live smoke as the first direct launch handoff while
  preserving default CLI behavior through the accepted adapter.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778759572099959`
- `2026-05-14 13:15:30 BST` Completed `DC-84`: added a private runtime launch
  command that maps clean smoke configuration to `wgpu` live smoke with clean
  control-profile and CMOS-path handoff, kept default CLI launch on the
  accepted runtime adapter, strengthened runtime/public API coverage for the
  handoff, and documented the boundary in README, SPEC, and this plan.
  Validation passed with formatting; focused runtime, platform, and public API
  tests; `cargo test --all-targets`; clippy with warnings denied;
  `make fidelity`; live smoke; markdownlint; and `git diff --check`.
  `make fidelity` matched 10 trace fixtures covering 15452 frames and reported
  new Rust line coverage `23/23` non-baselined added executable lines. Live
  smoke rendered 240 frames, saw 74 distinct frame signatures, observed
  attract, credit, and playing states, injected all required controls, and
  exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778760958421079`

### DC-85: Configured Interactive Launch Handoff

Status: `complete`

Goal: make configured interactive runtime launches use clean `RuntimeConfig`
for controls, audio, and persistence without changing the default CLI entry
path.

Scope:

- Separate default CLI launch ownership from configured interactive runtime
  launch ownership inside the private runtime bridge.
- Map clean control, audio, and CMOS configuration to the current `wgpu`
  interactive runtime path.
- Preserve `cargo run`, CLI parsing, and accepted-adapter behavior for default
  command-line entry.
- Keep smoke behavior and public API guards intact.

Acceptance criteria:

- `platform::run_with_config(RuntimeConfig::default())` can launch through
  clean configuration instead of CLI argument parsing.
- `platform::run()` preserves current command-line behavior.
- The accepted adapter remains private to the runtime bridge until it is
  replaced by clean gameplay systems.

Validation:

```sh
cargo fmt --check
cargo test --lib runtime::tests::
cargo test --lib platform::tests::
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_quarantined_adapters
cargo test --lib public_api_tests::clean_module_sources_keep_legacy_access_quarantined
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 13:18:01 BST` Started `DC-85`: posted the cycle start update and
  began separating default CLI launch from configured runtime launch so clean
  `RuntimeConfig` can drive interactive `wgpu` controls, audio, and CMOS
  handoff without changing command-line entry behavior.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778761092533899`
- `2026-05-14 13:39:11 BST` Completed `DC-85`: split the private runtime
  bridge so `platform::run()` preserves default command-line behavior through
  the accepted adapter while `platform::run_with_config` maps clean
  interactive controls, audio, and CMOS settings into the `wgpu` live launch
  path. Smoke config remains directly routed to `wgpu` live smoke. Validation
  passed with formatting; focused runtime, platform, and public API tests;
  `cargo test --all-targets`; clippy with warnings denied; `make fidelity`;
  live smoke; markdownlint; and `git diff --check`. `make fidelity` matched
  10 trace fixtures covering 15452 frames and reported new Rust line coverage
  `20/20` non-baselined added executable lines. Live smoke rendered 239 frames,
  saw 74 distinct frame signatures, observed attract, credit, and playing
  states, injected all required controls, and exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778762365968649`

### DC-86: Clean Smoke CLI Handoff

Status: `complete`

Goal: let the clean platform/runtime boundary own the supported smoke-launch
CLI path while preserving the accepted adapter for unsupported historical
commands.

Scope:

- Add a narrow clean CLI handoff for `--live-smoke` and the live configuration
  flags that already correspond to `RuntimeConfig`.
- Keep unsupported or historical commands delegated to the accepted CLI
  adapter.
- Preserve existing help text and command behavior unless the clean parser
  explicitly owns that path.
- Keep `wgpu` live-smoke output and metrics unchanged.

Acceptance criteria:

- `cargo run -- --live-smoke` reaches the config-driven runtime smoke launch
  instead of relying on accepted CLI parsing.
- Unknown or non-runtime historical commands still follow the accepted adapter.
- Public API guards identify the clean CLI-owned runtime path and keep legacy
  launch adapters quarantined.

Validation:

```sh
cargo fmt --check
cargo test --lib runtime::tests::
cargo test --lib platform::tests::
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_quarantined_adapters
cargo test --lib public_api_tests::clean_module_sources_keep_legacy_access_quarantined
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 13:42:20 BST` Started `DC-86`: posted the cycle start update
  and began moving supported `--live-smoke` CLI launches onto the clean
  `RuntimeConfig` path while preserving accepted CLI delegation for default
  live play, help, malformed smoke arguments, and historical commands.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778762555090619`
- `2026-05-14 14:04:00 BST` Completed `DC-86`: clean platform CLI parsing now
  owns valid `--live-smoke` launches and maps supported live configuration
  flags into `RuntimeConfig`, while default live play, help, malformed smoke
  arguments, and historical commands stay delegated to the accepted adapter.
  Public API guards now assert the clean smoke CLI path remains source-visible.
  Validation passed with formatting; focused platform, runtime, and public API
  tests; `cargo test --all-targets`; clippy with warnings denied;
  `make fidelity`; live smoke; markdownlint; and `git diff --check`.
  `make fidelity` matched 10 trace fixtures covering 15452 frames and reported
  new Rust line coverage `37/37` non-baselined added executable lines. Live
  smoke rendered 239 frames, saw 74 distinct frame signatures, observed
  attract, credit, and playing states, injected all required controls, and
  exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778763856173449`

### DC-87: Clean Interactive CLI Handoff

Status: `complete`

Goal: let the clean platform/runtime boundary own the supported default
interactive live-play CLI path while preserving the accepted adapter for help,
malformed live options, and historical commands.

Scope:

- Extend clean CLI handoff to the no-argument live launch and valid
  `--input-profile`, `--mute`, and `--cmos-path` interactive combinations.
- Keep `--help`, malformed live options, unsupported flags, and historical
  commands delegated to the accepted CLI adapter.
- Preserve the current help text and historical command behavior.
- Keep interactive launch mapping on `RuntimeConfig` and the `wgpu` live
  runtime bridge.

Acceptance criteria:

- No-argument launch and valid live configuration flags are classified as clean
  interactive runtime launches.
- Help, malformed live options, and historical commands still follow the
  accepted adapter.
- Public API guards identify the clean-owned interactive CLI path and keep
  root launch adapters quarantined.

Validation:

```sh
cargo fmt --check
cargo test --lib platform::tests::
cargo test --lib runtime::tests::
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_quarantined_adapters
cargo test --lib public_api_tests::clean_module_sources_keep_legacy_access_quarantined
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
cargo run -- --help
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 14:06:16 BST` Started `DC-87`: posted the cycle start update
  and began extending the clean CLI handoff to no-argument interactive live
  play plus valid `--input-profile`, `--mute`, and `--cmos-path`
  combinations, while preserving accepted adapter delegation for help,
  malformed live options, unsupported flags, and historical commands.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778763987144479`
- `2026-05-14 14:27:25 BST` Completed `DC-87`: clean platform CLI parsing now
  owns no-argument interactive live launch plus valid `--input-profile`,
  `--mute`, and `--cmos-path` combinations. `--live-smoke` remains
  clean-owned, while help, malformed live options, unsupported flags, and
  historical commands still delegate to the accepted adapter. Validation
  passed with formatting; focused platform, runtime, and public API tests;
  `cargo test --all-targets`; clippy with warnings denied; `make fidelity`;
  live smoke; help output; markdownlint; and `git diff --check`.
  `make fidelity` matched 10 trace fixtures covering 15452 frames and reported
  new Rust line coverage `3/3` non-baselined added executable lines. Live
  smoke rendered 239 frames, saw 74 distinct frame signatures, observed
  attract, credit, and playing states, injected all required controls, and
  exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778765260010409`

### DC-88: Clean CLI Classification Boundary

Status: `complete`

Goal: make clean CLI ownership explicit and maintainable by separating runtime
classification from launch dispatch while preserving current command behavior.

Scope:

- Extract the platform CLI classifier into a small clean boundary that returns
  either `RuntimeConfig` or accepted-adapter delegation.
- Keep no-argument launch, valid interactive live options, and valid
  `--live-smoke` paths clean-owned.
- Keep help, malformed live options, unsupported flags, and historical commands
  delegated to the accepted adapter.
- Keep public launch dispatch thin and source-visible for API guards.

Acceptance criteria:

- `platform::run()` dispatches through the clean classifier without embedding
  parsing branches in the launch function.
- Supported runtime CLI paths produce `RuntimeConfig` without touching the
  accepted adapter.
- Delegated paths remain behavior-compatible with the accepted CLI parser.
- Public API guards identify the classifier boundary and keep root launch
  adapters quarantined.

Validation:

```sh
cargo fmt --check
cargo test --lib platform::tests::
cargo test --lib runtime::tests::
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_quarantined_adapters
cargo test --lib public_api_tests::clean_module_sources_keep_legacy_access_quarantined
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
cargo run -- --help
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 14:31:38 BST` Started `DC-88`: posted the cycle start update
  and began separating clean CLI classification from runtime launch dispatch
  while keeping no-argument live play, valid live options, and valid
  `--live-smoke` paths clean-owned and preserving accepted adapter delegation
  for help, malformed live options, unsupported flags, and historical commands.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778765521036469`
- `2026-05-14 15:12:20 BST` Completed `DC-88`: platform launch now dispatches
  from an explicit clean CLI classification boundary. The classifier returns
  clean `RuntimeConfig` values for no-argument live play, valid live options,
  and valid `--live-smoke` paths while unsupported, malformed, help, and
  historical command paths remain accepted-adapter delegations. Validation
  passed with formatting; focused platform, runtime, and public API tests;
  `cargo test --all-targets`; clippy with warnings denied; `make fidelity`;
  live smoke; help output; markdownlint; and `git diff --check`.
  `make fidelity` matched 10 trace fixtures covering 15452 frames and reported
  new Rust line coverage `26/26` non-baselined added executable lines. Live
  smoke rendered 239 frames, saw 74 distinct frame signatures, observed
  attract, credit, and playing states, injected all required controls, and
  exited cleanly.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778767934967469`

### DC-89: Clean CLI Help Ownership

Status: `complete`

Goal: move help handling out of accepted-adapter delegation and into the clean
platform/runtime CLI surface while preserving current user-facing help output.

Scope:

- Add a clean help classification or runtime command for `--help` and `-h`.
- Preserve the current help text and command list while moving ownership away
  from the legacy app parser.
- Keep malformed live options, unsupported flags, and historical commands
  delegated to the accepted adapter until clean replacements exist.
- Update public API guards so help ownership is source-visible in the clean
  platform/runtime boundary.

Acceptance criteria:

- `cargo run -- --help` is served by clean runtime/platform code without
  invoking accepted CLI delegation.
- Help output remains behavior-compatible with the current text.
- Accepted-adapter delegation remains covered for malformed and historical
  command paths.
- README, SPEC, and PLAN stay aligned with the clean CLI ownership boundary.

Validation:

```sh
cargo fmt --check
cargo test --lib platform::tests::
cargo test --lib runtime::tests::
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_quarantined_adapters
cargo test --lib public_api_tests::clean_module_sources_keep_legacy_access_quarantined
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --help
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 15:22:47 BST` Started `DC-89`: began moving `--help` and `-h`
  handling into the clean platform/runtime CLI surface while preserving the
  current help text and keeping malformed, unsupported, and historical command
  paths delegated to the accepted adapter. Slack start update attempted twice
  before implementation, but the Slack connector timed out both times before
  returning a message link. A later start/status retry also timed out before
  completion.
- `2026-05-14 15:45:29 BST` Completed `DC-89`: `--help` and `-h` now classify
  as a clean help launch and dispatch through `runtime::run_help()` and
  `RuntimeCommand::Help`, with the existing help text owned by
  `runtime::help_text()`. Valid clean live args remain clean-runtime launches,
  while malformed live options, unsupported flags, and historical command paths
  still delegate to the accepted adapter. Validation passed with formatting;
  focused platform, runtime, and public API tests; `cargo test --all-targets`;
  clippy with warnings denied; `make fidelity`; clean help output;
  markdownlint; and `git diff --check`. `make fidelity` matched 10 trace
  fixtures covering 15452 frames and reported new Rust line coverage `23/23`
  non-baselined added executable lines.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778769990105679`

### DC-90: Clean CLI Error Surface

Status: `complete`

Goal: move malformed clean live option handling into the clean platform/runtime
CLI surface without taking ownership of historical ROM and fidelity commands
before their clean replacements exist.

Scope:

- Add typed clean CLI diagnostics for recognized clean live options with missing
  or invalid values, including `--input-profile` and `--cmos-path`.
- Keep default live play, valid live options, live smoke, and help owned by the
  clean platform/runtime path.
- Preserve accepted-adapter delegation for historical ROM/fidelity commands and
  any unsupported compatibility paths not yet represented in clean code.
- Update source guards and tests so accepted delegation is reserved for legacy
  ownership, not for clean option parse failures.

Acceptance criteria:

- Missing or invalid values for recognized clean live options return stable
  clean CLI errors without invoking accepted CLI delegation.
- Historical ROM/fidelity commands still run through the accepted adapter.
- Unknown compatibility paths remain delegated until the clean command inventory
  explicitly takes ownership.
- CLI exit behavior, diagnostics, and help output are covered by focused tests.

Validation:

```sh
cargo fmt --check
cargo test --lib platform::tests::
cargo test --lib runtime::tests::
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_quarantined_adapters
cargo test --lib public_api_tests::clean_module_sources_keep_legacy_access_quarantined
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --input-profile
cargo run -- --input-profile invalid
cargo run -- --cmos-path
cargo run -- --help
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 18:36:44 BST` Started `DC-90`: began moving malformed
  recognized clean live options onto the clean CLI error surface while keeping
  historical ROM/fidelity commands and unsupported compatibility paths
  delegated to the accepted adapter. Initial targets are missing or invalid
  `--input-profile` values and missing `--cmos-path` values.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778780203089619`
- `2026-05-14 19:19:40 BST` Completed `DC-90`: malformed recognized clean
  live options now stop in clean platform code with typed `CleanCliError`
  diagnostics instead of falling through to the accepted adapter. Missing
  `--input-profile`, unknown `--input-profile`, and missing `--cmos-path`
  keep the previous user-facing messages and exit through the binary error
  path, while historical ROM/fidelity commands and unsupported compatibility
  paths still delegate to the accepted adapter. Validation passed with
  formatting; focused platform, runtime, and public API tests;
  `cargo test --all-targets`; clippy with warnings denied; `make fidelity`;
  clean CLI error probes; help output; markdownlint; and `git diff --check`.
  `make fidelity` matched 10 trace fixtures covering 15452 frames and reported
  new Rust line coverage `14/14` non-baselined added executable lines.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778782792884799`

### DC-91: Clean CLI Legacy Command Inventory

Status: `complete`

Goal: make the remaining accepted-adapter CLI delegation explicit by
inventorying historical ROM/fidelity commands separately from unsupported
compatibility paths.

Scope:

- Add clean classifier structure that distinguishes known historical commands
  from truly unsupported compatibility paths before either branch enters the
  accepted adapter.
- Preserve current runtime behavior for ROM report, ROM verification, fidelity
  trace, fixture, and reference-trace commands.
- Keep default live play, valid live options, live smoke, help, and clean live
  option errors owned by the clean platform/runtime path.
- Update public API guards and focused tests so future cycles can retire
  historical commands one command family at a time.

Acceptance criteria:

- Known historical ROM/fidelity commands are classified explicitly before
  accepted-adapter dispatch.
- Unknown unsupported paths remain delegated only through a separate
  compatibility branch.
- Clean CLI behavior from DC-88 through DC-90 remains unchanged.
- The command inventory is test-covered and source-visible in public API
  guards.

Validation:

```sh
cargo fmt --check
cargo test --lib platform::tests::
cargo test --lib runtime::tests::
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_quarantined_adapters
cargo test --lib public_api_tests::clean_module_sources_keep_legacy_access_quarantined
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --rom-report
cargo run -- --fidelity-list-scenarios
cargo run -- --input-profile invalid
cargo run -- --help
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 19:22:02 BST` Started `DC-91`: separating the known
  historical ROM/fidelity command inventory from unsupported compatibility
  arguments before accepted-adapter dispatch. Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778782922103819`
- `2026-05-14 20:01:24 BST` Completed `DC-91`: added explicit
  `HistoricalCliCommand` classification for ROM report, ROM verification,
  fidelity trace, fixture, scenario, and reference-trace commands; added a
  separate `CompatibilityFallback` path for unsupported and removed
  compatibility arguments; preserved clean live/help/error ownership; and
  added focused platform/API tests plus entrypoint coverage for the
  historical adapter dispatch. Validation passed with `cargo fmt --check`,
  `cargo test --lib platform::tests::`,
  `cargo test --lib runtime::tests::`,
  `cargo test --lib public_api_tests::clean_runtime_and_oracle_use_quarantined_adapters`,
  `cargo test --lib public_api_tests::clean_module_sources_keep_legacy_access_quarantined`,
  `make fidelity` including 10 trace fixtures, 15452 frames, and 29/29
  non-baselined added executable Rust lines covered, plus CLI probes for
  `--rom-report`, `--fidelity-list-scenarios`, `--input-profile invalid`,
  and `--help`. Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778785334701229`

### DC-92: Clean ROM Report Command Ownership

Status: `complete`

Goal: retire the first historical CLI command family from the accepted adapter
by moving ROM report listing into clean runtime ownership while preserving
the current command output and optional verification path.

Scope:

- Add a clean runtime command/config path for `--rom-report` with no path and
  with one optional ROM directory path.
- Keep `--verify-roms` in the historical command inventory for now.
- Preserve current ROM report text, optional path validation, and clean
  live/help/error behavior.
- Update the historical inventory so `--rom-report` no longer enters the
  accepted adapter, while the other ROM/fidelity commands still do.
- Add focused platform/runtime tests and public API guards for the new clean
  ROM report path.

Acceptance criteria:

- `cargo run -- --rom-report` is served by clean runtime code and matches the
  current report text.
- `cargo run -- --rom-report /path/to/roms` preserves current scan behavior
  and error reporting.
- Unsupported extra `--rom-report` arguments remain rejected with the current
  message.
- Historical inventory still explicitly delegates `--verify-roms` and the
  fidelity commands.

Validation:

```sh
cargo fmt --check
cargo test --lib platform::tests::
cargo test --lib runtime::tests::
cargo test --lib rom_report::tests::
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_quarantined_adapters
cargo test --lib public_api_tests::clean_module_sources_keep_legacy_access_quarantined
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --rom-report
cargo run -- --rom-report <empty-temp-dir>
cargo run -- --rom-report --verify-roms
cargo run -- --verify-roms
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 20:03:44 BST` Started `DC-92`: moving `--rom-report`
  listing and optional ROM directory scanning from the historical
  accepted-adapter CLI branch into clean runtime ownership. Slack start
  update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778785424298289`
- `2026-05-14 20:31:06 BST` Completed `DC-92`: `--rom-report` now
  classifies as a clean CLI command, dispatches through `RuntimeCommand`, and
  uses a private `rom_report` facade for current listing/scanning text while
  `--verify-roms` and fidelity commands remain in the historical inventory.
  Public API guards now assert that `runtime.rs` does not reach into legacy
  ROM internals directly, and focused tests cover the clean classifier,
  runtime dispatch, report formatting, malformed arguments, and quarantine
  boundary. Validation passed with the full command set above, including
  `make fidelity` with 10 trace fixtures, 15452 frames, and 38/38
  non-baselined added executable Rust lines covered. Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778787066498619`

### DC-93: Clean Fidelity Scenario Listing Ownership

Status: `complete`

Goal: retire the next low-risk historical CLI command by moving the
read-only fidelity scenario listing command into clean runtime ownership while
leaving trace generation and trace checking in the historical adapter.

Scope:

- Add a clean runtime command/config path for `--fidelity-list-scenarios`.
- Preserve current scenario listing text and ordering.
- Keep fidelity trace generation, trace checking, fixture checking, reference
  checking, and scenario input writing in the historical command inventory.
- Put scenario listing text behind a small clean facade owned by `fidelity` or
  runtime-support code rather than calling the accepted adapter.
- Add focused platform/runtime/API tests for command classification,
  dispatch, and output contract.

Acceptance criteria:

- `cargo run -- --fidelity-list-scenarios` is served by clean runtime code and
  matches the current output.
- Historical inventory still explicitly delegates trace/check/write commands.
- Public API guards make the new ownership boundary visible.

Validation:

```sh
cargo fmt --check
cargo test --lib platform::tests::
cargo test --lib runtime::tests::
cargo test --lib scenario_listing::tests::
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_quarantined_adapters
cargo test --lib public_api_tests::clean_module_sources_keep_legacy_access_quarantined
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --fidelity-list-scenarios
cargo run -- --fidelity-list-scenarios extra
cargo run -- --mute --fidelity-list-scenarios
cargo run -- --fidelity-trace 1
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 20:32:55 BST` Started `DC-93`: moving the read-only
  `--fidelity-list-scenarios` command into clean runtime ownership while
  leaving trace generation, trace checking, fixture checking, reference
  checking, and scenario input writing in the historical inventory. Slack
  start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778787188057879`
- `2026-05-14 20:55:19 BST` Completed `DC-93`: `--fidelity-list-scenarios`
  now classifies as a clean CLI command, dispatches through
  `RuntimeCommand::FidelityScenarioList`, and uses a private
  `scenario_listing` facade that preserves the current scenario manifest
  text and ordering. Public API guards now expose the ownership boundary and
  keep the new facade as the only clean source allowed to read the legacy
  trace scenario manifest. Validation passed with the full command set above,
  including `make fidelity` with 10 trace fixtures, 15452 frames, and 17/17
  non-baselined added executable Rust lines covered. Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778789519436699`

### DC-94: Clean Fidelity Scenario Input Writer Ownership

Status: `complete`

Goal: move the read/write scenario input expansion helper command into clean
runtime ownership while keeping trace generation and trace checking in the
historical adapter.

Scope:

- Add a clean runtime command/config path for
  `--fidelity-write-scenario-inputs <output-dir>`.
- Preserve current directory creation, file naming, expanded input text, and
  completion message.
- Keep `--fidelity-trace`, `--fidelity-trace-inputs`,
  `--fidelity-trace-inputs-file`, `--fidelity-check-trace`,
  `--fidelity-check-trace-dir`, and
  `--fidelity-check-reference-trace-dir` in the historical command inventory.
- Reuse the scenario manifest facade added in DC-93 rather than adding a new
  legacy access path.
- Add focused platform/runtime/API tests for command classification,
  malformed arguments, dispatch, and output/file contract.

Acceptance criteria:

- `cargo run -- --fidelity-write-scenario-inputs <temp-dir>` is served by
  clean runtime code and writes the same input scripts as before.
- Historical inventory still explicitly delegates trace generation and trace
  checking commands.
- Public API guards keep the scenario manifest/input facade private and
  quarantined.

Validation:

```sh
cargo fmt --check
cargo test --lib platform::tests::
cargo test --lib runtime::tests::
cargo test --lib fidelity_scenarios::tests::
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_quarantined_adapters
cargo test --lib public_api_tests::clean_module_sources_keep_legacy_access_quarantined
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --fidelity-write-scenario-inputs <empty-temp-dir>
cargo run -- --fidelity-write-scenario-inputs
cargo run -- --fidelity-write-scenario-inputs <empty-temp-dir> extra
cargo run -- --mute --fidelity-write-scenario-inputs <empty-temp-dir>
cargo run -- --fidelity-trace-inputs 'coin,start_one'
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 21:14:36 BST` Started `DC-94`: moving
  `--fidelity-write-scenario-inputs <output-dir>` into clean runtime
  ownership while preserving directory creation, `*.inputs.txt` file naming,
  expanded input text, and completion output. Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778789688179609`
- `2026-05-14 21:39:12 BST` Completed `DC-94`: scenario listing and scenario
  input writing now share a private `fidelity_scenarios` clean facade.
  `--fidelity-write-scenario-inputs <output-dir>` classifies as a clean CLI
  command, dispatches through
  `RuntimeCommand::FidelityScenarioInputWriter`, and writes the same 12
  expanded scenario input scripts as the historical helper. Trace generation
  and trace checking commands remain in the historical inventory, and public
  API guards expose the new quarantine boundary. Validation passed with the
  full command set above, including `make fidelity` with 10 trace fixtures,
  15452 frames, and 26/26 non-baselined added executable Rust lines covered.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778791204633039`

### DC-95: Clean Verify ROM Command Ownership

Status: `complete`

Goal: retire the remaining ROM-oriented historical CLI command by moving
`--verify-roms /path/to/roms` into clean runtime ownership alongside
`--rom-report`.

Scope:

- Add a clean runtime command/config path for `--verify-roms <rom-dir>`.
- Preserve current required path, extra-argument, live-option conflict,
  failure report, and successful verification output behavior.
- Reuse the existing private ROM command facade instead of adding a new clean
  legacy access path.
- Keep fidelity trace generation and trace checking commands in the historical
  command inventory.
- Add focused platform/runtime/API tests for command classification,
  malformed arguments, dispatch, and output contract.

Acceptance criteria:

- `cargo run -- --verify-roms <rom-dir>` is served by clean runtime code and
  preserves the current verification output.
- Missing paths, extra args, and mixed live options preserve current messages.
- Historical inventory still explicitly delegates fidelity trace generation
  and trace checking commands.
- Public API guards make the new ROM verification ownership boundary visible.

Validation:

```sh
cargo fmt --check
cargo test --lib platform::tests::
cargo test --lib runtime::tests::
cargo test --lib rom_report::tests::
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_quarantined_adapters
cargo test --lib public_api_tests::clean_module_sources_keep_legacy_access_quarantined
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --verify-roms
cargo run -- --verify-roms <rom-dir> extra
cargo run -- --mute --verify-roms <rom-dir>
cargo run -- --verify-roms <empty-rom-dir>
cargo run -- --verify-roms assets/roms/defender
cargo run -- --fidelity-trace 1
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 21:47:30 BST` Started `DC-95`: moving
  `--verify-roms /path/to/roms` into clean platform/runtime ownership while
  preserving argument validation, failure report, and success output behavior.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778791679350619`
- `2026-05-14 22:30:47 BST` Completed `DC-95`: `--verify-roms
  /path/to/roms` now classifies as a clean platform command, dispatches
  through `RuntimeCommand::VerifyRoms`, and reuses the private `rom_report`
  facade for verified-set loading, mapped-image construction, incomplete-set
  reports, and success output. The historical command inventory now only owns
  fidelity trace generation and trace checking commands. Validation passed
  with the focused platform/runtime/rom_report/public API tests, `cargo test
  --all-targets`, `cargo clippy --all-targets -- -D warnings`, `make
  fidelity` with 10 trace fixtures, 15452 frames, and 47/47 non-baselined
  added executable Rust lines covered, CLI probes for missing, extra, mixed
  live-option, incomplete, complete verify fixture, and historical
  `--fidelity-trace 1` behavior, `cargo run -- --live-smoke` with 240
  rendered frames, markdownlint, `cargo fmt --check`, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778794248188319`

### DC-96: Clean Fidelity Trace Command Ownership

Status: `complete`

Goal: move `--fidelity-trace <frames>` into clean platform/runtime ownership
while preserving the current trace output and argument behavior.

Scope:

- Add a clean command/config path for `--fidelity-trace` with the current
  default one-frame trace.
- Preserve live-option conflict, invalid frame-count, zero frame-count, and
  extra-argument error messages.
- Add a private fidelity trace facade that keeps legacy trace generation
  quarantined behind clean runtime ownership.
- Leave trace-input and trace-check commands in the historical inventory for
  later slices.
- Add focused platform/runtime/API tests for classification, malformed
  arguments, dispatch, and output contract.

Acceptance criteria:

- `cargo run -- --fidelity-trace` and `cargo run -- --fidelity-trace 2` are
  served by clean runtime code and preserve the current trace text.
- Malformed `--fidelity-trace` invocations preserve current messages.
- Historical inventory still explicitly delegates trace-input and trace-check
  commands.
- Public API guards make the new trace generation boundary visible.

Validation:

```sh
cargo fmt --check
cargo test --lib platform::tests::
cargo test --lib runtime::tests::
cargo test --lib fidelity_traces::tests::
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_quarantined_adapters
cargo test --lib public_api_tests::clean_module_sources_keep_legacy_access_quarantined
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --fidelity-trace
cargo run -- --fidelity-trace 2
cargo run -- --fidelity-trace 0
cargo run -- --fidelity-trace wat
cargo run -- --fidelity-trace 1 extra
cargo run -- --mute --fidelity-trace 1
cargo run -- --fidelity-trace-inputs none
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 22:32:30 BST` Started `DC-96`: moving
  `--fidelity-trace <frames>` into clean platform/runtime ownership while
  preserving default frame count, malformed-count errors, trace text, and
  historical delegation for trace-input and trace-check commands.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778794361787019`
- `2026-05-14 22:55:51 BST` Completed `DC-96`: `--fidelity-trace
  <frames>` now classifies as a clean platform command, dispatches through
  `RuntimeCommand::FidelityTrace`, and generates the current idle trace through
  a private `fidelity_traces` facade. The facade keeps legacy trace generation
  quarantined while preserving default one-frame output, counted output,
  malformed-count messages, zero-count rejection, extra-argument rejection, and
  live-option conflict behavior. Trace-input and trace-check commands remain
  in the historical inventory for later slices. Validation passed with the
  focused platform/runtime/fidelity trace/public API tests, `cargo test
  --all-targets`, `cargo clippy --all-targets -- -D warnings`, `make
  fidelity` with 10 trace fixtures, 15452 frames, and 40/40 non-baselined
  added executable Rust lines covered, CLI probes for successful and malformed
  `--fidelity-trace` invocations plus historical `--fidelity-trace-inputs
  none`, `cargo run -- --live-smoke` with 240 rendered frames, markdownlint,
  `cargo fmt --check`, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778795765516439`

### DC-97: Clean Fidelity Trace Input Command Ownership

Status: `complete`

Goal: move `--fidelity-trace-inputs <script>` and
`--fidelity-trace-inputs-file <path>` into clean platform/runtime ownership
while preserving current trace output and argument behavior.

Scope:

- Add clean command/config paths for inline trace input scripts and trace input
  script files.
- Preserve live-option conflict, missing argument, extra argument, invalid
  script, and file-read failure behavior.
- Extend the private fidelity trace facade so legacy trace generation remains
  quarantined behind clean runtime ownership.
- Leave trace-check commands in the historical inventory for later slices.
- Add focused platform/runtime/API tests for classification, malformed
  arguments, dispatch, file reads, and output contract.

Acceptance criteria:

- `cargo run -- --fidelity-trace-inputs none` and
  `cargo run -- --fidelity-trace-inputs-file <path>` are served by clean
  runtime code and preserve current trace text.
- Malformed trace-input invocations preserve current messages.
- Historical inventory still explicitly delegates trace-check commands.
- Public API guards make the trace input boundary visible.

Validation:

```sh
cargo fmt --check
cargo test --lib platform::tests::
cargo test --lib runtime::tests::
cargo test --lib fidelity_traces::tests::
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_quarantined_adapters
cargo test --lib public_api_tests::clean_module_sources_keep_legacy_access_quarantined
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --fidelity-trace-inputs none
cargo run -- --fidelity-trace-inputs 'coin,start_one;fire,thrust;none'
cargo run -- --fidelity-trace-inputs
cargo run -- --fidelity-trace-inputs none extra
cargo run -- --mute --fidelity-trace-inputs none
cargo run -- --fidelity-trace-inputs-file <input-script-path>
cargo run -- --fidelity-trace-inputs-file
cargo run -- --fidelity-trace-inputs-file <input-script-path> extra
cargo run -- --mute --fidelity-trace-inputs-file <input-script-path>
cargo run -- --fidelity-check-trace-dir docs/fidelity/fixtures/local/rust-current
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 22:57:49 BST` Started `DC-97`: moving
  `--fidelity-trace-inputs <script>` and
  `--fidelity-trace-inputs-file <path>` into clean platform/runtime ownership
  while preserving argument validation, file-read behavior, trace output, and
  historical delegation for trace-check commands.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778795883407799`
- `2026-05-14 23:24:23 BST` Completed `DC-97`: clean platform/runtime code now
  owns inline and file-based trace input commands, the private fidelity trace
  facade owns inline and file trace text generation, trace-input commands are
  removed from the historical command inventory, and public API guards cover
  the new boundary. Validation passed: focused platform/runtime/fidelity trace
  and public API tests; `cargo test --all-targets`; `cargo clippy --all-targets
  -- -D warnings`; `make fidelity` with 10 trace fixtures, 15452 frames, and
  68/68 added executable Rust lines covered; CLI probes for inline/file
  success, malformed trace-input invocations, file-read failures, live-option
  conflicts, invalid scripts, and historical trace-check delegation;
  `cargo run -- --live-smoke` with 239 rendered frames; `markdownlint`;
  `cargo fmt --check`; and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778797461215109`

### DC-98: Clean Single Trace Check Command Ownership

Status: `complete`

Goal: move `--fidelity-check-trace <inputs> <expected>` into clean
platform/runtime ownership while preserving current trace comparison behavior.

Scope:

- Add a clean command/config path for checking one trace input script against
  one expected TSV trace.
- Preserve live-option conflict, missing argument, extra argument, file-read,
  and trace mismatch behavior.
- Extend the private fidelity trace facade so single trace comparisons remain
  quarantined behind clean runtime ownership.
- Leave fixture directory and reference fixture directory checks in the
  historical inventory for later slices.
- Add focused platform/runtime/API tests for classification, malformed
  arguments, dispatch, file reads, trace mismatch, and output contract.

Acceptance criteria:

- `cargo run -- --fidelity-check-trace <inputs> <expected>` is served by clean
  runtime code and preserves current match/mismatch text.
- Malformed single trace-check invocations preserve current messages.
- Historical inventory still explicitly delegates fixture-directory and
  reference-fixture-directory checks.
- Public API guards make the single trace-check boundary visible.

Validation:

```sh
cargo fmt --check
cargo test --lib platform::tests::
cargo test --lib runtime::tests::
cargo test --lib fidelity_traces::tests::
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_quarantined_adapters
cargo test --lib public_api_tests::clean_module_sources_keep_legacy_access_quarantined
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --fidelity-check-trace <input-script-path> <expected-trace-path>
cargo run -- --fidelity-check-trace
cargo run -- --fidelity-check-trace <input-script-path>
cargo run -- --fidelity-check-trace <input-script-path> <expected-trace-path> extra
cargo run -- --mute --fidelity-check-trace <input-script-path> <expected-trace-path>
cargo run -- --fidelity-check-trace-dir docs/fidelity/fixtures/local/rust-current
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 23:26:35 BST` Started `DC-98`: moving
  `--fidelity-check-trace <inputs> <expected>` into clean platform/runtime
  ownership while preserving argument validation, file-read behavior, trace
  comparison output, and historical delegation for fixture-directory checks.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778797606858679`
- `2026-05-14 23:50:33 BST` Completed `DC-98`: clean platform/runtime code now
  owns single trace comparisons through `RuntimeCommand::FidelityTraceCheck`,
  the private fidelity trace facade compares generated trace text against
  expected TSV files, and single trace checks are removed from the historical
  command inventory while fixture-directory checks remain delegated for later
  slices. Validation passed: focused platform/runtime/fidelity trace and
  public API tests; `cargo test --all-targets`; `cargo clippy --all-targets
  -- -D warnings`; `make fidelity` with 10 trace fixtures, 15452 frames, and
  54/54 added executable Rust lines covered; CLI probes for trace-check match,
  mismatch, malformed arguments, file-read failures, live-option conflict, and
  historical trace fixture directory delegation; `cargo run -- --live-smoke`
  with 239 rendered frames; `markdownlint`; `cargo fmt --check`; and
  `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778799033889449`

### DC-99: Clean Trace Fixture Directory Command Ownership

Status: `complete`

Goal: move `--fidelity-check-trace-dir <fixtures>` into clean platform/runtime
ownership while preserving current Rust-current fixture checking behavior.

Scope:

- Add a clean command/config path for checking a directory of paired
  `*.inputs.txt` and `*.expected.tsv` trace fixtures.
- Preserve live-option conflict, missing argument, extra argument, missing
  directory, empty directory, non-directory, missing pair, fixture ordering,
  mismatch, and summary text behavior.
- Extend the private fidelity trace facade so trace fixture discovery and
  checking remain quarantined behind clean runtime ownership.
- Leave reference fixture directory checks in the historical inventory for a
  later slice.
- Add focused platform/runtime/API tests for classification, malformed
  arguments, dispatch, fixture discovery, ordering, missing pairs, and output
  contract.

Acceptance criteria:

- `cargo run -- --fidelity-check-trace-dir <fixtures>` is served by clean
  runtime code and preserves current matched/skipped/error text.
- Malformed trace fixture directory invocations preserve current messages.
- Historical inventory still explicitly delegates reference fixture directory
  checks.
- Public API guards make the trace fixture directory boundary visible.

Validation:

```sh
cargo fmt --check
cargo test --lib platform::tests::
cargo test --lib runtime::tests::
cargo test --lib fidelity_traces::tests::
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_quarantined_adapters
cargo test --lib public_api_tests::clean_module_sources_keep_legacy_access_quarantined
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --fidelity-check-trace-dir \
  docs/fidelity/fixtures/local/rust-current
cargo run -- --fidelity-check-trace-dir
cargo run -- --fidelity-check-trace-dir \
  docs/fidelity/fixtures/local/rust-current extra
cargo run -- --mute --fidelity-check-trace-dir \
  docs/fidelity/fixtures/local/rust-current
cargo run -- --fidelity-check-reference-trace-dir \
  docs/fidelity/fixtures/local/reference
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-14 23:52:19 BST` Started `DC-99`: moving
  `--fidelity-check-trace-dir <fixtures>` into clean platform/runtime
  ownership while preserving argument validation, fixture discovery, skipped
  directory behavior, pair validation, manifest ordering, trace comparison, and
  historical delegation for reference fixture directory checks.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778799152129099`
- `2026-05-15 00:38:25 BST` Completed `DC-99`: clean platform/runtime
  now owns `--fidelity-check-trace-dir <fixtures>` through
  `RuntimeCommand::FidelityTraceFixtureDirectory`, with paired fixture
  discovery and trace comparison quarantined in the private fidelity trace
  facade. Historical delegation remains only for
  `--fidelity-check-reference-trace-dir`.
  Validation passed: focused platform/runtime/fidelity trace/public API tests,
  `cargo test --all-targets`, `cargo clippy --all-targets -- -D warnings`,
  `make fidelity`, CLI probes for matched/skipped/malformed/non-directory/
  missing-pair/mismatch cases, historical reference delegation, and
  `cargo run -- --live-smoke`.
  `make fidelity` matched `10` trace fixture(s), `15452` frame(s), and covered
  `146/146` non-baselined added executable Rust line(s). Live smoke rendered
  `239` frames.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778801904026079`

### DC-100: Clean Reference Trace Fixture Directory Command Ownership

Status: `complete`

Goal: move `--fidelity-check-reference-trace-dir <fixtures>` into clean
platform/runtime ownership while preserving local MAME reference fixture
validation behavior.

Scope:

- Add a clean command/config path for checking complete Phase 1 local reference
  trace fixture directories.
- Preserve live-option conflict, missing argument, extra argument,
  non-directory, missing fixture, input-program drift, header drift, frame-count
  drift, required-cell, required-sound-command, required-event, and summary text
  behavior.
- Extend the private fidelity trace facade so reference fixture validation
  remains quarantined behind clean runtime ownership.
- Remove the reference fixture directory command from the historical inventory
  so trace-check CLI paths are no longer delegated to the historical app.
- Add focused platform/runtime/API tests for classification, malformed
  arguments, dispatch, reference fixture validation, evidence deadlines, and
  output contract.

Acceptance criteria:

- `cargo run -- --fidelity-check-reference-trace-dir <fixtures>` is served by
  clean runtime code and preserves current validation and summary text.
- Malformed reference fixture directory invocations preserve current messages.
- Historical command inventory no longer owns trace-check CLI paths.
- Public API guards make the reference fixture directory boundary visible.

Validation:

```sh
cargo fmt --check
cargo test --lib platform::tests::
cargo test --lib runtime::tests::
cargo test --lib fidelity_traces::tests::
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_quarantined_adapters
cargo test --lib public_api_tests::clean_module_sources_keep_legacy_access_quarantined
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --fidelity-check-reference-trace-dir \
  docs/fidelity/fixtures/local/reference
cargo run -- --fidelity-check-reference-trace-dir
cargo run -- --fidelity-check-reference-trace-dir \
  docs/fidelity/fixtures/local/reference extra
cargo run -- --mute --fidelity-check-reference-trace-dir \
  docs/fidelity/fixtures/local/reference
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 00:41:20 BST` Started `DC-100`: moving
  `--fidelity-check-reference-trace-dir <fixtures>` into clean
  platform/runtime ownership while preserving local reference fixture
  validation, malformed argument errors, required evidence checks, and summary
  text.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778802025722099`
- `2026-05-15 01:25:52 BST` Completed `DC-100`: clean platform/runtime
  now owns `--fidelity-check-reference-trace-dir <fixtures>` through
  `RuntimeCommand::FidelityReferenceTraceFixtureDirectory`, with local
  reference fixture validation quarantined in the private fidelity trace
  facade. The clean CLI no longer has a historical command inventory for
  trace-check paths.
  Validation passed: focused platform/runtime/fidelity trace/public API tests,
  `cargo test --all-targets`, `cargo clippy --all-targets -- -D warnings`,
  `make fidelity`, CLI probes for reference fixture match, malformed args,
  live conflict, missing directory, non-directory, missing expected trace,
  input-program drift, and header drift, plus `cargo run -- --live-smoke`.
  `make fidelity` matched `10` Rust-current trace fixture(s), `15452`
  frame(s), and covered `232/232` non-baselined added executable Rust line(s).
  Live smoke rendered `239` frames.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778804751380139`

### DC-101: Clean Unsupported CLI Argument Ownership

Status: `complete`

Goal: remove the accepted CLI fallback from clean platform/runtime dispatch so
unsupported and removed CLI arguments are reported by clean code directly.

Scope:

- Replace the `CompatibilityFallback` classification with clean
  `UnknownArgument` and removed renderer-selection errors.
- Preserve user-facing error text for unknown arguments and removed
  `--renderer` / `--presentation` selection.
- Remove `RuntimeCommand::AcceptedCli`, `RuntimeHost::run_cli`, and the
  `accepted_behavior::run_runtime()` fallback from clean runtime dispatch.
- Update public API guards so clean runtime no longer has an accepted CLI
  escape hatch.
- Add focused platform/runtime/API tests for unsupported arguments, removed
  renderer selection, and absence of accepted runtime fallback wiring.

Acceptance criteria:

- `cargo run -- --unknown` fails through clean platform code with the existing
  `unknown argument: --unknown` text.
- `cargo run -- --renderer wgpu` and `cargo run -- --presentation wgpu` fail
  through clean platform code with the existing `wgpu-only` text.
- Clean platform/runtime no longer calls `crate::runtime::run_cli()` or
  `crate::accepted_behavior::run_runtime()`.
- Public API guards make the removal visible.

Validation:

```sh
cargo fmt --check
cargo test --lib platform::tests::
cargo test --lib runtime::tests::
cargo test --lib public_api_tests::clean_runtime_and_oracle_use_quarantined_adapters
cargo test --lib public_api_tests::clean_module_sources_keep_legacy_access_quarantined
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --unknown
cargo run -- --live-smoke --unknown
cargo run -- --renderer wgpu
cargo run -- --presentation wgpu
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 01:28:41 BST` Started `DC-101`: moving unsupported and
  removed CLI argument handling fully into clean platform ownership, removing
  the accepted CLI fallback from clean runtime dispatch, and preserving
  user-facing unknown-argument and `wgpu-only` renderer-selection errors.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778804917918359`
- `2026-05-15 01:53:21 BST` Completed `DC-101`: removed the clean runtime
  accepted CLI fallback, made unknown and removed renderer-selection arguments
  clean platform errors, deleted the unused accepted runtime helper, and
  removed redundant `Clean` prefixes from internal CLI classification variants.
  Validation passed with focused platform/runtime/API tests,
  `cargo test --all-targets`, `cargo clippy --all-targets -- -D warnings`,
  `make fidelity` (10 local fixtures, 15452 frames, 91/91 new executable Rust
  lines), clean CLI failure probes, `cargo run -- --live-smoke` (239 rendered
  frames), `cargo fmt --check`, markdownlint, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778806424037359`

### DC-102: Clean Sprite Batch Draw Planning

Status: `complete`

Goal: make the clean renderer draw plan resolve sprite instances through the
renderer-owned texture atlas while keeping temporary raster upload separate as
fidelity evidence.

Scope:

- Add a sprite batch representation to `SceneDrawPlan` that records each
  sprite's atlas region, layer, position, size, and tint.
- Resolve scene sprites through `NativeRendererResources::atlas` during
  `NativeSceneRenderer::prepare`.
- Report missing atlas entries as explicit draw-plan evidence instead of
  silently treating every sprite as drawable.
- Preserve the temporary raster upload path as separate from sprite batches.
- Add focused renderer tests for atlas-backed batching, missing sprite regions,
  and mixed raster/sprite plans.
- Update README/SPEC module text for atlas-backed sprite batches and the
  current clean runtime bridge wording.

Acceptance criteria:

- Sprite-first clean scenes produce atlas-backed sprite batches in the native
  renderer plan.
- Missing sprite atlas regions are counted and do not request sprite
  pipelines.
- Raster upload remains an independent temporary fidelity payload.

Validation:

```sh
cargo fmt --check
cargo test --lib renderer::tests::
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 01:56:13 BST` Started `DC-102`: adding atlas-backed sprite
  batch draw-plan evidence to the clean renderer while preserving temporary
  raster upload as a separate fidelity path.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778806586742169`
- `2026-05-15 02:18:57 BST` Completed `DC-102`: added atlas-backed sprite
  draw batches to `SceneDrawPlan`, recorded missing sprite atlas regions,
  kept temporary raster upload separate, added default terrain/star atlas
  entries, exported the new sprite-batch plan types, and refreshed README/SPEC
  module wording. Validation passed with focused renderer tests, a public API
  guard, `cargo test --all-targets`, clippy with warnings denied,
  `make fidelity` (10 local fixtures, 15452 frames, 58/58 new executable Rust
  lines), `cargo run -- --live-smoke` (239 rendered frames),
  `cargo fmt --check`, markdownlint, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778807950561589`

### DC-103: Clean Renderer Viewport Draw Planning

Status: `complete`

Goal: make the clean renderer draw plan describe how native scene coordinates
fit into a target `wgpu` surface without falling back to legacy video geometry.

Scope:

- Add an aspect-preserving viewport layout to the clean renderer plan.
- Keep `NativeSceneRenderer::prepare` as the scene-sized convenience path and
  add a target-surface preparation path for real `wgpu` surfaces.
- Handle empty scene or target surfaces without division-by-zero or misleading
  scale values.
- Use the same viewport calculation for sprite-batch plans and temporary raster
  upload plans.
- Add focused renderer tests for centered scaling, empty surfaces, sprite
  scenes, and raster scenes.
- Update README/SPEC module text for viewport-aware draw planning.

Acceptance criteria:

- Scene draw plans record the native scene surface, target surface, centered
  viewport rectangle, and scale.
- Empty scene or target surfaces produce an empty viewport plan.
- Sprite-only and raster-backed scenes use the same viewport calculation.

Validation:

```sh
cargo fmt --check
cargo test --lib renderer::tests::
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 02:21:12 BST` Started `DC-103`: adding clean viewport layout
  evidence to `SceneDrawPlan` so native scene coordinates can map into a target
  `wgpu` surface without relying on legacy video geometry.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778808084056079`
- `2026-05-15 02:44:19 BST` Completed `DC-103`: added `ViewportLayout` to the
  clean renderer plan, implemented target-aware preparation through
  `NativeSceneRenderer::prepare_for_target`, kept `prepare` as the scene-sized
  convenience path, applied the same viewport fit to sprite and temporary
  raster plans, exported the viewport type, and refreshed README/SPEC wording.
  Validation passed with focused renderer tests, a public API guard,
  `cargo test --all-targets`, clippy with warnings denied, `make fidelity` (10
  local fixtures, 15452 frames, 31/31 new executable Rust lines),
  `cargo run -- --live-smoke` (239 rendered frames), `cargo fmt --check`,
  markdownlint, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778809459870549`
  Slack final validation follow-up:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778810539805979`

### DC-104: Clean Renderer GPU Pass Planning

Status: `complete`

Goal: extend the clean renderer draw plan with GPU-ready pass constants derived
from scene data, so the next native `wgpu` path can bind clear color, viewport,
and scene-coordinate projection without consulting legacy video geometry.

Scope:

- Convert clean `Color` values into normalized `wgpu::Color` clear values.
- Add a viewport command representation that maps `ViewportLayout` to the
  values used by `wgpu::RenderPass::set_viewport`.
- Add scene-to-clip projection constants for native top-left scene coordinates.
- Attach the GPU pass constants to `SceneDrawPlan`.
- Add focused renderer tests for color normalization, empty viewports,
  viewport command values, projection constants, and sprite/raster draw plans.
- Update README/SPEC module text for GPU pass planning.

Acceptance criteria:

- Clean draw plans expose GPU-ready pass constants from clean scene data only.
- Empty scene or target surfaces do not request a viewport command.
- Sprite-only and raster-backed scenes share the same GPU pass constants.

Validation:

```sh
cargo fmt --check
cargo test --lib renderer::tests::
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 03:04:11 BST` Started `DC-104`: adding GPU-ready renderer pass
  constants for clear color, viewport commands, and scene-to-clip projection
  data derived from clean scene draw plans.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778810662415339`
- `2026-05-15 03:28:11 BST` Completed `DC-104`: added GPU-ready pass constants
  to `SceneDrawPlan`, including normalized `wgpu::Color`, optional viewport
  command values, and scene-to-clip projection uniforms for native top-left
  scene coordinates; kept sprite batches and temporary raster upload separate;
  exported the new clean renderer contracts; and refreshed README/SPEC wording.
  Validation passed with focused renderer tests, a public API guard,
  `cargo test --all-targets` (1144 library tests plus binary/example tests),
  clippy with warnings denied, `make fidelity` (10 local fixtures, 15452
  frames, 33/33 new executable Rust lines), `cargo run -- --live-smoke` (239
  rendered frames), `cargo fmt --check`, markdownlint, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778812090473359`

### DC-105: Clean Sprite Instance Buffer Planning

Status: `complete`

Goal: extend clean sprite draw planning with GPU instance-buffer records derived
from renderer-owned atlas batches, so a later native `wgpu` renderer can upload
sprite instance data without consulting legacy video state.

Scope:

- Add GPU instance-buffer records for sprite draw instances.
- Preserve native scene coordinates for sprite rectangles.
- Convert atlas regions into normalized UV origin/size using the
  renderer-owned atlas surface.
- Convert tint colors into normalized RGBA instance data.
- Keep temporary raster upload separate from sprite instance buffers.
- Add focused renderer tests for UV normalization, tint normalization,
  batch-to-buffer conversion, and empty atlas handling.
- Update README/SPEC module text for sprite instance-buffer planning.

Acceptance criteria:

- Clean draw plans expose GPU instance-buffer data alongside logical sprite
  batches.
- Sprite instance-buffer records are derived only from clean scene sprites and
  renderer-owned atlas metadata.
- Raster-backed scenes do not create sprite instance buffers unless they also
  include drawable sprites.

Validation:

```sh
cargo fmt --check
cargo test --lib renderer::tests::
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 03:29:54 BST` Started `DC-105`: adding clean GPU instance-buffer
  records for sprite batches, with native scene rectangles, normalized atlas
  UVs, and normalized tint data.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778812209675969`
- `2026-05-15 03:53:46 BST` Completed `DC-105`: added public
  `SpriteInstanceBuffer` and `SpriteInstanceBufferRecord` draw-plan data
  derived from renderer-owned atlas batches, preserved scene-space sprite
  rectangles, normalized atlas UVs and tint colors for GPU upload, skipped
  instance buffers for empty atlases or raster-only scenes, and documented the
  clean renderer contract in README/SPEC. Validation passed with focused
  renderer tests, the public API guard, `cargo test --all-targets` (1146
  library tests plus binary/example tests), clippy with warnings denied,
  `make fidelity` (10 local fixtures, 15452 frames, 38/38 new executable Rust
  lines), `cargo run -- --live-smoke` (239 rendered frames), `cargo fmt
  --check`, markdownlint, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778813618499279`

### DC-106: Clean Sprite Instance GPU ABI

Status: `complete`

Goal: make clean sprite instance-buffer records directly usable as a stable
`wgpu` instance vertex buffer, so native rendering can upload and bind sprite
instance data without ad hoc conversion at the presenter boundary.

Scope:

- Give `SpriteInstanceBufferRecord` a stable GPU byte layout.
- Expose byte stride and `wgpu` vertex attributes for scene origin, scene size,
  atlas UV origin, atlas UV size, and tint.
- Expose upload-ready bytes from `SpriteInstanceBuffer`.
- Keep logical sprite batches and temporary raster upload separate from the GPU
  instance ABI.
- Add focused renderer tests for byte layout, vertex attributes, and upload
  bytes.
- Update README/SPEC module text for the sprite instance GPU ABI.

Acceptance criteria:

- Sprite instance records can be safely cast to upload bytes.
- The renderer owns the instance-buffer vertex layout used by future `wgpu`
  sprite pipelines.
- Raster-only scenes still do not expose sprite instance upload bytes.

Validation:

```sh
cargo fmt --check
cargo test --lib renderer::tests::
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 03:56:02 BST` Started `DC-106`: adding a stable GPU ABI for
  clean sprite instance buffers, including byte layout, `wgpu` vertex
  attributes, upload bytes, focused renderer tests, and docs.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778813773647089`
- `2026-05-15 04:17:47 BST` Completed `DC-106`: made
  `SpriteInstanceBufferRecord` a stable `repr(C)` bytemuck-safe GPU record,
  exposed its `wgpu` instance vertex-buffer layout, added upload-ready bytes
  for `SpriteInstanceBuffer`, kept raster-only scenes outside the sprite upload
  path, and documented the clean sprite instance GPU ABI in README/SPEC.
  Validation passed with focused renderer tests, the public API guard,
  `cargo test --all-targets` (1148 library tests plus binary/example tests),
  clippy with warnings denied, `make fidelity` (10 local fixtures, 15452
  frames, 9/9 new executable Rust lines), `cargo run -- --live-smoke` (240
  rendered frames), `cargo fmt --check`, markdownlint, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778815079514559`

### DC-107: Clean Sprite Quad GPU ABI

Status: `complete`

Goal: add renderer-owned static quad geometry for instanced sprite draws, so
future native `wgpu` sprite pipelines can bind both vertex and instance data
from clean renderer contracts.

Scope:

- Add a stable bytemuck-safe sprite quad vertex record.
- Expose the quad vertex `wgpu` layout without conflicting with the sprite
  instance attribute locations.
- Expose renderer-owned quad vertices, indices, index format, counts, and
  upload-ready bytes.
- Choose triangle winding that remains front-facing after the clean top-left
  scene projection maps into `wgpu` clip space.
- Add focused renderer tests for layout, upload bytes, index order, and winding.
- Update README/SPEC module text for sprite quad geometry ownership.

Acceptance criteria:

- Sprite quad vertex/index data is owned by the clean renderer contract.
- Sprite quad upload bytes can be used without presenter-side repacking.
- Quad geometry and instance records have distinct vertex attribute locations.

Validation:

```sh
cargo fmt --check
cargo test --lib renderer::tests::
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 04:19:53 BST` Started `DC-107`: adding clean renderer-owned
  sprite quad vertex/index geometry, GPU vertex layout, upload bytes, focused
  renderer tests, and docs.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778815203805089`
- `2026-05-15 04:44:46 BST` Completed `DC-107`: added clean renderer-owned
  `SpriteQuadVertex` and `SpriteQuadGeometry` contracts with bytemuck-safe quad
  vertices, `u16` indices, index format/counts, upload bytes, and a distinct
  `wgpu` vertex-buffer layout; locked the index winding as front-facing after
  the clean top-left scene projection; exported the new renderer types; and
  documented sprite quad geometry ownership in README/SPEC. Validation passed
  with focused renderer tests, the public API guard, `cargo test --all-targets`
  (1151 library tests plus binary/example tests), clippy with warnings denied,
  `make fidelity` (10 local fixtures, 15452 frames, 17/17 new executable Rust
  lines), `cargo run -- --live-smoke` (239 rendered frames), `cargo fmt
  --check`, markdownlint, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778816705196489`

### DC-108: Clean Sprite Draw Command Planning

Status: `complete`

Goal: derive indexed instanced sprite draw commands from clean sprite instance
buffers and renderer-owned quad geometry, so the future native `wgpu` renderer
can bind clean buffers and issue sprite draws without presenter-side planning.

Scope:

- Add draw-command metadata for sprite pipeline, layer, instance range, vertex
  count, index count, index format, and upload byte counts.
- Derive sprite draw commands from `SpriteInstanceBuffer` values after atlas
  validation and resource pipeline filtering.
- Keep logical sprite batches, GPU instance buffers, quad geometry, and
  temporary raster upload as separate renderer-owned contracts.
- Add focused renderer tests for command derivation, multiple command ranges,
  raster-only scenes, and disabled sprite pipelines.
- Update README/SPEC module text for clean sprite draw-command planning.

Acceptance criteria:

- `SceneDrawPlan` exposes sprite draw commands for each drawable instance
  buffer.
- Sprite draw commands reference only clean renderer-owned quad geometry and
  instance-buffer metadata.
- Raster-only scenes and unavailable sprite pipelines do not produce sprite
  draw commands.

Validation:

```sh
cargo fmt --check
cargo test --lib renderer::tests::
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 04:47:02 BST` Started `DC-108`: adding clean sprite
  draw-command metadata derived from renderer-owned quad geometry and sprite
  instance buffers, with focused renderer tests and docs.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778816833640799`
- `2026-05-15 05:29:22 BST` Completed `DC-108`: added `SpriteDrawCommand`
  metadata to `SceneDrawPlan`, derived commands from clean sprite instance
  buffers and renderer-owned quad/index geometry, exported the public renderer
  contract, documented sprite draw-command planning, and covered command
  metadata, cumulative instance ranges, disabled sprite pipelines, missing
  atlas or empty buffers, raster-only scenes, and raster/sprite separation.
  Validation passed with focused renderer tests (33/33), the public API guard,
  `cargo test --all-targets` (1154 library tests plus binary/example tests),
  clippy with warnings denied, `make fidelity` (10 local fixtures, 15452
  frames, 39/39 new executable Rust lines), `cargo run -- --live-smoke` (239
  rendered frames), `cargo fmt --check`, markdownlint, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778819363660199`

### DC-109: Clean Sprite Instance Upload Planning

Status: `complete`

Goal: flatten clean sprite instance-buffer records into one upload-ready
instance stream, so the indexed sprite draw commands have a concrete
renderer-owned buffer target with matching instance ranges and byte spans.

Scope:

- Add a `SpriteInstanceUpload` contract for concatenated sprite instance
  records and upload-ready bytes.
- Derive the upload stream from `SpriteInstanceBuffer` values after atlas
  validation and resource pipeline filtering.
- Wire sprite instance upload data into `SceneDrawPlan` alongside logical
  batches, per-pipeline instance buffers, draw commands, quad geometry, and
  temporary raster upload.
- Keep raster-only scenes, unavailable sprite pipelines, missing atlas regions,
  and empty atlas surfaces outside the sprite instance upload path.
- Add focused renderer tests for upload concatenation, byte lengths, command
  range alignment, raster-only scenes, and disabled sprite pipelines.
- Update README/SPEC module text for clean sprite instance upload planning.

Acceptance criteria:

- `SceneDrawPlan` exposes an upload-ready sprite instance stream whenever
  sprite draw commands exist.
- Sprite draw-command instance ranges and byte spans index into the flattened
  upload stream without presenter-side repacking.
- Raster-only scenes and unavailable sprite pipelines do not produce sprite
  instance upload data.

Validation:

```sh
cargo fmt --check
cargo test --lib renderer::tests::
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 05:31:40 BST` Started `DC-109`: adding clean sprite instance
  upload planning that flattens draw-plan records into one upload-ready stream
  matching `SpriteDrawCommand` instance ranges and byte spans.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778819514207039`
- `2026-05-15 05:53:22 BST` Completed `DC-109`: added
  `SpriteInstanceUpload` as the flattened upload-ready stream for sprite
  instance records, wired it into `SceneDrawPlan`, aligned it with
  `SpriteDrawCommand` ranges and byte spans, kept raster-only and unavailable
  sprite paths outside the upload contract, and documented the renderer
  instance-upload stream in README/SPEC. Validation passed with focused
  renderer tests (34/34), the public API guard, `cargo test --all-targets`
  (1155 library tests plus binary/example tests), clippy with warnings denied,
  `make fidelity` (10 local fixtures, 15452 frames, 17/17 new executable Rust
  lines), `cargo run -- --live-smoke` (239 rendered frames), `cargo fmt
  --check`, markdownlint, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778820801408189`

### DC-110: Clean Sprite WGPU Buffer Upload Planning

Status: `complete`

Goal: describe the `wgpu` buffer uploads needed by clean sprite draw plans, so
future native renderer code can create quad vertex, quad index, and flattened
instance buffers without presenter-side classification.

Scope:

- Add renderer-owned sprite buffer upload metadata for quad vertices, quad
  indices, and flattened sprite instances.
- Use `wgpu::BufferUsages` directly for vertex, index, and copy-destination
  buffer creation intent.
- Wire the sprite buffer upload plan into `SceneDrawPlan` whenever sprite
  instance upload data exists.
- Keep raster-only scenes, unavailable sprite pipelines, missing atlas regions,
  and empty atlas surfaces outside sprite buffer uploads.
- Add focused renderer tests for buffer roles, labels, byte lengths, `wgpu`
  usage flags, upload bytes, and absent sprite paths.
- Update README/SPEC module text for sprite `wgpu` buffer upload planning.

Acceptance criteria:

- `SceneDrawPlan` exposes `wgpu` buffer upload metadata for sprite draw plans.
- Quad vertex/index and instance upload bytes remain renderer-owned and
  presenter-ready.
- Raster-only scenes and unavailable sprite pipelines do not produce sprite
  buffer upload plans.

Validation:

```sh
cargo fmt --check
cargo test --lib renderer::tests::
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 05:55:20 BST` Started `DC-110`: adding clean sprite `wgpu`
  buffer upload metadata for quad vertex, quad index, and flattened instance
  data with `wgpu::BufferUsages`, focused renderer tests, and docs.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778820931672629`
- `2026-05-15 06:17:24 BST` Completed `DC-110`: added
  `SpriteBufferUploadPlan` metadata for quad vertices, quad indices, and
  flattened sprite instances, wired upload plans into `SceneDrawPlan` only
  when sprite instance data exists, kept raster-only/unavailable/missing-atlas
  paths outside the upload contract, exported the public renderer types, and
  documented sprite `wgpu` buffer uploads in README/SPEC. Validation passed
  with focused renderer tests (35/35), the public API guard,
  `cargo test --all-targets` (1156 library tests plus binary/example tests),
  clippy with warnings denied, `make fidelity` (10 local fixtures, 15452
  frames, 33/33 new executable Rust lines), `cargo run -- --live-smoke` (240
  rendered frames), `cargo fmt --check`, markdownlint, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778822243196599`

### DC-111: Clean Sprite Render Pass Planning

Status: `complete`

Goal: derive clean sprite render-pass binding and draw metadata from the sprite
buffer upload plan and indexed draw commands, so future native `wgpu` code can
bind renderer-owned buffers and issue indexed instanced draws without
presenter-side classification.

Scope:

- Add renderer-owned sprite vertex/index buffer binding metadata with stable
  `wgpu` vertex buffer slots.
- Derive indexed draw ranges from existing sprite draw commands.
- Wire an optional sprite render-pass plan into `SceneDrawPlan` whenever sprite
  buffer uploads and draw commands exist.
- Keep raster-only scenes, unavailable sprite pipelines, missing atlas regions,
  empty atlas surfaces, and empty command lists outside sprite render-pass
  planning.
- Add focused renderer tests for buffer slots, byte spans, index format, draw
  ranges, and absent sprite paths.
- Update README/SPEC module text for sprite render-pass planning.

Acceptance criteria:

- `SceneDrawPlan` exposes sprite render-pass metadata only for drawable clean
  sprite plans.
- The pass plan identifies quad vertex, instance vertex, and index buffer
  bindings with `wgpu`-compatible slots and byte spans.
- Indexed draw ranges match the existing sprite draw commands exactly.

Validation:

```sh
cargo fmt --check
cargo test --lib renderer::tests::
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 06:20:38 BST` Started `DC-111`: adding clean sprite render-pass
  planning on top of the sprite `wgpu` upload plan, including stable vertex
  buffer slots, index binding metadata, indexed draw ranges, focused renderer
  tests, and docs.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778822438553379`
- `2026-05-15 06:43:23 BST` Completed `DC-111`: added renderer-owned sprite
  render-pass metadata for quad vertex, instance vertex, and index bindings,
  derived indexed draw ranges from existing sprite draw commands, wired the
  optional pass plan into `SceneDrawPlan` only for drawable sprite plans, kept
  raster-only/unavailable/missing-atlas/empty-atlas paths outside pass
  planning, exported the public renderer types, and documented sprite
  render-pass planning in README/SPEC. Validation passed with focused renderer
  tests (36/36), the public API guard, `cargo test --all-targets` (1157
  library tests plus binary/example tests), clippy with warnings denied,
  `make fidelity` (10 local fixtures, 15452 frames, 52/52 new executable Rust
  lines), `cargo run -- --live-smoke` (239 rendered frames), `cargo fmt
  --check`, markdownlint, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778823804345559`

### DC-112: Clean Sprite Pipeline Planning

Status: `complete`

Goal: describe the `wgpu` shader and render-pipeline state needed by the clean
sprite render-pass plan, so future native sprite rendering can create pipelines
without presenter-side classification.

Scope:

- Add renderer-owned sprite WGSL shader metadata and entry-point names.
- Add sprite vertex-buffer layout metadata for the quad vertex and instance
  streams, using stable pass binding slots.
- Add sprite render-pipeline metadata for primitive topology, color target,
  alpha blending, write mask, and multisample state.
- Wire an optional sprite pipeline plan into `SceneDrawPlan` whenever a sprite
  render-pass plan exists.
- Keep raster-only scenes, unavailable sprite pipelines, missing atlas regions,
  empty atlas surfaces, and empty command lists outside sprite pipeline
  planning.
- Add focused renderer tests for shader descriptors, vertex layouts, color
  target settings, custom texture formats, and absent sprite paths.
- Update README/SPEC module text for sprite pipeline planning.

Acceptance criteria:

- `SceneDrawPlan` exposes sprite pipeline metadata only for drawable clean
  sprite plans.
- The pipeline plan identifies the renderer-owned WGSL shader, vertex entry,
  fragment entry, quad/instance vertex layouts, and `wgpu` color target state.
- Raster-only scenes and unavailable sprite paths do not produce sprite
  pipeline plans.

Validation:

```sh
cargo fmt --check
cargo test --lib renderer::tests::
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 06:46:10 BST` Started `DC-112`: adding clean sprite `wgpu`
  pipeline planning with renderer-owned WGSL shader metadata, quad/instance
  vertex layouts, primitive/color target/multisample state, focused renderer
  tests, and docs.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778823967741159`
- `2026-05-15 07:09:26 BST` Completed `DC-112`: added renderer-owned sprite
  WGSL shader metadata and shader module descriptor data, quad/instance vertex
  layout plans, alpha-blended color target state, primitive state, multisample
  state, settings-driven target texture formats, optional `SceneDrawPlan`
  wiring only for drawable sprite plans, public exports, and README/SPEC
  documentation for sprite pipeline planning. Validation passed with focused
  renderer tests (39/39), the public API guard, `cargo test --all-targets`
  (1160 library tests plus binary/example tests), clippy with warnings denied,
  `make fidelity` (10 local fixtures, 15452 frames, 65/65 new executable Rust
  lines), `cargo run -- --live-smoke` (239 rendered frames), `cargo fmt
  --check`, markdownlint, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778825367225579`

### DC-113: Clean Sprite Resource Binding Planning

Status: `complete`

Goal: describe the `wgpu` resource bindings needed by the clean sprite
pipeline and render-pass plans, so future native sprite rendering can create
projection uniform buffers and sprite atlas bind groups without presenter-side
classification.

Scope:

- Add upload metadata for the scene-projection uniform buffer used by sprite
  shaders.
- Add renderer-owned bind group layout metadata for scene projection and sprite
  atlas resources.
- Add sprite atlas texture view and sampler binding metadata that matches the
  WGSL shader bindings.
- Wire optional sprite resource binding plans into `SceneDrawPlan` whenever
  both sprite pipeline and render-pass plans exist.
- Keep empty surfaces, raster-only scenes, unavailable sprite pipelines,
  missing atlas regions, empty atlas surfaces, and empty command lists outside
  sprite resource binding planning.
- Add focused renderer tests for uniform upload bytes, buffer usages, bind group
  entries, shader binding alignment, custom target plans, and absent paths.
- Update README/SPEC module text for sprite resource binding planning.

Acceptance criteria:

- `SceneDrawPlan` exposes sprite resource binding metadata only for drawable
  clean sprite plans with a valid scene projection.
- Binding metadata uses `wgpu` shader stages, binding types, buffer usages,
  texture sample types, texture view dimensions, and sampler binding types.
- Raster-only scenes and invalid sprite paths do not produce sprite resource
  binding plans.

Validation:

```sh
cargo fmt --check
cargo test --lib renderer::tests::
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 07:11:18 BST` Started `DC-113`: adding clean sprite resource
  binding planning with projection uniform upload metadata, sprite bind group
  layout entries, atlas texture/sampler bindings, focused renderer tests, and
  docs.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778825476528979`
- `2026-05-15 07:37:01 BST` Completed `DC-113`: added renderer-owned sprite
  resource binding planning for projection uniform uploads, scene-projection
  and sprite-atlas bind group layouts, atlas texture/sampler metadata, and
  optional `SceneDrawPlan` wiring only for valid drawable sprite plans. Verified
  with `cargo test --all-targets` (1162 library tests, 2 binary tests, 2
  example tests), `cargo clippy --all-targets -- -D warnings`, `make fidelity`
  (10 fixtures, 15452 frames, 81/81 non-baselined added executable Rust lines
  covered), `cargo run -- --live-smoke` (239 rendered frames), `cargo fmt
  --check`, markdownlint, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778827021576789`

### DC-114: Clean Sprite Atlas Texture Upload Planning

Status: `complete`

Goal: add renderer-owned sprite atlas pixel data and `wgpu` texture upload
metadata so future native sprite rendering can create and populate the atlas
texture without presenter-side classification or temporary full-frame raster
data.

Scope:

- Add RGBA pixel ownership to the clean sprite atlas.
- Generate deterministic default sprite atlas pixels for the current clean
  sprite regions.
- Add sprite atlas texture upload metadata for texture format, usage, copy
  layout, extent, byte length, bytes, and nonblank evidence.
- Wire atlas texture upload metadata into sprite resource binding planning only
  for valid drawable sprite plans.
- Keep empty atlas surfaces, empty pixel buffers, raster-only scenes, missing
  atlas regions, unavailable sprite pipelines, and empty command lists outside
  atlas texture upload planning.
- Add focused renderer tests for atlas pixel ownership, upload descriptor
  fields, copy layout, resource-plan wiring, and absent paths.
- Update README/SPEC module text for sprite atlas texture upload planning.

Acceptance criteria:

- `SceneDrawPlan` exposes sprite atlas texture upload metadata only for
  drawable clean sprite plans.
- The upload metadata uses `wgpu` texture format, usage flags, texture
  dimension, copy layout, and copy extent values needed by `Queue::write_texture`.
- Default clean sprite atlas pixels are deterministic and nonblank.

Validation:

```sh
cargo fmt --check
cargo test --lib renderer::tests::
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 07:40:53 BST` Started `DC-114`: adding renderer-owned sprite
  atlas RGBA data and `wgpu` texture upload metadata, with focused renderer
  tests and docs.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778827268523169`
- `2026-05-15 08:21:58 BST` Completed `DC-114`: added RGBA pixel ownership to
  `TextureAtlas`, deterministic nonblank default sprite atlas pixels, sprite
  atlas texture upload metadata for `wgpu` texture creation and
  `Queue::write_texture` copy layout, and resource-plan wiring only for valid
  drawable sprite plans. Verified with focused renderer tests (42/42),
  `cargo test --all-targets` (1163 library tests, 2 binary tests, 2 example
  tests), `cargo clippy --all-targets -- -D warnings`, `make fidelity` (10
  fixtures, 15452 frames, 103/103 non-baselined added executable Rust lines
  covered), `cargo run -- --live-smoke` (239 rendered frames), `cargo fmt
  --check`, markdownlint, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778829734086439`

### DC-115: Clean Sprite Pipeline Layout Planning

Status: `complete`

Goal: connect the clean sprite pipeline plan with the sprite resource binding
plans by describing the ordered `wgpu` pipeline layout metadata needed to bind
scene projection and sprite atlas resources without presenter-side
classification.

Scope:

- Add sprite pipeline layout metadata with stable bind group ordering.
- Preserve scene-projection and sprite-atlas bind group roles, labels, group
  indices, and entry counts in the layout plan.
- Include the `wgpu` immediate-size value needed by `PipelineLayoutDescriptor`.
- Wire optional sprite pipeline layout plans into `SceneDrawPlan` only when
  sprite pipeline and sprite resource binding plans are both present.
- Keep raster-only scenes, missing atlas regions, empty atlas surfaces, invalid
  atlas pixel data, unavailable sprite pipelines, empty command lists, and empty
  targets outside sprite pipeline layout planning.
- Add focused renderer tests for layout ordering, descriptor-ready metadata,
  resource-plan alignment, custom target settings, and absent paths.
- Update README/SPEC module text for sprite pipeline layout planning.

Acceptance criteria:

- `SceneDrawPlan` exposes sprite pipeline layout metadata only for drawable
  clean sprite plans.
- The layout plan orders scene projection at group 0 and sprite atlas at group
  1, matching the WGSL shader and resource binding plan.
- Raster-only scenes and invalid sprite paths do not produce sprite pipeline
  layout plans.

Validation:

```sh
cargo fmt --check
cargo test --lib renderer::tests::
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 08:24:21 BST` Started `DC-115`: adding renderer-owned sprite
  pipeline layout metadata with ordered scene-projection and sprite-atlas bind
  group layout slots, descriptor-ready immediate size, focused renderer tests,
  and docs.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778829878218629`
- `2026-05-15 08:49:08 BST` Completed `DC-115`: added descriptor-ready sprite
  pipeline layout metadata with ordered scene-projection group 0 and sprite
  atlas group 1, wired optional layout plans into `SceneDrawPlan` only for
  drawable sprite plans with sprite pipeline, sprite resource bindings, and a
  nonempty viewport, and kept raster-only, invalid sprite-resource, empty
  command, unavailable-pipeline, and empty-target paths outside layout
  planning. Verified with focused renderer tests (44/44), public API guard
  (1/1), `cargo test --all-targets` (1165 library tests, 2 binary tests, 2
  example tests), `cargo clippy --all-targets -- -D warnings`,
  `make fidelity` (10 fixtures, 15452 frames, 23/23 non-baselined added
  executable Rust lines covered), `cargo run -- --live-smoke` (239 rendered
  frames), `cargo fmt --check`, markdownlint, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778831347938799`

### DC-116: Clean Sprite Render Pipeline Descriptor Planning

Status: `complete`

Goal: connect the clean sprite pipeline, layout, shader, and target state into
descriptor-ready metadata for `wgpu` render pipeline creation without
presenter-side classification.

Scope:

- Add sprite render pipeline descriptor metadata that names the shader entries,
  layout, vertex buffers, primitive state, color target, and multisample state.
- Preserve the ordered sprite pipeline layout and `wgpu` immediate-size value in
  the descriptor plan.
- Wire optional descriptor plans into `SceneDrawPlan` only when the sprite render
  pass, sprite pipeline, sprite resource bindings, sprite pipeline layout, and
  nonempty viewport are all present.
- Keep raster-only scenes, missing atlas regions, empty atlas surfaces, invalid
  atlas pixel data, unavailable sprite pipelines, empty command lists, and empty
  targets outside sprite render pipeline descriptor planning.
- Add focused renderer tests for descriptor-ready metadata, layout/pipeline
  alignment, custom target settings, and absent paths.
- Update README/SPEC module text for sprite render pipeline descriptor planning.

Acceptance criteria:

- `SceneDrawPlan` exposes sprite render pipeline descriptor metadata only for
  drawable clean sprite plans.
- Descriptor metadata matches the WGSL shader entries, layout label, ordered
  bind group count, vertex-buffer count, color target format, and primitive
  state used by the sprite pipeline.
- Raster-only scenes and invalid sprite paths do not produce sprite render
  pipeline descriptor plans.

Validation:

```sh
cargo fmt --check
cargo test --lib renderer::tests::
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 08:52:00 BST` Started `DC-116`: adding descriptor-ready sprite
  render pipeline metadata that combines shader entries, ordered pipeline
  layout slots, vertex buffers, primitive state, color target, and multisample
  state, with focused renderer tests and docs.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778831518959189`
- `2026-05-15 09:14:57 BST` Completed `DC-116`: added descriptor-ready sprite
  render pipeline metadata that combines shader entries, ordered pipeline
  layout slots, vertex buffers, primitive state, color target, and multisample
  state; wired optional descriptor plans into `SceneDrawPlan` only when sprite
  render pass, sprite pipeline, sprite resource bindings, sprite pipeline
  layout, and a nonempty viewport are present; and kept raster-only, invalid
  sprite-resource, empty-command, unavailable-pipeline, and empty-target paths
  outside descriptor planning. Verified with focused renderer tests (45/45),
  public API guard (1/1), `cargo test --all-targets` (1166 library tests, 2
  binary tests, 2 example tests), `cargo clippy --all-targets -- -D warnings`,
  `make fidelity` (10 fixtures, 15452 frames, 31/31 non-baselined added
  executable Rust lines covered), `cargo run -- --live-smoke` (239 rendered
  frames), `cargo fmt --check`, markdownlint, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778832895406269`

### DC-117: Clean Sprite Render-Pass Encoder Command Planning

Status: `complete`

Goal: connect clean sprite render-pass, resource binding, pipeline layout, and
render pipeline descriptor plans into ordered `wgpu` render-pass encoder command
metadata without presenter-side scene classification.

Scope:

- Add sprite render-pass encoder command metadata for setting the render
  pipeline, projection and atlas bind groups, quad and instance vertex buffers,
  index buffer, and indexed draw ranges.
- Preserve command ordering needed by `wgpu::RenderPass`: bind pipeline and
  resources before draw commands.
- Wire optional encoder command plans into `SceneDrawPlan` only when the sprite
  render pass, sprite resource bindings, sprite pipeline layout, sprite render
  pipeline descriptor, and nonempty viewport are all present.
- Keep raster-only scenes, missing atlas regions, empty atlas surfaces, invalid
  atlas pixel data, unavailable sprite pipelines, empty command lists, and empty
  targets outside sprite render-pass encoder command planning.
- Add focused renderer tests for command ordering, buffer/bind-group metadata,
  draw range preservation, custom target settings, and absent paths.
- Update README/SPEC module text for sprite render-pass encoder command
  planning.

Acceptance criteria:

- `SceneDrawPlan` exposes sprite render-pass encoder command metadata only for
  drawable clean sprite plans.
- Encoder command metadata orders pipeline, bind groups, vertex buffers, index
  buffer, and draw commands in the sequence expected by `wgpu::RenderPass`.
- Raster-only scenes and invalid sprite paths do not produce sprite render-pass
  encoder command plans.

Validation:

```sh
cargo fmt --check
cargo test --lib renderer::tests::
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 09:17:11 BST` Started `DC-117`: adding ordered `wgpu`
  render-pass encoder command metadata for clean sprite draws, covering
  pipeline, projection and atlas bind groups, vertex and index buffers, indexed
  draw ranges, focused renderer tests, and docs.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778833031980069`
- `2026-05-15 09:40:19 BST` Completed `DC-117`: added ordered sprite
  `wgpu::RenderPass` encoder command metadata for setting the render pipeline,
  projection and atlas bind groups, quad and instance vertex buffers, index
  buffer, and indexed draw ranges; wired optional encoder command plans into
  `SceneDrawPlan` only when sprite render pass, sprite resource bindings,
  sprite pipeline layout, sprite render pipeline descriptor, and a nonempty
  viewport are present; and kept raster-only, invalid sprite-resource,
  empty-command, unavailable-pipeline, and empty-target paths outside encoder
  command planning. Verified with focused renderer tests (46/46), public API
  guard (1/1), `cargo test --all-targets` (1167 library tests, 2 binary tests,
  2 example tests), `cargo clippy --all-targets -- -D warnings`,
  `make fidelity` (10 fixtures, 15452 frames, 58/58 non-baselined added
  executable Rust lines covered), `cargo run -- --live-smoke` (240 rendered
  frames), `cargo fmt --check`, markdownlint, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778834419221119`

### DC-118: Clean Frame-Level GPU Command Planning

Status: `complete`

Goal: combine clean `wgpu` pass state, sprite render-pass encoder plans, and
temporary raster evidence into one ordered scene-level frame command stream so
presenters do not need to classify scene internals.

Scope:

- Add frame-level GPU command metadata for pass clear color, viewport,
  scene-projection upload presence, sprite render-pass encoder execution, and
  temporary raster evidence.
- Wire the frame command plan into `SceneDrawPlan` for every prepared scene.
- Preserve sprite command emission only when the sprite render-pass encoder plan
  is present and the target has a nonempty viewport.
- Preserve temporary raster upload evidence as a separate frame command without
  treating it as the final clean sprite path.
- Add focused renderer tests for sprite-only, raster-only, mixed sprite/raster,
  empty-target, and invalid sprite-resource paths.
- Update README/SPEC module text for frame-level GPU command planning.

Acceptance criteria:

- `SceneDrawPlan` exposes an ordered frame-level command plan.
- Sprite scenes include pass setup and sprite render-pass execution commands
  without presenter-side scene classification.
- Raster-only and mixed raster paths preserve temporary raster evidence as a
  separate command, and invalid sprite paths do not produce sprite execution
  commands.

Validation:

```sh
cargo fmt --check
cargo test --lib renderer::tests::
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 09:42:00 BST` Started `DC-118`: adding ordered scene-level
  `wgpu` frame command metadata for pass setup, optional sprite render-pass
  encoder execution, and temporary raster evidence, with focused renderer tests
  and docs.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778834515838479`
- `2026-05-15 10:06:23 BST` Completed `DC-118`: added `WgpuFramePlan` and
  `WgpuFrameCommand` metadata so prepared draw plans expose ordered frame
  commands for pass begin, viewport setup, scene-projection upload presence,
  temporary raster evidence, and sprite render-pass encoder execution. Empty
  and invalid sprite paths now keep sprite execution absent from frame command
  planning, while mixed sprite/raster paths preserve both sprite execution and
  temporary raster upload evidence as separate commands. README and SPEC now
  describe the renderer's frame-level GPU command planning surface.
  Validation passed with `cargo test --lib renderer::tests::` (47 passed),
  `cargo test --lib public_api_tests::clean_contracts_have_public_game_simulation`,
  `cargo test --all-targets` (1168 library tests, 2 binary tests, and
  2 example tests), `cargo clippy --all-targets -- -D warnings`,
  `make fidelity` (10 fixtures, 15452 frames, 42/42 non-baselined added
  executable Rust lines covered), `cargo run -- --live-smoke` (239 rendered
  frames), `cargo fmt --check`, markdownlint, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778835971827569`

### DC-119: Clean Gameplay World Sprite Surface

Status: `complete`

Goal: move the clean gameplay scene beyond player/HUD placeholders by giving
the clean game domain explicit world entities for terrain, stars, enemies, and
humans that render through renderer-owned sprite atlas IDs.

Scope:

- Add clean world snapshot types for terrain, starfield, enemies, and humans.
- Seed the first clean playing wave with deterministic world entities.
- Render those world entities through clean `RenderScene` sprites.
- Add renderer-owned atlas entries for enemy and human sprites.
- Update focused clean game and renderer tests for sprite-layer coverage.
- Update README/SPEC module text for the expanded clean gameplay world surface.

Acceptance criteria:

- `GameState` exposes clean world entity snapshots without legacy memory labels.
- The clean playing scene includes terrain, starfield, enemy, human, player,
  projectile, and HUD sprites through renderer-owned sprite IDs.
- The default sprite atlas resolves every sprite used by the clean game scene.

Validation:

```sh
cargo fmt --check
cargo test --lib game::tests::
cargo test --lib renderer::tests::texture_atlas_owns_sprite_regions
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 10:08:34 BST` Started `DC-119`: adding clean domain world
  snapshots for terrain, stars, enemies, and humans, renderer-owned atlas
  entries for the new sprites, and focused tests/docs for the sprite-first
  clean gameplay scene surface.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778836113993949`
- `2026-05-15 10:53:08 BST` Completed `DC-119`: added clean `WorldSnapshot`
  domain data for terrain, stars, enemies, and humans on `GameState`, seeded
  deterministic first-wave world entities when the clean game starts, and
  rendered terrain, starfield, enemy, human, player, projectile, and HUD
  sprites through `RenderScene`. The renderer now owns sprite IDs and atlas
  entries for lander and human sprites, and the focused clean game tests verify
  that emitted world sprites are atlas-backed and that carried humans use their
  highlighted tint branch. README and SPEC now describe the expanded clean
  gameplay world surface.
  Validation passed with `cargo test --lib game::tests::` (11 passed),
  `cargo test --lib renderer::tests::texture_atlas_owns_sprite_regions`,
  `cargo test --lib public_api_tests::clean_contracts_have_public_game_simulation`,
  `cargo test --all-targets` (1170 library tests, 2 binary tests, and
  2 example tests), `cargo clippy --all-targets -- -D warnings`,
  `make fidelity` (10 fixtures, 15452 frames, 80/80 non-baselined added
  executable Rust lines covered), `cargo run -- --live-smoke` (240 rendered
  frames), `cargo fmt --check`, markdownlint, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778838787881489`

### DC-120: Clean Enemy Motion System

Status: `complete`

Goal: move clean first-wave enemies through deterministic gameplay systems
instead of leaving them as static scene placeholders.

Scope:

- Add clean enemy velocity data to world enemy snapshots.
- Add a deterministic enemy motion system in `src/systems.rs`.
- Advance clean enemies during playing frames before scene rendering.
- Preserve atlas-backed clean world sprite rendering.
- Add focused systems and game tests for enemy movement and wrapping.
- Update README/SPEC module text for clean enemy-motion ownership.

Acceptance criteria:

- Clean enemy movement is represented by gameplay-domain data, not legacy
  memory tables.
- Playing frames advance enemy positions deterministically through a system.
- Enemy sprites continue to render through renderer-owned atlas-backed scene
  sprites after movement.

Validation:

```sh
cargo fmt --check
cargo test --lib systems::tests::enemy_motion_system
cargo test --lib game::tests::
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 10:54:30 BST` Started `DC-120`: adding clean enemy velocity,
  deterministic enemy motion, focused systems/game tests, and docs for moving
  clean world entities through systems rather than legacy memory tables.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778838869364999`
- `2026-05-15 11:17:12 BST` Completed `DC-120`: added clean enemy velocity
  to enemy snapshots, introduced `ScreenVelocity` and `EnemyMotionSystem` for
  deterministic wrapping movement, and advanced clean first-wave enemies
  through the system during playing frames before scene generation. README and
  SPEC now describe clean enemy-motion ownership without relying on legacy
  memory-table names.
  Validation passed with
  `cargo test --lib systems::tests::enemy_motion_system`,
  `cargo test --lib game::tests::` (12 passed),
  `cargo test --lib public_api_tests::clean_contracts_have_public_game_simulation`,
  `cargo test --all-targets` (1172 library tests, 2 binary tests, and
  2 example tests), `cargo clippy --all-targets -- -D warnings`,
  `make fidelity` (10 fixtures, 15452 frames, 13/13 non-baselined added
  executable Rust lines covered), `cargo run -- --live-smoke` (239 rendered
  frames), `cargo fmt --check`, markdownlint, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778840224712949`

### DC-121: Clean Projectile World Motion Surface

Status: `complete`

Goal: move active projectiles into the clean gameplay world snapshot and update
them through deterministic motion instead of keeping them as private renderer
bookkeeping inside `Game`.

Scope:

- Add clean projectile snapshot data to `WorldSnapshot`.
- Add deterministic projectile velocity and movement in `src/systems.rs`.
- Launch projectiles into the clean world snapshot with direction-derived
  velocity.
- Advance and cull active projectiles during playing frames.
- Render projectile sprites from clean world data.
- Update focused systems/game tests plus README/SPEC module text.

Acceptance criteria:

- Active projectiles are visible in `GameState` as gameplay-domain world data.
- Projectile movement and culling are represented by a deterministic system.
- Projectile sprites continue to render through renderer-owned atlas-backed
  scene sprites.

Validation:

```sh
cargo fmt --check
cargo test --lib systems::tests::projectile
cargo test --lib game::tests::
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 11:20:04 BST` Started `DC-121`: moving active projectiles into
  clean world snapshots, adding projectile velocity/motion/culling systems,
  and keeping projectile rendering atlas-backed from gameplay-domain data.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778840426841509`
- `2026-05-15 11:43:01 BST` Completed `DC-121`: added
  `ProjectileSnapshot` to `WorldSnapshot`, moved active projectile ownership
  out of private `Game` bookkeeping, and introduced `ProjectileMotionSystem`
  for direction-derived velocity, deterministic advancement, and screen-exit
  culling. `Game` now launches projectiles into clean world state and renders
  projectile sprites from that gameplay-domain data.
  Validation passed with `cargo test --lib systems::tests::projectile`
  (2 passed), `cargo test --lib game::tests::` (14 passed),
  `cargo test --lib public_api_tests::clean_contracts_have_public_game_simulation`,
  `cargo test --all-targets` (1175 library tests, 2 binary tests, and
  2 example tests), `cargo clippy --all-targets -- -D warnings`,
  `make fidelity` (10 fixtures, 15452 frames, 32/32 non-baselined added
  executable Rust lines covered), `cargo run -- --live-smoke` (239 rendered
  frames), `cargo fmt --check`, markdownlint, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778841782224139`

### DC-122: Clean Projectile Enemy Collision System

Status: `complete`

Goal: resolve projectile-to-enemy hits through clean collision and scoring
systems instead of relying on hidden object-list or renderer-side state.

Scope:

- Add clean axis-aligned collision primitives in `src/systems.rs`.
- Detect the first active projectile/enemy overlap deterministically.
- Remove the hit projectile and enemy from `WorldSnapshot`.
- Award clean score for destroyed enemies and emit a gameplay event.
- Keep projectile and enemy sprites absent after collision resolution.
- Update focused systems/game tests plus README/SPEC module text.

Acceptance criteria:

- Projectile/enemy collisions are represented by gameplay-domain systems, not
  legacy object lists or memory labels.
- A hit removes exactly the colliding projectile and enemy from clean world
  state.
- Enemy score changes are visible in `GameState` and collision output is
  reflected in `GameEvents` and scene sprites.

Validation:

```sh
cargo fmt --check
cargo test --lib systems::tests::collision
cargo test --lib game::tests::
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 11:44:28 BST` Started `DC-122`: adding clean projectile/enemy
  collision detection, hit removal, score award, gameplay event output, and
  docs/tests for collision ownership.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778841890018489`
- `2026-05-15 12:26:46 BST` Completed `DC-122`: added clean AABB collision
  primitives, deterministic first projectile/enemy hit detection, gameplay
  removal of the hit projectile and enemy, current-player score awards,
  `EnemyDestroyed` gameplay output, and absent hit sprites in the clean scene.
  Added focused systems/game coverage including the second-player scoring
  branch exposed by the first coverage run. Validation passed with
  `cargo fmt --check`, `cargo test --lib systems::tests::collision` (2 tests),
  `cargo test --lib game::tests::` (16 tests), `cargo test --all-targets`
  (1179 lib tests, 2 bin tests, 2 example tests),
  `cargo clippy --all-targets -- -D warnings`, `make fidelity` (10 fixtures,
  15452 frames, new Rust line coverage 63/63),
  `cargo run -- --live-smoke` (239 rendered frames),
  `markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md
  docs/fidelity/live-audio.md`, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778844407133399`

### DC-123: Clean Wave Completion System

Status: `complete`

Goal: move wave clear and next-wave spawning into clean gameplay systems
instead of leaving enemy exhaustion as an inert world state.

Scope:

- Add a deterministic clean wave-completion system in `src/systems.rs`.
- When the last clean enemy is destroyed, emit wave-clear gameplay output
  without immediately respawning over the hit frame.
- Start the next wave on the following playing frame with clean world
  repopulation and sprite-visible enemies.
- Keep player, score, lives, smart bombs, terrain, humans, projectiles, and
  enemy wave state owned by clean domain structures.
- Update focused systems/game tests plus README/SPEC/PLAN docs.

Acceptance criteria:

- Enemy exhaustion is represented by gameplay-domain wave output, not renderer
  state or legacy object lists.
- The frame that destroys the last enemy shows the hit projectile and enemy
  absent and reports wave-clear output.
- The following playing frame advances `GameState.wave`, repopulates clean
  enemies deterministically, and renders those enemies as atlas-backed sprites.

Validation:

```sh
cargo fmt --check
cargo test --lib systems::tests::wave
cargo test --lib game::tests::clean_game_wave
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 12:28:19 BST` Started `DC-123`: adding clean wave-completion
  evaluation, delayed next-wave spawning, wave gameplay events, and focused
  systems/game/docs coverage.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778844499996319`
- `2026-05-15 12:50:46 BST` Completed `DC-123`: added clean `WaveSystem`
  progress/clear evaluation, saturating next-wave numbering, `WaveCleared`
  output on the last-enemy destruction frame, delayed `WaveStarted` output, and
  deterministic clean enemy repopulation on the following playing frame. The
  last-hit frame keeps the hit projectile/enemy absent while the next wave
  renders atlas-backed enemies from clean world state. Validation passed with
  `cargo fmt --check`, `cargo test --lib systems::tests::wave` (1 test),
  `cargo test --lib game::tests::clean_game_wave` (1 test),
  `cargo test --lib game::tests::` (17 tests),
  `cargo test --lib systems::tests::` (15 tests), `cargo test --all-targets`
  (1181 lib tests, 2 bin tests, 2 example tests),
  `cargo clippy --all-targets -- -D warnings`, `make fidelity` (10 fixtures,
  15452 frames, new Rust line coverage 36/36),
  `cargo run -- --live-smoke` (239 rendered frames),
  `markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md
  docs/fidelity/live-audio.md`, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778845846596549`

### DC-124: Clean Bonus Award System

Status: `complete`

Goal: move score mutation, high-score tracking, and bonus-threshold awards into
clean gameplay systems instead of mutating score fields directly in `Game`.

Scope:

- Add a deterministic clean scoring/bonus system in `src/systems.rs`.
- Route enemy score awards through that system.
- Update the current player's score, high score, next bonus threshold, lives,
  and smart bombs through clean domain output.
- Emit `BonusAwarded` when a point award crosses the configured bonus
  threshold.
- Update focused systems/game tests plus README/SPEC/PLAN docs.

Acceptance criteria:

- Enemy scoring is represented by gameplay-domain scoring output, not direct
  field mutation in collision handling.
- Crossing `ScoreSnapshot.next_bonus` awards clean player stock and emits a
  gameplay event on the same frame as the score award.
- Scores, high score, next bonus, lives, smart bombs, and collision/wave events
  remain visible through `GameState` and `GameEvents`.

Validation:

```sh
cargo fmt --check
cargo test --lib systems::tests::score
cargo test --lib game::tests::clean_game_bonus
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 12:53:05 BST` Started `DC-124`: adding clean score/bonus
  evaluation, routing enemy score awards through it, emitting bonus gameplay
  output, and updating focused systems/game/docs coverage.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778845983462809`
- `2026-05-15 13:15:06 BST` Completed `DC-124`: added clean `ScoreSystem`
  scoring output for current-player point awards, high-score tracking, bonus
  threshold advancement, and player stock updates; routed enemy score awards
  through it; and emitted `BonusAwarded` when a collision score crosses
  `next_bonus` while preserving collision and wave event ordering. Validation
  passed with `cargo fmt --check`, `cargo test --lib systems::tests::score`
  (3 tests), `cargo test --lib game::tests::clean_game_bonus` (1 test),
  `cargo test --lib game::tests::` (18 tests),
  `cargo test --lib systems::tests::` (18 tests), `cargo test --all-targets`
  (1185 lib tests, 2 bin tests, 2 example tests),
  `cargo clippy --all-targets -- -D warnings`, `make fidelity` (10 fixtures,
  15452 frames, new Rust line coverage 43/43),
  `cargo run -- --live-smoke` (239 rendered frames),
  `markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md
  docs/fidelity/live-audio.md`, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778847304885369`

### DC-125: Clean Smart Bomb Resolution System

Status: `complete`

Goal: resolve smart bomb enemy clearing through clean gameplay systems instead
of treating smart bomb input as a stock-only event.

Scope:

- Add deterministic clean smart-bomb resolution in `src/systems.rs`.
- Route `SmartBombPressed` handling through that system after stock
  consumption.
- Remove affected enemies from clean world state.
- Award enemy score through `ScoreSystem`, emit enemy/bonus/wave gameplay
  output, and keep cleared enemies absent from scene sprites.
- Update focused systems/game tests plus README/SPEC/PLAN docs.

Acceptance criteria:

- Smart bomb effects are represented by gameplay-domain output, not renderer
  state or legacy object lists.
- A smart bomb with active enemies clears those clean enemies, awards score,
  and emits enemy-destroyed gameplay output in the same frame.
- Cleared enemies are absent from the frame scene, and wave clear still uses
  the clean wave system.

Validation:

```sh
cargo fmt --check
cargo test --lib systems::tests::smart_bomb
cargo test --lib game::tests::clean_game_smart_bomb
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 13:16:15 BST` Started `DC-125`: adding clean smart-bomb enemy
  clear output, routing smart-bomb scoring through `ScoreSystem`, preserving
  clean wave-clear handling, and updating focused systems/game/docs coverage.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778847373539249`
- `2026-05-15 13:38:48 BST` Completed `DC-125`: added `SmartBombSystem` and
  routed smart-bomb input through clean enemy clearing, score/bonus awards,
  wave-clear output, and scene sprite removal. Validation passed with
  `cargo fmt --check`, `cargo test --lib systems::tests::smart_bomb`,
  `cargo test --lib game::tests::clean_game_smart_bomb`, full game and systems
  test modules, public API guard, `cargo test --all-targets` (1187 lib, 2 bin,
  and 2 example tests), `cargo clippy --all-targets -- -D warnings`,
  `make fidelity` (10 fixtures, 15452 frames, new Rust line coverage 15/15),
  `cargo run -- --live-smoke` (240 rendered frames), `markdownlint README.md
  SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md`,
  and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778848724960339`

### DC-126: Clean Player Damage System

Status: `complete`

Goal: resolve player/enemy collisions and life loss through clean gameplay
systems instead of leaving enemy contact as an unmodeled scene overlap.

Scope:

- Add deterministic clean player/enemy collision output in `src/systems.rs`.
- Add a clean player-damage system for life decrement and game-over status.
- Route playing-frame player/enemy overlap through those systems after enemy
  and projectile updates.
- Remove the colliding enemy from clean world state, emit player-destruction
  output, and preserve wave-clear behavior for surviving players.
- Update focused systems/game tests plus README/SPEC/PLAN docs.

Acceptance criteria:

- Player damage is represented by gameplay-domain output, not renderer state or
  temporary oracle structures.
- A player/enemy overlap decrements clean player lives and removes the
  colliding clean enemy in the same frame.
- The final-life collision enters `GameOver`, emits explicit gameplay output,
  and leaves gameplay sprites absent from the game-over scene.

Validation:

```sh
cargo fmt --check
cargo test --lib systems::tests::player_damage
cargo test --lib systems::tests::collision_system_reports_first_player_enemy_hit
cargo test --lib game::tests::clean_game_player_enemy
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 13:40:41 BST` Started `DC-126`: adding clean player/enemy
  collision detection, deterministic player stock damage, player-destroyed and
  game-over gameplay output, scene cleanup, and focused systems/game/docs
  coverage.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778848852483269`
- `2026-05-15 14:02:46 BST` Completed `DC-126`: added clean player/enemy
  collision output and `PlayerDamageSystem`; routed contact through clean
  gameplay state so collisions remove the colliding enemy, decrement lives,
  emit `PlayerDestroyed`, preserve wave clear when the player survives, and
  enter `GameOver` with gameplay sprites absent on the final life. Validation
  passed with `cargo fmt --check`,
  `cargo test --lib systems::tests::player_damage`,
  `cargo test --lib systems::tests::collision_system_reports_first_player_enemy_hit`,
  `cargo test --lib game::tests::clean_game_player_enemy`, full game and
  systems test modules, public API guard, `cargo test --all-targets` (1191 lib,
  2 bin, and 2 example tests), `cargo clippy --all-targets -- -D warnings`,
  `make fidelity` (10 fixtures, 15452 frames, new Rust line coverage 45/45),
  `cargo run -- --live-smoke` (240 rendered frames), `markdownlint README.md
  SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md`,
  and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778850177520539`

### DC-127: Clean Operator Control System

Status: `complete`

Goal: route operator-facing clean inputs through deterministic gameplay-domain
handling instead of leaving diagnostics, audits, and high-score reset fields as
unused input surface.

Scope:

- Add a clean operator trigger system in `src/systems.rs`.
- Debounce diagnostics, audits, and high-score reset inputs at the clean game
  boundary.
- Emit clean gameplay events for diagnostics/audits selection.
- Reset the clean high-score field through gameplay state when requested.
- Update focused systems/game tests plus README/SPEC/PLAN docs.

Acceptance criteria:

- Operator inputs are represented by clean gameplay-domain output rather than
  ad hoc renderer or runtime behavior.
- Holding an operator input does not emit repeated gameplay events every frame.
- High-score reset updates `ScoreSnapshot.high_score` without changing current
  player scores or bonus thresholds.

Validation:

```sh
cargo fmt --check
cargo test --lib systems::tests::operator_control
cargo test --lib game::tests::clean_game_operator
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 14:04:00 BST` Started `DC-127`: adding clean operator trigger
  handling for diagnostics, audits, and high-score reset, including debounce
  behavior, gameplay events, docs, and focused systems/game coverage.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778850249279209`
- `2026-05-15 14:26:17 BST` Completed `DC-127`: added
  `OperatorControlSystem` and routed diagnostics, audits, and high-score reset
  through clean `Game` state. Operator controls now emit edge-triggered
  gameplay events, held inputs do not repeat every frame, and high-score reset
  clears only `ScoreSnapshot.high_score` while preserving current player scores
  and bonus thresholds. Validation passed with `cargo fmt --check`,
  `cargo test --lib systems::tests::operator_control`,
  `cargo test --lib game::tests::clean_game_operator`, full game and systems
  test modules, public API guard, `cargo test --all-targets` (1193 lib, 2 bin,
  and 2 example tests), `cargo clippy --all-targets -- -D warnings`,
  `make fidelity` (10 fixtures, 15452 frames, new Rust line coverage 31/31),
  `cargo run -- --live-smoke` (239 rendered frames), `markdownlint README.md
  SPEC.md PLAN.md docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md`,
  and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778851591636559`

### DC-128: Clean High-Score Entry Start System

Status: `complete`

Goal: activate the clean high-score entry phase at game over instead of leaving
the `HighScoreEntry` phase and entry-start event dormant.

Scope:

- Add a deterministic clean high-score qualification system in `src/systems.rs`.
- Route final-life game-over through high-score qualification.
- Enter `HighScoreEntry` and emit `HighScoreEntryStarted` for qualifying
  scores.
- Preserve `GameOver` for non-qualifying scores and keep gameplay sprites
  absent from both terminal scenes.
- Update focused systems/game tests plus README/SPEC/PLAN docs.

Acceptance criteria:

- High-score entry start is represented by clean gameplay-domain state and
  events.
- A positive current-player score that matches or exceeds the tracked high
  score enters `HighScoreEntry` after the final life is lost.
- A zero or non-qualifying score remains in `GameOver`.

Validation:

```sh
cargo fmt --check
cargo test --lib systems::tests::high_score
cargo test --lib game::tests::clean_game_high_score_entry
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 14:27:33 BST` Started `DC-128`: adding clean high-score
  qualification, routing final-life game-over through `HighScoreEntry` for
  qualifying scores, preserving non-qualifying `GameOver`, and updating focused
  systems/game/docs coverage.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778851664328309`
- `2026-05-15 15:08:25 BST` Completed `DC-128`: added
  `HighScoreEntrySystem` and routed final-life game-over through clean
  high-score qualification. Qualifying current-player scores now enter
  `HighScoreEntry` and emit `HighScoreEntryStarted`; zero and non-qualifying
  scores remain in `GameOver`, with gameplay sprites absent from terminal
  scenes. Validation passed with `cargo fmt --check`,
  `cargo test --lib systems::tests::high_score`,
  `cargo test --lib game::tests::clean_game_high_score_entry`, full game and
  systems test modules, public API guard, `cargo test --all-targets` (1196 lib,
  2 bin, and 2 example tests), `cargo clippy --all-targets -- -D warnings`,
  `make fidelity` (10 fixtures, 15452 frames, new Rust line coverage 13/13
  after adding second-player qualification coverage), `cargo run -- --live-smoke`
  (239 rendered frames), `markdownlint README.md SPEC.md PLAN.md
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md`, and
  `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778854120229339`

### DC-129: Clean High-Score Initials Entry System

Status: `complete`

Goal: handle high-score initials through clean gameplay state after
`HighScoreEntry` starts.

Scope:

- Add deterministic clean high-score initials state and entry handling in
  `src/systems.rs`.
- Extend clean `GameInput` with typed initials and backspace input.
- Route `HighScoreEntry` frames through the clean initials system.
- Emit `HighScoreInitialAccepted` and `HighScoreSubmitted`, and return to
  attract after submission.
- Update focused systems/game tests plus README/SPEC/PLAN docs.

Acceptance criteria:

- Initials entry is represented by clean gameplay-domain state and events.
- Alphabetic inputs are normalized to uppercase, accepted up to three initials,
  and backspace removes the previous initial without submitting.
- The third accepted initial emits `HighScoreSubmitted` and returns the clean
  game to `Attract`.

Validation:

```sh
cargo fmt --check
cargo test --lib systems::tests::high_score
cargo test --lib game::tests::clean_game_high_score_initials
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 15:10:42 BST` Started `DC-129`: adding clean high-score initials
  state, typed-initial and backspace input handling, accepted/submitted
  gameplay events, attract return after submission, and focused
  systems/game/docs coverage.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778854255315089`
- `2026-05-15 15:37:21 BST` Completed `DC-129`: added clean high-score
  initials state and frame output, routed typed initials and backspace through
  `GameInput` and `Game`, emitted `HighScoreInitialAccepted` and
  `HighScoreSubmitted`, returned submitted entries to attract, and updated
  README/SPEC/PLAN plus compatibility constructors. Validation passed with
  `cargo fmt --check`, `cargo test --lib systems::tests::high_score`,
  `cargo test --lib game::tests::clean_game_high_score_initials`,
  `cargo test --all-targets` (1198 library tests, 2 binary tests, 2 example
  tests), `cargo clippy --all-targets -- -D warnings`, `make fidelity`
  (10 fixtures, 15452 frames, 45/45 new executable Rust lines covered),
  `cargo run -- --live-smoke` (239 rendered frames with attract, credit, and
  playing evidence), `markdownlint README.md SPEC.md PLAN.md
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md`, and
  `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778855841540879`

### DC-130: Clean WGPU Live Runtime Facade

Status: `complete`

Goal: move the clean runtime boundary off direct legacy live presenter and
input-profile imports while preserving current WGPU live behavior.

Scope:

- Add a clean `src/live_wgpu.rs` facade for interactive and smoke WGPU live
  launch.
- Move `src/runtime.rs` to clean live launch/profile/report contracts instead
  of importing the temporary presenter and input-profile modules directly.
- Keep the existing temporary presenter bridge quarantined behind the new
  facade until the full live event loop is clean-owned.
- Update public guard tests plus README/SPEC/PLAN docs.

Acceptance criteria:

- `src/runtime.rs` no longer references the temporary WGPU presenter module or
  low-level input profile module directly.
- The only clean source file allowed to adapt to those live bridge modules is
  `src/live_wgpu.rs`.
- `cargo run -- --live-smoke` behavior and output stay unchanged.

Validation:

```sh
cargo fmt --check
cargo test --lib runtime::tests
cargo test --lib public_api_tests
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 19:53:49 BST` Started `DC-130`: adding a clean WGPU live runtime
  facade, moving runtime off direct temporary presenter/input imports, updating
  guard tests and docs, and preserving current WGPU live smoke behavior.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778871226066989`
- `2026-05-15 20:15:33 BST` Completed `DC-130`: added `src/live_wgpu.rs` as
  the clean WGPU live launch facade, moved `src/runtime.rs` to
  `LiveInputProfile` and `LiveSmokeReport` contracts instead of direct
  temporary presenter/input imports, and updated README/SPEC/PLAN plus public
  quarantine guard tests. Validation passed with `cargo fmt --check`,
  `cargo test --lib live_wgpu::tests`, `cargo test --lib runtime::tests`,
  `cargo test --lib public_api_tests`, `cargo test --all-targets` (1201
  library tests, 2 binary tests, 2 example tests),
  `cargo clippy --all-targets -- -D warnings`, `make fidelity` (10 fixtures,
  15452 frames, 6/6 new executable Rust lines covered),
  `cargo run -- --live-smoke` (239 rendered frames with attract, credit, and
  playing evidence), `markdownlint README.md SPEC.md PLAN.md
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md`, and
  `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778872547604119`

### DC-131: Clean ROM Verification Facade

Status: `complete`

Goal: move optional ROM listing, scan, and verification command code off direct
legacy ROM module types while preserving current CLI behavior.

Scope:

- Add a clean `src/roms.rs` facade for optional ROM descriptors, scan reports,
  and verification summaries.
- Move `src/rom_report.rs` to clean ROM facade contracts instead of importing
  legacy ROM types/functions directly.
- Keep `--rom-report` and `--verify-roms` text and error behavior unchanged.
- Update public guard tests plus README/SPEC/PLAN docs.

Acceptance criteria:

- `src/rom_report.rs` no longer references the legacy ROM module directly.
- The only clean source file allowed to adapt to the legacy ROM module is
  `src/roms.rs`.
- Existing ROM listing, report, and verification tests pass unchanged in
  behavior.

Validation:

```sh
cargo fmt --check
cargo test --lib roms::tests
cargo test --lib rom_report::tests
cargo test --lib public_api_tests
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 20:17:21 BST` Started `DC-131`: adding a clean ROM verification
  facade, moving `rom_report` off direct legacy ROM imports, updating guard
  tests and docs, and preserving optional ROM command output.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778872635858059`
- `2026-05-15 20:58:43 BST` Completed `DC-131`: added `src/roms.rs` as the
  clean optional ROM verification facade, moved `src/rom_report.rs` off direct
  legacy ROM imports, updated guard tests and README/SPEC module docs, and
  preserved the current `--rom-report`/`--verify-roms` behavior. Validation
  passed with focused ROM facade/report/public API tests, `cargo test
  --all-targets` (1205 library tests, 2 binary tests, 2 example tests),
  `cargo clippy --all-targets -- -D warnings`, `make fidelity` (10 fixtures,
  15452 frames, and 6/6 new executable Rust lines covered), `cargo run --
  --live-smoke` (239 rendered frames), markdownlint, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778875121767719`

### DC-132: Clean Fidelity Scenario Manifest Facade

Status: `complete`

Goal: move fidelity scenario listing and input-script writing off direct
legacy trace scenario types while preserving current CLI behavior.

Scope:

- Add a clean crate-private fidelity scenario manifest facade for scenario
  descriptors and expanded input text.
- Move `src/fidelity_scenarios.rs` to clean manifest contracts instead of
  importing legacy trace scenario types/functions directly.
- Keep `--fidelity-list-scenarios` and `--fidelity-write-scenario-inputs`
  output and file behavior unchanged.
- Update public guard tests plus README/SPEC/PLAN docs.

Acceptance criteria:

- `src/fidelity_scenarios.rs` no longer references `crate::legacy_fidelity`
  directly.
- The only clean source files allowed to adapt to legacy fidelity trace
  generation are the dedicated fidelity facade modules.
- Existing scenario listing and input-writing tests pass unchanged in behavior.

Validation:

```sh
cargo fmt --check
cargo test --lib fidelity_manifest::tests
cargo test --lib fidelity_scenarios::tests
cargo test --lib public_api_tests
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 21:01:36 BST` Started `DC-132`: adding a clean fidelity
  scenario manifest facade, moving `fidelity_scenarios` off direct legacy trace
  scenario imports, updating guard tests and docs, and preserving current
  scenario listing/input-writing behavior.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778875284634339`
- `2026-05-15 21:23:09 BST` Completed `DC-132`: added
  `src/fidelity_manifest.rs` as the clean scenario manifest/input expansion
  facade, moved `src/fidelity_scenarios.rs` off direct legacy fidelity imports,
  updated public guard tests and README/SPEC module docs, and preserved the
  current scenario listing/input-writing behavior. Validation passed with
  focused manifest/scenario/public API tests, `cargo test --all-targets` (1207
  library tests, 2 binary tests, 2 example tests), `cargo clippy --all-targets
  -- -D warnings`, `make fidelity` (10 fixtures, 15452 frames, and 6/6 new
  executable Rust lines covered), `cargo run -- --live-smoke` (239 rendered
  frames), markdownlint, `cargo fmt --check`, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778876604034919`

### DC-133: Clean Fidelity Trace Engine Facade

Status: `complete`

Goal: move fidelity trace command orchestration off direct legacy trace engine
calls while preserving current CLI behavior.

Scope:

- Add a clean crate-private fidelity trace engine facade for trace generation,
  trace comparison, and trace schema header access.
- Move `src/fidelity_traces.rs` to clean trace-engine and manifest contracts
  instead of importing legacy trace functions directly.
- Keep `--fidelity-trace`, `--fidelity-trace-inputs`,
  `--fidelity-check-trace`, fixture-directory checks, and reference fixture
  validation behavior unchanged.
- Update public guard tests plus README/SPEC/PLAN docs.

Acceptance criteria:

- `src/fidelity_traces.rs` no longer references `crate::legacy_fidelity`
  directly.
- Legacy trace generation and comparison access is isolated in
  `src/fidelity_trace_engine.rs`.
- Existing trace command, fixture, and reference validation tests pass
  unchanged in behavior.

Validation:

```sh
cargo fmt --check
cargo test --lib fidelity_trace_engine::tests
cargo test --lib fidelity_traces::tests
cargo test --lib public_api_tests
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 21:25:33 BST` Started `DC-133`: adding a clean fidelity trace
  engine facade, moving `fidelity_traces` off direct legacy trace generation
  and comparison imports, updating guard tests and docs, and preserving current
  trace CLI behavior.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778876729244129`
- `2026-05-15 21:47:47 BST` Completed `DC-133`: added
  `src/fidelity_trace_engine.rs`, moved `src/fidelity_traces.rs` to the clean
  manifest and trace-engine contracts, refreshed public guard tests plus
  README/SPEC/PLAN module docs, and preserved current trace command behavior.
  Validation passed: `cargo fmt --check`,
  `cargo test --lib fidelity_trace_engine::tests`,
  `cargo test --lib fidelity_traces::tests`, `cargo test --lib public_api_tests`,
  `cargo test --all-targets` (1211 library tests, 2 binary tests, and 2 example
  tests), `cargo clippy --all-targets -- -D warnings`, `make fidelity` (10
  fixture traces, 15452 frames, and 16/16 new executable Rust lines covered),
  `cargo run -- --live-smoke` (239 rendered frames), markdownlint, and
  `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778878068481899`

### DC-134: Clean Game Smoke Command

Status: `complete`

Goal: add a clean smoke command that exercises `Game` and native renderer draw
planning without entering the legacy live presenter path.

Scope:

- Add a crate-private clean game smoke runner that steps `Game` through a
  deterministic input script.
- Prepare each emitted `RenderScene` with `NativeSceneRenderer` and summarize
  sprite-only draw-plan evidence.
- Add a `--game-smoke` CLI/runtime command that prints a stable smoke report.
- Update public guard tests plus README/SPEC/PLAN docs.

Acceptance criteria:

- `--game-smoke` runs through the clean game and native renderer draw-planning
  path without importing legacy live or presenter modules.
- The report proves attract, credited, and playing frames, nonzero sprite
  instances, no raster payloads, no missing sprite atlas regions, and a clean
  exit.
- Existing `--live-smoke` behavior remains unchanged.

Validation:

```sh
cargo fmt --check
cargo test --lib game_smoke::tests
cargo test --lib runtime::tests
cargo test --lib platform::tests
cargo test --lib public_api_tests
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --game-smoke
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 21:54:47 BST` Started `DC-134`: adding a clean `--game-smoke`
  command that steps `Game` through scripted controls, prepares emitted scenes
  with `NativeSceneRenderer`, reports sprite-only draw-plan evidence, and
  preserves existing `--live-smoke` behavior.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778878489373369`
- `2026-05-15 22:20:49 BST` Completed `DC-134`: added the clean
  `--game-smoke` command, crate-private smoke runner, runtime/platform wiring,
  public guard checks, and README/SPEC documentation. The smoke report proves
  24 clean game frames, attract/credited/playing coverage, 290 sprite
  instances, 92 sprite draw commands, zero raster frames, zero missing sprite
  atlas regions, and clean exit. Validation passed with `cargo fmt --check`,
  `cargo test --lib game_smoke::tests`, `cargo test --lib runtime::tests`,
  `cargo test --lib platform::tests`, `cargo test --lib public_api_tests`,
  `cargo test --all-targets` (1221 library tests, 2 binary tests, and
  2 example tests), `cargo clippy --all-targets -- -D warnings`,
  `make fidelity` (10 fixture traces, 15452 frames, and 16/16 new executable
  Rust lines covered), `cargo run -- --game-smoke`,
  `cargo run -- --live-smoke` (240 rendered frames), markdownlint, and
  `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778880047960349`

### DC-135: Clean Game Smoke WGPU Frame-Plan Evidence

Status: `complete`

Goal: extend the clean game smoke so it proves frame-level `wgpu` command
planning, not only scene sprite and draw-plan counts.

Scope:

- Add `WgpuFramePlan` evidence to the clean game smoke report.
- Validate sprite render-pass command coverage for the clean smoke frames.
- Validate that the clean smoke path emits no temporary raster frame commands.
- Keep the existing `--game-smoke` and `--live-smoke` behavior otherwise
  unchanged.

Acceptance criteria:

- `--game-smoke` reports nonzero frame commands and sprite render-pass command
  coverage.
- `--game-smoke` fails if a temporary raster frame command appears in the clean
  smoke path.
- Focused smoke and public guard tests cover the updated behavior.

Validation:

```sh
cargo fmt --check
cargo test --lib game_smoke::tests
cargo test --lib public_api_tests
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --game-smoke
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 22:23:00 BST` Started `DC-135`: extending `--game-smoke` so it
  verifies frame-level `wgpu` command plans, reports sprite render-pass command
  coverage, and keeps temporary raster commands out of the clean smoke path.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778880191445779`
- `2026-05-15 23:01:48 BST` Completed `DC-135`: extended `--game-smoke` with
  `wgpu` frame-command evidence, sprite render-pass command coverage, and
  temporary raster command validation. The smoke report now proves 96 frame
  commands, 24 sprite render-pass commands, zero temporary raster commands,
  290 sprite instances, and 92 sprite draw commands across 24 clean frames.
  Validation passed with `cargo fmt --check`,
  `cargo test --lib game_smoke::tests`, `cargo test --lib public_api_tests`,
  `cargo test --all-targets` (1222 library tests, 2 binary tests, and
  2 example tests), `cargo clippy --all-targets -- -D warnings`,
  `make fidelity` (10 fixture traces, 15452 frames, and 15/15 new executable
  Rust lines covered), `cargo run -- --game-smoke`,
  `cargo run -- --live-smoke` (240 rendered frames), markdownlint, and
  `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778882543476649`

### DC-136: Clean Game Smoke GPU Resource Evidence

Status: `complete`

Goal: extend the clean game smoke so it proves the sprite `wgpu` resource and
encoder plans behind the frame commands.

Scope:

- Add resource-binding, pipeline-layout, render-pipeline descriptor, and
  render-pass encoder evidence to the clean game smoke report.
- Report upload byte evidence for sprite instances, sprite atlas texture data,
  and scene-projection uniforms.
- Validate that every clean smoke frame produces sprite resource and encoder
  plans without falling back to temporary raster commands.
- Keep `--game-smoke` and `--live-smoke` behavior otherwise unchanged.

Acceptance criteria:

- `--game-smoke` reports sprite resource-binding frames, pipeline-layout
  frames, render-pipeline descriptor frames, encoder frames, encoder commands,
  encoder draws, and upload byte totals.
- The clean smoke fails if any required sprite resource or encoder evidence is
  missing.
- Focused smoke and public guard tests cover the updated behavior.

Validation:

```sh
cargo fmt --check
cargo test --lib game_smoke::tests
cargo test --lib public_api_tests
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --game-smoke
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 23:04:24 BST` Started `DC-136`: extending `--game-smoke` beyond
  frame-command counts into sprite resource bindings, pipeline layout, render
  pipeline descriptor, render-pass encoder commands/draws, and upload byte
  evidence for the clean `wgpu` sprite path.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778882675313219`
- `2026-05-15 23:26:20 BST` Completed `DC-136`: extended `--game-smoke` with
  clean sprite resource-binding, pipeline-layout, render-pipeline descriptor,
  render-pass encoder, and upload byte evidence. The smoke report now proves
  24 resource-binding, pipeline-layout, render-pipeline descriptor, and
  render-pass encoder frames; 236 sprite encoder commands; 92 encoder draws
  matching 92 sprite draw commands; 13920 sprite instance upload bytes;
  1572864 atlas upload bytes; 384 scene-projection upload bytes; and zero
  temporary raster commands. Validation passed with `cargo fmt --check`,
  `cargo test --lib game_smoke::tests`, `cargo test --lib public_api_tests`,
  `cargo test --all-targets` (1224 library tests, 2 binary tests, and
  2 example tests), `cargo clippy --all-targets -- -D warnings`,
  `make fidelity` (10 fixture traces, 15452 frames, and 49/49 new executable
  Rust lines covered), `cargo run -- --game-smoke`,
  `cargo run -- --live-smoke` (239 rendered frames), markdownlint, and
  `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778884008047749`

### DC-137: Clean Game Smoke Sprite Coverage Evidence

Status: `complete`

Goal: extend the clean game smoke so it proves the scene path covers the
gameplay sprite categories required by the sprite-first rewrite.

Scope:

- Add layer coverage evidence for terrain, starfield, objects, projectiles, and
  HUD sprites to the clean game smoke report.
- Add sprite-ID coverage evidence for player, lander, human, projectile,
  terrain, star, and score text sprites.
- Validate that the clean smoke fails when required gameplay sprite coverage is
  missing.
- Keep `--game-smoke` and `--live-smoke` behavior otherwise unchanged.

Acceptance criteria:

- `--game-smoke` reports nonzero layer counts for terrain, starfield, objects,
  projectiles, and HUD.
- `--game-smoke` reports all required clean gameplay sprite IDs.
- Focused smoke and public guard tests cover the updated behavior.

Validation:

```sh
cargo fmt --check
cargo test --lib game_smoke::tests
cargo test --lib public_api_tests
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --game-smoke
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-15 23:28:32 BST` Started `DC-137`: extending `--game-smoke` so it
  verifies gameplay sprite coverage across terrain, starfield, object,
  projectile, and HUD layers plus required sprite IDs for player, lander,
  human, projectile, terrain, star, and score text.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778884124624519`
- `2026-05-16 00:08:13 BST` Completed `DC-137`: extended `--game-smoke` with
  gameplay sprite coverage evidence from clean `RenderScene` data. The smoke
  report now proves 105 terrain sprites, 63 starfield sprites, 93 object
  sprites, 5 projectile sprites, 24 HUD sprites, and required sprite IDs for
  score text, stars, terrain, enemy landers, humans, the player ship, and
  player projectiles. Validation passed with `cargo fmt --check`,
  `cargo test --lib game_smoke::tests`, `cargo test --lib public_api_tests`,
  `cargo test --all-targets` (1226 library tests, 2 binary tests, and
  2 example tests), `cargo clippy --all-targets -- -D warnings`,
  `make fidelity` (10 fixture traces, 15452 frames, and 42/42 new executable
  Rust lines covered), `cargo run -- --game-smoke`,
  `cargo run -- --live-smoke` (239 rendered frames), markdownlint, and
  `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778886613013399`

### DC-138: Clean Game Smoke Sprite Pipeline Coverage Evidence

Status: `complete`

Goal: extend the clean game smoke so it proves required gameplay sprites reach
the expected native `wgpu` draw pipelines, not only clean scene layers.

Scope:

- Add draw-command layer coverage evidence for terrain, starfield, objects,
  projectiles, and HUD sprites to the clean game smoke report.
- Add native pipeline coverage evidence for terrain, starfield, sprites,
  projectiles, and HUD text draw commands.
- Validate that the clean smoke fails when required draw-command or pipeline
  coverage is missing.
- Keep gameplay behavior, `--game-smoke`, and `--live-smoke` behavior
  otherwise unchanged.

Acceptance criteria:

- `--game-smoke` reports nonzero draw-command counts for terrain, starfield,
  object, projectile, and HUD layers.
- `--game-smoke` reports all required clean native sprite pipelines.
- Focused smoke and public guard tests cover the updated behavior.

Validation:

```sh
cargo fmt --check
cargo test --lib game_smoke::tests
cargo test --lib public_api_tests
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --game-smoke
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-16 00:12:06 BST` Started `DC-138`: extending `--game-smoke` so it
  verifies native draw-command coverage for terrain, starfield, object,
  projectile, and HUD layers plus required clean sprite pipelines for terrain,
  starfield, sprites, projectiles, and HUD text.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778886737931019`
- `2026-05-16 00:50:25 BST` Completed `DC-138`: extended `--game-smoke` with
  native draw-command and pipeline coverage evidence from clean `SceneDrawPlan`
  data. The smoke report now proves 21 terrain, 21 starfield, 21 object,
  5 projectile, and 24 HUD draw commands while covering `hud_text`,
  `starfield`, `terrain`, `sprites`, and `projectiles` pipelines. Validation
  passed with `cargo fmt --check`, `cargo test --lib game_smoke::tests`,
  `cargo test --lib public_api_tests`, `cargo test --all-targets` (1228
  library tests before the final coverage-only branch test, 2 binary tests, and
  2 example tests), `cargo clippy --all-targets -- -D warnings`,
  `make fidelity` (1229 library tests, 2 binary tests, 2 example tests,
  10 fixture traces, 15452 frames, and 37/37 new executable Rust lines
  covered), `cargo run -- --game-smoke`, `cargo run -- --live-smoke`
  (240 rendered frames), markdownlint, and `git diff --check`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778889040826379`

### DC-139: Clean Game Smoke Sprite Draw-Instance Coverage Evidence

Status: `complete`

Goal: extend the clean game smoke so it proves clean sprite instances are
actually covered by native `wgpu` draw commands, not only that each layer has a
draw command.

Scope:

- Add draw-instance coverage evidence for terrain, starfield, objects,
  projectiles, and HUD sprites to the clean game smoke report.
- Validate that drawn sprite instance totals match clean sprite instance totals.
- Validate that per-layer drawn sprite instance totals match the clean scene
  sprite layer totals.
- Keep gameplay behavior, `--game-smoke`, and `--live-smoke` behavior
  otherwise unchanged.

Acceptance criteria:

- `--game-smoke` reports drawn sprite instance totals for terrain, starfield,
  object, projectile, and HUD layers.
- `--game-smoke` fails if any clean sprite instances are not represented by
  native draw commands.
- Focused smoke and public guard tests cover the updated behavior.

Validation:

```sh
cargo fmt --check
cargo test --lib game_smoke::tests
cargo test --lib public_api_tests
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --game-smoke
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-16 00:52:22 BST` Started `DC-139`: extending `--game-smoke` so it
  verifies drawn sprite instance coverage for terrain, starfield, object,
  projectile, and HUD layers, and so total drawn instances must match clean
  sprite instances.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778889152440539`
- `2026-05-16 01:15:07 BST` Completed `DC-139`: `--game-smoke` now reports
  290 drawn sprite instances matching 290 clean sprite instances, with terrain
  105, starfield 63, object 93, projectile 5, and HUD 24 draw instances. Added
  mismatch validation, focused smoke regression tests, and a public guard that
  requires the smoke path to consume native draw-command instance counts.
  Validation: `cargo fmt --check`, `cargo test --lib game_smoke::tests`,
  `cargo test --lib public_api_tests`, `cargo test --all-targets`,
  `cargo clippy --all-targets -- -D warnings`, `cargo run -- --game-smoke`,
  `cargo run -- --live-smoke` (239 rendered frames), markdownlint,
  `git diff --check`, and `make fidelity` (42/42 non-baselined added
  executable Rust lines covered).
  Slack in-flight update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778889599148589`
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778890523940999`

### DC-140: Clean Game Smoke Sprite Upload-Instance Coverage Evidence

Status: `complete`

Goal: extend the clean game smoke so it proves the native sprite instance
upload contains the same clean sprite instances that are represented by native
draw commands.

Scope:

- Add sprite instance upload record-count evidence to the clean game smoke
  report.
- Validate that uploaded sprite instance records match clean sprite instance
  totals.
- Validate that uploaded sprite instance records match the drawn sprite
  instance totals.
- Keep gameplay behavior, `--game-smoke`, and `--live-smoke` behavior
  otherwise unchanged.

Acceptance criteria:

- `--game-smoke` reports uploaded sprite instance records.
- `--game-smoke` fails if uploaded sprite instance records diverge from clean
  sprite instances or native draw-command instance totals.
- Focused smoke and public guard tests cover the updated behavior.

Validation:

```sh
cargo fmt --check
cargo test --lib game_smoke::tests
cargo test --lib public_api_tests
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --game-smoke
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-16 01:17:28 BST` Started `DC-140`: extending `--game-smoke` so it
  reports uploaded sprite instance records and validates that the native
  instance upload matches both clean sprite instances and drawn sprite
  instances.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778890666333329`
- `2026-05-16 01:39:28 BST` Completed `DC-140`: `--game-smoke` now reports
  `sprite_instance_upload_records`; validation fails if uploaded sprite
  records diverge from native draw-command instance totals; the current smoke
  run reports `sprite_instances: 290`, `drawn_sprite_instances: 290`,
  `sprite_instance_upload_records: 290`, and
  `sprite_instance_upload_bytes: 13920`. Validation passed with
  `cargo fmt --check`, `cargo test --lib game_smoke::tests`,
  `cargo test --lib public_api_tests`, `cargo test --all-targets`,
  `cargo clippy --all-targets -- -D warnings`, `cargo run -- --game-smoke`,
  `cargo run -- --live-smoke`, `markdownlint README.md SPEC.md PLAN.md
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md`,
  `git diff --check`, and `make fidelity` with `new Rust line coverage: 5/5
  non-baselined added executable line(s)`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778891966098949`

### DC-141: Clean Game Smoke Sprite Buffer Upload-Plan Evidence

Status: `complete`

Goal: extend the clean game smoke so it proves renderer-owned sprite buffer
upload plans are present and aligned with the flattened sprite instance upload
stream.

Scope:

- Add sprite buffer upload-plan frame and byte-count evidence to the clean game
  smoke report.
- Report quad vertex, quad index, and instance buffer upload bytes.
- Validate that sprite buffer upload plans exist for every clean smoke frame.
- Validate that sprite buffer instance upload bytes match flattened sprite
  instance upload bytes.
- Keep gameplay behavior, `--game-smoke`, and `--live-smoke` behavior
  otherwise unchanged.

Acceptance criteria:

- `--game-smoke` reports sprite buffer upload-plan frames and buffer byte
  totals.
- `--game-smoke` fails if sprite buffer upload plans are missing from any
  frame.
- `--game-smoke` fails if sprite buffer instance bytes diverge from flattened
  instance upload bytes.
- Focused smoke and public guard tests cover the updated behavior.

Validation:

```sh
cargo fmt --check
cargo test --lib game_smoke::tests
cargo test --lib public_api_tests
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --game-smoke
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-16 01:41:10 BST` Started `DC-141`: extending `--game-smoke` so it
  reports sprite buffer upload-plan frames, quad vertex/index upload bytes, and
  instance buffer upload bytes, then validates those uploads against the
  flattened sprite instance upload stream.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778892078200859`
- `2026-05-16 02:02:20 BST` Completed `DC-141`: `--game-smoke` now reports
  sprite buffer upload-plan frames plus quad vertex, quad index, and instance
  buffer upload bytes. The current smoke run reports
  `sprite_buffer_upload_frames: 24`, `sprite_quad_vertex_upload_bytes: 1536`,
  `sprite_quad_index_upload_bytes: 288`,
  `sprite_buffer_instance_upload_bytes: 13920`, and
  `sprite_instance_upload_bytes: 13920`. Validation passed with
  `cargo fmt --check`, `cargo test --lib game_smoke::tests`,
  `cargo test --lib public_api_tests`, `cargo test --all-targets`,
  `cargo clippy --all-targets -- -D warnings`, `cargo run -- --game-smoke`,
  `cargo run -- --live-smoke`, `markdownlint README.md SPEC.md PLAN.md
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md`,
  `git diff --check`, and `make fidelity` with `new Rust line coverage: 21/21
  non-baselined added executable line(s)`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778893353769969`

### DC-142: Clean Game Smoke Sprite Render-Pass Plan Evidence

Status: `complete`

Goal: extend the clean game smoke so it proves renderer-owned sprite
render-pass plans are present and aligned with native sprite draw commands.

Scope:

- Add sprite render-pass plan frame, draw, and instance evidence to the clean
  game smoke report.
- Validate that sprite render-pass plans exist for every clean smoke frame.
- Validate that sprite render-pass draw counts match native sprite draw
  command totals.
- Validate that sprite render-pass instance counts match native drawn sprite
  instance totals.
- Keep gameplay behavior, `--game-smoke`, and `--live-smoke` behavior
  otherwise unchanged.

Acceptance criteria:

- `--game-smoke` reports sprite render-pass plan frames, draws, and instances.
- `--game-smoke` fails if sprite render-pass plans are missing from any frame.
- `--game-smoke` fails if render-pass draw or instance totals diverge from
  native draw-command evidence.
- Focused smoke and public guard tests cover the updated behavior.

Validation:

```sh
cargo fmt --check
cargo test --lib game_smoke::tests
cargo test --lib public_api_tests
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --game-smoke
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-16 02:03:55 BST` Started `DC-142`: extending `--game-smoke` so it
  reports sprite render-pass plan frames, draw counts, and instance counts,
  then validates those values against native sprite draw commands and drawn
  sprite instances.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778893444143389`
- `2026-05-16 02:25:23 BST` Completed `DC-142`: `--game-smoke` now reports
  `sprite_render_pass_plan_frames: 24`,
  `sprite_render_pass_plan_draws: 92`, and
  `sprite_render_pass_plan_instances: 290`, with render-pass draw and instance
  totals validated against `sprite_draw_commands: 92` and
  `drawn_sprite_instances: 290`. Validation passed with
  `cargo fmt --check`, `cargo test --lib game_smoke::tests`,
  `cargo test --lib public_api_tests`, `cargo run -- --game-smoke`,
  `markdownlint README.md SPEC.md PLAN.md
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md`,
  `git diff --check`, `cargo run -- --live-smoke`,
  `cargo test --all-targets`, `cargo clippy --all-targets -- -D warnings`,
  and `make fidelity`; coverage artifacts were refreshed at
  `2026-05-16 02:24 BST`, and
  `python3 tools/check_new_rust_coverage.py --lcov target/coverage/lcov.info
  --base HEAD --uncovered-baseline tools/new_rust_coverage_baseline.txt`
  reported `new Rust line coverage: 15/15 non-baselined added executable
  line(s)`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778894748207669`

### DC-143: Clean Game Smoke WGPU Frame Sprite Instance Evidence

Status: `complete`

Goal: extend the frame-level `wgpu` command plan so clean smoke proves sprite
execution carries both draw and instance totals through to the final frame
command stream.

Scope:

- Add sprite instance totals to the frame-level `ExecuteSpriteRenderPass`
  command plan.
- Expose frame-plan sprite draw and instance totals through `--game-smoke`.
- Validate that frame-plan sprite draw totals match native sprite draw
  commands.
- Validate that frame-plan sprite instance totals match native drawn sprite
  instances.
- Keep gameplay behavior, `--game-smoke`, and `--live-smoke` behavior
  otherwise unchanged.

Acceptance criteria:

- `WgpuFramePlan` records sprite pass draw and instance totals.
- `--game-smoke` reports frame-plan sprite draws and instances.
- `--game-smoke` fails if frame-level sprite draw or instance totals diverge
  from native draw-command evidence.
- Focused renderer, smoke, and public guard tests cover the updated behavior.

Validation:

```sh
cargo fmt --check
cargo test --lib renderer::tests
cargo test --lib game_smoke::tests
cargo test --lib public_api_tests
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --game-smoke
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-16 02:27:56 BST` Started `DC-143`: extending the final frame-level
  `wgpu` sprite execution command so it carries sprite instance totals as well
  as draw totals, then validating those totals in `--game-smoke`.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778894887533839`
- `2026-05-16 02:50:24 BST` Completed `DC-143`: `ExecuteSpriteRenderPass`
  now carries sprite instance totals alongside command and draw counts,
  `WgpuFramePlan` exposes aggregate sprite draw and instance totals, and
  `--game-smoke` now reports `sprite_frame_plan_draws: 92` plus
  `sprite_frame_plan_instances: 290`, matching `sprite_draw_commands: 92` and
  `drawn_sprite_instances: 290`. Validation passed with
  `cargo fmt --check`, `cargo test --lib renderer::tests`,
  `cargo test --lib game_smoke::tests`, `cargo test --lib public_api_tests`,
  `cargo run -- --game-smoke`, `markdownlint README.md SPEC.md PLAN.md
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md`,
  `git diff --check`, `cargo test --all-targets`,
  `cargo clippy --all-targets -- -D warnings`, `cargo run -- --live-smoke`,
  and `make fidelity`; coverage artifacts were refreshed at
  `2026-05-16 02:50 BST`, and the coverage checker reported
  `new Rust line coverage: 33/33 non-baselined added executable line(s)`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778896239989579`

### DC-144: Clean Game Smoke WGPU Frame Sprite Encoder Command Evidence

Status: `complete`

Goal: extend the final frame-level `wgpu` command evidence so clean smoke proves
sprite render-pass encoder command totals survive into the ordered frame
command stream.

Scope:

- Add aggregate sprite encoder command totals to `WgpuFramePlan`.
- Expose frame-plan sprite encoder command totals through `--game-smoke`.
- Validate that frame-plan sprite encoder command totals match sprite
  render-pass encoder evidence.
- Keep the existing frame-plan sprite draw and instance evidence intact.
- Keep gameplay behavior, `--game-smoke`, and `--live-smoke` behavior otherwise
  unchanged.

Acceptance criteria:

- `WgpuFramePlan` reports aggregate sprite encoder command totals.
- `--game-smoke` reports frame-plan sprite encoder command totals.
- `--game-smoke` fails if frame-level sprite encoder command totals diverge
  from native encoder-plan evidence.
- Focused renderer, smoke, and public guard tests cover the updated behavior.

Validation:

```sh
cargo fmt --check
cargo test --lib renderer::tests
cargo test --lib game_smoke::tests
cargo test --lib public_api_tests
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --game-smoke
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-16 02:54:14 BST` Started `DC-144`: extending the final frame-level
  `wgpu` sprite execution evidence so it reports sprite encoder command totals
  alongside draw and instance totals, then validating those totals in
  `--game-smoke`.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778896420884149`
- `2026-05-16 03:16:02 BST` Completed `DC-144`: `WgpuFramePlan` now exposes
  aggregate sprite encoder command totals from `ExecuteSpriteRenderPass`, and
  `--game-smoke` now reports `sprite_frame_plan_encoder_commands: 236`,
  matching `sprite_encoder_commands: 236`, alongside
  `sprite_frame_plan_draws: 92` and `sprite_frame_plan_instances: 290`.
  Validation passed with `cargo fmt --check`,
  `cargo test --lib renderer::tests`, `cargo test --lib game_smoke::tests`,
  `cargo test --lib public_api_tests`, `cargo run -- --game-smoke`,
  `markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md
  docs/fidelity/live-audio.md`, `git diff --check`,
  `cargo test --all-targets`, `cargo clippy --all-targets -- -D warnings`,
  `cargo run -- --live-smoke`, and `make fidelity`; coverage artifacts were
  refreshed at `2026-05-16 03:15 BST`, and the coverage checker reported
  `new Rust line coverage: 12/12 non-baselined added executable line(s)`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778897775341499`

### DC-145: Clean Game Smoke WGPU Frame Projection Upload Evidence

Status: `complete`

Goal: extend the final frame-level `wgpu` command evidence so clean smoke proves
scene-projection upload bytes survive into the ordered frame command stream.

Scope:

- Add aggregate scene-projection upload byte totals to `WgpuFramePlan`.
- Expose frame-plan scene-projection upload bytes through `--game-smoke`.
- Validate that frame-plan projection bytes match sprite resource-binding
  projection upload evidence.
- Keep the existing frame-plan sprite command, draw, and instance evidence
  intact.
- Keep gameplay behavior, `--game-smoke`, and `--live-smoke` behavior otherwise
  unchanged.

Acceptance criteria:

- `WgpuFramePlan` reports aggregate scene-projection upload bytes.
- `--game-smoke` reports frame-plan scene-projection upload bytes.
- `--game-smoke` fails if frame-level projection bytes diverge from resource
  binding projection upload evidence.
- Focused renderer, smoke, and public guard tests cover the updated behavior.

Validation:

```sh
cargo fmt --check
cargo test --lib renderer::tests
cargo test --lib game_smoke::tests
cargo test --lib public_api_tests
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --game-smoke
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-16 03:17:39 BST` Started `DC-145`: extending final frame-level
  `wgpu` command evidence so it reports scene-projection upload bytes and
  validating those bytes against resource-binding projection upload evidence
  in `--game-smoke`.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778897895059149`
- `2026-05-16 03:38:58 BST` Completed `DC-145`: `WgpuFramePlan` now exposes
  aggregate scene-projection upload bytes from ordered
  `UploadSceneProjection` frame commands, and `--game-smoke` reports
  `frame_plan_scene_projection_upload_bytes: 384`, matching
  `scene_projection_upload_bytes: 384`. Validation now fails if the frame-plan
  projection upload total diverges from the resource-binding projection upload
  evidence. Validation passed with `cargo fmt --check`,
  `cargo test --lib renderer::tests`, `cargo test --lib game_smoke::tests`,
  `cargo test --lib public_api_tests`, `cargo run -- --game-smoke`,
  `markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md
  docs/fidelity/live-audio.md`, `git diff --check`,
  `cargo test --all-targets`, `cargo clippy --all-targets -- -D warnings`,
  `cargo run -- --live-smoke`, and `make fidelity`; coverage artifacts were
  refreshed at `2026-05-16 03:38 BST`, and the coverage checker reported
  `new Rust line coverage: 13/13 non-baselined added executable line(s)`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778899153140279`

### DC-146: Clean Game Smoke WGPU Frame Viewport Command Evidence

Status: `complete`

Goal: extend the final frame-level `wgpu` command evidence so clean smoke
proves viewport setup reaches the ordered frame command stream for every clean
frame.

Scope:

- Add aggregate viewport command totals to `WgpuFramePlan`.
- Expose frame-plan viewport command totals through `--game-smoke`.
- Validate that every clean smoke frame carries exactly one viewport command in
  the ordered frame command stream.
- Keep the existing frame-plan projection, sprite command, draw, and instance
  evidence intact.
- Keep gameplay behavior, `--game-smoke`, and `--live-smoke` behavior otherwise
  unchanged.

Acceptance criteria:

- `WgpuFramePlan` reports aggregate viewport command totals.
- `--game-smoke` reports frame-plan viewport command totals.
- `--game-smoke` fails if frame-level viewport command totals diverge from the
  number of clean smoke frames.
- Focused renderer, smoke, and public guard tests cover the updated behavior.

Validation:

```sh
cargo fmt --check
cargo test --lib renderer::tests
cargo test --lib game_smoke::tests
cargo test --lib public_api_tests
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --game-smoke
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-16 03:41:40 BST` Started `DC-146`: extending final frame-level
  `wgpu` command evidence so it reports viewport command totals and validates
  that every clean smoke frame carries viewport setup in the ordered frame
  command stream.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778899298772809`
- `2026-05-16 04:02:42 BST` Completed `DC-146`: `WgpuFramePlan` now exposes
  aggregate viewport command totals from ordered `SetViewport` frame commands,
  and `--game-smoke` reports `frame_plan_viewport_commands: 24`, matching the
  24 clean smoke frames. Validation now fails if frame-plan viewport command
  totals diverge from the frame count. Validation passed with
  `cargo fmt --check`, `cargo test --lib renderer::tests`,
  `cargo test --lib game_smoke::tests`, `cargo test --lib public_api_tests`,
  `cargo run -- --game-smoke`, `markdownlint README.md SPEC.md PLAN.md
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md`,
  `git diff --check`, `cargo test --all-targets`,
  `cargo clippy --all-targets -- -D warnings`, `cargo run -- --live-smoke`,
  and `make fidelity`; coverage artifacts were refreshed at
  `2026-05-16 04:02 BST`, and the coverage checker reported
  `new Rust line coverage: 10/10 non-baselined added executable line(s)`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778900577365789`

### DC-147: Clean Game Smoke WGPU Frame Begin-Pass Command Evidence

Status: `complete`

Goal: extend the final frame-level `wgpu` command evidence so clean smoke
proves each clean frame begins an ordered render pass before viewport,
projection, and sprite execution commands.

Scope:

- Add aggregate begin-render-pass command totals to `WgpuFramePlan`.
- Expose frame-plan begin-pass command totals through `--game-smoke`.
- Validate that every clean smoke frame carries exactly one begin-pass command
  in the ordered frame command stream.
- Keep the existing frame-plan viewport, projection, sprite command, draw, and
  instance evidence intact.
- Keep gameplay behavior, `--game-smoke`, and `--live-smoke` behavior otherwise
  unchanged.

Acceptance criteria:

- `WgpuFramePlan` reports aggregate begin-render-pass command totals.
- `--game-smoke` reports frame-plan begin-pass command totals.
- `--game-smoke` fails if frame-level begin-pass command totals diverge from
  the number of clean smoke frames.
- Focused renderer, smoke, and public guard tests cover the updated behavior.

Validation:

```sh
cargo fmt --check
cargo test --lib renderer::tests
cargo test --lib game_smoke::tests
cargo test --lib public_api_tests
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --game-smoke
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-16 04:04:56 BST` Started `DC-147`: extending final frame-level
  `wgpu` command evidence so it reports begin-pass command totals and
  validates that every clean smoke frame starts the ordered frame command
  stream with a render-pass begin command.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778900706383839`
- `2026-05-16 04:26:40 BST` Completed `DC-147`: `WgpuFramePlan` now exposes
  aggregate begin-render-pass command totals from ordered `BeginRenderPass`
  frame commands, and `--game-smoke` reports
  `frame_plan_begin_render_pass_commands: 24`, matching the 24 clean smoke
  frames. Validation now fails if frame-plan begin-pass command totals diverge
  from the frame count. The smoke evidence also preserved
  `frame_plan_viewport_commands: 24`,
  `frame_plan_scene_projection_upload_bytes: 384`,
  `sprite_frame_plan_encoder_commands: 236`,
  `sprite_frame_plan_draws: 92`, and `sprite_frame_plan_instances: 290`.
  Validation passed with `cargo fmt --check`,
  `cargo test --lib renderer::tests`, `cargo test --lib game_smoke::tests`,
  `cargo test --lib public_api_tests`, `cargo run -- --game-smoke`,
  `markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md
  docs/fidelity/live-audio.md`, `git diff --check`, `cargo test --all-targets`,
  `cargo clippy --all-targets -- -D warnings`, `cargo run -- --live-smoke`,
  and `make fidelity`; coverage artifacts were refreshed at
  `2026-05-16 04:25:57 BST` and `2026-05-16 04:25:59 BST`, and the coverage
  checker reported
  `new Rust line coverage: 10/10 non-baselined added executable line(s)`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778901986517359`

### DC-148: Clean Game Smoke Ordered Sprite-Only WGPU Frame Stream Evidence

Status: `complete`

Goal: extend the final frame-level `wgpu` command evidence so clean smoke
proves every clean frame uses the expected ordered sprite-only command stream:
begin render pass, set viewport, upload scene projection, then execute the
sprite render pass.

Scope:

- Add an explicit ordered sprite-only frame-stream predicate to `WgpuFramePlan`.
- Expose ordered sprite-only frame-stream totals through `--game-smoke`.
- Validate that every clean smoke frame carries the expected ordered
  sprite-only frame command stream.
- Keep temporary raster frame commands rejected in clean gameplay smoke.
- Keep gameplay behavior, `--game-smoke`, and `--live-smoke` behavior otherwise
  unchanged.

Acceptance criteria:

- `WgpuFramePlan` can identify the expected ordered sprite-only command stream.
- `--game-smoke` reports ordered sprite-only frame-stream totals.
- `--game-smoke` fails if ordered sprite-only frame-stream totals diverge from
  the number of clean smoke frames.
- Focused renderer, smoke, and public guard tests cover the updated behavior.

Validation:

```sh
cargo fmt --check
cargo test --lib renderer::tests
cargo test --lib game_smoke::tests
cargo test --lib public_api_tests
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --game-smoke
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-16 04:29:11 BST` Started `DC-148`: extending final frame-level
  `wgpu` command evidence so clean smoke reports and validates ordered
  sprite-only frame command streams for every clean frame.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778902161201349`
- `2026-05-16 04:51:08 BST` Completed `DC-148`: `WgpuFramePlan` now exposes
  an ordered sprite-only frame command predicate for the expected begin-pass,
  viewport, scene-projection upload, and sprite render-pass execution stream.
  `--game-smoke` reports `frame_plan_ordered_sprite_only_frames: 24`, matching
  the 24 clean smoke frames, while preserving
  `frame_plan_begin_render_pass_commands: 24`,
  `frame_plan_viewport_commands: 24`, `temporary_raster_commands: 0`,
  `sprite_frame_plan_draws: 92`, and `sprite_frame_plan_instances: 290`.
  Validation now fails if ordered sprite-only frame-stream totals diverge from
  the frame count. Validation passed with `cargo fmt --check`,
  `cargo test --lib renderer::tests`, `cargo test --lib game_smoke::tests`,
  `cargo test --lib public_api_tests`, `cargo run -- --game-smoke`,
  `markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md
  docs/fidelity/live-audio.md`, `git diff --check`, `cargo test --all-targets`,
  `cargo clippy --all-targets -- -D warnings`, `cargo run -- --live-smoke`,
  and `make fidelity`; coverage artifacts were refreshed at
  `2026-05-16 04:50:37 BST` and `2026-05-16 04:50:38 BST`, and the coverage
  checker reported
  `new Rust line coverage: 13/13 non-baselined added executable line(s)`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778903465313059`

### DC-149: Clean Game Smoke Sprite Resource Bind-Group Evidence

Status: `complete`

Goal: extend the `wgpu` resource evidence so clean smoke proves every clean
frame prepares both expected sprite resource bind groups and their binding
entries, not only a resource-binding plan object and upload bytes.

Scope:

- Add aggregate bind-group and binding-entry totals to
  `SpriteResourceBindingPlan`.
- Expose sprite resource bind-group and binding-entry totals through
  `--game-smoke`.
- Validate those totals against the clean smoke frame count.
- Keep existing frame command, sprite command, upload, and resource evidence
  intact.
- Keep gameplay behavior, `--game-smoke`, and `--live-smoke` behavior otherwise
  unchanged.

Acceptance criteria:

- `SpriteResourceBindingPlan` reports expected bind-group and binding-entry
  totals.
- `--game-smoke` reports sprite resource bind-group and binding-entry totals.
- `--game-smoke` fails if those totals diverge from the per-frame expected
  values.
- Focused renderer, smoke, and public guard tests cover the updated behavior.

Validation:

```sh
cargo fmt --check
cargo test --lib renderer::tests
cargo test --lib game_smoke::tests
cargo test --lib public_api_tests
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --game-smoke
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-16 04:53:36 BST` Started `DC-149`: extending clean smoke resource
  evidence so it reports and validates WGPU sprite resource bind-group and
  binding-entry totals for every clean frame.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778903624623269`
- `2026-05-16 05:18:22 BST` Completed `DC-149`: `SpriteResourceBindingPlan`
  now exposes expected bind-group and binding-entry totals, and
  `--game-smoke` reports `sprite_resource_binding_frames: 24`,
  `sprite_resource_bind_groups: 48`, and
  `sprite_resource_binding_entries: 72` while preserving
  `frame_plan_ordered_sprite_only_frames: 24`,
  `temporary_raster_commands: 0`, `sprite_frame_plan_draws: 92`, and
  `sprite_frame_plan_instances: 290`. Validation now fails if the resource
  bind-group or binding-entry totals diverge from the per-frame expectations.
  Validation passed with `cargo fmt --check`,
  `cargo test --lib renderer::tests`, `cargo test --lib game_smoke::tests`,
  `cargo test --lib public_api_tests`, `cargo test --all-targets`,
  `cargo clippy --all-targets -- -D warnings`, `cargo run -- --game-smoke`,
  `cargo run -- --live-smoke`, `make fidelity`,
  `markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md
  docs/fidelity/live-audio.md`, and `git diff --check`; coverage artifacts
  were refreshed at `2026-05-16 05:17:28 BST` and
  `2026-05-16 05:17:29 BST`, and the coverage checker reported
  `new Rust line coverage: 16/16 non-baselined added executable line(s)`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778905091907579`

### DC-150: Clean Game Smoke Sprite Pipeline-Layout Bind-Group Evidence

Status: `complete`

Goal: extend the `wgpu` pipeline-layout evidence so clean smoke proves every
clean frame carries the expected sprite pipeline-layout bind groups and binding
entries, not only a pipeline-layout plan object.

Scope:

- Add aggregate bind-group and binding-entry totals to
  `SpritePipelineLayoutPlan`.
- Expose sprite pipeline-layout bind-group and binding-entry totals through
  `--game-smoke`.
- Validate those totals against the clean smoke frame count and the sprite
  resource-binding evidence.
- Keep existing frame command, sprite command, upload, resource, descriptor,
  and encoder evidence intact.
- Keep gameplay behavior, `--game-smoke`, and `--live-smoke` behavior otherwise
  unchanged.

Acceptance criteria:

- `SpritePipelineLayoutPlan` reports expected bind-group and binding-entry
  totals.
- `--game-smoke` reports sprite pipeline-layout bind-group and binding-entry
  totals.
- `--game-smoke` fails if those totals diverge from the per-frame expected
  values or from the resource-binding totals.
- Focused renderer, smoke, and public guard tests cover the updated behavior.

Validation:

```sh
cargo fmt --check
cargo test --lib renderer::tests
cargo test --lib game_smoke::tests
cargo test --lib public_api_tests
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --game-smoke
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-16 05:20:26 BST` Started `DC-150`: extending clean smoke
  pipeline-layout evidence so it reports and validates WGPU sprite
  pipeline-layout bind-group and binding-entry totals for every clean frame.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778905235441659`
- `2026-05-16 05:43:16 BST` Completed `DC-150`:
  `SpritePipelineLayoutPlan` now exposes expected bind-group and binding-entry
  totals, and `--game-smoke` reports `sprite_pipeline_layout_frames: 24`,
  `sprite_pipeline_layout_bind_groups: 48`, and
  `sprite_pipeline_layout_binding_entries: 72` while matching
  `sprite_resource_bind_groups: 48` and
  `sprite_resource_binding_entries: 72`. Existing clean renderer evidence
  remains intact with `temporary_raster_commands: 0`,
  `frame_plan_ordered_sprite_only_frames: 24`,
  `sprite_frame_plan_draws: 92`, and `sprite_frame_plan_instances: 290`.
  Validation now fails if the pipeline-layout bind-group or binding-entry
  totals diverge from the resource-binding totals. Validation passed with
  `cargo fmt --check`, `cargo test --lib renderer::tests`,
  `cargo test --lib game_smoke::tests`, `cargo test --lib public_api_tests`,
  `cargo test --all-targets`, `cargo clippy --all-targets -- -D warnings`,
  `cargo run -- --game-smoke`, `cargo run -- --live-smoke`, `make fidelity`,
  `markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md
  docs/fidelity/live-audio.md`, and `git diff --check`; coverage artifacts
  were refreshed at `2026-05-16 05:42:37 BST` and
  `2026-05-16 05:42:38 BST`, and the coverage checker reported
  `new Rust line coverage: 16/16 non-baselined added executable line(s)`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778906585665669`

### DC-151: Clean Game Smoke Sprite Render Pipeline Descriptor Evidence

Status: `complete`

Goal: extend the `wgpu` render pipeline descriptor evidence so clean smoke
proves every clean frame carries the expected descriptor shape, not only a
descriptor plan object.

Scope:

- Add aggregate layout bind-group, vertex-buffer, and color-target totals to
  `SpriteRenderPipelineDescriptorPlan`.
- Expose sprite render pipeline descriptor layout, vertex-buffer, and
  color-target totals through `--game-smoke`.
- Validate those totals against the clean smoke frame count and pipeline-layout
  evidence.
- Keep existing frame command, sprite command, upload, resource, layout, and
  encoder evidence intact.
- Keep gameplay behavior, `--game-smoke`, and `--live-smoke` behavior otherwise
  unchanged.

Acceptance criteria:

- `SpriteRenderPipelineDescriptorPlan` reports expected layout bind-group,
  vertex-buffer, and color-target totals.
- `--game-smoke` reports sprite render pipeline descriptor shape totals.
- `--game-smoke` fails if those totals diverge from the per-frame expected
  values or from the pipeline-layout totals.
- Focused renderer, smoke, and public guard tests cover the updated behavior.

Validation:

```sh
cargo fmt --check
cargo test --lib renderer::tests
cargo test --lib game_smoke::tests
cargo test --lib public_api_tests
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --game-smoke
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-16 05:45:13 BST` Started `DC-151`: extending clean smoke render
  pipeline descriptor evidence so it reports and validates WGPU descriptor
  layout bind-group, vertex-buffer, and color-target totals for every clean
  frame.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778906721909189`
- `2026-05-16 06:07:13 BST` Completed `DC-151`:
  `SpriteRenderPipelineDescriptorPlan` now exposes expected layout bind-group,
  vertex-buffer, and color-target totals, and `--game-smoke` reports
  `sprite_render_pipeline_descriptor_frames: 24`,
  `sprite_render_pipeline_descriptor_layout_bind_groups: 48`,
  `sprite_render_pipeline_descriptor_vertex_buffers: 48`, and
  `sprite_render_pipeline_descriptor_color_targets: 24`. The descriptor layout
  bind-group evidence matches `sprite_pipeline_layout_bind_groups: 48`, while
  existing clean renderer evidence remains intact with
  `temporary_raster_commands: 0`,
  `frame_plan_ordered_sprite_only_frames: 24`,
  `sprite_frame_plan_draws: 92`, and `sprite_frame_plan_instances: 290`.
  Validation now fails if descriptor layout bind-group totals diverge from the
  pipeline-layout totals, or if descriptor vertex-buffer/color-target totals
  diverge from the per-frame expectations. Validation passed with
  `cargo fmt --check`, `cargo test --lib renderer::tests`,
  `cargo test --lib game_smoke::tests`, `cargo test --lib public_api_tests`,
  `cargo run -- --game-smoke`,
  `markdownlint README.md SPEC.md PLAN.md docs/fidelity/refactor-freeze.md
  docs/fidelity/live-audio.md`, `git diff --check`,
  `cargo test --all-targets`, `cargo clippy --all-targets -- -D warnings`,
  `cargo run -- --live-smoke`, and `make fidelity`; coverage artifacts were
  refreshed at `2026-05-16 06:07:03 BST` and
  `2026-05-16 06:07:04 BST`, and the coverage checker reported
  `new Rust line coverage: 25/25 non-baselined added executable line(s)`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778908047112119`

### DC-152: Clean Game Smoke Sprite Render-Pass Encoder Command-Shape Evidence

Status: `complete`

Goal: extend the `wgpu` render-pass encoder evidence so clean smoke proves
every clean frame carries the expected encoder command mix, not only aggregate
encoder command and draw counts.

Scope:

- Add set-pipeline, set-bind-group, set-vertex-buffer, and set-index-buffer
  command totals to `SpriteRenderPassEncoderPlan`.
- Expose sprite render-pass encoder command-shape totals through
  `--game-smoke`.
- Validate those totals against the clean smoke frame count, pipeline-layout
  bind-group evidence, and descriptor vertex-buffer evidence.
- Keep existing frame command, sprite command, upload, resource, layout,
  descriptor, and draw evidence intact.
- Keep gameplay behavior, `--game-smoke`, and `--live-smoke` behavior otherwise
  unchanged.

Acceptance criteria:

- `SpriteRenderPassEncoderPlan` reports expected set-pipeline, set-bind-group,
  set-vertex-buffer, and set-index-buffer command totals.
- `--game-smoke` reports sprite render-pass encoder command-shape totals.
- `--game-smoke` fails if those totals diverge from per-frame expectations or
  upstream layout/descriptor evidence.
- Focused renderer, smoke, and public guard tests cover the updated behavior.

Validation:

```sh
cargo fmt --check
cargo test --lib renderer::tests
cargo test --lib game_smoke::tests
cargo test --lib public_api_tests
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make fidelity
cargo run -- --game-smoke
cargo run -- --live-smoke
markdownlint README.md SPEC.md PLAN.md \
  docs/fidelity/refactor-freeze.md docs/fidelity/live-audio.md
git diff --check
```

Work log:

- `2026-05-16 06:09:27 BST` Started `DC-152`: extending clean smoke
  render-pass encoder evidence so it reports and validates WGPU encoder
  set-pipeline, set-bind-group, set-vertex-buffer, and set-index-buffer command
  totals for every clean frame.
  Slack start update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778908176756169`
- `2026-05-16 06:31:13 BST` Completed `DC-152`:
  `SpriteRenderPassEncoderPlan` now exposes set-pipeline, set-bind-group,
  set-vertex-buffer, and set-index-buffer command totals, and `--game-smoke`
  reports `sprite_render_pass_encoder_frames: 24`,
  `sprite_encoder_set_pipeline_commands: 24`,
  `sprite_encoder_set_bind_group_commands: 48`,
  `sprite_encoder_set_vertex_buffer_commands: 48`, and
  `sprite_encoder_set_index_buffer_commands: 24`. The bind-group command
  evidence matches `sprite_pipeline_layout_bind_groups: 48`, and the
  vertex-buffer command evidence matches
  `sprite_render_pipeline_descriptor_vertex_buffers: 48`, while existing
  sprite-only evidence remains intact with `temporary_raster_commands: 0`,
  `frame_plan_ordered_sprite_only_frames: 24`,
  `sprite_encoder_commands: 236`, `sprite_encoder_draws: 92`, and
  `sprite_frame_plan_instances: 290`. Validation now fails if encoder
  command-shape totals diverge from per-frame expectations or upstream
  layout/descriptor evidence. Validation passed with `cargo fmt --check`,
  `cargo test --lib renderer::tests`, `cargo test --lib game_smoke::tests`,
  `cargo test --lib public_api_tests`, `cargo run -- --game-smoke`,
  `cargo test --all-targets`, `cargo clippy --all-targets -- -D warnings`,
  `cargo run -- --live-smoke`, and `make fidelity`; coverage artifacts were
  refreshed at `2026-05-16 06:31:01 BST` and
  `2026-05-16 06:31:02 BST`, and the coverage checker reported
  `new Rust line coverage: 48/48 non-baselined added executable line(s)`.
  Slack completion update:
  `https://xyzzytools.slack.com/archives/C0B1RNM8ZJ5/p1778909540882489`
