import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { createHash, randomUUID } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { documentFixtures } from '../../../tests/fixtures/production-readiness-documents/fixtures.mjs';
import { qualifyA2A, qualifyA2ACancellation } from '../../acceptance/fixtures/protocol-qualification/a2a-client.mjs';
import { loadCandidateArtifactInputs, captureCandidateArtifact, assertCandidateArtifactStable, candidateSettingKeys, canonical } from '../support/candidate-artifact-binding.mjs';

setDefaultTimeout(420_000);
const KB = '/api/uar/knowledge-bases';
const terminal = new Set(['done', 'completed', 'error', 'failed', 'cancelled']);
const digest = bytes => createHash('sha256').update(bytes).digest('hex');

async function setup(w, stopped = false, requireReady = true) {
  w.caseId = process.env.UAR_QUALIFICATION_CASE_ID;
  assert.match(w.caseId ?? '', /^[a-z0-9][a-z0-9-]{7,63}$/);
  w.workspace = process.env.UAR_WORKSPACE_ID;
  assert.ok(w.workspace, 'Externally provisioned nonsecret workspace required');
  assert.match(process.env.UAR_SOURCE_SHA ?? '', /^[a-f0-9]{40}$/);
  w.base = new URL(process.env.UAR_BASE_URL);
  assert.ok(['http:', 'https:'].includes(w.base.protocol) && !w.base.username && !w.base.password && !w.base.search);
  w.directory = resolve('target/production-artifact-qualification', w.caseId);
  await mkdir(w.directory, { recursive: true });
  w.statePath = resolve(w.directory, 'state.json');
  w.evidence = { kind: 'uar.production-artifact-qualification/v1', sourceCase: 'scripts/behaviors/features/production-artifact-qualification.feature',
    case_id: w.caseId, authentication_supplied_by_test: false, host_identity_setup: false, exchanges: [], observations: [], reliability: {},
    source_sha: process.env.UAR_SOURCE_SHA, base_url: w.base.href, workspace: w.workspace };
  w.image = inspectOwned(w);
  assert.equal(w.image.running, !stopped, 'Exact owned container must be in the selected stage');
  if (!stopped && requireReady) await request(w, 'GET', '/readyz');
}

async function binding(w) {
  w.bindingInputs ??= await loadCandidateArtifactInputs(process.env.UAR_CANDIDATE_ARTIFACT_BINDING);
  const settings = {};
  for (const key of candidateSettingKeys) settings[key] = (await request(w, 'GET', `/api/uar/settings/${key}`)).effective;
  return captureCandidateArtifact(w.bindingInputs, { container: inspectOwned(w), platform: { os: 'linux', architecture: 'amd64' },
    settings, baseUrl: w.base.href, workspace: w.workspace, expectRunning: true });
}
Given('the clean release candidate identity is bound for measurement', async function () {
  this.evidence.candidateBinding = { before: await binding(this) };
  this.evidence.identity = this.evidence.candidateBinding.before.identity;
});

function inspectOwned(w) {
  const container = process.env.UAR_ARTIFACT_CONTAINER;
  assert.match(container ?? '', /^uar-pr476-[a-z0-9-]+$/);
  // Projection intentionally excludes Config.Env and all credential/config bytes.
  const projection = '{"id":{{json .Id}},"image":{{json .Image}},"labels":{{json .Config.Labels}},"running":{{json .State.Running}},"started_at":{{json .State.StartedAt}},"finished_at":{{json .State.FinishedAt}},"exit_code":{{json .State.ExitCode}},"mounts":{{json .Mounts}}}';
  const value = JSON.parse(execFileSync('docker', ['inspect', '--format', projection, container], { encoding: 'utf8', timeout: 15_000 }));
  assert.equal(value.labels?.['dev.prometheus.qualification.case'], w.caseId, 'Container ownership must match this case');
  assert.equal(value.labels['dev.prometheus.qualification.base-url'], w.base.href, 'Identity endpoint must belong to this owned container fixture');
  assert.equal(value.labels['dev.prometheus.qualification.workspace'], w.workspace);
  assert.match(value.image, /^sha256:[a-f0-9]{64}$/);
  const platform = JSON.parse(execFileSync('docker', ['image', 'inspect', '--format', '{"os":{{json .Os}},"architecture":{{json .Architecture}}}', value.image], { encoding: 'utf8', timeout: 15_000 }));
  assert.deepEqual(platform, { os: 'linux', architecture: 'amd64' });
  const data = value.mounts.filter(mount => mount.Destination === '/var/lib/uar');
  assert.equal(data.length, 1);
  assert.equal(data[0].Type, 'volume', 'Use an explicitly owned data volume');
  assert.equal(data[0].Name, process.env.UAR_ARTIFACT_VOLUME);
  assert.ok(data[0].Name?.startsWith(`uar-pr476-${w.caseId}-`));
  const volumeLabels = JSON.parse(execFileSync('docker', ['volume', 'inspect', '--format', '{{json .Labels}}', data[0].Name], { encoding: 'utf8', timeout: 15_000 }));
  assert.equal(volumeLabels?.['dev.prometheus.qualification.case'], w.caseId);
  assert.equal(value.labels['org.opencontainers.image.revision'], process.env.UAR_SOURCE_SHA);
  value.volume = data[0].Name;
  delete value.mounts;
  w.evidence.observations.push({ container, ...value, platform });
  return value;
}

async function request(w, method, path, payload, expected = [200]) {
  const multipart = payload instanceof FormData;
  const response = await fetch(new URL(path, w.base), { method, redirect: 'error', signal: AbortSignal.timeout(120_000),
    headers: { 'x-uar-workspace-id': w.workspace, ...(path.startsWith('/a2a/v1/') && { 'a2a-version': '1.0' }), ...(!multipart && { 'Content-Type': 'application/json' }) },
    body: payload === undefined ? undefined : multipart ? payload : JSON.stringify(payload) });
  const text = await response.text();
  w.evidence.exchanges.push({ method, path, status: response.status, response_sha256: digest(text) });
  assert.ok(expected.includes(response.status), `${method} ${path}: HTTP ${response.status}`);
  return text ? response.headers.get('content-type')?.includes('application/json') ? JSON.parse(text) : text : null;
}

async function newKb(w, label) {
  const kb = await request(w, 'POST', KB, { name: `PR476 ${w.caseId} ${label}`, description: 'Owned synthetic qualification data',
    config: { chunk_strategy: 'fixed', chunk_size: 512, retrieval_min_score: 0, retrieval_top_k: 20 } }, [201]);
  assert.ok(kb.id); return `${KB}/${encodeURIComponent(kb.id)}`;
}

async function upload(w, path, fixture, allowFailed = false) {
  const form = new FormData(); form.append('file', new Blob([fixture.bytes], { type: fixture.type }), fixture.name);
  const response = await request(w, 'POST', `${path}/documents`, form, allowFailed ? [202, 400, 422] : [202]);
  if (!response?.id) { assert.ok(allowFailed); return { refused: true }; }
  const deadline = Date.now() + 180_000;
  while (Date.now() < deadline) {
    const document = await request(w, 'GET', `${path}/documents/${encodeURIComponent(response.id)}`);
    if (document.status === 'failed') { assert.ok(allowFailed, `${fixture.name} extraction failed`); return document; }
    if (document.status === 'indexed') { assert.equal(allowFailed, false, 'Malformed input must not index'); assert.ok(document.chunk_count > 0); return document; }
    await delay(500);
  }
  throw new Error(`${fixture.name} ingestion did not reach an honest terminal outcome`);
}

async function state(w) {
  const saved = JSON.parse(await readFile(w.statePath, 'utf8'));
  assert.equal(saved.case_id, w.caseId); assert.equal(saved.workspace, w.workspace); assert.equal(saved.base_url, w.base.href);
  return saved;
}
async function save(w, saved, create = false) {
  await writeFile(w.statePath, JSON.stringify(saved, null, 2) + '\n', { flag: create ? 'wx' : 'w', mode: 0o600 });
}
async function canary(w, saved) {
  const document = await request(w, 'GET', `${saved.kb_path}/documents/${encodeURIComponent(saved.document.id)}`);
  assert.equal(document.status, 'indexed');
  const result = await request(w, 'POST', `${saved.kb_path}/search`, { query: 'What qualification seal does the retained canary record?', limit: 20, min_score: 0 });
  const own = result.results.filter(row => row.document_id === saved.document.id);
  assert.ok(own.some(row => row.content.includes(saved.marker)));
  return { id: document.id, contentSha256: digest(own.map(row => row.content).sort().join('\n')) };
}

Given('the exact owned Linux API artifact is externally provisioned', async function () { await setup(this); });
Given('the owned Linux container is stopped externally', async function () { await setup(this, true); });

