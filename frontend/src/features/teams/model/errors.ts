import { TeamsApiError, type TeamsErrorInfo } from "@/platform/entities";

/** Plain-data form of any thrown value, so components render the runtime's own code and detail. */
export function errorInfoOf(error: unknown): TeamsErrorInfo {
  if (error instanceof TeamsApiError) return error.info;
  return {
    kind: "runtime",
    status: 0,
    code: "unexpected",
    detail: null,
    message: error instanceof Error ? error.message : "Unexpected error.",
    capability: null,
    uncertain: false,
  };
}
