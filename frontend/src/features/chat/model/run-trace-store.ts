import { create } from "zustand";

import {
  fetchRunCheckpoints,
  fetchRunSurfaceReplay,
  resolveRuntimeAgentArtifact,
  resumeRunFromLatestCheckpoint,
  subscribeRunTraceSnapshot,
} from "@/features/chat/api/run-trace-api";
import {
  DEFAULT_RUN_TRACE_FILTERS,
  projectA2uiReplay,
  projectRunTrace,
} from "@/features/chat/model/run-trace-projection";
import type {
  ResumeRunResponse,
  RunCheckpoint,
  RunReplayProjection,
  RunTraceContext,
  RunTraceFilters,
  RunTraceNetworkState,
  RunTracePhase,
  RunTraceProjection,
  RuntimeAgentArtifact,
} from "@/features/chat/model/run-trace-types";
import type {
  PersistedRunEventKind,
  PersistedRunSnapshot,
  PersistedRunSnapshotSubscription,
} from "@/platform/pglite/run-event-repository";

const EMPTY_SNAPSHOT: PersistedRunSnapshot = { run: null, events: [] };
const DEFAULT_EXPANSION = new Set([
  "phase:context",
  "phase:skill",
  "phase:memory",
  "phase:retrieval",
  "phase:reasoning",
  "phase:tool",
  "phase:generate",
  "phase:lifecycle",
]);

