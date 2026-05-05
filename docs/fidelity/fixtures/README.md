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
CRC values needed to compare behavior. `video_crc32` is the MAME/Rust-visible
Defender pixel-nibble CRC convention: decode the visible `292x240` cabinet
window from Williams video RAM, store one 4-bit palette nibble per visible
pixel, and CRC those nibble bytes. Do not store ROM bytes, screenshots, or raw
pixel dumps in checked-in fixtures.

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
