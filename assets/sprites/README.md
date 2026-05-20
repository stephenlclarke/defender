# Sprite Assets

This directory holds checked-in sprite and sprite-sheet image artifacts.

The current PNG set was moved from the archived prototype `assets/arcade/`
directory so runtime sprite inputs have a single home. Most files remain
prototype references until a cycle explicitly reclassifies them for clean
runtime use with documented provenance.

`DC-156` temporarily reclassifies `ship1.png`, `lander1.png`, `humanoid1.png`,
`player-shot.png`, and `font-sheet.png` as clean-runtime R2 atlas inputs. That
reclassification is recorded in `docs/fidelity/gaps.md` and exists only to
retire solid placeholder atlas regions while later cycles replace these inputs
with source-cited red-label, ROM/MAME-derived, or generated assets with stronger
provenance.

`DC-164` maps the matching red-label picture labels for that reclassified
runtime subset into clean sprite evidence: `PLAPIC`/`PLBPIC`, `LNDP1`-`LNDP3`,
`ASTP1`-`ASTP4`, and `LASP1`. Step 12 also maps the existing enemy-family
prototype PNGs where source picture labels already identify the family:
`SCZP1` to `mutant1.png`, `UFOP1`-`UFOP3` to `baiter1.png`,
`TIEP1`-`TIEP4` to `bomber1.png`, `PRBP1` to `pod1.png`, and `SWPIC1` to
`swarmer1.png`. Step 13 maps the bounded display/reward subset:
`BMBP1`/`BMBP2` to `bomb1.png`, `BXPIC` to `podexpl.png`, `SWXP1` to
`swarmexpl.png`, `C25P1` to `score250_1.png`, `C5P1` to `score500_1.png`,
`PLAMIN` to `littleship.png`, and `SBPIC` to `smartbomb.png`. `ASXP1`,
`NULOB`, and `TEREX` do not use PNG files here; Step 14 maps them from
`assets/red-label/object-images.tsv` bytes. Lifecycle behavior and final render
presentation remain unmapped until a later cycle reclassifies them with
source-backed evidence.

Do not add duplicate sprite files under `assets/arcade/`, `docs/`, or source
directories. New non-legacy sound artifacts belong under `assets/sounds/`;
pre-existing legacy `.wav` cue files remain under `assets/arcade/`.
