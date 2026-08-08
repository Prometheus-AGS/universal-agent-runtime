import { useGraphStore } from "@/platform/entities";
import { beforeEach, describe, expect, test, vi } from "vitest";

import * as api from "../api/runtime-console-api";
import { useRuntimeConsoleStore } from "./runtime-console-store";
import type { RuntimeApprovalEntity } from "@/entities/types";

vi.mock("../api/runtime-console-api", () => ({
  fetchRuntimeProviderHealth: vi.fn(),
  fetchRuntimeA2uiSchemas: vi.fn(),
  fetchRuntimeModelRoute: vi.fn(),
  resolveRuntimeApproval: vi.fn(),
}));

const approval: RuntimeApprovalEntity = {
  id: "approval:call-1",
  run_id: "run-1",
  tool_call_id: "call-1",
  tool_name: "write_file",
  status: "pending",
  updated_at: "2026-07-11T00:00:00Z",
};

describe("Runtime Console store", () => {
  beforeEach(() => {
    vi.resetAllMocks();
    useGraphStore.setState({ entities: {} } as never);
  });

  test("resolves an approval once through the service and updates graph state", async () => {
    vi.mocked(api.resolveRuntimeApproval).mockResolvedValueOnce();
    useGraphStore.getState().upsertEntity("RuntimeApproval", approval.id, approval);

    await expect(useRuntimeConsoleStore.getState().resolveApproval(approval, true)).resolves.toBe(true);
    expect(api.resolveRuntimeApproval).toHaveBeenCalledWith("run-1", true);
    expect(useGraphStore.getState().entities.RuntimeApproval?.[approval.id]).toMatchObject({ status: "approved" });

    await expect(useRuntimeConsoleStore.getState().resolveApproval({ ...approval, status: "approved" }, false))
      .resolves.toBe(false);
    expect(api.resolveRuntimeApproval).toHaveBeenCalledTimes(1);
  });

  test("rolls optimistic state back when resolution fails", async () => {
    vi.mocked(api.resolveRuntimeApproval).mockRejectedValueOnce(new Error("no pending approval"));
    useGraphStore.getState().upsertEntity("RuntimeApproval", approval.id, approval);

    await expect(useRuntimeConsoleStore.getState().resolveApproval(approval, false)).resolves.toBe(false);
    expect(useGraphStore.getState().entities.RuntimeApproval?.[approval.id]).toMatchObject({ status: "pending" });
  });
});
