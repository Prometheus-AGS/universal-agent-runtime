import { defineConfig } from '@playwright/test';
import { defineBddConfig, cucumberReporter } from 'playwright-bdd';
import { resolve } from 'node:path';

const evidence = process.env.UAR_BROWSER_EVIDENCE_DIR;
if (!evidence) throw new Error('Set UAR_BROWSER_EVIDENCE_DIR to a fresh evidence directory');
const testDir = defineBddConfig({
  features: 'features/browser-history-reload.feature',
  steps: 'browser/browser-history-reload.steps.mjs',
  outputDir: resolve('target/manual-browser-history-reload-generated'),
});
export default defineConfig({
  testDir, timeout: 480_000, expect: { timeout: 120_000 }, workers: 1, retries: 0,
  outputDir: resolve(evidence, 'artifacts'),
  reporter: [['list'], ['json', { outputFile: resolve(evidence, 'playwright.json') }],
    cucumberReporter('json', { outputFile: resolve(evidence, 'cucumber.json') })],
  use: { baseURL: process.env.UAR_BASE_URL, channel: 'chrome', headless: true,
    viewport: { width: 1280, height: 900 }, video: 'off', screenshot: 'on', trace: 'on',
    extraHTTPHeaders: { 'x-uar-workspace-id': process.env.UAR_WORKSPACE_ID ?? '' } },
});
