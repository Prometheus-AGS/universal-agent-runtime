import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { citesMarker, parseAnswerJson } from '../support/answer-oracle.mjs';

setDefaultTimeout(125_000);
const root = '/api/uar/knowledge-bases';

async function request(world, method, path, payload, expected = 200) {
  const headers = {};
  if (process.env.UAR_WORKSPACE_ID) headers['x-uar-workspace-id'] = process.env.UAR_WORKSPACE_ID;
  const multipart = payload instanceof FormData;
  if (payload !== undefined && !multipart) headers['Content-Type'] = 'application/json';
  const response = await fetch(new URL(path, world.base), {
    method, headers, redirect: 'error', signal: AbortSignal.timeout(120_000),
    body: payload === undefined ? undefined : multipart ? payload : JSON.stringify(payload),
  });
  const text = await response.text();
  assert.equal(response.status, expected, `HTTP ${response.status}: ${text.slice(0, 1500)}`);
  return JSON.parse(text);
}

async function retain(world) {
  await mkdir(dirname(world.record), { recursive: true });
  await writeFile(world.record, JSON.stringify(world.evidence, null, 2) + '\n', { mode: 0o600 });
}

Given('the saved mathematics agent and a real embedding model are available', async function () {
  this.base = new URL(process.env.UAR_BASE_URL ?? 'http://127.0.0.1:3000');
  assert.ok(['http:', 'https:'].includes(this.base.protocol));
  assert.ok(!this.base.username && !this.base.password);
  assert.ok(process.env.UAR_MODEL, 'Set UAR_MODEL to the real provider/model');
  const saved = JSON.parse(await readFile(process.env.UAR_MATH_AGENT_RECORD ??
    '.prometheus/verification/agents/math-specialist-db64c2a8.json', 'utf8'));
  this.agentId = saved.agent_id;
  this.agent = await request(this, 'GET', `/api/agents/${encodeURIComponent(this.agentId)}`);
  this.marker = saved.agent.prompt.system.match(/MATH-[a-f0-9-]+/)[0];
  this.embedding = await request(this, 'GET', '/api/uar/settings/embedding/provenance');
  assert.equal(this.embedding.status, 'serving');
  assert.ok(this.embedding.provenance?.asset_manifest_sha256 || this.embedding.endpoint,
    'A real embedding model must have local asset provenance or a configured endpoint');
  this.profile = `wheel-profile-${randomUUID()}`;
  this.seal = randomUUID();
  this.record = resolve(`.prometheus/verification/knowledge/${this.profile}.json`);
  this.evidence = { platform: process.platform, agent_id: this.agentId, profile: this.profile,
    retrieval_seal: this.seal, embedding: this.embedding, retained_for_future_use: true };
});

When('I create a specialized mathematics knowledge base and upload a research note', async function () {
  const space = this.embedding.space;
  this.kb = await request(this, 'POST', root, { name: this.profile,
    description: 'Sourced wheel-graph critical group mathematics with a private retrieval profile',
    config: { chunk_strategy: 'fixed', chunk_size: 2048, retrieval_min_score: 0,
      retrieval_top_k: 3, embedding_provider: space.backend, embedding_model: space.model,
      vector_dimensions: space.dimension } }, 201);
  this.kbPath = `${root}/${encodeURIComponent(this.kb.id)}`;
  this.evidence.knowledge_base = this.kb;
  await retain(this);
  const note = await readFile('scripts/behaviors/knowledge/wheel-graphs.md', 'utf8');
  const profile = `\nLocal research profile ${this.profile}: rim_vertices=11; lucas_number=199; sandpile_group_order=39601; retrieval_seal=${this.seal}.\n`;
  const form = new FormData();
  form.append('file', new Blob([profile + note], { type: 'text/plain' }), 'wheel-graphs.txt');
  this.document = await request(this, 'POST', `${this.kbPath}/documents`, form, 202);
  this.evidence.document = this.document;
  await retain(this);
});

