# Task 13.3 model verification setup source review

Scope: newly authored HTTP helper headers/timeout, protected model-verification request, verification evidence/trace reset, isolated limiter config, and latest handoff section only. Prior policy/ingestion repairs were not re-reviewed. Phase: `uar-post-lifecycle-verification-readiness`, Execute. Verification CLOSED for this reviewer; lead owns authorized execution.

Artifact echo:

```python
verification = http(base, "/api/providers/" + PROVIDER_ID + "/test", owner_token,
    {"model": MODEL_ID}, extra_headers={"X-UAR-Admin-Key": admin_key}, timeout=75)
require(verification.get("ok") is True and verification.get("received_text") is True
    and verification.get("provider_id") == PROVIDER_ID and verification.get("model_id") == MODEL_ID,
    "actual provider verification did not return the selected model proof")
```

Disposition: no concrete defect found in the newly authored setup changes. The generated settings admin key is taken from the actual written configuration, and HTTP helper preserves both the bearer authorization and supplied admin header. The source router mounts the provider test route at the requested path. The complete test_provider handler performs real inference with empty tools, rejects absent text, checks the configuration fingerprint, and awaits persisted ModelVerification before returning its matching success response. The helper's 75-second override applies only to this request; source contains two sequential 30-second inference/collection deadlines. No fabricated verification record is written by the supervisor.

The actual peer handles tool-free requests with ordinary text. The supervisor saves the endpoint response and peer trace before calling `_control`; the peer's `_control` clears requests/effects while setting scenario `single`. Requiring the returned configured flag plus empty requests trace prevents verification traffic from contaminating matrix request accounting. This source review does not independently establish runtime traffic or persisted state.

The isolated configuration retains rate_limit_enabled=true and sets existing requests_per_second=200.0 and burst_size=400.0 fields, matching the server limiter construction. The receipt records that same config object. No production default, auth or governance setting is changed. Increased test-instance capacity addresses the reported 429 masking scenario; its runtime sufficiency remains unverified.

Inspected current HEAD-relative diff, complete HTTP helper and provider test handler, route/admin binding, fixture response/reset paths, configuration fields and limiter construction. Only git diff/rg/sed source reads and this assigned documentation write ran; no import, script, static analysis, check, test, probe, startup, network or build ran. Only this review file changed. No extra guards or unrelated implementation were requested.

Gate003 failures are lead-supplied receipts. Runtime proof persistence, verification response, trace reset, limiter behavior and five-interface acceptance remain unverified here. Reviewer is isolated native same-family Codex fallback without producer generation history; different-family certification remains pending. Manual sycophancy screening found no unsupported runtime success claim or praise quota. Machine detection was not executed under the source-only assignment; no score is claimed.
