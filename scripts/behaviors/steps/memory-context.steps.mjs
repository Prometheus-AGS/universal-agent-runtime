import assert from 'node:assert/strict';
import { createHash, randomUUID } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { safeUrl, poisonFixture, isolationState } from '../support/capability-fixture.mjs';

setDefaultTimeout(180_000);
const sha = value => createHash('sha256').update(value).digest('hex');
const xml = value => value.replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;');
const memoryId = row => typeof row.id === 'string' ? row.id
  : `${row.id.table}:${row.id.key?.String ?? row.id.key}`;
const textContent = content => typeof content === 'string' ? content
  : Array.isArray(content) ? content.filter(part => part.type === 'text').map(part => part.text).join('\n') : '';
const submitted = w => w.context.filter(event => event.status === 'submitted');

async function request(w, owner, method, path, payload, expected = [200]) {
  let response;
  let raw;
  try {
    response = await fetch(new URL(path, w.bases[owner]), {
      method, redirect: 'error', signal: AbortSignal.timeout(120_000),
      headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': w.profile.workspace },
      body: payload === undefined ? undefined : JSON.stringify(payload),
    });
    raw = await response.text();
  } catch {
    poisonFixture('memory-context-owned-http-outcome-uncertain');
    throw new Error('Owned memory fixture HTTP outcome is uncertain; reconcile before another case');
  }
  w.evidence.exchanges.push({ owner, method, path, status: response.status, response_sha256: sha(raw) });
  assert.ok(expected.includes(response.status), `${owner} ${method} ${path}: HTTP ${response.status}`);
  return { status: response.status, body: raw ? JSON.parse(raw) : null };
}
async function journal(w) {
  const raw = await readFile(w.fixture.peer.journal_path, 'utf8');
  assert.ok(raw.endsWith('\n'), 'Peer journal row must be fully committed before reading');
  const rows = raw.trim().split('\n').map(line => JSON.parse(line));
  assert.ok(rows.every(row => row.namespace === w.fixture.namespace), 'Journal belongs to this fixture only');
  assert.equal(rows[0].protocol, 'uar.memory-context-peer/v1');
  assert.ok(rows.every((row, index) => row.ordinal === index + 1), 'No missing or rewritten journal row');
  return rows;
}
async function chatRows(w) {
  return (await journal(w)).filter(row => row.kind === 'chat' && row.case_id === w.id);
}
function events(raw) {
  return raw.split(/\r?\n\r?\n/).flatMap(block => {
    const lines = block.split(/\r?\n/);
    const name = lines.find(line => line.startsWith('event:'))?.slice(6).trim();
    const data = lines.filter(line => line.startsWith('data:')).map(line => line.slice(5).replace(/^ /, '')).join('\n');
    if (!data || data === '[DONE]') return [];
    return [{ name, data: JSON.parse(data) }];
  });
}
async function assertOneRequest(w) {
  const rows = await chatRows(w);
  assert.equal(rows.length, 1, 'One case must produce exactly one model request; no retry/fallback');
  const row = rows[0];
  assert.equal(row.request.model, w.profile.wire_model);
  assert.ok(!row.request.tools?.length, 'No recall/list/native tools can supply the fact');
  assert.ok(row.request.messages.every(message => message.role !== 'assistant' && message.role !== 'tool'), 'No historical answer or tool result can supply the fact');
  const users = row.request.messages.filter(message => message.role === 'user');
  assert.equal(users.length, 1, 'Fresh turn has exactly one user message');
  assert.ok(!textContent(users[0].content).includes(w.fact), 'The query never carries the answer');
  w.evidence.provider_observation = { journal_ordinal: row.ordinal, request_sha256: row.request_sha256,
    model: row.request.model, messages: row.request.messages };
  return row;
}

