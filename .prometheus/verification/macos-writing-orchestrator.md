
## 2026-10-26 — VERIFIED: persistent writer and orchestrator intent routing on Mac

- Persistent agent `writing-specialist`, **Blog and Article Writer**, registered alongside `math-specialist-db64c2a8` and retained across the local policy-fix restarts. [Definition](agents/writing-specialist.json).
- The user task requested a community-garden blog post; it contained no writer ID or private byline.
- AG-UI reported `selectionMethod=intent`, source`orchestrator-agent`, selected`writing-specialist`, child thread`80b5c703-ced7-45ab-8873-024c1b2ff274`. The math agent was not selected.
- The normal local spawn-agent approval was answered through AG-UI resume; no governance bypass or permit-all policy was used.
- The selected writer produced a useful 248-word article containing its saved private byline. [Actual draft](2026-10-26T07-44-36.207Z-community-garden-blog.md).
- Cucumber:1scenario/7steps passed, exit0. [Report](2026-10-26T07-44-36.207Z-writing-orchestrator-macos.json). The final stream terminal was success; the added explicit completion predicate was also evaluated against these captured events without another request. [Combined evidence](agents/writing-specialist.verified.json).
- Setup failures retained: unauthenticated request401, missing policy directory denial, invalid copied policy syntax, and normal approval interruption. The local instance now uses two explicit Cedar permits limited to orchestrator spawn/wait; the older iOS policy-template syntax issue is recorded for later work, not edited here.
- Current UAR stays running at`http://127.0.0.1:53587`, PID26523; owner exec session27203. Both agents retained. No earlier verified behavior reran. Scope binds the recorded native build/configuration.
