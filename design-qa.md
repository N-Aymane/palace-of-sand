# Cinematic upgrade design QA

Source visual truth: `C:/Users/ayman/.codex/generated_images/01a15953-2188-7a12-b1b0-8895683283c2/exec-00ef9ed6-ae25-47c8-888d-f84186cbcfcd.png` (1586 x 992).

Implementation: `test-results/cinematic-verified/desktop-chromium-playing.png` (1440 x 1203 full page, CSS viewport 1440 x 1100, deviceScaleFactor 1). Mobile: `test-results/cinematic-verified/mobile-webkit-playing.png` (2532 x 1170 physical pixels; landscape CSS viewport 844 x 390, deviceScaleFactor 3).

State: opening dungeon after start and landing, before sword pickup. The source is a gameplay-only art direction, while captures include the existing website frame. Comparison uses the game content, allowing for the fixed 320 x 189 simulation geometry and surrounding page. The source hero faces right; the actual starting character faces left.

Full-view evidence: source and both rendered screenshots opened together in one comparison input. Dungeon arches, stone platforms, human anatomy, ivory clothing/red sash, and warm torch accents follow the selected direction. The source is a composition reference, not a pixel-perfect room artwork requirement. Modular geometry and lighter foreground masonry are intentional for collision readability.

Focused evidence: reviewed the live local viewport for foot contact, character silhouette and responsive controls; inspected enlarged desktop HUD labels in the full-resolution capture.

## Findings and comparison history

- P2: low-density HUD glyphs appeared blurred next to high-resolution artwork. Fixed by rasterizing interface text at four times logical density and scaling its drawing coordinates. Reopened the source and revised desktop/mobile screenshots together: HUD labels are now crisp at the same viewport and layout.
- Loading overlay initially did not respond to the graphics-ready info signal. Fixed the console channel observer; local browser now shows CINEMATIC EDITION / READY and playable raster artwork.

- P1 runtime follow-up: mobile verification caught a deleted font-texture binding after typography changes. Prewarming the glyph atlas before drawing addresses the cause; CI run 39449760577 passed all four browser scenarios with no console errors after this fix. Source and final desktop/WebKit captures were reopened together; no actionable visual mismatch remains.

## Required fidelity surfaces

- Typography: existing editorial shell preserved; HUD and touch labels now use higher-density rasterization. Revised capture inspected; the label blur is resolved.
- Spacing/layout: room geometry and viewport aspect ratio preserved; touch controls remain below the gameplay region, with no horizontal overflow in browser checks.
- Colors: cool dark background stone, warm foreground stone, ivory/red character and amber fire follow the concept palette. Foreground intentionally brighter than the mock for legibility.
- Image quality: real PNG artwork, transparent character/prop sheets, smooth texture sampling and 1280-pixel render target replace procedural placeholder figures and stone. No visible alpha rectangles in inspected idle/run states.
- Copy: contextual sword/exit hint, health/time and controls remain functional. No design-brief language leaked into gameplay.

## Verification

Local desktop Edge and mobile Chromium emulation passed startup/input/overflow/console checks. CI run 39449760577 passed Rust tests including full input-only level completion, formatting, strict Clippy, release WASM build and all four Chromium/WebKit scenarios. Mobile WebKit is an engine emulation check, not a physical iPhone test.

## Implementation checklist

- [x] Replace live renderer and runtime assets.
- [x] Implement and test smoother movement physics.
- [x] Fix asset-ready loading overlay.
- [x] Inspect revised high-density typography build.
- [x] Deploy and verify the bundle at https://palace-of-sand.vercel.app.

## Follow-up polish

P3: additional transition frames could reduce residual pose variation in generated sheets; small bottom hint text retains the classic compact font. These do not block play or the requested visual upgrade.

final result: passed
