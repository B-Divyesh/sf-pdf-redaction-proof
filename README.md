# Redaction Proof

Redaction Proof checks a redacted or converted PDF before you share it. The
desktop app finds covered text, invisible text, document details, comments,
forms, layers, attachments, and automatic actions. It writes a separate clean
PDF and a JSON proof tied to both files by SHA-256.

The app does not upload PDFs or run PDF JavaScript. It does not claim legal
certainty. Every report includes the limits of the check.

## Product editions

Single-file checking, cleaning, and JSON export are free. A US$12 one-time Pro
license adds multi-file selection and a batch summary. The Sociobot billing API
handles checkout and license checks.

## Try the sample

Open <https://pdf-redaction-proof.sociobot.in/?demo=1> for the isolated sample.
It starts with a 12-page board packet result. The sample stays in memory and
saves no demo records. Choose **Reset demo** to restore it.

The desktop app also offers **Load sample project** on its first screen.

## Development

Prerequisites: Node.js 22+, Rust stable, and the [Tauri 2 system
dependencies](https://v2.tauri.app/start/prerequisites/) for your OS.

```sh
npm ci
npm run dev          # browser preview of the desktop UI
npm run dev:site     # landing site
npm run tauri dev    # native desktop app
```

On Linux, Tauri needs WebKitGTK 4.1 and related headers. Ubuntu CI installs
`libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`.

## Test and build

```sh
npm test             # Vitest + Rust corpus + Playwright/axe, desktop and 390px
npm run check        # TypeScript and Rust checks
npm run build        # app -> dist/app; deployable site -> dist/site
npm run build:site   # exact static deploy command -> dist/site
```

Playwright is pinned to 1.58.2. If its browser is not already available, run
`npx playwright install chromium`.

The release workflow runs on tags matching `v*`. It uses Tauri’s GitHub action
to build an unsigned universal macOS DMG, Windows MSI/EXE, and Linux
AppImage/DEB, then publishes `SHA256SUMS` and `latest.json`.

## Install

The website at <https://pdf-redaction-proof.sociobot.in> detects the operating
system and selects an installer from GitHub's latest-release API. Successful
release details are cached for one hour. If GitHub is unavailable, the page
shows a quiet publishing state and links to the releases page.

```sh
curl -fsSL https://pdf-redaction-proof.sociobot.in/install.sh | sh
```

On Windows PowerShell:

```powershell
irm https://pdf-redaction-proof.sociobot.in/install.ps1 | iex
```

Both scripts fetch `SHA256SUMS` from the GitHub Release and verify the selected
asset before installing or opening it. v1 binaries are unsigned; macOS users
must right-click the app and choose **Open**, and Windows may show SmartScreen.

## How the audit works

The trusted desktop process opens the chosen PDF and sends only its bytes and
base name to a separate parser process. The parser enters an operating-system
sandbox before parsing: Linux denies filesystem and network system calls with
Landlock and seccomp, macOS uses an application sandbox profile, and Windows
uses a restricted token. The worker also has a 500 MB input limit; Unix builds
cap it at 60 CPU seconds and 1.5 GB address space. Standard
text operators are spatially compared with later filled rectangles and PDF
redaction annotations. Sanitizing removes identified overlapping/invisible text
operators and strips metadata, name trees, actions, annotations, forms, and
optional-content configuration from a new copy, then immediately re-audits it.

The current geometry analysis is strongest for axis-aligned standard text and
rectangles. Unusual transforms, clipping paths, Form XObjects, and secrets
baked into images require visual review. This limitation is intentionally
visible in the app and report.

## Privacy and source

There is no telemetry, third-party runtime script, CDN font, or PDF upload.
Only a Pro license token and dated verification verdict are stored locally.
See `/privacy/` and `/terms/` on the site. The source is MIT licensed.
