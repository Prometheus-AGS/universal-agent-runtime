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

  test("sends /api/generate-title request payload without custom session header", async () => {
    const fetchMock = mock(async () => {
      return new Response(JSON.stringify({ title: "Thread title" }), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      });
    });
    globalThis.fetch = fetchMock as typeof fetch;

    await generateThreadTitle("User says hello", "Assistant replies");

    expect(fetchMock).toHaveBeenCalledTimes(1);
    const call = fetchMock.mock.calls[0];
    const input = call?.[0];
    const init = call?.[1];
    expect(input).toBe("/api/generate-title");
    const sessionIdHeader = getHeader(init, "X-UAR-Session-ID");
    expect(sessionIdHeader).toBeNull();

    const body = typeof init?.body === "string" ? JSON.parse(init.body) as Record<string, unknown> : null;
    expect(body).toBeObject();
    expect(typeof body?.message).toBe("string");
    expect(typeof body?.assistant_message).toBe("string");
  });
});
