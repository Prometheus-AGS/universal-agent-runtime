import assert from 'node:assert/strict';
import { createHash, randomUUID } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { performance } from 'node:perf_hooks';

export function parseJson(raw) {
  try { return JSON.parse(raw); } catch { throw new Error('Invalid JSON from owned fixture boundary; body suppressed'); }
}

export const digest = value => createHash('sha256').update(value).digest('hex');
const assetRoot = new URL('../fixtures/capability-dispatch/', import.meta.url);
const label = /^[A-Za-z0-9_-]{1,96}$/;

function sortedJsonValue(value) {
  if (Array.isArray(value)) return value.map(sortedJsonValue);
  if (value && typeof value === 'object') {
    return Object.fromEntries(Object.keys(value).sort().map(key => [key, sortedJsonValue(value[key])]));
  }
  return value;
}

function adoptedContractDigest(adopted) {
  assert.equal(adopted.digestNormalization?.algorithm, 'sha256', 'Unsupported adopted contract digest algorithm');
  assert.equal(adopted.digestNormalization?.representation,
    'UTF8 JSON sortedkeys compactseparators; exclude identity.contractDigest and digestNormalization',
    'Unsupported adopted contract digest normalization');
  const normalized = structuredClone(adopted);
  delete normalized.identity.contractDigest;
  delete normalized.digestNormalization;
  return digest(Buffer.from(JSON.stringify(sortedJsonValue(normalized)), 'utf8'));
}

export function safeUrl(value) {
  const url = new URL(value);
  assert.ok(['http:', 'https:'].includes(url.protocol) && !url.username && !url.password && !url.search && !url.hash,
    'Fixture endpoints must be explicit credential-free HTTP URLs');
  assert.ok(['127.0.0.1', '[::1]'].includes(url.hostname),
    'Selected local fixture endpoints must use literal loopback addresses in the externally owned namespace');
  return url;
}

// This latch belongs only to the focused fixture module. An aborted client
// cannot prove that a server-side mutation stopped after bytes left the client.
let poisonReason;
export function poisonFixture(reason) { poisonReason ??= reason; }
export function isolationState() { return { poisoned: Boolean(poisonReason), reason: poisonReason ?? null }; }

function lifetime(label) {
  const io = { controller: new AbortController(), deadline: performance.now() + 100_000, pending: new Set() };
  io.timer = setTimeout(() => {
    poisonFixture(`${label}-deadline-expired`);
    io.controller.abort(new Error('Owned fixture operation deadline expired'));
  }, 100_000);
  return io;
}
export function beginCase(w) {
  assert.equal(Boolean(poisonReason), false, 'Focused fixture is poisoned; reconcile prior owned effects before a new run');
  w.caseIO = lifetime('case');
}
export function beginCleanup(w) { w.cleanupIO = lifetime('cleanup'); return w.cleanupIO; }
export function requireLive(io) {
  assert.ok(io, 'Owned fixture lifetime must exist before I/O');
  if (performance.now() >= io.deadline && !io.controller.signal.aborted) {
    poisonFixture('owned-operation-deadline-expired');
    io.controller.abort(new Error('Owned fixture operation deadline expired'));
  }
  io.controller.signal.throwIfAborted();
}
export async function finishIO(io) {
  if (!io) return;
  if (performance.now() >= io.deadline) poisonFixture('owned-operation-deadline-expired');
  clearTimeout(io.timer);
  io.controller.abort(new Error('Owned fixture lifetime closed'));
  // Every registered promise covers fetch AND full body consumption. Joining
  // settles client ownership; it deliberately makes no server-cancellation claim.
  while (io.pending.size) await Promise.allSettled([...io.pending]);
}
async function requestIO(io, operation) {
  requireLive(io);
  const pending = (async () => {
    try {
      const result = await operation(io.controller.signal);
      requireLive(io);
      return result;
    } catch {
      poisonFixture('owned-http-outcome-uncertain');
      throw new Error('Owned fixture HTTP operation failed; remote effect requires reconciliation');
    }
  })();
  io.pending.add(pending);
  try { return await pending; } finally { io.pending.delete(pending); }
}
export function readOwnedFile(w, path, encoding) {
  requireLive(w.caseIO);
  // Local fixture reads do not leave a best-effort asynchronous filesystem
  // promise behind a Cucumber timeout. A blocking read cannot advance the runner;
  // elapsed deadline is checked again before any subsequent network operation.
  const bytes = readFileSync(path, encoding);
  requireLive(w.caseIO);
  return bytes;
}

