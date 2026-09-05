import { graphStore } from "@prometheus-ags/prometheus-entity-management";
import { copyPresentationSelection, type PresentationSelectionMode } from "../presentation-assignments/contracts";
import { PRESENTATION_ADMISSION_ID, presentationListKey } from "../presentations/domain";
import { PRESENTATION_CATALOG_ENTITY, PRESENTATION_ENTITY, type Presentation, type PresentationCatalog } from "../presentations/contracts";

import {
  AGENT_SESSION_DRAFT_ENTITY,
  AGENT_SESSION_ENTITY,
  SESSION_PROMPT_CACHING_ENTITY,
} from "./contracts";
import type {
  AgentSession,
  AgentSessionConfig,
  AgentSessionDraft,
  AgentSessionField,
  SessionPromptCaching,
} from "./contracts";
import {
  fetchAgentSessionConfig,
  fetchSessionPromptCaching,
  saveAgentSessionConfig,
  SessionConfigurationSaveError,
} from "./api/session-configuration-api";

const ALL_SESSION_FIELDS: AgentSessionField[] = [
  "agent_id",
  "model",
  "tools",
  "skills",
  "knowledge_bases",
  "mcp_servers",
  "presentations",
  "tool_approval",
  "prompt_caching_enabled",
];

const draftGenerations = new Map<string, number>();
const draftSaveControllers = new Map<string, AbortController>();
const sessionMutationTails = new Map<string, Promise<void>>();
const agentSelectionGenerations = new Map<string, number>();
const confirmedAgentIds = new Map<string, string | null>();

export function agentSessionDraftId(
  sessionId: string,
  editorId: string,
): string {
  return `${sessionId}:${editorId}`;
}

function copyConfig(config: AgentSessionConfig): AgentSessionConfig {
  return {
    agent_id: config.agent_id,
    model: config.model,
    tools: config.tools ? [...config.tools] : null,
    skills: config.skills ? [...config.skills] : null,
    knowledge_bases: config.knowledge_bases
      ? [...config.knowledge_bases]
      : null,
    mcp_servers: config.mcp_servers ? [...config.mcp_servers] : null,
    presentations: copyPresentationSelection(config.presentations),
    tool_approval: config.tool_approval,
    prompt_caching_enabled: config.prompt_caching_enabled,
  };
}

function mergeConfigFields(
  base: AgentSessionConfig,
  source: AgentSessionConfig,
  fields: readonly AgentSessionField[],
): AgentSessionConfig {
  const merged = copyConfig(base);
  for (const field of fields) {
    switch (field) {
      case "agent_id":
        merged.agent_id = source.agent_id;
        break;
      case "model":
        merged.model = source.model;
        break;
      case "tools":
        merged.tools = source.tools ? [...source.tools] : null;
        break;
      case "skills":
        merged.skills = source.skills ? [...source.skills] : null;
        break;
      case "knowledge_bases":
        merged.knowledge_bases = source.knowledge_bases
          ? [...source.knowledge_bases]
          : null;
        break;
      case "mcp_servers":
        merged.mcp_servers = source.mcp_servers
          ? [...source.mcp_servers]
          : null;
        break;
      case "tool_approval":
        merged.tool_approval = source.tool_approval;
        break;
      case "presentations":
        merged.presentations = copyPresentationSelection(source.presentations);
        break;
      case "prompt_caching_enabled":
        merged.prompt_caching_enabled = source.prompt_caching_enabled;
        break;
    }
  }
  return merged;
}

function replaceSessionPromptCaching(
  effective: SessionPromptCaching,
): SessionPromptCaching {
  graphStore
    .getState()
    .replaceEntity(SESSION_PROMPT_CACHING_ENTITY, effective.id, effective);
  return effective;
}

export async function loadSessionPromptCaching(
  sessionId: string,
  signal?: AbortSignal,
): Promise<SessionPromptCaching> {
  const effective = await fetchSessionPromptCaching(sessionId, signal);
  if (signal?.aborted) throw new DOMException("Request aborted", "AbortError");
  return replaceSessionPromptCaching(effective);
}

