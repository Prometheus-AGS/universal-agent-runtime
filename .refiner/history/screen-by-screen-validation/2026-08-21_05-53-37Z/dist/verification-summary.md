# Artifact refinement summary: screen-by-screen-validation

Content type: `direct:content`
Tested source: `f8e203b64462681597155f83660a9f35e03efa4c`
Profile: local Chromium, Vite, UAR `server-full`, deterministic stub LLM,
fresh embedded SurrealKV, browser PGlite. No result transfers beyond it.

## Constraint results

| Constraint | Result | Deterministic evidence |
|---|---|---|
| `screen-evidence-complete` | Pass | The retained `CI=1` run reports 32 passed scenarios; the product-screen feature contributes 20 primary-function scenarios and all 29 matrix evidence paths resolve to files in `f8e203b6`. |
| `interaction-strength` | Pass | Approvals denied a real pending run; Providers changed and restored the default route; Models selected Compare state; Agents selected the created agent in chat; Tools opened source/schema metadata; Knowledge used keyboard selection and passed the targeted axe rule; Memory, About, and Cost asserted live values. |
| `memory-and-fail-closed-controls` | Pass | MCP wrote and reread explicit `global`, `agent`, and `user` rows; exact assistant text rejects alternatives without including sibling artifacts; credential requests observed 200/401; distinct subjects in one tenant observed owner access and cross-user denial with explicit status and body-shape checks. |
| `bundle-process-source-integrity` | Pass | The transcript binds source `f8e203b6`, both frozen lock hashes, all recursive submodule pins, fresh ports, and `CI=1`; both fingerprints replay; 32 uniquely named H.264 videos have positive duration; 54 artifact entries replay; the finalized report contains the transcript/tree metadata; a one-byte tamper was rejected. |
| `scope-process-and-truth` | Pass | Strict OpenSpec exits 0, profile limits are explicit, and immutable decision `screen-validation-plan-projection-lag` records that canonical plan revision 7 followed the three operator-approved repairs. |

## Commands and observed results

```text
pnpm install --frozen-lockfile
pnpm -C frontend install --frozen-lockfile
pnpm test:bdd:prepare
```

Observed in the retained transcript: both supply-chain validators passed, both
lockfile resolution steps were skipped, both lock hashes remained unchanged,
the 13-asset frontend bundle validated, and the locked `server-full` binaries
were built from source.

```text
CI=1 pnpm exec playwright test -c tests/bdd/playwright.config.ts \
  tests/bdd/.features-gen/features/product-screen-validation.feature.spec.js \
  --grep 'Providers changes|Auth mints|MCP health'
```

Observed: exit 0, 3 passed.

```text
CI=1 pnpm exec playwright test -c tests/bdd/playwright.config.ts \
  tests/bdd/.features-gen/features/product-screen-validation.feature.spec.js \
  tests/bdd/.features-gen/features/cross-screen-security.feature.spec.js \
  tests/bdd/.features-gen/features/local-first-resilience.feature.spec.js \
  tests/bdd/.features-gen/features/chat-agent-switching.feature.spec.js \
  tests/bdd/.features-gen/features/chat-no-kb.feature.spec.js \
  tests/bdd/.features-gen/features/chat-skill-activation.feature.spec.js \
  tests/bdd/.features-gen/features/chat-kb-retrieval.feature.spec.js \
  tests/bdd/.features-gen/features/rag-citation.feature.spec.js
```

Observed: exit 0, 32 passed in 4.8 minutes with one worker and no server reuse.

```text
bash openspec/changes/screen-by-screen-validation/evidence/validate-bundle-f8e203b6.sh \
  docs/certifications/product-screens/f8e203b6
```

Observed: `MODULE_FINGERPRINT_MATCH`,
`GIT_TREE_FINGERPRINT_MATCH`, `ARTIFACTS_MATCH=54`,
`VIDEOS_H264_POSITIVE_DURATION=32`, `UNIQUE_VIDEO_HASHES=32`,
`SCREENSHOT_COUNT=20`, `DUPLICATE_ARTIFACT_PATHS=0`,
`CUCUMBER_SCENARIOS=32`, `CUCUMBER_FAILED=0`,
`REPORT_FINALIZED_MANIFEST_PRESENT`, and `TAMPER_CONTROL_REJECTED`.

```text
openspec validate screen-by-screen-validation --strict --no-interactive
```

Observed: exit 0, `Change 'screen-by-screen-validation' is valid`.

## Integrity receipts

- Root lock SHA-256: `645e3af883e8d62b74d13be20453c083431ed3cf2ef3ca20a5b1a84152273350`
- Nested frontend lock SHA-256: `43c00bbfe5b85e42c12a5fda74ab987750863794f00104a12ecd24a59f822593`
- Matrix SHA-256: `acc5d6e67fabb1a6f518330bd83dd405d43bd645e5ed6ac70682325af9e42025`
- Bundle manifest SHA-256: `2db50eb2265fcce7959b778b7d1321418711b345d4ef53c545f04d8373177fa1`
- Cucumber report SHA-256: `a183c93ed47f040040df9a45de0fce3d53337d0aa84bea4ef8a9e8e1cc59e66a`
- Certification transcript SHA-256: `63bda1918616cbe20d800491b0db61161912d128b8ff0ffd274eba0cb29031ac`
- Knowledge screen SHA-256: `a3acb69969b4777b0d7062acf930e7f36b5ecc1912d2fd088b5b8757a04e1d81`
- Approval projection SHA-256: `5d29355cd0f6d32b55b66cc67d9341dd30c37db71e386ea1de584e98a384bb93`
- Product-screen steps SHA-256: `8e020e565fdfef7f59fd19ae9adcbc438d629aabe4699a5679293d89c994ab50`
- Security steps SHA-256: `c6683135de2c8a899791cfae4c34916545c85ecf5f50244a7d91254d4630482f`
- Local-first steps SHA-256: `b6a457834b06a8d46977686a24c301aab2fcaa4aea64f0bc4d7783ad8666c03e`
- Feature-source fingerprint: `sha256:5e67038718410a0771ec2af3f03f5e24b890cb5653824e519d3abb8dfb843119`
- Committed-tree fingerprint: `12bd9b50e71b93e8bbad5b9a6bceac1eed9690f05875d983e86176bd5f92e8be`

## Uncomfortable fact

The required mint helper failed three ways on the observed Playwright output:
VP8 cannot be stream-copied into MP4, `ffmpeg` consumed bytes from the helper's
filename stream without `-nostdin`, and repeated `video.webm` basenames
collapsed to one output. The failed bundles were rejected and preserved under
`/tmp`. A temporary helper copy added H.264 transcoding, `-nostdin`, and
scenario-directory names; repository source was not patched. The final bundle
is accepted only because all 32 outputs were then codec-, duration-, uniqueness-,
hash-, and byte-validated.

The bundle report records the tested source commit. It cannot record the hash of
the evidence commit that contains it without self-reference, so the corrected
design requires a subsequent immutable receipt to name that first evidence
commit.
