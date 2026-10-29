import assert from 'node:assert/strict';
import { execFileSync, spawn } from 'node:child_process';
import { createHash, randomBytes, randomUUID } from 'node:crypto';
import { mkdir, open, readFile, rm, writeFile } from 'node:fs/promises';
import { createServer } from 'node:net';
import { homedir } from 'node:os';
import { join, resolve } from 'node:path';
import { processEnvironment } from '../behaviors/support/restart-owned-uar.mjs';

// Host fixture for C52: a disposable UAR on its own empty, host-writable store.
// The owned UAR, its store and its proxy are never touched. Credentials stay in
// process memory and the fixture's private config; they never enter evidence.

const ROOT = resolve('target/c52-disposable');
const NAMESPACE = 'prompt';
const DATABASE = 'prompt';
const READY_TIMEOUT_MS = 180_000;
const EXIT_TIMEOUT_MS = 60_000;
// The Surreal server must match the pinned server version (versions.toml surrealdb_server).
const SURREAL_BIN = join(homedir(), '.prometheus/bin/surreal');
const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/;

const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const sleep = ms => new Promise(done => setTimeout(done, ms));

function freePort() {
  return new Promise((done, fail) => {
    const server = createServer();
    server.once('error', fail);
    server.listen(0, '127.0.0.1', () => {
      const { port } = server.address();
      server.close(() => done(port));
    });
  });
}

async function waitReady(url, child) {
  const deadline = Date.now() + READY_TIMEOUT_MS;
  while (Date.now() < deadline) {
    if (child && (child.exitCode !== null || child.signalCode !== null)) {
      throw new Error(`Exited before ready (${child.exitCode ?? child.signalCode}): ${url}`);
    }
    try {
      const response = await fetch(url, { signal: AbortSignal.timeout(5_000) });
      if (response.ok) return;
    } catch { /* not listening yet */ }
    await sleep(500);
  }
  throw new Error(`Not ready within ${READY_TIMEOUT_MS} ms: ${url}`);
}

async function launch(binary, args, { cwd, env, log }) {
  const handle = await open(log, 'a', 0o600);
  const child = spawn(binary, args, { cwd, env, detached: true, stdio: ['ignore', handle.fd, handle.fd] });
  await new Promise((done, fail) => { child.once('spawn', done); child.once('error', fail); });
  await handle.close();
  return child;
}

async function stop(child) {
  if (!child || child.exitCode !== null || child.signalCode !== null) return true;
  const exited = new Promise(done => child.once('exit', () => done(true)));
  child.kill('SIGTERM');
  return Promise.race([exited, sleep(EXIT_TIMEOUT_MS).then(() => false)]);
}

async function pinnedSurreal() {
  const pin = /^surrealdb_server\s*=\s*"([^"]+)"/m.exec(await readFile('versions.toml', 'utf8'))?.[1];
  assert.ok(pin, 'versions.toml has no surrealdb_server pin');
  const version = execFileSync(SURREAL_BIN, ['version'], { encoding: 'utf8', timeout: 10_000 }).trim();
  assert.equal(version.split(' ')[0], pin, `Surreal server ${version} does not match the pinned ${pin}`);
  return version;
}

function surrealSql(fx, query, scoped = true) {
  const scope = scoped ? ['--namespace', NAMESPACE, '--database', DATABASE] : [];
  return execFileSync(SURREAL_BIN, ['sql', '--endpoint', `ws://127.0.0.1:${fx.ports.surreal}`, ...scope,
    '--username', 'root', '--auth-level', 'root', '--json', '--hide-welcome'], {
    input: query, encoding: 'utf8', timeout: 30_000, env: { PATH: process.env.PATH, HOME: process.env.HOME, SURREAL_PASS: fx.password },
  });
}

async function startUar(fx, label) {
  const log = join(fx.dir, `uar-${label}.log`);
  fx.uar = await launch(fx.binary, ['--config', fx.configPath], { cwd: fx.dir, env: fx.uarEnv, log });
  await waitReady(`${fx.uarUrl}/health`, fx.uar);
  return { label, pid: fx.uar.pid, log };
}

/** Starts a fresh Surreal server, a UAR on it and an owner proxy in front of it. */
export async function startDisposable(record, ttlSecs) {
  assert.ok(Number.isInteger(ttlSecs) && ttlSecs > 0);
  const owned = JSON.parse(await readFile('target/prompt-macos-launch.json', 'utf8'));
  const proxy = JSON.parse(await readFile('target/manual-identity-proxy/launch.json', 'utf8'));
  const id = randomUUID();
  const fx = { id, dir: join(ROOT, id), password: randomBytes(24).toString('hex'), binary: owned.binary,
    ports: { surreal: await freePort(), http: await freePort(), grpc: await freePort(), proxy: await freePort() } };
  fx.uarUrl = `http://127.0.0.1:${fx.ports.http}`;
  fx.baseUrl = `http://127.0.0.1:${fx.ports.proxy}`;
  const surrealVersion = await pinnedSurreal();
  await mkdir(join(fx.dir, 'ingest'), { recursive: true, mode: 0o700 });
  try {
    await provision(fx, record, owned, proxy, ttlSecs, surrealVersion);
  } catch (error) {
    // A failed startup must not leave the Surreal server, UAR or proxy running.
    // Keep the UAR's error lines first: teardown deletes the logs.
    record.startup_error_lines = await logLines(join(fx.dir, 'uar-initial.log'), '"level":"ERROR"')
      .then(lines => lines.map(line => line.slice(0, 1_000)), () => []);
    await stopDisposable(fx, record).catch(() => undefined);
    throw error;
  }
  return fx;
}

