// Host fixture: a dedicated UAR on its own database server, for scenarios the
// shared owned runtime cannot exercise (a remote SurrealDB, PostgreSQL 18).
//
// This is external infrastructure, not test logic. It generates every database
// password and the identity signing secret per run and passes them only through
// child-process environments. None is written to disk, logged, returned to the
// steps, or placed on a command line.
import { execFileSync, spawn } from 'node:child_process';
import { randomBytes, randomUUID } from 'node:crypto';
import { mkdir, open, readFile, writeFile } from 'node:fs/promises';
import { createServer } from 'node:net';
import { resolve } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';

// Pinned in versions.toml (`surrealdb_image`).
const SURREAL_IMAGE = 'surrealdb/surrealdb:v3.3.0@sha256:681c6c22c287421b5c7d99e0fde79b6e0d32c36c1ddeaab2762a1661cb04cd20';
// flint-forge PostgreSQL 18 data-plane image (Know-Me-Tools/flint-forge images/postgres18).
const POSTGRES_IMAGE = 'tribehealth/flint-forge-pg:18@sha256:55c12cbf613346315a48a5c465e804c1ca61ccd1be283db4683ca7c690a9069c';
const UAR_DATABASE_INIT = resolve('scripts/acceptance/fixtures/flint-forge-uar-db.sh');
const TARGETS = {
  surreal: process.env.UAR_SURREAL_TARGET ?? 'target/uar-team-concurrency-repair/debug',
  postgres: process.env.UAR_POSTGRES_TARGET ?? 'target/deps-postgres/debug',
};
const PROXY_BINARY = resolve(process.env.UAR_PROXY_TARGET ?? 'target/uar-team-concurrency-repair/debug', 'uar-jwt-proxy');
// Keyless deterministic LLM (src/bin/stub-llm.rs), wired the way tests/bdd/playwright.config.ts does.
const STUB_LLM_BINARY = resolve(process.env.UAR_STUB_LLM_TARGET ?? 'target/deps-postgres/debug', 'stub-llm');
const STUB_LLM_FIXTURES = resolve('scripts/acceptance/fixtures/dedicated-runtime-stub-llm.json');
export const STUB_LLM_MODEL = 'openai/gpt-5.4-mini';
export const STUB_LLM_PROMPT = 'Postgres session probe: reply with SESSION-OK.';
const STARTUP_DEADLINE_MS = 180_000;
const DATABASE_DEADLINE_MS = 120_000;
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

function containerRunning(container) {
  try { return docker(['inspect', '-f', '{{.State.Running}}', container]) === 'true'; } catch { return false; }
}

// Plain objects merge recursively; arrays and scalars replace.
function merge(base, overrides) {
  const out = { ...base };
  for (const [key, value] of Object.entries(overrides ?? {})) {
    const plain = value && typeof value === 'object' && !Array.isArray(value);
    out[key] = plain && base[key] && typeof base[key] === 'object' ? merge(base[key], value) : value;
  }
  return out;
}

async function startSurreal(handle, secrets, port) {
  const data = resolve(handle.root, 'surreal-data');
  await mkdir(data, { recursive: true, mode: 0o777 });
  execFileSync('docker', ['run', '-d', '--name', handle.container, '-p', `127.0.0.1:${port}:8000`,
    '-v', `${data}:/data`, '-e', 'SURREAL_USER', '-e', 'SURREAL_PASS', SURREAL_IMAGE,
    'start', '--bind', '0.0.0.0:8000', 'surrealkv:/data/uar.db'],
  { env: { PATH: process.env.PATH, HOME: process.env.HOME, SURREAL_USER: 'root', SURREAL_PASS: secrets.database },
    timeout: 300_000, stdio: ['ignore', 'pipe', 'pipe'] });
  handle.containerCreated = true;
  await waitFor('SurrealDB health', async () => (await fetch(`http://127.0.0.1:${port}/health`)).ok, 60_000);
  return {
    persistence: { provider: 'surreal', database_url: `http://127.0.0.1:${port}` },
    env: { UAR_PERSISTENCE__SURREAL_USER: 'root', UAR_PERSISTENCE__SURREAL_PASS: secrets.database },
  };
}

