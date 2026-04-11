import type { DiscoveryResponse } from "@/types";

export async function fetchToolsDiscovery(): Promise<DiscoveryResponse & { data?: DiscoveryResponse }> {
  const res = await fetch("/api/tools");
  if (!res.ok) throw new Error(`${res.status}`);
  return res.json() as Promise<DiscoveryResponse & { data?: DiscoveryResponse }>;
}
