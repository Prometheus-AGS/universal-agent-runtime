import assert from 'node:assert/strict';
import { randomInt, randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { parseAnswerJson } from '../support/answer-oracle.mjs';

setDefaultTimeout(240_000);

async function http(w, method, path, payload, expected = [200]) {
  const exchange = { method, path, request: payload };
  w.evidence.exchanges.push(exchange);
  const response = await fetch(new URL(path, w.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(170_000),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': w.workspace,
      ...(path.includes('/stream') && { Accept: 'text/event-stream' }) },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  if (method === 'POST' && path === '/api/agents' && response.status === 201) w.created = true;
  exchange.status = response.status;
  exchange.response = await response.text();
  assert.ok(expected.includes(response.status), `${method} ${path}: HTTP ${response.status}: ${exchange.response.slice(0, 800)}`);
  return { response, text: exchange.response };
}

// Parse SSE into {event, data} records.
function frames(raw) {
  return raw.split(/\r?\n\r?\n/).flatMap(block => {
    const lines = block.split(/\r?\n/);
    const event = lines.find(line => line.startsWith('event:'))?.slice(6).trim() ?? null;
    const data = lines.filter(line => line.startsWith('data:')).map(line => line.slice(5).trimStart()).join('\n');
    if (!data || data === '[DONE]') return [];
    return [{ event, data: JSON.parse(data) }];
  });
}

Given('a disposable saved agent with a private instruction marker for native runs', async function () {
  this.started = performance.now();
  this.id = randomUUID();
  this.agentId = `disposable-native-stream-${this.id}`;
  this.session = randomUUID();
  this.model = process.env.UAR_MODEL;
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.base = new URL(process.env.UAR_BASE_URL);
  assert.ok(this.model && this.workspace);
  assert.ok(['http:', 'https:'].includes(this.base.protocol) && !this.base.username && !this.base.password);
  this.record = resolve(`.prometheus/verification/native-run-streams/${this.id}.json`);
  this.marker = `NATIVE-${randomUUID()}`;
  this.evidence = { scenario: 'I03-I04-NATIVE-RUN-STREAMS', client_platform: process.platform, agent_id: this.agentId,
    session_id: this.session, model: this.model, authentication_supplied_by_test: false,
    follow_up_contract: 'server session continuity via session_id', exchanges: [] };
  const agent = JSON.parse((await http(this, 'GET', '/api/agents/default-agent')).text);
  agent.id = this.agentId;
  agent.version = '1.0.0';
  agent.metadata = { ...agent.metadata, title: 'Disposable native-stream agent', tags: ['disposable', 'verification'] };
  agent.runtime.entry = 'default';
  agent.prompt = { system: `Return only a JSON object with keys marker and answer. Always set marker to "${this.marker}". Put the requested numeric result in answer.`, instructions: [] };
  const split = this.model.indexOf('/');
  agent.policy.provider = { default: { provider: this.model.slice(0, split), model: this.model.slice(split + 1) }, fallbacks: [] };
  // An empty allow list means Auto (every eligible tool); one unmatched placeholder selects none.
  agent.policy.tools.allow = [`native-no-tools-${this.id.slice(0, 8)}`];
  agent.memory.kb.enabled = false;
  await http(this, 'POST', '/api/agents', agent, [201]);
  const left = randomInt(21, 80), right = randomInt(11, 40), extra = randomInt(101, 500);
  this.more = randomInt(11, 90);
  this.firstText = `What is (${left} multiplied by ${right}) plus ${extra}?`;
  this.expected = [left * right + extra, left * right + extra + this.more];
  this.evidence.expected = this.expected;
});

// Admit the run, then subscribe at once; the stream closes after the terminal event.
async function admitAndStream(w, text, query) {
  const created = JSON.parse((await http(w, 'POST', '/api/uar/runs',
    { agent_id: w.agentId, session_id: w.session, input: text }, [200, 201, 202])).text);
  assert.ok(created.run_id, 'Admission must report the run id');
  const path = `/api/uar/runs/${encodeURIComponent(created.run_id)}/stream${query}`;
  const { response, text: raw } = await http(w, 'GET', path);
  assert.match(response.headers.get('content-type') ?? '', /^text\/event-stream\b/);
  return { runId: created.run_id, streamUrl: created.stream_url, records: frames(raw) };
}

async function twoRuns(w, mode, query) {
  w.mode = mode;
  w.turns = [await admitAndStream(w, w.firstText, query),
    await admitAndStream(w, `Add ${w.more} to your previous answer.`, query)];
}

When('I admit a fresh calculation and then a dependent follow-up as native runs and subscribe to each default stream', async function () {
  await twoRuns(this, 'legacy-default', '');
});

When('I admit a fresh calculation and then a dependent follow-up as native runs and subscribe to each agui_spec stream', async function () {
  // Without a cursor agui_spec joins at the latest event with snapshots; last_event_id=0 requests the full sequence.
  await twoRuns(this, 'agui_spec', '?stream_mode=agui_spec&last_event_id=0');
});

async function assertSessionRuns(w) {
  const runs = JSON.parse((await http(w, 'GET', '/api/uar/runs?limit=100')).text)
    .filter(run => run.conversation_id === w.session);
  w.evidence.runs = runs.map(r => ({ run_id: r.run_id, agent_id: r.agent_id, status: r.status }));
  assert.equal(runs.length, 2, 'One run per admission');
  assert.deepEqual(new Set(runs.map(r => r.run_id)), new Set(w.turns.map(t => t.runId)));
  for (const run of runs) assert.equal(run.agent_id, w.agentId);
}

Then('each legacy run stream delivers incremental text, completes, and answers correctly in one session', async function () {
  this.evidence.turns = [];
  for (const [index, turn] of this.turns.entries()) {
    const { records } = turn;
    const doneAt = records.findIndex(r => r.event === 'agui.done');
    assert.ok(doneAt >= 0, 'The stream must end with agui.done');
    assert.ok(!records.some(r => r.event === 'agui.error' || r.event === 'agui.cancelled'), 'The run must not fail');
    const deltas = records.slice(0, doneAt).filter(r => r.event === 'agui.message.delta');
    assert.ok(deltas.length >= 1, 'Content deltas must arrive before agui.done');
    const answer = parseAnswerJson(deltas.map(r => r.data?.delta?.text ?? r.data?.text ?? '').join(''));
    this.evidence.turns.push({ run_id: turn.runId, stream_url: turn.streamUrl,
      events: [...new Set(records.map(r => r.event))], deltas: deltas.length, answer });
    assert.equal(answer.marker, this.marker);
    assert.equal(Number(answer.answer), this.expected[index]);
  }
  await assertSessionRuns(this);
  this.evidence.result = 'VERIFIED';
});

Then('each official run stream starts and finishes its run once with correct answers in one session', async function () {
  this.evidence.turns = [];
  for (const [index, turn] of this.turns.entries()) {
    const events = turn.records.filter(r => !r.event?.startsWith('runtime.')).map(r => r.data);
    assert.ok(!events.some(e => ['RUN_ERROR', 'RUN_CANCELLED'].includes(e.type)), 'The run must succeed');
    const starts = events.filter(e => e.type === 'RUN_STARTED');
    const finishes = events.filter(e => e.type === 'RUN_FINISHED');
    assert.equal(starts.length, 1);
    assert.equal(finishes.length, 1);
    assert.equal(starts[0].runId, turn.runId, 'RUN_STARTED names the admitted run');
    const finishedAt = events.indexOf(finishes[0]);
    const content = events.slice(0, finishedAt).filter(e => e.type === 'TEXT_MESSAGE_CONTENT');
    assert.ok(content.length >= 1, 'Text content must arrive before RUN_FINISHED');
    const answer = parseAnswerJson(content.map(e => e.delta).join(''));
    this.evidence.turns.push({ run_id: turn.runId, types: [...new Set(events.map(e => e.type))],
      content_events: content.length, answer });
    assert.equal(answer.marker, this.marker);
    assert.equal(Number(answer.answer), this.expected[index]);
  }
  await assertSessionRuns(this);
  this.evidence.result = 'VERIFIED';
});

After(async function ({ result }) {
  if (!this.evidence) return;
  let cleanupError;
  try { await http(this, 'DELETE', `/api/uar/sessions/${this.session}`, undefined, [204, 404]); }
  catch (error) { cleanupError = error; }
  if (this.created) {
    try { await http(this, 'DELETE', `/api/agents/${this.agentId}`, undefined, [204]); }
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
