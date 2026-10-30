import { createHash } from 'node:crypto';
import { createServer } from 'node:http';

/**
 * Deterministic OpenAI-compatible embeddings server (also the Mistral wire
 * shape). Equal text gives an equal unit vector of `dimension` values. Each
 * request's model and input count is recorded in `server.requests`; the
 * request's Authorization header is never read or recorded.
 */
export function startEmbeddingStub(dimension) {
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
  const server = createServer((req, res) => {
    let body = '';
    req.on('data', chunk => { body += chunk; });
    req.on('end', () => {
      const parsed = JSON.parse(body || '{}');
      const input = parsed.input ?? [];
      const texts = Array.isArray(input) ? input : [input];
      requests.push({ path: req.url, model: parsed.model, inputs: texts.length });
      res.writeHead(200, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify({ object: 'list', model: parsed.model,
        data: texts.map((text, index) => ({ object: 'embedding', index, embedding: vector(String(text)) })),
        usage: { prompt_tokens: 0, total_tokens: 0 } }));
    });
  });
  server.requests = requests;
  return new Promise(resolveServer => server.listen(0, '127.0.0.1', () => resolveServer(server)));
}
