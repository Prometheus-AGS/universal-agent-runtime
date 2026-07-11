import { beforeEach, describe, expect, test } from "vitest";
import { useGraphStore } from "@prometheus-ags/prometheus-entity-management";
import type { EntityType } from "@prometheus-ags/prometheus-entity-management";
import { ingestRuntimeEvent } from "./runtime-ingest";
import {
  RUNTIME_REPLAY_AGUI_ID,
  RUNTIME_REPLAY_A2UI_ID,
  RUNTIME_REPLAY_APPROVAL_ID,
  RUNTIME_REPLAY_ARTIFACT_ID,
  RUNTIME_REPLAY_MEMORY_ID,
  RUNTIME_REPLAY_PROVIDER_ID,
  RUNTIME_REPLAY_ROUTE_ID,
  RUNTIME_REPLAY_RUN_ID,
  RUNTIME_REPLAY_TOOL_ID,
  RUNTIME_REPLAY_WORKFLOW_MIRROR_MEMORY_ID,
  replayAgUiEvents,
  replayAllRuntimeFixtures,
  replayRuntimeEvents,
  replayRuntimeUpdates,
  resetRuntimeReplayGraph,
  runtimeReplayEntityTypes,
} from "./runtime-replay-fixtures";

function entity<T extends Record<string, unknown>>(type: EntityType, id: string) {
  return useGraphStore.getState().entities[type]?.[id] as T | undefined;
}

function entityCount(type: EntityType) {
  return Object.keys(useGraphStore.getState().entities[type] ?? {}).length;
}

