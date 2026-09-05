import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { createHmac, randomBytes } from 'node:crypto';
import { once } from 'node:events';
import { mkdir, mkdtemp, readFile, writeFile } from 'node:fs/promises';
import { createServer } from 'node:http';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';

// Opt-in phase-boundary smoke. This forwards every inference request to the
// configured real provider without supplying fixtures or changing its response.
const binary = resolve(process.argv[2] ?? 'target/debug/uar-sidecar');
const options = new Set(process.argv.slice(3));
const integrationHelper = options.has('--integration-helper');
const beforeFirstResponse = options.has('--before-first-response');
const cancellationPoint = beforeFirstResponse ? 'awaiting-first-provider-response' : 'during-child-text';
const base = process.env.LITER_LLM_BASE_URL;
const key = process.env.LITER_LLM_MASTER_KEY;
assert(base && key, 'LITER_LLM_BASE_URL and LITER_LLM_MASTER_KEY are required');
const model = process.env.UAR_SMOKE_MODEL ?? 'gpt-5.4';
const scratch = await mkdtemp(join(tmpdir(), 'uar-live-cancellation-'));
const secret = randomBytes(32).toString('hex');
const calls = [];
const lifecycles = [];
const shadows = [];
let server;
let serverExit;
let startupError;
let runId;
let rootCancelled = false;
let cancellation;
let address;
let serverLog = '';

const observer = createServer(async (request, response) => {
  const controller = new AbortController();
  const chunks = [];
  for await (const chunk of request) chunks.push(chunk);
  const body = Buffer.concat(chunks);
  const record = {
    index: calls.length, status: null, textChunks: 0, closed: false,
    upstreamCompleted: false, downstreamAborted: false, readAborted: false, childRequest: false,
  };
  response.on('close', () => {
    record.downstreamAborted = !response.writableFinished;
    controller.abort();
  });
  calls.push(record);
  try {
    const payload = JSON.parse(body.toString('utf8'));
    record.inputBytes = body.length;
    record.model = payload.model;
    record.messageCount = payload.messages?.length ?? 0;
    record.toolCount = payload.tools?.length ?? 0;
    record.streaming = payload.stream === true;
    record.toolNames = payload.tools?.map(tool => tool.function?.name) ?? [];
    record.parameterNames = Object.keys(payload).filter(name => !['messages', 'tools'].includes(name));
    record.childRequest = payload.messages?.some(message => message.role === 'system'
      && JSON.stringify(message.content).includes('Answer the delegated task using concrete evidence. Distinguish observations from assumptions.')) ?? false;
    console.log(`provider attempt ${record.index}: ${record.childRequest ? 'child' : 'router'}, ${record.inputBytes} request bytes`);
    const suffix = request.url.replace(/^\/v1/, '');
    const upstream = await fetch(`${base.replace(/\/$/, '')}${suffix}`, {
      method: request.method,
      headers: { authorization: `Bearer ${key}`, 'content-type': 'application/json' },
      body: body.length ? body : undefined,
      signal: controller.signal,
    });
    record.status = upstream.status;
    console.log(`provider attempt ${record.index}: HTTP ${record.status}`);
    response.writeHead(upstream.status, {
      'content-type': upstream.headers.get('content-type') ?? 'application/json',
    });
    let pending = '';
    for await (const chunk of upstream.body) {
      pending += Buffer.from(chunk).toString('utf8');
      const lines = pending.split('\n');
      pending = lines.pop();
      for (const line of lines) {
        if (!line.startsWith('data:') || line.includes('[DONE]')) continue;
        const event = JSON.parse(line.slice(5));
        if (event.choices?.some(choice => choice.delta?.content)) record.textChunks++;
      }
      response.write(chunk);
    }
    record.upstreamCompleted = true;
    response.end();
  } catch (error) {
    record.readAborted = controller.signal.aborted && error.name === 'AbortError';
    if (!controller.signal.aborted) {
      record.error = error.name;
      response.destroy();
    }
  } finally {
    record.closed = true;
  }
});
observer.listen(0, '127.0.0.1');
await once(observer, 'listening');

