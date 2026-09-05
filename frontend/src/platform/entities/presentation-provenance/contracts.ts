import { z } from "zod";
import type { PersistedRunSnapshot } from "@/platform/pglite/run-event-repository";

export const PRESENTATION_PROVENANCE_ENTITY = "PresentationProvenance";
const revision = z.number().int().positive().max(Number.MAX_SAFE_INTEGER);
const fallbackReason = z.enum([
  "client_rendering_not_declared", "incompatible_profile", "no_eligible_templates",
  "parent_text_ceiling", "no_surface_published", "surface_generation_failed",
]).nullable();

export const presentationObservationSchema = z.object({
  version: z.literal(1),
  requested_mode: z.enum(["auto", "text", "a2ui", "hybrid"]).nullable(),
  effective_mode: z.enum(["legacy", "auto", "text", "a2ui", "hybrid"]),
  admission_fallback_reason: fallbackReason,
  fallback_reason: fallbackReason,
  run_outcome: z.enum(["running", "finished", "failed", "cancelled"]),
  eligible_templates: z.array(z.object({ presentation_id: z.string().min(1), revision }).strict()),
  published_templates: z.array(z.object({ template_id: z.string().min(1), revision }).strict()),
  surface_published: z.boolean(),
  generation_failed: z.boolean(),
  receipt_status: z.enum(["available", "unavailable"]),
  client_display: z.literal("unconfirmed"),
}).strict();

export type PresentationObservation = z.infer<typeof presentationObservationSchema>;
export type PresentationProvenance = {
  id: string;
  admission_id: string;
  status: "loading" | "ready" | "missing" | "unsupported" | "error" | "idle";
  source_event_id: string | null;
  source_sequence: number | null;
  observation: PresentationObservation | null;
};

function object(value: unknown): Record<string, unknown> | undefined {
  return value !== null && typeof value === "object" && !Array.isArray(value)
    ? value as Record<string, unknown> : undefined;
}

/** Select by wire order, not the order in which a reconnect persisted events. */
export function decodePresentationProvenance(runId: string, snapshot: PersistedRunSnapshot): Pick<PresentationProvenance, "status" | "source_event_id" | "source_sequence" | "observation"> {
  let candidate: { value: unknown; eventId: string; sequence: number } | undefined;
  const seen = new Set<string>();
  const events = snapshot.events.filter((event) => event.runId === runId
    && (event.payload.runId === undefined || event.payload.runId === runId)
    && Number.isSafeInteger(event.wireSequence) && event.wireSequence >= 0)
    .sort((left, right) => left.wireSequence - right.wireSequence || left.seq - right.seq);
  for (const event of events) {
    if (seen.has(event.eventId)) continue;
    seen.add(event.eventId);
    const select = (value: unknown) => { candidate = { value, eventId: event.eventId, sequence: event.wireSequence }; };
    if (event.type === "CUSTOM" && event.payload.name === "uar.presentation.snapshot") {
      select(event.payload.value);
    } else if (event.type === "STATE_SNAPSHOT") {
      select(object(event.payload.snapshot)?.presentation);
    } else if (event.type === "STATE_DELTA" && Array.isArray(event.payload.delta)) {
      for (const rawOperation of event.payload.delta) {
        const operation = object(rawOperation);
        if (!operation) continue;
        if (operation.path === "/presentation") {
          select(operation.op === "add" || operation.op === "replace" ? operation.value : undefined);
        } else if (operation.path === "" || operation.path === "/") {
          select(object(operation.value)?.presentation);
        }
      }
    }
  }
  const source = { source_event_id: candidate?.eventId ?? null, source_sequence: candidate?.sequence ?? null };
  if (candidate?.value === undefined) return { ...source, status: "missing", observation: null };
  const parsed = presentationObservationSchema.safeParse(candidate.value);
  return parsed.success
    ? { ...source, status: "ready", observation: parsed.data }
    : { ...source, status: "unsupported", observation: null };
}
