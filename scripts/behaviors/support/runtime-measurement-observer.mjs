// External observer SOURCE. It never starts/stops a container or changes runtime
// settings. Invocation requires separate named operator permission after gates open.
// argv: ABSOLUTE_SELECTED_MANIFEST ABSOLUTE_OBSERVER_SELECTION
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { createHash, randomUUID } from 'node:crypto';
import { appendFile, readFile, lstat } from 'node:fs/promises';
import { isAbsolute } from 'node:path';
import { fileURLToPath } from 'node:url';
import { setTimeout as delay } from 'node:timers/promises';
import { loadCandidateArtifactInputs, captureCandidateArtifact, assertCandidateArtifactStable, candidateSettingKeys } from './candidate-artifact-binding.mjs';
import { validate, canonical } from './runtime-measurement-source.mjs';
const sha = value => createHash('sha256').update(value).digest('hex');
const [manifestPath, selectionPath] = process.argv.slice(2);
assert.ok(isAbsolute(manifestPath) && isAbsolute(selectionPath));
const manifestBytes = await readFile(manifestPath), selectionBytes = await readFile(selectionPath);
const manifest = validate(JSON.parse(manifestBytes)), selection = JSON.parse(selectionBytes);
assert.equal(selection.kind, 'uar.runtime-measurement-observer-selection/v1');
for (const key of ['durationMs', 'sampleIntervalMs', 'commandTimeoutMs', 'httpTimeoutMs', 'maxCommandBytes', 'maxSamples', 'maxJournalBytes']) {
  assert.ok(Number.isSafeInteger(selection[key]) && selection[key] > 0, `Missing owner observer limit ${key}`);
}
assert.ok(selection.sampleIntervalMs <= manifest.observer.maxSampleGapMs);
assert.ok(['resources', 'cold', 'warm'].includes(selection.mode));
assert.match(selection.container ?? '', /^uar-pr476-[a-z0-9-]+$/);
assert.match(selection.caseId ?? '', /^[a-z0-9][a-z0-9-]{7,63}$/);
assert.ok(isAbsolute(selection.bindingInputsPath));
const revision = sha(await readFile(fileURLToPath(import.meta.url)));
assert.equal(revision, manifest.observer.sourceRevision, 'Freeze the actual observer file SHA, not a hand-written revision label');
const inputs = await loadCandidateArtifactInputs(selection.bindingInputsPath);
assert.equal(inputs.config.baseUrl, new URL(manifest.profile.gateway).href);
const observationId = randomUUID();
const common = { experimentId: manifest.selection.experimentId, observerRevision: revision, observationId,
  manifestSha256: sha(manifestBytes), selectionSha256: sha(selectionBytes), identity: manifest.identity };
