# Goals — uar-final-production-hardening-2026-07

Created 2026-07-10 via `/kbd-assess` invoked directly by the operator with an
explicit success criterion: **"After this cycle, we ONLY succeed if we are
100% ready for customer use and consumption."** The prior phase
(`uar-production-ready-uiux-2026-07`, closed 8/8) ended with an honest "not
yet externally shippable" verdict; this phase exists to close that verdict.

## Success criterion (operator-stated, binary)

The repository is **100% ready for external customer use** at the end of this
cycle. Anything that would make an engineer at a customer say "this headline
feature doesn't work" or "this is an open security hole" is a blocker, not a
disclosure.

## Goals

1. **Fix RAG/KB retrieval end-to-end.** `VectorMatcher::embed_batch` returns
   placeholder zero-vector embeddings (`model.forward()` commented out),
   confirmed to break knowledge-base search (`POST /api/knowledge/{id}/search`
   returns `{"results":[]}` for exact-phrase matches against indexed docs) and
   therefore "chat with your documents." Success = the currently-red
   `chat-kb-retrieval.feature` BDD scenario passes without weakening, plus
   direct API search returns real ranked matches.
2. **Zero open security alerts.** Resolve the 2 standing GitHub Dependabot
   alerts (1 high, 1 moderate) plus anything else `gh api dependabot/alerts`
   and `cargo audit`/`pnpm audit` surface at assess time.
3. **Runtime Console functions to task for an external admin.** Replace the
   "not yet wired" banners (Provider Health, Memory Activity, Protocols page
   panels, Artifacts panel) with real backing data, or — where a panel has no
   backing concept in the product — remove it. No gated placeholders visible
   to a customer.
4. **Test coverage credible for production claims.** Close the
   visibility-only-assertion gap in existing e2e specs; raise coverage on
   load-bearing frontend paths; all suites green in CI (bdd-chat currently
   expected-5/6 — must become 6/6 once Goal 1 lands).
5. **Config surface is not a trap.** Bare `PORT`/`JWT_REQUIRED` (and any other
   dead `Cli` env passthroughs) either work or fail loudly; the `UAR_*__*`
   convention documented where a first-time deployer will actually see it.
6. **Public docs exist.** The deferred Docusaurus site (or an equivalent the
   operator approves) is live, requiring the hosting-target decision to be
   made this cycle rather than re-deferred.
7. **Everything else standing between the repo and "customer-consumable" that
   thorough research (including web research against current
   production-readiness practice for self-hosted Rust/LLM runtime software)
   surfaces at assess time** — e.g. versioning/release artifacts, container
   images, licensing clarity, operational runbooks, rate-limit/secret
   handling defaults. Assessment must enumerate these concretely, not
   hand-wave.

## Non-goals

- New product features beyond what "the advertised surface actually works"
  requires.
- Re-litigating archived changes from prior phases.
