import { describe, expect, test } from "vitest";
import { byteDigest } from "@/platform/entities";

import { analysePackageSelection, teamDefinitionKeysOf, type SelectedPackageFile } from "./package-analysis";

const encode = (text: string): Uint8Array => new TextEncoder().encode(text);
const file = (name: string, text: string): SelectedPackageFile => ({ name, bytes: encode(text) });

async function manifestFor(files: Array<{ path: string; text: string; kind?: string }>) {
  const listed = await Promise.all(
    files.map(async ({ path, text, kind }) => ({
      path,
      kind: kind ?? "AgentDefinition",
      definition: { id: `urn:test:${path}`, version: "1.0.0", digest: "sha256:00" },
      byteDigest: await byteDigest(encode(text)),
    })),
  );
  return JSON.stringify({ kind: "PackageManifest", id: "urn:test:pkg", version: "1.0.0", files: listed });
}

describe("analysePackageSelection", () => {
  test("accepts a manifest whose files all match and ignores order", async () => {
    const manifest = await manifestFor([{ path: "team.json", text: "{}", kind: "TeamDefinition" }, { path: "agent.json", text: '{"a":1}' }]);
    const result = await analysePackageSelection([file("agent.json", '{"a":1}'), file("manifest.json", manifest), file("team.json", "{}")]);
    expect(result.ready).toBe(true);
    expect(Object.keys(result.files).sort()).toEqual(["agent.json", "team.json"]);
    expect(teamDefinitionKeysOf(result.manifest!)).toEqual(["urn:test:team.json@1.0.0"]);
  });

  test("names a missing file and a digest mismatch and is not ready", async () => {
    const manifest = await manifestFor([{ path: "team.json", text: "{}" }, { path: "agent.json", text: '{"a":1}' }]);
    const result = await analysePackageSelection([file("manifest.json", manifest), file("agent.json", '{"a":2}')]);
    expect(result.ready).toBe(false);
    expect(result.reports.find((report) => report.path === "team.json")?.status).toBe("missing");
    expect(result.reports.find((report) => report.path === "agent.json")?.status).toBe("digest");
  });

  test("reports an unlisted file without blocking", async () => {
    const manifest = await manifestFor([{ path: "team.json", text: "{}" }]);
    const result = await analysePackageSelection([file("manifest.json", manifest), file("team.json", "{}"), file("notes.json", "{}")]);
    expect(result.ready).toBe(true);
    expect(result.reports.find((report) => report.path === "notes.json")?.status).toBe("extra");
    expect(result.files["notes.json"]).toBeUndefined();
  });

  test("rejects a selection with no manifest", async () => {
    const result = await analysePackageSelection([file("team.json", "{}")]);
    expect(result.ready).toBe(false);
    expect(result.problems[0]).toContain("manifest");
  });

  test("matches a manifest path with a folder by file name", async () => {
    const manifest = await manifestFor([{ path: "definitions/team.json", text: "{}" }]);
    const result = await analysePackageSelection([file("manifest.json", manifest), file("team.json", "{}")]);
    expect(result.ready).toBe(true);
    expect(Object.keys(result.files)).toEqual(["definitions/team.json"]);
  });
});
