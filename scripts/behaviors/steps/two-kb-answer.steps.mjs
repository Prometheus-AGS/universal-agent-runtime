import assert from 'node:assert/strict';
import { randomInt, randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { citesMarker, parseAnswerJson } from '../support/answer-oracle.mjs';
import { assertOwnedEndpoint } from '../support/restart-owned-uar.mjs';

setDefaultTimeout(600_000);
const root = '/api/uar/knowledge-bases';
async function request(world, method, path, payload, expected = 200, timeout = 15_000) {
  const multipart = payload instanceof FormData;
  const exchange = { method, path, request: multipart ? { file: world.documentText } : payload };
  world.evidence.exchanges.push(exchange);
  const response = await fetch(new URL(path, world.base), { method, redirect: 'error', signal: AbortSignal.timeout(timeout),
    headers: { ...(!multipart && { 'Content-Type': 'application/json' }), 'x-uar-workspace-id': world.workspace },
    body: payload === undefined ? undefined : multipart ? payload : JSON.stringify(payload) });
  if (method === 'POST' && path === '/api/agents' && response.status === 201) world.agentCreated = true;
  exchange.status = response.status;
  exchange.response = await response.text();
  assert.equal(response.status, expected, `${method} ${path}: ${response.status}: ${exchange.response.slice(0, 800)}`);
  if (!exchange.response || expected === 404) return exchange.response;
  return JSON.parse(exchange.response);
}

Given('a disposable agent and two indexed knowledge bases with separate private facts', async function () {
  this.started = performance.now();
  this.id = randomUUID();
  this.record = resolve(`.prometheus/verification/two-kb-answer/${this.id}.json`);
  this.evidence = { scenario: 'TWO-KB-ANSWER-01', platform: process.platform,
    authentication_supplied_by_test: false, previous_scenarios_rerun: false, history_replayed: false,
    global_configuration_changes: false, exchanges: [], sources: [] };
  this.base = new URL(process.env.UAR_BASE_URL);
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.model = process.env.UAR_MODEL;
  assert.ok(this.workspace && this.model);
  this.evidence.endpoint = await assertOwnedEndpoint(this.base);
  this.evidence.embedding = await request(this, 'GET', '/api/uar/settings/embedding/provenance');
  assert.equal(this.evidence.embedding.status, 'serving');
  assert.equal(this.evidence.embedding.space.backend, 'fastembed');
  assert.equal(this.evidence.embedding.endpoint, null);
  this.project = `inventory-project-${this.id}`;
  this.marker = `AGENT-${randomUUID()}`;
  for (const [name, minimum, maximum] of [['amber', 110, 400], ['cobalt', 510, 900]]) {
    const source = { name, units: randomInt(minimum, maximum), seal: `${name.toUpperCase()}-${randomUUID()}` };
    const kb = await request(this, 'POST', root, { name: `${name} inventory ${this.id}`, config: {
      chunk_strategy: 'fixed', chunk_size: 512, retrieval_min_score: 0, retrieval_top_k: 5 } }, 201);
    assert.ok(kb.id);
    source.kb_id = kb.id;
    source.path = `${root}/${encodeURIComponent(kb.id)}`;
    this.evidence.sources.push(source);
    this.documentText = `For project ${this.project}, the ${name} inventory has ${source.units} units. Its private source seal is ${source.seal}. This inventory note is the authoritative source for ${name} only.`;
    const form = new FormData();
    form.append('file', new Blob([this.documentText], { type: 'text/plain' }), `${name}-inventory.txt`);
    const document = await request(this, 'POST', `${source.path}/documents`, form, 202);
    assert.ok(document.id);
    source.document_id = document.id;
    const deadline = Date.now() + 120_000;
    while (Date.now() < deadline) {
      const doc = await request(this, 'GET', `${source.path}/documents/${encodeURIComponent(document.id)}`);
      assert.notEqual(doc.status, 'failed', JSON.stringify(doc));
      if (doc.status === 'indexed') { assert.ok(doc.chunk_count > 0); source.document = doc; break; }
      await delay(500);
    }
    assert.ok(source.document, 'Both source documents must finish indexing');
    source.kb = await request(this, 'GET', source.path);
    assert.equal(source.kb.document_count, 1);
    assert.deepEqual(source.kb.config.indexed_embedding, this.evidence.embedding.space);
  }
  assert.notEqual(this.evidence.sources[0].kb_id, this.evidence.sources[1].kb_id);
  assert.notEqual(this.evidence.sources[0].document_id, this.evidence.sources[1].document_id);
  const agent = await request(this, 'GET', '/api/agents/default-agent');
  this.agentId = `disposable-two-kb-${this.id}`;
  this.agentPath = `/api/agents/${this.agentId}`;
  agent.id = this.agentId; agent.version = '1.0.0';
  agent.metadata = { ...agent.metadata, title: 'Disposable two-KB inventory agent', description: 'Owned by one manual BDD.', tags: ['disposable', 'verification'] };
  agent.runtime.entry = 'default';
  agent.prompt = { system: `Return only JSON with agent_marker, amber, cobalt, and total. Always set agent_marker to "${this.marker}". For each inventory, return an object with units, seal, and citation. Read units and exact private source seal only from the retrieved source for the requested project, copying its numbered citation such as [1]. Calculate total from both inventory quantities. If either inventory is unavailable return null for that inventory and total. Never invent quantities or seals.`, instructions: [] };
  const split = this.model.indexOf('/');
  assert.ok(split > 0 && split < this.model.length - 1);
  agent.policy.provider = { default: { provider: this.model.slice(0, split), model: this.model.slice(split + 1) }, fallbacks: [] };
  agent.policy.tools.allow = [];
  agent.memory.kb = { ...agent.memory.kb, enabled: false, knowledge_bases: [], citation_required: true };
  for (const source of this.evidence.sources) assert.ok(!JSON.stringify(agent).includes(source.seal));
  await request(this, 'POST', '/api/agents', agent, 201);
  this.savedAgent = await request(this, 'GET', this.agentPath);
  assert.equal(this.savedAgent.memory.kb.enabled, false);
  assert.deepEqual(this.savedAgent.memory.kb.knowledge_bases, []);
  Object.assign(this.evidence, { agent_before: this.savedAgent, project: this.project });
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(`${this.record}.before.json`, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
});

When('I link both knowledge bases to the agent', async function () {
  this.ids = this.evidence.sources.map(source => source.kb_id);
  const patched = await request(this, 'PATCH', this.agentPath, { memory: { kb: { enabled: true, knowledge_bases: this.ids } } });
  const saved = await request(this, 'GET', this.agentPath);
  assert.deepEqual(saved, patched);
  assert.equal(saved.memory.kb.enabled, true);
  assert.deepEqual(saved.memory.kb.knowledge_bases, this.ids);
  assert.equal(saved.extensions['uar.run_policy'].knowledge_bases.mode, 'selected');
  assert.deepEqual([...saved.extensions['uar.run_policy'].knowledge_bases.ids].sort(), [...this.ids].sort());
  for (const key of ['id', 'prompt', 'policy', 'runtime']) assert.deepEqual(saved[key], this.savedAgent[key]);
  this.evidence.agent_linked = saved;
});

When('I ask for the two inventories and their combined total in a fresh conversation', async function () {
  this.session = randomUUID();
  const question = `For project ${this.project}, report the amber and cobalt inventories with their units, private source seals, and numbered citations, then calculate the combined total.`;
  const payload = { agent_id: this.agentId, model: this.model, session_id: this.session, stream: false,
    memory_enabled: false, max_completion_tokens: 4096, messages: [{ role: 'user', content: question }] };
  for (const source of this.evidence.sources) {
    assert.ok(!question.replace(this.project, '').includes(String(source.units)) && !question.includes(source.seal), 'Do not supply either private fact in the question');
  }
  this.completion = await request(this, 'POST', '/v1/chat/completions', payload, 200, 120_000);
  assert.equal(this.completion.model, this.model);
  assert.equal(this.completion.session_id, this.session);
  assert.equal(this.completion.choices?.[0]?.finish_reason, 'stop');
  assert.match(this.completion.id, /^chatcmpl-.+/);
  this.runId = this.completion.id.slice('chatcmpl-'.length);
  this.evidence.run = await request(this, 'GET', `/api/uar/runs/${encodeURIComponent(this.runId)}`);
  this.evidence.events = await request(this, 'GET', `/api/uar/runs/${encodeURIComponent(this.runId)}/events`);
  assert.equal(this.evidence.run.agent_id, this.agentId);
  assert.equal(this.evidence.run.conversation_id, this.session);
  assert.equal(this.evidence.run.status, 'done');
  assert.equal(this.evidence.events.gapReason ?? null, null);
  assert.ok(Array.isArray(this.evidence.events.events) && this.evidence.events.events.length > 0);
  Object.assign(this.evidence, { session_id: this.session, run_id: this.runId, completion: this.completion });
});

Then('the answer combines both private facts with a matching citation for each knowledge base', async function () {
  const answer = parseAnswerJson(this.completion.choices[0].message.content);
  assert.equal(answer.agent_marker, this.marker);
  assert.equal(answer.total, this.evidence.sources.reduce((sum, source) => sum + source.units, 0));
  const citations = this.evidence.events.events.filter(event => event.eventName === 'agui.rag_citations').flatMap(event => event.data.citations ?? []);
  assert.ok(citations.every(citation => this.ids.includes(citation.knowledge_base_id)));
  this.evidence.matched_citations = [];
  for (const source of this.evidence.sources) {
    const part = answer[source.name];
    assert.ok(part);
    assert.equal(part.units, source.units);
    assert.equal(part.seal, source.seal);
    const matched = citations.filter(citation => citation.knowledge_base_id === source.kb_id
      && citation.document_id === source.document_id && citation.snippet.includes(source.seal)
      && citesMarker(part.citation, citation.marker));
    assert.ok(matched.length > 0, `Require actual ${source.name} KB/document citation matching its answer`);
    this.evidence.matched_citations.push(...matched);
    assert.deepEqual(await request(this, 'GET', source.path), source.kb);
    assert.deepEqual(await request(this, 'GET', `${source.path}/documents/${encodeURIComponent(source.document_id)}`), source.document);
  }
  assert.equal(new Set(this.evidence.matched_citations.map(c => c.knowledge_base_id)).size, 2);
  this.evidence.answer = answer;
  this.behaviorPassed = true;
});

After({ timeout: 75_000 }, async function ({ result }) {
  if (!this.evidence) return;
  const errors = [];
  const resources = this.evidence.sources.map(source => ({ path: source.path, source }));
  if (this.agentCreated) resources.unshift({ path: this.agentPath });
  for (const resource of resources) {
    try {
      await request(this, 'DELETE', resource.path, undefined, 204);
      if (resource.source) resource.source.cleaned_up = true;
      else this.evidence.agent_deleted = true;
    } catch (error) { errors.push(error); }
  }
  this.evidence.status = errors.length ? 'FAILED' : result.status;
  this.evidence.result = !errors.length && this.behaviorPassed && result.status === 'PASSED' ? 'VERIFIED' : 'FAILED';
  this.evidence.cleanup_errors = errors.map(error => error.message);
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify({ evidence: this.record, status: this.evidence.status }), 'application/json');
  if (errors.length) throw new AggregateError(errors, 'Owned resource cleanup failed');
});