// The flint-forge image creates its extensions in POSTGRES_DB and pins pg_cron
// to `flint`, so the server keeps `flint` and UAR gets its own `uar` database.
async function startPostgres(handle, secrets, port) {
  execFileSync('docker', ['run', '-d', '--name', handle.container, '-p', `127.0.0.1:${port}:5432`,
    '-e', 'POSTGRES_PASSWORD', '-e', 'POSTGRES_DB=flint', '-e', 'UAR_DB_PASSWORD',
    '-v', `${UAR_DATABASE_INIT}:/docker-entrypoint-initdb.d/50-uar-database.sh:ro`, POSTGRES_IMAGE],
  { env: { PATH: process.env.PATH, HOME: process.env.HOME, POSTGRES_PASSWORD: secrets.superuser, UAR_DB_PASSWORD: secrets.database },
    timeout: 300_000, stdio: ['ignore', 'pipe', 'pipe'] });
  handle.containerCreated = true;
  // The init phase runs a socket-only server; TCP readiness means init finished.
  await waitFor(`PostgreSQL readiness (docker logs ${handle.container})`,
    async () => docker(['exec', handle.container, 'pg_isready', '-h', '127.0.0.1', '-d', 'uar']).includes('accepting connections'),
    DATABASE_DEADLINE_MS, () => !containerRunning(handle.container));
  return {
    persistence: { provider: 'postgres' },
    env: { UAR_PERSISTENCE__DATABASE_URL: `postgres://uar:${encodeURIComponent(secrets.database)}@127.0.0.1:${port}/uar` },
  };
}

/**
 * Start a database server, a dedicated UAR, and its identity proxy.
 * `holder.handle` is set before anything starts, so a caller can always call
 * `stop()`. The handle carries non-secret values only.
 *
 * options.backend: 'surreal' | 'postgres'
 * options.label: names the run directory under target/
 * options.config: overrides merged into the generated UAR config
 * options.env: extra non-secret environment for UAR
 * options.stubLlm: start the keyless stub LLM and point UAR's default model at it
 */
