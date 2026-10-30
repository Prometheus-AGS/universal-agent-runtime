// Host fixture for #441: a dedicated UAR on a remote SurrealDB server.
//
// This is external infrastructure, not test logic. It generates the SurrealDB
// root password and the identity signing secret per run and passes them only
// through child-process environments. Neither is written to disk, logged,
// returned to the steps, or placed on a command line.
import { execFileSync, spawn } from 'node:child_process';
import { randomBytes, randomUUID } from 'node:crypto';
import { mkdir, open, readFile, writeFile } from 'node:fs/promises';
import { createServer } from 'node:net';
import { resolve } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';

// Pinned in versions.toml (`surrealdb_image`).
const SURREAL_IMAGE = 'surrealdb/surrealdb:v3.3.0@sha256:681c6c22c287421b5c7d99e0fde79b6e0d32c36c1ddeaab2762a1661cb04cd20';
const TARGET = process.env.UAR_REMOTE_SURREAL_TARGET ?? 'target/uar-team-concurrency-repair/debug';
const STARTUP_DEADLINE_MS = 180_000;
const STOP_DEADLINE_MS = 45_000;

function freePort() {
  return new Promise((resolvePort, reject) => {
    const server = createServer();
    server.once('error', reject);
    server.listen(0, '127.0.0.1', () => {
      const { port } = server.address();
      server.close(() => resolvePort(port));
    });
  });
}

function alive(pid) {
  try { process.kill(pid, 0); return true; }
  catch (error) { if (error.code === 'ESRCH') return false; throw error; }
}

function docker(args) {
  return execFileSync('docker', args, { encoding: 'utf8', timeout: 120_000, stdio: ['ignore', 'pipe', 'pipe'] }).trim();
}

async function launch(binary, args, cwd, env, logPath) {
  const log = await open(logPath, 'wx', 0o600);
  try {
    const child = spawn(binary, args, { cwd, env, detached: true, stdio: ['ignore', log.fd, log.fd] });
    await new Promise((resolveSpawn, reject) => { child.once('spawn', resolveSpawn); child.once('error', reject); });
    child.unref();
    return child.pid;
  } finally { await log.close(); }
}

// `exited` is checked outside the probe so a dead process fails at once.
async function waitFor(label, check, deadlineMs, exited = () => false) {
  const deadline = Date.now() + deadlineMs;
  while (Date.now() < deadline) {
    if (exited()) throw new Error(`${label}: process exited before it was ready`);
    if (await check().catch(() => false)) return;
    await delay(500);
  }
  throw new Error(`${label} deadline exceeded`);
}

async function terminate(pid) {
  if (!pid || !alive(pid)) return true;
  process.kill(pid, 'SIGTERM');
  const deadline = Date.now() + STOP_DEADLINE_MS;
  while (alive(pid) && Date.now() < deadline) await delay(250);
  return !alive(pid);
}

/**
 * Start SurrealDB, a dedicated UAR, and its identity proxy. `holder.handle` is
 * set before anything starts, so a caller can always call `stop()`. The handle
 * carries non-secret values only.
 */
