import { useToolApprovalStore } from "@/stores/tool-approval-store";

export function useToolApprovalActions() {
  const submitApproval = useToolApprovalStore((s) => s.submitApproval);
  const submitArtifactResponse = useToolApprovalStore((s) => s.submitArtifactResponse);
  return { submitApproval, submitArtifactResponse };
}
