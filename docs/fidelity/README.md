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

Current trace format:

- `assets/red-label/trace-schema.tsv` defines the first TSV frame trace schema,
  and `src/fidelity.rs` embeds it.
- The schema records input bits, frame number, player/session state, RNG bytes,
  optional raw object-table, SPTR-linked shell-object-list, and native
  video-frame CRC-32 values, raw sound-command writes, and machine events.
- The current Rust trace fills the object-table and SPTR-head CRC-32 columns
  from the table-backed red-label runtime memory. Trace generation starts from
  cold-boot object RAM, while the normal live/test constructor remains
  pre-initialized for translated routine unit tests. Golden equivalence for
  those columns still depends on local MAME/source expected traces.
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
- `defender --fidelity-check-trace-dir docs/fidelity/fixtures/local` checks all
  paired `*.inputs.txt` / `*.expected.tsv` fixtures in a local directory. A
  missing directory is skipped so the target remains usable before local
  fixtures have been generated.
- `defender --fidelity-list-scenarios` lists the checked-in Phase 1 reference
  trace scenarios from `assets/red-label/trace-scenarios.tsv`.
- `defender --fidelity-write-scenario-inputs docs/fidelity/fixtures/local`
  writes expanded `*.inputs.txt` files for every Phase 1 scenario.
- `defender --fidelity-check-reference-trace-dir docs/fidelity/fixtures/local`
  validates that every required Phase 1 scenario has a matching input script
  and expected trace using the checked-in schema header.
- `defender --verify-roms /path/to/roms` validates local red-label ROM files and
  verifies that they map into the embedded MAME red-label ROM regions.
- Future subsystem traces should extend this format only when a red-label
  routine needs more observable state.

Initial local workflow:

1. Build or provide the red-label ROM set locally under `assets/roms/` or set
   `DEFENDER_ROM_DIR`.
2. Install MAME or set `DEFENDER_MAME` to the local MAME executable.
3. Run `make reference-inputs` to write the Phase 1 scenario input scripts.
4. Run `make reference-traces` to generate MAME/source traces into the ignored
   local fixture directory.
5. Run `make reference-fixtures-check` to verify that all required Phase 1
   fixtures are present and use `assets/red-label/trace-schema.tsv`.
6. Run `make trace-fixtures` to compare current Rust traces against local
   expected fixtures when that comparison is appropriate for the subsystem.
7. Translate one routine/table at a time into `src/`.
8. Add a unit test for the translated routine and a fidelity test against the
   trace fixture.
9. Keep `make fidelity` green before moving to the next routine.

Local trace fixtures belong under `docs/fidelity/fixtures/local/` using paired
`<scenario>.inputs.txt` and `<scenario>.expected.tsv` files. Run
`make trace-fixtures` to check every local pair.

Optional local configuration is documented in `docs/fidelity/local.env.example`.
The checked-in MAME runner is `tools/generate_reference_traces.py`; it invokes
`tools/mame_defender_trace.lua` with MAME's Lua `-autoboot_script` mechanism.
Each generated scenario uses an isolated, freshly cleared MAME state directory
under `docs/fidelity/fixtures/local/mame-state/<scenario>/` so NVRAM/config
state cannot leak between local reference traces.

Set `DEFENDER_TRACE_DEBUG=/path/to/debug.tsv` when invoking
`tools/mame_defender_trace.lua` directly to write a local diagnostic stream
with the main CPU PC, selected red-label RAM bytes, and object/shell CRCs.
Debug traces are investigation aids only and are not part of the checked-in
fixture schema.

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
