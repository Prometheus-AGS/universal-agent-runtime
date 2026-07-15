import { describe, expect, it, vi } from "vitest";
import { createInspectorStore } from "../src/stores/inspector-store";

const create = { version: "v0.9" as const, createSurface: { surfaceId: "s", catalogId: "urn:uar:a2ui:catalog:1" } };
const update = { version: "v0.9" as const, updateComponents: { surfaceId: "s", components: [{ id: "root", component: "Text", text: "Hello" }] } };
describe("Inspector store", () => {
  it("buffers valid messages while frozen and applies them on resume", () => { const store = createInspectorStore(); store.getState().ingest(create); store.getState().toggleFreeze(); store.getState().ingest(update); expect(store.getState()).toMatchObject({ frozen: true, queued: 1 }); store.getState().toggleFreeze(); expect(store.getState().surface?.componentsModel.get("root")?.type).toBe("Text"); expect(store.getState().queued).toBe(0); });
  it("preserves the last-good surface for malformed messages", () => { const store = createInspectorStore(); store.getState().ingest(create); const surface = store.getState().surface; store.getState().ingest({ bad: true }); expect(store.getState().surface).toBe(surface); expect(store.getState().messages.at(-1)?.valid).toBe(false); });
  it("bounds history and reports dropped messages", () => { const store = createInspectorStore(2); store.getState().ingest({ bad: 1 }); store.getState().ingest({ bad: 2 }); store.getState().ingest({ bad: 3 }); expect(store.getState().messages).toHaveLength(2); expect(store.getState().dropped).toBe(1); });
  it("copies selected JSON through the injected service", async () => { const copy = vi.fn(async () => undefined); const store = createInspectorStore(2, copy); store.getState().ingest({ bad: true }); await store.getState().copySelected(); expect(copy).toHaveBeenCalledWith(expect.stringContaining('"bad": true')); });
});
