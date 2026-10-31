// Supplementary focused-fixture ownership probe; no product or model acceptance.
// Uses the exact committed helper's real 100-second deadline, without a patch.
import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { createHash, randomUUID } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beginCase, finishIO, isolationState, peerRequest } from '../../../../scripts/behaviors/support/capability-fixture.mjs';

const root = new URL('../../../../', import.meta.url);
const output = new URL(`target/production-readiness-476/fixture-lifetime-${randomUUID()}/`, root);
mkdirSync(output, { recursive: false });
const timers = new Set();
const sockets = new Set();
let arrivals = 0;
const server = createServer((request, response) => {
  arrivals++;
  const timer = setTimeout(() => {
    timers.delete(timer);
    if (!response.destroyed) { response.writeHead(200, { 'Content-Type': 'application/json' }); response.end('{}'); }
  }, 10_000);
  timers.add(timer);
});
server.on('connection', socket => { sockets.add(socket); socket.on('close', () => sockets.delete(socket)); });
const receipt = { kind: 'supplementary-focused-fixture-lifetime', status: 'FAIL', product_acceptance: false,
  helper_sha256: createHash('sha256').update(readFileSync(new URL('scripts/behaviors/support/capability-fixture.mjs', root))).digest('hex') };
const world = {};
const started = performance.now();
try {
  await new Promise((resolve, reject) => { server.once('error', reject); server.listen(0, '127.0.0.1', resolve); });
  const peer = { url: new URL(`http://127.0.0.1:${server.address().port}/`) };
  beginCase(world);
  let rejected = false;
  try {
    for (let index = 0; index < 12; index++) await peerRequest(world, peer, '/slow-status');
  } catch { rejected = true; }
  await finishIO(world.caseIO);
  const elapsed = performance.now() - started;
  assert.equal(rejected, true, 'Real shared deadline must stop the serial requests');
  assert.ok(elapsed >= 95_000 && elapsed < 175_000, 'Internal deadline must settle before the 180-second outer wrapper');
  assert.equal(world.caseIO.pending.size, 0, 'All client requests must settle before cleanup/next case');
  assert.equal(isolationState().poisoned, true, 'Uncertain remote outcome must poison the focused run');
  assert.throws(() => beginCase({}), { name: 'AssertionError' });
  const stoppedAt = arrivals;
  await new Promise(resolve => setTimeout(resolve, 500));
  assert.equal(arrivals, stoppedAt, 'A refused subsequent case cannot mutate the peer');
  receipt.status = 'PASS';
  receipt.elapsed_ms = elapsed;
  receipt.actual_arrivals = arrivals;
  receipt.client_pending = world.caseIO.pending.size;
  receipt.poisoned_next_case = 'REFUSED';
  receipt.server_cancellation = 'NOT_CLAIMED';
} finally {
  await finishIO(world.caseIO);
  for (const timer of timers) clearTimeout(timer);
  for (const socket of sockets) socket.destroy();
  await new Promise(resolve => server.close(resolve));
  receipt.receipt = fileURLToPath(new URL('result.json', output));
  writeFileSync(new URL('result.json', output), JSON.stringify(receipt, null, 2) + '\n', { flag: 'wx' });
  process.stdout.write(JSON.stringify(receipt) + '\n');
}
if (receipt.status !== 'PASS') process.exitCode = 1;
