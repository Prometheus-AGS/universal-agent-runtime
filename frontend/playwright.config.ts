import { defineConfig } from "@playwright/test";

const e2ePort = Number(process.env.UAR_FRONTEND_E2E_PORT ?? "8080");
const e2eHost = process.env.UAR_FRONTEND_E2E_HOST ?? "127.0.0.1";
const baseURL = `http://${e2eHost}:${e2ePort}`;

export default defineConfig({
  testDir: "./e2e",
  timeout: 30_000,
  retries: 1,
  use: {
    baseURL,
    headless: true,
    screenshot: "only-on-failure",
  },
  webServer: {
    command: `bun run dev -- --host ${e2eHost} --port ${e2ePort} --strictPort`,
    url: baseURL,
    reuseExistingServer: process.env.UAR_FRONTEND_E2E_REUSE === "true",
    timeout: 60_000,
  },
});
