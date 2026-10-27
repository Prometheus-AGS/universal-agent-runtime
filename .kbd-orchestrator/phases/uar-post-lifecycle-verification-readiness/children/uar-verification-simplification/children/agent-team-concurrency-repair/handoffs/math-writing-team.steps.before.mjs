import assert from 'node:assert/strict';
import { createHash, randomUUID, randomInt } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';

setDefaultTimeout(185_000);
const basePath = '/api/v1/collaboration';
const profile = 'urn:prometheus:uar:collaboration:0.1.0-draft.2';
const reservation = { tokens: 32768, costMicrounits: 1000000, elapsedSeconds: 120 };
const limits = { concurrentTurns: 2, maxMembers: 2, maxDepth: 1, maxPendingTasks: 8 };
const budget = { maxTokens: 98304, maxCostMicrounits: 3000000, currency: 'USD', maxElapsedSeconds: 360 };
const canonical = value => JSON.stringify(value, (_, item) => item && typeof item === 'object' && !Array.isArray(item)
  ? Object.fromEntries(Object.entries(item).sort(([a], [b]) => a < b ? -1 : a > b ? 1 : 0)) : item);
const digest = value => 'sha256:' + createHash('sha256').update(value).digest('hex');
const finalize = document => { delete document.contentDigest; document.contentDigest = digest(canonical(document)); return document; };
const reference = document => ({ id: document.id, version: document.version, digest: document.contentDigest });

