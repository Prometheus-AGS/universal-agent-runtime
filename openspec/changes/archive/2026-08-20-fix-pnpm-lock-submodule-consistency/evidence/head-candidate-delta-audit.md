# HEAD-to-candidate causal delta audit

Date: 2026-08-20

The initial operator candidate was not accepted on its frozen result alone.
Direct comparison against `HEAD:pnpm-lock.yaml` found two noncausal changes and
restored them before the clean full-install proof.

Command:

```bash
node - <<'NODE'
const { execFileSync } = require('child_process');
const YAML = require('yaml');
const fs = require('fs');
const head = YAML.parse(execFileSync(
  'git', ['show', 'HEAD:pnpm-lock.yaml'],
  { encoding: 'utf8', maxBuffer: 20e6 },
));
const candidate = YAML.parse(fs.readFileSync('pnpm-lock.yaml', 'utf8'));
let specifierChanges = [];
let valueOnly = [];
for (const importer of new Set([
  ...Object.keys(head.importers),
  ...Object.keys(candidate.importers),
])) {
  for (const section of ['dependencies', 'devDependencies', 'optionalDependencies']) {
    const before = head.importers?.[importer]?.[section] || {};
    const after = candidate.importers?.[importer]?.[section] || {};
    for (const dependency of new Set([...Object.keys(before), ...Object.keys(after)])) {
      const beforeSpecifier = before[dependency]?.specifier;
      const afterSpecifier = after[dependency]?.specifier;
      if (beforeSpecifier !== afterSpecifier) {
        specifierChanges.push(`${importer}:${section}:${dependency}`);
      } else if (JSON.stringify(before[dependency]) !== JSON.stringify(after[dependency])) {
        valueOnly.push(`${importer}:${section}:${dependency}`);
      }
    }
  }
}
console.log(`IMPORTER_SPECIFIER_CHANGES=${specifierChanges.length}`);
console.log(`SPECIFIER_CHANGES_OUTSIDE_SUBMODULE=${specifierChanges.filter(
  value => !value.startsWith('frontend/packages/prometheus-entity-management'),
).length}`);
console.log(`UNCHANGED_SPECIFIER_VALUE_CHANGES=${valueOnly.length}`);
console.log(`UNCHANGED_SPECIFIER_VALUES_OUTSIDE_SUBMODULE=${valueOnly.filter(
  value => !value.startsWith('frontend/packages/prometheus-entity-management'),
).length}`);
const commonPackages = Object.keys(head.packages).filter(
  key => key in candidate.packages
    && JSON.stringify(head.packages[key]) !== JSON.stringify(candidate.packages[key]),
);
const commonSnapshots = Object.keys(head.snapshots).filter(
  key => key in candidate.snapshots
    && JSON.stringify(head.snapshots[key]) !== JSON.stringify(candidate.snapshots[key]),
);
console.log(`COMMON_PACKAGE_MUTATIONS=${commonPackages.length}`);
console.log(`COMMON_SNAPSHOT_MUTATIONS=${commonSnapshots.length}`);
for (const key of commonSnapshots) console.log(`CAUSAL_COMMON_SNAPSHOT=${key}`);
const yWebrtc = candidate.snapshots[
  'y-webrtc@10.3.0(supports-color@11.0.0)(yjs@13.6.31)'
];
console.log(`Y_WEBRTC_WS=${yWebrtc.optionalDependencies.ws}`);
console.log(`SYNC_DIRECT_WS=${candidate.importers[
  'frontend/packages/prometheus-entity-management/packages/entity-graph-sync'
].devDependencies.ws.version}`);
console.log(`CONFIG_ARRAY_SUPPORTS_11_MINIMATCH=${candidate.snapshots[
  '@eslint/config-array@0.23.5(supports-color@11.0.0)'
].dependencies.minimatch}`);
console.log(`CONFIG_ARRAY_SUPPORTS_10_MINIMATCH=${candidate.snapshots[
  '@eslint/config-array@0.23.5(supports-color@10.2.2)'
].dependencies.minimatch}`);
NODE
```

Observed exit: `0`

Observed output:

```text
IMPORTER_SPECIFIER_CHANGES=102
SPECIFIER_CHANGES_OUTSIDE_SUBMODULE=0
UNCHANGED_SPECIFIER_VALUE_CHANGES=20
UNCHANGED_SPECIFIER_VALUES_OUTSIDE_SUBMODULE=18
COMMON_PACKAGE_MUTATIONS=0
COMMON_SNAPSHOT_MUTATIONS=3
CAUSAL_COMMON_SNAPSHOT=@storybook/addon-vitest@10.5.6(@vitest/browser-playwright@4.1.10)(@vitest/browser@4.1.10)(@vitest/runner@4.1.10)(react@19.2.8)(storybook@10.5.6(@types/react@19.2.17)(prettier@3.9.6)(react@19.2.8))(vitest@4.1.10)
CAUSAL_COMMON_SNAPSHOT=@vitest/coverage-v8@4.1.10(@vitest/browser@4.1.10)(vitest@4.1.10)
CAUSAL_COMMON_SNAPSHOT=@vitest/ui@4.1.10(vitest@4.1.10)
Y_WEBRTC_WS=8.21.0
SYNC_DIRECT_WS=8.21.1
CONFIG_ARRAY_SUPPORTS_11_MINIMATCH=10.2.5
CONFIG_ARRAY_SUPPORTS_10_MINIMATCH=10.2.6
```

The 102 specifier changes are confined to the advanced entity-management
submodule. The 20 value-only changes are peer-context propagation from its new
`tsx`, jsdom 30, Node/React type, TypeScript, and Immer graph. The remaining
three common snapshot mutations only retarget those causal peer contexts.

The restored HEAD edges are deliberately split from new direct requirements:
the unchanged supports-color-11 config-array stays on minimatch 10.2.5 while
the new supports-color-10 context uses 10.2.6; unchanged `y-webrtc` stays on
`ws` 8.21.0 while `entity-graph-sync` directly uses its pinned `ws` 8.21.1.
