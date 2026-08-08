#!/usr/bin/env bash
set -euo pipefail

change_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${change_dir}/../../.." && pwd)"
cd "${repo_root}"

node <<'NODE'
const fs = require("node:fs");

const packageJson = JSON.parse(fs.readFileSync("frontend/package.json", "utf8"));
const devDependencies = packageJson.devDependencies ?? {};
const exact = {
  "tailwindcss": "4.3.3",
  "@tailwindcss/vite": "4.3.3",
  "tw-animate-css": "1.4.0",
};

for (const [name, version] of Object.entries(exact)) {
  if (devDependencies[name] !== version) {
    throw new Error(`${name} must be pinned exactly to ${version}`);
  }
}

for (const removed of ["tailwindcss-animate", "autoprefixer", "postcss"]) {
  if (packageJson.dependencies?.[removed] || devDependencies[removed]) {
    throw new Error(`${removed} must not remain a direct dependency`);
  }
}

if (devDependencies.vite !== "^8.1.4") {
  throw new Error("The two maintained lockfiles must share the frontend Vite 8.1.4 resolution");
}

for (const lockPath of ["frontend/pnpm-lock.yaml", "pnpm-lock.yaml"]) {
  const lock = fs.readFileSync(lockPath, "utf8");
  if (!/vite:\n\s+specifier: \^?8\.1\.4\n\s+version: 8\.1\.4/.test(lock)) {
    throw new Error(`${lockPath} does not resolve the frontend importer to Vite 8.1.4`);
  }
  if (/vite@8\.(?:1\.3|2\.1)/.test(lock)) {
    throw new Error(`${lockPath} contains a divergent Vite 8 resolution`);
  }
}

const components = JSON.parse(fs.readFileSync("frontend/components.json", "utf8"));
if (components.tailwind?.config !== "") {
  throw new Error("frontend/components.json must use the Tailwind 4 empty config path");
}
if (components.tailwind?.css !== "src/index.css") {
  throw new Error("frontend/components.json must retain src/index.css as its CSS entry");
}

const indexCss = fs.readFileSync("frontend/src/index.css", "utf8");
const tokenCss = fs.readFileSync("frontend/src/shared/theme/tokens.css", "utf8");

function extractBlock(css, marker) {
  const markerStart = css.indexOf(marker);
  if (markerStart < 0) throw new Error(`Missing CSS block: ${marker}`);
  const open = css.indexOf("{", markerStart);
  let depth = 0;
  for (let cursor = open; cursor < css.length; cursor += 1) {
    if (css[cursor] === "{") depth += 1;
    if (css[cursor] === "}") depth -= 1;
    if (depth === 0) return css.slice(open + 1, cursor);
  }
  throw new Error(`Unclosed CSS block: ${marker}`);
}

function variables(block) {
  return Object.fromEntries(
    [...block.matchAll(/--([\w-]+):\s*([^;]+);/g)].map((match) => [match[1], match[2].trim()]),
  );
}

function hslToRgb(value) {
  const match = value.match(/^(\d+(?:\.\d+)?)\s+(\d+(?:\.\d+)?)%\s+(\d+(?:\.\d+)?)%$/);
  if (!match) throw new Error(`Expected an HSL channel triple, got: ${value}`);
  const [, rawHue, rawSaturation, rawLightness] = match;
  const hue = Number(rawHue);
  const saturation = Number(rawSaturation) / 100;
  const lightness = Number(rawLightness) / 100;
  const chroma = (1 - Math.abs(2 * lightness - 1)) * saturation;
  const x = chroma * (1 - Math.abs((hue / 60) % 2 - 1));
  const offset = lightness - chroma / 2;
  const base = hue < 60 ? [chroma, x, 0]
    : hue < 120 ? [x, chroma, 0]
      : hue < 180 ? [0, chroma, x]
        : hue < 240 ? [0, x, chroma]
          : hue < 300 ? [x, 0, chroma]
            : [chroma, 0, x];
  return base.map((channel) => Math.round((channel + offset) * 255));
}

