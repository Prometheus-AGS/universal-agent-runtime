// External fixture owner launches this synthetic peer from the SAME frozen
// manifest; the Cucumber feature never launches or reconfigures it.
import assert from 'node:assert/strict';
import { appendFileSync, readFileSync } from 'node:fs';
import { createServer } from 'node:http';
import { setTimeout as delay } from 'node:timers/promises';
import { validate, canonical, digest } from './runtime-measurement-source.mjs';

const manifestBytes = readFileSync(process.env.UAR_RUNTIME_MEASUREMENT_MANIFEST);
const manifest = validate(JSON.parse(manifestBytes.toString('utf8')));
const selectionSha256 = digest(manifestBytes);
const journalPath = manifest.peerJournal;
const tasks = new Map(manifest.workload.map(task => [task.id, task]));
const starts = new Map();
let sequence = 0, requests = 0;
function record(event, fields) {
  appendFileSync(journalPath, `${JSON.stringify({ kind: 'provider_observation', event, sequence: ++sequence,
    experimentId: manifest.selection.experimentId, selectionSha256, identity: manifest.identity,
    timeUnixMs: Date.now(), ...fields })}\n`, { mode: 0o600 });
}
const server = createServer(async (req, res) => {
  const receivedAt = performance.now(), receivedTimeUnixMs = Date.now();
  let requestId, callIndex, taskId, start;
  try {
    assert.ok(req.method === 'POST' && ['/v1/chat/completions', '/chat/completions'].includes(req.url));
    assert.ok(++requests <= manifest.limits.maxPeerRequests);
    const chunks = []; let bytes = 0;
    for await (const chunk of req) {
      bytes += chunk.length; assert.ok(bytes <= manifest.limits.maxRequestBytes); chunks.push(chunk);
    }
    const body = JSON.parse(Buffer.concat(chunks).toString('utf8'));
    assert.equal(body.stream, true); assert.equal(body.model, manifest.model.providerModelId);
    const messages = body.messages;
    assert.ok(Array.isArray(messages));
    const users = messages.filter(message => message.role === 'user' && typeof message.content === 'string');
    const last = users.at(-1)?.content ?? '';
    const match = /\nUAR_MEASUREMENT:([A-Za-z0-9_-]+):([A-Za-z0-9_-]+):([a-f0-9-]{36})$/.exec(last);
    assert.ok(match); assert.equal(match[1], manifest.selection.experimentId);
    taskId = match[2]; requestId = match[3];
    const task = tasks.get(taskId); assert.ok(task);
    assert.equal(last.slice(0, match.index), task.messages[0].content, 'Original selected synthetic input must reach provider');
    callIndex = (starts.get(requestId) ?? 0) + 1; starts.set(requestId, callIndex);
    start = receivedAt;
    record('request_received', { requestId, taskId, callIndex, timeUnixMs: receivedTimeUnixMs, requestBytes: bytes,
      requestSha256: digest(canonical(body)), selectedInputSha256: digest(task.messages[0].content),
      offeredToolCount: Array.isArray(body.tools) ? body.tools.length : 0, model: body.model });
    res.on('close', () => {
      if (!res.writableEnded) record('connection_closed', { requestId, taskId, callIndex, elapsedMs: performance.now() - start });
    });
    res.writeHead(200, { 'Content-Type': 'text/event-stream', 'Cache-Control': 'no-cache' });
    await delay(task.providerDelayMs);
    for (const [index, chunk] of task.responseChunks.entries()) {
      await delay(chunk.delayMs);
      if (res.destroyed) return;
      res.write(`data: ${JSON.stringify({ id: `synthetic-${requestId}`, object: 'chat.completion.chunk', model: body.model,
        choices: [{ index: 0, delta: { content: chunk.text }, finish_reason: null }] })}\n\n`);
      if (index === 0) record('first_semantic_sent', { requestId, taskId, callIndex, elapsedMs: performance.now() - start });
    }
    if (res.destroyed) return;
    res.end(`data: ${JSON.stringify({ id: `synthetic-${requestId}`, object: 'chat.completion.chunk', model: body.model,
      choices: [{ index: 0, delta: {}, finish_reason: 'stop' }], usage: task.reportedUsage })}\n\ndata: [DONE]\n\n`);
    record('response_completed', { requestId, taskId, callIndex, elapsedMs: performance.now() - start,
      responseSha256: digest(task.expectedText), reportedUsage: task.reportedUsage });
  } catch {
    record('fixture_refused', { requestId: requestId ?? null, taskId: taskId ?? null, callIndex: callIndex ?? null });
    if (!res.headersSent) res.writeHead(422, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ error: { code: 'selected_measurement_input_refused' } }));
  }
});
const bind = process.env.UAR_RUNTIME_MEASUREMENT_BIND ?? '127.0.0.1';
assert.ok(['127.0.0.1', '0.0.0.0'].includes(bind), 'All-interface mode needs externally isolated container ownership');
const port = Number(process.env.PORT);
assert.ok(Number.isSafeInteger(port) && port >= 0 && port <= 65535, 'External owner supplies explicit peer port');
server.listen(port, bind, () => {
  record('peer_started', {});
  console.log(`RUNTIME_MEASUREMENT_PEER_PORT=${server.address().port}`);
});
