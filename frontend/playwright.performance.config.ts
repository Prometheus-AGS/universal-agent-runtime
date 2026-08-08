import { defineConfig } from "@playwright/test";

const performancePort = Number(process.env.UAR_FRONTEND_PERFORMANCE_PORT ?? "4174");
const performanceHost = process.env.UAR_FRONTEND_PERFORMANCE_HOST ?? "127.0.0.1";
const baseURL = `http://${performanceHost}:${performancePort}`;

export default defineConfig({
  testDir: "./e2e",
  testMatch: "performance-budget.spec.ts",
  timeout: 30_000,
  retries: 0,
  workers: 1,
  reporter: [
    ["line"],
    ["json", { outputFile: "../openspec/changes/a11y-and-responsive-certification/receipts/performance-playwright.json" }],
  ],
  outputDir: "test-results/performance-playwright/artifacts",
  expect: { timeout: 12_000 },
  use: {
    baseURL,
    browserName: "chromium",
    headless: true,
    screenshot: "only-on-failure",
    trace: "off",
  },
  webServer: {
    command: `pnpm run preview --host ${performanceHost} --port ${performancePort} --strictPort`,
    url: baseURL,
    reuseExistingServer: false,
    timeout: 60_000,
  },
});
