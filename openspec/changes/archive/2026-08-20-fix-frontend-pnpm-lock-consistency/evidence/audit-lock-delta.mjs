import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";

const requireFromFrontend = createRequire(path.resolve("frontend/package.json"));
const YAML = requireFromFrontend("yaml");

const expectedRawHash =
  "0a7145d678283ac45de05ffd6773e1a3ba939ac915cd1c2673383c50242f472a";
const expectedCandidateHash =
  "43c00bbfe5b85e42c12a5fda74ab987750863794f00104a12ecd24a59f822593";
const sourceCommit = "1274039a28f0072bc0e6629a9dab327bdcd9417d";
const restoredSnapshotKeys = [
  "@typescript-eslint/project-service@8.64.0(supports-color@7.2.0)(typescript@5.9.3)",
  "chromatic@16.10.0(@chromatic-com/playwright@0.14.11(@playwright/test@1.62.1)(@testing-library/dom@10.4.1)(prettier@2.8.8)(react@19.2.8))",
  "storybook@10.2.13(@testing-library/dom@10.4.1)(prettier@2.8.8)(react@19.2.8)",
];
const entityImporterPrefix = "packages/prometheus-entity-management";
const sections = ["dependencies", "devDependencies", "optionalDependencies"];
const manifestSections = [
  "dependencies",
  "devDependencies",
  "optionalDependencies",
  "peerDependencies",
];

function argument(name) {
  const index = process.argv.indexOf(name);
  assert(index >= 0 && process.argv[index + 1], `missing ${name}`);
  return process.argv[index + 1];
}

function sha256(buffer) {
  return createHash("sha256").update(buffer).digest("hex");
}

function stable(value) {
  return JSON.stringify(value);
}

function changedKeys(beforeRows = {}, afterRows = {}) {
  return [...new Set([...Object.keys(beforeRows), ...Object.keys(afterRows)])]
    .filter((key) => stable(beforeRows[key]) !== stable(afterRows[key]))
    .sort();
}

function manifestPath(importer) {
  return importer === "."
    ? "frontend/package.json"
    : `frontend/${importer}/package.json`;
}

function directAnchor(importer, section, dependency) {
  return `${manifestPath(importer)}#${section}.${dependency}`;
}

function resolveManifestEdge(importer, lockSection, dependency, beforeRow, afterRow) {
  const manifest = JSON.parse(readFileSync(manifestPath(importer), "utf8"));
  const candidates = manifestSections
    .filter((section) => dependency in (manifest[section] ?? {}))
    .map((section) => ({ section, specifier: manifest[section][dependency] }));
  assert(candidates.length > 0, `manifest edge missing: ${importer}:${dependency}`);
  let selected;
  if (afterRow) {
    const exact = candidates.filter(
      (candidate) => candidate.specifier === afterRow.specifier,
    );
    assert(
      exact.length > 0,
      `candidate importer specifier has no manifest edge: ${importer}:${dependency}:${afterRow.specifier}`,
    );
    selected =
      exact.find((candidate) => candidate.section === lockSection) ?? exact[0];
  } else {
    selected =
      candidates.find((candidate) => candidate.section === "peerDependencies") ??
      candidates[0];
  }
  return {
    anchor: directAnchor(importer, selected.section, dependency),
    manifestSection: selected.section,
    manifestSpecifier: selected.specifier,
    candidateImporterSpecifier: afterRow?.specifier ?? null,
    projection:
      afterRow === undefined
        ? "removed-auto-peer-projection"
        : selected.section === lockSection
          ? "direct"
          : "lock-section-projection",
    previousImporterSpecifier: beforeRow?.specifier ?? null,
  };
}

function dependencyKey(name, value) {
  if (!value || String(value).startsWith("link:")) return null;
  return `${name}@${value}`;
}

function basePackageKey(snapshotKey) {
  return snapshotKey.split("(", 1)[0];
}

