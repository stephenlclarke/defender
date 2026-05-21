# Known Fidelity Gaps

This file records behavior that must not be guessed in arcade-core code.

Status as of `2026-05-17`: the accepted red-label target remains the oracle for
the rewrite. The profiled clean-fidelity gate now matches all 12 embedded Phase
1 scenario input programs, but strict R9 final acceptance is still blocked by
the bounded accepted-surface limits below. Older entries remain closed fidelity
history, local-reference/tooling notes, archived prototype cleanup, or
post-acceptance validation records.

## Clean Rewrite Equivalence

- `DC-153` adds the first clean-vs-accepted harness. The harness compares the
  real clean `Game` to the accepted oracle with shared embedded Phase 1 scenario
  inputs and reports the first divergent frame and fields as TSV. The initial
  R0 expectation is that `attract_boot` and `start_game` diverge immediately in
  accepted boot/default state (`wave`, `lives`, high score) and render evidence
  because the clean runtime starts from its simplified domain defaults while the
  accepted oracle adapts source-backed cabinet state and visual signatures.
  Later R1-R9 work must either make selected scenarios pass or replace this
  broad entry with source-backed, scenario-specific gaps before changing
  gameplay behavior.
- `2026-05-16 22:06:05 BST`: `DC-164` expands the default
  `make clean-fidelity` gate from the R3 cabinet pair to all 12 embedded Phase 1
  scenarios. The full profiled gate matches `attract_boot`, `start_game`,
  `first_300_frames`, `firing`, `thrust_reverse`, `smart_bomb`, `hyperspace`,
  `abduction`, `death`, `wave_advance`, `planet_destruction`, and
  `high_score_entry`. This is R9 gate hardening, not strict rewrite closure: the
  long-scenario profiles still compare only the source-backed surfaces currently
  exposed by the accepted adapter.
- `2026-05-16 22:14:41 BST`: `DC-164` adds a first high-score accepted-surface
  slice. Active accepted high-score initials now flow from
  `MachineSnapshot::high_score_entry` through the neutral accepted facade and
  into the clean oracle `HighScoreInitialsState`, so future clean-fidelity
  profiles can fail on real initials drift instead of silently seeing an empty
  oracle value.
- `2026-05-16 22:32:17 BST`: `DC-164` extends that high-score accepted-surface
  slice with entry score/rank metadata and submitted player/score metadata.
  The clean `GameState`, accepted facade, oracle adapter, and clean-fidelity
  comparator now carry these fields, and the `high_score_entry` fixture still
  matches across 3428 frames. Full R9 high-score closure remains blocked on
  source-backed submitted table insertion and post-entry return evidence, not
  on the active entry/session metadata surface.
- `2026-05-16 22:51:40 BST`: `DC-164` adds source-shaped high-score table
  snapshots for the all-time and today's-greatest eight-row tables. Clean
  submission now inserts completed initials into both tables, updates the
  visible high score from the all-time table top row, and the accepted facade
  carries the same source-owned table snapshot from the machine memory. The
  `high_score_entry` fixture still matches across 3428 frames with
  `state.high_score_tables` compared. Full R9 high-score closure remains
  blocked on source-backed post-entry return timing/display evidence, not on
  submitted table insertion.
- `2026-05-16 23:48:56 BST`: `DC-164` adds source-shaped game-over return
  timing to the clean/accepted comparison surface. Clean final-life game over
  now waits through the source-backed 40-tick player-death sleep before
  qualifying high-score entry, non-qualifying scores wait through the 0xFF
  `HALL13` delay before the hall-of-fame display, and submitted initials enter
  the 60-tick hall-of-fame display stall before returning to attract. The
  machine snapshot, accepted facade, oracle adapter, and clean-fidelity
  comparator now carry `state.game_over`; the `death` and `high_score_entry`
  fixtures match with that field compared. Generic attract-mode hall-of-fame
  rotation timers are normalized out unless the clean runtime is also in a
  game-over return display, so this slice does not force unrelated attract
  choreography into the game-over contract.
- `2026-05-16 23:59:12 BST`: `DC-164` adds source-backed wave/enemy profile
  data to the clean/accepted comparison surface. Clean `GameState` now carries
  a `WaveProfileSnapshot` derived from `assets/red-label/wave-table.tsv`, the
  accepted facade maps the same fields from `MachineSnapshot::wave_profile`,
  and clean-fidelity compares `state.wave_profile`. This covers source-owned
  enemy counts, wave timing, velocities, shot timers, baiter delay, and related
  `WVTAB` profile fields without inventing live object positions or sprite
  identities. The targeted `start_game`, `first_300_frames`, `wave_advance`,
  and `planet_destruction` scenarios match with the new field compared.
- `2026-05-17 00:22:37 BST`: `DC-164` adds a neutral source object-list
  evidence surface. `MachineSnapshot` now carries source object active,
  inactive, projectile-list, visible-active counts plus a stable object-data
  evidence CRC; the accepted facade maps that into the clean oracle as
  `state.world.object_evidence`. The strict/full clean-fidelity profile can now
  fail on object-list evidence drift instead of seeing an empty accepted world
  placeholder. The current profiled R9 scenarios still do not compare live
  object positions, source picture/type identities, lifecycle transitions, or
  visual sprite presentation; those remain the active strict R9 blockers.
- `2026-05-17 00:45:07 BST`: `DC-164` extends that object evidence with bounded
  source object detail. The accepted surface now carries the first source object
  entries from the active, inactive, and projectile lists, including object
  address/slot, screen position, raw world position, raw velocity, picture
  address, and type byte. Clean `WorldSnapshot` derives comparable detail rows
  only from clean-owned enemies, humans, and projectiles. This promotes object
  positions and raw object-table detail into the strict/full comparison surface
  without assigning arcade identity to `OPICT`/`OTYP`; source object/sprite
  identity mapping, lifecycle transitions, and visual sprite presentation remain
  active strict R9 blockers.
- `2026-05-17 00:59:30 BST`: `DC-164` extends the bounded object-detail evidence
  with source object-picture descriptor metadata. Source details now carry the
  red-label picture label, descriptor size, primary image address, and alternate
  image address whenever the object `OPICT` word resolves to
  `assets/red-label/object-pictures.tsv`; clean details carry only explicit
  clean-domain categories such as lander, human, and player projectile. This
  exposes source picture identity without guessing the clean-to-arcade sprite
  mapping or turning descriptor metadata into render presentation parity.
- `2026-05-17 01:15:11 BST`: `DC-164` adds the first bounded source
  picture-to-clean sprite bridge. Red-label `PLAPIC`/`PLBPIC`, `LNDP1`-`LNDP3`,
  `ASTP1`-`ASTP4`, and `LASP1` labels now map to the clean sprite IDs backed by
  the already reclassified `ship1.png`, `lander1.png`, `humanoid1.png`, and
  `player-shot.png` assets. Source object-detail rows expose that mapped clean
  sprite when the label is in this explicit set, and clean rows expose the
  sprite rendered from their clean-domain category. Probe, swarmer, bomber,
  pod, baiter, mine, explosion, score-popup, mini-player, smart-bomb, and
  lifecycle mappings remain active blockers until their assets or source render
  paths are reclassified with stronger evidence.
- `2026-05-17 01:32:42 BST`: `DC-164` expands that bridge to the remaining
  enemy-family source pictures that already have prototype PNGs in
  `assets/sprites/`: `SCZP1` maps to `ENEMY_MUTANT`/`mutant1.png`,
  `UFOP1`-`UFOP3` to `ENEMY_BAITER`/`baiter1.png`, `TIEP1`-`TIEP4` to
  `ENEMY_BOMBER`/`bomber1.png`, `PRBP1` to `ENEMY_POD`/`pod1.png`, and
  `SWPIC1` to `ENEMY_SWARMER`/`swarmer1.png`. Bomb shells, explosions, score
  popups, miniplayer, smart-bomb visuals, real lifecycle transitions, and final
  render presentation remain active R9 blockers.
- `2026-05-17 01:45:10 BST`: `DC-164` extends the picture-to-clean sprite
  bridge again for non-enemy display/reward labels with existing transitional
  PNGs in `assets/sprites/`: `BMBP1`/`BMBP2` map to
  `ENEMY_BOMB`/`bomb1.png`, `BXPIC` to `BOMB_EXPLOSION`/`podexpl.png`,
  `SWXP1` to `SWARMER_EXPLOSION`/`swarmexpl.png`, `C25P1` to
  `SCORE_POPUP_250`/`score250_1.png`, `C5P1` to
  `SCORE_POPUP_500`/`score500_1.png`, `PLAMIN` to
  `PLAYER_LIFE_STOCK`/`littleship.png`, and `SBPIC` to
  `SMART_BOMB_STOCK`/`smartbomb.png`. `ASXP1`, `NULOB`, and `TEREX` remain
  unmapped until a later cycle has stronger asset/render evidence. Real bomb
  lifecycle, explosion timing, score-popup lifecycle, stock-count drawing,
  terrain-blow presentation, and final render presentation remain active R9
  blockers.
- `2026-05-17 01:59:29 BST`: `DC-164` closes the residual object-picture
  bridge labels with source object-image bytes instead of prototype PNGs:
  `ASXP1` maps to `ASTRONAUT_EXPLOSION` from `ASXD10`, `NULOB` maps to a
  transparent `NULL_OBJECT` from `NULD10`, and `TEREX` maps to
  `TERRAIN_EXPLOSION` from `TERX0`. All labels in
  `assets/red-label/object-pictures.tsv` now have an explicit clean sprite
  evidence target or transparent null target. This still does not implement
  astronaut death lifecycle behavior, null-object allocation behavior,
  terrain-blow timing/presentation, explosion lifecycle, or final render
  presentation parity.
- `2026-05-17 02:17:41 BST`: `DC-164` adds neutral expanded-object lifecycle
  evidence to the accepted comparison surface. Source appearance/explosion
  slots now flow through `MachineSnapshot`, the accepted facade, and the oracle
  into `state.world.expanded_objects`, including active count, last slot, slot
  kind, descriptor address, mapped clean sprite when the descriptor bridge knows
  one, erase pointer, center/top-left bytes, and attached object address when
  present. This promotes source expanded-object slot state into the strict/full
  surface without implementing explosion timing, score-popup lifecycle,
  terrain-blow behavior, stock-count drawing, clean spawning/physics, or final
  render presentation parity.
- `2026-05-17 02:34:43 BST`: `DC-164` moves current-player stock-count drawing
  from evidence-only into the clean sprite scene. Playing clean and oracle
  scenes now draw life stock from `PLAMIN`/`PLAYER_LIFE_STOCK` with the
  source-backed five-icon cap and smart-bomb stock from
  `SBPIC`/`SMART_BOMB_STOCK` with the three-icon cap, using the documented
  source display positions and steps. This removes the current-player stock
  drawing blocker, but does not implement two-player top-display parity,
  score-popup lifecycle, explosion timing, terrain-blow presentation, clean
  spawning/physics, or final render presentation parity.
- `2026-05-17 02:45:24 BST`: `DC-164` extends stock-count drawing to the
  two-player top-display positions. Clean and oracle playing scenes now draw
  player-two life stock from `PLAMIN`/`PLAYER_LIFE_STOCK` at the source-backed
  player-two life-stock origin and player-two smart-bomb stock from
  `SBPIC`/`SMART_BOMB_STOCK` at the source-backed player-two smart-bomb origin,
  using the same caps and steps as the source top-display path. This removes
  the P2 stock-icon portion of two-player top-display parity, but does not
  implement two-player score/text rendering, full turn/session switching,
  score-popup lifecycle, explosion timing, terrain-blow presentation, clean
  spawning/physics, or final render presentation parity.
- `2026-05-17 03:11:43 BST`: `DC-164` replaces the clean/oracle monolithic score
  placeholder with source-backed score digit sprites. The default atlas now
  decodes `NUMBR0`-`NUMBR9` from `assets/red-label/score-digits.tsv`, and clean
  plus oracle scenes draw player-one and player-two score fields at the
  top-display origins with six-position transfer order, two trailing zeroes, and
  leading-zero blanking. This removes the score-field portion of two-player
  top-display parity, but does not implement full two-player turn/session
  switching, remaining title/status/high-score text rendering, score-popup
  lifecycle, explosion timing, terrain-blow presentation, clean spawning/physics,
  or final render presentation parity.
- `2026-05-17 03:27:29 BST`: `DC-164` promotes source-backed `ST2`
  credited-start admission into the clean game. `start_two` now requires two
  credits, consumes both credits, enters play as player one with
  `player_count == 2`, and immediately exposes the existing player-one and
  player-two score/stock top-display fields. This removes the admission portion
  of the two-player flow blocker, but does not implement full two-player
  turn/session switching after player death, player-two respawn flow, remaining
  title/status/high-score text rendering, score-popup lifecycle, explosion
  timing, terrain-blow presentation, clean spawning/physics, or final render
  presentation parity.
- `2026-05-17 07:53:14 BST`: `DC-164` adds the source-backed final-life
  two-player switch/respawn slice. Clean final-life death now enters the
  `PLE02` `0x60`-tick player-switch sleep when the other player still has
  stock, records the source-shaped switch-from/switch-to players in
  `state.game_over`, then hands off to the other player through the clean
  playfield entry path. `MachineSnapshot`, the accepted facade, and the oracle
  adapter now carry the same switch timing evidence. Remaining R9 blockers are
  later two-player turn/session sequencing and high-score ordering, remaining
  title/status/high-score text rendering, score-popup lifecycle, explosion
  timing, terrain-blow presentation, clean spawning/physics, and final render
  presentation parity.
- `2026-05-17 08:26:47 BST`: `DC-164` adds the source-backed player-switch
  prompt text slice. Clean and oracle scenes now draw `PLYR1`/`PLYR2` as
  `PLAYER ONE`/`PLAYER TWO` at `0x3C78` plus `GO` as `GAME OVER` at `0x3E88`
  during the translated `PLE02` switch sleep, using
  `assets/red-label/messages.tsv` and `assets/red-label/message-glyphs.tsv`
  for message text, glyph dimensions, and atlas pixels. Remaining R9 blockers
  are later two-player turn/session sequencing and high-score ordering,
  remaining title/status/high-score text rendering, score-popup lifecycle,
  explosion timing, terrain-blow presentation, clean spawning/physics, and
  final render presentation parity.
- `2026-05-17 08:30:58 BST`: `DC-164` adds the source-backed final
  game-over prompt text slice. Clean and oracle scenes now draw `GO` as
  `GAME OVER` at `0x3E80` during the final `PLE2` player-death game-over
  sleep, reusing `assets/red-label/messages.tsv` and
  `assets/red-label/message-glyphs.tsv` for text and glyph pixels. Remaining
  R9 blockers are later two-player turn/session sequencing and high-score
  ordering, remaining title/status/high-score text rendering, score-popup
  lifecycle, explosion timing, terrain-blow presentation, clean
  spawning/physics, and final render presentation parity.
- `2026-05-17 08:47:49 BST`: `DC-164` adds the source-backed high-score entry
  prompt text slice. Clean and oracle scenes now draw the active entry player
  label at `0x3E38`, `HOFV1`-`HOFV4` instruction lines from `0x1458` with the
  source vertical offsets, and entered initials from `0x46AC` with the source
  horizontal offsets while `GamePhase::HighScoreEntry` is active. This reuses
  `assets/red-label/messages.tsv` and `assets/red-label/message-glyphs.tsv`
  for text and glyph pixels. Remaining R9 blockers are later two-player
  turn/session sequencing and high-score ordering, remaining title/status and
  high-score display text outside the entry prompt, score-popup lifecycle,
  explosion timing, terrain-blow presentation, clean spawning/physics, and
  final render presentation parity.
- `2026-05-17 09:07:31 BST`: `DC-164` adds the source-backed hall-of-fame
  display text slice. Clean and oracle scenes now draw `HALDIS` headings from
  `HALLD_TITLE`, `HALLD_TODAYS`, `HALLD_ALL_TIME`, and `HALLD_GREATEST` at the
  translated screen addresses, plus both visible high-score tables with rank
  digits, initials, and six-character score fields while
  `hall_of_fame_stall_remaining` is active. The table rows use the source table
  starts, `0x0A` row step, initials offset, score offset, and leading-blank
  score rules, reusing `assets/red-label/messages.tsv`,
  `assets/red-label/message-glyphs.tsv`, and `assets/red-label/score-digits.tsv`.
  Remaining R9 blockers are later two-player turn/session sequencing and
  high-score ordering, title/status text plus high-score underline/logo
  presentation, score-popup lifecycle, explosion timing, terrain-blow
  presentation, clean spawning/physics, and final render presentation parity.
- `2026-05-17 09:25:24 BST`: `DC-164` adds the source-backed high-score entry
  underline word slice. Clean and oracle scenes now draw the `HOFUL`
  initials-entry underline words while `GamePhase::HighScoreEntry` is active,
  using the source start `0x45B7`, `0x0800` initial step, and
  `[0x0400,0x0300,0x0200,0x0100]` word offsets. The clean scene marks the
  active cursor underline separately from inactive underline words with a small
  atlas-backed sprite. Remaining R9 blockers are later two-player
  turn/session sequencing and high-score ordering, title/status text,
  hall-of-fame display underline/logo presentation, exact entry underline
  palette/blink/color behavior, score-popup lifecycle, explosion timing,
  terrain-blow presentation, clean spawning/physics, and final render
  presentation parity.
- `2026-05-17 09:40:02 BST`: `DC-164` adds the source-backed hall-of-fame
  display underline word slice. Clean and oracle scenes now draw the `HALDIS`
  display underline words while `hall_of_fame_stall_remaining` is active,
  using the source left base `0x1E7B` and the two verified offset segments
  `0x5F..0x41` and `0x1E..0x00`, producing underline word positions from
  `0x7D7B` through `0x1E7B`. The display underline words reuse the small
  atlas-backed clean sprite from the entry underline slice. Remaining R9
  blockers are later two-player turn/session sequencing and high-score
  ordering, title/status text, expanded hall-of-fame logo presentation, exact
  underline palette/blink/color behavior, score-popup lifecycle, explosion
  timing, terrain-blow presentation, clean spawning/physics, and final render
  presentation parity.