let published = 0;
async function emit(value) {
  const bytes = Buffer.from(`${JSON.stringify({ ...common, ...value })}\n`);
  let size = 0;
  try { const metadata = await lstat(manifest.observer.journalPath); assert.ok(metadata.isFile()); size = metadata.size; }
  catch (error) { if (error.code !== 'ENOENT') throw error; }
  assert.ok(size + bytes.length <= selection.maxJournalBytes);
  await appendFile(manifest.observer.journalPath, bytes, { mode: 0o600 }); published++;
}
function docker(...args) {
  return execFileSync('docker', args, { encoding: 'utf8', timeout: selection.commandTimeoutMs, maxBuffer: selection.maxCommandBytes });
}
function inspect() {
  // Existing artifact workflow projection: never read Env, cmdline or full config.
  const projection = '{"id":{{json .Id}},"image":{{json .Image}},"labels":{{json .Config.Labels}},"running":{{json .State.Running}},"started_at":{{json .State.StartedAt}},"finished_at":{{json .State.FinishedAt}},"exit_code":{{json .State.ExitCode}},"mounts":{{json .Mounts}}}';
  const container = JSON.parse(docker('inspect', '--format', projection, selection.container));
  assert.equal(container.labels['dev.prometheus.qualification.case'], selection.caseId);
  const mount = container.mounts.filter(row => row.Destination === '/var/lib/uar');
  assert.equal(mount.length, 1); assert.equal(mount[0].Type, 'volume');
  assert.equal(mount[0].Name, selection.volume); container.volume = mount[0].Name; delete container.mounts;
  const labels = JSON.parse(docker('volume', 'inspect', '--format', '{{json .Labels}}', container.volume));
  assert.equal(labels['dev.prometheus.qualification.case'], selection.caseId);
  const platform = JSON.parse(docker('image', 'inspect', '--format', '{"os":{{json .Os}},"architecture":{{json .Architecture}}}', container.image));
  return { container, platform };
}
async function request(path) {
  return fetch(new URL(path, inputs.config.baseUrl), { redirect: 'error', signal: AbortSignal.timeout(selection.httpTimeoutMs),
    headers: { 'x-uar-workspace-id': inputs.config.workspace } });
}
async function responseBytes(response) {
  const chunks = []; let length = 0;
  for await (const chunk of response.body ?? []) {
    length += chunk.length; if (length > manifest.limits.maxResponseBytes) throw new Error('Observer HTTP response exceeds frozen limit');
    chunks.push(Buffer.from(chunk));
  }
  return Buffer.concat(chunks);
}
async function snapshot(retainedSettings) {
  const actual = inspect(); const settings = retainedSettings ?? {};
  if (!retainedSettings) for (const key of candidateSettingKeys) {
    const response = await request(`/api/uar/settings/${key}`); assert.equal(response.status, 200);
    settings[key] = JSON.parse((await responseBytes(response)).toString('utf8')).effective;
  }
  const value = captureCandidateArtifact(inputs, { ...actual, settings, baseUrl: inputs.config.baseUrl,
    workspace: inputs.config.workspace, expectRunning: !retainedSettings });
  assert.deepEqual(value.identity, manifest.identity); return value;
}
// Linux procfs fields: parse after final ')' (comm may contain spaces/parentheses).
// utime/stime are fields 14/15; starttime is field 22; status VmRSS is KiB.
// This measures the unique runtime process, not container/cgroup total memory.
const procProgram = String.raw`import os,json,re
from pathlib import Path
matches=[]
for path in Path('/proc').iterdir():
 if path.name.isdecimal():
  try:
   if os.readlink(path/'exe') == '/usr/local/bin/universal-agent-runtime': matches.append(path)
  except (FileNotFoundError,PermissionError,ProcessLookupError): pass
if len(matches)!=1:
 print(json.dumps({'availability':'unavailable','reason':'unique exact runtime executable not observed','matches':len(matches)}))
else:
 path=matches[0]
 try:
  before=(path/'stat').read_text(); fields=before[before.rfind(')')+2:].split()
  status=(path/'status').read_text(); after=(path/'stat').read_text(); end=after[after.rfind(')')+2:].split()
  assert fields[19]==end[19]
  ticks=int(end[11])+int(end[12]); hz=os.sysconf('SC_CLK_TCK')
  rss=re.search(r'^VmRSS:\s+(\d+) kB$',status,re.M); assert rss and hz>0
  print(json.dumps({'availability':'measured','pid':int(path.name),'startTicks':end[19],
   'cpuUsageNs':ticks*1000000000//hz,'rssBytes':int(rss.group(1))*1024,'clockTicksPerSecond':hz}))
 except (FileNotFoundError,PermissionError,ProcessLookupError,AssertionError,ValueError,IndexError):
  print(json.dumps({'availability':'unavailable','reason':'runtime changed or required procfs fields unavailable'}))`;
