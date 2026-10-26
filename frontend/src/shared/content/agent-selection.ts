import type { AgentSelectionChunk, Chunk, ChunkBase } from "./chunk";
import type { ContentBlock } from "./content-block";

export const AGENT_SELECTION_MIME = "application/vnd.uar.agent-selection+json";

function record(value: unknown): Record<string, unknown> {
  return value && typeof value === "object" && !Array.isArray(value)
    ? value as Record<string, unknown> : {};
}

function identity(value: unknown): string | undefined {
  return typeof value === "string" && value.trim().length > 0 ? value : undefined;
}

/** One parser for host events and persisted notices; unknown identity stays raw. */
export function agentSelectionChunk(
  value: unknown,
  base: Omit<ChunkBase, "kind">,
  announce = false,
): AgentSelectionChunk | undefined {
  const data = record(value);
  const agent = record(data.agent);
  const agentId = identity(agent.id ?? data.agentId);
  const sourceAgentId = identity(data.sourceAgentId);
  const selectedThreadId = identity(data.selectedThreadId);
  const selectionMethod = identity(data.selectionMethod);
  const rosterRevision = identity(data.rosterRevision);
  if (!agentId || !sourceAgentId || !selectedThreadId || !selectionMethod || !rosterRevision) return undefined;
  const id = `agent-selection:${selectedThreadId}`;
  if (data.id !== undefined && data.id !== id) return undefined;
  return {
    ...base, id, kind: "agent-selection", sourceAgentId, agentId,
    agentName: identity(agent.title ?? data.agentName) ?? agentId,
    selectedThreadId, selectionMethod, rosterRevision,
    sourceEventId: identity(data.sourceEventId) ?? base.id,
    announce,
  };
}

export function selectionArtifact(
  id: string, content: string, base: Omit<ChunkBase, "kind">,
): AgentSelectionChunk | undefined {
  try {
    const selection = agentSelectionChunk(JSON.parse(content), base);
    return selection?.id === id ? selection : undefined;
  } catch {
    return undefined;
  }
}

/** Normalize even the preferred stored-chunks path; persisted live flags are ignored. */
export function restoreSelectionChunks(chunks: readonly Chunk[], content?: readonly ContentBlock[]): Chunk[] {
  const restored = chunks.map<Chunk>((chunk) => {
    if (chunk?.kind !== "agent-selection") return chunk;
    const selection = agentSelectionChunk(chunk, chunk, false);
    const portable = selection && content?.some((block) => {
      if (block.type !== "artifact" || block.kind !== AGENT_SELECTION_MIME) return false;
      const restored = selectionArtifact(block.id, block.content, selection);
      return restored?.id === selection.id
        && restored.sourceAgentId === selection.sourceAgentId
        && restored.agentId === selection.agentId
        && restored.agentName === selection.agentName
        && restored.selectedThreadId === selection.selectedThreadId
        && restored.selectionMethod === selection.selectionMethod
        && restored.rosterRevision === selection.rosterRevision;
    });
    return (selection && (content === undefined || portable) ? selection : undefined) ?? {
      id: chunk.id, at: chunk.at, seq: chunk.seq, runId: chunk.runId,
      kind: "raw", type: "invalid_agent_selection", payload: { reason: "Invalid stored selection metadata" },
    };
  });
  // A corrupt preferred chunks cache must not hide a valid canonical artifact.
  for (const block of content ?? []) {
    if (block.type !== "artifact" || block.kind !== AGENT_SELECTION_MIME) continue;
    const selection = selectionArtifact(block.id, block.content, {
      id: block.id, at: restored[0]?.at ?? "", seq: restored.length,
    });
    if (selection && !restored.some((chunk) => chunk.kind === "agent-selection" && chunk.id === selection.id)) {
      restored.push(selection);
    }
  }
  return restored;
}

export function portableSelection(chunk: AgentSelectionChunk): string {
  const stored = { ...chunk };
  delete stored.announce;
  return JSON.stringify(stored);
}
