import assert from 'node:assert/strict';
import { createHash, randomInt, randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { assertOwnedEndpoint } from '../support/restart-owned-uar.mjs';

setDefaultTimeout(240_000);
const ROOT = '/api/uar/knowledge-bases';

async function request(w, method, path, payload, expected = 200, timeout = 30_000) {
  const multipart = payload instanceof FormData;
  const exchange = { method, path, request: multipart ? { file: w.uploadName } : payload };
  w.evidence.exchanges.push(exchange);
  const response = await fetch(new URL(path, w.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(timeout),
    headers: { ...(!multipart && { 'Content-Type': 'application/json' }), 'x-uar-workspace-id': w.workspace },
    body: payload === undefined ? undefined : multipart ? payload : JSON.stringify(payload),
  });
  exchange.status = response.status;
  const text = await response.text();
  exchange.response = text.slice(0, 3000);
  const accepted = Array.isArray(expected) ? expected : [expected];
  assert.ok(accepted.includes(response.status), `${method} ${path}: ${response.status}: ${text.slice(0, 800)}`);
  return text && response.ok ? JSON.parse(text) : null;
}

async function uploadIndexed(w, name, content) {
  w.uploadName = name;
  const form = new FormData();
  form.append('file', new Blob([content], { type: 'text/plain' }), name);
  const uploaded = await request(w, 'POST', `${w.kbPath}/documents`, form, 202);
  const deadline = Date.now() + 120_000;
  while (Date.now() < deadline) {
    const doc = await request(w, 'GET', `${w.kbPath}/documents/${encodeURIComponent(uploaded.id)}`);
    assert.notEqual(doc.status, 'failed', JSON.stringify(doc));
    if (doc.status === 'indexed') {
      assert.ok(doc.chunk_count > 0);
      return { ...doc, sha256: createHash('sha256').update(content).digest('hex') };
    }
    await delay(500);
  }
  throw new Error('Document ingestion deadline exceeded');
}

Given('a disposable knowledge base with one indexed note holding a private value', async function () {
  this.started = performance.now();
  this.id = randomUUID();
  this.record = resolve(`.prometheus/verification/kb-source-replace/${this.id}.json`);
  this.base = new URL(process.env.UAR_BASE_URL);
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.model = process.env.UAR_MODEL;
  this.sessions = [];
  this.evidence = { scenario: 'C47-KB-SOURCE-REPLACE-01', platform: process.platform, model: this.model,
    authentication_supplied_by_test: false, previous_scenarios_rerun: false, exchanges: [] };
  assert.ok(this.workspace && this.model?.includes('/'));
  this.evidence.endpoint = await assertOwnedEndpoint(this.base);
  this.evidence.embedding = await request(this, 'GET', '/api/uar/settings/embedding/provenance');
  assert.equal(this.evidence.embedding.status, 'serving');
  this.oldValue = randomInt(100, 499);
  this.newValue = randomInt(500, 999);
  this.oldSeal = `OLD-${randomUUID().slice(0, 8).toUpperCase()}`;
  this.newSeal = `NEW-${randomUUID().slice(0, 8).toUpperCase()}`;
  const kb = await request(this, 'POST', ROOT, { name: `KB replacement BDD ${this.id}`, description: 'Disposable replacement source',
    config: { chunk_strategy: 'fixed', chunk_size: 512, retrieval_min_score: 0, retrieval_top_k: 10 } }, 201);
  this.kbId = kb.id;
  this.kbPath = `${ROOT}/${encodeURIComponent(this.kbId)}`;
  this.old = await uploadIndexed(this, 'harbor-ledger-old.txt',
    `The Larkspur harbor ledger records ${this.oldValue} moorings. Ledger seal ${this.oldSeal}.`);
  Object.assign(this.evidence, { knowledge_base_id: this.kbId, old_document: this.old, old_value: this.oldValue, new_value: this.newValue });
});

Given('a disposable agent linked to that knowledge base with citations required', async function () {
  const agent = await request(this, 'GET', '/api/agents/default-agent');
  this.agentId = `disposable-kb-replace-${this.id.slice(0, 8)}`;
  this.agentPath = `/api/agents/${this.agentId}`;
  Object.assign(agent, { id: this.agentId, version: '1.0.0', extensions: {} });
  agent.metadata = { ...agent.metadata, title: 'Disposable harbor ledger agent', tags: ['disposable', 'verification'] };
  agent.runtime.entry = 'default';
  agent.prompt = { system: 'Answer only from the retrieved knowledge base sources. Return only JSON with moorings (number), seal (string) and citation (the numbered citation such as [1]). If the sources do not contain it, return null values. Never guess.', instructions: [] };
  const split = this.model.indexOf('/');
  agent.policy.provider = { default: { provider: this.model.slice(0, split), model: this.model.slice(split + 1) }, fallbacks: [] };
  agent.policy.tools.allow = [];
  agent.memory.kb = { ...agent.memory.kb, enabled: true, knowledge_bases: [this.kbId], citation_required: true };
  await request(this, 'POST', '/api/agents', agent, 201);
  this.agentCreated = true;
});

When('I delete the old note and index a replacement note with a different value', async function () {
  await request(this, 'DELETE', `${this.kbPath}/documents/${encodeURIComponent(this.old.id)}`, undefined, 204);
  await request(this, 'GET', `${this.kbPath}/documents/${encodeURIComponent(this.old.id)}`, undefined, 404);
  this.replacement = await uploadIndexed(this, 'harbor-ledger-new.txt',
    `The Larkspur harbor ledger records ${this.newValue} moorings. Ledger seal ${this.newSeal}.`);
  assert.notEqual(this.replacement.id, this.old.id);
  this.evidence.replacement_document = this.replacement;
});

Then('retrieval returns only the replacement note', async function () {
  const search = await request(this, 'POST', `${this.kbPath}/search`, { query: 'How many moorings does the Larkspur harbor ledger record, and what is its seal?', limit: 10, min_score: 0 });
  this.evidence.retrieval = search;
  assert.ok(search.results.length > 0, 'Retrieval must return the replacement');
  assert.ok(search.results.every(row => row.document_id !== this.old.id && !row.content.includes(this.oldSeal)), 'The old note is absent from retrieval');
  assert.ok(search.results.some(row => row.document_id === this.replacement.id && row.content.includes(this.newSeal)));
  const documents = await request(this, 'GET', `${this.kbPath}/documents`);
  assert.deepEqual(documents.map(doc => doc.id), [this.replacement.id]);
});

Then('a fresh conversation answers with only the replacement value and its current citation', async function () {
  const session = randomUUID();
  this.sessions.push(session);
  const completion = await request(this, 'POST', '/v1/chat/completions', { agent_id: this.agentId, model: this.model, session_id: session,
    stream: false, memory_enabled: false, max_completion_tokens: 4096,
    messages: [{ role: 'user', content: 'How many moorings does the Larkspur harbor ledger record, and what is its seal?' }] }, 200, 150_000);
  const final = completion.choices[0].message.content.replace(/<think>[\s\S]*?<\/think>/g, '').trim();
  let answer;
  try { answer = JSON.parse(final); } catch {
    const blocks = [...final.matchAll(/```(?:json)?\s*([\s\S]*?)```/g)];
    assert.equal(blocks.length, 1, `Require one JSON answer: ${final.slice(0, 300)}`);
    answer = JSON.parse(blocks[0][1]);
  }
  const runId = completion.id.slice('chatcmpl-'.length);
  const events = (await request(this, 'GET', `/api/uar/runs/${encodeURIComponent(runId)}/events`)).events;
  const citations = events.filter(e => e.eventName === 'agui.rag_citations').flatMap(e => e.data.citations ?? []);
  Object.assign(this.evidence, { run_id: runId, answer, citations });
  assert.equal(answer.moorings, this.newValue);
  assert.equal(answer.seal, this.newSeal);
  assert.ok(!final.includes(this.oldSeal) && !final.includes(String(this.oldValue)), 'The old fact is absent from the answer');
  assert.ok(citations.every(c => c.document_id !== this.old.id), 'No citation points at the deleted note');
  const matched = citations.filter(c => c.document_id === this.replacement.id && c.snippet.includes(this.newSeal)
    && String(answer.citation).startsWith(`[${c.marker}]`));
  assert.ok(matched.length > 0, 'The answer cites the replacement note');
  this.evidence.result = 'VERIFIED';
});

After({ timeout: 60_000 }, async function ({ result }) {
  if (!this.evidence) return;
  const errors = [];
  for (const session of this.sessions) {
    try { await request(this, 'DELETE', `/api/uar/sessions/${session}`, undefined, [204, 404]); }
    catch (error) { errors.push(`Session cleanup ${session}: ${error}`); }
  }
  for (const path of [this.agentCreated && this.agentPath, this.kbPath].filter(Boolean)) {
    try { await request(this, 'DELETE', path, undefined, [204, 404]); }
    catch (error) { errors.push(`${path}: ${error}`); }
  }
  this.evidence.cleanup_errors = errors;
  this.evidence.status = errors.length ? 'FAILED' : result.status;
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify({ evidence: this.record, status: this.evidence.status }), 'application/json');
  assert.deepEqual(errors, [], 'Cleanup must succeed');
});
