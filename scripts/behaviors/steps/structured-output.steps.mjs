import assert from 'node:assert/strict';
import { randomInt, randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';

setDefaultTimeout(180_000);

async function http(w, method, path, payload, expected = [200]) {
  const exchange = { method, path, request: payload };
  w.evidence.exchanges.push(exchange);
  const response = await fetch(new URL(path, w.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(170_000),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': w.workspace },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  if (method === 'POST' && path === '/api/agents' && response.status === 201) w.created = true;
  exchange.status = response.status;
  exchange.response = await response.text();
  assert.ok(expected.includes(response.status), `${method} ${path}: HTTP ${response.status}: ${exchange.response.slice(0, 800)}`);
  return exchange.response;
}

async function disposableAgent(w) {
  const agent = JSON.parse(await http(w, 'GET', '/api/agents/default-agent'));
  agent.id = w.agentId;
  agent.version = '1.0.0';
  agent.metadata = { ...agent.metadata, title: 'Disposable structured-output agent', tags: ['disposable', 'verification'] };
  agent.runtime.entry = 'default';
  agent.prompt = { system: 'You are a precise calculator. Answer in the format the request requires.', instructions: [] };
  const split = w.model.indexOf('/');
  agent.policy.provider = { default: { provider: w.model.slice(0, split), model: w.model.slice(split + 1) }, fallbacks: [] };
  // An empty allow list means Auto (every eligible tool); one unmatched placeholder selects none.
  agent.policy.tools.allow = [`structured-no-tools-${w.id.slice(0, 8)}`];
  agent.memory.kb.enabled = false;
  await http(w, 'POST', '/api/agents', agent, [201]);
}

Given('a disposable agent on a structured-output model with a schema holding a fresh discriminator', async function () {
  this.started = performance.now();
  this.id = randomUUID();
  this.model = process.env.UAR_STRUCTURED_MODEL ?? 'liter/glm-5.3';
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.base = new URL(process.env.UAR_BASE_URL);
  assert.ok(this.workspace, 'Set UAR_WORKSPACE_ID');
  assert.ok(['http:', 'https:'].includes(this.base.protocol) && !this.base.username && !this.base.password);
  this.record = resolve(`.prometheus/verification/structured-output/${this.id}.json`);
  this.agentId = `disposable-structured-output-${this.id}`;
  this.session = randomUUID();
  // Fresh discriminator: a field name and its only allowed value, neither of which appears in the prompt.
  this.field = `tag_${randomUUID().slice(0, 8)}`;
  this.token = `D-${randomUUID()}`;
  this.schema = {
    type: 'object',
    properties: { [this.field]: { type: 'string', enum: [this.token] }, result: { type: 'integer' } },
    required: [this.field, 'result'],
    additionalProperties: false,
  };
  this.evidence = { scenario: 'E05-STRUCTURED-OUTPUT', client_platform: process.platform, agent_id: this.agentId,
    model: this.model, session_id: this.session, discriminator: { field: this.field, token: this.token },
    schema: this.schema, authentication_supplied_by_test: false, exchanges: [] };
  await disposableAgent(this);
});

When('I ask for a fresh calculation through \\/v1\\/chat\\/completions with that response_format', async function () {
  const left = randomInt(21, 80), right = randomInt(11, 40), extra = randomInt(101, 500);
  this.expected = left * right + extra;
  this.prompt = `What is (${left} multiplied by ${right}) plus ${extra}?`;
  assert.ok(!this.prompt.includes(this.field) && !this.prompt.includes(this.token));
  this.evidence.expected = this.expected;
  this.evidence.prompt = this.prompt;
  const raw = await http(this, 'POST', '/v1/chat/completions', {
    agent_id: this.agentId, model: this.model, session_id: this.session, stream: false,
    memory_enabled: false, max_completion_tokens: 4096,
    response_format: { type: 'json_schema', json_schema: { name: 'calculation', strict: true, schema: this.schema } },
    messages: [{ role: 'user', content: this.prompt }],
  });
  this.completion = JSON.parse(raw);
});

// Exact validation of the supplied schema: required keys, no extra keys, enum and integer types.
function validate(schema, value) {
  assert.equal(typeof value, 'object');
  assert.ok(value !== null && !Array.isArray(value), 'The answer must be a JSON object');
  assert.deepEqual(Object.keys(value).sort(), [...schema.required].sort(), 'Exactly the schema keys are required');
  for (const [key, rule] of Object.entries(schema.properties)) {
    if (rule.enum) assert.ok(rule.enum.includes(value[key]), `${key} must be one of the schema's enum values`);
    if (rule.type === 'integer') assert.ok(Number.isInteger(value[key]), `${key} must be an integer`);
    if (rule.type === 'string') assert.equal(typeof value[key], 'string');
  }
}

Then('the final answer validates against the exact schema, carries the discriminator and the correct result', async function () {
  const content = this.completion?.choices?.[0]?.message?.content;
  assert.equal(typeof content, 'string', 'The completion must carry final message content');
  this.evidence.final_content = content;
  this.evidence.finish_reason = this.completion.choices[0].finish_reason;
  const answer = JSON.parse(content.trim());
  this.evidence.parsed = answer;
  validate(this.schema, answer);
  assert.equal(answer[this.field], this.token);
  assert.equal(answer.result, this.expected);
  const runs = JSON.parse(await http(this, 'GET', '/api/uar/runs?limit=100'));
  const run = runs.find(entry => entry.conversation_id === this.session);
  assert.ok(run, 'The completion must be backed by a recorded run in this session');
  this.evidence.run = run;
  this.evidence.result = 'VERIFIED';
});

After(async function ({ result }) {
  if (!this.evidence) return;
  let cleanupError;
  if (this.created) {
    try { await http(this, 'DELETE', `/api/agents/${this.agentId}`, undefined, [204]); }
    catch (error) { cleanupError = error; }
  }
  this.evidence.scenario_status = cleanupError ? 'FAILED' : result.status;
  if (cleanupError) this.evidence.cleanup_error = cleanupError.message;
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify({ evidence: this.record, status: this.evidence.scenario_status }), 'application/json');
  if (cleanupError) throw cleanupError;
});
