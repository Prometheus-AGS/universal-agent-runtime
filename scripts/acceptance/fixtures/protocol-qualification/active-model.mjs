// Owned deterministic provider peer for active-cancellation and admission races.
// External host provisions this endpoint as a selected candidate provider. The
// fixture never installs credentials or claims real-model/accounting accuracy.
import { createServer } from 'node:http';
import { createHash, randomUUID } from 'node:crypto';
const hash = text => createHash('sha256').update(text).digest('hex');
const messageText = message => typeof message.content === 'string' ? message.content : (message.content ?? []).map(part => part.text ?? '').join('');
const observations = [];
const server = createServer((request, response) => {
  if (request.method === 'GET' && request.url === '/observations') {
    response.writeHead(200, { 'Content-Type': 'application/json' }).end(JSON.stringify(observations)); return;
  }
  if (request.method === 'GET' && request.url === '/v1/models') {
    response.writeHead(200, { 'Content-Type': 'application/json' }).end(JSON.stringify({ object: 'list', data: [] })); return;
  }
  if (request.method !== 'POST' || request.url !== '/v1/chat/completions') { response.writeHead(404).end(); return; }
  let body = '';
  request.setEncoding('utf8'); request.on('data', chunk => { body += chunk; });
  request.on('end', () => {
    let payload;
    try { payload = JSON.parse(body); } catch { response.writeHead(400).end(); return; }
    const latest = [...(payload.messages ?? [])].reverse().find(message => message.role === 'user')?.content;
    const text = typeof latest === 'string' ? latest : (latest ?? []).map(part => part.text ?? '').join('');
    const marker = /PR476-[a-zA-Z0-9-]+/.exec(text)?.[0];
    if (!marker) { response.writeHead(400).end(); return; }
    const row = { requestId: randomUUID(), marker, model: payload.model, stream: payload.stream === true,
      messageSha256: (payload.messages ?? []).map(message => hash(messageText(message))),
      arrived_at: Date.now(), chunks: 0, closed: false, ended: false };
    observations.push(row);
    response.on('close', () => { row.closed = true; row.closed_at = Date.now(); });
    const requestedTool = text.includes('PR476-MCP-WAIT-') ? 'qualification_wait' : text.includes('PR476-MCP-ERROR-') ? 'qualification_fail'
      : text.includes('Use qualification_echo') ? 'qualification_echo' : null;
    const hasToolResult = (payload.messages ?? []).some(message => message.role === 'tool');
    if (requestedTool && !hasToolResult) {
      const declared = (payload.tools ?? []).find(tool => tool.function?.name?.endsWith(`__${requestedTool}`));
      if (!declared) { row.ended = true; response.writeHead(400).end(); return; }
      const call = { id: `call-${marker}`, type: 'function', function: { name: declared.function.name, arguments: JSON.stringify({ marker }) } };
      row.tool = declared.function.name;
      row.ended = true;
      if (!payload.stream) {
        response.writeHead(200, { 'Content-Type': 'application/json' }).end(JSON.stringify({ id: row.requestId, object: 'chat.completion', model: payload.model,
          choices: [{ index: 0, message: { role: 'assistant', content: null, tool_calls: [call] }, finish_reason: 'tool_calls' }] }));
      } else {
        response.writeHead(200, { 'Content-Type': 'text/event-stream', 'Cache-Control': 'no-cache' });
        const envelope = { id: row.requestId, object: 'chat.completion.chunk', model: payload.model };
        response.end(`data: ${JSON.stringify({ ...envelope, choices: [{ index: 0, delta: { tool_calls: [{ index: 0, ...call }] }, finish_reason: null }] })}\n\ndata: ${JSON.stringify({ ...envelope, choices: [{ index: 0, delta: {}, finish_reason: 'tool_calls' }] })}\n\ndata: [DONE]\n\n`);
      }
      return;
    }
    const retained = (payload.messages ?? []).filter(message => message.role === 'user').map(messageText)
      .filter(value => value.startsWith('PR476-HISTORY-'));
    const answer = text.startsWith('PR476-HISTORY-') ? retained.join('\n')
      : requestedTool && hasToolResult ? requestedTool === 'qualification_echo' ? 'The requested tool response has been received.' : 'The MCP tool reported failure; no effect was performed.'
      : text.includes('PR476-ACTIVE-CANCEL') ? 'PR476 active controlled response has started.' : marker;
    if (!payload.stream) {
      row.ended = true;
      response.writeHead(200, { 'Content-Type': 'application/json' }).end(JSON.stringify({ id: row.requestId, object: 'chat.completion', model: payload.model,
        choices: [{ index: 0, message: { role: 'assistant', content: answer }, finish_reason: 'stop' }] })); return;
    }
    response.writeHead(200, { 'Content-Type': 'text/event-stream', 'Cache-Control': 'no-cache' });
    const delta = { id: row.requestId, object: 'chat.completion.chunk', model: payload.model,
      choices: [{ index: 0, delta: { content: answer }, finish_reason: null }] };
    response.write(`data: ${JSON.stringify(delta)}\n\n`); row.chunks++;
    if (text.includes('PR476-INTERRUPTED-')) {
      // A real transport interruption after a semantic chunk, without a stop
      // envelope or [DONE]. Never turn this fault into an ordinary completion.
      row.interrupted = true;
      const cut = setTimeout(() => response.destroy(), 150);
      response.on('close', () => clearTimeout(cut));
      return;
    }
    const finish = () => {
      row.ended = true;
      response.end(`data: ${JSON.stringify({ ...delta, choices: [{ index: 0, delta: {}, finish_reason: 'stop' }] })}\n\ndata: [DONE]\n\n`);
    };
    if (!text.includes('PR476-ACTIVE-CANCEL')) { finish(); return; }
    // A finite live stream exposes a real cancellable request. Expiry produces
    // normal completion, which the cancellation assertion must reject.
    const expiry = setTimeout(finish, 45_000);
    const heartbeat = setInterval(() => response.write(': controlled active stream\n\n'), 1000);
    response.on('close', () => { clearTimeout(expiry); clearInterval(heartbeat); });
  });
});
server.listen(0, '127.0.0.1', () => process.stdout.write(`MODEL_URL http://127.0.0.1:${server.address().port}\n`));
