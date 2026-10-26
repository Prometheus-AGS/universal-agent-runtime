import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';

setDefaultTimeout(125_000);

async function request(world, method, path, payload) {
  const headers = { 'Content-Type': 'application/json' };
  if (process.env.UAR_WORKSPACE_ID) headers['x-uar-workspace-id'] = process.env.UAR_WORKSPACE_ID;
  return fetch(new URL(path, world.base), {
    method, headers, redirect: 'error', signal: AbortSignal.timeout(120_000),
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
}

async function json(response, status = 200) {
  const text = await response.text();
  assert.equal(response.status, status, `HTTP ${response.status}: ${text.slice(0, 1000)}`);
  return JSON.parse(text);
}

Given('the running UAR contains the saved mathematics specialist', async function () {
  this.startedAt = performance.now();
  this.base = new URL(process.env.UAR_BASE_URL ?? 'http://127.0.0.1:3000');
  assert.ok(['http:', 'https:'].includes(this.base.protocol));
  assert.ok(!this.base.username && !this.base.password, 'UAR_BASE_URL must not contain URL credentials');
  assert.ok(process.env.UAR_MODEL, 'Set UAR_MODEL to the configured real provider/model');
  const record = JSON.parse(await readFile(process.env.UAR_MATH_AGENT_RECORD ??
    '.prometheus/verification/agents/math-specialist-db64c2a8.json', 'utf8'));
  this.mathId = record.agent_id;
  const math = await json(await request(this, 'GET', `/api/agents/${encodeURIComponent(this.mathId)}`));
  assert.equal(math.id, this.mathId);
  this.template = await json(await request(this, 'GET', '/api/agents/default-agent'));
  this.events = [];
});

When('I register a persistent blog and article writing agent', async function () {
  this.writerId = 'writing-specialist';
  this.record = resolve(process.env.UAR_WRITING_AGENT_RECORD ?? '.prometheus/verification/agents/writing-specialist.json');
  const existing = await request(this, 'GET', `/api/agents/${this.writerId}`);
  if (existing.status === 200) {
    const saved = JSON.parse(await readFile(this.record, 'utf8'));
    const registered = await existing.json();
    assert.equal(saved.agent_id, this.writerId);
    assert.equal(registered.prompt.system, saved.agent.prompt.system, 'Existing writer differs from the retained definition');
    this.writer = registered;
    this.byline = saved.byline;
    this.created = false;
    return;
  }
  assert.equal(existing.status, 404, 'Unexpected writer lookup result');
  await existing.body?.cancel();
  this.byline = `By Wren Editor (writer-${randomUUID().slice(0, 8)})`;
  const artifact = structuredClone(this.template);
  Object.assign(artifact, { id: this.writerId, version: '1.0.0', extensions: {} });
  artifact.metadata = { ...artifact.metadata, title: 'Blog and Article Writer',
    description: 'Professional writing specialist for blog posts, articles, editorial drafts, storytelling, copywriting and clear engaging prose for a requested audience.',
    tags: ['writing', 'blog', 'articles', 'editorial', 'copywriting'] };
  artifact.prompt = { system: 'You are a professional blog and article writer. Create publishable drafts with clear headlines, coherent paragraphs and engaging prose. Follow the requested topic, audience, tone and approximate length. '
    + `End every draft with your exact byline: ${this.byline}. Provide the final draft directly, without planning commentary.`, instructions: [] };
  artifact.runtime.entry = 'default';
  const model = process.env.UAR_MODEL;
  const split = model.indexOf('/');
  assert.ok(split > 0, 'UAR_MODEL must be provider/model');
  artifact.policy.provider = { default: { provider: model.slice(0, split), model: model.slice(split + 1) }, fallbacks: [] };
  artifact.policy.tools.allow = [];
  artifact.memory.kb.enabled = false;
  this.writer = await json(await request(this, 'POST', '/api/agents', artifact), 201);
  assert.equal(this.writer.id, this.writerId);
  this.created = true;
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify({ agent_id: this.writerId, agent: this.writer,
    byline: this.byline, base_url: this.base.origin, model, retained_for_future_use: true }, null, 2) + '\n',
    { flag: 'wx', mode: 0o600 });
});

Then('the writing agent is saved alongside the mathematics specialist', async function () {
  const saved = await json(await request(this, 'GET', `/api/agents/${this.writerId}`));
  assert.equal(saved.prompt.system, this.writer.prompt.system);
  const roster = await json(await request(this, 'GET', '/api/agents'));
  const ids = roster.runtime_agents.map(agent => agent.id);
  assert.ok(ids.includes(this.writerId) && ids.includes(this.mathId));
});

async function readEvents(world, response) {
  assert.equal(response.status, 200, `HTTP ${response.status}: ${response.ok ? '' : (await response.text()).slice(0, 1000)}`);
  assert.match(response.headers.get('content-type') ?? '', /^text\/event-stream\b/);
  let buffer = '';
  const decoder = new TextDecoder();
  const drain = () => {
    let separator;
    while ((separator = /\r?\n\r?\n/.exec(buffer))) {
      const block = buffer.slice(0, separator.index);
      buffer = buffer.slice(separator.index + separator[0].length);
      const data = block.split(/\r?\n/).filter(line => line.startsWith('data:'))
        .map(line => line.slice(5).replace(/^ /, '')).join('\n');
      if (!data) continue;
      const event = JSON.parse(data);
      world.events.push(event);
      assert.ok(!['RUN_ERROR', 'RUN_CANCELLED'].includes(event.type), JSON.stringify(event));
    }
  };
  for await (const bytes of response.body) { buffer += decoder.decode(bytes, { stream: true }); drain(); }
  buffer += decoder.decode();
  drain();
  assert.equal(buffer.trim(), '', 'Stream ended with an incomplete event');
}

When('I ask the orchestrator to write a community garden blog post', async function () {
  this.threadId = randomUUID();
  this.runId = randomUUID();
  this.task = 'Write a blog post of approximately 100 to 160 words about how a community garden helps neighbors connect. Include a headline and at least two substantive paragraphs.';
  const path = '/api/uar/ag-ui/agents/orchestrator-agent';
  await readEvents(this, await request(this, 'POST', path, {
    threadId: this.threadId, runId: this.runId, protocolVersion: '1.0',
    messages: [{ id: randomUUID(), role: 'user', content: this.task }], tools: [], context: [],
    forwardedProps: { uar: { model: process.env.UAR_MODEL } },
  }));
  const terminal = this.events.find(event => event.type === 'RUN_FINISHED');
  if (terminal?.outcome?.type === 'interrupt') {
    const interrupts = terminal.outcome.interrupts;
    assert.equal(interrupts.length, 1, 'Expected one local delegation approval');
    const interrupt = interrupts[0];
    assert.equal(interrupt.reason, 'tool_approval');
    assert.ok(this.events.some(event => event.type === 'TOOL_CALL_START'
      && event.toolCallId === interrupt.toolCallId && event.toolCallName === 'spawn_agent'),
    'Only this requested local agent-spawn approval may be answered');
    this.approvalEvents = this.events;
    this.approval = { interrupt_id: interrupt.id, tool: 'spawn_agent', approved: true,
      parent_run_id: this.runId };
    const parentRunId = this.runId;
    this.runId = randomUUID();
    this.events = [];
    await readEvents(this, await request(this, 'POST', path, {
      threadId: this.threadId, runId: this.runId, parentRunId, protocolVersion: '1.0',
      messages: [], tools: [], context: [], forwardedProps: { uar: {} },
      resume: [{ interruptId: interrupt.id, status: 'resolved', payload: { approved: true } }],
    }));
  }
});

Then('intent classification selects the writing agent instead of the mathematics specialist', function () {
  const selections = this.events.filter(event => event.type === 'CUSTOM' && event.name === 'uar.agent.selected');
  assert.equal(selections.length, 1, 'Expected one agent-selection event');
  const event = selections[0];
  this.selection = event.value;
  assert.equal(event.threadId, this.threadId);
  assert.equal(event.runId, this.runId);
  assert.equal(event.agentId, 'orchestrator-agent');
  assert.equal(this.selection.sourceAgentId, 'orchestrator-agent');
  assert.equal(this.selection.selectionMethod, 'intent');
  assert.equal(this.selection.agent.id, this.writerId);
  assert.notEqual(this.selection.agent.id, this.mathId);
  assert.ok(this.selection.selectedThreadId);
  const terminals = this.events.filter(item => item.type === 'RUN_FINISHED');
  assert.equal(terminals.length, 1, 'Expected one terminal event after approval');
  assert.equal(terminals[0].outcome?.type, 'success', 'The resumed run must complete, not interrupt again');
  const selectedAt = this.events.indexOf(event);
  const finishedAt = this.events.findIndex(item => item.type === 'RUN_FINISHED');
  const contentAt = this.events.findIndex(item => item.type === 'TEXT_MESSAGE_CONTENT');
  assert.ok(selectedAt < contentAt && contentAt < finishedAt, 'Selection must precede the answer and successful completion');
});

Then('the writing agent returns a blog draft using its saved byline', function () {
  const text = this.events.filter(event => event.type === 'TEXT_MESSAGE_CONTENT').map(event => event.delta ?? '').join('');
  this.draft = text.replace(/<think>[\s\S]*?<\/think>/g, '').trim();
  assert.ok(this.draft.includes(this.byline), 'Missing the byline known only to the saved writing agent');
  assert.match(this.draft, /garden/i);
  assert.match(this.draft, /community|neighbou?r/i);
  const wordCount = this.draft.split(/\s+/).length;
  assert.ok(wordCount >= 80 && wordCount <= 250, `Expected a short blog draft; received ${wordCount} words`);
  const substantive = this.draft.split(/\r?\n\s*\r?\n/).filter(paragraph => paragraph.split(/\s+/).length >= 15);
  assert.ok(substantive.length >= 2, 'Expected at least two substantive paragraphs');
  this.wordCount = wordCount;
});

After(async function ({ result }) {
  this.attach(JSON.stringify({ behavior: 'Persist a writing agent and route a writing task by intent',
    status: result.status, client_platform: process.platform, model: process.env.UAR_MODEL,
    writer_id: this.writerId, math_id: this.mathId, agent_record: this.record, writer_created: this.created,
    task: this.task, approval: this.approval, approval_events: this.approvalEvents, selection: this.selection, events: this.events, draft: this.draft,
    word_count: this.wordCount, service_kept_running: true, agents_retained: true,
    duration_ms: Math.round(performance.now() - this.startedAt) }, null, 2), 'application/json');
});
