import { expect, test } from "vitest";
import { A2uiPublicationProjection } from "./a2ui-publication-projection";
import type { A2uiDisplayChunk } from "./chunk";

function artifact(runId: string, generation: string, operation: object): A2uiDisplayChunk {
  return { kind: "a2ui-display", id: runId, runId, at: "", seq: 1, component: "a2ui", profile: "uar.a2ui/1", validation: "valid",
    payload: { content: JSON.stringify({ version: "1.0", ...operation }), metadata: { source_run_id: runId,
      publication_status: "published", surface_generations: { parcel: generation } } } };
}

test("projects a follow-up publication with its prior tree and its own authority only", () => {
  const projection = new A2uiPublicationProjection();
  const create = artifact("first", "G1", { createSurface: { catalogId: "urn:uar:a2ui:catalog:1", surfaceId: "parcel",
    components: [{ id: "root", component: "Text", text: { "@path": "/parcel" } }], dataModel: { parcel: "425" } } });
  const update = artifact("second", "G2", { updateDataModel: { surfaceId: "parcel", path: "/parcel", value: "900" } });
  expect(projection.project(update)).toBe(update);
  expect(projection.project(create)).toBe(create);
  const result = projection.project(update);
  expect(result).toMatchObject({ runId: "second", payload: { metadata: { source_run_id: "second", surface_generations: { parcel: "G2" } } } });
  const { content } = result.payload as { content: string };
  expect(JSON.parse(content)).toEqual([JSON.parse((create.payload as { content: string }).content), JSON.parse((update.payload as { content: string }).content)]);
});
