# Verify release evidence

Release evidence is produced locally from a clean checkout by
`scripts/prepare-release-evidence-local.sh`. The output includes
`release-manifest.json`, `SHA256SUMS`, archive/binary/source/image SBOMs,
Sigstore bundles, SLSA provenance, and the exact product support matrix. The
manifest binds hashed local test and security-audit receipts to the release
source SHA. GitHub Actions are not a product-test, build, certification, or
signing runner; they are reserved for deployment execution and validation.

Run the local security gate first, then prepare evidence from already-built
platform archives and an already-published digest-addressed candidate image.
Both commands require a clean checkout and write source-bound receipts. Signing
and publication remain explicit operator-authorized release actions.

```bash
GH_TOKEN='<dependabot-read-token>' \
UAR_SECURITY_IMAGE='ghcr.io/prometheus-ags/universal-agent-runtime@sha256:<digest>' \
  scripts/security-audit-local.sh target/security-evidence

RELEASE_TAG=v1.0.0-rc.3 \
GA_TAG=v1.0.0 \
SUPERSEDED_GA_SHA=null \
REPOSITORY=Prometheus-AGS/universal-agent-runtime \
IMAGE_REFERENCE=ghcr.io/prometheus-ags/universal-agent-runtime \
IMAGE_DIGEST=sha256:<digest> \
COSIGN_CERTIFICATE_IDENTITY='<approved-local-builder-identity>' \
COSIGN_CERTIFICATE_OIDC_ISSUER='<approved-oidc-issuer>' \
TEST_EVIDENCE_FILE='<source-bound-test-receipt.json>' \
SECURITY_AUDIT_EVIDENCE_FILE='target/security-evidence/security-audit-evidence.json' \
BUILD_RECEIPT_FILE='<source-bound-build-receipt.json>' \
  scripts/prepare-release-evidence-local.sh target/platform-archives target/release-evidence
```

Download the supply checksum index first, authenticate it, then download only
the files it names and verify that exact set:

```bash
mkdir uar-release
gh release download v1.0.0 --repo Prometheus-AGS/universal-agent-runtime \
  --dir uar-release --pattern SHA256SUMS --pattern SHA256SUMS.sigstore.json
cd uar-release
cosign verify-blob --bundle SHA256SUMS.sigstore.json \
  --certificate-identity '<approved-local-builder-identity>' \
  --certificate-oidc-issuer '<approved-oidc-issuer>' \
  SHA256SUMS
while read -r _ asset; do
  gh release download v1.0.0 --repo Prometheus-AGS/universal-agent-runtime \
    --dir . --pattern "$asset"
done < SHA256SUMS
sha256sum --check SHA256SUMS
node verify-release.mjs release-manifest.json
```

`SHA256SUMS` covers every immutable supply-evidence file except itself and its
Sigstore bundle. The keyless signature bundle is the non-recursive trust root,
avoiding an impossible checksum/signature self-reference while ensuring that
added, removed, or changed supply files fail verification. The approved signer
identity and issuer are release policy inputs; never take them from an
unverified downloaded manifest. Candidate-certification outputs use a separate
signed index described below and never modify this supply set.

Verify an archive's keyless signature and SLSA provenance. Replace the archive
name as needed; keep the certificate identity and issuer constraints intact.

```bash
cosign verify-blob \
  --bundle universal-agent-runtime-linux-x64.tar.gz.sigstore.json \
  --certificate-identity '<approved-local-builder-identity>' \
  --certificate-oidc-issuer '<approved-oidc-issuer>' \
  universal-agent-runtime-linux-x64.tar.gz
```

Validate the machine-readable links and local digests with Node.js 22 or later.
The standalone verifier is itself a release asset, so this does not require a
repository checkout:

```bash
node verify-release.mjs release-manifest.json
```

Verify the signed multi-architecture image by digest (the digest is in the
manifest):

```bash
cosign verify \
  --certificate-identity '<approved-local-builder-identity>' \
  --certificate-oidc-issuer '<approved-oidc-issuer>' \
  ghcr.io/prometheus-ags/universal-agent-runtime@sha256:<digest>
```

CycloneDX and SPDX JSON SBOMs are supplied for each archive, each executable
inside an installable platform archive, the source tree, and the image. The
offline source archive has source-appropriate SBOMs but no invented runtime
binary trace. The manifest records each packaged executable path and SHA-256 so
the standalone verifier can extract it and prove which binary the signed
platform archive contains. It also records the source commit and Git tree,
Cargo lockfile, provider catalog and packaged-model digests, local builder
identity, immutable source-bound local test and audit receipts, and the
downloadable product support matrix used for the release decision.

## Candidate installation certification

Run the release-candidate driver against downloaded evidence on Linux. It
extracts the archive into an isolated directory, starts the packaged binary on
the configurable default port, exercises the OpenAI-compatible BossFang sidecar
seam against a deterministic provider, verifies surfaced provider failures and
recovery, measures parallel load and a streaming/reconnect soak, and performs a
cold backup/restore. Set `UAR_CANDIDATE_IMAGE` to the manifest's digest-bound
image reference to include the non-root container journey. A published immutable
candidate defaults to a 10,800-second soak; `UAR_SOAK_DURATION_SECONDS` may be
set to a short value only for local pre-candidate script validation.

```bash
UAR_CANDIDATE_IMAGE='ghcr.io/prometheus-ags/universal-agent-runtime@sha256:<digest>' \
UAR_PREVIOUS_ARTIFACT_DIR=previous-release \
UAR_REQUIRE_UPGRADE_JOURNEY=1 \
  scripts/certify-release-candidate.sh uar-release target/release-candidate-certification

scripts/package-candidate-certification-local.sh \
  target/release-candidate-certification uar-release target/candidate-certification-assets
```

