import assert from 'node:assert/strict';
import { randomInt, randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { assertOwnedEndpoint } from '../support/restart-owned-uar.mjs';

setDefaultTimeout(360_000);
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
async function prompt(world) {
  const session = randomUUID();
  const question = `Calculate ${world.left} + ${world.right}. Also give the private retrieval seal for laboratory profile ${world.profile} if it is available in retrieved knowledge.`;
  assert.ok(!question.includes(world.seal));
  const completion = await request(world, 'POST', '/v1/chat/completions', { agent_id: world.agentId,
    model: world.model, session_id: session, stream: false, memory_enabled: false, max_completion_tokens: 4096,
    messages: [{ role: 'user', content: question }] }, 200, 120_000);
  assert.equal(completion.model, world.model);
  assert.equal(completion.session_id, session);
  assert.equal(completion.choices?.[0]?.finish_reason, 'stop');
  assert.match(completion.id, /^chatcmpl-.+/);
  const runId = completion.id.slice('chatcmpl-'.length);
  const run = await request(world, 'GET', `/api/uar/runs/${encodeURIComponent(runId)}`);
  const events = await request(world, 'GET', `/api/uar/runs/${encodeURIComponent(runId)}/events`);
  assert.equal(run.agent_id, world.agentId);
  assert.equal(run.conversation_id, session);
  assert.equal(run.status, 'done');
  assert.equal(events.gapReason ?? null, null);
  assert.ok(Array.isArray(events.events) && events.events.length > 0);
  const citations = events.events.filter(event => event.eventName === 'agui.rag_citations').flatMap(event => event.data.citations ?? []);
  const final = completion.choices[0].message.content.replace(/^(?:\s*<think>[\s\S]*?<\/think>)+\s*/, '').trim();
  const fenced = /^```(?:json)?\s*\n([\s\S]*?)\n```$/.exec(final);
  const answer = JSON.parse(fenced ? fenced[1] : final);
  assert.equal(answer.agent_marker, world.marker);
  assert.equal(answer.sum, world.left + world.right);
  return { question, session, run_id: runId, run, events, citations, completion, answer };
}

Given('a disposable agent linked to a knowledge base containing a private fact', async function () {
  this.started = performance.now();
  this.id = randomUUID();
  this.record = resolve(`.prometheus/verification/linked-kb-delete/${this.id}.json`);
  this.evidence = { scenario: 'LINKED-KB-DELETE-01', platform: process.platform, authentication_supplied_by_test: false,
    previous_scenarios_rerun: false, history_replayed: false, configuration_changes: false, exchanges: [] };
  this.base = new URL(process.env.UAR_BASE_URL);
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.model = process.env.UAR_MODEL;
  assert.ok(this.workspace && this.model);
  this.evidence.endpoint = await assertOwnedEndpoint(this.base);
  this.evidence.embedding = await request(this, 'GET', '/api/uar/settings/embedding/provenance');
  assert.equal(this.evidence.embedding.status, 'serving');
  assert.equal(this.evidence.embedding.space.backend, 'fastembed');
  assert.equal(this.evidence.embedding.endpoint, null);
  this.profile = `lab-profile-${this.id}`;
  this.seal = `KNOWLEDGE-${randomUUID()}`;
  this.marker = `AGENT-${randomUUID()}`;
  this.left = randomInt(100, 500); this.right = randomInt(100, 500);
  this.documentText = `Laboratory profile ${this.profile} has the private retrieval seal ${this.seal}. This exact seal is available only in this laboratory knowledge document.`;
  const kb = await request(this, 'POST', root, { name: `Linked KB deletion ${this.id}`, config: {
    chunk_strategy: 'fixed', chunk_size: 512, retrieval_min_score: 0, retrieval_top_k: 5 } }, 201);
  this.kbId = kb.id;
  assert.ok(this.kbId);
  this.kbPath = `${root}/${encodeURIComponent(this.kbId)}`;
  Object.assign(this.evidence, { knowledge_base_id: this.kbId, profile: this.profile, expected_seal: this.seal });
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(`${this.record}.before.json`, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  const form = new FormData();
  form.append('file', new Blob([this.documentText], { type: 'text/plain' }), 'laboratory-profile.txt');
  const document = await request(this, 'POST', `${this.kbPath}/documents`, form, 202);
  this.docId = document.id;
  assert.ok(this.docId);
  const deadline = Date.now() + 120_000;
  let indexed = false;
  while (Date.now() < deadline) {
    const doc = await request(this, 'GET', `${this.kbPath}/documents/${encodeURIComponent(this.docId)}`);
    assert.notEqual(doc.status, 'failed', JSON.stringify(doc));
    if (doc.status === 'indexed') { assert.ok(doc.chunk_count > 0); this.evidence.document = doc; indexed = true; break; }
    await delay(500);
  }
  assert.ok(indexed, 'The original document must finish indexing');
  const agent = await request(this, 'GET', '/api/agents/default-agent');
  this.agentId = `disposable-kb-delete-${this.id}`;
  this.agentPath = `/api/agents/${this.agentId}`;
  agent.id = this.agentId; agent.version = '1.0.0';
  agent.metadata = { ...agent.metadata, title: 'Disposable linked knowledge deletion agent', description: 'Owned by one manual BDD.', tags: ['disposable', 'verification'] };
  agent.runtime.entry = 'default';
  agent.prompt = { system: `Return only JSON with agent_marker, sum, knowledge_seal and citation. Always set agent_marker to "${this.marker}". Calculate the requested sum independently. Copy knowledge_seal only from retrieved knowledge for the requested laboratory profile; copy its numbered citation such as [1]. If that profile is absent from retrieved knowledge, set knowledge_seal and citation to null. Never invent a private seal.`, instructions: [] };
  const split = this.model.indexOf('/');
  assert.ok(split > 0 && split < this.model.length - 1);
  agent.policy.provider = { default: { provider: this.model.slice(0, split), model: this.model.slice(split + 1) }, fallbacks: [] };
  agent.policy.tools.allow = [];
  agent.memory.kb = { ...agent.memory.kb, enabled: true, knowledge_bases: [this.kbId], citation_required: true };
  assert.ok(!JSON.stringify(agent).includes(this.seal));
  await request(this, 'POST', '/api/agents', agent, 201);
  this.savedAgent = await request(this, 'GET', this.agentPath);
  assert.deepEqual(this.savedAgent.memory.kb.knowledge_bases, [this.kbId]);
  assert.equal(this.savedAgent.memory.kb.enabled, true);
  this.evidence.agent_before = this.savedAgent;
});

Given('the agent answers using that fact with a matching knowledge citation', async function () {
  this.before = await prompt(this);
  this.evidence.before = this.before;
  assert.equal(this.before.answer.knowledge_seal, this.seal);
  assert.ok(this.before.citations.some(citation => citation.knowledge_base_id === this.kbId
    && citation.document_id === this.docId && citation.snippet.includes(this.seal)
    && (this.before.answer.citation === `[${citation.marker}]`
      || this.before.answer.citation === `[${citation.marker}] (${citation.document_name})`)));
});

When('I delete its knowledge base without editing the agent', async function () {
  await request(this, 'DELETE', this.kbPath, undefined, 204);
  this.kbDeleted = true;
  await request(this, 'GET', this.kbPath, undefined, 404);
  const agent = await request(this, 'GET', this.agentPath);
  assert.deepEqual(agent, this.savedAgent, 'The agent definition and its original KB reference must remain unchanged');
  this.evidence.agent_after_deletion = agent;
});

Then('the same agent answers a fresh conversation without the deleted fact or citations', async function () {
  const after = await prompt(this);
  this.evidence.after = after;
  assert.notEqual(after.session, this.before.session);
  assert.notEqual(after.run_id, this.before.run_id);
  assert.equal(after.question, this.before.question);
  assert.equal(after.answer.knowledge_seal, null);
  assert.equal(after.answer.citation, null);
  assert.equal(after.citations.length, 0);
  assert.ok(!JSON.stringify(after.completion).includes(this.seal));
  assert.ok(!JSON.stringify(after.events).includes(this.seal));
  this.behaviorPassed = true;
});

After({ timeout: 60_000 }, async function ({ result }) {
  if (!this.evidence) return;
  const errors = [];
  for (const [needed, path, field] of [[this.agentCreated, this.agentPath, 'agent_deleted'],
    [this.kbPath && !this.kbDeleted, this.kbPath, 'kb_cleanup_deleted']]) {
    if (!needed) continue;
    try { await request(this, 'DELETE', path, undefined, 204); this.evidence[field] = true; }
    catch (error) { errors.push(error); }
  }
  this.evidence.knowledge_base_deleted = !!this.kbDeleted || !!this.evidence.kb_cleanup_deleted;
  this.evidence.status = errors.length ? 'FAILED' : result.status;
  this.evidence.result = !errors.length && this.behaviorPassed && result.status === 'PASSED' ? 'VERIFIED' : 'FAILED';
  this.evidence.cleanup_errors = errors.map(error => error.message);
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify({ evidence: this.record, status: this.evidence.status }), 'application/json');
  if (errors.length) throw new AggregateError(errors, 'Owned resource cleanup failed');
});
