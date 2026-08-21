#!/usr/bin/env node

// Backward-compatible local entrypoint. Release product checks no longer run in
// GitHub Actions; the standing contract is the deployment-only workflow policy.
await import("./validate-github-actions-policy.mjs");
