import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { assertOwnedEndpoint, restartOwnedUar } from '../support/restart-owned-uar.mjs';
import { prepareEmbeddingHost, restoreEmbeddingHost } from '../../manual-host/embedding-config.mjs';

setDefaultTimeout(600_000);
const settings = '/api/uar/settings';
const root = '/api/uar/knowledge-bases';
const fields = ['embedding.backend', 'embedding.model', 'embedding.vector_dimension'];
// Same width as the retained local model; OpenAI emits 384 because send_dimensions is on.
const target = { backend: 'openai', model: 'text-embedding-3-small', dimension: 384 };
const targetValues = [target.backend, target.model, target.dimension];
const DOCUMENTS = 60;

async function request(w, method, path, payload, status = 200, timeout = 30_000) {
  const multipart = payload instanceof FormData;
  const exchange = { method, path, request: multipart ? '<file>' : payload };
  w.evidence.exchanges.push(exchange);
  const response = await fetch(new URL(path, w.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(timeout),
    headers: { ...(!multipart && { 'Content-Type': 'application/json' }), 'x-uar-workspace-id': w.workspace },
    body: payload === undefined ? undefined : multipart ? payload : JSON.stringify(payload),
  });
  exchange.status = response.status;
  const text = await response.text();
  exchange.response = text.slice(0, 1500);
  const accepted = Array.isArray(status) ? status : [status];
  assert.ok(accepted.includes(response.status), `${method} ${path}: ${response.status}: ${text.slice(0, 800)}`);
  return text ? JSON.parse(text) : null;
}
const currentSettings = async w => Promise.all(fields.map(field => request(w, 'GET', `${settings}/llm.${field}`)));
async function changeSettings(w, values, rows) {
  const response = await request(w, 'PUT', `${settings}/llm`, {
    data: Object.fromEntries(fields.map((field, i) => [field, values[i]])),
    expected_revisions: Object.fromEntries(fields.map((field, i) => [field, rows[i].revision])) });
  assert.equal(response.status, 'updated', JSON.stringify(response));
}
const regeneration = w => request(w, 'GET', `${w.kbPath}/regeneration`);
const note = (i, seal) => `Archive note ${i}. ` + Array.from({ length: 6 }, (_, p) =>
  `Paragraph ${p + 1} of archive note ${i} describes shelf ${i * 10 + p} in the Wren repository and records seal ${seal}-${i}-${p}.`).join(' ');

Given('a disposable multi-document knowledge base indexed with the original embedding model', async function () {
  this.started = performance.now();
  this.id = randomUUID();
  this.record = resolve(`.prometheus/verification/regeneration-resume/${this.id}.json`);
  this.base = new URL(process.env.UAR_BASE_URL);
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.evidence = { scenario: 'C49-REGENERATION-RESUME-01', platform: process.platform, target,
    authentication_supplied_by_test: false, previous_scenarios_rerun: false, polls: [], exchanges: [],
    hosting: { embedding_caps_usd: { openai: 10, cohere: 10 }, openai_used: true, cohere_used: false,
      openai_rate_usd_per_million_tokens: 0.02 } };
  this.evidence.endpoint = await assertOwnedEndpoint(this.base);
  this.original = await currentSettings(this);
  this.originalValues = this.original.map(row => row.saved);
  this.originalSpace = (await request(this, 'GET', `${settings}/embedding/provenance`)).space;
  assert.equal(this.originalSpace.dimension, target.dimension);
  this.seal = `WREN-${randomUUID().slice(0, 6).toUpperCase()}`;
  const kb = await request(this, 'POST', root, { name: `Regeneration resume BDD ${this.id}`, description: 'Disposable multi-chunk corpus',
    config: { chunk_strategy: 'fixed', chunk_size: 128, retrieval_min_score: 0, retrieval_top_k: 5 } }, 201);
  this.kbPath = `${root}/${encodeURIComponent(kb.id)}`;
  // Upload one document at a time and wait for it: concurrent uploads into one
  // KB hit retryable store write conflicts (finding recorded separately).
  let bytes = 0;
  for (let i = 1; i <= DOCUMENTS; i += 1) {
    const text = note(i, this.seal);
    bytes += Buffer.byteLength(text);
    const form = new FormData();
    form.append('file', new Blob([text], { type: 'text/plain' }), `archive-${i}.txt`);
    const uploaded = await request(this, 'POST', `${this.kbPath}/documents`, form, 202);
    const deadline = Date.now() + 60_000;
    while ((await request(this, 'GET', `${this.kbPath}/documents/${uploaded.id}`)).status !== 'indexed') {
      assert.ok(Date.now() < deadline, `archive-${i} did not index`);
      await delay(250);
    }
  }
  this.evidence.hosting.corpus_bytes = bytes;
  this.evidence.hosting.estimated_cost_ceiling_usd = bytes * 0.02 / 1_000_000;
  const docs = await request(this, 'GET', `${this.kbPath}/documents`);
  if (docs.length === DOCUMENTS && docs.every(doc => doc.status === 'indexed')) this.documentIds = docs.map(doc => doc.id).sort();
  assert.equal(this.documentIds?.length, DOCUMENTS, 'All source documents indexed');
  this.evidence.original_kb = await request(this, 'GET', this.kbPath);
  assert.deepEqual(this.evidence.original_kb.config.indexed_embedding, this.originalSpace);
});

