const TITLE_GEN_URL = "/api/generate-title";

interface GenerateTitleResponse {
  title?: string;
  content?: string;
  message?: string;
  choices?: { message?: { content?: string } }[];
}

function cleanTitle(raw: string): string {
  return raw.trim().replace(/^["']|["']$/g, "").trim().slice(0, 80);
}

export async function generateThreadTitle(userMsg: string, assistantMsg: string): Promise<string> {
  const fallback = "New conversation";
  if (!userMsg.trim() || !assistantMsg.trim()) return fallback;
  try {
    const res = await fetch(TITLE_GEN_URL, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        message: userMsg,
        assistant_message: assistantMsg,
      }),
    });
    if (!res.ok) return fallback;
    const json = (await res.json()) as GenerateTitleResponse;
    const title = json.title ?? json.content ?? json.message ?? json.choices?.[0]?.message?.content ?? null;
    if (title?.trim()) return cleanTitle(title);
    return fallback;
  } catch {
    return fallback;
  }
}
