# Palace of Sand

A Rust/WebAssembly recreation of the opening dungeon from DOS Prince of Persia, with original cinematic raster artwork and animated character sheets and synthesized sound. Desktop keyboard and mobile touch controls. Static hosting on Vercel.

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

Arrow keys move, jump/climb, and crouch. Shift is careful step / grab / pickup / sword attack. Up parries during combat. Escape pauses. R restarts. Mobile has a multi-touch direction pad and action button.

## Project record

- [Milestones](docs/PROGRESS.md)
- [Reference evidence and fidelity](docs/REFERENCES.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Local preview and deployment](docs/DEPLOYMENT.md)

This is an unofficial fan project. Newly authored code and artwork are MIT licensed; reference material retains its original ownership. No original executable, graphics, or sound files are distributed.
