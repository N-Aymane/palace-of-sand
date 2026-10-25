# Mobile handheld and safe falls

The browser shell fills the mobile webview using dynamic viewport height and safe-area insets. Portrait and landscape show the same undistorted 8:5 game screen. The lower controller deck contains a D-pad, A/action and B/parry controls; toolbar actions remain reachable above it. Desktop keeps the original page layout and keyboard input.

Pointer capture permits direction plus action at once. Pointer cancellation, focus loss and orientation changes release held controls to prevent stuck movement. JavaScript only forwards existing keyboard inputs; collision, combat and movement remain in Rust/WASM.

Ordinary landings are safe, but falls exceeding two room heights (378 logical pixels / six floor levels) are lethal. Short unmarked bottom gaps return to the last safe floor; extremely deep ones remain lethal. Spikes take one heart per contact rather than instant death. Hold A/action, Z or Shift near the sword to collect it; no fresh key press on the exact tile is required.

Verification: safe-floor, cross-room fall, spike and bottom-gap regression tests; the full campaign replay; mobile viewport width/height, aspect ratio, bottom control placement, no scroll overflow, simultaneous touch and cancellation, plus keyboard/touch combat checks. Physical-device safe-area behavior still requires a real notched phone; browser emulation covers Chromium and WebKit layouts.
