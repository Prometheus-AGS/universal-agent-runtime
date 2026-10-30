import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Before, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { STUB_LLM_MODEL, STUB_LLM_PROMPT, startDedicatedRuntime } from '../../acceptance/fixtures/dedicated-runtime.mjs';
import { startEmbeddingStub } from '../../acceptance/fixtures/embedding-stub.mjs';

setDefaultTimeout(240_000);
const TOOL_PROMPT = 'Parity tool probe: look up code alpha.';
const LOOKUP_SCHEMA = { type: 'object', properties: { key: { type: 'string' } }, required: ['key'] };
const LOOKUP_DESCRIPTION = 'Look up a code by key.';

// The fixture's identity proxy supplies the owner; the test sends no credentials.
async function request(w, method, path, payload, expected = [200], headers = {}) {
  const res = await fetch(new URL(path, w.runtime.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(120_000),
    headers: { 'Content-Type': 'application/json', ...headers },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  const text = await res.text();
  w.evidence.exchanges.push({ method, path, request: payload, status: res.status, response: text.slice(0, 4000) });
  assert.ok(expected.includes(res.status), `${method} ${path}: ${res.status}: ${text.slice(0, 800)}`);
  return text ? JSON.parse(text) : null;
}

Before(function ({ pickle }) {
  this.scenarioName = pickle.name;
});

async function startRuntime(world, options = {}) {
  world.started = performance.now();
  world.holder = {};
  world.owned = [];
  world.record = resolve(`.prometheus/verification/runtime-configuration/${randomUUID()}.json`);
  world.evidence = { scenario: world.scenarioName, platform: process.platform,
    authentication_supplied_by_test: false, exchanges: [], fixture: {} };
  world.runtime = await startDedicatedRuntime(world.evidence.fixture, world.holder,
    { backend: 'postgres', label: 'runtime-configuration', ...options });
}

/** JSON log entries of the most recent UAR launch only. */
async function latestLaunchLog(world) {
  return (await readFile(world.runtime.uarLogs.at(-1), 'utf8')).split('\n');
}

async function latestLaunchEntries(world) {
  return (await latestLaunchLog(world)).flatMap(line => {
    try { return [JSON.parse(line)]; } catch { return []; }
  });
}

// --- #419: LLM model precedence -------------------------------------------

Given('a dedicated UAR whose config file names model {string} with LLM_MODEL {string} and UAR_LLM__MODEL {string}',
  { timeout: 420_000 }, async function (configModel, legacyModel, modernModel) {
    await startRuntime(this, { config: { llm: { model: configModel } },
      env: { LLM_MODEL: legacyModel, UAR_LLM__MODEL: modernModel } });
    this.evidence.models = { config: configModel, LLM_MODEL: legacyModel, UAR_LLM__MODEL: modernModel, loaded: [] };
  });

Then('UAR loads model {string}', async function (model) {
  const entry = (await latestLaunchEntries(this)).find(e => e.fields?.message === 'LLM configuration loaded');
  assert.ok(entry, 'UAR logged the loaded LLM configuration');
  const loaded = entry.fields.model;
  this.evidence.models.loaded.push(loaded);
  assert.ok(loaded === model || loaded === model.split('/')[1], `Loaded ${loaded}, expected ${model}`);
  if (model === this.evidence.models.config) this.evidence.result = 'VERIFIED';
});

When('UAR restarts without UAR_LLM__MODEL', { timeout: 300_000 }, async function () {
  await this.runtime.restartUar(undefined, { UAR_LLM__MODEL: null });
});

When('UAR restarts without LLM_MODEL', { timeout: 300_000 }, async function () {
  await this.runtime.restartUar(undefined, { LLM_MODEL: null });
});

// --- #420: memory embedding API ---------------------------------------------

Given('a dedicated UAR with memory using the Mistral embedding API against a stub embedder', { timeout: 420_000 }, async function () {
  this.embeddingStub = await startEmbeddingStub(1024);
  const { port } = this.embeddingStub.address();
  // The stub ignores the key; it is a fixed placeholder, not a credential.
  await startRuntime(this, { config: { memory: { enabled: true, embedding_api: 'mistral', embedding_model: 'mistral-embed',
    embedding_base_url: `http://127.0.0.1:${port}/v1`, embedding_api_key: 'test-stub-key', embedding_dim: 1024 } } });
});

Then('UAR reports the Mistral embedding API at startup', async function () {
  const entries = await latestLaunchEntries(this);
  const ready = entries.find(e => e.fields?.message === 'Memory embedding service ready');
  const initialized = entries.map(e => e.fields?.message ?? '').find(m => m.startsWith('MemoryService initialized'));
  this.evidence.memory_startup = { ready: ready?.fields, initialized };
  assert.equal(ready?.fields?.embedding_api, 'mistral');
  assert.equal(Number(ready?.fields?.dimensions), 1024);
  assert.match(initialized ?? '', /embedding_api=mistral, embedding_model=mistral-embed/);
});

When('I save a memory record', async function () {
  const fact = `RC2-MEMORY-${randomUUID().slice(0, 8)}`;
  const saved = await request(this, 'POST', '/api/memory', { content: `The project code word is ${fact}.`,
    categories: ['personal_fact'] });
  assert.equal(saved?.status, 'success');
});

Then('the stub embedder received a {string} request', async function (model) {
  this.evidence.embedding_requests = this.embeddingStub.requests;
  assert.ok(this.embeddingStub.requests.some(r => r.model === model && r.path === '/v1/embeddings'),
    `The stub received a ${model} request`);
});

When('UAR restarts with the embedding API {string}', { timeout: 300_000 }, async function (api) {
  this.rejectedApi = api;
  await assert.rejects(this.runtime.restartUar({ memory: { embedding_api: api } }), /exited before it was ready/);
});

Then('UAR refuses to start and names the unsupported embedding API', async function () {
  const log = (await latestLaunchLog(this)).join('\n');
  const named = log.includes(`embedding_api=\\"${this.rejectedApi}\\" is not supported`)
    || log.includes(`embedding_api="${this.rejectedApi}" is not supported`);
  this.evidence.rejection_logged = named;
  assert.ok(named, 'The startup failure names the unsupported embedding API');
  this.evidence.result = 'VERIFIED';
});

// --- Stub model runtime (#385, #396) -----------------------------------------

Given('a dedicated UAR with the stub model', { timeout: 420_000 }, async function () {
  await startRuntime(this, { stubLlm: true });
  // A fresh runtime admits a model only after it is verified, as an operator does in settings.
  const [provider, model] = STUB_LLM_MODEL.split('/');
  const verification = await request(this, 'POST', `/api/uar/providers/${provider}/test`, { model, output_token_cap: 128 });
  assert.equal(verification.ok, true, `Model verification: ${JSON.stringify(verification).slice(0, 300)}`);
});

Given('an agent with a per-turn token limit of {int}', async function (limit) {
  const agent = await request(this, 'GET', '/api/agents/default-agent');
  this.agentId = `rc2-budget-${randomUUID()}`;
  agent.id = this.agentId;
  agent.metadata = { ...agent.metadata, title: 'Budgeted verification agent', tags: ['verification'] };
  agent.extensions = { ...agent.extensions, budgets: { max_tokens_per_turn: limit } };
  await request(this, 'POST', '/api/agents', agent, [201]);
  this.owned.push(`/api/agents/${this.agentId}`);
  this.evidence.budget = { max_tokens_per_turn: limit };
});

When('I ask that agent with max_tokens {int}', async function (maxTokens) {
  this.completion = await request(this, 'POST', '/v1/chat/completions', { model: STUB_LLM_MODEL, agent_id: this.agentId,
    max_tokens: maxTokens, stream: false, messages: [{ role: 'user', content: STUB_LLM_PROMPT }] });
  this.evidence.budget.max_tokens = maxTokens;
});

Then('the agent answers', async function () {
  this.evidence.answer = this.completion.choices?.[0]?.message?.content;
  assert.match(this.evidence.answer ?? '', /SESSION-OK/);
  this.evidence.result = 'VERIFIED';
});

// --- #396: Messages and chat completions parity ------------------------------

const chat = (world, content, extra = {}) => request(world, 'POST', '/v1/chat/completions', { model: STUB_LLM_MODEL,
  session_id: randomUUID(), max_tokens: 256, stream: false, messages: [{ role: 'user', content }], ...extra });
const messages = (world, content, extra = {}) => request(world, 'POST', '/v1/messages', { model: STUB_LLM_MODEL,
  session_id: randomUUID(), max_tokens: 256, messages: [{ role: 'user', content }], ...extra });

When('I ask the default agent through chat completions', async function () {
  this.chatAnswer = await chat(this, STUB_LLM_PROMPT);
});

When('I ask the default agent through Messages', async function () {
  this.messagesAnswer = await messages(this, STUB_LLM_PROMPT);
});

Then('both endpoints return the same answer', async function () {
  const chatText = this.chatAnswer.choices?.[0]?.message?.content;
  const messagesText = (this.messagesAnswer.content ?? []).filter(b => b.type === 'text').map(b => b.text).join('');
  this.evidence.parity = { chat: chatText, messages: messagesText };
  assert.match(chatText ?? '', /SESSION-OK/);
  assert.equal(messagesText, chatText);
});

// The keyless stub reports no usage. Chat completions then omits `usage`, and
// Messages, whose usage fields are required, reports zero. Matching non-zero
// counts need a model that reports usage; the stub cannot show that.
Then('both endpoints agree that the model reported no token usage', async function () {
  const chatUsage = this.chatAnswer.usage;
  const messagesUsage = this.messagesAnswer.usage;
  this.evidence.parity.usage = { chat: chatUsage ?? null, messages: messagesUsage };
  assert.equal(chatUsage?.completion_tokens ?? 0, 0);
  assert.equal(messagesUsage?.output_tokens, 0);
  assert.equal(messagesUsage?.input_tokens, 0);
  this.evidence.result = 'VERIFIED';
});

When('I ask the default agent to use my lookup_code tool through chat completions', async function () {
  this.chatAnswer = await chat(this, TOOL_PROMPT, { tools: [{ type: 'function',
    function: { name: 'lookup_code', description: LOOKUP_DESCRIPTION, parameters: LOOKUP_SCHEMA } }] });
});

When('I ask the default agent to use my lookup_code tool through Messages', async function () {
  this.messagesAnswer = await messages(this, TOOL_PROMPT, { tools: [{ name: 'lookup_code',
    description: LOOKUP_DESCRIPTION, input_schema: LOOKUP_SCHEMA }] });
});

Then('chat completions returns a lookup_code tool call', async function () {
  const call = this.chatAnswer.choices?.[0]?.message?.tool_calls?.[0];
  this.evidence.parity = { chat_tool_call: call };
  assert.equal(call?.function?.name, 'lookup_code');
  this.chatToolInput = JSON.parse(call.function.arguments);
});

Then('Messages stops with tool_use and a lookup_code block with the same input', async function () {
  const block = (this.messagesAnswer.content ?? []).find(b => b.type === 'tool_use');
  this.evidence.parity.messages_tool_use = { stop_reason: this.messagesAnswer.stop_reason, block };
  assert.equal(this.messagesAnswer.stop_reason, 'tool_use');
  assert.equal(block?.name, 'lookup_code');
  assert.deepEqual(block.input, this.chatToolInput);
  this.evidence.result = 'VERIFIED';
});

After(async function ({ result }) {
  if (!this.evidence) return;
  const errors = [];
  if (this.runtime?.uarAlive()) {
    for (const path of [...this.owned].reverse()) {
      try { await request(this, 'DELETE', path, undefined, [200, 204, 404]); }
      catch (error) { errors.push(`${path}: ${error}`); }
    }
  }
  if (this.holder?.handle) errors.push(...await this.holder.handle.stop());
  if (this.embeddingStub) await new Promise(done => this.embeddingStub.close(done));
  this.evidence.cleanup_errors = errors;
  this.evidence.status = errors.length ? 'FAILED' : result.status;
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify({ evidence: this.record, status: this.evidence.status }), 'application/json');
  assert.deepEqual(errors, [], 'Disposable cleanup must succeed');
});
