import { graphStore } from "@prometheus-ags/prometheus-entity-management";
import { useChatStreamStore } from "@/stores/chat-stream-store";
import { acquireChatStream } from "./stream-ownership";
import { A2uiActionError, inspectActionSource, postA2uiAction } from "./api";
import { A2UI_ACTION_ENTITY, a2uiActionId, type A2uiActionInput, type A2uiActionReceipt } from "./contracts";
import { persistA2uiActionReceipt } from "./durability";

const liveTasks = new Set<string>();
const consumed = new Set<string>();
function read(id: string) { return graphStore.getState().readEntity<A2uiActionReceipt>(A2UI_ACTION_ENTITY, id); }
function write(receipt: A2uiActionReceipt) { graphStore.getState().replaceEntity(A2UI_ACTION_ENTITY, receipt.id, receipt); }

export function reconcileA2uiAction(runId: string, surfaceId: string): void {
  const id = a2uiActionId(runId, surfaceId);
  const receipt = read(id);
  if (receipt && !liveTasks.has(id) && (receipt.status === "pending" || receipt.status === "responding")) {
    write({ ...receipt, status: receipt.continuation_run_id ? "interrupted" : "uncertain" });
    void persistA2uiActionReceipt().catch(() => {});
  }
}

export async function submitA2uiAction(runId: string, input: A2uiActionInput): Promise<void> {
  if (!runId || !input.surfaceId) throw new Error("Actions unavailable: this surface is not linked to a run.");
  reconcileA2uiAction(runId, input.surfaceId);
  const id = a2uiActionId(runId, input.surfaceId);
  if (liveTasks.has(id) || consumed.has(id) || (read(id) && read(id)?.status !== "rejected")) {
    throw new Error("This surface action has already been submitted.");
  }
  const lease = acquireChatStream();
  liveTasks.add(id);
  let receipt: A2uiActionReceipt = { id, source_run_id: runId, surface_id: input.surfaceId,
    continuation_run_id: null, status: "pending" };
  let posted = false;
  try {
    write(receipt);
    await persistA2uiActionReceipt();
    const threadId = await inspectActionSource(runId);
    posted = true;
    consumed.add(id);
    const continuation = await postA2uiAction(runId, input);
    receipt = { ...receipt, continuation_run_id: continuation, status: "responding" };
    write(receipt);
    await persistA2uiActionReceipt();
    await useChatStreamStore.getState().attachContinuation(threadId, continuation, lease);
    receipt = { ...receipt, status: "completed" };
    write(receipt);
    await persistA2uiActionReceipt();
  } catch (error) {
    receipt = { ...receipt, status: receipt.continuation_run_id ? "interrupted"
      : !posted || (error instanceof A2uiActionError && !error.uncertain) ? "rejected" : "uncertain" };
    if (receipt.status === "rejected") consumed.delete(id);
    try { write(receipt); await persistA2uiActionReceipt(); } catch { /* The returned error retains uncertainty; never retry an admitted POST. */ }
    throw new Error(receipt.status === "interrupted" ? "The response was interrupted."
      : receipt.status === "uncertain" ? "Action outcome is unconfirmed. Do not submit again."
        : "Action was rejected. Try again.", { cause: error });
  } finally { liveTasks.delete(id); lease.release(); }
}