async function enqueueSessionMutation<T>(
  sessionId: string,
  operation: () => Promise<T>,
): Promise<T> {
  const previous = sessionMutationTails.get(sessionId) ?? Promise.resolve();
  const current = previous.then(operation, operation);
  const settled = current.then(
    () => undefined,
    () => undefined,
  );
  sessionMutationTails.set(sessionId, settled);
  try {
    return await current;
  } finally {
    if (sessionMutationTails.get(sessionId) === settled) {
      sessionMutationTails.delete(sessionId);
    }
  }
}

function replaceCanonicalAgentSession(
  sessionId: string,
  config: AgentSessionConfig,
): AgentSession {
  const graph = graphStore.getState();
  const current = graph.readEntity<AgentSession>(
    AGENT_SESSION_ENTITY,
    sessionId,
  );
  const session: AgentSession = {
    ...copyConfig(config),
    id: sessionId,
    session_id: sessionId,
    revision: (current?.revision ?? 0) + 1,
  };
  graph.replaceEntity(AGENT_SESSION_ENTITY, sessionId, session);
  return session;
}

export async function loadAgentSession(
  sessionId: string,
  signal?: AbortSignal,
): Promise<AgentSession | null> {
  return enqueueSessionMutation(sessionId, async () => {
    const graph = graphStore.getState();
    const startingRevision =
      graph.readEntity<AgentSession>(AGENT_SESSION_ENTITY, sessionId)
        ?.revision ?? 0;
    const loaded = await fetchAgentSessionConfig(sessionId, signal);
    if (signal?.aborted)
      throw new DOMException("Request aborted", "AbortError");
    const current = graphStore
      .getState()
      .readEntity<AgentSession>(AGENT_SESSION_ENTITY, sessionId);
    if ((current?.revision ?? 0) !== startingRevision) return current ?? null;
    confirmedAgentIds.set(sessionId, loaded?.agent_id ?? null);
    if (loaded) return replaceCanonicalAgentSession(sessionId, loaded);
    if (current)
      graphStore.getState().removeEntity(AGENT_SESSION_ENTITY, sessionId);
    return null;
  });
}

export function openAgentSessionDraft(
  sessionId: string,
  editorId: string,
  fallback: AgentSessionConfig,
): string {
  const graph = graphStore.getState();
  const canonical = graph.readEntity<AgentSession>(
    AGENT_SESSION_ENTITY,
    sessionId,
  );
  const config = copyConfig(canonical ?? fallback);
  const id = agentSessionDraftId(sessionId, editorId);
  draftSaveControllers.get(id)?.abort();
  const generation = (draftGenerations.get(id) ?? 0) + 1;
  draftGenerations.set(id, generation);
  const draft: AgentSessionDraft = {
    ...config,
    id,
    session_id: sessionId,
    editor_id: editorId,
    generation,
    baseline_revision: canonical?.revision ?? 0,
    dirty_fields: [],
    save_status: "idle",
    error: null,
    presentation_retained_ids: [...(config.presentations?.ids ?? [])],
    presentation_owner_id: null,
    presentation_error: null,
    save_uncertain: false,
  };
  graph.replaceEntity(AGENT_SESSION_DRAFT_ENTITY, id, draft);
  return id;
}

export async function loadAndOpenAgentSessionDraft(
  sessionId: string,
  editorId: string,
  fallback: AgentSessionConfig,
  signal?: AbortSignal,
): Promise<string> {
  await Promise.all([
    loadAgentSession(sessionId, signal),
    loadSessionPromptCaching(sessionId, signal),
  ]);
  return openAgentSessionDraft(sessionId, editorId, fallback);
}

export function setAgentSessionDraftField<K extends AgentSessionField>(
  draftId: string,
  field: K,
  value: AgentSessionConfig[K],
): void {
  const graph = graphStore.getState();
  const draft = graph.readEntity<AgentSessionDraft>(
    AGENT_SESSION_DRAFT_ENTITY,
    draftId,
  );
  if (
    !draft ||
    draft.save_status === "saving" ||
    draft.save_uncertain ||
    Object.is(draft[field], value)
  )
    return;

  graph.upsertEntity(AGENT_SESSION_DRAFT_ENTITY, draftId, {
    [field]: value,
    dirty_fields: draft.dirty_fields.includes(field)
      ? draft.dirty_fields
      : [...draft.dirty_fields, field],
    save_status: "idle",
    error: null,
  });
}

