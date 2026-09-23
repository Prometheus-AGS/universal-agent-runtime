// One installed-target diagnostic, not a general-purpose database client.
// --prepare is passive. --collect is reserved for task2.1 after source review.
import fs from 'node:fs';
import http from 'node:http';
import net from 'node:net';
import { createHash, randomBytes } from 'node:crypto';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const exec = promisify(execFile);
const MAX = 256 * 1024;
const FILE_MAX = 1024 * 1024;
const CONFIG = '/Users/gqadonis/Projects/graph-explorer/.uar/config.yaml';
const ENV = '/Users/gqadonis/.uar/service.env';
const HOME_ENV = '/Users/gqadonis/.env';
const BASE = JSON.parse(fs.readFileSync(path.join(HERE, 'evidence/identity.json'), 'utf8'));
const QUERIES = Object.freeze([
  'RETURN true;', 'RETURN true;',
  'SELECT id FROM skills LIMIT 1 TIMEOUT 5s;',
  'SELECT id FROM skills LIMIT 32 TIMEOUT 5s;', 'RETURN true;',
]);
const LOGS = Object.freeze([
  '/Users/gqadonis/.prometheus/logs/universal-agent-runtime/operational.log',
  '/Users/gqadonis/.prometheus/logs/universal-agent-runtime/stderr.log',
  '/Users/gqadonis/.prometheus/logs/surrealdb-native.stdout.log',
  '/Users/gqadonis/.prometheus/logs/surrealdb-native.stderr.log',
]);
const now = () => new Date().toISOString();
const mono = () => performance.now();
class Stop extends Error {}
const fail = code => { throw new Stop(code); };
const safeCode = error => error instanceof Stop ? error.message : 'unclassified_local_failure';
const requireThat = (ok, code) => { if (!ok) fail(code); };
let stopped = null;
let active = null;
let expires = Infinity;
let sessionExpires = Infinity;
const stop = code => { stopped ??= code; active?.destroy(); };

async function local(command, args) {
  try {
    const result = await exec(command, args, { encoding: 'utf8', timeout: 4000,
      maxBuffer: MAX, stdio: ['ignore', 'pipe', 'pipe'],
      env: { PATH: '/usr/bin:/bin:/usr/sbin:/sbin', LC_ALL: 'C', TZ: 'UTC' } });
    return result.stdout.trim();
  } catch { fail('local_identity_or_snapshot_command_failed'); }
}

function smallFile(file) {
  const fd = fs.openSync(file, 'r');
  try {
    const size = fs.fstatSync(fd).size;
    requireThat(size <= MAX, 'local_file_size');
    const b = Buffer.alloc(size);
    requireThat(fs.readSync(fd, b, 0, size, 0) === size, 'local_file_changed');
    return b;
  } finally { fs.closeSync(fd); }
}

async function digest(file) {
  const h = createHash('sha256');
  for await (const chunk of fs.createReadStream(file, { highWaterMark: 65536 })) h.update(chunk);
  return h.digest('hex');
}

async function pressure() {
  const raw = await local('/usr/sbin/sysctl', ['-n', 'kern.memorystatus_vm_pressure_level']);
  requireThat(/^[124]$/.test(raw), 'unknown_pressure_state');
  return Number(raw); // XNU dispatch bits: normal1, warning2, critical4.
}