- `2026-05-17 09:57:07 BST`: `DC-164` adds the source-backed attract credits
  text slice. Clean and oracle scenes now draw the `CREDV` `CREDITS:` label at
  source screen address `0x28E5` and the visible credit count digits at
  `0x48E5` during normal `GamePhase::Attract`, while suppressing that overlay
  during the hall-of-fame display stall. Remaining R9 blockers are later
  two-player turn/session sequencing and high-score ordering, title/status
  text beyond attract credits, expanded hall-of-fame logo presentation, exact
  underline palette/blink/color behavior, score-popup lifecycle, explosion
  timing, terrain-blow presentation, clean spawning/physics, and final render
  presentation parity.
- `2026-05-17 10:14:55 BST`: `DC-164` adds the source-backed hall-of-fame
  Defender logo slice. Clean and oracle scenes now draw the `HALDIS` expanded
  logo while `hall_of_fame_stall_remaining` is active, using source screen
  address `0x3038`, dimensions `0x3C` by `0x18`, and an atlas-backed sprite
  generated from the compressed source logo bytes used by the accepted adapter.
  Remaining R9 blockers are later two-player turn/session sequencing and
  high-score ordering, title/status text beyond attract credits, exact
  logo/underline palette/blink/color behavior, score-popup lifecycle,
  explosion timing, terrain-blow presentation, clean spawning/physics, and
  final render presentation parity.
- `2026-05-17 10:36:53 BST`: `DC-164` adds the source-backed attract presents
  text slice. Clean and oracle scenes now draw the translated attract
  `ELECTRONICS INC.` and `PRESENTS` text at source screen addresses `0x3258`
  and `0x3E6C` during ordinary attract frames, while suppressing it during the
  hall-of-fame display stall. Remaining R9 blockers are later two-player
  turn/session sequencing and high-score ordering, broader title/status text
  outside attract credits/presents, exact logo/underline palette/blink/color
  behavior, score-popup lifecycle, explosion timing, terrain-blow
  presentation, clean spawning/physics, and final render presentation parity.
- `2026-05-17 10:50:59 BST`: `DC-164` routes that attract presents projection
  through a clean source-message control helper. The clean renderer now applies
  the `ELECV` row-feed and horizontal-cursor controls, so the same source
  message text positions `ELECTRONICS INC.` at `0x3258` and `PRESENTS` at
  `0x3E6C` in both clean and oracle scenes. Remaining R9 blockers are later
  two-player turn/session sequencing and high-score ordering, broader
  title/status text outside attract credits/presents, exact logo/underline
  palette/blink/color behavior, score-popup lifecycle, explosion timing,
  terrain-blow presentation, clean spawning/physics, and final render
  presentation parity.
- `2026-05-17 11:06:23 BST`: `DC-164` adds the source-backed attract
  instruction text slice. Clean and oracle scenes now draw `SCANV`, `LANDV`,
  `MUTV`, `BAITV`, `BOMBV`, `SWRMPV`, and `SWARMV` through the source message
  control helper at source screen addresses `0x4330`, `0x1C70`, `0x3C70`,
  `0x5F70`, `0x1CA8`, `0x40A8`, and `0x5CA8` during ordinary attract frames,
  while suppressing them during the hall-of-fame display stall. Remaining R9
  blockers are later two-player turn/session sequencing and high-score
  ordering, broader title/status text outside attract credits/presents/
  instruction labels, attract logo/page timing, exact logo/underline
  palette/blink/color behavior, score-popup lifecycle, explosion timing,
  terrain-blow presentation, clean spawning/physics, and final render
  presentation parity.
- `2026-05-17 11:20:46 BST`: `DC-164` adds the source-backed two-player start
  prompt slice. Clean and oracle scenes now draw `PLYR1`/`PLYR2` as
  `PLAYER ONE`/`PLAYER TWO` at source screen address `0x3C80` while a
  two-player start handoff is pending. The slice intentionally leaves
  one-player start, broader player-switch/session sequencing, palette/blink/
  color behavior, and gameplay lifecycle rules unchanged. Remaining R9
  blockers are later two-player turn/session sequencing and high-score
  ordering, broader title/status text outside covered prompt/attract surfaces,
  attract logo/page timing, exact logo/underline palette/blink/color behavior,
  score-popup lifecycle, explosion timing, terrain-blow presentation, clean
  spawning/physics, and final render presentation parity.
- `2026-05-17 11:33:30 BST`: `DC-164` adds the source-backed wave-completion
  status text slice. Clean and oracle scenes now draw `ATWV` `ATTACK WAVE` at
  `0x3850`, the source-shaped wave number at `0x6550`, `COMPV` `COMPLETED` at
  `0x3D60`, `BONSX` `BONUS X` at `0x3C90`, and the source-shaped multiplier
  digit at `0x5890` on the existing clean wave-cleared frame. The slice does
  not add the source survivor bonus loop, source sleep timing, score-popup
  lifecycle, explosion timing, terrain-blow behavior, or wave lifecycle state.
  Remaining R9 blockers are later two-player turn/session sequencing and
  high-score ordering, broader title/status text outside covered prompt/
  attract/wave-completion surfaces, attract logo/page timing, exact
  logo/underline palette/blink/color behavior, score-popup lifecycle,
  explosion timing, terrain-blow presentation, clean spawning/physics, and
  final render presentation parity.
- `2026-05-17 11:49:32 BST`: `DC-164` adds the source-backed survivor bonus
  icon presentation slice. Clean and oracle scenes now draw one source `ASTP3`
  survivor icon for each remaining clean human on the existing wave-cleared
  frame, starting at source screen address `0x3CA0` and stepping by
  `+0x0400`. The slice does not add the source survivor bonus loop,
  per-survivor scoring cadence, source sleep timing, score-popup lifecycle,
  explosion timing, terrain-blow behavior, or wave lifecycle state. Remaining
  R9 blockers are later two-player turn/session sequencing and high-score
  ordering, broader title/status text outside covered prompt/attract/
  wave-completion surfaces, attract logo/page timing, exact logo/underline
  palette/blink/color behavior, score-popup lifecycle, explosion timing,
  terrain-blow presentation, clean spawning/physics, and final render
  presentation parity.
- `2026-05-17 12:03:01 BST`: `DC-164` adds the source-backed normal-attract
  Defender wordmark presentation slice. Clean and oracle scenes now draw the
  existing source-expanded Defender logo sprite at source restore screen
  address `0x3090` during normal attract, while suppressing it during the
  hall-of-fame display stall. The slice does not add the Williams logo table
  walker, `PRES`/`DEFEND` page scheduler behavior, copyright bitmap
  presentation, exact palette/blink/color behavior, object appearance
  sequencing, gameplay lifecycle changes, or the survivor bonus loop/cadence.
  Remaining R9 blockers are later two-player turn/session sequencing and
  high-score ordering, broader title/status text outside covered prompt/
  attract/wave-completion surfaces, Williams/copyright attract page timing,
  exact logo/underline palette/blink/color behavior, score-popup lifecycle,
  explosion timing, terrain-blow presentation, clean spawning/physics, and
  final render presentation parity.
- `2026-05-17 12:15:25 BST`: `DC-164` adds the source-backed normal-attract
  copyright bitmap presentation slice. The clean renderer now owns an
  atlas-backed copyright strip sprite generated from the checked-in `CPRTAB`
  bytes, and clean plus oracle normal attract scenes draw it at source screen
  address `0x3BD0` while suppressing it during the hall-of-fame display stall.
  The slice does not add the Williams logo table walker, `PRES`/`DEFEND` page
  scheduler behavior, copyright wait gates, exact palette/blink/color
  behavior, object appearance sequencing, gameplay lifecycle changes, or the
  survivor bonus loop/cadence. Remaining R9 blockers are later two-player
  turn/session sequencing and high-score ordering, broader title/status text
  outside covered prompt/attract/wave-completion surfaces,
  Williams/copyright attract wait timing, exact logo/underline
  palette/blink/color behavior, score-popup lifecycle, explosion timing,
  terrain-blow presentation, clean spawning/physics, and final render
  presentation parity.
- `2026-05-17 12:37:10 BST`: `DC-164` adds the source-backed normal-attract
  Williams logo presentation slice. The clean renderer now owns an
  atlas-backed Williams logo sprite generated from the checked-in `LGOTAB`
  final pixel pattern, and clean plus oracle normal attract scenes draw it at
  source screen address `0x363C` while suppressing it during the
  hall-of-fame display stall. The slice does not add the live `LGOTAB`
  table-walker timing, fast/normal page-rate switch, `PRES`/`DEFEND` page
  scheduler behavior, copyright wait gates, exact palette/blink/color
  behavior, object appearance sequencing, gameplay lifecycle changes, or the
  survivor bonus loop/cadence. Remaining R9 blockers are later two-player
  turn/session sequencing and high-score ordering, broader title/status text
  outside covered prompt/attract/wave-completion surfaces,
  Williams/copyright attract wait timing, exact logo/underline
  palette/blink/color behavior, score-popup lifecycle, explosion timing,
  terrain-blow presentation, clean spawning/physics, and final render
  presentation parity.
- `2026-05-17 12:54:07 BST`: `DC-164` adds the source-backed top-display border
  presentation slice. The clean renderer now owns an atlas-backed border word
  sprite, and clean plus oracle playing scenes draw the source `BORDER`
  geometry: the bottom display line, scanner side boundaries, top scanner
  boundary, and scanner marker bars at translated source screen positions. The
  slice does not add scanner/radar animation, score-popup lifecycle, explosion
  timing, terrain-blow lifecycle, clean spawning/physics, live top-display
  scheduling, exact palette/blink/color behavior, or broader title/status text.
  Remaining R9 blockers are later two-player turn/session sequencing and
  high-score ordering, broader title/status text outside covered
  prompt/attract/top-display-border/wave-completion surfaces,
  Williams/copyright attract wait timing, exact logo/underline/border
  palette/blink/color behavior, score-popup lifecycle, explosion timing,
  terrain-blow presentation, clean spawning/physics, scanner/radar animation,
  and final render presentation parity.
- `2026-05-17 13:13:55 BST`: `DC-164` adds the bounded source object-detail
  sprite presentation slice. Clean and oracle playing scenes now project source
  object-detail rows that already carry `screen_position`, `picture_size`, and a
  mapped clean `SpriteId`: active rows draw on the object layer and projectile
  rows draw on the projectile layer. Inactive rows and transparent `NULOB`
  details remain evidence-only. The slice does not add clean spawning/physics,
  lifecycle transitions, expanded-object slot rendering, score-popup lifecycle,
  explosion timing, terrain-blow presentation, scanner/radar animation, or exact
  palette/blink/color behavior. Remaining R9 blockers are later two-player
  turn/session sequencing and high-score ordering, broader title/status text
  outside covered prompt/attract/top-display-border/wave-completion surfaces,
  Williams/copyright attract wait timing, exact logo/underline/border
  palette/blink/color behavior, score-popup lifecycle, explosion timing,
  terrain-blow presentation, clean spawning/physics, scanner/radar animation,
  expanded-object slot rendering, and final render presentation parity.
- `2026-05-17 16:28:43 BST`: `DC-164` adds the clean attract page scheduler
  slice for R9-B2. `GameState` now carries `AttractPresentationSnapshot`
  page-frame evidence for the Williams logo, presents copy, Defender wordmark,
  copyright wait, and instruction-page surfaces. Clean and oracle scenes gate
  the title-program sprites from that snapshot while leaving the existing
  credits projection and hall-of-fame stall suppression intact. Focused
  `attract_boot` and `start_game` clean-fidelity scenarios match with this
  scheduler. Remaining R9 blockers are later two-player turn/session
  sequencing and high-score ordering, exact logo/underline/border
  palette/blink/color behavior, live Williams logo table-walker animation,
  score-popup lifecycle, explosion timing, terrain-blow presentation, clean
  spawning/physics, scanner/radar animation, and final render presentation
  parity.
- `2026-05-17 16:47:14 BST`: `DC-164` adds the source visual-state contract
  slice for R9-B3. Clean `SOURCE_VISUAL_STATE` records the source Williams
  status/color/rate evidence, attract instruction color words, Hall of Fame
  display and entry color indices, Hall of Fame blink color and 15-tick sleep,
  Hall of Fame active/inactive underline words, and top-display border/scanner
  marker words. Clean and oracle scenes route HUD, attract title, top-display
  border, Hall of Fame logo/text, and underline tints through that contract
  while preserving the current white/gray clean sprite output. Remaining R9
  blockers are later two-player turn/session sequencing and high-score
  ordering, live Williams logo table-walker animation, hardware palette/RGB
  render audit residuals, score-popup lifecycle, explosion timing,
  terrain-blow presentation, clean spawning/physics, scanner/radar animation,
  and final render presentation parity.
- `2026-05-17 17:35:09 BST`: `DC-164` adds the source scanner/radar state and
  sprite slice for R9-B4. Clean `WorldSnapshot` now carries
  `ScannerRadarSnapshot` with the source scanner process cadence `[2, 2, 4]`,
  selected scanner map `1`, scan-left calculation, object erase-table addresses,
  source `SETEND`, object blips, and player blip bytes. Source `OBJCOL`
  scanner colors now flow through the machine snapshot, accepted facade, oracle
  adapter, and clean object evidence, and clean/oracle scenes draw
  atlas-backed scanner object/player HUD blips at translated source scanner
  screen positions. Phase 2 validation passed with the full all-scenario
  clean-fidelity gate and the broad fidelity gate. Remaining R9 blockers are
  score-popup lifecycle, explosion timing, terrain-blow presentation, clean
  spawning/physics, later two-player turn/session sequencing and high-score
  ordering, hardware palette/RGB render audit residuals, and final render
  presentation parity.
- `2026-05-17 17:51:14 BST`: R9-C1 adds source-backed score-popup lifecycle
  evidence and clean projection. `C25P1` and `C5P1` expanded-object rows are
  now classified as score popups with source 50-tick lifetime metadata,
  250/500 values, 6x6 descriptor sizes, and mapped `SCORE_POPUP_250` /
  `SCORE_POPUP_500` sprite identity through the machine snapshot, accepted
  facade, oracle adapter, and clean scene path. Clean `WorldSnapshot` can spawn
  score popups at a source top-left position, project them as expanded-object
  sprites, and remove them when the 50-frame lifetime expires without changing
  score arithmetic. Remaining R9 blockers are explosion timing,
  terrain-blow presentation, clean spawning/physics and rescue/abduction entry
  points, later two-player turn/session sequencing and high-score ordering,
  hardware palette/RGB render audit residuals, and final render presentation
  parity.
- `2026-05-17 18:10:59 BST`: R9-C2 adds the source expanded-object explosion
  timing slice. `EXST`/`EXPU` rows now carry source frame/lifetime metadata
  from `RSIZE = 0x0100`, `+0x00AA` per update, and the `> 0x30` high-byte kill
  threshold through the machine snapshot, accepted facade, oracle adapter, and
  clean `WorldSnapshot`. Clean lander projectile collisions spawn timed
  expanded-object explosions, clean world helpers cover the mapped
  `LNDP1`/`BXPIC`/`SWXP1`/`ASXP1`/`PLAPIC` descriptor families, and clean/oracle
  scenes scale explosion sprites from the source `RSIZE` high byte. Remaining
  B06 work is the bank-7 `PXVCT`/`PX1A` player death pixel-cloud and any later
  object-ecology entry points that must start non-lander explosion families in
  real gameplay.
- `2026-05-17 18:33:00 BST`: Step 48 / R9-C2 closes B06 by adding the
  player-death bank-7 `PXVCT`/`PX1A` pixel-cloud surface. Machine snapshots,
  the accepted facade, the oracle adapter, clean `WorldSnapshot`, and
  clean/oracle scenes now carry the source color table, color counter,
  frame index, visible piece positions, and 4x1 versus split 4x2 pixel shape.
  Clean player/enemy contact starts that cloud from the source-shaped player
  center offset. Future object-ecology work may add more gameplay entry points
  that start already mapped non-lander explosion descriptor families, but that
  belongs to B08 rather than the explosion timing blocker.
- `2026-05-17 18:49:07 BST`: Step 49 / R9-C3 closes B07 by adding
  source-backed terrain-blow mutation and presentation evidence. Machine
  snapshots, the accepted facade, the oracle adapter, clean `WorldSnapshot`,
  and clean-fidelity comparison now carry the source `TERBLO` terrain-blown
  status bit, stage, iteration, sleep, pseudo-color, overload counter,
  terrain/scanner erase entry counts, and remaining nonzero terrain words.
  Clean planet destruction clears terrain segments, disables scanner terrain,
  and projects the two-per-pass `TEREX` terrain explosions through the
  expanded-object sprite path with source `TERBLO` / `AHSND` command evidence
  on entry. Full rescue/abduction object ecology and the gameplay entry points
  that remove humans remain B08 work.
- `2026-05-17 19:06:15 BST`: R9-C4 adds the first clean object-ecology
  progress slice. Clean wave startup now derives the active enemy batch from
  `WaveProfileSnapshot` instead of using the previous wave-number lander
  shortcut: wave 1 starts with five active landers from the source wave-size
  field, and later source-exposed bomber/pod families enter the clean active
  batch with deterministic positions and source-profile-derived pixel
  velocities. Clean object evidence, scanner colors, scene sprites, collision
  sizes, scores, and explosion entry points now recognize lander, mutant,
  bomber, pod, swarmer, and baiter families. The player-death pixel cloud is
  also cleared before high-score entry handoff so high-score scenes remain
  prompt/table-only after the final death sleep. Remaining B08 work is
  reserve/inactive transitions, exact family movement/projectile behavior,
  abduction/carry/fall/catch/rescue/loss, pod-to-swarmer spawning, and baiter
  runtime entry points.
- `2026-05-17 19:18:07 BST`: R9-C4 adds source-profile reserve accounting and
  active/reserve transitions. Clean `WorldSnapshot` now carries
  `EnemyReserveSnapshot` counts for source-profile enemies outside the active
  batch, clean object evidence reports those counts as inactive objects, and
  gameplay activates the next reserve batch before emitting `WaveCleared`.
  Targeted clean-fidelity still matches `start_game`, `smart_bomb`, and
  `wave_advance`. Remaining B08 work is exact per-family movement/projectile
  behavior, abduction/carry/fall/catch/rescue/loss, pod-to-swarmer spawning,
  and baiter runtime entry points.