export function markAgentSessionDraftSaving(
  draftId: string,
  generation: number,
): void {
  const graph = graphStore.getState();
  const draft = graph.readEntity<AgentSessionDraft>(
    AGENT_SESSION_DRAFT_ENTITY,
    draftId,
  );
  if (!draft || draft.generation !== generation) return;
  graph.upsertEntity(AGENT_SESSION_DRAFT_ENTITY, draftId, {
    save_status: "saving",
    error: null,
  });
}

export function markAgentSessionDraftError(
  draftId: string,
  error: string,
  generation?: number,
): void {
  const graph = graphStore.getState();
  const draft = graph.readEntity<AgentSessionDraft>(
    AGENT_SESSION_DRAFT_ENTITY,
    draftId,
  );
  if (!draft || (generation !== undefined && draft.generation !== generation))
    return;
  graph.upsertEntity(AGENT_SESSION_DRAFT_ENTITY, draftId, {
    save_status: "error",
    error,
  });
}

export function commitAgentSessionDraft(
  draftId: string,
  generation: number,
  sessionId: string,
  saved: AgentSessionConfig,
  changedFields: readonly AgentSessionField[],
): boolean {
  const graph = graphStore.getState();
  const draft = graph.readEntity<AgentSessionDraft>(
    AGENT_SESSION_DRAFT_ENTITY,
    draftId,
  );
  const canonical = graph.readEntity<AgentSession>(
    AGENT_SESSION_ENTITY,
    sessionId,
  );
  confirmedAgentIds.set(sessionId, saved.agent_id);
  replaceCanonicalAgentSession(
    sessionId,
    mergeConfigFields(canonical ?? saved, saved, [...changedFields, "presentations"]),
  );
  if (!draft || draft.generation !== generation) return false;
  graph.removeEntity(AGENT_SESSION_DRAFT_ENTITY, draftId);
  draftSaveControllers.delete(draftId);
  return true;
}

export function cancelAgentSessionDraft(draftId: string): void {
  draftSaveControllers.get(draftId)?.abort();
  draftSaveControllers.delete(draftId);
  graphStore.getState().removeEntity(AGENT_SESSION_DRAFT_ENTITY, draftId);
}

export function readAgentSessionDraftConfig(
  draftId: string,
): AgentSessionConfig | null {
  const draft = graphStore
    .getState()
    .readEntity<AgentSessionDraft>(AGENT_SESSION_DRAFT_ENTITY, draftId);
  return draft ? copyConfig(draft) : null;
}

