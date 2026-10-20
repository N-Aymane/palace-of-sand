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

## Next implementation pass

After the user selects a direction, create actual game-ready environment textures and consistent character animation assets. Keep collision data independent of the new artwork. Improve stride mechanics, weight transfer, anticipation, landings, ledge climbing, cloth motion, and state transitions; interpolate rendering between fixed simulation ticks so smoother presentation does not change deterministic gameplay.

Product Design's ideation workflow requires visual selection before build work. The current game remains playable locally on port 4173 while selection is pending.
