import { describe, expect, it } from "vitest";
import { detectPlatform, parseRelease, RELEASE_API } from "./site";

const release = {
  tag_name: "v0.2.0",
  html_url: "https://github.com/B-Divyesh/sf-pdf-redaction-proof/releases/tag/v0.2.0",
  assets: [
    { name: "Redaction.Proof_0.2.0_universal.dmg", browser_download_url: "https://github.com/mac.dmg" },
    { name: "Redaction.Proof_0.2.0_x64_en-US.msi", browser_download_url: "https://github.com/windows.msi" },
    { name: "Redaction.Proof_0.2.0_amd64.AppImage", browser_download_url: "https://github.com/linux.AppImage" },
  ],
};

describe("download platform detection", () => {
  it("detects macOS", () => expect(detectPlatform("Mozilla Macintosh")).toBe("macos"));
  it("detects Windows", () => expect(detectPlatform("Windows NT 10.0")).toBe("windows"));
  it("falls back to Linux", () => expect(detectPlatform("X11 Linux x86_64")).toBe("linux"));
});

describe("GitHub release metadata", () => {
  it("uses the CORS-enabled GitHub API", () => {
    expect(RELEASE_API).toBe("https://api.github.com/repos/B-Divyesh/sf-pdf-redaction-proof/releases/latest");
  });

  it("selects one installer for each platform", () => {
    expect(parseRelease(release)?.platforms).toEqual({
      macos: release.assets[0],
      windows: release.assets[1],
      linux: release.assets[2],
    });
  });

  it("rejects releases before all platform installers exist", () => {
    expect(parseRelease({ ...release, assets: release.assets.slice(0, 2) })).toBeNull();
  });
});
