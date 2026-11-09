import { createHash } from 'node:crypto';
import { createServer } from 'node:http';

/**
 * Deterministic OpenAI-compatible embeddings server (also the Mistral wire
 * shape). Equal text gives an equal unit vector of `dimension` values. Each
 * request's model and input count is recorded in `server.requests`; the
 * request's Authorization header is never read or recorded.
 */
export function startEmbeddingStub(dimension, options = {}) {
  // Explicit fixture modes, never inferred from caller text/model. Defaults
  // preserve the existing runtime-configuration fixture's measured-zero reply.
  const allowedModes = new Set(['zero', 'nonzero', 'absent', 'prompt-only', 'total-only',
    'malformed-usage', 'duplicate-index', 'missing-index', 'out-of-range-index',
    'incomplete', 'bad-dimension', 'provider-error']);
  if (Object.keys(options).some(key => !['mode', 'holdResponses', 'namespace'].includes(key))) {
    throw new Error('Unknown embedding fixture option');
  }
  let modeSpec = options.mode ?? 'zero';
  let modes = Array.isArray(modeSpec) ? modeSpec : [modeSpec];
  if (!modes.length || modes.some(mode => !allowedModes.has(mode))) {
    throw new Error('Unknown embedding fixture mode');
  }
  if (options.holdResponses !== undefined && typeof options.holdResponses !== 'boolean') {
    throw new Error('holdResponses must be boolean');
  }
  if (options.namespace !== undefined && !/^[A-Za-z0-9_-]{1,96}$/.test(options.namespace)) {
    throw new Error('Invalid embedding fixture namespace');
  }
  const vector = text => {
    const values = [];
    for (let block = 0; values.length < dimension; block++) {
      const digest = createHash('sha256').update(`${block}:${text}`).digest();
      for (const byte of digest) values.push(byte / 255 - 0.5);
    }
    const unit = values.slice(0, dimension);
    const norm = Math.hypot(...unit) || 1;
    return unit.map(v => v / norm);
  };
  const requests = [];
  let arrivals = 0;
  const heldResponses = new Set();
  let holdResponses = options.holdResponses ?? false;
  let sequenceIndex = 0;
  const releaseResponses = () => {
    holdResponses = false;
    for (const send of heldResponses) send();
    heldResponses.clear();
  };
  const server = createServer((req, res) => {
    if (req.url.startsWith('/__embedding/')) {
      const reply = (status, data) => {
        res.writeHead(status, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify(data));
      };
      if (!options.namespace) { reply(404, { error: 'fixture_controls_disabled' }); return; }
      if (req.method === 'GET' && req.url === '/__embedding/state') {
        reply(200, { namespace: options.namespace, arrivals, overflowed: arrivals > 256, requests, held: heldResponses.size });
        return;
      }
      if (req.method !== 'POST') { reply(404, { error: 'fixture_control_unknown' }); return; }
      let controlBody = '';
      req.on('data', chunk => { controlBody += chunk; if (controlBody.length > 4096) req.destroy(); });
      req.on('end', () => {
        let control;
        try { control = JSON.parse(controlBody); } catch { reply(400, { error: 'fixture_control_invalid' }); return; }
        if (control.namespace !== options.namespace) { reply(409, { error: 'fixture_namespace_mismatch' }); return; }
        if (req.url === '/__embedding/release') { releaseResponses(); reply(200, { released: true }); return; }
        const next = Array.isArray(control.mode) ? control.mode : [control.mode];
        if (req.url !== '/__embedding/control' || Object.keys(control).some(key => !['namespace', 'mode', 'holdResponses'].includes(key))
          || !next.length || next.some(mode => !allowedModes.has(mode)) || typeof control.holdResponses !== 'boolean'
          || heldResponses.size) {
          reply(409, { error: 'fixture_control_refused' }); return;
        }
        modeSpec = control.mode;
        modes = next;
        sequenceIndex = 0;
        holdResponses = control.holdResponses;
        reply(200, { configured: true });
      });
      return;
    }
    arrivals += 1;
    if (arrivals > 256) {
      res.writeHead(503, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify({ error: 'fixture_arrival_capacity' }));
      return;
    }
    let body = '';
    req.on('data', chunk => { body += chunk; });
    req.on('end', () => {
      let parsed;
      try { parsed = JSON.parse(body || '{}'); } catch {
        res.writeHead(400, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ error: 'fixture_invalid_json' }));
        return;
      }
      const input = parsed.input ?? [];
      const texts = Array.isArray(input) ? input : [input];
      requests.push({ path: req.url, model: parsed.model, inputs: texts.length });
      server.emit('embedding-request', requests.at(-1));
      const mode = Array.isArray(modeSpec) ? modes[sequenceIndex++] : modes[0];
      const send = () => {
        if (res.destroyed) return;
        if (mode === undefined || mode === 'provider-error') {
          res.writeHead(mode === undefined ? 500 : 502, { 'Content-Type': 'application/json' });
          res.end(JSON.stringify({ error: mode === undefined ? 'fixture_sequence_exhausted' : 'synthetic-provider-credential-body' }));
          return;
        }
        const data = texts.map((text, index) => ({ object: 'embedding', index,
          embedding: vector(options.namespace ? `${parsed.model}:${String(text)}` : String(text)) }));
        if (mode === 'duplicate-index' && data.length > 1) data[1].index = data[0].index;
        if (mode === 'duplicate-index' && data.length < 2) {
          res.writeHead(500, { 'Content-Type': 'application/json' });
          res.end(JSON.stringify({ error: 'fixture_duplicate_requires_batch' }));
          return;
        }
        if (mode === 'missing-index' && data.length) delete data[0].index;
        if (mode === 'out-of-range-index' && data.length) data[0].index = data.length;
        if (mode === 'incomplete') data.pop();
        if (mode === 'bad-dimension' && data.length) data[0].embedding.push(0);
        const reply = { object: 'list', model: parsed.model, data };
        if (mode !== 'absent') reply.usage = { prompt_tokens: 0, total_tokens: 0 };
        if (mode === 'nonzero') reply.usage = { prompt_tokens: 7, total_tokens: 9 };
        if (mode === 'prompt-only') reply.usage = { prompt_tokens: 7 };
        if (mode === 'total-only') reply.usage = { total_tokens: 9 };
        if (mode === 'malformed-usage') reply.usage = { prompt_tokens: 'synthetic-provider-credential-body' };
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify(reply));
      };
      if (holdResponses) {
        heldResponses.add(send);
        res.once('close', () => heldResponses.delete(send));
      } else send();
    });
  });
  server.requests = requests;
  // A mounted-route swap case can hold an admitted request, publish new host
  // settings externally, then release only this owned peer's responses.
  server.releaseResponses = releaseResponses;
  return new Promise(resolveServer => server.listen(0, '127.0.0.1', () => resolveServer(server)));
}
