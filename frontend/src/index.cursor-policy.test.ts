import { describe, expect, test } from "vitest";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));

describe("cursor policy", () => {
  test("interactive elements enforce pointer cursor", () => {
    const css = readFileSync(resolve(__dirname, "index.css"), "utf8");

    expect(css).toContain("nav a");
    expect(css).toContain("nav button");
    expect(css).toContain("cursor: pointer !important");
    expect(css).toContain("cursor: not-allowed !important");
  });
});
