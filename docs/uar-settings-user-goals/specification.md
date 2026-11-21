# UAR User Goals & Settings Specification

Revision 0.1 · 19 November 2026 · Proposed specification, not implementation certification

## Start with the person, not the panel

A person should be able to set up their working world: choose the model for a conversation, equip an agent with the right knowledge and skills, establish shared defaults, and keep personal history private. An agent should be able to explain the configuration it actually used. A successful Save notification is not enough.

This specification covers all 30 exposed Settings families, 19 administration surfaces, chat, developer-only controls and relevant backend schema surfaces identified in the source inventory. It describes intended behavior and meaningful use cases. It is not an adopted implementation contract, a claim that every control currently works, or a demand for an 80-scenario release gate. The source snapshot is f2c9e36e. Fine-grained dialogs and every individual setting consumer have not been exhaustively qualified.

## Three different things must stay different

**Preferences** express what someone wants: a default model, enabled skill or presentation style. **Policy** determines what is permitted: provider eligibility, data access, tool grants and spending ceilings. **Private records** are the content itself: history, memories, documents and credentials. A preference never grants access. A global default never makes personal records globally readable.

## Scope contract

G = global deployment defaults or operator policy. U = authenticated My Defaults. A = an agent. T = an agent team. C = a conversation/session. KB = an independently owned knowledge-base resource. These symbols describe proposed valid scopes, not proof of present support.

R-01 · Resolve identity and permission on the server before reading records or accepting mutations. With JWT, use the verified user and tenant, not client-supplied user IDs. Without JWT, clearly identify the deployment’s actual operator/identity boundary; do not invent a shared personal identity or anonymous access grant.

R-02 · Resolve each setting by its own supported scope rules. There is no universal precedence ladder. For model selection, the proposed order is explicit conversation override, applicable agent/team assignment, personal default where permitted, then system default. Conflicts between team and agent assignments must be defined by that run’s ownership contract and shown before launch. Policy filters every candidate.

R-03 · Show requested value, effective value, origin, restrictions and activation timing. Label live, next-turn, new-run, restart-required, reindex-required and unavailable distinctly. Clearing an override previews the inherited result. Saving a record must not claim runtime activation.

R-04 · An agent’s effective-configuration report contains the actual provider/model, enabled skills, permitted KBs, context strategy and provenance, but never credentials or unauthorized records. Resolve again at the operation boundary when policy or membership changes.

R-05 · Every visible editing control has an operative path and a visible result. An unsupported surface is hidden or explicitly read-only with its actual source. Do not silently delete underlying configuration or unrelated working features.

## The core workflows

R-06 · **One model picker, wherever a model is chosen.** Search a provider-grouped catalog by recognizable name. Filter by capability and availability; explain unavailable choices. Use the same component for the system default, personal default, agent/team assignment, conversation override and supported model-specific features. No raw provider/model ID typing for normal use. Advanced identifiers remain inspectable. The provider banner must report the authoritative default, not the first catalog entry.

R-07 · **Change the system default without restarting the world.** An authorized operator uses the dedicated live configuration path. Show confirmation only after registry/runtime acknowledgement. New eligible turns follow the new default; explicit chat overrides remain in effect. Do not silently interrupt in-flight generation. A chat override changes that conversation, not the whole server. Always show which provider/model actually served the turn, including approved fallback.

R-08 · **Embeddings belong to indexes.** Put authoritative embedding configuration in one discoverable place. A personal preference can select an eligible embedding profile for resources the user may create or own; it cannot re-embed another owner’s shared KB. Show model, dimensions and index version/readiness. Changing the default affects eligible future work; existing vectors retain their original embedding contract until an explicit migration/reindex completes. Never mix incompatible vectors silently.

R-09 · **Knowledge bases are named resources, not magic text boxes.** Create/manage a KB centrally, process authorized documents, display readiness and assign it to an agent, team or conversation. Permission and assignment are separate. Retrieval returns evidence/citations from authorized sources. Removing an assignment blocks subsequent use through that assignment without deleting shared resources. Already generated history remains governed by its own retention rules.

R-10 · **Chunking is understandable.** Offer sensible supported defaults, explain the strategy in plain language, and preview a representative authorized document. Advanced size, overlap and semantic thresholds show units and constraints. Display the index rebuild consequence before committing. A user default is a preference for eligible new work, not authority over existing shared content.

R-11 · **Skills are visible at the point of work.** A Skills control in chat, Skills sections on agents/teams, a global catalog and My Defaults show enabled, inherited, disabled and unavailable states. Matching selects only from the allowed enabled catalog. Explain the selected skill and why; keep algorithm parameters advanced. Enabling a skill does not grant its preferred tools, model, network or filesystem privileges.

R-12 · **History and memory have owners.** Separate capture/injection preferences from records and sharing grants. Show which permitted sources contributed context. Conversation, agent and team scope cannot silently pool everyone’s memories. Changing a preference does not erase records or retroactively rewrite history. Deletion, export and sharing remain explicit operations with the required authority.

R-13 · **Progress is truthful and recoverable.** Long indexing, ingestion and approval operations show their real state and failures. SSE can deliver live status when the server supports it; reconnect reconciles against authoritative state, not invented success. Loading, disabled, unavailable and denied are distinct. Keyboard and screen-reader users can complete the same workflows.

## Evidence: what is uncomfortable today

The visible UI is ahead of some runtime wiring. Source inspection found rejected duplicate RAG embedding writes, a generic provider setting with boot-only hydration, a default-provider banner reading the first model rather than the effective default, and four namespaces without established settings consumers. Namespace availability is not proof that a control works. These gaps directly undermine the user’s ability to configure the system; they are not merely cosmetic defects.

Source findings are not fresh runtime reproductions. Eleven families include boot-configured fields rather than demonstrated Settings-DB consumers. Some families mix operative and non-operative fields; classifications below must not be generalized to every key. Existing evidence is preserved and only affected behavior should be requalified when implementation changes.

## Complete settings-family inventory

### F-01 · Models & embeddings (`llm`)

Actor: User, agent owner, operator.

Proposed scope: G / U / A / C preferences; credentials and embedding resources restricted.

Visible effect / intent: Choose a provider-grouped model, change the live default, or override one chat. Display requested and effective model, source and activation timing. Embedding settings describe target index consequences.

