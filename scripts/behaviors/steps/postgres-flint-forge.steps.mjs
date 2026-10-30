import assert from 'node:assert/strict';
import { createHash, randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { createServer } from 'node:http';
import { dirname, resolve } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { After, Before, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { STUB_LLM_MODEL, STUB_LLM_PROMPT, startDedicatedRuntime } from '../../acceptance/fixtures/dedicated-runtime.mjs';

setDefaultTimeout(240_000);
const SHUTDOWN_KEY = 'server.shutdown_timeout_secs';
const TEAM_DURABILITY = ['supports_channel_observers', 'supports_durable_observers',
  'supports_durable_agent_instances', 'supports_durable_checkpoint_resume'];

// The fixture's identity proxy supplies the owner; the test sends no credentials.
async function request(w, method, path, payload, expected = [200], headers = {}) {
  const res = await fetch(new URL(path, w.runtime.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(120_000),
    headers: { 'Content-Type': 'application/json', ...headers },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  const text = await res.text();
  w.evidence.exchanges.push({ method, path, request: payload, status: res.status, response: text.slice(0, 4000) });
  assert.ok(expected.includes(res.status), `${method} ${path}: ${res.status}: ${text.slice(0, 800)}`);
  return text ? JSON.parse(text) : null;
}

// Deterministic OpenAI-compatible embeddings: equal text gives an equal vector.
function startEmbeddingStub(dimension) {
  const vector = text => {
    const values = [];
    for (let block = 0; values.length < dimension; block++) {
      const digest = createHash('sha256').update(`${block}:${text}`).digest();
      for (const byte of digest) values.push(byte / 255 - 0.5);
    }
    const unit = values.slice(0, dimension);
    const norm = Math.hypot(...unit) || 1;
    return unit.map(v => v / norm);
  };
  const server = createServer((req, res) => {
    let body = '';
    req.on('data', chunk => { body += chunk; });
    req.on('end', () => {
      const input = JSON.parse(body || '{}').input ?? [];
      const texts = Array.isArray(input) ? input : [input];
      res.writeHead(200, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify({ object: 'list', model: 'stub-embed-1024',
        data: texts.map((text, index) => ({ object: 'embedding', index, embedding: vector(String(text)) })),
        usage: { prompt_tokens: 0, total_tokens: 0 } }));
    });
  });
  return new Promise(resolveServer => server.listen(0, '127.0.0.1', () => resolveServer(server)));
}

Before(function ({ pickle }) {
  this.scenarioName = pickle.name;
});

async function startRuntime(world, options = {}) {
  world.started = performance.now();
  world.holder = {};
  world.owned = [];
  const id = randomUUID();
  world.record = resolve(`.prometheus/verification/postgres-flint-forge/${id}.json`);
  world.evidence = { scenario: world.scenarioName, platform: process.platform,
    authentication_supplied_by_test: false, exchanges: [], fixture: {} };
  world.runtime = await startDedicatedRuntime(world.evidence.fixture, world.holder,
    { backend: 'postgres', label: 'postgres-flint-forge', ...options });
  world.uarPid = world.runtime.uarPid;
}

const shutdownTimeout = seconds => ({ server: { shutdown_timeout_secs: seconds } });

Given('a dedicated UAR on the flint-forge PostgreSQL 18 image', { timeout: 420_000 }, async function () {
  await startRuntime(this);
});

Given('a dedicated UAR on the flint-forge PostgreSQL 18 image with shutdown timeout {int}', { timeout: 420_000 }, async function (seconds) {
  await startRuntime(this, { config: shutdownTimeout(seconds) });
});

Given('a dedicated UAR on the flint-forge PostgreSQL 18 image with a {int} second session TTL and the stub model', { timeout: 420_000 }, async function (ttl) {
  await startRuntime(this, { stubLlm: true, config: { sessions: { persisted_ttl_secs: ttl } } });
});

Given('a dedicated UAR on the flint-forge PostgreSQL 18 image with memory enabled', { timeout: 420_000 }, async function () {
  // UAR runs with the run directory as its cwd, so the default ./data/memory.db stays inside it.
  await startRuntime(this, { config: { memory: { enabled: true } } });
});

Given('a dedicated UAR on the flint-forge PostgreSQL 18 image with a {int}-dimension embedder', { timeout: 420_000 }, async function (dimension) {
  this.embeddingStub = await startEmbeddingStub(dimension);
  const { port } = this.embeddingStub.address();
  // The stub ignores the key; it is a fixed placeholder, not a credential.
  await startRuntime(this, { config: { embedding: { backend: 'openai', model: 'stub-embed-1024',
    base_url: `http://127.0.0.1:${port}/v1`, api_key: 'test-stub-key', vector_dimension: dimension, send_dimensions: false } } });
  this.evidence.embedding = { backend: 'openai-compatible stub', dimension };
});

Then('readiness reports the postgres persistence backend as ok', async function () {
  const res = await fetch(new URL('/readyz', this.runtime.runtime));
  const body = await res.json();
  this.evidence.readiness = body;
  assert.equal(res.status, 200);
  assert.deepEqual(body.checks.persistence, { backend: 'postgres', status: 'ok' });
  assert.equal(body.checks.postgres, 'ok');
});

When('I save a skill and an agent', async function () {
  this.skill = `pg18-baseline-${randomUUID().slice(0, 8)}`;
  const saved = await request(this, 'POST', '/api/uar/skills',
    { name: this.skill, description: 'PostgreSQL 18 baseline probe', triggers: { keywords: [this.skill] }, prompt_overlay: 'Probe.' }, [201]);
  assert.equal(saved.skill_id, this.skill);
  this.owned.push(`/api/uar/skills/${this.skill}`);
  const agent = await request(this, 'GET', '/api/agents/default-agent');
  this.agentId = `pg18-baseline-${randomUUID()}`;
  agent.id = this.agentId;
  agent.metadata = { ...agent.metadata, title: 'PostgreSQL 18 baseline agent', tags: ['verification'] };
  await request(this, 'POST', '/api/agents', agent, [201]);
  this.owned.push(`/api/agents/${this.agentId}`);
});

When('UAR restarts on the same database', { timeout: 300_000 }, async function () {
  await this.runtime.restartUar();
  this.evidence.restarts = (this.evidence.restarts ?? 0) + 1;
});

When('UAR restarts with shutdown timeout {int}', { timeout: 300_000 }, async function (seconds) {
  await this.runtime.restartUar(shutdownTimeout(seconds));
});

Then('the skill and the agent are still there', async function () {
  assert.equal((await request(this, 'GET', `/api/uar/skills/${this.skill}`)).skill_id, this.skill);
  assert.equal((await request(this, 'GET', `/api/agents/${this.agentId}`)).id, this.agentId);
  this.evidence.result = 'VERIFIED';
});

async function shutdownSetting(world) {
  const setting = await request(world, 'GET', `/api/uar/settings/${SHUTDOWN_KEY}`);
  world.evidence.settings = [...(world.evidence.settings ?? []), { effective: setting.effective, source: setting.source, is_drift: setting.is_drift }];
  return setting;
}

Then('the shutdown timeout setting is {int} from the config file', async function (seconds) {
  const setting = await shutdownSetting(this);
  assert.equal(setting.effective, seconds);
  assert.equal(setting.is_drift, false);
  if (seconds === 34) this.evidence.result = 'VERIFIED';
});

When('I override the shutdown timeout to {int} through the API', async function (seconds) {
  const current = await shutdownSetting(this);
  await request(this, 'PUT', `/api/uar/settings/${SHUTDOWN_KEY}`, { value: seconds, expected_revision: current.revision });
  // The setting applies on restart: the API stores it now and reports it pending.
  const saved = await shutdownSetting(this);
  assert.equal(saved.saved, seconds);
  assert.equal(saved.application_status, "restart_required");
});

Then('the shutdown timeout setting stays {int} and is reported as drift', async function (seconds) {
  const setting = await shutdownSetting(this);
  assert.equal(setting.effective, seconds);
  assert.equal(setting.is_drift, true);
  const drift = await request(this, 'GET', '/api/uar/settings/drift');
  assert.ok(drift.some(entry => entry.key === SHUTDOWN_KEY), 'The overridden setting is listed as drift');
});

When('I reset the shutdown timeout setting', async function () {
  const current = await shutdownSetting(this);
  await request(this, 'DELETE', `/api/uar/settings/${SHUTDOWN_KEY}`, undefined, [200, 204], { 'If-Match': current.revision });
});

When('I chat in a new session', async function () {
  // A fresh runtime admits a model only after it is verified, as an operator does in settings.
  const [provider, model] = STUB_LLM_MODEL.split('/');
  const verification = await request(this, 'POST', `/api/uar/providers/${provider}/test`, { model, output_token_cap: 128 });
  assert.equal(verification.ok, true, `Model verification: ${JSON.stringify(verification).slice(0, 300)}`);
  this.session = randomUUID();
  const completion = await request(this, 'POST', '/v1/chat/completions', { model: STUB_LLM_MODEL, session_id: this.session,
    stream: false, messages: [{ role: 'user', content: STUB_LLM_PROMPT }] });
  this.evidence.answer = completion.choices?.[0]?.message?.content;
  assert.match(this.evidence.answer ?? '', /SESSION-OK/);
});

async function listedSessions(world) {
  const snapshot = await request(world, 'GET', '/api/uar/sessions');
  return JSON.stringify(snapshot);
}

Then('the session is listed for its owner', async function () {
  assert.ok((await listedSessions(this)).includes(this.session), 'The new session is listed');
});

Then('the session disappears within {int} seconds', { timeout: 200_000 }, async function (seconds) {
  const started = Date.now();
  while (Date.now() - started < seconds * 1000) {
    if (!(await listedSessions(this)).includes(this.session)) {
      this.evidence.expired_after_ms = Date.now() - started;
      return;
    }
    await delay(5_000);
  }
  assert.fail(`The session was still listed after ${seconds} s`);
});

Then('it stays gone after UAR restarts', { timeout: 300_000 }, async function () {
  await this.runtime.restartUar();
  assert.ok(!(await listedSessions(this)).includes(this.session), 'The expired session stays deleted');
  this.evidence.purge_logged = (await this.runtime.logMessages()).includes('persisted session purged by TTL sweep');
  assert.ok(this.evidence.purge_logged, 'The TTL sweep logged the purge');
  this.evidence.result = 'VERIFIED';
});

When('I save a memory record', async function () {
  this.memoryFact = `PG18-MEMORY-${randomUUID().slice(0, 8)}`;
  const saved = await request(this, 'POST', '/api/memory', { content: `The project code word is ${this.memoryFact}.`,
    categories: ['personal_fact'] });
  assert.equal(saved?.status, 'success');
  this.memoryId = saved.id;
  this.owned.push(`/api/memory/records/${this.memoryId}`);
});

Then('the memory record is still readable', async function () {
  const record = await request(this, 'GET', `/api/memory/records/${this.memoryId}`);
  assert.ok(JSON.stringify(record).includes(this.memoryFact), 'The saved memory text survives the restart');
});

Then('the capability matrix names the memory store', async function () {
  const matrix = await request(this, 'GET', '/api/uar/persistence/capabilities');
  this.evidence.capabilities = matrix;
  assert.equal(matrix.persistence.backend, 'postgres');
  assert.ok(matrix.memory?.store, 'The memory store is reported');
  this.evidence.result = 'VERIFIED';
});

When('I save a skill', async function () {
  this.skill = `pg18-dimension-${randomUUID().slice(0, 8)}`;
  this.skillText = `Reconcile the ${this.skill} ledger for quarterly audits`;
  const saved = await request(this, 'POST', '/api/uar/skills',
    { name: this.skill, description: this.skillText, triggers: { keywords: [this.skill] }, prompt_overlay: 'Probe.' }, [201]);
  assert.equal(saved.skill_id, this.skill);
  this.owned.push(`/api/uar/skills/${this.skill}`);
});

async function assertSkillMatches(world) {
  const result = await request(world, 'GET', `/api/uar/skills/match?q=${encodeURIComponent(world.skillText)}`);
  world.evidence.matches = [...(world.evidence.matches ?? []), result];
  assert.ok(JSON.stringify(result).includes(world.skill), 'Skill matching returns the saved skill');
}

Then('skill matching finds it', async function () { await assertSkillMatches(this); });

Then('skill matching still finds it', async function () {
  await assertSkillMatches(this);
  const messages = await this.runtime.logMessages();
  this.evidence.skill_save_failures = messages.filter(m => /Failed to persist skill|Failed to generate embedding for skill|does not match the active embedding space|failed to re-embed skills/i.test(m));
  assert.deepEqual(this.evidence.skill_save_failures, []);
  this.evidence.result = 'VERIFIED';
});

Then('the capability matrix reports the team durability features as unsupported', async function () {
  const matrix = await request(this, 'GET', '/api/uar/persistence/capabilities');
  this.evidence.capabilities = matrix;
  assert.equal(matrix.persistence.backend, 'postgres');
  for (const flag of TEAM_DURABILITY) assert.equal(matrix.persistence.capabilities[flag], false, flag);
});

Then('a follow-up request is refused with capability_unsupported', async function () {
  // Follow-ups are workspace-scoped; the header selects a workspace, it is not a credential.
  const refusal = await request(this, 'GET', '/api/uar/followups', undefined, [503],
    { 'x-uar-workspace-id': 'bdd-postgres-flint-forge' });
  this.evidence.refusal = refusal;
  assert.equal(refusal?.error?.code ?? refusal?.code, 'capability_unsupported');
  this.evidence.result = 'VERIFIED';
});

After(async function ({ result }) {
  if (!this.evidence) return;
  const errors = [];
  if (this.runtime?.uarAlive()) {
    for (const path of [...this.owned].reverse()) {
      try { await request(this, 'DELETE', path, undefined, [200, 204, 404]); }
      catch (error) { errors.push(`${path}: ${error}`); }
    }
  }
  if (this.holder?.handle) errors.push(...await this.holder.handle.stop());
  if (this.embeddingStub) await new Promise(done => this.embeddingStub.close(done));
  this.evidence.cleanup_errors = errors;
  this.evidence.status = errors.length ? 'FAILED' : result.status;
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify({ evidence: this.record, status: this.evidence.status }), 'application/json');
  assert.deepEqual(errors, [], 'Disposable cleanup must succeed');
});
