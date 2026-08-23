## Context

The implementation changes become code-complete before this change performs the bounded local deployment and functional checks. Documentation must match the shipped paths and clearly separate observed macOS behavior from Linux/Windows templates.

## Decisions

- Build React first, then the locked release server-full profile.
- Install under `~/.uar`, listen on localhost port 1906, and preserve existing state.
- Exercise local proxy, Kimi K3, and MiniMax M3 through installed UAR with at most six short requests, each capped at 120 seconds and 64 output tokens.
- Use one restart cycle and no soak.
- Reflect only after every required provider succeeds; unavailable credentials/capacity stop the phase before reflection.
- Use Alibaba's released `qwen3.8-max` identifier. Migrate only exact native values observed in the interrupted installation: selected `alibaba/qwen3.7-max`, malformed `QWEN_TOKENPLAN_API_KEY`, and the phase-owned `qwen3-coder-plus` provider seed. Preserve every non-matching operator value.
- Keep the compile-time catalog as the authority for `/api/models`. Advance the parent gitlinks for `models.dev` and `vendor/git/liter-llm` to commits containing the released Alibaba model, regenerate UAR's reviewed offline snapshot from the pinned catalogs, and do not patch UAR's endpoint or author source inside either submodule.

## Risks

- A passing health endpoint does not prove inference; retain actual model-produced responses with identifiers.
- Browser UI evidence can accidentally exercise a direct provider path; network observation must show the installed UAR boundary.
- Linux and Windows instructions may look authoritative despite not being deployed here; platform limitations remain adjacent to each verification claim.
- A broad "upgrade every Qwen string" migration could overwrite operator intent; the correction is deliberately exact-match and its negative control uses a custom Alibaba fixture that must remain byte-identical outside additive fields.
