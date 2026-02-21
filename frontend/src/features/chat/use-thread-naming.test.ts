import { afterEach, describe, expect, mock, test } from "bun:test";
import { generateThreadTitle } from "./use-thread-naming";

function getHeader(init: RequestInit | undefined, key: string): string | null {
  if (!init?.headers) return null;
  if (init.headers instanceof Headers) return init.headers.get(key);
  if (Array.isArray(init.headers)) {
    const found = init.headers.find(([k]) => k.toLowerCase() === key.toLowerCase());
    return found?.[1] ?? null;
  }
  const record = init.headers as Record<string, string>;
  for (const [k, v] of Object.entries(record)) {
    if (k.toLowerCase() === key.toLowerCase()) return v;
  }
  return null;
}

describe("generateThreadTitle", () => {
  afterEach(() => {
    mock.restore();
  });

  test("sends OpenAI-style non-streaming payload without custom session header", async () => {
    const fetchMock = mock(async () => {
      return new Response(JSON.stringify({ content: "Thread title" }), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      });
    });
    globalThis.fetch = fetchMock as typeof fetch;

    await generateThreadTitle("User says hello", "Assistant replies");

    expect(fetchMock).toHaveBeenCalledTimes(1);
    const call = fetchMock.mock.calls[0];
    const init = call?.[1];
    const sessionIdHeader = getHeader(init, "X-UAR-Session-ID");
    expect(sessionIdHeader).toBeNull();

    const body = typeof init?.body === "string" ? JSON.parse(init.body) as Record<string, unknown> : null;
    expect(body).toBeObject();
    expect(body?.stream).toBe(false);
    expect(Array.isArray(body?.messages)).toBe(true);

    const messages = body?.messages as Array<{ role?: string; content?: string }>;
    expect(messages[0]?.role).toBe("user");
    expect(typeof messages[0]?.content).toBe("string");
    expect(messages[0]?.content).toContain("Generate a concise 4-6 word title");
  });
});
