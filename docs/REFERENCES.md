# Reference ledger

## Selected reference

SDLPoP commit `3c5add5fb7f83d4ceb542823ab66d00146c4271b` (retrieved 2026-10-20): https://github.com/NagyD/SDLPoP/tree/3c5add5fb7f83d4ceb542823ab66d00146c4271b

- `src/types.h`: documents the 24-room, 10-column, 3-row level format and room adjacency.
- `data/LEVELS/res2001.bin`: public opening-level layout reference; downloaded locally for inspection, excluded from Git and deployment.
- https://macroquad.rs/articles/wasm/ — official WebAssembly loading instructions.
- https://vercel.com/docs/build-output-api — static deployment format.

## Evidence boundaries

The project uses public source/data documentation. No DOS executable has been supplied or executed; no REA binary-analysis evidence is claimed. The reverse-engineering skill routes complete source inspection through normal tools.

The Rust simulation and artwork are newly authored. Movement tuning, collision behavior, combat AI, and sound are approximations unless a specific measurement is recorded here. This project does not claim binary-identical DOS behavior.

## Opening-level observations

Reference record SHA-256: `ae8dd7787c65e2ecd2a21c6126c751c47e9a614edde0a80af0c86d8afeadc06c` (2305 bytes).

Observed by offline format decoding: start room 1; loose slab at room 1 tile 26; sword at room 15 tile 22; guards in rooms 3 and 21; exit at room 9 tile 13. The room 9 tile 0 opener targets that exit. All 24 room links and switch event lists are preserved in the JSON transcription.

`scripts/import-level.mjs` documents the independent decoder and can reproduce the transcription from the reference record. The record itself, original art, sounds, and SDLPoP source code are excluded from distribution. `assets/opening.json` transcribes the original game's arrangement and is reference-derived data, not newly authored level design; original ownership is retained.

Design choices, not DOS measurements: simulation at 60 Hz; run speed 100 pixels/second; jump impulse 225 pixels/second; gravity 600 pixels/second squared; simplified guard windups and gate timers. Animation uses original raster character sheets, render interpolation, and short state transitions. Exact DOS animation tables, timing quirks, and exploits are not reproduced.

## Accessible fall rules (v0.4.0)

At the user's request, ordinary landings no longer inflict damage based on drop height. Falling below a room without a lower connection recovers to the last safe supporting floor without health loss. Explicit hazards retain their own damage behavior. This intentionally differs from the original DOS fall-damage rules.
