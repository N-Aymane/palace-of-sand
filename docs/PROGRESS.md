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