Current source evidence: Mixed: generic default-provider hydration is boot-time; llm.model database value lacks a consumer; llm.embedding.* has a validated live-swap path. Other knobs need per-key tracing.

Behavior case F-01: A user chooses a reasoning model for one complex chat; the next eligible turn reports that model while other chats retain their defaults. Governing rules: R-01–R-05; applicable workflow rules R-06–R-13.

### F-02 · Provider connections (`provider`)

Actor: Operator; users with delegated connection authority.

Proposed scope: G connection; U only for explicitly owned connections.

Visible effect / intent: Manage protocol, endpoints and secret references separately from model selection. Show configured, healthy and eligible states.

Current source evidence: Provider service and registry exist; source analysis identifies an existing-YAML merge defect. Persistence does not prove live registry state.

Behavior case F-02: An operator adds a provider and sees whether it is usable before an agent can select its models. Governing rules: R-01–R-05; applicable workflow rules R-06–R-13.

### F-03 · Vision (`vision`)

Actor: User / agent owner when supported.

Proposed scope: G / U / A / C only after runtime support.

Visible effect / intent: Offer image capability only when the effective model/runtime supports it; otherwise explain unavailability.

Current source evidence: Model override and auto-detect controls exist; no settings consumer established. Hide unsupported editing.

Behavior case F-03: An agent receiving an image must disclose that its current model cannot process it rather than pretend a vision toggle enabled it. Governing rules: R-01–R-05; applicable workflow rules R-06–R-13.

### F-04 · Context configuration (legacy surface) (`context_management`)

Actor: Operator.

Proposed scope: Read-only until connected; no promised overrides.

Visible effect / intent: Explain context capacity and summarization; do not maintain a second inert configuration surface.

Current source evidence: Strategy, token/message thresholds and summarizer controls exist without an established consumer.

Behavior case F-04: A user cannot be told a larger context limit is active merely because Save succeeded. Governing rules: R-01–R-05; applicable workflow rules R-06–R-13.

### F-05 · Conversation context strategy (`context_strategy`)

Actor: User / agent owner.

Proposed scope: G / U / A / C where supported.

Visible effect / intent: Use plain-language supported trimming strategies, visible consequences and advanced configuration.

Current source evidence: Runtime conversation trimming consumes this namespace; exact per-scope behavior remains to be implemented or qualified.

Behavior case F-05: A long conversation shows when older context is summarized or omitted and preserves the current task. Governing rules: R-01–R-05; applicable workflow rules R-06–R-13.

### F-06 · Chunking & retrieval preparation (`rag`)

Actor: Knowledge-base owner; authorized user.

Proposed scope: G / U defaults; KB resource configuration; A / C selection of compatible resources.

Visible effect / intent: Preview chunks on an authorized sample; show strategy, size and overlap, plus reindex impact. Link to authoritative embedding settings.

Current source evidence: Chunking consumers exist. rag.embedding_* writes are explicitly rejected and can reject co-edited values.

Behavior case F-06: A KB owner previews a contract split before committing a new indexing configuration; existing vectors are not portrayed as changed. Governing rules: R-01–R-05; applicable workflow rules R-06–R-13.

### F-07 · Knowledge-base defaults (`knowledge_bases`)

Actor: User / KB owner / agent owner.

Proposed scope: G / U defaults; KB ownership; A / T / C assignments.

Visible effect / intent: Manage named KBs centrally; assign allowed ready KBs to agents/teams/chats; show requested and effective assignments.

Current source evidence: Mixed defaults and real resource records; all default-field effects are not independently traced.

Behavior case F-07: An agent assigned Product Manual retrieves cited material from that KB; removing its assignment affects subsequent retrieval without deleting shared documents. Governing rules: R-01–R-05; applicable workflow rules R-06–R-13.

### F-08 · Memory behavior & storage (`memory`)

Actor: User for personal behavior; operator for storage.

Proposed scope: G policy/default; U private records; A / T / C explicitly authorized use.

Visible effect / intent: Separate capture/injection preferences and relevance budget from storage credentials and shared namespaces. Show restart-required changes honestly.

Current source evidence: Memory service startup configuration and explicit restart-required handling exist; not a blanket hot-swap path.

Behavior case F-08: Two signed-in users can choose memory differently without exposing either user’s private records to the other. Governing rules: R-01–R-05; applicable workflow rules R-06–R-13.

### F-09 · Local model files (`models`)

Actor: Operator.

Proposed scope: G resource configuration; authorized resource owner.

Visible effect / intent: Manage model/tokenizer directories and matching thresholds, separate from the provider catalog picker.

Current source evidence: Exposed directory and vector-match controls; consumers not independently traced in this review.

Behavior case F-09: A user selecting a model sees availability, not an editable server filesystem path. Governing rules: R-01–R-05; applicable workflow rules R-06–R-13.

### F-10 · File intake (`file_processing`)

Actor: Operator; uploading user.

Proposed scope: G capability/policy; U / A / C preferences within limits.

Visible effect / intent: Show supported formats and size/count limits before upload; report actual processing status.

Current source evidence: YAML-at-boot path; Settings-DB edits not established as operative.

Behavior case F-10: A user uploading a large scan sees the actual limit and a useful correction, not an upload silently governed by a different value. Governing rules: R-01–R-05; applicable workflow rules R-06–R-13.

### F-11 · Unstructured integration (`unstructured`)

Actor: Operator.

Proposed scope: G approved integration.

Visible effect / intent: Configure endpoint and secret reference privately; report effective ingestion availability.

Current source evidence: YAML ingestion configuration, not established as DB-driven.

Behavior case F-11: A KB owner sees that the required parser is unavailable before assuming an upload is searchable. Governing rules: R-01–R-05; applicable workflow rules R-06–R-13.

### F-12 · OCR model integration (`mistral_ocr`)

Actor: Operator; authorized resource owner.

Proposed scope: G integration; KB processing choice when supported.

Visible effect / intent: Use a supported model picker and secret reference; distinguish provider processing from local processing.

Current source evidence: API key/model controls are backed by boot-time YAML ingestion configuration.

Behavior case F-12: A user handling restricted documents sees whether OCR would leave the approved environment. Governing rules: R-01–R-05; applicable workflow rules R-06–R-13.

### F-13 · Document extraction (`kreuzberg`)

Actor: Operator / KB owner.

