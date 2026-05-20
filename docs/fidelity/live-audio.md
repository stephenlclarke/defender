# Live Audio Acceptance

Status: `DC-51` accepted the contract on `2026-05-13`; `DC-52` added the
first bounded runtime on `2026-05-13`; `DC-59` moved the active runtime surface
to gameplay-facing `SoundEvent` batches on `2026-05-13`; `DC-162` added the
first default synthesized device backend on `2026-05-16`.

This note defines the source-backed contract for live audio runtime code.
R7 adds audible synthesized device output, while exact analog waveform fidelity
remains out of scope.

## Accepted Surface

`FrameOutput::sound_commands()` is the authoritative timing surface for live
audio. Each `SoundCommand` in that frame output is a source-cited main-board
PIA port-B command emitted by the arcade core after one red-label frame step.
Runtime audio code maps that accepted timing to `SoundEvent` batches and must
consume those batches in core-frame order.

`FrameOutput::sound_board` is a diagnostic surface. It preserves the last
latched command, CB1 assertion state, and latch write count, but it is not
sufficient as an audio queue because it intentionally stores only the latest
latched command state.

The deterministic DAC byte signatures in `src/sound.rs` are the content guard
for sound-board synthesis. They prove source-visible byte order for translated
sound-ROM routines, but their tick unit is a translated DAC write, not a
cycle-accurate 6808 CPU or analog sample duration.

## Runtime Contract

- Default live play owns clean `Game` frames and does not compile or step the
  legacy `ArcadeMachine`.
- The audio path receives copies of gameplay-facing `SoundEvent` batches; it
  must not call legacy machine code, mutate gameplay state, or affect trace
  output.
- Audio delivery must be best effort. A missing device, backend creation
  failure, queue overflow, or shutdown race may silence audio, but must not
  change gameplay, CMOS persistence, live smoke behavior, or process exit
  semantics.
- The audio backend is behind a trait with a synthesized device path plus
  disabled and null no-device paths for tests, smoke mode, unsupported
  platforms, missing devices, and explicit disablement.
- The active runtime implements this as `src/audio.rs`: `LiveAudioRuntime`
  queues copied `LiveAudioEventBatch` values on a bounded non-blocking channel,
  `LiveAudioBackend` owns backend behavior, `DeviceLiveAudioBackend` owns host
  output, and `NullLiveAudioBackend` opens no device and emits no audible
  output.
- Sound events retain core-frame order. Accepted command timing remains
  available only through explicit `legacy-tools` fidelity/oracle checks.
- The audio queue is bounded and non-blocking from the presentation and core
  worker perspective. If the queue cannot accept work, the backend records the
  dropped event batch and continues without blocking the core.

## Cadence And Buffering

Core frame cadence is `FRAME_RATE_MILLIHZ` (`60_100` millihertz). Audio event
batches are therefore frame-indexed at the same cadence as live core stepping,
not at redraw cadence.

The runtime should use the host output sample rate reported by the selected
audio backend. Tests should use a deterministic mock rate, with `48_000` Hz as
the default fixture rate. Any DAC-byte-to-PCM conversion must be owned by the
audio thread or backend, not by the arcade core.

Until cycle-accurate sound CPU scheduling exists, source-visible command
timing and deterministic DAC byte order are the acceptance target. The runtime
must not claim exact analog waveform fidelity or exact 6808-cycle sample
spacing.

## Lifecycle Rules

- Pause or suspend: no new core frames means no new event batches. The audio
  backend may drain already queued buffers, then idle.
- Resume: resetting the live core clock creates a timing discontinuity. The
  audio backend should flush pending queued event batches on that boundary.
- Window close and normal exit: stop accepting new event batches, drain or
  drop pending audio according to backend policy, and join the audio thread
  without delaying CMOS save.
- Smoke mode: use a disabled no-device audio path. `--live-smoke` must not
  require or open an audio device, and smoke reports must stay deterministic.
- Normal interactive mode: attempt the synthesized device backend by default.
  If no host output device or supported sample format is available, fall back to
  the null backend without changing gameplay or process exit semantics.
- Explicit disablement: `--mute` disables live audio event delivery entirely.
- CMOS persistence: audio owns no CMOS state. The final core-owned CMOS snapshot
  after live shutdown remains the persistence source.

## Acceptance Matrix

The checked-in fixture
`assets/red-label/live-audio-acceptance.tsv` lists the paths that must remain
covered before audible output can be enabled:

- `coin_credit`: `trace-requirements.tsv` and the `CNSND` timeline guard
  command `0xE6`.
- `one_player_start`: `trace-requirements.tsv` and the `ST1SND` timeline guard
  command `0xF5`.
- `thrust_start_stop`: the live thrust frame test and thrust command-sequence
  fixture guard commands `0xE9` and `0xF0`.
- `smart_bomb`: the translated smart-bomb entry test and `SBSND` timeline guard
  commands `0xEE` and `0xE8`.
- `hyperspace`: the local exact `hyperspace` reference trace guards frame
  command timing, and the `HYPER` DAC signature test guards content if command
  code `25` reaches the sound board.
- `player_explosion`: player-death tests, `PDSND` timelines, and direct
  `PDTH5` / `PLE2` `SNDOUT` rows guard commands `0xEE`, `0xE8`, and `0xEC`.
- `terrain_blow_start`: the terrain-blow start test and `AHSND` timeline guard
  command `0xEE`.
- `terrain_blow_complete`: the terrain-blow completion test and `TBSND`
  timeline guard commands `0xEB`, `0xEE`, and `0xE8`.
- `high_score_entry`: the local exact `high_score_entry` reference trace and
  high-score flow fixture guard command timing. No additional high-score-only
  sound command is accepted without source or MAME evidence.

`assets::tests::live_audio_acceptance_matrix_is_fixture_backed` fails if the
matrix drops a required path, references a missing sound-table timeline label,
or names a command that is not present in the embedded sound command fixtures.

## Non-Blocking Gaps

The following work remains out of scope for the first live audio runtime
prototype:

- cycle-accurate 6808 CPU scheduling,
- analog filtering and hardware speaker characterization,
- external waveform fixtures,
- MAME-derived command-sequence fixtures beyond the existing trace columns,
- untranslated sound-ROM routines not yet covered by deterministic DAC byte
  signatures.

Those gaps affect later audio fidelity work, but they do not block an event-fed
live audio backend that can be disabled and tested through the surfaces above.