export async function saveAgentSessionDraft(draftId: string): Promise<boolean> {
  const graph = graphStore.getState();
  const draft = graph.readEntity<AgentSessionDraft>(
    AGENT_SESSION_DRAFT_ENTITY,
    draftId,
  );
  if (!draft || draft.save_status === "saving" || draft.save_uncertain) return false;
  if (draft.dirty_fields.includes("presentations") && !admittedSessionPresentationDraft(graph, draftId)) {
    markAgentSessionDraftError(draftId, "Reload the Presentation catalog before saving this assignment draft.");
    return false;
  }

  const generation = draft.generation;
  const snapshot = copyConfig(draft);
  const dirtyFields = [...draft.dirty_fields];
  const controller = new AbortController();
  draftSaveControllers.get(draftId)?.abort();
  draftSaveControllers.set(draftId, controller);
  markAgentSessionDraftSaving(draftId, generation);
  try {
    return await enqueueSessionMutation(draft.session_id, async () => {
      const currentDraft = graphStore
        .getState()
        .readEntity<AgentSessionDraft>(AGENT_SESSION_DRAFT_ENTITY, draftId);
      if (
        controller.signal.aborted ||
        !currentDraft ||
        currentDraft.generation !== generation
      ) {
        return false;
      }
      const latest = await fetchAgentSessionConfig(
        draft.session_id,
        controller.signal,
      );
      const changedFields = latest ? dirtyFields : ALL_SESSION_FIELDS;
      const payload = mergeConfigFields(
        latest ?? snapshot,
        snapshot,
        changedFields,
      );
      // A preflight GET is not a write baseline. Only intentional assignment
      // edits send explicit intent; other saves use atomic host preservation.
      if (!dirtyFields.includes("presentations")) payload.presentations = null;
      if (dirtyFields.includes("presentations") && !admittedSessionPresentationDraft(graphStore.getState(), draftId)) {
        throw new Error("Presentation admission changed. Your draft is retained; reload the catalog before saving.");
      }
      const saved = await saveAgentSessionConfig(
        draft.session_id,
        payload,
        controller.signal,
      );
      const committed = commitAgentSessionDraft(
        draftId,
        generation,
        draft.session_id,
        saved,
        changedFields,
      );
      // POST already confirmed the saved configuration. A later derived read
      // failure must not turn it into an unconfirmed write or enable replay.
      try { await loadSessionPromptCaching(draft.session_id, controller.signal); }
      catch { graphStore.getState().removeEntity(SESSION_PROMPT_CACHING_ENTITY, draft.session_id); }
      return committed;
    });
  } catch (error) {
    const current = graphStore.getState().readEntity<AgentSessionDraft>(AGENT_SESSION_DRAFT_ENTITY, draftId);
    if (error instanceof SessionConfigurationSaveError && error.uncertain && current?.generation === generation) {
      graphStore.getState().upsertEntity(AGENT_SESSION_DRAFT_ENTITY, draftId, { save_uncertain: true });
    }
    if (!controller.signal.aborted) {
      markAgentSessionDraftError(draftId, (error as Error).message, generation);
    }
    return false;
  } finally {
    if (draftSaveControllers.get(draftId) === controller) {
      draftSaveControllers.delete(draftId);
    }
  }
}

export async function selectAgentForSession(
  sessionId: string,
  agentId: string,
): Promise<boolean> {
  const graph = graphStore.getState();
  const previous = graph.readEntity<AgentSession>(
    AGENT_SESSION_ENTITY,
    sessionId,
  );
  const optimisticConfig: AgentSessionConfig = previous
    ? { ...copyConfig(previous), agent_id: agentId }
    : {
        agent_id: agentId,
        model: null,
        tools: null,
        skills: null,
        knowledge_bases: null,
        mcp_servers: null,
        tool_approval: null,
        prompt_caching_enabled: null,
      };
  replaceCanonicalAgentSession(sessionId, optimisticConfig);
  const selectionGeneration =
    (agentSelectionGenerations.get(sessionId) ?? 0) + 1;
  agentSelectionGenerations.set(sessionId, selectionGeneration);
  let rollbackAgentId = confirmedAgentIds.get(sessionId);
  let rollbackKnown = confirmedAgentIds.has(sessionId);
  try {
    await enqueueSessionMutation(sessionId, async () => {
      const current = await fetchAgentSessionConfig(sessionId);
      rollbackAgentId = current?.agent_id ?? null;
      rollbackKnown = true;
      confirmedAgentIds.set(sessionId, rollbackAgentId);
      const saved = await saveAgentSessionConfig(sessionId, {
        ...copyConfig(current ?? optimisticConfig),
        agent_id: agentId,
        presentations: null,
      });
      confirmedAgentIds.set(sessionId, saved.agent_id);
      if (agentSelectionGenerations.get(sessionId) !== selectionGeneration)
        return;
      const canonical = graphStore
        .getState()
        .readEntity<AgentSession>(AGENT_SESSION_ENTITY, sessionId);
      replaceCanonicalAgentSession(
        sessionId,
        mergeConfigFields(canonical ?? saved, saved, ["agent_id"]),
      );
    });
    return true;
  } catch {
    if (agentSelectionGenerations.get(sessionId) === selectionGeneration) {
      const current = graphStore
        .getState()
        .readEntity<AgentSession>(AGENT_SESSION_ENTITY, sessionId);
      if (rollbackKnown && rollbackAgentId) {
        const rollbackSource = current
          ? { ...copyConfig(current), agent_id: rollbackAgentId }
          : { ...optimisticConfig, agent_id: rollbackAgentId };
        replaceCanonicalAgentSession(
          sessionId,
          mergeConfigFields(current ?? rollbackSource, rollbackSource, [
            "agent_id",
          ]),
        );
      } else if (rollbackKnown) {
        graphStore.getState().removeEntity(AGENT_SESSION_ENTITY, sessionId);
      } else {
        void loadAgentSession(sessionId).catch(() => undefined);
      }
    }
    return false;
  }
}