Proposed scope: G processing defaults; KB resource config.

Visible effect / intent: Explain OCR, language, tables, metadata and output; avoid conflicting chunk policies in two places.

Current source evidence: Boot-time YAML extraction/OCR configuration; no established settings-DB activation.

Behavior case F-13: A scanned table uses the selected extractor, and the KB preview shows actual extracted text before indexing. Governing rules: R-01–R-05; applicable workflow rules R-06–R-13.

### F-14 · Reliability & retry limits (`resilience`)

Actor: Operator; agent owner within policy.

Proposed scope: G ceiling; A / T / C narrower budgets where supported.

Visible effect / intent: Offer meaningful limits with advanced retry details; expose remaining deadline and terminal failure.

Current source evidence: Policy consumers exist; not every rate, delay, backoff and retry key has been independently traced.

Behavior case F-14: A throttled provider retries within the remaining budget without replaying an already completed external action. Governing rules: R-01–R-05; applicable workflow rules R-06–R-13.

### F-15 · Server operation (`server`)

Actor: Operator.

Proposed scope: G deployment only.

Visible effect / intent: Show actual address, ports and logging source; separate saved configuration from running process state.

Current source evidence: YAML boot path; legacy shutdown timeout explicitly unused.

Behavior case F-15: An operator sees a restart requirement instead of believing a saved port edit moved the live server. Governing rules: R-01–R-05; applicable workflow rules R-06–R-13.

### F-16 · Storage infrastructure (`persistence`)

Actor: Operator.

Proposed scope: G deployment resource.

Visible effect / intent: Display effective storage backend and health; protect credentials; never offer per-user database replacement as a preference.

Current source evidence: YAML boot configuration, including database, vector and cache settings.

Behavior case F-16: A user changes personal memory capture without changing everyone’s database or vector dimensions. Governing rules: R-01–R-05; applicable workflow rules R-06–R-13.

### F-17 · Execution environment (`sandbox`)

Actor: Operator; agent chooses eligible capabilities.

Proposed scope: G infrastructure policy; A / T / C constraints only.

Visible effect / intent: Show the effective runner and limits; unavailable environments remain unavailable.

Current source evidence: Runtime selection is environment-driven; no consumer established for the editable settings namespace.

Behavior case F-17: An agent denied network access cannot gain it by toggling its sandbox preference. Governing rules: R-01–R-05; applicable workflow rules R-06–R-13.

### F-18 · Task routing (`intent_classifier`)

Actor: Operator / agent owner.

Proposed scope: G defaults; A / T configuration within policy.

Visible effect / intent: Describe routing intent, confidence and alternatives; put numeric thresholds in advanced controls.

Current source evidence: Runtime classifier exists; per-key hot activation is unconfirmed.

Behavior case F-18: An ambiguous task shows the selected route and uncertainty rather than silently launching the wrong workflow. Governing rules: R-01–R-05; applicable workflow rules R-06–R-13.

### F-19 · Authentication & mutation authority (`security`)

Actor: Operator/security administrator.

Proposed scope: G policy only.

Visible effect / intent: Show verified identity and enforcement mode; keep JWT and mutation controls outside ordinary user preferences.

Current source evidence: YAML boot/security boundary. Optional JWT does not remove authorization requirements.

Behavior case F-19: An authenticated user can save My Defaults but cannot change tenant-wide authentication. Governing rules: R-01–R-05; applicable workflow rules R-06–R-13.

### F-20 · Allowed actions & approvals (`governance`)

Actor: Policy administrator; agent subject to policy.

Proposed scope: G / tenant policy; narrower A / T / C restrictions.

Visible effect / intent: Explain what needs approval and why; preferences cannot expand allowed actions.

Current source evidence: Explicit live governance path exists; individual policy outcomes still require behavior evidence.

Behavior case F-20: An agent proposes a restricted action and waits for the correct approval rather than treating a skill toggle as permission. Governing rules: R-01–R-05; applicable workflow rules R-06–R-13.

### F-21 · Response review policy (`sycophancy`)

Actor: Operator / policy owner.

Proposed scope: G policy; narrower supported agent choices.

Visible effect / intent: Explain review behavior and log-only versus enforcing mode without implying an unsupported live switch.

Current source evidence: YAML boot consumer.

Behavior case F-21: A response flagged for unsupported agreement is handled according to the effective review policy, not the last saved inert setting. Governing rules: R-01–R-05; applicable workflow rules R-06–R-13.

### F-22 · Agent behavior defaults (`agent_config`)

Actor: Agent/team owner.

Proposed scope: G defaults; U defaults for owned resources; A / T; C overrides when supported.

Visible effect / intent: Edit agent context, orchestration, governance, tools, budgets and resilience in one authoritative place.

Current source evidence: Agent-default controls duplicate agent editor; full per-key effect unconfirmed.

Behavior case F-22: An owner changes a team’s tool budget and sees which new runs inherit it and which existing run remains unchanged. Governing rules: R-01–R-05; applicable workflow rules R-06–R-13.

### F-23 · Presentation ceiling (`presentation_policy`)

Actor: Operator; user chooses within ceiling.

Proposed scope: G ceiling; U / A / T / C preferences.

Visible effect / intent: Separate permitted templates from preferred format; explain when policy constrains a choice.

Current source evidence: Dedicated global assignment/run-policy route requires admin authority.

Behavior case F-23: A chat requests a rich presentation but receives an allowed format with a clear policy reason. Governing rules: R-01–R-05; applicable workflow rules R-06–R-13.

### F-24 · Skill activation (`skill_config`)

Actor: User / agent/team owner.

Proposed scope: G catalog/defaults; U / A / T / C enablement.

Visible effect / intent: Show named enabled, inherited, disabled and unavailable skills; explain selection; keep permission checks separate.

Current source evidence: Skill configuration/reconciliation exists; exact per-field activation not fully traced.

Behavior case F-24: A user enables a writing skill for this chat; the agent names the chosen skill, and other chats do not silently change. Governing rules: R-01–R-05; applicable workflow rules R-06–R-13.

### F-25 · Native tool access (`native_tools`)

Actor: Operator; agent/user within grants.

Proposed scope: G capability boundary; A / T / C narrowing.

Visible effect / intent: Show filesystem, web and terminal permissions and session-search limits without revealing private paths to unauthorized users.