function changedDirectEdges(before, after) {
  const edges = [];
  for (const importer of new Set([
    ...Object.keys(before.importers ?? {}),
    ...Object.keys(after.importers ?? {}),
  ])) {
    for (const section of sections) {
      const oldDependencies = before.importers?.[importer]?.[section] ?? {};
      const newDependencies = after.importers?.[importer]?.[section] ?? {};
      for (const dependency of new Set([
        ...Object.keys(oldDependencies),
        ...Object.keys(newDependencies),
      ])) {
        const oldRow = oldDependencies[dependency];
        const newRow = newDependencies[dependency];
        if (oldRow?.specifier !== newRow?.specifier) {
          assert(
            importer === entityImporterPrefix ||
              importer.startsWith(`${entityImporterPrefix}/`),
            `specifier mutation escaped pinned submodule: ${importer}:${dependency}`,
          );
          const manifestEdge = resolveManifestEdge(
            importer,
            section,
            dependency,
            oldRow,
            newRow,
          );
          edges.push({
            importer,
            section,
            dependency,
            ...manifestEdge,
            before: oldRow ?? null,
            after: newRow ?? null,
          });
        }
      }
    }
  }
  return edges;
}

function reachableSnapshots(lock, edges, side) {
  const queue = [];
  const origins = new Map();
  for (const edge of edges) {
    const key = dependencyKey(edge.dependency, edge[side]?.version);
    if (!key || origins.has(key)) continue;
    origins.set(key, edge.anchor);
    queue.push(key);
  }
  while (queue.length > 0) {
    const key = queue.shift();
    const row = lock.snapshots?.[key];
    if (!row) continue;
    for (const section of ["dependencies", "optionalDependencies"]) {
      for (const [dependency, value] of Object.entries(row[section] ?? {})) {
        const next = dependencyKey(dependency, value);
        if (!next || origins.has(next)) continue;
        origins.set(next, origins.get(key));
        queue.push(next);
      }
    }
  }
  return origins;
}

function contextTokens(value) {
  const tokens = new Set();
  const expression = /(@[^@()\s/]+\/[^@()\s]+|[^@()\s/]+)@([^()\s]+)/g;
  for (const match of String(value ?? "").matchAll(expression)) {
    tokens.add(`${match[1]}@${match[2]}`);
  }
  return tokens;
}

function symmetricTokenChanges(beforeValue, afterValue) {
  const beforeTokens = contextTokens(beforeValue);
  const afterTokens = contextTokens(afterValue);
  return [
    ...[...beforeTokens]
      .filter((token) => !afterTokens.has(token))
      .map((token) => ({ direction: "removed", token })),
    ...[...afterTokens]
      .filter((token) => !beforeTokens.has(token))
      .map((token) => ({ direction: "added", token })),
  ].sort((left, right) =>
    `${left.direction}:${left.token}`.localeCompare(`${right.direction}:${right.token}`),
  );
}

function anchorsForToken(token, reachable) {
  return [...new Set(
    [...reachable.entries()]
      .filter(([snapshot]) => basePackageKey(snapshot) === token)
      .map(([, anchor]) => anchor),
  )].sort();
}

function classifyContextChange(
  beforeValue,
  afterValue,
  beforeReachable,
  afterReachable,
) {
  const changes = symmetricTokenChanges(beforeValue, afterValue).map((change) => {
    const reachable = change.direction === "added" ? afterReachable : beforeReachable;
    return {
      ...change,
      causalAnchors: anchorsForToken(change.token, reachable),
    };
  });
  return {
    changes,
    causalAnchors: [...new Set(changes.flatMap((change) => change.causalAnchors))].sort(),
    unclassifiedTokens: changes
      .filter((change) => change.causalAnchors.length === 0)
      .map((change) => `${change.direction}:${change.token}`),
  };
}

