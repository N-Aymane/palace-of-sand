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
