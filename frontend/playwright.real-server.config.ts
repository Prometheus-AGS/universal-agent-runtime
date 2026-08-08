import crypto from "node:crypto";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "@playwright/test";

const frontendDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(frontendDir, "..");
const appPort = Number(process.env.UAR_REAL_E2E_PORT ?? "3003");
const stubPort = Number(process.env.UAR_REAL_E2E_STUB_PORT ?? "4601");
const appBaseUrl = `http://127.0.0.1:${appPort}`;
const stubBaseUrl = `http://127.0.0.1:${stubPort}`;
const persistencePath = path.join(
  os.tmpdir(),
  `uar-provider-route-e2e-${crypto.randomUUID()}`,
);

export default defineConfig({
  testDir: "./e2e",
  fullyParallel: false,
  workers: 1,
  retries: 0,
  reporter: [
    ["line"],
    ["json", { outputFile: "../openspec/changes/a11y-and-responsive-certification/receipts/real-server-playwright.json" }],
  ],
  timeout: 60_000,
  use: {
    baseURL: appBaseUrl,
    headless: true,
    trace: "retain-on-failure",
    screenshot: "only-on-failure",
  },
  webServer: [
    {
      command: `${path.join(repoRoot, "target/debug/stub-llm")} ${path.join(repoRoot, "tests/bdd/fixtures/bdd-chat.json")}`,
      cwd: repoRoot,
      env: { STUB_LLM_PORT: String(stubPort) },
      url: `${stubBaseUrl}/v1/models`,
      reuseExistingServer: false,
      timeout: 30_000,
    },
    {
      command: path.join(repoRoot, "target/debug/universal-agent-runtime"),
      cwd: repoRoot,
      env: {
        UAR_SERVER__PORT: String(appPort),
        UAR_LLM__BASE_URL: `${stubBaseUrl}/v1`,
        UAR_LLM__API_KEY: "provider-route-test-key",
        UAR_LLM__MODEL: "openai/gpt-5.4-mini",
        UAR_PERSISTENCE__PROVIDER: "surreal",
        UAR_PERSISTENCE__DATABASE_URL: `surrealkv://${persistencePath}`,
        UAR_SECURITY__JWT_REQUIRED: "false",
        UAR_SECURITY__SETTINGS_MUTATION_AUTH_REQUIRED: "false",
      },
      url: `${appBaseUrl}/readyz`,
      reuseExistingServer: false,
      // Cold boot is slow on dev machines (SurrealKV init + MCP stdio spawn
      // measure ~66s on first run); 60s was intermittently too short.
      timeout: 180_000,
    },
  ],
});
