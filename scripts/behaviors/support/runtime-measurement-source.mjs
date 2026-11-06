// Single-feature helpers only. No process/service management or product imports.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile, stat } from 'node:fs/promises';
import { isAbsolute } from 'node:path';
import { assertCandidateArtifactStable } from './candidate-artifact-binding.mjs';

export const dimensions = [
  'firstResponseP50Ms', 'firstResponseP95Ms', 'firstResponseP99Ms',
  'completeP50Ms', 'completeP95Ms', 'completeP99Ms', 'throughputPerSecond', 'errorRate', 'cancelRate',
  'coldStartupMs', 'warmStartupMs', 'cpuCoresUsed', 'rssPeakBytes', 'runtimeQueuePeak', 'runtimeBackpressureDelta',
];
export const reliabilityCases = Object.fromEntries(['cancellation', 'toolLoop', 'longConversation', 'interruptedStream', 'backendOutageRecovery', 'controlledRestart']
  .map(key => [key, 'scripts/behaviors/features/production-artifact-qualification.feature']));
export const digest = value => createHash('sha256').update(value).digest('hex');
export function canonical(value) {
  return JSON.stringify(value, (_key, item) => item && typeof item === 'object' && !Array.isArray(item)
    ? Object.fromEntries(Object.entries(item).sort(([a], [b]) => a.localeCompare(b))) : item);
}
export const unavailable = reason => ({ availability: 'unavailable', reason, value: null });
export const measured = (value, basis, count) => ({ availability: 'measured', value, basis, ...(count === undefined ? {} : { count }) });
export function localUrl(value) {
  const url = new URL(value);
  assert.ok(['http:', 'https:'].includes(url.protocol) && ['127.0.0.1', '[::1]'].includes(url.hostname));
  assert.ok(!url.username && !url.password && !url.search && !url.hash);
  return url;
}
const text = value => assert.ok(typeof value === 'string' && value.trim().length > 0, 'Required owner selection is missing');
const positive = value => assert.ok(Number.isSafeInteger(value) && value > 0, 'Required positive owner budget is missing');
const nonnegative = value => assert.ok(Number.isFinite(value) && value >= 0, 'Required finite owner selection is missing');
export function validate(manifest) {
  assert.equal(manifest.kind, 'uar.runtime-measurement/v1');
  assert.equal(manifest.selection?.status, 'selected', 'Measurement refuses an absent/UNSELECTED owner selection before HTTP');
  text(manifest.selection.ownerDecision);
  assert.match(manifest.selection.experimentId ?? '', /^[A-Za-z0-9_-]+$/);
  assert.equal(manifest.selection.externalSpendAllowed, false);
  assert.equal(manifest.selection.syntheticProviderOnly, true);
  assert.equal(manifest.selection.comparison, 'NONE'); text(manifest.selection.rigDisposition);
  const identity = manifest.identity;
  assert.match(identity?.sourceCommit ?? '', /^[a-f0-9]{40}$/);
  assert.match(identity?.imageDigest ?? '', /^sha256:[a-f0-9]{64}$/);
  assert.match(identity?.configDigest ?? '', /^[a-f0-9]{64}$/);
  assert.match(identity?.sourceInventoryDigest ?? '', /^[a-f0-9]{64}$/);
  assert.match(identity?.settingsDigest ?? '', /^[a-f0-9]{64}$/);
  assert.equal(identity.platform, 'linux/amd64');
  for (const key of ['description', 'cpuModel', 'kernel', 'os', 'containerRuntime']) text(manifest.hardware?.[key]);
  positive(manifest.hardware.allocatedCpuCount); positive(manifest.hardware.memoryLimitBytes);
  for (const key of ['bundle', 'persistence', 'gateway', 'durabilityDisclosure']) text(manifest.profile?.[key]);
  assert.equal(manifest.profile.bundle, 'server-full'); localUrl(manifest.profile.gateway);
  assert.equal(manifest.profile.egressPolicy, 'synthetic_peer_only', 'External host must isolate all outbound model traffic to the synthetic peer');
  text(manifest.model?.selector); text(manifest.model?.providerRevision); text(manifest.model?.modelRevision);
  assert.ok(manifest.model.selector.includes('/')); text(manifest.model.agentId); text(manifest.model.providerModelId);
  assert.match(manifest.model.agentArtifactSha256 ?? '', /^[a-f0-9]{64}$/);
  assert.match(manifest.model.providerSettingsSha256 ?? '', /^[a-f0-9]{64}$/);
  assert.equal(manifest.settings?.memoryEnabled, false);
  assert.equal(manifest.settings?.streamMode, 'openai');
  assert.equal(typeof manifest.settings.promptCachingEnabled, 'boolean');
  assert.ok(Number.isFinite(manifest.settings.temperature));
  assert.deepEqual(manifest.settings.tools, [], 'Tool reliability reuses existing receipts, not an unbounded benchmark agent');
  text(manifest.settings.retryDisclosure);
  assert.ok(Array.isArray(manifest.workload) && manifest.workload.length > 0);
  const taskIds = new Set();
  for (const task of manifest.workload) {
    assert.match(task.id ?? '', /^[A-Za-z0-9_-]+$/); assert.ok(!taskIds.has(task.id)); taskIds.add(task.id);
    assert.ok(Array.isArray(task.messages) && task.messages.length === 1, 'Baseline selects one synthetic user input; conversation reliability reuses existing cases');
    for (const message of task.messages) {
      assert.ok(['user', 'assistant'].includes(message.role)); text(message.content);
      assert.equal(Object.keys(message).sort().join(','), 'content,role');
    }
    assert.equal(task.messages.at(-1).role, 'user'); text(task.expectedText);
    assert.equal(task.synthetic, true); nonnegative(task.providerDelayMs);
    assert.ok(Array.isArray(task.responseChunks) && task.responseChunks.length > 0);
    for (const chunk of task.responseChunks) { text(chunk.text); nonnegative(chunk.delayMs); }
    assert.equal(task.responseChunks.map(chunk => chunk.text).join(''), task.expectedText);
    for (const key of ['prompt_tokens', 'completion_tokens', 'total_tokens']) {
      assert.ok(Number.isSafeInteger(task.reportedUsage[key]) && task.reportedUsage[key] >= 0);
    }
    assert.equal(task.reportedUsage.total_tokens, task.reportedUsage.prompt_tokens + task.reportedUsage.completion_tokens);
  }
  assert.equal(manifest.workloadSha256, digest(canonical(manifest.workload)), 'Workload/data/prompt/tool oracle changed after freeze');
  assert.ok(Array.isArray(manifest.concurrency) && manifest.concurrency.length > 0);
  manifest.concurrency.forEach(positive); assert.equal(new Set(manifest.concurrency).size, manifest.concurrency.length);
  positive(manifest.repetitions); positive(manifest.requestsPerRound);
  assert.ok(manifest.concurrency.every(level => manifest.requestsPerRound >= level));
  for (const key of ['maxRequests', 'maxExperimentMs', 'frameworkTimeoutMs', 'requestTimeoutMs', 'maxRequestBytes', 'maxResponseBytes', 'maxJournalBytes', 'observationWaitMs', 'pollIntervalMs', 'maxPeerRequests']) positive(manifest.limits?.[key]);
  assert.ok(manifest.concurrency.length * manifest.repetitions * manifest.requestsPerRound <= manifest.limits.maxRequests);
  for (const key of ['input', 'output']) positive(manifest.tokenBudgets?.[key]);
  assert.equal(manifest.tokenBudgets.reasoning, 0, 'Synthetic text baseline has no requested reasoning stream; owner must explicitly select zero');
  for (const task of manifest.workload) {
    assert.ok(task.reportedUsage.prompt_tokens <= manifest.tokenBudgets.input);
    assert.ok(task.reportedUsage.completion_tokens <= manifest.tokenBudgets.output);
  }
  text(manifest.tokenBudgets.inputEvidenceLimit);
  for (const phase of ['cold', 'warm']) text(manifest.startupDefinitions?.[phase]);
  assert.match(manifest.observer?.sourceRevision ?? '', /^(?:[a-f0-9]{40}|[a-f0-9]{64})$/); text(manifest.observer?.clockDefinition);
  nonnegative(manifest.observer.maxClockSkewMs); positive(manifest.observer.maxSampleGapMs);
  assert.ok(isAbsolute(manifest.observer.journalPath) && isAbsolute(manifest.peerJournal));
  assert.deepEqual(Object.keys(manifest.thresholds ?? {}).sort(), [...dimensions].sort(), 'Unknown/missing threshold dimension cannot be silently ignored');
  assert.deepEqual(Object.keys(manifest.reliability ?? {}).sort(), Object.keys(reliabilityCases).sort(), 'Unknown/missing reliability dimension cannot be silently ignored');
  let selected = 0;
  for (const dimension of dimensions) {
    const threshold = manifest.thresholds?.[dimension];
    assert.ok(threshold && ['selected', 'unselected'].includes(threshold.decision), `Missing explicit threshold choice: ${dimension}`);
    if (threshold.decision === 'unselected') { text(threshold.reason); continue; }
    assert.ok(['lte', 'gte'].includes(threshold.operator)); nonnegative(threshold.value); selected++;
  }
  assert.ok(selected > 0, 'No selected threshold: measurement promises remain unselected');
  for (const [key, sourceCase] of Object.entries(reliabilityCases)) {
    const selection = manifest.reliability?.[key];
    assert.ok(selection && ['selected', 'unselected'].includes(selection.decision), `Missing explicit reliability selection: ${key}`);
    assert.equal(selection.sourceCase, sourceCase, 'Only the fresh bound Linux artifact workflow is eligible; historical receipts remain unavailable');
    if (selection.decision === 'unselected') { text(selection.reason); continue; }
    assert.ok(isAbsolute(selection.receiptPath)); assert.match(selection.sha256, /^[a-f0-9]{64}$/);
    assert.match(selection.sourceCaseSha256, /^[a-f0-9]{64}$/);
  }
  return manifest;
}
export async function boundedBytes(path, limit) {
  const metadata = await stat(path);
  assert.ok(metadata.isFile() && metadata.size <= limit, 'Observer/receipt file missing or exceeds selected bound');
  const bytes = await readFile(path); assert.ok(bytes.length <= limit); return bytes;
}
export async function journal(path, limit) {
  const bytes = await boundedBytes(path, limit);
  const raw = bytes.toString('utf8');
  return { bytes, rows: raw.slice(0, raw.lastIndexOf('\n') + 1).split('\n').filter(Boolean).map(row => JSON.parse(row)) };
}
export function percentiles(values) {
  if (!values.length) return { p50: unavailable('no successful semantic samples'), p95: unavailable('no successful semantic samples'), p99: unavailable('no successful semantic samples') };
  const sorted = [...values].sort((a, b) => a - b);
  const q = percentile => measured(sorted[Math.max(0, Math.ceil(percentile * sorted.length) - 1)], 'nearest_rank_successful_samples', sorted.length);
  return { p50: q(0.5), p95: q(0.95), p99: q(0.99) };
}
function identityMatches(row, identity) {
  return ['sourceCommit', 'sourceInventoryDigest', 'imageDigest', 'configDigest', 'settingsDigest', 'platform'].every(key => row?.[key] === identity[key]);
}
export function observerMeasurements(rows, manifest, window) {
  const bound = rows.filter(row => row.kind === 'observation_boundary' && row.status === 'OBSERVED'
    && row.experimentId === manifest.selection.experimentId && row.observerRevision === manifest.observer.sourceRevision
    && identityMatches(row.candidateBinding?.before?.identity, manifest.identity)
    && identityMatches(row.candidateBinding?.after?.identity, manifest.identity));
  const observations = new Set(bound.filter(row => {
    try { assertCandidateArtifactStable(row.candidateBinding.before, row.candidateBinding.after, { allowRestart: row.mode !== 'resources' }); return true; }
    catch { return false; }
  }).map(row => row.observationId));
  const relevant = rows.filter(row => observations.has(row.observationId) && row.experimentId === manifest.selection.experimentId
    && identityMatches(row.identity, manifest.identity) && row.observerRevision === manifest.observer.sourceRevision);
  const startup = phase => {
    const definition = digest(manifest.startupDefinitions[phase]);
    const starts = relevant.filter(row => row.kind === 'startup_start' && row.phase === phase && row.definitionSha256 === definition);
    const pairs = starts.flatMap(start => {
      const ready = relevant.find(row => row.kind === 'startup_ready' && row.phase === phase && row.runtimeEpoch === start.runtimeEpoch
        && row.definitionSha256 === definition && row.endpoint === '/readyz' && row.statusCode === 200);
      const valid = ready && /^\d+$/.test(String(start.monotonicNs)) && /^\d+$/.test(String(ready.monotonicNs))
        && Number.isFinite(ready.timeUnixMs) && ready.timeUnixMs <= window.startUnixMs + manifest.observer.maxClockSkewMs;
      const elapsed = valid ? Number(BigInt(ready.monotonicNs) - BigInt(start.monotonicNs)) / 1e6 : NaN;
      return Number.isFinite(elapsed) && elapsed >= 0 ? [{ runtimeEpoch: start.runtimeEpoch, elapsedMs: elapsed }] : [];
    });
    return pairs.length ? { ...measured(Math.max(...pairs.map(pair => pair.elapsedMs)), `external_${phase}_startup_monotonic_start_to_ready_max`, pairs.length), observations: pairs, distribution: percentiles(pairs.map(pair => pair.elapsedMs)), definitionSha256: definition }
      : unavailable(`no exact-artifact external ${phase} startup pair matching owner definition`);
  };
  const skew = manifest.observer.maxClockSkewMs;
  const padding = manifest.observer.maxSampleGapMs + skew;
  const candidates = relevant.filter(row => row.kind === 'resource_sample' && Number.isFinite(row.timeUnixMs)
    && row.timeUnixMs >= window.startUnixMs - padding && row.timeUnixMs <= window.endUnixMs + padding).sort((a, b) => a.timeUnixMs - b.timeUnixMs);
  const first = candidates.findLastIndex(row => row.timeUnixMs <= window.startUnixMs + skew);
  const last = candidates.findIndex(row => row.timeUnixMs >= window.endUnixMs - skew);
  const samples = first >= 0 && last >= first ? candidates.slice(first, last + 1) : [];
  const spansWindow = samples.length >= 2 && typeof samples[0].runtimeEpoch === 'string'
    && samples.every(sample => sample.runtimeEpoch === samples[0].runtimeEpoch)
    && samples.slice(1).every((sample, index) => sample.timeUnixMs - samples[index].timeUnixMs <= manifest.observer.maxSampleGapMs);
  const dimension = (key, basis, aggregate) => {
    if (!spansWindow) return unavailable('external resource samples do not cover experiment interval/cadence');
    if (!samples.every(sample => sample[key]?.availability === 'measured' && sample[key].source === basis
      && Number.isSafeInteger(sample[key].value) && sample[key].value >= 0)) return unavailable(`raw ${key} unavailable or source differs`);
    const values = samples.map(sample => sample[key].value);
    if (['cpuUsageNs', 'runtimeBackpressureTotal'].includes(key) && values.slice(1).some((value, index) => value < values[index])) {
      return unavailable(`counter reset/nonmonotonic ${key} observation`);
    }
    const value = aggregate(values);
    return Number.isFinite(value) && value >= 0 ? measured(value, basis, values.length)
      : unavailable(`invalid/reset/nonmonotonic ${key} observation`);
  };
  const elapsedMs = samples.length >= 2 ? samples.at(-1).timeUnixMs - samples[0].timeUnixMs : 0;
  return {
    coldStartupMs: startup('cold'), warmStartupMs: startup('warm'),
    cpuCoresUsed: dimension('cpuUsageNs', 'process_cpu_cumulative_nanoseconds', values => {
      const delta = values.at(-1) - values[0];
      return delta >= 0 && elapsedMs > 0 ? delta / 1e6 / elapsedMs : NaN;
    }),
    rssPeakBytes: dimension('rssBytes', 'process_rss_bytes', values => Math.max(...values)),
    runtimeQueuePeak: dimension('runtimeQueueDepth', 'runtime_owned_queue_observation', values => Math.max(...values)),
    runtimeBackpressureDelta: dimension('runtimeBackpressureTotal', 'runtime_owned_backpressure_counter', values => values.at(-1) - values[0]),
    rawResourceSamples: samples,
  };
}
export async function reliabilityEvidence(manifest) {
  const result = {};
  for (const key of Object.keys(reliabilityCases)) {
    const selection = manifest.reliability[key];
    const sourceCase = selection.sourceCase;
    if (selection.decision === 'unselected') { result[key] = unavailable(`owner unselected: ${selection.reason}`); continue; }
    const source = await boundedBytes(sourceCase, manifest.limits.maxJournalBytes);
    assert.equal(digest(source), selection.sourceCaseSha256, 'Referenced existing case changed');
    const bytes = await boundedBytes(selection.receiptPath, manifest.limits.maxJournalBytes);
    assert.equal(digest(bytes), selection.sha256, 'Existing reliability receipt changed');
    const receipt = JSON.parse(bytes.toString('utf8'));
    if (!identityMatches(receipt.identity, manifest.identity)) {
      result[key] = unavailable('existing receipt has no matching current source/artifact/config/settings identity'); continue;
    }
    if ((receipt.status ?? receipt.outcome) !== 'PASSED' || (receipt.cleanup_errors?.length ?? 0) > 0
      || receipt.authentication_supplied_by_test !== false) {
      result[key] = unavailable('existing receipt failed/uncompleted/cleanup unconfirmed'); continue;
    }
    if (receipt.kind !== 'uar.production-artifact-qualification/v1' || receipt.sourceCase !== sourceCase
      || !identityMatches(receipt.candidateBinding?.before?.identity, manifest.identity)
      || !identityMatches(receipt.candidateBinding?.after?.identity, manifest.identity)) {
      result[key] = unavailable('fresh Linux artifact before/after binding is absent'); continue;
    }
    assertCandidateArtifactStable(receipt.candidateBinding.before, receipt.candidateBinding.after, { allowRestart: key === 'controlledRestart' });
    const row = receipt.reliability?.[key];
    const array = value => Array.isArray(value) && value.length > 0;
    const done = value => typeof value?.runId === 'string' && value.status === 'done';
    const canary = value => typeof value?.id === 'string' && /^[a-f0-9]{64}$/.test(value.contentSha256);
    let valid = false;
    if (key === 'cancellation') valid = row?.status === 'cancelled' && typeof row.runId === 'string'
      && done(row.followUp) && array(row.calls) && array(row.terminalEvents)
      && array(row.provider) && row.provider.every(item => item.closed === true && item.ended === false);
    if (key === 'toolLoop') valid = done(row) && array(row.providerRequests) && array(row.toolCalls)
      && row.toolCalls.every(item => item.phase === 'completed' && item.effect_performed === true && item.invocation_id && item.request_id)
      && array(row.effectRows) && array(row.accountingBefore)
      && canonical(row.accountingBefore) === canonical(row.accountingAfter)
      && canonical(row.providerRequests) === canonical(row.providerRequestsAfterReplay)
      && canonical(row.effectRows) === canonical(row.effectRowsAfterReplay);
    if (key === 'longConversation') valid = typeof row?.sessionId === 'string' && row.turns?.length >= 2
      && row.turns.every(item => done(item) && /^[a-f0-9]{64}$/.test(item.inputSha256))
      && array(row.canonicalHistoryMessageIds) && row.canonicalHistoryMessageIds.every(item => typeof item === 'string' && item.length > 0)
      && array(row.providerRequests) && row.providerRequests.every(item => item.requestId && array(item.messageSha256))
      && row.retainedInputSha256?.length === row.turns.length
      && row.turns.every(item => row.retainedInputSha256.includes(item.inputSha256)
        && row.providerRequests.at(-1).messageSha256.includes(item.inputSha256));
    if (key === 'interruptedStream') valid = row?.faulted?.status === 'error' && typeof row.faulted.runId === 'string'
      && array(row.faulted.calls) && array(row.faulted.terminalEvents) && done(row.followUp)
      && array(row.provider) && row.provider.every(item => item.interrupted && item.closed && !item.ended && item.chunks > 0)
      && row.faulted.terminalEvents.some(item => item.eventName === 'agui.error')
      && row.faulted.terminalEvents.every(item => item.eventName !== 'agui.done' || (item.data?.outcome?.type ?? item.data?.status) !== 'success');
    if (key === 'controlledRestart') valid = row?.prior?.running === true && row.stopped?.running === false
      && row.stopped.exit_code === 0 && row.restarted?.running === true
      && row.prior.image === row.restarted.image && row.prior.volume === row.restarted.volume
      && Date.parse(row.restarted.started_at) > Date.parse(row.prior.started_at)
      && canary(row.canaryBefore) && canonical(row.canaryBefore) === canonical(row.canaryAfter);
    if (key === 'backendOutageRecovery') valid = row?.runtimeBefore?.id === row?.runtimeAfter?.id
      && typeof row?.runtimeBefore?.id === 'string' && row.runtimeBefore.started_at === row.runtimeAfter.started_at
      && row.runtimeBefore.image === row.runtimeAfter.image && row.runtimeBefore.volume === row.runtimeAfter.volume
      && row.readiness?.some(item => item.status === 503) && row.readiness.at(-1).status === 200
      && row.transport?.lostIndex >= 0 && row.transport.recoveredIndex > row.transport.lostIndex
      && canary(row.canaryBefore) && canonical(row.canaryBefore) === canonical(row.canaryAfter) && typeof row.writeAfter?.id === 'string';
    result[key] = valid ? { availability: 'measured', sourceCase, receiptSha256: selection.sha256, facts: row,
      limitation: 'Selected deterministic artifact subcase only; no genuine summary quality, broad loss/duplicate-effect guarantee, real-model quality, process release or RSS recovery inferred' }
      : unavailable('fresh artifact receipt lacks actual supported subcase facts; generic PASS is insufficient');
  }
  return result;
}
