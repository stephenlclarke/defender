# Fidelity Trace Fixtures

This directory defines the local trace fixture layout used while translating
red-label behavior into Rust.

The checked-in repository must not contain ROM payloads or generated trace data
derived from copyrighted ROM bytes. Keep generated fixtures under the ignored
local directory:

- `docs/fidelity/fixtures/local/`
- `docs/fidelity/fixtures/local/reference/` for MAME/source reference fixtures.
- `docs/fidelity/fixtures/local/rust-current/` for optional exact Rust-current
  comparison fixtures.

Fixture pairs use the same stem and these suffixes:

- `<scenario>.inputs.txt`: semicolon-separated cabinet action sets, one frame
  per segment, using the same syntax as `--fidelity-trace-inputs-file`.
- `<scenario>.expected.tsv`: expected red-label trace rows generated from MAME
  or assembled source for the same input script.

Example pair:

- `boot.inputs.txt`
- `boot.expected.tsv`

The expected TSV header must match `assets/red-label/trace-schema.tsv`. Trace
rows should contain only observable numeric state, checksums, event values, and
CRC values needed to compare behavior. Do not store ROM bytes in fixtures.

Run local fixture checks with:

- `cargo run -- --fidelity-check-trace-dir docs/fidelity/fixtures/local/rust-current`
- `cargo run -- --fidelity-list-scenarios`
- `cargo run -- --fidelity-write-scenario-inputs docs/fidelity/fixtures/local/reference`
- `cargo run -- --fidelity-check-reference-trace-dir docs/fidelity/fixtures/local/reference`
- `make trace-fixtures`
- `make reference-inputs`
- `make reference-traces`
- `make reference-fixtures-check`

`make trace-fixtures` uses `FIDELITY_TRACE_FIXTURES`, which defaults to
`docs/fidelity/fixtures/local/rust-current`.
`make reference-traces` uses `DEFENDER_MAME`, `DEFENDER_ROM_DIR`, and
`DEFENDER_REFERENCE_TRACE_DIR`, which defaults to
`docs/fidelity/fixtures/local/reference`; see
`docs/fidelity/local.env.example`.