When('I ingest the five tiny qualification document formats', async function () {
  this.cleanupKbs = []; this.documents = [];
  for (const fixture of documentFixtures()) {
    const path = await newKb(this, fixture.name); this.cleanupKbs.push(path);
    const document = await upload(this, path, fixture);
    this.documents.push({ path, fixture, document });
    this.evidence.observations.push({ format: fixture.name, mime: fixture.type, source_sha256: digest(fixture.bytes), document });
  }
});
Then('each format retrieves its own fact with document provenance', async function () {
  for (const { path, fixture, document } of this.documents) {
    const found = await request(this, 'POST', `${path}/search`, { query: 'What fact does this qualification document contain?', limit: 20, min_score: 0 });
    const own = found.results.filter(row => row.document_id === document.id);
    const content = own.map(row => row.content).join('\n');
    assert.ok(own.length > 0, `${fixture.name} provenance missing`);
    for (const expected of fixture.expected) assert.ok(content.includes(expected), `${fixture.name} expected fact absent`);
    this.evidence.observations.push({ format: fixture.name, retrieved_document_id: document.id, expected: fixture.expected, retrieved: content });
  }
});
Then('a malformed Office input is refused without a searchable fact', async function () {
  const path = await newKb(this, 'malformed'); this.cleanupKbs.push(path);
  const marker = `MALFORMED-${randomUUID()}`;
  const result = await upload(this, path, { name: 'corrupt.docx', type: 'application/vnd.openxmlformats-officedocument.wordprocessingml.document', bytes: Buffer.from(marker) }, true);
  assert.ok(result.refused || result.status === 'failed');
  const found = await request(this, 'POST', `${path}/search`, { query: marker, limit: 20, min_score: 0 });
  assert.ok(found.results.every(row => !row.content.includes(marker)));
});

When('I save the artifact lifecycle canary', async function () {
  const saved = { case_id: this.caseId, workspace: this.workspace, base_url: this.base.href, prior: this.image,
    marker: `PR476-${randomUUID()}` };
  await save(this, saved, true);
  saved.kb_path = await newKb(this, 'lifecycle'); await save(this, saved);
  saved.document = await upload(this, saved.kb_path, { name: 'retained.txt', type: 'text/plain', bytes: Buffer.from(`The qualification seal is ${saved.marker}.`) });
  saved.candidateBinding = this.evidence.candidateBinding;
  saved.canaryBefore = await canary(this, saved);
  const historyInput = `PR476-HISTORY-RESTART-${randomUUID()} private synthetic restart fact ${randomUUID()}`;
  const session = randomUUID(); const before = await reliabilityRun(this, historyInput, session);
  assert.equal(before.status, 'done');
  saved.conversation = { session, agent: this.reliabilityAgent, agentDigest: this.frozenAgentDigest, profile: this.frozenRunProfile,
    input: historyInput, inputSha256: digest(historyInput), before };
  // This explicitly owned retained fixture crosses an external process transition.
  // Cleanup is transferred to the observe-restart stage, never silently dropped.
  this.cleanupAgent = undefined; this.cleanupSessions = this.cleanupSessions.filter(id => id !== session);
  await save(this, saved);
});
Then('the retained canary and exact prior artifact are recorded', async function () { const saved = await state(this); await canary(this, saved); this.evidence.observations.push({ retained: saved.document.id, prior_image: saved.prior.image }); });
Then('its observed clean exit and retained mount are recorded', async function () {
  const saved = await state(this); assert.equal(this.image.image, saved.prior.image); assert.equal(this.image.volume, saved.prior.volume);
  assert.equal(this.image.exit_code, 0); assert.ok(Date.parse(this.image.finished_at) > Date.parse(saved.prior.started_at));
  saved.stopped = this.image; await save(this, saved);
});
Then('the same artifact restarted after the captured clean stop and retains the canary', async function () {
  const saved = await state(this); assert.ok(saved.stopped, 'Actual stopped stage required');
  assert.equal(this.image.image, saved.prior.image); assert.equal(this.image.volume, saved.prior.volume);
  assert.ok(Date.parse(this.image.started_at) > Date.parse(saved.stopped.finished_at));
  const after = await canary(this, saved); saved.restarted = this.image;
  assert.ok(saved.conversation, 'Actual pre-restart retained conversation is required');
  this.reliabilityAgent = saved.conversation.agent; this.cleanupAgent = saved.conversation.agent;
  this.frozenAgentDigest = saved.conversation.agentDigest; this.frozenRunProfile = saved.conversation.profile;
  const input = `PR476-HISTORY-RECALL-${randomUUID()} recall retained synthetic inputs`;
  const recalled = await reliabilityRun(this, input, saved.conversation.session); assert.equal(recalled.status, 'done');
  assert.ok(eventText(await runEvents(this, recalled.runId)).includes(saved.conversation.input), 'Current input does not supply the pre-restart private fact');
  const observed = (await modelObservations(this)).filter(row => input.startsWith(row.marker));
  assert.equal(observed.length, 1); assert.ok(observed[0].messageSha256.includes(saved.conversation.inputSha256));
  this.evidence.observations.push({ retained_conversation: saved.conversation.session, original_run: saved.conversation.before.runId,
    recalled_run: recalled.runId, original_input_sha256: saved.conversation.inputSha256, provider: observed });
  if (saved.candidateBinding) {
    const current = await binding(this);
    assertCandidateArtifactStable(saved.candidateBinding.before, current, { allowRestart: true });
    assert.deepEqual(after, saved.canaryBefore);
    this.evidence.candidateBinding = { before: saved.candidateBinding.before, after: current };
    this.evidence.identity = current.identity;
    this.evidence.reliability.controlledRestart = { prior: saved.prior, stopped: saved.stopped, restarted: saved.restarted,
      canaryBefore: saved.canaryBefore, canaryAfter: after };
  }
  await save(this, saved);
});
Then('a distinct current artifact retains the prior canary and volume', async function () {
  const saved = await state(this); assert.notEqual(this.image.image, saved.prior.image, 'Same-image restart cannot prove upgrade');
  assert.notEqual(this.image.labels['org.opencontainers.image.revision'], saved.prior.labels['org.opencontainers.image.revision']);
  assert.equal(this.image.volume, saved.prior.volume); await canary(this, saved); saved.upgraded = this.image; await save(this, saved);
});
When('I bind the owned backup and delete the lifecycle canary', async function () {
  const saved = await state(this); assert.ok(saved.upgraded, 'Upgrade stage must precede backup/restore');
  saved.backup_sha256 = digest(await readFile(resolve(this.directory, 'backup.tar')));
  assert.ok(saved.backup_sha256); await canary(this, saved);
  await request(this, 'DELETE', `${saved.kb_path}/documents/${encodeURIComponent(saved.document.id)}`, undefined, [204]);
  saved.deleted_after_backup = true; saved.before_restore = this.image; await save(this, saved);
});
Then('the deleted canary is absent before restore', async function () {
  const saved = await state(this);
  await request(this, 'GET', `${saved.kb_path}/documents/${encodeURIComponent(saved.document.id)}`, undefined, [404]);
  const found = await request(this, 'POST', `${saved.kb_path}/search`, { query: saved.marker, limit: 20, min_score: 0 });
  assert.ok(found.results.every(row => row.document_id !== saved.document.id && !row.content.includes(saved.marker)));
  saved.absence_observed = true; await save(this, saved);
});
Then('the bound backup is unchanged and the deleted canary has returned', async function () {
  const saved = await state(this); assert.equal(saved.absence_observed, true);
  assert.equal(digest(await readFile(resolve(this.directory, 'backup.tar'))), saved.backup_sha256);
  assert.equal(this.image.image, saved.upgraded.image); assert.equal(this.image.volume, saved.prior.volume);
  assert.ok(Date.parse(this.image.started_at) > Date.parse(saved.before_restore.started_at));
  await canary(this, saved); saved.restored = this.image; await save(this, saved);
});
Then('the original artifact retains the restored canary after rollback', async function () {
  const saved = await state(this); assert.ok(saved.restored);
  assert.equal(this.image.image, saved.prior.image); assert.equal(this.image.volume, saved.prior.volume);
  await canary(this, saved); saved.rolled_back = this.image;
  await request(this, 'DELETE', saved.kb_path, undefined, [204]); saved.canary_cleanup = true; await save(this, saved);
});

When('the official pinned A2A client uses the advertised JSONRPC streaming interface', async function () {
  const card = await request(this, 'GET', '/.well-known/agent-card.json');
  assert.equal(card.capabilities.streaming, true);
  assert.ok(card.supportedInterfaces.some(entry => entry.protocolBinding === 'JSONRPC' && entry.protocolVersion === '1.0'));
  const marker = `PR476-${randomUUID()}`;
  this.a2a = await qualifyA2A({ url: this.base.href, workspace: this.workspace, text: `Reply with exactly ${marker}.`, messageId: randomUUID() });
  assert.ok(JSON.stringify(this.a2a.events).includes(marker), 'Independent decoded stream must contain the requested synthetic answer');
  this.evidence.observations.push({ independent_a2a: this.a2a, advertised_card: card });
});
Then('it observes a completed task and unsupported A2A methods fail explicitly', async function () {
  assert.ok(this.a2a.status);
  const unsupported = await request(this, 'POST', '/a2a/v1/compiler', { jsonrpc: '2.0', id: randomUUID(), method: 'QualificationUnsupportedMethod', params: {} }, [200, 400]);
  assert.equal(unsupported.error?.code, -32601);
  const response = await fetch(new URL('/a2a/v1/compiler', this.base), { method: 'POST', redirect: 'error', signal: AbortSignal.timeout(15_000),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': this.workspace, 'a2a-version': '999.0' },
    body: JSON.stringify({ jsonrpc: '2.0', id: randomUUID(), method: 'QualificationUnsupportedMethod', params: {} }) });
  const refused = await response.json();
  assert.equal(refused.error?.code, -32009);
  this.evidence.observations.push({ unsupported_version_status: response.status, unsupported_version_code: refused.error.code });
});

