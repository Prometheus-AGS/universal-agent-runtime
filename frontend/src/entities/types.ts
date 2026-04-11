// frontend/src/entities/types.ts
// Canonical entity types for the normalized graph.
// These mirror backend API response shapes.

export interface ProviderEntity {
  id: string;
  display_name: string;
  base_url?: string;
  configured: boolean;
  auth_env_var?: string;
  endpoints: string[];
  model_count: number;
}

export interface ModelEntity {
  id: string;
  name: string;
  provider_id: string;
  context: number;
  tool_call: boolean;
  reasoning: boolean;
  vision: boolean;
}

export interface AgentEntity {
  id: string;
  name: string;
  description: string;
  system_prompt: string;
  model?: string;
  protocol?: "auto" | "openai-chat" | "openai-responses";
  skills: string[];
  tools: string[];
  knowledge_bases: string[];
  mcp_servers: string[];
  context_strategy: ContextStrategy;
  tool_approval: "auto" | "ask" | "deny";
  status: "active" | "draft" | "disabled";
  spec_id?: string;
  created_at: string;
  updated_at: string;
}

export interface ContextStrategy {
  max_history_messages: number;
  inject_memory: boolean;
  inject_knowledge: boolean;
  memory_scope: "session" | "agent" | "global";
  auto_capture: boolean;
}

export interface AgentSessionEntity {
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

export interface SkillEntity {
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

export interface ToolEntity {
  id: string;
  name: string;
  description: string;
  namespace: string;
  input_schema: Record<string, unknown>;
  output_schema?: Record<string, unknown>;
  transport: "internal" | "http" | "mcp";
  built_in: boolean;
}

export interface KnowledgeBaseEntity {
  id: string;
  name: string;
  description?: string;
  document_count: number;
  config?: Record<string, unknown>;
  created_at: string;
  updated_at: string;
}

export interface DocumentEntity {
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

export interface ThreadEntity {
  id: string;
  title: string;
  agent_id?: string;
  agent_name?: string;
  last_message_preview?: string;
  created_at: string;
  updated_at: string;
}
