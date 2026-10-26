import { describe, expect, it } from "vitest";
import { agentSelectionChunk, AGENT_SELECTION_MIME, portableSelection, restoreSelectionChunks, selectionArtifact } from "./agent-selection";
import { toChunks } from "./to-chunks";
import type { ContentBlock } from "./content-block";

const base = { id: "wire-event:4", at: "2026-10-25T00:00:00Z", seq: 4, runId: "root-run" };
const wire = { sourceAgentId: "orchestrator-agent", agent: { id: "registered-agent", title: "Registered agent" }, selectedThreadId: "persisted-child", selectionMethod: "intent", rosterRevision: "roster-snapshot" };

describe("host agent selection across live and persisted boundaries", () => {
  it("preserves semantic identity but never restores a live announcement", () => {
    const live = agentSelectionChunk(wire, base, true)!;
    const portable: ContentBlock = { type: "artifact", id: live.id, kind: AGENT_SELECTION_MIME, content: portableSelection(live) };
    expect(live.id).toBe("agent-selection:persisted-child");
    expect(live.announce).toBe(true);
    expect(JSON.parse(portable.content)).not.toHaveProperty("announce");
    const projected = toChunks([portable], { messageId: "saved-message", at: base.at, finalized: true });
    expect(projected[0]).toMatchObject({ kind: "agent-selection", id: live.id, announce: false, sourceEventId: base.id });
    // PGlite prefers nonempty stored chunks; that boundary must sanitize too.
    expect(restoreSelectionChunks([live], [portable])[0]).toMatchObject({ id: live.id, announce: false });
  });

  it("rejects malformed or mismatched reserved data before it can hide a selection", () => {
    expect(agentSelectionChunk({ ...wire, selectedThreadId: "" }, base)).toBeUndefined();
    const live = agentSelectionChunk(wire, base, true)!;
    expect(selectionArtifact("wrong-id", portableSelection(live), base)).toBeUndefined();
    const mismatched: ContentBlock = { type: "artifact", id: live.id, kind: AGENT_SELECTION_MIME,
      content: portableSelection({ ...live, agentId: "another-agent" }) };
    expect(restoreSelectionChunks([live], [mismatched])[0]).toMatchObject({ kind: "raw", type: "invalid_agent_selection" });
    expect(restoreSelectionChunks([{ ...live, agentId: "corrupt-cache" }], [{ ...mismatched, content: portableSelection(live) }]))
      .toEqual([expect.objectContaining({ kind: "raw" }), expect.objectContaining({ kind: "agent-selection", agentId: live.agentId, announce: false })]);
  });
});
