#!/usr/bin/env node

import { readFileSync, readdirSync, statSync } from "node:fs";
import { dirname, relative, resolve } from "node:path";
import ts from "typescript";

const root = resolve(import.meta.dirname, "..");
const src = resolve(root, "frontend/src");
const fixtureIndex = process.argv.indexOf("--fixture-dir");
const scanRoot = fixtureIndex >= 0 ? resolve(root, process.argv[fixtureIndex + 1]) : src;
const infrastructureFetchExceptions = new Set([
  // Shared transport bootstrap, not feature/domain I/O.
  "frontend/src/entities/sync.ts",
  // Loads the PGlite WASM/data assets used to initialize client persistence.
  "frontend/src/platform/pglite/assets.ts",
]);
const legacyPerRowGraphWriteLocations = new Set([
  // Existing debt outside this change's permitted product surface. Exact
  // locations make any edit re-enter the gate instead of silently expanding it.
  "frontend/src/features/agents/model/agents-admin-store.ts:40",
  "frontend/src/features/agents/model/agents-admin-store.ts:37",
  "frontend/src/features/agents/model/agents-graph.ts:32",
  "frontend/src/features/compiler/model/compiler-sessions.ts:21",
  "frontend/src/features/compiler/model/compiler-store.ts:25",
  "frontend/src/features/compiler/model/compiler-store.ts:26",
  "frontend/src/features/knowledge/model/knowledge-store.ts:60",
  "frontend/src/features/knowledge/model/knowledge-store.ts:74",
  "frontend/src/features/knowledge/model/knowledge-store.ts:57",
  "frontend/src/features/knowledge/model/knowledge-store.ts:71",
  "frontend/src/features/memory/model/memory-admin-store.ts:33",
  "frontend/src/features/memory/model/memory-admin-store.ts:34",
  "frontend/src/features/memory/model/memory-admin-store.ts:61",
  "frontend/src/features/memory/model/memory-graph.ts:24",
  "frontend/src/features/models/model/models-graph.ts:45",
  "frontend/src/features/models/model/models-store.ts:49",
  "frontend/src/features/providers/model/providers-graph.ts:36",
  "frontend/src/features/providers/model/providers-graph.ts:51",
  "frontend/src/features/providers/model/providers-graph.ts:86",
  "frontend/src/features/providers/model/providers-store.ts:70",
  "frontend/src/features/providers/model/providers-store.ts:76",
  "frontend/src/features/providers/model/providers-store.ts:183",
  "frontend/src/features/settings/model/settings-graph.ts:21",
  "frontend/src/features/settings/model/settings-store.ts:64",
  "frontend/src/features/settings/model/settings-store.ts:111",
  "frontend/src/features/settings/model/settings-store.ts:88",
  "frontend/src/features/settings/model/settings-store.ts:122",
  "frontend/src/features/settings/model/settings-store.ts:123",
  "frontend/src/features/skills/model/skills-admin-store.ts:37",
  "frontend/src/features/skills/model/skills-admin-store.ts:34",
  "frontend/src/features/skills/model/skills-graph.ts:16",
  "frontend/src/features/tools/model/mcp-health-store.ts:27",
  "frontend/src/features/tools/model/mcp-health-store.ts:24",
  "frontend/src/features/tools/model/mcp-status.ts:32",
  "frontend/src/features/tools/model/tool-graph.ts:48",
  "frontend/src/features/tools/model/tools-admin-store.ts:44",
  "frontend/src/features/tools/model/tools-admin-store.ts:46",
]);

function walk(dir) {
  return readdirSync(dir).flatMap((entry) => {
    const path = resolve(dir, entry);
    return statSync(path).isDirectory() ? walk(path) : [path];
  });
}

function repoPath(path) {
  return relative(root, path).replaceAll("\\", "/");
}

function isProductionTs(path) {
  return /\.(ts|tsx)$/.test(path) && !/\.(test|spec)\.(ts|tsx)$/.test(path) && !path.includes("/__tests__/");
}

function layerFor(path) {
  const name = path.split("/").at(-1) ?? "";
  if (path.includes("/services/") || path.includes("/api/")) return "service";
  if (path.includes("/stores/") || /-store\.(?:ts|tsx)$/.test(name)) return "store";
  if (path.includes("/hooks/") || /^use(?:-|[A-Z])[^.]*\.(ts|tsx)$/.test(name)) return "hook";
  if (path.endsWith(".tsx")) return "component";
  return "module";
}