- `2026-05-17 19:28:49 BST`: R9-C4 adds the clean pod-to-swarmer destruction
  transition. Projectile and smart-bomb pod kills now reuse the shared clean
  enemy-destroy path, spawn the pod explosion, award the 1000-point pod score,
  and append a deterministic mini-swarmer batch bounded by the source `MMSW`
  request limit of six and the active swarmer cap of twenty. Focused clean
  tests cover projectile pod kills, smart-bomb pod kills, and the active
  swarmer cap; targeted clean-fidelity still matches `smart_bomb` and
  `wave_advance`. Remaining B08 work is exact per-family movement/projectile
  behavior, source RNG/velocity/sleep/shot-timer parity for mini-swarmers,
  abduction/carry/fall/catch/rescue/loss, and baiter runtime entry points.
- `2026-05-17 19:37:32 BST`: R9-C4 adds the clean baiter runtime entry
  transition. Clean `Game` now carries the source `GEXEC`/`UFOST` timer shape:
  baiter entry advances on the 15-frame game-exec cadence, accelerates when
  source enemy totals fall to eight or fewer, skips the zero-enemy wave-clear
  case, and caps active baiters at twelve. The clean spawn is deterministic and
  uses the existing baiter sprite, object evidence, scanner, collision, score,
  and scene paths. Focused tests cover timer-edge spawn, active-cap behavior,
  and low-enemy acceleration; targeted clean-fidelity still matches
  `start_game` and `wave_advance`. Remaining B08 work is exact per-family
  movement/projectile behavior, source RNG/velocity/sleep/shot-timer parity
  for mini-swarmers, exact baiter velocity/shot behavior, and
  abduction/carry/fall/catch/rescue/loss.
- `2026-05-17 19:45:09 BST`: R9-C4 adds the clean lander
  abduction/carry/release slice from source `LANDG` / `LKIL1` evidence. Clean
  landers now capture aligned humans into the carried state, move carried
  humans with the fleeing lander, tint carried humans through the existing
  scene path, and release the passenger when the carrying lander is destroyed.
  Focused tests cover capture/carry motion and passenger release on lander
  kill; targeted clean-fidelity still matches `abduction`. Remaining B08 work
  is exact per-family movement/projectile behavior,
  source RNG/velocity/sleep/shot-timer parity for mini-swarmers, exact baiter
  velocity/shot behavior, falling astronaut motion, player catch, rescue
  scoring, safe/fatal landing, and human-loss transitions.
- `2026-05-20 22:06:29 BST`: R9-C4 tightens the clean lander carry
  association. Source-shaped lander flee/orbit decisions now use the passenger
  carried by that lander instead of a global "any carried human" flag, and
  carried human positions stay with the matching lander in multi-lander scenes.
- `2026-05-20 22:14:47 BST`: R9-C4 adds the bounded source `LANDF` /
  `LNDFXA` passenger pull-in slice. Source-shaped clean landers now stop
  fleeing at the upper pull edge, pull the carried passenger upward one screen
  row at a time, and only consume the passenger/convert to a mutant once the
  passenger has reached the lander.
- `2026-05-20 22:21:05 BST`: R9-C4 tightens the clean source `LANDG`
  capture transition. Source-shaped clean landers now seed the source flee
  vector and `LANDF` sleep countdown on the capture frame before carrying the
  passenger toward the pull-in edge.
- `2026-05-20 22:29:08 BST`: R9-C4 adds the bounded source `LNDFXA`
  cleared-target give-up edge. Source-shaped clean landers already at the
  top pull edge now leave active play and return to the lander reserve when no
  matching carried passenger remains, instead of falling through the generic
  no-human mutation path.
- `2026-05-20 22:43:43 BST`: R9-C4 adds the bounded source `LANDG`
  target-approach slice. Source-shaped clean landers already in the grab state
  now perform the one-step source approach toward the aligned uncarried human,
  clear active velocity, sleep for one frame, and keep running the lander shot
  timer before capture.
- `2026-05-20 22:53:13 BST`: R9-C4 adds bounded explicit selected-human
  target state to clean source landers. Source-shaped clean landers can retain
  a target human index, enter `LANDG` from `LANDS0` only when that selected
  target passes the source close-X check, keep captures target-specific, and
  retarget from cleared or player-carried target slots. Default source lander
  spawns still leave the target unset until source target-list restoration is
  modeled for clean humans.
- `2026-05-20 23:11:39 BST`: R9-C4 adds the bounded clean-human `TLIST`
  metadata prerequisite. Initial clean humans now carry deterministic source
  target-list slot addresses starting at `0xA11A` with a two-byte stride.
  Runtime lander target assignment remained unchanged until the later
  source target-list cursor slice.
- `2026-05-20 23:33:42 BST`: R9-C4 adds the bounded source-restored clean
  human placement/count slice. Initial clean worlds now restore ten
  source-shaped humans through the `PLRES` / `TLIST` target-group algorithm,
  keep deterministic target-list slot addresses, and update initial active
  object/sprite evidence counts.
- `2026-05-20 23:44:06 BST`: R9-C4 adds the bounded source target-list cursor
  assignment slice. Clean worlds now carry the source `TPTR`-shaped cursor,
  initial and reserve source lander spawns select restored `TLIST` humans by
  advancing that cursor before scanning, and retargeting reuses the same cursor
  when a selected target is no longer live.
- `2026-05-20 23:53:49 BST`: R9-C4 adds the bounded source human
  restore-evidence slice. Source-restored clean humans now retain the `PLRES`
  `LSEED` X low byte as the source X fraction and carry the odd-`LSEED`
  `ASTP3` astronaut picture choice in clean object-detail evidence, while
  default clean human rows continue to report `ASTP1`.
- `2026-05-21 00:06:59 BST`: R9-C4 adds the bounded source `ASTRO`
  target-list walk slice. Clean worlds now carry a separate source `ASTRO`
  process cursor/sleep state, advance one restored uncarried `TLIST` human per
  source cadence, apply source fixed-point X steps and terrain-relative Y
  steps, and cycle object-detail evidence from `ASTP1` through `ASTP4`.
- `2026-05-21 00:17:03 BST`: R9-C4 adds bounded source enemy hit sound-command
  evidence. Clean projectile and smart-bomb enemy destruction now surface the
  source family sound command byte through `SoundEvent::UnmappedSoundCommand`:
  landers use `LHSND` (`0xF9`), mutants use `SCHSND` (`0xE8`), bombers use
  `TIHSND` (`0xFE`), pods use `PRHSND` (`0xFA`), swarmers use `SWHSND`
  (`0xF8`), and baiters use `UFHSND` (`0xF8`). Remaining B08 work is exact
  per-family movement/projectile behavior beyond the covered enemy-hit command
  evidence and focused source ecology fixtures for those transitions.
- `2026-05-21 00:25:05 BST`: R9-C4 adds bounded source enemy shot sound-command
  evidence. Clean lander, mutant, baiter, and mini-swarmer projectile launches
  now surface the source shot sound command byte only when the source-shaped
  shell allocation succeeds: landers use `LSHSND` (`0xFC`), mutants use
  `SSHSND` (`0xF6`), baiters use `USHSND` (`0xFC`), and mini-swarmers use
  `SWSSND` (`0xF3`). Full mini-swarmer shell-list attempts and bomber
  `BOMBST` shell allocations remain silent. Remaining B08 work is exact
  per-family movement/projectile behavior beyond the covered enemy-hit and
  enemy-shot command evidence and focused source ecology fixtures for those
  transitions.
- `2026-05-21 00:36:04 BST`: R9-C4 adds bounded source astronaut sound-command
  evidence for the covered clean human ecology transitions. Killed carrying
  landers now release passengers from the source carried/pull positions and
  surface `ASCSND` (`0xE5`) instead of the ordinary lander hit command when a
  passenger is released; player catches surface `ACSND` (`0xF7`); safe landings
  surface `ALSND` (`0xE0`). Remaining B08 work is exact per-family
  movement/projectile behavior beyond the covered enemy-hit, enemy-shot, and
  astronaut command evidence plus focused source ecology fixtures for those
  transitions.
- `2026-05-21 00:45:16 BST`: R9-C4 adds bounded source lander abduction
  sound-command evidence for the covered pickup and top-edge pull-in
  transitions. Clean lander pickup now surfaces `LPKSND` (`0xF4`), and the
  source-shaped pull-in transition now surfaces `LSKSND` (`0xF1`) without
  repeating it on follow-up pull frames. Remaining B08 work is exact
  per-family movement/projectile behavior beyond the covered enemy-hit,
  enemy-shot, lander-abduction, and astronaut command evidence plus focused
  source ecology fixtures for those transitions.
- `2026-05-21 00:53:56 BST`: R9-C4 adds bounded source player-action
  sound-command evidence for the covered clean fire and smart-bomb edges.
  Successful clean laser launches now surface `LASSND` (`0xEB`), while
  accepted smart-bomb inputs surface the first `SBSND` command (`0xEE`) before
  per-enemy destruction sounds. Remaining B08 work is exact per-family
  movement/projectile behavior beyond the covered enemy-hit, enemy-shot,
  player-action, lander-abduction, and astronaut command evidence plus focused
  source ecology fixtures for those transitions.
- `2026-05-21 02:19:50 BST`: R9-C4 adds bounded source thrust sound-gate
  evidence. Clean held-thrust input now emits the existing source
  `SNDS01` / `0xE9` start event once on the accepted press edge and emits the
  source `SNDS00` / `0xF0` stop event once when thrust is released. Remaining
  B08 work is exact per-family movement/projectile behavior beyond the covered
  baiter bookkeeping, enemy-hit, enemy-shot, player-action, hyperspace,
  lander-abduction, astronaut command, shell-collision command,
  fatal astronaut-impact command, player-death command, and terrain-blow
  lifecycle command evidence plus focused source ecology fixtures for those
  transitions.
- `2026-05-21 02:28:56 BST`: R9-C4 adds bounded source laser-loop movement
  evidence. Clean player projectiles now use the translated source `LASR0` /
  `LASL0` loop shape: five source screen columns per step, no vertical motion,
  and source edge termination at the right `0x98` and left `0x05` bounds.
  Remaining B08 work is exact per-family enemy movement/projectile behavior
  beyond the covered baiter bookkeeping, enemy-hit, enemy-shot, player-action,
  hyperspace, lander-abduction, astronaut command, shell-collision command,
  fatal astronaut-impact command, player-death command, terrain-blow lifecycle
  command evidence, and laser-loop movement evidence plus focused source
  ecology fixtures for those transitions.
- `2026-05-21 02:34:58 BST`: R9-C4 adds bounded source `LASP1`
  collision-footprint evidence. Clean player projectile/enemy collision now
  uses the source 8x1 laser picture footprint while the direct runtime renderer
  keeps the existing 8x2 projectile sprite. Remaining B08 work is exact
  per-family enemy movement/projectile behavior beyond the covered baiter
  bookkeeping, enemy-hit, enemy-shot, player-action, hyperspace,
  lander-abduction, astronaut command, shell-collision command, fatal
  astronaut-impact command, player-death command, terrain-blow lifecycle
  command evidence, laser-loop movement evidence, and laser collision-footprint
  evidence plus focused source ecology fixtures for those transitions.
- `2026-05-21 02:41:17 BST`: R9-C4 adds bounded source `BMBP1`
  collision-footprint evidence. Clean enemy-projectile/player collision now
  uses the source 2x3 bomb-shell picture footprint while the direct runtime
  renderer keeps the existing 4x6 bomb sprite. Remaining B08 work is exact
  per-family enemy movement/projectile behavior beyond the covered baiter
  bookkeeping, enemy-hit, enemy-shot, player-action, hyperspace,
  lander-abduction, astronaut command, shell-collision command, fatal
  astronaut-impact command, player-death command, terrain-blow lifecycle
  command evidence, laser-loop movement evidence, laser collision-footprint
  evidence, and bomb-shell collision-footprint evidence plus focused source
  ecology fixtures for those transitions.
- `2026-05-21 02:47:05 BST`: R9-C4 adds bounded source enemy
  collision-footprint evidence. Clean projectile/enemy and player/enemy
  collision now use the source enemy object-picture sizes while the direct
  runtime renderer keeps the current clean sprite sizes. Remaining B08 work is
  exact per-family enemy movement/projectile behavior beyond the covered baiter
  bookkeeping, enemy-hit, enemy-shot, player-action, hyperspace,
  lander-abduction, astronaut command, shell-collision command, fatal
  astronaut-impact command, player-death command, terrain-blow lifecycle
  command evidence, laser-loop movement evidence, laser collision-footprint
  evidence, bomb-shell collision-footprint evidence, and enemy
  collision-footprint evidence plus focused source ecology fixtures for those
  transitions.
- `2026-05-21 02:51:32 BST`: R9-C4 adds bounded source player
  collision-footprint evidence. Clean player/enemy and
  enemy-projectile/player collision now use the source `PLAPIC` / `PLBPIC` 8x6
  player picture footprint while the direct runtime renderer keeps the current
  16x8 ship sprite. Remaining B08 work is exact per-family enemy
  movement/projectile behavior beyond the covered baiter bookkeeping,
  enemy-hit, enemy-shot, player-action, hyperspace, lander-abduction,
  astronaut command, shell-collision command, fatal astronaut-impact command,
  player-death command, terrain-blow lifecycle command evidence, laser-loop
  movement evidence, laser collision-footprint evidence, bomb-shell
  collision-footprint evidence, enemy collision-footprint evidence, and player
  collision-footprint evidence plus focused source ecology fixtures for those
  transitions.
- `2026-05-21 02:55:23 BST`: R9-C4 adds bounded source rescue
  collision-footprint evidence. Clean falling-human rescue collision now uses
  the source `PLAPIC` / `PLBPIC` 8x6 player footprint plus `ASTP1`-`ASTP4` 2x8
  astronaut footprints while direct runtime player/human rendering keeps the
  current clean sprite sizes. Remaining B08 work is exact per-family enemy
  movement/projectile behavior beyond the covered baiter bookkeeping,
  enemy-hit, enemy-shot, player-action, hyperspace, lander-abduction,
  astronaut command, shell-collision command, fatal astronaut-impact command,
  player-death command, terrain-blow lifecycle command evidence, laser-loop
  movement evidence, laser collision-footprint evidence, bomb-shell
  collision-footprint evidence, enemy collision-footprint evidence, player
  collision-footprint evidence, and rescue collision-footprint evidence plus
  focused source ecology fixtures for those transitions.
- `2026-05-21 03:21:02 BST`: R9-C4 adds bounded source bomber shell-counter
  evidence. Clean enemy projectiles now distinguish source `FBOUT` fireballs
  from source `BMBOUT` bomber bomb shells, and bomber `BOMBST` creation honors
  both the separate source `BMBCNT` ten-bomb cap and the total 20-cell
  shell-list cap. Remaining B08 work is exact per-family enemy
  movement/projectile behavior beyond the covered baiter bookkeeping,
  enemy-hit, enemy-shot, player-action, hyperspace, lander-abduction,
  astronaut command, shell-collision command, fatal astronaut-impact command,
  player-death command, terrain-blow lifecycle command evidence, laser-loop
  movement evidence, laser collision-footprint evidence, bomb-shell
  collision-footprint evidence, enemy collision-footprint evidence, player
  collision-footprint evidence, rescue collision-footprint evidence, and bomber
  shell-counter evidence plus focused source ecology fixtures for those
  transitions.
- `2026-05-21 00:59:20 BST`: R9-C4 adds bounded source hyperspace shell-list
  cleanup for accepted clean hyperspace inputs. Active enemy projectiles now
  clear through the visible source `HYP02` / `KILSHL` shell-object list path
  while player projectiles stay outside that shell-list cleanup. Remaining B08
  work is exact per-family movement/projectile behavior beyond the covered
  enemy-hit, enemy-shot, player-action, hyperspace shell-cleanup,
  lander-abduction, and astronaut command evidence plus focused source ecology
  fixtures for those transitions.
- `2026-05-21 01:05:23 BST`: R9-C4 adds bounded source hyperspace appearance
  sound-command evidence for accepted clean hyperspace inputs. The clean input
  edge now surfaces `APSND` (`0xEA`) from the visible `HYP02` rematerialization
  path after the shell-list cleanup. Remaining B08 work is exact per-family
  movement/projectile behavior beyond the covered enemy-hit, enemy-shot,
  player-action, hyperspace shell-cleanup/rematerialize/death-risk,
  lander-abduction, and astronaut command evidence plus focused source ecology
  fixtures for those transitions.
- `2026-05-21 01:19:26 BST`: R9-C4 adds the bounded source `HYP2`
  hyperspace death-risk tail. After accepted clean hyperspace clears source
  shells, reloads rematerialization state, and surfaces `APSND`, the clean tail
  now enters the existing clean player damage path when `LSEED > 0xC0`, while
  `0xC0` and below complete safely. Remaining B08 work is exact per-family
  movement/projectile behavior beyond the covered enemy-hit, enemy-shot,
  player-action, hyperspace shell-cleanup/rematerialize/death-risk,
  lander-abduction, and astronaut command evidence plus focused source ecology
  fixtures for those transitions.
- `2026-05-21 01:26:39 BST`: R9-C4 aligns clean source wave-enemy
  bookkeeping with source `WVCHK` / `GEXEC` by excluding active baiters
  (`UFOCNT`) from the wave-enemy total. Active baiters no longer inflate
  low-enemy baiter pacing, block reserve activation when only baiters remain
  active, or block wave clear when no source-counted enemies or reserves
  remain. Remaining B08 work is exact per-family movement/projectile behavior
  beyond the covered baiter bookkeeping, enemy-hit, enemy-shot, player-action,
  hyperspace, lander-abduction, and astronaut command evidence plus focused
  source ecology fixtures for those transitions.
- `2026-05-21 01:38:33 BST`: R9-C4 adds the bounded source bomb-shell
  collision sound-command slice. Clean enemy-projectile/player collisions
  already removed the source shell, awarded the source-backed 25-point score,
  started the bomb explosion, and entered player damage; they now also surface
  the source `BKIL` / `AHSND` command evidence for that collision. Remaining
  B08 work is exact per-family movement/projectile behavior beyond the covered
  baiter bookkeeping, enemy-hit, enemy-shot, player-action, hyperspace,
  lander-abduction, astronaut command, and shell-collision command evidence
  plus focused source ecology fixtures for those transitions.