Given('the official MCP peer and scenario-scoped tool policy are externally provisioned', async function () {
  this.peerUrl = new URL(process.env.UAR_QUALIFICATION_MCP_URL);
  assert.equal(this.peerUrl.protocol, 'http:'); assert.ok(['127.0.0.1', 'localhost', '[::1]'].includes(this.peerUrl.hostname));
  assert.ok(!this.peerUrl.username && !this.peerUrl.password && !this.peerUrl.search);
  const rpc = async (method, params) => {
    const response = await fetch(this.peerUrl, { method: 'POST', redirect: 'error', signal: AbortSignal.timeout(15_000),
      headers: { 'Content-Type': 'application/json', Accept: 'application/json, text/event-stream', 'MCP-Protocol-Version': '2025-11-25' },
      body: JSON.stringify({ jsonrpc: '2.0', id: randomUUID(), method, params }) });
    assert.equal(response.status, 200); return response.json();
  };
  const initialized = await rpc('initialize', { protocolVersion: '2025-11-25', capabilities: {}, clientInfo: { name: 'qualification-negative-control', version: '1' } });
  assert.equal(initialized.result?.protocolVersion, '2025-11-25');
  assert.equal(initialized.result.serverInfo.name, 'uar-qualification-official-sdk-peer');
  const listing = await rpc('tools/list', {});
  assert.deepEqual(listing.result?.tools.map(tool => tool.name).sort(), ['qualification_echo', 'qualification_fail', 'qualification_wait']);
  const unsupported = await rpc('QualificationUnsupportedMethod', {});
  assert.equal(unsupported.error?.code, -32601);
  const invalid = await rpc('tools/call', { name: 'qualification_echo', arguments: { marker: 'INVALID' } });
  assert.ok(invalid.error || invalid.result?.isError, 'Independent SDK malformed tool input must fail');
  this.evidence.observations.push({ mcp_sdk: 'rmcp=3.8.0', protocol: initialized.result.protocolVersion, server: initialized.result.serverInfo, listing: listing.result, unsupported, invalid });
  this.peerJournal = resolve(this.directory, 'mcp-calls.txt');
  this.protocolJournal = resolve(this.directory, 'mcp-calls.protocol.jsonl');
  assert.equal(await readFile(this.peerJournal, 'utf8'), '', 'Create-only official peer journal must initially be empty');
  assert.equal(await readFile(this.protocolJournal, 'utf8'), '', 'SDK call-lifetime journal must initially be empty');
  this.agentId = `pr476-mcp-${this.caseId}`; this.tool = `pr476peer__qualification_echo`;
  this.waitTool = 'pr476peer__qualification_wait'; this.errorTool = 'pr476peer__qualification_fail';
  const agent = await request(this, 'GET', '/api/agents/default-agent');
  agent.id = this.agentId; agent.version = '1.0.0'; agent.runtime.entry = 'default';
  agent.metadata = { ...agent.metadata, title: 'Owned protocol qualification', tags: ['qualification'] };
  agent.prompt = { system: 'Use exactly the requested qualification tool once with the requested marker. If denied or failed report it and never claim an effect was performed.', instructions: [] };
  agent.policy.tools.allow = [this.tool, this.waitTool, this.errorTool]; agent.memory.kb.enabled = false;
  await request(this, 'POST', '/api/agents', agent, [201]); this.cleanupAgent = this.agentId;
});
async function mcpRun(w, marker, approved) {
  const session = randomUUID(); w.cleanupSessions.push(session);
  const created = await request(w, 'POST', '/api/uar/runs', { agent_id: w.agentId, input: `Use qualification_echo with marker ${marker}.`, session_id: session, mcp_servers: [{ name: 'pr476peer', url: w.peerUrl.href }] }, [200, 201, 202]);
  const run = created.run_id ?? created.runId; assert.ok(run);
  const deadline = Date.now() + 150_000; let approvals = 0, approval;
  while (Date.now() < deadline) {
    const pending = (await request(w, 'GET', `/api/uar/runs/${run}/tool-approval/pending`)).pending;
    if (pending) {
      assert.equal(pending.name, w.tool); assert.ok(++approvals <= 2);
      if (approved) approval = mcpApproval(pending, run);
      await request(w, 'POST', `/api/uar/runs/${run}/tool-approval`, { approved, approval_id: pending.approvalId });
    }
    const status = await request(w, 'GET', `/api/uar/runs/${run}`);
    if (terminal.has(status.status)) {
      assert.ok(approvals > 0, 'Actual governed approval is required');
      if (approved) assert.equal(status.status, 'done');
      return { run_id: run, status: status.status, approved, approvals, approval };
    }
    await delay(500);
  }
  throw new Error('MCP governed run did not reach terminal state');
}
When('I admit one MCP echo and deny another through native run tool approvals', async function () {
  this.allowedMarker = `PR476-${randomUUID()}`; this.deniedMarker = `PR476-${randomUUID()}`;
  this.allowedRun = await mcpRun(this, this.allowedMarker, true);
  this.evidence.observations.push(this.allowedRun);
  this.evidence.observations.push(await mcpRun(this, this.deniedMarker, false));
});
Then('the peer journal contains exactly the admitted marker', async function () {
  assert.deepEqual((await readFile(this.peerJournal, 'utf8')).trim().split('\n'), [this.allowedMarker]);
  const runId = this.allowedRun.run_id;
  const providerRequests = (await modelObservations(this)).filter(row => row.marker === this.allowedMarker);
  assert.equal(providerRequests.length, 2, 'One tool dispatch and one post-tool provider response are required');
  assert.equal(providerRequests[0].tool, this.tool); assert.equal(providerRequests[1].tool, undefined);
  const toolCalls = await protocolEntries(this, this.allowedMarker);
  assert.equal(toolCalls.length, 1); assert.equal(toolCalls[0].phase, 'completed');
  assert.ok(toolCalls[0].request_id !== undefined);
  const effects = () => readFile(this.peerJournal, 'utf8').then(text => text.trim().split('\n').filter(Boolean).map(marker => ({ marker })));
  const effectRows = await effects();
  const accountingBefore = await request(this, 'GET', `/api/uar/accounting/calls/${runId}`);
  assert.ok(Array.isArray(accountingBefore) && accountingBefore.length > 0);
  const events = await runEvents(this, runId);
  const invocations = events.filter(event => event.eventName === 'agui.tool_call.complete' && event.data?.name === this.tool);
  assert.equal(invocations.length, 1); assert.ok(invocations[0].data.id);
  assert.equal(invocations[0].data.id, this.allowedRun.approval.model_call_id);
  const admission = await mcpAdmission(this, runId, this.tool, this.allowedRun.approval);
  assert.ok(admission.records.some(row => row.state === 'succeeded'));
  toolCalls[0].model_call_id = invocations[0].data.id;
  toolCalls[0].invocation_id = admission.invocation_id;
  // Replay is retained readback only. It must not create a new provider call,
  // MCP invocation, synthetic effect or accounting liability.
  assert.deepEqual(await runEvents(this, runId), events);
  const accountingAfter = await request(this, 'GET', `/api/uar/accounting/calls/${runId}`);
  const providerRequestsAfterReplay = (await modelObservations(this)).filter(row => row.marker === this.allowedMarker);
  const effectRowsAfterReplay = await effects();
  assert.deepEqual(accountingAfter, accountingBefore); assert.deepEqual(providerRequestsAfterReplay, providerRequests);
  assert.deepEqual(effectRowsAfterReplay, effectRows);
  this.evidence.reliability.toolLoop = { runId, status: this.allowedRun.status, providerRequests, toolCalls, effectRows,
    accountingBefore, accountingAfter, providerRequestsAfterReplay, effectRowsAfterReplay };
});

After(async function ({ result }) {
  if (!this.evidence) return;
  const errors = [];
  if (this.evidence.candidateBinding && !this.evidence.candidateBinding.after) {
    try {
      const after = await binding(this);
      assertCandidateArtifactStable(this.evidence.candidateBinding.before, after, { allowRestart: false });
      this.evidence.candidateBinding.after = after;
    } catch (error) { errors.push(error.message); }
  }
  for (const path of this.cleanupKbs ?? []) {
    try { await request(this, 'DELETE', path, undefined, [204]); } catch (error) { errors.push(error.message); }
  }
  if (this.cleanupAgent) {
    try { await request(this, 'DELETE', `/api/agents/${this.cleanupAgent}`, undefined, [204]); } catch (error) { errors.push(error.message); }
  }
  for (const session of this.cleanupSessions ?? []) {
    try { await request(this, 'DELETE', `/api/uar/sessions/${session}`, undefined, [204, 404]); } catch (error) { errors.push(error.message); }
  }
  this.evidence.status = errors.length ? 'FAILED' : result.status;
  this.evidence.cleanup_errors = errors;
  const receipt = resolve('.prometheus/verification/production-artifact-qualification', `${this.caseId}-${randomUUID()}.json`);
  await mkdir(dirname(receipt), { recursive: true });
  await writeFile(receipt, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify({ receipt, status: this.evidence.status }), 'application/json');
  assert.deepEqual(errors, [], 'Owned fixture cleanup must succeed');
});

const FULL = '/api/uar/full-harness/v1';