Current source evidence: Native tool controls exist; per-key timing not independently verified.

Behavior case F-25: A team can read its allowed project but cannot search a different user’s private chat history. Governing rules: R-01–R-05; applicable workflow rules R-06–R-13.

### F-26 · Automatic skill improvement (`skill_evolution`)

Actor: Operator; authorized skill owner.

Proposed scope: Unsupported editor until connected; future policy-owned scope.

Visible effect / intent: Explain proposed reflection/update/delete behavior and require actual implementation before enabling controls.

Current source evidence: Runtime initializes defaults; no settings consumer established.

Behavior case F-26: A user cannot be told that automatic skill deletion is disabled by a setting the runtime never reads. Governing rules: R-01–R-05; applicable workflow rules R-06–R-13.

### F-27 · ACP connectivity (`acp`)

Actor: Operator.

Proposed scope: G protocol deployment.

Visible effect / intent: Report real endpoint availability, authentication and session lifetime; distinguish saved settings from running service.

Current source evidence: YAML boot path.

Behavior case F-27: A coding client sees a reachable authorized connection rather than a toggle that has not started any endpoint. Governing rules: R-01–R-05; applicable workflow rules R-06–R-13.

### F-28 · Model failover (`llm_failover`)

Actor: Operator; user selects allowed preferences.

Proposed scope: G routing policy; A / T / C authorized candidate set.

Visible effect / intent: Show eligible fallback models and restrictions; no implicit cross-provider data sharing.

Current source evidence: YAML boot controls; not an established live Settings-DB switch.

Behavior case F-28: A failed primary may use only an authorized alternative; the turn reports the actual provider/model used. Governing rules: R-01–R-05; applicable workflow rules R-06–R-13.

### F-29 · Prompt caching (`prompt_caching`)

Actor: Operator / user within privacy policy.

Proposed scope: G policy; U / A / T / C preferences where supported.

Visible effect / intent: Explain cache scope, lifetime and provider implications; cache is not shared personal memory.

Current source evidence: Dedicated cache settings exist; exact activation not independently traced.

Behavior case F-29: A user disables eligible caching for a sensitive chat without assuming the provider’s retention policy changed. Governing rules: R-01–R-05; applicable workflow rules R-06–R-13.

### F-30 · My Defaults (`user_settings`)

Actor: Verified authenticated user.

Proposed scope: U; anonymous deployment/operator scope separately labeled.

Visible effect / intent: Support applicable preference families with server-enforced user/tenant identity; show inherit/on/off semantics.

Current source evidence: Dedicated user settings API/UI exists; not evidence that every requested family is implemented.

Behavior case F-30: A signed-in user saves preferences, signs out, and another user cannot inherit the first user’s private history or personal defaults. Governing rules: R-01–R-05; applicable workflow rules R-06–R-13.

## Navigation and management coverage

These are exposed routes, not a claim that every action is qualified. All must be discoverable to the appropriate actor, searchable by purpose and absent or read-only when unauthorized.

### M-01 · Run cockpit (`runtime`)
Actor: Operator / authorized user. Case: Observe runtime availability and eligible controls; show real connection state. Governing rules R-01, R-03, R-05.

### M-02 · Runs (`runs`)
Actor: Run owner / authorized supervisor. Case: Inspect progress, cancellation and outcomes without gaining another user’s private payloads. Governing rules R-01, R-03, R-05.

### M-03 · Approvals (`approvals`)
Actor: Authorized approver. Case: Review a specific action with destination, effect and current authority; stale approval cannot authorize a different action. Governing rules R-01, R-03, R-05.

### M-04 · Protocols (`protocols`)
Actor: Operator. Case: Inspect effective protocol bindings and authentication, not simulated connectivity. Governing rules R-01, R-03, R-05.

### M-05 · Providers (`providers`)
Actor: Operator / delegated owner. Case: Manage eligible connections and the live authoritative default. Governing rules R-01, R-03, R-05.

### M-06 · Credentials (`credentials`)
Actor: Credential owner / operator. Case: Create or rotate authorized references with masked values; agents never receive raw secret values through settings reports. Governing rules R-01, R-03, R-05.

### M-07 · Model catalog (`models`)
Actor: User / operator. Case: Search provider-grouped models and capabilities, availability and effective selection. Governing rules R-01, R-03, R-05.

### M-08 · Skills (`skills`)
Actor: Skill owner / user. Case: Import, create, edit, enable and delete within authority; inspect title, version, keywords, preferred tools, model override, description and prompt overlay. Governing rules R-01, R-03, R-05.

### M-09 · Presentations (`presentations`)
Actor: User / template owner. Case: Choose a template within the global ceiling; distinguish assignment from permission. Governing rules R-01, R-03, R-05.

### M-10 · Agents (`agents`)
Actor: Agent owner. Case: Configure behavior and resource bindings; identify activation on new versus existing runs. Governing rules R-01, R-03, R-05.

### M-11 · Agent teams (`teams`)
Actor: Team owner. Case: Assign members and shared resources explicitly; team membership does not merge private memories. Governing rules R-01, R-03, R-05.

### M-12 · Tools (`tools`)
Actor: Operator / authorized user. Case: Discover MCP/native capabilities and grants; unavailable tools explain their cause. Governing rules R-01, R-03, R-05.

### M-13 · Authentication (`auth`)
Actor: Operator/security administrator. Case: Inspect identity and enforcement; personal preference cannot disable policy. Governing rules R-01, R-03, R-05.

### M-14 · Knowledge (`knowledge`)
Actor: KB owner / authorized user. Case: Manage documents, indexing/readiness, assignments and citations. Governing rules R-01, R-03, R-05.

### M-15 · Memory records (`memory`)
Actor: Record owner / explicitly authorized user. Case: Inspect, correct or remove permitted memory; distinguish record lifecycle from capture preferences. Governing rules R-01, R-03, R-05.

### M-16 · Skill/package compiler (`compiler`)
Actor: Authorized author. Case: Prepare a valid skill/package with clear diagnostics; compilation is not deployment or permission. Governing rules R-01, R-03, R-05.

### M-17 · Settings (`settings`)
Actor: Actor appropriate to each family. Case: Discover supported controls and their source, scope, activation and effect. Governing rules R-01, R-03, R-05.

