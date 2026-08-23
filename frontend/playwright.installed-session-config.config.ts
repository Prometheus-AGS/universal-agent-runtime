import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./e2e",
  testMatch: "chat-session-config.spec.ts",
  timeout: 180_000,
  retries: 0,
  workers: 1,
  reporter: [
    ["line"],
    ["json", { outputFile: "../.prometheus/evidence/session-configuration/playwright-report.json" }],
  ],
  outputDir: "../.prometheus/evidence/session-configuration/playwright-artifacts",
  use: {
    baseURL: "http://127.0.0.1:1906",
    headless: true,
    serviceWorkers: "block",
    screenshot: "only-on-failure",
    trace: "retain-on-failure",
  },
});
