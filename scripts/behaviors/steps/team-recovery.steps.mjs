// C54: a team recovers after cancellation and an owned restart without duplicate work.
// The settled tally stays byte-identical, the cancelled note stays honest, recovery
// re-dispatches only what its design permits, and an explicit replacement round
// reaches a dependent conclusion over the original settled artifact.
import assert from 'node:assert/strict';
import { createHash, randomUUID, randomInt } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { parseAnswerJson, stripReasoning } from '../support/answer-oracle.mjs';
import { restartOwnedUar } from '../support/restart-owned-uar.mjs';

setDefaultTimeout(185_000);
const RESTART_TIMEOUT_MS = 420_000;
const basePath = '/api/v1/collaboration';
const profile = 'urn:prometheus:uar:collaboration:0.1.0-draft.2';
const responseTokenCap = 64000;
const inputTokenAllowance = 64000;
const reservation = { tokens: inputTokenAllowance + responseTokenCap, costMicrounits: 1000000, elapsedSeconds: 120 };
const limits = { concurrentTurns: 2, maxMembers: 2, maxDepth: 1, maxPendingTasks: 8 };
// Five admissions at most (tally, cancelled note, replacement note, replacement join);
// an unknown-usage cancelled attempt keeps its whole reservation.
const budget = { maxTokens: 5 * reservation.tokens, maxCostMicrounits: 5000000, currency: 'USD', maxElapsedSeconds: 900 };
const TALLY = 'tally';
const NOTE = 'note';
const JOIN = 'join';
const NOTE_RETRY = 'note-retry';
const JOIN_RETRY = 'join-retry';
const SOLO = 'Everything this task needs is already in its task data: do not call team_delegate, team_send or team_wait.';
const TASK_WAIT_MS = 150_000;
const CANCEL_SETTLE_MS = 90_000;
const READY_WAIT_MS = 90_000;
const ACTIVE = ['queued', 'running', 'cancellation_requested'];
const canonical = value => JSON.stringify(value, (_, item) => item && typeof item === 'object' && !Array.isArray(item)
  ? Object.fromEntries(Object.entries(item).sort(([a], [b]) => a < b ? -1 : a > b ? 1 : 0)) : item);
const digest = value => 'sha256:' + createHash('sha256').update(value).digest('hex');
const finalize = document => { delete document.contentDigest; document.contentDigest = digest(canonical(document)); return document; };
const reference = document => ({ id: document.id, version: document.version, digest: document.contentDigest });
const answerText = output => stripReasoning(typeof output === 'string' ? output : JSON.stringify(output));

async function raw(world, method, path, payload) {
  const response = await fetch(new URL(path, world.base), { method, redirect: 'error',
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': process.env.UAR_WORKSPACE_ID },
    body: payload === undefined ? undefined : JSON.stringify(payload), signal: AbortSignal.timeout(150_000) });
  const text = await response.text();
  let body = text;
  try { body = JSON.parse(text); } catch { /* keep text for evidence */ }
  return { status: response.status, body };
}
async function request(world, method, path, payload, expected = 200) {
  const { status, body } = await raw(world, method, path, payload);
  assert.equal(status, expected, `${method} ${path}: HTTP ${status}: ${JSON.stringify(body).slice(0, 2000)}`);
  return body;
}
const api = (world, method, suffix, payload, expected) => request(world, method, basePath + suffix, payload, expected);
const teamPath = world => `/team-instances/${encodeURIComponent(world.team.id)}`;
const readTeam = world => api(world, 'GET', teamPath(world));
const summary = world => api(world, 'GET', teamPath(world) + '/execution');
const artifactsOf = world => api(world, 'GET', teamPath(world) + '/artifacts');
const memberFor = (team, role) => team.members.find(member => member.role === role);
const attemptsFor = (execution, taskId) => execution.attempts.filter(row => row.taskId === taskId);
const taskStates = team => Object.fromEntries(team.tasks.map(task => [task.id,
  { status: task.status, revision: task.revision, stateReason: task.stateReason ?? task.state_reason ?? null }]));