Given('the externally provisioned memory-context fixture profile {string}', async function (name) {
  assert.equal(isolationState().poisoned, false, 'Previous owned uncertainty must be externally reconciled before another case');
  this.id = randomUUID();
  this.recordPath = resolve(`.prometheus/verification/memory-context/${this.id}.json`);
  this.fixture = JSON.parse(await readFile(process.env.UAR_MEMORY_CONTEXT_FIXTURE, 'utf8'));
  assert.equal(this.fixture.schema, 'uar.memory-context-fixture/v1');
  assert.match(this.fixture.namespace, /^memory-context-[A-Za-z0-9_-]{1,64}$/);
  assert.match(this.fixture.identity.source_commit, /^[0-9a-f]{40}$/);
  assert.match(this.fixture.identity.artifact_sha256, /^[0-9a-f]{64}$/);
  this.profile = this.fixture.profiles[name];
  assert.ok(this.profile, `Externally provisioned profile ${name} is required`);
  assert.match(this.profile.model, /^[A-Za-z0-9_-]+\/[A-Za-z0-9_-]+$/);
  assert.equal(this.profile.wire_model, this.profile.model.slice(this.profile.model.indexOf('/') + 1));
  assert.ok(this.profile.workspace);
  assert.match(this.profile.config_sha256, /^[0-9a-f]{64}$/);
  assert.ok(this.profile.independent_memory_store_id.startsWith(this.fixture.namespace));
  assert.equal(this.profile.auto_capture, false);
  assert.equal(this.profile.embedding_dimension, 2);
  assert.equal(this.profile.embedding_model, 'memory-context-fixture-2');
  assert.equal(this.profile.fallbacks_enabled, false);
  assert.equal(this.profile.retries_enabled, false);
  assert.equal(this.profile.injection_min_similarity, name === 'absent' ? null : 0.5);
  this.bases = { primary: safeUrl(this.profile.base_url), foreign: safeUrl(this.profile.foreign_base_url) };
  assert.notEqual(this.bases.primary.href, this.bases.foreign.href, 'External identity endpoints must differ');
  assert.equal(this.profile.identity_binding.same_runtime, true);
  assert.equal(this.profile.identity_binding.distinct_verified_principals, true);
  const status = await fetch(new URL('/__memory_context/status', safeUrl(this.fixture.peer.base_url)),
    { redirect: 'error', signal: AbortSignal.timeout(10_000) }).then(response => response.json());
  assert.equal(status.namespace, this.fixture.namespace);
  assert.equal(status.dimension, 2);
  assert.equal(sha(await readFile(new URL('../fixtures/memory-context/peer.mjs', import.meta.url))), this.fixture.peer.source_sha256);
  this.evidence = { case_id: this.id, scope: 'controlled final-input wiring; no real-model recall claim', profile: name,
    identity: this.fixture.identity, config_sha256: this.profile.config_sha256,
    independent_memory_store_id: this.profile.independent_memory_store_id,
    authentication_supplied_by_test: false, services_started_by_test: false,
    selected_cutoff: this.profile.injection_min_similarity, exchanges: [], sessions: [] };
  for (const owner of ['primary', 'foreign']) {
    const listing = (await request(this, owner, 'GET', '/api/memory/records')).body;
    assert.deepEqual(listing.memories, [], 'Dedicated fixture owner memory partition must start empty');
  }
  for (const [key, value] of Object.entries({ 'memory.enabled': true, 'memory.inject_context': true,
    'memory.auto_capture': false, 'memory.injection_min_similarity': this.profile.injection_min_similarity })) {
    assert.equal((await request(this, 'primary', 'GET', `/api/uar/settings/${key}`)).body.effective, value,
      `Effective ${key} must match the externally frozen startup profile`);
  }
  const artifact = (await request(this, 'primary', 'GET', '/api/agents/default-agent')).body;
  this.agentId = `disposable-memory-context-${this.id}`;
  artifact.id = this.agentId;
  artifact.version = '1.0.0';
  artifact.metadata = { ...artifact.metadata, title: 'Synthetic independent memory boundary', tags: ['memory-context-fixture'] };
  artifact.prompt = { system: 'Answer from supplied untrusted memory data if present. Never call a tool. This is a synthetic input-boundary case.', instructions: [] };
  artifact.runtime.entry = 'default';
  const split = this.profile.model.indexOf('/');
  assert.ok(split > 0);
  artifact.policy.provider = { default: { provider: this.profile.model.slice(0, split), model: this.profile.model.slice(split + 1) }, fallbacks: [] };
  artifact.policy.skills.prefer = [];
  artifact.policy.tools.allow = [];
  artifact.memory.kb.enabled = false;
  artifact.memory.conversation.enabled = true;
  artifact.extensions = { 'uar.run_policy': { version: 1, context_strategy: { type: 'none' }, memory_enabled: true,
    skills: { mode: 'none' }, tools: { mode: 'none' }, mcp_servers: { mode: 'none' },
    knowledge_bases: { mode: 'none' }, presentations: { mode: 'none' } } };
  this.agentMayExist = true;
  await request(this, 'primary', 'POST', '/api/agents', artifact, [201]);
  await journal(this);
});

