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
