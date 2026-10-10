import { create } from "zustand";

import { postArtifactResponse, postToolApproval } from "@/services/run-tools-api";

interface ToolApprovalActions {
  submitApproval: (runId: string, approvalId: string, approved: boolean) => Promise<void>;
  submitArtifactResponse: (runId: string, body: Record<string, unknown>) => Promise<Response>;
}

export const useToolApprovalStore = create<ToolApprovalActions>(() => ({
  submitApproval: (runId, approvalId, approved) => postToolApproval(runId, approvalId, approved),
  submitArtifactResponse: (runId, body) => postArtifactResponse(runId, body),
}));