function importedModules(content) {
  const modules = [];
  const staticImport = /\b(?:import|export)\s+(?:[^"'`;]*?\sfrom\s*)?["']([^"']+)["']/g;
  const dynamicImport = /\bimport\s*\(\s*["']([^"']+)["']\s*\)/g;
  for (const match of content.matchAll(staticImport)) modules.push(match[1]);
  for (const match of content.matchAll(dynamicImport)) modules.push(match[1]);
  return modules;
}

function lineNumberAt(content, index) {
  return content.slice(0, Math.max(0, index)).split("\n").length;
}

function firstMatchIndex(content, pattern) {
  const match = pattern.exec(content);
  return match?.index ?? 0;
}

function sourceLine(sourceFile, node) {
  return sourceFile.getLineAndCharacterOfPosition(node.getStart(sourceFile)).line + 1;
}

function importBindings(sourceFile, moduleName, importedNames) {
  const names = new Set();
  const namespaces = new Set();
  for (const statement of sourceFile.statements) {
    if (!ts.isImportDeclaration(statement) || statement.moduleSpecifier.text !== moduleName) continue;
    const clause = statement.importClause;
    if (!clause) continue;
    if (clause.name) namespaces.add(clause.name.text);
    if (clause.namedBindings && ts.isNamespaceImport(clause.namedBindings)) {
      namespaces.add(clause.namedBindings.name.text);
    } else if (clause.namedBindings && ts.isNamedImports(clause.namedBindings)) {
      for (const element of clause.namedBindings.elements) {
        const imported = element.propertyName?.text ?? element.name.text;
        if (importedNames.has(imported)) names.add(element.name.text);
      }
    }
  }
  return { names, namespaces };
}

function isNamedCall(node, bindings, memberName) {
  if (!ts.isCallExpression(node)) return false;
  if (ts.isIdentifier(node.expression)) return bindings.names.has(node.expression.text);
  return ts.isPropertyAccessExpression(node.expression)
    && ts.isIdentifier(node.expression.expression)
    && bindings.namespaces.has(node.expression.expression.text)
    && node.expression.name.text === memberName;
}

function isFunctionScope(node) {
  return ts.isFunctionDeclaration(node)
    || ts.isFunctionExpression(node)
    || ts.isArrowFunction(node)
    || ts.isMethodDeclaration(node)
    || ts.isGetAccessorDeclaration(node)
    || ts.isSetAccessorDeclaration(node)
    || ts.isConstructorDeclaration(node);
}

function renderBodySetterLines(sourceFile, reactUseState) {
  const results = [];
  function visit(node) {
    if (
      ts.isVariableDeclaration(node)
      && ts.isArrayBindingPattern(node.name)
      && node.name.elements.length >= 2
      && ts.isBindingElement(node.name.elements[1])
      && ts.isIdentifier(node.name.elements[1].name)
      && node.initializer
      && isNamedCall(node.initializer, reactUseState, "useState")
    ) {
      const setter = node.name.elements[1].name.text;
      let owner = node.parent;
      while (owner && !isFunctionScope(owner)) owner = owner.parent;
      if (owner?.body) {
        const findCalls = (candidate) => {
          if (candidate !== owner && isFunctionScope(candidate)) return;
          if (
            ts.isCallExpression(candidate)
            && ts.isIdentifier(candidate.expression)
            && candidate.expression.text === setter
          ) {
            results.push(sourceLine(sourceFile, candidate));
          }
          ts.forEachChild(candidate, findCalls);
        };
        ts.forEachChild(owner.body, findCalls);
      }
    }
    ts.forEachChild(node, visit);
  }
  visit(sourceFile);
  return results;
}

function perRowGraphWriteLines(sourceFile) {
  const results = [];
  const graphWrites = new Set([
    "upsertEntity",
    "upsertEntities",
    "replaceEntity",
    "removeEntity",
    "patchEntity",
    "publishFlintMutation",
  ]);
  const collectWrites = (body) => {
    const visit = (node) => {
      if (ts.isCallExpression(node)) {
        const name = ts.isIdentifier(node.expression)
          ? node.expression.text
          : ts.isPropertyAccessExpression(node.expression)
            ? node.expression.name.text
            : null;
        if (name && graphWrites.has(name)) results.push(sourceLine(sourceFile, node));
      }
      ts.forEachChild(node, visit);
    };
    visit(body);
  };
  const visit = (node) => {
    if (ts.isForStatement(node) || ts.isForOfStatement(node) || ts.isForInStatement(node)) {
      collectWrites(node.statement);
    } else if (
      ts.isCallExpression(node)
      && ts.isPropertyAccessExpression(node.expression)
      && (node.expression.name.text === "forEach" || node.expression.name.text === "map")
    ) {
      for (const argument of node.arguments) {
        if (ts.isArrowFunction(argument) || ts.isFunctionExpression(argument)) collectWrites(argument.body);
      }
    }
    ts.forEachChild(node, visit);
  };
  visit(sourceFile);
  return results;
}

function hasZustandStoreCreation(sourceFile, zustandCreate) {
  let found = false;
  const visit = (node) => {
    if (isNamedCall(node, zustandCreate, "create") || isNamedCall(node, zustandCreate, "createStore")) {
      found = true;
      return;
    }
    if (!found) ts.forEachChild(node, visit);
  };
  visit(sourceFile);
  return found;
}

function importsSingleLayer(specifier, layer) {
  if (!specifier.startsWith("@/") && !specifier.startsWith(".")) return false;
  const name = specifier.split("/").at(-1) ?? "";
  if (layer === "services") {
    return /(?:^|\/)services(?:\/|$)/.test(specifier)
      || /(?:^|\/)api(?:\/|$)/.test(specifier)
      || /-api$/.test(name);
  }
  if (layer === "stores") {
    return /(?:^|\/)stores(?:\/|$)/.test(specifier) || /-store$/.test(name);
  }
  if (layer === "hooks") {
    return /(?:^|\/)hooks(?:\/|$)/.test(specifier) || /^use(?:-|[A-Z])/.test(name);
  }
  return new RegExp(`(?:^|/)${layer}(?:/|$)`).test(specifier);
}

function importsLayer(modules, layers) {
  const candidates = layers.split("|");
  return modules.some((specifier) =>
    candidates.some((layer) => importsSingleLayer(specifier, layer)),
  );
}

function importTargetPath(sourcePath, specifier) {
  if (specifier.startsWith("@/")) {
    return resolve(scanRoot, specifier.slice(2));
  }
  if (specifier.startsWith(".")) {
    return resolve(dirname(sourcePath), specifier);
  }
  return null;
}

function architectureLayer(targetPath) {
  if (!targetPath) return null;
  const relativePath = relative(scanRoot, targetPath).replaceAll("\\", "/");
  if (relativePath === "app" || relativePath.startsWith("app/")) return "app";
  if (relativePath === "features" || relativePath.startsWith("features/")) return "feature";
  if (relativePath === "shared" || relativePath.startsWith("shared/")) return "shared";
  if (relativePath === "platform" || relativePath.startsWith("platform/")) return "platform";
  return null;
}

function featureName(targetPath) {
  if (!targetPath) return null;
  const parts = relative(scanRoot, targetPath).replaceAll("\\", "/").split("/");
  return parts[0] === "features" ? parts[1] ?? null : null;
}

function isPublicFeatureEntry(specifier, targetPath) {
  if (!targetPath || !specifier.startsWith("@/features/")) return false;
  const relativeParts = relative(scanRoot, targetPath).replaceAll("\\", "/").split("/");
  const namedRootEntry = relativeParts.length === 3
    && [".ts", ".tsx"].some((suffix) => {
      try {
        return statSync(`${targetPath}${suffix}`).isFile();
      } catch {
        return false;
      }
    });
  const directoryIndex = !/\.(?:ts|tsx)$/.test(targetPath)
    && (
      relativeParts.length === 2
      || (relativeParts.length === 3 && ["api", "model"].includes(relativeParts[2]))
    )
    && ["/index.ts", "/index.tsx"].some((suffix) => {
      try {
        return statSync(`${targetPath}${suffix}`).isFile();
      } catch {
        return false;
      }
    });
  return namedRootEntry || directoryIndex;
}

function isUpwardImport(sourceLayer, targetLayer) {
  if (sourceLayer === "platform") {
    return targetLayer === "feature" || targetLayer === "app";
  }
  if (sourceLayer === "shared") {
    return targetLayer === "feature" || targetLayer === "app";
  }
  return sourceLayer === "feature" && targetLayer === "app";
}

const violations = [];
for (const path of walk(scanRoot).filter(isProductionTs)) {
  const content = readFileSync(path, "utf8");
  const file = repoPath(path);
  const scanRelativePath = relative(scanRoot, path).replaceAll("\\", "/");
  const layer = layerFor(path);
  const modules = importedModules(content);
  const sourceFile = ts.createSourceFile(
    file,
    content,
    ts.ScriptTarget.Latest,
    true,
    path.endsWith(".tsx") ? ts.ScriptKind.TSX : ts.ScriptKind.TS,
  );
  const reactUseState = importBindings(sourceFile, "react", new Set(["useState"]));
  const zustandCreate = importBindings(sourceFile, "zustand", new Set(["create", "createStore"]));
  const sourceArchitectureLayer = architectureLayer(path);
  const addViolation = (rule, line = 1) => violations.push(`${file}:${line}|${rule}`);
  const directEntitySpecifier = modules.find((specifier) => (
    specifier === "@prometheus-ags/prometheus-entity-management"
    || specifier.startsWith("@prometheus-ags/prometheus-entity-management/")
    || specifier === "@prometheus-ags/entity-graph-core"
    || specifier.startsWith("@prometheus-ags/entity-graph-core/")
  ));

  if (
    layer !== "service" &&
    !infrastructureFetchExceptions.has(file) &&
    /(?:\bfetch|(?:window|globalThis)\.fetch)\s*\(/.test(content)
  ) {
    addViolation(`${layer}-direct-fetch`, lineNumberAt(content, firstMatchIndex(content, /(?:\bfetch|(?:window|globalThis)\.fetch)\s*\(/)));
  }
  if (layer === "component" && importsLayer(modules, "services")) {
    addViolation("component-service-import", lineNumberAt(content, firstMatchIndex(content, /\b(?:import|export)\b[^\n]*(?:services|api)/)));
  }
  if (layer === "component" && importsLayer(modules, "stores")) {
    addViolation("component-store-import", lineNumberAt(content, firstMatchIndex(content, /\b(?:import|export)\b[^\n]*store/)));
  }
  if (layer === "hook" && importsLayer(modules, "services")) {
    addViolation("hook-service-import", lineNumberAt(content, firstMatchIndex(content, /\b(?:import|export)\b[^\n]*(?:services|api)/)));
  }
  if (layer === "store" && importsLayer(modules, "components|hooks")) {
    addViolation("store-upward-import", lineNumberAt(content, firstMatchIndex(content, /\b(?:import|export)\b[^\n]*(?:components|hooks|use[A-Z])/)));
  }
  if (layer === "service" && importsLayer(modules, "components|hooks|stores")) {
    addViolation("service-upward-import", lineNumberAt(content, firstMatchIndex(content, /\b(?:import|export)\b[^\n]*(?:components|hooks|stores|store|use[A-Z])/)));
  }
  if (
    directEntitySpecifier
    && !scanRelativePath.startsWith("platform/entities/")
  ) {
    addViolation(
      "entity-facade-bypass",
      lineNumberAt(content, content.indexOf(directEntitySpecifier)),
    );
  }
  for (const line of renderBodySetterLines(sourceFile, reactUseState)) {
    addViolation("react-render-body-state-setter", line);
  }
  if (sourceArchitectureLayer === "feature") {
    for (const line of perRowGraphWriteLines(sourceFile)) {
      if (!legacyPerRowGraphWriteLocations.has(`${file}:${line}`)) {
        addViolation("feature-per-row-graph-write", line);
      }
    }
  }
  if (
    sourceArchitectureLayer === "feature"
    && hasZustandStoreCreation(sourceFile, zustandCreate)
    && /\b(?:ConfiguredProvider|ConfiguredModel|AgentSession|AgentSessionDraft)\b/.test(content)
  ) {
    addViolation(
      "duplicate-graph-owned-cache",
      lineNumberAt(content, firstMatchIndex(content, /\b(?:ConfiguredProvider|ConfiguredModel|AgentSession|AgentSessionDraft)\b/)),
    );
  }

  for (const specifier of modules) {
    const targetPath = importTargetPath(path, specifier);
    const targetArchitectureLayer = architectureLayer(targetPath);
    if (
      sourceArchitectureLayer &&
      targetArchitectureLayer &&
      isUpwardImport(sourceArchitectureLayer, targetArchitectureLayer)
    ) {
      addViolation(
        `${sourceArchitectureLayer}-upward-${targetArchitectureLayer}-import`,
        lineNumberAt(content, content.indexOf(specifier)),
      );
    }

    const sourceFeature = featureName(path);
    const targetFeature = featureName(targetPath);
    if (
      sourceFeature &&
      targetFeature &&
      sourceFeature !== targetFeature &&
      !isPublicFeatureEntry(specifier, targetPath)
    ) {
      addViolation("feature-cross-implementation-import", lineNumberAt(content, content.indexOf(specifier)));
    }
  }
}

const actual = [...new Set(violations)].sort();
if (process.argv.includes("--print")) {
  process.stdout.write(`${actual.join("\n")}\n`);
  process.exit(0);
}

if (actual.length) {
  console.error("Frontend boundary violations (move I/O and mutations to the owning layer):");
  for (const item of actual) console.error(`  + ${item}`);
  process.exit(1);
}

console.log("Frontend boundary gate passed (0 production violations).");
