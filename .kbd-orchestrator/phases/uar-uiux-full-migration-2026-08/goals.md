# Goals

- Discard the current web UI implementation and rebuild the frontend completely on top of the UI/UX specified in docs/ui/, replacing the existing interface rather than refining it
- Binding design authority is docs/ui/uar-frontend-migration-plan.md plus the design comps (UAR Console.dc.html, UAR Mood Board.dc.html, UAR Shell & Mobile.dc.html) and the delivered Slash Gate logo set in docs/ui/logo/
- The rebuilt UI consumes the CURRENT set of services the universal-agent-runtime already provides — the backend API surface is the fixed contract; this phase does not add or redesign runtime services
- Target stack per plan §0/§1: React 19, Vite 8, Tailwind 4 (CSS-first, tailwind.config.ts deleted), prometheus-entity-management for server entities, Zustand+Immer for transient state, PGlite for client-owned conversation persistence, AG-UI SSE transport, assistant-ui + Base UI-backed local wrappers for general controls, navigation, overlays, and sidebars, restyled Flat 2.0; Base UI is the D1 operator override of KnowMe §6.1/§6.3, not compliance with their shadcn requirement
- Strict feature-based clean architecture with lint-enforced downward dependency: app → features → shared → platform; kebab-case for every file and directory
- Flat 2.0 mechanically enforced: no borders, dividers, layout shadows, backdrop-blur, gradients, or variant=outline; separation by surface ladder and spacing only; 3px ember focus ring is the sole allowed exception
- Every text surface renders through one markdown renderer (shared/markdown/markdown-bubble.tsx) with a single sanitized plugin chain; mermaid/shiki lazy-loaded and never in the initial chunk
- Complete chunk catalog per plan §8 renders from the AG-UI event vocabulary, with run-trace bar, timeline, and inspector driven by the phase attribution model
- Removal targets: TanStack Query entirely, src/admin/, the terminal/CRT admin theme, tailwind.config.ts, postcss.config.js, highlight.js, and the retired stores
- Exit gates per plan §11: CI-enforced bundle/latency budgets, WCAG 2.2 AA accessibility certification, responsive sweep at 320/768/1024/1440 in both themes, and the §12 acceptance checklist
- Process: full KBD lifecycle assess → analyze → plan → execute → reflect with operator review between stages; every change carries an OpenSpec spec delta