function closestOppositeKey(key, oppositeKeys) {
  const base = basePackageKey(key);
  const candidates = oppositeKeys.filter((candidate) => basePackageKey(candidate) === base);
  assert(candidates.length > 0, `peer-context key lacks opposite base: ${key}`);
  return candidates
    .map((candidate) => ({
      candidate,
      distance: symmetricTokenChanges(key, candidate).length,
    }))
    .sort((left, right) =>
      left.distance - right.distance || left.candidate.localeCompare(right.candidate),
    )[0].candidate;
}

function snapshotBodyContextChanges(beforeRow, afterRow) {
  const changes = [];
  for (const section of ["dependencies", "optionalDependencies"]) {
    const beforeDependencies = beforeRow?.[section] ?? {};
    const afterDependencies = afterRow?.[section] ?? {};
    for (const dependency of new Set([
      ...Object.keys(beforeDependencies),
      ...Object.keys(afterDependencies),
    ])) {
      const beforeValue = beforeDependencies[dependency];
      const afterValue = afterDependencies[dependency];
      if (beforeValue === afterValue) continue;
      changes.push({
        dependency,
        before: beforeValue ?? null,
        after: afterValue ?? null,
        beforeContext: beforeValue ? `${dependency}@${beforeValue}` : "",
        afterContext: afterValue ? `${dependency}@${afterValue}` : "",
      });
    }
  }
  const normalizedBefore = structuredClone(beforeRow ?? {});
  const normalizedAfter = structuredClone(afterRow ?? {});
  delete normalizedBefore.dependencies;
  delete normalizedBefore.optionalDependencies;
  delete normalizedAfter.dependencies;
  delete normalizedAfter.optionalDependencies;
  assert.equal(
    stable(normalizedBefore),
    stable(normalizedAfter),
    "common snapshot changed outside dependency contexts",
  );
  return changes;
}

