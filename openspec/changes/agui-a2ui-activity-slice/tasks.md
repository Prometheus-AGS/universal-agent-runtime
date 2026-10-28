## 1. Runtime (src/uar/api)

- [x] 1.1 Project published A2UI batches as cumulative `ACTIVITY_SNAPSHOT` events with stable message ids, and strip `profile`
- [x] 1.2 Restate earlier runs' live surfaces at stream start, from durable plus live history
- [x] 1.3 Accept `forwardedProps.a2uiAction` as a same-conversation run: declaration check, request shape (no new user message, activity messages dropped), and `a2ui.user_action` input
- [x] 1.4 Refuse actions for agent-graph agents before reservation
- [x] 1.5 ~~Author unit tests~~ removed: verification is BDD only (operator, 2026-10-28)

## 2. Console (frontend)

- [x] 2.1 Run the UI routing (Impeccable critique with two isolated subagents and an adversarial review, frontend-design, UI/UX Pro Max, Vercel skills, PEM skill)
- [x] 2.2 Render click-started user turns as a compact action line, never raw JSON, both live and on reload
- [x] 2.3 ~~Author the console tests~~ removed: verification is BDD only (operator, 2026-10-28)

## 3. Acceptance

- [x] 3.1 Rewrite I19 (`scripts/behaviors/features/agui-route-a2ui.feature`) for the likely path, using a PEM 4.3.0 headless client in `scripts/behaviors/clients/a2ui-pem/`
- [x] 3.2 Confirm the owned binary's rebuild command
- [x] 3.3 Record the implementation-complete inventory, then wait for the operator's gate instruction
