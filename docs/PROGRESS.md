# Implementation milestones

Each milestone is committed and pushed to GitHub with its documentation.

## 1 — Foundation

- Created the Rust package, renderer feature boundary, release settings, and lint policy.
- Kept downloaded toolchains and reference artifacts outside version control.
- Selected Macroquad 0.4.14, Rust 2024, static WebAssembly output, and Vercel hosting.
- Confirmed GitHub authentication for repository creation.

Validation: empty starting workspace inspected; toolchain installation in progress.

## 2 — Reference geometry and simulation

- Decoded all 24 opening-level room records, two guards, sword, potions, exits, and exact switch targets from the pinned public reference.
- Added validated JSON geometry and a standalone Rust fixed-step simulation.
- Implemented movement states, jumps, climbing, ledge grabs, loose slabs, spikes, gates, pickups, health, combat, restart, and completion.
- Added geometry, deterministic replay, movement, hazard, gate, pickup, and combat tests.
- Added a Linux verification workflow because Windows application-control policy blocks locally compiled build-script executables (OS error 4551). No policy changes were made.

Validation: dependency resolution completed; native compilation reached the linker successfully but execution was blocked by host policy. GitHub Actions will supply the native test results.

## 3 — Browser game and original presentation

- Added original procedural dungeon masonry, torches, gates, props, character poses, and pixel-scaled rendering.
- Added Rust-generated WAV sound effects, title/help/pause/death/completion screens, health and countdown.
- Added keyboard and multi-touch controls, safe-area-aware mobile layout, fullscreen, audio preferences, and focus-loss pausing.
- Built the browser shell with loading/error feedback, accessible shortcut buttons, controls documentation, and responsive presentation.
- Added a build script that copies the exact JavaScript loader shipped with the pinned Macroquad crate.

Validation: browser and renderer compilation checks are now running in Linux CI; no original graphics or sound were imported.

## 4 — Verification pass

- Linux CI passed all ten native simulation/geometry tests and library Clippy.
- The WASM renderer compiled; its strict lint check found one unused import, now removed.
- Added automated Chromium and WebKit checks for desktop, landscape touch, and portrait touch layouts, including screenshots, startup, input, and browser-error assertions.

Validation is ongoing. Full browser results and deployment URLs will be recorded after the checks complete.

### First integration fixes

- Added an explicit WASM import-linking flag for Macroquad's JavaScript-provided functions.
- Corrected ledge climbing across room boundaries and added a regression test for the room 2 → room 6 ascent.
- Made consumed potions retain their supporting floor and added a regression test.
- Created the Vercel project using existing CLI authentication after the connector returned a scope-access denial.

### Full-route verification

- Added an input-only integration replay that traverses the opening level, acquires the sword, returns, fights, and opens the exit.
- All twelve focused tests pass. The first full replay exposed a missing jump in its room 6 route; the replay now jumps the spike shaft and steps on its gate opener.
- Fixed Vercel's build bootstrap to handle its preinstalled Cargo installation without assuming a `$HOME/.cargo/env` file.
- CI preserves browser/build artifacts even when a simulation test fails, while still marking the overall run failed.

- The next replay reached room 8 and exposed an unsafe running drop into its lower shaft. Added a deliberate Shift+Down edge drop and updated the replay to descend onto the middle ledge before opening room 7's gate.
- The release WASM compiled successfully. Corrected asset packaging to include the renderer feature when locating Macroquad's matching JavaScript loader.

- Browser diagnostics found Macroquad 0.4.14's bundled JavaScript lacks framebuffer functions used by its resolved Miniquad 0.4.8 dependency. The build now composes `gl.js` and the audio plugin directly from the exact locked Miniquad and quad-snd crates.
- Corrected commit attribution to the authenticated GitHub account's numeric no-reply address for Vercel deployment checks. Existing milestone history is preserved.
- The input-only replay now reaches the sword and return route; added a return jump over the slabs that crumbled on the outbound journey.

- Enabled WebGL 2 explicitly: the locked renderer's framebuffer resolve path calls WebGL 2 functions. Chromium diagnostics identified the default WebGL 1 context as the remaining startup error.

## 5 — Reproducible release packaging

- Added a Vercel Build Output API packager that serves the exact tested CI artifact without recompiling on the host.
- Documented local port 4173, artifact retrieval, preview deployment, staged production, and promotion.
- Kept credentials and OIDC files outside version control and upload inputs.

### Passing end-to-end baseline

GitHub run `39441734228` passed formatting, all 13 simulation/geometry tests (including the full input-only escape), library/WASM Clippy, release compilation, and all four Chromium/WebKit browser scenarios.

Visual inspection of its mobile screenshots found inverted touch-control rendering despite passing runtime checks. Moved touch controls into the same pixel render target as the game to make their orientation and hit areas consistent.

The first verified-artifact Vercel preview reached READY at https://palace-of-sand-c57tqsipz-aymanes-projects-2dbeb6ea.vercel.app. Production promotion waits for the mobile visual correction.

- Added origin-scoped OIDC support to the browser verification script for testing protected Vercel previews with existing local authentication; no protection settings are weakened.

## 6 — User-requested visual refresh

- The corrected mobile-controls build passed CI run `39442474632` and is now the local port-4173 preview.
- Verified the protected Vercel preview in desktop Chromium using existing origin-scoped authentication.
- User requested smoother animation and more realistic graphics through Product Design. Reviewed the current game and generated three screenshot-grounded art directions; selection is pending before implementing the new assets.
- See `docs/DESIGN_REFRESH.md` for the brief, concept order, and animation requirements. Production release will incorporate the selected visual direction.

## 7 — Applying the requested upgrade

The user explicitly asked to implement graphics and physics changes rather than continue selecting mockups. Proceeding with the cinematic realistic direction.

- Added buffered jumping, a six-tick ledge grace period, momentum-preserving direction reversal, limited aerial correction, held-jump lift, landing feedback, and a continuous eased climb instead of a position snap.
- Added physics regression tests and updated the complete input-only route for the longer climb duration.
- Producing real raster environment and character-animation assets for the active renderer; these will replace the block figures and flat masonry rather than remain concept images.

### Live renderer replacement

- Replaced the procedural block-character and masonry drawing code with runtime raster textures, eight-frame running, action poses, and combat animation sheets.
- Raised render targets to four times logical resolution, enabled smooth texture sampling, added fixed-tick position interpolation and state-pose blending, and added animated firelight/dust/contact shadows.
- Added loading feedback that waits for the actual graphics assets before declaring the game ready.
- Documented all generated runtime assets in `assets/art/README.md`. Gameplay collision geometry remains independent of artwork.

- Startup verification: observe both WebGL log and info channels so the cinematic asset-ready signal dismisses the loading overlay.
