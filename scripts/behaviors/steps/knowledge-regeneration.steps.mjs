import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { assertOwnedEndpoint } from '../support/restart-owned-uar.mjs';

setDefaultTimeout(900_000);
const settings = '/api/uar/settings';
const root = '/api/uar/knowledge-bases';
const fields = ['embedding.backend', 'embedding.model', 'embedding.vector_dimension'];
const target = { backend: 'openai', model: 'text-embedding-3-small', dimension: 1536 };
const targetValues = [target.backend, target.model, target.dimension];

async function request(world, method, path, payload, status = 200, timeout = 15_000) {
  const multipart = payload instanceof FormData;
  const exchange = { method, path, request: multipart ? { file: world.documentText } : payload };
  world.evidence.exchanges.push(exchange);
  const response = await fetch(new URL(path, world.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(timeout),
    headers: { ...(!multipart && { 'Content-Type': 'application/json' }), 'x-uar-workspace-id': world.workspace },
    body: payload === undefined ? undefined : multipart ? payload : JSON.stringify(payload),
  });
  exchange.status = response.status;
  exchange.response = await response.text();
  assert.equal(response.status, status, `${method} ${path}: ${response.status}: ${exchange.response.slice(0, 800)}`);
  return exchange.response ? JSON.parse(exchange.response) : null;
}
async function currentSettings(world) {
  const rows = [];
  for (const field of fields) rows.push(await request(world, 'GET', `${settings}/llm.${field}`));
  return rows;
}
async function changeSettings(world, values, rows) {
  const response = await request(world, 'PUT', `${settings}/llm`, {
    data: Object.fromEntries(fields.map((field, i) => [field, values[i]])),
    expected_revisions: Object.fromEntries(fields.map((field, i) => [field, rows[i].revision])),
  });
  assert.equal(response.status, 'updated', JSON.stringify(response));
  for (const field of fields) assert.equal(response.activation[`llm.${field}`], 'live');
}
async function indexedDocument(world) {
  const deadline = Date.now() + 120_000;
  while (Date.now() < deadline) {
    const doc = await request(world, 'GET', `${world.kbPath}/documents/${world.docId}`);
    assert.notEqual(doc.status, 'failed', JSON.stringify(doc));
    if (doc.status === 'indexed') { assert.ok(doc.chunk_count > 0); return doc; }
    await delay(500);
  }
  throw new Error('Document ingestion deadline exceeded');
}

