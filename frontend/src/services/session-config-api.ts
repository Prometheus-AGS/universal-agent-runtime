/** Persist an agent selection or session-level agent configuration. */
export async function saveSessionAgentConfig(
  threadId: string,
  config: Record<string, unknown>,
): Promise<void> {
  const response = await fetch(
    `/api/uar/sessions/${encodeURIComponent(threadId)}/agent-config`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(config),
    },
  );
  if (!response.ok) {
    throw new Error(`Session configuration save failed: ${response.status}`);
  }
}