export async function initialize(w) {
  assert.ok(process.env.UAR_BASE_URL, 'Prerequisite: explicit owned external identity endpoint UAR_BASE_URL');
  assert.ok(process.env.UAR_WORKSPACE_ID, 'Prerequisite: UAR_WORKSPACE_ID');
  assert.ok(process.env.UAR_CAPABILITY_FIXTURE, 'Prerequisite: externally provisioned UAR_CAPABILITY_FIXTURE manifest');
  w.base = safeUrl(process.env.UAR_BASE_URL);
  w.workspace = process.env.UAR_WORKSPACE_ID;
  const bytes = readOwnedFile(w, process.env.UAR_CAPABILITY_FIXTURE);
  const adopted = parseJson(readOwnedFile(w, new URL('../../../.kbd-orchestrator/phases/production-readiness-476-capability-dispatch/adopted-contract.json', import.meta.url), 'utf8'));
  w.fixture = parseJson(bytes.toString('utf8'));
  assert.ok(label.test(w.fixture.namespace) && w.fixture.namespace.startsWith('capability-'), 'Owned fixture namespace required');
  const recomputedContractDigest = adoptedContractDigest(adopted);
  assert.equal(recomputedContractDigest, adopted.identity.contractDigest, 'Adopted contract content must match its recomputed digest');
  assert.equal(w.fixture.identity.contractDigest, recomputedContractDigest, 'Manifest must bind actual current adopted scenario contract content');
  assert.equal(w.fixture.identity.platform, 'linux/amd64', 'Selected artifact lane is Linux amd64');
  assert.match(w.fixture.identity.sourceCommit, /^[0-9a-f]{40}$/, 'Prerequisite: full 40-hex committed source identity');
  assert.match(w.fixture.identity.imageDigest, /^sha256:[0-9a-f]{64}$/, 'Prerequisite: exact local Docker image ID');
  for (const field of ['configDigest', 'contractDigest']) {
    assert.match(w.fixture.identity[field], /^[0-9a-f]{64}$/, `Prerequisite: exact 64-hex ${field}`);
  }
  assert.ok(Array.isArray(w.fixture.peers) && w.fixture.peers.length >= 2, 'At least two isolated controlled peers required');
  w.peers = new Map(w.fixture.peers.map(peer => [peer.id, { ...peer, url: safeUrl(peer.url) }]));
  assert.equal(w.peers.size, w.fixture.peers.length, 'Peer IDs must be unique');
  w.id = randomUUID();
  w.case = `cd-${w.id}`;
  w.session = randomUUID();
  w.sessionIds = [w.session];
  w.agentIds = [];
  w.assets = {};
  for (const [name, mime] of [['png', 'image/png'], ['jpeg', 'image/jpeg']]) {
    const raw = readOwnedFile(w, new URL(`synthetic.${name}`, assetRoot));
    w.assets[name] = { raw, mime, hash: digest(raw), url: `data:${mime};base64,${raw.toString('base64')}` };
  }
  w.privateText = `PRIVATE_CAPABILITY_${randomUUID()}`;
  w.evidence = { asset_hashes: Object.fromEntries(Object.entries(w.assets).map(([name, value]) => [name, value.hash])), schema: 'uar.capability-dispatch/v1', case: w.case, identity: w.fixture.identity,
    fixture_manifest_sha256: digest(bytes), recomputed_contract_sha256: recomputedContractDigest, authentication_supplied_by_test: false,
    interpretation: 'deterministic wire/admission only; no visual understanding claim', exchanges: [], observations: [] };
  const required = ['capable', 'text_only', 'no_tools', 'no_structured', 'tiny', 'disabled',
    'cooldown', 'failure_primary', 'fallback', 'failure_exact', 'failure_scope', 'outside'];
  assert.ok(required.every(key => w.fixture.models[key]), 'Prerequisite: every isolated model role including outside must be provisioned');
  const registered = parseJson((await http(w, 'GET', '/api/uar/providers', undefined, [200])).raw);
  const ownedIds = new Set(Object.values(w.fixture.models).map(entry => entry.destination.split('/')[0]));
  assert.deepEqual(registered.providers.map(entry => entry.id).sort(), [...ownedIds].sort(),
    'Owned candidate registry must contain only the declared controlled fixture providers');
  for (const entry of registered.providers) {
    assert.equal(entry.credential_configured, false, 'No real provider credentials belong in this fixture runtime');
    assert.ok(entry.id.startsWith(`${w.fixture.namespace}-`), 'Every registration must belong to the generated namespace');
    assert.ok([...w.peers.values()].some(peer => safeUrl(peer.wireBaseUrl).href === safeUrl(entry.base_url).href),
      'Every registered destination must point at an owned controlled peer');
  }
  w.outside = await model(w, 'outside');
  assert.ok(Object.entries(w.fixture.models).every(([key, value]) => key === 'outside' || value.peer !== w.outside.peer),
    'Global outside fallback requires its independently observed peer');
  function safeFallbacks(entries) {
    assert.ok(Array.isArray(entries), 'Global fallback list must be an actual array');
    return entries.map(entry => {
      assert.ok(entry && typeof entry === 'object' && entry.api_key == null, 'Global fixture fallback must remain keyless');
      return { model: entry.model, base_url: safeUrl(entry.base_url).href };
    });
  }
  const expectedGlobal = [{ model: w.outside.destination, base_url: safeUrl(w.peers.get(w.outside.peer).wireBaseUrl).href }];
  assert.deepEqual(safeFallbacks(w.fixture.failover.fallback_models), expectedGlobal,
    'Manifest must configure the actual outside role as the real global fallback');
  assert.equal(w.fixture.failover.enabled, true);
  assert.equal(w.fixture.failover.strategy, 'priority');
  assert.ok(w.fixture.failover.error_threshold >= 2, 'Global control plus restricted attempt must precede primary cooldown');
  const settings = parseJson((await http(w, 'GET', '/api/uar/settings/llm-failover', undefined, [200])).raw);
  const effective = {};
  const revisions = {};
  for (const field of ['enabled', 'strategy', 'error_threshold', 'cooldown_secs', 'fallback_models']) {
    const matches = settings.filter(row => row.key === `llm_failover.${field}`);
    assert.equal(matches.length, 1, 'Required global failover setting must have exactly one authoritative row');
    const row = matches[0];
    assert.equal(row.application_status, 'effective', 'Pending/saved settings cannot prove the configured global competitor');
    const actual = field === 'fallback_models' ? safeFallbacks(row.effective) : row.effective;
    const expected = field === 'fallback_models' ? expectedGlobal : w.fixture.failover[field];
    assert.deepEqual(actual, expected, 'Effective global failover must equal the frozen owned fixture configuration');
    effective[field] = actual;
    revisions[field] = row.revision;
  }
  w.evidence.effective_global_failover = effective;
  w.evidence.global_failover_setting_revisions = revisions;
  assert.equal(w.fixture.llm?.max_retries, 0, 'Fixture client/UAR LLM retry cap must be zero');
  assert.equal(w.fixture.resilience?.retries_enabled, false, 'Fixture host establishment retries must be disabled');
  assert.equal(w.fixture.resilience?.retry_max_attempts, 1, 'Fixture host attempt bound must be one');
  const retrySettings = {
    'llm.max_retries': w.fixture.llm.max_retries,
    'resilience.retries_enabled': w.fixture.resilience.retries_enabled,
    'resilience.retry_max_attempts': w.fixture.resilience.retry_max_attempts,
  };
  w.evidence.effective_retry_settings = {};
  for (const [key, expected] of Object.entries(retrySettings)) {
    const row = parseJson((await http(w, 'GET', `/api/uar/settings/${encodeURIComponent(key)}`, undefined, [200])).raw);
    assert.equal(row.key, key, 'Retry prerequisite must read its actual existing setting key');
    assert.equal(row.application_status, 'effective', 'A pending retry setting is not an effective attempt bound');
    assert.equal(row.effective, expected, 'Effective client/host retry control must equal the owned fixture manifest');
    w.evidence.effective_retry_settings[key] = { effective: row.effective, revision: row.revision };
  }
  const dedicated = ['cooldown', 'failure_primary', 'failure_exact', 'failure_scope'].map(key => w.fixture.models[key]);
  assert.equal(new Set(dedicated.map(entry => entry.destination.split('/')[0])).size, dedicated.length,
    'Failure/cooldown cases require dedicated providers to preserve independent health state');
  assert.equal(new Set(dedicated.map(entry => entry.peer)).size, dedicated.length,
    'Failure/cooldown cases require dedicated observed peers');
}

