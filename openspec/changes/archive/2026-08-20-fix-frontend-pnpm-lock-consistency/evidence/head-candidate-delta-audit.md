# HEAD-to-candidate causal delta audit

Date: 2026-08-20

The executable audit is
`evidence/audit-lock-delta.mjs`; the complete per-mutation result is
`evidence/lock-delta-classification.json`. It maps all 693 importer-field,
package-key, snapshot-key, and common-body mutations to an exact pinned
submodule manifest edge or to a peer-context propagation whose changed tokens
have dependency-graph paths to specific changed edges. Any missing token path,
empty anchor, escaped specifier mutation, unexpected count, common package-body
mutation, nonexistent manifest selector, candidate specifier mismatch, or extra
raw-to-accepted transformation exits non-zero.

Summary command:

```bash
set -euo pipefail
node <<'NODE'
const fs = require('fs');
const cp = require('child_process');
const YAML = require('/Users/gqadonis/node_modules/yaml');
const before = YAML.parse(cp.execFileSync('git', ['show', '1274039a28f0072bc0e6629a9dab327bdcd9417d:frontend/pnpm-lock.yaml'], { encoding: 'utf8' }));
const after = YAML.parse(fs.readFileSync('frontend/pnpm-lock.yaml', 'utf8'));
let specifierAdded = 0, specifierRemoved = 0, specifierChanged = 0, versionChanged = 0;
for (const importer of new Set([...Object.keys(before.importers ?? {}), ...Object.keys(after.importers ?? {})])) {
  for (const section of ['dependencies', 'devDependencies', 'optionalDependencies']) {
    const oldDeps = before.importers?.[importer]?.[section] ?? {};
    const newDeps = after.importers?.[importer]?.[section] ?? {};
    for (const name of new Set([...Object.keys(oldDeps), ...Object.keys(newDeps)])) {
      if (!(name in oldDeps)) specifierAdded += 1;
      else if (!(name in newDeps)) specifierRemoved += 1;
      else {
        if (oldDeps[name]?.specifier !== newDeps[name]?.specifier) specifierChanged += 1;
        if (oldDeps[name]?.version !== newDeps[name]?.version) versionChanged += 1;
      }
    }
  }
}
function compareSection(section) {
  const oldRows = before[section] ?? {}, newRows = after[section] ?? {};
  return {
    added: Object.keys(newRows).filter((key) => !(key in oldRows)).length,
    removed: Object.keys(oldRows).filter((key) => !(key in newRows)).length,
    changedKeys: Object.keys(oldRows).filter((key) => key in newRows && JSON.stringify(oldRows[key]) !== JSON.stringify(newRows[key])),
  };
}
const packages = compareSection('packages');
const snapshots = compareSection('snapshots');
console.log(`IMPORTER_SPECIFIER_ADDED=${specifierAdded}`);
console.log(`IMPORTER_SPECIFIER_REMOVED=${specifierRemoved}`);
console.log(`IMPORTER_SPECIFIER_CHANGED=${specifierChanged}`);
console.log(`IMPORTER_VERSION_CHANGED=${versionChanged}`);
console.log(`PACKAGE_KEYS_ADDED=${packages.added}`);
console.log(`PACKAGE_KEYS_REMOVED=${packages.removed}`);
console.log(`COMMON_PACKAGE_BODIES_CHANGED=${packages.changedKeys.length}`);
console.log(`SNAPSHOT_KEYS_ADDED=${snapshots.added}`);
console.log(`SNAPSHOT_KEYS_REMOVED=${snapshots.removed}`);
console.log(`COMMON_SNAPSHOT_BODIES_CHANGED=${snapshots.changedKeys.length}`);
for (const key of snapshots.changedKeys) console.log(`CAUSAL_COMMON_SNAPSHOT=${key}`);
if (specifierAdded !== 21 || specifierRemoved !== 3 || specifierChanged !== 20 || versionChanged !== 56) process.exit(2);
if (packages.added !== 168 || packages.removed !== 67 || packages.changedKeys.length !== 0) process.exit(3);
if (snapshots.added !== 237 || snapshots.removed !== 118 || snapshots.changedKeys.length !== 3) process.exit(4);
console.log('HEAD_TO_CANDIDATE_DELTA_PASS');
NODE
```

Observed exit: `0`

Observed output:

```text
IMPORTER_SPECIFIER_ADDED=21
IMPORTER_SPECIFIER_REMOVED=3
IMPORTER_SPECIFIER_CHANGED=20
IMPORTER_VERSION_CHANGED=56
PACKAGE_KEYS_ADDED=168
PACKAGE_KEYS_REMOVED=67
COMMON_PACKAGE_BODIES_CHANGED=0
SNAPSHOT_KEYS_ADDED=237
SNAPSHOT_KEYS_REMOVED=118
COMMON_SNAPSHOT_BODIES_CHANGED=3
CAUSAL_COMMON_SNAPSHOT=@storybook/addon-vitest@10.5.7(@vitest/browser-playwright@4.1.10)(@vitest/browser@4.1.10)(@vitest/runner@4.1.10)(react@19.2.8)(storybook@10.5.7(@types/react@19.2.15)(prettier@2.8.8)(react@19.2.8))(vitest@4.1.10)
CAUSAL_COMMON_SNAPSHOT=@vitest/coverage-v8@4.1.10(@vitest/browser@4.1.10)(vitest@4.1.10)
CAUSAL_COMMON_SNAPSHOT=@vitest/ui@4.1.10(vitest@4.1.10)
HEAD_TO_CANDIDATE_DELTA_PASS
```

