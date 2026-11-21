import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, test, vi } from "vitest";

import type { ModelReceipt } from "@/features/chat/model/model-receipt";

import { AnswerReceipt } from "./answer-receipt";

const mocks = vi.hoisted(() => ({ receipt: null as ModelReceipt | null }));

vi.mock("@/features/chat/model/use-answer-receipt", () => ({
  useAnswerReceipt: () => mocks.receipt,
}));
vi.mock("@/platform/entities", () => ({ isModelOrigin: () => true }));

describe("AnswerReceipt", () => {
  beforeEach(() => {
    mocks.receipt = null;
  });

  test("renders nothing until the runtime sends a receipt", () => {
    const { container } = render(<AnswerReceipt messageId="m1" />);
    expect(container).toBeEmptyDOMElement();
  });

  test("shows the serving model, its origin and the time the system default became it", () => {
    mocks.receipt = {
      provider: "bdd-provider-b",
      model: "claude-5-haiku-20261101",
      origin: "system_default",
      since: "2026-11-20T10:11:12Z",
    };
    render(<AnswerReceipt messageId="m1" />);

    expect(screen.getByTestId("answer-receipt-model")).toHaveTextContent(
      /^bdd-provider-b\/claude-5-haiku-20261101$/,
    );
    expect(screen.getByTestId("answer-receipt-origin")).toHaveTextContent(/^system default$/);
    expect(screen.getByTestId("answer-receipt-since")).toHaveTextContent(/\d{2}:\d{2}:\d{2}/);
  });

  test("shows no time for an origin other than the system default", () => {
    mocks.receipt = { provider: "bdd-provider-c", model: "qwen-5-turbo", origin: "conversation", since: null };
    render(<AnswerReceipt messageId="m1" />);

    expect(screen.getByTestId("answer-receipt-origin")).toHaveTextContent(/^this conversation$/);
    expect(screen.queryByTestId("answer-receipt-since")).not.toBeInTheDocument();
  });

  test("states no origin when the runtime could not", () => {
    mocks.receipt = { provider: "openai", model: "gpt-5.4-mini", origin: null, since: null };
    render(<AnswerReceipt messageId="m1" />);

    expect(screen.getByTestId("answer-receipt-model")).toBeInTheDocument();
    expect(screen.queryByTestId("answer-receipt-origin")).not.toBeInTheDocument();
  });
});
