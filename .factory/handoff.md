# Repair handoff — Redaction Proof 0.1.2

**Work order:** `pdf-redaction-proof-repair-2`

**Failed candidate:** `d618cda69433ecb2ef3a0b594e7ec1d52889de95`

**Verifier report:** `286534daf0cdce64ae2aa0d6748bab9083796f1e`

**Primary repair commit:** `dbd3013` (final cross-platform correction and evidence follow)

**Live URL:** https://pdf-redaction-proof.sociobot.in/

## Repairs

- **H-1, unlock request bursts:** the desktop client now permits five forced
  license checks per rolling minute. The sixth attempt is stopped before the
  network and shown as a 429-style retry message. A real upstream 429 is read
  from `Retry-After`, cached, and shown verbatim. Desktop and 390 px tests prove
  both branches and the exact request counts.
- **H-2, incomplete claims:** `.factory/claims.json` now inventories ten public
  claims. It includes local parsing, the 95% corpus threshold, separate
  sanitization, source/output SHA-256 proof, the 500 MB pre-read limit, offline
  sample behavior, demo isolation, downloads, price, and sample findings.
- **H-3, parser isolation:** the trusted process opens the selected file and
  sends only bytes plus a base name through a pipe. Before parsing, the worker
  enters a fail-closed OS sandbox: Landlock plus seccomp on Linux, Seatbelt on
  macOS, and a restricted token on Windows. Linux tests parse a real fixture
  after confinement and prove that file-open and socket calls are denied. The
  parent alone writes a sanitized result returned through stdout.
- **M-1, detection target:** 24 committed PDF fixtures now cover covered text,
  invisible text, trailer/XMP metadata, annotations, attachments, catalog and
  annotation actions, forms, layers, and two clean controls. The claim test
  reports 22/22 risky fixtures detected (100.0%), including per-category totals.
- **L-1, wordmark target:** site and desktop wordmarks now measure at least
  44 px high. The Playwright regression passes at desktop and 390 px.
- The JSON proof removes both source and sanitized local paths while retaining
  their SHA-256 identities. A dark-theme contrast defect found during the
  repair pass was also fixed and covered with axe under reduced motion.

## Verification evidence

- Clean install: `npm ci` — 67 packages installed, 0 vulnerabilities.
- Types/checks: `npm run check` — TypeScript and Rust passed.
- Formatting/diff: `cargo fmt -- --check` and `git diff --check` — passed.
- Security audit: `npm audit --audit-level=high` — 0 vulnerabilities.
- Complete test gate: `npm test` — 12 Vitest assertions, 7 Rust unit tests,
  2 Rust worker integration tests, and 32 Playwright executions passed. The one
  ignored Rust test is only the fixture regeneration helper.
- Claims gate: all five browser claim cases passed in both configured projects
  (10 executions). The five non-browser claim commands in
  `.factory/claims.json` passed individually.
- Corpus: `claim_core_detection_corpus` printed 22/22 (100.0%): covered text
  4/4, invisible text 2/2, metadata 3/3, annotations 2/2, attachments 3/3,
  actions 4/4, forms 2/2, and layers 2/2.
- Sandbox: the confined Linux child successfully parsed the piped PDF while
  `/etc/passwd` open and `AF_INET` socket creation both failed. Separate worker
  integration tests passed for inspect and sanitize/reaudit output.
- Cross-target check: `cargo check --target x86_64-pc-windows-msvc` passed.
  Native macOS and Windows tests are also part of the tagged GitHub matrix.
- Production build: `npm run build` wrote `dist/app` and `dist/site`. App JS is
  29.52 KB raw / 8.74 KB gzip. Site JS is 4.53 KB raw / 1.96 KB gzip; site CSS
  is 11.54 KB raw / 3.32 KB gzip.
- Browser matrix: desktop 1440×900 and mobile 390×844, light/dark, and reduced
  motion passed for `/`, `/?demo=1`, `/privacy/`, `/terms/`, and `/404/` with
  one `h1`, a `main`, no horizontal overflow, no console/request errors, and
  zero axe serious/critical findings. The live wordmark is 44 px high.
- Privacy/network: the sample app made no external request. The live website's
  only outbound origin was the documented `https://api.github.com` release API.
- Offline/update: the built-in desktop sample works with external requests
  blocked. Cached release metadata remains usable when GitHub is offline, and
  uncached failure shows the release-page fallback without a console error.
- Lighthouse mobile production preview: Performance 100, Accessibility 100,
  Best Practices 100, SEO 100, LCP 1.055 s, CLS 0, TBT 0. Lighthouse 12.8.2
  still reported `TARGET_CRASHED` while collecting its final full-page
  screenshot after recording those scores; repeated Playwright runs did not
  reproduce a page crash.

## Deployment and identity

- Static deployment command:
  `/opt/fleet/lib/deploy-static.sh pdf-redaction-proof /work/repo/dist/site`.
- Azure Static Web Apps deployment ID:
  `66bab714-70fa-47c1-92fd-740ec764edc6` (`Succeeded`).
- `verify-url.sh` returned HTTPS 200 in 1143 ms with no console errors, a title,
  `lang=en`, one `h1`, one `main`, complete image alt text, and named buttons.
- Local and live `index.html` SHA-256 both equal
  `d0882796b9f21f4d5c121fbc9edc2b0a27a4cece16caae897165b5b2717acacb`.
- Live CSP, HSTS, `nosniff`, referrer, and permissions headers are present.
- Tag `v0.1.2` is the release tag for the repaired unsigned macOS, Windows, and
  Linux artifacts. Its workflow and checksum evidence are recorded below once
  complete.

## External response-policy evidence

The application-side burst path is repaired and tested. The shared Sociobot
billing service itself remains outside this repository and deployment. A fresh
30-request invalid-token probe on 2026-08-28 still returned 30 HTTP 200 responses
and no `Retry-After`. The billing-service operator must add an IP/token limit to
that shared endpoint if direct third-party traffic must also receive HTTP 429.
Changing that service from this static desktop work order would change scope and
deployment ownership.

## Known limits and operator action

- The geometry check remains strongest for axis-aligned text, opaque rectangles,
  and standard PDF operators. Rotated text, clipping, Form XObjects, and secrets
  baked into images still require visual review; the UI and report say so.
- The app intentionally has no self-updater. New versions are published through
  the release page and the site's current-installer lookup.
- Releases are unsigned until the operator supplies `APPLE_CERTIFICATE`,
  `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_ID`,
  `APPLE_PASSWORD`, `APPLE_TEAM_ID`, `WINDOWS_CERT_PFX`, and
  `WINDOWS_CERT_PASSWORD`.