const attemptView = row => row && ({ id: row.id, taskId: row.taskId, runId: row.runId, status: row.status,
  executionOutcome: row.executionOutcome ?? null, effectDisposition: row.effectDisposition,
  accountingState: row.accountingState, usage: row.usage ?? null, stateReason: row.stateReason ?? null,
  updatedAt: row.updatedAt });

Given('the saved math and writing specialists form an owned recovery team', async function () {
  assert.ok(process.env.UAR_WORKSPACE_ID && process.env.UAR_MODEL, 'UAR_WORKSPACE_ID and UAR_MODEL are required');
  this.base = new URL(process.env.UAR_BASE_URL ?? 'http://127.0.0.1:3000');
  assert.ok(['http:', 'https:'].includes(this.base.protocol) && !this.base.username && !this.base.password);
  this.uuid = randomUUID();
  this.id = `urn:uar:team-recovery:${this.uuid}`;
  this.record = resolve(`.prometheus/verification/team-recovery/${this.uuid}.json`);
  this.evidence = { catalog_id: 'C54', trace: 'U19.team-cancel-recovery', platform: process.platform,
    model: process.env.UAR_MODEL, base_url: this.base.origin, workspace: process.env.UAR_WORKSPACE_ID,
    started_at: new Date().toISOString() };
  this.caps = await api(this, 'GET', '/capabilities');
  assert.ok(this.caps.capabilities.includes('collaboration_team_execution_v1'), 'Actual team execution unavailable');
  assert.notEqual(this.caps.catalogStorage.backend, 'memory', 'Team catalog must be durable across the restart');
  this.evidence.instance_before = this.caps.instance;
  this.math = await request(this, 'GET', '/api/agents/math-specialist-db64c2a8');
  this.writer = await request(this, 'GET', '/api/agents/writing-specialist');
  const savedWriter = JSON.parse(await readFile('.prometheus/verification/agents/writing-specialist.json', 'utf8'));
  this.byline = savedWriter.byline;
  assert.ok(this.writer.prompt.system.includes(this.byline), 'Saved writer byline changed');
  this.marker = this.math.prompt.system.match(/MATH-[a-f0-9-]+/)[0];
  this.evidence.source_agents = [this.math, this.writer].map(agent => ({ id: agent.id, version: agent.version,
    digest: digest(canonical(agent)) }));

  const template = JSON.parse(await readFile('scripts/acceptance/fixtures/team-agent.json', 'utf8'));
  const provenance = { source: 'Saved UAR specialists captured for the C54 team recovery', authors: ['UAR operator'] };
  const definitions = [this.math, this.writer].map(agent => {
    const document = structuredClone(template);
    Object.assign(document, { id: agent.id, version: `1.0.0-recovery-${this.uuid}`,
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
  const teamDefinition = JSON.parse(await readFile('scripts/acceptance/fixtures/team-template.json', 'utf8'));
  const guidance = 'Complete the assigned taskInput.request directly. Use your saved specialist instructions. Do not call tools or delegate: the host already created the project task graph. Selected artifacts are completed results from earlier tasks supplied as task data; use their exact values and never guess a missing result. Do not reveal planning or reasoning.';
  Object.assign(teamDefinition, { id: this.id + ':definition', title: 'Seed library recovery project',
    purpose: 'Calculate a seed-library tally, record a volunteer note, then write an update from both results', provenance,
    members: definitions.map((definition, index) => ({ role: index === 0 ? 'math' : 'writer', kind: 'agent',
      definition: reference(definition), min: 1, max: 1,
      responsibility: index === 0 ? 'Calculate the tally' : 'Write the note and the update' })),
    coordinatorRole: 'writer', communication: [
      { fromRole: 'math', toRole: 'writer', modes: ['queue-only', 'trigger-turn'] },
      { fromRole: 'writer', toRole: 'math', modes: ['queue-only', 'trigger-turn'] }],
    limits, budget, instructions: { revision: 1, digest: digest(guidance), text: guidance } });
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
  this.evidence.install = await api(this, 'POST', '/packages:install', {
    commandId: randomUUID(), manifest: canonical(manifest), files }, 201);
  const binding = JSON.parse(await readFile('scripts/acceptance/fixtures/team-binding.json', 'utf8'));
  const split = process.env.UAR_MODEL.indexOf('/');
  assert.ok(split > 0, 'UAR_MODEL must be provider/model');
  Object.assign(binding, { id: this.id + ':binding', package: reference(manifest), provenance,
    ownerId: this.caps.bindingOwnerId, workspaceId: process.env.UAR_WORKSPACE_ID,
    runtimeInstanceId: this.caps.instance.id, revision: 1,
    modelBindings: [{ requestedAlias: 'project-model', providerId: process.env.UAR_MODEL.slice(0, split),
      modelId: process.env.UAR_MODEL.slice(split + 1), credentialRef: 'protected-credential://uar-acceptance/fixture-primary' }],
    skillBindings: [], contextGrants: [], representationGrantRefs: [], effectiveLimits: limits, effectiveBudget: budget,
    storage: { backend: this.caps.catalogStorage.backend, connectionRef: 'protected-connection://uar-acceptance/catalog', durableTransactions: true } });
  finalize(binding);
  this.evidence.binding = await api(this, 'POST', '/deployment-bindings', {
    commandId: randomUUID(), expectedRevision: 0, binding }, 201);
  assert.equal(this.evidence.binding.preflight.activationSupported, true);
  this.team = await api(this, 'POST', '/team-instances', { commandId: randomUUID(),
    deploymentBindingId: binding.id, teamDefinition: reference(teamDefinition),
    input: { project: 'Tally a seed library, record a volunteer note, then write an update from both results.' }, memberSlots: [] }, 201);
  assert.equal(this.team.members.length, 2);
  this.evidence.team = { id: this.team.id, members: this.team.members.map(member => ({ id: member.id, role: member.role,
    definition: member.definition.id })) };
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
  await api(world, 'POST', teamPath(world) + `/tasks/${id}/claim`, { commandId: randomUUID(),
    expectedTeamRevision: team.revision, expectedTaskRevision: task.revision, memberId: memberFor(team, task.role).id });
}
async function admit(world, id, artifacts = []) {
  const team = await readTeam(world);
  const task = team.tasks.find(task => task.id === id);
  const result = await raw(world, 'POST', basePath + teamPath(world) + `/tasks/${id}/admit`, { commandId: randomUUID(),
    expectedTeamRevision: team.revision, expectedTaskRevision: task.revision,
    memberId: memberFor(team, task.role).id, reservation, contextArtifactIds: artifacts });
  assert.equal(result.status, 200, `Admission of ${id} failed: ${JSON.stringify(result.body).slice(0, 2000)}`);
  return result.body;
}

// Wait for one attempt per task to settle; any non-success terminal state fails.
async function waitSucceeded(world, ids) {
  const deadline = Date.now() + TASK_WAIT_MS;
  do {
    const value = await summary(world);
    for (const id of ids) {
      const rows = attemptsFor(value, id);
      assert.equal(rows.length, 1, `Expected exactly one attempt for ${id}, found ${rows.length}`);
      assert.ok(!['failed', 'cancelled', 'uncertain', 'yielded'].includes(rows[0].status), JSON.stringify(attemptView(rows[0])));
    }
    if (ids.every(id => attemptsFor(value, id)[0].status === 'succeeded')) return value;
    await delay(250);
  } while (Date.now() < deadline);
  assert.fail(`Team tasks ${ids.join(', ')} did not finish within ${TASK_WAIT_MS / 1000} seconds`);
}

async function callsFor(world, runId) {
  return request(world, 'GET', `/api/uar/accounting/calls/${encodeURIComponent(runId)}`);
}
const callSummary = calls => ({ count: calls.length,
  settled: calls.filter(call => call.state === 'settled').length,
  ids: calls.map(call => call.admission?.identity?.call_id ?? null),
  states: calls.map(call => call.state), digest: digest(canonical(calls)) });
async function threadFor(world, attemptId) {
  const sessions = await request(world, 'GET', '/api/uar/sessions');
  const thread = sessions.find(session => session.id === `team-attempt:${attemptId}`) ?? null;
  return thread && { id: thread.id, status: thread.status ?? null, messages: thread.messages?.length ?? null,
    roles: (thread.messages ?? []).map(message => message.role) };
}

Given('a math tally, a long volunteer note and a dependent join are submitted', async function () {
  this.trays = randomInt(13, 29); this.packets = randomInt(17, 37);
  this.total = this.trays * this.packets;
  this.codename = `SEEDLOCK-${randomUUID().slice(0, 8)}`;
  this.requests = {
    [TALLY]: `A seed library stores ${this.trays} trays with ${this.packets} seed packets in each tray. Calculate the total number of seed packets. Return your saved specialist/answer JSON.`,
    [NOTE]: `Private fact for this project only: the seed library's heirloom codename is ${this.codename}. Write a detailed 1200 to 1500 word volunteer handbook with at least eight titled sections covering intake, sorting, labelling, storage, lending, returns, germination testing and safety. State this exact codename once in the introduction. Include your saved byline.`,
    [JOIN]: 'Write a 60 to 120 word update for volunteers using ONLY the two selected completed artifacts. State the exact total number of seed packets from the math specialist result in decimal digits, and the exact heirloom codename from the volunteer note. Never guess either value. End with your saved byline. Return the update directly. ' + SOLO,
    // C36 note wording plus an explicit no-teammate rule: attempts 1-2 failed when the
    // writer tried to fetch the tally through team_wait (see c54-team-recovery-attempt-00{1,2}-fail.json).
    [NOTE_RETRY]: `Private first-round fact for this project only: the seed library's heirloom codename is ${this.codename}. Write a 40 to 70 word note to volunteers that states this exact codename once. Include your saved byline. ${SOLO}`,
  };
  this.requests[JOIN_RETRY] = this.requests[JOIN];
  assert.ok(!this.requests[JOIN].includes(this.codename) && !this.requests[JOIN].includes(String(this.total)));
  this.evidence.private_fact = { codename: this.codename, tally_inputs: { trays: this.trays, packets: this.packets },
    expected_total: this.total };
  this.evidence.requests = this.requests;
  await createTask(this, TALLY, 'math', this.requests[TALLY]);
  await createTask(this, NOTE, 'writer', this.requests[NOTE]);
  await createTask(this, JOIN, 'writer', this.requests[JOIN], [TALLY, NOTE]);
  for (const id of [TALLY, NOTE]) await claim(this, id);
  this.evidence.tasks_initial = taskStates(await readTeam(this));
});

When('the math tally settles with its artifact', async function () {
  this.evidence.tally_admission = attemptView(await admit(this, TALLY));
  const execution = await waitSucceeded(this, [TALLY]);
  this.tally = attemptsFor(execution, TALLY)[0];
  assert.equal(this.tally.effectDisposition, 'confirmed');
  const tally = parseAnswerJson(this.tally.output);
  assert.equal(tally.specialist, this.marker);
  assert.equal(Number(tally.answer), this.total, 'Math task produced the wrong tally');
  const calls = await callsFor(this, this.tally.runId);
  assert.ok(calls.length > 0 && calls.every(call => call.state === 'settled' && call.observed_tokens > 0),
    'Settled tally has no settled model call');
  const artifacts = (await artifactsOf(this)).filter(artifact => artifact.taskId === TALLY);
  assert.equal(artifacts.length, 1, 'Expected exactly one tally artifact');
  assert.equal(artifacts[0].attemptId, this.tally.id);
  this.tallyArtifactId = artifacts[0].id;
  this.evidence.tally_settled = { attempt: attemptView(this.tally), artifact_id: this.tallyArtifactId,
    calls: callSummary(calls), answer: tally };
});

async function cancelAttempt(world, attemptId, reason) {
  let result;
  for (let tries = 0; tries < 5; tries++) {
    const team = await readTeam(world);
    result = await raw(world, 'POST', basePath + teamPath(world) + `/attempts/${encodeURIComponent(attemptId)}/cancel`,
      { commandId: randomUUID(), expectedTeamRevision: team.revision, reason });
    const revisionRace = result.status === 409 && JSON.stringify(result.body).includes('team revision changed');
    if (!revisionRace) return result;
  }
  return result;
}

When('I cancel the volunteer note once its attempt is running', async function () {
  const admitted = await admit(this, NOTE);
  this.evidence.note_admission = attemptView(admitted);
  const deadline = Date.now() + READY_WAIT_MS;
  let current = admitted;
  while (current.status === 'queued' && Date.now() < deadline) {
    await delay(200);
    current = attemptsFor(await summary(this), NOTE)[0];
  }
  assert.equal(current.status, 'running', `Note attempt is not running: ${JSON.stringify(attemptView(current))}`);
  const registration = await raw(this, 'GET', basePath + teamPath(this)
    + `/attempts/${encodeURIComponent(current.id)}/run-registration`);
  this.evidence.note_run_registration = { status: registration.status,
    attempt: registration.status === 200 ? attemptView(registration.body) : registration.body };
  assert.equal(registration.status, 200, `Note run did not register: ${JSON.stringify(registration.body).slice(0, 500)}`);
  assert.equal(registration.body.status, 'running', 'Note attempt left running before cancellation');
  const cancel = await cancelAttempt(this, current.id, 'C54 cancel the active volunteer note');
  this.evidence.note_cancel = { status: cancel.status, attempt: cancel.status === 200 ? attemptView(cancel.body) : cancel.body };
  assert.equal(cancel.status, 200, `Cancellation was refused: ${JSON.stringify(cancel.body).slice(0, 1000)}`);
  assert.ok(['cancellation_requested', 'cancelled'].includes(cancel.body.status), `Unexpected cancel receipt ${cancel.body.status}`);
  this.noteAttemptId = current.id;
  this.noteRunId = current.runId;
  const settleDeadline = Date.now() + CANCEL_SETTLE_MS;
  let settled;
  do {
    settled = attemptsFor(await summary(this), NOTE)[0];
    if (!ACTIVE.includes(settled.status)) break;
    await delay(250);
  } while (Date.now() < settleDeadline);
  this.noteBeforeRestart = settled;
  this.evidence.note_after_cancel = attemptView(settled);
});

Then('the cancelled note reports an honest non-success state', async function () {
  const note = this.noteBeforeRestart;
  assert.ok(['cancelled', 'uncertain'].includes(note.status),
    `Cancelled note did not reach an honest terminal state: ${JSON.stringify(attemptView(note))}`);
  assert.notEqual(note.executionOutcome, 'succeeded');
  assert.equal(note.output ?? null, null, 'Cancelled note carries an output');
  const artifacts = await artifactsOf(this);
  assert.equal(artifacts.filter(artifact => artifact.taskId === NOTE).length, 0, 'Cancelled note published an artifact');
  const team = await readTeam(this);
  this.evidence.tasks_after_cancel = taskStates(team);
  assert.notEqual(team.tasks.find(task => task.id === NOTE).status, 'succeeded');
  assert.equal(team.tasks.find(task => task.id === JOIN).status, 'queued', 'Join was released by a cancelled parent');
  this.before = await snapshot(this);
  this.evidence.before_restart = this.before.evidence;
});

async function snapshot(world) {
  const execution = await summary(world);
  const artifacts = await artifactsOf(world);
  const tally = attemptsFor(execution, TALLY);
  const note = attemptsFor(execution, NOTE);
  const tallyArtifact = artifacts.find(artifact => artifact.id === world.tallyArtifactId) ?? null;
  const tallyCalls = await callsFor(world, world.tally.runId);
  const noteCalls = await callsFor(world, world.noteRunId);
  const team = await readTeam(world);
  return { tallyArtifact, tallyAttempt: tally[0], noteAttempt: note[0], tallyCalls, noteCalls,
    attemptIds: execution.attempts.map(row => row.id).sort(),
    evidence: {
      attempt_count: execution.attempts.length, attempts_per_task: Object.fromEntries([TALLY, NOTE, JOIN]
        .map(id => [id, attemptsFor(execution, id).length])),
      tally_artifact_digest: tallyArtifact && digest(canonical(tallyArtifact)),
      tally_attempt_digest: tally[0] && digest(canonical(tally[0])),
      artifact_ids: artifacts.map(artifact => artifact.id).sort(),
      tally_calls: callSummary(tallyCalls), note_calls: callSummary(noteCalls),
      note_attempt: attemptView(note[0]), note_thread: await threadFor(world, world.noteAttemptId),
      tasks: taskStates(team), uncertain_attempts: execution.uncertainAttempts ?? null,
      committed: execution.committed ?? null, reserved: execution.reserved ?? null,
    } };
}

When('I restart the owned UAR', { timeout: RESTART_TIMEOUT_MS }, async function () {
  const restart = {};
  this.evidence.restart = restart;
  await restartOwnedUar(restart);
  const deadline = Date.now() + READY_WAIT_MS;
  let caps;
  do {
    caps = await raw(this, 'GET', basePath + '/capabilities').catch(error => ({ status: 0, body: String(error) }));
    if (caps.status === 200) break;
    await delay(500);
  } while (Date.now() < deadline);
  assert.equal(caps.status, 200, `Restarted UAR is not serving collaboration: ${JSON.stringify(caps.body).slice(0, 500)}`);
  this.evidence.instance_after = caps.body.instance;
  const owner = await raw(this, 'GET', basePath + '/execution-owner');
  this.evidence.execution_owner_after = owner;
  assert.equal(owner.status, 200);
  assert.equal(owner.body.ownsExecution, true, 'Restarted runtime does not own team execution');
});

Then('the settled tally artifact, attempt and model calls are byte-identical after the restart', async function () {
  this.after = await snapshot(this);
  this.evidence.after_restart = this.after.evidence;
  assert.ok(this.before.tallyArtifact && this.after.tallyArtifact, 'Tally artifact missing around the restart');
  assert.equal(canonical(this.after.tallyArtifact), canonical(this.before.tallyArtifact), 'Tally artifact changed across the restart');
  assert.equal(canonical(this.after.tallyAttempt), canonical(this.before.tallyAttempt), 'Settled tally attempt changed across the restart');
  assert.equal(canonical(this.after.tallyCalls), canonical(this.before.tallyCalls), 'Settled tally model calls changed across the restart');
  assert.deepEqual(this.after.attemptIds, this.before.attemptIds, 'The restart created or removed attempts');
});

Then('the cancelled note is still reported honestly with no new model calls', async function () {
  const note = this.after.noteAttempt;
  assert.ok(['cancelled', 'uncertain'].includes(note.status), `Note after restart: ${JSON.stringify(attemptView(note))}`);
  assert.notEqual(note.executionOutcome, 'succeeded');
  assert.equal(note.output ?? null, null);
  assert.equal(this.after.noteCalls.length, this.before.noteCalls.length, 'Cancelled note gained model calls across the restart');
  assert.deepEqual(callSummary(this.after.noteCalls).ids, callSummary(this.before.noteCalls).ids);
  this.evidence.note_calls_unchanged_bytes = canonical(this.after.noteCalls) === canonical(this.before.noteCalls);
});

When('I ask the team to recover only what its recovery design permits', async function () {
  const team = await readTeam(this);
  const result = await raw(this, 'POST', basePath + teamPath(this) + '/recover',
    { commandId: randomUUID(), expectedTeamRevision: team.revision, reason: 'C54 resume permitted work after the owned restart' });
  this.evidence.recover = { status: result.status, body: Array.isArray(result.body) ? result.body.map(attemptView) : result.body };
  assert.equal(result.status, 200, `Recovery control failed: ${JSON.stringify(result.body).slice(0, 1000)}`);
  this.recovered = result.body;
  this.afterRecover = await snapshot(this);
  this.evidence.after_recover = this.afterRecover.evidence;
});

Then('recovery re-dispatches nothing and the dependent of the cancelled note stays blocked', async function () {
  assert.deepEqual(this.recovered, [], 'Recovery re-dispatched attempts');
  assert.deepEqual(this.afterRecover.attemptIds, this.before.attemptIds, 'Recovery created attempts');
  assert.equal(canonical(this.afterRecover.tallyArtifact), canonical(this.before.tallyArtifact));
  assert.equal(canonical(this.afterRecover.tallyCalls), canonical(this.before.tallyCalls));
  assert.equal(this.afterRecover.noteCalls.length, this.before.noteCalls.length);
  const team = await readTeam(this);
  const join = team.tasks.find(task => task.id === JOIN);
  const probe = await raw(this, 'POST', basePath + teamPath(this) + `/tasks/${JOIN}/state`, { commandId: randomUUID(),
    expectedTeamRevision: team.revision, expectedTaskRevision: join.revision, status: 'ready',
    reason: 'C54 probe: release the original join over a cancelled parent' });
  this.evidence.original_join_release_probe = probe;
  assert.equal(probe.status, 409, `Original join was released over a cancelled parent: ${JSON.stringify(probe.body).slice(0, 500)}`);
  assert.equal((await readTeam(this)).tasks.find(task => task.id === JOIN).status, 'queued');
  this.evidence.permitted_resume = {
    recovery_redispatches: 'only queued attempts; running/cancellation_requested become uncertain (compiler/collaboration/team_execution/recovery.rs:155-169, runtime/team_execution/controller.rs:486-498)',
    cancelled_task: 'terminal and immutable (compiler/collaboration/team_planning/claims.rs:62-65); queued->ready requires succeeded parents (claims.rs:165-169); admission requires a ready task (team_execution/admission.rs:100-111)',
    action: 'submit an explicit replacement note task and a replacement join over the settled tally artifact',
  };
});

When('I submit an explicit replacement note and a replacement join over the settled tally artifact', async function () {
  await createTask(this, NOTE_RETRY, 'writer', this.requests[NOTE_RETRY]);
  await createTask(this, JOIN_RETRY, 'writer', this.requests[JOIN_RETRY], [TALLY, NOTE_RETRY]);
  await claim(this, NOTE_RETRY);
  this.evidence.note_retry_admission = attemptView(await admit(this, NOTE_RETRY));
  const execution = await waitSucceeded(this, [NOTE_RETRY]);
  this.noteRetry = attemptsFor(execution, NOTE_RETRY)[0];
  assert.equal(this.noteRetry.effectDisposition, 'confirmed');
  const note = answerText(this.noteRetry.output);
  assert.ok(note.includes(this.codename), 'Replacement note lacks the codename');
  const artifacts = await artifactsOf(this);
  const retryArtifacts = artifacts.filter(artifact => artifact.taskId === NOTE_RETRY);
  assert.equal(retryArtifacts.length, 1);
  assert.equal(retryArtifacts[0].attemptId, this.noteRetry.id);
  this.artifactIds = [this.tallyArtifactId, retryArtifacts[0].id];
  this.evidence.note_retry = { attempt: attemptView(this.noteRetry), artifact_id: retryArtifacts[0].id,
    calls: callSummary(await callsFor(this, this.noteRetry.runId)), answer: note };
  assert.equal((await readTeam(this)).tasks.find(task => task.id === JOIN_RETRY).status, 'ready', 'Replacement join not released');
  await claim(this, JOIN_RETRY);
  this.evidence.join_retry_admission = attemptView(await admit(this, JOIN_RETRY, this.artifactIds));
  this.execution = await waitSucceeded(this, [JOIN_RETRY]);
  this.joinRetry = attemptsFor(this.execution, JOIN_RETRY)[0];
});

Then('the replacement join reaches a legitimate conclusion from both artifacts', async function () {
  const join = this.joinRetry;
  assert.equal(join.effectDisposition, 'confirmed');
  const calls = await callsFor(this, join.runId);
  assert.ok(calls.length > 0 && calls.every(call => call.state === 'settled' && call.observed_tokens > 0));
  const answer = answerText(join.output);
  this.evidence.conclusion = answer;
  this.evidence.join_retry = { attempt: attemptView(join), contextArtifactIds: join.contextArtifactIds, calls: callSummary(calls) };
  assert.deepEqual([...join.contextArtifactIds].sort(), [...this.artifactIds].sort());
  const receipt = this.execution.contextReceipts?.find(row => row.authority.attemptId === join.id);
  assert.ok(receipt, 'Missing replacement join context receipt');
  const selected = receipt.selections.filter(row => row.sourceKind === 'artifact' && row.disposition === 'selected')
    .map(row => row.sourceId).sort();
  this.evidence.join_retry_selected_artifacts = selected;
  assert.deepEqual(selected, [...this.artifactIds].sort(), 'Replacement join did not consume exactly the settled artifacts');
  assert.ok(answer.includes(String(this.total)) || answer.includes(this.total.toLocaleString('en-US')),
    'Conclusion lacks the settled tally total');
  assert.ok(answer.includes(this.codename), 'Conclusion lacks the codename from the replacement note');
  assert.ok(answer.includes(this.byline), 'Conclusion lacks the saved writer byline');
});

Then('no settled tally work or cancelled note attempt was silently repeated', async function () {
  const final = await snapshot(this);
  this.evidence.final = final.evidence;
  const execution = await summary(this);
  assert.equal(attemptsFor(execution, TALLY).length, 1, 'Tally executed more than once');
  assert.equal(attemptsFor(execution, NOTE).length, 1, 'Cancelled note was re-dispatched');
  assert.equal(attemptsFor(execution, JOIN).length, 0, 'Original join ran over a cancelled parent');
  assert.equal(canonical(final.tallyArtifact), canonical(this.before.tallyArtifact), 'Tally artifact changed by the end');
  assert.equal(canonical(final.tallyAttempt), canonical(this.before.tallyAttempt), 'Tally attempt changed by the end');
  assert.equal(canonical(final.tallyCalls), canonical(this.before.tallyCalls), 'Tally model calls changed by the end');
  assert.equal(final.noteCalls.length, this.before.noteCalls.length, 'Cancelled note gained model calls by the end');
  assert.ok(['cancelled', 'uncertain'].includes(final.noteAttempt.status));
  this.evidence.result = 'VERIFIED';
});

async function cleanup(world) {
  const errors = [];
  const actions = [];
  if (!world.team) return { actions, errors, retained: [] };
  try {
    const execution = await summary(world);
    for (const attempt of execution.attempts.filter(row => ['queued', 'running'].includes(row.status))) {
      const result = await cancelAttempt(world, attempt.id, 'C54 scenario cleanup');
      actions.push({ cancel_attempt: attempt.id, status: result.status });
      if (result.status !== 200) errors.push({ cancel_attempt: attempt.id, result });
    }
    const team = await readTeam(world);
    for (const task of team.tasks.filter(row => ['queued', 'ready'].includes(row.status))) {
      const current = await readTeam(world);
      const fresh = current.tasks.find(row => row.id === task.id);
      const result = await raw(world, 'POST', basePath + teamPath(world) + `/tasks/${task.id}/state`, { commandId: randomUUID(),
        expectedTeamRevision: current.revision, expectedTaskRevision: fresh.revision, status: 'cancelled', reason: 'C54 scenario cleanup' });
      actions.push({ cancel_task: task.id, status: result.status });
      if (result.status !== 200) errors.push({ cancel_task: task.id, result });
    }
  } catch (error) {
    errors.push({ error: String(error) });
  }
  // The collaboration API exposes no delete route for team instances, bindings or packages.
  return { actions, errors, retained: [world.team.id, `${world.id}:binding`, `${world.id}:package`] };
}

After(async function ({ result }) {
  if (!this.evidence) return;
  this.evidence.scenario_status = result.status;
  if (result.message) this.evidence.failure_message = result.message;
  this.evidence.cleanup = await cleanup(this);
  this.evidence.finished_at = new Date().toISOString();
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { mode: 0o600, flag: 'wx' });
  this.attach(JSON.stringify({ evidence: this.record, status: result.status }), 'application/json');
});
