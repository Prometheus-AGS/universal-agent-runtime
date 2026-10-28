import assert from 'node:assert/strict';
import { randomInt, randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { parseAnswerJson } from '../support/answer-oracle.mjs';

setDefaultTimeout(240_000);

async function http(w, method, path, payload, expected = 200) {
  const exchange = { method, path, request: payload };
  w.evidence.exchanges.push(exchange);
  const response = await fetch(new URL(path, w.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(170_000),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': w.workspace,
      ...(payload?.stream && { Accept: 'text/event-stream' }) },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  if (method === 'POST' && path === '/api/agents' && response.status === 201) w.created = true;
  exchange.status = response.status;
  exchange.response = await response.text();
  assert.equal(response.status, expected, `${method} ${path}: HTTP ${response.status}: ${exchange.response.slice(0, 800)}`);
  return { response, text: exchange.response };
}

// Parse SSE into {event, data} records; data stays raw for [DONE].
function frames(raw) {
  return raw.split(/\r?\n\r?\n/).flatMap(block => {
    const lines = block.split(/\r?\n/);
    const event = lines.find(line => line.startsWith('event:'))?.slice(6).trim() ?? null;
    const data = lines.filter(line => line.startsWith('data:')).map(line => line.slice(5).trimStart()).join('\n');
    if (!data) return [];
    if (data === '[DONE]') return [{ event, done: true }];
    return [{ event, data: JSON.parse(data) }];
  });
}

function aguiText(records) {
  const deltas = records.filter(r => r.event === 'agui.message.delta' || r.data?.kind === 'message.delta' || r.data?.kind === 'delta');
  return { count: deltas.length, text: deltas.map(r => r.data?.delta?.text ?? r.data?.text ?? '').join('') };
}

function openaiText(records) {
  const chunks = records.filter(r => r.data?.object === 'chat.completion.chunk');
  return { chunks, count: chunks.filter(c => typeof c.data.choices?.[0]?.delta?.content === 'string' && c.data.choices[0].delta.content.length).length,
    text: chunks.map(c => c.data.choices?.[0]?.delta?.content ?? '').join('') };
}

Given('a disposable saved agent with a private instruction marker for chat streaming', async function () {
  this.started = performance.now();
  this.id = randomUUID();
  this.agentId = `disposable-stream-modes-${this.id}`;
  this.session = randomUUID();
  this.model = process.env.UAR_MODEL;
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.base = new URL(process.env.UAR_BASE_URL);
  assert.ok(this.model && this.workspace);
  assert.ok(['http:', 'https:'].includes(this.base.protocol) && !this.base.username && !this.base.password);
  this.record = resolve(`.prometheus/verification/chat-stream-modes/${this.id}.json`);
  this.marker = `STREAM-${randomUUID()}`;
  this.evidence = { scenario: 'I01-I02-CHAT-STREAM-MODES', client_platform: process.platform, agent_id: this.agentId,
    session_id: this.session, model: this.model, authentication_supplied_by_test: false,
    follow_up_contract: 'server session continuity via session_id', exchanges: [] };
  const agent = JSON.parse((await http(this, 'GET', '/api/agents/default-agent')).text);
  agent.id = this.agentId;
  agent.version = '1.0.0';
  agent.metadata = { ...agent.metadata, title: 'Disposable stream-mode agent', tags: ['disposable', 'verification'] };
  agent.runtime.entry = 'default';
  agent.prompt = { system: `Return only a JSON object with keys marker and answer. Always set marker to "${this.marker}". Put the requested numeric result in answer.`, instructions: [] };
  const split = this.model.indexOf('/');
  agent.policy.provider = { default: { provider: this.model.slice(0, split), model: this.model.slice(split + 1) }, fallbacks: [] };
  // An empty allow list means Auto (every eligible tool); one unmatched placeholder selects none.
  agent.policy.tools.allow = [`stream-no-tools-${this.id.slice(0, 8)}`];
  agent.memory.kb.enabled = false;
  await http(this, 'POST', '/api/agents', agent, 201);
  const left = randomInt(21, 80), right = randomInt(11, 40), extra = randomInt(101, 500);
  this.more = randomInt(11, 90);
  this.firstText = `What is (${left} multiplied by ${right}) plus ${extra}?`;
  this.expected = [left * right + extra, left * right + extra + this.more];
  this.evidence.expected = this.expected;
});

async function streamTurn(w, path, mode, text) {
  const { response, text: raw } = await http(w, 'POST', path, { agent_id: w.agentId, model: w.model, session_id: w.session,
    stream: true, stream_mode: mode, memory_enabled: false, max_completion_tokens: 4096,
    messages: [{ role: 'user', content: text }] });
  assert.match(response.headers.get('content-type') ?? '', /^text\/event-stream\b/);
  return frames(raw);
}

async function twoTurns(w, path, mode) {
  w.mode = mode;
  w.turns = [await streamTurn(w, path, mode, w.firstText),
    await streamTurn(w, path, mode, `Add ${w.more} to your previous answer.`)];
}

When('I stream a fresh calculation and then a dependent follow-up with stream_mode agui on \\/api\\/chat\\/completion', async function () {
  await twoTurns(this, '/api/chat/completion', 'agui');
});

When('I stream a fresh calculation and then a dependent follow-up with stream_mode dual on \\/v1\\/chat\\/completions', async function () {
  await twoTurns(this, '/v1/chat/completions', 'dual');
});

async function sessionRuns(w) {
  const runs = JSON.parse((await http(w, 'GET', '/api/uar/runs?limit=100')).text);
  return runs.filter(run => run.conversation_id === w.session);
}

Then('each legacy AG-UI stream delivers incremental text, completes, and answers correctly in one session', async function () {
  this.evidence.turns = [];
  for (const [index, records] of this.turns.entries()) {
    assert.ok(!records.some(r => r.data?.object === 'chat.completion.chunk'), 'Legacy AG-UI mode must not emit OpenAI chunks');
    assert.ok(records.some(r => r.event === 'agui.done' || r.data?.kind === 'done'), 'The stream must end with agui.done');
    assert.ok(!records.some(r => r.event === 'agui.error' || r.data?.kind === 'error'), 'The stream must not error');
    const agui = aguiText(records);
    assert.ok(agui.count >= 1, 'Genuine content deltas are required');
    const answer = parseAnswerJson(agui.text);
    this.evidence.turns.push({ events: [...new Set(records.map(r => r.event))], deltas: agui.count, answer });
    assert.equal(answer.marker, this.marker);
    assert.equal(Number(answer.answer), this.expected[index]);
  }
  const runs = await sessionRuns(this);
  this.evidence.runs = runs.map(r => ({ run_id: r.run_id, agent_id: r.agent_id, status: r.status }));
  assert.equal(runs.length, 2);
  for (const run of runs) assert.equal(run.agent_id, this.agentId);
  this.evidence.result = 'VERIFIED';
});

Then("each dual stream's OpenAI and AG-UI text agree, belong to one run, and answer correctly in one session", async function () {
  this.evidence.turns = [];
  for (const [index, records] of this.turns.entries()) {
    const openai = openaiText(records);
    const agui = aguiText(records);
    assert.ok(openai.count >= 1 && agui.count >= 1, 'Both vocabularies must carry content deltas');
    assert.equal(openai.text, agui.text, 'Both vocabularies must carry the same single answer');
    const ids = new Set(openai.chunks.map(c => c.data.id));
    assert.equal(ids.size, 1, 'All OpenAI chunks belong to one completion');
    const runId = [...ids][0].replace(/^chatcmpl-/, '');
    const aguiRunIds = new Set(records.filter(r => r.data?.request_id).map(r => r.data.request_id));
    assert.ok(aguiRunIds.size <= 1 && (aguiRunIds.size === 0 || aguiRunIds.has(runId)), 'AG-UI events correlate to the same run');
    assert.ok(records.some(r => r.done), 'The OpenAI vocabulary ends with [DONE]');
    const answer = parseAnswerJson(openai.text);
    this.evidence.turns.push({ run_id: runId, openai_deltas: openai.count, agui_deltas: agui.count, answer });
    assert.equal(answer.marker, this.marker);
    assert.equal(Number(answer.answer), this.expected[index]);
  }
  const runs = await sessionRuns(this);
  this.evidence.runs = runs.map(r => ({ run_id: r.run_id, agent_id: r.agent_id, status: r.status }));
  assert.equal(runs.length, 2, 'One run per request: no second dispatch');
  assert.deepEqual(new Set(runs.map(r => r.run_id)), new Set(this.evidence.turns.map(t => t.run_id)));
  this.evidence.result = 'VERIFIED';
});

After(async function ({ result }) {
  if (!this.evidence) return;
  let cleanupError;
  try { await http(this, 'DELETE', `/api/uar/sessions/${this.session}`, undefined, 204); }
  catch (error) { if (!String(error).includes('HTTP 404')) cleanupError = error; }
  if (this.created) {
    try { await http(this, 'DELETE', `/api/agents/${this.agentId}`, undefined, 204); }
    catch (error) { cleanupError ??= error; }
  }
  this.evidence.mode = this.mode;
  this.evidence.scenario_status = cleanupError ? 'FAILED' : result.status;
  if (cleanupError) this.evidence.cleanup_error = cleanupError.message;
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify({ evidence: this.record, status: this.evidence.scenario_status }), 'application/json');
  if (cleanupError) throw cleanupError;
});
