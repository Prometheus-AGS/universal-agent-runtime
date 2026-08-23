# Validation plan — `uar-documentation-publication-contract`

1. Check proposal, design, delta specs, and tasks for consistent ownership,
   explicit exclusions, and truthful phase boundaries.
2. Run all isolated positive and negative controls for source classification,
   route coverage, provenance, sanitization, child-validator propagation, and
   Pages-publisher cardinality.
3. Run the composed validator against the repository and require the known
   missing-route and competing-publisher failures to remain visible.
4. Run the documentation-truth and GitHub Actions policy validators separately
   and retain their observed results.
5. Validate the OpenSpec change strictly and preserve the prior portal-change
   artifact hashes.
6. Audit the diff for runtime, React application, provider/model, realtime,
   vendored, and raw `.prometheus` changes.
7. Persist the review report, schema-valid PMPO state, evidence limits, and
   convergence decision without claiming publication readiness.
