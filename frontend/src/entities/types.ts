// frontend/src/entities/types.ts
// Canonical entity types for the normalized graph.
// These mirror backend API response shapes.

export interface ProviderEntity extends Record<string, unknown> {
  id: string;
  display_name: string;
  base_url?: string;
  configured: boolean;
  auth_env_var?: string;
  endpoints: string[];
  model_count: number;
  /** Mirrors `CatalogProviderSummary.status` so the Provider page can render
   *  credential-blocked badges without a separate fetch. */
  status?: "configured" | "credential-blocked" | "available";
  status_detail?: string;
}

/**
 * Singleton entity holding global provider metadata (the default provider
 * id and that provider's default model). Keyed by the literal string
 * `"current"`. Backed by the `/api/uar/providers` response's `default_id`
 * field and the matching provider's `default_model` field on each fetch.
 */
export interface ProviderMetaEntity extends Record<string, unknown> {
  id: "current";
  default_id: string | null;
  default_model: string | null;
}

export interface ModelEntity extends Record<string, unknown> {
  id: string;
  name: string;
  provider_id: string;
  context: number;
  tool_call: boolean;
  reasoning: boolean;
  vision: boolean;
  model_id?: string;
  provider_name?: string;
  provider_configured?: boolean;
}

// AgentEntity is stored in the graph as the raw `UarAgent` shape from the
// `/api/agents` response — nested `metadata`, `policy`, `memory`, etc. The
// flat-field interface that lived here historically was a typed lie that
// existing consumers worked around by reading nested fields anyway.
//
// Aliasing to `UarAgent` makes the typed contract match reality. The legacy
// flat interface was removed after all consumers migrated to the real shape.
import type { UarAgent } from "@/types";
export type AgentEntity = UarAgent;

export interface ContextStrategy extends Record<string, unknown> {
  max_history_messages: number;
  inject_memory: boolean;
  inject_knowledge: boolean;
  memory_scope: "session" | "agent" | "global";
  auto_capture: boolean;
}

export interface AgentSessionEntity extends Record<string, unknown> {
  id: string;
  agent_id: string;
  session_id: string;
  model?: string;
  skills?: string[];
  tools?: string[];
  knowledge_bases?: string[];
  mcp_servers?: string[];
  context_strategy?: Partial<ContextStrategy>;
  tool_approval?: "auto" | "ask" | "deny";
}

export interface SkillEntity extends Record<string, unknown> {
  id: string;
  title: string;
  version: string;
  description: string;
  triggers: { keywords: string[]; semantic?: boolean };
  prompt_overlay?: string;
  preferred_tools: string[];
  enabled: boolean;
  provider_id?: string;
  source?: string;
  source_path?: string;
}

export interface ToolEntity extends Record<string, unknown> {
  id: string;
  name: string;
  description: string;
  namespace: string;
  input_schema: Record<string, unknown>;
  output_schema?: Record<string, unknown>;
  transport: "internal" | "http" | "mcp";
  built_in: boolean;
}

export interface SettingEntity extends Record<string, unknown> {
  id: string;
  key: string;
  name: string;
  data: unknown;
  settings_type_id: string;
  updated_at?: string;
}

export interface SettingsTypeEntity extends Record<string, unknown> {
  id: string;
  key: string;
  name: string;
  schema: Record<string, unknown>;
  updated_at?: string;
}

export interface KnowledgeBaseEntity extends Record<string, unknown> {
  id: string;
  name: string;
  description?: string;
  document_count: number;
  config?: Record<string, unknown>;
  created_at: string;
  updated_at: string;
}

export interface DocumentEntity extends Record<string, unknown> {
  id: string;
  kb_id: string;
  filename: string;
  status: "pending" | "processing" | "indexed" | "failed";
  chunk_count: number;
  mime_type?: string;
  error_message?: string;
  created_at: string;
  updated_at: string;
}

export interface ThreadEntity extends Record<string, unknown> {
  id: string;
  title: string;
  agent_id?: string;
  agent_name?: string;
  last_message_preview?: string;
  created_at: string;
  updated_at: string;
}