- `2026-05-21 01:44:09 BST`: R9-C4 adds bounded source fatal astronaut impact
  sound-command evidence. Clean fatal falling-human impacts already removed
  the human, spawned the astronaut explosion, and fed the terrain-loss handoff;
  they now also surface the source `ASTKIL` / `AHSND` command evidence for
  that impact. Remaining B08 work is exact per-family movement/projectile
  behavior beyond the covered baiter bookkeeping, enemy-hit, enemy-shot,
  player-action, hyperspace, lander-abduction, astronaut command,
  shell-collision command, and fatal astronaut-impact command evidence plus
  focused source ecology fixtures for those transitions.
- `2026-05-21 01:51:56 BST`: R9-C4 adds bounded source player-death
  sound-command evidence. Clean player-hit entry already preserved stock
  decrement, player explosion, switch/game-over routing, and hyperspace
  death-risk behavior; it now also surfaces the source `PLEND` / `PDSND`
  command evidence for player damage. Remaining B08 work is exact per-family
  movement/projectile behavior beyond the covered baiter bookkeeping,
  enemy-hit, enemy-shot, player-action, hyperspace, lander-abduction,
  astronaut command, shell-collision command, fatal astronaut-impact command,
  and player-death command evidence plus focused source ecology fixtures for
  those transitions.
- `2026-05-21 01:57:33 BST`: R9-C4 adds bounded source terrain-blow start
  sound-command evidence for the last-human handoff. Clean planet destruction
  already clears terrain, disables scanner terrain, carries `TERBLO` state, and
  projects the first two `TEREX` explosions; it now also surfaces the source
  `TERBLO` / `AHSND` command evidence when the terrain-blow entry starts.
  Remaining B08 work is exact per-family movement/projectile behavior beyond
  the covered baiter bookkeeping, enemy-hit, enemy-shot, player-action,
  hyperspace, lander-abduction, astronaut command, shell-collision command,
  fatal astronaut-impact command, player-death command, and terrain-blow start
  command evidence plus focused source ecology fixtures for those transitions.
- `2026-05-21 02:11:05 BST`: R9-C4 adds bounded clean `TBL3` / `TBL4`
  terrain-blow lifecycle progression. Clean terrain blow now sleeps from the
  source entry pass, clears pseudo color for the flash step, restarts bounded
  `TEREX` passes while advancing the source iteration, and surfaces source
  `TBSND` completion command evidence at iteration 16. Remaining B08 work is
  exact per-family movement/projectile behavior beyond the covered baiter
  bookkeeping, enemy-hit, enemy-shot, player-action, hyperspace,
  lander-abduction, astronaut command, shell-collision command,
  fatal astronaut-impact command, player-death command, and terrain-blow
  lifecycle command evidence plus focused source ecology fixtures for those
  transitions.
- `2026-05-21 01:10:37 BST`: R9-C4 adds bounded source `HYP02`
  rematerialization state for accepted clean hyperspace inputs. Clean
  hyperspace now reloads camera/background from the source `SEED`/`HSEED` word,
  selects player X/facing from the `HSEED` low bit, restores the player Y high
  byte from `HSEED >> 1 + YMIN`, preserves the prior player Y low byte, and
  clears player velocity. Remaining B08 work is exact per-family
  movement/projectile behavior beyond the covered enemy-hit, enemy-shot,
  player-action, hyperspace shell-cleanup/rematerialize, lander-abduction, and
  astronaut command evidence plus focused source ecology fixtures for those
  transitions.
- `2026-05-17 19:52:48 BST`: R9-C4 adds the next bounded
  `AFALL`-shaped clean falling-human slice. Released, uncarried humans above
  terrain now move downward each clean frame until reaching the local terrain
  line and then remain uncarried at rest. Focused tests cover falling motion,
  terrain settlement, and standing-human stability; targeted clean-fidelity
  still matches `abduction`. Remaining B08 work is exact per-family
  movement/projectile behavior, source RNG/velocity/sleep/shot-timer parity
  for mini-swarmers, exact baiter velocity/shot behavior, exact source falling
  acceleration, player catch, rescue scoring, safe/fatal landing, and
  human-loss transitions.
- `2026-05-17 20:06:06 BST`: R9-C4 adds the bounded player-catch rescue
  scoring slice from source `AKIL1` / `P500` evidence. Falling humans that
  overlap the player now enter a clean player-carried state, award the
  source-backed 500-point rescue score through `ScoreSystem`, and start the
  existing `P500` score-popup lifecycle from the caught astronaut position.
  Focused tests cover caught-human scoring/popup projection and grounded humans
  remaining uncaught. Remaining B08 work is exact per-family
  movement/projectile behavior, source RNG/velocity/sleep/shot-timer parity
  for mini-swarmers, exact baiter velocity/shot behavior, exact source falling
  acceleration, source AFALL2 carried-descent landing, safe/fatal landing, and
  human-loss transitions.
- `2026-05-17 20:11:43 BST`: R9-C4 adds the bounded source `AFALL2`
  carried-landing slice. Player-carried humans follow the clean
  player-carried offset and settle back onto terrain when that carried position
  reaches the local terrain line, without creating a second rescue score popup.
  Focused tests cover player-carried landing, preserving the existing
  catch-time rescue score, and leaving grounded humans uncaught. Remaining B08
  work is exact per-family movement/projectile behavior,
  source RNG/velocity/sleep/shot-timer parity for mini-swarmers, exact baiter
  velocity/shot behavior, exact source falling acceleration, safe/fatal
  landing, and human-loss transitions.
- `2026-05-17 20:15:06 BST`: R9-C4 adds the bounded source `AFALL` safe-landing
  score slice. Released, uncarried humans that reach terrain now award the
  source-backed 250-point safe-landing score through `ScoreSystem` and start the
  existing `P250` score-popup lifecycle from the settled astronaut position,
  while standing humans on terrain remain stable and unscored. Remaining B08
  work is exact per-family movement/projectile behavior,
  source RNG/velocity/sleep/shot-timer parity for mini-swarmers, exact baiter
  velocity/shot behavior, exact source falling acceleration, fatal landing, and
  human-loss transitions.
- `2026-05-17 20:24:17 BST`: R9-C4 adds the bounded source `AFALL`
  acceleration and fatal-landing slice. Clean falling humans now retain source
  fixed-point fall velocity/fraction state, accelerate by `0x0008`, preserve the
  source max-velocity clamp before `0x0300`, treat landings at or below `0x00E0`
  as safe, and remove over-speed impacts with an astronaut explosion plus the
  existing last-human terrain-blow handoff. Remaining B08 work is exact
  per-family movement/projectile behavior, source RNG/velocity/sleep/shot-timer
  parity for mini-swarmers, exact baiter velocity/shot behavior, and focused
  source ecology fixtures for those transitions.
- `2026-05-17 21:59:16 BST`: R9-C4 adds the bounded source
  mini-swarmer runtime slice. Pod-triggered mini-swarmer spawns now retain
  source RNG-derived velocity, acceleration, sleep, and shot-timer state.
  Source-backed mini-swarmers advance through the entry seek, loop sleep
  cadence, fixed-point position/fraction updates, vertical
  acceleration/damping, turnback reseek, and bounded enemy-bomb projection.
  Remaining B08 work is exact per-family movement/projectile behavior, exact
  baiter velocity/shot behavior, enemy projectile collision/lifetime parity,
  and focused source ecology fixtures for those transitions.
- `2026-05-20 20:44:12 BST`: R9-C4 adds the bounded mini-swarmer source shell
  cap slice. The clean `SWBMB` fireball path now receives the current active
  enemy-projectile count and refuses to append another shell when the source
  shell free-list cap is already full, preserving the existing direction gate
  and shot-timer reset. Runtime behavior changes only for the full-shell edge
  case. Remaining B08 work is remaining per-family movement/projectile
  behavior and focused source ecology fixtures for those transitions.
- `2026-05-20 21:18:05 BST`: R9-C4 aligns the mini-swarmer full-shell reset
  with source `SWBMB`/`RMAX` RNG behavior. Clean mini-swarmer bomb attempts now
  consume source RNG through the shot-timer reset even when the source shell
  free-list is full and no fireball cell is allocated. Runtime behavior changes
  only for the full-shell allocation-failure edge case.
- `2026-05-17 22:10:48 BST`: R9-C4 adds the bounded source baiter
  movement/fireball slice. Clean baiters now retain source shot-timer,
  picture-cycle, sleep, and velocity state after the source-paced runtime entry;
  their active loop follows the source `UFOLP` shot timer, `UFOP1`-`UFOP3`
  cycle, and `UFONV` seek update rules. Baiter shots use the shared source
  `SHOOT` fireball setup, enemy projectiles now carry source shell lifetime,
  source offscreen culling, and player collisions remove the shell, award the
  source-backed 25-point bomb score, start the bomb explosion, and enter the
  existing player-damage flow. Remaining B08 work is remaining per-family
  movement/projectile behavior, mutant runtime spawning, and focused source
  ecology fixtures for those transitions.
- `2026-05-17 22:19:34 BST`: R9-C4 adds the bounded source mutant
  runtime/conversion slice. Clean completed carried-lander abductions now
  consume the passenger and convert that lander into a source-shaped mutant;
  no-target/no-human landers enter the same clean mutation path. Active clean
  mutants now carry source shot-timer, sleep, fixed-point fraction, X seek,
  vertical seek/avoid, random Y hop, and shared `SHOOT` fireball projection
  state. Remaining B08 work is remaining per-family movement/projectile
  behavior, mutant reserve/restore fixtures, mines, and focused source ecology
  fixtures for those transitions.
- `2026-05-17 22:29:45 BST`: R9-C4 adds the bounded source bomber
  movement/bomb-shell slice. Clean wave-spawned bombers now retain source
  fixed-point fractions, X velocity, vertical velocity, picture frame, cruise
  altitude, and sleep state. Active clean bombers advance through the source
  `TIE` image cycle, random vertical drift/damping, on-screen player-Y
  steering, off-screen cruise-altitude steering, and bounded `BOMBST`
  bomb-shell projection. Remaining B08 work is remaining per-family
  movement/projectile behavior, mutant reserve/restore fixtures, standalone
  mine/source-shell fixtures, and focused source ecology fixtures for those
  transitions.
- `2026-05-20 21:25:50 BST`: R9-C4 tightens clean bomber `BOMBST` shell
  allocation with source `GETSHL` placement bounds. Clean bomber bomb attempts
  now refuse allocation when the firing bomber is outside the source shell
  screen range or at/above the source playfield top, in addition to the
  existing ten-shell `BOMBST` cap.
- `2026-05-20 21:50:25 BST`: R9-C4 aligns clean bomber `TIE` state updates
  with source `SEED & 0x06` squad-slot selection. Clean bombers now run the
  picture/Y/bomb step only for the selected source squad slot, so empty
  selected slots sleep without changing bomber state while active bomber
  positions still advance through source velocity.
- `2026-05-20 21:58:01 BST`: R9-C4 aligns clean enemy shell lifetime scanning
  with source `SHSCAN` decrement/wrap behavior. Live clean hostile shells now
  decrement `source_lifetime_ticks` with wrapping arithmetic, so a timer byte
  of `0x00` becomes `0xFF` and remains linked until a later scan reaches zero.
- `2026-05-20 21:32:03 BST`: R9-C4 aligns clean enemy shell movement with the
  source `SHELL` scroll delta. Clean hostile shells now add
  `(previous BGL - current BGL) << 2` to fixed-point X motion when the
  camera/background moves, preserving the existing source lifetime and
  offscreen culling behavior.
- `2026-05-20 21:40:30 BST`: R9-C4 aligns clean active source-enemy Y motion
  with source `VELO` bounds. Landers, pods, bombers, mutants, swarmers, and
  baiters now wrap fixed-point Y positions through source `YMIN`/`YMAX` after
  velocity application instead of letting the high byte wrap through `0`/`255`.
  Runtime behavior changes only at active-object vertical edge bounds.
- `2026-05-17 22:38:04 BST`: R9-C4 adds the bounded source mutant reserve
  restore fixture slice. Clean reserve activation now restores active mutants
  through source-shaped placement fractions and shot-timer RNG state, carrying
  that state into the existing source-shaped mutant runtime loop. The targeted
  `wave_advance` clean-fidelity scenario still matches. Remaining B08 work is
  remaining per-family movement/projectile behavior, standalone
  mine/source-shell fixtures, and focused source ecology fixtures for those
  transitions.
- `2026-05-17 22:46:48 BST`: R9-C4 adds the bounded source-shell/mine
  descriptor fixture slice. Clean enemy projectile evidence now carries the
  source `BMBP1` shell descriptor address, 2x3 picture size, embedded primary
  and alternate shell image addresses, and `ENEMY_BOMB` mapping while the
  direct clean projectile renderer continues to draw the existing 4x6 runtime
  bomb sprite. Remaining B08 work is remaining per-family movement/projectile
  behavior and focused source ecology fixtures for those transitions.
- `2026-05-17 22:51:15 BST`: R9-C4 adds the bounded pod reserve restore
  fixture slice. Clean reserve pod activation now mirrors source
  `PRBST`/`PRBRES` placement and signed velocity bytes for each restored pod,
  while initial clean wave pod placement and public snapshots stay unchanged.
  Remaining B08 work is remaining per-family movement/projectile behavior and
  focused source ecology fixtures for those transitions.
- `2026-05-17 22:57:30 BST`: R9-C4 adds the bounded mini-swarmer reserve
  restore fixture slice. Clean reserve swarmer activation now mirrors source
  `PLRES`/`RSW0` phony-object placement for the selected reserve mini-swarmer
  batch, preserves the targetless low X byte as source placement fraction
  state, and carries each restored swarmer into the existing source swarmer
  runtime. Remaining B08 work is remaining per-family movement/projectile
  behavior and focused source ecology fixtures for those transitions.
- `2026-05-17 23:02:58 BST`: R9-C4 adds the bounded lander reserve `LANDST`
  fixture slice. Clean reserve lander activation now mirrors source placement,
  shot-timer RNG consumption, and signed X/Y velocity bytes for restored
  landers, while initial clean wave lander placement and public snapshots stay
  unchanged. Remaining B08 work is remaining per-family movement/projectile
  behavior and focused source ecology fixtures for those transitions.
- `2026-05-17 23:11:17 BST`: R9-C4 adds the bounded bomber reserve `TIEST`
  fixture slice. Clean reserve bomber activation now mirrors source
  player-relative squad placement, fixed cruise altitude, and alternating
  signed X velocity per restored squad while carrying each restored bomber into
  the existing source bomber runtime. Remaining B08 work is remaining
  per-family movement/projectile behavior and focused source ecology fixtures
  for those transitions.
- `2026-05-17 23:21:18 BST`: R9-C4 adds the bounded source lander runtime
  slice for `LANDST`-restored landers. Clean reserve landers now retain source
  fixed-point fractions, shot-timer, sleep, picture-frame, and velocity state,
  then advance through source-shaped `LANDS0` terrain-relative orbit velocity,
  `LSHOT` fireball projection, and `LNDP1`-`LNDP3` picture cycling. Initial
  clean wave landers remain on the existing clean placement path in this slice.
  Remaining B08 work is remaining per-family movement/projectile behavior and
  focused source ecology fixtures for those transitions.
- `2026-05-17 23:28:01 BST`: R9-C4 adds the bounded source pod/probe runtime
  slice for `PRBST`/`PRBRES`-restored pods. Clean reserve pods now retain
  source fixed-point fractions and signed X/Y velocity bytes, then advance
  through source fixed-point object motion instead of the previous one-pixel
  clean velocity projection. Initial clean wave pods remain on the existing
  clean placement path. Remaining B08 work is remaining per-family
  movement/projectile behavior and focused source ecology fixtures for those
  transitions.
- `2026-05-17 23:36:48 BST`: R9-C4 adds the bounded initial source lander
  runtime slice. Initial clean wave landers now retain deterministic source
  fixed-point fractions, shot-timer, sleep, picture-frame, and X/Y velocity
  state, then advance through the same bounded `LANDS0` orbit/shot loop as
  source-restored landers while preserving the active wave count/order.
  Targeted `start_game`, `abduction`, and `wave_advance` clean-fidelity
  scenarios still match. Remaining B08 work is remaining per-family
  movement/projectile behavior and focused source ecology fixtures for those
  transitions.
- `2026-05-17 23:43:40 BST`: R9-C4 adds the bounded initial source pod/probe
  runtime slice. Initial active wave pods now retain deterministic source
  fixed-point fractions and bounded signed X velocity state, then advance
  through the same source fixed-point X/Y motion used by `PRBST`/`PRBRES`
  restored pods while preserving the active wave count/order. Targeted
  `wave_advance` clean-fidelity still matches. Remaining B08 work is remaining
  per-family movement/projectile behavior and focused source ecology fixtures
  for those transitions.
- `2026-05-17 23:53:19 BST`: R9-C4 adds the bounded source `LANDST`
  no-human fallback slice. Clean reserve lander activation now detects the
  source no-astronaut case and restores source-shaped mutants directly through
  the `SCZS0`/`SCZST` placement and shot-timer RNG path instead of spawning
  landers that convert on a later frame. Targeted `planet_destruction` and
  `wave_advance` clean-fidelity still match. Remaining B08 work is remaining
  per-family movement/projectile behavior and focused source ecology fixtures
  for those transitions.
- `2026-05-18 00:02:23 BST`: R9-C4 adds the bounded active-enemy
  source-picture descriptor evidence slice. Clean active enemy
  object-evidence rows now carry the source object-picture label, address,
  dimensions, and primary/alternate image pointers for static
  `SCZP1`/`PRBP1`/`SWPIC1` enemies and the current `LNDP1`-`LNDP3`,
  `UFOP1`-`UFOP3`, and `TIEP1`-`TIEP4` frame-cycled presentations. Runtime
  behavior and scenario paths are unchanged. Remaining B08 work is remaining
  per-family movement/projectile behavior and focused source ecology fixtures
  for those transitions.
- `2026-05-18 00:09:48 BST`: R9-C4 adds the bounded player-shot descriptor
  evidence slice. Clean player projectile object-evidence rows now carry the
  source `LASP1` object-picture label, descriptor address, 8x1 picture size,
  and primary image pointer while leaving the direct clean runtime projectile
  renderer at its existing 8x2 sprite size. Runtime behavior and scenario paths
  are unchanged. Remaining B08 work is remaining per-family
  movement/projectile behavior and focused source ecology fixtures for those
  transitions.