export const agentSessionDraftActions = {
  cancel: cancelAgentSessionDraft,
  commit: commitAgentSessionDraft,
  markError: markAgentSessionDraftError,
  markSaving: markAgentSessionDraftSaving,
  loadAndOpen: loadAndOpenAgentSessionDraft,
  loadSession: loadAgentSession,
  loadPromptCaching: loadSessionPromptCaching,
  open: openAgentSessionDraft,
  readConfig: readAgentSessionDraftConfig,
  save: saveAgentSessionDraft,
  selectAgent: selectAgentForSession,
  setField: setAgentSessionDraftField,
  admitPresentations: admitSessionPresentations,
  setPresentationMode,
  togglePresentation,
  resetPresentations,
  reconcileSaved: reconcileSavedSessionConfiguration,
} as const;

type GraphState = ReturnType<typeof graphStore.getState>;

export function admittedSessionPresentationDraft(state: GraphState, draftId: string): AgentSessionDraft | null {
  const catalog = state.entities[PRESENTATION_CATALOG_ENTITY]?.[PRESENTATION_ADMISSION_ID] as PresentationCatalog | undefined;
  const draft = state.entities[AGENT_SESSION_DRAFT_ENTITY]?.[draftId] as AgentSessionDraft | undefined;
  return catalog?.status === "ready" && catalog.owner_id && draft?.presentation_owner_id === catalog.owner_id
    && draft.presentation_admission_id === PRESENTATION_ADMISSION_ID ? draft : null;
}

async function admitSessionPresentations(draftId: string): Promise<void> {
  const state = graphStore.getState();
  const catalog = state.entities[PRESENTATION_CATALOG_ENTITY]?.[PRESENTATION_ADMISSION_ID] as PresentationCatalog | undefined;
  const draft = state.readEntity<AgentSessionDraft>(AGENT_SESSION_DRAFT_ENTITY, draftId);
  if (!draft || catalog?.status !== "ready" || !catalog.owner_id || admittedSessionPresentationDraft(state, draftId)) return;
  if (draft.dirty_fields.includes("presentations") || draft.save_status === "saving") return;
  let config: AgentSessionConfig | null;
  try { config = await fetchAgentSessionConfig(draft.session_id); }
  catch {
    const current = graphStore.getState();
    const currentCatalog = current.entities[PRESENTATION_CATALOG_ENTITY]?.[PRESENTATION_ADMISSION_ID] as PresentationCatalog | undefined;
    const currentDraft = current.readEntity<AgentSessionDraft>(AGENT_SESSION_DRAFT_ENTITY, draftId);
    if (currentCatalog?.generation === catalog.generation && currentCatalog.owner_id === catalog.owner_id
      && currentDraft?.generation === draft.generation && currentDraft.save_status !== "saving") {
      current.upsertEntity(AGENT_SESSION_DRAFT_ENTITY, draftId, { presentation_error: "Could not load the current assignment. Reload assignment to retry." });
    }
    return;
  }
  const current = graphStore.getState();
  const currentCatalog = current.entities[PRESENTATION_CATALOG_ENTITY]?.[PRESENTATION_ADMISSION_ID] as PresentationCatalog | undefined;
  const currentDraft = current.readEntity<AgentSessionDraft>(AGENT_SESSION_DRAFT_ENTITY, draftId);
  if (currentCatalog?.status !== "ready" || currentCatalog.generation !== catalog.generation
    || currentCatalog.owner_id !== catalog.owner_id || currentDraft?.generation !== draft.generation
    || currentDraft.dirty_fields.includes("presentations") || currentDraft.save_status === "saving") return;
  current.upsertEntity(AGENT_SESSION_DRAFT_ENTITY, draftId, {
    presentations: copyPresentationSelection(config?.presentations),
    presentation_retained_ids: [...(config?.presentations?.ids ?? [])],
    presentation_owner_id: catalog.owner_id,
    presentation_admission_id: PRESENTATION_ADMISSION_ID,
    presentation_error: null,
  });
}

