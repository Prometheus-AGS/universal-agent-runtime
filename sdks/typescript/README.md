# universal-agent-runtime TypeScript SDK

TypeScript SDK for universal-agent-runtime.

## Installation

```bash
npm install @prometheus-ags/universal-agent-runtime-sdk
# or
bun add @prometheus-ags/universal-agent-runtime-sdk
```

## Usage

```typescript
import { Client } from '@prometheus-ags/universal-agent-runtime-sdk';

const client = new Client('http://localhost:1906');

// Chat API
const chat = await client.chat.send('Hello!');
console.log('Session:', chat.session_id);

// Knowledge Base API
const kbs = await client.knowledge.list();
for (const kb of kbs) {
  console.log(`KB: ${kb.name} (${kb.id})`);
}

// Search
const results = await client.knowledge.search('kb-id', 'query');
for (const result of results.results) {
  console.log(`Score: ${result.score.toFixed(2)} - ${result.content}`);
}

// SSE Streaming
const eventSource = client.runs.stream('run-id');
eventSource.onmessage = (event) => {
  console.log('Event:', event.data);
};
```

## License

MIT