The 44 importer specifier additions/removals/changes are machine-asserted to be
confined to the pinned entity-management importers. The 56 resolved-value
changes include causal peer contexts propagated into the other seven importers
by the new submodule `tsx`, `supports-color`, jsdom 30, React type, TypeScript,
and Immer graph. The machine-readable file names the causal manifest anchors
for every one rather than relying on these aggregate counts.

The structural peer audit pairs added/removed snapshot contexts by base package,
derives their symmetric changed tokens, and follows each token through the
before or after graph to an exact changed edge. The corrected output contains
131 peer-context records, zero blanket all-edge anchors, and zero empty anchors.
For example, the `yup@1.7.1` context on `@hookform/resolvers` traces to
`frontend/packages/prometheus-entity-management/package.json#devDependencies.@cucumber/cucumber`;
it is not attributed only to the direct `ajv` edge.

Focused causal-anchor replay:

```bash
set -euo pipefail
node <<'NODE'
const audit = require('./openspec/changes/fix-frontend-pnpm-lock-consistency/evidence/lock-delta-classification.json');
const peerRecords = audit.mutations.filter((row) => row.classification === 'peer-context-propagation');
const blanketRecords = peerRecords.filter((row) => row.causalAnchors.length === audit.directManifestEdges.length);
const emptyRecords = peerRecords.filter((row) => row.causalAnchors.length === 0);
const yupRecords = audit.mutations.filter((row) => JSON.stringify(row.contextChanges ?? []).includes('yup@1.7.1'));
console.log(`PEER_RECORDS=${peerRecords.length}`);
console.log(`ALL_EDGE_RECORDS=${blanketRecords.length}`);
console.log(`EMPTY_ANCHOR_RECORDS=${emptyRecords.length}`);
console.log(`YUP_RECORDS=${yupRecords.length}`);
if (blanketRecords.length || emptyRecords.length || !yupRecords.length) process.exit(2);
if (!yupRecords.some((row) => (row.contextChanges ?? []).some((change) =>
  change.token === 'yup@1.7.1' &&
  change.causalAnchors.some((anchor) => anchor.endsWith('#devDependencies.@cucumber/cucumber'))
))) process.exit(3);
console.log('SPECIFIC_CAUSAL_ANCHORS_PASS');
NODE
```

Observed exit: `0`

```text
PEER_RECORDS=131
ALL_EDGE_RECORDS=0
EMPTY_ANCHOR_RECORDS=0
YUP_RECORDS=3
SPECIFIC_CAUSAL_ANCHORS_PASS
```

The lock importer section is not assumed to be the manifest section. pnpm can
project auto-installed peers under importer `dependencies`; the auditor resolves
each edge against the actual `package.json` sections. The three removed React
workspace projections resolve to `peerDependencies`, while their replacement
development edges resolve separately to `devDependencies`.

Manifest-edge replay:

```bash
set -euo pipefail
node <<'NODE'
const fs = require('node:fs');
const audit = require('./openspec/changes/fix-frontend-pnpm-lock-consistency/evidence/lock-delta-classification.json');
let missing = 0;
let mismatch = 0;
for (const edge of audit.directManifestEdgeDetails) {
  const [file, selector] = edge.anchor.split('#');
  const [section, ...nameParts] = selector.split('.');
  const dependency = nameParts.join('.');
  const manifest = JSON.parse(fs.readFileSync(file, 'utf8'));
  if (!(dependency in (manifest[section] ?? {}))) missing += 1;
  else if (edge.candidateImporterSpecifier !== null && manifest[section][dependency] !== edge.candidateImporterSpecifier) mismatch += 1;
}
console.log(`DIRECT_MANIFEST_EDGES=${audit.directManifestEdgeDetails.length}`);
console.log(`MISSING_ANCHOR_PATHS=${missing}`);
console.log(`PRESENT_ANCHOR_VALUE_MISMATCHES=${mismatch}`);
if (missing || mismatch) process.exit(2);
console.log('DIRECT_MANIFEST_EDGE_VALIDATION_PASS');
NODE
```

Observed exit: `0`

```text
DIRECT_MANIFEST_EDGES=44
MISSING_ANCHOR_PATHS=0
PRESENT_ANCHOR_VALUE_MISMATCHES=0
DIRECT_MANIFEST_EDGE_VALIDATION_PASS
```

The full regeneration originally changed six common snapshot bodies. The
accepted candidate restores HEAD's project-service 8.64.0 internal edges,
Chromatic 16.10.0 semver edge, and Storybook 10.2.13 semver/ws edges. Frozen
metadata and empty-tree installation both pass after those restorations. The
three remaining common-body mutations only retarget the causal Vitest/Vite
peer contexts shown above.

Limit: this is a causal lock-delta audit, not a dependency freshness, security,
or cross-platform verdict.
