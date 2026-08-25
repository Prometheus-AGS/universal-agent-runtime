## Context

See `proposal.md` for motivation. The frontend already owns one namespace conversion table, and its save path uses that table correctly. The backend exposes only canonical plural/hyphenated namespace routes before a generic setting-key fallback. The installed LaunchAgent serves the frontend static bundle and provider configuration from the same process, so browser evidence must be collected after a full native reinstall. KBD source is a gitlink and must be changed upstream, then pinned here.

## Goals / Non-Goals

**Goals:**

- Make settings read and write transports share one canonical namespace-to-slug contract.
- Prove route construction in a focused unit test and prove shipped behavior against the installed service.
- Preserve provider identities/configuration across the native reinstall.
- Preserve the former KBD run as audit while keeping only the successor phase current.

**Non-Goals:**

- Add backend aliases for incorrect singular or underscored paths.
- Change settings payloads, persistence, provider configuration, save behavior, entity schemas, or realtime subscriptions.
- Refactor the settings store, hooks, or panel composition.
- Merge either review branch or add hosted test automation.

## Decisions

### Reuse the existing conversion function at the read boundary

`fetchSettingsNamespace(namespace)` will call the existing namespace converter before URL construction. This is the minimum change and keeps routing knowledge in one transport module. Duplicating a second read-only map was rejected because it would allow reads and saves to drift again. Backend aliases were rejected because they would preserve the frontend defect and make the generic key route more ambiguous.

### Test transport behavior without widening component coverage

An adjacent Vitest test will mock `fetch` and assert the exact provider, Context Management, and server URLs plus the current non-2xx error string. This isolates the defect at its owning boundary; component tests are retained but are not the primary proof of transport construction.

### Certify the installed static bundle with a dedicated Playwright surface

A local Playwright config/spec will target `http://127.0.0.1:1906`, observe settings requests and responses, visit Provider Overrides and Context Management, assert configured provider rendering, and fail on settings-route 404s, singular provider reads, or underscored namespace reads. It will not start or replace the service.

### Preserve KBD source ownership

The rollover implementation remains on the pushed upstream review branch. UAR records only the exact gitlink commit. The installed CLI and daemon are built from that commit before the first successor event; only the Sovereign Sync LaunchAgent is restarted.

## Risks / Trade-offs

- [Risk] A unit test could pass while the installed bundle remains stale → Mitigation: validate the static bundle, install through the native installer, and run the port-1906 browser test afterward.
- [Risk] Native installation could change provider configuration or IDs → Mitigation: capture LaunchAgent state and provider IDs/count before installation and compare the same canonical endpoint afterward.
- [Risk] The origin/main merge could introduce unrelated failures → Mitigation: perform the already-inspected merge at Execute start, keep its history visible, and run the full local gate list without repairing unrelated defects silently.
- [Trade-off] The installed Playwright check depends on a local LaunchAgent → It is intentionally local deployment evidence and is excluded from the general frontend test suite and GitHub Actions.

## Migration Plan

1. Enter Execute, merge `origin/main`, and pin the pushed KBD commit.
2. Implement the read conversion and both regression tests.
3. Run frontend, bundle, OpenSpec, and locked Rust release gates locally.
4. Capture provider IDs/count and LaunchAgent status.
5. Install with `packaging/native/macos/install.sh`, preserving configuration and static backups.
6. Compare health/readiness and provider IDs/count, then run the installed browser proof.
7. If installation or live proof fails, keep the review branch unmerged and use the installer's backups/previous binary while retaining all recorded evidence.