export type RuntimeStatus =
  | "queued"
  | "running"
  | "waiting"
  | "completed"
  | "failed"
  | "cancelled";

export type RuntimeRunPhase =
  | "context"
  | "skill"
  | "memory"
  | "retrieval"
  | "reasoning"
  | "tool"
  | "generate";

export interface RuntimeRunEntity extends Record<string, unknown> {
  id: string;
  thread_id?: string;
  agent_id?: string;
  model?: string;
  provider_id?: string;
  status: RuntimeStatus;
  title?: string;
  started_at?: string;
  completed_at?: string;
  updated_at: string;
  /** Present on `run_finished` (CH-06 cost-budgets-backend). */
  cost_usd_estimate?: number | null;
  input_tokens?: number | null;
  output_tokens?: number | null;
  total_tokens?: number | null;
  /** Transport-observed phase durations in milliseconds, completed at run terminal. */
  phase_timings?: Record<RuntimeRunPhase, number>;
}

export interface RuntimeRunStepEntity extends Record<string, unknown> {
  id: string;
  run_id: string;
  kind: "message" | "reasoning" | "tool" | "approval" | "artifact" | "state" | "error";
  status: RuntimeStatus;
  title: string;
  summary?: string;
  started_at?: string;
  completed_at?: string;
  updated_at: string;
}

export interface RuntimeToolCallEntity extends Record<string, unknown> {
  id: string;
  run_id: string;
  step_id?: string;
  tool_name: string;
  namespace?: string;
  status: RuntimeStatus;
  input?: Record<string, unknown>;
  output?: Record<string, unknown>;
  error?: string;
  updated_at: string;
}

export interface RuntimeApprovalEntity extends Record<string, unknown> {
  id: string;
  run_id: string;
  tool_call_id?: string;
  status: "pending" | "approved" | "denied" | "expired";
  reason?: string;
  updated_at: string;
}

export interface RuntimeArtifactEntity extends Record<string, unknown> {
  id: string;
  run_id: string;
  kind: "file" | "image" | "html" | "json" | "text" | "a2ui";
  title: string;
  uri?: string;
  mime_type?: string;
  updated_at: string;
}

export interface RuntimeMemoryEventEntity extends Record<string, unknown> {
  id: string;
  run_id?: string;
  memory_id?: string;
  action: "recall" | "write" | "update" | "delete" | "workflow_mirror";
  summary: string;
  source_tool?: string;
  updated_at: string;
}

export interface RuntimeAgUiEventEntity extends Record<string, unknown> {
  id: string;
  run_id: string;
  event_type: string;
  sequence: number;
  phase?: RuntimeRunPhase | null;
  received_at?: string;
  payload: Record<string, unknown>;
  updated_at: string;
}

export interface RuntimeA2uiSurfaceEntity extends Record<string, unknown> {
  id: string;
  run_id?: string;
  schema_id?: string;
  title: string;
  status: "draft" | "rendered" | "error";
  payload?: Record<string, unknown>;
  updated_at: string;
}

export interface RuntimeModelRouteDecisionEntity extends Record<string, unknown> {
  id: string;
  run_id?: string;
  selected_model: string;
  selected_provider?: string;
  needs_tools?: boolean;
  needs_vision?: boolean;
  min_context?: number;
  reason?: string;
  updated_at: string;
}

export interface RuntimeProviderHealthEntity extends Record<string, unknown> {
  id: string;
  provider_id: string;
  status: "healthy" | "degraded" | "offline" | "unknown";
  latency_ms?: number;
  error?: string;
  updated_at: string;
}

export interface RuntimeBudgetAlertEntity extends Record<string, unknown> {
  id: string;
  run_id: string;
  scope: "run" | "task" | "session" | "agent" | "global";
  scope_id: string;
  spent_usd: number;
  limit_usd: number;
  exceeded: boolean;
  updated_at: string;
}
