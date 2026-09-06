# Verify redacted PDFs before sharing — independent QA 3

**Verdict: FAIL**

- Findings: **2** — 1 high, 1 medium
- Untested public claims: **2**
- Implementation candidate: `967c5c62c599937a80bd2f5254844356e34c6407`
- Documentation candidate: `a9e477058ff20f45ee9d817b3ce2b4beedd56501`
- Release: `v0.1.6`
- Live URL: https://pdf-redaction-proof.sociobot.in/
- Verified: 2026-09-06

The working product paths passed. This review still fails because two public
security and installation claims do not have complete claim tests. Passing all
declared commands does not remove missing or incomplete claim coverage.

## First screen

Fresh desktop and phone browsers showed these items before scrolling:

- Job: “Check a redacted PDF before you send it.”
- Audience: people sharing PDFs who need to find hidden text and document
  details before disclosure.
- First action: “Try it with sample data”.

The words are plain, the title names the job, and the first action is clear.

## Findings

### H-1 — The local parser privacy claim is not tested on every shipped platform

The landing page and privacy policy state that the PDF parser has no file or
network access. Claim `local-processing` states that the sandbox cannot open
files or network sockets.

Its declared command runs
`claim_document_privacy_sandbox_denies_filesystem_and_network_syscalls`. That
test is compiled only under `#[cfg(all(test, target_os = "linux"))]` in
`src-tauri/src/sandbox.rs`. It directly proves denial on Linux only. On macOS
and Windows the release suite runs a worker test that proves parsing succeeds
after sandbox setup, but it never tries to open a file or network socket and
asserts no denial.

The `v0.1.6` workflow passed on Linux, macOS, and Windows, but those successful
jobs do not prove the full cross-platform privacy promise. Add platform tests
that attempt and fail file and network access on macOS and Windows, or narrow
the public and claim text to the behavior actually proved.

### M-1 — Installer checksum verification is a public claim with no claim entry

The README says both one-line installers fetch `SHA256SUMS` and verify the
selected asset before installing or opening it. The live download control also
says “Checksums published.” This is a reliance-worthy supply-chain claim.

No entry in `.factory/claims.json` tests it. Claim `release-downloads` mocks the
GitHub release API and checks the selected installer URL and cache. It does not
run either installer, fetch `SHA256SUMS`, accept a valid asset, or reject a
modified asset.

The current Linux DEB was checked manually and its checksum matches. That
proves the current artifact, not the public promise on every build. Add a
tagged installer-integrity claim that exercises both scripts with valid and
corrupt fixtures.

## Declared claim commands

All 14 declared commands passed from the fresh checkout after installing the
README-documented Linux prerequisites. The combined command output is
`/work/.evidence/pdf-redaction-proof-verification-3/claim-commands.log`.

| Claim | Result | Evidence |
| --- | --- | --- |
| `release-downloads` | PASS, 2 browser projects | Installer URL and one cache reuse proved; M-1 remains outside this claim. |
| `release-cache-period` | PASS, 2 browser projects | Cache used at 3,590 seconds and refreshed at 3,600 seconds. |
| `site-network-privacy` | PASS, 2 browser projects | Only the preview origin and mocked GitHub API were requested. |
| `demo-isolation` | PASS, 2 browser projects | Sample reset and empty `demo:` namespace. |
| `sample-findings` | PASS, 2 browser projects | Covered text, author metadata, and two hidden items shown. |
| `single-file-price` | PASS, 2 browser projects | Free scope, US$12 offer, disabled purchase, and no checkout request. |
| `local-processing` | PASS on Linux | File and socket denial proved on Linux; cross-platform gap is H-1. |
| `detection-corpus` | PASS | 22/22 risky fixtures detected, 100.0% across eight categories. |
| `sanitized-copy` | PASS | Original bytes retained, separate copy cleaned, and copy checked again. |
| `json-proof` | PASS | Both SHA-256 identities retained and local paths removed. |
| `input-limit` | PASS | 500 MB plus one byte rejected before content read. |
| `linux-worker-resource-limits` | PASS | 60 CPU seconds and 1,610,612,736 address-space bytes observed. |
| `offline-sample` | PASS, 2 browser projects | Built-in sample loaded with no external request. |
| `license-token` | PASS, 2 browser projects | GET sent only the token; upstream 429 and `Retry-After: 42` handled. |

## Clean checkout and build

The fresh checkout was `/tmp/pdf-redaction-proof-verify3` at documentation SHA
`a9e4770`. `npm ci` installed 66 packages and reported zero vulnerabilities.
The documented WebKitGTK, AppIndicator, SVG, and `patchelf` prerequisites were
installed before native results were measured.

- `npm test` passed: 12 Vitest tests, 7 Rust unit tests, 3 Rust worker tests,
  40 Playwright tests, and 2 expected phone-layout skips.
- `npm run check`, `cargo fmt -- --check`, `npm audit --audit-level=high`,
  and `git diff --check` passed.
- `npm run build` created `dist/app` and `dist/site`.
- App JavaScript is 29.37 KB raw and 8.70 KB gzip.
- Site JavaScript is 4.53 KB raw and 1.96 KB gzip. Site CSS is 12.09 KB raw
  and 3.43 KB gzip.
- The clean checkout remained unchanged.

## Live demo, privacy, and routes

Fresh 1440×900 and 390×844 contexts had no unexpected console, page, or
request errors. Their first screens, demo states, and populated sample results
are in `/work/.evidence/pdf-redaction-proof-verification-3/live-*.png`.

One click opened a realistic `sample-board-minutes.pdf` result with 12 pages,
842 KB, a failed audit, covered text, and author metadata. The result began in
the first viewport on desktop and phone. The persistent label remained visible
after scrolling to the bottom. Reset reported “Sample audit reset.” A seeded
real-data sentinel stayed unchanged, and no `demo:` storage key was created.

