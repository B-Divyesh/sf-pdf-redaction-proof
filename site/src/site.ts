import "./site.css";
import "./contrast.css";
import "./purchase.css";

export type Platform = "macos" | "windows" | "linux";

type GitHubAsset = {
  name: string;
  browser_download_url: string;
};

export type GitHubRelease = {
  tag_name: string;
  html_url: string;
  assets: GitHubAsset[];
};

type DownloadMetadata = {
  version: string;
  releaseUrl: string;
  platforms: Record<Platform, GitHubAsset>;
};

type CachedRelease = {
  cachedAt: number;
  metadata: DownloadMetadata;
};

export const RELEASE_PAGE = "https://github.com/B-Divyesh/sf-pdf-redaction-proof/releases/latest";
export const RELEASE_API = "https://api.github.com/repos/B-Divyesh/sf-pdf-redaction-proof/releases/latest";
export const RELEASE_CACHE_KEY = "pdf-redaction-proof:release-metadata:v1";
export const RELEASE_CACHE_MS = 60 * 60 * 1000;

const labels: Record<Platform, string> = { macos: "macOS", windows: "Windows", linux: "Linux" };

export function detectPlatform(userAgent = navigator.userAgent): Platform {
  if (/Mac|iPhone|iPad/.test(userAgent)) return "macos";
  if (/Win/.test(userAgent)) return "windows";
  return "linux";
}

function firstAsset(assets: GitHubAsset[], patterns: RegExp[]): GitHubAsset | undefined {
  for (const pattern of patterns) {
    const match = assets.find(asset => pattern.test(asset.name));
    if (match) return match;
  }
}

export function parseRelease(release: GitHubRelease): DownloadMetadata | null {
  if (!release || typeof release.tag_name !== "string" || !Array.isArray(release.assets)) return null;
  const macos = firstAsset(release.assets, [/universal\.dmg$/i, /\.dmg$/i]);
  const windows = firstAsset(release.assets, [/\.msi$/i, /setup\.exe$/i, /\.exe$/i]);
  const linux = firstAsset(release.assets, [/\.appimage$/i, /_amd64\.deb$/i, /\.deb$/i]);
  if (!macos || !windows || !linux) return null;
  const releaseUrl = /^https:\/\/github\.com\//.test(release.html_url) ? release.html_url : RELEASE_PAGE;
  return {
    version: release.tag_name.replace(/^v/, ""),
    releaseUrl,
    platforms: { macos, windows, linux },
  };
}

function readCachedRelease(now = Date.now()): DownloadMetadata | null {
  try {
    const value = localStorage.getItem(RELEASE_CACHE_KEY);
    if (!value) return null;
    const cached = JSON.parse(value) as CachedRelease;
    if (!cached.cachedAt || now - cached.cachedAt >= RELEASE_CACHE_MS) return null;
    return cached.metadata?.platforms ? cached.metadata : null;
  } catch {
    return null;
  }
}

function cacheRelease(metadata: DownloadMetadata) {
  try {
    localStorage.setItem(RELEASE_CACHE_KEY, JSON.stringify({ cachedAt: Date.now(), metadata } satisfies CachedRelease));
  } catch {
    // Downloads still work when browser storage is disabled or full.
  }
}

function renderDownloads(metadata: DownloadMetadata) {
  const platform = detectPlatform();
  const button = document.querySelector<HTMLAnchorElement>("#download-button");
  const note = document.querySelector<HTMLElement>("#download-note");
  if (!button || !note) return;
  button.href = metadata.platforms[platform].browser_download_url;
  button.textContent = `Download for ${labels[platform]}`;
  note.textContent = `Version ${metadata.version} · Checksums published`;
  note.dataset.state = "ready";
  document.querySelectorAll<HTMLAnchorElement>("[data-platform]").forEach(link => {
    const key = link.dataset.platform as Platform;
    const asset = metadata.platforms[key];
    if (asset) link.href = asset.browser_download_url;
  });
}

function renderPublishingState() {
  const button = document.querySelector<HTMLAnchorElement>("#download-button");
  const note = document.querySelector<HTMLElement>("#download-note");
  if (!button || !note) return;
  button.href = RELEASE_PAGE;
  button.textContent = "View releases";
  note.textContent = "Downloads are being published. Check the GitHub release page soon.";
  note.dataset.state = "pending";
  document.querySelectorAll<HTMLAnchorElement>("[data-platform]").forEach(link => { link.href = RELEASE_PAGE; });
}

export async function resolveDownloads(fetcher: typeof fetch = fetch) {
  const cached = readCachedRelease();
  if (cached) {
    renderDownloads(cached);
    return;
  }

  try {
    const response = await fetcher(RELEASE_API, {
      headers: {
        Accept: "application/vnd.github+json",
        "X-GitHub-Api-Version": "2022-11-28",
      },
    });
    if (!response.ok) {
      renderPublishingState();
      return;
    }
    const metadata = parseRelease(await response.json() as GitHubRelease);
    if (!metadata) {
      renderPublishingState();
      return;
    }
    cacheRelease(metadata);
    renderDownloads(metadata);
  } catch {
    renderPublishingState();
  }
}

function capturePurchase() {
  const url = new URL(location.href);
  const token = url.searchParams.get("license");
  if (!token) return;
  try { localStorage.setItem("sb_license:pdf-redaction-proof", token); } catch { /* The token remains visible for manual copy. */ }
  url.searchParams.delete("license");
  history.replaceState({}, "", `${url.pathname}${url.search}${url.hash}`);
  const panel = document.querySelector<HTMLElement>("#purchase-return");
  const field = document.querySelector<HTMLInputElement>("#returned-license");
  if (!panel || !field) return;
  panel.hidden = false;
  field.value = token;
  document.querySelector<HTMLButtonElement>("#copy-license")?.addEventListener("click", async event => {
    const button = event.currentTarget as HTMLButtonElement;
    try { await navigator.clipboard.writeText(token); button.textContent = "License copied"; }
    catch { field.select(); document.querySelector<HTMLElement>("#copy-status")!.textContent = "Press Ctrl/Cmd+C to copy the selected token."; }
  });
}

function enterDemo() {
  const url = new URL(location.href);
  if (url.searchParams.get("demo") !== "1") return;
  document.title = "Demo — Redaction Proof";
  document.body.classList.add("demo-active");
  const banner = document.querySelector<HTMLElement>("#demo-banner");
  const art = document.querySelector<HTMLElement>("#hero-art");
  const sample = document.querySelector<HTMLElement>("#sample-audit");
  const title = document.querySelector<HTMLElement>("#page-title");
  const lede = document.querySelector<HTMLElement>("#hero-lede");
  if (banner) banner.hidden = false;
  if (art) art.hidden = true;
  if (sample) sample.hidden = false;
  if (title) title.textContent = "Review a sample redaction audit.";
  if (lede) lede.textContent = "This built-in sample shows hidden text and metadata. Demo actions stay in memory and save nothing.";
  document.querySelector<HTMLButtonElement>("#reset-demo")?.addEventListener("click", () => {
    if (!sample) return;
    sample.classList.remove("demo-reset");
    requestAnimationFrame(() => sample.classList.add("demo-reset"));
    document.querySelector<HTMLElement>("#demo-status")!.textContent = "Sample audit reset.";
  });
  requestAnimationFrame(() => title?.focus({ preventScroll: true }));
}

if (typeof document !== "undefined") {
  capturePurchase();
  enterDemo();
  void resolveDownloads();
}
