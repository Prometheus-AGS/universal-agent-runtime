import assert from 'node:assert/strict';
import { test } from 'node:test';
import { Duplex } from 'node:stream';
import { EventEmitter } from 'node:events';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { SocketReader, exchange, newReport, sendPong, writeFrame,
  bindInterrupts, clearSocket, writeFdJson } from './collector.mjs';

class Peer extends Duplex {
  constructor(input = Buffer.alloc(0), behavior = () => 'ok') {
    super();
    this.input = input;
    this.behavior = behavior;
    this.sent = [];
    this.references = [];
    this.push(input.length ? input : Buffer.alloc(0));
  }
  _read() {}
  _write(chunk, encoding, callback) {
    this.references.push(chunk);
    this.sent.push(Buffer.from(chunk));
    const action = this.behavior(this.sent.length, callback);
    if (action === 'ok') callback();
    if (action === 'error') callback(new Error('synthetic'));
  }
}

function textFrame(value = { id: 1, result: 'synthetic-secret-token' }) {
  const payload = Buffer.from(JSON.stringify(value));
  const header = payload.length < 126 ? Buffer.from([0x81, payload.length])
    : Buffer.from([0x81, 126, payload.length >> 8, payload.length & 255]);
  return Buffer.concat([header, payload]);
}
const ping = () => Buffer.from([0x89, 0]);

async function run(input, options = {}) {
  const socket = new Peer(input, options.behavior);
  const reader = new SocketReader(socket);
  const report = newReport();
  const credentials = { user: 'synthetic-user', pass: 'synthetic-password' };
  let error;
  const deadline = performance.now() + (options.budget ?? 2000);
  try { await exchange(socket, reader, credentials, deadline, report); }
  catch (caught) { error = caught.message; }
  finally { clearSocket(socket); }
  assert.equal(credentials.user, '');
  assert.equal(credentials.pass, '');
  assert.ok(socket.references.every(buffer => buffer.every(byte => byte === 0)));
  assert.ok(report.counters.frameHeadersObserved <= 2);
  assert.ok(report.counters.pongWriteAttempts <= 1);
  assert.ok(report.counters.signinResponses <= 1);
  assert.ok(!JSON.stringify(report).includes('synthetic-secret-token'));
  return { socket, report, error };
}

test('direct Text with token: result category only; no Pong', async () => {
  const { report, socket, error } = await run(textFrame());
  assert.equal(error, undefined);
  assert.equal(report.state, 'signin_success');
  assert.equal(socket.sent.length, 1);
  assert.equal(report.counters.signinResponses, 1);
});

test('buffered Ping and Text: exact six-byte masked Pong then success', async () => {
  const input = Buffer.concat([ping(), textFrame()]);
  const { report, socket, error } = await run(input);
  assert.equal(error, undefined);
  assert.equal(report.state, 'signin_success');
  assert.equal(socket.sent.length, 2);
  assert.equal(socket.sent[1].length, 6);
  assert.deepEqual([...socket.sent[1].subarray(0, 2)], [0x8a, 0x80]);
  assert.equal(report.counters.frameHeadersObserved, 2);
  assert.ok(input.every(byte => byte === 0));
});

test('sign-in error is terminal and content-free', async () => {
  const { report, error } = await run(textFrame({ id: 1, error: { code: -1, message: 'synthetic-secret-token' } }));
  assert.equal(error, undefined);
  assert.equal(report.state, 'signin_error');
});

const rejected = [
  ['repeated Ping', () => Buffer.concat([ping(), ping()])],
  ['nonempty Ping', () => Buffer.from([0x89, 1, 42])],
  ['unexpected Pong', () => Buffer.from([0x8a, 0])],
  ['Close', () => Buffer.from([0x88, 0])],
  ['continuation', () => Buffer.from([0x80, 0])],
  ['fragmented Text', () => Buffer.from([0x01, 1, 42])],
  ['fragmented Ping', () => Buffer.from([0x09, 0])],
  ['Binary', () => Buffer.from([0x82, 1, 42])],
  ['reserved opcode', () => Buffer.from([0x8b, 0])],
  ['masked server frame', () => Buffer.from([0x81, 0x80, 1, 2, 3, 4])],
  ['RSV-marked frame', () => Buffer.from([0xc1, 1, 42])],
  ['nonminimal 16-bit length', () => Buffer.from([0x81, 126, 0, 1, 42])],
  ['nonminimal 64-bit length', () => Buffer.from([0x81, 127, 0, 0, 0, 0, 0, 0, 0, 1, 42])],
  ['invalid 64-bit high bit', () => Buffer.from([0x81, 127, 128, 0, 0, 0, 0, 0, 0, 0])],
];
for (const [name, build] of rejected) {
  test('stop: ' + name, async () => {
    const { report, error } = await run(build());
    assert.equal(error, 'unsupported_websocket_frame');
    assert.equal(report.counters.signinResponses, 0);
    assert.equal(report.counters.payloadBytesIntentionallyRead, 0);
  });
}

