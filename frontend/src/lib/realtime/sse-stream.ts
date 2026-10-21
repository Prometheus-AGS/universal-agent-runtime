/**
 * Minimal fetch-based SSE reader.
 *
 * Native EventSource reconnects on its own schedule. Realtime recovery is a
 * lifecycle decision in UAR, so this reader exposes one connection attempt and
 * leaves a later attempt to an explicit caller event.
 */

export interface SseFrame {
  event: string;
  data: string;
}

export interface SseStream {
  abort(): void;
  done: Promise<void>;
}

export interface OpenSseStreamOptions {
  onFrame(frame: SseFrame): void;
  onOpen?(): void;
}

/**
 * A host or UI may explicitly request a fresh connection after it has
 * re-established an authenticated runtime relationship. This event never
 * authorizes access; the subsequent HTTP request remains authenticated.
 */
export const SSE_REINITIALIZE_EVENT = "uar:realtime-reinitialize";

export function installSseRecoveryEvents(options: {
  recover(): void;
  reinitialize(): void;
}): () => void {
  if (typeof window === "undefined" || typeof document === "undefined") {
    return () => {};
  }

  const onOnline = () => options.recover();
  const onVisibilityChange = () => {
    if (document.visibilityState === "visible") options.recover();
  };
  const onReinitialize = () => options.reinitialize();

  window.addEventListener("online", onOnline);
  document.addEventListener("visibilitychange", onVisibilityChange);
  window.addEventListener(SSE_REINITIALIZE_EVENT, onReinitialize);
  return () => {
    window.removeEventListener("online", onOnline);
    document.removeEventListener("visibilitychange", onVisibilityChange);
    window.removeEventListener(SSE_REINITIALIZE_EVENT, onReinitialize);
  };
}

function parseBlock(block: string): SseFrame | null {
  let event = "message";
  const data: string[] = [];

  for (const line of block.split(/\r?\n/)) {
    if (line.length === 0 || line.startsWith(":")) continue;
    const separator = line.indexOf(":");
    const field = separator === -1 ? line : line.slice(0, separator);
    const value =
      separator === -1
        ? ""
        : line.charCodeAt(separator + 1) === 32
          ? line.slice(separator + 2)
          : line.slice(separator + 1);

    if (field === "event") event = value;
    if (field === "data") data.push(value);
  }

  return data.length > 0 ? { event, data: data.join("\n") } : null;
}

function consumeBuffer(
  buffer: string,
  onFrame: (frame: SseFrame) => void,
): string {
  let boundary = buffer.search(/\r?\n\r?\n/);
  while (boundary !== -1) {
    const block = buffer.slice(0, boundary);
    const delimiterLength =
      buffer[boundary] === "\r" ? 4 : buffer[boundary + 1] === "\r" ? 3 : 2;
    buffer = buffer.slice(boundary + delimiterLength);
    const frame = parseBlock(block);
    if (frame) onFrame(frame);
    boundary = buffer.search(/\r?\n\r?\n/);
  }
  return buffer;
}

export function openSseStream(
  url: string,
  options: OpenSseStreamOptions,
): SseStream {
  const controller = new AbortController();
  const done = (async () => {
    const response = await fetch(url, {
      credentials: "include",
      headers: { Accept: "text/event-stream" },
      signal: controller.signal,
    });
    if (!response.ok) {
      throw new Error(`SSE connection failed: ${response.status}`);
    }
    if (!response.body) {
      throw new Error("SSE connection has no response body");
    }

    options.onOpen?.();
    const reader = response.body.getReader();
    const decoder = new TextDecoder();
    let buffer = "";
    try {
      for (;;) {
        const { done, value } = await reader.read();
        if (done) break;
        buffer = consumeBuffer(buffer + decoder.decode(value, { stream: true }), options.onFrame);
      }
      buffer = consumeBuffer(buffer + decoder.decode(), options.onFrame);
    } finally {
      reader.releaseLock();
    }
  })();

  return {
    abort: () => controller.abort(),
    done,
  };
}