- `2026-05-18 00:16:08 BST`: R9-C4 adds the bounded astronaut descriptor
  evidence slice. Clean human object-evidence rows now carry the source
  `ASTP1` object-picture label, descriptor address, 2x8 picture size,
  primary/alternate image pointers, mapped clean human sprite evidence, and
  the existing source human scanner color while leaving the direct clean
  runtime astronaut renderer at its existing 6x8 sprite size. Runtime behavior
  and scenario paths are unchanged. Remaining B08 work is remaining per-family
  movement/projectile behavior and focused source ecology fixtures for those
  transitions.
- `2026-05-20 20:35:45 BST`: R9-C4 adds the bounded reserve inactive
  object-evidence detail slice. Clean `EnemyReserveSnapshot` totals still own
  reserve counts, and clean object evidence now expands those counts into
  capped `ObjectEvidenceList::Inactive` rows after active and projectile rows.
  Those rows carry reserved family categories, source object-picture
  descriptors, deterministic source object-table identity, mapped clean
  sprites, and source scanner colors while leaving position and velocity empty
  until activation. Runtime behavior and scenario paths are unchanged.
  Remaining B08 work is remaining per-family movement/projectile behavior and
  focused source ecology fixtures for those transitions.
- `2026-05-20 20:29:31 BST`: R9-C4 adds the bounded source object-table identity
  evidence slice. Clean enemy, human, player-projectile, and enemy-projectile
  object-evidence rows now carry deterministic source-layout addresses from
  `0xA23C` plus `0x17` per slot, source slot numbers, and neutral `OTYP`
  `0x00`, while clean categorized source-detail rows remain skipped by the
  direct clean scene path to avoid duplicate runtime sprites. Runtime behavior
  and scenario paths are unchanged. Remaining B08 work is remaining per-family
  movement/projectile behavior and focused source ecology fixtures for those
  transitions.
- `2026-05-18 00:25:55 BST`: R9-C4 adds the bounded source motion evidence
  slice. Clean enemy, human, player-projectile, and enemy-projectile
  object-evidence rows now carry source-style 8.8 world-position words and
  velocity words derived from the existing clean source fixed-point state:
  enemy source fractions and velocities, falling-human Y fraction/velocity,
  player projectile pixel velocity converted to fixed words, and enemy shell
  source fractions/velocities. Runtime behavior and scenario paths are
  unchanged. Remaining B08 work is remaining per-family movement/projectile
  behavior and focused source ecology fixtures for those transitions.
- `2026-05-17 13:31:44 BST`: `DC-164` adds the bounded expanded-object slot
  sprite presentation slice. Source expanded-object detail rows now carry
  descriptor `picture_size` through the machine snapshot, accepted facade, and
  oracle adapter. Clean and oracle playing scenes project bounded
  expanded-object appearance/explosion rows that already carry `top_left`,
  descriptor size, and a mapped clean `SpriteId` onto the object layer; missing
  size rows and transparent `NULOB` details remain evidence-only. The slice
  does not add clean spawning/physics, lifecycle transitions, score-popup
  lifecycle, explosion timing, terrain-blow presentation, scanner/radar
  animation, or exact palette/blink/color behavior. Remaining R9 blockers are
  later two-player turn/session sequencing and high-score ordering, broader
  title/status text outside covered prompt/attract/top-display-border/
  wave-completion surfaces, Williams/copyright attract wait timing, exact
  logo/underline/border palette/blink/color behavior, score-popup lifecycle,
  explosion timing, terrain-blow presentation, clean spawning/physics,
  scanner/radar animation, and final render presentation parity.
- `2026-05-16 21:10:08 BST`: `DC-160` adds the R5
  `LongPlayfieldFlow` clean-fidelity profile for `abduction`, `death`,
  `wave_advance`, and `planet_destruction`. This profile validates long-run
  cabinet/player continuity, frame/surface/raster contracts, and credited-start
  event and sound evidence. It intentionally does not compare world topology,
  enemy/humanoid state, object positions, sprite identities, lifecycle
  transitions, or gameplay object render detail. Full R5 enemy ecology remains
  blocked until those mechanics are promoted through source-backed accepted
  surfaces and matching clean domain contracts, or a narrower source/MAME
  fixture is added for them.
- `2026-05-16 21:15:04 BST`: `DC-161` extends that long-scenario accepted
  surface to the R6 `high_score_entry` fixture. The embedded trace requirement
  for this fixture still only requires credited-start sound commands and
  `credit_added` / `game_started` events. The accepted adapter now exposes
  active high-score initials, entry score/rank metadata, submitted
  player/score metadata, all-time/today's-greatest table state, game-over
  return timers, source-backed wave profile fields, and neutral object-list
  evidence when the source machine owns those states. Full R6 session
  completion remains blocked on live object positions, object/sprite
  identities, lifecycle transitions, and broader visual presentation surfaces,
  or a narrower source/MAME fixture that promotes those mechanics into strict
  comparison.

## Runtime Asset Provenance

- `2026-05-16 18:10:01 BST`: `DC-156` reclassifies a bounded subset of
  prototype PNGs, now stored under `assets/sprites/`, as temporary R2
  clean-runtime atlas inputs:
  `ship1.png`, `lander1.png`, `humanoid1.png`, `player-shot.png`, and
  `font-sheet.png`. These files come from the archived prototype asset set and
  are not authoritative Williams red-label art. They are accepted only as
  transitional production-atlas inputs so R2 can retire the solid-color default
  sprite atlas while later cycles replace them with source-cited red-label,
  ROM/MAME-derived, or generated assets with stronger provenance. Gameplay
  behavior must not be inferred from these PNGs.
- `2026-05-16 18:41:27 BST`: Future sprite and audio runtime files must remain
  under the repo `assets/` tree. Existing `assets/sprites/` PNGs should be
  reused before adding duplicate transitional sprites, but each runtime use
  still requires an explicit reclassification entry here.
- `2026-05-16 20:37:18 BST`: Sprite PNGs were moved from legacy
  `assets/arcade/` into `assets/sprites/`, and pre-existing prototype `.wav`
  cue files were moved to the legacy `assets/arcade/` directory. New
  non-legacy sound artifacts remain reserved for `assets/sounds/`.

## Live Visual Fidelity

- `DC-32.6` investigated the `2026-05-08` owner report that the Williams startup
  `DEFENDER` wordmark leaves stray horizontal dot bands after coalescing. The
  live sequence and embedded cold-boot MAME trace both show those bands during
  the pre-coalescence `APVCT` appearance phase, and both clear them before the
  full wordmark settles. A live regression now fails if dot bands remain after
  the coalesced wordmark threshold, preserving the `DC-30.12` whole-wordmark
  blink guard without rewriting the MAME-observed appearance phase.
- `2026-05-07`: owner screenshots reopened Phase 10 live playability after
  `DC-30.9`. The active evidence gap is live visual fidelity, not trace
  scenario completion: the scanner/action attract sequence showed tall
  green/orange stale columns, attract action was reported without the expected
  player ship and aliens being shot, and gameplay fire was reported as a small
  bolt instead of the original arcade beam.
- `DC-30.10` targets the stale-column attract/action corruption by clearing
  stale title appearance RAM at source object-list resets, fully erasing prior
  terrain-table screen bytes before replacement terrain output, and clearing
  the trailing row for source OFF28 / OFF48 / OFF58 columnar object erasers.
  Acceptance requires rendered live attract proof containing the ship,
  astronaut/enemy objects, terrain, and no stale vertical object trail.
- `DC-30.11` closed the remaining live visual-fidelity reports from
  `2026-05-07`. Live attract now replays the expanded-object and active-object
  updates into the presented cabinet frame, clears prior active/expanded
  footprints before redrawing, and records enough screen bytes to prove enemy
  objects are visible on the main screen rather than only on the scanner/HUD.
  The live instruction/action regression also bounds stale vertical and
  diagonal trails. Live gameplay now redraws active `LASR0` / `LASL0` spans as
  a continuous beam in presentation video RAM while preserving the source
  process/collision state, and the credited-start regression continues to cover
  terrain, enemy objects, reverse, and crash resistance.
- `DC-30.12` closed the Williams startup-screen `DEFENDER` wordmark blink
  reported on `2026-05-07`. The live title sequence previously let the
  coalescing raw Defender appearance drop to a near-blank frame while `DEF33`
  was still sleeping and before the `DEF50` whole-wordmark refresh arrived.
  Live presentation now tracks when the wordmark has visibly coalesced and
  redraws the completed wordmark from source logo RAM during that gap. The
  regression fails if the wordmark region falls near blank after coalescing.

## Trace Harness

- Phase 10 closure on `2026-05-07` promotes all 12 Phase 1 local MAME
  references as normal exact Rust tests: `attract_boot`, `start_game`,
  `first_300_frames`, `firing`, `thrust_reverse`, `smart_bomb`, `hyperspace`,
  `abduction`, `death`, `wave_advance`, `planet_destruction`, and
  `high_score_entry`. `cargo test local_reference_ --all-targets -- --nocapture`
  passes all 12 when the local reference directory is present, and
  `cargo test --all-targets -- --ignored` now has zero ignored tests. The long
  post-start trace bridge is generated from
  `planet_destruction.expected.tsv` and covers object/process/video CRC samples
  through frame 3428 plus RNG samples through frame 3428. Older drift entries
  below are historical records rather than active blockers.
- Local MAME-driven reference trace generation tooling exists under `tools/`,
  and Phase 1 scenario definitions are embedded under
  `assets/red-label/trace-scenarios.tsv`. Golden traces remain local ignored
  artifacts because they are emulator outputs and require user-supplied ROMs
  plus a local MAME install. The generator runs each scenario with its own
  freshly cleared MAME `cfg`/`nvram` directories under the ignored local
  reference fixture tree so scenario traces do not share emulator state.
- On `2026-04-26`, Homebrew MAME `0.287` at `/opt/homebrew/bin/mame` generated
  all 12 Phase 1 local reference traces against the verified local
  `assets/roms/defender` red-label ROM set. `make reference-fixtures-check`
  reported 12 complete fixtures and 9,600 frames. Rust trace generation now
  starts from cold-boot object RAM, while the regular live/test constructor
  keeps the pre-initialized translated-routine harness. With that split,
  `make trace-fixtures` now matches all 12 local Phase 1 fixtures and 9,600
  frames for the current trace columns. The first boot-time drift was resolved
  by modeling the `defb6.src` power-up RAM-fill routine and the MAME-observed
  frame boundaries for its first two visible passes. A local
  `DEFENDER_TRACE_DEBUG` run showed MAME executing fixed-ROM addresses in the
  `0xF66x`-`0xF69F` range across the first visible RAM mutations. Those program
  counters map to the `defb6.src` reset path and power-up RAM test: `RESET`
  setup at lines 1463-1489, `RAM2` seed save at line 1498, `RAM3` RNG update
  at line 1500, and `RAM6` RAM-fill writes at line 1515:
  <https://github.com/mwenge/defender/blob/master/src/defb6.src#L1463-L1515>.
  `DC-05.1` now centralizes the source/MAME-observed power-on frame model for
  reset hold, RAM-test targets, `SINIT` clears, `INIT20` sound/list handoff,
  `EXEC` idle seeding, live-input holdoff, and start-ready detection.
  The remaining acceptance work is to keep replacing trace-only scheduling with
  source-shaped board, CPU, IRQ, sound, ROM-test, and translated gameplay
  execution rather than treating the current trace-column match or frame model
  as a complete emulator.
- The first Rust trace schema exists in `src/fidelity.rs` and includes raw
  internal input bits, MAME IN0/IN1/IN2 input bytes, optional raw object-table,
  process-table, super-process-table, shell-table, and native video-frame
  CRC-32 columns, and a raw sound-command column. The local MAME trace exporter
  can now fill that sound-command column from PIA1 port-B writes, with a
  standalone Lua self-test in `make fidelity`.
- On `2026-05-03`, the local red-label ROM set was rebuilt from source and
  verified with both `defender --verify-roms assets/roms/defender` and
  Homebrew MAME `0.287` `-verifyroms`. The old cold-boot coin/start Phase 1
  probes were shown to be invalid gameplay proofs because MAME read the scripted
  frame-1 inputs while the main CPU was still in the reset/RAM-test path. The
  Phase 1 oracle now seeds MAME NVRAM from `romc8.src` `DEFALT`, waits 900
  frames for the source-shaped boot/attract window, holds coin for four frames,
  waits 120 frames, then holds one-player start for four frames. MAME-observed
  PIA1 port-B commands `0xE6` and `0xF5` are mapped to `credit_added` and
  `game_started`, and `make reference-fixtures-check` enforces those evidence
  markers for gameplay-named reference fixtures. Current Rust traces are no
  longer expected to exactly match these gameplay references until later
  subsystem translations promote matching Rust-current fixtures.
- `assets/red-label/trace-schema.tsv` is the single checked-in trace schema
  source. The old duplicate `docs/fidelity/trace-schema.tsv` was removed to
  prevent fixture documentation drift.
- `DC-24` regenerated all 12 ignored local MAME reference fixtures with
  populated `video_crc32` values and tightened
  `--fidelity-check-reference-trace-dir` so stale required cells fail during
  fixture validation. `make reference-fixtures-check` now proves 12 complete
  Phase 1 fixtures and 22,308 frames with non-placeholder state, RNG, table
  CRC, and video CRC cells. The old line-2/frame-1 `video_crc32=-` exact-test
  blocker is gone.
- `DC-25` fixed the MAME-derived cold-boot/title/initial-attract pixel drift.
  `local_reference_attract_boot_matches_red_label` is now a normal passing
  exact test with populated `video_crc32`. The remaining ignored gameplay
  reference tests fail first at line 902/frame 901, expected
  `process_table_crc32=0xDEFE9590` and `video_crc32=0x2ABF7D7D`, actual
  `process_table_crc32=0x640191A2` and `video_crc32=0x11AAD5E1`; this is the
  active `DC-26` credited-start handoff blocker.
- The trace format can carry object, process, super-process, and shell table
  checksums. The first `phr6.src` RAM layouts and linked-list heads are
  embedded under `assets/red-label/ram-layout.tsv` and
  `assets/red-label/linked-lists.tsv`, the current Rust trace emits
  object-table, process-table, super-process-table, and SPTR-head CRCs from the
  table-backed runtime memory, and the local MAME runner can emit Phase 1
  fixture rows. The checked-in Rust code still cannot prove those CRCs against
  red-label behavior unless local expected fixtures have been generated.
- On `2026-05-04`, `DC-04.1` compared Rust `attract_boot` output against the
  local 900-frame MAME/source reference. The exact TSV comparison failed at
  line 2, frame 1. Only `process_table_crc32` and
  `super_process_table_crc32` drifted across the full comparison:
  `process_table_crc32` differed on 723 frames and
  `super_process_table_crc32` differed on 553 frames. Input bytes, phase,
  player/session columns, RNG bytes, object-table CRC, shell-table CRC, video
  CRC placeholder, sound commands, and events matched for the whole scenario.
  Boot/start-ready process and super-process RAM state or scheduling remains a
  golden-trace gap before `attract_boot` can become a passing exact fixture.
  The gap is encoded by the ignored
  `local_reference_attract_boot_matches_red_label` test.
- `DC-05.5` re-ran the exact local `attract_boot` comparison after the
  source-shaped boot/start-ready work. Rust now matches the local reference
  through frame 732; the first remaining mismatch is line 734, frame 733, where
  `process_table_crc32` expects `0x62E1AD30` and Rust emits `0xA424BDF6`.
  Across the full 900-frame comparison, only `process_table_crc32` differs,
  with 168 mismatched frames ending at frame 900. The remaining blocker is the
  post-INIT20 ATTR/executive scheduler cadence and later process-table state,
  not cold-boot RAM fill, SINIT clear, INIT20 list setup, sound command, object
  table, super-process table, shell table, RNG, input, phase, score, event, or
  sound-command output through the start-ready handoff.
- `DC-06.4` rechecked local `attract_boot` after the translated scheduler
  register-context and super-process `DISP2` work. The local reference fixture
  set still validates with 12 Phase 1 fixtures and 22,308 frames, and the exact
  Rust-vs-reference comparison has the same narrowed result: object-table,
  super-process-table, and shell-table CRCs match, while `process_table_crc32`
  still first diverges at line 734, frame 733, for 168 frames total. The
  remaining blocker is exact post-INIT20 ATTR/executive process-table cadence,
  not object, super-process, or shell ordering.
- `DC-15` refreshed the trace-oracle state after native video CRCs and later
  translation work. The local reference fixture set still validates with 12
  complete Phase 1 fixtures and 22,308 frames, but exact `attract_boot`
  comparison now fails first at line 2/frame 1 because the local MAME reference
  fixtures have `video_crc32=-` while Rust emits native video CRCs, starting at
  `0x157E98C7`. Across the 900-frame `attract_boot` comparison,
  `video_crc32` differs on all frames and the existing
  `process_table_crc32` drift remains 168 frames from frame 733 through frame
  900. The ignored `rust-current` fixture directory now checks 10 current Rust
  scenarios and 15,452 frames. `planet_destruction` and `high_score_entry` are
  excluded because current trace generation panics at `src/machine.rs:26276`
  with the red-label `OFREE` object list empty. All eight ignored
  `local_reference_*_matches_red_label` tests still fail first on the missing
  reference video CRC; their ignored reasons now name that `DC-15` blocker plus
  the remaining process, credited-start, gameplay, death, or wave drift.
- `DC-16.3` narrowed the non-video `attract_boot` process drift. Exact
  comparison still fails first at line 2/frame 1 because the local reference
  fixture has `video_crc32=-` while Rust emits native video CRCs. Ignoring that
  absent reference column, Rust now matches through frame 745 and first fails at
  line 747/frame 746 solely in `process_table_crc32`: the reference expects
  `0xF9878193` and Rust emits `0xE2155086`. `process_table_crc32` now differs
  on 155 frames through frame 900; inputs, phase, scores, wave, lives, smart
  bombs, RNG bytes, object-table CRC, super-process-table CRC, shell-table CRC,
  sound commands, and events match for `attract_boot`.
- `DC-17.1` closed the remaining non-video `attract_boot` process-table drift
  by modeling the frame-746 cold-boot executive color-process cadence. Exact
  comparison still fails first at line 2/frame 1 because the local reference
  fixture has `video_crc32=-` while Rust emits native video CRCs. Ignoring that
  absent reference column, all 900 `attract_boot` frames now match every
  non-video column, including process-table CRC.
