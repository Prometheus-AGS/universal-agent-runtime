import { describe, expect, test } from "vitest";
import { matchesServerHistory, missingServerMessages } from "./session-backfill";
import { toChunks } from "@/shared/content/to-chunks";
import type { ContentBlock, RichMessage } from "@/types/chat-content";
import type { ServerSessionMessage } from "@/services/chat-sessions-api";

describe("owner history projection", () => {
  test("retains stable identities, rich images and interrupted truth", () => {
    const server = [{ message_id: "original-user", role: "user" as const, content: [
      { type: "text" as const, text: "inspect" }, { type: "image" as const, url: "data:image/png;base64,eA==", mime: "image/png", dataBase64: null }], status: "complete" },
    { message_id: "run-r-assistant-0", run_id: "r", role: "assistant" as const, content: "partial", status: "interrupted", created_at: null, provenance: "run_events" }];
    const first = missingServerMessages([], server);
    const second = missingServerMessages([], server);
    expect(first.map((row) => row.id)).toEqual(second.map((row) => row.id));
    expect(first[0].content).toContainEqual(expect.objectContaining({ type: "image", url: "data:image/png;base64,eA==" }));
    expect(first[1]).toMatchObject({ id: "run-r-assistant-0", runId: "r", status: "interrupted" });
    expect(first[1].createdAt.getTime()).toBe(0);
    expect(missingServerMessages(first, server)).toEqual([]);
  });
  test("legacy history never invents complete status or current time", () => {
    const rows = missingServerMessages([], [{ role: "assistant", content: "legacy" }]);
    expect(rows[0].id).toBe("legacy:0:assistant");
    expect(rows[0].status).toBe("interrupted");
    expect(rows[0].createdAt.getTime()).toBe(0);
  });
  test("historical approvals retain settled decision and cannot become fresh authorization", () => {
    const rows = missingServerMessages([], [{ role: "assistant", message_id: "a", content: "answer", status: "complete", run_id: "r",
      run_metadata: { approvals_availability: "available", approvals: [{ approval_id: "approval", tool_call_id: "call", tool_name: "write",
        state: "approved", historical: true, resolvable: false, decision: { approved: true, decided_at: "2026-11-02T00:00:00Z" } }] } }]);
    const chunks = toChunks(rows[0].content, { messageId: "a", at: new Date(0).toISOString(), runId: "r", finalized: true });
    expect(chunks).toContainEqual(expect.objectContaining({ kind: "tool-approval", historical: true, decision: "approved", approvalId: "approval" }));
  });
});


test("a cached run segment does not suppress later canonical identities or repeat run evidence", () => {
  const metadata = { usage_availability: "available", usage: { input_tokens: 1, output_tokens: 2, total_tokens: 3 },
    approvals: [{ approval_id: "approved", tool_call_id: "call", state: "approved", decision: { approved: true } }] };
  const server = [
    { message_id: "user", role: "user" as const, content: "question", status: "complete" },
    { message_id: "run-r-assistant-0", run_id: "r", role: "assistant" as const, content: "preamble", run_metadata: metadata },
    { message_id: "run-r-assistant-1", run_id: "r", role: "assistant" as const, content: "final", status: "complete", run_metadata: metadata },
  ];
  const canonical = missingServerMessages([], server);
  const missing = missingServerMessages(canonical.slice(0, 2), server);
  expect(missing.map((row) => row.id)).toEqual(["run-r-assistant-1"]);
  expect(missing[0].historyPosition).toBe(2);
  const projected = canonical.flatMap((row) => toChunks(row.content, { messageId: row.id, at: new Date(0).toISOString(), runId: row.runId, finalized: true }));
  expect(projected.filter((chunk) => chunk.kind === "usage")).toHaveLength(1);
  expect(projected.filter((chunk) => chunk.kind === "tool-approval")).toHaveLength(1);
  expect(missingServerMessages([...canonical.slice(0, 2), ...missing], server)).toEqual([]);
});

test("legacy user aliases remain matchable while divergent user history is refused", () => {
  const local = [{ id: "original-local-user", role: "user" as const, content: [{ type: "text" as const, text: "question" }], createdAt: new Date(0) },
    { id: "original-local-answer", role: "assistant" as const, content: [{ type: "text" as const, text: "legacy answer" }], createdAt: new Date(0) }];
  const server = [{ message_id: "legacy:session:0:user", role: "user" as const, content: "question" },
    { message_id: "legacy:session:1:assistant", role: "assistant" as const, content: "legacy answer" }];
  expect(matchesServerHistory(local, server)).toBe(true);
  expect(missingServerMessages(local, server)).toEqual([]);
  expect(matchesServerHistory(local, [{ ...server[0], content: "other question" }])).toBe(false);
  expect(missingServerMessages(local, [{ ...server[0], content: "other question" }])).toEqual([]);
});

