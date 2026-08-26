import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./e2e",
  testMatch: "prompt-caching-installed.spec.ts",
  timeout: 60_000,
  retries: 0,
  workers: 1,
  reporter: [
    ["line"],
    [
      "json",
      {
        outputFile:
          "../.prometheus/evidence/prompt-caching/playwright-report.json",
      },
    ],
  ],
  outputDir: "../.prometheus/evidence/prompt-caching/playwright-artifacts",
  use: {
    baseURL: "http://127.0.0.1:1906",
    headless: true,
    serviceWorkers: "block",
    screenshot: "only-on-failure",
    trace: "retain-on-failure",
  },
});
