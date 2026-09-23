// Single-attempt diagnostic for the first WebSocket response frame header.
// --prepare is passive; --collect is the sole authorized network attempt;
// --self-test is offline and reserved for phase-end verification.
import fs from 'node:fs';
import net from 'node:net';
import path from 'node:path';
import { execFile } from 'node:child_process';
import { createHash, randomBytes } from 'node:crypto';
import { EventEmitter } from 'node:events';
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
const OBSERVATION_PATH = path.join(EVIDENCE_DIR, 'frame-observation.json');
const LOCAL_MAX = 256 * 1024;
const EVIDENCE_MAX = 64 * 1024;
const UPGRADE_MS = 15_000;
const FRAME_HEADER_MS = 15_000;
const TOTAL_MS = 30_000;
const EXPECTED_PRIOR_OBSERVATION_SHA256 = 'dbcac041a81e21775302e56a629daac3dd06eb7fdccb6229b928bfcef83abf09';
const EXPECTED_PRIOR_COLLECTOR_SHA256 = '5dbbe09b51e705681d4d9c4be0da8315500df2172095ed094c14888a9dd0f467';
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
  const contents = smallFile(CONFIG_PATH).toString('utf8');
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
    // Strings are immutable; the source buffer is not retained or emitted.
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
  return target;
}