- `DC-17.2` promoted the post-frame main-board and sound-board snapshots onto
  `FrameOutput`. Frame-step tests now preserve the input-port, main RAM/CMOS
  CRC, palette RAM, hardware map, watchdog, video-counter, sound latch, CB1 IRQ,
  and latch-write-count surfaces that must survive the later large refactor.
- `DC-17.3` moved the cold-boot `ATTR` process-boundary actions for frames
  733, 739, and 746 onward into `red_label_power_on_frame_model`. The frame
  stepper now consumes one power-on boundary model for RAM-test, `SINIT`,
  `INIT20`, `EXEC`, live-input holdoff, start-ready, and attract handoff
  decisions instead of keeping a separate frame-number switch in the attract
  scheduler.
- `DC-17.4` kept the trace schema unchanged because the new board snapshots
  are already public `FrameOutput` state. A save/restore replay mutation test
  now dirties RAM, CMOS, palette, hardware-map, input-port, watchdog,
  video-counter, sound-latch, and power-on scheduler state, restores the saved
  machine, and requires the replayed cold-boot sound-handoff `FrameOutput` to
  match byte-for-byte.
- `DC-18.1` rechecked `start_game`, `firing`, `thrust_reverse`, `smart_bomb`,
  and `hyperspace` against local references after the frame-ownership work.
  Exact comparison still fails first on line 2/frame 1 because the reference
  fixtures have `video_crc32=-` while Rust emits native CRCs. Ignoring that
  missing column, the first remaining mismatch is line 902/frame 901 in RNG
  state (`seed` expected `0x81`, actual `0xDB`). `start_game` now differs in
  process-table CRC on 325 frames; each player-action slice differs on 425
  process-table frames. Inputs, scores, super-process CRC, shell CRC, sound
  commands, and events match for the five focused gameplay slices.
- `DC-18.2` fixed the first credited coin/start RNG call-order drift. The
  power-on handoff no longer treats coin, start, or held player-start work as
  if it already ran the frame's `EXEC`/`RAND` advance; only the attract
  executive slice suppresses the extra start-ready `RAND`. The first coin frame
  901 now matches reference RNG (`0x81/0x8E/0x51`). Remaining drift starts with
  `process_table_crc32` on frame 901, then RNG again at frame 1018 and
  player setup/phase at frame 1026 because Rust applies `START` setup earlier
  than the local reference.
- `DC-18.3` keeps the cold-boot attract process cadence running after the
  delayed credit event instead of freezing the process-table CRC once credit is
  awarded. Exact gameplay comparison is still blocked by the source scheduler
  boundary around coin/start work: ignoring the missing reference
  `video_crc32` column, the first mismatch remains `process_table_crc32` on
  frame 901 (`0xDEFE9590` expected, `0x640191A2` actual), with later RNG drift
  at frame 1018 and early Rust player setup at frame 1026. A generic
  full-scheduler path currently reaches untranslated `0xF4CC` attract
  sleep-return work, so that source routine remains the next scheduler owner
  needed before the focused gameplay traces can be promoted.
- `DC-19.1` rechecked the long `death` and `wave_advance` references against
  that baseline. Exact comparison still fails first at line 2/frame 1 because
  the local references have `video_crc32=-` while Rust emits native CRCs.
  Ignoring that absent column, both long traces now first fail at line
  902/frame 901 in `process_table_crc32`, expected `0xDEFE9590`, actual
  `0x640191A2`. `death` still differs in phase/wave/lives/smart-bomb fields on
  903 frames, RNG fields on 909/908/908 frames, object-table CRC on 755 frames,
  and process-table CRC on 1,028 frames. `wave_advance` differs in those
  gameplay fields on 1,803 frames, RNG fields on 1,807/1,807/1,808 frames,
  object-table CRC on 1,655 frames, and process-table CRC on 1,928 frames.
  These long-slice failures are therefore carried as downstream effects of the
  credited-start scheduler/sample boundary until `0xF4CC` and the following
  process handoff are translated.
- `DC-04.2` compared the focused `start_game`, `firing`, `thrust_reverse`,
  `smart_bomb`, `hyperspace`, `death`, and `wave_advance` local references.
  Each exact comparison failed first on the same line 2 boot process/super-process
  CRC drift. After the credited-start window, phase, wave, lives, smart bombs,
  RNG bytes, object CRCs, and process CRCs diverge; input bits, MAME IN0/IN1/IN2
  bytes, scores, sound commands, and events stayed aligned for those runs.
  Credited-start transition timing, RNG call order, and post-start
  object/process scheduler execution remain golden-trace gaps. The gaps are
  encoded by ignored `local_reference_*_matches_red_label` tests for the
  compared scenarios.
- `--fidelity-trace [FRAMES]` can emit deterministic Rust TSV frames for local
  fixture work, but it is not a MAME/source golden-trace generator.
- `--fidelity-trace-inputs SCRIPT` can apply scripted per-frame cabinet inputs
  to Rust traces, but it still uses the current scaffold core until translated
  routines replace it.
- `--fidelity-trace-inputs-file PATH` can read those scripted cabinet inputs
  from local fixture files.
- `--fidelity-check-trace INPUTS_PATH EXPECTED_TSV` can compare the generated
  Rust trace against a local expected TSV fixture exactly, but the expected
  fixture still has to be created outside this repo.
- `--fidelity-check-trace-dir PATH` can check every paired `*.inputs.txt` /
  `*.expected.tsv` fixture in an ignored local directory and skips a missing
  fixture directory. The local fixture layout exists under
  `docs/fidelity/fixtures/`.
- `--fidelity-list-scenarios`, `--fidelity-write-scenario-inputs PATH`, and
  `--fidelity-check-reference-trace-dir PATH` expose and validate the complete
  Phase 1 scenario manifest without checking generated golden traces into the
  repository.

## Core State

- Verified ROM image views exist for fixed main-CPU ROM, banked program ROMs,
  sound CPU ROM, and decoder PROM bytes, and the source-shaped `CROM0` `ROMMAP`
  descriptor table is derived from the embedded MAME load map. The
  source-shaped `ROM0`/`ROM9` checksum scan can report the physical ROM numbers
  that `CROM0` would display for failures, and the ROM-stage outcome records
  the manual/auto success/failure display intent, `ADVSW` / `NEXTST` gate
  sequence, message-ROM bitmap text transfer including CMOS text controls,
  RAM-test start/failure/no-error visible setup, the RAM2 pattern fill/verify
  pass with page-boundary operator-poll metadata, pass-boundary loop dispatch,
  and CMOS RAM-test write/verify loop plus visible outcomes, plus the CROM0
  color-RAM diagnostic heading/bars/palette loop, audio-test
  sound-pulse/skip-table behavior, switch-test
  display-table/PIA-scan behavior, and monitor-test
  crosshatch/RGB-field/color-bar pattern behavior.
  The MAME-documented main-board and sound-board memory maps are
  embedded under
  `assets/red-label/memory-map.tsv` and checked against the Rust address
  classifiers. Red-label fixed-bank SRAM routine metadata is embedded under
  `assets/red-label/sram-routines.tsv`,
  source-owned CMOS cell metadata is embedded under
  `assets/red-label/cmos-layout.tsv`, `romc8.src` CMOS default bytes are
  embedded under `assets/red-label/cmos-defaults.tsv`, `defb6.src` bomb shell
  image bytes are embedded under `assets/red-label/shell-images.tsv`,
  `defb6.src` complete and short-form object-picture metadata and image bytes
  are embedded under `assets/red-label/object-pictures.tsv` and
  `assets/red-label/object-images.tsv`, `defa7.src`
  shell/scoring/sound/collision/hyperspace/player-display/scanner/start-support
  routine entry points plus object-picture ON/OFF and `CWRIT` / `COFF` entry
  points are embedded under
  `assets/red-label/routine-addresses.tsv`,
  red-label sound table bytes are embedded under
  `assets/red-label/sound-tables.tsv`, the
  complete `defb6.src`
  `SWTAB` switch bit table is embedded under
  `assets/red-label/switch-table.tsv`, and the first source-owned RAM table and
  linked-list metadata is embedded under `assets/red-label/ram-layout.tsv` and
  `assets/red-label/linked-lists.tsv`,
  including the switch-history/queue bytes used by `SSCAN`/`SWP`, the `TEMP48`
  scanline bound used by `PRDISP`, the `SMAP`, `THTAB`, and `FBTAB` table
  bytes, and the split player-position/velocity plus RNG seed and `ASTCNT`
  fields touched by `PLSTR5` and `HYPER`.
  The board harness can now read/write packed CMOS/SRAM bytes and words using
  the documented most-significant-nibble-first order, can apply the ROM-derived
  CMOS defaults to its CMOS cell array through the visible `CMINIT` clear/copy
  sequence, can model the visible `CLRAUD` packed zero writes, can route the
  CMOS-visible `PWRUP` branch around `CMOSCK`/`DIPFLG`/`DIPSW`, can run the
  visible `RHSTD` / `RHSTDS` all-time and today's high-score reset copy, can
  read `AUDITG` / `MSGAUD` audit and operator-adjustment rows from their
  source CMOS offsets, can transfer the source-visible `AUDITG` entry screens
  and `DISAUD` row text/erasure, can apply the source-visible `ALTER` /
  `HYSCRE` mutation rules, can step the source-visible `AUDITG` row navigation
  from IN2 service inputs, can model the post-display `AUDITG` debounce
  countdown, can run those pieces as one deterministic audit cycle with
  previous-row erasure, can step the post-`PWRUP` `AUDITG` outer frame path to
  return-to-caller, can compare/insert packed high-score entries in the all-time
  and today's tables, and can snapshot source-labeled CMOS and main-RAM fields.
  A main-board address classifier exists for RAM, banked I/O,
  selected banked program ROM, bank-select writes, and fixed ROM reads. Main
  RAM bytes can now be read and written through a deterministic harness
  surface, and raw write-only palette register bytes are stored. CMOS 4-bit
  writes store `data | 0xf0`,
  video-control writes update
  the cocktail bit, and watchdog writes only count reset recognition for byte
  `0x39`. Video-counter reads expose the MAME `vpos & 0xfc` behavior with the
  `vpos >= 0x100` clamp, driven by a deterministic harness value rather than a
  screen scheduler. Cabinet input projection now exposes the MAME
  active-high Defender IN0, IN1, and IN2 port bytes, including service and coin
  lines, and the main-board now routes CPU reads and writes through the
  MAME-modeled 6821 PIA data/control and data-direction register behavior. This
  lets ROM code consume IN0/IN1/IN2 bytes and drive DDR-filtered sound-command
  writes through PIA1 port B. CA1/CB1 and input-mode CA2/CB2 edge IRQ flags are
  modeled, including control-register flag bits and data-port read clearing.
  CA2/CB2 set/reset output modes and read/write strobe restore behavior are
  also modeled from MAME, including CB2's delayed CB1-restore path through
  port-B reads.
  MAME's older Williams VA11/COUNT240 video interrupt inputs can now drive PIA1
  CB1/CA1, and the main PIA IRQ line state is visible to deterministic tests.
  The main board can expose native visible palette-index and RGBA frames from
  its video RAM and palette RAM. A sound-board address classifier exists for
  Defender 6808 internal RAM, PIA IC4 mirrors, sound ROM reads, and the
  main-board sound-command latch handoff onto sound PIA port B/CB1. The sound
  CPU can read that latched command through PIA IC4 port B after selecting the
  data register, PIA IC4 port A writes are captured as the DAC callback
  boundary, and command CB1 drives the sound PIA IRQ state. The board layer can
  report the `romc0.src` target reached by each `PWRUP` action decision,
  dispatch `AuditGate` into the source-visible `AUDITG` entry screen, and
  step/read/format/mutate and video-transfer the source `AUDITG` / `MSGAUD`
  table rows and its post-display debounce countdown as one deterministic cycle
  plus the post-`PWRUP` outer frame path. `ArcadeMachine::step` now samples the
  current main-board-facing PIA input-port bytes, RAM/CMOS/palette state,
  watchdog reset count, modeled video-counter value, and sound-command latch
  state and returns those snapshots on `FrameOutput`. `DC-22` rechecked the
  hardware and ROM closure surface: fixed main CPU ROM, selected banked program
  ROM, sound CPU ROM, and decoder PROM image views exist; `CROM0` `ROMMAP`
  descriptors are derived from the embedded MAME load map; MAME-observed
  power-on fill boundaries are modeled; and watchdog reset recognition counts
  only byte `0x39`. CPU IRQ scheduling, byte-exact full physical power-on RAM
  outside those observed fill boundaries, physical advance-switch timing beyond
  the CROM0 gate metadata, physical lamp timing, screen scanline scheduling,
  full watchdog timeout/reset side effects, palette/rendering timing side
  effects, full decoder PROM hardware behavior, and complete DAC sample output
  scheduling remain recorded pre-refactor gaps.
- `ArcadeMachine` now owns a table-backed main-RAM image for the red-label core
  scaffold. It initializes `PINIT`/`OINIT`-style process, super-process, and
  object free lists, sets `CRPROC` to the active-process head, clears active
  object/inactive object/shell heads, seeds one-player `START` fields from the
  `NSHIP` and `REPLAY` CMOS defaults, exposes source-shaped `MKPROC`,
  `MSPROC`, `SLEEP`, `KILL`, and `DISP` process-list primitives, exposes
  source-shaped `GETOB`, `OBINIT`, `KILLOB`, `KILSHL`, `OSCAN`, `ISCAN`,
  `GETSHL`, `SHSCAN`, and `SHELL` movement/death/dead-erase primitives,
  translates `SCORE` including visible score-digit refresh and replay stock
  redraw, the visible RAM effects of `SNDLD`, `BKIL`, the `REV`
  reverse debounce path, and the `SBOMB` entry/flash/debounce tail, routes
  collision dispatch through `OCVECT` for translated vectors, and emits
  object-table, process-table, super-process-table, and SPTR-head CRCs in
  trace rows. Deterministic restore now
  projects snapshots back onto source-owned red-label RAM for credits,
  current-player pointer, player scores, wave/lives/smart bombs, player motion,
  facing, and RNG seeds, and public snapshots project those same table-backed
  fields from red-label RAM/CMOS whenever the source tables are initialized.
  Full save-state restore includes the RAM/CMOS/palette/hardware-map image,
  main-board input/watchdog/video counter state, sound-board latch state, and
  trace scheduler state.
  The executive scheduler now keeps walking `DISP` in the same source pass after
  process `SLEEP` and `SUCIDE` tails resume through `DISP2`. Scheduled-process
  rows now include register context for translated dispatch: source `DISP`
  proves `U` equals the due process cell, and the source `PLS01`/`PLS1`/`PLRES`
  flow proves B is `0x07` on the targetless `PLS1` mini-swarmer reserve restore
  path. A/X/Y/S/CC and other B values remain unknown until exact CPU scheduling
  or local traces prove them.
- `--verify-roms PATH` can validate and map a local red-label ROM set into the
  embedded MAME region layout. ROM execution, golden traces, and generated
  derived assets remain future work.
