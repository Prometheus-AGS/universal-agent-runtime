import {
  useEntities,
  useGraphStore,
} from "@prometheus-ags/prometheus-entity-management";

import {
  AGENT_SESSION_DRAFT_ENTITY,
  AGENT_SESSION_ENTITY,
  CONFIGURED_MODEL_ENTITY,
} from "./contracts";
import type {
  AgentSession,
  AgentSessionConfig,
  AgentSessionDraft,
  AgentSessionField,
  ConfiguredModel,
} from "./contracts";
import { agentSessionDraftActions } from "./domain";

export function useConfiguredModels() {
  return useEntities<ConfiguredModel>(CONFIGURED_MODEL_ENTITY);
}

export function useAgentSession(sessionId: string): AgentSession | null {
  return useGraphStore((state) =>
    (state.entities[AGENT_SESSION_ENTITY]?.[sessionId] as AgentSession | undefined) ?? null,
  );
}

export function useAgentSessionDraftField<K extends AgentSessionField>(
  draftId: string,
  field: K,
): AgentSessionConfig[K] | undefined {
  return useGraphStore((state) => {
    const draft = state.entities[AGENT_SESSION_DRAFT_ENTITY]?.[draftId] as
      | AgentSessionDraft
      | undefined;
    return draft?.[field];
  });
}

export function useAgentSessionDraftStatus(
  draftId: string,
): AgentSessionDraft["save_status"] | null {
  return useGraphStore((state) => {
    const draft = state.entities[AGENT_SESSION_DRAFT_ENTITY]?.[draftId] as
      | AgentSessionDraft
      | undefined;
    return draft?.save_status ?? null;
  });
}

export function useAgentSessionDraftError(draftId: string): string | null {
  return useGraphStore((state) => {
    const draft = state.entities[AGENT_SESSION_DRAFT_ENTITY]?.[draftId] as
      | AgentSessionDraft
      | undefined;
    return draft?.error ?? null;
  });
}

export function useAgentSessionDraftActions() {
  return agentSessionDraftActions;
}
