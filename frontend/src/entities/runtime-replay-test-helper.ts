import { useGraphStore } from "@/platform/entities";
import type { EntityType } from "@/platform/entities";
import {
  RUNTIME_REPLAY_APPROVAL_ID,
  RUNTIME_REPLAY_RUN_ID,
  replayAllRuntimeFixtures,
  replayRuntimeEvents,
  replayRuntimeUpdates,
  resetRuntimeReplayGraph,
  runtimeReplayEntityTypes,
} from "./runtime-replay-fixtures";

interface RuntimeReplayTestHelper {
  reset: () => void;
  replayAll: () => void;
  replayUpdates: () => void;
  replayApprovalStatus: (status: "approved" | "denied" | "expired") => void;
  snapshot: () => Record<string, number>;
}

declare global {
  interface Window {
    __uarRuntimeReplay?: RuntimeReplayTestHelper;
  }
}

function entityCount(type: EntityType) {
  return Object.keys(useGraphStore.getState().entities[type] ?? {}).length;
}

export function installRuntimeReplayTestHelper() {
  if (typeof window === "undefined") return;

  window.__uarRuntimeReplay = {
    reset: resetRuntimeReplayGraph,
    replayAll: replayAllRuntimeFixtures,
    replayUpdates: replayRuntimeUpdates,
    replayApprovalStatus: (status) => {
      replayRuntimeEvents([
        {
          type: "approval_updated",
          id: RUNTIME_REPLAY_APPROVAL_ID,
          run_id: RUNTIME_REPLAY_RUN_ID,
          sequence: Date.now(),
          payload: {
            status,
            reason: `Replay approval ${status}`,
          },
        },
      ]);
    },
    snapshot: () => Object.fromEntries(
      runtimeReplayEntityTypes.map((type) => [type, entityCount(type)]),
    ),
  };
}