export function privacy(w, raw) {
  return !raw.includes('data:image/') && !raw.includes(w.privateText)
    && Object.values(w.assets).every(asset => !raw.includes(asset.raw.toString('base64')));
}

export async function http(w, method, path, payload, expected, io = w.caseIO) {
  requireLive(io);
  const body = payload === undefined ? undefined : JSON.stringify(payload);
  const { response, raw } = await requestIO(io, async signal => {
    const response = await fetch(new URL(path, w.base), {
      method, redirect: 'error', signal,
      headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': w.workspace },
      body,
    });
    const raw = await response.text();
    return { response, raw };
  });
  w.evidence.exchanges.push({ method, path, status: response.status, response_bytes: Buffer.byteLength(raw),
    response_sha256: digest(raw), diagnostics_safe: privacy(w, raw) });
  assert.ok(privacy(w, raw), 'Public diagnostics leaked an isolated inline fixture sentinel');
  if (expected) assert.ok(expected.includes(response.status), `Owned API operation returned HTTP ${response.status}`);
  return { status: response.status, raw };
}

export async function peerRequest(w, peer, path, payload, io = w.caseIO) {
  requireLive(io);
  const body = payload ? JSON.stringify(payload) : undefined;
  return requestIO(io, async signal => {
    const response = await fetch(new URL(path, peer.url), { method: payload ? 'POST' : 'GET',
      redirect: 'error', signal, headers: { 'Content-Type': 'application/json' },
      body });
    const raw = await response.text();
    assert.equal(response.status, 200, 'Owned controlled peer operation must succeed');
    return parseJson(raw);
  });
}