const encode = value => Buffer.from(JSON.stringify(value)).toString('base64url');
const unsigned = `${encode({ alg: 'HS256', typ: 'JWT' })}.${encode({
  sub: 'phase-live-smoke', roles: ['service'], uar_instance_id: 'phase-live-smoke',
  exp: Math.floor(Date.now() / 1000) + 900,
})}`;
const jwt = `${unsigned}.${createHmac('sha256', secret).update(unsigned).digest('base64url')}`;
const auth = { authorization: `Bearer ${jwt}`, 'content-type': 'application/json' };
async function api(path, body) {
  const result = await fetch(`${address}${path}`, {
    method: body === undefined ? 'GET' : 'POST', headers: auth,
    body: body === undefined ? undefined : JSON.stringify(body),
    signal: AbortSignal.timeout(60000),
  });
  assert(result.ok, `${path}: HTTP ${result.status}`);
  return result.json();
}

try {
  const config = {
    security: { jwt_required: true, jwt_secret: secret, settings_admin_key: secret },
    resilience: { rate_limit_enabled: false, stream_start_timeout_ms: 90000, retry_max_attempts: 1 },
    persistence: { provider: 'surreal', database_url: `surrealkv://${join(scratch, 'db')}` },
    llm: { model: `openai/${model}`, base_url: `http://127.0.0.1:${observer.address().port}/v1`, api_key: 'smoke-forwarder', protocol: 'chat', timeout_secs: 120, max_retries: 0 },
    server: { host: '127.0.0.1', grpc_port: 0, shutdown_timeout_secs: 30 },
    harness: { mode: 'shadow' },
    memory: { enabled: false },
    native_tools: { file_tools_enabled: false, terminal_exec_enabled: false, web_fetch_enabled: false },
  };
  await writeFile(join(scratch, 'config.json'), JSON.stringify(config), { mode: 0o600 });
  await writeFile(join(scratch, 'mcp.json'), '{"mcpServers":{}}');
  await mkdir(join(scratch, 'policies'));
  for (const name of ['default.cedar', 'tool-approval.cedar', 'skill-mutation.cedar']) {
    const policy = await readFile(new URL(`../../../../policies/${name}`, import.meta.url));
    await writeFile(join(scratch, 'policies', name), policy);
  }
  const childEnv = Object.fromEntries(Object.entries(process.env).filter(([name]) =>
    !/^(UAR_|LLM_|JWT_|CONFIG_FILE$|PORT$|OPENAI_|ANTHROPIC_|LITER_LLM_)/.test(name)));
  const launchEnv = { ...childEnv, RUST_LOG: process.env.UAR_SMOKE_LOG ?? 'warn', UAR_SECURITY__JWT_REQUIRED: 'true' };
  if (integrationHelper) Object.assign(launchEnv, {
    UAR_TEST_SERVER_CHILD: '1',
    UAR_TEST_SERVER_LLM_BASE_URL: config.llm.base_url,
    UAR_TEST_SERVER_LLM_MODEL: config.llm.model,
    UAR_TEST_SERVER_MEMORY: '0',
    UAR_TEST_SERVER_PERSISTENCE_PATH: join(scratch, 'db'),
    UAR_TEST_SERVER_CONTROL_DIR: scratch,
    UAR_SECURITY__JWT_SECRET: secret,
    UAR_SECURITY__SETTINGS_ADMIN_KEY: secret,
    UAR_LLM__API_KEY: config.llm.api_key,
    UAR_LLM__PROTOCOL: config.llm.protocol,
    UAR_LLM__TIMEOUT_SECS: '120',
    UAR_LLM__MAX_RETRIES: '0',
    UAR_RESILIENCE__STREAM_START_TIMEOUT_MS: '90000',
    UAR_RESILIENCE__RETRY_MAX_ATTEMPTS: '1',
    UAR_HARNESS__MODE: 'shadow',
    UAR_MEMORY__ENABLED: 'false',
    UAR_NATIVE_TOOLS__FILE_TOOLS_ENABLED: 'false',
    UAR_NATIVE_TOOLS__TERMINAL_EXEC_ENABLED: 'false',
    UAR_NATIVE_TOOLS__WEB_FETCH_ENABLED: 'false',
  });
  const args = integrationHelper
    ? ['--exact', 'live::harness::tests::process_server_helper', '--nocapture', '--test-threads=1']
    : ['--config', join(scratch, 'config.json'), '--jwt-required', 'true'];
  server = spawn(binary, args, {
    cwd: scratch, env: launchEnv, stdio: ['pipe', 'pipe', 'pipe'],
  });
  serverExit = once(server, 'exit').catch(error => { startupError = error; });
  server.stdout.on('data', chunk => { serverLog += chunk; });
  server.stderr.on('data', chunk => { serverLog += chunk; });
  const readyDeadline = Date.now() + 120000;
  while (!address) {
    if (integrationHelper) address = await readFile(join(scratch, 'ready'), 'utf8').catch(error => {
      if (error.code !== 'ENOENT') throw error;
    });
    const match = serverLog.match(/READY:(\d+)/);
    if (match) address = `http://127.0.0.1:${match[1]}`;
    if (!address) {
      if (startupError) throw startupError;
      assert(server.exitCode === null && server.signalCode === null, 'sidecar exited before readiness');
      assert(Date.now() < readyDeadline, 'sidecar readiness timeout');
      await delay(100);
    }
  }
  console.log(`${integrationHelper ? 'integration server helper' : 'sidecar'} ready; isolated database; harness=shadow`);
  const catalog = await api('/api/uar/discovery/agents');
  const artifact = catalog.runtime_agents.find(agent => agent.id === 'orchestrator-agent');
  assert(artifact, 'orchestrator artifact is registered');
  artifact.policy.tools.allow = ['spawn_agent', 'wait_agents', 'list_agents'];
  artifact.policy.skills.max_active = 0;
  artifact.memory.kb.enabled = false;
  artifact.extensions['uar.run_policy'] = {
    version: 1,
    skills: { mode: 'none' },
    mcp_servers: { mode: 'none' },
    knowledge_bases: { mode: 'none' },
    memory_enabled: false,
  };
  const created = await api('/api/uar/runs', {
    artifact,
    input: 'Produce a numbered list of 100 detailed examples of elementary arithmetic with explanations. Answer directly without tools or further delegation; begin the long answer immediately. The orchestrator is authorized to delegate this task once to its general-purpose specialist for this cancellation smoke.',
  });
  runId = created.run_id;
  const streamAbort = new AbortController();
  const openDeadline = setTimeout(() => streamAbort.abort(), 60000);
  let stream;
  try {
    stream = await fetch(`${address}${created.stream_url}`, { headers: auth, signal: streamAbort.signal });
  } finally {
    clearTimeout(openDeadline);
  }
  assert(stream.ok, 'root event stream opens');
  const approvals = new Set();
  const events = (async () => {
    let pending = '';
    for await (const bytes of stream.body) {
      pending += Buffer.from(bytes).toString('utf8').replaceAll('\r\n', '\n');
      let boundary;
      while ((boundary = pending.indexOf('\n\n')) >= 0) {
        const block = pending.slice(0, boundary);
        pending = pending.slice(boundary + 2);
        const name = block.split('\n').find(line => line.startsWith('event:'))?.slice(6).trim();
        const data = block.split('\n').filter(line => line.startsWith('data:')).map(line => line.slice(5).trim()).join('\n');
        if (!data) continue;
        const value = JSON.parse(data);
        if (name?.startsWith('agui.subagent.')) lifecycles.push(value.lifecycle);
        if (name === 'agui.cancelled') rootCancelled = true;
        if (value.approval_id && !approvals.has(value.approval_id)) {
          assert((value.tool ?? value.name) === 'spawn_agent', 'only spawn approval is authorized by this smoke');
          approvals.add(value.approval_id);
          await api(`/api/uar/runs/${runId}/tool-approval`, { approved: true, approval_id: value.approval_id });
        }
        if (name === 'agui.artifact' && value.artifact_type === 'provider_event' && value.title === 'resolved_step') {
          const content = JSON.parse(value.content);
          const shadow = content.payload?.manifest?.shadow;
          if (shadow) shadows.push(shadow);
        }
        if (name === 'agui.error') throw new Error(`run error: ${value.code}: ${value.message}`);
      }
    }
  })();
  // Surface a rejected stream immediately, including while waiting for inference.
  let eventFailure;
  events.catch(error => { eventFailure = error; });
  const inferenceDeadline = Date.now() + 180000;
  try {
    let childCall;
    let child;
    while (!childCall) {
      if (eventFailure) throw eventFailure;
      assert(Date.now() < inferenceDeadline, `child did not reach cancellation point: ${cancellationPoint}`);
      const latest = new Map(lifecycles.map(item => [item.child_thread_id, item]));
      assert(latest.size <= 1, 'this smoke requires exactly one delegated child');
      child = [...latest.values()].find(item => item.status === 'running'
        && item.artifact_id === 'general-purpose' && item.parent_run_id === runId);
      childCall = child && calls.find(call => call.childRequest && !call.closed && !call.upstreamCompleted
        && (beforeFirstResponse ? call.status === null && call.textChunks === 0
          : call.status === 200 && call.textChunks > 0));
      if (childCall) break;
      await delay(100);
    }
    const firstChildIndex = calls.findIndex(call => call.childRequest);
    assert(firstChildIndex > 0 && calls.slice(firstChildIndex).every(call => call.childRequest),
      'router attempts precede the uniquely identified child attempts');
    assert(calls.slice(0, firstChildIndex).some(call => call.status === 200 && call.textChunks > 0),
      'router emitted real provider text');
    assert(calls.filter(call => call !== childCall).every(call => call.closed),
      'the selected child is the only active provider attempt');
    childCall.child_thread_id = child.child_thread_id;
    childCall.child_run_id = child.child_run_id;
    const atCancellation = { status: childCall.status, textChunks: childCall.textChunks, closed: childCall.closed };
    console.log(`real router response and child request observed; cancelling root at ${cancellationPoint}`);
    const attemptsBeforeCancellation = calls.length;
    cancellation = await api(`/api/uar/runs/${runId}/cancel`, {});
    assert.equal(cancellation.cancelled, true);
    await Promise.race([events, delay(30000).then(() => { throw new Error('cancellation event timeout'); })]);
    assert(rootCancelled, 'root cancellation is observable');
    assert(lifecycles.some(item => item.child_thread_id === child.child_thread_id
      && item.child_run_id === child.child_run_id && item.status === 'cancelled'
      && item.terminal_outcome === 'cancelled'), 'the selected child cancellation is observable');
    const closeDeadline = Date.now() + 10000;
    while (!childCall.closed && Date.now() < closeDeadline) await delay(50);
    assert(childCall.closed && childCall.downstreamAborted && childCall.readAborted
      && childCall.error === undefined && !childCall.upstreamCompleted,
      'root cancellation aborted the active child provider stream before teardown');
    assert(calls.every(call => call.closed), 'no provider attempt remains active after cancellation');
    assert.equal(calls.length, attemptsBeforeCancellation, 'cancellation does not start another provider attempt');
    assert.equal(new Set(lifecycles.map(item => item.child_thread_id)).size, 1);
    const report = { model, launcher: integrationHelper ? 'integration-server-helper' : 'sidecar', harness_mode: 'shadow', cancellation_point: cancellationPoint, at_cancellation: atCancellation, run_id: runId, calls, lifecycles, shadows, root_cancelled: rootCancelled, result: 'passed' };
    console.log(JSON.stringify(report, null, 2));
  } finally {
    streamAbort.abort();
    await events.catch(() => {});
  }
} catch (error) {
  console.error(JSON.stringify({ result: 'failed', error: error.message, calls, lifecycles, scratch }));
  process.exitCode = 1;
} finally {
  try {
    if (runId && !cancellation) await api(`/api/uar/runs/${runId}/cancel`, {}).catch(() => {});
    if (server && !startupError && server.exitCode === null && server.signalCode === null) {
      if (integrationHelper) {
        const controls = await Promise.allSettled([
          writeFile(join(scratch, 'shutdown'), 'shutdown'),
          writeFile(join(scratch, 'allow-exit'), 'exit'),
        ]);
        if (controls.some(result => result.status === 'rejected')) process.exitCode = 1;
      }
      server.kill('SIGTERM');
      const forced = setTimeout(() => server.kill('SIGKILL'), 35000);
      try { await serverExit; } finally { clearTimeout(forced); }
    }
  } finally {
    observer.closeAllConnections();
    await new Promise(resolveClose => observer.close(resolveClose));
    const redactedLog = [secret, key, base].reduce((log, value) => log.replaceAll(value, '[REDACTED]'), serverLog);
    await writeFile(join(scratch, 'server.log'), redactedLog, { mode: 0o600 });
    console.log(`isolated smoke data retained at ${scratch}`);
  }
}
