import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { randomUUID, randomInt } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { createServer } from 'node:net';
import { dirname, join, resolve } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';

setDefaultTimeout(125_000);

async function request(world, method, path, payload) {
  const headers = { 'Content-Type': 'application/json' };
  const response = await fetch(new URL(path, world.base), {
    method, headers, redirect: 'error', signal: AbortSignal.timeout(120_000),
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  const text = await response.text();
  assert.ok(response.ok, `${method} ${path}: HTTP ${response.status}: ${text.slice(0, 1000)}`);
  return { status: response.status, body: JSON.parse(text) };
}

async function startServer(world) {
  // Refuse an occupied native listener before starting or modifying any server.
  await new Promise((resolvePort, rejectPort) => {
    const reservation = createServer();
    reservation.once('error', rejectPort);
    reservation.listen(world.listener.port, world.listener.host, () =>
      reservation.close(error => error ? rejectPort(error) : resolvePort()));
  });
  const child = spawn(world.binary, ['--config', world.config], {
    cwd: dirname(world.config), env: world.serverEnvironment, stdio: ['ignore', 'pipe', 'pipe'],
  });
  world.server = child;
  world.serverOutput = '';
  world.exited = new Promise((resolveExit) => {
    child.once('error', error => resolveExit({ error: error.message }));
    child.once('exit', (code, signal) => resolveExit({ code, signal }));
  });
  for (const stream of [child.stdout, child.stderr]) {
    stream.on('data', data => { world.serverOutput = (world.serverOutput + data.toString()).slice(-4000); });
  }
  const deadline = Date.now() + 60_000;
  while (Date.now() < deadline) {
    assert.ok(child.pid && child.exitCode === null && child.signalCode === null, 'Owned UAR exited during startup');
    try {
      const response = await fetch(new URL('/api/agents/default-agent', world.base), {
        redirect: 'error', signal: AbortSignal.timeout(1000),
      });
      if (response.ok) {
        world.defaultAgent = await response.json();
        assert.ok(child.exitCode === null && child.signalCode === null, 'Owned UAR exited during readiness');
        return;
      }
      await response.body?.cancel();
    } catch (error) {
      if (!(error instanceof TypeError || error.name === 'TimeoutError')) throw error;
    }
    await delay(250);
  }
  throw new Error('Owned UAR did not become ready within 60 seconds');
}

async function stopServer(world) {
  if (!world.server) return;
  const child = world.server;
  if (child.exitCode === null && child.signalCode === null) child.kill('SIGTERM');
  const exit = await Promise.race([world.exited, delay(30_000, null, { ref: false })]);
  if (!exit) {
    child.kill('SIGKILL');
    await world.exited;
    world.server = null;
    throw new Error('Owned UAR did not shut down gracefully within 30 seconds');
  }
  world.server = null;
  assert.equal(exit.code, 0, `Owned UAR shutdown failed: ${JSON.stringify(exit)}`);
}

Given('a dedicated UAR instance with durable storage and a configured real model', async function () {
  assert.ok(process.env.UAR_SERVER_BIN && process.env.UAR_SERVER_CONFIG && process.env.UAR_MODEL,
    'Set UAR_SERVER_BIN, UAR_SERVER_CONFIG and UAR_MODEL');
  this.binary = resolve(process.env.UAR_SERVER_BIN);
  this.config = resolve(process.env.UAR_SERVER_CONFIG);
  this.base = new URL(process.env.UAR_BASE_URL ?? 'http://127.0.0.1:3000');
  assert.ok(['127.0.0.1', 'localhost', '[::1]'].includes(this.base.hostname), 'Use a loopback client endpoint');
  const configured = JSON.parse(await readFile(this.config, 'utf8'));
  this.listener = { host: configured.server.host, port: configured.server.port };
  assert.ok(['127.0.0.1', 'localhost', '::1'].includes(this.listener.host), 'Use a loopback native listener');
  assert.ok(['http:', 'https:'].includes(this.base.protocol), 'UAR_BASE_URL must use HTTP or HTTPS');
  assert.ok(!this.base.username && !this.base.password, 'UAR_BASE_URL must not contain URL credentials');
  this.serverEnvironment = {
    ...process.env,
    HOME: join(dirname(this.config), 'home'), TMPDIR: join(dirname(this.config), 'tmp'), RUST_LOG: 'warn',
  };
  this.startedAt = performance.now();
  await startServer(this);
});

When('I create and save a mathematics specialist agent', async function () {
  const artifact = structuredClone(this.defaultAgent);
  this.agentId = `math-specialist-${randomUUID().slice(0, 8)}`;
  this.marker = `MATH-${randomUUID()}`;
  this.instructions = `You are a mathematics specialist for arithmetic, calculations and numerical problems. ` +
    `Always return only a JSON object with keys specialist and answer. ` +
    `Set specialist to "${this.marker}". Set answer to the numeric result requested by the user.`;
  artifact.id = this.agentId;
  artifact.version = '1.0.0';
  artifact.metadata = { ...artifact.metadata, title: 'Mathematics Specialist',
    description: 'Specialist for arithmetic, numerical calculations, addition, subtraction, multiplication and division.',
    tags: ['mathematics', 'arithmetic', 'calculations'] };
  artifact.prompt = { system: this.instructions, instructions: [] };
  artifact.runtime.entry = 'default';
  const model = process.env.UAR_MODEL;
  const split = model.indexOf('/');
  assert.ok(split > 0, 'UAR_MODEL must be provider/model');
  artifact.policy.provider = { default: { provider: model.slice(0, split), model: model.slice(split + 1) }, fallbacks: [] };
  artifact.policy.tools.allow = [];
  artifact.memory.kb.enabled = false;
  const created = await request(this, 'POST', '/api/agents', artifact);
  assert.equal(created.status, 201);
  assert.equal(created.body.id, this.agentId);
  this.savedArtifact = created.body;
  this.record = resolve(process.env.UAR_AGENT_RECORD ?? `.prometheus/verification/agents/${this.agentId}.json`);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify({ agent_id: this.agentId, agent: this.savedArtifact,
    base_url: this.base.origin, server_config: this.config, model, status: 'CREATED_NOT_YET_VERIFIED' }, null, 2) + '\n',
    { flag: 'wx', mode: 0o600 });
});

When('I restart that UAR instance', async function () {
  await stopServer(this);
  await startServer(this);
  this.restarted = true;
});

Then('the saved agent definition is still available', async function () {
  const saved = await request(this, 'GET', `/api/agents/${encodeURIComponent(this.agentId)}`);
  assert.equal(saved.body.id, this.agentId);
  assert.equal(saved.body.prompt.system, this.instructions);
  assert.deepEqual(saved.body.policy.provider.default, this.savedArtifact.policy.provider.default);
});

Then('querying the agent follows its saved instructions', async function () {
  const left = randomInt(10, 50);
  const right = randomInt(10, 50);
  const response = await request(this, 'POST', '/v1/chat/completions', {
    agent_id: this.agentId, model: process.env.UAR_MODEL, session_id: randomUUID(),
    stream: false, memory_enabled: false, max_completion_tokens: 512,
    messages: [{ role: 'user', content: `What is ${left} + ${right}? Apply your saved instructions.` }],
  });
  this.answer = response.body.choices?.[0]?.message?.content;
  assert.equal(response.body.model, process.env.UAR_MODEL);
  assert.equal(typeof this.answer, 'string');
  const final = this.answer.replace(/^\s*<think>[\s\S]*?<\/think>\s*/, '').trim();
  assert.deepEqual(JSON.parse(final), { specialist: this.marker, answer: left + right });
  this.verified = true;
});

Given('the saved mathematics specialist is available in the running UAR', async function () {
  assert.ok(process.env.UAR_AGENT_RECORD && process.env.UAR_MODEL,
    'Set UAR_AGENT_RECORD and UAR_MODEL');
  this.startedAt = performance.now();
  this.base = new URL(process.env.UAR_BASE_URL ?? 'http://127.0.0.1:3000');
  assert.ok(!this.base.username && !this.base.password, 'UAR_BASE_URL must not contain URL credentials');
  this.record = resolve(process.env.UAR_AGENT_RECORD);
  const saved = JSON.parse(await readFile(this.record, 'utf8'));
  this.agentId = saved.agent_id;
  this.instructions = saved.agent.prompt.system;
  const marker = this.instructions.match(/Set specialist to "([^"]+)"/);
  assert.ok(marker, 'Saved specialist marker is missing');
  this.marker = marker[1];
  const loaded = await request(this, 'GET', `/api/agents/${encodeURIComponent(this.agentId)}`);
  assert.equal(loaded.body.prompt.system, this.instructions);
  this.keepRunning = true;
});

