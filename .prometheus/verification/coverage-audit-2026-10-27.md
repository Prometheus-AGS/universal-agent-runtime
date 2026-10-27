# Manual coverage audit — 2026-10-27

The audit confirms **35 bounded Mac behavior labels across 32 workflow outcomes**, with four missed original-checklist overlaps corrected. No test was rerun. The uncomfortable finding is that the previous headline made all outcomes look like clean end-to-end scenario passes: B03 is supported by a failed combined feature's successful persistence steps plus a separate passing query. Its original failure remains failed.

## Audited counts

| What is counted | Count | Meaning |
|---|---:|---|
| Supported manual behavior labels | 35 | B01–B35; no label added merely for a setup/cleanup assertion |
| Manual workflow outcomes | 32 | 31 directly completed workflows and 1 composite persistence/query outcome |
| Passing formal scenarios | 31 | 30 direct Cucumber cases plus 1 Playwright BDD case; each linked report has one all-passed scenario |
| Passing direct prompt execution | 1 | The original standalone configured default-agent check |
| Successful selected execution records | 32 | 31 formal reports + 1 direct receipt, not report wrappers or hooks |
| Authored feature files | 32 | 31 have a passing selected report; persisted-agent.feature's combined create/restart/query execution failed |
| Original bundled checklist entries | 97 | Original IDs retained for planning; independent of the manual-label count |
| Original entries with partial manual evidence | 28 | Previously 24; four missed overlaps are now credited |
| Original entries without matching scenario credit | 69 | Previously 73; no failure inferred from absence of evidence |
| Complete original-contract acceptance claims | 0 | Exact bundled residual predicates remain; this does not invalidate the 35 narrower manual behavior claims |

B05–B08 are four behavior labels from one KB scenario. B03's composite proof does not turn its failed original feature into a passing execution. There are 35 label references to 32 unique primary receipts. All linked receipts/supporting raw files exist; all 31 selected formal reports have passing scenario steps/hooks. The saved report-tree inspection found no additional uncatalogued passing report. These counts are backed by [the machine-readable evidence index](coverage-audit-2026-10-27.json), including file hashes and report statuses.

## Corrected coverage

| Original entry | Existing evidence now credited | Still unproved |
|---|---|---|
| U05.chat.agui_spec.custom | B18 browser request actually uses native /api/chat/completion with custom agent and agui_spec | Original same-profile two-turn fact-follow-up parity |
| U24.shipped-ui.custom | B18 shipped custom-agent selection, prompt, stream, rendered controls and continued answer | Original conversational fact turns and saved-history reload |
| U16.knowledge-adapter-contract | B26 retained Surreal agent/KB/document and cited private knowledge across process restart |Full logical/job/chunk/vector comparison, cohort cleanup, PostgreSQL parity |
| U25.context-threshold-restart | B23/B24 none-strategy unchanged context and cold fact recall |Five other strategies and the exact original warm-before-restart sequence |

The catalog also now credits B04's asserted saved-agent roster listing and B09's team creation/exact initial membership within their existing labels. These do not add scenarios. B12 already includes custom-agent deletion; repeated cleanup does not count it again.

U05.openai.agui_spec.custom now names the later multi-turn, summary and restart evidence B19–B24, not just the single A2UI publication. U14 now includes B33–B35 (selective unlink, disable, re-enable). Neither expansion creates a new original ID.

## Corrected claim boundaries

- **Budget reduction:** B21 proves final input fitting and preserved facts. Its structural receipt reports changed=false, source=prepared=12869 and final=4830; it does not prove truncate_middle performed a structural transformation. B22 separately proves real summarization. The original approximate-count/fit_guarantee=false limit remains.
- **Protected agents:** Original U04 has one exposed definition-delete path. Both refusals were observed; remove the unsupported suggestion that unspecified additional management paths remain. Post-refusal prompting, restart/uniqueness and subsequent prompting still remain for that bundled case.
- **Orchestrator replay:** Original U07 explicitly excludes browser rendering. Remove the unrelated browser-indicator gap; exact event replay/idempotency and second-selection predicates remain.
- **Team work:** B09 already proves selected result artifact IDs reaching the final writer and settled calls. It does not prove rejection of deliberately premature join admission, team_roster round trips or recovery.
- **Knowledge update:** The original remaining update behavior is deletion of an old document followed by upload of a replacement, with only the new private fact retrieved/cited. The audit does not invent a PUT-document API requirement.

## Scope and maintenance

Maintain [behavior-status.md](behavior-status.md) as the single current index; [macos-behavior-ledger.md](macos-behavior-ledger.md) keeps chronological evidence. Older 11/86 and 24/73 summary snapshots are superseded, not deleted. Raw receipts and failed reports were not modified. Historical scope remains Mac with the recorded runtime/store/model; Linux, iOS and PostgreSQL parity are not established. No current-checkout regression claim follows from this audit.

Independent source/evidence reports: [early cases](coverage-audit-2026-10-27-early.md), [late cases](coverage-audit-2026-10-27-late.md), [original routes and predicates](coverage-audit-2026-10-27-mapping.md). Inspection used source reads and standard-library parsing of existing JSON. No tests, service/API calls, build, credential work, publication, source repair or canonical KBD transition occurred. Original repetition/signature/authentication ceremonies are not reinstated.

## Final independent review

An artifact-only critic inspected the maintained status, this summary, the JSON index, all three audit reports and the original scenario manifest. It reported no material findings: 35 labels / 32 outcomes, 31 formal successes + 1 direct prompt, and 28 partial + 69 unverified original IDs reconcile. This review checked documentation consistency; it did not revalidate current behavior or independently establish raw receipt authenticity. No test or runtime operation was executed.
