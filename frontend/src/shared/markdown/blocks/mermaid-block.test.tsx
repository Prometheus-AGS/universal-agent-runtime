// @vitest-environment jsdom
import { render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, test, vi } from "vitest";
import { MermaidBlock } from "./mermaid-block";

const mermaidCapture = vi.hoisted(() => ({
  initialize: vi.fn(),
  render: vi.fn(),
}));

vi.mock("mermaid", () => ({
  default: {
    initialize: mermaidCapture.initialize,
    render: mermaidCapture.render,
  },
}));

describe("MermaidBlock", () => {
  beforeEach(() => {
    mermaidCapture.initialize.mockReset();
    mermaidCapture.render.mockReset();
    mermaidCapture.render.mockResolvedValue({
      svg: '<svg viewBox="0 0 10 10"><script>alert(1)</script><path d="M0 0L10 10" onload="alert(1)" /></svg>',
    });
  });

  test("uses strict configuration, sanitizes SVG, and retains a text alternative", async () => {
    const source = "graph TD; A-->B";
    const { container } = render(<MermaidBlock source={source} theme="dark" />);

    expect(await screen.findByRole("img", { name: "Mermaid diagram" })).toBeInTheDocument();
    expect(mermaidCapture.initialize).toHaveBeenCalledWith(expect.objectContaining({
      securityLevel: "strict",
      startOnLoad: false,
      secure: expect.arrayContaining(["securityLevel", "startOnLoad"]),
    }));
    expect(container.querySelector("script")).not.toBeInTheDocument();
    expect(container.querySelector("path")).not.toHaveAttribute("onload");
    expect(screen.getByText("Diagram source")).toBeInTheDocument();
    expect(screen.getByText(source)).toBeInTheDocument();
  });

  test("shows escaped source when Mermaid rejects the diagram", async () => {
    mermaidCapture.render.mockRejectedValue(new Error("Parse error on line 1"));
    render(<MermaidBlock source="not a diagram" theme="dark" />);

    expect(await screen.findByText("Parse error on line 1")).toBeInTheDocument();
    expect(screen.getByText("not a diagram")).toBeInTheDocument();
  });

  test("re-renders when the resolved application theme changes", async () => {
    const { rerender } = render(<MermaidBlock source="graph TD; A-->B" theme="dark" />);
    await waitFor(() => expect(mermaidCapture.render).toHaveBeenCalledTimes(1));

    rerender(<MermaidBlock source="graph TD; A-->B" theme="light" />);

    await waitFor(() => expect(mermaidCapture.render).toHaveBeenCalledTimes(2));
  });
});
