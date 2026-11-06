#!/usr/bin/env node
// External owner launches this synthetic provider only after the final gate opens.
// This file never launches UAR/MCP, provisions a provider, or reads credentials.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { closeSync, openSync, writeSync } from 'node:fs';
import http from 'node:http';
const namespace = process.env.UAR_NATIVE_CHAT_NAMESPACE;
const toolName = process.env.UAR_NATIVE_CHAT_EFFECT_TOOL_NAME;
const port = Number(process.env.UAR_NATIVE_CHAT_PEER_PORT);
const journal = process.env.UAR_NATIVE_CHAT_PEER_JOURNAL;
assert.match(namespace ?? '', /^native-chat-[A-Za-z0-9_-]{1,64}$/);
assert.ok(toolName && Number.isSafeInteger(port) && port > 1024 && port <= 65535);
assert.ok(journal?.startsWith('/'));
const fd = openSync(journal, 'ax', 0o600);
let ordinal = 0;
const record = row => writeSync(fd, `${JSON.stringify({ namespace, ordinal: ++ordinal, ...row })}\n`);
const digest = value => createHash('sha256').update(value).digest('hex');
record({ kind: 'started', protocol: 'uar.native-chat-admission-peer/v1' });
const server = http.createServer(async (req, res) => {
  const reply = (status, body) => { res.writeHead(status, { 'content-type': 'application/json' }); res.end(JSON.stringify(body)); };
  try {
    if (req.method !== 'POST' || !['/v1/chat/completions', '/chat/completions'].includes(req.url)) return reply(404, { error: 'synthetic_route_unknown' });
    const chunks = []; let size = 0;
    for await (const chunk of req) {
      size += chunk.length;
      if (size > 2 * 1024 * 1024) { record({ kind: 'provider', case_id: null, stage: 'oversized' }); return reply(413, { error: 'synthetic_body_limit' }); }
      chunks.push(chunk);
    }
    const raw = Buffer.concat(chunks);
    let body;
    try { body = JSON.parse(raw.toString('utf8')); }
    catch { record({ kind: 'provider', case_id: null, stage: 'invalid_json' }); return reply(400, { error: 'synthetic_json_invalid' }); }
    const messages = Array.isArray(body.messages) ? body.messages : [];
    const latest = messages.filter(message => message.role === 'user').at(-1);
    const text = typeof latest?.content === 'string' ? latest.content : JSON.stringify(latest?.content ?? '');
    const match = text.match(/UAR_NATIVE_CHAT:(native-chat-[A-Za-z0-9_-]+):([a-f0-9-]{36}):(success|effect-preamble|effect|approval|partial-error|hold)/);
    // External model probing is visible in the journal, outside fixture baselines.
    if (!match) {
      record({ kind: 'provider', case_id: null, stage: 'unattributed', model: body.model, request_sha256: digest(raw) });
      return reply(200, { id: 'native-chat-provisioning', object: 'chat.completion', model: body.model,
        choices: [{ index: 0, message: { role: 'assistant', content: 'READY' }, finish_reason: 'stop' }],
        usage: { prompt_tokens: 1, completion_tokens: 1, total_tokens: 2 } });
    }
    const [, ownedNamespace, caseId, mode] = match;
    const key = `${namespace}:${caseId}`;
    const afterTool = messages.some(message => message.role === 'tool' && JSON.stringify(message.content).includes(key));
    const stage = ['effect', 'effect-preamble', 'approval'].includes(mode) ? afterTool ? 'after_tool' : 'tool_request' : mode;
    record({ kind: 'provider', case_id: caseId, mode, stage, model: body.model, request_sha256: digest(raw),
      image_parts: Array.isArray(latest?.content) ? latest.content.filter(part => part.type === 'image_url').length : 0 });
    if (ownedNamespace !== namespace) return reply(409, { error: 'synthetic_namespace_mismatch' });
    if (stage === 'tool_request' && !body.tools?.some(tool => tool.function?.name === toolName)) return reply(422, { error: 'governed_record_tool_not_declared' });
    assert.equal(body.stream, true, 'Mounted native fixture uses the actual streamed provider path');
    res.writeHead(200, { 'content-type': 'text/event-stream', 'cache-control': 'no-cache', connection: 'close' });
    res.flushHeaders();
    const frame = (delta, finish = null, usage) => res.write(`data: ${JSON.stringify({ id: `native-${caseId}`, object: 'chat.completion.chunk', model: body.model,
      choices: [{ index: 0, delta, finish_reason: finish }], ...(usage && { usage }) })}\n\n`);
    if (stage === 'tool_request') {
      if (mode === 'effect-preamble') frame({ role: 'assistant', content: `NATIVE-CHAT-PREAMBLE:${caseId}` });
      frame({ role: 'assistant', tool_calls: [{ index: 0, id: `record-${caseId}`, type: 'function', function: { name: toolName,
        arguments: JSON.stringify({ key, value: 'native-chat-admission-value' }) } }] });
      frame({}, 'tool_calls', { prompt_tokens: 11, completion_tokens: 3, total_tokens: 14 });
      return res.end('data: [DONE]\n\n');
    }
    frame({ role: 'assistant', content: ['partial-error', 'hold'].includes(mode) ? 'synthetic native partial' : `NATIVE-CHAT-OK:${caseId}` });
    if (mode === 'partial-error') return res.end();
    if (mode === 'hold') {
      record({ kind: 'partial_ready', case_id: caseId });
      const timer = setTimeout(() => res.destroy(), 45_000);
      res.once('close', () => { clearTimeout(timer); record({ kind: 'held_closed', case_id: caseId }); });
      return;
    }
    frame({}, 'stop', { prompt_tokens: 11, completion_tokens: 3, total_tokens: 14 });
    res.end('data: [DONE]\n\n');
  } catch {
    if (!res.headersSent) reply(400, { error: 'synthetic_request_invalid' });
    else res.destroy();
  }
});
server.listen(port, '127.0.0.1');
for (const signal of ['SIGINT', 'SIGTERM']) process.once(signal, () => server.close(() => { closeSync(fd); process.exitCode = 0; }));