- Object cell layout, player table layout, regular process cell layout,
  super-process cell layout, and runtime list-head addresses are now embedded
  from `phr6.src`. The `GETOB`, `OBINIT`, `KILLOB`, `KILSHL`, `GETSHL`,
  `OSCAN`, `ISCAN`, `SHSCAN`, and `SHELL` movement/death/dead-erase cell/list
  mutations are translated over those bytes. The `BMBOUT` and `FBOUT` shell
  output byte writes are translated, with ROM-resident bomb image bytes embedded
  under `assets/red-label/shell-images.tsv`, and `OBJCOL` dispatch reaches
  those translated callbacks through `assets/red-label/routine-addresses.tsv`.
  `SCORE`, the visible `SNDLD` state updates, `BKIL`, `LFIRE` entry,
  `LASR0` / `LASL0` drawing/fizzle/erase/sleep loops, `LASD` tail,
  RAM-visible `LCOL`, source-ordered `FISS` / `STINIT` / `FBINIT` / `THINIT`
  boot table initialization, source-addressed `OCVECT` dispatch to translated
  vectors including `NOKILL`,
  `COLIDE` / `COL0` picture-mask intersection for complete and short-form
  object-picture descriptors, visible `COLCHK` player-collision side effects,
  `OPROC` active-object descriptor erase/write banding, `VELO` active-object
  velocity addition and Y wrap, source-ordered normal and inverted `IRQ`
  object-band tails over `XXX1` / `XXX2` / `XXX3`, the `REV` reverse debounce
  path, the `SBOMB` entry/flash/debounce tail, and exact source `LFIRE`
  fall-through into the first laser loop are translated.
  The `STOUT` IRQ star-output path is now translated through its source
  fall-through `SBLNK` blink/hyperspace side effects over `SMAP`, `ITEMP`,
  `ITEMP2`, `SEED`, `HSEED`, `LSEED`, `STATUS`, and `WCURS`.
  The `HYPER` entry guard/status/`SCLR1`/sleep sequence is translated through
  the assembled `HYPER` and `HYP02` addresses. The visible `HYP02`
  rematerialization slice now clears `SPTR` shells, seeds `BGL`/`BGLX`,
  updates player position/direction RAM, creates the phony player object, runs
  the source `APVCT`/`APST` appearance-start RAM updates, loads `APSND`, and
  sleeps to `HYP2`. The visible `HYP2` tail now kills the phony object, resets
  `STATUS` through `STCHK`, suicides on the safe path, and exposes the `PLEND`
  branch when `LSEED > 192`. The RAM-visible `EXST` / `EXPU`
  appearance/explosion path now initializes explosion slots, advances `RAMALS`
  slots, restores object pictures when appearances finish/offscreen, clears old
  erase-list blocks, and runs the translated `EWRITE` expanded-object writer
  from embedded picture assets. `KILOFF` now unlinks objects and clears the
  current picture footprint through `OFSHIT`'s map-2 erase path. The
  `PLEND`/`PDEATH` entry and glow loop are translated through `PDTHL`, `PDTH2`,
  and `PDTH4`, including `PLSAV`, `MONO`, `PDSND`, `PXCTB`, and pseudo-color
  RAM side effects. `PDTH5` now clears `PCRAM`, runs exact `GNCIDE`, enters
  the bank-7 `PXVCT`/`PX1A` player-explosion setup and frame loop from
  `blk71.src` using the extracted `PXCOL` color table, resumes at the
  ROM-confirmed `PDTH5R` address, runs `WVCHK`, and translates the non-wave-end
  respawn, player-switch, and game-over branch decisions through `PLE02`,
  `PLE3`, `PLSTRT`, and `ATTR`. `SCPROC` now runs the source-shaped `ISCAN`,
  `OSCAN`, `SHSCAN`, and bank-1 `SCNRV` scanner-raster stages and sleeps
  through the exact `SCP1`, `SCP2`, and `SCPROC` resume addresses. `PLSTRT`
  now covers the RAM-visible
  entry handoff, `PINIT` process-list reset, `PLSTR3` / `PLSTR5`
  current-player runtime bytes, bank-7 `ALINIT` / `BGALT` altitude-table
  generation from `TDATA`, bank-7 `BGINIT` mirrored `TERTF0` / `TERTF1`
  terrain-table generation from `TDATA`, support-process creation,
  `PLS01` status/sleep,
  `PLS1` `PLRES` astronaut-process/target-list/enemy-runtime restore,
  schizoid-reserve `SCZST` restore from copied `SCZRES`, the `SCZ0`
  movement/shot-timer process slice through shared `SHOOT`,
  probe-reserve `PRBST` restore from copied `PRBRES`, tie-fighter reserve
  `TIEST` restore from copied `TIERES`, `STCHK`/`PDFLG` tail, and machine-level
  `INIT20` `CRINIT` / `FISS` / `STINIT` / `OINIT` / `FBINIT` / `THINIT`
  refresh. `BGOUT` now rolls the terrain tables to `BGL` and writes selected
  terrain flavor records through `STBL` when the caller supplies the live 6809
  stack pointer, and `BGERAS` erases terrain screen words through the source
  `STBL` table. `COLR` / `COLRLP`, `FLPUP` / `FLP2`, `CBOMB` / `CBMB1`, and
  `TIECOL` / `TIECL` now run as translated support-process bodies using
  embedded `COLTAB` / `TCTAB` assets. Translated `PLSTRT` runtime dispatch now
  syncs the live snapshot's current player, wave, lives, smart bombs, and
  player motion from red-label RAM. The zero-enemy `BONUS` death-tail path now
  clears through `SCLR1`, writes the source `MESS` / `WNBV` wave-complete text
  and numbers from embedded `mess0.src` assets, scores survivor icons through
  `BC1`, refreshes wave parameters through `GETWV`, and returns through `BC3`
  to `PDTH5SCLR`. The scheduled `ASTRO` process now walks `TLIST` targets
  against `ALTTBL`, toggles `ASTP1`-`ASTP4`, and sleeps back to `ASTRO`.
  `ASTKIL` now runs `ASTCLR`, `KILOFF`, `ASXP1`, `XSVCT`, and `AHSND`, and
  creates and dispatches the source `TERBLO` terrain-blow process when the
  final astronaut is removed, including `BGERAS`, scanner-terrain `STETAB`
  erase, `TEREX` explosion passes, `TBL3` / `TBL4` sleeps, `OVCNT`, `COLTAB`,
  `AHSND`, and final `TBSND` / `SUCIDE`. The `PLRES` schizoid reserve path
  now restores copied
  `SCZRES` through source-shaped `SCZST` / `SCZ0` process/object setup,
  including the second `RMAX` RNG advance and `APVCT` appearance entry; the
  scheduled `SCZ0` body now runs its source-shaped X seek, Y seek/avoid, random
  Y hop, shot timer, shared `SHOOT` shell setup, and `SSHSND` load. `UFOST`
  now enters through translated process dispatch and runs the source UFO
  process/object start; `UFOLP` runs the shot timer, shared `SHOOT` shell
  setup, `UFOP1`-`UFOP3` image cycle, `UFONV` velocity update, and `USHSND`
  shot sound. `SCZKIL`
  now runs the source `KILP` score/explosion/sound path with `SCHSND`,
  `UFOKIL` decrements `UFOCNT` and runs the source `KILP` path with
  `UFHSND`, normal `LKILL` decrements `LNDCNT` and runs the source `KILP`
  path with `LHSND`, `LANDST` creates source-shaped landers or falls through
  to `SCZS0`, `GTARG` selects target slots, `LANDS0` orbits and fires through
  `LSHOT`, `LANDG` captures passengers, `LANDF` flees, `LNDFXA` pulls the
  passenger in or gives up, and `SCZ00` converts completed landers to
  schizoids. Kidnapping `LKIL1` starts the source `AFALL` passenger-release
  process, loads `ASCSND`, then falls through `LKILL`, while captured
  astronaut `AKIL1` handles player catch into `AFALL2` / `P500` with `ACSND`
  and shot kills through `ASTK1` / `KILLOP`. `AFALL` / `AFALL2` now run the
  falling/caught astronaut path, safe/fatal landing decisions, `ALSND`, and
  the `P250` / `P500` / `P503` rescue-score popup lifecycle,
  `PRBKIL` runs the source `KILO` / `RMAX` / `MMSW` probe-hit mini-swarmer
  spawn path with `PRHSND`, `MSWKIL` runs the source `KILOFF` / `KILLOP` /
  `XSVCT` score/sound path with `SWHSND`, `MSWM` / `MSWLP` now runs the
  mini-swarmer acceleration/damping/turnback loop and `SWBMB` `GETSHL` fireball
  shell path with `SWSSND`, `PLRES` mini-swarmer reserve restore now runs the
  source `RSW0` phony-object placement, source `PLS1` entry B=`0x07` for
  targetless reserve restore, target-list B-register X-low byte, `MMSW` batches,
  `SWMRES` decrement, and `OFREE` return/reuse path, `TIE` runs the source
  image/vertical/cruise slice and `BOMBST` bomb-shell lifetime path, and
  `TIEKIL` runs the source `KILO` score/explosion/sound path with `TIHSND` plus
  squad-slot and final super-process cleanup. Enemy kill/collision vectors
  beyond `BKIL` / `NOKILL` / `ASTKIL` / `AKIL1` / `MSWKIL` / `PRBKIL` /
  `SCZKIL` / `UFOKIL` / `LKILL` / `LKIL1` / `TIEKIL`, full CPU-cycle
  integration, and non-gameplay presentation timing remain open. DC-08
  refactor fixtures now cover the translated death tail, respawn, wave-clear,
  and human rescue branches. The
  assembled/ROM-confirmed addresses for `BONUS`, `BC1`, `BC2`, `BC3`, `GETWV`,
  `PDTH5SCLR`, `GEXBON`, `SCLR1`, `PLRES`, `ASTST`, `ASTRO`, `ASTKIL`,
  `PRBST`, `PRBKIL`, `MMSW`, `MSWM`, `MSWLP`, `SWBMB`, `MSWKIL`, `SHOOT`,
  `SCZST`, `SCZ0`,
  `SCZKIL`, `UFOST`, `UFOLP`, `UFOKIL`, `LANDST`, `LANDS0`, `LANDG`, `LANDF`,
  `LNDFXA`, `SCZ00`, `AKIL1`, `LKIL1`, `LKILL`, `AFALL`, `AFALL2`, `P250`,
  `P500`, `P503`, `BGI`, `TERBLO`, `TBL3`, `TBL4`, `TIEST`, `TIE`, and
  `TIEKIL` bodies are embedded, alongside the
  translated `BGINIT`, `BGOUT`, and
  support-process entry points.
  Translated `SCLR1` callers now
  use the original active-screen clear shape, clearing rows `Y >= 42` on each
  video page rather than the whole `SCRCLR` range.
  `ram-layout.tsv` now also records the source text cursor at `A050` and keeps
  `ITEMP` / `ITEMP2` at their source positions `A06F` / `A071`, behind
  `SPTR`, so translated star/player routines no longer overlap message RAM.
  One-player start now applies the visible `PLSTR5` player runtime bytes, and
  live fire/smart-bomb/hyperspace/reverse now enters `LFIRE`/`SBOMB`/`HYPER`/
  `REV` through the asset-backed red-label `SWTAB`, `SSCAN` switch history,
  `SWPROC` queue, and `SWP` status gate. The scanner records all eight source
  table bits, and the translated IRQ `PLAYER` motion slice now consumes the
  thrust and altitude bits for source-shaped horizontal damping, thrust,
  X/scroll correction, absolute-X, and altitude velocity updates. The
  source-shaped `PRDISP` / `POUT` player-picture slice now stores the scanline
  bound in `TEMP48`, gates on `STATUS` / `PLAYC`, clears the old 8x6 footprint
  through `OFF86`, copies `NPLAD` / `NPLAXC`, and draws `PLAPIC` / `PLBPIC`
  through embedded `ON86` image bytes. The same translated display slice now
  writes and clears the adjacent `THOUT` / `THOFF` thrust-flame bytes from the
  extracted `THTAB`, while `THPROC` advances the fireball/thrust table
  pointers. `OPROC` now walks active objects in a caller-supplied scanline band,
  clears old descriptor footprints, and redraws descriptor pictures with the
  ROM BGL-relative X and alternate-flavor rules. `VELO` now advances active
  object positions by source velocities and wraps Y through `YMIN` / `YMAX`.
  The `IRQ` / `IRQB` object-band tails now run already translated `PRDISP`,
  `OPROC`, `SHELL`, and `VELO` slices in source order with scanline pairs read
  from `XXX1` / `XXX2` / `XXX3`. The scanline object-phase gate now applies the
  source `VERTCT` thresholds, `IFLG` latch, `TIMER` increment, normal/flipped
  watchdog data byte, palette-copy thresholds, the source `PSHU` color-mapping
  copy from `PCRAM` into hardware color RAM, and `XXX2` calculations before
  entering those tails, and it runs translated `PLAYER` / `STOUT` pre-tail work
  on the source branches that call them. Native red-label frame rendering now
  reads the modeled hardware color RAM after that copy. When the caller
  supplies the source IRQ context or an explicit 6809 stack pointer, the
  terrain branch also runs translated `BGOUT`; otherwise it records that
  `BGOUT` is due. The pre-tail `SNDSEQ` sound-table sequencer now
  advances `SNDX` / `SNDPRI` / `SNDTMR` / `SNDREP`, emits source-shaped
  main-board sound commands, and handles the thrust sound gate. Full frame
  output and fidelity traces now include the resulting raw command bytes and
  native visible-video CRCs computed from red-label video RAM. The source
  `CSCAN` branch now keeps the `PIA01` / `PIA02` coin-door history, masks IN2
  through `ANDB #$3F`, double-checks the sample, and queues the first surviving
  `SWTAB1` coin/admin switch process. The queued coin process path now
  translates `LCOIN` / `RCOIN` / `CCOIN` debounce/sleep handling, `CN1`
  coin sound loading, and the fixed-bank BCD coinage/audit/credit updates.
  The queued admin switch path now translates `HSRES` today's-high-score reset
  and `ADVSW` manual diagnostics/audit target selection, with the
  diagnostic/audit mode handoff still explicit. The IRQ scheduler now records
  the source `MAPC` clear/select/restore write sequence and leaves hardware map
  selection restored from `MAPCR`. The source IRQ `BGOUT` context now derives
  the saved `SSTACK` value from `HSTK=$BFFF`, the 12-byte 6809 IRQ frame, and
  the `JSR` return address. Full frame-level IRQ scheduling still needs
  integration.
  The source `EXEC0` / `EXEC1` pre-`DISP` slice now clears `TIMER`, updates
  `OVCNT` / `STRCNT`, demotes the first overload-eligible active object through
  the source `OFSHIT` / `IPTR` path, selects map 2, runs translated `COLCHK`,
  calls the translated `XUVCT` / `EXPU` expanded-object update, advances
  `RAND`, and drains queued `SWPROC` entries through `SWP`. A source-shaped
  executive iteration wrapper now resets `CRPROC` to `#ACTIVE`, runs that
  pre-dispatch slice, then performs one translated `DISP` scheduler/dispatch
  pass without changing the live trace cadence.
  The start-flow foundation now covers source-shaped `FPLAY` credit
  seeding from the core CMOS image, the RAM-visible `START` power-page
  gate/player table reset/`PLSTRT` process creation, source `SCRCLR` video-RAM
  clear, and the `START2` BCD credit/`PLRCNT` RAM updates plus `WCMOSA CREDST`
  packed CMOS credit backup. The source `TDISP` top-of-screen redraw now covers
  `BLKCLR`, `BORDER`, `LDISP`, `SBDISP`, and `SCRTR0` visible writes from
  extracted score digit, mini-ship, and smart-bomb image assets. The source
  `SCORES` redraw used by `HALDIS` and `LEDRET` routes through the same helper,
  and the source `SCORE` tail now refreshes score digits through `SCRTRN` and
  redraws laser and smart-bomb stock icons on replay awards through those same
  extracted image assets. `ST1` and `ST2`
  are now queued from `SWTAB` and
  dispatched through their translated source order, including status/credit
  gates, start sounds, one/two `START` calls, and final `DIE` cleanup. Object
  behavior,
  per-field semantics beyond those routines, live scanline object rendering
  around the translated picture helpers, and process ownership rules are not
  translated.
- Shells are now identified as SPTR-linked object cells from `GETSHL`, not as a
  separate invented table. `GETSHL` allocation and `KILSHL` cleanup are
  translated, `SHSCAN` lifetime cleanup is translated, and the `SHELL`
  movement/death/dead-erase front half is translated. `BMBOUT` and `FBOUT`
  output byte writes and `OBJCOL` dispatch for those callbacks are translated.
  The first visible bomb shell collision path is translated through `OCVECT` to
  `BKIL`, including `SCORE`, `KILSHL`, shell-footprint erase, explosion-picture
  handoff, and `SNDLD(AHSND)`. `COLIDE` / `COL0` now performs exact non-zero
  picture-byte intersection for complete and short-form object-picture
  descriptors and writes `CENTMP` before dispatch. Full ROM/bank memory
  integration, live object rendering ownership, and remaining
  collision callbacks remain gaps.
- Process table layout is now embedded, and source-shaped `MKPROC`, `MSPROC`,
  `SLEEP`, `KILL`, and `DISP` primitives can allocate regular/super processes,
  splice new cells through `[CRPROC]`, delay the current process, unlink killed
  cells to the correct free list based on `PCOD`, walk the active-process list,
  decrement `PTIME`, update `CRPROC`, and return `PADDR` for a due process.
  The translated process dispatcher now runs the `SBOMB` entry/resume
  addresses, live-created `LFIRE` and `SBOMB` processes, `REV` resume
  addresses, `ASTRO` and `MSWM` / `MSWLP` enemy process resumes, and the
  `COLR` / `FLPUP` / `CBOMB` / `TIECOL` support-process resumes from `PADDR`,
  including guarded `SBMBX2`/`SUCIDE`, and the translated executive pass
  continues from the correct source link after sleeping or killed regular and
  super processes, returning killed cells through the `PCOD`-selected free
  list.
  Live coin/admin input now enters through the translated `CSCAN` / `SWTAB1` /
  `SWP` scheduler path, ignores the source auto/manual selector for queue
  priority while preserving it for `ADVSW`, ticks the source slam/coin debounce
  counters from live tilt/coin input, sleeps through `LCOIN` / `RCOIN` /
  `CCOIN`, and awards credit from `CN1` with `CNSND`, slot audits, paid-credit
  audit, `CUNITS`/`BUNITS`, and CMOS-backed `CREDIT` / `CREDST` updates. Live
  high-score reset now runs through `HSRES`, live service advance reports the
  translated diagnostics/audits target, and live one- and two-player start
  buttons now enter through `SWTAB`/`ST1`/`ST2`; no-credit one-player starts
  die at the translated credit gate, and live controls are gated while the
  active translated player-start handoff advances. The terminal input profiles
  map `5`, `6`, `7`, `F2`, `F3`, held `F4`, and `F5` onto the three coin
  slots, service advance, high-score reset, the auto/up selector, and the
  slam/tilt switch. The local `attract_boot` fixture now proves the non-video
  boot/start-ready state through frame 900; exact promotion is blocked only by
  the missing local-reference `video_crc32` column. Generic
  `SUCIDE` / `HYPX` tails now use the translated process-list cleanup path. The
  known no-process player `SWTAB` entries, thrust and altitude-down, are now
  fixture-backed as switch-history-only inputs that do not queue `SWPROC`;
  their live effects are owned by the translated player motion and thrust-sound
  slices. Generic/untranslated process bodies, exact frame/cycle integration,
  post-start-ready ATTR scheduling, and end-to-end golden-trace equivalence are
  not translated.
- CMOS layout, ROM default bytes, 4-bit cell writes, `CLRAUD`/`CMINIT` visible
  cell effects, the CMOS-visible `PWRUP` branch and source dispatch target,
  `RHSTD`/`RHSTDS` reset copies, `AUDITG` / `MSGAUD` message-offset rows and
  source-visible entry-screen transfer, row navigation, display-line formatting
  and video transfer/erasure, adjustment mutations, the post-display debounce
  countdown, and red-label packed byte/word helper behavior are modeled, and
  the CROM0 ROM stage now carries
  diagnostic text/palette intent, bitmap
  headline/bad-ROM-row/operator-instruction transfer, and `ADVSW` / `NEXTST`
  gate metadata plus the RAM-test start/failure/no-error visible setup, RAM2
  pattern fill/verify pass with page-boundary operator-poll metadata,
  pass-boundary loop dispatch, and CMOS RAM-test write/verify loop plus visible
  outcomes and the CROM0 color-RAM diagnostic heading/bars/palette loop plus
  audio-test sound-pulse/skip-table behavior,
  switch-test display-table/PIA-scan behavior, monitor-test
  crosshatch/RGB-field/color-bar pattern behavior, monitor-to-`AUDITG` entry
  transfer, post-`PWRUP` `AuditGate` entry transfer, `AUDITG` outer frame-step
  dispatch, packed high-score table comparison/insertion, and optional
  file-backed CMOS persistence. Deterministic initials-entry submission follows
  the `amode1.src` player-one/player-two order, uses the source
  today's-greatest qualification gate, writes submitted entries into `THSTAB`,
  inserts into all-time CMOS when that table also qualifies, and translated
  player-death game-over sleeps can enter live `GameOver` for that path. Exact
  `HALLOF` now runs source `GNCIDE`, `STINIT`, credit mirroring, and entry-flag
  clearing before `HALL1` qualification or the power-on `AMODES` branch.
  `HALLOF` / `HOFIN` initials-entry screen rendering now writes source player
  labels, hall-of-fame instructions, initials, underlines, and palette state
  into red-label video RAM, and gameplay `PLE2` now writes source `GO` text at
  `0x3E80`. Submitted high-score sessions now render the source-backed
  `HALDIS` hall-of-fame table display. The source `PLE2`/`PLE3` 40-tick
  game-over sleep gates `HALLOF`, and non-qualifying sessions now wait through
  the source `HALL13` no-entry delay before `HALDIS`.