export async function startDedicatedRuntime(record, holder, options) {
  const { backend, label = backend, config: overrides = {}, env: extraEnv = {}, stubLlm = false } = options;
  if (!(backend in TARGETS)) throw new Error(`Unsupported backend ${backend}`);
  const id = randomUUID();
  const root = resolve(`target/dedicated-runtime/${label}/${id}`);
  await mkdir(root, { recursive: true });
  const [databasePort, uarPort, grpcPort, proxyPort] = [await freePort(), await freePort(), await freePort(), await freePort()];
  const secrets = {
    database: randomBytes(24).toString('base64url'),
    superuser: randomBytes(24).toString('base64url'),
    jwt: randomBytes(48).toString('base64url'),
  };
  const handle = { id, root, backend, container: `uar-bdd-${backend}-${id.slice(0, 8)}`, uarPid: null, proxyPid: null, stubPid: null,
    base: new URL(`http://127.0.0.1:${proxyPort}`), runtime: new URL(`http://127.0.0.1:${uarPort}`),
    uarLogs: [], proxyLog: resolve(root, 'proxy.log'), containerCreated: false, launches: 0 };
  holder.handle = handle;
  handle.stop = async () => {
    const errors = [];
    for (const [name, pid] of [['proxy', handle.proxyPid], ['uar', handle.uarPid], ['stub-llm', handle.stubPid]]) {
      if (!(await terminate(pid))) errors.push(`${name} did not stop within ${STOP_DEADLINE_MS} ms`);
    }
    if (handle.containerCreated) {
      try { docker(['rm', '-f', handle.container]); } catch (error) { errors.push(`container removal: ${error.message}`); }
    }
    return errors;
  };
  Object.assign(record, { fixture_root: root, backend, container: handle.container,
    database_image: backend === 'postgres' ? POSTGRES_IMAGE : SURREAL_IMAGE,
    runtime_url: handle.runtime.href, base_url: handle.base.href,
    credentials_generated_by_fixture: true, credentials_seen_by_steps: false });

  const database = backend === 'postgres'
    ? await startPostgres(handle, secrets, databasePort)
    : await startSurreal(handle, secrets, databasePort);
  record.database_url_host = `127.0.0.1:${databasePort}`;

  // The stub key is a fixed placeholder the stub ignores, not a credential.
  const llmEnv = {};
  if (stubLlm) {
    const stubPort = await freePort();
    handle.stubPid = await launch(STUB_LLM_BINARY, [STUB_LLM_FIXTURES], root,
      { HOME: process.env.HOME, PATH: process.env.PATH, STUB_LLM_PORT: String(stubPort) }, resolve(root, 'stub-llm.log'));
    await waitFor('stub LLM', async () => (await fetch(`http://127.0.0.1:${stubPort}/v1/models`)).ok,
      30_000, () => !alive(handle.stubPid));
    Object.assign(llmEnv, { UAR_LLM__BASE_URL: `http://127.0.0.1:${stubPort}/v1`,
      UAR_LLM__API_KEY: 'test-stub-key', UAR_LLM__MODEL: STUB_LLM_MODEL });
    record.stub_llm = { model: STUB_LLM_MODEL, fixtures: STUB_LLM_FIXTURES };
  }

  const configPath = resolve(root, 'config.json');
  let currentConfig = merge({
    server: { host: '127.0.0.1', port: uarPort, grpc_port: grpcPort, log_format: 'json', cors_allowed_origins: [],
      ingest_directory: resolve(root, 'ingest') },
    // Same as the shared owned runtime: tests never send an admin key.
    security: { jwt_required: false, settings_mutation_auth_required: false },
    persistence: { ...database.persistence, vector_dimension: 384, external_cache_enabled: false },
    memory: { enabled: false },
  }, overrides);
  const uarEnv = { HOME: process.env.HOME, PATH: process.env.PATH,
    UAR_MODELS_DIR: resolve('src/uar/runtime/matching/models'), ...llmEnv, ...extraEnv, ...database.env,
    UAR_SECURITY__JWT_SECRET: secrets.jwt };

  const startUar = async () => {
    await writeFile(configPath, JSON.stringify(currentConfig, null, 2) + '\n', { mode: 0o600 });
    handle.launches += 1;
    const log = resolve(root, `uar-${handle.launches}.log`);
    handle.uarLogs.push(log);
    handle.uarPid = await launch(resolve(TARGETS[backend], 'universal-agent-runtime'), ['--config', configPath], root, uarEnv, log);
    await waitFor(`UAR readiness (log ${log})`,
      async () => (await fetch(new URL('/readyz', handle.runtime))).status === 200,
      STARTUP_DEADLINE_MS, () => !alive(handle.uarPid));
  };
  await startUar();
  handle.proxyPid = await launch(PROXY_BINARY,
    ['--config', configPath, '--upstream', handle.runtime.href, '--listen', handle.base.host], root,
    { HOME: process.env.HOME, PATH: process.env.PATH, UAR_SECURITY__JWT_SECRET: secrets.jwt,
      PROXY_JWT_SUB: `bdd-dedicated-${backend}` }, handle.proxyLog);
  await waitFor(`identity proxy (log ${handle.proxyLog})`, async () => (await fetch(new URL('/healthz', handle.base))).ok,
    60_000, () => !alive(handle.proxyPid));
  Object.assign(record, { uar_pid: handle.uarPid, proxy_pid: handle.proxyPid });

  handle.restartDatabase = () => docker(['restart', '--time', '10', handle.container]);
  /** Stop UAR gracefully and start it again on the same store, optionally with new config. */
  handle.restartUar = async (configOverrides) => {
    if (!(await terminate(handle.uarPid))) throw new Error('UAR did not stop gracefully; never force-kill');
    if (configOverrides) currentConfig = merge(currentConfig, configOverrides);
    await startUar();
    return handle.uarPid;
  };
  handle.uarAlive = () => alive(handle.uarPid);
  handle.logMessages = async () => {
    const lines = (await Promise.all(handle.uarLogs.map(path => readFile(path, 'utf8')))).join('\n').split('\n');
    return lines.flatMap(line => {
      try { return [JSON.parse(line).fields?.message].filter(Boolean); } catch { return []; }
    });
  };
  return handle;
}
