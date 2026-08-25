// @vitest-environment jsdom
import { render, screen } from "@testing-library/react";
import rehypeKatex from "rehype-katex";
import rehypeRaw from "rehype-raw";
import rehypeSanitize from "rehype-sanitize";
import { describe, expect, test, vi } from "vitest";
import { MarkdownBubble } from "./markdown-bubble";
import { rehypeChain } from "./plugins/rehype-chain";
import { remarkChain } from "./plugins/remark-chain";
import { sanitizeRawSvg } from "./plugins/sanitize-raw-svg";
import { createMarkdownSanitizeSchema } from "./plugins/sanitize-schema";

const assistantPrimitiveCapture = vi.hoisted(() => ({
  props: undefined as Record<string, unknown> | undefined,
  running: false,
}));

vi.mock("@assistant-ui/react", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@assistant-ui/react")>();

  return {
    ...actual,
    useAuiState: (selector: (state: unknown) => unknown) => selector({
      optional: {
        message: {
          status: { type: assistantPrimitiveCapture.running ? "running" : "complete" },
        },
      },
    }),
  };
});

vi.mock("@/hooks/use-theme", () => ({
  useTheme: () => ({ resolved: "dark" }),
}));

vi.mock("./blocks/code-block", () => ({
  CodeBlock: ({ language, source }: { language: string; source: string }) => {
    if (source === "explode") throw new Error("renderer exploded");
    return <div data-testid="lazy-code-block" data-language={language}>{source}</div>;
  },
}));

vi.mock("./blocks/mermaid-block", () => ({
  MermaidBlock: ({ source }: { source: string }) => <div data-testid="lazy-mermaid-block">{source}</div>,
}));

vi.mock("@assistant-ui/react-markdown", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@assistant-ui/react-markdown")>();

  return {
    ...actual,
    MarkdownTextPrimitive: (props: Record<string, unknown>) => {
      assistantPrimitiveCapture.props = props;
      const components = props.components as {
        code: React.ComponentType<React.ComponentPropsWithoutRef<"code">>;
        pre: React.ComponentType<React.ComponentPropsWithoutRef<"pre">>;
      };
      const Code = components.code;
      const Pre = components.pre;
      return (
        <div data-testid="assistant-markdown-primitive">
          <Pre><Code className="language-mermaid">graph TD; A--&gt;B</Code></Pre>
        </div>
      );
    },
  };
});

