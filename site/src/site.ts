import "./site.css";

type Platform = "macos" | "windows" | "linux";
type Manifest = { version: string; platforms: Record<Platform, { url: string; sha256: string; label?: string }> };
const RELEASE = "https://github.com/B-Divyesh/sf-pdf-redaction-proof/releases/latest";
const MANIFEST = `${RELEASE}/download/latest.json`;

export function detectPlatform(userAgent = navigator.userAgent): Platform {
  if (/Mac|iPhone|iPad/.test(userAgent)) return "macos";
  if (/Win/.test(userAgent)) return "windows";
  return "linux";
}

const labels: Record<Platform,string> = { macos:"macOS", windows:"Windows", linux:"Linux" };

async function resolveDownloads() {
  const platform = detectPlatform();
  const button = document.querySelector<HTMLAnchorElement>("#download-button")!;
  const note = document.querySelector<HTMLElement>("#download-note")!;
  try {
    const response = await fetch(MANIFEST, { headers: { Accept: "application/json" } });
    if (!response.ok) throw new Error("No release manifest");
    const manifest = await response.json() as Manifest;
    const target = manifest.platforms[platform];
    button.href = target.url;
    button.textContent = `Download for ${labels[platform]}`;
    note.textContent = `${manifest.version} · SHA-256 published`;
    document.querySelectorAll<HTMLAnchorElement>("[data-platform]").forEach(link => {
      const key = link.dataset.platform as Platform;
      if (manifest.platforms[key]) link.href = manifest.platforms[key].url;
    });
  } catch {
    button.href = RELEASE;
    button.textContent = `View ${labels[platform]} download`;
    note.textContent = "Release assets and checksums on GitHub";
    document.querySelectorAll<HTMLAnchorElement>("[data-platform]").forEach(link => link.href = RELEASE);
  }
}

if (typeof document !== "undefined") resolveDownloads();
