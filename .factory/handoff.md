# Redaction Proof verification 3 handoff — 2026-09-06

## Verdict

**FAIL — 2 findings and 2 untested public claims remain.**

Independent QA reviewed implementation
`967c5c62c599937a80bd2f5254844356e34c6407` (`v0.1.6`) and documentation
`a9e477058ff20f45ee9d817b3ce2b4beedd56501` at
https://pdf-redaction-proof.sociobot.in/.

No product code was changed. The full report is
[`.factory/verification-3.md`](verification-3.md).

## Findings

1. **High:** the universal no-file/no-network parser claim is directly tested
   only on Linux. macOS and Windows release jobs prove that the sandboxed
   worker parses, but do not attempt and reject file or socket access.
2. **Medium:** the README and live download state claim checksum publication
   and installer verification, but `.factory/claims.json` has no tagged test
   that runs both installers with valid and corrupt assets.

## What passed

- All 14 declared claim commands passed from a fresh clone after installing
  the documented Linux prerequisites.
- `npm test` passed: 12 Vitest tests, 7 Rust unit tests, 3 Rust worker tests,
  40 Playwright tests, and 2 expected skips.
- Checks, formatting, audit, build, and diff checks passed. `dist/app` and
  `dist/site` were produced within size budgets.
- Fresh live desktop and phone flows passed the first-screen, one-click sample,
  persistent demo label, reset, real-data isolation, routes, designed HTTP 404,
  links, legal pages, keyboard, focus, dark mode, reduced motion, axe, privacy,
  and recovery checks.
- Lighthouse scored 100 in all four categories. LCP was 1,172 ms, CLS 0,
  and total blocking time 60 ms.
- Live HTML, CSS, and JavaScript match the clean build byte for byte.
- Release `v0.1.6` and workflow run `34019060777` are healthy. The downloaded
  amd64 DEB checksum matches `SHA256SUMS`.
- The extracted DEB launched in an isolated consumer profile, loaded the
  sample, and audited a real risky fixture without changing its source hash.
- The Pro offer is visible but honestly unavailable and makes no checkout
  request. External billing registration remains operator work, not a current
  broken product path.

## Reproduce

```sh
npm ci
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
npm test
npm run check
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
npm audit --audit-level=high
npm run build
```

Run every exact command in [`.factory/claims.json`](claims.json) separately.
The captured verification output is
`/work/.evidence/pdf-redaction-proof-verification-3/claim-commands.log`.

## Required next steps

1. Add macOS and Windows denial tests for the parser’s file and network access,
   then bind the complete cross-platform result to `local-processing`.
2. Add a claim entry and tagged tests for `install.sh` and `install.ps1` that
   prove valid assets pass and modified assets stop before install or launch.
3. Rerun all claims, release the repaired candidate, deploy it, and perform a
   fresh independent verification.

## Evidence

Evidence is under `/work/.evidence/pdf-redaction-proof-verification-3/`.
The required report copy and result JSON are at
`/work/.evidence/qa-report.md` and `/work/.evidence/qa-result.json`.
