#!/usr/bin/env node

// Flat 2.0 census — counts border/shadow/blur idioms that actually RENDER a
// visible line, per docs/knowme-ui-ux-standard.md §3.1–3.3.
//
// Why this is narrower than "grep for border": `frontend/src/index.css` sets
// `--border: transparent` in the base and dark themes and applies
// `* { @apply border-transparent }` globally, so width-only classes (`border`,
// `border-b`, …) and the `border-border` token render NOTHING today. They are
// also the delivery mechanism for the high-contrast theme, which deliberately
// sets `--border: 0 0% 100%` ("HC exception: visible lines are the
// accessibility feature"). Counting or stripping them would break
// accessibility while reporting progress.
//
// What DOES render is any border that supplies its own color and therefore
// bypasses the token: arbitrary `border-[…]` values, Tailwind palette colors,
// and semantic color tokens. Those are the real violation surface.
//
// Exceptions live in frontend/flat2-allowlist.md and are declared below.

import { readFileSync, readdirSync, statSync } from "node:fs";
import { relative, resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const src = resolve(root, "frontend/src");

/** Borders that carry accessibility meaning — see flat2-allowlist.md §1. */
const ALLOWED_CLASSES = new Set(["border-ring"]);

/**
 * Deferred by CATEGORY — state and input chrome.
 *
 * NOT permanent exemptions. Each of these borders carries state (error,
 * success, checked, selected) or an input/control affordance. Deleting one
 * without adding a replacement surface loses information the user needs, and
 * KnowMe §3.3 forbids status conveyed by color alone. Converting them to
 * tinted surfaces is design work per component, not a codemod — so it is
 * scoped to `flat2-state-and-input-surfaces` rather than smuggled into a
 * mechanical purge.
 *
 * Keyed on the class, not the file: the previous file-keyed version missed 25
 * usages spread across 14 files, which is exactly the kind of silent
 * under-count a gate must not produce.
 */
const DEFERRED_CLASS_RE =
  /^border(?:-[btlrxy])?-(?:destructive|success|input|accent|sidebar|secondary|warning|info|muted|popover|card)(?:\/\d+)?$/;

/**
 * Deferred by FILE, with the change that owns each.
 */
const DEFERRED = new Map([
  // Owned by `flat2-state-and-input-surfaces`. Checked-state, control edges,
  // aria-invalid, and the dashed drop-target affordance.
  ["frontend/src/components/ui/checkbox.tsx", "state/input change"],
  ["frontend/src/components/ui/radio-group.tsx", "state/input change"],
  ["frontend/src/components/ui/slider.tsx", "state/input change"],
  ["frontend/src/components/ui/field.tsx", "state/input change"],
  ["frontend/src/components/ui/input.tsx", "state/input change"],
  ["frontend/src/components/ui/textarea.tsx", "state/input change"],
  ["frontend/src/components/ui/select.tsx", "state/input change"],
  ["frontend/src/components/ui/native-select.tsx", "state/input change"],
  ["frontend/src/components/ui/input-group.tsx", "state/input change"],
  ["frontend/src/components/ui/input-otp.tsx", "state/input change"],
  ["frontend/src/components/ui/combobox.tsx", "state/input change"],
  ["frontend/src/components/ui/calendar.tsx", "state/input change"],
  ["frontend/src/admin/pages/knowledge-page.tsx", "state/input change"],
]);

/** Files exempt with a written justification in flat2-allowlist.md. */
const ALLOWED_FILES = new Set([
  // Recharts legend swatches encode series color — functional data encoding,
  // not chrome. Removing the swatch removes the data key.
  "frontend/src/components/ui/chart.tsx",
]);

const PATTERNS = [
  { name: "arbitrary-value", re: /\bborder(?:-[btlrxy])?-\[[^\]]+\]/g },
  {
    name: "palette-color",
    re: /\bborder(?:-[btlrxy])?-(?:red|green|blue|amber|emerald|cyan|slate|zinc|gray|yellow|orange|purple|pink|rose|teal|indigo|violet|lime|fuchsia|sky|stone|neutral)-\d{2,3}(?:\/\d+)?\b/g,
  },
  {
    name: "semantic-color",
    re: /\bborder(?:-[btlrxy])?-(?:destructive|primary|secondary|success|warning|info|accent|muted|input|ring|popover|card|sidebar)(?:\/\d+)?\b/g,
  },
];

