// @vitest-environment jsdom
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, test, vi } from "vitest";
import { CodeBlock } from "./code-block";

const shikiCapture = vi.hoisted(() => ({
  codeToTokens: vi.fn(),
}));

vi.mock("shiki/bundle/full", () => ({
  bundledLanguages: { javascript: {}, js: {}, typescript: {}, ts: {} },
  codeToTokens: shikiCapture.codeToTokens,
}));

const tokenResult = (themeName: string, lineCount = 1) => ({
  themeName,
  tokens: Array.from({ length: lineCount }, (_, index) => [
    { content: index === 0 ? '<img src=x onerror="alert(1)">' : `line ${index + 1}`, color: "#ff6600", offset: index * 10 },
  ]),
});

describe("CodeBlock", () => {
  beforeEach(() => {
    shikiCapture.codeToTokens.mockReset();
    shikiCapture.codeToTokens.mockResolvedValue(tokenResult("github-dark"));
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: { writeText: vi.fn().mockResolvedValue(undefined) },
    });
  });

  test("renders Shiki tokens as React text rather than injected HTML", async () => {
    const { container } = render(<CodeBlock language="javascript" source={'<img src=x onerror="alert(1)">'} theme="dark" />);

    await waitFor(() => expect(container.querySelector("[data-shiki-code-block]")).toBeInTheDocument());
    expect(container).toHaveTextContent('<img src=x onerror="alert(1)">');
    expect(container.querySelector("img")).not.toBeInTheDocument();
    expect(container.querySelector("[data-shiki-code-block]")).toHaveAttribute("data-shiki-theme", "github-dark");
  });

  test("refreshes tokenization when the resolved theme changes", async () => {
    const { rerender } = render(<CodeBlock language="ts" source="const value = 1" theme="dark" />);
    await waitFor(() => expect(shikiCapture.codeToTokens).toHaveBeenCalledWith(
      "const value = 1",
      expect.objectContaining({ theme: "github-dark" }),
    ));

    shikiCapture.codeToTokens.mockResolvedValue(tokenResult("github-light"));
    rerender(<CodeBlock language="ts" source="const value = 1" theme="light" />);

    await waitFor(() => expect(shikiCapture.codeToTokens).toHaveBeenLastCalledWith(
      "const value = 1",
      expect.objectContaining({ theme: "github-light" }),
    ));
  });

  test("degrades unsupported languages to escaped source", async () => {
    render(<CodeBlock language="unknown-language" source="plain source" theme="dark" />);

    expect(await screen.findByText("Syntax preview unavailable; showing source")).toBeInTheDocument();
    expect(screen.getByText("plain source")).toBeInTheDocument();
    expect(shikiCapture.codeToTokens).not.toHaveBeenCalled();
  });

  test("adds line numbers after eight lines and exposes copy and wrap controls", async () => {
    shikiCapture.codeToTokens.mockResolvedValue(tokenResult("github-dark", 9));
    const { container } = render(<CodeBlock language="ts" source="nine lines" theme="dark" />);

    expect(await screen.findByText("9")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Wrap" }));
    expect(screen.getByRole("button", { name: "No wrap" })).toHaveAttribute("aria-pressed", "true");

    fireEvent.click(screen.getByRole("button", { name: "Copy" }));
    await waitFor(() => expect(navigator.clipboard.writeText).toHaveBeenCalledWith("nine lines"));
    expect(await screen.findByRole("button", { name: "Copied" })).toBeInTheDocument();
    expect(container.querySelector("pre")).toHaveClass("whitespace-pre-wrap");
  });
});