When('I ask that specialist to add two numbers', async function () {
  const left = randomInt(10, 50);
  const right = randomInt(10, 50);
  this.question = `What is ${left} + ${right}? Apply your saved instructions.`;
  this.expectedSum = left + right;
  const response = await request(this, 'POST', '/v1/chat/completions', {
    agent_id: this.agentId, model: process.env.UAR_MODEL, session_id: randomUUID(),
    stream: false, memory_enabled: false, max_completion_tokens: 512,
    messages: [{ role: 'user', content: this.question }],
  });
  assert.equal(response.body.model, process.env.UAR_MODEL);
  this.answer = response.body.choices?.[0]?.message?.content;
  assert.equal(typeof this.answer, 'string');
});

Then('its final answer uses its saved identity and the correct sum', function () {
  // MiniMax may prefix a completed reasoning block before the final answer.
  this.finalAnswer = this.answer.replace(/^\s*<think>[\s\S]*?<\/think>\s*/, '').trim();
  assert.deepEqual(JSON.parse(this.finalAnswer), { specialist: this.marker, answer: this.expectedSum });
  this.verified = true;
});

After(async function ({ result }) {
  let cleanupError;
  try {
    if (!this.keepRunning) await stopServer(this);
  } catch (error) {
    cleanupError = error;
  }
  const evidence = { behavior: this.keepRunning ? 'Query the previously persisted agent' : 'Create, persist and query a reusable agent',
    status: cleanupError ? 'FAILED' : result.status,
    service_kept_running: this.keepRunning ?? false,
    cleanup_confirmed: this.keepRunning ? undefined : !cleanupError,
    client_platform: process.platform, agent_id: this.agentId, agent_record: this.record,
    restarted: this.restarted ?? false, question: this.question, expected_sum: this.expectedSum,
    answer: this.finalAnswer ?? this.answer, model: process.env.UAR_MODEL,
    provider_reasoning_present: this.finalAnswer === undefined ? undefined : this.finalAnswer !== this.answer.trim(),
    duration_ms: this.startedAt === undefined ? undefined : Math.round(performance.now() - this.startedAt) };
  if (this.record) await writeFile(`${this.record}.${Date.now()}.result.json`, JSON.stringify(evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify(evidence, null, 2), 'application/json');
  if (cleanupError) throw cleanupError;
});
