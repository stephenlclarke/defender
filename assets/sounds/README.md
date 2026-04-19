# Sounds

This directory now holds legacy reference cue assets only.

The live game no longer decodes these `.wav` files at runtime. Audio is
generated directly in Rust from the Williams `VSNDRM1.SRC` routines translated
into `src/audio_rom.rs`, so compile and runtime do not depend on any external
sound directory.