export async function snapshots(w) {
  const states = {};
  for (const [id, peer] of w.peers) states[id] = await peerRequest(w, peer, '/__uar_fixture/status');
  return states;
}

export async function configure(w, failures = [], responseJson, io = w.caseIO) {
  for (const [id, peer] of w.peers) {
    for (const caseId of [w.case, 'default']) {
      await peerRequest(w, peer, '/__uar_fixture/control', { action: 'configure_chat', case: caseId,
        mode: failures.includes(id) ? 'provider_failure' : 'success',
        ...(responseJson ? { response_json: responseJson } : {}) }, io);
    }
  }
}

export async function model(w, key) {
  const config = w.fixture.models[key];
  assert.ok(config, `Prerequisite: owned model role ${key}`);
  const slash = config.destination.indexOf('/');
  assert.ok(slash > 0 && w.peers.has(config.peer), 'Destination and peer are required');
  const provider = config.destination.slice(0, slash), modelId = config.destination.slice(slash + 1);
  assert.ok(provider.startsWith(`${w.fixture.namespace}-`), 'Only generated fixture provider namespace is allowed');
  const view = parseJson((await http(w, 'GET', `/api/uar/providers/${encodeURIComponent(provider)}`, undefined, [200])).raw);
  assert.equal(view.id, provider);
  assert.equal(view.credential_configured, false, 'Controlled fixture registration must be keyless');
  assert.equal(safeUrl(view.base_url).href, safeUrl(w.peers.get(config.peer).wireBaseUrl).href, 'Registration must point at its owned peer');
  const declared = view.models.find(entry => entry.id === modelId);
  assert.ok(declared, 'Owned fixture model must be registered');
  for (const field of ['enabled', 'supports_vision', 'supports_tools', 'supports_structured_output', 'context_window', 'max_output_tokens']) {
    assert.equal(declared[field], config[field], `Provisioned ${key} ${field} must match the frozen manifest`);
  }
  assert.equal(view.enabled, config.provider_enabled, 'Provider enabled state must match manifest');
  if (key === 'text_only') assert.equal(declared.supports_vision, false);
  if (key === 'no_tools') assert.equal(declared.supports_tools, false);
  if (key === 'no_structured') assert.equal(declared.supports_structured_output, false);
  if (key === 'disabled') assert.ok(!view.enabled || !declared.enabled, 'Disabled fixture must actually be disabled');
  if (['capable', 'fallback', 'outside', 'failure_primary', 'failure_exact', 'failure_scope'].includes(key)) {
    assert.equal(declared.supports_vision, true, 'Successful image/failure-control roles require vision declarations');
    assert.equal(declared.enabled && view.enabled, true);
  }
  if (key === 'capable') {
    assert.equal(declared.supports_tools, true);
    assert.equal(declared.supports_structured_output, true);
  }
  return { ...config, provider, modelId };
}

