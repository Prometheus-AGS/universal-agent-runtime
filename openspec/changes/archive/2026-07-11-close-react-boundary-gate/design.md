## Context

The boundary checker reports ten remaining production violations after the
admin-surface certification. They are legacy subscriptions or mutations, not
infrastructure exceptions.

## Classification

| Surface | Violation | Owning remedy |
|---|---|---|
| App bootstrap | component → store hooks | application bootstrap hook |
| Admin welcome | component → store static calls | onboarding hook/store action |
| Theme toggle | component → store | theme hook |
| Enhanced thread | component → three stores | chat/thread/status hooks |
| Left sidebar | component → three stores | thread-sidebar hook |
| Top navigation | component → stores | navigation hook |
| Agent status | component → store | agent-status hook |
| Thread naming | hook → service | store-owned naming action |
| Chat page | component → service/store | chat-page hook/store action |

No remaining item qualifies for the asset/bootstrap exceptions. The only
permanent direct-fetch exceptions remain the explicit PGlite asset loader and
entity-sync transport bootstrap already encoded in the checker.

## Decisions

1. Components subscribe and submit intent only through hooks.
2. Hooks may compose stores but do not call services.
3. Stores own mutations and service calls; services own external I/O.
4. The legacy allowlist is deleted once the scan reaches zero.
5. CI keeps the same blocking checker and gains negative fixtures for every
   prohibited direction.

## UI quality distillation

The prior phase's UI/UX Pro Max, Impeccable audit/critique/harden/polish,
frontend-design, and Vercel React reviews remain applicable: preserve the
existing visual contract, keep components declarative, expose async errors and
disabled states, retain keyboard/focus semantics, and avoid UI-owned business
state. This change is architectural and intentionally introduces no redesign.
