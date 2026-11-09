import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { performance } from 'node:perf_hooks';
import { Before, After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { initialize, configure, agent, content, image, text, send, summarize, compareParts,
  assertSuccess, assertRefused, finalWireAccounting, privacy, http, digest, parseJson, beginCase, beginCleanup, finishIO, requireLive,
  readOwnedFile, poisonFixture, isolationState } from '../support/capability-fixture.mjs';

setDefaultTimeout(180_000);

Before({ timeout: 180_000 }, function () { beginCase(this); });

Given('the owned capability-dispatch fixture manifest and external identity endpoint', async function () {
  await initialize(this);
  await configure(this);
});

Given('a disposable capability agent using {string} with no fallback', async function (primary) {
  await agent(this, primary);
});
Given('a disposable capability agent using {string} with fallback {string}', async function (primary, fallback) {
  await agent(this, primary, [fallback]);
});

When('I send {string} content to {string} with streaming {string}', async function (kind, route, stream) {
  assert.ok(['/v1/chat/completions', '/api/chat/completion'].includes(route));
  this.sent = content(this, kind);
  this.expectedParts = summarize(this.sent);
  await send(this, route, stream === 'true', this.sent);
});

function userMessages(observation) {
  assert.equal(observation.truncated, false, 'Observation must contain every message');
  return observation.messages.filter(message => message.role === 'user');
}

Then('the actual capable destination receives the exact ordered parts', function () {
  assertSuccess(this, this.last);
  const users = userMessages(this.last.observations.at(-1));
  assert.ok(users.length > 0, 'Actual request must contain the current user turn');
  this.actualParts = users.at(-1).parts;
  compareParts(this.actualParts, this.expectedParts);
});

Then('swapping the image hashes fails the same actual-request comparator', function () {
  const wrong = structuredClone(this.expectedParts);
  const images = wrong.filter(part => part.type === 'image_url');
  assert.equal(images.length, 2);
  assert.notEqual(images[0].sha256, images[1].sha256);
  [images[0].sha256, images[1].sha256] = [images[1].sha256, images[0].sha256];
  assert.throws(() => compareParts(this.actualParts, wrong), { name: 'AssertionError' });
  this.evidence.swapped_hash_control = 'REJECTED_BY_SAME_COMPARATOR';
});

Given('a valid control has reached that actual peer', async function () {
  const valid = content(this, 'mixed');
  const result = await send(this, '/v1/chat/completions', false, valid);
  assertSuccess(this, result);
  compareParts(userMessages(result.observations.at(-1)).at(-1).parts, summarize(valid));
});

When('I submit the invalid admission variant {string}', async function (variant) {
  let value = [text(`UAR_CASE:${this.case} ${this.privateText}`), image(this, 'png')];
  const extras = {};
  const url = next => { value[1].image_url.url = next; };
  if (variant === 'remote-url') url('https://fixture.invalid/private.png');
  else if (variant === 'relative-url') url('/private.png');
  else if (variant === 'file-url') url('file:///private.png');
  else if (variant === 'upload-id') value[1] = { type: 'image', file_id: 'owned-unresolved-upload' };
  else if (variant === 'mime') url(this.assets.png.url.replace('image/png', 'image/gif'));
  else if (variant === 'base64') url('data:image/png;base64,%%%invalid%%%');
  else if (variant === 'base64-padding') url(`data:image/png;base64,${this.assets.png.raw.subarray(0, 8).toString('base64').replace(/=+$/, '')}`);
  else if (variant === 'base64-url-alphabet') url('data:image/png;base64,-_==');
  else if (variant === 'base64-whitespace') url(`${this.assets.png.url}\n`);
  else if (variant === 'empty-base64') url('data:image/png;base64,');
  else if (variant === 'mime-signature') url(this.assets.jpeg.url.replace('image/jpeg', 'image/png'));
  else if (variant === 'header-only') url(`data:image/png;base64,${this.assets.png.raw.subarray(0, 8).toString('base64')}`);
  else if (variant === 'decoded-bounds') {
    const bytes = readOwnedFile(this, new URL('../fixtures/capability-dispatch/oversized-dimensions.png', import.meta.url));
    url(`data:image/png;base64,${bytes.toString('base64')}`);
  } else if (variant === 'detail') value[1].image_url.detail = 'ultra';
  else if (variant === 'unknown-part') value.push({ type: 'audio', input_audio: { data: 'AAAA', format: 'wav' } });
  else if (variant === 'excess-image-count') value.push(...Array.from({ length: 8 }, () => image(this, 'png')));
  else if (variant === 'envelope-bounds') value.unshift(text('x'.repeat(10 * 1024 * 1024)));
  else if (variant === 'image-attachment') extras.attachments = [{ id: 'owned-unresolved-upload', filename: 'synthetic.png', content_type: 'image/png', url: '/api/attachments/owned-unresolved-upload' }];
  else if (variant === 'ambiguous-message') extras.message = 'Ambiguous convenience turn';
  else if (variant === 'earlier-image') extras.messages = [{ role: 'user', content: value }, { role: 'assistant', content: 'previous' }, { role: 'user', content: 'current text only' }];
  else throw new Error('Unknown adopted invalid variant');
  await send(this, '/v1/chat/completions', false, value, extras);
  const codes = {
    'remote-url': 'unsupported_image_source', 'relative-url': 'unsupported_image_source',
    'file-url': 'unsupported_image_source', 'upload-id': 'invalid_content_part',
    mime: 'unsupported_image_source', base64: 'invalid_image_base64',
    'base64-padding': 'invalid_image_base64', 'base64-url-alphabet': 'invalid_image_base64',
    'base64-whitespace': 'invalid_image_base64', 'empty-base64': 'invalid_image_base64', 'mime-signature': 'image_mime_mismatch',
    'header-only': ['image_mime_mismatch', 'invalid_image_data'], 'decoded-bounds': 'invalid_image_data', detail: 'invalid_image_detail',
    'unknown-part': 'invalid_content_part', 'excess-image-count': 'too_many_images',
    'image-attachment': 'unsupported_image_attachment', 'ambiguous-message': 'ambiguous_message_input',
    'earlier-image': 'unsupported_multimodal_history',
  };
  if (variant === 'envelope-bounds') assert.equal(this.last.status, 413, 'Existing HTTP envelope limit must refuse');
  else {
    assert.equal(this.last.status, 400, 'Invalid admission must be a client error');
    assert.ok([].concat(codes[variant]).includes(this.last.errorCode), 'Admission must identify its stable safe error class');
  }
});

Then('the public operation refuses and every owned peer has zero new sends', function () {
  assertRefused(this.last);
});

When('I request the hard capability {string} from that exact target', async function (demand) {
  let value = content(this, demand === 'vision' ? 'mixed' : 'text');
  const extras = { model: this.primary.destination };
  if (demand === 'tools') {
    extras.tools = [{ type: 'function', function: { name: 'owned_fixture_lookup', description: 'Synthetic caller-owned lookup', parameters: { type: 'object', properties: {} } } }];
    extras.tool_choice = 'auto';
  } else if (demand === 'structured') {
    extras.response_format = { type: 'json_schema', json_schema: { name: 'owned_result', strict: true,
      schema: { type: 'object', properties: { result: { type: 'integer', enum: [42] } }, required: ['result'], additionalProperties: false } } };
    this.expectedFormat = extras.response_format;
    await configure(this, [], { result: 42 });
  } else if (demand === 'context') value = `UAR_CASE:${this.case} ${'uncompressible-context-fixture '.repeat(2048)}`;
  else assert.equal(demand, 'vision');
  this.expectedParts = summarize(value);
  await send(this, '/v1/chat/completions', false, value, extras);
});

Then('the actual destination receives the hard control {string}', function (demand) {
  assertSuccess(this, this.last);
  const observation = this.last.observations.at(-1);
  if (demand === 'tools') {
    assert.equal(observation.tool_count, 1);
    assert.equal(observation.tools[0].name_sha256, digest('owned_fixture_lookup'));
  } else if (demand === 'structured') {
    assert.equal(observation.format_type, 'json_schema');
    const canonical = value => Array.isArray(value) ? value.map(canonical) : value && typeof value === 'object'
      ? Object.fromEntries(Object.keys(value).sort().map(key => [key, canonical(value[key])])) : value;
    assert.equal(observation.format_sha256, digest(JSON.stringify(canonical(this.expectedFormat))), 'Exact structured schema must reach the actual wire');
    assert.equal(this.last.completion, '{"result":42}');
  } else compareParts(userMessages(observation).at(-1).parts, this.expectedParts);
});

When('actual controlled failures reach the configured cooldown threshold', async function () {
  const failover = this.fixture.failover;
  assert.equal(failover.enabled, true, 'Prerequisite: owned runtime failover is enabled');
  assert.ok(Number.isInteger(failover.error_threshold) && failover.error_threshold >= 1 && failover.error_threshold <= 8,
    'Prerequisite: bounded actual configured error threshold');
  await configure(this, [this.primary.peer]);
  for (let attempt = 0; attempt < failover.error_threshold; attempt += 1) {
    const result = await send(this, '/v1/chat/completions', false, content(this, 'text'), { model: this.primary.destination });
    assert.equal(result.failed, true);
    assert.ok(result.observations.some(row => row.peer === this.primary.peer), 'Failure must reach the actual dedicated provider before cooldown');
  }
  const health = parseJson((await http(this, 'GET', '/api/uar/providers/health', undefined, [200])).raw);
  const own = health.providers[this.primary.provider];
  assert.ok(own && own.healthy === false && own.cooldown_remaining_secs > 0, 'Actual health snapshot must record the owned provider cooldown');
  await configure(this);
});
Then('the next exact attempt refuses with no new peer sends', async function () {
  assertRefused(await send(this, '/v1/chat/completions', false, content(this, 'text'), { model: this.primary.destination }));
});

Given('the real global outside fallback serves an unrestricted control', async function () {
  assert.ok(this.fallbacks.length > 0, 'Restricted case must retain its nonempty captured artifact fallback list');
  assert.equal(this.evidence.global_outside_control, undefined, 'One unrestricted control per case is sufficient');
  const restricted = { agent: this.agent, primary: this.primary, fallbacks: this.fallbacks, session: this.session };
  const key = Object.entries(this.fixture.models).find(([, value]) => value.destination === this.primary.destination)?.[0];
  assert.ok(['failure_primary', 'failure_exact', 'failure_scope'].includes(key), 'Global controls use only dedicated one-case failure roles');
  const beforeHealth = parseJson((await http(this, 'GET', '/api/uar/providers/health', undefined, [200])).raw).providers[restricted.primary.provider];
  assert.ok(!beforeHealth || (beforeHealth.healthy && beforeHealth.consecutive_errors === 0), 'Dedicated control provider must start without prior failures');
  const controlSession = randomUUID();
  this.sessionIds.push(controlSession);
  this.session = controlSession;
  const value = content(this, 'mixed');
  try {
    // Empty artifact list intentionally inherits the already-configured real
    // global list for this control only. The original restricted agent survives.
    await agent(this, key, []);
    await configure(this, [this.primary.peer]);
    const control = await send(this, '/v1/chat/completions', false, value);
    assert.equal(control.failed, false, 'Configured global fallback must actually serve an unrestricted request');
    assert.ok(control.completion.startsWith('controlled-success:'));
    assert.equal(control.observations.filter(row => row.peer === restricted.primary.peer).length, 1, 'Unrestricted control must make exactly one forced primary attempt before global fallback');
    const delivered = control.observations.filter(row => row.peer === this.outside.peer);
    assert.ok(delivered.length > 0, 'Actual global outside peer must receive the fallback request');
    assert.ok(control.observations.every(row => [restricted.primary.peer, this.outside.peer].includes(row.peer)), 'Control must use exactly the primary and configured global outside peer');
    const globalRoute = this.evidence.effective_global_failover.fallback_models[0].model;
    // Pinned generic serialization strips only the provider prefix matching its
    // construction model_hint. Global config retains qualified routing identity.
    const globalWireModel = globalRoute.slice(globalRoute.indexOf('/') + 1);
    assert.equal(globalWireModel, this.outside.wireModel ?? this.outside.modelId, 'Global generic wire alias must match the frozen outside model');
    for (const row of delivered) {
      assert.equal(row.model_sha256, digest(globalWireModel), 'Actual global-control wire model must preserve the pinned generic normalization');
      compareParts(userMessages(row).at(-1).parts, summarize(value));
    }
    const health = parseJson((await http(this, 'GET', '/api/uar/providers/health', undefined, [200])).raw).providers[restricted.primary.provider];
    assert.ok(health && health.healthy && health.consecutive_errors === 1
      && health.consecutive_errors < this.fixture.failover.error_threshold, 'Exactly one control failure must remain below cooldown before the restricted attempt');
    this.evidence.global_outside_control = { actual_peer: this.outside.peer,
      configured_destination: this.outside.destination, global_routing_model: globalRoute,
      global_wire_model_sha256: digest(globalWireModel), observed_calls: delivered.length,
      control_agent_id: this.agent, control_session_id: controlSession,
      restricted_agent_id: restricted.agent, restricted_session_id: restricted.session,
      primary_errors_before_restricted_attempt: health.consecutive_errors };
  } finally { Object.assign(this, restricted); }
  await configure(this);
});

When('the controlled primary fails before output on a mixed image turn', async function () {
  assert.equal(this.fixture.failover.enabled, true, 'Prerequisite: failover enabled in the owned runtime');
  await configure(this, [this.primary.peer]);
  this.sent = content(this, 'mixed');
  this.expectedParts = summarize(this.sent);
  await send(this, '/v1/chat/completions', false, this.sent);
});

Then('only the captured capable fallback succeeds with its own settings and images', function () {
  assert.equal(this.last.failed, false);
  assert.ok(this.last.completion.startsWith('controlled-success:'));
  const fallback = this.fallbacks[0];
  const rows = this.last.observations;
  assert.ok(rows.some(row => row.peer === this.primary.peer), 'Primary must actually receive the pre-output failing attempt');
  const delivered = rows.filter(row => row.peer === fallback.peer);
  assert.ok(delivered.length > 0, 'Captured fallback must actually receive the final request');
  assert.equal(rows.filter(row => row.peer === this.outside.peer).length, 0, 'Previously exercised real GLOBAL fallback must receive zero restricted-case calls');
  assert.ok(rows.every(row => [this.primary.peer, fallback.peer].includes(row.peer)), 'Outside peer must receive zero calls');
  for (const row of delivered) {
    assert.equal(row.model_sha256, digest(fallback.wireModel ?? fallback.modelId));
    compareParts(userMessages(row).at(-1).parts, this.expectedParts);
    const cap = row.max_completion_tokens ?? row.max_tokens;
    assert.ok(Number.isInteger(cap) && cap > 0 && cap <= Math.min(64, fallback.max_output_tokens), 'Fallback own output limit must survive preparation');
    assert.ok(fallback.wireSettings && Object.keys(fallback.wireSettings).length > 0, 'Prerequisite: frozen independently provisioned fallback wire settings');
    assert.ok(fallback.max_output_tokens < 64, 'Fixture fallback own output ceiling must differ from the requested primary ceiling');
    for (const [field, expected] of Object.entries(fallback.wireSettings)) {
      assert.ok(['temperature', 'max_tokens', 'max_completion_tokens'].includes(field), 'Only observed safe settings are admitted');
      assert.equal(row[field], expected, 'Fallback setting must match its own frozen configuration');
    }
  }
});

When('the primary attempt is {string} on a mixed image turn', async function (selection) {
  await configure(this, [this.primary.peer]);
  await send(this, '/v1/chat/completions', false, content(this, 'mixed'), selection === 'exact' ? { model: this.primary.destination } : {});
});
Then('the request fails without sending to the forbidden fallback or outside peer', function () {
  assert.equal(this.last.failed, true);
  assert.ok(this.last.observations.some(row => row.peer === this.primary.peer), 'Failure must originate at the actual primary');
  assert.equal(this.last.observations.filter(row => row.peer === this.outside.peer).length, 0, 'Previously exercised real GLOBAL fallback must stay outside the captured nonempty policy');
  assert.ok(this.last.observations.every(row => row.peer === this.primary.peer), 'No fallback or out-of-scope send is permitted');
});

When('an image turn is followed by fresh text in the same owned session', async function () {
  const first = content(this, 'mixed');
  this.historyParts = summarize(first);
  assertSuccess(this, await send(this, '/v1/chat/completions', false, first));
  const second = content(this, 'text');
  assert.ok(!JSON.stringify(second).includes('data:image/'), 'Follow-up must contain no image replay');
  await send(this, '/v1/chat/completions', false, second);
});
Then('the second actual request retains the earlier ordered image hashes', function () {
  assertSuccess(this, this.last);
  const users = userMessages(this.last.observations.at(-1));
  assert.ok(users.length >= 2, 'Warm continuation must retain canonical prior user turn');
  compareParts(users.at(-2).parts, this.historyParts);
  assert.equal(users.at(-1).parts.some(part => part.type === 'image_url'), false);
  const erased = this.historyParts.filter(part => part.type !== 'image_url');
  assert.throws(() => compareParts(users.at(-2).parts, erased), { name: 'AssertionError' });
});

Given('a finite-cost disposable capability agent can send an ordinary text control', async function () {
  await agent(this, 'capable', [], { finiteCost: 1 });
  // This real control excludes malformed/blanket finite-policy refusal. External
  // provisioning must supply usable source-supported price/accounting facts.
  assertSuccess(this, await send(this, '/v1/chat/completions', false, content(this, 'text')));
});
When('I send an admitted image under that same finite-cost policy', async function () {
  await send(this, '/v1/chat/completions', false, content(this, 'mixed'));
});

function assertEstimatedAccounting(facts, observations, expected) {
  const observed = observations.filter(row => row.peer === expected.peer && row.model_sha256 === digest(expected.wireModel));
  assert.ok(observed.length > 0 && observed.some(row => row.messages.some(message =>
    message.parts.some(part => part.type === 'image_url'))), 'Accounting must accompany an actual image-bearing destination request');
  assert.ok(facts.final.length > 0, 'Actual final-provider request budget receipt is required; attempt manifests are insufficient');
  for (const receipt of facts.final) {
    assert.equal(receipt.destination_model, expected.destination, 'Final accounting must identify the exact admitted destination');
    assert.equal(receipt.representation, 'final_provider_request');
    assert.equal(receipt.serialized_request, true);
    assert.equal(receipt.count_quality, 'approximate');
    assert.equal(receipt.counting_revision, 'approximate-cl100k-json-v1');
    assert.equal(receipt.fit_guarantee, false);
    assert.equal(receipt.reserved_output_tokens, null, 'Estimated lane must not claim an installed exact budget contract');
    assert.equal(receipt.output_ceiling_enforced, true);
    assert.equal(receipt.tools_included, true);
    assert.equal(receipt.instructions_included, true);
    assert.ok(Number.isInteger(receipt.counted_input_tokens) && receipt.counted_input_tokens > 0);
    assert.ok(Number.isInteger(receipt.serialized_bytes) && receipt.serialized_bytes > 0);
    assert.match(receipt.serialized_sha256, /^[0-9a-f]{64}$/);
  }
  for (const attempt of facts.attempts) assert.equal(attempt.budgeting?.fit_guarantee, false);
}

Then('the owned run events report estimated image accounting without a fit guarantee', async function () {
  const runs = parseJson((await http(this, 'GET', '/api/uar/runs', undefined, [200])).raw);
  const run = runs.find(entry => entry.conversation_id === this.session);
  assert.ok(run, 'Actual public run must exist');
  const events = parseJson((await http(this, 'GET', `/api/uar/runs/${encodeURIComponent(run.run_id)}/events`, undefined, [200])).raw);
  assert.equal(events.gapReason, null, 'Diagnostic receipt cannot rely on a retention gap');
  const facts = finalWireAccounting(events.events, run.run_id);
  const expected = { peer: this.primary.peer, destination: this.primary.destination,
    wireModel: this.primary.wireModel ?? this.primary.modelId };
  assertEstimatedAccounting(facts, this.last.observations, expected);
  assert.throws(() => assertEstimatedAccounting({ final: [], attempts: facts.attempts }, this.last.observations, expected),
    { name: 'AssertionError' }, 'Removing only actual final-wire receipts must fail the same comparator');
  this.evidence.accounting = { final_receipt_count: facts.final.length, attempt_count: facts.attempts.length,
    destination_model_sha256: digest(expected.destination), wire_model_sha256: digest(expected.wireModel), count_quality: 'approximate', fit_guarantee: false,
    missing_final_receipt_control: 'REJECTED_BY_SAME_COMPARATOR' };
});
Then('diagnostic sentinels are excluded by a sensitive privacy comparator', function () {
  assert.ok(this.evidence.exchanges.every(exchange => exchange.diagnostics_safe));
  const planted = JSON.stringify({ code: 'deliberate-isolated-comparator-control', image: this.assets.png.url });
  assert.equal(privacy(this, planted), false, 'Same diagnostics comparator must reject a planted actual inline fixture');
  assert.equal(privacy(this, this.privateText), false, 'Private text sentinel comparator must be sensitive');
  this.evidence.privacy_control = 'REJECTED_BY_SAME_COMPARATOR';
});

After({ timeout: 180_000 }, async function ({ result, pickle }) {
  const hookDeadline = performance.now() + 180_000;
  if (this.caseIO?.pending.size) poisonFixture('case-client-work-still-pending-at-cleanup');
  // Closing the case signal and joining all client requests/body reads precedes
  // cleanup. The expired case signal is never replaced by cleanup's fresh one.
  await finishIO(this.caseIO);
  if (!this.evidence) return;
  const ownedIds = [...(this.agentIds ?? [])];
  const ownedSessions = [...(this.sessionIds ?? [this.session])];
  const path = resolve(`.prometheus/verification/capability-dispatch/${this.id}.json`);
  const ownershipPath = resolve(`.prometheus/verification/capability-dispatch/${this.id}.ownership.json`);
  const ownership = {
    schema: 'uar.capability-dispatch.ownership/v1', case: this.case,
    scenario_ids: pickle.tags.map(tag => tag.name).filter(tag => /^@CD-\d+$/.test(tag)),
    identity: this.evidence.identity, fixture_manifest_sha256: this.evidence.fixture_manifest_sha256,
    effective_global_failover: this.evidence.effective_global_failover,
    global_failover_setting_revisions: this.evidence.global_failover_setting_revisions,
    effective_retry_settings: this.evidence.effective_retry_settings,
    global_outside_control: this.evidence.global_outside_control ?? null,
    fixture_namespace: this.fixture.namespace, workspace_id: this.workspace,
    owned_agent_ids: ownedIds, owned_session_id: this.session, owned_session_ids: ownedSessions,
    disposition: 'CLEANUP_PENDING_RECONCILIATION_REQUIRED_UNTIL_TERMINAL_RECEIPT',
    isolation_at_cleanup_start: isolationState(), planned_terminal_receipt: path,
  };
  // Persist before starting cleanup's deadline or issuing any DELETE/control.
  // This exclusive record stays immutable even if cleanup cannot publish a result.
  try {
    mkdirSync(dirname(ownershipPath), { recursive: true });
    writeFileSync(ownershipPath, JSON.stringify(ownership, null, 2) + '\n',
      { flag: 'wx', mode: 0o600, flush: true });
    this.attach(JSON.stringify({ ownership_record: ownershipPath, disposition: ownership.disposition }), 'application/json');
  } catch {
    poisonFixture('ownership-record-persistence-failed');
    this.attach(JSON.stringify({ ownership_record_unwritten: true, ownership }), 'application/json');
    throw new Error('Ownership record could not be persisted; cleanup was not started and owned effects require reconciliation');
  }
  // A slow synchronous ownership write must not start a fresh100second remote
  // cleanup budget after consuming the outer hook's remaining time.
  if (performance.now() + 100_000 >= hookDeadline) {
    poisonFixture('insufficient-hook-time-for-owned-cleanup');
    throw new Error('Ownership retained; cleanup was not started because its complete bounded lifetime no longer fits the hook');
  }
  const cleanup = beginCleanup(this);
  const failures = [];
  const confirmedDeletes = [];
  const confirmedSessionDeletes = [];
  try {
    for (const id of ownedIds) {
      try {
        await http(this, 'DELETE', `/api/agents/${encodeURIComponent(id)}`, undefined, [204, 404], cleanup);
        confirmedDeletes.push(id);
      } catch { failures.push('owned-agent-cleanup-failed'); poisonFixture('owned-agent-cleanup-failed'); }
    }
    for (const session of ownedSessions) {
      try {
        await http(this, 'DELETE', `/api/uar/sessions/${encodeURIComponent(session)}`, undefined, [204, 404], cleanup);
        confirmedSessionDeletes.push(session);
      } catch { failures.push('owned-session-cleanup-failed'); poisonFixture('owned-session-cleanup-failed'); }
    }
    if (isolationState().poisoned) failures.push('peer-reset-skipped-uncertain-prior-effect');
    else {
      try { await configure(this, [], undefined, cleanup); }
      catch { failures.push('owned-peer-reset-failed'); poisonFixture('owned-peer-reset-failed'); }
    }
    this.evidence.scenario_ids = pickle.tags.map(tag => tag.name).filter(tag => /^@CD-\d+$/.test(tag));
    this.evidence.isolation = isolationState();
    this.evidence.ownership_record = ownershipPath;
    this.evidence.owned_agent_ids = ownedIds;
    this.evidence.confirmed_delete_responses = confirmedDeletes;
    this.evidence.unresolved_agent_ids = isolationState().poisoned ? ownedIds : ownedIds.filter(id => !confirmedDeletes.includes(id));
    this.evidence.owned_session_id = this.session;
    this.evidence.owned_session_ids = ownedSessions;
    this.evidence.confirmed_session_delete_responses = confirmedSessionDeletes;
    this.evidence.unresolved_session_ids = isolationState().poisoned ? ownedSessions : ownedSessions.filter(session => !confirmedSessionDeletes.includes(session));
    this.evidence.unresolved_session_id = this.evidence.unresolved_session_ids.includes(this.session) ? this.session : null;
    this.evidence.result = failures.length || isolationState().poisoned || result.status === 'FAILED' ? 'FAIL' : result.status === 'PASSED' ? 'PASS' : 'NOT_EXERCISED';
    this.evidence.cleanup_failures = failures;
    // Receipt filesystem operations are synchronous: unlike best-effort aborted
    // fsPromises, they cannot survive this hook and race a subsequent case.
    requireLive(cleanup);
    mkdirSync(dirname(path), { recursive: true });
    requireLive(cleanup);
    writeFileSync(path, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
    requireLive(cleanup);
    this.attach(JSON.stringify({ evidence: path, result: this.evidence.result }), 'application/json');
    assert.equal(failures.length, 0, 'Owned fixture cleanup must complete; no uncertain retry');
    assert.equal(isolationState().poisoned, false, 'Owned effects require reconciliation before any subsequent fixture case');
  } catch (error) {
    poisonFixture('owned-cleanup-or-receipt-incomplete');
    throw error;
  } finally { await finishIO(cleanup); }
});

When('I send a text attachment before the literal current turn', async function () {
  const message = content(this, 'literal-json');
  const attachment = { id: `${this.fixture.namespace}-text`, filename: 'owned-note.txt',
    content_type: 'text/plain', url: '/owned-unresolved-note', text_content: 'synthetic attachment text' };
  this.expectedParts = summarize([text('[owned-note.txt]\nsynthetic attachment text'), text(message)]);
  await send(this, '/api/chat/completion', false, message, { attachments: [attachment] });
});

When('I submit a {string} convenience message with text-only messages to {string}', async function (variant, route) {
  assert.ok(['nonblank', 'blank'].includes(variant), 'Only adopted text convenience cases are allowed');
  assert.ok(['/v1/chat/completions', '/api/chat/completion'].includes(route));
  const last = `UAR_CASE:${this.case} last current text`;
  const convenience = variant === 'nonblank' ? `UAR_CASE:${this.case} convenience wins` : ' \n ';
  this.expectedParts = summarize(variant === 'nonblank' ? convenience : last);
  await send(this, route, false, last, { message: convenience, messages: [
    { role: 'user', content: 'earlier text' },
    { role: 'assistant', content: 'previous text response' },
    { role: 'user', content: last },
  ] });
});
