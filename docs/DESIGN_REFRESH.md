# Graphics and animation refresh

Requested 2026-10-20 through Product Design: make the graphics and animation substantially smoother and more realistic.

## Current-state review

The playable baseline is correct and verified, but its rectangle-based figure lacks natural anatomy, its state poses change abruptly, and its masonry/lighting look flat. Keep the original opening-level geometry, Rust/WASM architecture, keyboard/touch controls, and Vercel target.

## Visual exploration

Three independent concepts were generated with the built-in ImageGen tool, each using the actual desktop gameplay screenshot as its reference. The concepts are shown in chat in this order:

1. Classic rotoscoped remaster — natural anatomy and detailed stone, close to classic side-view staging.
2. Painted historical realism — softer sandstone, expressive running pose, painterly depth and cloth.
3. Cinematic physical realism — sculptural materials, more dramatic natural light, a prerendered 2.5D aesthetic on the same 2D gameplay plane.

Each prompt requested a single 1536×960 game viewport, preserved platform/pit positions, ivory clothing and red sash, realistic proportions, minimal health/timer HUD, and no surrounding website redesign. These are art-direction concepts, not implemented graphics or animation assets.

## Implemented direction

The user subsequently instructed immediate implementation of graphics and physics. The cinematic physical realism direction is now implemented with runtime PNG textures and character sheets, a four-times-resolution render target, pose transitions, and fixed-step position interpolation. The concepts above are reference images; the playable game uses newly generated modular assets documented in `assets/art/README.md`.

Physics now includes momentum on reversals, jump buffering, ledge grace, held-jump lift, air correction, and continuous climbing. Geometry and deterministic simulation remain separate from presentation. See `design-qa.md` for rendered verification and remaining polish.
