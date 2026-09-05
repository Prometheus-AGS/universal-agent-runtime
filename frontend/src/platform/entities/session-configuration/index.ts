export {
  AGENT_SESSION_DRAFT_ENTITY,
  AGENT_SESSION_ENTITY,
  CONFIGURED_MODEL_ENTITY,
  CONFIGURED_PROVIDER_ENTITY,
  SESSION_PROMPT_CACHING_ENTITY,
} from "./contracts";

export type {
  AgentSession,
  AgentSessionConfig,
  AgentSessionDraft,
  AgentSessionDraftStatus,
  AgentSessionField,
  ConfiguredModel,
  ConfiguredProvider,
  PromptCachingSource,
  SessionPromptCaching,
  ToolApproval,
} from "./contracts";

export {
  getRegisteredSessionConfigurationEntityTypes,
  registerSessionConfigurationEntities,
  SESSION_CONFIGURATION_REMOTE_ENTITY_TYPES,
} from "./registration";

export {
  agentSessionDraftActions,
  agentSessionDraftId,
  cancelAgentSessionDraft,
  commitAgentSessionDraft,
  loadAndOpenAgentSessionDraft,
  loadAgentSession,
  loadSessionPromptCaching,
  markAgentSessionDraftError,
  markAgentSessionDraftSaving,
  openAgentSessionDraft,
  readAgentSessionDraftConfig,
  saveAgentSessionDraft,
  selectAgentForSession,
  setAgentSessionDraftField,
} from "./domain";

export {
  useAgentSession,
  useAgentSessionDraftActions,
  useAgentSessionDraftError,
  useAgentSessionDraftField,
  useAgentSessionDraftStatus,
  useConfiguredModels,
  useSessionPromptCaching,
  useSessionPresentationReady,
  useSessionPresentationError,
  useAgentSessionDraftUncertain,
  useSessionPresentationMode,
  useSessionPresentationIds,
  useSessionPresentationMarked,
  useSessionPresentationRetainedCount,
  useSessionPresentationMatchCount,
} from "./use-session-configuration";

export type { PresentationSelection, PresentationSelectionMode } from "../presentation-assignments/contracts";
