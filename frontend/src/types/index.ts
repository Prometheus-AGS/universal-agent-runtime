/** A thread tracked locally in the persistent thread registry. */
export interface LocalThread {
  id: string;
  title: string;
  isEphemeral: boolean;
  createdAt: string;
  updatedAt: string;
}

/** UAR API — ProviderConfig (registry-managed provider override) */
export interface UarProvider {
  id: string;
  display_name?: string;
  base_url: string;
  api_key?: string;
  protocol?: string;
  /** Default model id used when a request omits one. Round-trips through PUT. */
  default_model?: string;
  enabled?: boolean;
  models?: UarModel[];
}

/** UAR API — ModelConfig (registry-managed model override) */
export interface UarModel {
  id: string;
  display_name?: string;
  context_window?: number;
  supports_tools?: boolean;
  supports_vision?: boolean;
  enabled?: boolean;
}

/** UAR API — ProvidersResponse */
export interface ProvidersResponse {
  providers: UarProvider[];
  default_id?: string;
}

/** UAR API — ModelsResponse (legacy shape) */
export interface ModelsResponse {
  models: UarModel[];
  active_model?: string;
}

// ---------------------------------------------------------------------------
// Catalog types (from /api/models and /api/catalog — backed by ModelCatalog)
// ---------------------------------------------------------------------------

/** Model capability flags from the compile-time catalog. */
export interface CatalogModelCapabilities {
  tool_call: boolean;
  reasoning: boolean;
  structured_output: boolean;
  streaming: boolean;
  attachment?: boolean;
}

/** Pricing per 1M tokens (USD). */
export interface CatalogModelCost {
  input: number;
  output: number;
  cache_read?: number;
  cache_write?: number;
}

/** Context / output token limits. */
export interface CatalogModelLimits {
  context: number;
  input: number;
  output: number;
}

/** A single model entry from /api/models. */
export interface CatalogModel {
  name: string;
  family?: string;
  limit: CatalogModelLimits;
  cost: CatalogModelCost;
  modalities: { input: string[]; output: string[] };
  tool_call: boolean;
  reasoning: boolean;
  structured_output: boolean;
  streaming: boolean;
  open_weights: boolean;
}

/** A provider entry from /api/models. */
export interface CatalogProvider {
  display_name: string;
  base_url?: string;
  configured: boolean;
  models: Record<string, CatalogModel>;
}

/** Full /api/models response: provider_id → CatalogProvider. */
export type CatalogModelsResponse = Record<string, CatalogProvider>;

/** A provider summary from /api/catalog. */
export interface CatalogProviderSummary {
  id: string;
  display_name: string;
  base_url?: string;
  model_count: number;
  configured: boolean;
  auth_env_var?: string;
  status?: "configured" | "credential-blocked" | "available";
  status_detail?: string;
  endpoints: string[];
}

/** Full /api/catalog response. */
export interface CatalogResponse {
  provider_count: number;
  model_count: number;
  providers: CatalogProviderSummary[];
}

/** UAR API — SkillExecutionConfig */
export interface UarSkillExecutionConfig {
  preferred_provider?: string | null;
  preferred_model?: string | null;
  max_tokens?: number | null;
}

/** UAR API — SkillConfig */
export interface UarSkill {
  skill_id: string;
  title: string;
  description?: string;
  version?: string;
  provider_id?: string;
  enabled?: boolean;
  triggers?: {
    keywords?: string[];
    semantic?: string | null;
  };
  preferred_tools?: string[];
  prompt_overlay?: string;
  execution_config?: UarSkillExecutionConfig;
}

/** UAR API — AgentMetadata */
export interface AgentMetadata {
  title?: string;
  description?: string;
}

/** UAR API — Provider selection within an agent policy */
export interface UarProviderSelection {
  provider: string;
  model: string;
}

/** UAR API — Agent policy */
export interface UarAgentPolicy {
  provider?: {
    default?: UarProviderSelection;
    fallbacks?: UarProviderSelection[];
  };
  tools?: {
    allow?: string[];
    deny?: string[];
    max_concurrent?: number;
  };
  skills?: {
    prefer?: string[];
    max_active?: number;
  };
}

/** UAR API — Agent memory config */
export interface UarAgentMemory {
  conversation?: { enabled?: boolean };
  kb?: {
    enabled?: boolean;
    knowledge_bases?: string[];
    citation_required?: boolean;
  };
}

/** UAR API — Tool bundle */
export interface UarToolBundle {
  id: string;
  tools?: string[];
  required?: boolean;
}

