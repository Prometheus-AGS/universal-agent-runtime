import { graphStore } from "@prometheus-ags/prometheus-entity-management";

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
} from "./api/session-configuration-api";

const ALL_SESSION_FIELDS: AgentSessionField[] = [
  "agent_id",
  "model",
  "tools",
  "skills",
  "knowledge_bases",
  "mcp_servers",
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
    mergeConfigFields(canonical ?? saved, saved, changedFields),
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
  if (!draft) return false;

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
      const saved = await saveAgentSessionConfig(
        draft.session_id,
        payload,
        controller.signal,
      );
      await loadSessionPromptCaching(draft.session_id, controller.signal);
      return commitAgentSessionDraft(
        draftId,
        generation,
        draft.session_id,
        saved,
        changedFields,
      );
    });
  } catch (error) {
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
} as const;
