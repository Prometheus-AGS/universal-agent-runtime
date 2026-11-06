import { graphStore } from "@prometheus-ags/prometheus-entity-management";
import { useChatStreamStore } from "@/stores/chat-stream-store";
import { acquireChatStream } from "./stream-ownership";
import { A2uiActionError, inspectActionSource, postA2uiAction, fetchSurfaceAuthority, fetchActionReceipt, parseSurfaceAuthority } from "./api";
import { A2UI_ACTION_ENTITY, A2UI_AUTHORITY_ENTITY, a2uiActionId, type A2uiActionInput, type A2uiActionReceipt } from "./contracts";
import { persistA2uiActionReceipt } from "./durability";

const liveTasks = new Set<string>();
const recovering = new Set<string>();
function write(receipt: A2uiActionReceipt) { graphStore.getState().replaceEntity(A2UI_ACTION_ENTITY, receipt.id, receipt); }
export function latestActionReceipt(runId: string, surfaceId: string): A2uiActionReceipt | undefined {
  return (Object.values(graphStore.getState().entities[A2UI_ACTION_ENTITY] ?? {}) as A2uiActionReceipt[])
    .filter((receipt) => receipt.source_run_id === runId && receipt.surface_id === surfaceId).at(-1);
}
export async function loadA2uiSurfaceAuthority(runId: string): Promise<void> {
  try {
    const authority = await fetchSurfaceAuthority(runId);
    graphStore.getState().replaceEntity(A2UI_AUTHORITY_ENTITY, runId, authority);
  } catch {
    graphStore.getState().replaceEntity(A2UI_AUTHORITY_ENTITY, runId, { id: runId, status: "unavailable", authoritative: false,
      surfaces: [], recovery_hint: "Surface history is unavailable. Reload the conversation to recover." });
  }
}
export function reconcileA2uiAction(runId: string, surfaceId: string): Promise<void> {
  const receipt = latestActionReceipt(runId, surfaceId);
  if (!receipt || liveTasks.has(a2uiActionId(runId, surfaceId)) || recovering.has(receipt.id)
    || ["completed", "rejected"].includes(receipt.status)) return Promise.resolve();
  write({ ...receipt, status: receipt.continuation_run_id ? "interrupted" : "uncertain" });
  if (!receipt.interaction_id) return Promise.resolve();
  recovering.add(receipt.id);
  return fetchActionReceipt(runId, receipt.interaction_id).then(async (remote) => {
    let status: A2uiActionReceipt["status"] = remote.outcome === "completed" ? "responding"
      : remote.outcome === "failed" || remote.outcome === "cancelled" ? "interrupted"
      : remote.outcome === "running" ? "responding" : "uncertain";
    write({ ...receipt, continuation_run_id: remote.continuation_run_id, status });
    await persistA2uiActionReceipt();
    if (remote.outcome === "running" || remote.outcome === "completed") {
      let lease: ReturnType<typeof acquireChatStream> | undefined;
      try {
        lease = acquireChatStream();
        liveTasks.add(a2uiActionId(runId, surfaceId));
        const threadId = await inspectActionSource(runId);
        await useChatStreamStore.getState().attachContinuation(threadId, remote.continuation_run_id, lease);
        status = "completed";
      } catch { status = "interrupted"; }
      finally { liveTasks.delete(a2uiActionId(runId, surfaceId)); lease?.release(); }
      write({ ...receipt, continuation_run_id: remote.continuation_run_id, status });
      await persistA2uiActionReceipt();
    }
  }).catch(() => { /* Unknown admission remains uncertain; reconciliation never posts. */ })
    .finally(() => recovering.delete(receipt.id));
}
export async function submitA2uiAction(runId: string, input: A2uiActionInput): Promise<void> {
  if (!runId || !input.surfaceId) throw new Error("Actions unavailable: this surface is not linked to a run.");
  const slot = a2uiActionId(runId, input.surfaceId);
  const previous = latestActionReceipt(runId, input.surfaceId);
  if (liveTasks.has(slot) || (previous && !["completed", "rejected"].includes(previous.status))) throw new Error("This surface action has already been submitted. Recover its outcome before continuing.");
  // Capture once before persistence or authority I/O; callers may keep editing.
  const protectedInput = { ...structuredClone(input), interaction_id: input.interaction_id ?? crypto.randomUUID() };
  const interactionId = protectedInput.interaction_id;
  const receiptId = a2uiActionId(runId, interactionId);
  if (graphStore.getState().readEntity(A2UI_ACTION_ENTITY, receiptId)) throw new Error("This logical action has already been submitted.");
  const lease = acquireChatStream();
  liveTasks.add(slot);
  let receipt: A2uiActionReceipt = { id: receiptId, interaction_id: interactionId, source_run_id: runId,
    surface_id: protectedInput.surfaceId, surface_generation: protectedInput.surface_generation,
    continuation_run_id: null, status: "pending", input: protectedInput };
  let posted = false;
  try {
    write(receipt);
    await persistA2uiActionReceipt();
    if (!protectedInput.surface_generation) throw new Error("This surface has no publication identity. Request a fresh surface before acting.");
    const authority = await fetchSurfaceAuthority(runId);
    graphStore.getState().replaceEntity(A2UI_AUTHORITY_ENTITY, runId, authority);
    const surfaces = authority.surfaces.filter((row) => row.surface_id === protectedInput.surfaceId);
    if (!authority.authoritative || authority.status !== "complete") throw new Error(authority.recovery_hint);
    if (surfaces.length !== 1 || surfaces[0].source_run_id !== runId
      || surfaces[0].surface_generation !== protectedInput.surface_generation) {
      throw new Error("This surface has changed. Request a fresh surface before acting.");
    }
    write(receipt);
    await persistA2uiActionReceipt();
    const threadId = await inspectActionSource(runId);
    posted = true;
    const continuation = await postA2uiAction(runId, protectedInput);
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
    try { write(receipt); await persistA2uiActionReceipt(); } catch { /* Never retry an uncertain POST. */ }
    throw new Error(receipt.status === "interrupted" ? "The response was interrupted."
      : receipt.status === "uncertain" ? "Action outcome is unconfirmed. Do not submit again."
        : "Action was rejected. Try again.", { cause: error });
  } finally { liveTasks.delete(slot); lease.release(); }
}

/** One atomic graph ingestion for a replay authority projection, never an action. */
export function ingestA2uiSurfaceAuthority(runId: string, threadId: string, value: unknown) {
  const authority = parseSurfaceAuthority(runId, value);
  const ids = new Set([runId, ...authority.surfaces.map((surface) => surface.source_run_id)]);
  if (!authority.authoritative) {
    for (const record of Object.values(graphStore.getState().entities[A2UI_AUTHORITY_ENTITY] ?? {}) as Array<Record<string, unknown>>) {
      if (record.thread_id === threadId) ids.add(String(record.id));
    }
  }
  graphStore.getState().upsertEntities(A2UI_AUTHORITY_ENTITY,
    [...ids].map((id) => ({ id, data: { ...authority, id, thread_id: threadId } })));
  return authority;
}
