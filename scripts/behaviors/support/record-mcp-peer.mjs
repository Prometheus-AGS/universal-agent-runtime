#!/usr/bin/env node
// Owned MCP peer for manual BDD scenarios. JSON-RPC 2.0 over newline-delimited stdio,
// or over loopback streamable HTTP (POST /mcp, JSON responses) when PEER_HTTP=1.
// Tools: record_write (stores one value under a key) and record_read (returns it).
// State lives in RECORD_FILE; every tools/call is appended to RECORD_LOG.
import { appendFileSync, existsSync, readFileSync, writeFileSync } from 'node:fs';
import { createServer } from 'node:http';
import { randomUUID } from 'node:crypto';
import { createInterface } from 'node:readline';

const RECORD_FILE = process.env.RECORD_FILE;
const RECORD_LOG = process.env.RECORD_LOG;
if (!RECORD_FILE || !RECORD_LOG) {
  process.stderr.write('RECORD_FILE and RECORD_LOG are required\n');
  process.exit(2);
}

const TOOLS = [
  { name: 'record_write', description: 'Store a value under a key in the owned record store. Returns the stored revision.',
    inputSchema: { type: 'object', properties: { key: { type: 'string', minLength: 1 }, value: { type: 'string', minLength: 1 } },
      required: ['key', 'value'], additionalProperties: false } },
  { name: 'record_read', description: 'Read the value stored under a key in the owned record store, with its revision.',
    inputSchema: { type: 'object', properties: { key: { type: 'string', minLength: 1 } }, required: ['key'], additionalProperties: false } },
];

const load = () => (existsSync(RECORD_FILE) ? JSON.parse(readFileSync(RECORD_FILE, 'utf8')) : {});

function call(name, args) {
  const records = load();
  if (name === 'record_write') {
    const revision = (records[args.key]?.revision ?? 0) + 1;
    records[args.key] = { value: args.value, revision };
    writeFileSync(RECORD_FILE, JSON.stringify(records, null, 2));
    return { stored: true, key: args.key, revision };
  }
  if (name === 'record_read') {
    const record = records[args.key];
    return record ? { found: true, key: args.key, value: record.value, revision: record.revision } : { found: false, key: args.key };
  }
  throw new Error(`unknown tool ${name}`);
}

const ok = (id, result) => ({ jsonrpc: '2.0', id, result });

// Returns the JSON-RPC response for a request, or null for a notification.
function handle(message) {
  const { id, method, params } = message;
  if (id === undefined || id === null) return null;
  if (method === 'initialize') {
    return ok(id, { protocolVersion: params?.protocolVersion ?? '2025-06-18', capabilities: { tools: {} },
      serverInfo: { name: 'owned-record-peer', version: '1.0.0' } });
  }
  if (method === 'ping') return ok(id, {});
  if (method === 'tools/list') return ok(id, { tools: TOOLS });
  if (method === 'tools/call') {
    const entry = { at: new Date().toISOString(), tool: params?.name, arguments: params?.arguments ?? {} };
    try {
      const result = call(params?.name, params?.arguments ?? {});
      appendFileSync(RECORD_LOG, JSON.stringify({ ...entry, result }) + '\n');
      return ok(id, { content: [{ type: 'text', text: JSON.stringify(result) }], isError: false });
    } catch (error) {
      appendFileSync(RECORD_LOG, JSON.stringify({ ...entry, error: String(error.message) }) + '\n');
      return ok(id, { content: [{ type: 'text', text: String(error.message) }], isError: true });
    }
  }
  return { jsonrpc: '2.0', id, error: { code: -32601, message: `Method not found: ${method}` } };
}

if (process.env.PEER_HTTP === '1') {
  const session = randomUUID();
  const server = createServer((req, res) => {
    if (req.url !== '/mcp') { res.writeHead(404).end(); return; }
    if (req.method === 'DELETE') { res.writeHead(200).end(); return; }
    if (req.method !== 'POST') { res.writeHead(405, { Allow: 'POST, DELETE' }).end(); return; }
    let body = '';
    req.setEncoding('utf8');
    req.on('data', chunk => { body += chunk; });
    req.on('end', () => {
      const parsed = JSON.parse(body);
      const responses = (Array.isArray(parsed) ? parsed : [parsed]).map(handle).filter(Boolean);
      if (responses.length === 0) { res.writeHead(202, { 'Mcp-Session-Id': session }).end(); return; }
      const payload = Array.isArray(parsed) ? responses : responses[0];
      res.writeHead(200, { 'Content-Type': 'application/json', 'Mcp-Session-Id': session }).end(JSON.stringify(payload));
    });
  });
  server.listen(0, '127.0.0.1', () => process.stdout.write(`PEER_URL http://127.0.0.1:${server.address().port}/mcp\n`));
} else {
  createInterface({ input: process.stdin }).on('line', line => {
    if (!line.trim()) return;
    const response = handle(JSON.parse(line));
    if (response) process.stdout.write(JSON.stringify(response) + '\n');
  });
}