async function identity() {
  for (const item of BASE.configurationFiles) {
    requireThat(createHash('sha256').update(smallFile(item.path)).digest('hex') === item.sha256,
      'configuration_identity_changed');
  }
  requireThat(!fs.existsSync('/Users/gqadonis/.uar/.env'), 'new_working_directory_dotenv');
  for (const file of [ENV, HOME_ENV]) {
    requireThat(!/^\s*(?:export\s+)?UAR_(?:PERSISTENCE|RESILIENCE)__\w+\s*=/m.test(smallFile(file).toString()),
      'environment_override_changed');
  }
  for (const svc of BASE.services) {
    const launch = await local('/bin/launchctl', ['print', `gui/${process.getuid()}/${svc.label}`]);
    requireThat(Number(launch.match(/^\s*pid = (\d+)/m)?.[1]) === svc.pid,
      'service_pid_changed');
    requireThat(!/^\s*UAR_(?:PERSISTENCE|RESILIENCE)__\w+\s*=>/m.test(launch),
      'launch_environment_override_changed');
    const started = await local('/bin/ps', ['-p', String(svc.pid), '-o', 'lstart=']);
    requireThat(Date.parse(started + ' UTC') === Date.parse(svc.startTime), 'service_start_changed');
    requireThat(await local('/bin/ps', ['-p', String(svc.pid), '-o', 'comm=']) === svc.executable,
      'service_executable_changed');
    const listeners = await local('/usr/sbin/lsof', ['-nP', '-a', '-p', String(svc.pid), '-iTCP', '-sTCP:LISTEN']);
    for (const address of svc.listeners) requireThat(listeners.includes(`TCP ${address} (LISTEN)`),
      'listener_identity_changed');
  }
  return { observedAt: now(), matchesBaseline: true, pids: BASE.services.map(s => s.pid) };
}

