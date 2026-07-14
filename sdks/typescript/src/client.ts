import { fetchEventSource, type EventSourceMessage } from "@microsoft/fetch-event-source";
import { z, type ZodType } from "zod";
import * as schemas from "./schemas.js";
import type {
  ChatRequest, ClientOptions, CreateRunRequest, IngestRequest, JsonValue,
  KnowledgeBaseInput, ResumeRunRequest, SearchRequest, SseEvent, StreamOptions,
} from "./types.js";

/** Error returned for failed transport, protocol, or validation operations. */
export class UarSdkError extends Error {
  constructor(message: string, readonly status?: number, readonly details?: unknown, options?: ErrorOptions) {
    super(message, options);
    this.name = "UarSdkError";
  }
}

class EventStream<T> implements AsyncIterable<T> {
  private values: T[] = [];
  private waiters: Array<(value: IteratorResult<T>) => void> = [];
  private failure?: unknown;
  private ended = false;

  push(value: T): void { const waiter = this.waiters.shift(); waiter ? waiter({ value, done: false }) : this.values.push(value); }
  close(): void { this.ended = true; this.flush(); }
  fail(error: unknown): void { this.failure = error; this.ended = true; this.flush(); }
  private flush(): void { for (const waiter of this.waiters.splice(0)) waiter({ value: undefined, done: true }); }
  async next(): Promise<IteratorResult<T>> {
    const value = this.values.shift();
    if (value !== undefined) return { value, done: false };
    if (this.failure) throw this.failure;
    if (this.ended) return { value: undefined, done: true };
    return new Promise((resolve) => this.waiters.push(resolve));
  }
  [Symbol.asyncIterator](): AsyncIterator<T> { return this; }
}

/** Typed client for the Universal Agent Runtime HTTP API. */
export class UarClient {
  private readonly baseUrl: string;
  private readonly fetcher: typeof globalThis.fetch;
  private readonly headers: Headers;

  constructor(baseUrl: string, options: ClientOptions = {}) {
    this.baseUrl = baseUrl.replace(/\/$/, "");
    this.fetcher = options.fetch ?? globalThis.fetch;
    this.headers = new Headers(options.headers);
    this.headers.set("accept", "application/json");
    if (options.apiKey) this.headers.set("authorization", `Bearer ${options.apiKey}`);
  }

  readonly chat = {
    complete: (request: ChatRequest) => this.request("/api/chat/completion", schemas.chatCompletionSchema, {
      method: "POST", body: { messages: request.messages, model: request.model, temperature: request.temperature, tools: request.tools },
      headers: request.sessionId ? { "x-uar-session-id": request.sessionId } : undefined,
    }),
    stream: (request: ChatRequest, options?: StreamOptions) => this.stream("/api/chat/completion", {
      method: "POST", body: { messages: request.messages, model: request.model, temperature: request.temperature, tools: request.tools, stream: true },
      headers: request.sessionId ? { "x-uar-session-id": request.sessionId } : undefined,
    }, options),
    structured: async <T>(request: ChatRequest, schema: ZodType<T>): Promise<T> => {
      const result = await this.request("/api/chat/completion", schemas.chatCompletionSchema, {
        method: "POST", body: { messages: request.messages, model: request.model, response_format: { type: "json_object" } },
      });
      const content = result.choices[0]?.message.content;
      if (!content) throw new UarSdkError("Structured response contained no content");
      try { return schema.parse(JSON.parse(content)); } catch (error) { throw new UarSdkError("Invalid structured response", undefined, content, { cause: error }); }
    },
  };

  readonly tools = { execute: (name: string, arguments_: Record<string, JsonValue> = {}) =>
    this.request(`/api/tools/${encodeURIComponent(name)}/execute`, schemas.toolResultSchema, { method: "POST", body: { arguments: arguments_ } }) };

  readonly embeddings = { create: (input: string | string[], model?: string) =>
    this.request("/v1/embeddings", schemas.embeddingResponseSchema, { method: "POST", body: { input, model } }) };

  readonly runs = {
    create: (request: CreateRunRequest) => this.request("/api/uar/runs", schemas.runResponseSchema, {
      method: "POST", body: { artifact: request.artifact, input: request.input, session_id: request.sessionId },
    }),
    stream: (runId: string, options?: StreamOptions) => this.stream(`/api/uar/runs/${encodeURIComponent(runId)}/stream`, {}, options),
    cancel: (runId: string) => this.request(`/api/uar/runs/${encodeURIComponent(runId)}/cancel`, schemas.cancelResponseSchema, { method: "POST" }),
    checkpoints: (runId: string) => this.request(`/api/uar/runs/${encodeURIComponent(runId)}/checkpoints`, schemas.checkpointListSchema),
    resume: (runId: string, request: ResumeRunRequest, checkpointId?: string) => this.request(
      `/api/uar/runs/${encodeURIComponent(runId)}/resume${checkpointId ? `/${encodeURIComponent(checkpointId)}` : ""}`,
      schemas.runResponseSchema, { method: "POST", body: { artifact: request.artifact, input: request.input, session_id: request.sessionId } },
    ),
  };

