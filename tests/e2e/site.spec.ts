import { test, expect, type Page } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

const apiUrl = "https://api.github.com/repos/B-Divyesh/sf-pdf-redaction-proof/releases/latest";
const release = {
  tag_name: "v0.1.0",
  html_url: "https://github.com/B-Divyesh/sf-pdf-redaction-proof/releases/tag/v0.1.0",
  assets: [
    { name: "Redaction.Proof_0.1.0_universal.dmg", browser_download_url: "https://github.com/B-Divyesh/sf-pdf-redaction-proof/releases/download/v0.1.0/Redaction.Proof_0.1.0_universal.dmg" },
    { name: "Redaction.Proof_0.1.0_x64_en-US.msi", browser_download_url: "https://github.com/B-Divyesh/sf-pdf-redaction-proof/releases/download/v0.1.0/Redaction.Proof_0.1.0_x64_en-US.msi" },
    { name: "Redaction.Proof_0.1.0_amd64.AppImage", browser_download_url: "https://github.com/B-Divyesh/sf-pdf-redaction-proof/releases/download/v0.1.0/Redaction.Proof_0.1.0_amd64.AppImage" },
  ],
};

async function mockRelease(page: Page, status = 200) {
  await page.route(apiUrl, route => route.fulfill({ status, contentType: "application/json", body: status === 200 ? JSON.stringify(release) : "{}" }));
}

function watchErrors(page: Page) {
  const consoleErrors: string[] = [];
  const failedRequests: string[] = [];
  page.on("console", message => { if (message.type() === "error") consoleErrors.push(message.text()); });
  page.on("pageerror", error => consoleErrors.push(error.message));
  page.on("requestfailed", request => failedRequests.push(`${request.url()}: ${request.failure()?.errorText}`));
  return { consoleErrors, failedRequests };
}

test("landing page has one clear heading and no serious accessibility violations", async ({ page }) => {
  await mockRelease(page);
  const errors = watchErrors(page);
  await page.goto("/");
  await expect(page).toHaveTitle("Redaction Proof — Check a PDF before you send it");
  await expect(page.locator("main h1")).toHaveCount(1);
  await expect(page.getByRole("link", { name: "Try it with sample data" })).toBeVisible();
  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations.filter(v => ["serious", "critical"].includes(v.impact || ""))).toEqual([]);
  expect(errors).toEqual({ consoleErrors: [], failedRequests: [] });
});

test("dark and reduced-motion modes have no serious accessibility violations", async ({ page }) => {
  await mockRelease(page);
  await page.emulateMedia({ colorScheme: "dark", reducedMotion: "reduce" });
  await page.goto("/");
  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations.filter(v => ["serious", "critical"].includes(v.impact || ""))).toEqual([]);
});

test("@claim:release-downloads resolves installers through the GitHub API and caches the result", async ({ page }) => {
  let apiCalls = 0;
  const requests: string[] = [];
  page.on("request", request => requests.push(request.url()));
  await page.route(apiUrl, route => { apiCalls += 1; return route.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify(release) }); });
  await page.goto("/");
  await expect(page.locator("#download-note")).toHaveText("Version 0.1.0 · Checksums published");
  await expect(page.locator("#download-button")).toHaveAttribute("href", /releases\/download\/v0\.1\.0\/Redaction\.Proof_0\.1\.0_.*\.(dmg|msi|AppImage)$/);
  expect(requests).toContain(apiUrl);
  expect(requests.some(url => url.includes("download/latest.json"))).toBe(false);
  await page.reload();
  await expect(page.locator("#download-note")).toHaveText("Version 0.1.0 · Checksums published");
  expect(apiCalls).toBe(1);
});

