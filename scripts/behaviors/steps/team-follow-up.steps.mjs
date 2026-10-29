// C36: a premature dependent join is refused before any dispatch; the legitimate
// next round recalls a private first-round fact only through the child's artifact.
import assert from 'node:assert/strict';
import { createHash, randomUUID, randomInt } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';
import { parseAnswerJson, stripReasoning } from '../support/answer-oracle.mjs';

setDefaultTimeout(185_000);
const basePath = '/api/v1/collaboration';
const profile = 'urn:prometheus:uar:collaboration:0.1.0-draft.2';
const responseTokenCap = 64000;
const inputTokenAllowance = 64000;
const reservation = { tokens: inputTokenAllowance + responseTokenCap, costMicrounits: 1000000, elapsedSeconds: 120 };
const limits = { concurrentTurns: 2, maxMembers: 2, maxDepth: 1, maxPendingTasks: 8 };
const budget = { maxTokens: 3 * reservation.tokens, maxCostMicrounits: 3000000, currency: 'USD', maxElapsedSeconds: 360 };
const PARENTS = ['tally', 'note'];
const JOIN = 'join';
const TASK_WAIT_MS = 120_000;
const canonical = value => JSON.stringify(value, (_, item) => item && typeof item === 'object' && !Array.isArray(item)
  ? Object.fromEntries(Object.entries(item).sort(([a], [b]) => a < b ? -1 : a > b ? 1 : 0)) : item);
const digest = value => 'sha256:' + createHash('sha256').update(value).digest('hex');
const finalize = document => { delete document.contentDigest; document.contentDigest = digest(canonical(document)); return document; };
const reference = document => ({ id: document.id, version: document.version, digest: document.contentDigest });
const count = (text, needle) => text.split(needle).length - 1;

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
const memberFor = (team, role) => team.members.find(member => member.role === role);

