# Verify redacted PDFs before sharing — independent QA 2

**Verdict: FAIL**

- Findings: **7** — 2 high, 2 medium, 3 low
- Untested public claims: **4**
- Implementation candidate: `7d3a118b91322b140642cd021b1504b86e1ebe2b`
- Documentation candidate: `3c39ce86e7a03b1749530380cf1e89f5a073fcdc`
- Release: `v0.1.3`
- Live URL: https://pdf-redaction-proof.sociobot.in/
- Verified: 2026-09-05

The product does not pass this verification. Passing test commands do not
override the broken purchase path, untested public claims, or the other user
experience findings below.

## First screen

In fresh desktop and phone browsers, before scrolling:

- Job: “Check a redacted PDF before you send it.”
- Audience: people sharing PDFs who need to find hidden text and document
  details before disclosure.
- First action: “Try it with sample data.”

These are plain, visible, and understandable on the first screen.

## Findings

### H-1 — The live purchase action is broken

The live “Buy Pro for $12” action points to the documented Sociobot checkout
URL, but that URL returned HTTP 404 on 2026-09-05. A visitor cannot complete
the advertised one-time purchase.

The `single-file-price` claim test only checks the displayed price, scope, and
checkout URL. It does not request the URL or prove that checkout works. The
test command passes while the live user path fails.

### H-2 — Four public claims have no complete claim test

The claim inventory is still incomplete. These reliance-worthy statements are
public but lack a claim entry and observable test that proves the full promise:

1. The combined privacy promise that no PDF content is uploaded and that the
   app has no telemetry, third-party runtime script, or CDN.
2. “The app does not ... run PDF JavaScript.”
3. The Unix worker limits of 60 CPU seconds and 1.5 GB of address space.
4. Successful GitHub release details are cached for one hour. The current test
   reloads immediately; it does not exercise the one-hour boundary.

The parser sandbox and offline sample tests prove narrower behavior. They do
not make these four broader or quantitative claims tested under the claims
contract.

### M-1 — The installed app shows a false loading state

A clean launch shows “Inspecting PDF structure…” before a file or sample is
chosen. It remains visible beside completed sample and real-file results.

`app/index.html:38` correctly adds `hidden`, but
`app/src/style.css:2` sets `.progress-panel { display: flex }` without the
site stylesheet's `[hidden] { display: none !important }` safeguard. Browser
inspection confirmed `hidden=""` and `visible: true` before any action.

Evidence: `/work/.evidence/verification-2/native-initial.png` and
`/work/.evidence/verification-2/native-after-sample-middle.png`.

### M-2 — The phone demo does not meet the demo presentation contract

At the fresh iPhone 13 viewport, one click enters the sample but the populated
result begins at CSS y=901.77 while the viewport is 664 px high. The first
screen after clicking does not already show the product result.

Below 520 px, the demo banner becomes static. After scrolling to the bottom it
is no longer in the viewport, so the required persistent “Demo — sample data,
nothing is saved” label is lost.

Evidence: `/work/.evidence/verification-2/live-phone-demo.png` and
`/work/.evidence/verification-2/live-browser-qa.json`.

### L-1 — Four phone link targets are narrower than 44 px

At 390 px, standalone links measured: Demo 42×44, Linux 38×44, a second Demo
42×44, and Terms 41×44 CSS px. Their heights pass, but their widths do not meet
the attached 44×44 px accessibility baseline.

### L-2 — Unknown URLs return HTTP 200 instead of HTTP 404

`/not-a-real-redaction-proof-page` renders the designed not-found page and a
working Return home link, but returns HTTP 200. A deliberate 404 is expected
for an unknown route. The `navigationFallback` rewrite takes precedence over
the configured 404 response override.

The direct `/404/` page is designed and usable; its intentional page is not
the defect.

### L-3 — Two headings use metaphor instead of naming the section