Given('a disposable knowledge base indexed with the original embedding model', async function () {
  this.started = performance.now();
  this.id = randomUUID();
  this.record = resolve(`.prometheus/verification/knowledge-regeneration/${this.id}.json`);
  this.evidence = { scenario: 'KNOWLEDGE-REGENERATION-01', platform: process.platform,
    authentication_supplied_by_test: false, previous_scenarios_rerun: false, exchanges: [],
    target, hosting: { embedding_caps_usd: { openai: 10, cohere: 10 }, cohere_used: false,
      openai_rate_usd_per_million_tokens: 0.02, actual_billing_usage: 'not exposed by UAR' } };
  this.base = new URL(process.env.UAR_BASE_URL);
  this.workspace = process.env.UAR_WORKSPACE_ID;
  assert.ok(this.workspace);
  this.evidence.endpoint = await assertOwnedEndpoint(this.base);
  this.original = await currentSettings(this);
  this.originalValues = this.original.map(row => row.saved);
  this.evidence.original_settings = this.original;
  this.originalSpace = (await request(this, 'GET', `${settings}/embedding/provenance`)).space;
  assert.deepEqual(this.originalSpace, { backend: 'fastembed', model: 'bge-small-en-v1.5', dimension: 384 });
  assert.deepEqual(this.originalValues, [this.originalSpace.backend, this.originalSpace.model, this.originalSpace.dimension]);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(`${this.record}.before.json`, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.seal = `REGEN-${randomUUID()}`;
  this.documentText = `The private Zephyr laboratory ledger assigns the violet lattice a normalization coefficient of 73. Its retrieval seal is ${this.seal}. This is a synthetic research fact unique to this document.`;
  this.query = 'What normalization coefficient and retrieval seal does the private Zephyr ledger assign to the violet lattice?';
  assert.ok(!this.query.includes(this.seal) && !this.query.includes('73'));
  // Two tiny ASCII inputs: a conservative byte-as-token bound is far below the existing $10 cap.
  assert.ok(/^[\x00-\x7f]*$/.test(this.documentText + this.query));
  const bytes = Buffer.byteLength(this.documentText + this.query);
  assert.ok(bytes < 1000);
  this.evidence.hosting.estimated_two_input_cost_ceiling_usd = bytes * 0.02 / 1_000_000;
  const kb = await request(this, 'POST', root, { name: `Regeneration BDD ${this.id}`,
    description: 'Disposable manual model migration scenario', config: { chunk_strategy: 'fixed', chunk_size: 512,
      retrieval_min_score: 0, retrieval_top_k: 5 } }, 201);
  this.kbPath = `${root}/${encodeURIComponent(kb.id)}`;
  this.evidence.knowledge_base_id = kb.id;
  const form = new FormData();
  form.append('file', new Blob([this.documentText], { type: 'text/plain' }), 'zephyr-ledger.txt');
  const doc = await request(this, 'POST', `${this.kbPath}/documents`, form, 202);
  this.docId = doc.id;
  this.evidence.original_document = await indexedDocument(this);
  this.evidence.original_kb = await request(this, 'GET', this.kbPath);
  assert.deepEqual(this.evidence.original_kb.config.indexed_embedding, this.originalSpace);
  assert.equal(this.evidence.original_kb.document_count, 1);
  this.evidence.seal = this.seal;
});

When('I change the active embedding model to a different vector dimension', async function () {
  const rows = await currentSettings(this);
  assert.deepEqual(rows.map(row => row.saved), this.originalValues);
  this.settingsTouched = true;
  await changeSettings(this, targetValues, rows);
  this.evidence.active = await request(this, 'GET', `${settings}/embedding/provenance`);
  assert.equal(this.evidence.active.status, 'serving');
  assert.deepEqual(this.evidence.active.space, target);
});

Then('the old knowledge index refuses incompatible retrieval', async function () {
  const refusal = await request(this, 'POST', `${this.kbPath}/search`, { query: this.query, limit: 5, min_score: 0 }, 409);
  assert.equal(refusal.error.code, 'embedding_space_mismatch');
  this.evidence.refusal = refusal;
});

When('I regenerate the knowledge index from its retained document', async function () {
  const initial = await request(this, 'POST', `${this.kbPath}/regeneration`, undefined, 202);
  assert.deepEqual(initial.previous, this.originalSpace);
  assert.deepEqual(initial.target, target);
  assert.deepEqual(initial.document_ids, [this.docId]);
  assert.ok(initial.generation);
  this.evidence.regeneration_started = initial;
  const deadline = Date.now() + 120_000;
  while (Date.now() < deadline) {
    const regen = await request(this, 'GET', `${this.kbPath}/regeneration`);
    assert.equal(regen.generation, initial.generation);
    assert.notEqual(regen.state, 'failed', JSON.stringify(regen));
    if (regen.state === 'complete') {
      assert.equal(regen.completed_documents, 1);
      assert.deepEqual(regen.document_ids, [this.docId]);
      assert.deepEqual(regen.target, target);
      this.evidence.regeneration_completed = regen;
      return;
    }
    await delay(500);
  }
  throw new Error('Regeneration deadline exceeded');
});

Then('retrieval returns the original private fact using the new embedding space', async function () {
  this.evidence.final_kb = await request(this, 'GET', this.kbPath);
  this.evidence.final_document = await indexedDocument(this);
  assert.equal(this.evidence.final_kb.document_count, 1);
  assert.equal(this.evidence.final_document.id, this.docId);
  assert.deepEqual(this.evidence.final_kb.config.indexed_embedding, target);
  const response = await request(this, 'POST', `${this.kbPath}/search`, { query: this.query, limit: 5, min_score: 0 }, 200, 75_000);
  this.evidence.retrieval = response;
  assert.ok(response.results.some(row => row.document_id === this.docId && row.content.includes(this.seal)
    && row.content.includes('73') && Number.isFinite(row.score) && row.score > 0), 'Require original document content retrieved from the regenerated index');
  this.behaviorPassed = true;
});

After({ timeout: 900_000 }, async function ({ result }) {
  if (!this.evidence) return;
  const errors = [];
  const cleanup = async operation => { try { await operation(); } catch (error) { errors.push(error); } };
  await cleanup(async () => {
    if (this.kbPath) {
      await request(this, 'DELETE', this.kbPath, undefined, 204);
      this.evidence.disposable_deleted = true;
    }
  });
  await cleanup(async () => {
    if (this.settingsTouched) {
      const rows = await currentSettings(this);
      rows.forEach((row, i) => assert.ok(row.saved === this.originalValues[i] || row.saved === targetValues[i], 'Refuse to overwrite outside settings changes'));
      await changeSettings(this, this.originalValues, rows);
      assert.deepEqual((await currentSettings(this)).map(row => row.saved), this.originalValues);
      this.evidence.original_settings_restored = true;
    }
  });
  await cleanup(async () => {
    if (this.settingsTouched) {
      const current = await request(this, 'GET', `${settings}/embedding/provenance`);
      assert.deepEqual(current.space, this.originalSpace);
      this.evidence.restored_embedding = current;
    }
  });
  this.evidence.status = errors.length ? 'FAILED' : result.status;
  this.evidence.result = !errors.length && this.behaviorPassed && result.status === 'PASSED' ? 'VERIFIED' : 'FAILED';
  this.evidence.cleanup_errors = errors.map(error => error.message);
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify({ evidence: this.record, status: this.evidence.status }), 'application/json');
  if (errors.length) throw new AggregateError(errors, 'Embedding fixture cleanup failed; see evidence');
});
