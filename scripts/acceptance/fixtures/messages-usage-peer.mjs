import { createServer } from 'node:http';

// External provisioning owns this keyless local provider. Importing the module
// starts nothing. The peer never reads headers or retains prompt/tool-result text.
export function startMessagesUsagePeer(namespace) {
  if (!/^[A-Za-z0-9_-]{1,96}$/.test(namespace)) throw new Error('Invalid messages fixture namespace');
  const loopModes = new Set(['loop-missing-known', 'loop-known-missing', 'loop-known', 'loop-zero', 'loop-missing-read', 'loop-missing-creation']);
  const modes = new Set(['nonzero', 'zero', 'absent', 'uncached-only', 'tool', 'before-output', 'mid-output', 'tool-handoff', ...loopModes]);
  let mode = 'nonzero';
  let loopStep = 0;
  let pendingLoop;
  let arrivals = 0;
  let closed = 0;
  let completed = 0;
  const requests = [];
  const active = new Set();
  const held = new Set();
  const frame = (res, type, data) => {
    if (!res.destroyed) res.write(`event: ${type}\ndata: ${JSON.stringify({ type, ...data })}\n\n`);
  };
  const server = createServer((req, res) => {
    const reply = (status, data) => {
      res.writeHead(status, { 'Content-Type': 'application/json' }); res.end(JSON.stringify(data));
    };
    if (req.method === 'GET' && req.url === '/__messages/state') {
      reply(200, { namespace, arrivals, closed, completed, active: active.size, held: held.size, overflowed: arrivals > 128, requests });
      return;
    }
    // Count every provider-side arrival before parsing/size/path refusal; a
    // rejected wire request must never masquerade as zero provider sends.
    const arrivalId = req.url.startsWith('/__messages/') ? null : ++arrivals;
    if (arrivals > 128) { reply(503, { error: 'fixture_arrival_capacity' }); return; }
    let body = '';
    req.on('data', chunk => { body += chunk; if (body.length > 1024 * 1024) req.destroy(); });
    req.on('end', () => {
      let request;
      try { request = JSON.parse(body); } catch { reply(400, { error: 'fixture_invalid_json' }); return; }
      if (req.url === '/__messages/control') {
        if (req.method !== 'POST' || request.namespace !== namespace || !modes.has(request.mode)
          || Object.keys(request).some(key => !['namespace', 'mode'].includes(key)) || active.size) {
          reply(409, { error: 'fixture_control_refused' }); return;
        }
        mode = request.mode; loopStep = 0; pendingLoop = undefined;
        reply(200, { configured: true }); return;
      }
      if (req.url === '/__messages/release') {
        if (request.namespace !== namespace || req.method !== 'POST') { reply(409, { error: 'fixture_namespace_mismatch' }); return; }
        for (const finish of [...held]) finish();
        reply(200, { released: true }); return;
      }
      if (!['/messages', '/v1/messages'].includes(req.url) || req.method !== 'POST') {
        reply(404, { error: 'fixture_provider_path_unknown' }); return;
      }
      const selectedMode = mode;
      const loop = loopModes.has(selectedMode);
      const step = loop ? loopStep + 1 : null;
      const tool = loop ? step === 1 : ['tool', 'tool-handoff'].includes(selectedMode);
      const toolName = loop ? 'native_echo' : 'lookup_code';
      const toolInput = loop ? { message: `${namespace}-governed-echo` } : { key: 'alpha' };
      const toolId = loop && step === 2 ? pendingLoop?.toolId : `${namespace}-tool-${arrivalId}`;
      if ((tool || loop) && !request.tools?.some(item => item.name === toolName)) {
        reply(422, { error: 'fixture_declared_caller_tool_missing' }); return;
      }
      let toolResultMatched = false;
      if (loop) {
        if (step > 2) { reply(409, { error: 'fixture_loop_extra_call' }); return; }
        const blocks = request.messages?.flatMap(message => Array.isArray(message.content) ? message.content : []) ?? [];
        const results = blocks.filter(block => block.type === 'tool_result');
        if (step === 1 && results.length) { reply(409, { error: 'fixture_loop_initial_result' }); return; }
        if (step === 2) {
          const calls = blocks.filter(block => block.type === 'tool_use' && block.id === pendingLoop?.toolId);
          const result = results.find(block => block.tool_use_id === pendingLoop?.toolId);
          let value;
          try { value = JSON.parse(result?.content); } catch { value = null; }
          toolResultMatched = calls.length === 1 && calls[0].name === toolName
            && JSON.stringify(calls[0].input) === JSON.stringify(toolInput)
            && results.length === 1 && result?.is_error !== true
            && JSON.stringify(value) === JSON.stringify({ echoed: toolInput });
          if (!toolResultMatched) { reply(409, { error: 'fixture_loop_real_result_missing' }); return; }
        }
        loopStep = step;
        if (step === 1) pendingLoop = { toolId };
      }
      const counters = selectedMode === 'zero' || selectedMode === 'loop-zero'
        ? { input_tokens: 0, output_tokens: 0, cache_read_input_tokens: 0, cache_creation_input_tokens: 0 }
        : selectedMode === 'uncached-only' ? { input_tokens: 7, output_tokens: 5 }
          : loop && step === 2 ? { input_tokens: 11, output_tokens: 13, cache_read_input_tokens: 17, cache_creation_input_tokens: 19 }
          : { input_tokens: 7, output_tokens: 5, cache_read_input_tokens: 3, cache_creation_input_tokens: 2 };
      if (selectedMode === 'loop-missing-read' && step === 1) delete counters.cache_read_input_tokens;
      if (selectedMode === 'loop-missing-creation' && step === 1) delete counters.cache_creation_input_tokens;
      const reported = selectedMode !== 'absent'
        && !(selectedMode === 'loop-missing-known' && step === 1)
        && !(selectedMode === 'loop-known-missing' && step === 2);
      const id = arrivalId;
      requests.push({ id, model: request.model, mode: selectedMode, stream: request.stream === true, tool, usage: reported ? counters : null,
        ...(loop && { loopStep: step, toolName, toolId, toolDeclared: true, toolResultMatched }) });
      active.add(res);
      let done = false;
      let timer;
      const finish = () => {
        held.delete(finish);
        if (done || res.destroyed) return;
        if (request.stream === true) {
          frame(res, 'content_block_stop', { index: 0 });
          frame(res, 'message_delta', { delta: { stop_reason: tool ? 'tool_use' : 'end_turn', stop_sequence: null },
            ...(reported && { usage: { output_tokens: counters.output_tokens } }) });
          frame(res, 'message_stop', {});
          res.end();
        }
        done = true; completed++;
      };
      res.once('close', () => { clearTimeout(timer); active.delete(res); held.delete(finish); closed++; });
      timer = setTimeout(() => { held.delete(finish); res.destroy(); }, 25_000);
      if (request.stream !== true) {
        const content = tool ? [{ type: 'tool_use', id: toolId, name: toolName, input: toolInput }]
          : [{ type: 'text', text: 'MESSAGES-USAGE-OK' }];
        reply(200, { id: `${namespace}-message-${id}`, type: 'message', role: 'assistant', model: request.model,
          content, stop_reason: tool ? 'tool_use' : 'end_turn', stop_sequence: null, ...(reported && { usage: counters }) });
        done = true; completed++; return;
      }
      res.writeHead(200, { 'Content-Type': 'text/event-stream' });
      res.flushHeaders();
      if (selectedMode === 'before-output') {
        held.add(finish); return;
      }
      frame(res, 'message_start', { message: { id: `${namespace}-message-${id}`, type: 'message', role: 'assistant',
        model: request.model, content: [], ...(reported && { usage: { ...counters, output_tokens: 0 } }) } });
      if (tool) {
        frame(res, 'content_block_start', { index: 0, content_block: { type: 'tool_use', id: toolId, name: toolName, input: {} } });
        frame(res, 'content_block_delta', { index: 0, delta: { type: 'input_json_delta', partial_json: JSON.stringify(toolInput) } });
      } else {
        frame(res, 'content_block_start', { index: 0, content_block: { type: 'text', text: '' } });
        frame(res, 'content_block_delta', { index: 0, delta: { type: 'text_delta', text: 'MESSAGES-USAGE-OK' } });
      }
      if (['mid-output', 'tool-handoff'].includes(selectedMode)) held.add(finish);
      else finish();
    });
  });
  return new Promise(resolve => server.listen(0, '127.0.0.1', () => resolve(server)));
}
