// Catalog-mode skill fixture for the owned runtime: a scoped Cedar permit for
// activate_skill, catalog presentation, approval answering and run collection.
import assert from 'node:assert/strict';
import { readFile, rm, writeFile } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { assertOwnedEndpoint, restartOwnedUar } from './restart-owned-uar.mjs';

const SETTING = 'harness.skill_activation_mode';
const TERMINAL_EVENTS = new Set(['agui.done', 'agui.error', 'agui.cancelled']);

export async function http(w, method, path, payload, expected = [200]) {
  const exchange = { method, path, request: payload };
  w.evidence.exchanges.push(exchange);
  const response = await fetch(new URL(path, w.base), {
    method, redirect: 'error', signal: AbortSignal.timeout(200_000),
    headers: { 'Content-Type': 'application/json', 'x-uar-workspace-id': w.workspace,
      ...(path.includes('/stream') && { Accept: 'text/event-stream' }) },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  exchange.status = response.status;
  const text = await response.text();
  exchange.response = path.includes('/stream') ? `[${text.length} bytes of SSE]` : text;
  assert.ok(expected.includes(response.status), `${method} ${path}: HTTP ${response.status}: ${text.slice(0, 800)}`);
  return text;
}
export const json = async (...args) => { const text = await http(...args); return text ? JSON.parse(text) : null; };

function sseBlocks(raw) {
  return raw.split(/\r?\n\r?\n/).flatMap(block => {
    const lines = block.split(/\r?\n/);
    const event = lines.find(line => line.startsWith('event:'))?.slice(6).trim() ?? null;
    const data = lines.filter(line => line.startsWith('data:')).map(line => line.slice(5).trimStart()).join('\n');
    return data && data !== '[DONE]' ? [{ event, data: JSON.parse(data) }] : [];
  });
}

// Permit only this agent to call activate_skill, switch to catalog presentation, restart.
export async function enableCatalog(w, label) {
  w.evidence.endpoint_binding = await assertOwnedEndpoint(w.base);
  const launch = JSON.parse(await readFile(resolve('target/prompt-macos-launch.json'), 'utf8'));
  w.policyFile = join(dirname(launch.config), 'policies', `${label}-activate-skill-${w.id}.cedar`);
  await writeFile(w.policyFile, `permit (\n    principal == Agent::"${w.agentId}",\n    action == Action::"execute_tool",\n    resource == Tool::"activate_skill"\n);\n`, { flag: 'wx', mode: 0o600 });
  w.evidence.scenario_policy = { file: w.policyFile, principal: w.agentId, tool: 'activate_skill' };
  w.evidence.restart_permit = {};
  await restartOwnedUar(w.evidence.restart_permit);
  w.originalSetting = await json(w, 'GET', `/api/uar/settings/${SETTING}`);
  w.evidence.original_setting = { saved: w.originalSetting.saved, effective: w.originalSetting.effective };
  await http(w, 'PUT', `/api/uar/settings/${SETTING}`, { value: 'catalog', expected_revision: w.originalSetting.revision });
  w.changedSetting = true;
  assert.equal((await json(w, 'GET', `/api/uar/settings/${SETTING}`)).effective, 'catalog');
}

export async function restoreCatalog(w, attempt) {
  if (w.changedSetting) await attempt('setting', async () => {
    const current = await json(w, 'GET', `/api/uar/settings/${SETTING}`);
    await http(w, 'PUT', `/api/uar/settings/${SETTING}`, { value: w.originalSetting.saved, expected_revision: current.revision });
    w.evidence.restored_setting = (await json(w, 'GET', `/api/uar/settings/${SETTING}`)).effective;
    assert.equal(w.evidence.restored_setting, w.originalSetting.effective);
  });
  if (w.policyFile) await attempt('permit', async () => {
    await rm(w.policyFile, { force: true });
    w.evidence.restart_restore = {};
    await restartOwnedUar(w.evidence.restart_restore);
  });
}

// Answer this run's activate_skill approvals from the run's event snapshot; deny anything else.
async function answerApprovals(w, runId, answered) {
  const snapshot = await json(w, 'GET', `/api/uar/runs/${encodeURIComponent(runId)}/events`, undefined, [200, 404]);
  for (const event of snapshot?.events ?? []) {
    if (event.eventName !== 'agui.tool_call.approval_required' || answered.has(event.data.approval_id)) continue;
    answered.add(event.data.approval_id);
    const approved = event.data.name === 'activate_skill';
    await http(w, 'POST', `/api/uar/runs/${encodeURIComponent(runId)}/approval`, { approved, approval_id: event.data.approval_id });
    w.evidence.approvals.push({ approval_id: event.data.approval_id, tool: event.data.name, approved, risk_reason: event.data.risk_reason });
  }
}

// Stream one chat turn while polling its run for approvals; the stream's own event names are recorded.
export async function chatWithApprovals(w, body) {
  w.evidence.approvals = [];
  const exchange = { method: 'POST', path: '/v1/chat/completions', request: body };
  w.evidence.exchanges.push(exchange);
  const response = await fetch(new URL('/v1/chat/completions', w.base), {
    method: 'POST', redirect: 'error', signal: AbortSignal.timeout(200_000),
    headers: { 'Content-Type': 'application/json', Accept: 'text/event-stream', 'x-uar-workspace-id': w.workspace },
    body: JSON.stringify({ ...body, stream: true, stream_mode: 'agui' }),
  });
  exchange.status = response.status;
  if (response.status !== 200) assert.fail(`chat stream HTTP ${response.status}: ${(await response.text()).slice(0, 600)}`);
  let runId = null, terminal = null, ended = false;
  const names = new Set();
  const reader = (async () => {
    const decoder = new TextDecoder();
    let buffer = '';
    for await (const chunk of response.body) {
      buffer += decoder.decode(chunk, { stream: true });
      const cut = buffer.lastIndexOf('\n\n');
      if (cut < 0) continue;
      const ready = buffer.slice(0, cut); buffer = buffer.slice(cut + 2);
      for (const { event, data } of sseBlocks(ready)) {
        names.add(event);
        runId ??= data.request_id ?? null;
        if (TERMINAL_EVENTS.has(event)) terminal = event;
      }
    }
    ended = true;
  })();
  const answered = new Set();
  const poller = (async () => {
    while (!ended) {
      if (!runId) {
        const runs = await json(w, 'GET', '/api/uar/runs?limit=100');
        runId = runs.find(run => run.conversation_id === body.session_id)?.run_id ?? null;
      }
      if (runId) await answerApprovals(w, runId, answered);
      await new Promise(done => setTimeout(done, 1_000));
    }
  })();
  await Promise.all([reader, poller]);
  w.evidence.chat_stream_event_names = [...names];
  exchange.response = `[stream ended: ${terminal}]`;
  assert.ok(runId, 'The chat turn must name its run');
  assert.equal(terminal, 'agui.done', `The run must complete: ${terminal}`);
  assert.ok(w.evidence.approvals.every(a => a.approved), `Only activate_skill may request approval: ${JSON.stringify(w.evidence.approvals)}`);
  return runId;
}

// Replay a finished run's official stream; collect per-step manifests, tool calls, answer text.
export async function collectRun(w, runId) {
  const events = sseBlocks(await http(w, 'GET', `/api/uar/runs/${encodeURIComponent(runId)}/stream?stream_mode=agui_spec&last_event_id=0`))
    .map(frame => frame.data);
  const manifests = events.filter(e => e.type === 'CUSTOM' && e.value?.artifactId === 'uar.turn:resolved_step')
    .map(e => JSON.parse(e.value.content).payload)
    .map(p => ({ step: p.step, selected_skills: p.manifest.selected_skills, selected_tools: p.manifest.selected_tools,
      fragments: p.manifest.fragments.map(f => ({ id: f.id, section: f.section })) }));
  const calls = new Map();
  for (const e of events) {
    if (e.type === 'TOOL_CALL_START') calls.set(e.toolCallId, { name: e.toolCallName, args: '', result: null });
    if (e.type === 'TOOL_CALL_ARGS' && calls.has(e.toolCallId)) calls.get(e.toolCallId).args += e.delta ?? '';
    // The native agui_spec projection carries the complete arguments on TOOL_CALL_END.
    if (e.type === 'TOOL_CALL_END' && calls.has(e.toolCallId) && e.arguments !== undefined) calls.get(e.toolCallId).final = e.arguments;
    if (e.type === 'TOOL_CALL_RESULT' && calls.has(e.toolCallId)) calls.get(e.toolCallId).result = e.content;
  }
  const parse = value => { try { return typeof value === 'string' ? JSON.parse(value) : value; } catch { return value; } };
  const tools = [...calls.values()].map(c => ({ name: c.name, args: parse(c.final ?? c.args), result: parse(c.result) }));
  const text = events.filter(e => e.type === 'TEXT_MESSAGE_CONTENT').map(e => e.delta).join('');
  const run = await json(w, 'GET', `/api/uar/runs/${encodeURIComponent(runId)}`);
  const activated = (run.activated_skills ?? []).map(s => typeof s === 'string' ? s : s.skill_id);
  Object.assign(w.evidence, { run_id: runId, run_status: run.status, effective_skills: run.effective_run_policy?.skills,
    activated_skills: activated, manifests, tool_calls: tools, answer_text: text });
  return { run, activated, manifests, tools, text };
}

export const hasBody = (manifest, skillId) => manifest.fragments.some(f => f.id === `skill.${skillId}`);