When('I save a fresh {string} memory through the independent memory API', async function (kind) {
  assert.ok(['relevant', 'below-cutoff', 'foreign'].includes(kind));
  this.memoryOwner = kind === 'foreign' ? 'foreign' : 'primary';
  this.fact = `MEMORY_FACT_${randomUUID()}`;
  this.factContent = `The synthetic storage locker verification seal is ${this.fact}. Case ${this.id}. ${kind === 'below-cutoff' ? 'MC_VECTOR_NEGATIVE' : 'MC_VECTOR_POSITIVE'}`;
  const saved = (await request(this, this.memoryOwner, 'POST', '/api/memory', {
    content: this.factContent, categories: ['memory-context-fixture'], agent_id: this.agentId,
  })).body;
  assert.equal(saved.status, 'success');
  assert.ok(saved.id);
  this.memoryId = saved.id;
  const record = (await request(this, this.memoryOwner, 'GET', `/api/memory/records/${encodeURIComponent(this.memoryId)}`)).body;
  assert.equal(record.content, this.factContent, 'Read the actual independent service row, not a client receipt');
  this.scope = record.scope.toLowerCase();
  this.memoryType = record.memory_type.toLowerCase();
  this.evidence.saved_memory = { id: this.memoryId, owner: this.memoryOwner, content_sha256: sha(this.factContent),
    synthetic_fact: this.fact, scope: this.scope, memory_type: this.memoryType };
  this.beforeTurnOrdinal = (await journal(this)).at(-1).ordinal;
});
When('I delete that exact independent memory', async function () {
  await request(this, this.memoryOwner, 'DELETE', `/api/memory/records/${encodeURIComponent(this.memoryId)}`, undefined, [200]);
  await request(this, this.memoryOwner, 'GET', `/api/memory/records/${encodeURIComponent(this.memoryId)}`, undefined, [404, 410]);
});
Then('the primary identity cannot read the foreign record', async function () {
  assert.equal(this.memoryOwner, 'foreign');
  await request(this, 'primary', 'GET', `/api/memory/records/${encodeURIComponent(this.memoryId)}`, undefined, [404]);
  await request(this, 'foreign', 'GET', `/api/agents/${this.agentId}`);
});
When('I submit a fresh memory-context turn in mode {string}', async function (mode) {
  this.mode = mode;
  this.query = `What is the synthetic storage locker verification seal? UAR_MEMORY_CASE:${this.fixture.namespace}:${this.id}:${mode}`;
  assert.ok(!this.query.includes(this.fact));
  this.session = randomUUID();
  this.evidence.sessions.push(this.session);
  const payload = { agent_id: this.agentId, model: this.profile.model, session_id: this.session,
    client_turn_id: randomUUID(), user_message_id: randomUUID(), memory_enabled: mode !== 'opt-out',
    stream: true, stream_mode: 'agui', max_completion_tokens: 32, messages: [{ role: 'user', content: this.query }] };
  let response;
  let raw;
  try {
    response = await fetch(new URL('/v1/chat/completions', this.bases.primary), {
      method: 'POST', redirect: 'error', signal: AbortSignal.timeout(150_000),
      headers: { 'Content-Type': 'application/json', Accept: 'text/event-stream', 'x-uar-workspace-id': this.profile.workspace },
      body: JSON.stringify(payload),
    });
    raw = await response.text();
  } catch {
    poisonFixture('memory-context-native-admission-unconfirmed');
    throw new Error('Native turn response is uncertain; retained admission must be externally reconciled');
  }
  this.evidence.stream_http_status = response.status;
  assert.equal(response.status, 200, 'Native streaming admission must produce inspectable terminal evidence');
  this.events = events(raw);
  this.context = this.events.filter(event => event.name === 'agui.memory.context').map(event => event.data);
  this.errors = this.events.filter(event => event.name === 'agui.error').map(event => event.data);
  this.runId = response.headers.get('x-uar-run-id') ?? this.events.find(event => event.data?.request_id)?.data.request_id;
  assert.ok(this.runId, 'Original native run identity is required');
  this.run = (await request(this, 'primary', 'GET', `/api/uar/runs/${encodeURIComponent(this.runId)}`)).body;
  this.evidence.turn = { session_id: this.session, run_id: this.runId, state: this.run.status,
    events: this.events, context: this.context, errors: this.errors };
  assert.equal(this.run.conversation_id, this.session);
  assert.equal(this.events.filter(event => /tool/i.test(event.name ?? '')).length, 0, 'No memory or other tool call may explain the fact');
  assert.ok(this.context.every(event => !JSON.stringify(event).includes(this.fact)), 'Host provenance is content-free');
});

