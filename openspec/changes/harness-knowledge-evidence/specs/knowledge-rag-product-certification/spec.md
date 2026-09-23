## ADDED Requirements

### Requirement: Knowledge certification exercises the governed answer path
Local knowledge certification SHALL preserve the existing create/index/search/ground/remove journey and test the production governed retrieval-to-answer path, including hybrid retrieval and embedding support. It SHALL capture source-version/claim links and verification decisions, not only lexical answer matches. The dataset and pass/fail criteria SHALL be frozen before implementation.

#### Scenario: Evidence fixtures
- **WHEN** fixtures contain supported, missing, conflicting, stale and cross-tenant evidence
- **THEN** verdicts and source links match the frozen oracle, unauthorized content is absent from captured model requests and user-visible audit, and unavailable evidence is not counted as passing

#### Scenario: Revocation and injection across restart
- **WHEN** a retrieval-bearing checkpoint resumes after source revocation or includes malicious source instructions
- **THEN** authorization and instruction-authority checks hold on the restored production path and retained traces demonstrate the outcome

