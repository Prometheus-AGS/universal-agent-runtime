#!/usr/bin/env node
// Owned provider fault proxy for manual BDD scenarios. Forwards every request to
// UPSTREAM (an OpenAI-compatible gateway) unchanged. While the fault is armed it
// relays the first response bytes, then destroys the client connection mid-stream.
// POST /__fault/arm?mode=unavailable instead answers every request with a 503 before
// contacting UPSTREAM, so the provider is down without any partial response.
// Control (loopback only): POST /__fault/arm, POST /__fault/clear, GET /__fault/log.
// The log records method, path, status, mode and byte counts; never headers or bodies.
import { createServer, request as forward } from 'node:http';

const UPSTREAM = new URL(process.env.UPSTREAM ?? 'http://127.0.0.1:4000');
const FAULT_AFTER_BYTES = Number(process.env.FAULT_AFTER_BYTES ?? 64);
let armed = false;
let armedMode = 'disconnect_mid_stream';
const log = [];

const server = createServer((req, res) => {
  if (req.url.startsWith('/__fault/')) {
    const control = new URL(req.url, 'http://fault.invalid');
    if (req.method === 'POST' && control.pathname === '/__fault/arm') {
      armed = true;
      armedMode = control.searchParams.get('mode') === 'unavailable' ? 'unavailable' : 'disconnect_mid_stream';
    } else if (req.method === 'POST' && control.pathname === '/__fault/clear') armed = false;
    else if (!(req.method === 'GET' && control.pathname === '/__fault/log')) { res.writeHead(404).end(); return; }
    res.writeHead(200, { 'Content-Type': 'application/json' }).end(JSON.stringify({ armed, log }));
    return;
  }
  const entry = { at: new Date().toISOString(), method: req.method, path: req.url, mode: armed ? armedMode : 'pass', status: null, bytes: 0, outcome: 'open' };
  log.push(entry);
  if (entry.mode === 'unavailable') {
    entry.status = 503;
    entry.outcome = 'refused';
    req.resume();
    res.writeHead(503, { 'Content-Type': 'application/json' }).end(JSON.stringify({ error: { message: 'provider unavailable', type: 'fault_proxy_unavailable' } }));
    return;
  }
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
