# Redaction Proof repair handoff — 2026-09-06

## Product and release

Redaction Proof is a local desktop check for people sharing converted or
redacted PDFs. It finds recoverable text and risky PDF structures, creates a
separate cleaned copy, and writes a JSON proof. It is a technical aid, not a
legal guarantee.

- Implementation release: `967c5c62c599937a80bd2f5254844356e34c6407`
  (`v0.1.6`).
- Static deployment: `88c484f0-23d1-4c9e-a5f7-b5bbf682d160`, succeeded.
- Live URL: <https://pdf-redaction-proof.sociobot.in>.
- The `v0.1.6` GitHub Actions run is
  [34019060777](https://github.com/B-Divyesh/sf-pdf-redaction-proof/actions/runs/34019060777).
  It completed successfully: all three clean test/build jobs and the checksum
  manifest job passed.

## Repairs

The earlier verification report is retained in
[`.factory/verification-2.md`](verification-2.md). Its current findings were
handled as follows.

- **Broken Pro checkout:** the hosted Sociobot checkout endpoint returned
  HTTP 404. This product cannot register that shared offer. The site and app
  now show the actual US$12 one-time Pro scope, but disable the purchase
  control and plainly say that billing registration is needed. They make no
  billing request, so visitors do not see a false loading state or console
  error. Free single-file checking, cleaning, and JSON proof remain available.
  `/work/.evidence/billing-offer.json` records the exact offer and named
  operator dependency.
- **Claim coverage:** `.factory/claims.json` now has 14 public claims. Each
  has one tagged outcome test. The price claim proves the displayed price,
  free scope, disabled purchase state, status message, and absence of checkout
  requests.
- **False audit loading state:** the desktop UI uses a real `hidden` rule, so
  progress disappears when an audit completes or errors.
- **Phone demo:** the direct demo has populated output in the first viewport,
  a sticky `Demo — sample data, nothing is saved` label, Reset demo, and a
  separate `demo:` storage namespace.
- **Touch targets, routes, and words:** standalone phone links meet the 44 px
  target minimum; unknown URLs return the designed page with HTTP 404; the
  previously metaphorical headings are plain section and error names.
- **Release reproducibility:** removed an unused Tauri opener plugin. The
  lockfile now pins an available `http-body-util` release, found during the
  documented clean install rather than hidden by a warm Cargo cache.

## Verification

From the documented clean setup:

- `npm ci` installed 66 packages and reported zero vulnerabilities.
- `npm run check`, `cargo fmt -- --check`, `git diff --check`, and
  `npm audit --audit-level=high` passed.
- `npm test` passed: 12 Vitest assertions; 7 Rust unit tests; 3 Rust worker
  tests; 40 Playwright tests; 2 expected mobile-only skips. The sole ignored
  Rust test regenerates already committed fixtures.
- Every command declared in `.factory/claims.json` passed individually:
  eight browser claims in desktop and 390 px projects, five native sandbox /
  corpus / output / input-limit / worker-limit claims, and the JSON-proof
  claim. The corpus printed **22/22 (100.0%)** risky fixtures detected:
  covered text 4/4, invisible text 2/2, metadata 3/3, annotations 2/2,
  attachments 3/3, actions 4/4, forms 2/2, and layers 2/2.
- `npm run build` produced `dist/app` and `dist/site`. App JavaScript is
  29.37 KB raw / 8.70 KB gzip. Site JavaScript is 4.53 KB raw / 1.96 KB gzip;
  site CSS is 12.09 KB raw / 3.43 KB gzip.
- `verify-url.sh` against HTTPS returned 200 in 861 ms with no console
  errors, the correct title, `lang=en`, exactly one `h1`, one `main`, and no
  missing image alt text or unnamed buttons. The built and live `index.html`
  SHA-256 are both
  `8d01f9cd48b9924a0ff531863b5aa5acb5c63ba7367d5c44568b2fc35ba49188`.
- Fresh desktop (1440×900) and phone (390×844) contexts both showed, before
  scrolling: job “Check a redacted PDF before you send it.”, audience “For
  people sharing PDFs who need to find hidden text and document details before
  disclosure.”, and action “Try it with sample data”. The click populated the
  sample audit, kept its label visible while scrolling, Reset demo reported
  `Sample audit reset.`, and produced no console errors. Their only origins
  were the product and the documented GitHub release API.
- `/privacy/`, `/terms/`, and `/404/` have correct titles and headings;
  `/does-not-exist` returned HTTP 404 with the designed recovery page.
- The disabled Pro control made no request to `api.sociobot.in` in a fresh
  live browser context.
- Release `v0.1.6` contains universal macOS DMG/app archive, Windows MSI/EXE,
  and Linux AppImage/DEB/RPM plus valid `SHA256SUMS` and `latest.json`. The
  manifest names one installer for macOS, Windows, and Linux. A newly
  downloaded DEB reported `redaction-proof` 0.1.6 for amd64 and its SHA-256
  `03bac5e7979f9dc213b5057efd9af72d081c0ca00deb59f44ceca65539a2422e`
  exactly matched `SHA256SUMS`. Extracted to a clean temporary directory, the
  GUI remained running for eight seconds under a virtual display.
- Lighthouse 12.8.2 recorded Performance 100, Accessibility 100, Best
  Practices 100, SEO 100, LCP 1,047 ms, CLS 0, and TBT 0. Chromium crashed
  while Lighthouse collected its final full-page screenshot after recording
  those values. Repeated Playwright browser runs did not crash.

Evidence is in `/work/.evidence/pdf-redaction-proof-repair-3/`; the compliant
catalog description is copied to `/work/.evidence/catalog-description.txt`.

## Known limits and operator action

- The billing-registration operator must register `pdf-redaction-proof` before
  Pro purchase and entitlement can be exercised end to end. The product does
  not fake this flow or make batch work free.
- The geometry analysis is strongest for normal axis-aligned text and opaque
  rectangles. Rotated text, clipping, Form XObjects, and image-baked secrets
  still require visual review; this is shown in the app and report.
- Desktop artifacts are unsigned. Signing needs the operator-managed macOS and
  Windows certificate secrets; none are stored in this repository.
- There is intentionally no automatic updater. The site reads GitHub's latest
  release metadata and falls back calmly if it is unavailable.
