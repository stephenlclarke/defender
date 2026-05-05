# Sounds

This directory now holds legacy reference cue assets only.

The clean-slate runtime does not decode these `.wav` files and does not embed
them into the shipped binary. They remain checked in only as archived prototype
reference cues.

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
gates, but CPU IRQ scheduling, cycle-accurate DAC scheduling, and the remaining
waveform routines are not translated yet. `DC-21.2` rechecked the local MAME
`start_game` command evidence for the trace-required `0xE6` credit and `0xF5`
start command frames; external waveform goldens are still absent.
Until commands are source-cited or fixture-verified, gameplay code must not map
Rust events such as `FirePressed` or `SmartBombPressed` to invented audio cues.
