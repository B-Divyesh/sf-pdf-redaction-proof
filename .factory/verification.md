# Independent verification — FAIL

**Candidate:** `d618cda69433ecb2ef3a0b594e7ec1d52889de95`<br>
**Verified URL:** https://pdf-redaction-proof.sociobot.in/<br>
**Date:** 2026-08-28<br>
**Disposition:** **FAIL — release-blocking findings remain.**

## Cold first read

Fresh Chromium opened the live URL with no prior storage. The first screen says
“Check a redacted PDF before you send it.” It identifies people sharing PDFs
as the audience and says it finds hidden text and document details. The first
primary action is **Try it with sample data**, explained as a built-in check
that saves nothing. It opens `?demo=1` in one click. This gate passes.

## Required claims gate

`npm ci` completed from this clean checkout (67 packages; audit reported 0
vulnerabilities). Every command recorded in `.factory/claims.json` was run
against its shipped demo entry point and passed in both configured Playwright
projects (desktop and 390 px mobile):

| Claim | Command | Result / evidence |
| --- | --- | --- |
| release-downloads | `npx playwright test --grep @claim:release-downloads --workers=1` | 2 passed; mocked GitHub release selects a platform installer and caches it. |
| demo-isolation | `npx playwright test --grep @claim:demo-isolation --workers=1` | 2 passed; `?demo=1`, reset, and empty `demo:` namespace confirmed. |
| sample-findings | `npx playwright test --grep @claim:sample-findings --workers=1` | 2 passed; covered text and author metadata shown. |
| single-file-price | `npx playwright test --grep @claim:single-file-price --workers=1` | 2 passed; free single-file limit, US$12 price, and checkout URL asserted. |
| local-processing | `npx playwright test --grep @claim:local-processing --workers=1` | 2 passed; built-in desktop sample made no external request. |

The listed tests pass. The claims inventory is incomplete; see H-2.

## Local quality gates

- `npm run check` — passed: TypeScript and Rust checks.
- `npm run build` — passed; created `dist/app` and `dist/site`.
- `npm test` — passed: 9 Vitest assertions, 4 Rust tests, and 22 Playwright
  executions.
- Production output: app JS 28.10 KB raw / 8.25 KB gzip; site JS 4.53 KB raw /
  1.96 KB gzip; site CSS 11.52 KB raw / 3.32 KB gzip.

## Live deployment and product exercise

- The local production `index.html` and all three referenced CSS/JS assets
  match live byte-for-byte by SHA-256. The live site is this candidate build.
- Fresh-context route exercise at desktop 1440×900 and mobile 390×844 covered
  `/`, `/?demo=1`, `/privacy/`, `/terms/`, and `/404/`. Each returned 200, had
  one `h1`, no horizontal overflow, no console/page/request errors, and no axe
  serious or critical finding. The demo banner, sample result, reset feedback,
  empty `demo:` storage, keyboard skip link, and reduced-motion preference were
  exercised.
- The only live outbound browser request was the documented CORS-enabled
  GitHub releases API. No analytics, third-party fonts, or document request was
  observed. The demo only obtains public release metadata.
- Live headers include HSTS, `nosniff`, referrer and permissions policies, and
  a restrictive CSP. Hashed assets use immutable one-year caching.
- Live mobile Lighthouse returned performance 1.00, accessibility 1.00, best
  practices 1.00, SEO 1.00, LCP 1137 ms, CLS 0. Lighthouse reported
  `TARGET_CRASHED` while collecting its final full-page screenshot after writing
  those results; Playwright did not reproduce a page crash.
- GitHub release `v0.1.0` includes DMG, MSI, EXE, AppImage, DEB, RPM,
  `SHA256SUMS`, and `latest.json`. A clean DEB download SHA-256
  `b32407fb180fc4d5aeaca313dada3724cd986888d338aacbf7ca80aac0e39ee2`
  matches `SHA256SUMS`; `dpkg-deb -I` reports version 0.1.0 with expected
  WebKit/GTK dependencies.

## Findings

### H-1 — Required product-unlock rate limit was not observed

`app/src/license.ts:27-37` calls
`GET https://api.sociobot.in/api/v1/products/pdf-redaction-proof/verify?...`.
The acceptance contract requires server-side product endpoints, including
product-unlock calls, to yield 429 plus `Retry-After` under a rapid request
burst.

On 2026-08-28, 30 invalid-token verification requests at 10-way concurrency
returned **30 × HTTP 200**, each `{"valid":false,"reason":"invalid"}`. No 429
and no `Retry-After` were observed. The threshold was **not observed through
30 requests**. This is release-blocking.

### H-2 — Reliance-worthy visitor claims are absent from the claim inventory

`.factory/claims.json` lists five claims, but the live landing page makes
additional claims without corresponding entries/tests: “PDF checks run on your
computer” (`site/index.html:57`), “The original file stays unchanged” (line
79), “Sanitized copy created” and “The JSON report identifies both files”
(line 81), the four detection claims (lines 88-91), and “The app sends no
document contents, filenames, hashes, or reports” (line 100).

The `local-processing` test only covers a built-in sample in a browser
development context. It does not prove the broad local-processing,
sanitization, export, or privacy claims. The claims contract makes unlisted
claim-like copy release-blocking until removed or covered by an observable demo
test.

### H-3 — PDF-worker isolation does not establish the required sandbox

The app uses a separate worker and Unix CPU/address-space limits
(`src-tauri/src/lib.rs:13-42`). It launches the same executable under the
desktop user’s identity with a user-selected path (`src-tauri/src/lib.rs:14-17,
52-60`). No filesystem, network, privilege, or syscall isolation is configured;
the parser reads the entire input in that worker (`src-tauri/src/pdf.rs:360-381`).

This is resource limiting, not a demonstrated sandbox for hostile PDFs as
required by the brief. Use an OS sandbox/least-privilege worker with explicit
file handles and deny-by-default filesystem/network access.

### M-1 — The stated 95% success measure is not demonstrated

The corpus-style test has 20 coordinate changes of one axis-aligned
text-plus-rectangle construction and accepts 19 detections. Metadata is covered
by one separate seed. This does not demonstrate 95% detection across a varied
hidden-text and metadata regression corpus. Add committed fixtures and a test
reporting detected/total for hidden text, metadata, annotations, attachments,
actions, and layers.

### L-1 — Wordmark hit area is below 44 px

At both live test viewports, the visible home wordmark link measured 32 px high.
Increase its hit area to at least 44×44 CSS px.

## What passed despite the fail disposition

The prior deployment-only release-download path is healthy: the live site uses
`api.github.com`, not GitHub’s non-CORS latest-download redirect. The one-click
demo, all five listed claim tests, build/test/check gates, release checksum,
privacy-oriented outbound behavior, accessibility smoke checks, bundle budgets,
and live candidate identity passed.

## Required next actions

1. Apply a product-unlock rate limit that returns 429 and `Retry-After`; record
   the observed threshold.
2. Add observable demo tests for every local-processing, sanitization, export,
   detection, and privacy claim, or remove unsupported copy.
3. Use an actual least-privilege PDF parser sandbox and a varied detection
   corpus proving the stated target.
4. Correct the wordmark hit area, rerun all gates, and reverify live.
