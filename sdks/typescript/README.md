# Universal Agent Runtime TypeScript SDK

Typed, runtime-validated TypeScript client for Universal Agent Runtime 1.0.
It supports Node.js 20+, modern browsers, serverless runtimes, and Next.js.

## Install

```bash
npm install @prometheus-ags/universal-agent-runtime-sdk
```

## Use

```typescript
import { UarClient } from "@prometheus-ags/universal-agent-runtime-sdk";

const client = new UarClient("http://localhost:1906", {
  apiKey: process.env.UAR_API_KEY,
});

const reply = await client.chat.complete({
  messages: [{ role: "user", content: "Hello" }],
});
console.log(reply.choices[0]?.message.content);

for await (const event of client.chat.stream({
  messages: [{ role: "user", content: "Stream a haiku" }],
})) {
  console.log(event);
}
```

The public client namespaces cover:

- `chat`: completion, SSE streaming, and Zod-validated structured output
- `tools`: namespaced tool execution
- `embeddings`: OpenAI-compatible embedding creation
- `runs`: create, stream, cancel, list checkpoints, and resume
- `knowledge`: knowledge-base CRUD, documents, and search
- `ingest`: content ingestion

Every JSON response is validated with Zod. Failed HTTP responses throw
`UarSdkError` with the HTTP `status` and parsed server `details` intact.
Streaming methods return `AsyncIterable<SseEvent>` and accept an `AbortSignal`
and `lastEventId` for cancellation and replay.

Six typechecked examples live in [`examples/`](examples/), including a Next.js
route-handler example. Generate the API reference with `npm run docs`.

## Verify

```bash
npm run typecheck
npm run lint
npm test
npm run build
npm run docs
npm run examples
```

## License

MIT
