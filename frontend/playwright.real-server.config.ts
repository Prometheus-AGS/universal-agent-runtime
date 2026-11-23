import crypto from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "@playwright/test";
import { writeCombinedProviderConfig } from "./e2e/support/frozen-fixtures";

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
// Directory holding stub-llm and universal-agent-runtime. A pre-change worktree has no
// target/ of its own, so the boundary baseline run can point this at the built binaries.
const binDir = process.env.UAR_REAL_E2E_BIN_DIR ?? path.join(repoRoot, "target/debug");
// Bundle the runtime serves. Explicit so a baseline worktree serves its own build.
const staticDir = process.env.UAR_STATIC_DIR ?? path.join(repoRoot, "static");
// The contract makes a supporting real-server start BLOCKED when index.html is absent; fail
// before any test runs rather than serve a page that cannot load.
if (!fs.existsSync(path.join(staticDir, "index.html"))) {
  throw new Error(`UAR_STATIC_DIR ${staticDir} has no index.html; build the frontend first.`);
}
// JSON report location. Kept under playwright-report/ (git-ignored) so a run leaves the tree clean.
const jsonReportFile =
  process.env.UAR_REAL_E2E_JSON_REPORT ??
  path.join(frontendDir, "playwright-report", "real-server-playwright.json");
// Provider config the runtime loads through CONFIG_FILE (src/config.rs:22): the frozen
// SCN-IDN-001 collision providers plus the frozen SCN-PICK-002 nano-gpt provider.
const providerConfigPath = writeCombinedProviderConfig();

export default defineConfig({
  testDir: "./e2e",
  fullyParallel: false,
  workers: 1,
  retries: 0,
  reporter: [
    ["line"],
    ["json", { outputFile: jsonReportFile }],
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
      command: `${path.join(binDir, "stub-llm")} ${path.join(repoRoot, "tests/bdd/fixtures/bdd-chat.json")}`,
      cwd: repoRoot,
      env: { STUB_LLM_PORT: String(stubPort) },
      url: `${stubBaseUrl}/v1/models`,
      reuseExistingServer: false,
      timeout: 30_000,
    },
    {
      command: path.join(binDir, "universal-agent-runtime"),
      cwd: repoRoot,
      env: {
        CONFIG_FILE: providerConfigPath,
        UAR_STATIC_DIR: staticDir,
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
