import type { AgentsResponse, UarAgent } from "@/types";

export async function fetchAgentsList(): Promise<UarAgent[]> {
  const res = await fetch("/api/agents");
  if (!res.ok) throw new Error(`${res.status}`);
  const data = await res.json() as AgentsResponse & { data?: AgentsResponse };
  const r = data.data?.runtime_agents ?? data.runtime_agents ?? [];
  const f = data.data?.federated_agents ?? data.federated_agents ?? [];
  return [...r.map((a) => ({ ...a, _type: "runtime" as const })), ...f.map((a) => ({ ...a, _type: "federated" as const }))];
}

export async function patchAgent(agentId: string, body: Record<string, unknown>): Promise<void> {
  const r = await fetch(`/api/agents/${agentId}`, {
    method: "PATCH",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  if (!r.ok) throw new Error(`${r.status}`);
}

async function requireOk(response: Response): Promise<void> {
  if (response.ok) return;
  const text = await response.text();
  throw new Error(text || `Agent request failed: ${response.status}`);
}

export async function createAgent(agent: unknown): Promise<void> {
  await requireOk(await fetch("/api/agents", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(agent),
  }));
}

export async function updateAgentFull(id: string, agent: unknown): Promise<void> {
  await requireOk(await fetch(`/api/agents/${encodeURIComponent(id)}`, {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(agent),
  }));
}

export async function deleteAgent(id: string): Promise<void> {
  await requireOk(await fetch(`/api/agents/${encodeURIComponent(id)}`, { method: "DELETE" }));
}

export async function generateAgentDefinition(description: string): Promise<unknown> {
  const response = await fetch("/api/a2a/compiler", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: crypto.randomUUID(),
      method: "message/send",
      params: {
        message: { role: "user", parts: [{ type: "text", text: description }] },
        metadata: { skill: "uar.compile" },
      },
    }),
  });
  await requireOk(response);
  return response.json();
}
