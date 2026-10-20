# Launch-bound host principal for uar-sidecar — design, ownership, focused commands (2026-10-20)

Owner: uar-lead (Opus 5.5). Written BEFORE any source edit, under the operator approval recorded in the
ledger addendum "Operator approval recorded BEFORE any edit — 2026-10-20T00:04:54Z".

## 1. Current architecture (lead source review)

- `src/bin/uar-sidecar.rs:146-220` (`prepare_sidecar_process`). Runs before Tokio and forces env:
  - loopback host, managed ownership, JWT off unless configured;
  - **`UAR_SKILL_EVOLUTION__ENABLED=false` and `UAR_MEMORY__ENABLED=false` unconditionally (`:191-194`)**,
    because "one sidecar serves every host session";
  - the launch token is the first stdin line (`:53-76`).
- `src/uar/security/sidecar_guard.rs:171-210` (`enforce`): rejects a bad Host (403 `bad_host`), any `Origin`
  (403 `origin_present`), a missing/bad token (401 `missing_token`/`bad_token`). On a token match it strips
  `Authorization` and installs `HostAuthenticated`.
- `src/uar/security/middleware.rs:163-283` (`auth_middleware`):
  - `x-uar-principal` is refused without `HostAuthenticated` (`:173`) and refused on remote profiles (`:215`).
  - With it, ANY header-valid subject (`sidecar_principal`, `:26-40`) becomes the request's `UserContext`,
    bound to `HostAuthority::for_launch` (`claims.rs:133-136`). That is per-request subject switching.
  - A host-authenticated request WITHOUT the header falls through to `resolve_authenticated_context(None)`
    (anonymous when JWT is off).
- `CredentialPrincipal::from_verified_context` accepts the launch-bound context as `local-host`
  (`claims.rs:221-228`). Memory (`memory/service.rs:691-693`, `controls.rs:96-106`) and the verified-owner
  thread/team/spawn paths consume the SAME `UserContext`, so no route-level change is needed.
- Memory injection searches "(session, user, agent) then global" (`memory/context_builder.rs:56-60`). With
  several subjects in one process, user/agent/global-scope content is owner-scoped, but the shared-sidecar
  threat is that a host mapping many end users through one process can mix them. That is the reason the
  memory-off default exists.

## 2. Design: opt-in launch-bound mode (smallest coherent change)

- **Activation:** `UAR_SIDECAR_BOUND_PRINCIPAL=<subject>` in the sidecar's launch environment. It is read
  only in `prepare_sidecar_process` (env, set by the launching host, which already holds the launch secret).
  It is never read from a config file or a request. It is validated with the same rules as
  `sidecar_principal` (non-"anonymous", 1-128 chars, `[A-Za-z0-9][A-Za-z0-9._:-]*`); invalid -> refuse to
  start (fail closed).
- **Bound mode, identity:** every host-authenticated request resolves to exactly the bound subject.
  - If `x-uar-principal` is absent, the bound subject is used.
  - If it equals the bound subject, it is accepted.
  - If it differs, 403 `principal_mismatch`.
  - There is no per-request switching; the process has exactly one owner for its lifetime.
  - Token/Origin/Host enforcement is unchanged (it runs first).
- **Bound mode, memory:** the unconditional `UAR_MEMORY__ENABLED=false` is replaced by "follow
  configuration" ONLY in bound mode. The cross-session-mixing rationale does not apply, because one process
  equals one owner. Session-scope capture stays session-filtered; user/agent/global-scope recall crosses only
  that owner's own sessions (same user). `UAR_SKILL_EVOLUTION__ENABLED=false` stays forced in both modes (not
  needed).
- **Shared mode (no bound principal): byte-for-byte unchanged behaviour.** Memory is still forced off and
  the per-request header still works as today, so existing hosts are unaffected.
