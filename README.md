# Palace of Sand

A Rust/WebAssembly recreation of the opening dungeon from DOS Prince of Persia, with original cinematic raster artwork and animated character sheets and synthesized sound. Desktop keyboard and mobile touch controls. Static hosting on Vercel.

Play: https://palace-of-sand.vercel.app

## Development

Install stable Rust, then `rustup target add wasm32-unknown-unknown`.

```sh
cargo test --lib
cargo fmt --check
cargo clippy --all-targets -- -D warnings
node scripts/build.mjs
node scripts/serve.mjs
```

Open http://127.0.0.1:4173. The current cinematic edition adds interpolated animation, momentum, buffered jumps, ledge grace, and continuous climbing. Build output is `dist/`. The build script uses the workspace-local toolchain when present, otherwise Rust from PATH.

## Controls

Arrow keys move, jump/climb, and crouch. Shift is careful step / grab / pickup / sword attack. Hold X (or Up near a guard) to block; time the raise just before impact to parry and stagger. Z or Shift strikes; hold to repeat. T / Sword practice starts an immediate duel. Escape pauses. R restarts. Mobile fills the webview with a handheld layout: the game screen above, a bottom D-pad, A/action and B/parry buttons, and simultaneous touch support. Normal stone landings cause no fall damage; unmarked bottom gaps return to the last safe floor. Spikes and blade traps remain hazardous.

## Project record

- [Milestones](docs/PROGRESS.md)
- [Reference evidence and fidelity](docs/REFERENCES.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Local preview and deployment](docs/DEPLOYMENT.md)

This is an unofficial fan project. Newly authored code and artwork are MIT licensed; reference material retains its original ownership. No original executable, graphics, or sound files are distributed.
