import { describe, expect, test } from "bun:test";
import {
  buildCreateSkillRequest,
  buildUpdateSkillRequest,
  parseCommaSeparated,
} from "./skills-page.utils";

describe("skills-page utils", () => {
  test("parseCommaSeparated trims and drops empty values", () => {
    expect(parseCommaSeparated(" alpha, beta ,, gamma  ")).toEqual(["alpha", "beta", "gamma"]);
    expect(parseCommaSeparated("")).toEqual([]);
  });

  test("buildCreateSkillRequest maps markdown-capable fields", () => {
    const payload = buildCreateSkillRequest({
      title: "Skill One",
      version: "1.2.3",
      description: "## Markdown Description",
      promptOverlay: "# Prompt Overlay",
      keywords: "tooling, markdown",
      preferredTools: "search, memory",
      enabled: true,
    });

    expect(payload.name).toBe("Skill One");
    expect(payload.version).toBe("1.2.3");
    expect(payload.description).toBe("## Markdown Description");
    expect(payload.prompt_overlay).toBe("# Prompt Overlay");
    expect(payload.triggers.keywords).toEqual(["tooling", "markdown"]);
    expect(payload.preferred_tools).toEqual(["search", "memory"]);
    expect(payload.enabled).toBe(true);
  });

  test("buildUpdateSkillRequest preserves id and partial updates", () => {
    const payload = buildUpdateSkillRequest({
      title: "Updated Title",
      version: "2.0.0",
      description: "Updated",
      promptOverlay: "### New Prompt",
      keywords: "updated,skills",
      preferredTools: "search",
      enabled: false,
    });

    expect(payload.title).toBe("Updated Title");
    expect(payload.version).toBe("2.0.0");
    expect(payload.description).toBe("Updated");
    expect(payload.prompt_overlay).toBe("### New Prompt");
    expect(payload.triggers.keywords).toEqual(["updated", "skills"]);
    expect(payload.preferred_tools).toEqual(["search"]);
    expect(payload.enabled).toBe(false);
  });
});
