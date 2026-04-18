# Sounds

Defender keeps its embedded cue files in this directory so audio assets follow
the same pattern as the sibling game repos.

The current `.wav` files are generated from the app's cue definitions and are
embedded with `include_bytes!`, so compile and runtime do not depend on any
external sound directory.
