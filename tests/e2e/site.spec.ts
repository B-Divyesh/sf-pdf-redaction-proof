import { test, expect } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

test("landing page has one clear primary heading and no serious accessibility violations", async ({ page }) => {
  const errors: string[] = [];
  page.on("console", message => { if (message.type() === "error") errors.push(message.text()); });
  await page.goto("/");
  await expect(page).toHaveTitle(/Redaction Proof/);
  await expect(page.locator("main h1")).toHaveCount(1);
  await expect(page.getByRole("link", { name: /Download|View/ })).toBeVisible();
  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations.filter(v => ["serious", "critical"].includes(v.impact || ""))).toEqual([]);
  expect(errors).toEqual([]);
});

test("policy pages are routed and readable", async ({ page }) => {
  await page.goto("/privacy/");
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("Privacy, without fine print.");
  await page.goto("/terms/");
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("Terms of use.");
});
