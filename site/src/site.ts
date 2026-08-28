import "./site.css";
import "./contrast.css";
import "./purchase.css";

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

function capturePurchase() {
  const url = new URL(location.href);
  const token = url.searchParams.get("license");
  if (!token) return;
  localStorage.setItem("sb_license:pdf-redaction-proof", token);
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

if (typeof document !== "undefined") {
  capturePurchase();
  if (location.hostname === "pdf-redaction-proof.sociobot.in") resolveDownloads();
  else {
    const platform = detectPlatform();
    document.querySelector<HTMLAnchorElement>("#download-button")!.textContent = `View ${labels[platform]} download`;
    document.querySelector<HTMLElement>("#download-note")!.textContent = "Release assets and checksums on GitHub";
    document.querySelectorAll<HTMLAnchorElement>("[data-platform]").forEach(link => link.href = RELEASE);
  }
}
