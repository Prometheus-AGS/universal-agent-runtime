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

  test("sends a UUID X-UAR-Session-ID header", async () => {
    const fetchMock = mock(async (_input: RequestInfo | URL, _init?: RequestInit) => {
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
    const sessionId = getHeader(init, "X-UAR-Session-ID");
    expect(sessionId).toBeString();
    expect(sessionId).toMatch(
      /^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i,
    );
  });
});