describe("user history identity", () => {
  const image = (value: string): ContentBlock => ({ type: "image", url: `data:image/png;base64,${value}`, mime: "image/png", dataBase64: null });
  const caption: ContentBlock = { type: "text", text: "inspect" };
  const cases: { name: string; local: ContentBlock[]; remote: ContentBlock[]; modern?: boolean; turn?: boolean; run?: boolean }[] = [
    { name: "equal captions with different images", local: [caption, image("eA==")], remote: [caption, image("eQ==")] },
    { name: "empty image-only prompts", local: [image("eA==")], remote: [image("eQ==")] },
    { name: "reordered images", local: [caption, image("eA=="), image("eQ==")], remote: [caption, image("eQ=="), image("eA==")] },
    { name: "mixed text and image order", local: [caption, image("eA==")], remote: [image("eA=="), caption] },
    { name: "identical media without stable identity", local: [caption, image("eA==")], remote: [caption, image("eA==")] },
    { name: "path-only media", local: [caption, { type: "image", url: null, dataBase64: null, mime: "image/png", path: "owned/original.png" }], remote: [caption] },
    { name: "canonical image_url media", local: [caption, image("eA==")], remote: [caption,
      { type: "image_url", image_url: { url: "data:image/png;base64,eA==" } } as unknown as ContentBlock] },
    { name: "unsupported media representation", local: [caption], remote: [caption,
      { type: "image_url", image_url: { file_id: "unresolved-image" } } as unknown as ContentBlock] },
    { name: "modern conflicting stable IDs", local: [caption], remote: [caption], modern: true },
    { name: "conflicting client turns on a legacy alias", local: [caption], remote: [caption], turn: true },
    { name: "run evidence on a legacy alias", local: [caption], remote: [caption], run: true },
    { name: "empty legacy text", local: [{ type: "text", text: " " }], remote: [{ type: "text", text: "" }] },
  ];
  for (const item of cases) test(`refuses ${item.name} without changing input`, () => {
    const local: RichMessage[] = [{ id: "original-user", role: "user", content: item.local, createdAt: new Date(0),
      ...(item.turn ? { clientTurnId: "original-turn" } : {}), ...(item.run ? { runId: "original-run" } : {}) }];
    const remote: ServerSessionMessage[] = [{ message_id: item.modern ? "different-user" : "legacy:thread:0:user", role: "user", content: item.remote,
      ...(item.turn ? { client_turn_id: "different-turn" } : {}) },
    { message_id: "different-answer", role: "assistant", content: "must not be added" }];
    const original = structuredClone(local);
    expect(matchesServerHistory(local, remote)).toBe(false);
    expect(missingServerMessages(local, remote)).toEqual([]);
    expect(local).toEqual(original);
  });

  test("exact stable identity keeps canonical image projection authoritative", () => {
    const local: RichMessage[] = [{ id: "original-user", role: "user", content: [caption, image("eA==")], createdAt: new Date(0), clientTurnId: "original-turn" }];
    for (const content of [[caption, image("eA==")], [caption, image("eQ==")], [caption,
      { type: "image_url", image_url: { url: "data:image/png;base64,eA==" } } as unknown as ContentBlock]]) {
      const remote: ServerSessionMessage[] = [{ message_id: "original-user", role: "user", content, client_turn_id: "original-turn" }];
      expect(matchesServerHistory(local, remote)).toBe(true);
      expect(missingServerMessages(local, remote)).toEqual([]);
      expect(missingServerMessages([], remote)[0].content).toContainEqual(expect.objectContaining({ type: "image" }));
    }
  });

  test("legacy nonempty text aliases still use trimmed ordered matching", () => {
    const local: RichMessage[] = ["first", "second"].map((text, index) => ({ id: `cached-${index}`, role: "user", content: [{ type: "text", text: ` ${text} ` }], createdAt: new Date(0) }));
    const remote: ServerSessionMessage[] = ["first", "second"].map((content, index) => ({ message_id: `legacy:thread:${index}:user`, role: "user", content }));
    expect(matchesServerHistory(local, remote)).toBe(true);
    expect(matchesServerHistory(local, [...remote].reverse())).toBe(false);
  });
});