function setPresentationMode(draftId: string, mode: PresentationSelectionMode): void {
  const state = graphStore.getState();
  const draft = admittedSessionPresentationDraft(state, draftId);
  if (!draft || draft.save_status === "saving" || draft.save_uncertain) return;
  const selection = copyPresentationSelection(draft.presentations) ?? { mode: "inherit", ids: [], denied_ids: [] };
  const retained = selection.mode === "selected" ? selection.ids : draft.presentation_retained_ids ?? [];
  state.upsertEntity(AGENT_SESSION_DRAFT_ENTITY, draftId, {
    presentation_retained_ids: [...retained],
    presentations: { ...selection, mode, ids: mode === "selected" ? [...retained] : [] },
    dirty_fields: [...new Set([...draft.dirty_fields, "presentations"])], save_status: "idle", error: null,
  });
}

function togglePresentation(draftId: string, id: string, excluded = false): void {
  const state = graphStore.getState();
  const draft = admittedSessionPresentationDraft(state, draftId);
  if (!draft || draft.save_status === "saving" || draft.save_uncertain) return;
  const selection = copyPresentationSelection(draft.presentations) ?? { mode: "inherit", ids: [], denied_ids: [] };
  if (!excluded && selection.mode !== "selected") return;
  const field = excluded ? "denied_ids" : "ids";
  const removing = selection[field].includes(id);
  const record = state.readEntity<Presentation>(PRESENTATION_ENTITY, id);
  if (!removing && (!record || record.owner_id !== draft.presentation_owner_id || !record.content.enabled
    || !state.lists[presentationListKey(record.owner_id)]?.ids.includes(id))) return;
  selection[field] = removing ? selection[field].filter((value) => value !== id) : [...selection[field], id];
  setAgentSessionDraftField(draftId, "presentations", selection);
}

function resetPresentations(draftId: string): void {
  const state = graphStore.getState();
  const draft = admittedSessionPresentationDraft(state, draftId);
  if (!draft || draft.save_status === "saving" || draft.save_uncertain) return;
  state.upsertEntity(AGENT_SESSION_DRAFT_ENTITY, draftId, { presentation_retained_ids: [] });
  setAgentSessionDraftField(draftId, "presentations", { mode: "inherit", ids: [], denied_ids: [] });
}

async function reconcileSavedSessionConfiguration(draftId: string): Promise<void> {
  const state = graphStore.getState();
  const draft = state.readEntity<AgentSessionDraft>(AGENT_SESSION_DRAFT_ENTITY, draftId);
  if (!draft?.save_uncertain || draft.save_status === "saving") return;
  markAgentSessionDraftSaving(draftId, draft.generation);
  try {
    await enqueueSessionMutation(draft.session_id, async () => {
      const saved = await fetchAgentSessionConfig(draft.session_id);
      const current = graphStore.getState();
      const currentDraft = current.readEntity<AgentSessionDraft>(AGENT_SESSION_DRAFT_ENTITY, draftId);
      if (currentDraft?.generation !== draft.generation) return;
      if (saved) replaceCanonicalAgentSession(draft.session_id, saved);
      else current.removeEntity(AGENT_SESSION_ENTITY, draft.session_id);
      current.upsertEntity(AGENT_SESSION_DRAFT_ENTITY, draftId, {
        save_uncertain: false, save_status: "idle",
        error: "Saved configuration checked. Your draft is retained; review the shown choices before saving again.",
      });
    });
  } catch {
    markAgentSessionDraftError(draftId, "Saved configuration is still unavailable. Your draft is retained; check again before saving.", draft.generation);
  }
}
