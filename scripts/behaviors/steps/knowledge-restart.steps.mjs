import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { citesMarker, parseAnswerJson } from '../support/answer-oracle.mjs';
import { assertOwnedEndpoint, restartOwnedUar } from '../support/restart-owned-uar.mjs';

setDefaultTimeout(180_000);
const savedRecord = '.prometheus/verification/knowledge/wheel-profile-aae19b22-4654-493f-b34e-d487a21c3464.json';

async function request(world, method, path, payload, timeout = 20_000) {
  const exchange = { method, path, request: payload };
  world.evidence.exchanges.push(exchange);
  const response = await fetch(new URL(path, world.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(timeout),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': world.workspace },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  exchange.status = response.status;
  exchange.response = await response.text();
  assert.equal(response.status, 200, `${method} ${path}: ${response.status}: ${exchange.response.slice(0, 800)}`);
  return JSON.parse(exchange.response);
}

async function snapshot(world) {
  const agent = await request(world, 'GET', `/api/agents/${encodeURIComponent(world.agentId)}`);
  const kb = await request(world, 'GET', world.kbPath);
  const document = await request(world, 'GET', world.docPath);
  const embedding = await request(world, 'GET', '/api/uar/settings/embedding/provenance');
  assert.equal(agent.id, world.agentId);
  assert.equal(agent.memory.kb.enabled, true);
  assert.ok(agent.memory.kb.knowledge_bases.includes(world.kbId));
  assert.equal(kb.id, world.kbId);
  assert.equal(document.id, world.docId);
  assert.equal(document.kb_id, world.kbId);
  assert.equal(document.status, 'indexed');
  assert.ok(document.chunk_count > 0);
  assert.equal(embedding.status, 'serving');
  assert.equal(embedding.space.backend, 'fastembed', 'Use the retained local embedder; no hosted fallback');
  assert.equal(embedding.endpoint, null);
  assert.ok(embedding.provenance?.asset_manifest_sha256);
  assert.deepEqual(embedding.space, kb.config.indexed_embedding);
  return { agent, kb, document, embedding };
}

Given('the saved math agent retains its indexed research knowledge base', async function () {
  this.started = performance.now();
  this.session = randomUUID();
  this.record = resolve(`.prometheus/verification/knowledge-restart/${this.session}.json`);
  this.evidence = { scenario: 'KNOWLEDGE-RESTART-01', platform: process.platform,
    authentication_supplied_by_test: false, previous_scenarios_rerun: false,
    knowledge_mutations: false, history_replayed: false, exchanges: [] };
  this.base = new URL(process.env.UAR_BASE_URL);
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.model = process.env.UAR_MODEL;
  assert.ok(['http:', 'https:'].includes(this.base.protocol) && !this.base.username && !this.base.password);
  assert.ok(this.workspace && this.model);
  const prior = JSON.parse(await readFile(savedRecord, 'utf8'));
  assert.equal(prior.result, 'VERIFIED');
  this.agentId = prior.agent_id;
  this.kbId = prior.knowledge_base.id;
  this.docId = prior.document.id;
  this.profile = prior.profile;
  this.seal = prior.retrieval_seal;
  this.kbPath = `/api/uar/knowledge-bases/${encodeURIComponent(this.kbId)}`;
  this.docPath = `${this.kbPath}/documents/${encodeURIComponent(this.docId)}`;
  this.evidence.endpoint_binding = await assertOwnedEndpoint(this.base);
  this.before = await snapshot(this);
  const route = this.before.agent.policy.provider.default;
  assert.equal(`${route.provider}/${route.model}`, this.model);
  this.marker = this.before.agent.prompt.system.match(/MATH-[a-f0-9-]+/)?.[0];
  assert.ok(this.marker);
  assert.ok(!JSON.stringify(this.before.agent.prompt).includes(this.seal), 'The seal must not be supplied by agent instructions');
  Object.assign(this.evidence, { agent_id: this.agentId, knowledge_base_id: this.kbId,
    document_id: this.docId, profile: this.profile, expected_seal: this.seal,
    session_id: this.session, model: this.model, before: this.before, prior_record: savedRecord });
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(`${this.record}.before-restart.json`, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
});

When('UAR restarts without changing the agent or knowledge store', { timeout: 420_000 }, async function () {
  this.evidence.restart = {};
  await restartOwnedUar(this.evidence.restart);
  this.evidence.after_endpoint_binding = await assertOwnedEndpoint(this.base);
  this.evidence.after = await snapshot(this);
  for (const key of ['agent', 'kb', 'document']) assert.deepEqual(this.evidence.after[key], this.before[key]);
  assert.deepEqual(this.evidence.after.embedding.space, this.before.embedding.space);
  assert.equal(this.evidence.after.embedding.provenance.asset_manifest_sha256,
    this.before.embedding.provenance.asset_manifest_sha256);
});

When('I ask the math agent about its retained research profile in a fresh conversation', async function () {
  const question = `For research profile ${this.profile}, what are its rim_vertices, lucas_number and sandpile_group_order? Include its exact retrieval_seal and a numbered citation to the source. Return the research-profile JSON.`;
  assert.ok(!question.includes(this.seal));
  const payload = { model: this.model, agent_id: this.agentId, session_id: this.session,
    stream: false, memory_enabled: false, max_completion_tokens: 4096,
    messages: [{ role: 'user', content: question }] };
  assert.ok(!JSON.stringify(payload).includes(this.seal));
  this.completion = await request(this, 'POST', '/v1/chat/completions', payload, 120_000);
  assert.equal(this.completion.choices?.[0]?.finish_reason, 'stop');
  assert.match(this.completion.id, /^chatcmpl-.+/);
  this.runId = this.completion.id.slice('chatcmpl-'.length);
  this.evidence.run = await request(this, 'GET', `/api/uar/runs/${encodeURIComponent(this.runId)}`);
  this.evidence.events = await request(this, 'GET', `/api/uar/runs/${encodeURIComponent(this.runId)}/events`);
});

Then('the answer uses the retained document with matching retrieval evidence', function () {
  const answer = parseAnswerJson(this.completion.choices[0].message.content);
  let previous = 2, current = 1;
  for (let n = 2; n <= 11; n++) [previous, current] = [current, previous + current];
  assert.equal(answer.specialist, this.marker);
  assert.equal(answer.profile, this.profile);
  assert.equal(answer.rim_vertices, 11);
  assert.equal(answer.lucas_number, current);
  assert.equal(answer.sandpile_group_order, current ** 2);
  assert.equal(answer.retrieval_seal, this.seal);
  assert.equal(this.evidence.run.agent_id, this.agentId);
  assert.equal(this.evidence.run.conversation_id, this.session);
  assert.equal(this.evidence.run.status, 'done');
  assert.equal(this.evidence.events.gapReason ?? null, null);
  const citations = this.evidence.events.events.filter(event => event.eventName === 'agui.rag_citations')
    .flatMap(event => event.data.citations ?? []);
  this.evidence.matched_citations = citations.filter(citation => citation.knowledge_base_id === this.kbId
    && citation.document_id === this.docId && citation.snippet.includes(this.seal)
    && citesMarker(answer.citation, citation.marker));
  assert.ok(this.evidence.matched_citations.length > 0, 'Require an actual retained-document retrieval matching the answer citation');
  assert.equal(this.evidence.restart.previous_process_exited, true);
  assert.equal(this.evidence.restart.new_process_ready, true);
  this.evidence.answer = answer;
  this.evidence.run_id = this.runId;
  this.evidence.result = 'VERIFIED';
});

After(async function ({ result }) {
  if (!this.evidence) return;
  this.evidence.status = result.status;
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify({ evidence: this.record, status: this.evidence.status }), 'application/json');
});