function idleNetworkState(): RunTraceNetworkState {
  const idle = { status: "idle" as const, error: null };
  return {
    snapshot: { ...idle },
    checkpoints: { ...idle },
    replay: { ...idle },
    agent: { ...idle },
    resume: { ...idle },
  };
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function initialProjection(): RunTraceProjection {
  return projectRunTrace({
    ...EMPTY_SNAPSHOT,
    filters: DEFAULT_RUN_TRACE_FILTERS,
    expandedNodeIds: new Set(),
    selectedNodeId: null,
  });
}

interface RunTraceStoreState {
  context: RunTraceContext | null;
  snapshot: PersistedRunSnapshot;
  projection: RunTraceProjection;
  filters: RunTraceFilters;
  expandedNodeIds: Set<string>;
  selectedNodeId: string | null;
  selectedCheckpointId: string | null;
  checkpoints: RunCheckpoint[];
  replay: RunReplayProjection | null;
  agentArtifact: RuntimeAgentArtifact | null;
  resumedRunId: string | null;
  network: RunTraceNetworkState;
  subscription: PersistedRunSnapshotSubscription | null;
  generation: number;
  selectContext: (context: RunTraceContext | null) => Promise<void>;
  toggleFilter: (kind: PersistedRunEventKind) => void;
  toggleExpanded: (nodeId: string) => void;
  selectNode: (nodeId: string) => void;
  selectPhase: (phase: RunTracePhase) => string | null;
  selectCheckpoint: (checkpointId: string | null) => void;
  refreshCheckpoints: () => Promise<void>;
  refreshReplay: () => Promise<void>;
  refreshAgentArtifact: () => Promise<void>;
  resumeLatestCheckpoint: (input?: string) => Promise<ResumeRunResponse | null>;
  dispose: () => Promise<void>;
}

function reproject(
  state: Pick<RunTraceStoreState, "snapshot" | "filters" | "expandedNodeIds" | "selectedNodeId">,
): Pick<RunTraceStoreState, "projection" | "selectedNodeId"> {
  const projection = projectRunTrace({
    ...state.snapshot,
    filters: state.filters,
    expandedNodeIds: state.expandedNodeIds,
    selectedNodeId: state.selectedNodeId,
  });
  return { projection, selectedNodeId: projection.selectedNodeId };
}

export const useRunTraceStore = create<RunTraceStoreState>((set, get) => ({
  context: null,
  snapshot: EMPTY_SNAPSHOT,
  projection: initialProjection(),
  filters: { ...DEFAULT_RUN_TRACE_FILTERS },
  expandedNodeIds: new Set(),
  selectedNodeId: null,
  selectedCheckpointId: null,
  checkpoints: [],
  replay: null,
  agentArtifact: null,
  resumedRunId: null,
  network: idleNetworkState(),
  subscription: null,
  generation: 0,

  selectContext: async (context) => {
    const generation = get().generation + 1;
    const previousSubscription = get().subscription;
    set({ generation, subscription: null });
    await previousSubscription?.unsubscribe();

    if (get().generation !== generation) return;
    if (!context) {
      set({
        context: null,
        snapshot: EMPTY_SNAPSHOT,
        projection: initialProjection(),
        filters: { ...DEFAULT_RUN_TRACE_FILTERS },
        expandedNodeIds: new Set(),
        selectedNodeId: null,
        selectedCheckpointId: null,
        checkpoints: [],
        replay: null,
        agentArtifact: null,
        resumedRunId: null,
        network: idleNetworkState(),
      });
      return;
    }

    const expandedNodeIds = new Set(DEFAULT_EXPANSION);
    expandedNodeIds.add(`run:${context.runId}`);
    set({
      context,
      snapshot: EMPTY_SNAPSHOT,
      projection: initialProjection(),
      filters: { ...DEFAULT_RUN_TRACE_FILTERS },
      expandedNodeIds,
      selectedNodeId: null,
      selectedCheckpointId: null,
      checkpoints: [],
      replay: null,
      agentArtifact: null,
      resumedRunId: null,
      network: {
        ...idleNetworkState(),
        snapshot: { status: "loading", error: null },
      },
    });

    const onSnapshot = (snapshot: PersistedRunSnapshot): void => {
      const current = get();
      if (current.generation !== generation || current.context?.runId !== context.runId) return;
      const next = { ...current, snapshot };
      set({ snapshot, ...reproject(next) });
    };
    try {
      const subscription = await subscribeRunTraceSnapshot(context.runId, onSnapshot);
      if (get().generation !== generation || get().context?.runId !== context.runId) {
        await subscription.unsubscribe();
        return;
      }
      set((state) => ({
        subscription,
        network: { ...state.network, snapshot: { status: "success", error: null } },
      }));
      if (get().snapshot.run === null) onSnapshot(subscription.initialSnapshot);
    } catch (error) {
      if (get().generation !== generation || get().context?.runId !== context.runId) return;
      set((state) => ({
        network: {
          ...state.network,
          snapshot: { status: "error", error: errorMessage(error) },
        },
      }));
    }

    void get().refreshCheckpoints();
    void get().refreshReplay();
    if (context.agentId) void get().refreshAgentArtifact();
  },

  toggleFilter: (kind) => {
    const current = get();
    const filters = { ...current.filters, [kind]: !current.filters[kind] };
    set({ filters, ...reproject({ ...current, filters }) });
  },

  toggleExpanded: (nodeId) => {
    const current = get();
    const expandedNodeIds = new Set(current.expandedNodeIds);
    if (expandedNodeIds.has(nodeId)) expandedNodeIds.delete(nodeId);
    else expandedNodeIds.add(nodeId);
    set({ expandedNodeIds, ...reproject({ ...current, expandedNodeIds }) });
  },

  selectNode: (nodeId) => {
    const current = get();
    if (!current.projection.nodesById.has(nodeId)) return;
    set(reproject({ ...current, selectedNodeId: nodeId }));
  },

  selectPhase: (phase) => {
    const current = get();
    const nodeId = current.projection.phaseNodeIds.get(phase) ?? null;
    if (!nodeId) return null;
    const expandedNodeIds = new Set(current.expandedNodeIds);
    if (current.projection.rootId) expandedNodeIds.add(current.projection.rootId);
    expandedNodeIds.add(nodeId);
    set({
      expandedNodeIds,
      ...reproject({ ...current, expandedNodeIds, selectedNodeId: nodeId }),
    });
    return nodeId;
  },

  selectCheckpoint: (checkpointId) => set({ selectedCheckpointId: checkpointId }),

  refreshCheckpoints: async () => {
    const context = get().context;
    if (!context) return;
    set((state) => ({
      network: { ...state.network, checkpoints: { status: "loading", error: null } },
    }));
    try {
      const response = await fetchRunCheckpoints(context.runId);
      if (get().context?.runId !== context.runId) return;
      set((state) => ({
        checkpoints: response.checkpoints,
        selectedCheckpointId: response.checkpoints.some(
          (checkpoint) => checkpoint.id === state.selectedCheckpointId,
        ) ? state.selectedCheckpointId : response.checkpoints.at(-1)?.id ?? null,
        network: { ...state.network, checkpoints: { status: "success", error: null } },
      }));
    } catch (error) {
      if (get().context?.runId !== context.runId) return;
      set((state) => ({
        network: {
          ...state.network,
          checkpoints: { status: "error", error: errorMessage(error) },
        },
      }));
    }
  },

  refreshReplay: async () => {
    const context = get().context;
    if (!context) return;
    set((state) => ({
      network: { ...state.network, replay: { status: "loading", error: null } },
    }));
    try {
      const replay = projectA2uiReplay(await fetchRunSurfaceReplay(context.runId));
      if (get().context?.runId !== context.runId) return;
      set((state) => ({
        replay,
        network: { ...state.network, replay: { status: "success", error: null } },
      }));
    } catch (error) {
      if (get().context?.runId !== context.runId) return;
      set((state) => ({
        network: { ...state.network, replay: { status: "error", error: errorMessage(error) } },
      }));
    }
  },

  refreshAgentArtifact: async () => {
    const context = get().context;
    if (!context?.agentId) return;
    set((state) => ({
      network: { ...state.network, agent: { status: "loading", error: null } },
    }));
    try {
      const agentArtifact = await resolveRuntimeAgentArtifact(context.agentId);
      if (get().context?.runId !== context.runId) return;
      set((state) => ({
        agentArtifact,
        network: { ...state.network, agent: { status: "success", error: null } },
      }));
    } catch (error) {
      if (get().context?.runId !== context.runId) return;
      set((state) => ({
        network: { ...state.network, agent: { status: "error", error: errorMessage(error) } },
      }));
    }
  },

  resumeLatestCheckpoint: async (input) => {
    const current = get();
    if (!current.context || !current.agentArtifact || current.checkpoints.length === 0) return null;
    set((state) => ({
      network: { ...state.network, resume: { status: "loading", error: null } },
    }));
    try {
      const response = await resumeRunFromLatestCheckpoint(current.context.runId, {
        artifact: current.agentArtifact,
        ...(input ? { input } : {}),
        ...(current.context.sessionId ? { session_id: current.context.sessionId } : {}),
      });
      if (get().context?.runId !== current.context.runId) return null;
      set((state) => ({
        resumedRunId: response.run_id,
        network: { ...state.network, resume: { status: "success", error: null } },
      }));
      return response;
    } catch (error) {
      if (get().context?.runId !== current.context.runId) return null;
      set((state) => ({
        network: { ...state.network, resume: { status: "error", error: errorMessage(error) } },
      }));
      return null;
    }
  },

  dispose: async () => {
    const subscription = get().subscription;
    set((state) => ({ generation: state.generation + 1, subscription: null }));
    await subscription?.unsubscribe();
  },
}));
