import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "./tests/e2e",
  timeout: 30_000,
  use: { baseURL: "http://127.0.0.1:4173", trace: "retain-on-failure" },
  webServer: [
    { command: "npm run build:site && npx vite preview --config vite.site.config.ts --host 127.0.0.1", port: 4173, reuseExistingServer: true },
    { command: "npm run dev -- --host 127.0.0.1", port: 1420, reuseExistingServer: true },
  ],
  projects: [
    { name: "desktop", use: { ...devices["Desktop Chrome"] } },
    { name: "mobile", use: { ...devices["iPhone 13"], browserName: "chromium" } },
  ],
});
