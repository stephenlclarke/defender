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
boundary. Command CB1 now drives the sound PIA IRQ state, but CPU IRQ
scheduling, DAC sample generation, and sound routines are not translated yet.
Until commands are source-cited or fixture-verified, gameplay code must not map
Rust events such as `FirePressed` or `SmartBombPressed` to invented audio cues.
