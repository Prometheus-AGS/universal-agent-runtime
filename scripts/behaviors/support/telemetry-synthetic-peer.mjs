// External infrastructure launches this bounded fixture, never the behavior steps.
// It owns no UAR settings, identity, credentials, paid provider or collector.
// Required env: UAR_TELEMETRY_NAMESPACE, UAR_TELEMETRY_TOOL_NAME,
// UAR_TELEMETRY_PEER_JOURNAL; optional PORT (default 0), bind host (loopback default).
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { appendFileSync } from 'node:fs';
import { createServer } from 'node:http';

const namespace = process.env.UAR_TELEMETRY_NAMESPACE;
const toolName = process.env.UAR_TELEMETRY_TOOL_NAME;
const journal = process.env.UAR_TELEMETRY_PEER_JOURNAL;
assert.match(namespace ?? '', /^[A-Za-z0-9_-]{1,96}$/);
assert.ok(toolName?.endsWith('telemetry_echo') && journal);
const bind = process.env.UAR_TELEMETRY_BIND ?? '127.0.0.1';
assert.ok(['127.0.0.1', '0.0.0.0'].includes(bind), 'All-interface mode requires an externally isolated container');
const port = Number(process.env.PORT ?? 0);
assert.ok(Number.isSafeInteger(port) && port >= 0 && port <= 65535);
let sequence = 0;
const cases = new Map();
const hash = value => createHash('sha256').update(value).digest('hex');
function record(event, fields = {}) {
  assert.ok(++sequence <= 512, 'Synthetic peer journal budget exceeded');
  appendFileSync(journal, `${JSON.stringify({ namespace, sequence, event, ...fields })}\n`, { mode: 0o600 });
}
function reply(res, status, value) {
  res.writeHead(status, { 'Content-Type': 'application/json' });
  res.end(JSON.stringify(value));
}
function rpc(res, id, result) { reply(res, 200, { jsonrpc: '2.0', id, result }); }
function text(content) {
  return typeof content === 'string' ? content : Array.isArray(content)
    ? content.filter(part => part?.type === 'text').map(part => part.text ?? '').join('\n') : '';
}
function completion(res, body, state, delta, finish) {
  const usage = { prompt_tokens: 13, completion_tokens: 5, total_tokens: 18 };
  if (body.stream !== true) {
    const message = { role: 'assistant', ...delta };
    if (message.tool_calls) message.tool_calls = message.tool_calls.map(({ index: _index, ...call }) => call);
    reply(res, 200, { id: `synthetic-${state.id}`, object: 'chat.completion', model: body.model,
      choices: [{ index: 0, message, finish_reason: finish }], usage });
    return;
  }
  res.writeHead(200, { 'Content-Type': 'text/event-stream', 'Cache-Control': 'no-cache' });
  const frame = (value, reason, counts) => `data: ${JSON.stringify({ id: `synthetic-${state.id}`,
    object: 'chat.completion.chunk', model: body.model,
    choices: [{ index: 0, delta: value, finish_reason: reason }], ...(counts ? { usage: counts } : {}) })}\n\n`;
  res.end(frame(delta, null) + frame({}, finish, usage) + 'data: [DONE]\n\n');
}
async function bodyOf(req) {
  let size = 0;
  const chunks = [];
  for await (const chunk of req) {
    size += chunk.length;
    assert.ok(size <= 2 * 1024 * 1024, 'Synthetic fixture request budget exceeded');
    chunks.push(chunk);
  }
  return JSON.parse(Buffer.concat(chunks).toString('utf8'));
}
const server = createServer(async (req, res) => {
  try {
    assert.equal(req.method, 'POST');
    const body = await bodyOf(req);
    if (req.url === '/mcp') {
      if (body.method?.startsWith('notifications/')) { res.writeHead(202).end(); return; }
      if (body.method === 'initialize') {
        rpc(res, body.id, { protocolVersion: body.params?.protocolVersion ?? '2025-03-26',
          capabilities: { tools: {} }, serverInfo: { name: 'owned-telemetry-fixture', version: '1' } }); return;
      }
      if (body.method === 'tools/list') {
        rpc(res, body.id, { tools: [{ name: 'telemetry_echo', description: 'Owned synthetic read-only telemetry echo.',
          inputSchema: { type: 'object', required: ['value'], properties: { value: { type: 'string' } }, additionalProperties: false },
          annotations: { readOnlyHint: true, destructiveHint: false, openWorldHint: false } }] }); return;
      }
      if (body.method === 'tools/call') {
        assert.equal(body.params?.name, 'telemetry_echo');
        const match = /^TELEMETRY_TOOL_ARG_([a-f0-9-]{36})$/.exec(body.params?.arguments?.value ?? '');
        assert.ok(match && cases.has(match[1]), 'Unknown owned tool case');
        const state = cases.get(match[1]);
        state.toolCalls += 1;
        const result = `TELEMETRY_TOOL_RESULT_${state.id}`;
        record('tool_executed', { caseId: state.id, toolId: state.toolId, toolCalls: state.toolCalls,
          argumentSha256: hash(body.params.arguments.value), resultSha256: hash(result) });
        rpc(res, body.id, { content: [{ type: 'text', text: result }], isError: false }); return;
      }
      reply(res, 400, { error: 'unsupported_fixture_method' }); return;
    }
    assert.ok(['/v1/chat/completions', '/chat/completions'].includes(req.url));
    const messages = body.messages;
    assert.ok(Array.isArray(messages));
    const user = messages.filter(message => message.role === 'user').map(message => text(message.content)).join('\n');
    const match = /UAR_TELEMETRY_CASE:([a-f0-9-]{36}):(done|cancelled)/.exec(user);
    assert.ok(match && user.includes(`TELEMETRY_PROMPT_${match[1]}`), 'Owned prompt marker required');
    const id = match[1];
    if (!cases.has(id)) {
      assert.ok(cases.size < 8, 'Synthetic case budget exceeded');
      cases.set(id, { id, terminal: match[2], attempts: 0, toolCalls: 0, toolId: `telemetry-${id}` });
    }
    const state = cases.get(id);
    state.attempts += 1;
    const hasResult = messages.some(message => message.role === 'tool'
      && text(message.content).includes(`TELEMETRY_TOOL_RESULT_${id}`));
    record('provider_arrival', { caseId: id, providerAttempt: state.attempts, toolId: state.toolId,
      promptSha256: hash(`TELEMETRY_PROMPT_${id}`), toolResultSeen: hasResult });
    if (state.attempts === 1) {
      record('provider_retryable', { caseId: id, providerAttempt: 1 });
      reply(res, 503, { error: { type: 'server_error', code: 'synthetic_retry', message: 'Owned presemantic retry fixture' } }); return;
    }
    if (!hasResult) {
      assert.ok(body.tools?.some(tool => tool.function?.name === toolName), 'Exact governed tool must be advertised');
      record('provider_tool_intent', { caseId: id, toolId: state.toolId });
      completion(res, body, state, { role: 'assistant', tool_calls: [{ index: 0, id: state.toolId,
        type: 'function', function: { name: toolName, arguments: JSON.stringify({ value: `TELEMETRY_TOOL_ARG_${id}` }) } }] }, 'tool_calls');
      return;
    }
    assert.equal(state.toolCalls, 1, 'Expected one actual governed tool execution');
    record('provider_tool_result_received', { caseId: id, toolId: state.toolId,
      resultSha256: hash(`TELEMETRY_TOOL_RESULT_${id}`) });
    if (state.terminal === 'cancelled') {
      // Hold a post-tool response until the actual UAR cancellation closes it.
      record('provider_held', { caseId: id });
      res.on('close', () => record('provider_cancelled', { caseId: id }));
      return;
    }
    completion(res, body, state, { role: 'assistant', content: 'TELEMETRY_SYNTHETIC_COMPLETE' }, 'stop');
    record('provider_completed', { caseId: id });
  } catch {
    record('fixture_refused');
    if (!res.headersSent) reply(res, 422, { error: 'owned_telemetry_fixture_refused' });
    else res.end();
  }
});
server.listen(port, bind, () => {
  record('fixture_identity', { fixture: 'telemetry-synthetic-peer/v1' });
  console.log(`TELEMETRY_PEER_PORT=${server.address().port}`);
});
