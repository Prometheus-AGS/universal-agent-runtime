/** Returns true if at least one LLM provider is configured. */
export async function checkHasConfiguredProviders(): Promise<boolean> {
  try {
    const res = await fetch("/api/uar/providers");
    if (!res.ok) return false;
    const data = (await res.json()) as { providers?: unknown[] };
    return (data.providers?.length ?? 0) > 0;
  } catch {
    return false;
  }
}

/** Returns true if at least one knowledge base exists. */
export async function checkHasKnowledgeBases(): Promise<boolean> {
  try {
    const res = await fetch("/api/knowledge");
    if (!res.ok) return false;
    const data = await res.json();
    const list = Array.isArray(data)
      ? data
      : (data as { knowledge_bases?: unknown[] })?.knowledge_bases ??
        (data as { data?: { knowledge_bases?: unknown[] } })?.data?.knowledge_bases ??
        [];
    return list.length > 0;
  } catch {
    return false;
  }
}
