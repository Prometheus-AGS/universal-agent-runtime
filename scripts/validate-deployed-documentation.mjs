#!/usr/bin/env node

import {existsSync, readFileSync} from "node:fs";
import {dirname, join, resolve} from "node:path";
import {fileURLToPath, pathToFileURL} from "node:url";

const scriptPath = fileURLToPath(import.meta.url);
const defaultRoot = resolve(dirname(scriptPath), "..");

function parseArguments(argv) {
  const options = {extraRoutes: []};
  for (let index = 0; index < argv.length; index += 1) {
    const key = argv[index];
    const value = argv[index + 1];
    if (!key?.startsWith("--") || !value || value.startsWith("--")) throw new Error(`Invalid argument near ${key}`);
    if (key === "--extra-route") options.extraRoutes.push(value);
    else options[key.slice(2)] = value;
    index += 1;
  }
  return options;
}

function routeUrl(baseUrl, route) {
  const normalizedBase = baseUrl.endsWith("/") ? baseUrl : `${baseUrl}/`;
  return new URL(String(route).replace(/^\/+/, ""), normalizedBase).href;
}

function requiredRoutes(root) {
  const manifestPath = join(root, "docs", "publication", "routes.json");
  if (!existsSync(manifestPath)) throw new Error(`Route manifest does not exist: ${manifestPath}`);
  const manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
  const routes = (manifest.routes ?? [])
    .filter((entry) => entry.status === "required")
    .map((entry) => entry.route);
  return [
    "",
    "docs/intro",
    "docs/architecture/intro",
    "docs/history/overview",
    "docs/history/testing-methodology",
    "docs/api/rust/",
    "docs/api/typescript/",
    ...routes,
  ];
}

async function fetchRoute(url, {retries, retryDelayMs}) {
  let lastError;
  for (let attempt = 1; attempt <= retries; attempt += 1) {
    try {
      const response = await fetch(url, {redirect: "follow", signal: AbortSignal.timeout(20_000)});
      const body = await response.text();
      if (response.ok) return {status: response.status, body, attempts: attempt};
      lastError = new Error(`HTTP ${response.status}`);
    } catch (error) {
      lastError = error;
    }
    if (attempt < retries) await new Promise((resolvePromise) => setTimeout(resolvePromise, retryDelayMs));
  }
  throw new Error(`${url}: ${lastError?.message ?? "request failed"}`);
}

export async function validateDeployedDocumentation({
  root = defaultRoot,
  baseUrl,
  extraRoutes = [],
  retries = 6,
  retryDelayMs = 10_000,
} = {}) {
  if (!baseUrl) throw new Error("--base-url is required");
  const routes = [...new Set([...requiredRoutes(resolve(root)), ...extraRoutes])];
  const results = [];
  for (const route of routes) {
    const url = routeUrl(baseUrl, route);
    const result = await fetchRoute(url, {retries, retryDelayMs});
    if ((route === "" || route === "docs/intro") && !result.body.includes("Universal Agent Runtime")) {
      throw new Error(`${url}: response is not the Universal Agent Runtime portal`);
    }
    if (/Tutorial - Basics|Docusaurus Tutorial/iu.test(result.body)) throw new Error(`${url}: stock Docusaurus identity remains`);
    results.push({route: route || "/", url, status: result.status, attempts: result.attempts});
    console.log(`PASS ${result.status} ${url}`);
  }
  return results;
}

async function main() {
  const args = parseArguments(process.argv.slice(2));
  const results = await validateDeployedDocumentation({
    root: args.root,
    baseUrl: args["base-url"],
    extraRoutes: args.extraRoutes,
    retries: Number(args.retries ?? 6),
    retryDelayMs: Number(args["retry-delay-ms"] ?? 10_000),
  });
  console.log(`Deployed documentation validation passed (${results.length} routes).`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  main().catch((error) => {
    console.error(`Deployed documentation validation failed: ${error.message}`);
    process.exit(1);
  });
}
