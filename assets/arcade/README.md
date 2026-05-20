# Arcade Prototype Notes

This directory holds legacy rule notes and pre-existing prototype `.wav` cue
assets from the archived prototype.

Prototype PNG sprite and sprite-sheet assets now live under `assets/sprites/`.
Files there remain archived prototype references unless a cycle explicitly
reclassifies them for clean runtime use with documented provenance.

Sprite files that are reused for transitional runtime work should come from this
`assets/sprites/` directory when an existing PNG fits the need. Do not add
duplicate sprite files elsewhere. Captured, generated, or prototype sound
artifacts created after the rewrite asset split belong under `assets/sounds/`;
pre-existing legacy `.wav` cue files remain here.

`DC-156` temporarily reclassifies `ship1.png`, `lander1.png`, `humanoid1.png`,
`player-shot.png`, and `font-sheet.png` as clean-runtime R2 atlas inputs. That
reclassification is recorded in `docs/fidelity/gaps.md` and exists only to
retire solid placeholder atlas regions while later cycles replace these inputs
with source-cited red-label, ROM/MAME-derived, or generated assets with stronger
provenance.

`DC-164` maps the matching red-label object-picture labels for that
reclassified runtime subset into clean sprite evidence and now extends the
bridge to the existing enemy-family prototype sprites in `assets/sprites/`:
`mutant1.png`, `baiter1.png`, `bomber1.png`, `pod1.png`, and `swarmer1.png`.
The bridge now also covers bounded bomb, explosion, score-popup, life-stock,
and smart-bomb-stock labels using existing `assets/sprites/` files. `ASXP1`,
`NULOB`, and `TEREX` are backed directly from `assets/red-label/object-images.tsv`
bytes rather than prototype PNGs. Lifecycle behavior and final presentation
work remain unmapped until explicitly reclassified.

Future runtime visuals should come from source-cited red-label data under
`assets/red-label/`, red-label ROM/MAME-derived fixtures, or generated assets
with documented provenance. Do not add new clean-slate runtime dependencies on
other legacy PNGs without first reclassifying the asset in the fidelity docs.
