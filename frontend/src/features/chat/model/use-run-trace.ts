import { useEffect, useMemo } from "react";
import { useShallow } from "zustand/react/shallow";

import { useRunTraceStore } from "@/features/chat/model/run-trace-store";
import type { RunTraceContext } from "@/features/chat/model/run-trace-types";

export function useRunTrace(context: RunTraceContext | null) {
  const selectContext = useRunTraceStore((state) => state.selectContext);
  const runId = context?.runId;
  const threadId = context?.threadId ?? null;
  const messageId = context?.messageId ?? null;
  const agentId = context?.agentId ?? null;
  const sessionId = context?.sessionId ?? null;
  const stableContext = useMemo(() => runId ? {
    runId,
    threadId,
    messageId,
    agentId,
    sessionId,
  } : null, [runId, threadId, messageId, agentId, sessionId]);
  useEffect(() => {
    void selectContext(stableContext);
    return () => {
      void useRunTraceStore.getState().dispose();
    };
  }, [stableContext, selectContext]);

  return useRunTraceStore(useShallow((state) => ({
    context: state.context,
    projection: state.projection,
    filters: state.filters,
    selectedNodeId: state.selectedNodeId,
    selectedCheckpointId: state.selectedCheckpointId,
    checkpoints: state.checkpoints,
    replay: state.replay,
    agentArtifact: state.agentArtifact,
    resumedRunId: state.resumedRunId,
    network: state.network,
    toggleFilter: state.toggleFilter,
    toggleExpanded: state.toggleExpanded,
    selectNode: state.selectNode,
    selectPhase: state.selectPhase,
    selectCheckpoint: state.selectCheckpoint,
    refreshCheckpoints: state.refreshCheckpoints,
    refreshReplay: state.refreshReplay,
    refreshAgentArtifact: state.refreshAgentArtifact,
    resumeLatestCheckpoint: state.resumeLatestCheckpoint,
  })));
}
