import { A2uiMessageSchema, MessageProcessor, type A2uiMessage } from "@prometheus-ags/a2ui-core/v0_9";
import { uarBasicCatalog } from "@prometheus-ags/a2ui-uar";
import { createStore } from "zustand/vanilla";
import type { A2uiStreamService } from "../services/a2ui-stream-service";
import type { InspectedMessage, InspectorConnection, InspectorSurface } from "../types";

const kindOf = (value: unknown) => value && typeof value === "object" ? Object.keys(value as object).find((key) => key !== "version") ?? "message" : "invalid";
export interface InspectorState {
  connection: InspectorConnection; messages: InspectedMessage[]; selectedId?: number; frozen: boolean; queued: number; dropped: number; filter: string; surface?: InspectorSurface;
  ingest(raw: unknown): void; select(id: number): void; toggleFreeze(): void; setFilter(value: string): void; copySelected(): Promise<void>; clear(): void; connect(service: A2uiStreamService): () => void; retry(): void;
}
export function createInspectorStore(maxMessages = 500, copyText: (text: string) => Promise<void> = (text) => navigator.clipboard.writeText(text)) {
  const processor = new MessageProcessor([uarBasicCatalog]);
  let nextId = 1;
  const pending: A2uiMessage[] = [];
  let activeService: A2uiStreamService | undefined;
  let disconnect: (() => void) | undefined;
  const process = (message: A2uiMessage) => { processor.processMessages([message]); const surfaceId = "createSurface" in message ? message.createSurface.surfaceId : "updateComponents" in message ? message.updateComponents.surfaceId : "updateDataModel" in message ? message.updateDataModel.surfaceId : "deleteSurface" in message ? message.deleteSurface.surfaceId : undefined; return surfaceId ? processor.model.getSurface(surfaceId) : undefined; };
  return createStore<InspectorState>((set, get) => ({
    connection: "idle", messages: [], frozen: false, queued: 0, dropped: 0, filter: "",
    ingest(raw) {
      const result = A2uiMessageSchema.safeParse(raw);
      const inspected: InspectedMessage = { id: nextId++, receivedAt: new Date().toISOString(), raw, valid: result.success, error: result.success ? undefined : result.error.issues.map((issue) => `${issue.path.join(".") || "message"}: ${issue.message}`).join("; "), kind: kindOf(raw) };
      const all = [...get().messages, inspected]; const overflow = Math.max(0, all.length - maxMessages); const messages = overflow ? all.slice(overflow) : all;
      if (!result.success) { set({ messages, dropped: get().dropped + overflow, selectedId: inspected.id }); return; }
      if (get().frozen) { pending.push(result.data); set({ messages, dropped: get().dropped + overflow, queued: pending.length }); return; }
      const surface = process(result.data);
      set({ messages, dropped: get().dropped + overflow, selectedId: inspected.id, surface: surface ?? get().surface });
    },
    select: (selectedId) => set({ selectedId }),
    toggleFreeze() { if (!get().frozen) { set({ frozen: true }); return; } let surface = get().surface; for (const message of pending.splice(0)) surface = process(message) ?? surface; set({ frozen: false, queued: 0, surface, selectedId: get().messages.at(-1)?.id }); },
    setFilter: (filter) => set({ filter }),
    async copySelected() { const message = get().messages.find(({ id }) => id === get().selectedId); if (message) await copyText(JSON.stringify(message.raw, null, 2)); },
    clear: () => { pending.length = 0; set({ messages: [], selectedId: undefined, queued: 0, dropped: 0, surface: undefined }); },
    connect(service) { activeService = service; disconnect?.(); disconnect = service.connect((payload) => get().ingest(payload), (connection) => set({ connection })); return () => { disconnect?.(); disconnect = undefined; }; },
    retry() { if (activeService) get().connect(activeService); },
  }));
}
export type InspectorStore = ReturnType<typeof createInspectorStore>;
