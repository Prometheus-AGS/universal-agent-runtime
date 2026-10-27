import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';

setDefaultTimeout(125_000);
const ORCHESTRATOR = 'orchestrator-agent';
const MATH_ID = 'math-specialist-db64c2a8';
const WRITER_ID = 'writing-specialist';
const ROUTE_PATH = `/api/uar/ag-ui/agents/${ORCHESTRATOR}`;
// The only documented refusal: an invalid classifier choice fails without falling back.
const NO_FALLBACK_REFUSAL = 'graph_routing_selection_invalid';

async function request(w, method, path, payload, expected = 200) {
  const response = await fetch(new URL(path, w.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(60_000),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': w.workspace },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  if (method === 'POST' && path === '/api/agents' && response.status === 201) w.created = true;
  const text = await response.text();
  w.evidence.exchanges.push({ method, path, request: payload, status: response.status, response: text.slice(0, 4000) });
  const accepted = Array.isArray(expected) ? expected : [expected];
  assert.ok(accepted.includes(response.status), `${method} ${path}: ${response.status}: ${text.slice(0, 800)}`);
  // A missing record may answer 404 with a plain-text body; only successful responses carry JSON.
  return text && response.ok ? JSON.parse(text) : null;
}

async function roster(w, label) {
  const listing = await request(w, 'GET', '/api/agents');
  const agents = listing.runtime_agents.map(a => ({ id: a.id, entry: a.runtime?.entry, title: a.metadata?.title }));
  const eligible = agents.filter(a => a.id !== ORCHESTRATOR && a.entry !== 'orchestrator').map(a => a.id).sort();
  w.evidence.roster_snapshots[label] = { agents, eligible };
  return eligible;
}

async function stream(w, payload) {
  const response = await fetch(new URL(ROUTE_PATH, w.base), {
    method: 'POST', redirect: 'error', signal: AbortSignal.timeout(120_000),
    headers: { 'Content-Type': 'application/json', Accept: 'text/event-stream', 'x-uar-workspace-id': w.workspace },
    body: JSON.stringify(payload),
  });
  const text = await response.text();
  assert.equal(response.status, 200, `${ROUTE_PATH}: ${response.status}: ${text.slice(0, 800)}`);
  assert.match(response.headers.get('content-type') ?? '', /^text\/event-stream\b/);
  return text.split(/\r?\n\r?\n/).flatMap(block => {
    const data = block.split(/\r?\n/).filter(line => line.startsWith('data:'))
      .map(line => line.slice(5).replace(/^ /, '')).join('\n');
    return data && data !== '[DONE]' ? [JSON.parse(data)] : [];
  });
}

// Route one fresh task through the orchestrator. The only approval answered is the
// local spawn_agent interrupt this run itself raised, as in the saved writer scenario.
async function route(w, label, task) {
  const threadId = randomUUID();
  const firstRunId = randomUUID();
  const record = { label, task, thread_id: threadId, runs: [] };
  w.evidence.routes[label] = record;
  w.threads.push(threadId);
  let events = await stream(w, { threadId, runId: firstRunId, protocolVersion: '1.0',
    messages: [{ id: randomUUID(), role: 'user', content: task }], tools: [], context: [],
    forwardedProps: { uar: { model: w.model } } });
  record.runs.push({ run_id: firstRunId, events });
  let runId = firstRunId;
  const first = events.find(e => e.type === 'RUN_FINISHED');
  if (first?.outcome?.type === 'interrupt') {
    const interrupts = first.outcome.interrupts;
    assert.equal(interrupts.length, 1, 'Expected one local delegation approval');
    const [interrupt] = interrupts;
    assert.equal(interrupt.reason, 'tool_approval');
    assert.ok(events.some(e => e.type === 'TOOL_CALL_START' && e.toolCallId === interrupt.toolCallId
      && e.toolCallName === 'spawn_agent'), 'Only the requested local agent-spawn approval may be answered');
    runId = randomUUID();
    events = await stream(w, { threadId, runId, parentRunId: firstRunId, protocolVersion: '1.0',
      messages: [], tools: [], context: [], forwardedProps: { uar: {} },
      resume: [{ interruptId: interrupt.id, status: 'resolved', payload: { approved: true } }] });
    record.approval = { interrupt_id: interrupt.id, tool: 'spawn_agent', approved: true, parent_run_id: firstRunId };
    record.runs.push({ run_id: runId, events });
  }
  const selections = events.filter(e => e.type === 'CUSTOM' && e.name === 'uar.agent.selected');
  const errors = events.filter(e => e.type === 'RUN_ERROR');
  const finished = events.filter(e => e.type === 'RUN_FINISHED');
  // Reasoning is never answer text: drop closed blocks, then an unclosed block to the end.
  const text = events.filter(e => e.type === 'TEXT_MESSAGE_CONTENT').map(e => e.delta ?? '').join('')
    .replace(/<think>[\s\S]*?<\/think>/g, '').replace(/<think>[\s\S]*$/, '').trim();
  Object.assign(record, { wire_run_id: runId, selection: selections[0]?.value ?? null,
    selection_count: selections.length, error: errors[0] ?? null, outcome: finished[0]?.outcome ?? null,
    answer: text.slice(0, 4000) });
  return { events, selections, errors, finished, text, runId, threadId };
}

function assertSelectedBeforeAnswer(result, expectedRunId, expectedThreadId) {
  assert.equal(result.selections.length, 1, 'Expected one agent-selection event');
  const [event] = result.selections;
  assert.equal(event.threadId, expectedThreadId);
  assert.equal(event.runId, expectedRunId);
  assert.equal(event.agentId, ORCHESTRATOR);
  assert.equal(event.value.sourceAgentId, ORCHESTRATOR);
  assert.equal(event.value.selectionMethod, 'intent');
  assert.ok(event.value.rosterRevision && event.value.selectedThreadId);
  assert.equal(result.finished.length, 1, 'Expected one terminal event');
  assert.equal(result.finished[0].outcome?.type, 'success', 'The routed run must complete');
  const selectedAt = result.events.indexOf(event);
  const contentAt = result.events.findIndex(e => e.type === 'TEXT_MESSAGE_CONTENT');
  const finishedAt = result.events.indexOf(result.finished[0]);
  assert.ok(selectedAt >= 0 && selectedAt < contentAt && contentAt < finishedAt,
    'Selection must precede the answer and successful completion');
  return event.value;
}

// After removal: either an eligible agent completes the task, or the router refuses
// with its documented no-fallback error. The deleted specialist is never chosen.
function assertRoutedWithinRoster(w, result, label) {
  const record = w.evidence.routes[label];
  assert.ok(!result.text.includes(w.marker), `${label} answered with the deleted specialist's signature`);
  if (result.errors.length) {
    const messages = result.errors.map(e => e.message);
    if (result.selections.length) {
      record.disposition = 'selected_then_failed';
      assert.fail(`${label} selected ${result.selections[0].value?.agent?.id} and then failed: ${messages.join('; ')}`);
    }
    assert.deepEqual(messages, [NO_FALLBACK_REFUSAL], `${label} failed for an undocumented reason`);
    record.disposition = 'refused_without_fallback';
    return;
  }
  const selection = assertSelectedBeforeAnswer(result, result.runId, result.threadId);
  assert.notEqual(selection.agent.id, w.agentId, `${label} selected the deleted specialist`);
  assert.ok(w.currentEligible.includes(selection.agent.id), `${label} selected ${selection.agent.id}, outside the live roster`);
  assert.notEqual(selection.rosterRevision, w.evidence.routes.eligible_specialist.selection.rosterRevision,
    'Routing used the roster revision that still contained the deleted specialist');
  assert.ok(result.text.length > 0, `${label} returned no answer`);
  record.disposition = 'routed_to_eligible_agent';
}

Given('the saved mathematics and writing specialists are in the live roster', async function () {
  this.started = performance.now();
  this.id = randomUUID();
  this.record = resolve(`.prometheus/verification/live-roster-routing/${this.id}.json`);
  this.base = new URL(process.env.UAR_BASE_URL);
  this.workspace = process.env.UAR_WORKSPACE_ID;
  this.model = process.env.UAR_MODEL;
  this.threads = [];
  this.evidence = { scenario: 'C34-LIVE-ROSTER-ROUTING-01', platform: process.platform,
    authentication_supplied_by_test: false, previous_scenarios_rerun: false, model: this.model,
    no_match_binding: 'Operator-approved 2026-10-27: unmatched intent must route only within the current eligible roster or be refused with graph_routing_selection_invalid; the absent explicit no-match path is a recorded product gap.',
    roster_snapshots: {}, routes: {}, exchanges: [] };
  assert.ok(['http:', 'https:'].includes(this.base.protocol) && !this.base.username && !this.base.password);
  assert.ok(this.workspace && this.model?.includes('/'));
  for (const id of [MATH_ID, WRITER_ID]) assert.equal((await request(this, 'GET', `/api/agents/${id}`)).id, id);
  const eligible = await roster(this, 'initial');
  assert.ok(eligible.includes(MATH_ID) && eligible.includes(WRITER_ID));
  this.template = await request(this, 'GET', '/api/agents/default-agent');
});

When('I register a disposable houseplant care specialist', async function () {
  this.agentId = `disposable-plant-clinic-${this.id.slice(0, 8)}`;
  this.marker = `FERNWELL-${randomUUID().slice(0, 8).toUpperCase()}`;
  const artifact = structuredClone(this.template);
  Object.assign(artifact, { id: this.agentId, version: '1.0.0', extensions: {} });
  artifact.metadata = { ...artifact.metadata, title: 'Houseplant Care Diagnostician',
    description: 'Diagnoses houseplant health problems such as browning or yellowing leaves, crispy fronds, root rot, overwatering, underwatering, humidity, light and pests for ferns, succulents and other indoor plants.',
    tags: ['houseplants', 'plant care', 'gardening', 'plant diagnosis', 'ferns'] };
  artifact.prompt = { system: 'You are a houseplant care diagnostician. Diagnose the most likely cause of the described plant problem and give two or three concrete care steps, in at most 120 words. '
    + `End every answer with this exact signature on its own line: Fernwell Plant Clinic ${this.marker}`, instructions: [] };
  artifact.runtime.entry = 'default';
  const split = this.model.indexOf('/');
  artifact.policy.provider = { default: { provider: this.model.slice(0, split), model: this.model.slice(split + 1) }, fallbacks: [] };
  artifact.policy.tools.allow = [];
  artifact.memory.kb.enabled = false;
  const saved = await request(this, 'POST', '/api/agents', artifact, 201);
  assert.equal(saved.id, this.agentId);
  Object.assign(this.evidence, { agent_id: this.agentId, marker: this.marker });
});

Then('the live roster includes the houseplant specialist', async function () {
  this.currentEligible = await roster(this, 'after_registration');
  assert.ok(this.currentEligible.includes(this.agentId));
});

When('I ask the orchestrator to diagnose a houseplant problem', async function () {
  this.eligibleRoute = await route(this, 'eligible_specialist',
    'My Boston fern\'s fronds are turning brown and crispy at the tips, even though I water it weekly. What is wrong and how do I fix it?');
});

Then('intent routing selects the houseplant specialist before its answer', function () {
  const r = this.eligibleRoute;
  assert.equal(r.errors.length, 0, JSON.stringify(r.errors));
  const selection = assertSelectedBeforeAnswer(r, r.runId, r.threadId);
  assert.equal(selection.agent.id, this.agentId);
});

Then('the houseplant specialist answers with its private clinic signature', function () {
  const text = this.eligibleRoute.text;
  assert.ok(text.includes(`Fernwell Plant Clinic ${this.marker}`), 'Missing the signature known only to the new specialist');
  assert.match(text, /humid|water|mist|dry/i);
});

When('I delete the houseplant specialist', async function () {
  await request(this, 'DELETE', `/api/agents/${this.agentId}`, undefined, 204);
  this.deleted = true;
  await request(this, 'GET', `/api/agents/${this.agentId}`, undefined, 404);
});

Then('the live roster no longer includes the houseplant specialist', async function () {
  this.currentEligible = await roster(this, 'after_deletion');
  assert.ok(!this.currentEligible.includes(this.agentId));
  assert.ok(this.currentEligible.includes(MATH_ID) && this.currentEligible.includes(WRITER_ID));
});

When('I ask the orchestrator a fresh houseplant question', async function () {
  this.freshRoute = await route(this, 'fresh_task_after_removal',
    'The lower leaves of my snake plant are turning yellow and mushy at the base. What is likely wrong and what should I do?');
});

Then('routing uses only a currently eligible agent and never the deleted specialist', function () {
  assertRoutedWithinRoster(this, this.freshRoute, 'fresh_task_after_removal');
});

When('I ask the orchestrator a request that matches no specialist', async function () {
  this.unmatchedRoute = await route(this, 'unmatched_intent',
    'In one sentence, suggest a cheese that pairs well with a dry Riesling.');
});

Then('that request is also routed only within the current eligible roster', function () {
  assertRoutedWithinRoster(this, this.unmatchedRoute, 'unmatched_intent');
  const revisions = ['fresh_task_after_removal', 'unmatched_intent']
    .map(label => this.evidence.routes[label].selection?.rosterRevision).filter(Boolean);
  if (revisions.length === 2) assert.equal(revisions[0], revisions[1], 'The unchanged live roster must keep one revision');
  this.evidence.result = 'VERIFIED';
});

After({ timeout: 120_000 }, async function ({ result }) {
  if (!this.evidence) return;
  const errors = [];
  if (this.created && !this.deleted) {
    try { await request(this, 'DELETE', `/api/agents/${this.agentId}`, undefined, [204, 404]); this.deleted = true; }
    catch (error) { errors.push(`Agent cleanup: ${error}`); }
  }
  this.evidence.disposable_agent_deleted = Boolean(this.deleted);
  for (const thread of this.threads) {
    try { await request(this, 'DELETE', `/api/uar/sessions/${thread}`, undefined, [204, 404]); }
    catch (error) { errors.push(`Session cleanup ${thread}: ${error}`); }
  }
  try {
    for (const id of [MATH_ID, WRITER_ID]) await request(this, 'GET', `/api/agents/${id}`);
    this.evidence.saved_agents_retained = true;
  } catch (error) { errors.push(`Saved agent check: ${error}`); }
  this.evidence.cleanup_errors = errors;
  this.evidence.status = errors.length ? 'FAILED' : result.status;
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify({ evidence: this.record, status: this.evidence.status }), 'application/json');
  assert.deepEqual(errors, [], 'Cleanup must succeed');
});