function classify(before, after) {
  const directEdges = changedDirectEdges(before, after);
  const beforeReachable = reachableSnapshots(before, directEdges, "before");
  const afterReachable = reachableSnapshots(after, directEdges, "after");
  const records = [];
  const unclassified = [];

  for (const importer of new Set([
    ...Object.keys(before.importers ?? {}),
    ...Object.keys(after.importers ?? {}),
  ])) {
    for (const section of sections) {
      const oldDependencies = before.importers?.[importer]?.[section] ?? {};
      const newDependencies = after.importers?.[importer]?.[section] ?? {};
      for (const dependency of new Set([
        ...Object.keys(oldDependencies),
        ...Object.keys(newDependencies),
      ])) {
        const oldRow = oldDependencies[dependency];
        const newRow = newDependencies[dependency];
        const directEdge = directEdges.find(
          (edge) =>
            edge.importer === importer &&
            edge.section === section &&
            edge.dependency === dependency,
        );
        if (oldRow?.specifier !== newRow?.specifier) {
          assert(directEdge, `direct edge missing: ${importer}:${section}:${dependency}`);
          records.push({
            section: "importers",
            mutation: oldRow
              ? newRow
                ? "specifier-changed"
                : "specifier-removed"
              : "specifier-added",
            key: `${importer}:${section}:${dependency}`,
            before: oldRow?.specifier ?? null,
            after: newRow?.specifier ?? null,
            classification: "pinned-submodule-manifest-edge",
            causalAnchors: [directEdge.anchor],
            manifestSpecifier: directEdge.manifestSpecifier,
            candidateImporterSpecifier: directEdge.candidateImporterSpecifier,
            projection: directEdge.projection,
          });
        }
        if (oldRow && newRow && oldRow.version !== newRow.version) {
          const direct = oldRow?.specifier !== newRow?.specifier;
          const context = direct
            ? null
            : classifyContextChange(
                oldRow.version,
                newRow.version,
                beforeReachable,
                afterReachable,
              );
          if (context?.unclassifiedTokens.length) {
            unclassified.push({
              key: `${importer}:${section}:${dependency}`,
              tokens: context.unclassifiedTokens,
            });
          }
          records.push({
            section: "importers",
            mutation: "resolved-value-changed",
            key: `${importer}:${section}:${dependency}`,
            before: oldRow?.version ?? null,
            after: newRow?.version ?? null,
            classification: direct
              ? "pinned-submodule-manifest-edge"
              : "peer-context-propagation",
            causalAnchors: direct
              ? [directEdge.anchor]
              : context.causalAnchors,
            contextChanges: context?.changes,
          });
        }
      }
    }
  }

  const packageChanges = changedKeys(before.packages, after.packages);
  for (const key of packageChanges) {
    const added = !(key in (before.packages ?? {}));
    const removed = !(key in (after.packages ?? {}));
    assert(added || removed, `common package body changed: ${key}`);
    const reachable = added ? afterReachable : beforeReachable;
    const anchors = [...reachable.entries()]
      .filter(([snapshot]) => basePackageKey(snapshot) === key)
      .map(([, anchor]) => anchor);
    assert(anchors.length > 0, `package mutation lacks causal path: ${key}`);
    records.push({
      section: "packages",
      mutation: added ? "key-added" : "key-removed",
      key,
      classification: "transitive-from-pinned-submodule-edge",
      causalAnchors: [...new Set(anchors)].sort(),
    });
  }

  const snapshotChanges = changedKeys(before.snapshots, after.snapshots);
  for (const key of snapshotChanges) {
    const added = !(key in (before.snapshots ?? {}));
    const removed = !(key in (after.snapshots ?? {}));
    const reachable = added ? afterReachable : beforeReachable;
    const directAnchorValue = reachable.get(key);
    let context = null;
    let pairedWith = null;
    if (!directAnchorValue && (added || removed)) {
      pairedWith = closestOppositeKey(
        key,
        Object.keys(added ? before.snapshots ?? {} : after.snapshots ?? {}),
      );
      context = classifyContextChange(
        added ? pairedWith : key,
        added ? key : pairedWith,
        beforeReachable,
        afterReachable,
      );
    } else if (!added && !removed) {
      const bodyChanges = snapshotBodyContextChanges(
        before.snapshots[key],
        after.snapshots[key],
      );
      const combinedChanges = [];
      const combinedAnchors = [];
      const combinedUnclassified = [];
      for (const bodyChange of bodyChanges) {
        const classified = classifyContextChange(
          bodyChange.beforeContext,
          bodyChange.afterContext,
          beforeReachable,
          afterReachable,
        );
        combinedChanges.push(
          ...classified.changes.map((change) => ({
            dependency: bodyChange.dependency,
            ...change,
          })),
        );
        combinedAnchors.push(...classified.causalAnchors);
        combinedUnclassified.push(...classified.unclassifiedTokens);
      }
      context = {
        changes: combinedChanges,
        causalAnchors: [...new Set(combinedAnchors)].sort(),
        unclassifiedTokens: combinedUnclassified,
      };
    }
    if (context?.unclassifiedTokens.length) {
      unclassified.push({ key, tokens: context.unclassifiedTokens });
    }
    const causalAnchors = directAnchorValue
      ? [directAnchorValue]
      : context?.causalAnchors ?? [];
    records.push({
      section: "snapshots",
      mutation: added ? "key-added" : removed ? "key-removed" : "body-changed",
      key,
      classification: directAnchorValue
        ? "transitive-from-pinned-submodule-edge"
        : "peer-context-propagation",
      causalAnchors,
      pairedWith,
      contextChanges: context?.changes,
    });
  }

  for (const record of records) {
    if (record.causalAnchors.length === 0) {
      unclassified.push({
        key: record.key,
        tokens: ["mutation-has-no-causal-anchor"],
      });
    }
  }
  return { directEdges, records, unclassified };
}

