import { expect, test } from "vitest";
import { acquireChatStream, ownsChatStream } from "./stream-ownership";

test("shared workflow reservation prevents a second sender and stale release cannot unlock a new owner", () => {
  const first = acquireChatStream();
  expect(() => acquireChatStream()).toThrow("Another response is active");
  first.release();
  const second = acquireChatStream();
  first.release();
  expect(ownsChatStream(second)).toBe(true);
  expect(() => acquireChatStream()).toThrow("Another response is active");
  second.release();
});
