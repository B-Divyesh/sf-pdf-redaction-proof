# Redaction Proof v0.1.0 — handoff

## What was built

- Tauri 2 desktop app with a Rust PDF parser and vanilla TypeScript interface.
- Local inspection for covered and invisible text operators, document/XMP
  metadata, annotations, attachments/name trees, actions, forms, and optional
  content layers.
- Non-destructive sanitizing to a new `.sanitized.pdf`: removes identified
  covered/invisible text operations and risky document structures, reapplies
  redaction regions, hashes the output, and immediately re-audits it.
- Portable JSON proof with schema/app version, timestamp, source/output SHA-256,
  counts, per-page findings, verdict, and explicit limitations. Local paths are
  removed from exported reports.
- Free complete single-file workflow and a US$12 one-time Pro license for batch
  selection/results. Checkout, callback capture, paste-to-restore, daily
  verification caching, revocation handling, and offline optimistic unlock use
  the Sociobot contract.
- Responsive download site, OS detection, generated hero art, privacy and terms
  pages, checksum-verifying shell/PowerShell installers, and secure headers.
- GitHub Actions release pipeline for universal macOS DMG, Windows MSI/EXE,
  Linux AppImage/DEB, `SHA256SUMS`, and `latest.json`.

## Verification completed locally

- `npm test`: 6 TypeScript unit assertions, 4 Rust tests, and 6 Playwright
  checks pass. Playwright covers Chromium desktop and a 390 px mobile viewport;
  axe reports zero serious/critical issues on both the site and app empty state.
- Seeded hidden-text regression: 20/20 opaque-overlay cases detected (100%),
  plus invisible-text and metadata fixtures; sanitized fixtures re-audit pass.
- `npm run check`: TypeScript and Rust checks pass.
- `npm run build`: reproducibly emits `dist/app` and `dist/site`; the deploy
  command is `npm run build:site` and `dist/site/index.html` is present.
- Bundle sizes: app JS 26.98 KB / CSS 10.73 KB; site JS 2.69 KB / CSS 8.87 KB;
  responsive hero WebP 16.9 KB mobile and 44.1 KB desktop.
- Lighthouse mobile, local production preview: Performance 100,
  Accessibility 100, Best Practices 100, SEO 100; LCP 1.2 s, CLS 0,
  total blocking time 0 ms, and no console errors.
- `npm audit`: 0 vulnerabilities.
- GitHub Actions run
  [`33159767413`](https://github.com/B-Divyesh/sf-pdf-redaction-proof/actions/runs/33159767413)
  passed for universal macOS, Windows x64, and Linux x64. Release
  [`v0.1.0`](https://github.com/B-Divyesh/sf-pdf-redaction-proof/releases/tag/v0.1.0)
  contains DMG, MSI, AppImage, DEB, RPM, `SHA256SUMS`, and valid `latest.json`
  assets. A clean public download of `Redaction.Proof_0.1.0_amd64.deb` was
  verified successfully against the published checksum.

## Known limits

- Geometry matching is intentionally conservative and strongest for standard,
  axis-aligned text operators, opaque rectangle fills, and `/Redact`
  annotations. Rotated/transformed glyphs, clipped paths, Form XObjects, and
  rasterized secrets may need manual review; the app never claims otherwise.
- PDF parsing runs in a dedicated, non-executing worker process; Unix builds
  additionally enforce 60-second CPU and 1.5 GB address-space limits. v0.1 does
  not yet apply an OS-level syscall/filesystem policy, and Windows relies on
  process separation plus the 500 MB input bound.
- Generated releases are unsigned until operator certificates are configured.
- Browser storage cannot directly unlock a desktop app, so the post-checkout
  landing panel exposes a copy button and the app provides token paste/restore.

## Needs operator action

1. Register the `pdf-redaction-proof` product, US$12 one-time price, and return
   URL with the Sociobot billing API. The source intentionally contains no
   hard-coded provider product ID.
2. Configure deployment with `npm run build:site` and `dist/site`.
3. Optional signing/notarization requires operator-owned credentials. Use
   secrets named `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`,
   `APPLE_SIGNING_IDENTITY`, `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID`,
   `WINDOWS_CERT_PFX`, and `WINDOWS_CERT_PASSWORD`, then wire them into the
   release workflow; the current workflow deliberately does not reference
   absent secrets and produces unsigned artifacts.
4. For a stronger hostile-file boundary in v0.2, add per-platform syscall and
   filesystem policy around the existing worker, plus transformed text, Form
   XObject, clipping, and image/OCR fixtures.
