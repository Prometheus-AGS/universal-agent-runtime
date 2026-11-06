import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { readFile, mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';

// Same existing runtime-configuration timeout when both step files are loaded.
setDefaultTimeout(240_000);
let poisoned = false;
const prompt = 'Return the owned Messages usage marker.';
const tool = { name: 'lookup_code', description: 'Owned synthetic caller lookup.',
  input_schema: { type: 'object', properties: { key: { type: 'string' } }, required: ['key'] } };
function local(value) {
  const url = new URL(value);
  assert.ok(['http:', 'https:'].includes(url.protocol) && ['127.0.0.1', '[::1]'].includes(url.hostname));
  assert.ok(!url.username && !url.password && !url.search && !url.hash);
  return url;
}
async function request(w, base, path, method = 'GET', body) {
  assert.ok(++w.messagesOps <= 140, 'Owned Messages HTTP budget exceeded');
  const response = await fetch(new URL(path, base), { method, redirect: 'error', signal: AbortSignal.timeout(35_000),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': w.messagesWorkspace },
    body: body === undefined ? undefined : JSON.stringify(body) });
  const raw = await response.text();
  let value;
  try { value = raw ? JSON.parse(raw) : null; } catch { value = null; }
  w.messagesRecord.exchanges.push({ method, path, status: response.status });
  return { status: response.status, value, raw };
}
async function state(w) {
  const response = await request(w, w.messagesControl, '/__messages/state');
  assert.equal(response.status, 200); assert.equal(response.value.namespace, w.messagesFixture.namespace);
  assert.equal(response.value.overflowed, false);
  return response.value;
}
async function configure(w, mode) {
  const response = await request(w, w.messagesControl, '/__messages/control', 'POST', { namespace: w.messagesFixture.namespace, mode });
  assert.equal(response.status, 200); assert.equal(response.value.configured, true);
}
function payload(w, session, stream, protocol = 'messages') {
  return { model: w.messagesFixture.model, agent_id: w.messagesFixture.agentId, session_id: session,
    stream, max_tokens: 64, messages: [{ role: 'user', content: prompt }],
    ...(protocol === 'messages' ? {} : { temperature: 0 }) };
}
function parseStream(raw) {
  return raw.split(/\r?\n\r?\n/).flatMap(frame => {
    const text = frame.split(/\r?\n/).filter(line => line.startsWith('data:')).map(line => line.slice(5).trimStart()).join('\n');
    if (!text || text === '[DONE]') return [];
    return [JSON.parse(text)];
  });
}
function messagesResult(response, stream) {
  assert.equal(response.status, 200);
  if (!stream) return response.value;
  const events = parseStream(response.raw);
  assert.ok(!events.some(event => event.type === 'error'));
  assert.equal(events.at(-1)?.type, 'message_stop');
  const delta = events.findLast(event => event.type === 'message_delta');
  assert.ok(delta?.usage);
  return { usage: delta.usage, uar_usage: delta.uar_usage,
    text: events.filter(event => event.delta?.type === 'text_delta').map(event => event.delta.text).join('') };
}
function chatResult(response, stream) {
  assert.equal(response.status, 200);
  if (!stream) return response.value;
  const events = parseStream(response.raw);
  assert.ok(!events.some(event => event.error));
  assert.ok(response.raw.includes('data: [DONE]'));
  return { usage: events.findLast(event => event.usage)?.usage,
    text: events.map(event => event.choices?.[0]?.delta?.content ?? '').join('') };
}
async function runs(w) {
  const response = await request(w, w.messagesBase, '/api/uar/runs');
  assert.equal(response.status, 200); assert.ok(Array.isArray(response.value)); return response.value;
}
async function newRun(w, before) {
  for (let attempt = 0; attempt < 25; attempt++) {
    const added = (await runs(w)).filter(run => !before.has(run.run_id) && run.agent_id === w.messagesFixture.agentId);
    if (added.length) { assert.equal(added.length, 1); w.messagesRuns.add(added[0].run_id); return added[0]; }
    await delay(100);
  }
  throw new Error('No unique owned run appeared at the actual mounted route');
}
async function ledger(w, run) {
  const response = await request(w, w.messagesBase, `/api/uar/accounting/calls/${encodeURIComponent(run.run_id)}`);
  assert.equal(response.status, 200); assert.ok(Array.isArray(response.value)); return response.value;
}
async function turn(w, path, session, stream, body) {
  const before = new Set((await runs(w)).map(run => run.run_id));
  const response = await request(w, w.messagesBase, path, 'POST', body);
  const run = await newRun(w, before);
  return { response, run, rows: await ledger(w, run), session, stream };
}
Given('the externally provisioned owned Messages usage fixture', async function () {
  assert.equal(poisoned, false, 'Unknown owned cleanup requires external reconciliation before another scenario');
  assert.ok(process.env.UAR_BASE_URL && process.env.UAR_WORKSPACE_ID && process.env.UAR_MESSAGES_USAGE_FIXTURE);
  const fixture = JSON.parse(await readFile(process.env.UAR_MESSAGES_USAGE_FIXTURE, 'utf8'));
  assert.equal(fixture.dedicated, true); assert.match(fixture.namespace, /^[A-Za-z0-9_-]{1,96}$/);
  assert.match(fixture.identity.sourceCommit, /^[a-f0-9]{40}$/);
  assert.match(fixture.identity.imageDigest, /^sha256:[a-f0-9]{64}$/);
  assert.match(fixture.identity.configDigest, /^[a-f0-9]{64}$/);
  assert.equal(fixture.identity.platform, 'linux/amd64');
  assert.ok(fixture.agentId.startsWith(`${fixture.namespace}-`));
  assert.equal(fixture.adapter, 'native_anthropic');
  this.messagesFixture = fixture; this.messagesBase = local(process.env.UAR_BASE_URL);
  this.messagesForeign = local(fixture.foreignOwnerBaseUrl); this.messagesControl = local(fixture.controlUrl);
  this.messagesWorkspace = process.env.UAR_WORKSPACE_ID; this.messagesOps = 0;
  this.messagesSessions = new Set(); this.messagesRuns = new Set();
  this.messagesRecord = { identity: fixture.identity, namespace: fixture.namespace, authentication_supplied_by_test: false, exchanges: [] };
  this.messagesReceipt = resolve(`.prometheus/verification/messages-usage/${randomUUID()}.json`);
  const initial = await state(this); assert.equal(initial.active, 0); this.messagesInitial = initial;
});
When('equivalent chat and Messages turns use {string} provider usage with streaming {string}', async function (mode, streaming) {
  const stream = streaming === 'true'; assert.ok(['true', 'false'].includes(streaming));
  await configure(this, mode); this.messagesMode = mode;
  const chatSession = randomUUID(); const messagesSession = randomUUID();
  this.messagesSessions.add(chatSession); this.messagesSessions.add(messagesSession);
  this.chatUsageTurn = await turn(this, '/v1/chat/completions', chatSession, stream, payload(this, chatSession, stream, 'chat'));
  this.messagesUsageTurn = await turn(this, '/v1/messages', messagesSession, stream, payload(this, messagesSession, stream));
});
Then('both wire formats preserve the applicable usage and one settlement per call', async function () {
  const chat = chatResult(this.chatUsageTurn.response, this.chatUsageTurn.stream);
  const messages = messagesResult(this.messagesUsageTurn.response, this.messagesUsageTurn.stream);
  const chatText = chat.text ?? chat.choices?.[0]?.message?.content;
  const messagesText = messages.text ?? messages.content?.filter(block => block.type === 'text').map(block => block.text).join('');
  assert.equal(chatText, 'MESSAGES-USAGE-OK'); assert.equal(messagesText, chatText);
  const reported = this.messagesMode !== 'absent'; const zero = this.messagesMode === 'zero';
  for (const key of ['input_tokens', 'output_tokens']) {
    assert.deepEqual(messages.uar_usage[key], { available: reported, source: reported ? 'provider_reported' : 'unavailable' });
  }
  if (reported) {
    const unknownCache = this.messagesMode === 'uncached-only';
    const input = zero ? 0 : 7, output = zero ? 0 : 5, read = zero || unknownCache ? 0 : 3, creation = zero || unknownCache ? 0 : 2;
    assert.equal(messages.usage.input_tokens, input); assert.equal(messages.usage.output_tokens, output);
    assert.equal(messages.usage.cache_read_input_tokens, unknownCache ? undefined : read);
    assert.equal(messages.usage.cache_creation_input_tokens, unknownCache ? undefined : creation);
    assert.equal(chat.usage.prompt_tokens, input + read + creation);
    assert.equal(chat.usage.completion_tokens, output); assert.equal(chat.usage.total_tokens, input + read + creation + output);
    assert.deepEqual(chat.usage.prompt_tokens_details, unknownCache ? undefined : { cached_tokens: read, cache_creation_tokens: creation });
    for (const result of [this.chatUsageTurn, this.messagesUsageTurn]) {
      assert.equal(result.rows.length, 1, 'Projection cannot create a second charge');
      assert.equal(result.rows[0].state, 'settled');
      assert.equal(result.rows[0].observed_tokens, input + read + creation + output);
      assert.ok(Number.isFinite(result.rows[0].observed_cost_usd));
    }
    assert.equal(this.chatUsageTurn.rows[0].observed_cost_usd, this.messagesUsageTurn.rows[0].observed_cost_usd);
  } else {
    assert.ok(chat.usage == null); assert.equal(messages.usage.input_tokens, 0); assert.equal(messages.usage.output_tokens, 0);
    assert.equal(messages.usage.cache_read_input_tokens, undefined); assert.equal(messages.usage.cache_creation_input_tokens, undefined);
    for (const result of [this.chatUsageTurn, this.messagesUsageTurn]) {
      assert.equal(result.rows.length, 1);
      assert.equal(result.rows[0].state, 'pending', 'Unavailable usage cannot settle liability as free work');
      assert.equal(result.rows[0].observed_cost_usd, null);
    }
  }
  const after = await state(this); assert.equal(after.arrivals - this.messagesInitial.arrivals, 2);
  this.messagesRecord.accounting = [this.chatUsageTurn, this.messagesUsageTurn].map(result => ({ runId: result.run.run_id,
    calls: result.rows.map(row => ({ callId: row.admission.identity.call_id, attempt: row.admission.identity.attempt,
      state: row.state, tokens: row.observed_tokens, cost: row.observed_cost_usd })) }));
});
When('one real tool-loop run through each protocol uses {string} usage with streaming {string}', async function (mode, streaming) {
  assert.ok(['true', 'false'].includes(streaming));
  assert.ok(['loop-missing-known', 'loop-known-missing', 'loop-known', 'loop-zero', 'loop-missing-read', 'loop-missing-creation'].includes(mode));
  assert.equal(this.messagesFixture.runtimeToolProfile, 'native_echo',
    'External provisioning must expose and permit the existing runtime-owned native_echo profile');
  this.messagesLoopMode = mode; this.messagesLoopTurns = [];
  for (const protocol of ['chat', 'messages']) {
    await configure(this, mode);
    const before = await state(this);
    const session = randomUUID(); this.messagesSessions.add(session);
    const body = payload(this, session, streaming === 'true', protocol);
    assert.equal(body.tools, undefined, 'The caller cannot implement the native tool loop');
    const path = protocol === 'messages' ? '/v1/messages' : '/v1/chat/completions';
    // One public POST creates this run. The runtime owns the tool result and
    // second provider request; no caller-result continuation is sent.
    const result = await turn(this, path, session, streaming === 'true', body);
    const after = await state(this);
    assert.equal(after.arrivals - before.arrivals, 2, 'One run must submit exactly two provider calls');
    assert.equal(after.completed - before.completed, 2);
    assert.equal(after.active, 0); assert.equal(after.held, 0);
    const peer = after.requests.filter(item => item.id > before.arrivals);
    assert.equal(peer.length, 2); assert.deepEqual(peer.map(item => item.loopStep), [1, 2]);
    assert.deepEqual(peer.map(item => item.tool), [true, false]);
    for (const item of peer) { assert.equal(item.mode, mode); assert.equal(item.toolName, 'native_echo'); assert.equal(item.toolDeclared, true); }
    assert.equal(peer[0].toolResultMatched, false); assert.equal(peer[1].toolResultMatched, true);
    assert.equal(peer[1].toolId, peer[0].toolId, 'Original provider tool identity survives real execution');
    const current = await request(this, this.messagesBase, `/api/uar/runs/${encodeURIComponent(result.run.run_id)}`);
    assert.equal(current.status, 200); assert.equal(current.value.status, 'done');
    assert.equal(current.value.run_id, result.run.run_id); assert.equal(current.value.agent_id, this.messagesFixture.agentId);
    const snapshot = await request(this, this.messagesBase, `/api/uar/runs/${encodeURIComponent(result.run.run_id)}/events`);
    assert.equal(snapshot.status, 200); assert.equal(snapshot.value.runId, result.run.run_id);
    assert.equal(snapshot.value.gapReason, null, 'Truncated history cannot prove tool-loop completion');
    const events = snapshot.value.events;
    assert.ok(!events.some(event => ['agui.error', 'agui.cancelled', 'agui.tool_call.denied', 'agui.tool_call.approval_required'].includes(event.eventName)));
    const starts = events.filter(event => event.eventName === 'agui.tool_call.complete');
    const ends = events.filter(event => event.eventName === 'agui.tool_result');
    assert.equal(starts.length, 1); assert.equal(ends.length, 1);
    const args = { message: `${this.messagesFixture.namespace}-governed-echo` };
    for (const event of [starts[0], ends[0]]) {
      assert.equal(event.data.request_id, result.run.run_id); assert.equal(event.data.id, peer[0].toolId);
      assert.equal(event.data.name, 'native_echo');
    }
    assert.deepEqual(JSON.parse(starts[0].data.arguments_json), args);
    assert.equal(ends[0].data.success, true); assert.deepEqual(JSON.parse(ends[0].data.content), { echoed: args });
    assert.ok(starts[0].eventId < ends[0].eventId);
    const terminal = events.filter(event => event.eventName === 'agui.done');
    assert.equal(terminal.length, 1); assert.equal(terminal[0].data.request_id, result.run.run_id);
    assert.ok(ends[0].eventId < terminal[0].eventId);
    this.messagesLoopTurns.push({ ...result, protocol, peer, usage: terminal[0].data.usage, toolCallId: peer[0].toolId });
  }
});
Then('whole-run usage is complete only when every submitted call reports it and original call liabilities remain', async function () {
  for (const result of this.messagesLoopTurns) {
    const reports = result.peer.map(item => item.usage);
    const first = { input_tokens: 7, output_tokens: 5, cache_read_input_tokens: 3, cache_creation_input_tokens: 2 };
    const second = { input_tokens: 11, output_tokens: 13, cache_read_input_tokens: 17, cache_creation_input_tokens: 19 };
    const zero = { input_tokens: 0, output_tokens: 0, cache_read_input_tokens: 0, cache_creation_input_tokens: 0 };
    const expectedReports = {
      'loop-missing-known': [null, second], 'loop-known-missing': [first, null],
      'loop-known': [first, second], 'loop-zero': [zero, zero],
      'loop-missing-read': [{ input_tokens: 7, output_tokens: 5, cache_creation_input_tokens: 2 }, second],
      'loop-missing-creation': [{ input_tokens: 7, output_tokens: 5, cache_read_input_tokens: 3 }, second],
    };
    assert.deepEqual(reports, expectedReports[this.messagesLoopMode], 'The provider oracle itself must match the selected case');
    const complete = reports.every(report => report !== null);
    const total = key => complete && reports.every(report => Number.isSafeInteger(report[key]))
      ? reports.reduce((sum, report) => sum + report[key], 0) : undefined;
    const input = complete ? reports.reduce((sum, report) => sum + report.input_tokens
      + (report.cache_read_input_tokens ?? 0) + (report.cache_creation_input_tokens ?? 0), 0) : undefined;
    const output = total('output_tokens'), read = total('cache_read_input_tokens'), creation = total('cache_creation_input_tokens');
    assert.ok(result.usage, 'Canonical terminal retains partial-report availability');
    assert.equal(result.usage.input_tokens, input ?? null); assert.equal(result.usage.output_tokens, output ?? null);
    assert.equal(result.usage.uncached_input_tokens, total('input_tokens'));
    assert.equal(result.usage.total_tokens, complete ? input + output : null);
    assert.equal(result.usage.cache_read_input_tokens, read); assert.equal(result.usage.cache_creation_input_tokens, creation);
    const wire = result.protocol === 'messages' ? messagesResult(result.response, result.stream) : chatResult(result.response, result.stream);
    const text = wire.text ?? (result.protocol === 'messages'
      ? wire.content?.filter(block => block.type === 'text').map(block => block.text).join('') : wire.choices?.[0]?.message?.content);
    assert.equal(text, 'MESSAGES-USAGE-OK');
    if (result.protocol === 'messages') {
      if (!result.stream) assert.equal(wire.stop_reason, 'end_turn');
      for (const key of ['input_tokens', 'output_tokens']) {
        assert.deepEqual(wire.uar_usage[key], { available: complete, source: complete ? 'provider_reported' : 'unavailable' });
      }
      // Native reports expose uncached input independently. Missing one cache
      // category must not relabel the other call's cached input as uncached.
      assert.equal(wire.usage.input_tokens, total('input_tokens') ?? 0);
      assert.equal(wire.usage.output_tokens, output ?? 0);
      assert.equal(wire.usage.cache_read_input_tokens, read); assert.equal(wire.usage.cache_creation_input_tokens, creation);
    } else if (complete) {
      assert.equal(wire.usage.prompt_tokens, input); assert.equal(wire.usage.completion_tokens, output);
      assert.equal(wire.usage.total_tokens, input + output);
      assert.deepEqual(wire.usage.prompt_tokens_details, {
        ...(read !== undefined && { cached_tokens: read }), ...(creation !== undefined && { cache_creation_tokens: creation }),
      });
    } else assert.ok(wire.usage == null, 'A partial sum is not whole-run OpenAI usage');
    assert.equal(result.rows.length, 2, 'One original accounting liability per actual provider call');
    const identities = result.rows.map(row => row.admission.identity);
    assert.equal(new Set(identities.map(identity => identity.call_id)).size, 2);
    for (const identity of identities) {
      assert.equal(identity.run_id, result.run.run_id); assert.match(identity.call_id, /^[a-f0-9-]{36}$/);
      assert.equal(identity.attempt, 1); assert.ok(identity.owner);
    }
    assert.equal(identities[0].owner, identities[1].owner);
    const knownTokens = reports.filter(Boolean).map(report => report.input_tokens + report.output_tokens
      + (report.cache_read_input_tokens ?? 0) + (report.cache_creation_input_tokens ?? 0)).sort((a, b) => a - b);
    const settled = result.rows.filter(row => row.state === 'settled');
    const pending = result.rows.filter(row => row.state === 'pending');
    assert.equal(settled.length, knownTokens.length); assert.equal(pending.length, reports.length - knownTokens.length);
    assert.deepEqual(settled.map(row => row.observed_tokens).sort((a, b) => a - b), knownTokens,
      'Known per-call evidence survives an unavailable aggregate');
    for (const row of settled) assert.ok(Number.isFinite(row.observed_cost_usd));
    for (const row of pending) {
      assert.equal(row.observed_cost_usd, null); assert.equal(row.observed_tokens, 0);
      assert.equal(row.reconciliation_evidence, null, 'Missing usage is not reconciled as free work');
    }
    const again = await ledger(this, result.run);
    const ordered = rows => [...rows].sort((a, b) => a.admission.identity.call_id.localeCompare(b.admission.identity.call_id));
    assert.deepEqual(ordered(again), ordered(result.rows), 'Reads cannot rewrite original identities or settle liabilities');
  }
  assert.notEqual(this.messagesLoopTurns[0].run.run_id, this.messagesLoopTurns[1].run.run_id);
  assert.equal((await state(this)).arrivals - this.messagesInitial.arrivals, 4);
  this.messagesRecord.toolLoops = this.messagesLoopTurns.map(result => ({ protocol: result.protocol,
    runId: result.run.run_id, toolCallId: result.toolCallId, usage: result.usage, providerCalls: result.peer,
    calls: result.rows.map(row => ({ callId: row.admission.identity.call_id, attempt: row.admission.identity.attempt,
      state: row.state, tokens: row.observed_tokens, cost: row.observed_cost_usd })) }));
});
When('I cancel an owned Messages turn at {string}', async function (stage) {
  await configure(this, stage);
  const session = randomUUID(); this.messagesSessions.add(session);
  const body = payload(this, session, true);
  if (stage === 'tool-handoff') body.tools = [tool];
  const before = new Set((await runs(this)).map(run => run.run_id));
  this.messagesPending = request(this, this.messagesBase, '/v1/messages', 'POST', body);
  this.messagesPending.catch(() => {});
  this.messagesCancelRun = await newRun(this, before);
  let held = false;
  for (let attempt = 0; attempt < 25; attempt++) {
    const peer = await state(this); if (peer.held === 1) { held = true; break; } await delay(100);
  }
  assert.equal(held, true);
  const cancelled = await request(this, this.messagesBase, `/api/uar/runs/${encodeURIComponent(this.messagesCancelRun.run_id)}/cancel`, 'POST');
  assert.equal(cancelled.status, 200);
  this.messagesCancelledResponse = await this.messagesPending; this.messagesPending = undefined;
});
Then('Messages ends with cancellation failure and the owned producer is closed', async function () {
  const response = this.messagesCancelledResponse;
  if (response.status === 200) {
    const events = parseStream(response.raw); assert.ok(events.some(event => event.type === 'error'));
    assert.ok(!events.some(event => event.type === 'message_stop'));
    assert.ok(!events.some(event => event.type === 'message_delta' && ['end_turn', 'tool_use'].includes(event.delta?.stop_reason)));
  } else assert.equal(response.status, 502);
  let peer;
  for (let attempt = 0; attempt < 25; attempt++) { peer = await state(this); if (!peer.active && !peer.held) break; await delay(100); }
  assert.equal(peer.active, 0); assert.equal(peer.held, 0);
  assert.equal(peer.completed, this.messagesInitial.completed);
  assert.equal(peer.arrivals - this.messagesInitial.arrivals, 1);
  const run = await request(this, this.messagesBase, `/api/uar/runs/${encodeURIComponent(this.messagesCancelRun.run_id)}`);
  assert.equal(run.status, 200); assert.equal(run.value.status, 'cancelled');
  const snapshot = await request(this, this.messagesBase, `/api/uar/runs/${encodeURIComponent(this.messagesCancelRun.run_id)}/events`);
  assert.equal(snapshot.status, 200);
  assert.ok(snapshot.value.events.some(event => event.eventName === 'agui.cancelled'));
  assert.ok(!snapshot.value.events.some(event => event.eventName === 'agui.done'));
  const rows = await ledger(this, this.messagesCancelRun); assert.equal(rows.length, 1); assert.equal(rows[0].state, 'pending');
  this.messagesRecord.cancelledRun = this.messagesCancelRun.run_id;
});
When('an owned caller-tool result is continued and replayed through Messages', async function () {
  await configure(this, 'tool');
  const session = randomUUID(); this.messagesSessions.add(session);
  const initialBody = { ...payload(this, session, false), tools: [tool] };
  const first = await turn(this, '/v1/messages', session, false, initialBody);
  assert.equal(first.response.status, 200); assert.equal(first.response.value.stop_reason, 'tool_use');
  const call = first.response.value.content.find(block => block.type === 'tool_use');
  assert.equal(call.name, 'lookup_code'); assert.deepEqual(call.input, { key: 'alpha' });
  this.messagesCallerEffects = 0;
  const ownedLookup = input => {
    assert.deepEqual(input, { key: 'alpha' });
    this.messagesCallerEffects++;
    return 'OWNED-CODE-73';
  };
  const callerResult = ownedLookup(call.input);
  this.messagesContinuation = { ...initialBody, messages: [...initialBody.messages,
    { role: 'assistant', content: first.response.value.content },
    { role: 'user', content: [{ type: 'tool_result', tool_use_id: call.id, content: callerResult }] }] };
  await configure(this, 'nonzero');
  this.messagesContinued = await turn(this, '/v1/messages', session, false, this.messagesContinuation);
  assert.equal(this.messagesContinued.response.status, 200);
  this.messagesReplayBefore = await state(this);
  this.messagesReplayLedger = await ledger(this, this.messagesContinued.run);
  this.messagesReplay = await request(this, this.messagesBase, '/v1/messages', 'POST', this.messagesContinuation);
  this.messagesForeignReplay = await request(this, this.messagesForeign, '/v1/messages', 'POST', this.messagesContinuation);
});
Then('replay and the foreign owner cannot create another provider call or settlement', async function () {
  assert.equal(this.messagesReplay.status, 200);
  assert.deepEqual(this.messagesReplay.value, this.messagesContinued.response.value);
  assert.ok([400, 403, 409].includes(this.messagesForeignReplay.status));
  const after = await state(this); assert.equal(after.arrivals, this.messagesReplayBefore.arrivals);
  assert.equal(after.arrivals - this.messagesInitial.arrivals, 2);
  assert.equal(this.messagesCallerEffects, 1);
  assert.deepEqual(await ledger(this, this.messagesContinued.run), this.messagesReplayLedger);
});
After(async function ({ result }) {
  if (!this.messagesRecord) return;
  const cleanup = [];
  for (const runId of this.messagesRuns) {
    try {
      const run = await request(this, this.messagesBase, `/api/uar/runs/${encodeURIComponent(runId)}`);
      if (run.status === 200 && ['pending', 'running', 'paused'].includes(run.value.status)) {
        const cancel = await request(this, this.messagesBase, `/api/uar/runs/${encodeURIComponent(runId)}/cancel`, 'POST');
        assert.equal(cancel.status, 200);
      }
    } catch { cleanup.push('owned-run-cleanup-uncertain'); }
  }
  if (this.messagesPending) { try { await this.messagesPending; } catch { cleanup.push('owned-response-outcome-uncertain'); } }
  try {
    let peer;
    for (let attempt = 0; attempt < 25; attempt++) { peer = await state(this); if (!peer.active && !peer.held) break; await delay(100); }
    assert.equal(peer.active, 0); assert.equal(peer.held, 0); await configure(this, 'nonzero');
  } catch { cleanup.push('owned-provider-cleanup-uncertain'); }
  for (const session of this.messagesSessions) {
    try {
      const deleted = await request(this, this.messagesBase, `/api/uar/sessions/${encodeURIComponent(session)}`, 'DELETE');
      assert.ok([200, 204, 404].includes(deleted.status));
    } catch { cleanup.push('owned-session-cleanup-uncertain'); }
  }
  poisoned ||= cleanup.length > 0;
  this.messagesRecord.status = cleanup.length ? 'FAILED' : result.status; this.messagesRecord.cleanup = cleanup;
  await mkdir(dirname(this.messagesReceipt), { recursive: true });
  await writeFile(this.messagesReceipt, JSON.stringify(this.messagesRecord, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify({ evidence: this.messagesReceipt, status: this.messagesRecord.status }), 'application/json');
  assert.deepEqual(cleanup, [], 'Uncertain owned effects require external reconciliation; no retries');
});

When('I submit the unsupported Messages image form {string}', async function (variant) {
  await configure(this, 'nonzero');
  const session = randomUUID(); this.messagesSessions.add(session);
  const controlTurn = await turn(this, '/v1/messages', session, false, payload(this, session, false));
  assert.equal(controlTurn.response.status, 200);
  assert.equal(controlTurn.response.value.stop_reason, 'end_turn');
  this.messagesImageBefore = await state(this);
  const body = payload(this, randomUUID(), false);
  this.messagesSessions.add(body.session_id);
  const image = { type: 'image', source: { type: 'base64', media_type: 'image/png', data: '!!!!' } };
  body.messages[0].content = [{ type: 'text', text: prompt }, image];
  const edits = {
    'missing-source': () => { delete image.source; },
    'missing-data': () => { delete image.source.data; },
    'remote-url': () => { image.source = { type: 'url', url: 'http://127.0.0.1:9/private-image-sentinel' }; },
    'unsupported-mime': () => { image.source.media_type = 'image/gif'; },
    'invalid-base64': () => {},
    'extra-source': () => { image.source.private = 'private-image-sentinel'; },
    'unknown-block': () => { image.type = 'document'; },
    'assistant-image': () => { body.messages[0].role = 'assistant'; },
    'system-image': () => { body.system = [image]; body.messages[0].content = prompt; },
    'tool-result-image': () => { body.messages[0].content = [{ type: 'tool_result', tool_use_id: 'not-a-grant', content: [image] }]; },
  };
  assert.ok(Object.hasOwn(edits, variant)); edits[variant]();
  this.messagesImageExpected = variant === 'unsupported-mime'
    ? 'Images require inline base64 PNG or JPEG data URLs'
    : variant === 'invalid-base64' ? 'Image requires valid standard padded base64'
      : 'Unsupported or malformed Messages content block';
  this.messagesImageResult = await request(this, this.messagesBase, '/v1/messages', 'POST', body);
});
Then('the Messages image request refuses explicitly without a new provider arrival', async function () {
  const response = this.messagesImageResult;
  assert.equal(response.status, 400);
  assert.equal(response.value.error.message, this.messagesImageExpected);
  assert.ok(!response.raw.includes('private-image-sentinel'));
  assert.equal((await state(this)).arrivals, this.messagesImageBefore.arrivals);
});
