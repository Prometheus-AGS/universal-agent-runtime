export async function fetchA2uiSchemas<T>(): Promise<T> {
  const res = await fetch("/api/uar/a2ui/schemas");
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  return res.json() as Promise<T>;
}