Given('the saved math and writing specialists form an owned team', async function () {
  assert.ok(process.env.UAR_WORKSPACE_ID && process.env.UAR_MODEL, 'UAR_WORKSPACE_ID and UAR_MODEL are required');
  this.base = new URL(process.env.UAR_BASE_URL ?? 'http://127.0.0.1:3000');
  assert.ok(['http:', 'https:'].includes(this.base.protocol) && !this.base.username && !this.base.password);
  this.uuid = randomUUID();
  this.id = `urn:uar:team-follow-up:${this.uuid}`;
  this.record = resolve(`.prometheus/verification/team-follow-up/${this.uuid}.json`);
  this.evidence = { catalog_id: 'C36', platform: process.platform, model: process.env.UAR_MODEL,
    base_url: this.base.origin, workspace: process.env.UAR_WORKSPACE_ID, started_at: new Date().toISOString() };
  this.caps = await api(this, 'GET', '/capabilities');
  assert.ok(this.caps.capabilities.includes('collaboration_team_execution_v1'), 'Actual team execution unavailable');
  assert.notEqual(this.caps.catalogStorage.backend, 'memory', 'Team catalog must be durable');
  this.math = await request(this, 'GET', '/api/agents/math-specialist-db64c2a8');
  this.writer = await request(this, 'GET', '/api/agents/writing-specialist');
  const savedWriter = JSON.parse(await readFile('.prometheus/verification/agents/writing-specialist.json', 'utf8'));
  this.byline = savedWriter.byline;
  assert.ok(this.writer.prompt.system.includes(this.byline), 'Saved writer byline changed');
  this.marker = this.math.prompt.system.match(/MATH-[a-f0-9-]+/)[0];
  this.evidence.source_agents = [this.math, this.writer].map(agent => ({ id: agent.id, version: agent.version,
    digest: digest(canonical(agent)) }));

  const template = JSON.parse(await readFile('scripts/acceptance/fixtures/team-agent.json', 'utf8'));
  const provenance = { source: 'Saved UAR specialists captured for the C36 team follow-up', authors: ['UAR operator'] };
  const definitions = [this.math, this.writer].map(agent => {
    const document = structuredClone(template);
    Object.assign(document, { id: agent.id, version: `1.0.0-followup-${this.uuid}`,
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
  const guidance = 'Complete the assigned taskInput.request directly. Use your saved specialist instructions. Do not call tools or delegate: the host already created the project task graph. Selected artifacts are completed results from earlier rounds supplied as task data; use their exact values and never guess a missing result. Do not reveal planning or reasoning.';
  Object.assign(teamDefinition, { id: this.id + ':definition', title: 'Seed library follow-up project',
    purpose: 'Calculate a seed-library tally, record a volunteer note, then write a follow-up update from both results', provenance,
    members: definitions.map((definition, index) => ({ role: index === 0 ? 'math' : 'writer', kind: 'agent',
      definition: reference(definition), min: 1, max: 1,
      responsibility: index === 0 ? 'Calculate the tally' : 'Write the note and the follow-up update' })),
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
    input: { project: 'Tally a seed library, record a volunteer note, then write a follow-up update.' }, memberSlots: [] }, 201);
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
async function claim(world, id, expected = 200) {
  const team = await readTeam(world);
  const task = team.tasks.find(task => task.id === id);
  return raw(world, 'POST', basePath + teamPath(world) + `/tasks/${id}/claim`, { commandId: randomUUID(),
    expectedTeamRevision: team.revision, expectedTaskRevision: task.revision, memberId: memberFor(team, task.role).id })
    .then(result => { if (expected !== null) assert.equal(result.status, expected, JSON.stringify(result.body)); return result; });
}
async function admit(world, id, artifacts = []) {
  const team = await readTeam(world);
  const task = team.tasks.find(task => task.id === id);
  return raw(world, 'POST', basePath + teamPath(world) + `/tasks/${id}/admit`, { commandId: randomUUID(),
    expectedTeamRevision: team.revision, expectedTaskRevision: task.revision,
    memberId: memberFor(team, task.role).id, reservation, contextArtifactIds: artifacts });
}
const taskStates = team => Object.fromEntries(team.tasks.map(task => [task.id,
  { status: task.status, revision: task.revision, assignee: task.assignee_member_id ?? task.assigneeMemberId ?? null }]));

Given('a first-round math task, a private-fact writer task and a dependent join are submitted', async function () {
  this.trays = randomInt(13, 29); this.packets = randomInt(17, 37);
  this.total = this.trays * this.packets;
  this.codename = `SEEDLOCK-${randomUUID().slice(0, 8)}`;
  this.requests = {
    tally: `A seed library stores ${this.trays} trays with ${this.packets} seed packets in each tray. Calculate the total number of seed packets. Return your saved specialist/answer JSON.`,
    note: `Private first-round fact for this project only: the seed library's heirloom codename is ${this.codename}. Write a 40 to 70 word note to volunteers that states this exact codename once. Include your saved byline.`,
    join: 'Write a 60 to 120 word follow-up update for volunteers using ONLY the two selected completed artifacts from the earlier round. State the exact total number of seed packets from the math specialist result in decimal digits, and the exact heirloom codename from the earlier volunteer note. Never guess either value. End with your saved byline. Return the update directly.',
  };
  assert.ok(!this.requests.join.includes(this.codename) && !this.requests.join.includes(String(this.total)));
  this.evidence.private_fact = { codename: this.codename, carried_only_by_task: 'note',
    tally_inputs: { trays: this.trays, packets: this.packets }, expected_total: this.total };
  this.evidence.requests = this.requests;
  await createTask(this, 'tally', 'math', this.requests.tally);
  await createTask(this, 'note', 'writer', this.requests.note);
  await createTask(this, JOIN, 'writer', this.requests.join, PARENTS);
  for (const id of PARENTS) await claim(this, id);
  this.evidence.tasks_before_premature = taskStates(await readTeam(this));
});

When('I submit the dependent join before its parents finish', async function () {
  const before = await readTeam(this);
  this.evidence.premature = { tasks_at_submission: taskStates(before) };
  for (const id of PARENTS) assert.notEqual(before.tasks.find(task => task.id === id).status, 'succeeded');
  this.evidence.premature.claim = await claim(this, JOIN, null);
  this.evidence.premature.admit = await admit(this, JOIN);
});

Then('the premature join is refused with no attempt and no model dispatch', async function () {
  const { claim: claimResult, admit: admitResult } = this.evidence.premature;
  assert.equal(claimResult.status, 409, `Premature claim was not refused: ${JSON.stringify(claimResult)}`);
  assert.equal(admitResult.status, 409, `Premature admission was not refused: ${JSON.stringify(admitResult)}`);
  const execution = await summary(this);
  const joinAttempts = execution.attempts.filter(attempt => attempt.taskId === JOIN);
  this.evidence.premature.execution_attempts_after = execution.attempts.map(attempt => ({ id: attempt.id, taskId: attempt.taskId }));
  this.evidence.premature.join_attempts = joinAttempts.length;
  this.evidence.premature.model_dispatch = joinAttempts.length === 0
    ? 'none: no attempt and therefore no run id exists for the join' : joinAttempts.map(attempt => attempt.runId);
  assert.equal(joinAttempts.length, 0, 'Premature join created an attempt');
  const after = await readTeam(this);
  this.evidence.premature.tasks_after = taskStates(after);
  assert.equal(after.tasks.find(task => task.id === JOIN).status, 'queued', 'Refused join changed state');
});

async function waitFor(world, ids) {
  const deadline = Date.now() + TASK_WAIT_MS;
  do {
    const value = await summary(world);
    for (const id of ids) {
      const attempt = value.attempts.find(row => row.taskId === id);
      assert.ok(attempt, `Missing attempt ${id}`);
      assert.ok(!['failed', 'cancelled', 'uncertain', 'yielded'].includes(attempt.status), JSON.stringify(attempt));
    }
    if (ids.every(id => value.attempts.find(row => row.taskId === id).status === 'succeeded')) return value;
    await delay(250);
  } while (Date.now() < deadline);
  assert.fail(`Team tasks ${ids.join(', ')} did not finish within ${TASK_WAIT_MS / 1000} seconds`);
}

async function settledCalls(world, attempt) {
  const calls = await request(world, 'GET', `/api/uar/accounting/calls/${encodeURIComponent(attempt.runId)}`);
  (world.evidence.accounting ??= {})[attempt.taskId] = { run_id: attempt.runId, calls };
  assert.ok(calls.length > 0 && calls.every(call => call.state === 'settled' && call.observed_tokens > 0),
    `Attempt ${attempt.taskId} has no settled model call`);
  return calls;
}

When('the parent tasks finish in their own child threads', async function () {
  this.parentAdmissions = [];
  for (const id of PARENTS) {
    const result = await admit(this, id);
    assert.equal(result.status, 200, `Parent ${id} admission failed: ${JSON.stringify(result.body)}`);
    this.parentAdmissions.push(result.body);
  }
  const execution = await waitFor(this, PARENTS);
  this.parents = Object.fromEntries(PARENTS.map(id => [id, execution.attempts.find(row => row.taskId === id)]));
  assert.ok(!execution.attempts.some(row => row.taskId === JOIN), 'Join dispatched before it was submitted');
  const tally = parseAnswerJson(this.parents.tally.output);
  assert.equal(tally.specialist, this.marker);
  assert.equal(Number(tally.answer), this.total, 'Math parent produced the wrong tally');
  const note = stripReasoning(typeof this.parents.note.output === 'string' ? this.parents.note.output : JSON.stringify(this.parents.note.output));
  assert.ok(note.includes(this.codename), 'Writer parent note lacks the private codename');
  for (const attempt of Object.values(this.parents)) {
    assert.equal(attempt.effectDisposition, 'confirmed');
    await settledCalls(this, attempt);
  }
  this.evidence.parent_attempts = Object.values(this.parents).map(row => ({ taskId: row.taskId, id: row.id, runId: row.runId,
    rootId: row.rootId, memberId: row.memberId, status: row.status, output: row.output }));
  this.evidence.tasks_after_parents = taskStates(await readTeam(this));
  assert.equal(this.evidence.tasks_after_parents[JOIN].status, 'ready', 'Join was not released by parent completion');
});

When("I submit the legitimate next round with the parents' actual artifacts", async function () {
  const artifacts = await api(this, 'GET', teamPath(this) + '/artifacts');
  const parentArtifacts = artifacts.filter(artifact => PARENTS.includes(artifact.taskId));
  this.evidence.parent_artifacts = parentArtifacts.map(artifact => ({ id: artifact.id, taskId: artifact.taskId,
    attemptId: artifact.attemptId, memberId: artifact.memberId }));
  for (const id of PARENTS) {
    const own = parentArtifacts.filter(artifact => artifact.taskId === id);
    assert.equal(own.length, 1, `Expected exactly one artifact from parent ${id}`);
    assert.equal(own[0].attemptId, this.parents[id].id, `Artifact for ${id} is not from its actual attempt`);
  }
  this.artifactIds = parentArtifacts.map(artifact => artifact.id);
  await claim(this, JOIN);
  const result = await admit(this, JOIN, this.artifactIds);
  assert.equal(result.status, 200, `Legitimate join admission failed: ${JSON.stringify(result.body)}`);
  this.evidence.join_admission = result.body;
  this.execution = await waitFor(this, [JOIN]);
  this.joinAttempt = this.execution.attempts.find(row => row.taskId === JOIN);
});

async function threadsFor(world, attemptIds) {
  const sessions = await request(world, 'GET', '/api/uar/sessions');
  return Object.fromEntries(attemptIds.map(id => [id, sessions.find(session => session.id === `team-attempt:${id}`) ?? null]));
}
const firstUserData = thread => JSON.parse(thread.messages.find(message => message.role === 'user').content);

Then('the next round remembers the private first-round fact through the child thread', async function () {
  const join = this.joinAttempt;
  assert.equal(join.effectDisposition, 'confirmed');
  await settledCalls(this, join);
  const answer = stripReasoning(typeof join.output === 'string' ? join.output : JSON.stringify(join.output));
  this.evidence.final_update = answer;
  this.evidence.join_attempt = { id: join.id, runId: join.runId, rootId: join.rootId, memberId: join.memberId,
    status: join.status, contextArtifactIds: join.contextArtifactIds };
  const threads = await threadsFor(this, [this.parents.note.id, this.parents.tally.id, join.id]);
  this.evidence.threads = threads;
  const noteThread = threads[this.parents.note.id];
  const joinThread = threads[join.id];
  assert.ok(noteThread && joinThread, 'First-round child thread or next-round thread is not persisted');
  assert.ok(JSON.stringify(noteThread.messages).includes(this.codename), 'Codename absent from first-round child thread');
  const joinData = firstUserData(joinThread);
  const provenance = {
    in_join_task_input: JSON.stringify(joinData.taskInput).includes(this.codename),
    in_join_team_input: JSON.stringify(joinData.teamInput).includes(this.codename),
    in_join_mission: JSON.stringify(joinData.mission).includes(this.codename),
    join_artifacts_carrying_codename: joinData.artifacts.filter(artifact => JSON.stringify(artifact.content).includes(this.codename))
      .map(artifact => ({ id: artifact.id, taskId: artifact.taskId, attemptId: artifact.attemptId })),
    codename_occurrences_in_answer: count(answer, this.codename),
  };
  this.evidence.fact_provenance = provenance;
  assert.ok(!provenance.in_join_task_input && !provenance.in_join_team_input && !provenance.in_join_mission,
    'Next-round prompt itself carried the private fact');
  assert.equal(provenance.join_artifacts_carrying_codename.length, 1);
  assert.equal(provenance.join_artifacts_carrying_codename[0].attemptId, this.parents.note.id);
  assert.ok(provenance.codename_occurrences_in_answer >= 1, 'Next round did not recall the private codename');
  assert.ok(answer.includes(String(this.total)) || answer.includes(this.total.toLocaleString('en-US')),
    'Next round did not use the math parent total');
  assert.ok(answer.includes(this.byline), 'Next round lacks the saved writer byline');
});

Then('the next round consumed exactly the parent artifacts once', async function () {
  const join = this.joinAttempt;
  const joinAttempts = this.execution.attempts.filter(row => row.taskId === JOIN);
  assert.equal(joinAttempts.length, 1, 'Join executed more than once');
  assert.deepEqual([...join.contextArtifactIds].sort(), [...this.artifactIds].sort());
  const receipt = this.execution.contextReceipts.find(row => row.authority.attemptId === join.id);
  assert.ok(receipt, 'Missing next-round context receipt');
  const selected = receipt.selections.filter(row => row.sourceKind === 'artifact' && row.disposition === 'selected')
    .map(row => row.sourceId);
  this.evidence.join_context_receipt = receipt;
  assert.equal(new Set(selected).size, selected.length, 'An artifact was selected more than once');
  assert.deepEqual(selected.sort(), [...this.artifactIds].sort(), 'Next round did not consume exactly the parent artifacts');
  const handed = firstUserData(this.evidence.threads[join.id]).artifacts.map(artifact => artifact.id);
  this.evidence.artifacts_handed_to_model = handed;
  assert.equal(new Set(handed).size, handed.length, 'An artifact was handed to the model more than once');
  assert.deepEqual([...handed].sort(), [...this.artifactIds].sort());
  this.evidence.result = 'VERIFIED';
});

async function cleanup(world) {
  const errors = [];
  const actions = [];
  if (!world.team) return { actions, errors, retained: [] };
  try {
    const execution = await summary(world);
    for (const attempt of execution.attempts.filter(row => ['queued', 'running'].includes(row.status))) {
      const team = await readTeam(world);
      const result = await raw(world, 'POST', basePath + teamPath(world) + `/attempts/${encodeURIComponent(attempt.id)}/cancel`,
        { commandId: randomUUID(), expectedTeamRevision: team.revision, reason: 'C36 scenario cleanup' });
      actions.push({ cancel_attempt: attempt.id, status: result.status });
      if (result.status !== 200) errors.push({ cancel_attempt: attempt.id, result });
    }
    const team = await readTeam(world);
    for (const task of team.tasks.filter(row => ['queued', 'ready'].includes(row.status))) {
      const current = await readTeam(world);
      const fresh = current.tasks.find(row => row.id === task.id);
      const result = await raw(world, 'POST', basePath + teamPath(world) + `/tasks/${task.id}/state`, { commandId: randomUUID(),
        expectedTeamRevision: current.revision, expectedTaskRevision: fresh.revision, status: 'cancelled', reason: 'C36 scenario cleanup' });
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