The landing-page heading “Look past the black box.” and the not-found heading
“This page is not in the file.” violate the attached plain-words contract.
They should state what the section or error is without metaphor.

## One-click sample and data separation

The desktop demo works in one click and shows a realistic, populated result:
`sample-board-minutes.pdf`, 12 pages, 842 KB, “Recoverable content found,”
covered text, and author metadata. Reset restores the sample and reports the
reset. “Start for real” leaves demo mode.

The flow created no `demo:` storage records and did not alter any existing
product storage. The only stored value was the documented public GitHub
release cache. The browser made requests only to the product origin and the
documented public GitHub API. No sample or real PDF was uploaded.

The desktop result is visible immediately. The phone presentation fails as
described in M-2.

## Declared claim commands

Every command in `.factory/claims.json` was run from the clean checkout. All
commands passed as written.

| Claim | Exact command | Result |
| --- | --- | --- |
| `release-downloads` | `npx playwright test --grep @claim:release-downloads --workers=1` | 2 passed |
| `demo-isolation` | `npx playwright test --grep @claim:demo-isolation --workers=1` | 2 passed |
| `sample-findings` | `npx playwright test --grep @claim:sample-findings --workers=1` | 2 passed |
| `single-file-price` | `npx playwright test --grep @claim:single-file-price --workers=1` | 2 passed, but the live checkout returned 404; H-1 |
| `local-processing` | `cargo test --manifest-path src-tauri/Cargo.toml claim_document_privacy_sandbox -- --nocapture` | passed |
| `detection-corpus` | `cargo test --manifest-path src-tauri/Cargo.toml claim_core_detection_corpus -- --nocapture` | passed: 22/22 risky fixtures detected |
| `sanitized-copy` | `cargo test --manifest-path src-tauri/Cargo.toml claim_sanitized_copy -- --nocapture` | passed |
| `json-proof` | `npx vitest run --testNamePattern @claim:json-proof` | passed |
| `input-limit` | `cargo test --manifest-path src-tauri/Cargo.toml claim_input_limit -- --nocapture` | passed |
| `offline-sample` | `npx playwright test --grep @claim:offline-sample --workers=1` | 2 passed |

Corpus output reported 100%: covered text 4/4, invisible text 2/2, metadata
3/3, annotations 2/2, attachments 3/3, actions 4/4, forms 2/2, and layers
2/2. Two clean controls were also present.

## Clean checkout gates

The clean checkout was `/tmp/pdf-redaction-proof-qa.T6VZPC`. `npm ci`
installed 67 packages and reported no audit vulnerabilities.

The first `npm run check` could not compile Tauri because this clean worker did
not yet have the README-documented WebKit/GTK system packages. After installing
those documented prerequisites, all declared gates ran normally:

- `npm run check` — passed.
- `npm run build` — passed and created `dist/app` and `dist/site`.
- `npm test` — passed: 12 Vitest tests, 7 Rust unit tests, 2 Rust worker
  integration tests, and 32 Playwright executions. One fixture-regeneration
  helper remains intentionally ignored.
- `cargo fmt -- --check` — passed.
- `git diff --check` — passed.
- `npm audit --audit-level=high` — passed with zero vulnerabilities.

Built sizes stayed within budget: app JS 29.52 KB raw / 8.74 KB gzip, app CSS
11.09 KB raw / 3.29 KB gzip, site JS 4.53 KB raw / 1.96 KB gzip, and site CSS
11.54 KB raw / 3.32 KB gzip.

## Live browser, accessibility, and recovery checks

- The live HTML, CSS, and JS hashes exactly match the clean build from the
  documentation candidate.
- `/`, `/?demo=1`, `/privacy/`, `/terms/`, and `/404/` have route titles,
  `lang=en`, one `h1`, a `main`, standard landmarks, ordered headings, image
  alternatives, and no horizontal overflow.
- Axe reported zero violations in fresh desktop, dark/reduced-motion, and
  phone-demo contexts.