describe("MarkdownBubble", () => {
  test("renders GFM tables, model-style hard breaks, and KaTeX", () => {
    const source = [
      "| name | value |",
      "| --- | --- |",
      "| alpha | 1 |",
      "",
      "first line",
      "second line",
      "",
      "Inline math: $x^2$",
    ].join("\n");

    const { container } = render(<MarkdownBubble source={source} />);

    expect(screen.getByRole("table")).toBeInTheDocument();
    expect(container.querySelector("p br")).toBeInTheDocument();
    expect(container.querySelector(".katex")).toBeInTheDocument();
    expect(container.querySelector("math")).not.toBeNull();
  });

  test("keeps raw parsing, sanitization, and KaTeX in the required order", () => {
    expect(rehypeChain[0]).toBe(rehypeRaw);
    expect(rehypeChain[1]).toEqual([rehypeSanitize, expect.any(Object)]);
    expect(rehypeChain[2]).toEqual([
      rehypeKatex,
      expect.objectContaining({ throwOnError: false, strict: "ignore" }),
    ]);
  });

  test("passes the same ordered chains and deferred rendering to assistant-ui mode", () => {
    assistantPrimitiveCapture.props = undefined;

    render(<MarkdownBubble text="context-owned message" />);
    const primitiveProps = assistantPrimitiveCapture.props as unknown as Record<string, unknown>;

    expect(screen.getByTestId("assistant-markdown-primitive")).toBeInTheDocument();
    expect(primitiveProps.remarkPlugins).toBe(remarkChain);
    expect(primitiveProps.rehypePlugins).toBe(rehypeChain);
    expect(primitiveProps.defer).toBe(true);
  });

  test("dispatches finalized fenced code separately from inline code", async () => {
    const { container } = render(<MarkdownBubble source={"`inline`\n\n```ts\nconst value = 1;\n```"} />);
    const inlineCode = container.querySelector("p code");

    expect(inlineCode).toHaveClass("rounded-md");
    expect(await screen.findByTestId("lazy-code-block")).toHaveAttribute("data-language", "ts");
    expect(screen.getByTestId("lazy-code-block")).toHaveTextContent("const value = 1;");
  });

  test("keeps a closed Mermaid fence as source until the assistant message finalizes", async () => {
    assistantPrimitiveCapture.running = true;
    const { rerender } = render(<MarkdownBubble />);

    expect(screen.getByText("Preview available when response finishes")).toBeInTheDocument();
    expect(screen.queryByTestId("lazy-mermaid-block")).not.toBeInTheDocument();

    assistantPrimitiveCapture.running = false;
    rerender(<MarkdownBubble className="finalized-test" />);

    expect(await screen.findByTestId("lazy-mermaid-block")).toHaveTextContent("graph TD; A-->B");
  });

  test("isolates one lazy block crash and keeps sibling prose", async () => {
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => undefined);
    render(<MarkdownBubble source={"before\n\n```ts\nexplode\n```\n\nafter"} />);

    expect(await screen.findByText("Preview unavailable; showing source")).toBeInTheDocument();
    expect(screen.getByText("explode")).toBeInTheDocument();
    expect(screen.getByText("before")).toBeInTheDocument();
    expect(screen.getByText("after")).toBeInTheDocument();
    consoleError.mockRestore();
  });

  test("does not forward react-markdown AST nodes to the DOM", () => {
    render(<MarkdownBubble source="# Heading" />);

    expect(screen.getByRole("heading", { name: "Heading" })).not.toHaveAttribute("node");
  });

  test("keeps malformed math from crashing the message bubble", () => {
    render(<MarkdownBubble source={"Malformed: $\\notARealCommand{value}$"} />);

    expect(screen.getByText(/Malformed:/)).toBeInTheDocument();
  });

  test.each([
    ["script", "before<script>globalThis.__markdownXss = true</script>after", "script"],
    ["iframe", '<iframe src="https://example.com"></iframe>', "iframe"],
    ["object", '<object data="https://example.com/payload"></object>', "object"],
  ])("removes executable %s elements", (_name, source, selector) => {
    const { container } = render(<MarkdownBubble source={source} />);
    expect(container.querySelector(selector)).not.toBeInTheDocument();
  });

  test("removes handlers, inline styles, arbitrary classes, and unsafe protocols", () => {
    const source = [
      '<img src="https://example.com/image.png" alt="safe" onerror="alert(1)" style="position:fixed">',
      '<span class="untrusted-class" onclick="alert(1)" style="display:none">content</span>',
      '<a href="javascript:alert(1)">bad link</a>',
      '<video src="javascript:alert(1)" controls></video>',
    ].join("\n");

    const { container } = render(<MarkdownBubble source={source} />);
    const image = screen.getByRole("img", { name: "safe" });
    const span = screen.getByText("content");
    const link = screen.getByText("bad link");
    const video = container.querySelector("video");

    expect(image).not.toHaveAttribute("onerror");
    expect(image).not.toHaveAttribute("style");
    expect(span).not.toHaveAttribute("onclick");
    expect(span).not.toHaveAttribute("style");
    expect(span).not.toHaveAttribute("class");
    expect(link.getAttribute("href") ?? "").not.toMatch(/^javascript:/i);
    expect(video?.getAttribute("src") ?? "").not.toMatch(/^javascript:/i);
  });

  test("preserves approved semantic HTML and limited SVG attributes", () => {
    const source = [
      "<details open><summary>Details</summary><mark>safe content</mark></details>",
      '<svg viewBox="0 0 10 10" role="img" aria-label="status glyph"><path d="M0 0L10 10" onclick="alert(1)"></path></svg>',
    ].join("\n");

    const { container } = render(<MarkdownBubble source={source} />);
    const details = container.querySelector("details");
    const svg = screen.getByRole("img", { name: "status glyph" });
    const path = svg.querySelector("path");

    expect(details).toHaveAttribute("open");
    expect(screen.getByText("safe content").tagName).toBe("MARK");
    expect(svg).toHaveAttribute("viewBox", "0 0 10 10");
    expect(path).toHaveAttribute("d", "M0 0L10 10");
    expect(path).not.toHaveAttribute("onclick");
  });

  test("forces safe external-link window isolation", () => {
    render(<MarkdownBubble source="[docs](https://example.com/docs)" />);
    const link = screen.getByRole("link", { name: "docs" });

    expect(link).toHaveAttribute("target", "_blank");
    expect(link).toHaveAttribute("rel", "noopener noreferrer");
  });
});

describe("markdown sanitizer schema", () => {
  test("keeps KaTeX input markers but never allows style attributes", () => {
    const schema = createMarkdownSanitizeSchema();
    const codeRules = schema.attributes?.code ?? [];
    const globalRules = schema.attributes?.["*"] ?? [];

    expect(JSON.stringify(codeRules)).toContain("math-inline");
    expect(JSON.stringify(codeRules)).toContain("math-display");
    expect(globalRules).not.toContain("style");
    expect(schema.tagNames).not.toContain("script");
    expect(schema.tagNames).not.toContain("iframe");
    expect(schema.tagNames).not.toContain("object");
  });

  test("sanitizes standalone SVG artifacts with the SVG profile", () => {
    const sanitized = sanitizeRawSvg(
      '<svg viewBox="0 0 10 10"><script>alert(1)</script><path d="M0 0L10 10" onload="alert(1)" /></svg>',
    );

    expect(sanitized).toContain("<svg");
    expect(sanitized).toContain("<path");
    expect(sanitized).not.toContain("<script");
    expect(sanitized).not.toContain("onload");
  });
});
