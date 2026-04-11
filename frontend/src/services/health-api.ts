export interface HealthzData {
  status?: string;
  version?: string;
  uptime?: number;
}

export async function fetchHealthz(): Promise<HealthzData | null> {
  const r = await fetch("/healthz");
  if (!r.ok) return null;
  return r.json() as Promise<HealthzData>;
}
