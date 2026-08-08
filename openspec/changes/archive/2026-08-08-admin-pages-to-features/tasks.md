## 1. Workflow, evidence, and migration contract

- [x] 1.1 Record the canonical C-14a start, complete the OpenSpec artifacts, and validate the new `frontend-configuration-surfaces` delta before implementation
- [x] 1.2 Inventory all thirteen page exports, routes, directly owned hooks/stores/APIs/helpers/tests, observed cross-consumers, and current Flat 2.0 allowlist entries
- [x] 1.3 Run the mandated UI/UX memory, UI Pro, Impeccable-equivalent audit/critique, frontend-design, React best-practices, and composition review; preserve a one-paragraph task-specific distillation
- [x] 1.4 Capture focused behavior baselines and protected-scope receipts without staging or modifying operator-owned paths

## 2. Shared boundary and low-coupling features

- [x] 2.1 Establish the public feature-entry convention, teach the existing boundary gate to recognize feature `api/` modules, and move only cross-domain loading, empty, and error projections into `shared/ui/configuration`
- [x] 2.2 Migrate the auth page, hook/store/tests, and auth API into `features/auth`, update every observed consumer, and pass focused plus cheap gates
- [x] 2.3 Migrate the credentials page, hook/store/tests, and credentials API into `features/credentials`, update every observed consumer, and pass focused plus cheap gates
- [x] 2.4 Migrate the tools page, detail UI, hook/store/tests, entity helper ownership, and tools API into `features/tools`, update every observed consumer, and pass focused plus cheap gates
- [x] 2.5 Migrate the compiler page, hook/store/tests, entity helper ownership, and compiler API into `features/compiler`, update every observed consumer, and pass focused plus cheap gates
- [x] 2.6 Migrate the cost dashboard and owned entity projections into `features/cost`, update every observed consumer, and pass focused plus cheap gates

## 3. Stateful configuration features

- [x] 3.1 Migrate the memory page, hook/store/tests, entity helper ownership, and memory API into `features/memory`, update every observed consumer, and pass focused plus cheap gates
- [x] 3.2 Migrate the skills page/helpers/tests, import UI, hook/store/tests, entity helper ownership, and skills API into `features/skills`, update every observed consumer, and pass focused plus cheap gates
- [x] 3.3 Migrate the knowledge page, directly owned hook/store/fetcher/tests, and knowledge API into `features/knowledge`, update every observed consumer, and pass focused plus cheap gates
- [x] 3.4 Migrate the agents page, editor/builder UI, hook/store/tests, entity helper ownership, and agents API into `features/agents`, update every observed consumer, and pass focused plus cheap gates
- [x] 3.5 Migrate the providers page/welcome UI, hooks/stores/tests, entity helper ownership, and providers API into `features/providers`, update every observed consumer, and pass focused plus cheap gates
- [x] 3.6 Migrate the models page/catalog model, hooks/stores/tests, entity helper ownership, and models API into `features/models`, update every observed consumer, and pass focused plus cheap gates

## 4. Runtime and settings features

- [x] 4.1 Migrate the runtime-console page/tests, hook/store/feed ownership, and runtime-console API into `features/runtime`, preserve run-trace/query/subscription behavior, and pass focused plus cheap gates
- [x] 4.2 Move the 3,336-line settings page intact with its hooks/stores/tests and settings/user-settings APIs into `features/settings`, preserve all behavior for C-14b, and pass focused plus cheap gates
- [x] 4.3 Update `pages/admin-page.tsx` so every migrated page resolves only through feature public entries while preserving the existing section inventory and provider onboarding banner

## 5. C-14a token and ownership cleanup

- [x] 5.1 Replace all C-14a-owned `hsl(var())` expressions in models, memory, cost, skills, and compiler with existing semantic Tailwind 4 tokens without introducing arbitrary palette values
- [x] 5.2 Shrink the C-03 Flat 2.0 allowlist for moved files and prove zero legacy page owners remain under `frontend/src/admin/pages/`
- [x] 5.3 Prove no migrated feature imports another feature's internals, no UI imports stores/APIs, and every observed external consumer uses an intentional public feature export

## 6. Verification, review, and closeout

- [x] 6.1 Run frontend typecheck, lint, architecture-boundary, Flat 2.0, and all focused migrated-feature tests after implementation is wired
- [x] 6.2 Run the full frontend suite and production build at the C-14a completion boundary; verify the existing bundle/performance ownership is not regressed
- [x] 6.3 Run strict OpenSpec change/capability validation, UI audit/critique/polish review, responsive smoke checks, and scoped diff-integrity checks
- [x] 6.4 Run isolated adversarial review on the completed C-14a artifact packet and resolve every critical finding
- [x] 6.5 Write `files.txt` and verification evidence, confirm protected/staged paths remain untouched, and record any explicit C-14b/C-14c handoff constraints
- [x] 6.6 Transition canonical KBD C-14a to complete, append the `.prometheus` waypoint, archive the OpenSpec change, emit the step-17 completion signal, and advance automatically to C-14b
