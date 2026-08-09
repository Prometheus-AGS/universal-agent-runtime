> **Read `EXECUTION-CONTRACT.md` first.** It fixes the order (this change is
> second), the pinned command, and what counts as satisfied.
>
> **Local gate:** every case added or renamed here is executed by the pinned
> local command from `conformance-baseline-gate` task 2.1, which runs the whole
> `live::capability_cases` module. This change adds no GitHub Actions work and
> must not add a case the local gate skips.

## 1. Define the taxonomy before relabelling

- [ ] 1.1 Document the full prefix set at the top of `capability_cases.rs`:
      `l1_` route present (reachable, nothing about behaviour) ·
      `l2_` wired (real call path, fixtures authored by the test) ·
      `l3_` exercised (correctness independent of stub output) ·
      `l4_` round-tripped (survives a restart) ·
      `shape_only_` response shape only, no semantics ·
      `absent_` asserts a documented absence ·
      `excluded_` published exclusion, reason in the doc comment.
- [ ] 1.2 Every case name carries exactly one defined prefix. A prefix that is
      not in the list is a build-time review failure.

## 2. Relabel what is currently overstated

- [ ] 2.1 Any case whose correctness depends on stub output is `l2_`, not `l3_`.
      Candidates from the baseline: C-03 provider registry, C-05 knowledge-base
      catalog, C-08 tools registry. Verify each individually — a case that only
      checks a registry the runtime populates itself may legitimately stay `l3_`.
- [ ] 2.2 Record the before/after label for every case changed, so the relabel
      is auditable rather than a silent downgrade.

## 3. Close the eight-capability hole, to target

Each row states the **minimum** evidence level. A case that cannot reach its
target becomes an `excluded_` case with the blocking reason named in its doc
comment — never a weaker pass in disguise.

- [ ] 3.1 **C-21 tenant isolation — target L3 + negative.** Boot with two
      tenants; tenant A writes; tenant B reads the same resource; assert B is
      **denied**, not merely 404. A 404 is ambiguous between "isolated" and "not
      found". If the harness cannot boot two tenants, this becomes `excluded_`
      with that structural reason stated — do not substitute a single-tenant
      smoke test.
- [ ] 3.2 **C-25 node DID — target L3.** `did:key` derivation is deterministic
      and offline; assert the derived DID against the published W3C test vector
      already used in `frf-did`, not against our own output.
- [ ] 3.3 **C-26 DID resolution + VC verification — target L3.** Offline for
      `did:key`. Include a **negative**: a credential issued by a different DID
      must be rejected.
- [ ] 3.4 **C-27 wallet — target L3.** Include the fail-closed cases:
      forged issuer rejected, expired credential rejected.
- [ ] 3.5 **C-16, C-18, C-19 — target L2 minimum.** Raise to `l3_` where the
      surface allows correctness assertions independent of stub output.
- [ ] 3.6 **C-24 peer mesh — `excluded_`.** Requires two devices; state that as
      the reason. Do not fake it with a single-node assertion.
- [ ] 3.7 Every new case includes a discriminator proving the **real handler**
      answered rather than the `/api/{*path}` catch-all, which returns
      `code: "api_route_not_found"`. A not-404 check alone is L1 and satisfies
      no row above.

## 4. Verification

- [ ] 4.1 All 27 capabilities have either a case at or above its target level,
      or an `excluded_` case with a stated reason. No capability is silently
      absent.
- [ ] 4.2 Full matrix green with the pinned command, quoted here so it need not
      be resolved from another change:
      `UAR_LIVE_INTEGRATION_BACKEND=recorded cargo test --locked
      --no-default-features --features server-full --test integration
      live::capability_cases -- --test-threads=1`
- [ ] 4.4 Append one row per case to
      `.kbd-orchestrator/phases/uar-spec-conformance-2026-08/verification.md`
      in the format defined by `EXECUTION-CONTRACT.md`.
- [ ] 4.3 The published result is a per-capability table with evidence levels.
      **No aggregate percentage and no runtime-level verdict** — an earlier
      method was killed in review for exactly that.
