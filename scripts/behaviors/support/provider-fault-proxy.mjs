#!/usr/bin/env node
// Owned provider fault proxy for manual BDD scenarios. Forwards every request to
// UPSTREAM (an OpenAI-compatible gateway) unchanged. While the fault is armed it
// relays the first response bytes, then destroys the client connection mid-stream.
// Control (loopback only): POST /__fault/arm, POST /__fault/clear, GET /__fault/log.
// The log records method, path, status, mode and byte counts; never headers or bodies.
import { createServer, request as forward } from 'node:http';

const UPSTREAM = new URL(process.env.UPSTREAM ?? 'http://127.0.0.1:4000');
const FAULT_AFTER_BYTES = Number(process.env.FAULT_AFTER_BYTES ?? 64);
let armed = false;
const log = [];

const server = createServer((req, res) => {
  if (req.url.startsWith('/__fault/')) {
    if (req.method === 'POST' && req.url === '/__fault/arm') armed = true;
    else if (req.method === 'POST' && req.url === '/__fault/clear') armed = false;
    else if (!(req.method === 'GET' && req.url === '/__fault/log')) { res.writeHead(404).end(); return; }
    res.writeHead(200, { 'Content-Type': 'application/json' }).end(JSON.stringify({ armed, log }));
    return;
  }
  const entry = { at: new Date().toISOString(), method: req.method, path: req.url, mode: armed ? 'disconnect_mid_stream' : 'pass', status: null, bytes: 0, outcome: 'open' };
  log.push(entry);
  const headers = { ...req.headers, host: UPSTREAM.host };
  const upstream = forward({ hostname: UPSTREAM.hostname, port: UPSTREAM.port, path: req.url, method: req.method, headers }, response => {
    entry.status = response.statusCode;
    res.writeHead(response.statusCode, response.headers);
    const faulting = entry.mode === 'disconnect_mid_stream';
    response.on('data', chunk => {
      entry.bytes += chunk.length;
      res.write(chunk);
      if (faulting && entry.bytes >= FAULT_AFTER_BYTES) {
        entry.outcome = 'disconnected';
        response.destroy();
        res.destroy();
      }
    });
    response.on('end', () => { if (entry.outcome === 'open') { entry.outcome = 'complete'; res.end(); } });
    response.on('error', () => { if (entry.outcome === 'open') entry.outcome = 'upstream_error'; });
  });
  upstream.on('error', () => { if (entry.outcome === 'open') { entry.outcome = 'upstream_error'; res.destroy(); } });
  req.pipe(upstream);
});

server.listen(0, '127.0.0.1', () => process.stdout.write(`PROXY_URL http://127.0.0.1:${server.address().port}\n`));
