import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { assertOwnedEndpoint } from '../support/restart-owned-uar.mjs';

setDefaultTimeout(420_000);
const root = '/api/uar/knowledge-bases';

async function request(world, method, path, payload, expected = 200) {
  const multipart = payload instanceof FormData;
  const exchange = { method, path, request: multipart ? { file: world.uploadText } : payload };
  world.evidence.exchanges.push(exchange);
  const response = await fetch(new URL(path, world.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(15_000),
    headers: { ...(!multipart && { 'Content-Type': 'application/json' }), 'x-uar-workspace-id': world.workspace },
    body: payload === undefined ? undefined : multipart ? payload : JSON.stringify(payload),
  });
  exchange.status = response.status;
  exchange.response = await response.text();
  assert.equal(response.status, expected, `${method} ${path}: ${response.status}: ${exchange.response.slice(0, 800)}`);
  if (!exchange.response || expected === 404) return exchange.response;
  return JSON.parse(exchange.response);
}

async function uploadIndexed(world, name, content) {
  world.uploadText = content;
  const form = new FormData();
  form.append('file', new Blob([content], { type: 'text/plain' }), name);
  const uploaded = await request(world, 'POST', `${world.kbPath}/documents`, form, 202);
  assert.ok(uploaded.id);
  const deadline = Date.now() + 120_000;
  while (Date.now() < deadline) {
    const doc = await request(world, 'GET', `${world.kbPath}/documents/${encodeURIComponent(uploaded.id)}`);
    assert.notEqual(doc.status, 'failed', JSON.stringify(doc));
    if (doc.status === 'indexed') {
      assert.ok(doc.chunk_count > 0);
      assert.equal(doc.kb_id, world.kbId);
      return doc;
    }
    await delay(500);
  }
  throw new Error('Document ingestion deadline exceeded');
}

Given('a disposable knowledge base with two indexed laboratory notes', async function () {
  this.started = performance.now();
  this.id = randomUUID();
  this.record = resolve(`.prometheus/verification/knowledge-document-delete/${this.id}.json`);
  this.evidence = { scenario: 'KNOWLEDGE-DOCUMENT-DELETE-01', platform: process.platform,
    authentication_supplied_by_test: false, previous_scenarios_rerun: false,
    configuration_changes: false, restart: false, exchanges: [] };
  this.base = new URL(process.env.UAR_BASE_URL);
  this.workspace = process.env.UAR_WORKSPACE_ID;
  assert.ok(this.workspace);
  this.evidence.endpoint = await assertOwnedEndpoint(this.base);
  this.evidence.embedding = await request(this, 'GET', '/api/uar/settings/embedding/provenance');
  assert.equal(this.evidence.embedding.status, 'serving');
  assert.equal(this.evidence.embedding.space.backend, 'fastembed');
  assert.equal(this.evidence.embedding.endpoint, null, 'Use the retained local embedder only');
  this.deletedSeal = `DELETE-${randomUUID()}`;
  this.retainedSeal = `RETAIN-${randomUUID()}`;
  this.search = { query: 'What coefficients and seals are recorded in the amber and cobalt lattice laboratory notes?', limit: 10, min_score: 0 };
  const kb = await request(this, 'POST', root, { name: `Document deletion BDD ${this.id}`,
    description: 'Disposable target and retained control documents', config: { chunk_strategy: 'fixed', chunk_size: 512,
      retrieval_min_score: 0, retrieval_top_k: 10 } }, 201);
  this.kbId = kb.id;
  assert.ok(this.kbId);
  this.kbPath = `${root}/${encodeURIComponent(this.kbId)}`;
  Object.assign(this.evidence, { knowledge_base_id: this.kbId, deleted_seal: this.deletedSeal, retained_seal: this.retainedSeal });
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(`${this.record}.before.json`, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.target = await uploadIndexed(this, 'amber-lattice.txt', `The amber lattice laboratory note records coefficient 73. Its private seal is ${this.deletedSeal}.`);
  this.control = await uploadIndexed(this, 'cobalt-lattice.txt', `The cobalt lattice laboratory note records coefficient 91. Its private seal is ${this.retainedSeal}.`);
  assert.notEqual(this.target.id, this.control.id);
  Object.assign(this.evidence, { target_document: this.target, retained_document: this.control });
});

Given('the same query retrieves both original notes', async function () {
  const kb = await request(this, 'GET', this.kbPath);
  assert.equal(kb.document_count, 2);
  assert.deepEqual(kb.config.indexed_embedding, this.evidence.embedding.space);
  const before = await request(this, 'POST', `${this.kbPath}/search`, this.search);
  assert.ok(before.results.some(row => row.document_id === this.target.id && row.content.includes(this.deletedSeal)));
  assert.ok(before.results.some(row => row.document_id === this.control.id && row.content.includes(this.retainedSeal)));
  this.evidence.before_retrieval = before;
});

When('I delete only the first laboratory note', async function () {
  await request(this, 'DELETE', `${this.kbPath}/documents/${encodeURIComponent(this.target.id)}`, undefined, 204);
  this.evidence.document_delete_accepted = true;
});

Then('retrieval excludes the deleted note and still returns the retained note', async function () {
  const after = await request(this, 'POST', `${this.kbPath}/search`, this.search);
  assert.ok(after.results.length > 0, 'An empty or broken index must not count as deletion success');
  assert.ok(after.results.every(row => row.document_id !== this.target.id && !row.content.includes(this.deletedSeal)));
  assert.ok(after.results.some(row => row.document_id === this.control.id && row.content.includes(this.retainedSeal)));
  this.evidence.after_retrieval = after;
  await request(this, 'GET', `${this.kbPath}/documents/${encodeURIComponent(this.target.id)}`, undefined, 404);
  const documents = await request(this, 'GET', `${this.kbPath}/documents`);
  assert.deepEqual(documents.map(doc => doc.id), [this.control.id]);
  const retained = await request(this, 'GET', `${this.kbPath}/documents/${encodeURIComponent(this.control.id)}`);
  assert.deepEqual(retained, this.control);
  const kb = await request(this, 'GET', this.kbPath);
  assert.equal(kb.id, this.kbId);
  assert.equal(kb.document_count, 1);
  assert.deepEqual(kb.config.indexed_embedding, this.evidence.embedding.space);
  Object.assign(this.evidence, { remaining_documents: documents, surviving_kb: kb });
  this.behaviorPassed = true;
});

After({ timeout: 45_000 }, async function ({ result }) {
  if (!this.evidence) return;
  let failure;
  try {
    if (this.kbPath) {
      await request(this, 'DELETE', this.kbPath, undefined, 204);
      this.evidence.disposable_kb_deleted = true;
    }
  } catch (error) { failure = error; this.evidence.cleanup_error = error.message; }
  this.evidence.status = failure ? 'FAILED' : result.status;
  this.evidence.result = !failure && this.behaviorPassed && result.status === 'PASSED' ? 'VERIFIED' : 'FAILED';
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify({ evidence: this.record, status: this.evidence.status }), 'application/json');
  if (failure) throw failure;
});
