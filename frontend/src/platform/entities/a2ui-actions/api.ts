import type { A2uiActionInput } from "./contracts";

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
    return ack.continuation_run_id;
  } catch { throw new A2uiActionError(true); }
}
