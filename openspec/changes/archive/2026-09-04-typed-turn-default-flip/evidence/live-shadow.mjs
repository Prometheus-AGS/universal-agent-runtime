import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { createHmac, randomBytes } from 'node:crypto';
import { once } from 'node:events';
import { mkdir, mkdtemp, readFile, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';

// Opt-in phase-end evidence. No fixtures or synthetic provider responses.
const binary = resolve(process.argv[2] ?? 'target/debug/uar-sidecar');
const base = process.env.LITER_LLM_BASE_URL;
const key = process.env.LITER_LLM_MASTER_KEY;
assert(base && key, 'LITER_LLM_BASE_URL and LITER_LLM_MASTER_KEY are required');
const model = process.env.UAR_SMOKE_MODEL ?? 'gpt-5.4';
const scratch = await mkdtemp(join(tmpdir(), 'uar-live-shadow-'));
const secret = randomBytes(32).toString('hex');
const encode = value => Buffer.from(JSON.stringify(value)).toString('base64url');
const unsigned = `${encode({ alg: 'HS256', typ: 'JWT' })}.${encode({
  sub: 'phase-live-shadow', roles: ['service'], uar_instance_id: 'phase-live-shadow',
  exp: Math.floor(Date.now() / 1000) + 900,
})}`;
const jwt = `${unsigned}.${createHmac('sha256', secret).update(unsigned).digest('base64url')}`;
const auth = { authorization: `Bearer ${jwt}`, 'content-type': 'application/json' };
const cases = [
  { id: 'basic-user-turn', input: 'Reply with only the word ready.', instructions: [] },
  { id: 'host-instructions', input: 'What is two plus three?', instructions: ['Answer arithmetic questions with only the resulting numeral.'] },
];
const receipts = [];
let server;
let serverExit;
let startupError;
let serverLog = '';
let address;
let activeRun;
async function api(path, body) {
  const response = await fetch(`${address}${path}`, {
    method: body === undefined ? 'GET' : 'POST', headers: auth,
    body: body === undefined ? undefined : JSON.stringify(body),
    signal: AbortSignal.timeout(60000),
  });
  assert(response.ok, `${path}: HTTP ${response.status}`);
  return response.json();
}

try {
  const config = {
    security: { jwt_required: true, jwt_secret: secret, settings_admin_key: secret },
    resilience: { rate_limit_enabled: false, stream_start_timeout_ms: 90000, retry_max_attempts: 1 },
    persistence: { provider: 'surreal', database_url: `surrealkv://${join(scratch, 'db')}` },
    llm: { model: `openai/${model}`, base_url: base, api_key: key, protocol: 'chat', timeout_secs: 120, max_retries: 0 },
    server: { host: '127.0.0.1', grpc_port: 0, shutdown_timeout_secs: 30 },
    harness: { mode: 'shadow' }, memory: { enabled: false },
    native_tools: { file_tools_enabled: false, terminal_exec_enabled: false, web_fetch_enabled: false },
  };
  await writeFile(join(scratch, 'config.json'), JSON.stringify(config), { mode: 0o600 });
  await writeFile(join(scratch, 'mcp.json'), '{"mcpServers":{}}');
  await mkdir(join(scratch, 'policies'));
  for (const name of ['default.cedar', 'tool-approval.cedar', 'skill-mutation.cedar']) {
    await writeFile(join(scratch, 'policies', name),
      await readFile(new URL(`../../../../policies/${name}`, import.meta.url)));
  }
  const childEnv = Object.fromEntries(Object.entries(process.env).filter(([name]) =>
    !/^(UAR_|LLM_|JWT_|CONFIG_FILE$|PORT$|OPENAI_|ANTHROPIC_|LITER_LLM_)/.test(name)));
  server = spawn(binary, ['--config', join(scratch, 'config.json'), '--jwt-required', 'true'], {
    cwd: scratch, env: { ...childEnv, RUST_LOG: process.env.UAR_SMOKE_LOG ?? 'warn' },
    stdio: ['pipe', 'pipe', 'pipe'],
  });
  serverExit = once(server, 'exit').catch(error => { startupError = error; });
  server.stdout.on('data', bytes => { serverLog += bytes; });
  server.stderr.on('data', bytes => { serverLog += bytes; });
  const readyDeadline = Date.now() + 120000;
  while (!address) {
    if (startupError) throw startupError;
    assert(server.exitCode === null && server.signalCode === null, 'sidecar exited before readiness');
    const match = serverLog.match(/READY:(\d+)/);
    if (match) address = `http://127.0.0.1:${match[1]}`;
    else {
      assert(Date.now() < readyDeadline, 'sidecar readiness timeout');
      await delay(100);
    }
  }
  console.log('sidecar ready; isolated database; harness=shadow');
  const catalog = await api('/api/uar/discovery/agents');
  const template = catalog.runtime_agents.find(agent => agent.id === 'default-agent');
  assert(template, 'default-agent is registered');
  for (const scenario of cases) {
    const artifact = structuredClone(template);
    artifact.prompt.instructions = scenario.instructions;
    artifact.policy.tools.allow = [];
    artifact.policy.skills.max_active = 0;
    artifact.memory.kb.enabled = false;
    artifact.extensions['uar.run_policy'] = {
      version: 1, tools: { mode: 'none' }, skills: { mode: 'none' },
      mcp_servers: { mode: 'none' }, knowledge_bases: { mode: 'none' }, memory_enabled: false,
    };
    const created = await api('/api/uar/runs', { artifact, input: scenario.input });
    activeRun = created.run_id;
    const receipt = { id: scenario.id, run_id: activeRun, shadows: [], completed: false, text_events: 0, event_names: [] };
    receipts.push(receipt);
    console.log(`running ${scenario.id}`);
    const stream = await fetch(`${address}${created.stream_url}`, {
      headers: auth, signal: AbortSignal.timeout(180000),
    });
    assert(stream.ok, 'event stream opens');
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
        if (!receipt.event_names.includes(name)) receipt.event_names.push(name);
        if (name === 'agui.artifact' && value.artifact_type === 'provider_event' && value.title === 'resolved_step') {
          const shadow = JSON.parse(value.content).payload?.manifest?.shadow;
          if (shadow) receipt.shadows.push(shadow);
        }
        if (name === 'agui.message.delta' && typeof value.delta?.text === 'string' && value.delta.text.length > 0) receipt.text_events++;
        if (name === 'agui.done') receipt.completed = true;
        assert(name !== 'agui.cancelled', 'cancelled runs are not parity evidence');
        assert(!value.approval_id, 'this smoke does not authorize tool execution');
        assert(name !== 'agui.error', `run failed: ${value.code}: ${value.message}`);
      }
    }
    assert(receipt.completed, 'real-provider run completed');
    assert(receipt.text_events > 0, 'real-provider text was observed');
    assert(receipt.shadows.length > 0, 'each live case requires nonempty shadow evidence');
    for (const shadow of receipt.shadows) {
      assert.equal(shadow.unexpected_difference_count, 0);
      assert.equal(shadow.dispatched_path, 'legacy');
    }
    activeRun = undefined;
    console.log(`${scenario.id}: completed; ${receipt.shadows.length} comparisons; zero unexpected differences`);
  }
  console.log(JSON.stringify({ result: 'passed', model, harness_mode: 'shadow', smoke_set_size: cases.length, cases: receipts }, null, 2));
} catch (error) {
  console.error(JSON.stringify({ result: 'failed', error: error.message, cases: receipts, scratch }));
  process.exitCode = 1;
} finally {
  if (activeRun) await api(`/api/uar/runs/${activeRun}/cancel`, {}).catch(() => {});
  if (server && !startupError && server.exitCode === null && server.signalCode === null) {
    server.kill('SIGTERM');
    const forced = setTimeout(() => server.kill('SIGKILL'), 35000);
    try { await serverExit; } finally { clearTimeout(forced); }
  }
  // Do not leave the provider credential in the retained scratch configuration.
  await writeFile(join(scratch, 'config.json'), '{"redacted":true}', { mode: 0o600 });
  const redacted = [secret, key, base].reduce((log, value) => log.replaceAll(value, '[REDACTED]'), serverLog);
  await writeFile(join(scratch, 'server.log'), redacted, { mode: 0o600 });
  console.log(`isolated smoke data retained at ${scratch}`);
}
