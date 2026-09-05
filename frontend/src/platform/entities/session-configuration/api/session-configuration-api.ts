import type { ProvidersResponse } from "@/types";
import { presentationSelectionSchema } from "../../presentation-assignments/contracts";

import type {
  AgentSessionConfig,
  PromptCachingSource,
  SessionPromptCaching,
} from "../contracts";

function sessionAuthorizationHeaders(): HeadersInit {
  const apiKey =
    (import.meta as unknown as { env: Record<string, string> }).env
      .VITE_UAR_API_KEY ?? "";
  return apiKey.startsWith("ey") ? { Authorization: `Bearer ${apiKey}` } : apiKey ? { "x-api-key": apiKey } : {};
}

function nullableString(value: unknown, field: string): string | null {
  if (value === null || value === undefined) return null;
  if (typeof value === "string") return value;
  throw new Error(
    `Session configuration field '${field}' must be a string or null`,
  );
}

function nullableStringList(value: unknown, field: string): string[] | null {
  if (value === null || value === undefined) return null;
  if (Array.isArray(value) && value.every((item) => typeof item === "string")) {
    return value;
  }
  throw new Error(
    `Session configuration field '${field}' must be a string array or null`,
  );
}

function nullableBoolean(value: unknown, field: string): boolean | null {
  if (value === null || value === undefined) return null;
  if (typeof value === "boolean") return value;
  throw new Error(
    `Session configuration field '${field}' must be a boolean or null`,
  );
}

export function decodeAgentSessionConfig(value: unknown): AgentSessionConfig {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error("Session configuration response must be an object");
  }
  const record = value as Record<string, unknown>;
  if (typeof record.agent_id !== "string") {
    throw new Error("Session configuration field 'agent_id' must be a string");
  }
  const toolApproval = nullableString(record.tool_approval, "tool_approval");
  if (toolApproval && !["auto", "ask", "deny"].includes(toolApproval)) {
    throw new Error("Session configuration field 'tool_approval' is invalid");
  }
  return {
    agent_id: record.agent_id,
    model: nullableString(record.model, "model"),
    tools: nullableStringList(record.tools, "tools"),
    skills: nullableStringList(record.skills, "skills"),
    knowledge_bases: nullableStringList(
      record.knowledge_bases,
      "knowledge_bases",
    ),
    mcp_servers: nullableStringList(record.mcp_servers, "mcp_servers"),
    presentations: record.presentations == null ? null : presentationSelectionSchema.parse(record.presentations),
    tool_approval: toolApproval as AgentSessionConfig["tool_approval"],
    prompt_caching_enabled: nullableBoolean(
      record.prompt_caching_enabled,
      "prompt_caching_enabled",
    ),
  };
}

const PROMPT_CACHING_SOURCES: PromptCachingSource[] = [
  "request",
  "session",
  "user",
  "global",
];

export function decodeSessionPromptCaching(
  sessionId: string,
  value: unknown,
): SessionPromptCaching {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error("Effective prompt-caching response must be an object");
  }
  const record = value as Record<string, unknown>;
  if (typeof record.enabled !== "boolean") {
    throw new Error(
      "Effective prompt-caching field 'enabled' must be a boolean",
    );
  }
  if (
    typeof record.source !== "string" ||
    !PROMPT_CACHING_SOURCES.includes(record.source as PromptCachingSource)
  ) {
    throw new Error("Effective prompt-caching field 'source' is invalid");
  }
  if (typeof record.global_default !== "boolean") {
    throw new Error(
      "Effective prompt-caching field 'global_default' must be a boolean",
    );
  }
  return {
    id: sessionId,
    session_id: sessionId,
    enabled: record.enabled,
    source: record.source as PromptCachingSource,
    session_override: nullableBoolean(
      record.session_override,
      "session_override",
    ),
    user_override: nullableBoolean(record.user_override, "user_override"),
    global_default: record.global_default,
  };
}

export function fetchConfiguredProvidersForEntities(
  signal?: AbortSignal,
): Promise<ProvidersResponse> {
  return fetch("/api/uar/providers", { signal }).then((response) => {
    if (!response.ok) {
      throw new Error(`Configured providers fetch failed: ${response.status}`);
    }
    return response.json() as Promise<ProvidersResponse>;
  });
}

export async function fetchAgentSessionConfig(
  sessionId: string,
  signal?: AbortSignal,
): Promise<AgentSessionConfig | null> {
  const response = await fetch(
    `/api/uar/sessions/${encodeURIComponent(sessionId)}/agent-config`,
    { headers: sessionAuthorizationHeaders(), signal },
  );
  if (response.status === 204 || response.status === 404) return null;
  if (!response.ok) {
    throw new Error(`Session configuration load failed: ${response.status}`);
  }
  return decodeAgentSessionConfig(await response.json());
}

export async function fetchSessionPromptCaching(
  sessionId: string,
  signal?: AbortSignal,
): Promise<SessionPromptCaching> {
  const response = await fetch(
    `/api/uar/sessions/${encodeURIComponent(sessionId)}/prompt-caching`,
    { headers: sessionAuthorizationHeaders(), signal },
  );
  if (!response.ok) {
    throw new Error(`Effective prompt-caching load failed: ${response.status}`);
  }
  return decodeSessionPromptCaching(sessionId, await response.json());
}

export class SessionConfigurationSaveError extends Error {
  constructor(message: string, readonly uncertain: boolean) { super(message); }
}

export async function saveAgentSessionConfig(
  sessionId: string,
  config: AgentSessionConfig,
  signal?: AbortSignal,
): Promise<AgentSessionConfig> {
  let response: Response;
  try {
    response = await fetch(
    `/api/uar/sessions/${encodeURIComponent(sessionId)}/agent-config`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...sessionAuthorizationHeaders(),
      },
      body: JSON.stringify(config),
      signal,
    },
    );
  } catch {
    throw new SessionConfigurationSaveError("The save result is unknown. Check saved configuration before saving again.", true);
  }
  if (!response.ok) {
    throw new SessionConfigurationSaveError(`Session configuration save failed: ${response.status}${response.status >= 500 ? ". Check saved configuration before saving again." : ""}`, response.status >= 500);
  }
  try {
    return decodeAgentSessionConfig(await response.json());
  } catch {
    throw new SessionConfigurationSaveError("The save response could not be confirmed. Check saved configuration before saving again.", true);
  }
}
