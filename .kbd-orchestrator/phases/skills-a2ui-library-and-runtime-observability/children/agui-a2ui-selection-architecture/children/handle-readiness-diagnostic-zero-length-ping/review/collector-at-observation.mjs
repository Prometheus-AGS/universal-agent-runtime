// One authorized sign-in attempt: direct Text or exactly one empty Ping/Pong then Text.
// Derived from the accepted header observer and restricted collector; originals immutable.
import fs from 'node:fs';
import net from 'node:net';
import path from 'node:path';
import { execFile } from 'node:child_process';
import { createHash, randomBytes } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import { promisify } from 'node:util';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = '/Users/gqadonis/Projects/prometheus/universal-agent-runtime';
const PRIOR = path.resolve(HERE, '../diagnose-database-readiness-latency');
const PRIOR_IDENTITY_PATH = path.join(PRIOR, 'evidence/identity.json');
const PRIOR_OBSERVATION_PATH = path.join(PRIOR, 'evidence/observations.json');
const PRIOR_COLLECTOR_PATH = path.join(PRIOR, 'collector.mjs');
const CONFIG_PATH = '/Users/gqadonis/Projects/graph-explorer/.uar/config.yaml';
const SERVICE_ENV_PATH = '/Users/gqadonis/.uar/service.env';
const HOME_ENV_PATH = '/Users/gqadonis/.env';
const EVIDENCE_DIR = path.join(HERE, 'evidence');
const CONTINUITY_PATH = path.join(EVIDENCE_DIR, 'continuity.json');
const OBSERVATION_PATH = path.join(EVIDENCE_DIR, 'signin-observation.json');
const LOCAL_MAX = 256 * 1024;
const EVIDENCE_MAX = 64 * 1024;
const UPGRADE_MS = 15_000;
const EXCHANGE_MS = 15_000;
const TOTAL_MS = 30_000;
const EXPECTED_PRIOR_OBSERVATION_SHA256 = 'dbcac041a81e21775302e56a629daac3dd06eb7fdccb6229b928bfcef83abf09';
const EXPECTED_PRIOR_COLLECTOR_SHA256 = '5dbbe09b51e705681d4d9c4be0da8315500df2172095ed094c14888a9dd0f467';
const CLASSIFIED = path.resolve(HERE, '../classify-readiness-diagnostic-websocket-frame');
const INHERITED = {
  'observer.mjs': '0d54cdbdcd409a68db07201f1c456d23297bf5708aa3e7febf6903e9894e1fbb',
  'evidence/frame-observation.json': '95c915f92378c6630451eeb5405df11519ca8b2176a6369c6f42005dd2caa58f',
  'evidence/continuity.json': '3354bdb79099520f71454cef71d5259acdbfde50da8b2f7d05288d64abb4768d',
};
const exec = promisify(execFile);
const now = () => new Date().toISOString();
const mono = () => performance.now();

class Stop extends Error {}

function fail(code) {
  throw new Stop(code);
}

function requireThat(condition, code) {
  if (!condition) fail(code);
}

function safeCode(error) {
  return error instanceof Stop ? error.message : 'unclassified_local_failure';
}

function smallFile(file) {
  const fd = fs.openSync(file, 'r');
  try {
    const size = fs.fstatSync(fd).size;
    requireThat(size <= LOCAL_MAX, 'local_file_size');
    const buffer = Buffer.alloc(size);
    requireThat(fs.readSync(fd, buffer, 0, size, 0) === size, 'local_file_changed');
    return buffer;
  } finally {
    fs.closeSync(fd);
  }
}

async function digest(file) {
  const hash = createHash('sha256');
  for await (const chunk of fs.createReadStream(file, { highWaterMark: 65_536 })) {
    hash.update(chunk);
  }
  return hash.digest('hex');
}

async function local(command, args) {
  try {
    const result = await exec(command, args, {
      cwd: ROOT,
      encoding: 'utf8',
      timeout: 4_000,
      maxBuffer: LOCAL_MAX,
      env: {
        PATH: '/usr/bin:/bin:/usr/sbin:/sbin',
        LC_ALL: 'C',
        TZ: 'UTC',
      },
    });
    return result.stdout.trim();
  } catch {
    fail('local_identity_command_failed');
  }
}