function hexToRgb(value) {
  const match = value.match(/^#([0-9a-f]{6})$/i);
  if (!match) throw new Error(`Expected a six-digit hex color, got: ${value}`);
  return [0, 2, 4].map((offset) => Number.parseInt(match[1].slice(offset, offset + 2), 16));
}

const roleMap = {
  background: "bg",
  chrome: "bg-2",
  surface: "surface",
  card: "card",
  "card-hov": "card-hov",
  muted: "muted",
  foreground: "fg",
  "fg-sub": "fg-sub",
  "fg-faint": "fg-faint",
  primary: "ember",
  "primary-foreground": "ember-fg",
  cyan: "cyan",
  success: "green",
  warning: "amber",
  destructive: "red",
};

for (const [theme, legacyMarker, tokenMarker] of [
  ["dark", ":root {", "@theme {"],
  ["light", ".light {", ".light {"],
  ["high-contrast", ".high-contrast {", ".high-contrast {"],
]) {
  const legacy = variables(extractBlock(indexCss, legacyMarker));
  const canonical = variables(extractBlock(tokenCss, tokenMarker));
  for (const [legacyRole, canonicalRole] of Object.entries(roleMap)) {
    const expected = hslToRgb(legacy[legacyRole]);
    const actual = hexToRgb(canonical[`color-${canonicalRole}`]);
    if (expected.some((channel, index) => Math.abs(channel - actual[index]) > 3)) {
      throw new Error(`${theme} ${canonicalRole} diverges from its staged HSL channel`);
    }
  }
}

const light = variables(extractBlock(tokenCss, ".light {"));
const systemLight = variables(extractBlock(tokenCss, ":root:not(.dark, .high-contrast) {"));
const phaseRoles = ["phase-context", "phase-skill", "phase-memory", "phase-retrieval", "phase-reasoning", "phase-tool", "phase-generate"];
const lightParityRoles = [...new Set([...Object.values(roleMap), ...phaseRoles])];
for (const canonicalRole of lightParityRoles) {
  if (light[`color-${canonicalRole}`] !== systemLight[`color-${canonicalRole}`]) {
    throw new Error(`System-light ${canonicalRole} must match the explicit light theme`);
  }
}
NODE

test ! -e frontend/tailwind.config.ts
test ! -e frontend/postcss.config.js

rg -q 'from "@tailwindcss/vite"' frontend/vite.config.ts
rg -q 'plugins: \[react\(\), tailwindcss\(\)\]' frontend/vite.config.ts
rg -q 'from "@tailwindcss/vite"' frontend/vite.config.js
rg -q 'plugins: \[react\(\), tailwindcss\(\)\]' frontend/vite.config.js
rg -q '^@import "\./shared/theme/tokens\.css";$' frontend/src/index.css
rg -q '^@import "tailwindcss" source\("\.\./\.\./\.\."\);$' frontend/src/shared/theme/tokens.css
rg -q '^@import "tw-animate-css";$' frontend/src/shared/theme/tokens.css
rg -q '^@source "\.\./\.\./\.\./packages/a2ui-uar/src";$' frontend/src/shared/theme/tokens.css
rg -q '^@custom-variant dark ' frontend/src/shared/theme/tokens.css
rg -q '^@theme \{' frontend/src/shared/theme/tokens.css
rg -q '^@theme inline \{' frontend/src/shared/theme/tokens.css
rg -q 'prefers-reduced-motion: reduce' frontend/src/shared/theme/tokens.css
rg -q 'animation-iteration-count: 1 !important' frontend/src/shared/theme/tokens.css

for token in \
  bg bg-2 surface card card-hov muted fg fg-sub fg-faint \
  ember ember-fg ember-tint cyan green amber red focus-ring \
  phase-context phase-skill phase-memory phase-retrieval phase-reasoning \
  phase-tool phase-generate; do
  rg -q -- "--color-${token}:" frontend/src/shared/theme/tokens.css
done

for token in display sans body ui reading mono; do
  rg -q -- "--font-${token}:" frontend/src/shared/theme/tokens.css
done

for token in sm md lg xl 2xl 3xl 4xl; do
  rg -q -- "--radius-${token}:" frontend/src/shared/theme/tokens.css
done

rg -q -- '--radius-sm: calc\(var\(--radius\) \* 0\.6\);' frontend/src/shared/theme/tokens.css
rg -q -- '--radius-4xl: calc\(var\(--radius\) \* 2\.6\);' frontend/src/shared/theme/tokens.css

for token in fast base slow; do
  rg -q -- "--duration-${token}:" frontend/src/shared/theme/tokens.css
done

for token in standard emphasis; do
  rg -q -- "--ease-${token}:" frontend/src/shared/theme/tokens.css
done

if rg -q -- '--ease-out:' frontend/src/shared/theme/tokens.css; then
  echo "Project easing tokens must not replace Tailwind's global ease-out role" >&2
  exit 1
fi

for animation in accordion-down accordion-up fade-in shimmer; do
  rg -q -- "--animate-${animation}:" frontend/src/shared/theme/tokens.css
  rg -q -- "@keyframes ${animation}" frontend/src/shared/theme/tokens.css
done

rg -q -- '--accordion-panel-height' frontend/src/components/ui/accordion.tsx
rg -q -- 'animate-accordion-down' frontend/src/components/ui/accordion.tsx

workflow_hits="$(rg -c "frontend/src/shared/theme/tokens\.css" .github/workflows/storybook-visual-regression.yml)"
test "${workflow_hits}" -eq 2

if rg --hidden -n 'tailwind\.config|postcss\.config' .github/workflows frontend \
  --glob '*.{css,html,js,json,mjs,cjs,ts,tsx,yaml,yml}' \
  --glob '!node_modules/**' \
  --glob '!coverage/**' \
  --glob '!test-results/**' \
  --glob '!dist/**' \
  --glob '!pnpm-lock.yaml' \
  --glob '!packages/prometheus-entity-management/docs/**' \
  --glob '!frontend/packages/prometheus-entity-management/docs/**'; then
  echo "Found a dangling reference to a deleted Tailwind/PostCSS config" >&2
  exit 1
fi

echo "Tailwind 4 CSS-first token foundation assertions passed."
