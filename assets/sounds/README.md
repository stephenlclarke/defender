# Sounds

This directory is reserved for new non-legacy sound artifacts.

Pre-existing prototype `.wav` cue files are legacy assets and live under
`assets/arcade/`. Do not move those archived cues here unless a later cycle
explicitly reclassifies them with stronger provenance. Source-derived command
and timing evidence still belongs under `assets/red-label/`.

The clean-slate runtime does not decode archived prototype `.wav` files and
does not embed them into the shipped binary. They remain checked in only as
legacy reference cues under `assets/arcade/`.

R7 live audio uses synthesized device output from clean `SoundEvent` batches.
That backend does not introduce new sound artifacts and does not reclassify the
archived prototype `.wav` cues. Missing host output falls back to the no-device
null backend; `--mute` disables audio delivery entirely.

Exact audio must be implemented from raw main-board sound command writes and
translated Williams sound-board routines from `VSNDRM1.SRC`. The clean-slate
runtime now has a source-cited sound-board RAM/PIA/ROM address surface and the
MAME-documented main-board PIA1 port-B data-direction-filtered command output
to command-latch byte/CB1 handoff. The sound CPU can read the latched command
through PIA IC4 port B, and PIA IC4 port A writes are captured at the DAC
boundary. Command CB1 now drives the sound PIA IRQ state. The `VSNDRM1.SRC`
`SETUP`, IRQ command decoder, normal `IRQ1` `GWAVE` / `VARI` command runner,
`SP1` / `BG1` / `BG2INC` / `LITE` / `BON2` / `BGEND` / `APPEAR` / `TURBO` /
`THRUST` / `CANNON` / `RADIO` / `HYPER` / `SCREAM` / `ORGANT` / `ORGANN`
special command runners, `GWLD` loader for `SVTAB` / `GWVTAB` / `GFRTAB`,
`VARILD` loader for `VVECT`, `SP1` / `BON2` / `BG2` pre-loop setup paths, the
`GWAVE` / `GPLAY` per-period DAC byte stream, the `VARI` / `VSWEEP` per-sweep
DAC byte stream, the `LITE` / `APPEAR` shared `LITEN` random-complement byte
stream, the `TURBO` / `NOISE` noise-decay byte stream, the `HYPER` phase-edge
DAC byte stream, the `BG1` / `THRUST` first-window `FNOISE` byte streams, the
`CANNON` / `FNOISE` filtered-noise decay byte stream, the `RADIO` / `RADSND`
timer-table byte stream, the `SCREAM` echo-cascade byte stream, the `ORGANN` /
`ORGNN1` first-duration `ORGAN` note byte stream, the `ORGNT1` / `ORGTAB`
organ tune byte streams, the source `IRQ` PIA command-read/CB1-clear prelude,
the source `IRQ3` background handoff, command return/readiness classification,
source-shaped `IRQ1` command-to-`IRQ3` background step, the top-level source
IRQ organ gate, the source IRQ organ-continuation gate, the source IRQ
prelude-to-flow cycle, and the shared `GEND` / `GEND40` / `GEND50` /
`GEND60` / `GEND61` echo and frequency-window updates, plus the source NMI
diagnostic checksum-to-VARI branch, are modeled through the
source-visible background/spinner/bonus plus GWAVE/VARI direct-page state
gates. Phase 10 acceptance uses source-visible command timing plus deterministic
DAC byte signatures as the documented audio tolerance rather than external
waveform files or hardware-cycle DAC reconstruction. The 12 promoted local MAME
reference scenarios exact-match the trace-required `0xE6` credit and `0xF5`
start command/event evidence wherever credited play is required.
Until commands are source-cited or fixture-verified, gameplay code must not map
Rust events such as `FirePressed` or `SmartBombPressed` to invented audio cues.
