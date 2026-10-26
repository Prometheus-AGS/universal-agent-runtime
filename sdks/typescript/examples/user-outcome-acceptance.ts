/** Actual shipped SDK acceptance client. Run only under the approved packet. */
import { writeFileSync } from "node:fs";
import { z } from "zod";
import { UarClient } from "../src/index.js";

const inputSchema = z.object({
  base_url: z.string().url(), authorization_env: z.string().min(1),
  agent_id: z.string().min(1), model: z.string().min(1), session_id: z.string().uuid(),
  mode: z.enum(["json", "stream"]), turns: z.array(z.string()).length(2),
  expected: z.array(z.record(z.string(), z.unknown())).length(2),
  timeout_seconds: z.number().positive(), max_bytes: z.number().int().positive(),
});
type Input = z.infer<typeof inputSchema>;
type Turn = { content: string; run_id: string; session_id: string; protocol_terminal: "success" };

function requireValue(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}
function object(value: unknown): Record<string, unknown> {
  requireValue(typeof value === "object" && value !== null && !Array.isArray(value), "object required");
  return value as Record<string, unknown>;
}
function useful(text: string, expected: Record<string, unknown>): void {
  const value = object(JSON.parse(text) as unknown);
  requireValue(JSON.stringify(Object.keys(value).sort()) === JSON.stringify(Object.keys(expected).sort())
    && Object.keys(expected).every((key) => value[key] === expected[key]), "useful-result predicate failed");
}

async function acceptance(input: Input): Promise<Record<string, unknown>> {
  const authorization = process.env[input.authorization_env];
  requireValue(typeof authorization === "string" && authorization.startsWith("Bearer ") && authorization.length > 7, "bearer credential required");
  const client = new UarClient(input.base_url, {
    apiKey: authorization.slice(7),
    fetch: (url, init) => globalThis.fetch(url, { ...init,
      signal: AbortSignal.any([...(init?.signal ? [init.signal] : []), AbortSignal.timeout(input.timeout_seconds * 1000)]) }),
  });
  const turns: Turn[] = [];
  for (let index = 0; index < input.turns.length; index++) {
    const text = input.turns[index];
    const expected = input.expected[index];
    requireValue(text && expected, "exact two-turn input required");
    const request = { messages: [{ role: "user" as const, content: text }],
      model: input.model, agentId: input.agent_id, sessionId: input.session_id };
    let content: string;
    let identifier: string;
    if (input.mode === "json") {
      const response = await client.chat.complete(request);
      const choice = response.choices[0];
      requireValue(response.choices.length === 1 && choice?.finish_reason === "stop", "SDK completion failed");
      requireValue(typeof choice.message.content === "string" && typeof response.id === "string", "completion identity/content missing");
      requireValue(response.model === input.model, "completion model mismatch");
      content = choice.message.content;
      identifier = response.id;
    } else {
      const fragments: string[] = [];
      const identifiers = new Set<string>();
      let terminal = 0;
      let done = 0;
      for await (const event of client.chat.stream(request, { signal: AbortSignal.timeout(input.timeout_seconds * 1000) })) {
        if (event.data === "[DONE]") { done++; continue; }
        const data = object(event.data);
        requireValue(!data.error && !["error", "agui.error", "agui.cancelled"].includes(event.event ?? ""), "SDK stream failed");
        if (data.object !== "chat.completion.chunk") continue;
        requireValue(typeof data.id === "string", "stream identity missing");
        identifiers.add(data.id);
        requireValue(Array.isArray(data.choices), "stream choices missing");
        for (const raw of data.choices) {
          const choice = object(raw);
          const delta = object(choice.delta);
          if (typeof delta.content === "string") fragments.push(delta.content);
          if (choice.finish_reason !== undefined && choice.finish_reason !== null) {
            requireValue(choice.finish_reason === "stop", "SDK stream terminal failed");
            terminal++;
          }
        }
        requireValue(Buffer.byteLength(fragments.join("")) <= input.max_bytes, "SDK stream output exceeded bound");
      }
      requireValue(terminal === 1 && done === 1 && identifiers.size === 1, "SDK stream lacks unique successful terminal");
      content = fragments.join("");
      const first = identifiers.values().next().value as string | undefined;
      requireValue(first, "stream identity missing");
      identifier = first;
    }
    useful(content, expected);
    requireValue(identifier.startsWith("chatcmpl-"), "SDK native run correlation missing");
    turns.push({ content, run_id: identifier.slice("chatcmpl-".length),
      session_id: input.session_id, protocol_terminal: "success" });
  }
  return { status: "PASS", client: "typescript_sdk", mode: input.mode, turns };
}

async function main(): Promise<void> {
  const output = process.env.UAR_ACCEPTANCE_CLIENT_OUTPUT;
  requireValue(output, "exclusive client receipt path required");
  let receipt: Record<string, unknown>;
  try {
    const raw = process.env.UAR_ACCEPTANCE_CLIENT_INPUT;
    requireValue(raw, "client input required");
    receipt = await acceptance(inputSchema.parse(JSON.parse(raw) as unknown));
  } catch (error: unknown) {
    receipt = { status: "FAIL", client: "typescript_sdk", error_type: error instanceof Error ? error.name : "UnknownError",
      error: error instanceof Error ? error.message.slice(0, 8192) : "Non-error rejection" };
    process.exitCode = 1;
  }
  writeFileSync(output, JSON.stringify(receipt), { flag: "wx", mode: 0o600 });
}
await main();
