import path from 'node:path';
import os from 'node:os';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';
import { defineConfig } from '@playwright/test';
import { defineBddConfig, cucumberReporter } from 'playwright-bdd';
import { APP_PORT, APP_BASE_URL, STUB_PORT, STUB_BASE_URL } from './support/ports';

// Mirrors the root tests/e2e/playwright.config.ts pattern (single `cargo run`
// serving the built frontend + API at 127.0.0.1:3001), plus a second
// webServer entry that boots the deterministic stub LLM first and points the
// main app at it — see design.md D2/D3 in
// openspec/changes/bdd-chat-scenario-suite/.
//
// Playwright spawns webServer commands with cwd = this config file's own
// directory (tests/bdd/) by default — both the main app (needs repo-root
// cwd to find config.yaml/static/) and cargo itself need the real repo root,
// so `cwd` is set explicitly below rather than relying on the default.
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, '..', '..');
const STUB_FIXTURES = path.resolve(__dirname, 'fixtures/bdd-chat.json');

// Fresh embedded SurrealKV store per run, same mechanism
// tests/integration/live/harness.rs uses for the Rust-side integration
// suite — `persistence.provider` has no default (config.rs) and this repo
// checkout ships no config.yaml/.env, so it must be supplied explicitly.
const PERSISTENCE_PATH = path.join(os.tmpdir(), `uar-bdd-persistence-${crypto.randomUUID()}`);

const testDir = defineBddConfig({
  features: 'features/*.feature',
  steps: ['steps/*.ts', 'support/world.ts'],
});

export default defineConfig({
  testDir,
  fullyParallel: false, // scenarios share one stub-llm request log (see world.ts) — keep sequential
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  workers: 1,
  // Paths below are relative to this config file's own directory
  // (tests/bdd/), not the repo root or CWD — confirmed via testDir/webServer
  // behavior; using repo-root-style "tests/bdd/..." prefixes here doubles
  // the nesting (tests/bdd/tests/bdd/...).
  reporter: [
    ['html', { outputFolder: 'playwright-report' }],
    ['list'],
    // Cucumber-format JSON report, consumed by the bdd-video-proof skill's
    // mint-certification-bundle.sh (task 15).
    cucumberReporter('json', { outputFile: 'cucumber-report.json' }),
  ],
  use: {
    // Tests open the page through the dev proxy (see webServer below), not
    // the backend's static serve — same shape as local development.
    baseURL: 'http://127.0.0.1:8085',
    trace: 'on-first-retry',
    video: 'on',
  },
  outputDir: 'test-results',
  webServer: [
    {
      // Runs the already-built binary directly rather than `cargo run` —
      // two concurrent `cargo run` webServer entries fight over the same
      // target-dir build lock (observed: the app entry blocked behind the
      // stub's build, then re-triggered its own ~80s rebuild, blowing the
      // startup timeout). `test:bdd`'s package.json script runs `cargo
      // build --bin stub-llm --bin universal-agent-runtime` first so both
      // binaries already exist by the time webServer starts them.
      command: `${path.join(REPO_ROOT, 'target/debug/stub-llm')} ${STUB_FIXTURES}`,
      cwd: REPO_ROOT,
      env: { STUB_LLM_PORT: String(STUB_PORT) },
      url: `${STUB_BASE_URL}/v1/models`,
      reuseExistingServer: !process.env.CI,
      timeout: 30_000,
    },
    {
      command: path.join(REPO_ROOT, 'target/debug/universal-agent-runtime'),
      cwd: REPO_ROOT,
      url: APP_BASE_URL,
      env: {
        // NOT `PORT` — confirmed dead: `Cli::port` (config.rs, `#[arg(long,
        // env = "PORT")]`) is parsed but never applied to `server.port`
        // anywhere in config.rs/main.rs. `UAR_SERVER__PORT` is config-rs's
        // own `Environment` source and is the one that actually works.
        UAR_SERVER__PORT: String(APP_PORT),
        UAR_LLM__BASE_URL: `${STUB_BASE_URL}/v1`,
        UAR_LLM__API_KEY: 'test-stub-key',
        UAR_LLM__MODEL: 'openai/gpt-5.4-mini',
        UAR_PERSISTENCE__PROVIDER: 'surreal',
        UAR_PERSISTENCE__DATABASE_URL: `surrealkv://${PERSISTENCE_PATH}`,
        // NOT `JWT_REQUIRED` — same dead-CLI-passthrough bug as `PORT`
        // above: `Cli::jwt_required` is parsed but never applied to
        // `security.jwt_required` in config.rs. `UAR_SECURITY__JWT_REQUIRED`
        // (config-rs `Environment` source) is what actually works.
        UAR_SECURITY__JWT_REQUIRED: 'false',
        // JWT is configured (secret present) but not required — the suite
        // opens the web page and calls the API without a token.
        UAR_SECURITY__JWT_SECRET: 'bdd-dev-secret-at-least-32-characters-long',
      },
      reuseExistingServer: !process.env.CI,
      // Cold boot (SurrealKV init + MCP stdio spawn) measures ~66s on dev
      // machines; 30s raced it every time (same fix as
      // frontend/playwright.real-server.config.ts).
      timeout: 180_000,
    },
    {
      // Frontend dev proxy (vite): the page is served from source and its
      // /api, /healthz, /readyz calls are proxied to the backend above —
      // the same shape as local development, no JWT in the browser.
      command: 'bun run dev --host 127.0.0.1 --port 8085 --strictPort',
      cwd: path.join(REPO_ROOT, 'frontend'),
      env: { UAR_BACKEND_URL: APP_BASE_URL },
      url: 'http://127.0.0.1:8085/threads',
      reuseExistingServer: !process.env.CI,
      timeout: 60_000,
    },
  ],
});