### M-18 · MCP health (`mcp-health`)
Actor: Operator / authorized user. Case: Diagnose approved tool connections with redacted health details. Governing rules R-01, R-03, R-05.

### M-19 · Cost & budgets (`cost`)
Actor: Owner / authorized supervisor. Case: See usage attributed to scope; distinguish estimates, actual costs and enforced limits. Governing rules R-01, R-03, R-05.

### Additional entry points and backend-only surfaces

M-20 · Chat (`/threads`): the user sees conversation model, skills and resources together with effective inheritance. Case: override one chat, clear the override and see the restored effective value without modifying other chats. Rules R-02, R-06, R-11, R-12.

M-21 · About (`/about`): display actual version/build and relevant diagnostics without secrets. Case: a user reports a settings mismatch with enough version information to reproduce it. Rules R-03, R-05.

M-22 · Developer A2UI testing: explicitly development-only, not a production capability or customer requirement. Case: a developer inspects a synthetic UI component without introducing live mutations. Rules R-01, R-05.

B-01 · Harness assembly/skill activation/reattachment belongs to agent/runtime setup. Case: an agent resumes an authorized session with its actual configuration and does not gain grants from reattachment.

B-02 · `skills.matching.engine` belongs to Skills advanced matching, not a new confusing Settings destination. Case: the selected algorithm chooses only eligible enabled skills and explains its result.

B-03 · `mcp.servers` belongs to Tools/Connections and MCP Health. Case: a tool can be configured but remains unavailable until its connection and authorization are valid.

B-04 · `tools.definitions` belongs to the permitted tool catalog. Case: adding a definition does not automatically enable the tool for every user or agent.

B-05 · `run_policy.global` belongs to operator policy. Case: a personal preference cannot raise a global action or budget ceiling. Backend schema presence alone does not prove any of these controls is exposed or operative. Rules R-01–R-05, R-11.

## Implementation plan: the shortest useful route

1. **Map control to effect.** For each existing exposed control, identify its authoritative owner, supported scopes, read/write path, runtime consumer and activation timing. Reuse the present routes and services; do not redesign the build or release process. Inert fields become truthful read-only operational information or disappear from editing until wired.
2. **Deliver one coherent configuration experience.** Implement the shared catalog picker, authoritative live system default, conversation override and effective model reporting. Include authenticated personal defaults and agent/team assignment where supported, with explicit conflict handling. This is the first useful outcome, not completion of all families.
3. **Connect resources coherently.** Make embeddings discoverable; remove duplicate inert RAG selectors while preserving chunking. Add named KB assignment/readiness, authorized chunk preview, visible scope-aware skill enablement and matching explanations. Use existing resource permissions and explicit reindex operations.
4. **Make private context safe and legible.** Apply user/session/agent/team boundaries to history and memory preferences and records. Make restart-required storage changes clear. Do not mistake a settings UI change for a new security boundary.
5. **Close the remaining exposed surface.** Resolve each inventory row to operative, restricted read-only or not offered. Give appropriate actors navigation to protocols, authentication, cost, memory, compiler, credentials, models and MCP health; those eight pages lacked navigation/palette/in-app discoverability in the inspected source. Runtime and approvals already had links despite lacking primary navigation.
6. **Qualify the meaningful workflow, then stop.** At the authorized phase boundary use end-to-end behavior evidence tied to the changed outcome: user can configure the intended scope, agent uses the effective value, unauthorized records remain inaccessible, and reset/reload preserves the correct result. Preserve accepted evidence. After an observed failure, repair and rerun only the affected authorized slice. Do not turn this inventory into mandatory tests for every field or an endless review loop.

## Delivery and acceptance examples

**Model selection:** An operator sets a system default; a user applies a chat override; the next eligible turn reports the actual model. Clearing the override shows the inherited effective value. Another chat retains its own valid configuration.

**Knowledge and skills:** An owner assigns a ready KB and enables a skill for an agent. The agent retrieves cited authorized evidence and names the skill used. An unavailable skill explains its missing capability; it does not bypass policy.

**Private context:** Two authenticated users have different defaults and private memories. Each sees only authorized history/memory; a shared agent does not collapse those boundaries. A non-JWT deployment reports its real identity model rather than promising per-user isolation it does not provide.

**Resource change:** A KB owner previews chunking or selects a new embedding profile. The interface explains which index changes, when it becomes usable, and what remains on the prior configuration. A personal preference cannot silently rebuild a shared KB.

These examples are a proposed acceptance focus, not adopted execution authority. Counts are coverage indexes, not a new release gate. No production code, runtime configuration or deployment was changed to produce this document.

## Walkthroughs: how the target experience should feel

**Not a shipped how-to.** These walkthroughs specify the intended experience. The evidence inventory identifies what currently exists and what is not yet operative. Screens, labels and scope support must be confirmed in the released build before these become product instructions.

**Choose the right model for this conversation.** Prerequisite: an authorized available provider and a supported chat override. Open the conversation model control, search by recognizable model name, inspect capabilities and select. Success is the next eligible response naming that actual model, not just a selected label. If denied or unavailable, keep the previous effective setting and show a usable reason. Choose Inherit to see and restore the valid default.

**Equip an agent for a document task.** Prerequisite: access to a ready, indexed KB and an allowed skill. Assign the named KB and enable the skill on that agent. Ask a realistic question whose answer is in the corpus. Success is relevant cited evidence from that KB and an explicit selected skill. If indexing is pending, show progress and allow retry when ready; do not invent a corpus-backed answer. Removing assignment stops future use through that binding.

**Make it personal without making it public.** Prerequisite: verified identity and supported personal defaults. Open My Defaults, choose applicable model, knowledge, chunking, skill and memory preferences, and inspect their effective scope. Success is that eligible work inherits them while another user does not acquire your records or preferences. If a shared resource constrains your choice, explain the resource owner’s configuration and the authorized alternative.

**Change an index deliberately.** Prerequisite: ownership or explicit configuration rights. Preview chunking on an authorized sample, inspect boundaries and choose an eligible embedding profile. Success means a new index becomes ready and compatible retrieval uses it; merely saving configuration does not meet the goal. On rebuild failure, show the state and any still-usable previous index, without implying a successful migration.

## Proposed VLBV behavior contracts

