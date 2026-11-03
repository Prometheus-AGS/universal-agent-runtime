# Governed media effects

The host obtains explicit user confirmation before issuing a media grant. The confirmation specifies the coordinator, session, exact room or recipient, purpose, expiry, and allowed action/modality/capability tuples. The host verifies each coordinator or worker identity before forwarding its request. Meeting observations and model output can propose an action, but cannot attest confirmation or establish identity.

UAR derives owner, tenant and host identity from authenticated request context. A configured verified service or an authenticated sidecar host can call the API. Use the same workspace and host identity throughout an effect. Workers use the host as their intermediary and receive no credential or dispatch claim. The host resolves any connector credentials itself, with distinct speech and video processor credential slots. UAR refuses media mutations on an in-memory collaboration catalog.

All routes are relative to `/api/v1/collaboration` and require `x-uar-workspace-id`. Request and response fields use camel case.

| Method and resource | Host operation |
| --- | --- |
| `POST /media-grants` | Attest confirmation and issue a grant atomically. Send `commandId`, `coordinatorId`, `scope`, `permissions`, and `expiresAt`. |
| `POST /media-grants/{id}/revoke` | Revoke the grant with a `commandId`. Already claimed effects require resolution through their receipts. |
| `POST /media-delegations` | Send `commandId`, `grantId`, `workerId`, a subset of `permissions`, and `expiresAt` no later than the grant. |
| `POST /media-effects` | Prepare intent with `commandId`, `grantId`, host-verified `actorId`, optional `delegationId`, exact `scope` and `permission`, and `contentDigest` for send content. |
| `GET /media-effects/{id}` | Read the current effect and its receipt history. |
| `POST /media-effects/{id}/dispatch` | Recheck authority and claim the effect. Send `{}` or matching transient `content` for a send. The response includes the one-time work claim and uses `Cache-Control: no-store`. |
| `POST /media-effects/{id}/cancel` | Cancel an unclaimed intent with a `commandId`. |
| `POST /media-effects/{id}/outcome` | Settle the attempt with `commandId`, `attemptId`, `outcome` (`applied` or `notApplied`), and optional opaque UUID `evidenceRef`. |
| `POST /media-effects/{id}/reconcile` | Settle an unknown attempt with the same fields and an independently obtained `evidenceRef`. Reconciliation is a read of peer evidence followed by a receipt write. |

A scope contains `sessionId`, `purpose`, and `target` (`{ "room": "opaque-id" }` or `{ "recipient": "opaque-id" }`). Permissions contain `action`, `modality` (`audio`, `video`, or `text`), and an opaque `capability`. Room actions are `join`, `publish`, `sendContent`, and `endMedia`. Recipient actions are `startCall`, `transfer`, and `endCall`. The supplied adapter simulates these actions without a real media or phone transport.

Use opaque target and capability IDs. The persisted grant references the immutable confirmation. Worker delegations inherit its session, target, and purpose. Supply content only to dispatch, with a 64 KiB UTF-8 limit. The prepared `contentDigest` uses the lowercase `sha256:<hex>` format. Receipts contain no media bytes, transcript, credentials, or connector response body.

## Retry and recovery

Generate and retain a command ID per logical command, including across reconnects. Repeating the same command and body returns the existing resource. Reusing the command ID with a changed body returns a conflict. The effect ID is the peer idempotency key.

A committed claim sets the effect to `unknown` before the host receives work. If the claim response or peer result is lost, inspect the original effect. An unknown effect cannot be claimed again. The host must only report `notApplied` if it has definitive evidence and has fenced any outstanding work that might still apply. Cancel before claiming to prevent dispatch. End active media through a separate authorized effect.

Catalog fields default to empty when an older catalog is read. Back up a populated media catalog before rolling back to a pre-media binary. An older writer can omit the new fields when replacing the catalog.

## Local acceptance fixture

The Cucumber target drives production HTTP routes and a durable UAR catalog. It launches the built `mock_media_peer` example as a separate JSON Lines process. That process retains only effect IDs, session digests, and evidence across UAR restarts, and can simulate an applied effect whose response was lost.

The fixture host provisions these nonsecret inputs:

- `UAR_BASE_URL` points to the disposable UAR through a trusted-host identity endpoint. `UAR_MEDIA_OTHER_TENANT_BASE_URL` addresses the same UAR through a second configured trusted host tenant. `UAR_MEDIA_UNTRUSTED_BASE_URL` supplies an ordinary non-host identity. The identity infrastructure owns credential provisioning and injection. A proxy that supplies ordinary user identity alone is insufficient.
- `UAR_WORKSPACE_ID` selects a disposable workspace. `UAR_MEDIA_MOCK_PEER` is the absolute path of the built example.
- The default lifecycle adapter requires a dedicated container labelled `uar.media-fixture=386`, selected by `UAR_MEDIA_SERVICE_CONTAINER`, with an embedded SurrealKV store on a host bind mount. Set `UAR_MEDIA_STORE_PATH`, `UAR_MEDIA_STORE_NAMESPACE`, `UAR_MEDIA_STORE_DATABASE`, and `UAR_MEDIA_STORE_RUNNER` (absolute path of the built store example).
- During provisioning, with the disposable container stopped and a fresh store, run `node scripts/acceptance/fixtures/media-fixture-control.mjs seed-legacy-catalog`. Retain its JSON stdout in the file named by `UAR_MEDIA_LEGACY_RECEIPT`. Start the container and leave the identity endpoints available across restarts.
- A different external supervisor can provide an executable through `UAR_MEDIA_FIXTURE_CONTROL`. It receives `legacy-catalog` (pre-start store receipt), `restart` (JSON with distinct positive `before` and `after` process IDs after readiness), or `snapshot` (JSON with actual `catalog` and `diagnostics` from the disposable fixture). Use synthetic fixture data and keep credentials out of outputs.

The focused local build and behavior commands for this suite are:

```sh
cargo build --locked --no-default-features --features minimal,local-models --bin universal-agent-runtime --example mock_media_peer --example surrealql_store_runner
node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/media-effects.cucumber.mjs
```

Set `UAR_CUCUMBER_REPORT` to retain a JSON report for each authorized attempt. The report attaches observed HTTP statuses, peer invocation counts, receipt IDs and outcomes, and restart process IDs. It excludes request and response bodies, the privacy canary, and raw diagnostics. Keep earlier attempt reports when retrying. The suite uses simulated meetings and calls.
