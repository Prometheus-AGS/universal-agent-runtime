#!/usr/bin/env node

import { readFileSync, readdirSync, statSync } from "node:fs";
import { relative, resolve } from "node:path";
import ts from "typescript";

const repoRoot = resolve(import.meta.dirname, "..");
const rootIndex = process.argv.indexOf("--root");
const scanRoot = rootIndex >= 0 ? resolve(repoRoot, process.argv[rootIndex + 1]) : resolve(repoRoot, "frontend");
const candidates = [];

function visit(path) {
  const stat = statSync(path);
  if (stat.isDirectory()) {
    if (["node_modules", "build", "coverage", "dist", "test-results"].includes(path.split("/").at(-1))) return;
    for (const entry of readdirSync(path)) visit(resolve(path, entry));
    return;
  }
  if (/\.(?:stories\.[cm]?[jt]sx?|storybook\.[cm]?[jt]s|[cm]?[jt]s)$/.test(path)) candidates.push(path);
}

visit(scanRoot);

const suppressions = [];

function propertyName(name) {
  if (ts.isIdentifier(name) || ts.isStringLiteralLike(name)) return name.text;
  if (
    ts.isComputedPropertyName(name)
    && (ts.isStringLiteralLike(name.expression) || ts.isNoSubstitutionTemplateLiteral(name.expression))
  ) {
    return name.expression.text;
  }
  return null;
}

function isOffLiteral(node) {
  return ts.isStringLiteralLike(node) && node.text === "off";
}

function isNamedAccess(node, expectedName) {
  if (ts.isPropertyAccessExpression(node)) return node.name.text === expectedName;
  if (ts.isElementAccessExpression(node) && node.argumentExpression) {
    return ts.isStringLiteralLike(node.argumentExpression) && node.argumentExpression.text === expectedName;
  }
  return false;
}

function isA11yTestAccess(node) {
  if (!isNamedAccess(node, "test")) return false;
  return isNamedAccess(node.expression, "a11y");
}

for (const path of candidates) {
  const source = readFileSync(path, "utf8");
  const sourceFile = ts.createSourceFile(
    path,
    source,
    ts.ScriptTarget.Latest,
    true,
    ts.getScriptKindFromFileName(path),
  );

  const report = (node) => {
    const line = sourceFile.getLineAndCharacterOfPosition(node.getStart(sourceFile)).line + 1;
    suppressions.push(`${relative(repoRoot, path)}:${line}`);
  };

  const visitNode = (node) => {
    if (
      ts.isPropertyAssignment(node)
      && propertyName(node.name) === "a11y"
      && ts.isObjectLiteralExpression(node.initializer)
    ) {
      const disabledTest = node.initializer.properties.find(
        (property) => ts.isPropertyAssignment(property)
          && propertyName(property.name) === "test"
          && isOffLiteral(property.initializer),
      );
      if (disabledTest) report(disabledTest);
    }

    if (
      ts.isBinaryExpression(node)
      && node.operatorToken.kind === ts.SyntaxKind.EqualsToken
      && isA11yTestAccess(node.left)
      && isOffLiteral(node.right)
    ) {
      report(node);
    }

    ts.forEachChild(node, visitNode);
  };

  visitNode(sourceFile);

  if (sourceFile.parseDiagnostics.length > 0) {
    const diagnostic = sourceFile.parseDiagnostics[0];
    const line = sourceFile.getLineAndCharacterOfPosition(diagnostic.start ?? 0).line + 1;
    console.error(`Storybook accessibility scan could not parse ${relative(repoRoot, path)}:${line}`);
    process.exit(1);
  }
}

if (suppressions.length > 0) {
  console.error("Storybook accessibility suppressions are forbidden:");
  for (const suppression of suppressions) console.error(`  ${suppression}`);
  process.exit(1);
}

console.log(`Storybook accessibility suppression gate passed (${candidates.length} story/config source files, 0 disabled checks).`);