These are proposed contracts to adopt for the agreed value slice. They have not been executed for this specification. Use real end-to-end observations at the authorized phase boundary, not mocks or schema validity as substitutes. Each contract is parameterized by the actual supported scopes and permitted fixtures; do not add an arbitrary Cartesian matrix.

### V-01 · Model choice governs the next eligible response

Goal/rules: choose the appropriate model without changing unrelated work; R-02, R-03, R-06, R-07. Actor: authorized operator and chat user. Starting state: two eligible catalog models and two conversations, one with an explicit override. Action: change the system default, send a new message in each chat, then clear the overridden chat’s selection and send again. Observable result: the actual inference route for each response agrees with visible effective configuration; clearing restores the correct inherited value. Evidence: displayed selection/origin plus correlated turn/provider/model receipt, not a catalog label alone. Control: unavailable or unauthorized selection is rejected without falsely changing effective state. In-flight work is not silently restarted.

### V-02 · An assigned resource produces authorized evidence

Goal/rules: equip an agent with useful knowledge; R-01, R-09. Actor: KB and agent owner. Starting state: a ready synthetic corpus with a uniquely identifiable fact and a second unassigned corpus. Action: assign the named KB, ask the agent for the fact, then remove assignment and ask in a fresh eligible retrieval context. Observable result: the first response cites authorized source evidence; subsequent retrieval does not use the removed binding or the unassigned corpus. Evidence: answer/citation plus scoped retrieval receipt. Control: pending indexing is reported rather than represented as an empty or ready corpus. Existing conversation text is not falsely claimed erased.

### V-03 · Skill selection is visible and bounded

Goal/rules: use the intended skill at the current scope; R-01, R-11. Actor: user and agent. Starting state: an allowed catalog with one usable skill and one requiring an unavailable capability. Action: enable the usable skill in one conversation, perform its realistic task, inspect the selection explanation, then attempt the unavailable skill. Observable result: the chosen enabled skill affects the task and is named; another conversation remains unchanged; unavailable capability does not become authorized. Evidence: resulting artifact/action and skill-selection/grant receipt. Control: clearing the override restores the documented inherited set.

### V-04 · Private context stays private across identities

Goal/rules: personal assistance without cross-user disclosure; R-01, R-12. Actor: two distinct authenticated users and a shared agent. Starting state: separate synthetic private memories/history plus an explicitly shared item. Action: each user interacts with the same agent and requests the other’s private fact/history. Observable result: private access is denied and absent from retrieval/output; only permitted shared context is available. Evidence: server-side access decision and scoped retrieval result, not just the model declining to answer. Control: a client-supplied user identifier cannot bypass the authenticated identity. Non-JWT deployment claims must match its actual supported identity boundary.

### V-05 · Index changes mean usable new retrieval

Goal/rules: understandable chunking and embedding changes; R-08, R-10. Actor: authorized KB owner. Starting state: a ready index with a recorded embedding profile/dimensions and an authorized representative document. Action: preview a supported chunk change and request the approved reindex. Observable result: preview is understandable, migration state is visible, and compatible retrieval uses the ready new version only after completion. Evidence: preview, job/index-version receipt and cited retrieval. Control: a non-owner personal default cannot rebuild shared content; a failed rebuild does not masquerade as a successful change.

### V-06 · Every shipped control tells the truth

Goal/rules: remove dead-end configuration; R-03, R-05, R-13. Actor: appropriate user/operator. Starting state: the agreed release’s exposed family inventory with known unsupported or boot-only fields. Action: discover the feature by goal, edit an operative control, inspect effective state, and inspect a restart-required or unsupported control. Observable result: operative behavior is observable; boot-only/unsupported surfaces are accurately read-only, hidden or labeled, never false live saves. Evidence: affected UI path plus actual consumer outcome or operational-state source. Control: one invalid field does not quietly make valid co-edits appear accepted.

## Definition of Done and exit contract

The following is a reusable exit contract for the agreed user-value slice, not a declaration that this work has passed it.

| Exit dimension | Required evidence | Not sufficient |
|---|---|---|
| Agreed goal | Named person/agent can complete the intended workflow at the agreed scopes | A count of settings fields or completed tasks |
| Business rule | Relevant R/F/M/V identifiers and ownership/permission rules are explicit | A generic claim that configuration is flexible |
| Behavior | Actual end-to-end result, effective consumer and visible state agree | Save success, compile success or a mocked response |
| Privacy and authority | Relevant identity, resource and action boundaries enforced server-side | A UI hidden from one user or an LLM refusal alone |
| Guidance and recovery | Prerequisites, steps, observable success, limits and recovery match the released build | A proposed spec presented as shipped instructions |
| Availability | Intended users can reach the released feature through the established delivery process | Code on a branch or a standalone renderer demonstration |
| Remaining limits | Unsupported scopes, partial results and blockers are named; user confirms acceptance of material tradeoffs | Marking an unexercised case PASS |

**Review-ready** means the proposed specification or implementation can be reviewed: sources and gaps are clear. **Delivery complete** means the agreed workflow is available and its exit evidence has passed at the authorized VLBV boundary, with usable guidance and known limitations. Neither label proves users are happy. Invite representative users to perform the workflow and ask: Can you find it? Do you understand what changed? Does the agent do what you intended? Can you recover without specialist help?

Preserve PASS evidence unless an affected source, fixture or environment changes. Report PASS, FAIL, BLOCKED and NOT_EXERCISED distinctly. After an observed failure, obtain the required repair authorization and rerun affected behavior only. No additional tests, runtime changes or process redesign are authorized by this document.

## Seed field index: exact source labels

The following 167 unique seed keys and labels are mechanically extracted from the settings source at f2c9e36e. This is a navigation/reference appendix, not runtime qualification, not the complete nested-schema or dialog inventory, and not a test mandate. Values and secrets are excluded. Scope and activation must follow the owning family above.