When('the research note finishes indexing', async function () {
  const deadline = Date.now() + 90_000;
  do {
    this.job = await request(this, 'GET', `${this.kbPath}/documents/${encodeURIComponent(this.document.id)}/ingestion`);
    this.evidence.ingestion = this.job;
    assert.notEqual(this.job.state, 'failed', JSON.stringify(this.job));
    if (this.job.state === 'indexed') break;
    await delay(250);
  } while (Date.now() < deadline);
  assert.equal(this.job.state, 'indexed', 'Research note did not finish indexing within 90 seconds');
  this.evidence.indexed_knowledge_base = await request(this, 'GET', this.kbPath);
  await retain(this);
});

When('I assign that knowledge base to the saved mathematics agent', async function () {
  const ids = [...new Set([...(this.agent.memory.kb.knowledge_bases ?? []), this.kb.id])];
  const system = `You are a mathematics specialist. Return only JSON and always set specialist to "${this.marker}". For ordinary arithmetic, return keys specialist and answer with the requested numeric result. For a research-profile question, answer ONLY from the retrieved knowledge documents: return keys specialist, profile, rim_vertices, lucas_number, sandpile_group_order, retrieval_seal, citation. Copy the exact profile and retrieval seal from the document and its numbered source marker such as [1]. If the profile is absent return specialist, answer="UNKNOWN", citation=null. Never invent a retrieval seal.`;
  this.updated = await request(this, 'PATCH', `/api/agents/${encodeURIComponent(this.agentId)}`, {
    memory: { kb: { ...this.agent.memory.kb, enabled: true, knowledge_bases: ids, citation_required: true } },
    prompt: { ...this.agent.prompt, system },
  });
  const saved = await request(this, 'GET', `/api/agents/${encodeURIComponent(this.agentId)}`);
  assert.equal(saved.memory.kb.enabled, true);
  assert.ok(saved.memory.kb.knowledge_bases.includes(this.kb.id));
  assert.equal(saved.prompt.system, system);
  this.evidence.agent = saved;
  await retain(this);
});

When('I ask the mathematics agent about the private research profile', async function () {
  this.question = `For research profile ${this.profile}, what are its rim_vertices, lucas_number and sandpile_group_order? Include its exact retrieval_seal and a numbered citation to the source. Return the research-profile JSON.`;
  assert.ok(!this.question.includes(this.seal));
  this.completion = await request(this, 'POST', '/v1/chat/completions', {
    model: process.env.UAR_MODEL, agent_id: this.agentId, session_id: randomUUID(),
    stream: false, memory_enabled: false, max_completion_tokens: 512,
    messages: [{ role: 'user', content: this.question }],
  });
  this.evidence.question = this.question;
  this.evidence.completion = this.completion;
  await retain(this);
  assert.equal(this.completion.choices?.[0]?.finish_reason, 'stop');
  assert.match(this.completion.id, /^chatcmpl-/);
  const runId = this.completion.id.slice('chatcmpl-'.length);
  this.evidence.run = await request(this, 'GET', `/api/uar/runs/${encodeURIComponent(runId)}`);
  this.evidence.events = await request(this, 'GET', `/api/uar/runs/${encodeURIComponent(runId)}/events`);
  await retain(this);
});

Then('the answer contains the correct mathematics and a citation to the retrieved note', async function () {
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
  assert.equal(this.evidence.run.status, 'done');
  assert.equal(this.evidence.events.gapReason ?? null, null);
  const citations = this.evidence.events.events.filter(event => event.eventName === 'agui.rag_citations')
    .flatMap(event => event.data.citations ?? []);
  assert.ok(citations.some(citation => citation.knowledge_base_id === this.kb.id
    && citation.document_id === this.document.id && citation.snippet.includes(this.seal)
    && citesMarker(answer.citation, citation.marker)),
  'Answer lacks a matching actual retrieved-document citation');
  this.evidence.answer = answer;
  this.evidence.result = 'VERIFIED';
  await retain(this);
});

After(async function ({ result }) {
  if (!this.evidence) return;
  this.evidence.scenario_status = result.status;
  await retain(this);
  this.attach(JSON.stringify(this.evidence), 'application/json');
});