async function provision(fx, record, owned, proxy, ttlSecs, surrealVersion) {
  const { id } = fx;
  fx.surreal = await launch(SURREAL_BIN, ['start', '--no-banner', '--bind', `127.0.0.1:${fx.ports.surreal}`, `surrealkv://${join(fx.dir, 'store')}`], {
    cwd: fx.dir, env: { PATH: process.env.PATH, HOME: process.env.HOME, SURREAL_USER: 'root', SURREAL_PASS: fx.password },
    log: join(fx.dir, 'surreal.log') });
  await waitReady(`http://127.0.0.1:${fx.ports.surreal}/health`, fx.surreal);
  // A fresh server has no namespaces; the store's namespace and database do not exist yet.
  const emptyStore = JSON.parse(surrealSql(fx, 'INFO FOR ROOT;', false));

  const config = JSON.parse(await readFile(owned.config, 'utf8'));
  config.server = { ...config.server, port: fx.ports.http, grpc_port: fx.ports.grpc, ingest_directory: join(fx.dir, 'ingest') };
  config.persistence = { ...config.persistence, database_url: `ws://127.0.0.1:${fx.ports.surreal}`,
    surreal_user: 'root', surreal_pass: fx.password, surreal_ns: NAMESPACE, surreal_db: DATABASE };
  config.sessions = { persisted_ttl_secs: ttlSecs };
  config.service_instance = { ...config.service_instance, instance_id: `uar-c52-${id.slice(0, 8)}` };
  fx.configPath = join(fx.dir, 'config.private.json');
  const configBytes = Buffer.from(JSON.stringify(config, null, 2) + '\n');
  await writeFile(fx.configPath, configBytes, { flag: 'wx', mode: 0o600 });

  fx.uarEnv = processEnvironment(owned.pid);
  const initial = await startUar(fx, 'initial');
  const proxyEnv = { ...processEnvironment(proxy.pid, 7), PATH: process.env.PATH };
  fx.proxy = await launch(proxy.binary, ['--config', fx.configPath, '--upstream', fx.uarUrl, '--listen', `127.0.0.1:${fx.ports.proxy}`],
    { cwd: fx.dir, env: proxyEnv, log: join(fx.dir, 'proxy.log') });
  await waitReady(`${fx.baseUrl}/health`, fx.proxy);

  Object.assign(record, { fixture_id: id, store: join(fx.dir, 'store'), surreal_pid: fx.surreal.pid, proxy_pid: fx.proxy.pid,
    base_url: fx.baseUrl, runtime_url: fx.uarUrl, binary_sha256: sha(await readFile(fx.binary)), config_sha256: sha(configBytes),
    persisted_ttl_secs: ttlSecs, surreal_version: surrealVersion, empty_store_before_start: emptyStore, startups: [initial] });
}

/** Repeat startup: stops only the disposable UAR and starts it again on the same store. */
export async function restartDisposable(fx, record, label) {
  const previous = fx.uar.pid;
  assert.equal(await stop(fx.uar), true, 'Disposable UAR did not exit');
  const startup = await startUar(fx, label);
  record.startups.push({ ...startup, previous_pid: previous });
  return startup;
}

/** Host-only aging of one captured session's persisted last activity; no clock is changed. */
export function ageSession(fx, sessionId, ownerId, lastActivity) {
  assert.match(sessionId, UUID);
  assert.ok(typeof ownerId === 'string' && ownerId.length > 0 && !/["\\\n]/.test(ownerId));
  assert.ok(Number.isFinite(Date.parse(lastActivity)));
  const output = surrealSql(fx, `UPDATE sessions SET last_activity = ${JSON.stringify(lastActivity)} `
    + `WHERE logical_id = ${JSON.stringify(sessionId)} AND owner_id = ${JSON.stringify(ownerId)} RETURN AFTER;`);
  const rows = JSON.parse(output).flat();
  assert.equal(rows.length, 1, 'Exactly one owned session record must be aged');
  assert.equal(rows[0].last_activity, lastActivity);
  return { session_id: sessionId, last_activity: lastActivity, rows_updated: rows.length };
}

/** Log lines of one disposable startup that contain the given message. */
export async function logLines(path, message) {
  return (await readFile(path, 'utf8')).split('\n').filter(line => line.includes(message));
}

/** Stops the proxy, UAR and Surreal server, then deletes the disposable directory. */
export async function stopDisposable(fx, record) {
  const stopped = {};
  for (const name of ['proxy', 'uar', 'surreal']) stopped[name] = await stop(fx[name]);
  record.teardown = { stopped };
  assert.ok(fx.dir.startsWith(`${ROOT}/`));
  if (Object.values(stopped).every(Boolean)) {
    await rm(fx.dir, { recursive: true, force: true });
    record.teardown.directory_removed = true;
  }
  fx.password = undefined;
  return record.teardown;
}
