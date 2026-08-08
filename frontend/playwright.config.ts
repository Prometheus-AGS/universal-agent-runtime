import { defineConfig } from "@playwright/test";

const e2ePort = Number(process.env.UAR_FRONTEND_E2E_PORT ?? "8080");
const e2eHost = process.env.UAR_FRONTEND_E2E_HOST ?? "127.0.0.1";
const baseURL = `http://${e2eHost}:${e2ePort}`;

export default defineConfig({
  testDir: "./e2e",
  testIgnore: [
    "a11y-responsive-certification.spec.ts",
    "performance-budget.spec.ts",
    "provider-route-real.spec.ts",
    "knowledge-rag-real.spec.ts",
  ],
  reporter: [
    ["line"],
    ["json", { outputFile: "../openspec/changes/a11y-and-responsive-certification/receipts/default-playwright.json" }],
  ],
  timeout: 30_000,
  retries: 1,
  // Cold Vite transforms of heavy admin chunks can exceed the 5s default
  // expect timeout on dev machines; give assertions room.
  expect: { timeout: 12_000 },
  use: {
    baseURL,
    headless: true,
    serviceWorkers: "block",
    screenshot: "only-on-failure",
  },
  webServer: {
    command: `bun run dev -- --host ${e2eHost} --port ${e2ePort} --strictPort`,
    url: baseURL,
    reuseExistingServer: process.env.UAR_FRONTEND_E2E_REUSE === "true",
    timeout: 60_000,
  },
});