export async function startRemoteSurrealRuntime(record, holder) {
  const id = randomUUID();
  const root = resolve(`target/remote-surreal-restart/${id}`);
  const data = resolve(root, 'surreal-data');
  await mkdir(data, { recursive: true, mode: 0o777 });
  const [surrealPort, uarPort, grpcPort, proxyPort] = [await freePort(), await freePort(), await freePort(), await freePort()];
  const container = `uar-bdd-surreal-${id.slice(0, 8)}`;
  const surrealPass = randomBytes(24).toString('base64url');
  const jwtSecret = randomBytes(48).toString('base64url');
  const settingsAdminKey = randomBytes(32).toString('base64url');
  const handle = { id, root, container, uarPid: null, proxyPid: null,
    base: new URL(`http://127.0.0.1:${proxyPort}`), runtime: new URL(`http://127.0.0.1:${uarPort}`),
    uarLog: resolve(root, 'uar.log'), proxyLog: resolve(root, 'proxy.log'), containerCreated: false };
  holder.handle = handle;
  handle.stop = async () => {
    const errors = [];
    for (const [name, pid] of [['proxy', handle.proxyPid], ['uar', handle.uarPid]]) {
      if (!(await terminate(pid))) errors.push(`${name} did not stop within ${STOP_DEADLINE_MS} ms`);
    }
    if (handle.containerCreated) {
      try { docker(['rm', '-f', container]); } catch (error) { errors.push(`container removal: ${error.message}`); }
    }
    return errors;
  };
  Object.assign(record, { fixture_root: root, surreal_image: SURREAL_IMAGE, container,
    surreal_url: `http://127.0.0.1:${surrealPort}`, runtime_url: handle.runtime.href, base_url: handle.base.href,
    credentials_generated_by_fixture: true, credentials_seen_by_steps: false });

  // Values come from this process's environment, so they never reach argv.
  execFileSync('docker', ['run', '-d', '--name', container, '-p', `127.0.0.1:${surrealPort}:8000`,
    '-v', `${data}:/data`, '-e', 'SURREAL_USER', '-e', 'SURREAL_PASS', SURREAL_IMAGE,
    'start', '--bind', '0.0.0.0:8000', 'surrealkv:/data/uar.db'],
  { env: { PATH: process.env.PATH, HOME: process.env.HOME, SURREAL_USER: 'root', SURREAL_PASS: surrealPass },
    timeout: 300_000, stdio: ['ignore', 'pipe', 'pipe'] });
  handle.containerCreated = true;
  await waitFor('SurrealDB health', async () => (await fetch(`http://127.0.0.1:${surrealPort}/health`)).ok, 60_000);

  const config = resolve(root, 'config.json');
  await writeFile(config, JSON.stringify({
    server: { host: '127.0.0.1', port: uarPort, grpc_port: grpcPort, log_format: 'json', cors_allowed_origins: [],
      ingest_directory: resolve(root, 'ingest') },
    security: { jwt_required: false },
    persistence: { provider: 'surreal', database_url: record.surreal_url, vector_dimension: 384, external_cache_enabled: false },
    memory: { enabled: false },
  }, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  const base = { HOME: process.env.HOME, PATH: process.env.PATH,
    UAR_MODELS_DIR: resolve('src/uar/runtime/matching/models') };
  handle.uarPid = await launch(resolve(TARGET, 'universal-agent-runtime'), ['--config', config], root,
    { ...base, UAR_PERSISTENCE__SURREAL_USER: 'root', UAR_PERSISTENCE__SURREAL_PASS: surrealPass,
      UAR_SECURITY__JWT_SECRET: jwtSecret, UAR_SECURITY__SETTINGS_ADMIN_KEY: settingsAdminKey }, handle.uarLog);
  await waitFor(`UAR readiness (log ${handle.uarLog})`,
    async () => (await fetch(new URL('/readyz', handle.runtime))).status === 200,
    STARTUP_DEADLINE_MS, () => !alive(handle.uarPid));
  handle.proxyPid = await launch(resolve(TARGET, 'uar-jwt-proxy'),
    ['--config', config, '--upstream', handle.runtime.href, '--listen', handle.base.host], root,
    { ...base, UAR_SECURITY__JWT_SECRET: jwtSecret, PROXY_JWT_SUB: 'bdd-remote-surreal' }, handle.proxyLog);
  await waitFor(`identity proxy (log ${handle.proxyLog})`, async () => (await fetch(new URL('/healthz', handle.base))).ok,
    60_000, () => !alive(handle.proxyPid));
  Object.assign(record, { uar_pid: handle.uarPid, proxy_pid: handle.proxyPid, uar_log: handle.uarLog });

  handle.restartSurreal = () => docker(['restart', '--time', '10', container]);
  handle.uarAlive = () => alive(handle.uarPid);
  handle.logMessages = async () => (await readFile(handle.uarLog, 'utf8')).split('\n').flatMap(line => {
    try { return [JSON.parse(line).fields?.message].filter(Boolean); } catch { return []; }
  });
  return handle;
}
