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

- Visual QA found low-resolution HUD glyphs against the new artwork. Labels now rasterize at four times the simulation density to match the 1280-pixel render target. Local Edge desktop and touch checks passed; the live local tab shows cinematic artwork.

- Final typography build passed Rust/replay/lint/WASM checks and desktop Chromium. Mobile CI capture timed out waiting for an animated canvas to stabilize; changed the test to capture its bounding rectangle directly and allow slower software-rendered CI frames. No gameplay assertion was removed.

- Final local mobile verification caught a Macroquad font-atlas texture invalidation when introducing large glyph sizes mid-frame. Prewarm all interface glyph sizes before queuing rendering; keep the browser console-error assertion as the regression gate.

### Verified cinematic release

- CI run `39449760577` passed all Rust tests, the complete deterministic escape replay, formatting, strict library/WASM Clippy, release compilation, and desktop Chromium, mobile Chromium, landscape WebKit, and portrait WebKit browser scenarios.
- The verified bundle is now served locally on port 4173. WASM SHA-256: `a8cacc0ce43b4bfd061bd4d31525750437c6bf6b3352ef9cee2469cfafcc2825`.
- Visual QA compared the selected direction and final desktop/mobile renders; blocking issues were corrected. See `design-qa.md`.
- Updated Vercel preview: https://palace-of-sand-ch2be9xt4-aymanes-projects-2dbeb6ea.vercel.app.

- Verified protected preview with desktop and touch input, then promoted the same bundle to https://palace-of-sand.vercel.app. Unauthenticated production `build.json` returned HTTP 200 with the verified WASM digest.
- Protected-preview test authentication is now explicit (`TEST_PROTECTED=1`), so development OIDC headers are never attached to public production tests.

- Production desktop Chromium and mobile touch verification passed. Local preview remains http://127.0.0.1:4173/; all milestone code and documentation are pushed to the private GitHub repository.

## 8 � Working sword combat and timed parry (2026-10-21)

- Replaced single-tick attack detection with an active strike window and one hit per swing. Holding Shift/Z repeats strikes, and late recovery taps buffer the next swing.
- Fighters keep facing their opponent while retreating; strikes stop movement and cannot cancel hurt recovery. Dead or struck guards cannot finish cancelled attacks.
- Added X and a dedicated mobile PARRY button. Hold to block; raise defence within twelve simulation ticks of impact for a perfect parry, guard stagger and a counterattack opening. Up still defends in a nearby sword encounter.
- Added visible HIT/BLOCK/PARRY feedback and combat hints. Added Sword practice (T) for an immediate repeatable encounter; campaign sword pickup is preserved.
- Added combat regression coverage and browser input checks that defend against a live guard, then hold keyboard/touch attack through three successful hits.

- First CI pass confirmed all simulation tests and full escape replay. Fixed a strict WASM logging-macro lint before browser verification; added whole-blade obstruction checking and a wall regression case.

- Obstruction test edits the level's validated ASCII row representation, matching the renderer-independent geometry format.

- Run `39618665118` passed 24 Rust/level tests, lint and release compilation. Desktop browser combat passed; CI mobile landed two strikes before its capture/input deadline. The identical build passed desktop and actual emulated-touch combat locally in Edge. Prevented canvas long-press context menus and allowed slower software-rendered browser checks more time without weakening the three-hit assertion; failures now save their screenshot and console log.

### Combat release verification

- CI job for run `39621192204` passed 23 focused tests plus the full campaign replay, formatting, strict library/WASM Clippy, release compilation, and all four browser scenarios.
- Both desktop Chromium keyboard and mobile Chromium touch defended against a real guard strike, then held attack through three confirmed hits. WebKit landscape and portrait checks passed.
- Local Edge also passed the keyboard and touch combat flow. Protected preview HTTP checks returned the v0.3.0 bundle using existing Vercel authentication; no project environment download was needed.

- Promoted v0.3.0 to https://palace-of-sand.vercel.app. The public build metadata confirms the verified WASM digest; local port 4173 is updated to the same bundle. See `docs/COMBAT.md` for keyboard, touch and Sword practice instructions.

- Production keyboard combat passed. A mobile startup request timed out once; a fresh public mobile check then passed defence and all three touch strikes. CI also passed all four browser scenarios.

## 9 � Full-screen mobile handheld layout (2026-10-25)

- Replaced the small canvas-with-controls presentation on touch devices with a full-height webview shell, safe-area padding, a separate game screen, and a bottom-mounted handheld control deck.
- Added a physical-style direction pad and A/action and B/parry buttons. Pointer capture supports simultaneous controls and releases held inputs on cancellation, focus loss and orientation changes. All gameplay remains in Rust; the adapter only sends existing keyboard inputs.
- Gameplay keeps its 8:5 aspect ratio; the mobile deck fills the rest of the screen without stretching the game or requiring page scrolling. Added portrait/landscape viewport and control-placement browser assertions.

### Safe falls (requested during mobile work)

