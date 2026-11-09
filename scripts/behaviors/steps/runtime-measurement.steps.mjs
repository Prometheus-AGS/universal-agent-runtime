import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { validate, dimensions, canonical, digest, boundedBytes, journal, percentiles, measured,
  unavailable, localUrl, observerMeasurements, reliabilityEvidence } from '../support/runtime-measurement-source.mjs';

// Cucumber registers a timeout before steps execute. It comes from the same
// owner manifest, not an invented benchmark/soak duration or default SLO.
assert.ok(process.env.UAR_RUNTIME_MEASUREMENT_MANIFEST, 'Owner-selected measurement manifest is required');
const registeredManifestBytes = readFileSync(process.env.UAR_RUNTIME_MEASUREMENT_MANIFEST);
const registeredManifest = JSON.parse(registeredManifestBytes.toString('utf8'));
assert.ok(Number.isSafeInteger(registeredManifest.limits?.frameworkTimeoutMs) && registeredManifest.limits.frameworkTimeoutMs > 0,
  'Owner frameworkTimeoutMs is required; UNSELECTED manifest is not runnable');
setDefaultTimeout(registeredManifest.limits.frameworkTimeoutMs);
async function safeJson(w, path, signal) {
  const response = await fetch(new URL(path, w.measurementBase), { redirect: 'error', signal,
    headers: { 'x-uar-workspace-id': w.measurementWorkspace } });
  const raw = await response.text();
  assert.ok(Buffer.byteLength(raw) <= w.measurement.limits.maxResponseBytes);
  return response.ok ? JSON.parse(raw) : null;
}
async function sample(w, task, concurrency, repetition, ordinal) {
  const m = w.measurement, requestId = randomUUID(), sessionId = randomUUID();
  const content = `${task.messages[0].content}\nUAR_MEASUREMENT:${m.selection.experimentId}:${task.id}:${requestId}`;
  // A present tools field selects caller-owned tool admission, even when empty.
  const body = { model: m.model.selector, agent_id: m.model.agentId, session_id: sessionId,
    client_turn_id: requestId, user_message_id: randomUUID(), stream: true, stream_mode: 'openai',
    temperature: m.settings.temperature, memory_enabled: false,
    prompt_caching_enabled: m.settings.promptCachingEnabled, max_completion_tokens: m.tokenBudgets.output,
    messages: [{ role: 'user', content }] };
  const serialized = JSON.stringify(body);
  const started = performance.now(), deadline = Math.min(w.measurementDeadline - performance.now(), m.limits.requestTimeoutMs);
  const row = { requestId, sessionId, taskId: task.id, concurrency, repetition, ordinal, startUnixMs: Date.now(),
    requestSha256: digest(serialized), expectedTextSha256: digest(task.expectedText), events: [],
    firstResponseMs: null, completeMs: null, nativeRunId: null, nativeState: null, cancellation: unavailable('native terminal not yet reconciled'),
    responseBytes: 0, statusCode: null, outcome: 'unconfirmed' };
  let output = '', finished = false, terminalSeen = false;
  const operationSignal = AbortSignal.timeout(Math.max(1, Math.ceil(deadline)));
  try {
    assert.ok(deadline > 0, 'Selected experiment duration exhausted');
    assert.ok(Buffer.byteLength(serialized) <= m.limits.maxRequestBytes, 'Selected request size budget exceeded');
    const response = await fetch(new URL('/v1/chat/completions', w.measurementBase), {
      method: 'POST', redirect: 'error', signal: operationSignal,
      headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': w.measurementWorkspace }, body: serialized,
    });
    row.statusCode = response.status;
    assert.equal(response.status, 200, 'Mounted measurement request did not succeed');
    assert.ok(response.headers.get('content-type')?.startsWith('text/event-stream'), 'Headers are not semantic first response');
    const reader = response.body.getReader(), decoder = new TextDecoder();
    let pending = '';
    const consume = block => {
      const lines = block.split(/\r?\n/);
      const eventName = lines.find(line => line.startsWith('event:'))?.slice(6).trim() ?? 'message';
      const data = lines.filter(line => line.startsWith('data:')).map(line => line.slice(5).trimStart()).join('\n');
      if (!data) return;
      const elapsedMs = performance.now() - started;
      if (data === '[DONE]') {
        assert.equal(terminalSeen, false, 'Duplicate terminal marker is not a complete response');
        row.events.push({ eventName, kind: 'terminal', elapsedMs, dataSha256: digest(data) });
        row.completeMs = elapsedMs; terminalSeen = true; return;
      }
      assert.equal(terminalSeen, false, 'Data after terminal is not a complete response');
      const value = JSON.parse(data);
      if (value.usage) row.wireUsageObservation = { basis: 'wire_usage_observation_not_billing',
        fields: Object.fromEntries(['prompt_tokens', 'completion_tokens', 'total_tokens'].filter(key => Number.isSafeInteger(value.usage[key]) && value.usage[key] >= 0)
          .map(key => [key, value.usage[key]])), payloadSha256: digest(canonical(value.usage)) };
      assert.ok(!value.error && eventName !== 'error', 'Streamed measurement reported an error');
      const text = (value.choices ?? []).map(choice => choice.delta?.content ?? '').join('');
      const toolSemantic = (value.choices ?? []).some(choice => choice.delta?.tool_calls?.some(call => call.function?.name || call.function?.arguments));
      const semantic = text.length > 0 || toolSemantic;
      if (semantic && row.firstResponseMs === null) row.firstResponseMs = elapsedMs;
      assert.equal(toolSemantic, false, 'Selected baseline cannot silently add tool-work semantics');
      output += text;
      if ((value.choices ?? []).some(choice => choice.finish_reason === 'stop')) finished = true;
      row.events.push({ eventName, kind: semantic ? 'semantic_text' : 'metadata', elapsedMs, dataSha256: digest(data) });
    };
    try {
      while (true) {
        const chunk = await reader.read();
        if (chunk.done) { pending += decoder.decode(); break; }
        row.responseBytes += chunk.value.byteLength;
        assert.ok(row.responseBytes <= m.limits.maxResponseBytes, 'Selected response byte budget exceeded');
        pending += decoder.decode(chunk.value, { stream: true });
        let match;
        while ((match = /\r?\n\r?\n/.exec(pending))) {
          const block = pending.slice(0, match.index); pending = pending.slice(match.index + match[0].length); consume(block);
        }
      }
      if (pending.trim()) consume(pending);
    } finally {
      try { await reader.cancel(); row.readerRelease = 'observed'; }
      catch { row.readerRelease = 'unconfirmed'; }
    }
    assert.ok(terminalSeen && finished && row.firstResponseMs !== null, 'No complete semantic response: partial stream is not completion');
    assert.equal(output, task.expectedText, 'Deterministic task output mismatch');
    row.outcome = 'success';
  } catch (error) {
    row.outcome = 'error';
    row.errorKind = operationSignal.aborted ? 'request_or_experiment_deadline' : error?.name === 'AssertionError' ? 'wire_or_task_assertion' : 'transport_or_parse_error';
  }
  row.elapsedToReadEndMs = performance.now() - started; row.endUnixMs = Date.now(); row.outputSha256 = digest(output);
  return row;
}
async function reconcile(w, row) {
  const m = w.measurement, requestId = row.requestId, sessionId = row.sessionId;
  // This is outside timed stream latency. Read-only original-intent lookup gives
  // cancellation truth; an HTTP/client abort never becomes a fabricated cancel.
  try {
    const receipt = await safeJson(w, `/api/uar/chat-turns/${encodeURIComponent(requestId)}?session_id=${encodeURIComponent(sessionId)}`,
      AbortSignal.timeout(m.limits.requestTimeoutMs));
    if (receipt?.client_turn_id === requestId && receipt.session_id === sessionId && typeof receipt.run_id === 'string') {
      row.nativeRunId = receipt.run_id; row.nativeState = receipt.state;
      row.cancellation = ['complete', 'error', 'cancelled'].includes(receipt.state)
        ? measured(receipt.state === 'cancelled', 'owner_scoped_native_terminal_receipt')
        : unavailable(`native outcome ${receipt.state} remains nonterminal/unknown`);
    }
  } catch { row.cancellation = unavailable('native terminal lookup unavailable'); }
  if (row.outcome !== 'success' && !['complete', 'error', 'cancelled'].includes(row.nativeState)) {
    w.measurementContaminated = true;
    if (!row.nativeRunId) { row.cleanup = { terminalReleaseConfirmed: false, reason: 'unknown admission; original intent retained, no fresh execution' }; return; }
    // Cancel only this measured owned run; do not detach background work and
    // silently contaminate later concurrency rounds. No restart/service control.
    try {
      const response = await fetch(new URL(`/api/uar/runs/${encodeURIComponent(row.nativeRunId)}/cancel`, w.measurementBase), {
        method: 'POST', redirect: 'error', signal: AbortSignal.timeout(m.limits.requestTimeoutMs),
        headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': w.measurementWorkspace }, body: '{}',
      });
      row.cleanup = { requestedOwnedCancellation: true, httpStatus: response.status, terminalReleaseConfirmed: false };
    } catch { row.cleanup = { requestedOwnedCancellation: true, terminalReleaseConfirmed: false }; }
    w.measurementContaminated = true;
  }
}
function summarize(samples, wallMs) {
  const successful = samples.filter(row => row.outcome === 'success');
  const first = percentiles(successful.map(row => row.firstResponseMs)), complete = percentiles(successful.map(row => row.completeMs));
  return {
    firstResponseP50Ms: first.p50, firstResponseP95Ms: first.p95, firstResponseP99Ms: first.p99,
    completeP50Ms: complete.p50, completeP95Ms: complete.p95, completeP99Ms: complete.p99,
    throughputPerSecond: samples.length && wallMs > 0 ? measured(successful.length / (wallMs / 1000), 'successful_terminal_responses_per_measured_round_wall_second', samples.length) : unavailable('no attempted round samples'),
    errorRate: samples.length ? measured(samples.filter(row => row.outcome === 'error').length / samples.length, 'all_attempted_requests', samples.length) : unavailable('no attempted round samples'),
    cancelRate: samples.length > 0 && samples.every(row => row.cancellation.availability === 'measured')
      ? measured(samples.filter(row => row.cancellation.value).length / samples.length, 'actual_native_terminal_cancellations_per_attempt', samples.length)
      : unavailable('one or more native terminal outcomes unavailable; client abort is not cancel truth'),
  };
}
Given('a complete owner-selected packaged-runtime measurement manifest', async function () {
  this.measurementReceiptPath = resolve(`.prometheus/verification/runtime-measurement/${randomUUID()}.json`);
  this.measurementReceipt = { kind: 'uar.runtime-measurement-result/v1', outcome: 'selection_unconfirmed', samples: [], rounds: [],
    authentication_supplied_by_test: false, services_started: false, paid_calls_authorized: false,
    quality: 'unverified_real_model_not_selected', comparison: 'NONE', pureFrameworkOverhead: 'not_established' };
  assert.ok(process.env.UAR_RUNTIME_MEASUREMENT_MANIFEST && process.env.UAR_BASE_URL && process.env.UAR_WORKSPACE_ID,
    'Owner manifest and externally provisioned credential-free gateway/workspace are required');
  const bytes = await boundedBytes(process.env.UAR_RUNTIME_MEASUREMENT_MANIFEST, 4 * 1024 * 1024);
  this.measurement = validate(JSON.parse(bytes.toString('utf8')));
  assert.equal(digest(bytes), digest(registeredManifestBytes), 'Owner selection changed after timeout registration');
  assert.ok(this.measurement.limits.maxExperimentMs + this.measurement.limits.observationWaitMs
    + 2 * this.measurement.limits.maxRequests * this.measurement.limits.requestTimeoutMs < this.measurement.limits.frameworkTimeoutMs,
  'Owner framework timeout must cover selected experiment and bounded per-request reconciliation');
  this.measurementBase = localUrl(process.env.UAR_BASE_URL);
  assert.equal(this.measurementBase.href, localUrl(this.measurement.profile.gateway).href, 'Gateway differs from frozen owner profile');
  this.measurementWorkspace = process.env.UAR_WORKSPACE_ID;
  this.measurementReceipt.selectionSha256 = digest(bytes); this.measurementReceipt.frozenSelection = this.measurement;
  this.measurementReceipt.percentileConvention = 'nearest rank, successful semantic samples only; sample counts retained';
  this.measurementReceipt.identity = this.measurement.identity;
  this.measurementReceipt.tokenEvidence = 'synthetic_reported_usage_only_no_real_token_fit_guarantee';
  // All selection validation precedes the first mounted HTTP request.
  this.measurementPeerBaseline = await journal(this.measurement.peerJournal, this.measurement.limits.maxJournalBytes);
  this.measurementObserverBaseline = await journal(this.measurement.observer.journalPath, this.measurement.limits.maxJournalBytes);
  assert.ok(this.measurementPeerBaseline.rows.some(row => row.event === 'peer_started'
    && row.selectionSha256 === this.measurementReceipt.selectionSha256), 'External synthetic peer must use the exact frozen selection');
});
When('the selected streaming workload is measured at its declared concurrency and repetitions', async function () {
  const m = this.measurement;
  this.measurementWindow = { startUnixMs: Date.now() };
  this.measurementDeadline = performance.now() + m.limits.maxExperimentMs;
  for (const concurrency of m.concurrency) {
    for (let repetition = 0; repetition < m.repetitions; repetition++) {
      assert.ok(!this.measurementContaminated, 'Unconfirmed owned work prevents a new measured round');
      assert.ok(performance.now() < this.measurementDeadline, 'Selected total experiment deadline exhausted');
      const started = performance.now(), startUnixMs = Date.now(), rows = []; let next = 0, active = 0, peak = 0;
      const worker = async () => {
        while (next < m.requestsPerRound) {
          if (this.measurementContaminated || performance.now() >= this.measurementDeadline) return;
          const index = next++; active++; peak = Math.max(peak, active);
          try { rows.push(await sample(this, m.workload[index % m.workload.length], concurrency, repetition, index)); }
          finally { active--; }
        }
      };
      await Promise.all(Array.from({ length: concurrency }, worker));
      const wallMs = performance.now() - started, endUnixMs = Date.now();
      this.measurementWindow.endUnixMs = endUnixMs;
      // Native read/cancel observation occurs after the timed round, so it does
      // not insert hidden per-request lookup delays into measured throughput.
      for (const row of rows) await reconcile(this, row);
      rows.sort((a, b) => a.ordinal - b.ordinal);
      this.measurementReceipt.samples.push(...rows);
      this.measurementReceipt.rounds.push({ concurrency, repetition, attempted: rows.length, declared: m.requestsPerRound,
        window: { startUnixMs, endUnixMs }, clientPeakInFlight: peak, clientInitialBacklog: Math.max(0, m.requestsPerRound - concurrency), wallMs,
        metrics: summarize(rows, wallMs), completeSelectionExecuted: rows.length === m.requestsPerRound });
      assert.equal(rows.length, m.requestsPerRound, 'Partial selection is retained but cannot qualify');
    }
  }
});
Then('raw measurements retain semantic latencies provider time startup resource and reliability availability', async function () {
  const m = this.measurement;
  assert.ok(this.measurementWindow?.endUnixMs);
  const deadline = Date.now() + m.limits.observationWaitMs;
  let observer, peer;
  do {
    observer = await journal(m.observer.journalPath, m.limits.maxJournalBytes);
    peer = await journal(m.peerJournal, m.limits.maxJournalBytes);
    const complete = peer.rows.filter(row => row.event === 'response_completed' && row.selectionSha256 === this.measurementReceipt.selectionSha256);
    const resourcesSelected = ['cpuCoresUsed', 'rssPeakBytes', 'runtimeQueuePeak', 'runtimeBackpressureDelta']
      .some(name => m.thresholds[name].decision === 'selected');
    const resourcesPublished = !resourcesSelected || observer.rows.some(row => row.kind === 'resource_sample'
      && row.experimentId === m.selection.experimentId && row.timeUnixMs >= this.measurementWindow.endUnixMs - m.observer.maxClockSkewMs
      && observer.rows.some(boundary => boundary.kind === 'observation_boundary' && boundary.status === 'OBSERVED' && boundary.observationId === row.observationId));
    if (resourcesPublished && this.measurementReceipt.samples.filter(row => row.outcome === 'success').every(sample => complete.some(row => row.requestId === sample.requestId))) break;
    await delay(m.limits.pollIntervalMs);
  } while (Date.now() < deadline);
  for (const [current, previous] of [[peer, this.measurementPeerBaseline], [observer, this.measurementObserverBaseline]]) {
    assert.ok(current.bytes.length >= previous.bytes.length);
    assert.equal(digest(current.bytes.subarray(0, previous.bytes.length)), digest(previous.bytes), 'External observer/peer journal was replaced or rewritten');
  }
  this.measurementReceipt.observerJournalSha256 = digest(observer.bytes);
  this.measurementReceipt.providerJournalSha256 = digest(peer.bytes);
  this.measurementReceipt.observer = observerMeasurements(observer.rows, m, this.measurementWindow);
  for (const round of this.measurementReceipt.rounds) round.observer = observerMeasurements(observer.rows, m, round.window);
  const samePeer = peer.rows.filter(row => row.selectionSha256 === this.measurementReceipt.selectionSha256
    && row.experimentId === m.selection.experimentId);
  for (const sample of this.measurementReceipt.samples) {
    const rows = samePeer.filter(row => row.requestId === sample.requestId);
    const completed = rows.filter(row => row.event === 'response_completed');
    const received = rows.filter(row => row.event === 'request_received');
    sample.providerObservations = rows;
    sample.providerTime = completed.length === 1 && received.length === 1 && Number.isFinite(completed[0].elapsedMs)
      ? measured(completed[0].elapsedMs, 'synthetic_peer_receive_to_response_end_local_monotonic')
      : unavailable('no unique complete provider service interval; retries/interruptions are not pure overhead samples');
    sample.endToEndResidual = sample.outcome === 'success' && sample.providerTime.availability === 'measured'
      && sample.completeMs >= sample.providerTime.value
      ? measured(sample.completeMs - sample.providerTime.value, 'end_to_end_minus_provider_service_includes_gateway_network_runtime_serialization')
      : unavailable('missing/noncomparable provider interval; no framework overhead claim');
    if (sample.outcome === 'success') {
      assert.ok(received.some(row => row.selectedInputSha256 === digest(m.workload.find(task => task.id === sample.taskId).messages[0].content)),
        'Synthetic selected input was not independently observed');
    }
  }
  this.measurementReceipt.reliability = await reliabilityEvidence(m);
  this.measurementReceipt.responseBackpressure = {
    status429: this.measurementReceipt.samples.filter(row => row.statusCode === 429).length,
    status503: this.measurementReceipt.samples.filter(row => row.statusCode === 503).length,
    limitation: 'HTTP429 is observed refusal; HTTP503 is not automatically classified as runtime queue backpressure',
  };
});
Then('only selected thresholds with available evidence may qualify', function () {
  const m = this.measurement, findings = [];
  for (const round of this.measurementReceipt.rounds) {
    for (const name of dimensions) {
      const threshold = m.thresholds[name]; if (threshold.decision !== 'selected') continue;
      const value = round.metrics[name] ?? round.observer[name];
      if (value?.availability !== 'measured' || !Number.isFinite(value.value)) {
        findings.push({ concurrency: round.concurrency, repetition: round.repetition, metric: name, outcome: 'BLOCKED', reason: value?.reason ?? 'unavailable' }); continue;
      }
      const pass = threshold.operator === 'lte' ? value.value <= threshold.value : value.value >= threshold.value;
      findings.push({ concurrency: round.concurrency, repetition: round.repetition, metric: name,
        outcome: pass ? 'PASS' : 'FAIL', observed: value.value, operator: threshold.operator, selectedThreshold: threshold.value });
    }
  }
  for (const [name, selection] of Object.entries(m.reliability)) {
    if (selection.decision === 'selected') findings.push({ reliability: name,
      outcome: this.measurementReceipt.reliability[name]?.availability === 'measured' ? 'PASS' : 'BLOCKED',
      reason: this.measurementReceipt.reliability[name]?.reason });
  }
  this.measurementReceipt.thresholdFindings = findings;
  this.measurementReceipt.outcome = findings.some(row => row.outcome === 'FAIL') ? 'FAIL'
    : findings.some(row => row.outcome === 'BLOCKED') || this.measurementContaminated ? 'BLOCKED' : 'SELECTED_THRESHOLDS_MET';
  assert.equal(this.measurementReceipt.outcome, 'SELECTED_THRESHOLDS_MET', 'Retained negative/unavailable results cannot qualify selected promises');
});
After(async function ({ result }) {
  if (!this.measurementReceipt) return;
  this.measurementReceipt.frameworkStatus = result?.status ?? 'UNKNOWN';
  if (result?.status !== 'PASSED' && ['selection_unconfirmed', 'SELECTED_THRESHOLDS_MET'].includes(this.measurementReceipt.outcome)) {
    this.measurementReceipt.outcome = 'UNVERIFIED_OR_FAILED';
  }
  await mkdir(dirname(this.measurementReceiptPath), { recursive: true });
  await writeFile(this.measurementReceiptPath, `${JSON.stringify(this.measurementReceipt, null, 2)}\n`, { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify({ receipt: this.measurementReceiptPath, outcome: this.measurementReceipt.outcome }), 'application/json');
});