async function request(world, method, path, payload, expected = 200) {
  const response = await fetch(new URL(path, world.base), { method, redirect: 'error',
    headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${process.env.UAR_AUTH_TOKEN}`,
      'x-uar-workspace-id': process.env.UAR_WORKSPACE_ID },
    body: payload === undefined ? undefined : JSON.stringify(payload), signal: AbortSignal.timeout(150_000) });
  const text = await response.text();
  assert.equal(response.status, expected, `${method} ${path}: HTTP ${response.status}: ${text.slice(0, 2000)}`);
  return JSON.parse(text);
}
const api = (world, method, suffix, payload, expected) => request(world, method, basePath + suffix, payload, expected);
const teamPath = world => `/team-instances/${encodeURIComponent(world.team.id)}`;
const readTeam = world => api(world, 'GET', teamPath(world));
const summary = world => api(world, 'GET', teamPath(world) + '/execution');

async function retain(world) {
  await mkdir(dirname(world.record), { recursive: true });
  await writeFile(world.record, JSON.stringify(world.evidence, null, 2) + '\n', { mode: 0o600 });
}

function output(value) {
  if (typeof value !== 'string') return value;
  const final = value.replace(/^\s*<think>[\s\S]*?<\/think>\s*/, '').trim();
  const fence = /^```(?:json)?\s*\n([\s\S]*?)\n```$/.exec(final);
  try { return JSON.parse(fence ? fence[1] : final); } catch { return final; }
}

Given('the saved math and writing agents and executable team API are available', async function () {
  assert.ok(process.env.UAR_AUTH_TOKEN && process.env.UAR_WORKSPACE_ID && process.env.UAR_MODEL);
  this.base = new URL(process.env.UAR_BASE_URL ?? 'http://127.0.0.1:3000');
  assert.ok(['http:', 'https:'].includes(this.base.protocol) && !this.base.username && !this.base.password);
  this.id = `urn:uar:math-writing:${randomUUID()}`;
  this.record = resolve(`.prometheus/verification/teams/${this.id.split(':').at(-1)}.json`);
  this.evidence = { platform: process.platform, model: process.env.UAR_MODEL, retained_for_future_use: true };
  this.caps = await api(this, 'GET', '/capabilities');
  assert.ok(this.caps.capabilities.includes('collaboration_team_execution_v1'), 'Actual team execution unavailable');
  assert.notEqual(this.caps.catalogStorage.backend, 'memory', 'Team catalog must be durable');
  this.math = await request(this, 'GET', '/api/agents/math-specialist-db64c2a8');
  this.writer = await request(this, 'GET', '/api/agents/writing-specialist');
  const savedWriter = JSON.parse(await readFile('.prometheus/verification/agents/writing-specialist.json', 'utf8'));
  this.byline = savedWriter.byline;
  assert.ok(this.writer.prompt.system.includes(this.byline));
  this.marker = this.math.prompt.system.match(/MATH-[a-f0-9-]+/)[0];
  this.evidence.source_agents = [this.math, this.writer];
  this.evidence.capabilities = this.caps;
  await retain(this);
});

When('I create a team containing those two specialists', async function () {
  const template = JSON.parse(await readFile('scripts/acceptance/fixtures/team-agent.json', 'utf8'));
  const provenance = { source: 'Saved UAR specialists captured for the mathematics/article project', authors: ['UAR operator'] };
  const definitions = [this.math, this.writer].map(agent => {
    const document = structuredClone(template);
    Object.assign(document, { id: agent.id, version: `1.0.0-team-${this.id.split(':').at(-1)}`,
      title: agent.metadata.title, whenToUse: agent.metadata.description,
      role: 'assistant', instructions: [agent.prompt.system, ...agent.prompt.instructions].join('\n'),
      provenance, input: {}, output: {}, skills: [], permittedChildren: [],
      context: { mode: 'selected', artifacts: [], history: 'none', memoryScopes: [] },
      requestedLimits: limits,
      models: [{ role: 'primary', capabilities: ['text', 'tool-calling'], preferredAliases: ['project-model'] }],
      sourceDescriptor: agent,
      sourceIdentity: { profile: agent.kind, id: agent.id, version: agent.version,
        digest: digest(canonical(agent)), revision: null },
      promptDialect: { required: true, value: 'default' },
      renameMapping: { sourceId: agent.id, targetId: agent.id, reason: 'unchanged' } });
    return finalize(document);
  });
  this.definitions = definitions;
  const teamDefinition = JSON.parse(await readFile('scripts/acceptance/fixtures/team-template.json', 'utf8'));
  this.guidance = 'Complete the assigned taskInput.request directly. Use your saved specialist instructions. Do not call tools or delegate: the host already created the project task graph. The writer receives selected artifacts as task data; for the final article integrate both completed math and outline outputs, never guess a missing result. Do not reveal planning or reasoning.';
  Object.assign(teamDefinition, { id: this.id + ':definition', title: 'Mathematics and editorial project',
    purpose: 'Explain a calculated community-garden water saving in an accessible article', provenance,
    members: definitions.map((definition, index) => ({ role: index === 0 ? 'math' : 'writer', kind: 'agent',
      definition: reference(definition), min: 1, max: 1, responsibility: index === 0 ? 'Calculate the project result' : 'Develop and write the article' })),
    coordinatorRole: 'writer', communication: [
      { fromRole: 'math', toRole: 'writer', modes: ['queue-only', 'trigger-turn'] },
      { fromRole: 'writer', toRole: 'math', modes: ['queue-only', 'trigger-turn'] }],
    limits, budget, instructions: { revision: 1, digest: digest(this.guidance), text: this.guidance } });
  finalize(teamDefinition);
  const documents = { 'math.json': definitions[0], 'writer.json': definitions[1], 'team.json': teamDefinition };
  const files = Object.fromEntries(Object.entries(documents).map(([path, value]) => [path, canonical(value)]));
  const manifest = finalize({ profile, kind: 'PackageManifest', id: this.id + ':package', version: '1.0.0',
    provenance, requiredCapabilities: ['collaboration_definition_packages_v2'], extensions: {},
    entrypoints: [reference(teamDefinition)],
    files: Object.entries(documents).map(([path, document]) => ({ path, kind: document.kind,
      definition: reference(document), byteDigest: digest(files[path]) })),
    lock: definitions.map((definition, index) => ({ requestedBy: teamDefinition.id,
      reference: reference(definition), resolvedPath: index === 0 ? 'math.json' : 'writer.json' })),
    capabilityDeclarations: [{ capability: 'collaboration_definition_packages_v2', required: true }],
    resolution: 'exact-version-and-digest' });
  this.evidence.package_documents = documents;
  this.evidence.manifest = manifest;
  await retain(this);
  this.evidence.install = await api(this, 'POST', '/packages:install', {
    commandId: randomUUID(), manifest: canonical(manifest), files }, 201);
  const binding = JSON.parse(await readFile('scripts/acceptance/fixtures/team-binding.json', 'utf8'));
  const split = process.env.UAR_MODEL.indexOf('/');
  assert.ok(split > 0);
  Object.assign(binding, { id: this.id + ':binding', package: reference(manifest), provenance,
    ownerId: this.caps.bindingOwnerId, workspaceId: process.env.UAR_WORKSPACE_ID,
    runtimeInstanceId: this.caps.instance.id, revision: 1,
    modelBindings: [{ requestedAlias: 'project-model', providerId: process.env.UAR_MODEL.slice(0, split),
      modelId: process.env.UAR_MODEL.slice(split + 1), credentialRef: 'protected-credential://uar-acceptance/fixture-primary',
      profile: { id: 'uar.openai-compatible-chat.settings-v1', revision: 1 }, settingsRevision: 1 }],
    skillBindings: [], contextGrants: [], representationGrantRefs: [], effectiveLimits: limits, effectiveBudget: budget,
    storage: { backend: this.caps.catalogStorage.backend, connectionRef: 'protected-connection://uar-acceptance/catalog', durableTransactions: true } });
  finalize(binding);
  this.evidence.binding = await api(this, 'POST', '/deployment-bindings', {
    commandId: randomUUID(), expectedRevision: 0, binding }, 201);
  assert.equal(this.evidence.binding.preflight.activationSupported, true);
  this.team = await api(this, 'POST', '/team-instances', { commandId: randomUUID(),
    deploymentBindingId: binding.id, teamDefinition: reference(teamDefinition),
    input: { project: 'Calculate garden water savings and write an accessible article about the result.' }, memberSlots: [] }, 201);
  assert.equal(this.team.members.length, 2);
  assert.deepEqual(this.team.members.map(member => member.definition.id).sort(), [this.math.id, this.writer.id].sort());
  this.evidence.team = this.team;
  await retain(this);
});

async function createTask(world, id, role, requestText, dependencies = []) {
  const team = await readTeam(world);
  await api(world, 'POST', teamPath(world) + '/tasks', { commandId: randomUUID(), taskId: id,
    expectedTeamRevision: team.revision, title: id, role, input: { request: requestText },
    outputContract: {}, dependsOn: dependencies }, 201);
}
async function claim(world, id) {
  const team = await readTeam(world);
  const task = team.tasks.find(task => task.id === id);
  const member = team.members.find(member => member.role === task.role);
  await api(world, 'POST', teamPath(world) + `/tasks/${id}/claim`, { commandId: randomUUID(),
    expectedTeamRevision: team.revision, expectedTaskRevision: task.revision, memberId: member.id });
}
async function admit(world, id, artifacts = []) {
  const team = await readTeam(world);
  const task = team.tasks.find(task => task.id === id);
  return api(world, 'POST', teamPath(world) + `/tasks/${id}/admit`, { commandId: randomUUID(),
    expectedTeamRevision: team.revision, expectedTaskRevision: task.revision,
    memberId: team.members.find(member => member.role === task.role).id, reservation, contextArtifactIds: artifacts });
}

When('I submit a project with independent math and editorial tasks and a dependent article', async function () {
  this.beds = randomInt(17, 29); this.plants = randomInt(21, 35); this.litres = randomInt(11, 18);
  this.expected = this.beds * this.plants * this.litres;
  this.title = `The Quiet Garden ${randomUUID().slice(0, 8)}`;
  const mathRequest = `A community garden has ${this.beds} beds, each with ${this.plants} plants. A drip system saves ${this.litres} litres per plant each week. Calculate the total litres saved per week. Return your saved specialist/answer JSON.`;
  const outlineRequest = `Prepare a 100-word editorial outline for an accessible article about community-garden water saving. Its exact headline must be "${this.title}". Discuss neighbor cooperation, responsible interpretation of calculated estimates, and a practical takeaway. Do not invent or calculate savings figures. Include your saved byline.`;
  const finalRequest = 'Write an accessible article of 100 to 160 words using BOTH selected completed artifacts: the math specialist result supplies the weekly litres saved, and the editorial outline supplies the exact headline and themes. Include the exact numeric math result in decimal digits and the unit litres per week, the outline headline, community cooperation and a practical takeaway. End with your saved byline. Return the article directly.';
  this.evidence.project = { request: 'Calculate the garden savings while independently planning an article; then combine the two completed results into the final article.',
    inputs: { beds: this.beds, plants_per_bed: this.plants, litres_per_plant_per_week: this.litres }, expected_litres: this.expected,
    tasks: [{ id: 'calculation', role: 'math', request: mathRequest }, { id: 'outline', role: 'writer', request: outlineRequest },
      { id: 'article', role: 'writer', request: finalRequest, dependsOn: ['calculation', 'outline'] }] };
  await retain(this);
  await createTask(this, 'calculation', 'math', mathRequest);
  await createTask(this, 'outline', 'writer', outlineRequest);
  await createTask(this, 'article', 'writer', finalRequest, ['calculation', 'outline']);
  await claim(this, 'calculation'); await claim(this, 'outline');
});

When('UAR executes the math and editorial tasks concurrently', async function () {
  this.mathAttempt = await admit(this, 'calculation');
  this.writerAttempt = await admit(this, 'outline');
  this.evidence.independent_attempts = [this.mathAttempt, this.writerAttempt];
  await retain(this);
  assert.notEqual(this.mathAttempt.rootId, this.writerAttempt.rootId);
  assert.notEqual(this.mathAttempt.memberId, this.writerAttempt.memberId);
  for (const attempt of [this.mathAttempt, this.writerAttempt]) {
    await api(this, 'GET', teamPath(this) + `/attempts/${encodeURIComponent(attempt.id)}/run-registration`);
  }
  // The same math run is running before AND after observing the writer running.
  // Terminal runs cannot restart under the same ID: this brackets real overlap.
  const snapshots = [];
  for (const attempt of [this.mathAttempt, this.writerAttempt, this.mathAttempt]) {
    const run = await request(this, 'GET', `/api/uar/runs/${encodeURIComponent(attempt.runId)}`);
    snapshots.push({ observed_at: new Date().toISOString(), run });
  }
  this.evidence.overlap = snapshots;
  await retain(this);
  for (const row of snapshots) assert.equal(row.run.status, 'running', 'No proven simultaneous live execution');
  assert.equal(snapshots[0].run.agent_id, this.math.id);
  assert.equal(snapshots[1].run.agent_id, this.writer.id);
  assert.notEqual(snapshots[0].run.conversation_id, snapshots[1].run.conversation_id);
});

async function waitFor(world, ids) {
  const deadline = Date.now() + 120_000;
  do {
    const value = await summary(world);
    world.evidence.latest_execution = value;
    for (const id of ids) {
      const attempt = value.attempts.find(attempt => attempt.taskId === id);
      assert.ok(attempt, `Missing attempt ${id}`);
      assert.ok(!['failed', 'cancelled', 'uncertain', 'yielded'].includes(attempt.status), JSON.stringify(attempt));
    }
    if (ids.every(id => value.attempts.find(attempt => attempt.taskId === id).status === 'succeeded')) return value;
    await delay(100);
  } while (Date.now() < deadline);
  assert.fail('Team tasks did not finish within 120 seconds');
}

Then('both specialists produce their own successful results', async function () {
  const execution = await waitFor(this, ['calculation', 'outline']);
  assert.ok(!execution.attempts.some(attempt => attempt.taskId === 'article'), 'Dependent article dispatched early');
  this.mathDone = execution.attempts.find(attempt => attempt.taskId === 'calculation');
  this.outlineDone = execution.attempts.find(attempt => attempt.taskId === 'outline');
  const answer = output(this.mathDone.output);
  assert.equal(answer.specialist, this.marker);
  assert.equal(answer.answer, this.expected);
  const outline = typeof this.outlineDone.output === 'string' ? this.outlineDone.output : JSON.stringify(this.outlineDone.output);
  assert.ok(outline.includes(this.title) && outline.includes(this.byline));
  for (const attempt of [this.mathDone, this.outlineDone]) {
    assert.equal(attempt.effectDisposition, 'confirmed');
    const calls = await request(this, 'GET', `/api/uar/accounting/calls/${encodeURIComponent(attempt.runId)}`);
    assert.ok(calls.length > 0 && calls.every(call => call.state === 'settled'
      && call.admission.model === process.env.UAR_MODEL && call.observed_tokens > 0));
    (this.evidence.provider_calls ??= {})[attempt.runId] = calls;
  }
  this.evidence.independent_results = [this.mathDone, this.outlineDone];
  await retain(this);
});

When('the writer receives both results to finish the project', async function () {
  const artifacts = await api(this, 'GET', teamPath(this) + '/artifacts');
  this.evidence.artifacts = artifacts;
  this.artifactIds = artifacts.filter(artifact => ['calculation', 'outline'].includes(artifact.taskId))
    .map(artifact => artifact.id);
  assert.equal(this.artifactIds.length, 2, 'Need two actual published result artifacts for the dependent task');
  await claim(this, 'article');
  this.articleAttempt = await admit(this, 'article', this.artifactIds);
  this.evidence.article_attempt = this.articleAttempt;
  await retain(this);
});

Then('the final article incorporates the math result and the editorial work', async function () {
  const execution = await waitFor(this, ['article']);
  const article = execution.attempts.find(attempt => attempt.taskId === 'article');
  assert.equal(article.memberId, this.outlineDone.memberId);
  assert.equal(article.effectDisposition, 'confirmed');
  const text = typeof article.output === 'string' ? article.output.replace(/^\s*<think>[\s\S]*?<\/think>\s*/, '').trim() : JSON.stringify(article.output);
  assert.ok(text.includes(String(this.expected)) || text.includes(this.expected.toLocaleString('en-US')));
  assert.match(text, /litres|liters/i);
  assert.match(text, /week/i);
  assert.ok(text.includes(this.title) && text.includes(this.byline));
  assert.match(text, /neighbou?r|cooperat|community/i);
  const words = text.split(/\s+/).length;
  assert.ok(words >= 80 && words <= 250, `Expected a short article, received ${words} words`);
  const calls = await request(this, 'GET', `/api/uar/accounting/calls/${encodeURIComponent(article.runId)}`);
  assert.ok(calls.length > 0 && calls.every(call => call.state === 'settled'
    && call.admission.model === process.env.UAR_MODEL && call.observed_tokens > 0));
  this.evidence.provider_calls[article.runId] = calls;
  const receipt = execution.contextReceipts.find(receipt => receipt.authority.attemptId === article.id);
  assert.ok(receipt, 'Missing actual final-writer selected context receipt');
  const selected = receipt.selections.filter(selection => selection.sourceKind === 'artifact'
    && selection.disposition === 'selected').map(selection => selection.sourceId);
  assert.deepEqual(selected.sort(), [...this.artifactIds].sort(), 'Final writer did not receive both actual output artifacts');
  this.evidence.final_context = receipt;
  this.evidence.final_article = text;
  this.evidence.result = 'VERIFIED';
  await retain(this);
});

After(async function ({ result }) {
  if (!this.evidence) return;
  this.evidence.scenario_status = result.status;
  await retain(this);
  this.attach(JSON.stringify(this.evidence), 'application/json');
});