- Removed all height-based landing damage on ordinary floors. Spikes, blades and combat retain their explicit damage rules.
- Unmarked gaps below an unconnected room now return the player to the last safe floor without losing health instead of killing them.
- Added regression cases for a high-speed long fall at one health, a multi-room drop, a spike landing, and recovery from an unmarked bottom gap. Release version: 0.4.0.

- Local desktop and landscape touch layout/combat checks passed. Added explicit portrait Chromium sizing and simultaneous direction/action cancellation checks alongside the existing WebKit portrait/landscape checks.

### Published mobile and safe-fall release

- Released v0.4.0 from passing run `40196162122` (source `a5756e5`). The expanded portrait/multi-touch suite also passed run `40196288087` (test-only commit `b1bdc03`): 27 focused tests, the complete campaign input replay, formatting, strict native/WASM Clippy, release WASM, and five Chromium/WebKit scenarios.
- Public production portrait and landscape Chromium checks passed, including full webview dimensions, bottom controls, no page scrolling, two-finger input/cancellation and touch combat.
- Production deployment: `dpl_BwGTB2LVWYndYD6hKe65qb1BL249`; https://palace-of-sand.vercel.app. WASM SHA-256: `5657a76f0378a02b375f09e7361443a9cf474127be1cee6f965dd3491bc82f0f`.
- Local http://127.0.0.1:4173 serves the same v0.4.0 runtime bundle. See `docs/MOBILE.md` for layout, input and fall rules.

## 10 � Revised falls, single-heart spikes and reliable sword pickup

- User clarified that truly large falls should kill. Only drops exceeding two full room heights (378 logical pixels / six floor levels) are lethal; shorter drops remain safe. Huge falls also remain lethal during combat damage immunity.
- Spikes remove one heart per contact instead of instant death. Staying on the same spike tile does not repeatedly drain health; leaving and entering again can inflict another hit. A final remaining heart can still be lost.
- Sword pickup accepts holding action while approaching, a nearby same-floor reach of 28 pixels, and a short pre-landing tap. Collection is recorded once and preserves the supporting floor.
- Added regression tests for lethal huge falls, repeated spike contact, last-heart spike damage, held/nearby pickup, pre-landing pickup and floor separation.

### Continued gameplay polish

- User asked to continue improving the game. Added permanent NO SWORD / ARMED status to make weapon collection unambiguous after the pickup cue fades.
- Added a steady FALL! warning before a drop crosses the lethal threshold, with an actual multi-room falling replay verifying that the warning appears before death and clears afterward.
- v0.5.0 passed 32 gameplay tests, strict lint, release WASM and five browser scenarios (run `40197719983`). This follow-up polish is v0.5.1.

- Promoted the verified v0.5.0 bundle to https://palace-of-sand.vercel.app. Public portrait and landscape touch checks passed. Local port 4173 serves the same gameplay fixes while the feedback polish build is verified.

### Verified feedback-polish release

- v0.5.1, source `d840e9e`, passed run `40213901337`: 32 focused tests plus the complete campaign replay, formatting, strict library/WASM Clippy, release compilation and all five desktop/portrait/landscape Chromium/WebKit scenarios.
- Added permanent equipment status and a steady warning during dangerously long falls. The HUD uses already-prewarmed font sizes, preserving the high-density renderer and touch layout.
- Promoted deployment `dpl_JC36YJbC5TKZ7pcJpj6KXNGoqtH9` to https://palace-of-sand.vercel.app. Verified artifact SHA-256: `d360a199bc699a6488c3b2a0d6c2bf49bb0257ae1251678bda741eb4f8e59ccf`.

- Public v0.5.1 portrait and landscape touch/combat checks passed. Production metadata matches the tested artifact exactly. All code, controls and release documentation are pushed to the GitHub repository.

### Sword identification fix (v0.5.2)

- User screenshot showed the room 11 skeleton, not the collectible sword. Its full atlas cell was compressed vertically, making bones resemble a blade.
- Crop skeleton art to its visible bounds, enlarge the actual sword slightly, and label it SWORD with a pointer. Nearby bones now have an explicit contextual hint. Pickup rules and level geometry are unchanged.
- Verification pending CI release build and browser checks.

- v0.5.2 passed run 40216214978 (33 gameplay tests and five browser scenarios) and is deployed publicly and on local port 4173. Public metadata matches the verified artifact.
- Public portrait Chromium touch/layout smoke check passed after promotion.

### Code-protected shrines and campaign (v0.6.0)

- Added versioned, validated Rust checkpoints at safe shrines. Save authorization and authenticated encryption run on the server; one sealed save slot stays in the browser.
- Added three original follow-on chapters (Courtyard, Cistern, Palace), carrying the campaign timer forward. Later chapters start armed. Chapter selection is available in Save / Load.
- Added code unlock dialog, save/load bridge through the Macroquad filesystem plugin, stable save markers and a timed chomper trap.
- Added API authorization/tamper tests, checkpoint roundtrip tests, campaign transition tests and input-only playthroughs. Full verification is pending CI.
