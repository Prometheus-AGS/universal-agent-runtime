/** A thread tracked locally in the persistent thread registry. */
export interface LocalThread {
  id: string;
  title: string;
  isEphemeral: boolean;
  createdAt: string;
  updatedAt: string;
}

/** UAR API — ProviderConfig */
export interface UarProvider {
  id: string;
  display_name?: string;
  base_url: string;
  api_key?: string;
  protocol?: string;
  enabled?: boolean;
  models?: UarModel[];
}

/** UAR API — ModelConfig */
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

/** UAR API — ModelsResponse */
export interface ModelsResponse {
  models: UarModel[];
  active_model?: string;
}

/** UAR API — SkillConfig */
export interface UarSkill {
  skill_id: string;
  title: string;
  description?: string;
  provider_id?: string;
  enabled?: boolean;
}

/** UAR API — AgentMetadata */
export interface AgentMetadata {
  title?: string;
  description?: string;
}

/** UAR API — RuntimeAgent */
export interface UarAgent {
  id: string;
  kind?: string;
  metadata?: AgentMetadata;
  skills?: UarSkill[];
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