The only browser origins during the full flow were the product and the public
GitHub release API. The disabled Pro control made no billing request. The site
does not expose an active broken checkout.

`/`, `/?demo=1`, `/privacy/`, `/terms/`, and `/404/` returned their expected
pages. An unknown URL returned the designed page with HTTP 404 and a working
route home. Chromium logs the expected failed-document message for that
deliberate 404; it is not a defect. Route titles, one `h1`, `lang=en`, one
`main`, header, navigation, footer, heading order, image alternatives, named
controls, and horizontal reflow all passed. Every crawled internal, source,
release, and platform-download link responded successfully.

The clean build and live index SHA-256 are both
`8d01f9cd48b9924a0ff531863b5aa5acb5c63ba7367d5c44568b2fc35ba49188`.
All three referenced CSS and JavaScript files also match byte for byte.

## Accessibility, keyboard, and recovery

Playwright axe found zero violations on every main route, the direct demo,
phone, and dark reduced-motion contexts. The URL verifier returned HTTP 200 in
729 ms with no console errors, correct title and language, one heading and
main landmark, image alternatives, and named buttons.

The skip link is the first keyboard target, shows a 3 px solid focus outline,
and moves focus to `main`. The tested phone controls meet 44×44 CSS px. A
640 px reflow check had no horizontal overflow. Reduced motion produced no
animations or transitions. Invalid license, sixth-attempt local limiting,
upstream 429 recovery, release API failure, fresh-cache recovery, offline
sample use, reset, invalid PDF, and the 500 MB boundary are covered by passing
tests.

Lighthouse 12.8.2 scored Performance 100, Accessibility 100, Best Practices
100, and SEO 100. LCP was 1,172 ms, CLS 0, total blocking time 60 ms, and total
transfer 43,493 bytes.

## Installed release

Release workflow `34019060777` completed successfully at implementation SHA
`967c5c6`. Linux, macOS, Windows, and manifest jobs all passed their test and
build steps. Release `v0.1.6` includes DMG/app archive, MSI/EXE, AppImage, DEB,
RPM, `SHA256SUMS`, and `latest.json`.

The downloaded Linux DEB is version 0.1.6 for amd64. Its SHA-256 is
`03bac5e7979f9dc213b5057efd9af72d081c0ca00deb59f44ceca65539a2422e`,
which exactly matches `SHA256SUMS`. `latest.json` names one installer and hash
for macOS, Windows, and Linux.

The DEB was extracted into a clean consumer directory with isolated config,
data, and cache paths. Its GUI launched under a virtual display. The initial
screen had no false progress state. Keyboard use loaded the bundled sample and
showed its populated failed audit. The installed artifact also opened
`covered-text-tj.pdf` through its native file picker and reported recoverable
content. The source hash remained
`9b9ef4aaf0c69c711ec00e889467d6d98127c9860cf607a54da160da3bcec01f`.
Native sanitizing and proof output passed the worker and corpus tests.

## Earlier finding disposition

| Earlier finding | Current disposition |
| --- | --- |
| Verification 2 H-1, broken purchase | Resolved. Purchase is plainly unavailable, the free scope remains, and no checkout request occurs. |
| Verification 2 H-2, four incomplete claims | The four named gaps were removed or covered: network privacy, Linux limits, and the one-hour boundary now have tagged tests; the JavaScript sentence was removed. New H-1 and M-1 remain. |
| Verification 2 M-1, false loading state | Resolved in browser tests and the installed app. |
| Verification 2 M-2, phone result and banner | Resolved. Result starts in view and the label remains visible at page bottom. |
| Verification 2 L-1, narrow phone targets | Resolved. The named links and current phone action set meet 44×44 px. |
| Verification 2 L-2, unknown URL returned 200 | Resolved. Unknown URLs return the designed HTTP 404. |
| Verification 2 L-3, metaphor headings | Resolved. Headings say “What the check finds.” and “Page not found.” |
| Verification 1 H-1, license burst control | Resolved in the app: five requests allowed, the sixth blocked, and upstream 429 delay honored. |
| Verification 1 H-3, parser isolation | Linux denial and cross-platform worker startup pass. Full macOS and Windows denial evidence is still missing under current H-1. |
| Verification 1 M-1, narrow corpus | Resolved with 24 varied fixtures, 22 risky cases, 2 clean controls, and 100% detection. |
| Verification 1 L-1, wordmark target | Resolved at 44 px or larger. |

## Scope notes

This is a static site and desktop app, not a product backend. Tenant isolation,
server restart persistence, health endpoints, and backend request limits do not
apply. The app-side license limiter and upstream 429 recovery do apply and
passed. There is intentionally no automatic updater. The deterministic PDF
security job does not benefit from adding an AI step, so missed AI leverage is
not a finding.

The external billing-registration operator still needs to register the Pro
offer. The current honest unavailable state is not a product defect.

## Evidence

- Declared claims: `/work/.evidence/pdf-redaction-proof-verification-3/claim-commands.log`
- Live browser details: `/work/.evidence/pdf-redaction-proof-verification-3/live-browser-qa.json`
- URL verifier: `/work/.evidence/pdf-redaction-proof-verification-3/verify.json`
- Lighthouse: `/work/.evidence/pdf-redaction-proof-verification-3/lighthouse.json`
- Release and checksums: `/work/.evidence/pdf-redaction-proof-verification-3/release.json`, `latest.json`, and `SHA256SUMS`
- Installed app: `/work/.evidence/pdf-redaction-proof-verification-3/native-initial.png`, `native-after-sample.png`, and `native-real-result.png`

**Final verdict: FAIL — 2 findings and 2 untested public claims remain.**
