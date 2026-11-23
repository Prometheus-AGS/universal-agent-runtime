import crypto from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "@playwright/test";

/**
 * SCN-SET-003 (uiux-settings-truthful-status-and-removals, task 5.3): the system default has
 * exactly one live control. Boots the real UAR binary against two real `stub-llm` listeners,
 * each serving one configured provider, and drives the Providers page.
 *
 * Authentication is never part of the scenario: the server runs with JWT and the settings
 * mutation guard off, and the spec sends no Authorization or admin-key header.
 *
 * Ports are distinct from `playwright.real-server.config.ts` (3003 / 4601). The static assets
 * come from `UAR_STATIC_DIR` when set (the baseline comparison points it at the pre-change
 * build), otherwise from `<repo>/static`.
 */
const frontendDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(frontendDir, "..");
const targetDebugDir =
  process.env.UAR_SETTINGS_TRUTH_TARGET_DIR ?? path.join(repoRoot, "target/debug");
const staticDir = process.env.UAR_STATIC_DIR ?? path.join(repoRoot, "static");
// The contract makes a supporting real-server start BLOCKED when index.html is absent; fail
// before any test runs rather than serve a page that cannot load.
if (!fs.existsSync(path.join(staticDir, "index.html"))) {
  throw new Error(`UAR_STATIC_DIR ${staticDir} has no index.html; build the frontend first.`);
}

const appPort = Number(process.env.UAR_SETTINGS_TRUTH_PORT ?? "3013");
const stubAPort = Number(process.env.UAR_SETTINGS_TRUTH_STUB_A_PORT ?? "4611");
const stubBPort = Number(process.env.UAR_SETTINGS_TRUTH_STUB_B_PORT ?? "4612");
const appBaseUrl = `http://127.0.0.1:${appPort}`;
const stubABaseUrl = `http://127.0.0.1:${stubAPort}`;
const stubBBaseUrl = `http://127.0.0.1:${stubBPort}`;

const scratchDir = path.join(os.tmpdir(), `uar-settings-truth-${crypto.randomUUID()}`);
fs.mkdirSync(scratchDir, { recursive: true });

/**
 * Provider 1 (`openai`) is seeded from `UAR_LLM__MODEL` / `UAR_LLM__BASE_URL` (the registry
 * default at boot). Provider 2 (`anthropic`) comes from the config file's `providers:` array
 * (`src/config.rs:300`, seeded at `src/server.rs:1070-1073`). The key is a placeholder for the
 * local stub only: the catalog marks a provider `configured` only when its registry entry has a
 * non-empty key (`src/server.rs:4087`, `api_catalog`), and `stub-llm` ignores Authorization.
 *
 * The frozen SCN-SET fixture `scratch-config.yaml` is read at run time and written first, so its
 * keys (`security.settings_mutation_auth_required`, `resilience.requests_per_second`) are in
 * force. The runner then appends only what it must own: the `providers:` block. The frozen file
 * must not define `providers` (a duplicate top-level key is invalid YAML), and a key the runner
 * sets through the process environment (`UAR_SECURITY__SETTINGS_MUTATION_AUTH_REQUIRED`) keeps the
 * runner's value. The baseline comparison runs this file inside the pre-change worktree, where
 * the fixture does not exist yet; that run is recognised by its explicit `UAR_STATIC_DIR`
 * and proceeds without the fixture. Any other run without the fixture stops here.
 */
const frozenConfigPath = path.join(
  repoRoot,
  "openspec/changes/uiux-settings-truthful-status-and-removals/fixtures/scratch-config.yaml",
);
function readFrozenScratchConfig(): string {
  if (!fs.existsSync(frozenConfigPath)) {
    if (process.env.UAR_STATIC_DIR !== undefined) return "";
    throw new Error(`frozen SCN-SET fixture not found: ${frozenConfigPath}`);
  }
  const frozen = fs.readFileSync(frozenConfigPath, "utf8");
  if (/^providers\s*:/m.test(frozen)) {
    throw new Error(`${frozenConfigPath} defines "providers", which the runner owns`);
  }
  return frozen.endsWith("\n") ? frozen : `${frozen}\n`;
}

const configPath = path.join(scratchDir, "config.yaml");
fs.writeFileSync(
  configPath,
  [
    readFrozenScratchConfig(),
    "providers:",
    "  - id: anthropic",
    "    display_name: Anthropic",
    `    base_url: ${stubBBaseUrl}/v1`,
    "    api_key: settings-truth-stub-key",
    "    protocol: auto",
    "    default_model: stub-b-model",
    "    models:",
    "      - id: stub-b-model",
    "    enabled: true",
    "",
  ].join("\n"),
);

const stubFixture = path.join(repoRoot, "tests/bdd/fixtures/bdd-chat.json");

export default defineConfig({
  testDir: "./e2e",
  testMatch: "settings-truth.spec.ts",
  fullyParallel: false,
  workers: 1,
  retries: 0,
  timeout: 60_000,
  // Stdout only: the boundary checks require a clean git tree between commands.
  reporter: [["line"]],
  // `test-results/` is gitignored (.gitignore:75).
  outputDir: "./test-results/settings-truth",
  use: {
    baseURL: appBaseUrl,
    headless: true,
    serviceWorkers: "block",
    trace: "retain-on-failure",
    screenshot: "only-on-failure",
  },
  webServer: [
    {
      command: `${path.join(targetDebugDir, "stub-llm")} ${stubFixture}`,
      cwd: repoRoot,
      env: { STUB_LLM_PORT: String(stubAPort) },
      url: `${stubABaseUrl}/v1/models`,
      reuseExistingServer: false,
      timeout: 30_000,
    },
    {
      command: `${path.join(targetDebugDir, "stub-llm")} ${stubFixture}`,
      cwd: repoRoot,
      env: { STUB_LLM_PORT: String(stubBPort) },
      url: `${stubBBaseUrl}/v1/models`,
      reuseExistingServer: false,
      timeout: 30_000,
    },
    {
      command: path.join(targetDebugDir, "universal-agent-runtime"),
      cwd: repoRoot,
      env: {
        CONFIG_FILE: configPath,
        UAR_STATIC_DIR: staticDir,
        UAR_SERVER__PORT: String(appPort),
        UAR_LLM__BASE_URL: `${stubABaseUrl}/v1`,
        UAR_LLM__API_KEY: "settings-truth-stub-key",
        UAR_LLM__MODEL: "openai/gpt-5.4-mini",
        UAR_PERSISTENCE__PROVIDER: "surreal",
        UAR_PERSISTENCE__DATABASE_URL: `surrealkv://${path.join(scratchDir, "store")}`,
        UAR_SECURITY__JWT_REQUIRED: "false",
        UAR_SECURITY__SETTINGS_MUTATION_AUTH_REQUIRED: "false",
      },
      url: `${appBaseUrl}/readyz`,
      reuseExistingServer: false,
      // Cold boot (SurrealKV init + MCP stdio spawn) has measured ~66s.
      timeout: 180_000,
    },
  ],
});
