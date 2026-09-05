import { z } from "zod";
import { presentationAssignmentId, presentationSelectionSchema, type PresentationAssignment, type PresentationAssignmentTarget, type PresentationSelection } from "./contracts";

export class AssignmentApiError extends Error {
  constructor(message: string, readonly status: number, readonly uncertain = false) { super(message); }
}

function headers(global: boolean): Record<string, string> {
  const credential = import.meta.env.VITE_UAR_API_KEY ?? "";
  const admin = import.meta.env.VITE_UAR_ADMIN_KEY ?? "";
  return {
    "Content-Type": "application/json",
    ...(credential ? credential.startsWith("ey") ? { Authorization: `Bearer ${credential}` } : { "x-api-key": credential } : {}),
    ...(global && admin ? { "X-UAR-Admin-Key": admin } : {}),
  };
}

async function request(path: string, global: boolean, method = "GET", body?: unknown): Promise<unknown> {
  const writing = method !== "GET";
  let response: Response;
  try { response = await fetch(path, { method, headers: headers(global), cache: "no-store", body: body === undefined ? undefined : JSON.stringify(body) }); }
  catch { throw new AssignmentApiError(writing ? "Save result unknown. Check saved assignment before saving again." : "Could not load assignment. Check the connection and reload.", 0, writing); }
  if (!response.ok) {
    const message = response.status === 401 ? "An authenticated runtime credential is required. Reload after signing in."
      : response.status === 403 ? "This assignment requires authorized access. Global policy also requires the configured admin key."
        : response.status === 404 ? "This policy or local agent is unavailable. No empty policy was substituted."
          : response.status === 409 ? "Policy changed elsewhere. Your draft is retained; reload the saved assignment to resolve it."
            : writing ? "The server did not confirm the save. Your draft is retained." : "Assignment is unavailable. Reload to retry.";
    throw new AssignmentApiError(message, response.status, writing && response.status >= 500);
  }
  try { return await response.json(); }
  catch { throw new AssignmentApiError(writing ? "Save response unreadable. Check saved assignment before saving again." : "Assignment response unreadable. No policy was loaded.", response.status, writing); }
}

const object = z.record(z.unknown());

function agentPolicy(agent: unknown): Record<string, unknown> {
  const artifact = object.parse(agent);
  const extensions = artifact.extensions == null ? {} : object.parse(artifact.extensions);
  return extensions["uar.run_policy"] == null ? {} : object.parse(extensions["uar.run_policy"]);
}

function record(owner: string, target: PresentationAssignmentTarget, policy: Record<string, unknown>): PresentationAssignment {
  return { id: presentationAssignmentId(owner, target), owner_id: owner, target, policy,
    selection: presentationSelectionSchema.parse(policy.presentations ?? {}) };
}

export async function fetchAssignment(owner: string, target: PresentationAssignmentTarget): Promise<PresentationAssignment> {
  if (target.scope === "global") {
    const body = object.parse(await request("/api/uar/settings/presentation-policy", true));
    return record(owner, target, object.parse(body.policy));
  }
  const agent = object.parse(await request(`/api/agents/${encodeURIComponent(target.agentId)}`, false));
  if (agent.id !== target.agentId) throw new AssignmentApiError("The returned agent does not match this assignment. Reload to retry.", 502);
  return record(owner, target, agentPolicy(agent));
}

export async function saveAssignment(baseline: PresentationAssignment, selection: PresentationSelection): Promise<PresentationAssignment> {
  const { owner_id: owner, target } = baseline;
  if (target.scope === "agent") {
    const response = await request(`/api/agents/${encodeURIComponent(target.agentId)}`, false, "PATCH", { extensions: { "uar.run_policy": { presentations: selection } } });
    try { return record(owner, target, agentPolicy(response)); }
    catch { throw new AssignmentApiError("Agent save response unreadable. Check saved assignment before saving again.", 200, true); }
  }
  const response = await request("/api/uar/settings/presentation-policy", true, "PUT", { expected_policy: baseline.policy, presentations: selection });
  try { return record(owner, target, object.parse(object.parse(response).policy)); }
  catch { throw new AssignmentApiError("Global save response unreadable. Check saved assignment before saving again.", 200, true); }
}
