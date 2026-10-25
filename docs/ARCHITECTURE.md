# Architecture

`src/lib.rs` owns deterministic simulation. It accepts an input snapshot once per fixed tick and emits sound/gameplay events. It has no browser or graphics dependency.

`src/level.rs` validates embedded JSON room data. Rooms have ten columns and three floor rows, explicit adjacency, pressure-plate targets, and entity spawns. Rendering never determines collisions.

`src/main.rs` owns Macroquad rendering, keyboard/touch input, synthesized audio, menus, and the accumulator loop. The browser shell scales the canvas and provides loading/error handling. All game rules execute in Rust.

Tests exercise the library natively. The renderer is compiled separately for `wasm32-unknown-unknown`. Static output is generated locally or in CI and served directly by Vercel.

`src/campaign.rs` advances the four-chapter campaign and captures/validates shrine checkpoints. Saved data contains mutable simulation state; trusted embedded data always supplies the level geometry. `web/checkpoint.js` handles the access menu and browser storage, while `server/checkpoint.mjs` verifies save codes, signs one-hour access cookies, and seals checkpoints. This small Node Vercel Function is the only backend. It stores no player records; the encrypted slot stays on the player's browser. See [save and campaign details](SAVES-AND-CAMPAIGN.md).
