// Shared pure/read-only metadata binding. Import does not inspect Docker, call an
// API, run Git, read Env/configuration, mutate files or confer release authority.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile, lstat } from 'node:fs/promises';
import { isAbsolute } from 'node:path';
export const candidateSettingKeys = Object.freeze(['server.log_format', 'memory.inject_context',
  'memory.injection_min_similarity', 'llm.model', 'llm.timeout_secs', 'llm.max_retries',
  'llm.parallel_tool_calls', 'llm.cost_tracking', 'llm.tracing']);
export const canonical = value => JSON.stringify(value, (_key, item) => item && typeof item === 'object' && !Array.isArray(item)
  ? Object.fromEntries(Object.entries(item).sort(([a], [b]) => a < b ? -1 : a > b ? 1 : 0)) : item);
const sha = value => createHash('sha256').update(value).digest('hex');
const unprefix = value => String(value ?? '').replace(/^sha256:/, '');
function url(value) { const parsed = new URL(value); assert.ok(['http:', 'https:'].includes(parsed.protocol)
  && ['127.0.0.1', '[::1]'].includes(parsed.hostname) && !parsed.username && !parsed.password && !parsed.search && !parsed.hash); return parsed.href; }
function settings(values) {
  assert.deepEqual(Object.keys(values).sort(), [...candidateSettingKeys].sort(), 'Only the complete nonsecret setting whitelist is eligible');
  for (const key of candidateSettingKeys) {
    const value = values[key];
    if (['server.log_format', 'llm.model'].includes(key)) assert.ok(typeof value === 'string' && !value.includes('://') && !/[\r\n]/.test(value));
    else if (key === 'memory.injection_min_similarity') assert.ok(value === null || (Number.isFinite(value) && value >= -1 && value <= 1));
    else if (['llm.timeout_secs', 'llm.max_retries'].includes(key)) assert.ok(Number.isSafeInteger(value) && value >= 0);
    else assert.equal(typeof value, 'boolean');
  }
  return structuredClone(values);
}
export async function loadCandidateArtifactInputs(path) {
  assert.ok(isAbsolute(path)); const metadata = await lstat(path);
  assert.ok(metadata.isFile() && metadata.size <= 1024 * 1024);
  const raw = await readFile(path), config = JSON.parse(raw.toString('utf8'));
  assert.equal(config.kind, 'uar.candidate-artifact-binding-inputs/v1');
  assert.ok(Number.isSafeInteger(config.maxInputBytes) && config.maxInputBytes > 0);
  const documents = {}, references = {};
  for (const key of ['inventory', 'inputProvenance', 'imageBuild']) {
    const ref = config[key]; assert.ok(isAbsolute(ref.path));
    assert.match(unprefix(ref.sha256), /^[a-f0-9]{64}$/);
    const entry = await lstat(ref.path); assert.ok(entry.isFile() && entry.size <= config.maxInputBytes);
    const bytes = await readFile(ref.path); assert.ok(bytes.length <= config.maxInputBytes);
    assert.equal(sha(bytes), unprefix(ref.sha256), 'Bound artifact input changed');
    documents[key] = JSON.parse(bytes.toString('utf8')); references[key] = { path: ref.path, sha256: sha(bytes) };
  }
  const { inventory, inputProvenance: provenance, imageBuild: build } = documents;
  assert.equal(inventory.schema, 'uar.production-readiness.source-inventory/v1');
  assert.equal(inventory.identity.working_index_status, '', 'Dirty implementation snapshots cannot qualify a release artifact');
  assert.ok(!inventory.identity.merge_head, 'Pending merge is not a clean release');
  assert.match(inventory.source_identity_sha256, /^[a-f0-9]{64}$/);
  assert.equal(sha(canonical(inventory.identity)), inventory.source_identity_sha256, 'Inventory identity preimage differs');
  const regular = inventory.files.filter(row => row.kind === 'regular').map(row => ({ path: row.path, sha256: row.sha256 }))
    .sort((a, b) => a.path < b.path ? -1 : a.path > b.path ? 1 : 0);
  assert.equal(`sha256:${sha(canonical(regular))}`, inventory.source_identity, 'Build inventory regular-source preimage differs');
  assert.equal(provenance.schema_version, 1); assert.equal(provenance.input_boundary, 'release', 'Dot/local_candidate provenance is not supported for this lane');
  assert.equal(provenance.source_sha, inventory.identity.head); assert.equal(provenance.source_tree, inventory.identity.head_tree);
  assert.match(provenance.source_sha, /^[a-f0-9]{40}$/); assert.match(provenance.source_tree, /^[a-f0-9]{40}$/);
  assert.equal(build.exit_code, 0); assert.ok(Array.isArray(build.command) && build.command.length > 0);
  assert.deepEqual(build.command, build.operator_approved_command, 'Real approved build command required');
  assert.equal(unprefix(build.source_identity), unprefix(inventory.source_identity), 'Image build must bind actual inventory regular-source preimage');
  assert.equal(build.image_id, build.image_source_id); assert.match(build.image_id, /^sha256:[a-f0-9]{64}$/);
  assert.match(unprefix(config.runtimeConfigDigest), /^[a-f0-9]{64}$/);
  return { config: { ...config, baseUrl: url(config.baseUrl), expectedSettings: settings(config.expectedSettings) },
    documents, references, bindingInputsSha256: sha(raw) };
}
export function captureCandidateArtifact(inputs, { container, platform, settings: actualSettings, baseUrl, workspace, expectRunning = true }) {
  const { config, documents, references } = inputs;
  assert.deepEqual(platform, { os: 'linux', architecture: 'amd64' });
  assert.match(container.id, /^[a-f0-9]{64}$/); assert.equal(container.image, documents.imageBuild.image_id);
  assert.equal(container.running, expectRunning); assert.ok(typeof container.volume === 'string' && container.volume.length > 0);
  assert.ok(Number.isFinite(Date.parse(container.started_at)));
  assert.equal(url(baseUrl), config.baseUrl); assert.equal(workspace, config.workspace);
  assert.equal(container.labels['dev.prometheus.qualification.base-url'], config.baseUrl);
  assert.equal(container.labels['dev.prometheus.qualification.workspace'], workspace);
  assert.equal(container.labels['org.opencontainers.image.revision'], documents.inputProvenance.source_sha);
  assert.equal(unprefix(container.labels['dev.prometheus.qualification.source-inventory-sha256']), documents.inventory.source_identity_sha256);
  assert.equal(unprefix(container.labels['dev.prometheus.qualification.config-sha256']), unprefix(config.runtimeConfigDigest));
  const projection = settings(actualSettings); assert.deepEqual(projection, config.expectedSettings);
  return { identity: { sourceCommit: documents.inputProvenance.source_sha,
    sourceInventoryDigest: documents.inventory.source_identity_sha256, imageDigest: container.image,
    configDigest: unprefix(config.runtimeConfigDigest), settingsDigest: sha(canonical(projection)), platform: 'linux/amd64' },
    bindingInputsSha256: inputs.bindingInputsSha256, references, baseUrl: config.baseUrl, workspace,
    container: { id: container.id, image: container.image, running: container.running,
      started_at: container.started_at, finished_at: container.finished_at, exit_code: container.exit_code, volume: container.volume },
    settings: projection, settingsObservation: expectRunning ? 'live_selected_effective_values' : 'retained_prior_live_values_while_stopped',
    configBoundary: 'trusted_host_sealed_launch_digest_label; no raw Env or full configuration dump' };
}
export function assertCandidateArtifactStable(before, after, { allowRestart = false } = {}) {
  assert.deepEqual(after.identity, before.identity); assert.deepEqual(after.references, before.references);
  assert.equal(after.bindingInputsSha256, before.bindingInputsSha256); assert.equal(after.baseUrl, before.baseUrl);
  assert.equal(after.workspace, before.workspace); assert.equal(after.container.volume, before.container.volume);
  if (allowRestart) assert.ok(Date.parse(after.container.started_at) > Date.parse(before.container.started_at));
  else { assert.equal(after.container.id, before.container.id); assert.equal(after.container.started_at, before.container.started_at); }
}
