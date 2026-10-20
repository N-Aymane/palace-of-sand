# Cinematic raster artwork

Generated with the built-in ImageGen tool for this project's graphics upgrade. These are actual runtime assets, loaded by `src/visual.rs`, copied to `dist/art/` by the build script. No original Prince of Persia graphics are bundled.

Art direction: realistic side-view cinematic adventurer, ivory linen and red sash, worn limestone and bronze, warm firelight and cool shadow. The third displayed design concept was the visual reference. All atlas cells use fractions of the actual image dimensions, rather than assuming the generator honored requested pixel sizes.

| File | Purpose / prompt summary |
| --- | --- |
| dungeon-backdrop.png | Orthographic dark limestone background wall with two recessed arches, no gameplay platforms or characters |
| stone-wall.png | Tileable-looking diffuse ashlar masonry, chipped limestone and sandstone, neutral lighting |
| dungeon-props.png | Transparent 4×2 atlas: column, torch, gate, exit arch, sword, potion, skeleton, spikes |
| hero-run.png | Transparent 4×2 atlas, eight natural right-facing run-cycle poses of the same clothed adult adventurer |
| hero-actions.png | Transparent 4×2 atlas: idle, crouch, jump, fall, hang, climb, landing, defeated/resting |
| hero-combat-v2.png | Transparent 4×2 atlas: guard stance, windup, thrust, followthrough, recover, parry, recoil, guard step |

The initial `hero-combat.png` is an unconsumed intermediate. The revised atlas leaves sufficient room for the sword; the thrust uses a measured source region extending into its transparent gutter. Per-frame foot baselines correct small generated alignment differences. Sprite sampling is linear; rendering uses a 1280×800 target for the 320×200 logical game.
