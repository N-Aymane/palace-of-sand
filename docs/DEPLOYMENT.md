# Build, verify, and release

GitHub repository: https://github.com/N-Aymane/palace-of-sand (private).

The `Verify and build` workflow runs formatting, simulation tests (including the input-only escape replay), strict library/WASM Clippy, a release WASM build, and Chromium/WebKit browser checks. A failed simulation still allows diagnostic artifacts to be collected, but fails the overall workflow.

## Local preview

```sh
npm ci
node scripts/build.mjs
node scripts/serve.mjs
```

The server listens on **127.0.0.1:4173**. It is intentionally local to this device. Windows application-control policy currently prevents local Rust build-script execution on the development machine; download the compiled `palace-of-sand-web` artifact from a successful GitHub run into `dist/` instead.

## Release the tested bundle

Use the successful run's web artifact, preserving `build.json` and its WASM digest. The linked Vercel project is `palace-of-sand`, account scope `team_JAj2uNFkSeGwPoCyuPlOgGY8`.

```sh
node scripts/package-vercel.mjs
npx vercel@64.7.0 deploy --prebuilt --yes --environment preview --scope team_JAj2uNFkSeGwPoCyuPlOgGY8
```

Verify the preview before staging production with the same static output:

For a protected preview, `TEST_PROTECTED=1 TEST_URL=<preview-url> npm run test:browser` uses the linked project's short-lived development OIDC token from the environment or `.env.local`. The token header is restricted to that exact origin. Use `TEST_SCENARIOS=desktop-chromium,mobile-chromium` when only Chromium is installed; Linux CI also covers WebKit.

```sh
npx vercel@64.7.0 deploy --prebuilt --prod --skip-domain --yes --scope team_JAj2uNFkSeGwPoCyuPlOgGY8
npx vercel@64.7.0 promote <verified-staged-production-url> --scope team_JAj2uNFkSeGwPoCyuPlOgGY8
```

The ordinary source build path is also available through `vercel.json` and `scripts/vercel-build.sh`. Prebuilt releases keep the tested WASM artifact identical to the deployed artifact.

Credentials and local OIDC development files are excluded from Git and deployment uploads. Configure CLI authentication directly; do not put tokens in command arguments or repository files.

## Released cinematic edition

Public game: https://palace-of-sand.vercel.app

Released from passing CI run `39449760577`, commit `a5d3292`. Preview was checked with desktop and touch controls before production promotion. Production serves the identical verified WASM digest recorded in `build.json`. The project-scoped alias may require Vercel authentication; share the public game URL above.

## Combat and parry release — 2026-10-21

Version `0.3.0`, source commit `cba37fd`, verified by run `39621192204` (24 Rust/level tests, strict lint, release WASM and four browser scenarios). Public URL remains https://palace-of-sand.vercel.app.

Production deployment: `dpl_FGhMSFtQ48wJwEdQg4fMRgc1unGt`. Verified WASM SHA-256: `cbdb254f69c5e4daef68f07f921ca88facc10d13d636924a1b03b361617602f1`. The public metadata endpoint returns HTTP 200 with this exact build digest. Both the local port-4173 bundle and production use this artifact.

See `COMBAT.md` for controls, timing, practice mode and coverage. Preview HTTP verification used the authenticated Vercel CLI. Development-environment retrieval was rejected by automatic approval review, so no environment payload was downloaded; browser verification also ran locally and on public production.

Production combat checks passed for keyboard and touch. One transient mobile startup timeout was cleared by a fresh run; no gameplay assertion was removed.

## Mobile and safe-fall release — 2026-10-25

Version 0.4.0 is live at https://palace-of-sand.vercel.app. Deployment `dpl_BwGTB2LVWYndYD6hKe65qb1BL249` uses the passing runtime artifact from run `40196162122`. Run `40196288087` added and passed portrait Chromium and simultaneous touch/cancellation coverage without changing runtime code. Total: 28 gameplay/level tests and five browser scenarios. Public production portrait/landscape touch checks passed after promotion.

WASM SHA-256: `5657a76f0378a02b375f09e7361443a9cf474127be1cee6f965dd3491bc82f0f`. Local port 4173 uses the same bundle. The mobile shell and safe-fall behavior are documented in `MOBILE.md`.

## Revised gameplay release — 2026-10-25

Version 0.5.0 (source `64f4672`) passed run `40197719983`: 32 gameplay/level tests, formatting, strict library/WASM Clippy, release WASM and five browser scenarios. Public portrait and landscape touch/combat checks passed after promotion.

Deployment: `dpl_6WpFnx2eKigkHcDHP6111W2b7Xgi`. WASM SHA-256: `592e7758904a98f8818e64d524722a1351c550c92fe25b9f4dab03551d97d411`. Rules: only extremely large drops kill; spikes take one heart per contact; sword pickup supports held action, same-floor reach and a pre-landing input buffer.

## Continued gameplay polish — v0.5.1

Source `d840e9e`, run `40213901337`: 33 gameplay/level tests, strict lint, release WASM and five browser scenarios passed. Added persistent NO SWORD / ARMED inventory status and a pre-landing dangerous-fall warning.

Production: https://palace-of-sand.vercel.app, deployment `dpl_JC36YJbC5TKZ7pcJpj6KXNGoqtH9`. WASM SHA-256: `d360a199bc699a6488c3b2a0d6c2bf49bb0257ae1251678bda741eb4f8e59ccf`. Local port 4173 uses the same bundle.

### v0.5.2 â€” sword identification

- Source: 0ec42a6; GitHub run 40216214978 passed formatting, strict Clippy, 33 gameplay tests, release WASM and five browser scenarios.
- Deployment: dpl_7Bv3ta9THzTiGmcxomRVWP5HdUGM, promoted to https://palace-of-sand.vercel.app.
- WASM SHA-256: 0015c08df5e1f92e9a7bbf2c144cc79b52d10f6530e1017d8b66d1167f77855f. Public build metadata matches the verified bundle. Local port 4173 serves the same release.

