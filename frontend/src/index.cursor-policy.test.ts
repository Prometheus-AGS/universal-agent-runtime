import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

describe("cursor policy", () => {
  test("interactive elements enforce pointer cursor", () => {
    const css = readFileSync(resolve(import.meta.dir, "index.css"), "utf8");

    expect(css).toContain("nav a");
    expect(css).toContain("nav button");
    expect(css).toContain("cursor: pointer !important");
    expect(css).toContain("cursor: not-allowed !important");
  });
});