| Source key | Display label |
|---|---|
| server.port | Port |
| server.host | Host |
| server.shutdown_timeout_secs | Legacy Shutdown Timeout (unused) |
| server.log_format | Log Format |
| server.grpc_port | A2A gRPC Port |
| security.jwt_required | JWT Required |
| security.jwt_secret | JWT Secret |
| security.settings_mutation_auth_required | Require Admin Key for Settings Mutation |
| resilience.rate_limit_enabled | Rate Limiting |
| resilience.requests_per_second | Req/s |
| resilience.burst_size | Burst Size |
| resilience.request_timeout_ms | Request Timeout (ms) |
| resilience.stream_start_timeout_ms | Stream Start Timeout (ms) |
| resilience.stream_idle_timeout_ms | Stream Idle Timeout (ms) |
| resilience.retries_enabled | Retries Enabled |
| resilience.retry_max_attempts | Max Retry Attempts |
| resilience.retry_base_delay_ms | Retry Base Delay (ms) |
| resilience.retry_backoff_multiplier | Retry Backoff Multiplier |
| resilience.retry_max_delay_ms | Retry Max Delay (ms) |
| resilience.retry_jitter_mode | Retry Jitter Mode |
| resilience.retry_respect_retry_after | Respect Retry-After |
| resilience.retryable_http_statuses | Retryable HTTP Statuses |
| resilience.retryable_transport_errors | Retryable Transport Errors |
| resilience.retry_budget_ms | Retry Budget (ms) |
| resilience.timeout_disabled | Timeout Disabled |
| harness.mode | Assembly Mode |
| harness.skill_activation_mode | Skill Activation Mode |
| harness.skill_reattachment | Skill Reattachment Budget |
| persistence.provider | Provider |
| persistence.database_url | Database URL |
| persistence.vector_dimension | Vector Dimension |
| persistence.external_cache_enabled | External Cache Enabled |
| persistence.surreal_user | SurrealDB User |
| persistence.surreal_pass | SurrealDB Password |
| file_processing.provider | Provider |
| file_processing.upload_dir | Upload Directory |
| file_processing.max_files_per_prompt | Max Files Per Prompt |
| file_processing.max_file_size | Max File Size |
| file_processing.max_total_size | Max Total Size |
| file_processing.allowed_mime_types | Allowed MIME Types |
| vision.model | Model Override |
| vision.auto_detect | Auto-detect |
| models.models_dir | Models Directory |
| models.vector_threshold | Vector Match Threshold |
| knowledge_bases.default | Default KB |
| knowledge_bases.named | Named KBs |
| knowledge_bases.named_keys | Named KB Keys |
| skills.matching.engine | Matching Engine |
| intent_classifier.backend | Backend |
| intent_classifier.topk | Top-K |
| intent_classifier.accept_threshold | Accept Threshold |
| intent_classifier.margin_threshold | Margin Threshold |
| intent_classifier.wasm_component_path | WASM Path |
| unstructured.api_url | API URL |
| unstructured.api_key | API Key |
| mistral_ocr.api_key | API Key |
| mistral_ocr.ocr_model | OCR Model |
| kreuzberg.ocr_enabled | OCR Enabled |
| kreuzberg.force_ocr | Force OCR |
| kreuzberg.ocr_backend | OCR Backend |
| kreuzberg.ocr_language | OCR Language |
| kreuzberg.pdf_dpi | PDF DPI |
| kreuzberg.extract_tables | Extract Tables |
| kreuzberg.extract_metadata | Extract Metadata |
| kreuzberg.output_format | Output Format |
| kreuzberg.chunking.max_characters | Chunk Max Characters |
| kreuzberg.chunking.overlap | Chunk Overlap |
| context_management.strategy | Strategy |
| context_management.max_tokens | Max Tokens |
| context_management.trigger_threshold | Trigger Threshold |
| context_management.max_messages | Max Messages |
| context_management.summary_budget | Summary Budget |
| context_management.summarization_model | Summarization Model |
| context_strategy.type | Strategy |
| context_strategy.config | Strategy Config |
| rag.chunking_strategy | Chunking Strategy |
| rag.chunk_size | Chunk Size |
| rag.chunk_tokens | Chunk Tokens |
| rag.semantic_threshold | Semantic Threshold |
| rag.embedding_provider | Embedding Provider |
| rag.embedding_model | Embedding Model |
| agent_config.orchestrated | Orchestrated Agent |
| memory.enabled | Enable Memory |
| memory.auto_capture | Auto-capture |
| memory.inject_context | Inject Context |
| memory.injection_min_similarity | Minimum Injection Similarity |
| memory.embedding_provider | Embedding Provider |
| memory.embedding_model | Embedding Model |
| memory.openai_api_key | OpenAI API Key |
| memory.cohere_api_key | Cohere API Key |
| memory.max_context_tokens | Max Context Tokens |
| memory.vector_weight | Vector Weight |
| memory.bm25_weight | BM25 Weight |
| memory.db_path | DB Path |
| memory.mcp_http_enabled | MCP HTTP Enabled |
| memory.mcp_http_path | MCP HTTP Path |
| memory.namespace | Namespace |
| memory.database | Database |
| memory.surreal_endpoint | SurrealDB Endpoint |
| memory.surreal_user | SurrealDB User |
| memory.surreal_pass | SurrealDB Password |
| llm.model | Default Model |
| llm.api_key | API Key |
| llm.base_url | Base URL |
| llm.protocol | Protocol |
| llm.timeout_secs | Timeout (s) |
| llm.max_retries | Max Retries |
| llm.parallel_tool_calls | Parallel Tool Calls |
| llm.cost_tracking | Cost Tracking |
| llm.tracing | OTEL Tracing |
| llm.cooldown_secs | Cooldown (s) |
| llm.health_check_secs | Health Check Interval (s) |
| llm.thinking_budget | Thinking Budget |
| llm.cache | Response Cache |
| llm.budget | Budget |
| llm.rate_limit | Rate Limit |
| llm.default_provider | Default LLM Provider ID |
| llm.embedding.backend | Embedding Backend |
| llm.embedding.model | Embedding Model |
| llm.embedding.vector_dimension | Embedding Vector Dimension |
| llm.embedding.send_dimensions | Send Embedding Dimensions |
| llm_failover.enabled | Enabled |
| llm_failover.strategy | Strategy |
| llm_failover.error_threshold | Error Threshold |
| llm_failover.cooldown_secs | Cooldown (seconds) |
| llm_failover.fallback_models | Fallback Models |
| sandbox.runner | Runner |
| sandbox.remote_url | Remote URL |
| sandbox.default_memory_mib | Default Memory (MiB) |
| sandbox.default_timeout_secs | Default Timeout (s) |
| sandbox.network_enabled | Network Enabled |
| sandbox.max_concurrent | Max Concurrent Sessions |
| sycophancy.enabled | Enabled |
| sycophancy.strictness | Strictness |
| sycophancy.auto_correct_threshold | Auto-correct Threshold |
| sycophancy.reflect_threshold | Reflect Threshold |
| sycophancy.log_only | Log Only |
| native_tools.file_tools_enabled | File Tools Enabled |
| native_tools.file_allowed_paths | Allowed File Paths |
| native_tools.file_max_size_kb | Max Read Size (KB) |
| native_tools.file_write_max_kb | Max Write Size (KB) |
| native_tools.web_fetch_enabled | Web Fetch Enabled |
| native_tools.web_fetch_timeout_secs | Web Fetch Timeout (s) |
| native_tools.web_fetch_max_size_kb | Web Fetch Max Size (KB) |
| native_tools.web_fetch_allowed_domains | Allowed Domains |
| native_tools.terminal_exec_enabled | Terminal Exec Enabled |
| native_tools.terminal_shell | Shell |
| native_tools.terminal_timeout_secs | Terminal Timeout (s) |
| native_tools.terminal_use_sandbox | Use Sandbox |
| native_tools.session_search_enabled | Session Search Enabled |
| native_tools.session_search_max_results | Max Search Results |
| mcp.servers | Configured MCP Servers |
| tools.definitions | Stored Tool Definitions |
| skill_evolution.enabled | Enabled |
| skill_evolution.trigger_model | Trigger Model |
| skill_evolution.min_tool_calls | Min Tool Calls to Trigger |
| skill_evolution.max_skills_per_run | Max Skills per Run |
| skill_evolution.allow_update | Allow Updating Skills |
| skill_evolution.allow_deletion | Allow Deleting Skills |
| skill_evolution.min_executions_before_delete | Min Executions Before Delete |
| skill_evolution.reflection_prompt | Reflection Prompt |
| acp.enabled | Enabled |
| acp.path | Mount Path |
| acp.auth_required | Auth Required |
| acp.session_ttl_secs | Session TTL (seconds) |
| run_policy.global | Global Run Policy |
| prompt_caching.enabled | Enable Prompt Caching (Global Default) |

