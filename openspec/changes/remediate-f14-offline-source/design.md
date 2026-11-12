# Design

Keep the owning repositories authoritative for their lockfiles. Start from the exact recorded Git pins. Reuse the existing patches where compatible, resolve the remaining advisories, and propagate reviewed leaf commits through parent pins. Do not advance a branch tip or replace an archive entry to conceal a finding. Account for every saved finding, including unknown severity. Remove an obsolete dependency input only if its non-use is established and normal consumer behavior is preserved.

Freeze the final clean source and all recursive pins after implementation. Existing release scripts produce the full archive and run recursive OSV. The archive manifest and digest and a no-network build from an extraction accompany the scan result. Report failed and unverified observations honestly. Stop the first failing check and preserve its receipt. Signing, Grype and issue disposition remain in the parent release workflow.
