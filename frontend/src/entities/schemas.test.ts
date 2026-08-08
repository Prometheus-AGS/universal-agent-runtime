import { describe, expect, test } from "vitest";

import { getEntityJsonSchema, getSchema } from "@/platform/entities";
import { registerAllSchemas } from "@/entities/schemas";

describe("runtime SQL-derived schemas", () => {
  test("registers run fields without losing graph relations", () => {
    registerAllSchemas();

    const fields = getEntityJsonSchema({ entityType: "RuntimeRun" })?.schema.properties ?? {};
    expect(fields).toHaveProperty("phase_timings");
    expect(fields).toHaveProperty("thread_id");
    expect(getSchema("RuntimeRun")?.relations).toHaveProperty("steps");
  });

  test("registers durable event identity and both order fields", () => {
    registerAllSchemas();

    const fields = getEntityJsonSchema({ entityType: "RuntimeAgUiEvent" })?.schema.properties ?? {};
    expect(fields).toHaveProperty("event_id");
    expect(fields).toHaveProperty("seq");
    expect(fields).toHaveProperty("wire_sequence");
  });
});