Then('the exact saved fact appears only in the captured retrieved System content', async function () {
  const row = await assertOneRequest(this);
  const systems = row.request.messages.filter(message => message.role === 'system').map(message => textContent(message.content));
  const expected = `<uar-retrieved-content>\n${xml(`[MEMORY CONTEXT]\n[${this.scope}/${this.memoryType}] ${this.factContent}`)}\n</uar-retrieved-content>`;
  assert.ok(systems.some(system => system.includes(expected)), 'Independent peer must capture the full original marked memory fragment');
  assert.ok(row.request.messages.filter(message => message.role !== 'system').every(message => !textContent(message.content).includes(this.fact)));
  assert.ok(!this.evidence.provider_observation.messages.some(message => message.role === 'tool'));
  this.evidence.expected_marked_memory_sha256 = sha(expected);
});
Then('the host reports the original memory IDs submitted with provider-response evidence', async function () {
  const claims = submitted(this);
  assert.equal(claims.length, 1);
  const claim = claims[0];
  assert.equal(claim.reason_code, 'memory_content_submitted');
  assert.deepEqual(claim.provenance.memory_ids, [this.memoryId]);
  assert.equal(claim.provenance.retrieval_method, 'owner_scoped_three_lane_hybrid_cosine_eligible');
  assert.equal(claim.provenance.evidence_basis, 'final_driver_request_and_provider_response');
  assert.match(claim.provenance.request_content_sha256, /^[0-9a-f]{64}$/);
  assert.equal(claim.provenance.model, this.profile.model);
  const rows = (await journal(this)).filter(row => row.kind === 'embedding' && row.ordinal > this.beforeTurnOrdinal);
  assert.ok(rows.some(row => row.inputs.some(input => input.sha256 === sha(this.factContent))), 'Actual eligibility engine re-embeds the saved candidate');
  assert.ok(rows.some(row => row.inputs.some(input => input.sha256 === sha(this.query))), 'Actual eligibility engine embeds the fresh query');
  // Host hash is logical final driver messages/system, peer hash is literal wire.
  // No claim that different dialect serialization hashes are byte-identical.
});
Then('memory-context reports {string} with reason {string}', function (status, reason) {
  assert.ok(this.context.some(event => event.status === status && event.reason_code === reason),
    `Exact host exclusion ${status}/${reason} is required`);
});
Then('no saved fact or submitted claim reaches the provider boundary', async function () {
  assert.deepEqual(submitted(this), []);
  const rows = await chatRows(this);
  if (this.mode === 'budget-refusal') assert.equal(rows.length, 0);
  else await assertOneRequest(this);
  assert.ok(rows.every(row => !JSON.stringify(row.request.messages).includes(this.fact)), 'Excluded fact must be absent from the actual peer input');
});
Then('the independent memory engine compared opposite candidate and query vectors', async function () {
  const rows = (await journal(this)).filter(row => row.kind === 'embedding' && row.ordinal > this.beforeTurnOrdinal);
  const inputs = rows.flatMap(row => row.inputs);
  assert.ok(inputs.some(input => input.sha256 === sha(this.factContent) && input.vector[0] === -1));
  assert.ok(inputs.some(input => input.sha256 === sha(this.query) && input.vector[0] === 1));
});
Then('the native run completes without a tool or replayed history', function () {
  assert.equal(this.run.status, 'done');
  assert.deepEqual(this.errors, []);
});
Then('the final context budget refuses the turn before the model peer receives it', async function () {
  assert.equal(this.run.status, 'error');
  assert.ok(this.errors.some(error => error.code === 'context_overflow'), 'Must fail at the real final context bound, not unrelated admission');
  assert.deepEqual(await chatRows(this), []);
});
Then('no submitted claim is published and the native run fails', function () {
  assert.deepEqual(submitted(this), []);
  assert.equal(this.run.status, 'error');
  assert.ok(this.errors.length > 0);
});
Then('partial provider text remains an error outcome rather than completed recall', function () {
  const deltas = this.events.filter(event => event.name === 'agui.message.delta');
  for (const { data } of deltas) {
    assert.equal(data.request_id, this.runId);
    assert.equal(typeof data.delta.text, 'string');
  }
  assert.equal(deltas.map(({ data }) => data.delta.text).join(''), 'synthetic partial response');
  assert.equal(this.run.status, 'error');
  assert.ok(this.errors.length > 0);
  assert.ok(!this.events.some(event => event.name === 'agui.done'));
});

