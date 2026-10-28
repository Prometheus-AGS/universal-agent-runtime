// Run two chat turns through the shipped TypeScript SDK; no credentials.
// Reads JSON inputs on stdin and writes one JSON receipt to stdout.
import { UarClient } from '../../../sdks/typescript/dist/index.js';

async function readStdin() {
  let text = '';
  for await (const chunk of process.stdin) text += chunk;
  return JSON.parse(text);
}

async function turn(client, mode, text, request) {
  const body = { ...request, messages: [{ role: 'user', content: text }] };
  if (mode === 'json') {
    const value = await client.chat.complete(body);
    return { id: value.id, choices: value.choices.length,
      finish_reason: value.choices[0].finish_reason, content: value.choices[0].message.content };
  }
  const chunks = [], ids = new Set(), finishes = [], events = new Set();
  for await (const event of client.chat.stream(body)) {
    events.add(event.event ?? 'message');
    const data = event.data;
    if (!data || typeof data !== 'object' || data.object !== 'chat.completion.chunk') continue;
    if (data.id) ids.add(data.id);
    for (const choice of data.choices ?? []) {
      const fragment = choice.delta?.content;
      if (typeof fragment === 'string' && fragment) chunks.push(fragment);
      if (choice.finish_reason != null) finishes.push(choice.finish_reason);
    }
  }
  return { ids: [...ids].sort(), content_chunks: chunks.length, finish_reasons: finishes,
    events: [...events].sort(), content: chunks.join('') };
}

const inputs = await readStdin();
const client = new UarClient(inputs.base_url, { headers: { 'x-uar-workspace-id': inputs.workspace } });
const turns = [];
for (const text of inputs.turns) {
  turns.push(await turn(client, inputs.mode, text,
    { model: inputs.model, agentId: inputs.agent_id, sessionId: inputs.session_id }));
}
process.stdout.write(JSON.stringify({ client: 'typescript_sdk', mode: inputs.mode, turns }));