test('oversized Text rejected before payload allocation', async () => {
  const { error, report } = await run(Buffer.from([0x81, 127, 0, 0, 0, 0, 0, 4, 0, 1]));
  assert.equal(error, 'text_response_size');
  assert.equal(report.counters.payloadBytesIntentionallyRead, 0);
});

test('empty Text rejected', async () => {
  assert.equal((await run(Buffer.from([0x81, 0]))).error, 'text_response_size');
});

for (const [name, frame, expected] of [
  ['wrong RPC id', () => textFrame({ id: 2, result: 'secret' }), 'rpc_identity_shape'],
  ['missing result', () => textFrame({ id: 1 }), 'rpc_result_shape'],
  ['both result and error', () => textFrame({ id: 1, result: 'secret', error: {} }), 'rpc_result_shape'],
  ['malformed JSON', () => Buffer.from([0x81, 1, 123]), 'rpc_json_shape'],
  ['invalid UTF-8', () => Buffer.from([0x81, 1, 255]), 'rpc_json_shape'],
]) {
  test(name + ': clear response bytes', async () => {
    const input = frame();
    assert.equal((await run(input)).error, expected);
    assert.ok(input.every(byte => byte === 0));
  });
}

test('stalled Pong write respects remaining exchange deadline', async () => {
  const { error, report } = await run(ping(), {
    budget: 60, behavior: n => n === 1 ? 'ok' : 'stall',
  });
  assert.equal(error, 'signin_exchange_deadline');
  assert.equal(report.counters.pongWritesCompleted, 0);
  assert.equal(report.counters.frameHeadersObserved, 1);
});

test('Pong write failure after partial send is terminal', async () => {
  const { error, report } = await run(ping(), { behavior: n => n === 1 ? 'ok' : 'error' });
  assert.equal(error, 'websocket_transport_error');
  assert.equal(report.counters.pongWritesCompleted, 0);
});

test('sign-in write must settle before buffered Ping is read', async () => {
  let writes = 0;
  const { error, report } = await run(Buffer.concat([ping(), textFrame()]), {
    behavior(n, callback) {
      writes = n;
      if (n === 1) { setTimeout(callback, 20); return 'pending'; }
      return 'ok';
    },
  });
  assert.equal(error, undefined);
  assert.equal(writes, 2);
  assert.equal(report.state, 'signin_success');
});

test('stalled sign-in consumes deadline without reading or Pong', async () => {
  const { error, report, socket } = await run(Buffer.concat([ping(), textFrame()]), {
    budget: 40, behavior: () => 'stall',
  });
  assert.equal(error, 'signin_exchange_deadline');
  assert.equal(report.counters.frameHeadersObserved, 0);
  assert.equal(socket.sent.length, 1);
});

test('elapsed sign-in time is not reset for Pong', async () => {
  const { error, report } = await run(ping(), {
    budget: 100,
    behavior(n, callback) {
      if (n === 1) { setTimeout(callback, 60); return 'pending'; }
      setTimeout(callback, 70);
      return 'pending';
    },
  });
  assert.equal(error, 'signin_exchange_deadline');
  assert.equal(report.counters.pongWritesCompleted, 0);
});

test('expired deadline rejects buffered reads', async () => {
  const peer = new Peer(Buffer.from([1, 2]));
  const reader = new SocketReader(peer);
  await assert.rejects(reader.readExact(2, performance.now() - 1, 'deadline'), /deadline/);
  clearSocket(peer);
});

for (const [name, input, headers, pongs] of [
  ['incomplete first header', Buffer.from([0x81]), 0, 0],
  ['incomplete Text payload', Buffer.from([0x81, 5, 123]), 1, 0],
  ['completed Pong then stalled Text', ping(), 1, 1],
]) {
  test(name + ': read wait preserves original deadline and clears queued bytes', async () => {
    const started = performance.now();
    const { error, report, socket } = await run(input, {
      budget: 100,
      behavior(n, callback) {
        if (n === 1) { setTimeout(callback, 60); return 'pending'; }
        return 'ok';
      },
    });
    assert.equal(error, 'signin_exchange_deadline');
    assert.ok(performance.now() - started < 155, 'read must not receive a fresh 100ms budget');
    assert.equal(report.counters.frameHeadersObserved, headers);
    assert.equal(report.counters.pongWritesCompleted, pongs);
    assert.equal(report.counters.signinResponses, 0);
    assert.ok(socket.destroyed);
    assert.ok(input.every(byte => byte === 0));
    assert.equal(socket.listenerCount('readable'), 0);
  });
}

test('expired write deadline clears Pong without writing', async () => {
  const peer = new Peer();
  await assert.rejects(sendPong(peer, performance.now() - 1), /signin_exchange_deadline/);
  assert.equal(peer.sent.length, 0);
  clearSocket(peer);
});

test('synchronous write exception clears owned frame', async () => {
  const peer = new Peer();
  peer.write = () => { throw new Error('synthetic'); };
  const frame = Buffer.from([1, 2, 3]);
  await assert.rejects(writeFrame(peer, frame, performance.now() + 100), /websocket_transport_error/);
  assert.deepEqual([...frame], [0, 0, 0]);
  clearSocket(peer);
});

