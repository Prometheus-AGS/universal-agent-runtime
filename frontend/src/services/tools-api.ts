import type { DiscoveryResponse } from "@/types";

export interface ExecuteToolResult {
  result: unknown;
  duration_ms: number;
  success: boolean;
  error?: string;
}

export async function fetchToolsDiscovery(): Promise<DiscoveryResponse & { data?: DiscoveryResponse }> {
  const res = await fetch("/api/tools");
  if (!res.ok) throw new Error(`${res.status}`);
  return res.json() as Promise<DiscoveryResponse & { data?: DiscoveryResponse }>;
}

export async function executeTool(name: string, arguments_: Record<string, unknown>): Promise<ExecuteToolResult> {
  const response = await fetch(`/api/tools/${encodeURIComponent(name)}/execute`, {
    method: "POST",
    headers: { "Content-Type": "application/json", "X-Agent-Id": "admin-console" },
    body: JSON.stringify({ arguments: arguments_ }),
  });
  const body = await response.json().catch(() => null) as ExecuteToolResult | null;
  if (!response.ok) {
    throw new Error(body?.error ?? `Tool execution failed: ${response.status}`);
  }
  if (!body) throw new Error("Tool execution returned an empty response");
  return body;
}
