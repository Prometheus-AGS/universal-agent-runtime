import { describe, expect, it, vi } from "vitest";
import { UarClient, UarSdkError } from "../src/index.js";

const json = (body: unknown, status = 200) => new Response(JSON.stringify(body), { status, headers: { "content-type": "application/json" } });

describe("UarClient", () => {
  it("validates chat completions and sends authorization", async () => {
    const fetcher = vi.fn<typeof fetch>().mockResolvedValue(json({ choices: [{ message: { role: "assistant", content: "hello" } }] }));
    const client = new UarClient("https://runtime.example/", { apiKey: "secret", fetch: fetcher });
    const result = await client.chat.complete({ messages: [{ role: "user", content: "hi" }] });
    expect(result.choices[0]?.message.content).toBe("hello");
    expect(fetcher).toHaveBeenCalledWith("https://runtime.example/api/chat/completion", expect.objectContaining({ method: "POST" }));
    const init = fetcher.mock.calls[0]?.[1];
    expect(new Headers(init?.headers).get("authorization")).toBe("Bearer secret");
  });

  it("maps the complete run lifecycle", async () => {
    const fetcher = vi.fn<typeof fetch>()
      .mockResolvedValueOnce(json({ run_id: "run-1", stream_url: "/stream" }))
      .mockResolvedValueOnce(json({ cancelled: true }))
      .mockResolvedValueOnce(json({ run_id: "run-1", checkpoints: [] }))
      .mockResolvedValueOnce(json({ run_id: "run-2", stream_url: "/stream-2", resumed_from_run_id: "run-1" }));
    const client = new UarClient("https://runtime.example", { fetch: fetcher });
    await client.runs.create({ artifact: { name: "agent" }, input: "go" });
    await client.runs.cancel("run-1");
    await client.runs.checkpoints("run-1");
    await client.runs.resume("run-1", { artifact: { name: "agent" } }, "checkpoint/1");
    expect(fetcher.mock.calls.map(([url]) => url)).toEqual([
      "https://runtime.example/api/uar/runs",
      "https://runtime.example/api/uar/runs/run-1/cancel",
      "https://runtime.example/api/uar/runs/run-1/checkpoints",
      "https://runtime.example/api/uar/runs/run-1/resume/checkpoint%2F1",
    ]);
  });

  it("preserves failed response status and details", async () => {
    const client = new UarClient("https://runtime.example", { fetch: vi.fn<typeof fetch>().mockResolvedValue(json({ error: "denied" }, 403)) });
    await expect(client.tools.execute("private")).rejects.toMatchObject({ status: 403, details: { error: "denied" } } satisfies Partial<UarSdkError>);
  });

  it("rejects malformed successful responses", async () => {
    const client = new UarClient("https://runtime.example", { fetch: vi.fn<typeof fetch>().mockResolvedValue(json({ choices: "wrong" })) });
    await expect(client.chat.complete({ messages: [] })).rejects.toThrow("validation failed");
  });
});