for (const signal of ['SIGINT', 'SIGTERM']) {
  test(signal + ': destroys socket and removes signal listeners', async () => {
    const emitter = new EventEmitter();
    const socket = new Peer(ping(), () => 'stall');
    const reader = new SocketReader(socket);
    let interrupted = false;
    const unbind = bindInterrupts(() => socket, () => { interrupted = true; }, emitter);
    const report = newReport();
    const pending = exchange(socket, reader, { user: 'synthetic', pass: 'synthetic' },
      performance.now() + 1000, report);
    emitter.emit(signal);
    await assert.rejects(pending, /websocket_closed/);
    assert.ok(interrupted && socket.destroyed);
    clearSocket(socket);
    unbind();
    assert.equal(emitter.listenerCount(signal), 0);
    assert.ok(socket.references.every(b => b.every(v => v === 0)));
  });
}

test('actual collection receipt writer: cap, replacement, truncation and exclusive reservation', () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'zero-ping-receipt-'));
  const file = path.join(dir, 'receipt.json');
  let fd;
  try {
    fd = fs.openSync(file, 'wx', 0o600);
    assert.throws(() => fs.openSync(file, 'wx', 0o600), /EEXIST/);
    writeFdJson(fd, { status: 'synthetic-long-initial-receipt' });
    writeFdJson(fd, { status: 'short' });
    const expected = JSON.stringify({ status: 'short' }, null, 2) + '\n';
    assert.equal(fs.readFileSync(file, 'utf8'), expected);
    assert.equal(fs.statSync(file).size, Buffer.byteLength(expected));
    assert.throws(() => writeFdJson(fd, { value: 'x'.repeat(65536) }), /evidence_file_size/);
    assert.equal(fs.readFileSync(file, 'utf8'), expected);
  } finally {
    if (fd !== undefined) fs.closeSync(fd);
    if (fs.existsSync(file)) fs.unlinkSync(file);
    fs.rmdirSync(dir);
  }
});

test('preserved real receipt: digest, bounds, cardinality, exact allowed keys', () => {
  const bytes = fs.readFileSync(new URL('./evidence/signin-observation.json', import.meta.url));
  assert.equal(createHash('sha256').update(bytes).digest('hex'),
    '969d730640fb8c8e1ddede26bdc3b1236b22b3719d9e138fa67ec80011a515b8');
  assert.ok(bytes.length <= 65536);
  const receipt = JSON.parse(bytes);
  assert.equal(receipt.state, 'stopped');
  assert.equal(receipt.stopReason, 'unsupported_websocket_frame');
  assert.equal(receipt.frames.length, 2);
  assert.ok(receipt.frames.every(f => f.opcode === 9 && f.declaredLength === '0'));
  assert.equal(receipt.counters.pongWritesCompleted, 1);
  assert.equal(receipt.counters.signinResponses, 0);
  assert.ok(receipt.elapsedMs <= 30000 && receipt.exchangeElapsedMs <= 15000);
  const allowed = new Set(('startedAt completedAt state stopReason collectorSha256 continuity handshake frames '
    + 'signinStatus elapsedMs exchangeElapsedMs limits counters observedAt sourceHead inheritedEvidence '
    + 'observationSha256 collectorSha256 classifiedDigests services configurationFiles targetConfirmed gate '
    + 'retainedSecrets executableSha256 pidMatchesBaseline startTimeMatchesBaseline executablePathMatchesBaseline '
    + 'executableDigestMatchesBaseline listenersMatchBaseline sha256 matchesBaseline httpStatus negotiatedSubprotocol '
    + 'extensionsRequested extensionsResponsePresent fin rsv1 rsv2 rsv3 opcode opcodeClass masked lengthEncoding '
    + 'declaredLength minimalLengthEncoding lengthMostSignificantBitZero framingBytesConsumed upgrades signinRequests '
    + 'pongWrites textBytes evidenceBytes upgradeMs exchangeMs totalActiveMs websocketUpgradeAttempts pongWriteAttempts '
    + 'pongWritesCompleted frameHeadersObserved signinResponses payloadBytesIntentionallyRead payloadBytesRetained '
    + 'rawFramingBytesRetained credentialValuesRetained databaseQueries readinessRequests healthRequests inferenceRequests '
    + 'recordMutations serviceMutations retries').split(' '));
  const walk = value => {
    if (Array.isArray(value)) { value.forEach(walk); return; }
    if (!value || typeof value !== 'object') return;
    for (const [key, child] of Object.entries(value)) { assert.ok(allowed.has(key), key); walk(child); }
  };
  walk(receipt);
  for (const key of ['payloadBytesRetained', 'rawFramingBytesRetained', 'credentialValuesRetained',
    'databaseQueries', 'readinessRequests', 'healthRequests', 'inferenceRequests', 'recordMutations', 'retries']) {
    assert.equal(receipt.counters[key], 0, key);
  }
});