function credentials() {
  // This is deliberately not a YAML parser: only the exact reviewed literal
  // quoted configuration is supported. No interpolation, defaults or fallback.
  let section = '';
  const fields = {};
  for (const line of smallFile(CONFIG).toString().split('\n')) {
    if (/^[\w]+:/.test(line)) section = line.split(':')[0];
    if (section !== 'persistence') continue;
    const m = line.match(/^  (provider|database_url|surreal_ns|surreal_db|surreal_user|surreal_pass):\s*(.*)$/);
    if (!m) continue;
    requireThat(!Object.hasOwn(fields, m[1]), 'duplicate_target_field');
    try { fields[m[1]] = JSON.parse(m[2]); } catch { fail('unsupported_literal_config'); }
    requireThat(typeof fields[m[1]] === 'string' && !/\$\{|vault:\/\//.test(fields[m[1]]), 'unsupported_credential_source');
  }
  requireThat(fields.provider === 'surreal' && fields.database_url === 'ws://127.0.0.1:28000'
    && fields.surreal_ns === 'uar' && fields.surreal_db === 'runtime_openai_1536'
    && fields.surreal_user === 'root' && typeof fields.surreal_pass === 'string', 'target_config_mismatch');
  return { user: fields.surreal_user, pass: fields.surreal_pass };
}

async function snapshot() {
  const processes = [];
  for (const svc of BASE.services) {
    const values = (await local('/bin/ps', ['-p', String(svc.pid), '-o', 'pid=,%cpu=,rss=,etime='])).split(/\s+/);
    requireThat(values.length === 4 && values.slice(0, 3).every(v => /^\d+(?:\.\d+)?$/.test(v))
      && /^[\d:-]+$/.test(values[3]), 'process_snapshot_shape');
    processes.push({ pid: Number(values[0]), cpuPercent: Number(values[1]), rssKiB: Number(values[2]), elapsed: values[3] });
  }
  const vm = await local('/usr/bin/vm_stat', []);
  const memory = {};
  for (const key of ['Pages free', 'Pages active', 'Pages inactive', 'Pages wired down',
    'Pages occupied by compressor', 'Swapins', 'Swapouts']) {
    const value = vm.match(new RegExp('^' + key + ':\\s+(\\d+)\\.', 'm'));
    if (value) memory[key] = Number(value[1]);
  }
  const io = (await local('/usr/sbin/iostat', ['-Id', '-c', '1'])).split('\n');
  const devices = io[0].trim().split(/\s+/);
  const numbers = io.at(-1).trim().split(/\s+/).map(Number);
  requireThat(devices.every(d => /^disk\d+$/.test(d)) && numbers.every(Number.isFinite)
    && numbers.length === devices.length * 3, 'io_snapshot_shape');
  return { observedAt: now(), processes, pressure: await pressure(), memory,
    io: devices.map((device, i) => ({ device, kiBPerTransfer: numbers[3 * i],
      transfersSinceBoot: numbers[3 * i + 1], megabytesSinceBoot: numbers[3 * i + 2] })),
    ioScope: 'Cumulative since boot, not instantaneous throughput' };
}

function logs() {
  return LOGS.map(file => {
    try {
      const fd = fs.openSync(file, 'r');
      let b;
      try {
        const size = fs.fstatSync(fd).size;
        b = Buffer.alloc(Math.min(MAX, size));
        const read = fs.readSync(fd, b, 0, b.length, Math.max(0, size - b.length));
        b = b.subarray(0, read);
      } finally { fs.closeSync(fd); }
      const categories = {};
      for (const line of b.toString().split('\n')) {
        const timestamp = line.match(/\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z/)?.[0];
        for (const [name, pattern] of [['timeout', /timed?\s*out|timeout/i], ['error', /\bERROR\b/],
          ['db_connected', /SurrealDB.*connected|signin completed/i], ['serving', /listening|serving/i]]) {
          if (!pattern.test(line)) continue;
          categories[name] ??= { count: 0, firstTimestamp: null, lastTimestamp: null };
          categories[name].count++;
          if (timestamp) { categories[name].firstTimestamp ??= timestamp; categories[name].lastTimestamp = timestamp; }
        }
      }
      return { file, bytesRead: b.length, categories, scope: 'One bounded tail; timestamps may predate collection' };
    } catch { return { file, unavailable: true }; }
  });
}

async function deadline(ms, work) {
  requireThat(!stopped, stopped ?? 'stopped');
  const remaining = Math.min(ms, expires - mono(), sessionExpires - mono());
  requireThat(remaining > 0, 'observation_budget_expired');
  let timer;
  try {
    return await Promise.race([work(), new Promise((_, reject) => {
      timer = setTimeout(() => { stop('deadline'); reject(new Stop('deadline')); }, remaining);
    })]);
  } finally { clearTimeout(timer); }
}

function httpGet(port, route, evidence) {
  return new Promise((resolve, reject) => {
    let bytes = 0;
    const bodyBuffer = Buffer.alloc(MAX);
    const request = http.get({ host: '127.0.0.1', port, path: route, agent: false,
      maxHeaderSize: 16384, headers: { 'Accept-Encoding': 'identity', Connection: 'close' } }, response => {
      evidence.httpStatus = response.statusCode;
      evidence.responseBytes = 0;
      response.on('error', () => reject(new Stop('http_transport_error')));
      response.on('data', b => {
        bytes += b.length;
        evidence.responseBytes = bytes;
        if (bytes > MAX) { request.destroy(); reject(new Stop('http_response_size')); return; }
        b.copy(bodyBuffer, bytes - b.length);
      });
      response.on('end', () => {
        try {
          requireThat(response.statusCode === 200, 'http_non_200');
          const body = bodyBuffer.toString('utf8', 0, bytes);
          if (port === 1906) {
            const parsed = JSON.parse(body);
            requireThat(parsed && typeof parsed === 'object' && !Array.isArray(parsed), 'http_shape');
            requireThat(route === '/healthz' ? parsed.status === 'ok' : parsed.status === 'ready', 'http_health_state');
          } else requireThat(body.length === 0, 'db_health_shape');
          resolve({ httpStatus: response.statusCode, responseBytes: bytes });
        } catch (error) { reject(error instanceof Stop ? error : new Stop('http_shape')); }
      });
    });
    active = request;
    request.on('error', () => reject(new Stop('http_transport_error')));
  });
}

// Restricted RFC6455 text client: bounded uncompressed single-frame JSON only.
// Fragmented/binary/control frames are unsupported and terminate the experiment;
// no protocol fallback or retry is attempted. This is not a reusable WS SDK.
class JsonSocket {
  constructor() {
    this.socket = net.createConnection({ host: '127.0.0.1', port: 28000, highWaterMark: 16384 });
    active = this.socket;
    this.socket.on('error', () => { this.failed = true; stop('websocket_transport_error'); });
    this.socket.on('close', () => { if (!this.closing && !stopped) stop('websocket_closed'); });
    this.id = 0;
  }
  async read(n) {
    requireThat(n <= MAX, 'websocket_response_size');
    while (true) {
      requireThat(!this.failed && !this.socket.destroyed && !stopped, 'websocket_closed');
      const b = this.socket.read(n);
      if (b !== null) return b;
      requireThat(!this.socket.readableEnded, 'websocket_closed');
      await new Promise(resolve => {
        const done = () => {
          for (const event of ['readable', 'error', 'end', 'close']) this.socket.off(event, done);
          resolve();
        };
        for (const event of ['readable', 'error', 'end', 'close']) this.socket.once(event, done);
      });
    }
  }
  async open() {
    const key = randomBytes(16).toString('base64');
    this.socket.write(`GET /rpc HTTP/1.1\r\nHost: 127.0.0.1:28000\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: ${key}\r\nSec-WebSocket-Version: 13\r\nSec-WebSocket-Protocol: json\r\n\r\n`);
    let head = '';
    while (!head.endsWith('\r\n\r\n')) {
      requireThat(head.length < 16384, 'websocket_header_size');
      head += (await this.read(1)).toString('latin1');
    }
    const expected = createHash('sha1').update(key + '258EAFA5-E914-47DA-95CA-C5AB0DC85B11').digest('base64');
    const lines = head.split('\r\n');
    const headers = {};
    for (const line of lines.slice(1).filter(Boolean)) {
      const colon = line.indexOf(':');
      requireThat(colon > 0, 'websocket_handshake_shape');
      const name = line.slice(0, colon).toLowerCase();
      requireThat(!Object.hasOwn(headers, name), 'websocket_duplicate_header');
      headers[name] = line.slice(colon + 1).trim();
    }
    requireThat(/^HTTP\/1\.1 101 /.test(lines[0]) && headers['sec-websocket-accept'] === expected
      && headers['sec-websocket-protocol'] === 'json' && !headers['sec-websocket-extensions']
      && headers.upgrade?.toLowerCase() === 'websocket'
      && headers.connection?.toLowerCase().split(/\s*,\s*/).includes('upgrade'), 'websocket_handshake_rejected');
    return { responseBytes: Buffer.byteLength(head), codec: 'json' };
  }
  async rpc(method, params, evidence) {
    requireThat(!this.failed && !this.socket.destroyed && !stopped, 'websocket_closed');
    const id = ++this.id;
    const payload = Buffer.from(JSON.stringify({ id, method, params }));
    requireThat(payload.length < 65536, 'request_size');
    const mask = randomBytes(4);
    const short = payload.length < 126;
    const head = Buffer.alloc(short ? 6 : 8);
    head[0] = 0x81; head[1] = 0x80 | (short ? payload.length : 126);
    if (!short) head.writeUInt16BE(payload.length, 2);
    mask.copy(head, short ? 2 : 4);
    for (let i = 0; i < payload.length; i++) payload[i] ^= mask[i % 4];
    this.socket.write(Buffer.concat([head, payload]));
    const header = await this.read(2);
    requireThat(header[0] === 0x81 && (header[1] & 0x80) === 0, 'unsupported_websocket_frame');
    let length = header[1] & 0x7f;
    if (length === 126) length = (await this.read(2)).readUInt16BE();
    else if (length === 127) {
      const wide = (await this.read(8)).readBigUInt64BE();
      requireThat(wide <= BigInt(MAX), 'websocket_response_size');
      length = Number(wide);
    }
    requireThat(length > 0 && length <= MAX, 'websocket_response_size');
    const raw = await this.read(length);
    evidence.responseBytes = length;
    let message;
    try { message = JSON.parse(new TextDecoder('utf-8', { fatal: true }).decode(raw)); }
    catch { fail('rpc_json_shape'); }
    requireThat(message && typeof message === 'object' && message.id === id, 'rpc_identity_shape');
    evidence.rpcStatus = Object.hasOwn(message, 'error') ? 'ERR' : 'OK';
    requireThat(evidence.rpcStatus === 'OK', 'rpc_error');
    requireThat(Object.hasOwn(message, 'result'), 'rpc_result_missing');
    return { result: message.result, responseBytes: length, rpcStatus: 'OK' };
  }
  close() { this.closing = true; this.socket.destroy(); }
}

function queryResult(reply, index, evidence) {
  requireThat(Array.isArray(reply.result) && reply.result.length === 1, 'statement_shape');
  const statement = reply.result[0];
  requireThat(statement !== null && typeof statement === 'object' && !Array.isArray(statement), 'statement_shape');
  evidence.statementStatus = statement?.status === 'OK' ? 'OK' : statement?.status === 'ERR' ? 'ERR' : 'unexpected';
  const output = { responseBytes: reply.responseBytes, rpcStatus: 'OK', statementStatus: evidence.statementStatus };
  if (statement.time !== undefined) {
    requireThat(typeof statement.time === 'string' && /^\d+(?:\.\d+)?(?:ns|µs|μs|us|ms|s)$/.test(statement.time), 'statement_time_shape');
    output.serverReportedStatementTime = statement.time;
    evidence.serverReportedStatementTime = statement.time;
  }
  requireThat(statement?.status === 'OK', 'statement_error');
  if (index === 2 || index === 3) {
    requireThat(Array.isArray(statement.result) && statement.result.length <= (index === 2 ? 1 : 32)
      && statement.result.every(row => row && typeof row === 'object'
        && Object.keys(row).length === 1 && Object.hasOwn(row, 'id')), 'row_projection_shape');
    output.rowCount = statement.result.length;
  } else {
    requireThat(statement.result === true, 'scalar_result_shape');
    output.scalar = true;
  }
  return output;
}

async function collect() {
  const outputPath = path.join(HERE, 'evidence/observations.json');
  // Exclusive receipt is also the no-rerun marker; never delete it automatically.
  const fd = fs.openSync(outputPath, 'wx', 0o600);
  const report = { startedAt: now(), state: 'started', limits: { rounds: 2, totalMs: 360000,
    healthMs: 5000, readinessMs: 35000, sessionMs: 120000, setupMs: 15000, queryMs: 10000,
    responseBytes: MAX, logTailBytesPerFile: MAX, evidenceBytes: FILE_MAX },
    baselineIdentity: 'identity.json', identityChecks: [], snapshots: [], operations: [],
    stopReason: null, cancellation: 'Closing the client does not prove server cancellation' };
  function save() {
    const serialized = JSON.stringify(report, null, 2) + '\n';
    requireThat(Buffer.byteLength(serialized) <= FILE_MAX, 'evidence_file_size');
    fs.writeSync(fd, serialized, 0, 'utf8');
    fs.ftruncateSync(fd, Buffer.byteLength(serialized));
    fs.fsyncSync(fd);
  }
  save();
  let session;
  let sessionTimer;
  const interrupt = () => stop('operator_interruption');
  process.on('SIGINT', interrupt); process.on('SIGTERM', interrupt);
  const begin = mono();
  expires = begin + 360000;
  const globalTimer = setTimeout(() => stop('observation_budget_expired'), 360000);
  const names = ['db_health', 'uar_health_before', 'socket_open', 'signin', 'use',
    'scalar_before_1', 'scalar_before_2', 'skills_limit_1', 'skills_limit_32', 'scalar_after',
    'uar_ready', 'uar_health_after'];
  for (let round = 1; round <= 2; round++) for (const name of names) report.operations.push({ round, name, status: 'skipped', reason: 'not_reached' });
  try {
    report.identityChecks.push(await identity());
    for (const svc of BASE.services) requireThat(await digest(svc.executable) === svc.sha256, 'executable_digest_changed');
    requireThat(await local('/usr/bin/git', ['rev-parse', 'HEAD']) === BASE.sourceHead, 'source_head_changed');
    const auth = credentials();
    for (let round = 1; round <= 2; round++) {
      report.snapshots.push({ round, edge: 'before', ...await snapshot() });
      requireThat(report.snapshots.at(-1).pressure === 1, 'host_memory_pressure');
      for (const operation of report.operations.filter(o => o.round === round)) {
        requireThat(!stopped, stopped ?? 'stopped');
        report.identityChecks.push(await identity());
        const pressureValue = await pressure();
        report.lastPressureCheck = { observedAt: now(), value: pressureValue };
        requireThat(pressureValue === 1, 'host_memory_pressure');
        requireThat(!stopped, stopped ?? 'stopped');
        requireThat(mono() < expires && mono() < sessionExpires, 'observation_budget_expired');
        const { name } = operation;
        const start = mono();
        operation.startedAt = now(); operation.status = 'attempted'; delete operation.reason;
        const cap = name === 'uar_ready' ? 35000 : ['socket_open', 'signin', 'use'].includes(name) ? 15000
          : ['db_health', 'uar_health_before', 'uar_health_after'].includes(name) ? 5000 : 10000;
        operation.deadlineMs = cap;
        try {
          const result = await deadline(cap, async () => {
            if (name === 'db_health') return httpGet(28000, '/health', operation);
            if (name === 'uar_health_before' || name === 'uar_health_after') return httpGet(1906, '/healthz', operation);
            if (name === 'uar_ready') return httpGet(1906, '/readyz', operation);
            if (name === 'socket_open') {
              sessionExpires = mono() + 120000;
              sessionTimer = setTimeout(() => stop('session_deadline'), 120000);
              session = new JsonSocket(); return session.open();
            }
            if (name === 'signin') {
              const reply = await session.rpc('signin', [auth], operation);
              const token = reply.result;
              requireThat(token === null || typeof token === 'string'
                || (token && typeof token === 'object' && typeof token.token === 'string'), 'signin_shape');
              return { responseBytes: reply.responseBytes, rpcStatus: reply.rpcStatus };
            }
            if (name === 'use') {
              const reply = await session.rpc('use', ['uar', 'runtime_openai_1536'], operation);
              requireThat(reply.result === null, 'use_shape');
              return { responseBytes: reply.responseBytes, rpcStatus: reply.rpcStatus };
            }
            const index = names.indexOf(name) - 5;
            operation.query = QUERIES[index];
            const result = queryResult(await session.rpc('query', [QUERIES[index], {}], operation), index, operation);
            if (index === 4) { session.close(); session = null; sessionExpires = Infinity; clearTimeout(sessionTimer); }
            return result;
          });
          requireThat(!stopped && mono() - start <= cap && mono() <= expires, 'deadline');
          Object.assign(operation, result, { status: 'ok' });
        } catch (error) { operation.status = 'failed'; operation.reason = safeCode(error); throw error; }
        finally { operation.clientElapsedMs = Math.round((mono() - start) * 1000) / 1000; save(); }
      }
      report.snapshots.push({ round, edge: 'after', ...await snapshot() });
      report.identityChecks.push(await identity());
      requireThat(report.snapshots.at(-1).pressure === 1, 'host_memory_pressure');
      requireThat(!stopped && mono() <= expires, stopped ?? 'observation_budget_expired');
    }
    report.state = 'sample_complete';
  } catch (error) {
    stop(stopped ?? safeCode(error)); report.state = 'stopped'; report.stopReason = stopped;
  } finally {
    session?.close(); active?.destroy(); clearTimeout(globalTimer); clearTimeout(sessionTimer);
    report.collectionWallMsBeforeFinalPassive = Math.round(mono() - begin);
    process.off('SIGINT', interrupt); process.off('SIGTERM', interrupt);
    try { report.snapshots.push({ edge: 'final_passive', ...await snapshot() }); }
    catch (error) { report.finalSnapshotUnavailable = safeCode(error); }
    report.logTails = logs();
    for (const operation of report.operations.filter(o => o.status === 'skipped')) operation.reason = report.stopReason ?? 'not_reached';
    report.finishedAt = now(); report.totalWallMs = Math.round(mono() - begin);
    report.attemptedOperations = report.operations.filter(o => o.status !== 'skipped').length;
    report.skippedOperations = report.operations.filter(o => o.status === 'skipped').length;
    save(); fs.closeSync(fd);
  }
  console.log(JSON.stringify({ state: report.state, stopReason: report.stopReason,
    attemptedOperations: report.attemptedOperations, evidence: outputPath }));
}

try {
  requireThat(process.platform === 'darwin', 'unsupported_platform');
  if (process.argv.length === 3 && process.argv[2] === '--prepare') {
    console.log(JSON.stringify({ mode: 'passive_preparation', node: process.version,
      platform: process.platform, builtinModulesOnly: true, pressure: await pressure(),
      activeRequests: 0, queries: QUERIES, collectorExecutionNotValidated: true }));
  } else if (process.argv.length === 3 && process.argv[2] === '--collect') await collect();
  else console.log('Use --prepare for passive capabilities; --collect only in approved task2.1.');
} catch (error) {
  // Never echo native error objects: they can contain buffers or command output.
  console.log(JSON.stringify({ state: 'not_started_or_local_failure', reason: safeCode(error) }));
  process.exitCode = 1;
}