test("@claim:release-cache-period uses a cached release for one hour and refreshes at expiry", async ({ page }) => {
  let apiCalls = 0;
  await page.route(apiUrl, route => {
    apiCalls += 1;
    return route.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify(release) });
  });
  await page.goto("/");
  await page.evaluate(metadata => {
    localStorage.setItem("pdf-redaction-proof:release-metadata:v1", JSON.stringify({
      cachedAt: Date.now() - 3_590_000,
      metadata,
    }));
  }, {
    version: "cached",
    releaseUrl: release.html_url,
    platforms: { macos: release.assets[0], windows: release.assets[1], linux: release.assets[2] },
  });
  apiCalls = 0;
  await page.reload();
  await expect(page.locator("#download-note")).toHaveText("Version cached · Checksums published");
  expect(apiCalls).toBe(0);
  await page.evaluate(() => {
    const key = "pdf-redaction-proof:release-metadata:v1";
    const cached = JSON.parse(localStorage.getItem(key) || "{}");
    cached.cachedAt = Date.now() - 3_600_000;
    localStorage.setItem(key, JSON.stringify(cached));
  });
  await page.reload();
  await expect(page.locator("#download-note")).toHaveText("Version 0.1.0 · Checksums published");
  expect(apiCalls).toBe(1);
});

test("fresh cached metadata keeps downloads available when the API is offline", async ({ page }) => {
  await page.addInitScript(({ key, metadata }) => {
    localStorage.setItem(key, JSON.stringify({ cachedAt: Date.now(), metadata }));
    const nativeFetch = window.fetch.bind(window);
    window.fetch = (input, init) => String(input).includes("api.github.com") ? Promise.reject(new TypeError("offline")) : nativeFetch(input, init);
  }, {
    key: "pdf-redaction-proof:release-metadata:v1",
    metadata: {
      version: "0.1.0",
      releaseUrl: release.html_url,
      platforms: { macos: release.assets[0], windows: release.assets[1], linux: release.assets[2] },
    },
  });
  await page.goto("/");
  await expect(page.locator("#download-note")).toHaveText("Version 0.1.0 · Checksums published");
});

test("an unavailable release has a calm, working state without browser errors", async ({ page }) => {
  await page.addInitScript(url => {
    const nativeFetch = window.fetch.bind(window);
    window.fetch = (input, init) => String(input) === url ? Promise.reject(new TypeError("API unavailable")) : nativeFetch(input, init);
  }, apiUrl);
  const errors = watchErrors(page);
  await page.goto("/");
  await expect(page.locator("#download-note")).toHaveText("Downloads are being published. Check the GitHub release page soon.");
  await expect(page.locator("#download-button")).toHaveText("View releases");
  await expect(page.locator("#download-button")).toHaveAttribute("href", "https://github.com/B-Divyesh/sf-pdf-redaction-proof/releases/latest");
  expect(errors).toEqual({ consoleErrors: [], failedRequests: [] });
});

test("@claim:demo-isolation opens and resets the sample without saving demo records", async ({ page }) => {
  await mockRelease(page);
  await page.goto("/?demo=1");
  await expect(page).toHaveTitle("Demo — Redaction Proof");
  await expect(page.getByText("Demo — sample data, nothing is saved")).toBeVisible();
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("Review a sample redaction audit.");
  await expect(page.getByRole("heading", { name: "Recoverable content found" })).toBeVisible();
  await page.getByRole("button", { name: "Reset demo" }).click();
  await expect(page.locator("#demo-status")).toHaveText("Sample audit reset.");
  const demoKeys = await page.evaluate(() => Object.keys(localStorage).filter(key => key.startsWith("demo:")));
  expect(demoKeys).toEqual([]);
});

test("@claim:sample-findings shows covered text and author metadata", async ({ page }) => {
  await mockRelease(page);
  await page.goto("/?demo=1");
  await expect(page.locator("#sample-audit")).toContainText("Covered text");
  await expect(page.locator("#sample-audit")).toContainText("Author metadata");
  await expect(page.locator("#sample-audit")).toContainText("Two hidden items");
});

