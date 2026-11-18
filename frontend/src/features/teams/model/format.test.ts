import { describe, expect, test } from "vitest";

import { elideMiddle, lastSegment, shortId } from "./format";

describe("teams display helpers", () => {
  test("takes the last segment of a path or URN id", () => {
    expect(lastSegment("urn:uar:binding:binding-qwen-2")).toBe("binding-qwen-2");
    expect(lastSegment("a/b/binding-qwen-2")).toBe("binding-qwen-2");
    expect(lastSegment("plain")).toBe("plain");
    expect(lastSegment("trailing/")).toBe("trailing");
  });

  test("shortens a team id to its last characters", () => {
    expect(shortId("team-0123456789abcdef")).toBe("abcdef");
    expect(shortId("abc")).toBe("abc");
  });

  test("elides in the middle and keeps both ends", () => {
    expect(elideMiddle("short", 10)).toBe("short");
    const out = elideMiddle("urn:uar:deployment-binding:workspace:binding-qwen-2", 20);
    expect(out).toHaveLength(20);
    expect(out.startsWith("urn:uar:")).toBe(true);
    expect(out.endsWith("qwen-2")).toBe(true);
    expect(out).toContain("…");
  });
});