const absent = reason => ({ availability: 'unavailable', value: null, reason });
const startedAt = Date.now();
let before, after;
if (selection.mode === 'resources') {
  before = await snapshot();
  const epoch = `${before.container.id}:${before.container.started_at}`;
  let processEpoch;
  for (let sample = 0; Date.now() - startedAt <= selection.durationMs; sample++) {
    assert.ok(sample < selection.maxSamples, 'Observer sample budget exhausted');
    const container = inspect().container;
    assert.equal(container.id, before.container.id); assert.equal(container.started_at, before.container.started_at); assert.equal(container.running, true);
    const raw = JSON.parse(docker('exec', selection.container, 'python3', '-c', procProgram));
    const timeUnixMs = Date.now(); let cpu = absent(raw.reason), rss = absent(raw.reason);
    if (raw.availability === 'measured') {
      const nextEpoch = `${raw.pid}:${raw.startTicks}`; processEpoch ??= nextEpoch;
      assert.equal(nextEpoch, processEpoch, 'Runtime process replacement invalidates a resource interval');
      for (const key of ['cpuUsageNs', 'rssBytes']) assert.ok(Number.isSafeInteger(raw[key]) && raw[key] >= 0);
      cpu = { availability: 'measured', source: 'process_cpu_cumulative_nanoseconds', value: raw.cpuUsageNs };
      rss = { availability: 'measured', source: 'process_rss_bytes', value: raw.rssBytes };
    }
    await emit({ kind: 'resource_sample', timeUnixMs, monotonicNs: process.hrtime.bigint().toString(), runtimeEpoch: epoch,
      processEpoch: processEpoch ?? null, cpuUsageNs: cpu, rssBytes: rss, rawProcfs: raw,
      containerTotalRssBytes: absent('not observed; process RSS must not be represented as container total'),
      runtimeQueueDepth: absent('no runtime-owned queue observer selected'),
      runtimeBackpressureTotal: absent('no runtime-owned backpressure counter selected') });
    await delay(selection.sampleIntervalMs);
  }
  after = await snapshot(); assertCandidateArtifactStable(before, after);
} else {
  // Require an actual successful fresh artifact receipt from the prior live
  // instance. No synthetic expected-settings snapshot can stand in for live data.
  assert.ok(isAbsolute(selection.priorLiveReceipt?.path));
  const priorBytes = await readFile(selection.priorLiveReceipt.path);
  assert.equal(sha(priorBytes), selection.priorLiveReceipt.sha256);
  const prior = JSON.parse(priorBytes);
  assert.equal(prior.kind, 'uar.production-artifact-qualification/v1'); assert.equal(prior.status, 'PASSED');
  assert.equal(prior.authentication_supplied_by_test, false);
  before = prior.candidateBinding.after; assert.deepEqual(before.identity, manifest.identity);
  assert.equal(before.container.running, true);
  const stopped = await snapshot(before.settings);
  assert.equal(stopped.container.id, before.container.id); assert.equal(stopped.container.started_at, before.container.started_at);
  assert.equal(stopped.container.volume, before.container.volume);
  const boundary = { timeUnixMs: Date.now(), monotonicNs: process.hrtime.bigint().toString() };
  let ready;
  while (Date.now() - boundary.timeUnixMs <= selection.durationMs) {
    const actual = inspect();
    assert.equal(actual.container.id, before.container.id); assert.equal(actual.container.image, before.container.image);
    if (actual.container.running && Date.parse(actual.container.started_at) > Date.parse(before.container.started_at)) {
      try { const response = await request('/readyz'); const body = await responseBytes(response);
        if (response.status === 200) { ready = { timeUnixMs: Date.now(), monotonicNs: process.hrtime.bigint().toString(), bodySha256: sha(Buffer.from(body)) }; break; }
      } catch { /* bounded external transition: only a positive response closes startup */ }
    }
    await delay(selection.sampleIntervalMs);
  }
  assert.ok(ready, 'No positive new-instance ready boundary observed');
  after = await snapshot(); assertCandidateArtifactStable(before, after, { allowRestart: true });
  const runtimeEpoch = `${after.container.id}:${after.container.started_at}`;
  const phase = selection.mode, definitionSha256 = sha(manifest.startupDefinitions[phase]);
  await emit({ kind: 'startup_start', phase, definitionSha256, runtimeEpoch, ...boundary,
    boundary: 'observer witnessed owned container stopped before external start; includes external scheduling delay', stopped });
  await emit({ kind: 'startup_ready', phase, definitionSha256, runtimeEpoch, ...ready, endpoint: '/readyz', statusCode: 200 });
}
const finalInputs = await loadCandidateArtifactInputs(selection.bindingInputsPath);
assert.equal(finalInputs.bindingInputsSha256, inputs.bindingInputsSha256); assert.deepEqual(finalInputs.references, inputs.references);
assert.equal(sha(await readFile(manifestPath)), sha(manifestBytes)); assert.equal(sha(await readFile(selectionPath)), sha(selectionBytes));
await emit({ kind: 'observation_boundary', mode: selection.mode, status: 'OBSERVED', candidateBinding: { before, after },
  observerCost: 'separate Docker exec/procfs + inspect + selected setting reads; observer perturbation not subtracted', published });