async function modelObservations(w) {
  const response = await fetch(new URL('/observations', w.controlledModel), { redirect: 'error', signal: AbortSignal.timeout(15_000) });
  assert.equal(response.status, 200); return response.json();
}
Given('the owned controlled streaming model is externally provisioned', async function () {
  this.controlledModel = new URL(process.env.UAR_QUALIFICATION_MODEL_URL);
  assert.equal(this.controlledModel.protocol, 'http:');
  assert.ok(['127.0.0.1', 'localhost', '[::1]'].includes(this.controlledModel.hostname));
  assert.ok(!this.controlledModel.username && !this.controlledModel.password && !this.controlledModel.search);
  assert.equal(this.image.labels['dev.prometheus.qualification.model-url'], this.controlledModel.href);
  this.cleanupSessions ??= [];
});
async function runEvents(w, id) { return (await request(w, 'GET', `/api/uar/runs/${id}/events`)).events; }
async function waitRun(w, id) {
  const deadline = Date.now() + 120_000;
  while (Date.now() < deadline) {
    const view = await request(w, 'GET', `/api/uar/runs/${id}`);
    if (terminal.has(view.status)) return view;
    await delay(250);
  }
  throw new Error('Owned run did not reach a terminal outcome');
}
function eventText(events) {
  return events.filter(event => event.eventName === 'agui.message.delta').map(event => event.data?.delta?.text ?? '').join('');
}
async function nativeRun(w, input) {
  const session = randomUUID(); w.cleanupSessions.push(session);
  const result = await request(w, 'POST', '/api/uar/runs', { agent_id: 'default-agent', input, session_id: session }, [200, 201, 202]);
  const id = result.run_id ?? result.runId; assert.ok(id); return id;
}
Given('a frozen bounded reliability agent is retained for this case', async function () {
  const agent = structuredClone(await request(this, 'GET', '/api/agents/default-agent'));
  agent.id = `pr476-reliability-${this.caseId}`;
  agent.prompt = { system: 'Return the synthetic qualification peer response. Do not call tools.', instructions: [] };
  agent.policy.tools.allow = [`pr476-no-tools-${this.caseId}`]; agent.memory.kb.enabled = false;
  agent.extensions = { ...agent.extensions, budgets: { max_tokens_per_turn: 8192, max_tokens_per_session: 131072, timeout_seconds: 60 } };
  await request(this, 'POST', '/api/agents', agent, [201]);
  this.cleanupAgent = agent.id; this.reliabilityAgent = agent.id;
  this.frozenAgentDigest = digest(canonical(await request(this, 'GET', `/api/agents/${agent.id}`)));
});
async function reliabilityRun(w, input, session = randomUUID()) {
  if (!w.cleanupSessions.includes(session)) w.cleanupSessions.push(session);
  assert.equal(digest(canonical(await request(w, 'GET', `/api/agents/${w.reliabilityAgent}`))), w.frozenAgentDigest);
  const created = await request(w, 'POST', '/api/uar/runs', { agent_id: w.reliabilityAgent, input, session_id: session }, [200, 201, 202]);
  const id = created.run_id ?? created.runId; assert.ok(id);
  const view = await waitRun(w, id);
  const profile = { agent_revision: view.agent_revision, effective_model: view.effective_model, effective_run_policy: view.effective_run_policy };
  if (w.frozenRunProfile) assert.deepEqual(profile, w.frozenRunProfile);
  else { w.frozenRunProfile = profile; w.evidence.observations.push({ frozen_agent_sha256: w.frozenAgentDigest, profile }); }
  return { runId: id, status: view.status };
}
When('the controlled provider cuts a stream after its first semantic chunk', async function () {
  this.interruptionMarker = `PR476-INTERRUPTED-${randomUUID()}`;
  this.interrupted = await reliabilityRun(this, this.interruptionMarker);
});
Then('the interrupted run records error without successful completion', async function () {
  assert.equal(this.interrupted.status, 'error');
  const events = await runEvents(this, this.interrupted.runId);
  assert.ok(eventText(events).includes(this.interruptionMarker), 'Semantic partial output must reach UAR before transport loss');
  assert.ok(events.some(event => event.eventName === 'agui.error'), 'The actual transport failure must remain visible');
  assert.ok(events.every(event => event.eventName !== 'agui.done' || (event.data?.outcome?.type ?? event.data?.status) !== 'success'));
  const provider = (await modelObservations(this)).filter(row => row.marker === this.interruptionMarker);
  assert.ok(provider.length > 0 && provider.every(row => row.stream && row.chunks > 0 && row.interrupted && row.closed && !row.ended));
  const calls = await request(this, 'GET', `/api/uar/accounting/calls/${this.interrupted.runId}`);
  assert.ok(Array.isArray(calls) && calls.length > 0);
  this.evidence.reliability.interruptedStream = { faulted: { ...this.interrupted, terminalEvents: events, calls }, provider };
});
Then('a fresh run under the same frozen profile completes', async function () {
  const marker = `PR476-AFTER-INTERRUPTION-${randomUUID()}`;
  const followUp = await reliabilityRun(this, marker); assert.equal(followUp.status, 'done');
  assert.ok(eventText(await runEvents(this, followUp.runId)).includes(marker));
  this.evidence.reliability.interruptedStream.followUp = followUp;
});
When('sixteen sequential turns use the same retained conversation', async function () {
  this.historySession = randomUUID(); this.historyInputs = []; const turns = [];
  const namespace = randomUUID();
  for (let turn = 0; turn < 16; turn++) {
    const input = `PR476-HISTORY-${namespace}-${turn} synthetic fact ${randomUUID()}`;
    this.historyInputs.push(input);
    const outcome = await reliabilityRun(this, input, this.historySession); assert.equal(outcome.status, 'done');
    assert.ok(eventText(await runEvents(this, outcome.runId)).includes(input));
    turns.push({ ...outcome, inputSha256: digest(input) });
  }
  this.evidence.reliability.longConversation = { sessionId: this.historySession, turns };
});
Then('canonical history and later provider requests retain the original inputs', async function () {
  const history = (await request(this, 'GET', `/api/uar/sessions/${this.historySession}`)).history;
  assert.ok(Array.isArray(history));
  const rows = (await modelObservations(this)).filter(row => this.historyInputs.some(input => input.startsWith(row.marker)));
  assert.equal(rows.length, 16, 'Retries or duplicate provider dispatch must remain visible');
  const hashes = this.historyInputs.map(digest);
  for (let index = 0; index < hashes.length; index++) {
    const messages = history.filter(message => message.role === 'user' && message.content === this.historyInputs[index]);
    assert.equal(messages.length, 1); assert.ok(messages[0].message_id);
    assert.equal(messages[0].run_id, this.evidence.reliability.longConversation.turns[index].runId);
    assert.ok(rows.at(-1).messageSha256.includes(hashes[index]), 'Actual final provider request must retain each selected input');
  }
  Object.assign(this.evidence.reliability.longConversation, { providerRequests: rows, retainedInputSha256: hashes,
    canonicalHistoryMessageIds: history.map(message => message.message_id) });
});

