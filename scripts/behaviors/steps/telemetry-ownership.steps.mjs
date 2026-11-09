import assert from 'node:assert/strict';
import { createHash, randomUUID } from 'node:crypto';
import { readFile, stat, mkdir, writeFile } from 'node:fs/promises';
import { dirname, isAbsolute, resolve } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';

setDefaultTimeout(240_000);
const MAX_JOURNAL = 4 * 1024 * 1024;
let poisoned = false;
const sha = value => createHash('sha256').update(value).digest('hex');
function local(value) {
  const url = new URL(value);
  assert.ok(['http:', 'https:'].includes(url.protocol) && ['127.0.0.1', '[::1]'].includes(url.hostname));
  assert.ok(!url.username && !url.password && !url.search && !url.hash);
  return url;
}
function canonical(value) {
  return JSON.stringify(value, (_key, item) => item && typeof item === 'object' && !Array.isArray(item)
    ? Object.fromEntries(Object.entries(item).sort(([a], [b]) => a.localeCompare(b))) : item);
}
async function http(w, base, path, method = 'GET', body) {
  assert.ok(++w.telemetryOps <= 200, 'Bounded telemetry HTTP budget exceeded');
  const response = await fetch(new URL(path, base), { method, redirect: 'error', signal: AbortSignal.timeout(15_000),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': w.telemetryWorkspace },
    body: body === undefined ? undefined : JSON.stringify(body) });
  const raw = await response.text();
  assert.ok(Buffer.byteLength(raw) <= MAX_JOURNAL, 'Telemetry response budget exceeded');
  let value;
  try { value = JSON.parse(raw); } catch { value = null; }
  // Receipts retain no request/response bodies or real owner material.
  w.telemetryReceipt.exchanges.push({ method, path: path.split('?')[0], status: response.status });
  return { status: response.status, raw, value };
}
async function journalStart(path) {
  assert.ok(isAbsolute(path), 'External journal path must be explicit and absolute');
  const metadata = await stat(path);
  assert.ok(metadata.isFile() && metadata.size <= MAX_JOURNAL, 'Missing/big external journal');
  const bytes = await readFile(path);
  assert.ok(bytes.length <= MAX_JOURNAL && (bytes.length === 0 || bytes.at(-1) === 10), 'Journal must start at a complete bounded JSONL boundary');
  return { path, inode: metadata.ino, device: metadata.dev, offset: bytes.length, prefixSha256: sha(bytes) };
}
async function journalSince(cursor) {
  const metadata = await stat(cursor.path);
  assert.ok(metadata.ino === cursor.inode && metadata.dev === cursor.device && metadata.size <= MAX_JOURNAL,
    'Journal rotation/replacement/overflow cannot certify this run');
  const bytes = await readFile(cursor.path);
  assert.ok(bytes.length >= cursor.offset && bytes.length <= MAX_JOURNAL);
  assert.equal(sha(bytes.subarray(0, cursor.offset)), cursor.prefixSha256, 'Journal prefix changed');
  const raw = bytes.subarray(cursor.offset).toString('utf8');
  const complete = raw.slice(0, raw.lastIndexOf('\n') + 1);
  return { raw, rows: complete.split('\n').filter(Boolean).map(line => JSON.parse(line)) };
}
function privatePayloadAbsent(raw, sentinels) {
  for (const sentinel of sentinels) {
    for (const representation of [sentinel, JSON.stringify(sentinel).slice(1, -1), Buffer.from(sentinel).toString('base64')]) {
      assert.ok(!raw.includes(representation), 'Synthetic private content escaped into telemetry');
    }
  }
}
function logRows(rows) {
  return rows.map(row => ({ ...Object.assign({}, ...(row.spans ?? [])), ...(row.span ?? {}), ...(row.fields ?? {}),
    level: row.level, target: row.target }));
}
function attributes(rows = []) {
  return Object.fromEntries(rows.map(({ key, value }) => [key,
    value?.stringValue ?? value?.intValue ?? value?.doubleValue ?? value?.boolValue]));
}
function otlpRows(exports) {
  const spans = exports.flatMap(payload => {
    assert.ok(Array.isArray(payload.resourceSpans), 'Collector journal must contain actual OTLP JSON ExportTraceServiceRequest values');
    return payload.resourceSpans.flatMap(resource => resource.scopeSpans.flatMap(scope => scope.spans.map(span => ({
      ...span, resource: attributes(resource.resource?.attributes), own: attributes(span.attributes),
    }))));
  });
  const byId = new Map(spans.map(span => [`${span.traceId}/${span.spanId}`, span]));
  function inherited(span, seen = new Set()) {
    const key = `${span.traceId}/${span.spanId}`;
    assert.ok(!seen.has(key), 'Invalid cyclic exported span parent');
    seen.add(key);
    const parent = byId.get(`${span.traceId}/${span.parentSpanId}`);
    return { ...(parent ? inherited(parent, seen) : {}), ...span.own };
  }
  return spans.flatMap(span => {
    const base = { ...inherited(span), traceId: span.traceId, spanId: span.spanId,
      serviceName: span.resource['service.name'], spanName: span.name, endTimeUnixNano: span.endTimeUnixNano };
    return [base, ...(span.events ?? []).map(event => ({ ...base, ...attributes(event.attributes), message: event.name }))];
  });
}
async function peerRows(w) {
  const journal = await journalSince(w.telemetryJournals.peer);
  return journal.rows.filter(row => row.namespace === w.telemetryFixture.namespace && row.caseId === w.telemetryCase);
}
async function waitFor(label, read, predicate) {
  const deadline = Date.now() + 30_000;
  for (let attempt = 0; attempt < 100 && Date.now() < deadline; attempt++) {
    const value = await read();
    if (predicate(value)) return value;
    await delay(200);
  }
  throw new Error(`No actual bounded ${label} observation; unsupported/unavailable is not a pass`);
}
async function ledger(w) {
  const response = await http(w, w.telemetryBase, `/api/uar/accounting/calls/${encodeURIComponent(w.telemetryRun)}`);
  assert.equal(response.status, 200); assert.ok(Array.isArray(response.value) && response.value.length > 0);
  const identities = response.value.map(row => {
    assert.equal(row.admission.identity.run_id, w.telemetryRun);
    return `${row.admission.identity.call_id}/${row.admission.identity.attempt}`;
  });
  assert.equal(new Set(identities).size, identities.length, 'Duplicate accounting call identity');
  return response.value.sort((a, b) => canonical(a.admission.identity).localeCompare(canonical(b.admission.identity)));
}
function samples(raw, name) {
  return raw.split('\n').filter(line => line.startsWith(`${name}{`)).map(line => {
    const match = /^([^\s{]+)\{((?:[^"{}]|"(?:\\.|[^"\\])*")*)\}\s+([^\s]+)/.exec(line);
    assert.ok(match, 'Unsupported Prometheus sample');
    const labels = Object.fromEntries([...match[2].matchAll(/([a-zA-Z_][a-zA-Z0-9_]*)="((?:\\.|[^"\\])*)"/g)]
      .map(([, key, value]) => [key, JSON.parse(`"${value}"`)]));
    const value = Number(match[3]); assert.ok(Number.isFinite(value));
    return { labels, value };
  });
}
function routeCount(raw, template) {
  return samples(raw, 'uar_requests_total').filter(sample => sample.labels.path === template && sample.labels.method === 'GET')
    .reduce((sum, sample) => sum + sample.value, 0);
}
function terminalCost(w, raw) {
  const result = {};
  for (const suffix of ['count', 'sum']) {
    const selected = samples(raw, `uar_llm_cost_usd_${suffix}`).filter(sample =>
      sample.labels.provider === w.telemetryFixture.costMetricProvider && sample.labels.model === w.telemetryFixture.costMetricModel);
    assert.equal(selected.length, 1, 'A configured priced synthetic run must yield one observable terminal cost series');
    result[suffix] = selected[0].value;
  }
  assert.ok(result.count > 0 && result.sum > 0, 'Absent/zero cost cannot certify replay invariance');
  return result;
}
async function metrics(w) {
  const response = await http(w, w.telemetryBase, '/metrics');
  assert.equal(response.status, 200, 'Selected owned lane must own its recorder');
  return response.raw;
}
Given('the externally provisioned telemetry journals and isolated synthetic agent', async function () {
  assert.equal(poisoned, false, 'Prior uncertain owned cancellation requires external reconciliation');
  assert.ok(process.env.UAR_BASE_URL && process.env.UAR_WORKSPACE_ID && process.env.UAR_TELEMETRY_FIXTURE);
  const fixture = JSON.parse(await readFile(process.env.UAR_TELEMETRY_FIXTURE, 'utf8'));
  assert.equal(fixture.dedicated, true); assert.equal(fixture.syntheticProviderOnly, true);
  assert.equal(fixture.logLevel, 'debug', 'Privacy coverage requires externally enabled debug diagnostics');
  assert.match(fixture.namespace, /^[A-Za-z0-9_-]{1,96}$/);
  assert.match(fixture.identity.sourceCommit, /^[a-f0-9]{40}$/);
  assert.match(fixture.identity.imageDigest, /^sha256:[a-f0-9]{64}$/);
  assert.match(fixture.identity.configDigest, /^[a-f0-9]{64}$/);
  assert.equal(fixture.identity.platform, 'linux/amd64');
  assert.ok(fixture.agentId.startsWith(`${fixture.namespace}-`) && fixture.toolName.endsWith('telemetry_echo'));
  assert.ok(fixture.model.includes('/') && fixture.costMetricProvider && fixture.costMetricModel);
  this.telemetryFixture = fixture; this.telemetryBase = local(process.env.UAR_BASE_URL);
  this.telemetryForeign = local(fixture.foreignRecorderBaseUrl);
  assert.notEqual(this.telemetryForeign.href, this.telemetryBase.href, 'Foreign globals need their own externally owned process');
  this.telemetryWorkspace = process.env.UAR_WORKSPACE_ID; this.telemetryOps = 0;
  this.telemetryReceipt = { namespace: fixture.namespace, identity: fixture.identity,
    authentication_supplied_by_test: false, external_services_started: false, exchanges: [],
    unsupportedClaims: ['hard_exporter_thread_release', 'zero_span_loss', 'global_exactly_once_effects',
      'terminal_cost_histogram_is_idempotent_billing', 'restart_or_durable_event_replay_certification'] };
  this.telemetryReceiptPath = resolve(`.prometheus/verification/telemetry-ownership/${randomUUID()}.json`);
  this.telemetryJournals = {};
  for (const name of ['log', 'otlp', 'peer']) this.telemetryJournals[name] = await journalStart(fixture.journals[name]);
});
When('the mounted synthetic telemetry run reaches {string} after a retry and tool', async function (terminal) {
  assert.ok(['done', 'cancelled'].includes(terminal));
  this.telemetryCase = randomUUID(); this.telemetryTerminal = terminal;
  this.telemetrySentinels = [`TELEMETRY_PROMPT_${this.telemetryCase}`, `TELEMETRY_TOOL_ARG_${this.telemetryCase}`,
    `TELEMETRY_TOOL_RESULT_${this.telemetryCase}`];
  // Positive leak controls exercise the exact scanner against each private shape.
  for (const sentinel of this.telemetrySentinels) assert.throws(() => privatePayloadAbsent(JSON.stringify({ content: sentinel }), [sentinel]));
  const response = await http(this, this.telemetryBase, '/api/uar/runs', 'POST', {
    agent_id: this.telemetryFixture.agentId, model: this.telemetryFixture.model, session_id: randomUUID(),
    input: `UAR_TELEMETRY_CASE:${this.telemetryCase}:${terminal} ${this.telemetrySentinels[0]}`,
  });
  assert.equal(response.status, 200); assert.equal(typeof response.value.run_id, 'string');
  this.telemetryRun = response.value.run_id; this.telemetryRunActive = true;
  if (terminal === 'cancelled') {
    await waitFor('post-tool provider hold', () => peerRows(this), rows => rows.some(row => row.event === 'provider_held'));
    const cancelled = await http(this, this.telemetryBase, `/api/uar/runs/${encodeURIComponent(this.telemetryRun)}/cancel`, 'POST', {});
    assert.equal(cancelled.status, 200); assert.equal(cancelled.value.cancelled, true);
  }
  await waitFor('native terminal', async () => {
    const result = await http(this, this.telemetryBase, `/api/uar/runs/${encodeURIComponent(this.telemetryRun)}`);
    assert.equal(result.status, 200); return result.value;
  }, run => run.status === terminal);
  this.telemetryRunActive = false;
  const rows = await waitFor('actual tool/result and provider terminal', () => peerRows(this), rows =>
    rows.some(row => row.event === (terminal === 'done' ? 'provider_completed' : 'provider_cancelled')));
  const arrivals = rows.filter(row => row.event === 'provider_arrival');
  assert.deepEqual(arrivals.map(row => row.providerAttempt), [1, 2, 3], 'Expected only retry, tool request and post-tool request');
  assert.equal(rows.filter(row => row.event === 'provider_retryable').length, 1);
  assert.equal(rows.filter(row => row.event === 'provider_tool_intent').length, 1);
  const tool = rows.filter(row => row.event === 'tool_executed'); assert.equal(tool.length, 1);
  assert.equal(tool[0].argumentSha256, sha(this.telemetrySentinels[1]));
  assert.equal(tool[0].resultSha256, sha(this.telemetrySentinels[2]));
  assert.ok(rows.some(row => row.event === 'provider_tool_result_received' && row.resultSha256 === sha(this.telemetrySentinels[2])));
  assert.ok(rows.filter(row => row.event === 'provider_arrival').every(row => row.promptSha256 === sha(this.telemetrySentinels[0])));
  this.telemetryToolId = tool[0].toolId;
  this.telemetryReceipt.run = { runId: this.telemetryRun, terminal, toolId: this.telemetryToolId,
    sourcePayloadHashes: this.telemetrySentinels.map(sha), peerJournalSha256: sha(canonical(rows)) };
});
Then('raw log and OTLP exports correlate its run request attempt and tool without private payloads', async function () {
  const observed = await waitFor('correlated log and OTLP export', async () => {
    const log = await journalSince(this.telemetryJournals.log);
    const otlp = await journalSince(this.telemetryJournals.otlp);
    privatePayloadAbsent(log.raw, this.telemetrySentinels); privatePayloadAbsent(otlp.raw, this.telemetrySentinels);
    const logs = logRows(log.rows).filter(row => row.run_id === this.telemetryRun);
    const traces = otlpRows(otlp.rows).filter(row => row.run_id === this.telemetryRun);
    const retry = logs.find(row => String(row.message).includes('retrying before semantic events') && Number(row.attempt) === 1);
    const tool = logs.find(row => row.tool_id === this.telemetryToolId && row.tool_name === this.telemetryFixture.toolName);
    return { log, otlp, logs, traces, retry, tool };
  }, value => value.retry?.request_id && value.tool?.request_id === value.retry.request_id
    && value.traces.some(row => row.request_id === value.retry.request_id && Number(row.attempt) === 1)
    && value.traces.some(row => row.request_id === value.retry.request_id && row.tool_id === this.telemetryToolId)
    && value.traces.some(row => row.spanName === 'run' && Number(row.endTimeUnixNano) > 0));
  assert.ok(observed.logs.some(row => ['DEBUG', 'TRACE'].includes(row.level)), 'Filtered info-only logs cannot establish debug payload privacy');
  assert.ok(observed.traces.every(row => row.serviceName === 'universal-agent-runtime'));
  this.telemetryReceipt.correlation = { requestId: observed.retry.request_id, retryAttempt: observed.retry.attempt,
    runId: this.telemetryRun, toolId: this.telemetryToolId, logSha256: sha(observed.log.raw), otlpSha256: sha(observed.otlp.raw),
    logRecords: observed.logs.length, exportedSpanEventRecords: observed.traces.length, privatePayloadsAbsent: true };
});
Then('distinct resource IDs and queries share one bounded request metric route', async function () {
  const template = '/api/uar/runs/{id}';
  const before = await metrics(this);
  const ids = [this.telemetryRun, randomUUID()];
  const querySentinel = `TELEMETRY_QUERY_${randomUUID()}`;
  for (const [index, id] of ids.entries()) {
    const response = await http(this, this.telemetryBase, `/api/uar/runs/${encodeURIComponent(id)}?probe=${querySentinel}`);
    assert.equal(response.status, index === 0 ? 200 : 404);
  }
  const after = await metrics(this);
  assert.equal(routeCount(after, template) - routeCount(before, template), 2);
  for (const id of ids) assert.ok(!after.includes(id), 'Raw run ID entered metrics');
  assert.ok(!after.includes(querySentinel), 'Query entered metrics');
  this.telemetryReceipt.metricRoute = { template, distinctResources: 2, completedRequestsDelta: 2, rawResourceAndQueryAbsent: true };
});
Then('read-only event replay preserves durable call accounting peer effects and terminal cost observations', async function () {
  const before = await ledger(this);
  assert.ok(before.some(row => row.state === 'settled' && row.observed_tokens > 0));
  if (this.telemetryTerminal === 'cancelled') assert.ok(before.some(row => row.state === 'pending'), 'Held request must retain unresolved liability');
  const beforePeer = await peerRows(this);
  const beforeCost = terminalCost(this, await metrics(this));
  const snapshots = [];
  for (let count = 0; count < 2; count++) {
    const snapshot = await http(this, this.telemetryBase, `/api/uar/runs/${encodeURIComponent(this.telemetryRun)}/events?after=0`);
    assert.equal(snapshot.status, 200); assert.equal(snapshot.value.runId, this.telemetryRun);
    assert.equal(snapshot.value.gapReason, null); assert.ok(snapshot.value.events.length > 0);
    assert.equal(snapshot.value.retention, 'process-local-bounded'); snapshots.push(snapshot.value);
  }
  assert.deepEqual(snapshots[1], snapshots[0]);
  assert.deepEqual(await ledger(this), before, 'Read replay changed durable usage/call identities');
  assert.deepEqual(await peerRows(this), beforePeer, 'Read replay sent provider work or repeated a tool');
  assert.deepEqual(terminalCost(this, await metrics(this)), beforeCost, 'Read replay repeated terminal cost observation');
  const log = await journalSince(this.telemetryJournals.log), otlp = await journalSince(this.telemetryJournals.otlp);
  privatePayloadAbsent(log.raw, this.telemetrySentinels); privatePayloadAbsent(otlp.raw, this.telemetrySentinels);
  this.telemetryReceipt.replay = { observedReadCount: 2, snapshotSha256: sha(canonical(snapshots[0])),
    ledgerSha256: sha(canonical(before)), callStates: before.map(row => ({ callId: row.admission.identity.call_id,
      attempt: row.admission.identity.attempt, state: row.state, observedTokens: row.observed_tokens })),
    terminalCostObservation: beforeCost, costSemantics: 'terminal_run_total_operational_observation_not_billing',
    providerOrToolReplay: false, durableAccountingChanged: false };
});
Then('the owned scrape contains activity and the foreign-recorder scrape reports host ownership', async function () {
  const activity = await http(this, this.telemetryBase, '/api/uar/runs'); assert.equal(activity.status, 200);
  assert.ok((await metrics(this)).includes('uar_requests_total'));
  const foreign = await http(this, this.telemetryForeign, '/metrics');
  assert.equal(foreign.status, 503); assert.equal(foreign.value.code, 'telemetry_scrape_unavailable');
  assert.equal(foreign.value.metrics_status, 'host_owned');
  assert.ok(!foreign.raw.includes('uar_requests_total'), 'Foreign scrape must not present a detached series');
  this.telemetryReceipt.scrapeOwnership = { own: 'installed_activity_observed', foreign: 'host_owned_503' };
});
After(async function ({ result }) {
  if (!this.telemetryReceipt) return;
  if (this.telemetryRunActive) {
    try {
      const response = await http(this, this.telemetryBase, `/api/uar/runs/${encodeURIComponent(this.telemetryRun)}/cancel`, 'POST', {});
      assert.equal(response.status, 200);
      await waitFor('owned cleanup cancellation', async () => {
        const run = await http(this, this.telemetryBase, `/api/uar/runs/${encodeURIComponent(this.telemetryRun)}`);
        assert.equal(run.status, 200); return run.value.status;
      }, status => ['done', 'error', 'cancelled'].includes(status));
      this.telemetryReceipt.cleanup = 'owned_run_terminal_observed';
    } catch { poisoned = true; this.telemetryReceipt.cleanup = 'unconfirmed_external_reconciliation_required'; }
  }
  this.telemetryReceipt.outcome = result?.status ?? 'UNKNOWN';
  await mkdir(dirname(this.telemetryReceiptPath), { recursive: true });
  await writeFile(this.telemetryReceiptPath, `${JSON.stringify(this.telemetryReceipt, null, 2)}\n`, { flag: 'wx', mode: 0o600 });
});
