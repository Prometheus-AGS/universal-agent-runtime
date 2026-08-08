import { defineConfig } from "@playwright/test";

const accessibilityPort = Number(process.env.UAR_FRONTEND_ACCESSIBILITY_PORT ?? "4175");
const accessibilityHost = process.env.UAR_FRONTEND_ACCESSIBILITY_HOST ?? "127.0.0.1";
const baseURL = `http://${accessibilityHost}:${accessibilityPort}`;

export default defineConfig({
  testDir: "./e2e",
  testMatch: "a11y-responsive-certification.spec.ts",
  timeout: 45_000,
  retries: 0,
  workers: 1,
  reporter: [
    ["line"],
    ["json", { outputFile: "../openspec/changes/a11y-and-responsive-certification/receipts/accessibility-playwright.json" }],
  ],
  outputDir: "test-results/accessibility/artifacts",
  expect: { timeout: 12_000 },
  use: {
    baseURL,
    browserName: "chromium",
    headless: true,
    serviceWorkers: "block",
    screenshot: "only-on-failure",
    trace: "retain-on-failure",
  },
  webServer: {
    command: `pnpm run dev --host ${accessibilityHost} --port ${accessibilityPort} --strictPort`,
    url: baseURL,
    reuseExistingServer: false,
    timeout: 60_000,
  },
});
