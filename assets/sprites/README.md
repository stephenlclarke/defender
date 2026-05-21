# Sprite Assets

This directory holds checked-in sprite and sprite-sheet image artifacts.

The current PNG set was moved from the archived prototype `assets/arcade/`
directory so runtime sprite inputs have a single home. Most files remain
prototype references until a cycle explicitly reclassifies them for clean
runtime use with documented provenance.

R9-E3.5 moved the clean runtime sprite atlas off the transitional gameplay PNG
inputs for player ship, player projectile, enemy, bomb, explosion, reward,
stock, and smart-bomb sprites. Those runtime regions are now decoded from
`assets/red-label/object-images.tsv` with source palette overrides. The PNGs in
this directory remain historical/prototype references unless a future cycle
explicitly reclassifies an individual file.

`DC-156` temporarily reclassified `ship1.png`, `lander1.png`,
`humanoid1.png`, `player-shot.png`, and `font-sheet.png` as clean-runtime R2
atlas inputs to retire solid placeholders. After R9-E3.11, the font sheet only
remains an active runtime PNG input for legacy score/status helper regions; the
terrain helper now decodes source `TDATA` / `BGOUT` terrain words directly.
`ASXP1`, `NULOB`, `TEREX`, terrain words, and the R9-E3.5 runtime sprite set
are decoded from red-label source data.

Do not add duplicate sprite files under `assets/arcade/`, `docs/`, or source
directories. New non-legacy sound artifacts belong under `assets/sounds/`;
pre-existing legacy `.wav` cue files remain under `assets/arcade/`.
