# Refinement log — `harden-jwt-defaults`

## Iteration 1 — 2026-08-18T16:30:39Z

- Specify: derived four blocking constraints from the OpenSpec proposal, delta
  spec, task ledger, current phase plan, and generic KBD constraints.
- Plan: reject the published fallback at configuration load, apply one
  registered-claim policy to HS256 and JWKS, and keep UAR-issued tokens coherent.
- Execute: added configurable `nbf` validation, issuer/audience enforcement,
  API-key and proxy claim propagation, focused tests, and operator documentation.
- Reflect: existing config-manager fixtures that implicitly relied on the unsafe
  default failed; they were corrected to declare anonymous mode explicitly.
- Observe: security 37/0, configuration 19/0, config integration 5/0,
  config-manager 3/0, sidecar 3/0, API exchange 1/0, and proxy 1/0 passed.
- Persist: wrote the OpenSpec verification receipt and PMPO state. Independent
  artifact review remains the termination gate.
- Content hashes: config `081b3a79259b401ceadb3d2357d4f81f4c87304c41a8ea52e4814df85533fcaf`;
  verifier `a8d4c752fcbd606ccf325a7b469a96ffdaac64c77dca66e4fe2d1db937486efa`;
  middleware `7ff64a2d669da7000f96b3f19061649cd6e932bd26915570d2fe64d27db8ded2`;
  API keys `338e3cf70849b2417f10a31162e432eaa3da041d88bd2e0eba6263a0fa3b0b71`;
  server `ece70f13c4d50586f0c2bb8cb75588120010613edd85928e60dc7200de87afe0`;
  proxy `e35472c9afef0b91ae9daab2e75df0f8232a8d6dcdc209fcf68333f12d708e5a`.
- Content type: `direct:content`; evaluation is source inspection plus
  deterministic command evidence.

## Iteration 2 — 2026-08-18T16:49:59Z

- Reflect: independent review found that configuration-manager fixtures read the
  operator's home configuration and that a Vault-resolved secret could bypass
  the pre-resolution fallback check.
- Execute: made the fixtures hermetic and revalidated the effective secret after
  optional Vault resolution during startup, watcherless startup, and reload.
- Reflect: review also found stale public-site documentation and imprecise `nbf`
  wording that omitted jsonwebtoken's default 60-second leeway.
- Execute: updated both documentation sets and made the OpenSpec scenario state
  the clock-skew allowance explicitly.
- Observe: security 37/0, configuration 20/0, config integration 5/0,
  config-manager 3/0, sidecar 3/0, API exchange 1/0, and proxy 1/0 passed.
- Persist: refreshed receipts against the exact post-review candidate; final
  independent re-review remains the termination gate.
- Content hashes: config `a864048d615cdde91ee9cfae8f962e82e0eaa6981596b7b82ef5e610f65996e9`;
  config manager `f0a4a8dd8f2645630008291d8800105e468380bc91427decc0d6b8941c37dc59`;
  verifier `a8d4c752fcbd606ccf325a7b469a96ffdaac64c77dca66e4fe2d1db937486efa`;
  middleware `7ff64a2d669da7000f96b3f19061649cd6e932bd26915570d2fe64d27db8ded2`;
  API keys `338e3cf70849b2417f10a31162e432eaa3da041d88bd2e0eba6263a0fa3b0b71`;
  server `ece70f13c4d50586f0c2bb8cb75588120010613edd85928e60dc7200de87afe0`;
  proxy `e35472c9afef0b91ae9daab2e75df0f8232a8d6dcdc209fcf68333f12d708e5a`.
- Reflect: the judge found stale Cargo `filtered out` denominators in the
  receipt. Exact focused replays refreshed all six affected tails.
- Termination: the history-free critic and judge independently returned PASS on
  the corrected candidate. All four blocking constraints are satisfied.
