import { render, screen } from "@testing-library/react";
import { describe, expect, test } from "vitest";

import { A2UI_CATALOG_ID, A2UI_PROFILE, A2UI_VERSION } from "@/features/a2ui/a2ui-protocol";
import { MAX_A2UI_SOURCE_BYTES } from "@/features/a2ui/a2ui-rendering-limits";

import { A2uiDisplayBlock } from "./a2ui-artifact-block";

function policySurface(component = "Text") {
  return JSON.stringify([
    {
      version: A2UI_VERSION,
      createSurface: { surfaceId: "policy", catalogId: A2UI_CATALOG_ID },
    },
    {
      version: A2UI_VERSION,
      updateComponents: {
        surfaceId: "policy",
        components: [
          { id: "heading", component: "Text", text: { path: "/title" }, variant: "h2" },
          { id: "summary", component, text: { path: "/summary" }, variant: "body" },
          { id: "root", component: "Column", children: ["heading", "summary"] },
        ],
      },
    },
    {
      version: A2UI_VERSION,
      updateDataModel: {
        surfaceId: "policy",
        path: "/",
        value: {
          title: "Effective run policy",
          summary: "Tools · all · 12 available",
        },
      },
    },
  ]);
}

describe("A2uiDisplayBlock", () => {
  test("renders current-production messages through the canonical UAR surface", () => {
    const source = policySurface();
    render(
      <A2uiDisplayBlock
        artifactType="effective_run_policy"
        title="Effective run policy"
        content={source}
        language="a2ui"
        profile={A2UI_PROFILE}
      />,
    );

    expect(screen.getByRole("heading", { name: "Effective run policy" })).toBeInTheDocument();
    expect(screen.getByText("Tools · all · 12 available")).toBeInTheDocument();
    expect(screen.getByText(`A2UI ${A2UI_VERSION} · rendered · effective_run_policy`)).toBeInTheDocument();
    expect(screen.queryByText(source)).not.toBeInTheDocument();
  });

  test("fails closed and keeps malformed source behind a disclosure", () => {
    render(
      <A2uiDisplayBlock
        artifactType="effective_run_policy"
        title="Effective run policy"
        content='{"effective":true}'
        profile={A2UI_PROFILE}
      />,
    );

    expect(screen.getByRole("alert")).toHaveTextContent("could not be rendered");
    expect(screen.getByText("View A2UI source")).toBeInTheDocument();
    expect(screen.getByLabelText("Effective run policy A2UI source")).toHaveTextContent('{"effective":true}');
  });

  test("rejects components outside the certified UAR catalog", () => {
    render(
      <A2uiDisplayBlock
        artifactType="effective_run_policy"
        title="Effective run policy"
        content={policySurface("Image")}
        profile={A2UI_PROFILE}
      />,
    );

    expect(screen.getByRole("alert")).toHaveTextContent("Unapproved A2UI component: Image");
  });

  test("requires the declared UAR profile instead of fabricating one", () => {
    render(
      <A2uiDisplayBlock
        artifactType="effective_run_policy"
        title="Effective run policy"
        content={policySurface()}
      />,
    );

    expect(screen.getByRole("alert")).toHaveTextContent("missing its required UAR A2UI profile");
  });

  test("rejects artifacts that exceed the synchronous rendering budget", () => {
    render(
      <A2uiDisplayBlock
        artifactType="effective_run_policy"
        title="Effective run policy"
        content={"x".repeat(256 * 1024 + 1)}
        profile={A2UI_PROFILE}
      />,
    );

    expect(screen.getByRole("alert")).toHaveTextContent("exceeds the 256 KiB rendering limit");
  });

  test("bounds oversized source disclosure before profile validation", () => {
    render(
      <A2uiDisplayBlock
        artifactType="effective_run_policy"
        title="Effective run policy"
        content={`untrusted-prefix-${"x".repeat(MAX_A2UI_SOURCE_BYTES + 1)}`}
        validation="invalid"
        validationError="Rejected upstream"
      />,
    );

    const disclosure = screen.getByLabelText("Effective run policy A2UI source");
    expect(disclosure).toHaveTextContent("untrusted-prefix-");
    expect(disclosure).toHaveTextContent("A2UI source truncated at 256 KiB");
    expect(new TextEncoder().encode(disclosure.textContent ?? "").byteLength)
      .toBeLessThanOrEqual(MAX_A2UI_SOURCE_BYTES);
  });

  test("applies the message ceiling to JSONL as well as JSON arrays", () => {
    const jsonl = Array.from({ length: 129 }, (_, index) => JSON.stringify({
      version: A2UI_VERSION,
      deleteSurface: { surfaceId: `surface-${index}` },
    })).join("\n");
    render(
      <A2uiDisplayBlock
        artifactType="effective_run_policy"
        title="Effective run policy"
        content={jsonl}
        profile={A2UI_PROFILE}
      />,
    );

    expect(screen.getByRole("alert")).toHaveTextContent("exceeds the 128-message rendering limit");
  });
});
