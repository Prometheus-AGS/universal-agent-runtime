import { describe, expect, test } from "bun:test";
import {
  DEFAULT_STREAM_RETRY_POLICY,
  computeRetryDelayMs,
  isRetryableHttpStatus,
  isRetryableTransportError,
} from "./use-message-stream";

describe("stream retry policy", () => {
  test("retries only transient HTTP status codes", () => {
    expect(isRetryableHttpStatus(408)).toBe(true);
    expect(isRetryableHttpStatus(429)).toBe(true);
    expect(isRetryableHttpStatus(503)).toBe(true);
    expect(isRetryableHttpStatus(400)).toBe(false);
    expect(isRetryableHttpStatus(401)).toBe(false);
  });

  test("retries network transport errors but not aborts", () => {
    const networkErr = new Error("Failed to fetch");
    networkErr.name = "TypeError";
    expect(isRetryableTransportError(networkErr)).toBe(true);

    const changedErr = new Error("net::ERR_NETWORK_CHANGED");
    changedErr.name = "TypeError";
    expect(isRetryableTransportError(changedErr)).toBe(true);

    const abortErr = new Error("The operation was aborted.");
    abortErr.name = "AbortError";
    expect(isRetryableTransportError(abortErr)).toBe(false);
  });

  test("uses exponential backoff and honors retry-after", () => {
    expect(computeRetryDelayMs(0)).toBe(1000);
    expect(computeRetryDelayMs(1)).toBe(2000);
    expect(computeRetryDelayMs(4)).toBe(10000);

    expect(computeRetryDelayMs(0, "2")).toBe(2000);
    expect(computeRetryDelayMs(0, "10")).toBe(10000);
  });

  test("respects policy-specific retryable statuses", () => {
    const custom = {
      ...DEFAULT_STREAM_RETRY_POLICY,
      retryableHttpStatuses: [409, 429],
    };

    expect(isRetryableHttpStatus(409, custom)).toBe(true);
    expect(isRetryableHttpStatus(500, custom)).toBe(false);
  });
});
