# Evidence commit receipt — `screen-by-screen-validation`

Tested source commit:
`f8e203b64462681597155f83660a9f35e03efa4c`

First commit containing the finalized certification bundle:
`2fd96b07e396fe1e988232864a9eefef824b3aa3`

The evidence commit's parent is the tested source commit. The bundle manifest
is absent from that parent and present in the evidence commit.

## Command

```bash
bash openspec/changes/screen-by-screen-validation/evidence/validate-evidence-commit-2fd96b07.sh
```

The final command is the required failing control and was run with error capture.

## Observed output

```text
EVIDENCE_COMMIT=2fd96b07e396fe1e988232864a9eefef824b3aa3
EVIDENCE_PARENT=f8e203b64462681597155f83660a9f35e03efa4c
EVIDENCE_CONTAINS_BUNDLE=PASS
SOURCE_CONTAINS_BUNDLE_EXIT=128
SOURCE_BUNDLE_ABSENCE_CONTROL=PASS
SOURCE_BUNDLE_ABSENCE_STDERR=fatal: path 'docs/certifications/product-screens/f8e203b6/manifest.json' exists on disk, but not in 'f8e203b64462681597155f83660a9f35e03efa4c'
```

This receipt binds only the local Chromium `server-full` evidence bundle to
its tested source and first evidence commit. It transfers no product result to
another browser, profile, provider, deployment, or platform.

## Uncomfortable fact

The evidence commit cannot name itself inside a file it already contains. This
subsequent receipt is the immutable, non-self-referential binding required by
the corrected design.
