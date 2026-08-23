#!/usr/bin/env node

import {mkdirSync, readFileSync} from "node:fs";
import {dirname, join, resolve} from "node:path";
import {fileURLToPath} from "node:url";
import {chromium} from "playwright";

const scriptPath = fileURLToPath(import.meta.url);
const repositoryRoot = resolve(dirname(scriptPath), "..");

function parseArguments(argv) {
  const options = {};
  for (let index = 0; index < argv.length; index += 1) {
    const key = argv[index];
    const value = argv[index + 1];
    if (!key?.startsWith("--") || !value || value.startsWith("--")) throw new Error(`Invalid argument near ${key}`);
    options[key.slice(2)] = value;
    index += 1;
  }
  return options;
}

function siteUrl(baseUrl, route = "") {
  const normalizedBase = baseUrl.endsWith("/") ? baseUrl : `${baseUrl}/`;
  return new URL(route.replace(/^\/+/, ""), normalizedBase).href;
}

async function setTheme(page, expected) {
  for (let attempt = 0; attempt < 2; attempt += 1) {
    const actual = await page.locator("html").getAttribute("data-theme");
    if (actual === expected) return;
    const toggle = page.locator('button[title*="dark and light mode"]').first();
    await toggle.click();
    await page.waitForTimeout(150);
  }
  const actual = await page.locator("html").getAttribute("data-theme");
  if (actual !== expected) throw new Error(`theme switch failed: expected ${expected}, observed ${actual}`);
}

async function runAxe(page, axeSource) {
  await page.addScriptTag({content: axeSource});
  return page.evaluate(async () => {
    const result = await globalThis.axe.run(document, {runOnly: {type: "tag", values: ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa"]}});
    return result.violations.map((violation) => ({
      id: violation.id,
      impact: violation.impact,
      nodes: violation.nodes.length,
      help: violation.help,
      findings: violation.nodes.slice(0, 12).map((node) => ({target: node.target, summary: node.failureSummary})),
    }));
  });
}

async function verifyRoot(page, {baseURL, theme, label, screenshotDir, axeSource}) {
  await page.goto(baseURL, {waitUntil: "networkidle"});
  await setTheme(page, theme);
  await page.getByRole("heading", {level: 1, name: "One trusted boundary for agent execution."}).waitFor();
  await page.getByText("UAR coordinates models, tools, skills, knowledge, memory, policy, and streaming protocols", {exact: false}).first().waitFor();
  if (await page.getByText(/Tutorial - Basics|Docusaurus Tutorial/iu).count()) throw new Error(`${label}: stock Docusaurus identity remains`);
  const overflow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
  if (overflow > 1) throw new Error(`${label}: horizontal overflow is ${overflow}px`);
  await page.keyboard.press("Tab");
  const focus = await page.evaluate(() => {
    const element = document.activeElement;
    if (!element || element === document.body) return {tag: "BODY", visible: false};
    const style = getComputedStyle(element);
    return {tag: element.tagName, visible: style.outlineStyle !== "none" || style.boxShadow !== "none"};
  });
  if (!focus.visible) throw new Error(`${label}: first keyboard target has no visible focus`);
  const violations = await runAxe(page, axeSource);
  if (violations.length) throw new Error(`${label}: accessibility violations ${JSON.stringify(violations)}`);
  await page.screenshot({path: join(screenshotDir, `${label}.png`), fullPage: true});
  return {label, theme, overflow, focus};
}

async function verifyRepresentativePages(page, baseURL, axeSource) {
  const checks = [
    ["docs/architecture/intro", /Runtime architecture/iu],
    ["docs/providers/inference", /Verify genuine inference/iu],
    ["docs/security/authentication", /Authenticate Requests/iu],
    ["docs/history/corrections", /Corrections and reversals/iu],
    ["docs/history/testing-methodology", /Testing methodology history/iu],
  ];
  for (const [route, heading] of checks) {
    await page.goto(siteUrl(baseURL, route), {waitUntil: "networkidle"});
    await page.getByRole("heading", {level: 1, name: heading}).waitFor();
    const overflow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
    if (overflow > 1) throw new Error(`${route}: horizontal overflow is ${overflow}px`);
    const violations = await runAxe(page, axeSource);
    if (violations.length) throw new Error(`${route}: accessibility violations ${JSON.stringify(violations)}`);
  }

  await page.goto(siteUrl(baseURL, "docs/architecture/intro"), {waitUntil: "networkidle"});
  await page.locator(".docusaurus-mermaid-container svg").first().waitFor({state: "visible"});

  await page.keyboard.press(process.platform === "darwin" ? "Meta+k" : "Control+k");
  const search = page.getByRole("textbox", {name: "Search"}).first();
  await search.waitFor({state: "visible"});
  if (!(await search.evaluate((element) => element === document.activeElement))) {
    throw new Error("search shortcut did not focus the search field");
  }
  await search.fill("RustCrypto");
  await page.locator('[role="option"]', {hasText: /RustCrypto/iu}).first().waitFor({state: "visible"});
}

async function main() {
  const args = parseArguments(process.argv.slice(2));
  const baseURL = args["base-url"];
  if (!baseURL) throw new Error("--base-url is required");
  const screenshotDir = resolve(args["screenshot-dir"] ?? join(repositoryRoot, "openspec", "changes", "certify-and-publish-uar-docs", "evidence", "screenshots"));
  mkdirSync(screenshotDir, {recursive: true});
  const axeSource = readFileSync(join(repositoryRoot, "frontend", "node_modules", "axe-core", "axe.min.js"), "utf8");
  const browser = await chromium.launch({headless: true});
  const observations = [];
  try {
    for (const configuration of [
      {label: "desktop-dark", viewport: {width: 1440, height: 1000}, theme: "dark"},
      {label: "desktop-light", viewport: {width: 1440, height: 1000}, theme: "light"},
      {label: "mobile-dark", viewport: {width: 390, height: 844}, theme: "dark"},
      {label: "mobile-light", viewport: {width: 390, height: 844}, theme: "light"},
    ]) {
      const consoleFindings = [];
      const requestFailures = [];
      const context = await browser.newContext({baseURL, viewport: configuration.viewport, colorScheme: configuration.theme});
      const page = await context.newPage();
      page.on("console", (message) => {
        if (["error", "warning"].includes(message.type())) consoleFindings.push(`${message.type()}: ${message.text()}`);
      });
      page.on("requestfailed", (request) => requestFailures.push(`${request.url()}: ${request.failure()?.errorText ?? "failed"}`));
      observations.push(await verifyRoot(page, {baseURL, ...configuration, screenshotDir, axeSource}));
      if (configuration.label === "desktop-dark") await verifyRepresentativePages(page, baseURL, axeSource);
      if (consoleFindings.length) throw new Error(`${configuration.label}: console findings ${JSON.stringify(consoleFindings)}`);
      if (requestFailures.length) throw new Error(`${configuration.label}: network failures ${JSON.stringify(requestFailures)}`);
      await context.close();
    }
  } finally {
    await browser.close();
  }
  console.log(JSON.stringify({status: "pass", screenshots: screenshotDir, observations}, null, 2));
}

main().catch((error) => {
  console.error(`Documentation browser validation failed: ${error.message}`);
  process.exit(1);
});
