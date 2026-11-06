import type { A2uiActionInput, A2uiSurfaceAuthority } from "./contracts";

export class A2uiActionError extends Error {
  constructor(public readonly uncertain: boolean) {
    super(uncertain ? "Action outcome is unconfirmed. Do not submit again." : "Action was rejected. Try again.");
  }
}

export async function inspectActionSource(runId: string): Promise<string> {
  const response = await fetch(`/api/uar/runs/${encodeURIComponent(runId)}`, { cache: "no-store" });
  if (!response.ok) throw new A2uiActionError(false);
  const run: unknown = await response.json();
  if (!run || typeof run !== "object" || !("run_id" in run) || run.run_id !== runId
    || !("conversation_id" in run) || typeof run.conversation_id !== "string" || !run.conversation_id
    || !("status" in run) || run.status !== "done") throw new A2uiActionError(false);
  return run.conversation_id;
}

export async function postA2uiAction(runId: string, input: A2uiActionInput): Promise<string> {
  let response: Response;
  try {
    response = await fetch(`/api/uar/runs/${encodeURIComponent(runId)}/a2ui/actions`, {
      method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify(input),
    });
  } catch { throw new A2uiActionError(true); }
  // These statuses trace to authentication/source lookup or declared-action
  // validation before execute_request in the mounted handler. Others may follow effects.
  if (!response.ok) {
    let declaredRefusal = false;
    if (response.status === 422) {
      const body: unknown = await response.json().catch(() => null);
      declaredRefusal = Boolean(body && typeof body === "object" && "error" in body
        && body.error === "action is not declared by the active A2UI surface");
    }
    throw new A2uiActionError(![401, 403, 404].includes(response.status) && !declaredRefusal);
  }
  try {
    const ack: unknown = await response.json();
    if (response.status !== 202 || !ack || typeof ack !== "object"
      || !("run_id" in ack) || ack.run_id !== runId || !("status" in ack) || ack.status !== "continued"
      || !("continuation_run_id" in ack) || typeof ack.continuation_run_id !== "string"
      || !ack.continuation_run_id || ack.continuation_run_id === runId) throw new Error("invalid acknowledgment");
    if (input.interaction_id && (!("interaction_id" in ack) || ack.interaction_id !== input.interaction_id
      || !("retry_protected" in ack) || ack.retry_protected !== true)) throw new Error("unprotected acknowledgment");
    return ack.continuation_run_id;
  } catch { throw new A2uiActionError(true); }
}

export async function fetchSurfaceAuthority(runId: string): Promise<A2uiSurfaceAuthority> {
  const response = await fetch(`/api/uar/runs/${encodeURIComponent(runId)}/a2ui/surface-authority`, { cache: "no-store" });
  if (!response.ok) throw new Error("Surface history is unavailable. Reload the conversation to recover.");
  return parseSurfaceAuthority(runId, await response.json());
}
export function parseSurfaceAuthority(runId: string, value: unknown): A2uiSurfaceAuthority {
  if (!value || typeof value !== "object") throw new Error("Surface history is unavailable.");
  const authority = value as A2uiSurfaceAuthority;
  if (!["complete", "truncated", "expired", "unavailable"].includes(authority.status) || !Array.isArray(authority.surfaces) || typeof authority.authoritative !== "boolean"
    || typeof authority.recovery_hint !== "string" || authority.surfaces.some((surface) => !surface
      || typeof surface.surface_id !== "string" || typeof surface.surface_generation !== "string"
      || !surface.surface_generation || typeof surface.source_run_id !== "string")) throw new Error("Surface history is unavailable.");
  return { ...authority, id: runId };
}
export async function fetchActionReceipt(runId: string, interactionId: string): Promise<{ continuation_run_id: string; outcome: string }> {
  const response = await fetch(`/api/uar/runs/${encodeURIComponent(runId)}/a2ui/actions/${encodeURIComponent(interactionId)}`, { cache: "no-store" });
  if (!response.ok) throw new A2uiActionError(true);
  const receipt = await response.json();
  if (receipt.run_id !== runId || receipt.interaction_id !== interactionId || receipt.retry_protected !== true
    || typeof receipt.continuation_run_id !== "string") throw new A2uiActionError(true);
  return receipt;
}
