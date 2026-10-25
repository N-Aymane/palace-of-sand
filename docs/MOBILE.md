# Mobile handheld and safe falls

The browser shell fills the mobile webview using dynamic viewport height and safe-area insets. Portrait and landscape show the same undistorted 8:5 game screen. The lower controller deck contains a D-pad, A/action and B/parry controls; toolbar actions remain reachable above it. Desktop keeps the original page layout and keyboard input.

Pointer capture permits direction plus action at once. Pointer cancellation, focus loss and orientation changes release held controls to prevent stuck movement. JavaScript only forwards existing keyboard inputs; collision, combat and movement remain in Rust/WASM.

Normal-floor landings never damage health, including fast drops between rooms. An unmarked gap below an unconnected room returns the player to a safe floor without losing health. Spikes, timed blade traps and guard attacks retain their hazard rules.

Verification: safe-floor, cross-room fall, spike and bottom-gap regression tests; the full campaign replay; mobile viewport width/height, aspect ratio, bottom control placement, no scroll overflow, simultaneous touch and cancellation, plus keyboard/touch combat checks. Physical-device safe-area behavior still requires a real notched phone; browser emulation covers Chromium and WebKit layouts.
