// @vitest-environment jsdom
import { act, lazy, Suspense } from "react";
import { render, screen } from "@testing-library/react";
import { describe, expect, test, vi } from "vitest";
import { LazyMarkdownBlockBoundary } from "./lazy-markdown-block-boundary";
import { SourceCodeBlock } from "./source-code-block";

describe("LazyMarkdownBlockBoundary", () => {
  test("keeps escaped source while a module is pending and after its load rejects", async () => {
    let rejectModule: (reason: Error) => void = () => undefined;
    const LazyRejectedBlock = lazy(() => new Promise<{ default: React.FC }>((_resolve, reject) => {
      rejectModule = reject;
    }));
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => undefined);
    const source = "const pending = true;";

    render(
      <div>
        <p>Sibling prose</p>
        <LazyMarkdownBlockBoundary language="ts" resetKey={source} source={source}>
          <Suspense fallback={<SourceCodeBlock language="ts" source={source} status="Loading preview" />}>
            <LazyRejectedBlock />
          </Suspense>
        </LazyMarkdownBlockBoundary>
      </div>,
    );

    expect(screen.getByText("Loading preview")).toBeInTheDocument();
    expect(screen.getByText(source)).toBeInTheDocument();

    await act(async () => {
      rejectModule(new Error("module load rejected"));
    });

    expect(await screen.findByText("Preview unavailable; showing source")).toBeInTheDocument();
    expect(screen.getByText(source)).toBeInTheDocument();
    expect(screen.getByText("Sibling prose")).toBeInTheDocument();
    consoleError.mockRestore();
  });
});
