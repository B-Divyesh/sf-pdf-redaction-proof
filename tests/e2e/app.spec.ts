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
