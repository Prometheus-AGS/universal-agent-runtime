import { beforeEach, describe, expect, test, vi } from "vitest";

import * as api from "../api/a2ui-api";
import { useA2uiSchemasStore } from "./a2ui-schemas-store";

vi.mock("../api/a2ui-api", () => ({
  fetchA2uiSchemas: vi.fn(),
  triggerA2uiTest: vi.fn(),
}));

describe("A2UI schemas store", () => {
  beforeEach(() => {
    vi.resetAllMocks();
    useA2uiSchemasStore.setState({
      schemas: [],
      loading: false,
      error: null,
      triggering: false,
      triggerError: null,
    });
  });

  test("owns schema loading state", async () => {
    vi.mocked(api.fetchA2uiSchemas).mockResolvedValue([{
      schema_id: "a2ui/confirm",
      title: "Confirm",
      description: "Confirmation",
      artifact_type: "confirm",
      json_schema: {},
      builtin: true,
    }]);

    await useA2uiSchemasStore.getState().load();
    expect(useA2uiSchemasStore.getState()).toMatchObject({
      loading: false,
      error: null,
      schemas: [{ schema_id: "a2ui/confirm" }],
    });
  });

  test("owns trigger success and visible failure state", async () => {
    vi.mocked(api.triggerA2uiTest).mockResolvedValueOnce();
    const payload = { artifact_type: "confirm", title: "Confirm", content: "{}" };
    await expect(useA2uiSchemasStore.getState().trigger("run-1", payload)).resolves.toBe(true);
    expect(api.triggerA2uiTest).toHaveBeenCalledWith("run-1", payload);

    vi.mocked(api.triggerA2uiTest).mockRejectedValueOnce(new Error("run is no longer active"));
    await expect(useA2uiSchemasStore.getState().trigger("run-1", payload)).resolves.toBe(false);
    expect(useA2uiSchemasStore.getState()).toMatchObject({
      triggering: false,
      triggerError: "run is no longer active",
    });
  });
});
