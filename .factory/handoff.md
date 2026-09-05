# Repair handoff — Redaction Proof 0.1.3

## Independent verification 2 — FAIL (2026-09-05)

Independent QA of implementation `7d3a118b91322b140642cd021b1504b86e1ebe2b`
and documentation candidate `3c39ce86e7a03b1749530380cf1e89f5a073fcdc`
found **7 findings** (2 high, 2 medium, 3 low) and **4 untested public
claims**. The acceptance verdict is **FAIL**.

The complete evidence and reproduction details are in
[`.factory/verification-2.md`](verification-2.md). The release build, declared
tests, corpus, parser sandbox, native real-file inspection, separate sanitized
copy, checksums, accessibility automation, and Lighthouse gates passed. The
remaining blockers are:

- the live US$12 checkout returns HTTP 404;
- four public promises do not have complete claim coverage;
- the installed app always displays its “Inspecting PDF structure…” panel;
- the phone sample result is below the first viewport and its demo label does
  not remain visible while scrolling;
- four standalone phone links are narrower than 44 px;
- unknown site URLs render the designed page with HTTP 200 instead of 404; and
- two public headings use metaphor instead of plain section/error names.

No product code was changed during this verification. This section supersedes
the acceptance status implied by the repair evidence below; that evidence is
retained as implementation history.

**Work order:** `pdf-redaction-proof-repair-2`

**Failed candidate:** `d618cda69433ecb2ef3a0b594e7ec1d52889de95`

**Verifier report:** `286534daf0cdce64ae2aa0d6748bab9083796f1e`

**Repair commits:** `dbd3013` (main repair) and `7d3a118` (Windows worker correction)

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
  Native Linux, macOS, and Windows tests all passed in the tagged GitHub matrix.
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
  `0c274bb9-1346-4967-9fe5-6be98ab56e50` (`Succeeded`).
- `verify-url.sh` returned HTTPS 200 in 1053 ms with no console errors, a title,
  `lang=en`, one `h1`, one `main`, complete image alt text, and named buttons.
- Local and live `index.html` SHA-256 both equal
  `63f53738326066cd15d92b93fb298c43218ed1fe566626225a9870aba791eabd`.
- Live CSP, HSTS, `nosniff`, referrer, and permissions headers are present.
- Tag `v0.1.3` is the release tag for the repaired unsigned macOS, Windows, and
  Linux artifacts. GitHub Actions run `33177881600` completed successfully,
  including all three native test/build jobs and the manifest job.
- Release `v0.1.3` contains DMG, MSI, EXE, AppImage, DEB, RPM,
  `SHA256SUMS`, `latest.json`, and the macOS app archive. `latest.json` names
  all three platform installers and reports `v0.1.3`.
- A clean DEB download reports package `redaction-proof`, version `0.1.3`,
  architecture `amd64`. Its SHA-256
  `b7fe67ccf93403b070fc7f61bb33c2e9023d3e3c4e1bf79b62d4d26964706852`
  exactly matches the published `SHA256SUMS` entry.
- Fresh live desktop and 390 px contexts resolved the Linux download to the
  v0.1.3 AppImage and produced no console errors.

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
