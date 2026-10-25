# Shrines and campaign

The campaign contains four playable chapters. Chapter I retains the selected DOS opening layout. Courtyard (4 rooms), Cistern (5 rooms), and Palace (6 rooms) are newly authored follow-on levels inspired by the same mechanics; they are not imports of the original game's later maps. Guards, pressure plates, timed gates, spikes, potions, and the Palace's timed chomper provide the challenges. Completing an exit shows NEXT LEVEL and carries the elapsed campaign clock forward. Later chapters start with a sword. Death/restart restarts the current chapter.

Save / Load also offers chapter selection so players can try the new content without completing the opening dungeon first. Starting a chapter replaces the active run but preserves the saved slot.

## Player flow

1. Find a blue orb labelled SAVE and stand near its base, on stable ground and away from a living guard.
2. Open Save / Load. Enter the private save code and select Unlock. Access lasts one hour on this browser.
3. Select Save checkpoint. The confirmation appears only after the server seals the checkpoint and browser storage succeeds.
4. Select Load checkpoint to replace the current run. Loading also works after reloading the page. Press Play to resume explicitly.

There is one slot per browser **and origin**. Production and localhost have separate slots. Clearing site data removes the slot; private browsing may discard it when closed. No account or cross-device synchronization is involved. The previous slot is kept if saving fails. Saves are manual at shrines; reaching a shrine alone does not save.

## Implementation

- `src/campaign.rs` captures versioned mutable state and reconstructs geometry only from trusted embedded chapter data. It validates chapter, room, health, positions, guards, tile states, clock and safe shrine location. Transient movement and held inputs are cleared on load.
- `web/checkpoint.js` transports serialized Rust state, stores one sealed envelope, renders the access menu, and supplies `/palace/checkpoint` through the pinned Miniquad filesystem plugin. It does not implement gameplay.
- `/api/checkpoint` verifies the code server-side and issues a signed, HttpOnly, same-site session cookie. Saving requires this cookie. Checkpoints use AES-256-GCM authenticated encryption; loading rejects altered envelopes. Possession of a valid envelope allows loading, while writing a new valid envelope requires access.
- `SAVE_ACCESS_CODE` and `SAVE_SIGNING_KEY` are server-only Vercel secrets. The local development server reads the ignored `.env.save.local`. Neither secret belongs in web assets, Git, build metadata, logs, or documentation. Rotating the signing key invalidates existing checkpoints and sessions.
- This restricts the application's save feature; it is not a competitive anti-cheat system. A code holder can share access. There are no leaderboards or valuable server-side game records.

The same Node handler is exercised by local HTTP tests and deployed as a Vercel Function alongside the static WASM bundle. The prebuilt packaging script emits the function explicitly; the source `api/checkpoint.mjs` entry supports ordinary Vercel source builds.

## Verification

Rust tests cover chapter validation/progression, checkpoint state preservation and malformed state, the complete original opening route, and controls-only routes through all three new chapters. HTTP tests cover wrong codes, unauthenticated saves, signed sessions, save/load roundtrips, changed envelopes, and cross-origin writes. Browser tests exercise chapter selection, code unlock, a real Rust checkpoint save, load and load again after page reload. Desktop Chromium plus portrait/landscape Chromium and WebKit cover the existing controls and mobile layout.
