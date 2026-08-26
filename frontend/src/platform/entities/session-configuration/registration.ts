import {
  getRegisteredEntityTypes,
  registerEntityTransport,
  registerSchema,
} from "@prometheus-ags/prometheus-entity-management";
import type {
  EntityTransport,
  ListResult,
} from "@prometheus-ags/prometheus-entity-management";

import {
  AGENT_SESSION_DRAFT_ENTITY,
  AGENT_SESSION_ENTITY,
  CONFIGURED_MODEL_ENTITY,
  CONFIGURED_PROVIDER_ENTITY,
  SESSION_PROMPT_CACHING_ENTITY,
} from "./contracts";
import type {
  AgentSession,
  ConfiguredModel,
  ConfiguredProvider,
} from "./contracts";
import {
  fetchAgentSessionConfig,
  fetchConfiguredProvidersForEntities,
} from "./api/session-configuration-api";

export const SESSION_CONFIGURATION_REMOTE_ENTITY_TYPES = [
  CONFIGURED_PROVIDER_ENTITY,
  CONFIGURED_MODEL_ENTITY,
  AGENT_SESSION_ENTITY,
] as const;

let registered = false;

function emptyList<T>(): ListResult<T> {
  return { rows: [], total: 0, nextCursor: null };
}

const configuredProviderTransport: EntityTransport<ConfiguredProvider> = {
  identify: (provider) => provider.id,
  authoritative: false,
  list: async ({ signal }) => {
    const response = await fetchConfiguredProvidersForEntities(signal);
    const rows = response.providers
      .filter((provider) => provider.enabled !== false)
      .map((provider): ConfiguredProvider => ({
        id: provider.id,
        display_name: provider.display_name ?? provider.id,
        base_url: provider.base_url,
        default_model: provider.default_model ?? null,
        protocol: provider.protocol ?? null,
        enabled: true,
      }));
    return { rows, total: rows.length, nextCursor: null };
  },
};

const configuredModelTransport: EntityTransport<ConfiguredModel> = {
  identify: (model) => model.id,
  authoritative: false,
  list: async ({ signal }) => {
    const response = await fetchConfiguredProvidersForEntities(signal);
    const rows = response.providers
      .filter((provider) => provider.enabled !== false)
      .flatMap((provider) => {
        const configuredModels = provider.models ?? [];
        const models = configuredModels
          .filter((model) => model.enabled !== false)
          .map((model): ConfiguredModel => ({
            id: `${provider.id}/${model.id}`,
            provider_id: provider.id,
            provider_name: provider.display_name ?? provider.id,
            model_id: model.id,
            display_name: model.display_name ?? model.id,
            enabled: true,
            context_window: model.context_window ?? null,
            supports_tools: model.supports_tools ?? null,
            supports_vision: model.supports_vision ?? null,
          }));
        const defaultModel = provider.default_model;
        if (
          defaultModel &&
          !configuredModels.some((model) => model.id === defaultModel)
        ) {
          models.push({
            id: `${provider.id}/${defaultModel}`,
            provider_id: provider.id,
            provider_name: provider.display_name ?? provider.id,
            model_id: defaultModel,
            display_name: defaultModel,
            enabled: true,
            context_window: null,
            supports_tools: null,
            supports_vision: null,
          });
        }
        return models;
      });
    return { rows, total: rows.length, nextCursor: null };
  },
};

const agentSessionTransport: EntityTransport<AgentSession> = {
  identify: (session) => session.id,
  authoritative: false,
  list: async () => emptyList<AgentSession>(),
  get: async (sessionId, signal) => {
    const config = await fetchAgentSessionConfig(sessionId, signal);
    return config
      ? { ...config, id: sessionId, session_id: sessionId, revision: 1 }
      : null;
  },
};

export function registerSessionConfigurationEntities(): void {
  if (registered) return;

  registerSchema({ type: CONFIGURED_PROVIDER_ENTITY });
  registerSchema({ type: CONFIGURED_MODEL_ENTITY });
  registerSchema({ type: AGENT_SESSION_DRAFT_ENTITY });
  registerSchema({ type: SESSION_PROMPT_CACHING_ENTITY });
  registerEntityTransport(
    CONFIGURED_PROVIDER_ENTITY,
    configuredProviderTransport,
  );
  registerEntityTransport(CONFIGURED_MODEL_ENTITY, configuredModelTransport);
  registerEntityTransport(AGENT_SESSION_ENTITY, agentSessionTransport);
  registered = true;
}

export function getRegisteredSessionConfigurationEntityTypes(): string[] {
  const registeredTypes = getRegisteredEntityTypes();
  return SESSION_CONFIGURATION_REMOTE_ENTITY_TYPES.filter((type) =>
    registeredTypes.includes(type),
  );
}