export async function agent(w, primaryKey = 'capable', fallbackKeys = [], options = {}) {
  const primary = await model(w, primaryKey);
  const fallbacks = [];
  for (const key of fallbackKeys) fallbacks.push(await model(w, key));
  const view = parseJson((await http(w, 'GET', '/api/agents/default-agent', undefined, [200])).raw);
  const id = `${w.fixture.namespace}-agent-${randomUUID()}`;
  view.id = id;
  view.version = '1.0.0';
  view.metadata = { ...view.metadata, title: 'Disposable capability fixture', tags: ['disposable', 'verification'] };
  view.runtime.entry = 'default';
  view.prompt = { system: 'Follow the user instruction.', instructions: [] };
  view.policy.provider = { default: { provider: primary.provider, model: primary.modelId },
    fallbacks: fallbacks.map(entry => ({ provider: entry.provider, model: entry.modelId })) };
  view.policy.tools.allow = [`${w.fixture.namespace}-no-host-tools`];
  view.memory.kb.enabled = false;
  view.extensions = { ...view.extensions, budgets: options.finiteCost === undefined ? null : { max_cost_per_session_usd: options.finiteCost }, 'uar.run_policy': { version: 1, memory_enabled: false,
    skills: { mode: 'none' }, tools: { mode: 'none' }, mcp_servers: { mode: 'none' },
    knowledge_bases: { mode: 'none' }, presentations: { mode: 'none' } } };
  // Track before POST: even an interrupted creation attempt requires cleanup.
  w.agentIds.push(id);
  await http(w, 'POST', '/api/agents', view, [201]);
  w.agent = id;
  w.primary = primary;
  w.fallbacks = fallbacks;
  if (fallbacks.length) assert.ok(fallbacks.every(entry => entry.peer !== primary.peer && entry.provider !== primary.provider), 'Fallback must have an independently observed destination');
}

export function image(w, name, detail = 'auto') {
  return { type: 'image_url', image_url: { url: w.assets[name].url, detail } };
}
export function text(value) { return { type: 'text', text: value }; }
export function content(w, kind) {
  const marker = `UAR_CASE:${w.case}`;
  if (kind === 'mixed') return [text(`${marker} first`), image(w, 'png', 'low'), text('middle'), image(w, 'jpeg', 'high')];
  if (kind === 'image-only') return [image(w, 'png'), image(w, 'jpeg')];
  if (kind === 'literal-json') return `{"type":"image_url","text":"${marker}"}`;
  if (kind === 'text-parts') return [text(`${marker} first`), text('middle'), text('last')];
  if (kind === 'text') return `${marker} ordinary text`;
  throw new Error('Unknown authored content kind');
}