describe("runtime event replay entity sync", () => {
  beforeEach(() => {
    resetRuntimeReplayGraph();
  });

  test("replay fixtures cover all runtime console entity types without provider calls", () => {
    replayAllRuntimeFixtures();

    for (const entityType of runtimeReplayEntityTypes) {
      expect(entityCount(entityType), entityType).toBeGreaterThan(0);
    }
  });

  test("health, memory, routing, surfaces, and artifacts retain one run correlation", () => {
    replayAllRuntimeFixtures();

    const correlatedEntities = [
      entity("RuntimeArtifact", RUNTIME_REPLAY_ARTIFACT_ID),
      entity("RuntimeMemoryEvent", RUNTIME_REPLAY_MEMORY_ID),
      entity("RuntimeModelRouteDecision", RUNTIME_REPLAY_ROUTE_ID),
      entity("RuntimeA2uiSurface", RUNTIME_REPLAY_A2UI_ID),
      entity("RuntimeProviderHealth", `provider-health-${RUNTIME_REPLAY_PROVIDER_ID}`),
    ];

    for (const correlated of correlatedEntities) {
      expect(correlated?.run_id).toBe(RUNTIME_REPLAY_RUN_ID);
    }

    expect(entity("RuntimeRun", RUNTIME_REPLAY_RUN_ID)?.provider_id).toBe(
      RUNTIME_REPLAY_PROVIDER_ID,
    );
    expect(entity("RuntimeModelRouteDecision", RUNTIME_REPLAY_ROUTE_ID)?.selected_provider).toBe(
      RUNTIME_REPLAY_PROVIDER_ID,
    );
  });

  test("ingestRuntimeEvent maps replayed events to runtime entity types", () => {
    replayAllRuntimeFixtures();

    expect(entity("RuntimeRun", RUNTIME_REPLAY_RUN_ID)?.status).toBe("waiting");
    expect(entity("RuntimeRunStep", "replay-step-001")?.kind).toBe("tool");
    expect(entity("RuntimeToolCall", RUNTIME_REPLAY_TOOL_ID)?.tool_name).toBe("provider.health.check");
    expect(entity("RuntimeApproval", RUNTIME_REPLAY_APPROVAL_ID)?.status).toBe("pending");
    expect(entity("RuntimeArtifact", "replay-artifact-001")?.title).toBe("Replay Diagnostics Artifact");
    expect(entity("RuntimeMemoryEvent", "replay-memory-001")?.action).toBe("recall");
    expect(entity("RuntimeMemoryEvent", RUNTIME_REPLAY_WORKFLOW_MIRROR_MEMORY_ID)?.action).toBe("workflow_mirror");
    expect(entity("RuntimeA2uiSurface", "replay-a2ui-001")?.title).toBe("Replay A2UI Surface");
    expect(entity("RuntimeModelRouteDecision", "replay-route-001")?.selected_model).toBe("gpt-5.4");
    expect(entity("RuntimeProviderHealth", "provider-health-openai")?.latency_ms).toBe(42);
  });

  test("replayed ids are stable across explicit ids, payload ids, and sequence fallback", () => {
    ingestRuntimeEvent({
      type: "provider_health",
      run_id: RUNTIME_REPLAY_RUN_ID,
      sequence: 21,
      payload: {
        id: "payload-provider-health",
        provider_id: "payload-provider",
        status: "healthy",
      },
    });
    ingestRuntimeEvent({
      type: "memory_updated",
      run_id: RUNTIME_REPLAY_RUN_ID,
      sequence: 22,
      payload: {
        action: "update",
        summary: "Sequence fallback memory event",
      },
    });

    expect(entity("RuntimeProviderHealth", "payload-provider-health")?.provider_id).toBe("payload-provider");
    expect(entity("RuntimeMemoryEvent", `RuntimeMemoryEvent:${RUNTIME_REPLAY_RUN_ID}:22`)?.summary).toBe("Sequence fallback memory event");
  });

  test("later replayed events update existing entities without duplicate logical state", () => {
    replayRuntimeEvents();
    expect(entity("RuntimeApproval", RUNTIME_REPLAY_APPROVAL_ID)?.status).toBe("pending");
    expect(entityCount("RuntimeApproval")).toBe(1);

    ingestRuntimeEvent({
      type: "approval_updated",
      id: RUNTIME_REPLAY_APPROVAL_ID,
      run_id: RUNTIME_REPLAY_RUN_ID,
      sequence: 30,
      payload: {
        status: "approved",
        reason: "Replay approved",
      },
    });

    expect(entity("RuntimeApproval", RUNTIME_REPLAY_APPROVAL_ID)?.status).toBe("approved");
    expect(entity("RuntimeApproval", RUNTIME_REPLAY_APPROVAL_ID)?.reason).toBe("Replay approved");
    expect(entityCount("RuntimeApproval")).toBe(1);
  });

  test("later replayed A2UI surface events replace stale surface state", () => {
    replayAllRuntimeFixtures();
    expect(entity("RuntimeA2uiSurface", RUNTIME_REPLAY_A2UI_ID)?.payload).toEqual({
      body: "Replay payload rendered from entity graph",
    });

    replayRuntimeUpdates();

    expect(entity("RuntimeA2uiSurface", RUNTIME_REPLAY_A2UI_ID)?.payload).toEqual({
      body: "Replay payload updated from chunk",
    });
    expect(entityCount("RuntimeA2uiSurface")).toBe(1);
  });

  test("ingestAgUiEvent creates runtime AG-UI entities with run context", () => {
    replayAgUiEvents();

    const event = entity("RuntimeAgUiEvent", RUNTIME_REPLAY_AGUI_ID);
    expect(event?.run_id).toBe(RUNTIME_REPLAY_RUN_ID);
    expect(event?.event_type).toBe("agui.stream.start");
    expect(event?.sequence).toBe(1);
    expect(event?.payload).toMatchObject({ request_id: RUNTIME_REPLAY_RUN_ID, agent_id: "replay-agent" });
    expect(typeof event?.updated_at).toBe("string");
  });

  test("workflow mirror memory events preserve workflow metadata and provided timestamps", () => {
    replayRuntimeEvents();

    const event = entity("RuntimeMemoryEvent", RUNTIME_REPLAY_WORKFLOW_MIRROR_MEMORY_ID);
    expect(event?.action).toBe("workflow_mirror");
    expect(event?.workflow_kind).toBe("waypoint");
    expect(event?.workflow_id).toBe("current-waypoint");
    expect(event?.phase).toBe("runtime-console-validation-hardening");
    expect(event?.change).toBe("surreal-memory-workflow-mirror-tests");
    expect(event?.source_path).toBe(".kbd-orchestrator/current-waypoint.json");
    expect(event?.updated_at).toBe("2026-04-26T07:00:00.000Z");
  });
});
