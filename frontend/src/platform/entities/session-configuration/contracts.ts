export const CONFIGURED_PROVIDER_ENTITY = "ConfiguredProvider" as const;
export const CONFIGURED_MODEL_ENTITY = "ConfiguredModel" as const;
export const AGENT_SESSION_ENTITY = "AgentSession" as const;
export const AGENT_SESSION_DRAFT_ENTITY = "AgentSessionDraft" as const;
export const SESSION_PROMPT_CACHING_ENTITY = "SessionPromptCaching" as const;

export type ToolApproval = "auto" | "ask" | "deny";

/** Wire contract shared by GET/POST `/api/uar/sessions/{id}/agent-config`. */
export interface AgentSessionConfig extends Record<string, unknown> {
  agent_id: string;
  model: string | null;
  tools: string[] | null;
  skills: string[] | null;
  knowledge_bases: string[] | null;
  mcp_servers: string[] | null;
  tool_approval: ToolApproval | null;
  /** Missing legacy values decode to null (Inherit). */
  prompt_caching_enabled: boolean | null;
}

export type PromptCachingSource = "request" | "session" | "user" | "global";

/** Authoritative effective prompt-caching state for one session. */
export interface SessionPromptCaching extends Record<string, unknown> {
  id: string;
  session_id: string;
  enabled: boolean;
  source: PromptCachingSource;
  session_override: boolean | null;
  user_override: boolean | null;
  global_default: boolean;
}

/** Canonical, server-confirmed session configuration keyed by thread id. */
export interface AgentSession extends AgentSessionConfig {
  id: string;
  session_id: string;
  revision: number;
}

export type AgentSessionField = keyof AgentSessionConfig;
export type AgentSessionDraftStatus = "idle" | "saving" | "error";

/** Editor-local copy. It is never registered with a remote transport. */
export interface AgentSessionDraft extends AgentSessionConfig {
  id: string;
  session_id: string;
  editor_id: string;
  generation: number;
  baseline_revision: number;
  dirty_fields: AgentSessionField[];
  save_status: AgentSessionDraftStatus;
  error: string | null;
}

/** One provider configured for this UAR instance. */
export interface ConfiguredProvider extends Record<string, unknown> {
  id: string;
  display_name: string;
  base_url: string;
  default_model: string | null;
  protocol: string | null;
  enabled: boolean;
}

/** One configured model, keyed by its complete `provider/model` route. */
export interface ConfiguredModel extends Record<string, unknown> {
  id: string;
  provider_id: string;
  provider_name: string;
  model_id: string;
  display_name: string;
  enabled: boolean;
  context_window: number | null;
  supports_tools: boolean | null;
  supports_vision: boolean | null;
}