async function gatherContinuity() {
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
    const sha256 = createHash('sha256').update(smallFile(item.path)).digest('hex');
    requireThat(sha256 === item.sha256, 'configuration_identity_changed');
    configurationFiles.push({ sha256, matchesBaseline: true });
  }

  requireThat(!fs.existsSync('/Users/gqadonis/.uar/.env'), 'new_working_directory_dotenv');
  for (const file of [SERVICE_ENV_PATH, HOME_ENV_PATH]) {
    const contents = smallFile(file).toString('utf8');
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
    },
    services,
    configurationFiles,
    target,
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
    requireThat(count >= 0 && count <= 10, 'framing_prefix_limit');
    while (true) {
      requireThat(!this.failed && !this.ended && !this.socket.destroyed, 'websocket_closed');
      const chunk = this.socket.read(count);
      if (chunk !== null) return chunk;
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

async function sendSignin(socket, credentials, deadlineAt) {
  const payload = Buffer.from(JSON.stringify({
    id: 1,
    method: 'signin',
    params: [{ user: credentials.user, pass: credentials.pass }],
  }));
  requireThat(payload.length > 0 && payload.length < 65_536, 'signin_request_size');
  const mask = randomBytes(4);
  const short = payload.length < 126;
  const header = Buffer.alloc(short ? 6 : 8);
  header[0] = 0x81;
  header[1] = 0x80 | (short ? payload.length : 126);
  if (!short) header.writeUInt16BE(payload.length, 2);
  mask.copy(header, short ? 2 : 4);
  for (let index = 0; index < payload.length; index++) payload[index] ^= mask[index % 4];
  const frame = Buffer.concat([header, payload]);
  payload.fill(0);
  header.fill(0);
  mask.fill(0);
  const remaining = deadlineAt - mono();
  requireThat(remaining > 0, 'frame_header_deadline');
  await new Promise((resolve, reject) => {
    let timer;
    let settled = false;
    const cleanup = () => {
      clearTimeout(timer);
      socket.off('error', failed);
      socket.off('close', closed);
    };
    const finish = error => {
      if (settled) return;
      settled = true;
      cleanup();
      frame.fill(0);
      if (error) reject(error);
      else resolve();
    };
    const failed = () => finish(new Stop('websocket_transport_error'));
    const closed = () => finish(new Stop('websocket_closed'));
    timer = setTimeout(() => {
      finish(new Stop('frame_header_deadline'));
      socket.destroy();
    }, remaining);
    socket.once('error', failed);
    socket.once('close', closed);
    try {
      socket.write(frame, error => {
        if (error) finish(new Stop('websocket_transport_error'));
        else finish();
      });
    } catch {
      finish(new Stop('websocket_transport_error'));
    }
  });
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
  const base = await reader.readExact(2, deadlineAt, 'frame_header_deadline');
  const indicator = base[1] & 0x7f;
  const extensionCount = indicator < 126 ? 0 : indicator === 126 ? 2 : 8;
  const extension = extensionCount === 0
    ? Buffer.alloc(0)
    : await reader.readExact(extensionCount, deadlineAt, 'frame_header_deadline');
  const prefix = Buffer.concat([base, extension]);
  const metadata = decodeFramePrefix(prefix);
  prefix.fill(0);
  base.fill(0);
  extension.fill(0);
  return metadata;
}

async function runPrepare() {
  const report = {
    observedAt: now(),
    mode: 'prepare',
    state: 'started',
    limits: {
      websocketUpgrades: 1,
      signinRequests: 1,
      framingPrefixBytes: 10,
      evidenceBytes: EVIDENCE_MAX,
      upgradeMs: UPGRADE_MS,
      frameHeaderMs: FRAME_HEADER_MS,
      totalActiveMs: TOTAL_MS,
    },
    continuity: null,
    stopReason: null,
    activeNetworkRequests: 0,
  };
  try {
    report.continuity = await gatherContinuity();
    report.state = 'ready_for_single_authorized_attempt';
  } catch (error) {
    report.state = 'blocked';
    report.stopReason = safeCode(error);
  }
  writeExclusiveJson(CONTINUITY_PATH, report);
  process.stdout.write(JSON.stringify({
    state: report.state,
    stopReason: report.stopReason,
    activeNetworkRequests: report.activeNetworkRequests,
  }) + '\n');
  if (report.state !== 'ready_for_single_authorized_attempt') process.exitCode = 2;
}

async function runCollect() {
  fs.mkdirSync(EVIDENCE_DIR, { recursive: true, mode: 0o700 });
  const fd = fs.openSync(OBSERVATION_PATH, 'wx', 0o600);
  const report = {
    startedAt: now(),
    completedAt: null,
    mode: 'collect',
    state: 'started',
    stopReason: null,
    limits: {
      websocketUpgrades: 1,
      signinRequests: 1,
      framingPrefixBytes: 10,
      evidenceBytes: EVIDENCE_MAX,
      upgradeMs: UPGRADE_MS,
      frameHeaderMs: FRAME_HEADER_MS,
      totalActiveMs: TOTAL_MS,
    },
    continuity: null,
    handshake: null,
    frame: null,
    counters: {
      websocketUpgradeAttempts: 0,
      signinRequests: 0,
      frameHeadersObserved: 0,
      payloadBytesIntentionallyRead: 0,
      payloadBytesRetained: 0,
      rawFramingBytesRetained: 0,
      credentialValuesRetained: 0,
      databaseQueries: 0,
      readinessRequests: 0,
      healthRequests: 0,
      inferenceRequests: 0,
      recordMutations: 0,
      serviceMutations: 0,
      retries: 0,
      productTests: 0,
    },
  };
  writeFdJson(fd, report);
  let socket;
  let credentials;
  const started = mono();
  const totalDeadline = started + TOTAL_MS;
  try {
    const prepared = JSON.parse(smallFile(CONTINUITY_PATH).toString('utf8'));
    requireThat(prepared.state === 'ready_for_single_authorized_attempt', 'prepare_gate_not_ready');
    report.continuity = await gatherContinuity();
    requireThat(mono() < totalDeadline, 'total_active_deadline');

    const target = readTarget(true);
    credentials = target.credentials;
    delete target.credentials;
    report.target = target;

    report.counters.websocketUpgradeAttempts = 1;
    const upgradeDeadline = Math.min(totalDeadline, mono() + UPGRADE_MS);
    socket = net.createConnection({ host: target.host, port: target.port, highWaterMark: 16_384 });
    const reader = new SocketReader(socket);
    await waitForConnect(socket, upgradeDeadline);
    const key = randomBytes(16).toString('base64');
    socket.write(
      `GET ${target.path} HTTP/1.1\r\n`
      + `Host: ${target.host}:${target.port}\r\n`
      + 'Upgrade: websocket\r\n'
      + 'Connection: Upgrade\r\n'
      + `Sec-WebSocket-Key: ${key}\r\n`
      + 'Sec-WebSocket-Version: 13\r\n'
      + 'Sec-WebSocket-Protocol: json\r\n\r\n',
    );
    report.handshake = await readHandshake(reader, upgradeDeadline, key);

    requireThat(mono() < totalDeadline, 'total_active_deadline');
    report.counters.signinRequests = 1;
    const frameDeadline = Math.min(totalDeadline, mono() + FRAME_HEADER_MS);
    await sendSignin(socket, credentials, frameDeadline);
    credentials.user = '';
    credentials.pass = '';
    credentials = null;

    report.frame = await readFrameMetadata(reader, frameDeadline);
    report.counters.frameHeadersObserved = 1;
    socket.destroy();
    socket = null;
    report.state = 'header_observed';
  } catch (error) {
    report.state = 'stopped';
    report.stopReason = safeCode(error);
  } finally {
    if (credentials) {
      credentials.user = '';
      credentials.pass = '';
    }
    socket?.destroy();
    report.completedAt = now();
    report.clientElapsedMs = Math.round((mono() - started) * 1000) / 1000;
    writeFdJson(fd, report);
    fs.closeSync(fd);
  }
  process.stdout.write(JSON.stringify({
    state: report.state,
    stopReason: report.stopReason,
    counters: report.counters,
    frame: report.frame,
  }) + '\n');
  if (report.state !== 'header_observed') process.exitCode = 2;
}

async function runSelfTest() {
  const cases = [
    {
      name: 'short-final-text',
      prefix: Buffer.from([0x81, 0x05]),
      expected: { fin: true, opcode: 1, masked: false, declaredLength: '5', framingBytesConsumed: 2 },
    },
    {
      name: 'empty-ping',
      prefix: Buffer.from([0x89, 0x00]),
      expected: { fin: true, opcode: 9, masked: false, declaredLength: '0', framingBytesConsumed: 2 },
    },
    {
      name: 'fragmented-extended-text',
      prefix: Buffer.from([0x01, 0x7e, 0x00, 0x7e]),
      expected: { fin: false, opcode: 1, masked: false, declaredLength: '126', framingBytesConsumed: 4 },
    },
    {
      name: 'binary-64-bit',
      prefix: Buffer.from([0x82, 0x7f, 0, 0, 0, 0, 0, 1, 0, 0]),
      expected: { fin: true, opcode: 2, masked: false, declaredLength: '65536', framingBytesConsumed: 10 },
    },
    {
      name: 'masked-server-frame',
      prefix: Buffer.from([0x81, 0x80]),
      expected: { fin: true, opcode: 1, masked: true, declaredLength: '0', framingBytesConsumed: 2 },
    },
    {
      name: 'non-minimal-16-bit-length',
      prefix: Buffer.from([0x81, 0x7e, 0x00, 0x7d]),
      expected: { fin: true, opcode: 1, masked: false, declaredLength: '125', framingBytesConsumed: 4, minimalLengthEncoding: false },
    },
  ];
  const results = [];
  for (const fixture of cases) {
    const actual = decodeFramePrefix(fixture.prefix);
    for (const [key, value] of Object.entries(fixture.expected)) {
      requireThat(actual[key] === value, `fixture_${fixture.name}_${key}`);
    }
    results.push({ name: fixture.name, status: 'pass' });
    fixture.prefix.fill(0);
  }
  class StalledSocket extends EventEmitter {
    write() {}
    destroy() { this.emit('close'); }
  }
  let deadlineCode = null;
  try {
    await sendSignin(new StalledSocket(), { user: 'synthetic', pass: 'synthetic' }, mono() + 10);
  } catch (error) {
    deadlineCode = safeCode(error);
  }
  requireThat(deadlineCode === 'frame_header_deadline', 'fixture_signin_write_deadline');
  results.push({ name: 'signin-write-frame-deadline', status: 'pass' });
  process.stdout.write(JSON.stringify({ cases: results.length, passed: results.length, results }) + '\n');
}

const mode = process.argv[2];
if (mode === '--prepare') await runPrepare();
else if (mode === '--collect') await runCollect();
else if (mode === '--self-test') await runSelfTest();
else {
  process.stderr.write('usage: node observer.mjs --prepare|--collect|--self-test\n');
  process.exitCode = 64;
}
