## ADDED Requirements

### Requirement: Architectural Decision Record Accuracy

An architectural decision record's factual claims about pinned dependency state SHALL be re-verified against live manifest state whenever a related dependency change lands, rather than assumed correct from a prior reading.

#### Scenario: A decision record's claim no longer matches the manifest

- **Given** an architectural decision document makes a specific claim
  about how a dependency is pinned (branch, tag, or commit)
- **When** live `Cargo.toml` (or equivalent manifest) state is checked
  directly and found to differ from the claim
- **Then** the decision record MUST be corrected to match live state, and
  the correction MUST be disclosed rather than silently edited without a
  trace of what was wrong

#### Scenario: A parallel record also drifted

- **Given** correcting one document's claim reveals that a parallel
  document tracking the same fact (e.g. a "current pinned versions"
  table) has also drifted from live manifest state
- **When** the correction is made
- **Then** all drifted entries MUST be corrected in the same change, not
  just the one that prompted the investigation
