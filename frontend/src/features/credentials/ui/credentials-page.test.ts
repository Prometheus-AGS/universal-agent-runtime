import { describe, expect, it } from "vitest";
import { formatUpdated, maskedKey } from "./credentials-page";

describe("maskedKey", () => {
  it("prefixes hint with four bullets", () => {
    expect(maskedKey("1234")).toBe("••••1234");
  });

  it("handles empty hint", () => {
    expect(maskedKey("")).toBe("••••");
  });

  it("trims whitespace from hint", () => {
    expect(maskedKey("  ab  ")).toBe("••••ab");
  });
});

describe("formatUpdated", () => {
  it("returns a non-empty string for a valid ISO timestamp", () => {
    const result = formatUpdated("2026-05-29T12:00:00Z");
    expect(result.length).toBeGreaterThan(0);
    expect(result).not.toBe("—");
  });

  it("returns em-dash for an invalid timestamp", () => {
    expect(formatUpdated("not-a-date")).toBe("—");
  });

  it("returns em-dash for empty string", () => {
    expect(formatUpdated("")).toBe("—");
  });
});
