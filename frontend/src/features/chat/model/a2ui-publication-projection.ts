import { A2UI_PROFILE, A2UI_VERSION, a2uiMessageComponents, validateA2uiMessage, type A2uiMessage } from "@/features/a2ui/a2ui-protocol";
import { parseA2uiMessageSource } from "@/features/a2ui/a2ui-display-content";
import { MAX_A2UI_COMPONENTS, MAX_A2UI_MESSAGES, MAX_A2UI_SOURCE_BYTES, MAX_A2UI_SURFACES } from "@/features/a2ui/a2ui-rendering-limits";
import type { A2uiDisplayChunk } from "@/features/chat/model/chunk";

function record(value: unknown): Record<string, unknown> {
  return value && typeof value === "object" && !Array.isArray(value) ? value as Record<string, unknown> : {};
}

/** Fold ordered, host-published batches for display without altering stored history or action identity. */
export class A2uiPublicationProjection {
  private readonly active = new Map<string, A2uiMessage[]>();

  project(chunk: A2uiDisplayChunk): A2uiDisplayChunk {
    if (chunk.profile !== A2UI_PROFILE || chunk.validation !== "valid") return chunk;
    const payload = record(chunk.payload);
    const metadata = record(payload.metadata);
    const generations = record(metadata.surface_generations);
    if (typeof payload.content !== "string" || metadata.publication_status !== "published"
      || typeof metadata.source_run_id !== "string" || !metadata.source_run_id) return chunk;

    const candidate = new Map(this.active);
    const touched = new Map<string, A2uiMessage[]>();
    let extended = false;
    try {
      for (const raw of parseA2uiMessageSource(payload.content)) {
        const validated = validateA2uiMessage(raw);
        if (!validated.success || validated.data.version !== A2UI_VERSION) return chunk;
        const message = validated.data;
        const id = "createSurface" in message ? message.createSurface.surfaceId
          : "updateComponents" in message ? message.updateComponents.surfaceId
          : "updateDataModel" in message ? message.updateDataModel.surfaceId : message.deleteSurface.surfaceId;
        if (!Object.hasOwn(generations, id) || typeof generations[id] !== "string" || !generations[id]) return chunk;
        if ("createSurface" in message) {
          if (candidate.has(id) || candidate.size >= MAX_A2UI_SURFACES) return chunk;
          candidate.set(id, [message]);
          touched.set(id, [message]);
          continue;
        }
        const previous = candidate.get(id);
        if (!previous) return chunk;
        extended ||= !touched.has(id);
        const current = [...previous, message];
        if ("deleteSurface" in message) candidate.delete(id);
        else candidate.set(id, current);
        touched.set(id, current);
      }
      const cumulative = [...touched.values()].flat();
      const content = JSON.stringify(cumulative);
      if (cumulative.length > MAX_A2UI_MESSAGES
        || cumulative.reduce((count, message) => count + a2uiMessageComponents(message).length, 0) > MAX_A2UI_COMPONENTS
        || new TextEncoder().encode(content).byteLength > MAX_A2UI_SOURCE_BYTES) {
        for (const id of touched.keys()) this.active.delete(id);
        return chunk;
      }
      for (const id of touched.keys()) {
        const next = candidate.get(id);
        if (next) this.active.set(id, next);
        else this.active.delete(id);
      }
      return extended ? { ...chunk, payload: { ...payload, content } } : chunk;
    } catch {
      return chunk;
    }
  }
}
