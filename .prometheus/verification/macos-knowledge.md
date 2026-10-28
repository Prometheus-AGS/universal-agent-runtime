# Mac: mathematics knowledge base — VERIFIED

Date: 2026-10-26. Fifth operator-directed standalone Mac behavior.

Command (with existing private owner token/workspace supplied through environment):

```sh
node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/knowledge.cucumber.mjs
```

Observed: `1 scenario (1 passed)`, `7 steps (7 passed)`, `0m 5.252s`; exit 0. Six Gherkin behavior steps plus the evidence hook are reported as seven by this runner.

Research: Dsupin and Tengely, *Discrete logarithm problem in some families of sandpile groups*, section 3, https://arxiv.org/pdf/2011.08296 . Wheel notation has n rim vertices plus one hub. For odd n the critical group order is L_n squared. Independently calculated L_11=199 and order=39601. The note does not claim an invariant-factor decomposition.

The behavior created KB `58c0e39f-43e0-47a9-b699-7dea5d82417f`, uploaded document `35f6bbe7-5804-4253-b0d1-c77642365757`, observed indexed ingestion, persisted the KB binding to existing agent `math-specialist-db64c2a8`, then queried a fresh private research-profile alias. The alias/seal mapping was provided only in the document, never in system/user prompts. Answer matched 11, 199, 39601 and the unpredictable seal. Actual `agui.rag_citations` event matched KB ID, document ID, snippet seal and answer citation; native run reported the correct agent and done status.

Report: `/Users/gqadonis/Projects/prometheus/universal-agent-runtime/.prometheus/verification/2026-10-26T08-20-39.467Z-knowledge-macos.json`.
Combined evidence: `/Users/gqadonis/Projects/prometheus/universal-agent-runtime/.prometheus/verification/knowledge/wheel-profile-aae19b22-4654-493f-b34e-d487a21c3464.json`.
Current agent snapshot: `/Users/gqadonis/Projects/prometheus/universal-agent-runtime/.prometheus/verification/agents/math-specialist-db64c2a8.knowledge.json`. The earlier arithmetic-only snapshot remains historical; use this new snapshot for future definition comparisons.

Real inference: `liter/MiniMax-M3`. Real local FastEmbed BGE-small-en-v1.5, 384 dimensions; asset manifest `f6dcb84ec0c04198069459ab3374ad66c1afb1d313b10048440dbaa188daa8b3`. Existing model assets were copied to the running profile's model directory and embedding settings were persisted/activated through REST. No download or hosted embedding request. This task charged $0 Cohere and $0 OpenAI; separate $10 caps and all previous consumption remain.

Earlier failures retained: 08:18:48 root trailing slash returned 404 before creating a KB; corrected to documented mounted root. 08:19:29 created/indexed/bound KB and returned a correct grounded answer, but the assertion rejected a whole Markdown JSON fence and a marker followed by the actual filename. Parser now accepts those presentation forms while retaining all math, private-seal and provenance checks. The first indexed KB is retained as prior-attempt evidence and remains attached; the successful new KB above is also retained. No previous verified scenario was rerun. Independent artifact-only TypeScript review found no blockers, including the final formatting correction.

Scope: this running Mac artifact only: `25151ae66312e41c99060971645b2414b28817669a164f064cd59d3b0576828c`, built source `sha256:28137dc46cbb05aa9169f93b46232ea6aafc8d3f04e7c841274e4bbf4d109018`. This does not certify all current source, other backends/platforms, embedding regeneration or unsupported questions. No product source changed, speculative guards, publication or unrelated tests were added.

UAR remains running at http://127.0.0.1:53587, PID 26523; both specialists, the knowledge bases and all receipts remain. Stop after this passing behavior and wait for operator direction.