`previous-release` must contain either a distinct prior release's Linux archive
and `release-manifest.json`, or a controlled source rebuild plus
`previous-identity.json` recording the public source ref, resolved commit, and
archive digest. The local certification procedure uses the latter when the
prior public release has no downloadable archive. Both archive identities are checked before
the previous binary and candidate are run against the same persistence path.
The procedure accepts `previous_ref` only when GitHub resolves it as a published,
non-draft release tag, records the peeled commit for annotated tags, and rejects
the candidate's own tag or source commit. The previous runtime writes a custom
durable setting through the supported Settings API; the candidate must read the
same canonical record. The retained pre-upgrade backup is then restored to a
separate rollback path, the previous runtime is restarted, and it must read the
same record again. The command
writes `results.json`, lifecycle/failure/load/soak/upgrade/non-root JSON,
installed version, model listing, chat response, and process logs. Validate it
with `node scripts/validate-candidate-certification.mjs
target/release-candidate-certification` and retain the directory with the
candidate evidence.

Before executing candidate bytes, the local certification procedure authenticates the
supply `SHA256SUMS`, downloads exactly its indexed files, verifies every digest,
and runs the bundled `verify-release.mjs`. Package completed local results with
`scripts/package-candidate-certification-local.sh`. Publication subsequently
attaches four separate files without modifying the supply checksum set:

- `candidate-certification-<tag>.tar.gz`
- `candidate-certification-manifest.json`
- `CANDIDATE_CERTIFICATION_SHA256SUMS`
- `CANDIDATE_CERTIFICATION_SHA256SUMS.sigstore.json`

The checksum index covers the archive and manifest and has its own local
Sigstore bundle. The manifest binds the archive to the candidate tag/source
SHA, the supply manifest and checksum-index digests, the local builder identity,
and a hashed `results.json` receipt. Consumers authenticate this second index,
verify it, validate the manifest, extract the archive, and run
`validate-candidate-certification.mjs` against the retained results.

## Immutable GA promotion

GA promotion reuses the certified candidate archives and OCI manifest digest;
it does not run a build. Local candidate tooling accepts only
`vX.Y.Z-rc.N`; the signed GA tag is never an archive-build input. The promotion
tool defaults to a read-only preflight, requires the RC and GA semantic versions
to match, and refuses a stale GA tag unless its exact current commit is
acknowledged. Execution additionally requires an explicit confirmation and a
configured Git tag-signing identity.

The candidate evidence already contains the immutable `promotion.json`; GA
copies it unchanged and MUST NOT generate extra release assets after
certification. Its contract is:

```json
{
  "schema_version": 1,
  "candidate": "v1.0.0-rc.3",
  "ga": "v1.0.0",
  "source_sha": "<40-hex-candidate-commit>",
  "image": "ghcr.io/prometheus-ags/universal-agent-runtime@sha256:<digest>",
  "rebuild": false,
  "superseded_ga_sha": "<audited-old-commit-or-null>"
}
```

`promotion.json` is listed in `release-manifest.json` and covered by the
attested supply `SHA256SUMS`. It intentionally does not contain the release
manifest or checksum-index digest, which would create a recursive digest. If
the recorded stale GA target changes before promotion, abort and produce new
authenticated promotion evidence; do not rewrite this file or append an
unindexed `superseded-release.json` at GA time.

```bash
UAR_RELEASE_SIGNING_IDENTITY='<approved-local-builder-identity>' \
UAR_RELEASE_SIGNING_OIDC_ISSUER='<approved-oidc-issuer>' \
  scripts/promote-release-candidate.sh v1.0.0-rc.3 v1.0.0

UAR_RELEASE_SIGNING_IDENTITY='<approved-local-builder-identity>' \
UAR_RELEASE_SIGNING_OIDC_ISSUER='<approved-oidc-issuer>' \
UAR_EXPECTED_EXISTING_GA_SHA=<audited-old-sha> \
UAR_CONFIRM_GA_PROMOTION='v1.0.0-rc.3->v1.0.0' \
UAR_CONFIRM_GA_RELEASE_REPLACEMENT=v1.0.0 \
  scripts/promote-release-candidate.sh v1.0.0-rc.3 v1.0.0 --execute
```

The pre-staged `promotion.json` binds the GA tag to the candidate source and
unchanged image digest. The promotion preflight independently authenticates and
validates the supply and candidate-certification checksum roots, extracts and
validates the certification results, and requires the current GA tag to equal
`superseded_ga_sha` exactly. Execution additionally requires
`UAR_EXPECTED_EXISTING_GA_SHA` to repeat that authenticated value before the
stale tag can be replaced. GA publishes both authenticated candidate sets
unchanged and creates no GA-only asset.

Promotion accepts exactly two authenticated tag pre-states: the recorded
`superseded_ga_sha` before work starts, or the certified candidate SHA when
resuming. Candidate and GA annotated tags must pass `git verify-tag`. A new GA
tag replaces the raw remote tag ref atomically with `--force-with-lease`; it is
never deleted first. The OCI alias and public release are idempotent: an exact
digest/asset match is retained, while a mismatched existing release additionally
requires `UAR_CONFIRM_GA_RELEASE_REPLACEMENT=<ga-tag>` before replacement. This
allows a rerun after a tag, image, or release partial failure to converge on the
same authenticated state.