## Player

- The visible `PLSTR5` player-start RAM bytes are source-owned. Live player
  movement now runs the translated `PLAYER` IRQ slice, and live presentation
  runs translated `PRDISP` for the current player display band so reverse
  direction changes reach rendered/facing state.
- The `REV` switch process and debounce tail are translated, including
  `REVFLG`, `NPLAD`, `PIA21`, and process free-list side effects. The `SBOMB`
  switch process now starts from asset-backed `SWTAB`/`SSCAN`/`SWPROC`/`SWP`
  instead of scaffold inventory mutation. The visible `HYP02`/`HYP2`
  hyperspace rematerialization tail is translated up to the `PLEND` death
  branch, `PLEND`/`PDEATH` is translated through the glow-loop sleep into
  `PDTH5`, the `PXVCT`/`PX1A` explosion loop, and non-wave-end
  respawn/game-over branch decisions, and `KILOFF` object unlink/footprint
  erase is translated. Exact `PLAYER` motion now covers horizontal damping,
  thrust, X/scroll correction, absolute-X, and altitude velocity, and
  `PRDISP` / `POUT` now covers the 8x6 player body and `THOUT` / `THOFF`
  thrust-flame write/erase paths. `OPROC` now covers active-object descriptor
  erase/redraw inside caller scanline bands, and `VELO` now covers
  active-object velocity addition and Y wrap. The normal/inverted IRQ
  object-band tails now cover the ROM ordering around `PRDISP`, `OPROC`,
  `SHELL`, and `VELO` for `XXX1`/`XXX2` scanline pairs. The `GEXEC` / `GEX0`
  slice now initializes `PD`, `UFOTMR`, and `WAVTMR`, runs the source `WVCHK`
  gate, accelerates/decrements `UFOTMR`, dispatches `UFOST` and increments
  `UFOCNT` on baiter-timer expiry, launches `LANDST` squads from
  `WAVTMR` / `LNDRES` / `WAVSIZ`, restores `STRCNT`, advances `GTIME`,
  decrements `PD`, applies source `WDELT` intra-wall deltas to `ELIST`, and
  sleeps back to `GEX0`; the wave-clear path now runs `GNCIDE` / `PLSAV`,
  enters the shared `BONUS` body with the assembled `GEXBON` return site, and
  resumes through the translated `PLSTRT` / `PLSTR3` handoff after `BC3`.
  `GETWV` now
  increments `PWAV`, refreshes
  `PENEMY` from
  source-order `WVTAB`, applies CMOS difficulty/ceiling inter-wall `WDELT`
  updates, and restores `PTARG` on the `GA1+6` restore-wave cadence. DC-10
  source-backed fixtures now cover launch timing, reserve allocation,
  terrain/scanner mutation, `TERBLO`, `GETWV` / `WDELT`, survivor bonus, and
  wave-to-wave RAM/list mutations; whole-session MAME trace proof remains open.
  `BGI`
  now selects bank/map 7 and runs the translated `BGINIT` terrain-table
  generator. DC-08 refactor fixtures now lock the translated `BONUS` / `SCLR1`
  wave-clear death tail, respawn/game-over branches, human rescue, hyperspace,
  laser, smart-bomb, and reverse-player paths; remaining player-path risk is
  full CPU-cycle ownership and non-gameplay presentation timing.

## World And Enemies

- Terrain `TDATA`, `ALINIT` altitude-table generation, `BGINIT` terrain-table
  generation, RAM-level `BGOUT` terrain output, and `BGERAS` screen-table erase
  are translated. Live playing frames now schedule translated `BGOUT` with the
  source-derived IRQ stack pointer. Scanner mini-terrain `MTERR` and
  destroyed-planet `TERBLO` / `TBL3` / `TBL4` behavior are translated.
- Wave launch uses source-order `WVTAB` / `WDELT` data and translated `GEXEC` /
  `GEX0` pacing to launch `UFOST` baiters and `LANDST` squads from red-label
  reserve counters. Current source-backed fixtures cover the translated
  wave/world RAM, process-list, object-list, and terrain/scanner mutations, but
  not full-session MAME trace equivalence.
- Schizoid/mutant, UFO/baiter, lander, tie-fighter/bomber, probe/pod,
  mini-swarmer, astronaut, falling-astronaut, score-popup, and terrain-blow
  process slices are translated. End-to-end interaction proof against local
  MAME traces remains open until full-frame IRQ scheduling is integrated.
- Collision detection is translated for the currently extracted complete and
  short-form picture descriptors/images: landers, humanoids, tie fighters,
  probes, swarmers, UFOs, player pictures, laser, mini-player, smart bomb, bomb
  shells, score popups, and explosions. Remaining trace proof for complete-game
  collision behavior is tracked by the final fidelity phase.

## Video

- The MAME Williams screen-memory byte layout, native Defender visible-area
  pixel-nibble and palette-index extraction, RGB palette resistor conversion,
  and native RGBA cabinet-frame scaler are implemented. The translated
  `PRDISP` / `POUT`
  player-picture slice can write and erase the 8x6 ship body and
  thrust-flame bytes in video RAM. `OPROC` can erase and redraw active-object
  descriptor pictures in the caller's scanline band. `STOUT` / `SBLNK` can
  write, clear, blink, and hyperspace-shift the source star-map bytes in video
  RAM. Clean player-switch prompt sprites now mirror the source `PDTH5R`
  `PLYR1`/`PLYR2` plus `GO` messages during `PLE02` switch sleep. The `BONUS`
  `MESS` / `WNBV` calls can write the wave-complete text and numbers used by
  the survivor bonus screen. `HALLOF` / `HOFIN` now writes the initials-entry
  player label, hall-of-fame instructions, initials block, active underline
  words, and pseudo-color copy into video RAM, gameplay `PLE2` writes the
  source `GO` message at `0x3E80`, two-player `PDTH5R` writes the current
  player label at `0x3C78` plus `GO` at `0x3E88`, `PLSTR5` writes the
  two-player current-player start prompt at `0x3C80`, and submitted high-score
  sessions render the source-backed `HALDIS` hall-of-fame table. Source
  `SCINIT` now resets object lists, clears video RAM, zeroes `BGL`/`BGLX`,
  runs `BGI`, reloads `CRTAB`, and sets `STATUS=$DB` plus `PLAXC=$1030`;
  `PLE3` applies `ATTR`'s map-1 select, and `HALDIS` runs source `GNCIDE`
  before drawing; source `CREDS` writes the attract credits label and BCD
  number while maintaining `OCRED` / `ICREDF`; source `AMODES` prepares the
  Williams-page state, and source `LOGO` expands `DEFENDER`, walks `LGOTAB`,
  draws Williams-logo pixels, switches to the fast pass, and creates `PRES`;
  `PRES` / `PRES1` writes the source `ELECV` `ELECTRONICS INC.` / `PRESENTS`
  text and starts `DEFEND`. `DEFEND` / `DEFENS` drives the source 4x12
  wordmark appearances, `DEF33` / `DEF50` refreshes the whole wordmark until
  appearance RAM clears, `DEF44` starts `CBOMB` before the source `COPYRT`
  copyright strip and wait gates, direct `COPYRT` redraws without starting
  `CBOMB`, and `WILLIR` / `WILR1` performs the fast Williams-logo restore.
  `DEF51`, `CPR56`, `HALL12`, and `HALD4` now dispatch through their source
  `DEF50`, `HALDIS`, `HALL1` / `HALL13`, and `LEDRET` branch targets. `LEDRET`
  now drives the source instruction-page setup, rescue, enemy-table loop,
  laser process, direct `LASR` / `LASL` setup labels, and `TEXTP` / `TEXTP2`
  text cadence from `TEXTAB` / `TENT`; `HOFST`, `HOFBL`, `HOFUD` / `HOFUD1`,
  `ATTR` / `HALLOF`,
  `HALL1` / `HALL3A` / `HALL4` / `HALL5` / `HALL6`, and `HALD3` now tick the
  source initials-entry qualification, screen setup, stall, blink, up/down,
  fire-switch debounce/advance, initials submission, and hall-of-fame wait
  state. `SCNRV` now writes
  scanner terrain, object, and
  player blips through the source `SETAB` / `STETAB` erase tables. The live
  renderer now copies source `PCRAM` into palette RAM and presents translated
  video RAM through the native cabinet-frame scaler without a synthetic
  scaffold fallback, with native visible pixel-nibble checksum fixtures for the
  first `AMODES` / `LOGO` attract slice, source `SCORE` score/replay HUD
  refresh paths, and a live IRQ scanline frame. Source-native video fixture
  coverage now also locks exact pixel checksums and coarse visible-shape
  signatures for boot, attract, start, gameplay, death, high-score, and
  operator/AUDITG frames. `DC-20.1` added MAME-derived visible pixel-nibble CRC
  capture to the local reference exporter, using the same `292x240` Defender
  visible window and high/low nibble order as Rust. `DC-25` fixed the early
  boot/title/initial-attract pixel drift caught by the regenerated
  `attract_boot` smoke fixture; the exact 900-frame local `attract_boot`
  reference test now passes unignored with populated `video_crc32`. A
  `2026-05-05` live title-screen capture showed the `DEFENDER` wordmark/title
  graphic corrupted into large red/purple blocky bands, and the same
  live-review pass reported that the app did not advance beyond the initial
  Williams/`DEFENDER` screen. `DC-16.5` adds core-level coverage proving idle
  live attract reaches later attract processes, accepts credit, and starts
  play; full terminal screenshot proof remains with `DC-30`. Live playing
  frames now run upright `IRQ` and
  `IRQHK`-selected flipped `IRQB` video passes through the source `VERTCT` /
  `IFLG` scheduler, including map writes, timer/watchdog side effects, palette
  copy, translated `PLAYER`, `STOUT`, upper/lower `OPROC` / `PRDISP` bands,
  `SHELL`, IRQ `BGOUT`, and `VELO`. Initialized live machines now apply source
  `P1SW`, and cocktail player-start screen switching now runs translated
  `P1SW` / `P2SW` setup so the live IRQ pass is selected from source-written
  `IRQHK`. Idle live attract creates source `ATTR` and follows the immediate
  source power-on `AMODES` / `LOGO` handoff before sleeping to `LOGO0`.
  Remaining untranslated screens stay black. Full
  remaining live HUD/video presentation, general text, remaining boot/game-over
  presentation, non-IRQ CPU-cycle ownership, and sound-IRQ ownership are not
  translated. Live pacing is derived from
  the core red-label
  `FRAME_RATE_MILLIHZ` constant, and the live loop advances any due
  core frames before each terminal draw so presentation cadence no longer
  decides core frame count.

## Audio

- The source `SNDLD` table loads and `SNDSEQ` main-board sound-table sequencer
  are translated over `SNDX`, `SNDPRI`, `SNDTMR`, `SNDREP`, and `THFLG`.
  The `SNDLD` path preserves the source pre-priority `THFLG` clear and
  equal-priority interrupt rule. All embedded source sound tables can now expand
  into exact repeated command plans, the embedded `sound-table-timelines.tsv`
  fixture with table labels, addresses, `SNDSEQ` tick anchors, terminator
  pointers, and sequence-end ticks, and the source-derived
  `sound-table-command-sequences.tsv` fixture that expands each table command
  through `SNDOUT` into idle and complemented command writes. The source thrust
  gate has a matching `sound-thrust-command-sequences.tsv` fixture for the
  `SNDS01` start and `SNDS00` stop branches, and the direct `PDTH5`, `PLE2`,
  and `LNDFX0` `SNDOUT` callsites are covered by
  `sound-direct-command-sequences.tsv`. Fixture validators compare the embedded
  rows against the generated data and count timeline command and sequence-end
  rows plus command-sequence idle/command writes.
  `SNDSEQ` can model the source `SNDOUT` idle write followed by the
  complemented sound-number command for table playback and the thrust
  start/stop gate, and full frame output/fidelity traces carry the asserted raw
  command bytes. `DC-21.2` regenerated a local MAME `start_game` trace under
  `/tmp/defender-dc21-reference`; it records frame 731 `0xC0`, frame 912
  `0xE6`/`credit_added`, and frame 1027 `0xF5`/`game_started`, and current Rust
  matches those `sound_commands`/`events` rows exactly. Broader local MAME
  command-sequence fixtures and external waveform fixtures are still missing.
- `DC-51` accepted `FrameOutput::sound_commands()` as the live-audio command
  timing surface, with `FrameOutput::sound_board` retained as a diagnostic
  latch/IRQ surface and deterministic DAC byte signatures retained as the
  content guard for translated sound-ROM routines. The implementation contract
  is documented in `docs/fidelity/live-audio.md`, and
  `assets/red-label/live-audio-acceptance.tsv` ties coin, start, thrust, smart
  bomb, hyperspace, player explosion, terrain blow, and high-score paths to
  existing tests or generated sound fixtures. The remaining cycle-accurate
  sound CPU scheduling, external waveform fixtures, and broader MAME command
  fixtures are later fidelity work, not blockers for an event-fed live audio
  runtime prototype.
- `DC-52` added the first live audio runtime path, and `DC-59` moved the active
  `src/audio.rs` surface to gameplay-facing `SoundEvent` batches. Live core
  stepping still uses accepted `FrameOutput::sound_commands()` as the timing
  oracle, then maps those bytes to semantic events for the bounded
  non-blocking channel owned by `LiveAudioRuntime`; `LiveAudioBackend` keeps
  backend behavior swappable, `NullLiveAudioBackend` opens no device, worker
  failures are reported through structured shutdown diagnostics, and `--mute`
  disables the path. `--live-smoke` uses a disabled no-device audio path, so
  smoke evidence stays deterministic.
- Sound-board RAM/PIA/ROM address classification exists, and the MAME-documented
  main-board command latch byte/CB1 handoff is modeled from the PIA1 port-B
  output callback boundary. `ArcadeMachine::step` now records every emitted
  frame sound command through the same latch/CB1 boundary before trace output,
  while preserving the existing command sequence as the observable frame
  result. Sound CPU PIA IC4 register reads can consume the latched command
  byte, and PIA port-A writes are captured at the DAC boundary. Command CB1
  drives the sound PIA IRQ state. The `VSNDRM1.SRC` `SETUP` plus
  IRQ command decoder now classify raw command bytes into GWAVE, jump-table
  special, and VARI routine targets while applying the source-visible
  background/spinner/bonus flag gates. Normal `IRQ1` GWAVE/VARI commands can
  now run their translated loader and first waveform window from that dispatch
  result, `SP1` can run its `CABSHK` `VARILD` setup and first translated
  `VARI` sweep, `BON2` can run its first `BONV` `GWAVE` window or `GEND50`
  continuation, `LITE` / `APPEAR` special commands can run the shared
  translated `LITEN` stream, `BG2INC` can run its source flag update and first
  `BG2` setup, `BGEND` can report its source flag-only result, `BG1` /
  `THRUST` special commands can run their first translated `FNOISE` windows,
  `TURBO` / `CANNON` / `RADIO` / `HYPER` / `SCREAM` special commands can run
  their translated finite DAC streams, `ORGANT` / `ORGANN` can report their
  source organ-flag setup states, and the source `IRQ3` background handoff can
  kill `B2FLG` and run the first translated `BG1` / `BG2` continuation step
  when background flags are active. The `GWLD` loader now reads `SVTAB` /
  `GWVTAB` / `GFRTAB`
  from the loaded sound ROM and populates direct-page GWAVE
  state through `WVDECA` predecay, `VARILD` reads `VVECT` into the VARI
  direct-page parameters, and the `SP1`, `BON2`, and `BG2` pre-loop setup
  paths, the `GWAVE` / `GPLAY` per-period DAC byte stream, the `VARI` /
  `VSWEEP` per-sweep DAC byte stream, the `LITE` / `APPEAR` shared `LITEN`
  random-complement byte stream, the `TURBO` / `NOISE` noise-decay byte stream,
  the `HYPER` phase-edge DAC byte stream, the `BG1` / `THRUST` first-window
  `FNOISE` byte streams, the `CANNON` / `FNOISE` filtered-noise decay byte
  stream, the `RADIO` / `RADSND` timer-table byte stream, the `SCREAM`
  echo-cascade byte stream, the `ORGANN` / `ORGNN1` first-duration `ORGAN`
  note byte stream, the `ORGNT1` / `ORGTAB` organ tune byte streams, the
  source `IRQ` PIA command-read/CB1-clear prelude, the source `IRQ3`
  background handoff, command return/readiness classification, source-shaped
  `IRQ1` command-to-`IRQ3` background step, the top-level source IRQ organ
  gate, the source IRQ organ-continuation gate, the source IRQ prelude-to-flow
  cycle, plus the shared `GEND` / `GEND40` / `GEND50` / `GEND60` / `GEND61`
  echo and frequency-window updates and the source NMI diagnostic
  checksum-to-VARI branch are translated. The timed IRQ window reports DAC
  bytes in source order with monotonic DAC-write ticks, but those ticks are not
  6808 CPU cycles. Cycle-accurate DAC scheduling, independent CPU IRQ
  scheduling, broader live MAME command-sequence fixtures, and the remaining
  waveform routines are not translated.
- No external waveform fixtures exist yet; only deterministic in-repo DAC
  buffer signatures cover representative translated routines.
- End-to-end MAME golden traces for two-player sessions, high-score entry,
  operator/service screens, and cabinet input profiles do not exist yet. The
  runtime has source-native mutation fixtures for live two-player session flow,
  high-score display/submission, operator diagnostics/audits/reset paths, coin
  audits, credit backup, and disabled/enabled `xyzzy` separation, but those
  paths remain explicit pre-refactor gaps until locally generated MAME traces
  can be checked and promoted.

## Compatibility

- `xyzzy` exists as an overlay. Current auto-fire/smart-bomb overlay behavior
  has paired disabled/enabled tests, and each future hook needs the same paired
  proof that disabled-`xyzzy` arcade behavior remains unchanged.
- Planetoid controls exist as an input profile and must remain outside the
  arcade core. Cabinet action projection to red-label IN0/IN1/IN2 bytes exists;
  Planetoid and test keyboard choices remain compatibility inputs layered over
  those cabinet actions.