function writeExclusiveJson(file, value) {
  fs.mkdirSync(path.dirname(file), { recursive: true, mode: 0o700 });
  const serialized = JSON.stringify(value, null, 2) + '\n';
  requireThat(Buffer.byteLength(serialized) <= EVIDENCE_MAX, 'evidence_file_size');
  const fd = fs.openSync(file, 'wx', 0o600);
  try {
    fs.writeFileSync(fd, serialized, 'utf8');
    fs.fsyncSync(fd);
  } finally {
    fs.closeSync(fd);
  }
}

function writeFdJson(fd, value) {
  const serialized = JSON.stringify(value, null, 2) + '\n';
  requireThat(Buffer.byteLength(serialized) <= EVIDENCE_MAX, 'evidence_file_size');
  fs.writeSync(fd, serialized, 0, 'utf8');
  fs.ftruncateSync(fd, Buffer.byteLength(serialized));
  fs.fsyncSync(fd);
}

function readTarget(includeCredentials) {
  let section = '';
  const fields = {};
  const source = smallFile(CONFIG_PATH);
  let contents = source.toString('utf8');
  source.fill(0);
  try {
    for (const line of contents.split('\n')) {
      if (/^[\w]+:/.test(line)) section = line.split(':')[0];
      if (section !== 'persistence') continue;
      const match = line.match(/^  (provider|database_url|surreal_ns|surreal_db|surreal_user|surreal_pass):\s*(.*)$/);
      if (!match) continue;
      requireThat(!Object.hasOwn(fields, match[1]), 'duplicate_target_field');
      try {
        fields[match[1]] = JSON.parse(match[2]);
      } catch {
        fail('unsupported_literal_config');
      }
      requireThat(
        typeof fields[match[1]] === 'string' && !/\$\{|vault:\/\//.test(fields[match[1]]),
        'unsupported_credential_source',
      );
    }
  } finally {
    // Immutable strings cannot be securely erased; release references, never emit.
    contents = '';
  }
  requireThat(
    fields.provider === 'surreal'
      && fields.database_url === 'ws://127.0.0.1:28000'
      && fields.surreal_ns === 'uar'
      && fields.surreal_db === 'runtime_openai_1536'
      && fields.surreal_user === 'root'
      && typeof fields.surreal_pass === 'string'
      && fields.surreal_pass.length > 0,
    'target_config_mismatch',
  );
  const target = {
    host: '127.0.0.1',
    port: 28_000,
    path: '/rpc',
    authenticationForm: 'root_signin_literal',
    credentialValuesRetained: false,
  };
  if (includeCredentials) {
    target.credentials = { user: fields.surreal_user, pass: fields.surreal_pass };
  }
  for (const key of Object.keys(fields)) fields[key] = '';
  return target;
}

async function gatherContinuity() {
  const classifiedDigests = [];
  for (const [file, expected] of Object.entries(INHERITED)) {
    const actual = await digest(path.join(CLASSIFIED, file));
    requireThat(actual === expected, 'classified_artifact_changed');
    classifiedDigests.push(actual);
  }
  const baseline = JSON.parse(smallFile(PRIOR_IDENTITY_PATH).toString('utf8'));
  requireThat(baseline.sourceHead && Array.isArray(baseline.services), 'baseline_identity_shape');

  const currentSourceHead = await local('/usr/bin/git', ['rev-parse', 'HEAD']);
  requireThat(currentSourceHead === baseline.sourceHead, 'source_head_changed');

  const priorObservationSha256 = await digest(PRIOR_OBSERVATION_PATH);
  const priorCollectorSha256 = await digest(PRIOR_COLLECTOR_PATH);
  requireThat(priorObservationSha256 === EXPECTED_PRIOR_OBSERVATION_SHA256, 'prior_observation_changed');
  requireThat(priorCollectorSha256 === EXPECTED_PRIOR_COLLECTOR_SHA256, 'prior_collector_changed');

  const configurationFiles = [];
  for (const item of baseline.configurationFiles) {
    const source = smallFile(item.path);
    const sha256 = createHash('sha256').update(source).digest('hex');
    source.fill(0);
    requireThat(sha256 === item.sha256, 'configuration_identity_changed');
    configurationFiles.push({ sha256, matchesBaseline: true });
  }

  requireThat(!fs.existsSync('/Users/gqadonis/.uar/.env'), 'new_working_directory_dotenv');
  for (const file of [SERVICE_ENV_PATH, HOME_ENV_PATH]) {
    const source = smallFile(file);
    const contents = source.toString('utf8');
    source.fill(0);
    requireThat(
      !/^\s*(?:export\s+)?UAR_(?:PERSISTENCE|RESILIENCE)__\w+\s*=/m.test(contents),
      'environment_override_changed',
    );
  }

  const services = [];
  for (const service of baseline.services) {
    const launch = await local('/bin/launchctl', ['print', `gui/${process.getuid()}/${service.label}`]);
    const pid = Number(launch.match(/^\s*pid = (\d+)/m)?.[1]);
    requireThat(pid === service.pid, 'service_pid_changed');
    requireThat(
      !/^\s*UAR_(?:PERSISTENCE|RESILIENCE)__\w+\s*=>/m.test(launch),
      'launch_environment_override_changed',
    );
    const started = await local('/bin/ps', ['-p', String(pid), '-o', 'lstart=']);
    requireThat(Date.parse(started + ' UTC') === Date.parse(service.startTime), 'service_start_changed');
    const executable = await local('/bin/ps', ['-p', String(pid), '-o', 'comm=']);
    requireThat(executable === service.executable, 'service_executable_changed');
    const executableSha256 = await digest(executable);
    requireThat(executableSha256 === service.sha256, 'service_executable_digest_changed');
    const listeners = await local('/usr/sbin/lsof', [
      '-nP', '-a', '-p', String(pid), '-iTCP', '-sTCP:LISTEN',
    ]);
    for (const address of service.listeners) {
      requireThat(listeners.includes(`TCP ${address} (LISTEN)`), 'listener_identity_changed');
    }
    services.push({
      executableSha256,
      pidMatchesBaseline: true,
      startTimeMatchesBaseline: true,
      executablePathMatchesBaseline: true,
      executableDigestMatchesBaseline: true,
      listenersMatchBaseline: true,
    });
  }

  const target = readTarget(false);
  return {
    observedAt: now(),
    sourceHead: currentSourceHead,
    inheritedEvidence: {
      observationSha256: priorObservationSha256,
      collectorSha256: priorCollectorSha256,
      classifiedDigests,
    },
    services,
    configurationFiles,
    targetConfirmed: Boolean(target),
    gate: {
      matchesBaseline: true,
    },
    retainedSecrets: false,
  };
}

class SocketReader {
  constructor(socket) {
    this.socket = socket;
    this.failed = false;
    this.ended = false;
    socket.on('error', () => { this.failed = true; });
    socket.on('end', () => { this.ended = true; });
  }

  async wait(deadlineAt, timeoutCode) {
    const remaining = deadlineAt - mono();
    requireThat(remaining > 0, timeoutCode);
    await new Promise((resolve, reject) => {
      let timer;
      const done = () => {
        clearTimeout(timer);
        for (const event of ['readable', 'error', 'end', 'close']) this.socket.off(event, done);
        resolve();
      };
      timer = setTimeout(() => {
        for (const event of ['readable', 'error', 'end', 'close']) this.socket.off(event, done);
        this.socket.destroy();
        reject(new Stop(timeoutCode));
      }, remaining);
      for (const event of ['readable', 'error', 'end', 'close']) this.socket.once(event, done);
    });
  }

  async readExact(count, deadlineAt, timeoutCode) {
    requireThat(count >= 0 && count <= LOCAL_MAX, 'response_buffer_limit');
    while (true) {
      requireThat(mono() < deadlineAt, timeoutCode);
      requireThat(!this.failed && !this.ended && !this.socket.destroyed, 'websocket_closed');
      const chunk = this.socket.read(count);
      if (chunk !== null) {
        if (chunk.length !== count || mono() >= deadlineAt) {
          chunk.fill(0);
          fail(mono() >= deadlineAt ? timeoutCode : 'truncated_response');
        }
        return chunk;
      }
      await this.wait(deadlineAt, timeoutCode);
    }
  }
}

async function waitForConnect(socket, deadlineAt) {
  if (!socket.connecting) return;
  const remaining = deadlineAt - mono();
  requireThat(remaining > 0, 'upgrade_deadline');
  await new Promise((resolve, reject) => {
    let timer;
    const cleanup = () => {
      clearTimeout(timer);
      socket.off('connect', connected);
      socket.off('error', failed);
      socket.off('close', closed);
    };
    const connected = () => { cleanup(); resolve(); };
    const failed = () => { cleanup(); reject(new Stop('websocket_transport_error')); };
    const closed = () => { cleanup(); reject(new Stop('websocket_closed')); };
    timer = setTimeout(() => {
      cleanup();
      socket.destroy();
      reject(new Stop('upgrade_deadline'));
    }, remaining);
    socket.once('connect', connected);
    socket.once('error', failed);
    socket.once('close', closed);
  });
}

async function readHandshake(reader, deadlineAt, key) {
  const bytes = [];
  while (true) {
    requireThat(bytes.length < 16_384, 'websocket_handshake_size');
    const byte = await reader.readExact(1, deadlineAt, 'upgrade_deadline');
    bytes.push(byte[0]);
    byte.fill(0);
    const length = bytes.length;
    if (length >= 4 && bytes[length - 4] === 13 && bytes[length - 3] === 10
      && bytes[length - 2] === 13 && bytes[length - 1] === 10) break;
  }
  const raw = Buffer.from(bytes);
  const head = raw.toString('latin1');
  raw.fill(0);
  bytes.fill(0);
  const lines = head.split('\r\n');
  const status = Number(lines[0].match(/^HTTP\/1\.1 (\d{3}) /)?.[1]);
  const headers = {};
  for (const line of lines.slice(1).filter(Boolean)) {
    const colon = line.indexOf(':');
    requireThat(colon > 0, 'websocket_handshake_shape');
    const name = line.slice(0, colon).toLowerCase();
    requireThat(!Object.hasOwn(headers, name), 'websocket_duplicate_header');
    headers[name] = line.slice(colon + 1).trim();
  }
  const expected = createHash('sha1')
    .update(key + '258EAFA5-E914-47DA-95CA-C5AB0DC85B11')
    .digest('base64');
  const extensionsResponsePresent = Object.hasOwn(headers, 'sec-websocket-extensions');
  requireThat(
    status === 101
      && headers['sec-websocket-accept'] === expected
      && headers['sec-websocket-protocol'] === 'json'
      && !extensionsResponsePresent
      && headers.upgrade?.toLowerCase() === 'websocket'
      && headers.connection?.toLowerCase().split(/\s*,\s*/).includes('upgrade'),
    'websocket_handshake_rejected',
  );
  return {
    httpStatus: status,
    negotiatedSubprotocol: 'json',
    extensionsRequested: false,
    extensionsResponsePresent,
  };
}

async function writeFrame(socket, frame, deadlineAt) {
  try {
    requireThat(mono() < deadlineAt, 'signin_exchange_deadline');
    await new Promise((resolve, reject) => {
      let timer;
      let settled = false;
      const finish = error => {
        if (settled) return;
        settled = true;
        clearTimeout(timer);
        socket.off('error', failed);
        socket.off('close', closed);
        if (error) reject(error); else resolve();
      };
      const failed = () => finish(new Stop('websocket_transport_error'));
      const closed = () => finish(new Stop('websocket_closed'));
      timer = setTimeout(() => {
        finish(new Stop('signin_exchange_deadline'));
        socket.destroy();
      }, Math.max(1, deadlineAt - mono()));
      socket.once('error', failed);
      socket.once('close', closed);
      try {
        socket.write(frame, error => {
          if (mono() >= deadlineAt) finish(new Stop('signin_exchange_deadline'));
          else if (error) failed();
          else finish();
        });
      } catch { failed(); }
    });
  } finally { frame.fill(0); }
}

async function sendSignin(socket, credentials, deadlineAt) {
  let payload, mask, header, frame;
  try {
    payload = Buffer.from(JSON.stringify({
      id: 1, method: 'signin',
      params: [{ user: credentials.user, pass: credentials.pass }],
    }));
    requireThat(payload.length > 0 && payload.length < 65_536, 'signin_request_size');
    mask = randomBytes(4);
    const short = payload.length < 126;
    header = Buffer.alloc(short ? 6 : 8);
    header[0] = 0x81;
    header[1] = 0x80 | (short ? payload.length : 126);
    if (!short) header.writeUInt16BE(payload.length, 2);
    mask.copy(header, short ? 2 : 4);
    for (let i = 0; i < payload.length; i++) payload[i] ^= mask[i % 4];
    frame = Buffer.concat([header, payload]);
    await writeFrame(socket, frame, deadlineAt);
  } finally {
    for (const buffer of [payload, mask, header, frame]) buffer?.fill(0);
    credentials.user = '';
    credentials.pass = '';
  }
}

async function sendPong(socket, deadlineAt) {
  const frame = Buffer.alloc(6);
  let mask;
  try {
    frame[0] = 0x8a;
    frame[1] = 0x80;
    mask = randomBytes(4);
    mask.copy(frame, 2);
    await writeFrame(socket, frame, deadlineAt);
  } finally {
    frame.fill(0);
    mask?.fill(0);
  }
}

function opcodeClass(opcode) {
  const known = {
    0: 'continuation',
    1: 'text',
    2: 'binary',
    8: 'close',
    9: 'ping',
    10: 'pong',
  };
  if (Object.hasOwn(known, opcode)) return known[opcode];
  return opcode < 8 ? 'reserved_data' : 'reserved_control';
}

function decodeFramePrefix(prefix) {
  requireThat(Buffer.isBuffer(prefix) && prefix.length >= 2 && prefix.length <= 10, 'frame_prefix_shape');
  const first = prefix[0];
  const second = prefix[1];
  const indicator = second & 0x7f;
  const expectedBytes = indicator < 126 ? 2 : indicator === 126 ? 4 : 10;
  requireThat(prefix.length === expectedBytes, 'frame_prefix_length');
  let declaredLength;
  let lengthEncoding;
  let minimalLengthEncoding = true;
  let lengthMostSignificantBitZero = true;
  if (indicator < 126) {
    declaredLength = BigInt(indicator);
    lengthEncoding = '7-bit';
  } else if (indicator === 126) {
    declaredLength = BigInt(prefix.readUInt16BE(2));
    lengthEncoding = '16-bit';
    minimalLengthEncoding = declaredLength >= 126n;
  } else {
    declaredLength = prefix.readBigUInt64BE(2);
    lengthEncoding = '64-bit';
    lengthMostSignificantBitZero = (prefix[2] & 0x80) === 0;
    minimalLengthEncoding = declaredLength >= 65_536n;
  }
  const opcode = first & 0x0f;
  return {
    fin: (first & 0x80) !== 0,
    rsv1: (first & 0x40) !== 0,
    rsv2: (first & 0x20) !== 0,
    rsv3: (first & 0x10) !== 0,
    opcode,
    opcodeClass: opcodeClass(opcode),
    masked: (second & 0x80) !== 0,
    lengthEncoding,
    declaredLength: declaredLength.toString(10),
    minimalLengthEncoding,
    lengthMostSignificantBitZero,
    framingBytesConsumed: expectedBytes,
  };
}

async function readFrameMetadata(reader, deadlineAt) {
  let base, extension, prefix;
  try {
    base = await reader.readExact(2, deadlineAt, 'signin_exchange_deadline');
    const indicator = base[1] & 0x7f;
    const count = indicator < 126 ? 0 : indicator === 126 ? 2 : 8;
    extension = count ? await reader.readExact(count, deadlineAt, 'signin_exchange_deadline') : Buffer.alloc(0);
    prefix = Buffer.concat([base, extension]);
    return decodeFramePrefix(prefix);
  } finally {
    for (const buffer of [base, extension, prefix]) buffer?.fill(0);
  }
}

function acceptedHeader(frame) {
  return frame.fin && !frame.rsv1 && !frame.rsv2 && !frame.rsv3
    && !frame.masked && frame.minimalLengthEncoding && frame.lengthMostSignificantBitZero;
}

function newReport() {
  return {
    startedAt: now(), completedAt: null, state: 'started', stopReason: null,
    collectorSha256: null, continuity: null, handshake: null, frames: [],
    signinStatus: null, elapsedMs: null, exchangeElapsedMs: null,
    limits: { upgrades: 1, signinRequests: 1, pongWrites: 1, frames: 2,
      textBytes: LOCAL_MAX, evidenceBytes: EVIDENCE_MAX, upgradeMs: UPGRADE_MS,
      exchangeMs: EXCHANGE_MS, totalActiveMs: TOTAL_MS },
    counters: { websocketUpgradeAttempts: 0, signinRequests: 0, pongWriteAttempts: 0,
      pongWritesCompleted: 0, frameHeadersObserved: 0, signinResponses: 0,
      payloadBytesIntentionallyRead: 0, payloadBytesRetained: 0,
      rawFramingBytesRetained: 0, credentialValuesRetained: 0,
      databaseQueries: 0, readinessRequests: 0, healthRequests: 0,
      inferenceRequests: 0, recordMutations: 0, serviceMutations: 0, retries: 0 },
  };
}

async function exchange(socket, reader, credentials, deadlineAt, report) {
  report.counters.signinRequests = 1;
  await sendSignin(socket, credentials, deadlineAt);
  requireThat(mono() < deadlineAt, 'signin_exchange_deadline');
  let frame = await readFrameMetadata(reader, deadlineAt);
  report.frames.push(frame);
  report.counters.frameHeadersObserved++;
  if (acceptedHeader(frame) && frame.opcode === 9 && frame.lengthEncoding === '7-bit'
      && frame.declaredLength === '0') {
    report.counters.pongWriteAttempts++;
    await sendPong(socket, deadlineAt);
    report.counters.pongWritesCompleted++;
    frame = await readFrameMetadata(reader, deadlineAt);
    report.frames.push(frame);
    report.counters.frameHeadersObserved++;
  }
  requireThat(acceptedHeader(frame) && frame.opcode === 1, 'unsupported_websocket_frame');
  const size = BigInt(frame.declaredLength);
  requireThat(size > 0n && size <= BigInt(LOCAL_MAX), 'text_response_size');
  let raw, message;
  try {
    raw = await reader.readExact(Number(size), deadlineAt, 'signin_exchange_deadline');
    report.counters.payloadBytesIntentionallyRead = raw.length;
    try { message = JSON.parse(new TextDecoder('utf-8', { fatal: true }).decode(raw)); }
    catch { fail('rpc_json_shape'); }
    requireThat(message && typeof message === 'object' && !Array.isArray(message)
      && message.id === 1, 'rpc_identity_shape');
    const error = Object.hasOwn(message, 'error');
    requireThat(error !== Object.hasOwn(message, 'result'), 'rpc_result_shape');
    requireThat(error
      ? message.error && typeof message.error === 'object'
        && Number.isInteger(message.error.code) && typeof message.error.message === 'string'
      : typeof message.result === 'string' && message.result.length > 0, 'rpc_result_shape');
    requireThat(mono() < deadlineAt, 'signin_exchange_deadline');
    report.signinStatus = error ? 'error' : 'success';
    report.counters.signinResponses = 1;
    report.state = error ? 'signin_error' : 'signin_success';
  } finally {
    raw?.fill(0);
    message = null;
  }
}

function bindInterrupts(socketProvider, onStop, emitter = process) {
  const interrupt = () => {
    onStop();
    socketProvider()?.destroy();
  };
  emitter.on('SIGINT', interrupt);
  emitter.on('SIGTERM', interrupt);
  return () => {
    emitter.off('SIGINT', interrupt);
    emitter.off('SIGTERM', interrupt);
  };
}

function clearSocket(socket) {
  if (!socket) return;
  socket.destroy();
  // Drop byte buffers still queued in the paused readable stream.
  let chunk;
  while ((chunk = socket.read()) !== null) chunk.fill(0);
}

async function runCollect() {
  fs.mkdirSync(EVIDENCE_DIR, { recursive: true, mode: 0o700 });
  // Exclusive reservation happens before any network operation, and is never removed.
  const fd = fs.openSync(OBSERVATION_PATH, 'wx', 0o600);
  const report = newReport();
  let socket, credentials, started, exchangeStarted, totalTimer;
  let interrupted = false;
  const unbind = bindInterrupts(() => socket, () => { interrupted = true; });
  try {
    writeFdJson(fd, report);
    report.collectorSha256 = await digest(fileURLToPath(import.meta.url));
    report.continuity = await gatherContinuity();
    requireThat(!interrupted, 'operator_interrupt');
    const target = readTarget(true);
    credentials = target.credentials;
    delete target.credentials;
    started = mono();
    const totalDeadline = started + TOTAL_MS;
    totalTimer = setTimeout(() => socket?.destroy(), TOTAL_MS);
    report.counters.websocketUpgradeAttempts = 1;
    const upgradeDeadline = Math.min(totalDeadline, started + UPGRADE_MS);
    socket = net.createConnection({ host: target.host, port: target.port, highWaterMark: 16_384 });
    const reader = new SocketReader(socket);
    await waitForConnect(socket, upgradeDeadline);
    const key = randomBytes(16).toString('base64');
    const request = Buffer.from(
      `GET ${target.path} HTTP/1.1\r\nHost: ${target.host}:${target.port}\r\n`
      + 'Upgrade: websocket\r\nConnection: Upgrade\r\n'
      + `Sec-WebSocket-Key: ${key}\r\n`
      + 'Sec-WebSocket-Version: 13\r\nSec-WebSocket-Protocol: json\r\n\r\n',
    );
    await writeFrame(socket, request, upgradeDeadline);
    report.handshake = await readHandshake(reader, upgradeDeadline, key);
    exchangeStarted = mono();
    await exchange(socket, reader, credentials,
      Math.min(totalDeadline, exchangeStarted + EXCHANGE_MS), report);
  } catch (error) {
    report.state = report.counters.websocketUpgradeAttempts ? 'stopped' : 'skipped';
    report.stopReason = interrupted ? 'operator_interrupt' : safeCode(error);
  } finally {
    unbind();
    clearTimeout(totalTimer);
    if (credentials) { credentials.user = ''; credentials.pass = ''; }
    clearSocket(socket);
    report.completedAt = now();
    report.elapsedMs = started === undefined ? 0 : Math.round((mono() - started) * 1000) / 1000;
    report.exchangeElapsedMs = exchangeStarted === undefined ? null
      : Math.round((mono() - exchangeStarted) * 1000) / 1000;
    try { writeFdJson(fd, report); } finally { fs.closeSync(fd); }
  }
  process.stdout.write(JSON.stringify({
    state: report.state, stopReason: report.stopReason, counters: report.counters,
  }) + '\n');
  if (report.state !== 'signin_success') process.exitCode = 2;
}

export { SocketReader, Stop, newReport, exchange, decodeFramePrefix, sendSignin,
  sendPong, writeFrame, bindInterrupts, clearSocket, writeExclusiveJson };

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  if (process.argv[2] === '--collect') {
    try { await runCollect(); }
    catch { process.stderr.write('collector_receipt_or_local_failure\n'); process.exitCode = 2; }
  } else {
    process.stderr.write('usage: node collector.mjs --collect\n');
    process.exitCode = 64;
  }
}