After(async function ({ result }) {
  if (!this.evidence) return;
  const cleanup = [];
  for (const owner of ['primary', 'foreign']) {
    try {
      const listing = (await request(this, owner, 'GET', '/api/memory/records')).body;
      for (const row of listing.memories.filter(row => row.content.includes(this.id))) {
        await request(this, owner, 'DELETE', `/api/memory/records/${encodeURIComponent(memoryId(row))}`, undefined, [200]);
      }
    } catch { cleanup.push(`${owner} owned memory cleanup incomplete`); }
  }
  for (const session of this.evidence.sessions) {
    try { await request(this, 'primary', 'DELETE', `/api/uar/sessions/${encodeURIComponent(session)}`, undefined, [204, 404]); }
    catch { cleanup.push('Owned session cleanup incomplete'); }
  }
  if (this.agentMayExist) {
    try { await request(this, 'primary', 'DELETE', `/api/agents/${this.agentId}`, undefined, [204, 404]); }
    catch { cleanup.push('Owned agent cleanup incomplete'); }
  }
  if (cleanup.length) poisonFixture('memory-context-owned-cleanup-incomplete');
  this.evidence.cleanup_errors = cleanup;
  this.evidence.cucumber_status = result.status;
  await mkdir(dirname(this.recordPath), { recursive: true });
  await writeFile(this.recordPath, `${JSON.stringify(this.evidence, null, 2)}\n`, { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify({ evidence: this.recordPath, cucumber_status: result.status }), 'application/json');
  assert.deepEqual(cleanup, [], 'Do not carry unknown cleanup into another case');
});