### Additional parseable schema properties

Supplementary literal JSON schema properties, deduplicated against the seed list. Array entries use []. Rust-composed schemas that cannot be mechanically parsed are omitted; this is not a claim of exhaustive nested-schema coverage.

| Source property | Source label |
|---|---|
| harness.skill_reattachment.per_skill_tokens | per_skill_tokens |
| harness.skill_reattachment.total_tokens | total_tokens |
| provider.id | ID |
| provider.display_name | Display Name |
| provider.base_url | Base URL |
| provider.api_key | API Key |
| provider.protocol | Protocol |
| provider.enabled | Enabled |
| provider.default_model | Default Model |
| provider.models | Models |
| provider.models[].id | id |
| provider.models[].display_name | display_name |
| provider.models[].enabled | enabled |
| provider.models[].context_window | context_window |
| provider.models[].max_output_tokens | max_output_tokens |
| provider.models[].capabilities | capabilities |
| provider.models[].capabilities.supports_vision | Vision |
| provider.models[].capabilities.supports_tools | Tool Use |
| provider.models[].capabilities.supports_reasoning | Reasoning |
| provider.models[].capabilities.supports_embeddings | Embeddings |
| provider.models[].capabilities.supports_audio_input | Audio Input |
| provider.models[].capabilities.supports_audio_output | Audio Output |
| provider.models[].capabilities.supports_image_generation | Image Generation |
| provider.models[].capabilities.supports_streaming | Streaming |
| provider.models[].capabilities.supports_parallel_tool_calls | Parallel Tools |
| provider.models[].capabilities.supports_structured_output | JSON Output |
| provider.models[].capabilities.supports_prompt_caching | Prompt Caching |
| provider.models[].capabilities.supports_fine_tuning | Fine-Tuning |
| provider.models[].pricing | pricing |
| provider.models[].pricing.input_per_1m | Input/1M |
| provider.models[].pricing.output_per_1m | Output/1M |
| provider.models[].pricing.cached_input_per_1m | Cached/1M |
| provider.models[].pricing.reasoning_per_1m | Reasoning/1M |
| provider.models[].pricing.embedding_per_1m | Embedding/1M |
| provider.models[].pricing.image_input_per_1m | Image/1M |
| provider.models[].pricing.audio_input_per_1m | Audio/1M |
| provider.models[].pricing.currency | currency |
| skill_config.enabled | Enabled |
| skill_config.description | Description (override) |
| llm_failover.fallback_models[].model | model |
| llm_failover.fallback_models[].api_key | api_key |
| llm_failover.fallback_models[].base_url | base_url |

## Source ledger and confidence

Source snapshot: UAR commit f2c9e36e, reviewed 19 November 2026. Inventory: 30 Settings namespaces; 19 administration routes; chat, About and developer-only A2UI. Detailed field-level completeness and current runtime behavior remain unverified where stated.

- Settings navigation: https://github.com/Prometheus-AGS/universal-agent-runtime/blob/f2c9e36e/frontend/src/features/settings/ui/settings-navigation.tsx
- Administration routes: https://github.com/Prometheus-AGS/universal-agent-runtime/blob/f2c9e36e/frontend/src/pages/admin-page.tsx
- Settings schema: https://github.com/Prometheus-AGS/universal-agent-runtime/blob/f2c9e36e/src/uar/settings/manager.rs
- Settings mutation behavior: https://github.com/Prometheus-AGS/universal-agent-runtime/blob/f2c9e36e/src/uar/api/settings.rs
- Provider banner: https://github.com/Prometheus-AGS/universal-agent-runtime/blob/f2c9e36e/frontend/src/features/providers/ui/providers-page.tsx
- Runtime initialization: https://github.com/Prometheus-AGS/universal-agent-runtime/blob/f2c9e36e/src/uar/runtime/manager.rs

Repository links may require authorized access. User requirements were supplied for this specification; private conversation records and operational notes are deliberately excluded. Recommendations are labeled proposed and must be adopted before implementation scope is treated as approved.

## What done looks like

Not more settings. Not more green checks. A person can set up the right model, knowledge, skills and context for the work in front of them, understand what will happen, and trust that the agent follows those choices within policy. The interface and the runtime tell the same story.
