import { useGraphStore } from "@prometheus-ags/prometheus-entity-management";
import type { EntityType } from "@prometheus-ags/prometheus-entity-management";

const EVENT_TYPE_MAP: Record<string, EntityType> = {
  run_started: "RuntimeRun",
  run_updated: "RuntimeRun",
  run_finished: "RuntimeRun",
  run_failed: "RuntimeRun",
  step_started: "RuntimeRunStep",
  step_updated: "RuntimeRunStep",
  step_finished: "RuntimeRunStep",
  tool_call_started: "RuntimeToolCall",
  tool_call_delta: "RuntimeToolCall",
  tool_call_finished: "RuntimeToolCall",
  tool_call_failed: "RuntimeToolCall",
  approval_requested: "RuntimeApproval",
  approval_updated: "RuntimeApproval",
  artifact_created: "RuntimeArtifact",
  artifact_updated: "RuntimeArtifact",
  memory_recalled: "RuntimeMemoryEvent",
  memory_updated: "RuntimeMemoryEvent",
  a2ui_surface: "RuntimeA2uiSurface",
  model_route_decision: "RuntimeModelRouteDecision",
  provider_health: "RuntimeProviderHealth",
};

interface RuntimeEventEnvelope {
  type?: string;
  entity_type?: string;
  id?: string;
  run_id?: string;
  sequence?: number;
  data?: Record<string, unknown>;
  payload?: Record<string, unknown>;
}

function now() {
  return new Date().toISOString();
}

function eventId(event: RuntimeEventEnvelope, entityType: EntityType) {
  if (event.id) return event.id;
  if (event.data?.id && typeof event.data.id === "string") return event.data.id;
  if (event.payload?.id && typeof event.payload.id === "string") return event.payload.id;
  return `${entityType}:${event.run_id ?? "global"}:${event.sequence ?? Date.now()}`;
}

export function ingestRuntimeEvent(raw: unknown) {
  if (!raw || typeof raw !== "object") return;
  const event = raw as RuntimeEventEnvelope;
  const eventType = event.type ?? event.entity_type;
  const entityType = event.entity_type ?? (eventType ? EVENT_TYPE_MAP[eventType] : undefined);
  if (!entityType) return;

  const id = eventId(event, entityType);
  const data = {
    ...(event.payload ?? {}),
    ...(event.data ?? {}),
    id,
    run_id: event.run_id ?? event.data?.run_id ?? event.payload?.run_id,
    event_type: eventType,
    sequence: event.sequence,
    updated_at: now(),
  };

  useGraphStore.getState().upsertEntity(entityType, id, data);
}

export function ingestAgUiEvent(runId: string, event: RuntimeEventEnvelope) {
  const id = eventId(event, "RuntimeAgUiEvent");
  useGraphStore.getState().upsertEntity("RuntimeAgUiEvent", id, {
    id,
    run_id: runId,
    event_type: event.type ?? "ag_ui_event",
    sequence: event.sequence ?? Date.now(),
    payload: event.payload ?? event.data ?? {},
    updated_at: now(),
  });
}
