import { useMemo } from "react";

import { useRunTrace } from "@/features/chat/model/use-run-trace";
import type {
  RunTraceCanonicalPhase,
  RunTraceContext,
} from "@/features/chat/model/run-trace-types";
import { RunInspector } from "@/features/chat/ui/run-inspector";
import { RunTraceBar } from "@/features/chat/ui/run-trace-bar";
import { RunTraceTimeline } from "@/features/chat/ui/run-trace-timeline";

export function RunTracePanel({
  context,
  onOpenConversation,
  onRunHandoff,
  supplemental,
}: {
  context: RunTraceContext;
  onOpenConversation?: (threadId: string, messageId: string) => void;
  onRunHandoff?: (runId: string) => void;
  supplemental?: React.ReactNode;
}) {
  const trace = useRunTrace(context);
  const selectedNode = trace.selectedNodeId
    ? trace.projection.nodesById.get(trace.selectedNodeId)
    : undefined;
  const timing = selectedNode?.type === "event"
    ? trace.projection.timingsByEventId.get(selectedNode.event.eventId)
    : undefined;
  const selectedPhase = selectedNode?.type === "phase"
    ? selectedNode.phase
    : selectedNode?.type === "event" ? selectedNode.phase : null;
  const canonicalSelectedPhase = selectedPhase === "lifecycle" ? null : selectedPhase;
  const canResume = trace.checkpoints.length > 0 && trace.agentArtifact !== null;

  const selectPhase = (phase: RunTraceCanonicalPhase): void => {
    trace.selectPhase(phase);
  };
  const resume = async (): Promise<void> => {
    const response = await trace.resumeLatestCheckpoint();
    if (response) onRunHandoff?.(response.run_id);
  };

  const timeline = useMemo(() => (
    <RunTraceTimeline
      projection={trace.projection}
      filters={trace.filters}
      onToggleFilter={trace.toggleFilter}
      onToggleExpanded={trace.toggleExpanded}
      onSelectNode={trace.selectNode}
    />
  ), [trace.filters, trace.projection, trace.selectNode, trace.toggleExpanded, trace.toggleFilter]);

  return (
    <section aria-label={`Trace for run ${context.runId}`} className="grid min-h-0 flex-1 gap-3 overflow-auto bg-background p-3 xl:grid-cols-[minmax(0,1fr)_340px]">
      <div className="flex min-h-[32rem] min-w-0 flex-col gap-3 rounded-xl bg-surface p-3">
        {trace.network.snapshot.status === "loading" && (
          <p className="rounded-lg bg-card px-3 py-2 text-sm text-fg-sub" role="status">
            Loading the local run trace…
          </p>
        )}
        {trace.network.snapshot.status === "error" && (
          <p className="rounded-lg bg-card px-3 py-2 text-sm text-destructive" role="alert">
            Local run trace unavailable: {trace.network.snapshot.error}
          </p>
        )}
        <RunTraceBar
          segments={trace.projection.segments}
          selectedPhase={canonicalSelectedPhase}
          onSelectPhase={selectPhase}
        />
        {timeline}
      </div>
      <RunInspector
        context={context}
        selectedNode={selectedNode}
        timing={timing}
        checkpoints={trace.checkpoints}
        selectedCheckpointId={trace.selectedCheckpointId}
        replay={trace.replay}
        network={trace.network}
        canResume={canResume}
        onSelectCheckpoint={(id) => trace.selectCheckpoint(id)}
        onRefreshReplay={() => void trace.refreshReplay()}
        onResume={() => void resume()}
        onOpenConversation={onOpenConversation}
        supplemental={supplemental}
      />
    </section>
  );
}
