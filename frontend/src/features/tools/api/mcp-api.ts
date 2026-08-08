export async function fetchMcpHealth<T>(): Promise<T> {
  const res = await fetch("/api/uar/mcp/health");
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  return res.json() as Promise<T>;
}
