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

R9-E3.16 replaces the previous guessed live cue mapping with command-decoded
Williams sound-board synthesis. Clean `SoundEvent` values now route through
translated source command families for source GWAVE vectors, VARI sweeps,
lightning/materialize/turbo/cannon/radio/hyper/scream routines, thrust looping,
and coin/start/laser/smart-bomb/hyperspace command playback. This still creates
no new tracked sound artifacts; reference-video audio comparison is handled by
the local media harness when captured media is available. `ffmpeg` is available
locally at `/opt/homebrew/bin/ffmpeg` for the harness extraction step.
`make reference-media-check` accepts the separate MAME capture WAV through
`REFERENCE_AUDIO`, so clean candidate WAV output can be compared without relying
on container audio.
`make reference-clean-capture` writes an ignored sound-event TSV beside each
candidate GIF/WAV, which lets clean `SoundEvent` command timing be compared
against MAME trace `sound_commands` before judging waveform differences.
MAME captures can also write ignored per-frame sound-board DAC traces as
`target/reference-media/mame/traces/<basename>.sound-dac.tsv`; the isolated
`0xFE`, `0xFA`, `0xF8`, and `0xF3` non-lander command clips use those traces
to separate command timing failures from waveform-density failures.
Bounded clean candidate WAV output is rendered from the start of the input
program and trimmed after synthesis, preserving materialize/start/catch sound
tails that begin before the encoded visual window.
Foreground Williams sound-board commands are treated as one DAC stream in the
clean mixer, so a new foreground command interrupts the previous foreground
command instead of stacking another independent voice. The reference-media
checker uses a stochastic-noise gate for Defender noise commands, comparing
envelope, RMS ratio, peak level, and zero-crossing behavior when exact random
sample phase is not stable evidence. The default zero-crossing floor is set to
cover the measured low-frequency MAME pre-shot background window.
The GWAVE path models the source `WVDECA` routine as 8-bit wrapping
ROM-waveform subtraction, and clean cabinet DAC gain is calibrated against the
local MAME narrow `DP1V` / `0xFC` enemy-shot window. The `0xF7`
falling-human catch vector uses the source one-byte catch pattern with the
measured catch-window pitch/density calibration from the state-steered MAME
clip. The VARI path now follows the source sweep restart loop and uses a
separate VARI DAC gain calibrated against the state-steered safe-landing
`ALSND` / `0xE0` MAME window. The bounded `down029/fire2524` target6
non-lander shot/explosion/materialization media report now passes MAME audio
with envelope correlation `0.714`, RMS ratio `1.192`, and zero-crossing ratio
`1.298`. The standalone `0xEE` human-loss/lightning command now carries a
longer source-shaped tail so the `hold-up` pickup/pull media report passes the
MAME conversion/loss window with envelope correlation `0.613`, RMS ratio
`1.066`, and zero-crossing ratio `1.076`.
The tonal non-lander hit commands `0xFE` and `0xFA` use the same source GWAVE
vectors as the sound ROM, but their clean period density is calibrated against
isolated local MAME DAC/audio captures so the bomber-hit and pod-hit reports
pass without adding a global DAC-hold mixer.
The `0xEA` materialize command uses the source `APPEAR` / `LITEN` frequency
sweep cadence instead of a high-frequency placeholder; the down030
laser-plus-materialize all-axis report now passes with envelope correlation
`0.864`, RMS ratio `1.008`, and zero-crossing ratio `0.971`.
`make readme-media` also writes an ignored candidate WAV from the clean
`SoundEvent` timeline so `make reference-media-check` can compare real
candidate audio rather than a silent GIF container. The current README attract
candidate has no emitted sound events, so that WAV is correctly silent until a
gameplay-aligned candidate clip is generated.

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
