import { test, expect } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

test("desktop workbench empty state is accessible", async ({ page }) => {
  const errors: string[] = [];
  page.on("console", message => { if (message.type() === "error") errors.push(message.text()); });
  await page.goto("http://127.0.0.1:1420");
  await expect(page).toHaveTitle(/Redaction Proof/);
  await expect(page.locator("main h1")).toHaveCount(1);
  await expect(page.getByRole("button", { name: "Choose PDF", exact: true })).toBeVisible();
  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations.filter(v => ["serious", "critical"].includes(v.impact || ""))).toEqual([]);
  expect(errors).toEqual([]);
});

test("@claim:offline-sample loads the built-in audit without an external request", async ({ page }) => {
  const externalRequests: string[] = [];
  page.on("request", request => {
    if (!request.url().startsWith("http://127.0.0.1:1420")) externalRequests.push(request.url());
  });
  await page.goto("http://127.0.0.1:1420");
  await page.getByRole("button", { name: "Load sample project" }).click();
  await expect(page.getByRole("heading", { name: "Recoverable content found" })).toBeVisible();
  await expect(page.getByText("sample-board-minutes.pdf")).toBeVisible();
  expect(externalRequests).toEqual([]);
});

test("license verification is limited locally and reports a 429-style retry delay", async ({ page }) => {
  let requests = 0;
  await page.route("https://api.sociobot.in/api/v1/products/pdf-redaction-proof/verify?*", route => {
    requests += 1;
    return route.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify({ valid: false, reason: "invalid" }) });
  });
  await page.goto("http://127.0.0.1:1420");
  await page.getByRole("button", { name: "Have a license?" }).click();
  const field = page.getByLabel("License token");
  for (let attempt = 1; attempt <= 6; attempt++) {
    await field.fill(`invalid-${attempt}`);
    await page.getByRole("button", { name: "Verify" }).click();
    if (attempt < 6) await expect(page.locator("#license-status")).toContainText("License no longer active");
  }
  await expect(page.locator("#license-status")).toContainText("Too many license checks. Try again in");
  expect(requests).toBe(5);
});

test("license verification respects upstream 429 Retry-After", async ({ page }) => {
  let requests = 0;
  await page.route("https://api.sociobot.in/api/v1/products/pdf-redaction-proof/verify?*", route =>
    { requests += 1; return route.fulfill({ status: 429, headers: { "Access-Control-Allow-Origin": "*", "Access-Control-Expose-Headers": "Retry-After", "Retry-After": "42" }, body: "rate limited" }); });
  await page.goto("http://127.0.0.1:1420");
  await page.getByRole("button", { name: "Have a license?" }).click();
  await page.getByLabel("License token").fill("invalid-upstream");
  await page.getByRole("button", { name: "Verify" }).click();
  await expect(page.locator("#license-status")).toHaveText("Too many license checks. Try again in 42 seconds.");
  await page.getByRole("button", { name: "Verify" }).click();
  await expect(page.locator("#license-status")).toContainText("Too many license checks. Try again in");
  expect(requests).toBe(1);
});