- **Multi-user rule (documented, not new code):** one bound sidecar process per end user (own token, own
  store directory in fixtures). Cross-user isolation is enforced by owner-keyed storage plus single-owner
  processes.
- No new credential type. No change to the main binary's anonymous behaviour, cloud/remote auth, JWT
  enforcement, providers, or per-route owner checks.

## 3. Single-writer ownership

| Files | Writer | Model (provenance) |
|---|---|---|
| `src/bin/uar-sidecar.rs`, `src/uar/security/sidecar_guard.rs`, `src/uar/security/middleware.rs`, `src/server.rs` (only the `start_server_sidecar` signature/plumbing), their inline `#[cfg(test)]` tests | uar-trust-tools (resumed existing teammate) | Opus 5.5, inherited from `.claude/agents/uar-trust-tools.md` |
| Proof harness, scratch only: `uar-r14/identity/**` (scripts, mock, config, no source) | uar-state (resumed existing teammate) | Sonnet 5.5, inherited |
| Artifact-only review of the diff | artifact-critic (fresh) | Sonnet 5.5, requested |
| Build, tests, proof runs, grading, ledger | uar-lead | Opus 5.5, observed |

## 4. Exact focused commands under this approval (run serially by the lead, from the repo root)

Preparation: copy the existing `target/debug/uar-sidecar` (unknown provenance, built 14:08 local) read-only
to `uar-r14/preserved-bins/` with its sha256 before the build replaces it.

1. Build (debug, cached `target/`, sidecar bin only):
   `cargo build --locked --no-default-features --features server-full --bin uar-sidecar`
2. Focused unit tests for the new code (run once; a compile failure in unrelated tests is recorded, not chased):
   `cargo test --locked --no-default-features --features server-full --bin uar-sidecar bound_principal`
   `cargo test --locked --no-default-features --features server-full --lib sidecar_bound`
3. Runtime proofs (synthetic launch tokens, loopback mock LLM on 18915, sidecar on 1915, disposable
   stores under `uar-r14/identity/store-*`, deleted on exit; 1906/1907/1908 untouched):
   `bash /private/var/folders/ln/0wnpd96j26z2qhvx9m6hwt2r0000gn/T/opencode/uar-r14/identity/proof.sh`

Proof assertions (all hard):
- N1 missing token -> 401 `missing_token`. N2 wrong token -> 401 `bad_token`. N3 any `Origin` -> 403
  `origin_present`. N4 wrong Host -> 403 `bad_host`.
- N5 bound to `alice`: `x-uar-principal: bob` -> 403 `principal_mismatch`.
- N6 shared mode (no bound principal): memory remains disabled (default preserved).
- M1 same owner + same session: turn-1 auto-capture record -> turn-2 upstream request carries it inside
  `[MEMORY CONTEXT]` -> mock reply `RECALL:<codeword>` decided only from that block.
- M2 same owner, new session: no session-scope injection of the codeword.
- M3 cross-user on the SAME store (sequential boots): bob-bound process sees no alice records and gets no
  injection; an alice-bound reboot still recalls (persistence plus owner partitioning).
- T1 a coordinator run spawns two children (`spawn_agent`, approved via `/api/uar/runs/{id}/tool-approval`
  as the bound owner); `wait_agents` joins both results.
- T2 one child is interrupted/cancelled; after cancellation it produces no further events, tool calls or
  model requests.
- T3 a parallel run on a different session completes concurrently; its upstream requests contain none of the
  coordinator's or children's content, and neither run deadlocks or loses its result.

If the tier/hook layer requires separate approval of literal commands, or the implementation turns out to
need a materially different security change (e.g. touching JWT/remote auth or route owner checks), the lead
stops and returns that exact decision.

## 5. Grades preserved

v4 R21 FAIL; debug `f1c2f473…` R21 PARTIAL (T1-T6 PASS, memory missing); R20 PARTIAL; v4 tally 9/6/5/2/0/1.
Results of this round are graded for the NEW sidecar candidate identity only.