  readonly knowledge = {
    list: () => this.request("/api/knowledge", z.array(schemas.knowledgeBaseSchema)),
    create: (input: KnowledgeBaseInput) => this.request("/api/knowledge", schemas.knowledgeBaseSchema, { method: "POST", body: input }),
    get: (id: string) => this.request(`/api/knowledge/${encodeURIComponent(id)}`, schemas.knowledgeBaseSchema),
    update: (id: string, input: Partial<KnowledgeBaseInput>) => this.request(`/api/knowledge/${encodeURIComponent(id)}`, schemas.knowledgeBaseSchema, { method: "PUT", body: input }),
    delete: (id: string) => this.requestVoid(`/api/knowledge/${encodeURIComponent(id)}`, { method: "DELETE" }),
    documents: (id: string) => this.request(`/api/knowledge/${encodeURIComponent(id)}/documents`, z.array(schemas.documentSchema)),
    deleteDocument: (id: string, documentId: string) => this.requestVoid(`/api/knowledge/${encodeURIComponent(id)}/documents/${encodeURIComponent(documentId)}`, { method: "DELETE" }),
    search: (id: string, request: SearchRequest) => this.request(`/api/knowledge/${encodeURIComponent(id)}/search`, schemas.searchResponseSchema, {
      method: "POST", body: { query: request.query, limit: request.limit, min_score: request.minScore },
    }),
  };

  readonly ingest = { content: (request: IngestRequest) => this.request("/api/ingest", schemas.ingestResponseSchema, { method: "POST", body: request }) };

  private async request<T>(path: string, schema: ZodType<T>, init: { method?: string; body?: unknown; headers?: HeadersInit } = {}): Promise<T> {
    const response = await this.fetcher(`${this.baseUrl}${path}`, { method: init.method, headers: this.requestHeaders(init.headers, init.body), body: init.body === undefined ? undefined : JSON.stringify(init.body) });
    const body = await this.readBody(response);
    if (!response.ok) throw new UarSdkError(`UAR request failed with ${response.status}`, response.status, body);
    try { return schema.parse(body); } catch (error) { throw new UarSdkError("UAR response validation failed", response.status, body, { cause: error }); }
  }

  private async requestVoid(path: string, init: { method: string }): Promise<void> {
    const response = await this.fetcher(`${this.baseUrl}${path}`, { method: init.method, headers: this.requestHeaders() });
    if (!response.ok) throw new UarSdkError(`UAR request failed with ${response.status}`, response.status, await this.readBody(response));
  }

  private stream(path: string, init: { method?: string; body?: unknown; headers?: HeadersInit }, options: StreamOptions = {}): AsyncIterable<SseEvent> {
    const output = new EventStream<SseEvent>();
    const headers = this.requestHeaders(init.headers, init.body);
    if (options.lastEventId) headers.set("last-event-id", options.lastEventId);
    void fetchEventSource(`${this.baseUrl}${path}`, {
      method: init.method, headers: Object.fromEntries(headers.entries()), body: init.body === undefined ? undefined : JSON.stringify(init.body), signal: options.signal,
      openWhenHidden: true, fetch: this.fetcher,
      onopen: async (response) => { if (!response.ok) throw new UarSdkError(`SSE request failed with ${response.status}`, response.status, await this.readBody(response)); },
      onmessage: (message: EventSourceMessage) => output.push({ id: message.id || undefined, event: message.event || undefined, data: this.parseEventData(message.data) }),
      onclose: () => output.close(), onerror: (error) => { output.fail(error); throw error; },
    }).catch((error: unknown) => output.fail(error));
    return output;
  }

  private requestHeaders(extra?: HeadersInit, body?: unknown): Headers {
    const headers = new Headers(this.headers); new Headers(extra).forEach((value, key) => headers.set(key, value));
    if (body !== undefined) headers.set("content-type", "application/json"); return headers;
  }
  private async readBody(response: Response): Promise<unknown> { const text = await response.text(); if (!text) return undefined; try { return JSON.parse(text); } catch { return text; } }
  private parseEventData(data: string): JsonValue { try { return JSON.parse(data) as JsonValue; } catch { return data; } }
}
