# Verify release evidence

The `Supply-chain release evidence` workflow attaches `release-manifest.json`,
`SHA256SUMS`, SBOMs, Sigstore bundles and SLSA provenance to an existing GitHub
release. Verification trusts the GitHub OIDC identity recorded in Sigstore and
the GitHub artifact-attestation service, rather than an unsigned checksum page.

Download all assets for a tag, then verify archive checksums:

```bash
gh release download v1.0.0 --repo Prometheus-AGS/universal-agent-runtime --dir uar-release
cd uar-release
sha256sum --check SHA256SUMS
```

Verify an archive's keyless signature and SLSA provenance. Replace the archive
name as needed; keep the certificate identity and issuer constraints intact.

```bash
cosign verify-blob \
  --bundle universal-agent-runtime-linux-x64.tar.gz.sigstore.json \
  --certificate-identity-regexp '^https://github.com/Prometheus-AGS/universal-agent-runtime/.github/workflows/supply-chain.yml@refs/heads/' \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  universal-agent-runtime-linux-x64.tar.gz

gh attestation verify universal-agent-runtime-linux-x64.tar.gz \
  --repo Prometheus-AGS/universal-agent-runtime \
  --signer-workflow Prometheus-AGS/universal-agent-runtime/.github/workflows/supply-chain.yml
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
  --certificate-identity-regexp '^https://github.com/Prometheus-AGS/universal-agent-runtime/.github/workflows/supply-chain.yml@refs/heads/' \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  ghcr.io/prometheus-ags/universal-agent-runtime@sha256:<digest>
```

CycloneDX and SPDX JSON SBOMs are supplied for each archive, the source tree,
and the image. The manifest also records the source commit and Git tree, the
Cargo lockfile, provider catalog and packaged-model digests, workflow identity,
test run, security-audit workflow, and product support matrix used for the
release decision.

## Candidate installation certification

Run the release-candidate driver against downloaded evidence on Linux. It
extracts the archive into an isolated directory, starts the packaged binary on
the configurable default port, exercises the OpenAI-compatible BossFang sidecar
seam against a deterministic provider, and performs a cold backup/restore. Set
`UAR_CANDIDATE_IMAGE` to the manifest's digest-bound image reference to include
the non-root container journey.

```bash
UAR_CANDIDATE_IMAGE='ghcr.io/prometheus-ags/universal-agent-runtime@sha256:<digest>' \
  scripts/certify-release-candidate.sh uar-release candidate-evidence
```

The command writes `results.json`, installed version, model listing, chat
response, and process logs. Retain that directory with the candidate evidence.

## Immutable GA promotion

GA promotion reuses the certified candidate archives and OCI manifest digest;
it does not run a build. The promotion tool defaults to a read-only preflight
and refuses a stale GA tag unless its exact current commit is acknowledged.
Execution additionally requires an explicit confirmation and a configured Git
tag-signing identity.

```bash
scripts/promote-release-candidate.sh v1.0.0-rc.2 v1.0.0

UAR_EXPECTED_EXISTING_GA_SHA=<audited-old-sha> \
UAR_CONFIRM_GA_PROMOTION='v1.0.0-rc.2->v1.0.0' \
  scripts/promote-release-candidate.sh v1.0.0-rc.2 v1.0.0 --execute
```

The resulting `promotion.json` binds the GA tag to the candidate source,
candidate manifest digest, and unchanged image digest.
