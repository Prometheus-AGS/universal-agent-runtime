import {
  useEntities,
  useGraphStore,
} from "@prometheus-ags/prometheus-entity-management";

import {
  AGENT_SESSION_DRAFT_ENTITY,
  AGENT_SESSION_ENTITY,
  CONFIGURED_MODEL_ENTITY,
  SESSION_PROMPT_CACHING_ENTITY,
} from "./contracts";
import type {
  AgentSession,
  AgentSessionConfig,
  AgentSessionDraft,
  AgentSessionField,
  ConfiguredModel,
  SessionPromptCaching,
} from "./contracts";
import { admittedSessionPresentationDraft, agentSessionDraftActions } from "./domain";
import { PRESENTATION_ENTITY, type Presentation } from "../presentations/contracts";
import { presentationListKey } from "../presentations/domain";

const EMPTY_PRESENTATION_IDS: string[] = [];

export function useSessionPresentationReady(draftId: string): boolean {
  return useGraphStore((state) => admittedSessionPresentationDraft(state, draftId) !== null);
}

export function useSessionPresentationError(draftId: string): string | null {
  return useGraphStore((state) => (state.entities[AGENT_SESSION_DRAFT_ENTITY]?.[draftId] as AgentSessionDraft | undefined)?.presentation_error ?? null);
}

export function useAgentSessionDraftUncertain(draftId: string): boolean {
  return useGraphStore((state) => (state.entities[AGENT_SESSION_DRAFT_ENTITY]?.[draftId] as AgentSessionDraft | undefined)?.save_uncertain === true);
}

export function useSessionPresentationMode(draftId: string) {
  return useGraphStore((state) => {
    const draft = admittedSessionPresentationDraft(state, draftId);
    return draft ? draft.presentations?.mode ?? "inherit" : undefined;
  });
}

export function useSessionPresentationIds(draftId: string, excluded = false): string[] {
  return useGraphStore((state) => admittedSessionPresentationDraft(state, draftId)?.presentations?.[excluded ? "denied_ids" : "ids"] ?? EMPTY_PRESENTATION_IDS);
}

export function useSessionPresentationMarked(draftId: string, id: string, excluded = false): boolean {
  return useGraphStore((state) => admittedSessionPresentationDraft(state, draftId)?.presentations?.[excluded ? "denied_ids" : "ids"].includes(id) ?? false);
}

export function useSessionPresentationRetainedCount(draftId: string): number {
  return useGraphStore((state) => {
    const draft = admittedSessionPresentationDraft(state, draftId);
    return draft?.presentations?.mode === "selected" ? 0 : draft?.presentation_retained_ids?.length ?? 0;
  });
}

export function useSessionPresentationMatchCount(draftId: string, search: string): number {
  return useGraphStore((state) => {
    const draft = admittedSessionPresentationDraft(state, draftId);
    if (!draft?.presentation_owner_id) return 0;
    const catalogIds = state.lists[presentationListKey(draft.presentation_owner_id)]?.ids ?? EMPTY_PRESENTATION_IDS;
    const ids = new Set([...catalogIds, ...(draft.presentations?.ids ?? []), ...(draft.presentations?.denied_ids ?? [])]);
    const query = search.trim().toLocaleLowerCase();
    let count = 0;
    for (const id of ids) {
      const record = catalogIds.includes(id) ? state.entities[PRESENTATION_ENTITY]?.[id] as Presentation | undefined : undefined;
      const title = record?.owner_id === draft.presentation_owner_id ? record?.content.title ?? id : id;
      if (`${title} ${id}`.toLocaleLowerCase().includes(query)) count += 1;
    }
    return count;
  });
}

export function useConfiguredModels() {
  return useEntities<ConfiguredModel>(CONFIGURED_MODEL_ENTITY);
}

export function useAgentSession(sessionId: string): AgentSession | null {
  return useGraphStore(
    (state) =>
      (state.entities[AGENT_SESSION_ENTITY]?.[sessionId] as
        AgentSession | undefined) ?? null,
  );
}

export function useSessionPromptCaching(
  sessionId: string,
): SessionPromptCaching | null {
  return useGraphStore(
    (state) =>
      (state.entities[SESSION_PROMPT_CACHING_ENTITY]?.[sessionId] as
        SessionPromptCaching | undefined) ?? null,
  );
}

export function useAgentSessionDraftField<K extends AgentSessionField>(
  draftId: string,
  field: K,
): AgentSessionConfig[K] | undefined {
  return useGraphStore((state) => {
    const draft = state.entities[AGENT_SESSION_DRAFT_ENTITY]?.[draftId] as
      AgentSessionDraft | undefined;
    return draft?.[field];
  });
}

export function useAgentSessionDraftStatus(
  draftId: string,
): AgentSessionDraft["save_status"] | null {
  return useGraphStore((state) => {
    const draft = state.entities[AGENT_SESSION_DRAFT_ENTITY]?.[draftId] as
      AgentSessionDraft | undefined;
    return draft?.save_status ?? null;
  });
}

export function useAgentSessionDraftError(draftId: string): string | null {
  return useGraphStore((state) => {
    const draft = state.entities[AGENT_SESSION_DRAFT_ENTITY]?.[draftId] as
      AgentSessionDraft | undefined;
    return draft?.error ?? null;
  });
}

export function useAgentSessionDraftActions() {
  return agentSessionDraftActions;
}
