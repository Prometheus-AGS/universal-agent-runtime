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
