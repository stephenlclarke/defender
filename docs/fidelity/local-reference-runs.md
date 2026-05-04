# Local Reference Runs

This file records local MAME reference fixture runs. The generated fixtures are
machine-local verification artifacts and must not be checked in.

## 2026-05-04 Phase 1 Run

Plan step: `DC-03.5`.

Reference traces were generated on `2026-05-04 17:43:34 BST` and revalidated on
`2026-05-04 17:49:46 BST`.

Environment:

- MAME command: `mame`
- Resolved MAME path: `/opt/homebrew/bin/mame`
- MAME version: `0.287 (unknown)`
- ROM root: `assets/roms`
- Red-label machine ROM directory: `assets/roms/defender`
- Reference fixture directory:
  `docs/fidelity/fixtures/local/reference`

The local red-label ROM directory contained:

```text
decoder.2
decoder.3
defend.1
defend.10
defend.11
defend.12
defend.2
defend.3
defend.4
defend.6
defend.7
defend.8
defend.9
video_sound_rom_1.ic12
```

Commands run:

```sh
make reference-inputs
make reference-traces
make reference-fixtures-check
```

Results:

- `make reference-inputs` wrote 12 Phase 1 `*.inputs.txt` scripts from
  `assets/red-label/trace-scenarios.tsv`.
- `make reference-traces` processed all 12 scenarios through the local MAME
  runner and wrote ignored `*.expected.tsv` fixtures. MAME printed repeated
  `-video none` / `-seconds_to_run` warnings during the run, but the generator
  completed successfully.
- `make reference-fixtures-check` reported 12 complete Phase 1 fixtures and
  22,308 frames under `docs/fidelity/fixtures/local/reference`.

Additional validation:

- The generated input-script names matched the checked-in scenario manifest.
- Each generated input script expanded to the manifest-declared frame count.
- The checker rejected a bad expected-trace header.
- The checker rejected a missing `death.expected.tsv` scenario fixture.
- The checker rejected a truncated `start_game.expected.tsv` fixture with
  1,227 frames instead of the required 1,228.
- The existing 12 scenarios were reviewed against the next golden-equivalence
  work; no extra scenario was required before `DC-04`.

ROM and fixture policy:

- Keep ROM payloads out of git. Use ignored `assets/roms/` paths or set
  `DEFENDER_ROM_DIR` to an external red-label ROM build.
- Keep MAME-generated reference traces, generated input scripts, and per-run
  MAME state out of git under `docs/fidelity/fixtures/local/`.
- Commit only schema, scenario manifests, source metadata, fixture tooling, and
  documentation needed to reproduce local runs.