Given('the active embedding model is changed to a different model of the same width', async function () {
  this.hostState = {};
  this.evidence.host_provision_restart = {};
  await prepareEmbeddingHost(this.hostState, this.id, this.evidence.host_provision_restart);
  const rows = await currentSettings(this);
  assert.deepEqual(rows.map(row => row.saved), this.originalValues);
  this.settingsTouched = true;
  await changeSettings(this, targetValues, rows);
  this.evidence.active = await request(this, 'GET', `${settings}/embedding/provenance`);
  assert.deepEqual(this.evidence.active.space, target);
});

When('regeneration starts and the owned UAR is restarted before it completes', async function () {
  this.started_regeneration = await request(this, 'POST', `${this.kbPath}/regeneration`, undefined, 202);
  this.generation = this.started_regeneration.generation;
  assert.deepEqual([...this.started_regeneration.document_ids].sort(), this.documentIds);
  let snapshot;
  const deadline = Date.now() + 120_000;
  while (Date.now() < deadline) {
    snapshot = await regeneration(this);
    this.evidence.polls.push({ at: Date.now(), state: snapshot.state, completed: snapshot.completed_documents });
    assert.notEqual(snapshot.state, 'complete', 'Regeneration finished before it could be interrupted');
    if (snapshot.completed_documents >= 1) break;
    await delay(200);
  }
  this.evidence.before_interrupt = snapshot;
  this.evidence.interrupt_restart = {};
  await restartOwnedUar(this.evidence.interrupt_restart);
});

Then('after the restart the job reports an honest unfinished state and the sources are retained', async function () {
  const after = await regeneration(this);
  this.evidence.after_restart = after;
  assert.equal(after.generation, this.generation, 'The same generation is reported');
  assert.ok(after.state !== 'complete' || after.completed_documents === DOCUMENTS, 'No false complete state');
  assert.ok(after.completed_documents <= DOCUMENTS);
  const docs = await request(this, 'GET', `${this.kbPath}/documents`);
  assert.deepEqual(docs.map(doc => doc.id).sort(), this.documentIds, 'Retained sources survive the restart');
  const kb = await request(this, 'GET', this.kbPath);
  this.evidence.kb_after_restart = kb.config;
  if (after.state !== 'complete') {
    assert.notDeepEqual(kb.config.indexed_embedding, target, 'The KB is not marked ready in the target space before completion');
  }
});

When('regeneration is resumed through its designed operation', async function () {
  // Durable pending ingestion resumes after restart; a failed job is restarted explicitly.
  let snapshot = await regeneration(this);
  if (snapshot.state === 'failed') {
    this.evidence.explicit_restart = await request(this, 'POST', `${this.kbPath}/regeneration`, undefined, 202);
  }
  const deadline = Date.now() + 300_000;
  while (Date.now() < deadline) {
    snapshot = await regeneration(this);
    this.evidence.polls.push({ at: Date.now(), state: snapshot.state, completed: snapshot.completed_documents });
    if (snapshot.state === 'complete') break;
    assert.notEqual(snapshot.state, 'failed', JSON.stringify(snapshot));
    await delay(1000);
  }
  this.evidence.completed = snapshot;
  assert.equal(snapshot.state, 'complete', 'Regeneration completed after resume');
});

Then('it completes in one target generation and retrieval returns a private fact', async function () {
  const done = this.evidence.completed;
  assert.equal(done.completed_documents, DOCUMENTS);
  assert.deepEqual(done.target, target);
  const kb = await request(this, 'GET', this.kbPath);
  this.evidence.final_kb = kb.config;
  assert.deepEqual(kb.config.indexed_embedding, target, 'The KB is indexed in exactly the target space');
  const docs = await request(this, 'GET', `${this.kbPath}/documents`);
  assert.ok(docs.length === DOCUMENTS && docs.every(doc => doc.status === 'indexed'), 'Every document is indexed');
  const search = await request(this, 'POST', `${this.kbPath}/search`, { query: 'Which seal does paragraph 3 of archive note 17 record?', limit: 5, min_score: 0 }, 200, 75_000);
  this.evidence.retrieval = search.results.map(row => ({ document_id: row.document_id, score: row.score, content: row.content.slice(0, 160) }));
  // Chunk boundaries split paragraphs; any of note 17's private seals proves retrieval of that note.
  assert.ok(search.results.some(row => row.content.includes(`${this.seal}-17-`)), 'Retrieval returns a private fact of note 17 from the regenerated index');
  this.evidence.result = 'VERIFIED';
});

After({ timeout: 900_000 }, async function ({ result }) {
  if (!this.evidence) return;
  const errors = [];
  const cleanup = async operation => { try { await operation(); } catch (error) { errors.push(error.message); } };
  await cleanup(async () => { if (this.kbPath) await request(this, 'DELETE', this.kbPath, undefined, [204, 404]); });
  await cleanup(async () => {
    if (this.settingsTouched) {
      const rows = await currentSettings(this);
      await changeSettings(this, this.originalValues, rows);
      assert.deepEqual((await currentSettings(this)).map(row => row.saved), this.originalValues);
    }
  });
  await cleanup(async () => {
    if (this.hostState?.changed) {
      this.evidence.host_restore_restart = {};
      await restoreEmbeddingHost(this.hostState, this.evidence.host_restore_restart);
    }
  });
  await cleanup(async () => {
    const current = await request(this, 'GET', `${settings}/embedding/provenance`);
    assert.deepEqual(current.space, this.originalSpace);
    this.evidence.restored_embedding = current.space;
  });
  this.evidence.cleanup_errors = errors;
  this.evidence.status = errors.length ? 'FAILED' : result.status;
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify({ evidence: this.record, status: this.evidence.status }), 'application/json');
  assert.deepEqual(errors, [], 'Cleanup must succeed');
});