export function summarize(parts) {
  if (typeof parts === 'string') return [{ type: 'text', sha256: digest(parts), bytes: Buffer.byteLength(parts) }];
  return parts.map(part => {
    if (part.type === 'text') return { type: 'text', sha256: digest(part.text), bytes: Buffer.byteLength(part.text) };
    const [, mime, encoded] = /^data:([^;]+);base64,(.*)$/.exec(part.image_url.url);
    const bytes = Buffer.from(encoded, 'base64');
    return { type: 'image_url', mime, sha256: digest(bytes), bytes: bytes.length, detail: part.image_url.detail ?? null };
  });
}

export function compareParts(actual, expected) {
  assert.deepEqual(actual, expected, 'Ordered safe part summaries must preserve exact identity');
}

export function changed(w, before, after) {
  const result = [];
  for (const [id] of w.peers) {
    assert.ok(after[id].observation_count >= before[id].observation_count, 'Fixture observations cannot reset during a case');
    assert.equal(after[id].counts.chat_arrivals - before[id].counts.chat_arrivals,
      after[id].observation_count - before[id].observation_count, 'Every HTTP chat arrival must have a safe observation; a fixture-side body rejection is not zero-send');
    const rows = after[id].observations.filter(row => row.ordinal > before[id].observation_count);
    assert.equal(rows.length, after[id].observation_count - before[id].observation_count, 'Bounded observation ring must not truncate this case');
    result.push(...rows.map(row => ({ peer: id, ...row })));
  }
  w.evidence.observations.push(...result);
  return result;
}

export async function send(w, route, stream, value, extras = {}) {
  const payload = { agent_id: w.agent, session_id: w.session, memory_enabled: false,
    stream, max_completion_tokens: 64, messages: [{ role: 'user', content: value }], ...extras };
  const before = await snapshots(w);
  const response = await http(w, 'POST', route, payload);
  const after = await snapshots(w);
  const observations = changed(w, before, after);
  let completion = '';
  let failed = response.status >= 400;
  if (response.status === 200 && stream) {
    let terminal = false;
    for (const block of response.raw.split(/\r?\n\r?\n/)) {
      const data = block.split(/\r?\n/).filter(line => line.startsWith('data:')).map(line => line.slice(5).trim()).join('\n');
      if (!data) continue;
      if (data === '[DONE]') { terminal = true; continue; }
      let event;
      try { event = parseJson(data); } catch { throw new Error('Malformed JSON in completion stream'); }
      failed ||= Boolean(event.error) || ['RUN_ERROR', 'RUN_CANCELLED'].includes(event.type);
      completion += event.choices?.[0]?.delta?.content ?? '';
      terminal ||= event.choices?.[0]?.finish_reason === 'stop';
    }
    if (!failed) assert.ok(terminal, 'Successful stream needs an actual terminal frame');
  } else if (response.status === 200) {
    const body = parseJson(response.raw);
    failed ||= Boolean(body.error);
    completion = body.choices?.[0]?.message?.content ?? '';
  }
  let errorCode = null;
  if (response.status >= 400 && response.raw.trim().startsWith('{')) {
    const body = parseJson(response.raw);
    const candidate = body.error?.code ?? body.code;
    if (typeof candidate === 'string' && /^[a-z_]{1,80}$/.test(candidate)) errorCode = candidate;
  }
  w.last = { status: response.status, failed, completion, observations, errorCode };
  return w.last;
}

export function assertSuccess(w, result, destination = w.primary) {
  assert.equal(result.failed, false, 'Public completion must succeed');
  assert.ok(result.completion.startsWith('controlled-success:') || result.completion === '{"result":42}', 'Useful controlled completion required');
  assert.ok(result.observations.length > 0, 'Actual provider call required');
  for (const observation of result.observations) {
    assert.equal(observation.peer, destination.peer, 'Only admitted actual destination may receive the request');
    assert.equal(observation.model_sha256, digest(destination.wireModel ?? destination.modelId), 'Actual peer model must equal selected destination');
    assert.equal(observation.truncated, false);
  }
}

export function assertRefused(result) {
  assert.equal(result.failed, true, 'Invalid/ineligible request must visibly refuse');
  assert.equal(result.observations.length, 0, 'Refusal must occur before every isolated provider send');
}
