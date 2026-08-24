import type { ProvidersResponse } from "@/types";

import type { AgentSessionConfig } from "../contracts";

function nullableString(value: unknown, field: string): string | null {
  if (value === null || value === undefined) return null;
  if (typeof value === "string") return value;
  throw new Error(`Session configuration field '${field}' must be a string or null`);
}

function nullableStringList(value: unknown, field: string): string[] | null {
  if (value === null || value === undefined) return null;
  if (Array.isArray(value) && value.every((item) => typeof item === "string")) {
    return value;
  }
  throw new Error(`Session configuration field '${field}' must be a string array or null`);
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
    knowledge_bases: nullableStringList(record.knowledge_bases, "knowledge_bases"),
    mcp_servers: nullableStringList(record.mcp_servers, "mcp_servers"),
    tool_approval: toolApproval as AgentSessionConfig["tool_approval"],
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
    { signal },
  );
  if (response.status === 404) return null;
  if (!response.ok) {
    throw new Error(`Session configuration load failed: ${response.status}`);
  }
  return decodeAgentSessionConfig(await response.json());
}

export async function saveAgentSessionConfig(
  sessionId: string,
  config: AgentSessionConfig,
  signal?: AbortSignal,
): Promise<AgentSessionConfig> {
  const response = await fetch(
    `/api/uar/sessions/${encodeURIComponent(sessionId)}/agent-config`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(config),
      signal,
    },
  );
  if (!response.ok) {
    throw new Error(`Session configuration save failed: ${response.status}`);
  }
  return decodeAgentSessionConfig(await response.json());
}
