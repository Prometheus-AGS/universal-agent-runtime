// Node 24 stdio peer. Receipts are synthetic fixture evidence, never application logs.
import { appendFileSync, mkdirSync } from 'node:fs';
import { join } from 'node:path';
import { createInterface } from 'node:readline';

const [directory, mode = 'normal'] = process.argv.slice(2);
mkdirSync(directory, { recursive: true });
const record = (event, fields = {}) => appendFileSync(join(directory, 'receipts.jsonl'),
  `${JSON.stringify({ event, pid: process.pid, ...fields })}\n`);
record('started', { environment: process.env });
process.stderr.write('BAUAR-RAW-CHILD-STDERR-CANARY\n');
const heartbeat = setInterval(() => record('heartbeat'), 30);
const lines = createInterface({ input: process.stdin, crlfDelay: Infinity });
const reply = (id, result) => process.stdout.write(`${JSON.stringify({ jsonrpc: '2.0', id, result })}\n`);
lines.on('line', (line) => {
  let request;
  try { request = JSON.parse(line); }
  catch { record('invalid-stdin', { line }); return; }
  if (request.method === 'initialize') {
    record('initialize');
    if (mode === 'hold-initialize') return;
    reply(request.id, { protocolVersion: request.params.protocolVersion,
      capabilities: { tools: {} }, serverInfo: { name: 'bauar-stdio', version: '1' } });
  } else if (request.method === 'tools/list') {
    reply(request.id, { tools: [{ name: 'effect', description: 'Synthetic boundary effect',
      inputSchema: { type: 'object', properties: { action: { type: 'string' } }, required: ['action'] } }] });
  } else if (request.method === 'tools/call') {
    const action = request.params.arguments.action;
    record('effect', { action });
    if (action === 'hold') return;
    reply(request.id, { content: [{ type: 'text', text: `effect:${action}` }], isError: false });
    if (action === 'exit') setTimeout(() => process.exit(0), 20);
  }
});
lines.on('close', () => {
  record('eof');
  if (mode === 'ignore-eof' || mode === 'hold-initialize') return;
  clearInterval(heartbeat);
  process.exit(0);
});
