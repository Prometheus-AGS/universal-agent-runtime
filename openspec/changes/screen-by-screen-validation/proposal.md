## Why

Operator requires functional validation of every screen with live browser
tests and video proof: agents, skills lifecycle, RAG provenance, memory
levels, JWT auth, isolation, local-first - none of the 18 admin screens
have BDD coverage today.

## What Changes

- Screen-by-screen validation plan executed over the 20-screen inventory:
  purpose/function verification per screen, local-first behavior, any-agent
  conversations, orchestrator/default-agent live Q&A with expected answers,
  skills add/enable/disable, KB hits in UI, JWT flow, memory levels,
  cross-user isolation.
- Video-proof bundles minted from the BDD runs.

## Capabilities
### New Capabilities
- `product-validation-evidence`

## Impact
tests/bdd features and steps, certification bundle, validation report.