const rawPath = argument("--raw");
const outputPath = argument("--output");
const beforeText = execFileSync("git", [
  "show",
  `${sourceCommit}:frontend/pnpm-lock.yaml`,
], {
  encoding: "utf8",
});
const rawBuffer = readFileSync(rawPath);
const candidateBuffer = readFileSync("frontend/pnpm-lock.yaml");
assert.equal(sha256(rawBuffer), expectedRawHash, "unexpected raw resolver lock");
assert.equal(
  sha256(candidateBuffer),
  expectedCandidateHash,
  "unexpected accepted candidate lock",
);

const before = YAML.parse(beforeText);
const raw = YAML.parse(rawBuffer.toString("utf8"));
const candidate = YAML.parse(candidateBuffer.toString("utf8"));
assert.equal(stable(raw.importers), stable(candidate.importers));
assert.equal(stable(raw.packages), stable(candidate.packages));
const rawCandidateSnapshotChanges = changedKeys(raw.snapshots, candidate.snapshots);
assert.deepEqual(rawCandidateSnapshotChanges, [...restoredSnapshotKeys].sort());
for (const key of restoredSnapshotKeys) {
  assert.equal(stable(candidate.snapshots[key]), stable(before.snapshots[key]));
}
const transformed = structuredClone(raw);
for (const key of restoredSnapshotKeys) {
  transformed.snapshots[key] = structuredClone(before.snapshots[key]);
}
assert.equal(stable(transformed), stable(candidate));

const { directEdges, records, unclassified } = classify(before, candidate);
const counts = records.reduce((result, record) => {
  const key = `${record.section}:${record.mutation}`;
  result[key] = (result[key] ?? 0) + 1;
  return result;
}, {});
assert.deepEqual(counts, {
  "importers:resolved-value-changed": 56,
  "importers:specifier-added": 21,
  "importers:specifier-changed": 20,
  "importers:specifier-removed": 3,
  "packages:key-added": 168,
  "packages:key-removed": 67,
  "snapshots:body-changed": 3,
  "snapshots:key-added": 237,
  "snapshots:key-removed": 118,
});

const result = {
  schemaVersion: 1,
  sourceCommit,
  rawResolverSha256: sha256(rawBuffer),
  acceptedCandidateSha256: sha256(candidateBuffer),
  rawToAcceptedTransformation: restoredSnapshotKeys.map((key) => ({
    operation: "restore-common-snapshot-body-from-head",
    key,
  })),
  directManifestEdges: directEdges.map((edge) => edge.anchor).sort(),
  directManifestEdgeDetails: directEdges
    .map((edge) => ({
      lockImporter: edge.importer,
      lockSection: edge.section,
      dependency: edge.dependency,
      anchor: edge.anchor,
      manifestSection: edge.manifestSection,
      manifestSpecifier: edge.manifestSpecifier,
      candidateImporterSpecifier: edge.candidateImporterSpecifier,
      projection: edge.projection,
      previousImporterSpecifier: edge.previousImporterSpecifier,
    }))
    .sort((left, right) => left.anchor.localeCompare(right.anchor)),
  counts,
  classifiedMutationCount: records.length,
  unclassifiedMutationCount: unclassified.length,
  unclassified,
  mutations: records,
};
writeFileSync(outputPath, `${JSON.stringify(result, null, 2)}\n`);
console.log(`RAW_TO_ACCEPTED_TRANSFORMATIONS=${restoredSnapshotKeys.length}`);
console.log(`DIRECT_MANIFEST_EDGES=${directEdges.length}`);
console.log(`CLASSIFIED_MUTATIONS=${records.length}`);
console.log(`UNCLASSIFIED_MUTATIONS=${unclassified.length}`);
console.log(`AUDIT_SHA256=${sha256(readFileSync(outputPath))}`);
assert.equal(unclassified.length, 0, "lock mutations remain unclassified");
console.log("LOCK_DELTA_CLASSIFICATION_PASS");
