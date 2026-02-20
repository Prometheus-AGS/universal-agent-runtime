const TITLE_GEN_URL = "/api/chat/completion";
const TITLE_PROMPT = (u: string, a: string) =>
  `Generate a concise 4-6 word title that captures the topic of this conversation.\nReply with ONLY the title text — no quotes, punctuation, or explanation.\n\nUser: ${u.slice(0, 500)}\nAssistant: ${a.slice(0, 500)}`;

interface NonStreamingResponse { content?: string; message?: string; choices?: { message?: { content?: string } }[] }

export async function generateThreadTitle(userMsg: string, assistantMsg: string): Promise<string> {
  const fallback = "New conversation";
  if (!userMsg.trim() || !assistantMsg.trim()) return fallback;
  const ephemeralSessionId = `__title_gen__${Date.now()}`;
  try {
    const res = await fetch(TITLE_GEN_URL, {
      method: "POST",
      headers: { "Content-Type": "application/json", "X-UAR-Session-ID": ephemeralSessionId },
      body: JSON.stringify({ message: TITLE_PROMPT(userMsg, assistantMsg), stream: false }),
    });
    if (!res.ok) return fallback;
    const contentType = res.headers.get("content-type") ?? "";
    if (contentType.includes("application/json")) {
      const json = (await res.json()) as NonStreamingResponse;
      const title = json.content ?? json.message ?? json.choices?.[0]?.message?.content ?? null;
      if (title?.trim()) return cleanTitle(title);
      return fallback;
    }
    if (!res.body) return fallback;
    const reader = res.body.getReader();
    const decoder = new TextDecoder();
    let buffer = "", collected = "";
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      buffer += decoder.decode(value, { stream: true });
      const blocks = buffer.split("\n\n");
      buffer = blocks.pop() ?? "";
      for (const raw of blocks) {
        if (!raw.trim()) continue;
        let data = "", event = "message";
        for (const line of raw.split("\n")) {
          if (line.startsWith("event:")) event = line.slice(6).trim();
          else if (line.startsWith("data:")) data = line.slice(5).trim();
        }
        if (data === "[DONE]") break;
        if (event === "agui.message.delta" && data) {
          try { const p = JSON.parse(data) as { delta?: { text?: string } }; if (p.delta?.text) collected += p.delta.text; } catch { /**/ }
        } else if (event === "message" && data && data !== "[DONE]") {
          try { const p = JSON.parse(data) as { choices?: { delta?: { content?: string } }[] }; const c = p.choices?.[0]?.delta?.content; if (c) collected += c; } catch { /**/ }
        }
      }
    }
    if (collected.trim()) return cleanTitle(collected);
    return fallback;
  } catch { return fallback; }
}

function cleanTitle(raw: string): string {
  return raw.trim().replace(/^["']|["']$/g, "").trim().slice(0, 80);
}