test("@claim:site-network-privacy loads no tracker, font CDN, or third-party runtime service", async ({ page }) => {
  const origins = new Set<string>();
  page.on("request", request => origins.add(new URL(request.url()).origin));
  await mockRelease(page);
  await page.goto("/");
  await expect(page.locator("#download-note")).toHaveText("Version 0.1.0 · Checksums published");
  expect([...origins].sort()).toEqual(["http://127.0.0.1:4173", "https://api.github.com"]);
});

test("@claim:single-file-price states the free limit and exact one-time price", async ({ page }) => {
  await mockRelease(page);
  await page.goto("/");
  await expect(page.getByText("Single-file checks are free.")).toBeVisible();
  await expect(page.getByText("Pro costs US$12 once.")).toBeVisible();
  await expect(page.getByRole("link", { name: "Buy Pro for $12" })).toHaveAttribute("href", "https://api.sociobot.in/api/v1/products/pdf-redaction-proof/checkout");
  await page.goto("http://127.0.0.1:1420");
  await expect(page.getByText("Single-file checking, cleaning, and JSON proof stay free.")).toBeVisible();
  await expect(page.getByText("Pro adds multi-file selection for a one-time US$12 purchase.")).toBeVisible();
  await expect(page.getByRole("link", { name: "Buy Pro — $12 once" })).toHaveAttribute("href", "https://api.sociobot.in/api/v1/products/pdf-redaction-proof/checkout");
});

test("home wordmark has a 44px minimum pointer target", async ({ page }) => {
  await mockRelease(page);
  await page.goto("/");
  const box = await page.locator(".site-header .brand").boundingBox();
  expect(box?.width).toBeGreaterThanOrEqual(44);
  expect(box?.height).toBeGreaterThanOrEqual(44);
});

test("phone demo shows the sample immediately and keeps its label while scrolling", async ({ page }, testInfo) => {
  test.skip(testInfo.project.name !== "mobile", "This layout check uses the phone viewport.");
  await mockRelease(page);
  await page.goto("/");
  await page.getByRole("link", { name: "Try it with sample data" }).click();
  await expect(page).toHaveURL(/\?demo=1$/);
  const sample = page.locator("#sample-audit");
  await expect(sample).toContainText("Recoverable content found");
  const sampleBox = await sample.boundingBox();
  const viewport = page.viewportSize();
  expect(sampleBox?.y).toBeLessThan(viewport!.height);
  await page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));
  await expect(page.getByText("Demo — sample data, nothing is saved")).toBeInViewport();
});

test("phone navigation, platform, and footer links have 44px touch targets", async ({ page }, testInfo) => {
  test.skip(testInfo.project.name !== "mobile", "This layout check uses the phone viewport.");
  await mockRelease(page);
  await page.goto("/");
  for (const link of [
    page.locator(".site-header nav a").first(),
    page.locator(".platform-links a[data-platform=linux]"),
    page.locator("footer a[href='/?demo=1']"),
    page.locator("footer a[href='/terms/']"),
  ]) {
    const box = await link.boundingBox();
    expect(box?.width).toBeGreaterThanOrEqual(44);
    expect(box?.height).toBeGreaterThanOrEqual(44);
  }
});

test("keyboard users can skip to the main content", async ({ page }) => {
  await mockRelease(page);
  await page.goto("/");
  await page.keyboard.press("Tab");
  await expect(page.getByRole("link", { name: "Skip to content" })).toBeFocused();
  await page.keyboard.press("Enter");
  await expect(page.locator("main")).toBeFocused();
});

test("policy and missing pages have their own titles and one heading", async ({ page }) => {
  await page.goto("/privacy/");
  await expect(page).toHaveTitle("Privacy — Redaction Proof");
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("Privacy policy");
  await page.goto("/terms/");
  await expect(page).toHaveTitle("Terms — Redaction Proof");
  await expect(page.locator("main h1")).toHaveCount(1);
  await page.goto("/404/");
  await expect(page).toHaveTitle("Page not found — Redaction Proof");
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("Page not found.");
});