/** UAR API — RuntimeAgent */
export interface UarAgent {
  id: string;
  kind?: string;
  metadata?: AgentMetadata;
  skills?: UarSkill[];
  policy?: UarAgentPolicy;
  memory?: UarAgentMemory;
  tools?: { bundles?: UarToolBundle[] };
}

/** UAR API — AgentsResponse */
export interface AgentsResponse {
  runtime_agents?: UarAgent[];
  federated_agents?: UarAgent[];
}

/** UAR API — DiscoveredTool */
export interface UarTool {
  name: string;
  namespaced_name?: string;
  description?: string;
  source?: string;
  server?: string;
  input_schema?: Record<string, unknown>;
  parameters?: Record<string, unknown>;
}

/** UAR API — DiscoveryResponse */
export interface DiscoveryResponse {
  tools?: UarTool[];
  built_in_tools?: UarTool[];
}

/** UAR API — ApiKey */
export interface UarApiKey {
  id: string;
  name?: string;
  prefix?: string;
  created_at?: string;
  last_used?: string;
}

/** UAR API — KnowledgeBase */
export interface UarKnowledgeBase {
  id: string;
  name: string;
  description?: string;
  document_count?: number;
  config?: KbConfig;
  created_at?: string;
  updated_at?: string;
}

/** UAR API — KbConfig */
export interface KbConfig {
  embedding_provider?: string;
  embedding_model?: string;
  vector_dimensions?: number;
  file_processor?: string;
  chunk_strategy?: string;
}

/** UAR API — KnowledgeDocument */
export interface UarKnowledgeDocument {
  id: string;
  kb_id: string;
  filename: string;
  file_path?: string;
  mime_type?: string;
  chunk_count: number;
  status: DocumentStatus;
  error_message?: string;
  created_at: string;
  updated_at: string;
}

/** Document processing status */
export type DocumentStatus = "pending" | "processing" | "indexed" | "failed";

/** UAR API — SearchResult */
export interface KbSearchResult {
  content: string;
  score: number;
  metadata?: Record<string, unknown>;
  document_id?: string;
}

/** UAR API — CompilerSession */
export interface UarCompilerSession {
  id: string;
  status?: string;
  created_at?: string;
  skill_ids?: string[];
}

// ---------------------------------------------------------------------------
// Settings API types
// ---------------------------------------------------------------------------

/** A registered settings namespace (type) with its JSON Schema. */
export interface SettingsType {
  id: string;
  key: string;
  name: string;
  schema: Record<string, unknown>;
  created_at: string;
  updated_at?: string;
}

/** Source of a setting value. */
export type SettingSource = "Default" | "ConfigFile" | "EnvVar" | "Api";

/** Transient metadata attached to a setting at runtime. */
export interface SettingsMeta {
  source: SettingSource;
  is_drift: boolean;
  last_changed_at?: string;
}

/** A setting value plus its runtime metadata. */
export interface SettingWithMeta {
  id: string;
  settings_type_id: string;
  key: string;
  name: string;
  data: unknown;
  parent_id?: string;
  created_at: string;
  updated_at?: string;
  meta: SettingsMeta;
}

/** A drift item — setting differs from config-file default. */
export interface SettingsDriftItem {
  setting: SettingWithMeta;
  config_value: unknown;
}

// ---------------------------------------------------------------------------
// Per-user settings types
// ---------------------------------------------------------------------------

/** Granularity at which a user's prompt-caching preference applies. */
export type CachingScope = "session" | "user" | "agent";

/** Per-user prompt-caching preferences returned by GET /api/uar/user/settings. */
export interface UserSettings {
  user_id: string;
  prompt_caching_enabled: boolean | null;
  preferred_scope: CachingScope;
  updated_at: string;
}

// ---------------------------------------------------------------------------
// Chat Attachment types
// ---------------------------------------------------------------------------

/** Response from POST /api/upload for a single file. */
export interface UploadedFileResponse {
  id: string;
  filename: string;
  content_type: string;
  size: number;
  is_image: boolean;
  /** Retrieval URL: GET /api/attachments/{id} */
  url: string;
  /** Extracted text content for document files. */
  text_content?: string;
}

/** An attachment in-flight or ready to send. */
export interface PendingAttachment {
  /** Client-side temp ID before upload completes. */
  localId: string;
  file: File;
  status: "uploading" | "ready" | "error";
  /** Set once upload succeeds. */
  uploaded?: UploadedFileResponse;
  errorMessage?: string;
}

/** Serialized attachment reference sent in the chat completion request body. */
export interface AttachmentPayload {
  id: string;
  filename: string;
  content_type: string;
  url: string;
  text_content?: string;
}
