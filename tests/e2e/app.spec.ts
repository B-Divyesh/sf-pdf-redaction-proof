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

test("@claim:local-processing loads the built-in audit without an external request", async ({ page }) => {
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