function walk(dir) {
  return readdirSync(dir).flatMap((entry) => {
    const path = resolve(dir, entry);
    return statSync(path).isDirectory() ? walk(path) : [path];
  });
}

const isSource = (p) => /\.(tsx|ts|css)$/.test(p) && !/\.(test|spec)\.(tsx|ts)$/.test(p);
const repoPath = (p) => relative(root, p).replaceAll("\\", "/");

const findings = [];
const deferred = [];
for (const path of walk(src).filter(isSource)) {
  const file = repoPath(path);
  if (ALLOWED_FILES.has(file)) continue;

  const lines = readFileSync(path, "utf8").split("\n");
  lines.forEach((line, i) => {
    for (const { name, re } of PATTERNS) {
      for (const match of line.matchAll(re)) {
        if (ALLOWED_CLASSES.has(match[0])) continue;
        const hit = { file, line: i + 1, kind: name, text: match[0] };
        if (DEFERRED_CLASS_RE.test(match[0])) {
          deferred.push({ ...hit, owner: "state/input change" });
        } else if (DEFERRED.has(file)) {
          deferred.push({ ...hit, owner: DEFERRED.get(file) });
        } else {
          findings.push(hit);
        }
      }
    }
  });
}

// Deferred counts are reported, never hidden: a gate that silently drops work
// reads as "covered everything" when it did not.
const deferredByOwner = deferred.reduce(
  (acc, d) => ({ ...acc, [d.owner]: (acc[d.owner] ?? 0) + 1 }),
  {},
);

if (process.argv.includes("--json")) {
  process.stdout.write(`${JSON.stringify({ total: findings.length, findings }, null, 2)}\n`);
  process.exit(findings.length ? 1 : 0);
}

if (process.argv.includes("--by-file")) {
  const byFile = findings.reduce((acc, f) => ({ ...acc, [f.file]: (acc[f.file] ?? 0) + 1 }), {});
  for (const [file, n] of Object.entries(byFile).sort((a, b) => b[1] - a[1])) {
    console.log(`${String(n).padStart(4)}  ${file}`);
  }
  console.log(`\nTotal: ${findings.length}`);
  process.exit(findings.length ? 1 : 0);
}

function reportDeferred() {
  if (!deferred.length) return;
  console.log(`\nDeferred (${deferred.length}), owned by a later change:`);
  for (const [owner, n] of Object.entries(deferredByOwner).sort((a, b) => b[1] - a[1])) {
    console.log(`  ${owner}: ${n}`);
  }
  console.log("Rationale per file: frontend/flat2-allowlist.md.");
}

if (!findings.length) {
  console.log("Flat 2.0 census: 0 in-scope visible-border violations.");
  reportDeferred();
  process.exit(0);
}

const byKind = findings.reduce((acc, f) => ({ ...acc, [f.kind]: (acc[f.kind] ?? 0) + 1 }), {});
console.error(`Flat 2.0 census: ${findings.length} visible-border violation(s).`);
console.error("Each supplies its own color, bypassing --border and the high-contrast theme.\n");
for (const [kind, n] of Object.entries(byKind).sort((a, b) => b[1] - a[1])) {
  console.error(`  ${kind}: ${n}`);
}
console.error("\nRun with --by-file for per-file counts, --json for detail.");
console.error("Exceptions must be declared in frontend/flat2-allowlist.md.");
process.exit(1);
