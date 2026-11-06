#!/usr/bin/env node
// External owner launches this synthetic-only peer after final gate approval.
// It never starts UAR, reads credentials, or claims model understanding.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { closeSync, openSync, writeSync } from 'node:fs';
import http from 'node:http';

const namespace = process.env.UAR_MEMORY_CONTEXT_NAMESPACE;
const port = Number(process.env.UAR_MEMORY_CONTEXT_PEER_PORT);
const journal = process.env.UAR_MEMORY_CONTEXT_PEER_JOURNAL;
assert.match(namespace ?? '', /^memory-context-[A-Za-z0-9_-]{1,64}$/);
assert.ok(Number.isSafeInteger(port) && port > 1024 && port <= 65535);
assert.ok(journal?.startsWith('/'), 'External owner supplies an absolute fresh journal path');
const fd = openSync(journal, 'ax', 0o600);
const hash = value => createHash('sha256').update(value).digest('hex');
let ordinal = 0;
const record = row => writeSync(fd, `${JSON.stringify({ namespace, ordinal: ++ordinal, ...row })}\n`);
record({ kind: 'started', protocol: 'uar.memory-context-peer/v1', dimension: 2 });
const reply = (res, status, body) => {
  res.writeHead(status, { 'content-type': 'application/json' });
  res.end(JSON.stringify(body));
};
const server = http.createServer(async (req, res) => {
  try {
    if (req.method === 'GET' && req.url === '/__memory_context/status') {
      return reply(res, 200, { protocol: 'uar.memory-context-peer/v1', namespace, dimension: 2, ordinal });
    }
    if (req.method !== 'POST') return reply(res, 404, { error: 'unsupported synthetic route' });
    const chunks = [];
    let length = 0;
    for await (const chunk of req) {
      length += chunk.length;
      if (length > 2 * 1024 * 1024) return reply(res, 413, { error: 'synthetic body too large' });
      chunks.push(chunk);
    }
    const raw = Buffer.concat(chunks);
    const body = JSON.parse(raw.toString('utf8'));
    if (req.url === '/v1/embeddings' || req.url === '/embeddings') {
      const inputs = typeof body.input === 'string' ? [body.input] : body.input;
      if (!Array.isArray(inputs) || !inputs.every(input => typeof input === 'string')) {
        return reply(res, 400, { error: 'synthetic string embeddings required' });
      }
      if (body.dimensions !== undefined && body.dimensions !== 2) {
        return reply(res, 422, { error: 'fixture dimension must be two' });
      }
      const data = inputs.map((input, index) => ({ object: 'embedding', index,
        embedding: input.includes('MC_VECTOR_NEGATIVE') ? [-1, 0] : [1, 0] }));
      record({ kind: 'embedding', model: body.model,
        inputs: inputs.map((input, index) => ({ sha256: hash(input), vector: data[index].embedding })) });
      return reply(res, 200, { object: 'list', model: body.model, data,
        usage: { prompt_tokens: inputs.length, total_tokens: inputs.length } });
    }
    if (req.url !== '/v1/chat/completions' && req.url !== '/chat/completions') {
      return reply(res, 404, { error: 'unsupported synthetic route' });
    }
    const messages = body.messages;
    if (!Array.isArray(messages)) return reply(res, 400, { error: 'messages required' });
    const userText = messages.filter(message => message.role === 'user').map(message =>
      typeof message.content === 'string' ? message.content : JSON.stringify(message.content)).join('\n');
    const match = userText.match(/UAR_MEMORY_CASE:(memory-context-[A-Za-z0-9_-]+):([a-f0-9-]{36}):(success|error-first|partial-error|opt-out|budget-refusal)/);
    // Model verification/provisioning is external and carries no scenario fact.
    // Record no raw non-scenario content and never inspect/store headers.
    if (!match) {
      if (body.stream === true) {
        res.writeHead(200, { 'content-type': 'text/event-stream', 'cache-control': 'no-cache' });
        res.write(`data: ${JSON.stringify({ id: 'fixture-provisioning', object: 'chat.completion.chunk', model: body.model,
          choices: [{ index: 0, delta: { role: 'assistant', content: 'READY' }, finish_reason: null }] })}\n\n`);
        res.write(`data: ${JSON.stringify({ id: 'fixture-provisioning', object: 'chat.completion.chunk', model: body.model,
          choices: [{ index: 0, delta: {}, finish_reason: 'stop' }],
          usage: { prompt_tokens: 1, completion_tokens: 1, total_tokens: 2 } })}\n\n`);
        res.end('data: [DONE]\n\n');
        return;
      }
      return reply(res, 200, { id: 'fixture-provisioning', object: 'chat.completion', model: body.model,
        choices: [{ index: 0, message: { role: 'assistant', content: 'READY' }, finish_reason: 'stop' }],
        usage: { prompt_tokens: 1, completion_tokens: 1, total_tokens: 2 } });
    }
    if (match[1] !== namespace) return reply(res, 409, { error: 'fixture namespace mismatch' });
    const [, , caseId, mode] = match;
    record({ kind: 'chat', case_id: caseId, mode, request_sha256: hash(raw), request: body });
    if (mode === 'error-first') {
      return reply(res, 503, { error: { type: 'server_error', code: 'memory_context_fixture_error',
        message: 'Synthetic provider error before positive content' } });
    }
    res.writeHead(200, { 'content-type': 'text/event-stream', 'cache-control': 'no-cache', connection: 'close' });
    const frame = value => res.write(`data: ${JSON.stringify(value)}\n\n`);
    const chunk = delta => ({ id: `memory-context-${caseId}`, object: 'chat.completion.chunk', model: body.model,
      choices: [{ index: 0, delta, finish_reason: null }] });
    frame(chunk({ role: 'assistant', content: mode === 'partial-error' ? 'synthetic partial response' : 'synthetic boundary response' }));
    if (mode === 'partial-error') {
      // Deliberate EOF without finish_reason or [DONE]. LiterDriver explicitly
      // rejects a provider stream without terminal completion after this delta.
      res.end();
      return;
    }
    frame({ id: `memory-context-${caseId}`, object: 'chat.completion.chunk', model: body.model,
      choices: [{ index: 0, delta: {}, finish_reason: 'stop' }],
      usage: { prompt_tokens: 11, completion_tokens: 3, total_tokens: 14 } });
    res.end('data: [DONE]\n\n');
  } catch {
    if (!res.headersSent) reply(res, 400, { error: 'invalid synthetic request' });
    else res.destroy();
  }
});
server.listen(port, '127.0.0.1');
for (const signal of ['SIGINT', 'SIGTERM']) {
  process.once(signal, () => server.close(() => { closeSync(fd); process.exitCode = 0; }));
}