- Keyboard traversal reaches the skip link and all actions without a trap.
  The skip link moves focus to `main`, and the focused link has a visible 3 px
  ring. Route focus and polite status regions are present.
- Reduced motion removes animation. Dark mode, 200% text zoom, invalid license,
  reset, offline sample, GitHub API failure, and cached release recovery were
  exercised without browser console errors.
- Security headers include CSP, HSTS, `nosniff`, referrer policy, and
  permissions policy. Hashed assets have immutable caching.
- Privacy, terms, install scripts, robots, sitemap, source, and release links
  responded successfully. The purchase route is the exception in H-1.
- Live mobile Lighthouse 12.8.2 scored 100 for performance, accessibility,
  best practices, and SEO. LCP was 970.2 ms, CLS 0, and TBT 0.

`/opt/fleet/lib/verify-url.sh` also passed title, language, main landmark,
image alternative, button-name, load-time, and console checks.

## Installed release exercise

GitHub Actions run `33177881600` succeeded for Linux, macOS, Windows, and the
release manifest. Release `v0.1.3` includes DMG, MSI, EXE, AppImage, DEB, RPM,
the macOS archive, `latest.json`, and `SHA256SUMS`.

The Linux DEB was downloaded into a clean consumer environment. Its SHA-256,
`b7fe67ccf93403b070fc7f61bb33c2e9023d3e3c4e1bf79b62d4d26964706852`,
matches `SHA256SUMS`; package metadata reports version 0.1.3 and amd64. The
documented shell installer also downloaded the AppImage and verified its hash.
The container lacks FUSE, so the AppImage was exercised with its supported
extract-and-run mode; the DEB ran normally under a fresh OS user.

The installed app loaded its sample and displayed the expected two findings.
It also inspected a real risky fixture through the native file picker, found
surviving covered text, created a separate sanitized copy, and rechecked that
copy as a pass. The source file's SHA-256 before and after remained
`9b9ef4d65767b36a9dd28f4265911139f886d28b7c128e2a210a6081266ec01f`;
the separate output had a different hash. The false loading state in M-1 was
visible throughout these checks.

## Earlier finding disposition

| Earlier finding | Current disposition |
| --- | --- |
| H-1, no license burst control | Product-side repair verified: five forced checks per rolling minute; sixth blocked; upstream 429 and `Retry-After` honored. The shared gateway limitation remains below. |
| H-2, incomplete claims | **Not fully resolved.** Ten declared commands pass, but four public claims remain unlisted or incompletely tested; current H-2. |
| H-3, parser isolation | Resolved. Linux Landlock/seccomp denial checks and real worker integration pass; native CI passed on all release platforms. |
| M-1, narrow detection corpus | Resolved. 24 fixtures, 22 risky cases, two clean controls, and all required categories were exercised at 100%. |
| L-1, 32 px wordmark target | Resolved. The live wordmark measures 197.59×44 CSS px on desktop and phone. Current L-1 concerns different links. |

## Shared billing limitation

The app-side rate limit and upstream 429 handling are implemented and tested.
A fresh 30-request burst to the shared verification endpoint still returned
30 HTTP 200 responses and no `Retry-After`. This is recorded as an external
gateway limitation rather than a new product finding because this repository
cannot change that shared service and the app independently limits requests.

The broken checkout in H-1 is separate: it prevents the advertised purchase
from starting at all.

## Evidence

- Browser results: `/work/.evidence/verification-2/live-browser-qa.json`
- Lighthouse: `/work/.evidence/verification-2/lighthouse.json`
- URL verifier: `/work/.evidence/verification-2/verify.json`
- Release metadata and checksums: `/work/.evidence/verification-2/release.json`
  and `/work/.evidence/verification-2/SHA256SUMS`
- Screenshots: `/work/.evidence/verification-2/live-*.png` and
  `/work/.evidence/verification-2/native-*.png`

**Final verdict: FAIL — 7 findings and 4 untested public claims remain.**
