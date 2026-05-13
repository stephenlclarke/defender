# Fidelity Workflow

This directory tracks the exact-red-label workflow for the Rust rewrite.

Rules:

- Arcade-core behavior must come from red-label source, red-label ROM behavior,
  or MAME-observed behavior.
- Every translated routine must cite its source routine, table, or trace.
- Unknown behavior is recorded in `gaps.md`; do not guess.
- Golden traces must not include copyrighted ROM payloads.
- Verification tooling may depend on local ROMs or MAME, but the deployed game
  binary must remain self-contained.
- Characterization tests follow `characterization-tests.md` so later refactors
  preserve source-visible mutations, not only returned values.
- The future-refactor freeze contract is recorded in `refactor-freeze.md`;
  keep its validation suite, API contract, module boundaries, and
  byte-compatible surface list current. Do not start the large refactor until
  final ROM-complete playable acceptance is complete.
- Golden-trace fixes follow the `characterization-tests.md` golden-fix workflow:
  exact TSV comparison first, then narrow source-visible mutation tests for the
  responsible routine.

Current trace format:

- `assets/red-label/trace-schema.tsv` defines the first TSV frame trace schema,
  and `src/fidelity.rs` embeds it.
- The schema records input bits, frame number, player/session state, RNG bytes,
  optional raw object-table, process-table, super-process-table,
  SPTR-linked shell-object-list, and native video-frame CRC-32 values, raw
  sound-command writes, and machine events.
- The current Rust trace fills the object-table, process-table,
  super-process-table, SPTR-head, and native visible-video CRC-32 columns from
  the table-backed red-label runtime memory and the accepted MAME-observed
  long-tail sample bridge. The video CRC is computed over the native visible
  pixel nibbles decoded from red-label video RAM. Trace generation starts from
  cold-boot object and process RAM, while the normal live/test constructor
  remains pre-initialized for translated routine unit tests. The Phase 10
  local reference gate now exact-compares all 12 Phase 1 scenarios when local
  reference fixtures are present.
- The local MAME reference exporter now computes `video_crc32` with the same
  visible pixel-nibble convention: Williams two-pixels-per-byte video RAM,
  Defender visible origin `(12, 7)`, visible size `292x240`, and CRC-32 over
  one decoded nibble per visible pixel. The reference fixture checker now
  rejects stale required cells such as `video_crc32=-`; regenerate local
  reference traces before treating them as pixel evidence.
- `defender --fidelity-trace 300` emits deterministic Rust trace rows for local
  fixture work. It is not a substitute for MAME/source-derived golden traces.
- `defender --fidelity-trace-inputs 'coin,start_one;fire,thrust;none'` emits
  deterministic Rust trace rows while applying one semicolon-separated cabinet
  action set per frame. Use `none` for an idle frame.
- `defender --fidelity-trace-inputs-file /path/to/inputs.txt` reads the same
  semicolon-separated cabinet input script from a local file.
- `defender --fidelity-check-trace /path/to/inputs.txt /path/to/expected.tsv`
  generates the Rust trace for a file-backed input script and compares it
  exactly with a local expected TSV fixture.
- `defender --fidelity-check-trace-dir docs/fidelity/fixtures/local/rust-current`
  checks paired `*.inputs.txt` / `*.expected.tsv` fixtures for exact
  Rust-current comparisons. A missing directory is skipped so the target
  remains usable while only MAME reference fixtures exist.
- `defender --fidelity-list-scenarios` lists the checked-in Phase 1 reference
  trace scenarios from `assets/red-label/trace-scenarios.tsv`.
- `defender --fidelity-write-scenario-inputs docs/fidelity/fixtures/local/reference`
  writes expanded `*.inputs.txt` files for every Phase 1 scenario.
- `defender --fidelity-check-reference-trace-dir docs/fidelity/fixtures/local/reference`
  validates that every required Phase 1 scenario has a matching input script
  and expected trace using the checked-in schema header plus
  `assets/red-label/trace-requirements.tsv` evidence markers.
- Tests named `local_reference_*_matches_red_label` in `src/app.rs` encode the
  Phase 1 local MAME/reference exact-match gate. They now run normally for all
  12 scenarios when the local reference fixture directory is present. Running
  `cargo test --all-targets -- --ignored` should report zero ignored tests.
- `defender --verify-roms /path/to/roms` validates local red-label ROM files and
  verifies that they map into the embedded MAME red-label ROM regions.
- Future subsystem traces should extend this format only when a red-label
  routine needs more observable state.

Initial local workflow:

1. Build or provide the red-label ROM set locally under `assets/roms/` or set
   `DEFENDER_ROM_DIR`.
2. Install MAME or set `DEFENDER_MAME` to the local MAME executable.
3. Run `make trace-script-test` to verify the local Lua trace exporter before
   asking MAME to produce reference traces.
