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

Validate the machine-readable links and local digests with Node.js 22 or later:

```bash
node scripts/validate-release-manifest.mjs uar-release/release-manifest.json
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
and the image. The manifest also records the source commit, workflow identity,
test run, security-audit workflow, and product support matrix used for the
release decision.
