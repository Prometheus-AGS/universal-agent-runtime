import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { readFile, mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';

setDefaultTimeout(90_000);
let poisoned = false;
const inputs = ['owned-first', 'owned-second'];
const endpoint = '/v1/embeddings';
const sentinel = 'synthetic-provider-credential-body';
function localUrl(value) {
  const url = new URL(value);
  assert.ok(['http:', 'https:'].includes(url.protocol) && ['127.0.0.1', '[::1]'].includes(url.hostname));
  assert.ok(!url.username && !url.password && !url.search && !url.hash);
  return url;
}
async function http(world, base, path, method = 'GET', payload) {
  assert.ok(++world.operations <= 100, 'Owned embedding HTTP budget exceeded');
  const response = await fetch(new URL(path, base), {
    method, redirect: 'error', signal: AbortSignal.timeout(15_000),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': world.workspace },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  const raw = await response.text();
  assert.ok(!raw.includes(sentinel), 'Provider private sentinel escaped');
  let body;
  try { body = raw ? JSON.parse(raw) : null; } catch { body = null; }
  world.record.exchanges.push({ method, path, status: response.status });
  return { status: response.status, body };
}
async function peerState(world) {
  const result = await http(world, world.control, '/__embedding/state');
  assert.equal(result.status, 200);
  assert.equal(result.body.namespace, world.fixture.namespace);
  assert.equal(result.body.overflowed, false);
  assert.ok(Number.isSafeInteger(result.body.arrivals));
  return result.body;
}
async function control(world, mode, holdResponses = false) {
  const result = await http(world, world.control, '/__embedding/control', 'POST', {
    namespace: world.fixture.namespace, mode, holdResponses,
  });
  assert.equal(result.status, 200);
  assert.equal(result.body.configured, true);
}
async function release(world) {
  const result = await http(world, world.control, '/__embedding/release', 'POST', { namespace: world.fixture.namespace });
  assert.equal(result.status, 200);
  assert.equal(result.body.released, true);
}
function requestBody(world, model = world.fixture.model, encoding = 'float') {
  return { model, input: inputs, dimensions: world.fixture.dimension, encoding_format: encoding };
}
function vectors(body, encoding) {
  return body.data.map(row => {
    if (encoding === 'float') return row.embedding;
    const bytes = Buffer.from(row.embedding, 'base64');
    assert.equal(bytes.length % 4, 0);
    return Array.from({ length: bytes.length / 4 }, (_, i) => bytes.readFloatLE(i * 4));
  });
}
function assertVectors(world, result, model, encoding = 'float') {
  assert.equal(result.status, 200);
  assert.equal(result.body.model, `openai/${model}`);
  assert.deepEqual(result.body.data.map(row => row.index), [0, 1]);
  const actual = vectors(result.body, encoding);
  const expected = world.fixture.expectedVectors[model];
  assert.ok(Array.isArray(expected) && expected.length === inputs.length, 'Missing independent model/vector oracle');
  for (let row = 0; row < inputs.length; row++) {
    assert.equal(actual[row].length, world.fixture.dimension);
    assert.equal(expected[row].length, world.fixture.dimension);
    actual[row].forEach((value, column) => {
      assert.ok(Number.isFinite(value));
      assert.ok(Math.abs(value - expected[row][column]) < 0.000001, 'Vector order or owned model space differs');
    });
  }
}
function assertUsage(result, mode) {
  const prompt = ['nonzero', 'prompt-only'].includes(mode) ? 7 : 0;
  const total = ['nonzero', 'total-only'].includes(mode) ? 9 : 0;
  const availablePrompt = !['absent', 'total-only'].includes(mode);
  const availableTotal = !['absent', 'prompt-only'].includes(mode);
  assert.deepEqual(result.body.usage, { prompt_tokens: prompt, total_tokens: total });
  for (const [key, available] of [['prompt_tokens', availablePrompt], ['total_tokens', availableTotal]]) {
    assert.deepEqual(result.body.uar_usage[key], { available, source: available ? 'provider_reported' : 'unavailable' });
  }
}
Given('the externally provisioned owned embedding usage fixture', async function () {
  assert.equal(poisoned, false, 'Prior uncertain owned fixture state requires external reconciliation');
  assert.ok(process.env.UAR_BASE_URL && process.env.UAR_WORKSPACE_ID && process.env.UAR_EMBEDDING_FIXTURE);
  this.fixture = JSON.parse(await readFile(process.env.UAR_EMBEDDING_FIXTURE, 'utf8'));
  assert.equal(this.fixture.dedicated, true, 'Fixture must be isolated from every other chat/runtime');
  assert.match(this.fixture.namespace, /^[A-Za-z0-9_-]{1,96}$/);
  assert.match(this.fixture.identity.sourceCommit, /^[a-f0-9]{40}$/);
  assert.match(this.fixture.identity.imageDigest, /^sha256:[a-f0-9]{64}$/);
  assert.match(this.fixture.identity.configDigest, /^[a-f0-9]{64}$/);
  assert.equal(this.fixture.identity.platform, 'linux/amd64');
  assert.ok(Number.isSafeInteger(this.fixture.dimension) && this.fixture.dimension > 0 && this.fixture.dimension <= 16000);
  assert.ok(this.fixture.batchSize >= inputs.length, 'Mounted usage oracle expects one remote chunk');
  assert.match(this.fixture.model, new RegExp(`^${this.fixture.namespace}-[A-Za-z0-9_-]+$`));
  assert.match(this.fixture.nextModel, new RegExp(`^${this.fixture.namespace}-[A-Za-z0-9_-]+$`));
  assert.notEqual(this.fixture.model, this.fixture.nextModel);
  assert.notDeepEqual(this.fixture.expectedVectors[this.fixture.model], this.fixture.expectedVectors[this.fixture.nextModel]);
  this.base = localUrl(process.env.UAR_BASE_URL);
  this.control = localUrl(this.fixture.controlUrl);
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.operations = 0;
  this.record = { namespace: this.fixture.namespace, identity: this.fixture.identity, authentication_supplied_by_test: false, exchanges: [] };
  this.receipt = resolve(`.prometheus/verification/embedding-usage/${randomUUID()}.json`);
  this.initial = await peerState(this);
  assert.equal(this.initial.held, 0);
});
When('the owned embedding peer reports {string}', async function (mode) {
  await control(this, mode);
  this.before = (await peerState(this)).arrivals;
});
When('I request the configured embedding model using {string}', async function (encoding) {
  this.encoding = encoding;
  this.answer = await http(this, this.base, endpoint, 'POST', requestBody(this, this.fixture.model, encoding));
});
Then('embedding vectors and usage match the owned {string} oracle', async function (mode) {
  assertVectors(this, this.answer, this.fixture.model, this.encoding);
  assertUsage(this.answer, mode);
  assert.equal((await peerState(this)).arrivals - this.before, 1);
  this.record.usage = this.answer.body.uar_usage;
});
Then('the mounted embedding operation refuses the provider result privately', async function () {
  assert.equal(this.answer.status, 502);
  assert.equal(this.answer.body.error.code, 'embedding_provider_error');
  assert.equal(this.answer.body.data, undefined);
  assert.equal((await peerState(this)).arrivals - this.before, 1);
});
Then('both served embedding model names return exact dimensions', async function () {
  for (const model of [this.fixture.model, `openai/${this.fixture.model}`]) {
    const result = await http(this, this.base, endpoint, 'POST', requestBody(this, model));
    assertVectors(this, result, this.fixture.model);
    assertUsage(result, 'nonzero');
  }
  assert.equal((await peerState(this)).arrivals - this.before, 2);
});
Then('the embedding request variant {string} refuses before any backend arrival', async function (variant) {
  const body = requestBody(this);
  const variants = {
    'unknown-model': [404, () => { body.model = `${this.fixture.namespace}-not-served`; }],
    'wrong-dimensions': [400, () => { body.dimensions += 1; }],
    'empty-input': [400, () => { body.input = []; }],
    'empty-string': [400, () => { body.input = ['']; }],
    'oversized-batch': [400, () => { body.input = Array(2049).fill('owned'); }],
    'unsupported-format': [400, () => { body.encoding_format = 'unsupported'; }],
    'oversized-body': [400, () => { body.input = 'x'.repeat(10 * 1024 * 1024 + 1); }],
  };
  assert.ok(Object.hasOwn(variants, variant));
  const [status, edit] = variants[variant]; edit();
  const result = await http(this, this.base, endpoint, 'POST', body);
  assert.equal(result.status, status);
  assert.equal((await peerState(this)).arrivals, this.before);
});
Then('the externally provisioned {string} embedding ingress refuses before a backend arrival', async function (lane) {
  const statuses = { 'missing-identity': [401, 403], 'invalid-identity': [401, 403], 'denied-policy': [403], 'rate-limited': [429], unavailable: [503] };
  assert.ok(Object.hasOwn(statuses, lane));
  const ingress = this.fixture.refusalLanes[lane];
  assert.equal(ingress.namespace, this.fixture.namespace);
  const result = await http(this, localUrl(ingress.baseUrl), endpoint, 'POST', requestBody(this));
  assert.ok(statuses[lane].includes(result.status), 'External rejection did not match its admitted fixture lane');
  assert.equal((await peerState(this)).arrivals, this.before);
});
When('an owned embedding request is held while its configured backend is swapped', async function () {
  await control(this, 'nonzero', true);
  const before = (await peerState(this)).arrivals;
  this.pending = http(this, this.base, endpoint, 'POST', requestBody(this));
  // Observe rejection immediately without suppressing the original failure.
  this.pending.catch(() => {});
  let held = false;
  for (let attempt = 0; attempt < 20; attempt++) {
    const state = await peerState(this);
    if (state.arrivals === before + 1 && state.held === 1) { held = true; break; }
    await delay(100);
  }
  assert.equal(held, true, 'Request did not reach the owned held backend');
  const row = await http(this, this.base, '/api/uar/settings/llm.embedding.model');
  assert.equal(row.status, 200);
  assert.equal(row.body.saved, this.fixture.model, 'Only the dedicated expected embedding setting may change');
  this.oldModel = row.body.saved;
  this.swapAttempted = true;
  const changed = await http(this, this.base, '/api/uar/settings/llm', 'PUT', {
    data: { 'embedding.model': this.fixture.nextModel }, expected_revisions: { 'embedding.model': row.body.revision },
  });
  assert.equal(changed.status, 200);
  assert.equal(changed.body.status, 'updated');
  assert.equal(changed.body.activation['llm.embedding.model'], 'live');
  await release(this);
  this.answer = await this.pending;
  this.pending = undefined;
});
Then('the admitted request retains its original vectors model and usage', function () {
  assertVectors(this, this.answer, this.fixture.model);
  assertUsage(this.answer, 'nonzero');
});
Then('the next embedding request uses the new configured space and usage', async function () {
  await control(this, 'total-only');
  const answer = await http(this, this.base, endpoint, 'POST', requestBody(this, this.fixture.nextModel));
  assertVectors(this, answer, this.fixture.nextModel);
  assertUsage(answer, 'total-only');
  const state = await peerState(this);
  assert.deepEqual(state.requests.slice(-2).map(request => request.model), [this.fixture.model, this.fixture.nextModel]);
});
After(async function ({ result }) {
  if (!this.record) return;
  const cleanup = [];
  try { await release(this); } catch { cleanup.push('owned-peer-release-uncertain'); }
  if (this.pending) {
    try { await this.pending; } catch { cleanup.push('admitted-request-outcome-uncertain'); }
  }
  if (this.swapAttempted) {
    try {
      const row = await http(this, this.base, '/api/uar/settings/llm.embedding.model');
      assert.equal(row.status, 200);
      if (row.body.saved !== this.oldModel) {
        assert.equal(row.body.saved, this.fixture.nextModel, 'Concurrent setting change requires external reconciliation');
        const restored = await http(this, this.base, '/api/uar/settings/llm', 'PUT', {
          data: { 'embedding.model': this.oldModel }, expected_revisions: { 'embedding.model': row.body.revision },
        });
        assert.equal(restored.status, 200); assert.equal(restored.body.status, 'updated');
        assert.equal(restored.body.activation['llm.embedding.model'], 'live');
      }
    } catch { cleanup.push('owned-model-restore-uncertain'); }
  }
  try {
    const state = await peerState(this);
    assert.equal(state.held, 0);
    this.record.backendArrivals = state.arrivals - this.initial.arrivals;
    await control(this, 'zero');
  } catch { cleanup.push('owned-peer-state-uncertain'); }
  poisoned ||= cleanup.length > 0;
  this.record.status = cleanup.length ? 'FAILED' : result.status;
  this.record.cleanup = cleanup;
  await mkdir(dirname(this.receipt), { recursive: true });
  await writeFile(this.receipt, JSON.stringify(this.record, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify({ evidence: this.receipt, status: this.record.status }), 'application/json');
  assert.deepEqual(cleanup, [], 'Uncertain owned effects require external reconciliation; no automatic retry');
});
