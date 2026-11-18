import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, test } from "vitest";
import { byteDigest, canonicalJson, contentDigestOf, documentReference, withContentDigest } from "./digest";

// Tests run from `frontend/`; the shared fixtures live beside the Rust tests.
const FIXTURES = resolve(process.cwd(), "../tests/fixtures/collaboration");

function fixture(name: string): Record<string, unknown> & { contentDigest: string; id: string; version: string } {
  return JSON.parse(readFileSync(resolve(FIXTURES, name), "utf8"));
}

describe("collaboration digests", () => {
  test("SHA-256 of 'abc' matches the published known answer", async () => {
    expect(await byteDigest("abc")).toBe("sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
  });

  test.each(["team-definition.json", "deployment-binding.json", "package-manifest.json"])(
    "recomputes the contentDigest the runtime verified for %s",
    async (name) => {
      const document = fixture(name);
      expect(await contentDigestOf(document)).toBe(document.contentDigest);
      expect((await withContentDigest({ ...document, contentDigest: "sha256:stale" })).contentDigest).toBe(
        document.contentDigest,
      );
    },
  );

  test("sorts keys at every depth and keeps array order", () => {
    expect(canonicalJson({ b: [{ z: 1, a: 2 }, 3], a: { d: true, c: null } })).toBe(
      '{"a":{"c":null,"d":true},"b":[{"a":2,"z":1},3]}',
    );
  });

  test("builds a reference from a digested document", () => {
    const document = fixture("team-definition.json");
    expect(documentReference(document)).toEqual({
      id: "urn:uar:c03:team",
      version: "1.0.0",
      digest: document.contentDigest,
    });
  });
});