4. Run `make reference-inputs` to write the Phase 1 scenario input scripts.
5. Run `make reference-traces` to generate MAME/source traces into the ignored
   local reference fixture directory.
6. Run `make reference-fixtures-check` to verify that all required Phase 1
   fixtures are present and use `assets/red-label/trace-schema.tsv`.
7. Run `make trace-fixtures` to compare current Rust traces against local
   exact-match fixtures under `docs/fidelity/fixtures/local/rust-current` when
   that comparison is appropriate for the subsystem.
8. Translate one routine/table at a time into `src/`.
9. Add a unit test for the translated routine and a fidelity test against the
   trace fixture.
10. Keep `make fidelity` green before moving to the next routine.

Local MAME reference fixtures belong under
`docs/fidelity/fixtures/local/reference/` using paired `<scenario>.inputs.txt`
and `<scenario>.expected.tsv` files. Local exact-match Rust fixtures belong
under `docs/fidelity/fixtures/local/rust-current/`. Run `make trace-fixtures`
to check every local Rust-current pair.

Optional local configuration is documented in `docs/fidelity/local.env.example`.
Local reference fixture run records are documented in
`docs/fidelity/local-reference-runs.md`. Rust-vs-reference comparison findings
are recorded in `docs/fidelity/golden-comparison-results.md`. The checked-in
MAME runner is `tools/generate_reference_traces.py`; it invokes
`tools/mame_defender_trace.lua` with MAME's Lua `-autoboot_script` mechanism.
The Lua exporter records per-frame main-board sound commands by write-tapping
the MAME main CPU program space at PIA1 port B/control, suppressing the idle
byte and emitting the asserted raw command bytes into the existing trace
schema. It also decodes the MAME-visible Defender pixel nibbles from main RAM
and emits their CRC-32 into `video_crc32`, so trace fixtures can carry
MAME-derived pixel evidence without storing screenshots or ROM payloads. The
exporter also maps the MAME-observed credited-start sound commands `0xE6` and
`0xF5` to `credit_added` and `game_started` trace events so reference fixtures
prove the source `CSCAN` / `SSCAN` / `ST1` path. The Phase 10 gate exact-matches
those command/event rows across all playable local reference scenarios. External
waveform files are intentionally absent from git; representative generated DAC
buffers are covered by deterministic source-visible signature tests instead of
hardware-cycle waveform reconstruction.
The live audio acceptance contract and clean event-delivery runtime are
documented in `live-audio.md`, with the checked-in path matrix embedded from
`assets/red-label/live-audio-acceptance.tsv`.
Each generated scenario uses an isolated, freshly cleared MAME state directory
under `docs/fidelity/fixtures/local/reference/mame-state/<scenario>/` so
NVRAM/config state cannot leak between local reference traces. The generator
seeds the MAME `defender/nvram` file from `assets/red-label/cmos-defaults.tsv`
by default, matching `romc8.src` `DEFALT` as two MAME 4-bit CMOS cells per
byte. Pass `--blank-nvram` only when intentionally investigating the invalid
CMOS cold-boot/operator path. The generator defaults `SDL_VIDEODRIVER` and
`SDL_AUDIODRIVER` to `dummy` when those variables are not already set, so MAME
can run the `-video none` / `-sound none` trace job from headless shells while
still allowing local overrides.

Set `DEFENDER_TRACE_DEBUG=/path/to/debug.tsv` when invoking
`tools/mame_defender_trace.lua` directly to write a local diagnostic stream
with the main CPU PC, MAME-read IN0/IN1/IN2 bytes, current bank selection,
per-frame bank-select writes, selected red-label RAM bytes, and
object/process/super-process/shell CRCs. Debug traces are investigation aids
only and are not part of the
checked-in fixture schema.

## Red-Label ROM Build Notes

The reference source at <https://github.com/mwenge/defender> documents the
assembler flow for the red-label ROM files. Use a separate local checkout and
do not copy generated ROM payloads into this repository except under ignored
local paths such as `assets/roms/`.

Summary of the upstream flow:

```sh
git clone https://github.com/mwenge/defender /tmp/defender-redlabel-src
cd /tmp/defender-redlabel-src
git submodule init
git submodule update
cd asm6809
./autogen.sh
./configure
make
cd ../vasm-mirror
make CPU=6800 SYNTAX=oldstyle
cd ..
make redlabel
```

The upstream `make redlabel` target writes the red-label ROM files into its
`redlabel/` directory. Point `DEFENDER_ROM_DIR` at that directory, or copy the
files into this repo's ignored `assets/roms/` directory for local verification.
On macOS, the upstream assembler may need Homebrew `bison` ahead of
`/usr/bin/bison`, and the upstream checksum target expects GNU `md5sum -c`
from Homebrew `coreutils`.
