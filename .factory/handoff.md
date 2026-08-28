# Redaction Proof repair — handoff

## What changed

- Reproduced the failed browser path from candidate `290ab6a`: the release
  manifest request redirects from `github.com` without an
  `Access-Control-Allow-Origin` response header.
- Replaced that request with GitHub's CORS-enabled latest-release API:
  `https://api.github.com/repos/B-Divyesh/sf-pdf-redaction-proof/releases/latest`.
- Selects the published DMG, MSI, and AppImage from API asset data. Successful
  metadata is cached in local storage for one hour.
- A missing release, incomplete asset set, rate limit, offline request, invalid
  response, or disabled storage now shows “Downloads are being published” and
  links to GitHub Releases. Every promise and response branch is handled.
- Preserved the Tauri 2 desktop app, checksum installers, GitHub Actions matrix,
  GitHub Release assets, `SHA256SUMS`, and `latest.json` generation.
- Added the isolated `?demo=1` sample, persistent reset/exit banner, and a
  first-run **Load sample project** action in the desktop UI. Demo state is
  memory-only and never uses the real-data namespace.
- Reworked the first screen in plain words, added the standard product-preview
  and site section order, route metadata, social image, icons, designed 404,
  security configuration, and consistent route chrome.
- Added `.factory/claims.json`, `.factory/demo.md`, and the sentence-by-sentence
  `.factory/copy-audit.md`.

## Regression coverage

The focused tests now prove:

- no browser request uses `releases/latest/download/latest.json`;
- current API assets resolve to platform download links;
- a successful result is reused from the one-hour cache;
- fresh cached links remain available when the API is offline;
- API failure renders the calm publishing state without a page error;
- desktop and 390 px mobile layouts pass axe serious/critical checks;
- the keyboard skip link moves focus to `<main>`;
- demo reset and storage isolation work from `?demo=1`;
- every claim tag in `.factory/claims.json` passes.

## Verification evidence

- Clean dependency install: `npm ci` — 67 packages, 0 vulnerabilities.
- Original static deploy build command: `npm run build:site` — passed and wrote
  `dist/site/index.html`.
- Full artifact build: `npm run build` — passed; wrote `dist/app` and
  `dist/site`.
- Static output: site JS 4.53 KB raw / 1.96 KB gzip; site CSS 11.52 KB raw /
  3.32 KB gzip; mobile hero WebP 16.9 KB; desktop hero WebP 44.1 KB.
- `npm run check` — TypeScript and Rust checks passed.
- `npm test` — 9 Vitest assertions, 4 Rust tests, and 22 Playwright tests
  passed. Playwright covered desktop Chromium and a 390 px mobile profile.
- `npx playwright test --grep '@claim:'` — 10/10 project executions passed.
- `npm audit --audit-level=high` — 0 vulnerabilities.
- Local browser identity check — one `<h1>`, exact title, no horizontal
  overflow, no console/page errors, and a real v0.1.0 AppImage link on desktop
  and mobile demo routes.
- Lighthouse mobile production preview — Performance 100, Accessibility 100,
  Best Practices 100, SEO 100, LCP 1.1 s, CLS 0, TBT 0 ms.
- GitHub API response — `Access-Control-Allow-Origin: *`; release `v0.1.0`
  includes DMG, MSI, AppImage, DEB, RPM, EXE, `SHA256SUMS`, and `latest.json`.
- Clean public DEB download checksum matched `SHA256SUMS`:
  `b32407fb180fc4d5aeaca313dada3724cd986888d338aacbf7ca80aac0e39ee2`.

## Deployment

- Configuration: `npm run build:site`, deploy `dist/site` as the existing
  static artifact at `https://pdf-redaction-proof.sociobot.in`.
- Repair commit `b088443` was pushed to `origin/main` before deployment.
- `/opt/fleet/lib/deploy-static.sh pdf-redaction-proof /work/repo/dist/site`
  completed successfully. Azure deployment ID:
  `62d9b30c-7dd7-4344-83c8-eb1f09da07f9`.
- Factory `verify-url.sh` returned HTTPS 200 in 897 ms with zero errors, title
  and `lang` present, one `<h1>`, one `<main>`, and no missing image alt text or
  unnamed buttons.
- A fresh live Chromium context requested only the CORS-enabled GitHub API,
  resolved the v0.1.0 AppImage, and made zero old-manifest requests. `/`,
  `/?demo=1`, `/privacy/`, `/terms/`, and `/404/` had their expected titles,
  one `<h1>`, no overflow, no console errors, and no failed requests.
- Live response headers include CSP limited to the same origin, GitHub API, and
  Sociobot checkout, plus HSTS, `nosniff`, referrer, and permissions policies.

## Known limits

- PDF geometry matching remains strongest for normal axis-aligned text,
  opaque rectangles, and redaction annotations. Rotated text, unusual clipping,
  Form XObjects, and secrets inside images require visual review.
- Releases are unsigned until the operator supplies Apple and Windows signing
  credentials. This remains stated beside the installation instructions.
- Browser storage cannot activate the desktop app directly. The purchase return
  still exposes the token for copy, and the app accepts a pasted token.

## Needs operator action

- Optional signing/notarization needs `APPLE_CERTIFICATE`,
  `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_ID`,
  `APPLE_PASSWORD`, `APPLE_TEAM_ID`, `WINDOWS_CERT_PFX`, and
  `WINDOWS_CERT_PASSWORD`.
- A future parser release can add transformed text, Form XObject, clipping, and
  image/OCR fixtures without changing this deployment class.