// Optional remote-Surreal configuration only. These stages observe externally
// operated transitions; the legacy native/macOS runner is intentionally separate.
function remoteDatabase(w, running) {
  const name = process.env.UAR_SURREAL_CONTAINER;
  assert.match(name ?? '', /^uar-pr476-[a-z0-9-]+$/);
  assert.equal(w.image.labels['dev.prometheus.qualification.database-container'], name);
  const row = JSON.parse(execFileSync('docker', ['inspect', '--format',
    '{"id":{{json .Id}},"image":{{json .Image}},"reference":{{json .Config.Image}},"labels":{{json .Config.Labels}},"running":{{json .State.Running}},"mounts":{{json .Mounts}}}', name], { encoding: 'utf8', timeout: 15_000 }));
  assert.equal(row.labels?.['dev.prometheus.qualification.case'], w.caseId);
  assert.equal(row.running, running);
  assert.equal(row.reference, 'surrealdb/surrealdb:v3.3.0@sha256:681c6c22c287421b5c7d99e0fde79b6e0d32c36c1ddeaab2762a1661cb04cd20');
  const volume = process.env.UAR_SURREAL_VOLUME;
  assert.ok(volume?.startsWith(`uar-pr476-${w.caseId}-`));
  const mounts = row.mounts.filter(mount => mount.Type === 'volume' && mount.Name === volume);
  assert.equal(mounts.length, 1);
  const labels = JSON.parse(execFileSync('docker', ['volume', 'inspect', '--format', '{{json .Labels}}', volume], { encoding: 'utf8', timeout: 15_000 }));
  assert.equal(labels?.['dev.prometheus.qualification.case'], w.caseId);
  row.volume = volume; row.destination = mounts[0].Destination; delete row.mounts;
  return row;
}
function remoteMessages(w) {
  const logs = spawnSync('docker', ['logs', process.env.UAR_ARTIFACT_CONTAINER], { encoding: 'utf8', timeout: 15_000, maxBuffer: 8 * 1024 * 1024 });
  assert.equal(logs.error, undefined); assert.equal(logs.status, 0);
  const selected = [];
  for (const line of `${logs.stdout}\n${logs.stderr}`.split('\n')) {
    let row; try { row = JSON.parse(line); } catch { continue; }
    const message = row.fields?.message;
    if (['Remote transport lost; reconnecting', 'Remote transport recovered'].includes(message)) {
      assert.ok(Number.isFinite(Date.parse(row.timestamp))); selected.push({ timestamp: row.timestamp, message });
    }
  }
  return selected.sort((a, b) => Date.parse(a.timestamp) - Date.parse(b.timestamp));
}
const remoteStatePath = w => resolve(w.directory, 'remote-surreal.json');
async function remoteState(w) {
  const saved = JSON.parse(await readFile(remoteStatePath(w), 'utf8'));
  assert.equal(saved.case_id, w.caseId); assert.equal(saved.workspace, w.workspace); assert.equal(saved.base_url, w.base.href);
  assert.equal(w.image.id, saved.runtimeBefore.id); assert.equal(w.image.started_at, saved.runtimeBefore.started_at);
  assert.equal(w.image.image, saved.runtimeBefore.image); assert.equal(w.image.volume, saved.runtimeBefore.volume);
  return saved;
}
Given('the owned remote Surreal profile is externally provisioned', async function () {
  assert.equal((await request(this, 'GET', '/api/uar/settings/persistence.provider')).effective, 'surreal');
  const endpoint = new URL((await request(this, 'GET', '/api/uar/settings/persistence.database_url')).effective);
  assert.ok(['ws:', 'wss:', 'http:', 'https:'].includes(endpoint.protocol), 'Embedded memory/surrealkv cannot qualify remote outage');
  assert.ok(!endpoint.username && !endpoint.password && !endpoint.search && !endpoint.hash);
  assert.equal(this.image.labels['dev.prometheus.qualification.surreal-url'], endpoint.href);
  this.remoteDb = remoteDatabase(this, true);
  this.evidence.observations.push({ selected_profile: 'optional_remote_surreal', endpoint: endpoint.href, database: this.remoteDb });
});
When('I retain the remote profile canary and original process', async function () {
  assert.equal(this.evidence.candidateBinding.before.settings['server.log_format'], 'json');
  const saved = { case_id: this.caseId, workspace: this.workspace, base_url: this.base.href, runtimeBefore: this.image,
    databaseBefore: this.remoteDb, candidateBinding: this.evidence.candidateBinding, logsBefore: remoteMessages(this), marker: `PR476-REMOTE-${randomUUID()}` };
  await writeFile(remoteStatePath(this), JSON.stringify(saved, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  saved.kb_path = await newKb(this, 'remote-outage');
  saved.document = await upload(this, saved.kb_path, { name: 'remote-retained.txt', type: 'text/plain', bytes: Buffer.from(`The qualification seal is ${saved.marker}.`) });
  saved.canaryBefore = await canary(this, saved);
  await writeFile(remoteStatePath(this), JSON.stringify(saved, null, 2) + '\n', { mode: 0o600 });
});
Given('the owned Linux API artifact remains running during an external database outage', async function () { await setup(this, false, false); });
Then('the remote profile reports transport loss and unavailable persistence', async function () {
  const saved = await remoteState(this), database = remoteDatabase(this, false);
  assert.equal(database.id, saved.databaseBefore.id); assert.equal(database.image, saved.databaseBefore.image);
  assert.equal(database.volume, saved.databaseBefore.volume);
  const started = Date.now(); const readiness = [];
  const response = await fetch(new URL('/readyz', this.base), { redirect: 'error', signal: AbortSignal.timeout(15_000) });
  const body = await response.text(); readiness.push({ status: response.status, elapsed_ms: Date.now() - started, body_sha256: digest(body) });
  assert.equal(response.status, 503);
  let read;
  try {
    const unavailable = await fetch(new URL(`${saved.kb_path}/documents/${saved.document.id}`, this.base), { redirect: 'error',
      headers: { 'x-uar-workspace-id': this.workspace }, signal: AbortSignal.timeout(15_000) });
    read = { status: unavailable.status, response_sha256: digest(await unavailable.text()) };
    assert.ok([500, 503].includes(read.status), 'The known document must not be reported successfully read during the outage');
  } catch (error) {
    if (error.name !== 'TimeoutError') throw error;
    read = { outcome: 'read_timeout', effects: 'no_write_attempted' };
  }
  const messages = remoteMessages(this); assert.deepEqual(messages.slice(0, saved.logsBefore.length), saved.logsBefore);
  const lostIndex = messages.findIndex((row, index) => index >= saved.logsBefore.length && row.message === 'Remote transport lost; reconnecting');
  assert.ok(lostIndex >= 0);
  saved.outage = { database, readiness, read, lostIndex, messages, observed_at: Date.now() };
  await writeFile(remoteStatePath(this), JSON.stringify(saved, null, 2) + '\n', { mode: 0o600 });
  this.evidence.observations.push(saved.outage);
});
Then('the same remote profile recovers the canary and accepts an owned write', async function () {
  const saved = await remoteState(this); assert.ok(saved.outage, 'Actual observed stopped-database stage required');
  assert.equal(this.remoteDb.id, saved.databaseBefore.id); assert.equal(this.remoteDb.image, saved.databaseBefore.image);
  assert.equal(this.remoteDb.volume, saved.databaseBefore.volume);
  const after = await binding(this); assertCandidateArtifactStable(saved.candidateBinding.before, after, { allowRestart: false });
  const messages = remoteMessages(this); assert.deepEqual(messages.slice(0, saved.outage.messages.length), saved.outage.messages);
  const recoveredIndex = messages.findIndex((row, index) => index > saved.outage.lostIndex && row.message === 'Remote transport recovered');
  assert.ok(recoveredIndex > saved.outage.lostIndex);
  const canaryAfter = await canary(this, saved); assert.deepEqual(canaryAfter, saved.canaryBefore);
  const written = await upload(this, saved.kb_path, { name: 'remote-after.txt', type: 'text/plain', bytes: Buffer.from(`The recovered write seal is PR476-${randomUUID()}.`) });
  const ready = await fetch(new URL('/readyz', this.base), { redirect: 'error', signal: AbortSignal.timeout(15_000) });
  const readyBody = await ready.text(); assert.equal(ready.status, 200);
  const readiness = [...saved.outage.readiness, { status: ready.status, elapsed_ms: Date.now() - saved.outage.observed_at, body_sha256: digest(readyBody) }];
  this.evidence.identity = after.identity; this.evidence.candidateBinding = { before: saved.candidateBinding.before, after };
  this.evidence.reliability.backendOutageRecovery = { runtimeBefore: saved.runtimeBefore, runtimeAfter: this.image, readiness,
    transport: { lostIndex: saved.outage.lostIndex, recoveredIndex }, canaryBefore: saved.canaryBefore, canaryAfter, writeAfter: { id: written.id } };
  this.evidence.observations.push({ remote_transport: messages, read_during_outage: saved.outage.read, database: this.remoteDb });
  this.cleanupKbs = [saved.kb_path];
});
When('a controlled native run actually begins streaming and I cancel it', async function () {
  this.cancelMarker = `PR476-ACTIVE-CANCEL-${randomUUID()}`;
  this.cancelRun = await nativeRun(this, this.cancelMarker);
  const deadline = Date.now() + 30_000;
  let started = false;
  while (Date.now() < deadline) {
    const events = await runEvents(this, this.cancelRun);
    const status = (await request(this, 'GET', `/api/uar/runs/${this.cancelRun}`)).status;
    assert.equal(terminal.has(status), false, 'The required positive control is actually active before cancellation');
    if (eventText(events).length > 0) { started = true; break; }
    await delay(250);
  }
  assert.equal(started, true, 'A real controlled provider chunk must reach the public run stream');
  const before = (await modelObservations(this)).filter(row => row.marker === this.cancelMarker);
  assert.equal(before.length, 1); assert.equal(before[0].stream, true); assert.equal(before[0].ended, false);
  assert.equal((await request(this, 'POST', `/api/uar/runs/${this.cancelRun}/cancel`)).cancelled, true);
});
Then('cancellation closes the provider stream without later success or hidden liabilities', async function () {
  assert.equal((await waitRun(this, this.cancelRun)).status, 'cancelled');
  const first = await runEvents(this, this.cancelRun);
  await delay(1000);
  const later = await runEvents(this, this.cancelRun);
  assert.deepEqual(later, first, 'Terminal cancellation must not grow a late successful stream');
  for (const event of later.filter(event => event.eventName === 'agui.done')) assert.notEqual(event.data?.outcome?.type ?? event.data?.status, 'success');
  const observed = (await modelObservations(this)).filter(row => row.marker === this.cancelMarker);
  assert.equal(observed.length, 1); assert.equal(observed[0].closed, true); assert.equal(observed[0].ended, false);
  const calls = await request(this, 'GET', `/api/uar/accounting/calls/${this.cancelRun}`);
  assert.ok(Array.isArray(calls) && calls.length > 0);
  const unresolved = later.some(event => event.eventName === 'agui.custom' && event.data?.name === 'uar.presentation.diagnostic' && event.data?.value?.code === 'accounting_unresolved');
  for (const call of calls) assert.ok(call.state === 'settled' || (call.state === 'pending' && (call.reconciliation_evidence || unresolved)), 'Cancellation accounting must be settled or explicitly uncertain');
  this.evidence.observations.push({ cancellation_run: this.cancelRun, provider: observed, calls, terminal_events: later });
  this.evidence.reliability.cancellation = { runId: this.cancelRun, status: 'cancelled', provider: observed, calls, terminalEvents: later };
});
Then('a new controlled native run still completes after cancellation', async function () {
  const marker = `PR476-AFTER-CANCEL-${randomUUID()}`;
  const id = await nativeRun(this, marker); assert.equal((await waitRun(this, id)).status, 'done');
  assert.ok(eventText(await runEvents(this, id)).includes(marker));
  this.evidence.reliability.cancellation.followUp = { runId: id, status: 'done' };
});

async function fullPayload(w, marker) {
  const artifact = structuredClone(await request(w, 'GET', '/api/agents/default-agent'));
  artifact.id = `pr476-admission-${randomUUID()}`;
  artifact.policy.tools.allow = [`pr476-no-tools-${w.caseId}`]; artifact.memory.kb.enabled = false;
  artifact.extensions = { ...artifact.extensions, budgets: { max_tokens_per_turn: 8192, max_tokens_per_session: 32768, timeout_seconds: 60 } };
  const session = randomUUID(); w.cleanupSessions.push(session);
  return { admission_id: randomUUID(), native_task_id: `pr476-${randomUUID()}`, artifact, input: marker, session_id: session };
}
function identities(receipt) {
  return Object.fromEntries(['admission_id', 'native_task_id', 'task_id', 'run_id', 'runtime_epoch'].map(key => [key, receipt[key]]));
}
When('two concurrent identical full-harness admissions race on one bounded request', async function () {
  this.raceMarker = `PR476-RACE-${randomUUID()}`;
  this.racePayload = await fullPayload(this, this.raceMarker);
  this.raceReceipts = await Promise.all([request(this, 'POST', `${FULL}/tasks`, this.racePayload, [202]), request(this, 'POST', `${FULL}/tasks`, this.racePayload, [202])]);
});
Then('their stable identities and retained completion refer to one execution', async function () {
  const [first, second] = this.raceReceipts; assert.deepEqual(identities(first), identities(second));
  assert.ok(first.run_id && first.task_id && first.runtime_epoch);
  assert.equal((await waitRun(this, first.run_id)).status, 'done');
  const before = await runEvents(this, first.run_id);
  assert.ok(eventText(before).includes(this.raceMarker));
  const retry = await request(this, 'POST', `${FULL}/tasks`, this.racePayload, [202]);
  assert.deepEqual(identities(retry), identities(first));
  const conflict = await request(this, 'POST', `${FULL}/tasks`, { ...this.racePayload, input: 'changed request' }, [409]);
  assert.equal(conflict.error?.code, 'admission_digest_conflict');
  assert.deepEqual(await runEvents(this, first.run_id), before);
  const arrived = (await modelObservations(this)).filter(row => row.marker === this.raceMarker);
  assert.equal(arrived.length, 1, 'Concurrent retries/digest conflict must not dispatch another provider call');
  this.evidence.observations.push({ admission_race: this.raceReceipts.map(identities), retained_events: before, provider: arrived });
});
Then('an abandoned admission response is reconciled by identity without redispatch', async function () {
  const marker = `PR476-ABANDONED-${randomUUID()}`, payload = await fullPayload(this, marker);
  const response = await fetch(new URL(`${FULL}/tasks`, this.base), { method: 'POST', redirect: 'error', signal: AbortSignal.timeout(120_000),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': this.workspace }, body: JSON.stringify(payload) });
  assert.equal(response.status, 202);
  // Discard actual HTTP response bytes. This is HTTP reply loss, not a claim
  // that the database acknowledgement was lost or a transaction rolled back.
  await response.body?.cancel();
  const found = await request(this, 'GET', `${FULL}/admissions/${payload.admission_id}`);
  assert.equal(found.admission_id, payload.admission_id);
  assert.equal((await waitRun(this, found.run_id)).status, 'done');
  const replay = await request(this, 'POST', `${FULL}/tasks`, payload, [202]);
  assert.deepEqual(identities(replay), identities(found));
  assert.equal((await modelObservations(this)).filter(row => row.marker === marker).length, 1);
  this.evidence.observations.push({ abandoned_http_reply: true, database_commit_fault: false, recovered: identities(found) });
});

When('I capture a settled full-harness task and its runtime epoch', async function () {
  const descriptor = await request(this, 'GET', `${FULL}/capabilities`);
  assert.equal(descriptor.recovery, 'unsupported_after_restart'); assert.equal(descriptor.retention.mode, 'process_ephemeral'); assert.equal(descriptor.steer_supported, false);
  const marker = `PR476-EPOCH-${randomUUID()}`, payload = await fullPayload(this, marker);
  const receipt = await request(this, 'POST', `${FULL}/tasks`, payload, [202]);
  assert.equal((await waitRun(this, receipt.run_id)).status, 'done');
  const task = await request(this, 'GET', `${FULL}/tasks/${receipt.task_id}`);
  assert.ok(Number.isSafeInteger(task.cursor) && task.cursor > 0, 'Capture a real retained cursor before process replacement');
  const captured = { case_id: this.caseId, workspace: this.workspace, base_url: this.base.href, descriptor, receipt: task, marker, image: this.image,
    session_id: payload.session_id,
    provider_calls: (await modelObservations(this)).filter(row => row.marker === marker).length };
  assert.equal(captured.provider_calls, 1);
  // Preserve this session across the staged external restart; no scenario
  // cleanup can pretend to have tested an old cursor after deleting the run.
  this.cleanupSessions = this.cleanupSessions.filter(id => id !== payload.session_id);
  await writeFile(resolve(this.directory, 'epoch.json'), JSON.stringify(captured, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
});
Then('the captured epoch fixture is retained for the external transition', async function () {
  const captured = JSON.parse(await readFile(resolve(this.directory, 'epoch.json'), 'utf8'));
  assert.equal(captured.receipt.runtime_epoch, captured.descriptor.runtime_epoch); assert.ok(captured.receipt.task_id);
});
Then('the old task cursor admission and controls refuse under the new epoch without dispatch', async function () {
  const old = JSON.parse(await readFile(resolve(this.directory, 'epoch.json'), 'utf8'));
  assert.equal(old.case_id, this.caseId); assert.equal(old.workspace, this.workspace); assert.equal(old.base_url, this.base.href);
  const descriptor = await request(this, 'GET', `${FULL}/capabilities`);
  assert.notEqual(descriptor.runtime_epoch, old.descriptor.runtime_epoch, 'Actual replacement epoch required');
  assert.ok(Date.parse(this.image.started_at) > Date.parse(old.image.started_at));
  const dispatchBefore = await modelObservations(this);
  const task = `${FULL}/tasks/${old.receipt.task_id}`;
  for (const [method, path, body] of [
    ['GET', task], ['GET', `${task}/stream?last_event_id=${old.receipt.cursor}`],
    ['POST', `${task}/cancel`, { expected_revision: old.receipt.revision }],
    ['POST', `${task}/detach`, { expected_revision: old.receipt.revision, observer_id: 'old-observer' }],
    ['POST', `${task}/steer`, { expected_revision: old.receipt.revision, input: 'must not execute' }],
  ]) assert.equal((await request(this, method, path, body, [409])).error?.code, 'recovery_unsupported');
  assert.equal((await request(this, 'GET', `${FULL}/admissions/${old.receipt.admission_id}`, undefined, [404])).error?.code, 'task_unresolved');
  assert.equal((await modelObservations(this)).filter(row => row.marker === old.marker).length, old.provider_calls);
  assert.deepEqual(await modelObservations(this), dispatchBefore, 'Old controls must not dispatch any new model request');
  this.cleanupSessions.push(old.session_id);
  this.evidence.observations.push({ old_epoch: old.descriptor.runtime_epoch, new_epoch: descriptor.runtime_epoch, old_task_refused: old.receipt.task_id, provider_calls_unchanged: true });
});

Then('PostgreSQL capabilities and all selected unsupported APIs agree', async function () {
  const capabilities = await request(this, 'GET', '/api/uar/persistence/capabilities');
  assert.equal(capabilities.persistence.backend, 'postgres');
  for (const key of ['supports_channel_observers', 'supports_durable_observers', 'supports_durable_agent_instances', 'supports_durable_checkpoint_resume']) assert.equal(capabilities.persistence.capabilities[key], false);
  for (const path of ['/api/uar/followups', '/api/uar/schedules', '/api/uar/agent-instances/v1/', '/api/uar/observers/v1/', '/api/uar/channel-observers/v1/subscriptions']) {
    const refused = await request(this, 'GET', path, undefined, [503]);
    assert.equal(refused.error?.code ?? refused.code, 'capability_unsupported');
  }
  const team = await request(this, 'POST', `/api/v1/collaboration/team-instances/pr476-${this.caseId}/tasks/probe/admit`, {
    commandId: randomUUID(), expectedTeamRevision: 0, expectedTaskRevision: 0, memberId: 'probe',
    reservation: { tokens: 1, costMicrounits: 0, elapsedSeconds: 1 }, contextArtifactIds: [],
  }, [503]);
  assert.equal(team.error?.code, 'capability_unsupported');
  const refused = await request(this, 'POST', `/api/uar/runs/pr476-unsupported-${randomUUID()}/resume`, {}, [503]);
  assert.equal(refused.error?.code, 'checkpoint_resume_unsupported');
  this.evidence.observations.push({ postgres_capabilities: capabilities, unsupported_checkpoint_code: refused.error.code, memory_is_separate: capabilities.memory });
});

Given('the owned PG18 store and second identity endpoint are externally provisioned', async function () {
  const container = process.env.UAR_POSTGRES_CONTAINER, volume = process.env.UAR_POSTGRES_VOLUME;
  assert.match(container ?? '', /^uar-pr476-[a-z0-9-]+$/);
  assert.equal(this.image.labels['dev.prometheus.qualification.database-container'], container);
  const projected = '{"id":{{json .Id}},"reference":{{json .Config.Image}},"image":{{json .Image}},"labels":{{json .Config.Labels}},"running":{{json .State.Running}},"mounts":{{json .Mounts}}}';
  const store = JSON.parse(execFileSync('docker', ['inspect', '--format', projected, container], { encoding: 'utf8', timeout: 15_000 }));
  assert.equal(store.labels?.['dev.prometheus.qualification.case'], this.caseId); assert.equal(store.running, true);
  assert.equal(store.reference, 'tribehealth/flint-forge-pg:18@sha256:55c12cbf613346315a48a5c465e804c1ca61ccd1be283db4683ca7c690a9069c');
  const mounts = store.mounts.filter(mount => mount.Destination === '/var/lib/postgresql');
  assert.equal(mounts.length, 1); assert.equal(mounts[0].Type, 'volume'); assert.equal(mounts[0].Name, volume);
  assert.ok(volume?.startsWith(`uar-pr476-${this.caseId}-`));
  const labels = JSON.parse(execFileSync('docker', ['volume', 'inspect', '--format', '{{json .Labels}}', volume], { encoding: 'utf8', timeout: 15_000 }));
  assert.equal(labels?.['dev.prometheus.qualification.case'], this.caseId);
  this.otherOwnerBase = new URL(process.env.UAR_OTHER_OWNER_BASE_URL);
  assert.ok(['http:', 'https:'].includes(this.otherOwnerBase.protocol) && !this.otherOwnerBase.username && !this.otherOwnerBase.password && !this.otherOwnerBase.search);
  assert.notEqual(this.otherOwnerBase.href, this.base.href);
  assert.equal(this.image.labels['dev.prometheus.qualification.other-owner-base-url'], this.otherOwnerBase.href);
  const capabilities = await request(this, 'GET', '/api/uar/persistence/capabilities'); assert.equal(capabilities.persistence.backend, 'postgres');
  this.postgresStore = { container, id: store.id, reference: store.reference, image: store.image, volume };
  this.evidence.observations.push({ postgres_store: this.postgresStore, capabilities });
});
async function pgState(w) {
  const saved = JSON.parse(await readFile(resolve(w.directory, 'postgres.json'), 'utf8'));
  assert.equal(saved.case_id, w.caseId); assert.equal(saved.workspace, w.workspace); assert.equal(saved.base_url, w.base.href);
  for (const key of ['container', 'reference', 'image', 'volume']) assert.equal(w.postgresStore[key], saved.store[key], 'Restore must retain the selected PG18 artifact and owned volume');
  return saved;
}
async function savePg(w, saved, create = false) {
  await writeFile(resolve(w.directory, 'postgres.json'), JSON.stringify(saved, null, 2) + '\n', { flag: create ? 'wx' : 'w', mode: 0o600 });
}
async function otherOwnerCannotRead(w, saved) {
  const other = { ...w, base: w.otherOwnerBase };
  await request(other, 'GET', saved.kb_path, undefined, [404]);
  await request(other, 'GET', `${saved.kb_path}/documents/${saved.document.id}`, undefined, [404]);
  await request(other, 'POST', `${saved.kb_path}/search`, { query: saved.marker, limit: 20, min_score: 0 }, [404]);
}
When('I save a PG18 owner-scoped restore canary', async function () {
  const saved = { case_id: this.caseId, workspace: this.workspace, base_url: this.base.href, store: this.postgresStore,
    marker: `PR476-PG-${randomUUID()}` };
  await savePg(this, saved, true);
  saved.kb_path = await newKb(this, 'pg18-restore'); await savePg(this, saved);
  saved.document = await upload(this, saved.kb_path, { name: 'pg18-owned.txt', type: 'text/plain', bytes: Buffer.from(`The qualification seal is ${saved.marker}.`) });
  await canary(this, saved); await savePg(this, saved);
});
Then('the other externally supplied owner cannot read that canary', async function () { await otherOwnerCannotRead(this, await pgState(this)); });
When('I bind the PG18 backup and delete the PG18 restore canary', async function () {
  const saved = await pgState(this); await canary(this, saved); await otherOwnerCannotRead(this, saved);
  const backup = await readFile(resolve(this.directory, 'postgres-backup.sql')); assert.ok(backup.length > 0);
  saved.backup_sha256 = digest(backup);
  await request(this, 'DELETE', `${saved.kb_path}/documents/${saved.document.id}`, undefined, [204]);
  saved.deleted_after_backup = true; await savePg(this, saved);
});
Then('the PG18 canary is absent before external restore', async function () {
  const saved = await pgState(this); assert.equal(saved.deleted_after_backup, true);
  await request(this, 'GET', `${saved.kb_path}/documents/${saved.document.id}`, undefined, [404]);
  const result = await request(this, 'POST', `${saved.kb_path}/search`, { query: saved.marker, limit: 20, min_score: 0 });
  assert.ok(result.results.every(row => row.document_id !== saved.document.id && !row.content.includes(saved.marker)));
  saved.absence_observed = true; await savePg(this, saved);
});
Then('the PG18 backup is unchanged and restored data remains owner scoped', async function () {
  const saved = await pgState(this); assert.equal(saved.absence_observed, true);
  assert.equal(digest(await readFile(resolve(this.directory, 'postgres-backup.sql'))), saved.backup_sha256);
  await canary(this, saved); await otherOwnerCannotRead(this, saved);
  saved.restore_observed = true;
  await request(this, 'DELETE', saved.kb_path, undefined, [204]); saved.canary_cleanup = true; await savePg(this, saved);
  this.evidence.observations.push({ pg18_restore: saved.document.id, owner_isolation_preserved: true, backup_sha256: saved.backup_sha256 });
});

async function protocolEntries(w, marker) {
  return (await readFile(w.protocolJournal, 'utf8')).split('\n').filter(Boolean).map(line => JSON.parse(line)).filter(row => row.marker === marker);
}

function mcpApproval(pending, runId) {
  assert.equal(pending.rootRunId, runId);
  assert.ok(typeof pending.admissionId === 'string' && pending.admissionId.length > 0);
  assert.ok(typeof pending.toolCallId === 'string' && pending.toolCallId.length > 0);
  return { admission_id: pending.admissionId, model_call_id: pending.toolCallId };
}

async function mcpAdmission(w, runId, tool, approval) {
  const evidence = await request(w, 'GET', `/api/uar/runs/${runId}/tool-admission-evidence`);
  assert.equal(evidence.runId, runId);
  const records = evidence.records.filter(row => row.admission_id === approval.admission_id);
  assert.ok(records.some(row => row.state === 'claim_intent'), 'The exact approved call must have durable claim intent');
  const invocation = records[0].invocation_id;
  assert.ok(typeof invocation === 'string' && invocation.length > 0);
  for (const row of records) {
    assert.equal(row.run_id, runId);
    assert.equal(row.root_run_id, runId);
    assert.equal(row.tool_name, tool);
    assert.equal(row.invocation_id, invocation);
  }
  return { invocation_id: invocation, records };
}

async function approvedProtocolTool(w, tool, marker, phase) {
  const session = randomUUID(); w.cleanupSessions.push(session);
  const created = await request(w, 'POST', '/api/uar/runs', { agent_id: w.agentId, input: `Use ${tool.split('__').at(-1)} with marker ${marker}.`, session_id: session,
    mcp_servers: [{ name: 'pr476peer', url: w.peerUrl.href }] }, [200, 201, 202]);
  const id = created.run_id ?? created.runId; assert.ok(id);
  const deadline = Date.now() + 30_000; let approval;
  while (Date.now() < deadline) {
    const pending = (await request(w, 'GET', `/api/uar/runs/${id}/tool-approval/pending`)).pending;
    if (pending) {
      assert.equal(approval, undefined, 'The fixture never silently approves a retry');
      assert.equal(pending.name, tool);
      approval = mcpApproval(pending, id);
      if (tool === w.waitTool) {
        // Retain the exact resources even if approval or later readback loses its response.
        w.cleanupSessions = w.cleanupSessions.filter(candidate => candidate !== session);
        w.retainedMcpAgent = w.cleanupAgent;
        w.cleanupAgent = undefined;
      }
      await request(w, 'POST', `/api/uar/runs/${id}/tool-approval`, { approved: true, approval_id: pending.approvalId });
    }
    const rows = await protocolEntries(w, marker);
    if (rows.some(row => row.phase === phase)) {
      assert.ok(approval, 'Counterpart arrival requires actual governed approval');
      const actual = rows.filter(row => row.phase === phase); assert.equal(actual.length, 1);
      assert.ok(actual[0].request_id !== undefined && actual[0].request_id !== null, 'Original MCP request ID must be journalled');
      const started = (await runEvents(w, id)).filter(event => event.eventName === 'agui.tool_call.complete' && event.data?.name === tool);
      assert.equal(started.length, 1);
      assert.equal(started[0].data.id, approval.model_call_id);
      const { invocation_id } = await mcpAdmission(w, id, tool, approval);
      return { run_id: id, session, request_id: actual[0].request_id, invocation_id, ...approval, marker, tool };
    }
    await delay(100);
  }
  throw new Error('The independently pinned SDK counterpart never observed the approved original call');
}
When('an approved independent MCP request is pending and I cancel its native owner', async function () {
  const marker = `PR476-MCP-WAIT-${randomUUID()}`;
  this.mcpCancellation = await approvedProtocolTool(this, this.waitTool, marker, 'started');
  assert.equal(terminal.has((await request(this, 'GET', `/api/uar/runs/${this.mcpCancellation.run_id}`)).status), false, 'Actual MCP request must be pending before cancellation');
  assert.deepEqual((await protocolEntries(this, marker)).map(row => row.phase), ['started']);
  assert.equal((await request(this, 'POST', `/api/uar/runs/${this.mcpCancellation.run_id}/cancel`)).cancelled, true);
});
Then('the same SDK request terminates without completion or an effect claim', async function () {
  const original = this.mcpCancellation;
  const deadline = Date.now() + 15_000; let rows;
  while (Date.now() < deadline) {
    rows = await protocolEntries(this, original.marker);
    if (rows.some(row => row.phase !== 'started')) break;
    await delay(100);
  }
  const end = rows.filter(row => row.phase !== 'started');
  assert.equal(end.length, 1, 'The actual independent request must terminate once');
  assert.ok(['cancelled', 'handler_dropped'].includes(end[0].phase), 'Expiry or natural completion cannot pass cancellation');
  assert.equal(end[0].sdk_cancelled, true, 'The official counterpart SDK must observe request cancellation');
  for (const row of rows) { assert.deepEqual(row.request_id, original.request_id); assert.equal(row.effect_performed, false); }
  const status = await waitRun(this, original.run_id);
  assert.ok(['cancelled', 'error', 'failed'].includes(status.status), 'Connection uncertainty must not become successful execution');
  const events = await runEvents(this, original.run_id);
  const results = events.filter(event => event.eventName === 'agui.tool_result' && event.data?.id === original.model_call_id);
  for (const result of results) assert.equal(result.data.success, false);
  for (const event of events.filter(event => event.eventName === 'agui.done')) assert.notEqual(event.data?.outcome?.type ?? event.data?.status, 'success');
  let admission;
  const evidenceDeadline = Date.now() + 15_000;
  do {
    admission = await mcpAdmission(this, original.run_id, original.tool, original);
    assert.equal(admission.invocation_id, original.invocation_id);
    if (admission.records.some(row => row.state === 'outcome_unknown')) break;
    await delay(100);
  } while (Date.now() < evidenceDeadline);
  const { records } = admission;
  assert.equal(records.some(row => row.state === 'succeeded'), false);
  assert.ok(records.some(row => row.state === 'outcome_unknown'), 'The missing result for the claimed MCP call must stay explicitly unknown');
  assert.equal(records.some(row => ['cancelled', 'invalidated'].includes(row.state)), false, 'Local cancellation cannot settle the claimed MCP effect');
  const calls = await request(this, 'GET', `/api/uar/accounting/calls/${original.run_id}`);
  assert.ok(Array.isArray(calls) && calls.length > 0, 'The actual model/tool exchange must retain its admitted accounting');
  this.evidence.observations.push({ retained_uncertain_agent: this.retainedMcpAgent });
  this.evidence.observations.push({ independent_mcp_cancellation: original, counterpart: rows, status: status.status, results, admission: records, calls,
    retained_uncertain_session: original.session, remote_effect_rollback_claimed: false });
});
When('an approved independent MCP request returns an intentional tool error', async function () {
  this.mcpError = await approvedProtocolTool(this, this.errorTool, `PR476-MCP-ERROR-${randomUUID()}`, 'intentional_error');
  const status = await waitRun(this, this.mcpError.run_id);
  const results = (await runEvents(this, this.mcpError.run_id)).filter(event => event.eventName === 'agui.tool_result' && event.data?.id === this.mcpError.model_call_id);
  assert.equal(results.length, 1); assert.equal(results[0].data.success, false);
  const rows = await protocolEntries(this, this.mcpError.marker);
  assert.equal(rows.length, 1); assert.deepEqual(rows[0].request_id, this.mcpError.request_id); assert.equal(rows[0].effect_performed, false);
  const admission = await mcpAdmission(this, this.mcpError.run_id, this.mcpError.tool, this.mcpError);
  assert.equal(admission.invocation_id, this.mcpError.invocation_id);
  const { records } = admission;
  assert.ok(records.some(row => row.state === 'application_failed'), 'An explicit SDK isError result must stay an application failure');
  assert.equal(records.some(row => row.state === 'succeeded'), false);
  this.evidence.observations.push({ independent_mcp_error: this.mcpError, status: status.status, result: results[0], counterpart: rows, admission: records });
});
Then('the actual tool result is an error and the peer remains usable', async function () {
  const marker = `PR476-AFTER-MCP-CANCEL-${randomUUID()}`;
  await mcpRun(this, marker, true);
  const records = (await readFile(this.peerJournal, 'utf8')).trim().split('\n');
  assert.deepEqual(records, [this.allowedMarker, marker], 'Canceled/error/denied calls never append an echo effect and later binding stays usable');
});

When('the official A2A SDK cancels an actually active original task', async function () {
  this.a2aCancelMarker = `PR476-ACTIVE-CANCEL-A2A-${randomUUID()}`;
  try {
    this.a2aCancellation = await qualifyA2ACancellation({ url: this.base.href, workspace: this.workspace, text: this.a2aCancelMarker, messageId: randomUUID(),
      waitForActive: async task => {
        const deadline = Date.now() + 20_000;
        while (Date.now() < deadline) {
          const rows = (await modelObservations(this)).filter(row => row.marker === this.a2aCancelMarker);
          if (rows.length > 0) {
            assert.equal(rows.length, 1); assert.equal(rows[0].stream, true); assert.ok(rows[0].chunks > 0);
            assert.equal(rows[0].ended, false); assert.equal(rows[0].closed, false);
            this.evidence.observations.push({ original_a2a_task: { id: task.id, context_id: task.contextId }, active_provider: rows });
            return;
          }
          await delay(100);
        }
        throw new Error('Independent task must reach an actually open provider stream before CancelTask');
      },
    });
  } catch (error) {
    if (error.qualification) this.evidence.observations.push({ independent_a2a_cancellation_refused_or_failed: error.qualification });
    throw error;
  }
});
Then('the independent task remains canceled and original missing or terminal task errors are explicit', async function () {
  assert.ok(this.a2aCancellation?.original_task?.id);
  const rows = (await modelObservations(this)).filter(row => row.marker === this.a2aCancelMarker);
  assert.equal(rows.length, 1); assert.equal(rows[0].closed, true); assert.equal(rows[0].ended, false);
  assert.deepEqual(this.a2aCancellation.errors.map(error => error.reason), ['TASK_NOT_CANCELABLE', 'TASK_NOT_FOUND', 'TASK_NOT_FOUND']);
  this.evidence.observations.push({ independent_a2a_cancellation: this.a2aCancellation, provider: rows, provider_only_cancel_used: false });
});
Then('a new independent A2A task completes after cancellation', async function () {
  const marker = `PR476-A2A-AFTER-CANCEL-${randomUUID()}`;
  const result = await qualifyA2A({ url: this.base.href, workspace: this.workspace, text: `Reply exactly ${marker}.`, messageId: randomUUID() });
  assert.ok(JSON.stringify(result.events).includes(marker));
  this.evidence.observations.push({ independent_a2a_followup: result });
});

When('I write an embedded upgrade owner canary', async function () {
  const kbPath = await newKb(this, 'embedded-upgrade');
  const canary = await request(this, 'GET', kbPath);
  await save(this, {
    case_id: this.caseId, workspace: this.workspace, base_url: this.base.href,
    prior: this.image, kbPath, canaryId: canary.id, canaryName: canary.name,
  }, true);
});

Then('the separately provisioned owner cannot read the upgrade canary', async function () {
  const saved = await state(this);
  const base = new URL(process.env.UAR_OTHER_OWNER_BASE_URL);
  const workspace = process.env.UAR_OTHER_OWNER_WORKSPACE_ID;
  assert.ok(workspace);
  assert.ok(['http:', 'https:'].includes(base.protocol) && !base.username && !base.password && !base.search);
  assert.ok(base.href !== this.base.href || workspace !== this.workspace);
  assert.equal(this.image.labels['dev.prometheus.qualification.other-base-url'], base.href);
  assert.equal(this.image.labels['dev.prometheus.qualification.other-workspace'], workspace);
  const other = { base, workspace, caseId: this.caseId, evidence: this.evidence };
  const otherKb = await newKb(other, 'embedded-upgrade-owner-control');
  try {
    const control = await request(other, 'GET', otherKb);
    assert.ok(control.id, 'Second owner must have working access to its own record');
    await request(other, 'GET', saved.kbPath, undefined, [403, 404]);
  } finally {
    await request(other, 'DELETE', otherKb, undefined, [204]);
  }
  saved.otherOwnerDenied = true;
  await save(this, saved);
});

Then('the embedded upgrade artifact has stopped with its original volume', async function () {
  const saved = await state(this);
  assert.equal(saved.otherOwnerDenied, true);
  assert.equal(this.image.image, saved.prior.image);
  assert.equal(this.image.volume, saved.prior.volume);
  assert.equal(this.image.exit_code, 0);
  assert.ok(Date.parse(this.image.finished_at) > Date.parse(saved.prior.started_at));
  saved.stopped = this.image;
  await save(this, saved);
});

Then('the embedded upgrade artifact reopens and retains the owner canary', async function () {
  const saved = await state(this);
  assert.ok(saved.stopped, 'Capture the clean exit before reopen');
  assert.equal(this.image.image, saved.stopped.image);
  assert.equal(this.image.volume, saved.stopped.volume);
  assert.ok(Date.parse(this.image.started_at) > Date.parse(saved.stopped.finished_at));
  const canary = await request(this, 'GET', saved.kbPath);
  assert.equal(canary.id, saved.canaryId);
  assert.equal(canary.name, saved.canaryName);
  this.evidence.observations.push({
    operation: 'embedded-upgrade-reopen',
    canaryId: canary.id,
    otherOwnerDenied: saved.otherOwnerDenied,
    cleanExit: saved.stopped.exit_code,
  });
  await request(this, 'DELETE', saved.kbPath, undefined, [204]);
});
