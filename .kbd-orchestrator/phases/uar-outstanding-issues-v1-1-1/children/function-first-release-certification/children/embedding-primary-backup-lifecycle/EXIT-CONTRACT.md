# Exit contract — embedding-primary-backup-lifecycle (frozen before edits)

Child of `function-first-release-certification`, created 2026-10-19 15:10 UTC
via canonical typed phase events (create/activate). Candidate of record: v4
sha256 `96bf486872c57d22071596be6d81af42d3dfe7dee801a40226e507caf4225154`.
Running instances: 1907 (v4, embedded surrealkv, credential key), 1908 (v4,
remote surreal, no-key class).

## Frozen conditions (ALL must be evidenced for PASS)

1. **Inventory** (authored artifact): every approved embedding
   provider/model ID, endpoint, dimension, and credential reference/presence
   — never secret values. Sources: compiled backend catalog, settings schema,
   and the live deployments' persisted rows.
2. **Primary/backup resolution defined + truthful effective status**: the
   resolution rule (what is primary, what is backup, when each is used) is
   stated with file/line citations, and the status surfaced to
   operators/APIs matches the actually-wired backend (no `live` claims for
   unhonored values — the v4 contract).
3. **Pinned compatible vector-space identity** for ingestion AND query:
   `{provider, model, dimension}` binding proven at ingest (KV/row readback)
   and enforced at query; the pinned identity persists across a real restart
   and is restored on boot (row readback + overlay/hydration evidence). The
   query-enforcement leg is PASS only with a valid exercised probe; if no
   valid probe exists in this deployment, this condition is **UNVERIFIED**
   and the child is NOT READY — "structurally impossible" is never a PASS.
4. **Invalid config rejection without state loss**: an invalid embedding
   mutation is rejected (4xx, explicit reason) and the previously working
   configuration + retrieval are byte/behavior identical afterwards
   (retrieval re-probe passes).
5. **Bounded compatible failover/recovery**: swap primary → compatible
   backup → prove ingestion + query continue on the backup space identity;
   swap back → recovery proven. If NO approved compatible backup exists in
   the inventory (condition 1), report the exact blocker — and the child is
   **NOT READY**. A blocker report never satisfies this condition; the
   requested capability is primary+backup, and certifying it without a
   demonstrated backup would be a false PASS.
6. **Incompatible space refusal without data loss**: an incompatible target
   space — including an equal-dimension different model — is refused
   (mutation rejected pre-persist, or query refused with explicit mismatch),
   and existing documents/sources/KB bindings are provably intact afterwards.

## Hard limits

No new credentials, no purchase, no pay-as-you-go or fallback provider, no
unapproved data transmission, no security-surface relaxation, no
publication, no deploy, no auto-reembedding, no migration platform. Only
kimi-for-coding/k3 for any model call. A bounded same-source local build of
the existing dirty checkout IS authorized when a condition requires the
single smallest shared correction (the resulting candidate must be hashed
and recorded; v4 stays the candidate of record until such a build exists).
No manual KBD projection edits; close via canonical typed phase events.
Preserve 1907/1908 binary/config binding; fixture state changes must be
reversible.

## Verdict rule

PASS only when 1–6 are each evidenced with real receipts on v4. Otherwise
NOT READY with exactly one concrete blocker named.


## Operator continuation — 2026-10-21T00:17:37.409884+00:00

The operator preserved all frozen acceptance conditions and authorized new local-only live failover and incompatible-space implementation and fixture source. Restart resilience is still insufficient for condition5; query mismatch exercise is still required for condition3. No hosted calls, new credentials, paid usage or personal configuration mutation is authorized. Verification remains CLOSED; the older bounded-build permission above does not authorize any current execution. Existing v4/v5 receipts remain historical and cannot certify new source. The child remains NOT READY until real authorized evidence exists.
