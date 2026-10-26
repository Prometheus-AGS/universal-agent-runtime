import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import os from 'node:os';
import { After, Given, When, Then, setDefaultTimeout } from '@cucumber/cucumber';

setDefaultTimeout(125_000);

Given('UAR is running with a configured real model', function () {
  this.base = new URL(process.env.UAR_BASE_URL ?? 'http://127.0.0.1:3000');
  assert.ok(['http:', 'https:'].includes(this.base.protocol));
  assert.ok(!this.base.username && !this.base.password, 'UAR_BASE_URL must not contain URL credentials');
  assert.ok(process.env.UAR_MODEL, 'Set UAR_MODEL to the configured provider/model');
  this.events = [];
  this.answer = '';
  this.done = false;
});

When('I ask the default agent to count from 1 to 20 using streaming', async function () {
  const started = performance.now();
  const headers = { 'Content-Type': 'application/json', Accept: 'text/event-stream' };
  const response = await fetch(new URL('/v1/chat/completions', this.base), {
    method: 'POST', headers, redirect: 'error', signal: AbortSignal.timeout(120_000),
    body: JSON.stringify({
      agent_id: 'default-agent', model: process.env.UAR_MODEL, session_id: randomUUID(),
      stream: true, stream_mode: 'openai', memory_enabled: false, max_completion_tokens: 512,
      messages: [{ role: 'user', content: 'Count from 1 to 20, separated by single spaces. Reply with only those numbers.' }],
    }),
  });
  assert.equal(response.status, 200, `HTTP ${response.status}: ${response.ok ? '' : (await response.text()).slice(0, 1000)}`);
  assert.match(response.headers.get('content-type') ?? '', /^text\/event-stream\b/);
  const decoder = new TextDecoder();
  let buffer = '';
  const frame = (block) => {
    const eventName = block.split(/\r?\n/).find(line => line.startsWith('event:'))?.slice(6).trim();
    const data = block.split(/\r?\n/).filter(line => line.startsWith('data:'))
      .map(line => line.slice(5).replace(/^ /, '')).join('\n');
    if (!data) return;
    assert.equal(this.done, false, 'Received data after the terminal marker');
    if (data === '[DONE]') {
      this.done = true;
      this.events.push({ type: 'done', elapsed_ms: Math.round(performance.now() - started) });
      return;
    }
    const chunk = JSON.parse(data);
    assert.ok(!chunk.error, JSON.stringify(chunk.error));
    if (chunk.object !== 'chat.completion.chunk') {
      assert.match(eventName ?? '', /^runtime\./, 'Unexpected non-chat stream event');
      this.events.push({ type: 'runtime', name: eventName });
      return;
    }
    assert.equal(chunk.model, process.env.UAR_MODEL);
    for (const choice of chunk.choices ?? []) {
      const content = choice.delta?.content;
      if (typeof content === 'string' && content.length > 0) {
        assert.equal(this.finishReason, undefined, 'Received content after completion');
        this.answer += content;
        this.events.push({ type: 'content', text: content, elapsed_ms: Math.round(performance.now() - started) });
      }
      if (choice.finish_reason) {
        assert.equal(this.finishReason, undefined, 'Duplicate completion event');
        this.finishReason = choice.finish_reason;
        this.events.push({ type: 'finish', reason: choice.finish_reason });
      }
    }
  };
  const drain = () => {
    let separator;
    while ((separator = /\r?\n\r?\n/.exec(buffer))) {
      const block = buffer.slice(0, separator.index);
      buffer = buffer.slice(separator.index + separator[0].length);
      frame(block);
    }
  };
  for await (const bytes of response.body) {
    buffer += decoder.decode(bytes, { stream: true });
    drain();
  }
  buffer += decoder.decode();
  drain();
  assert.equal(buffer.trim(), '', 'Stream ended with an incomplete event');
  this.durationMs = Math.round(performance.now() - started);
});

Then('I receive multiple content events before the stream finishes', function () {
  const content = this.events.filter(event => event.type === 'content');
  assert.ok(content.length >= 2, `Expected multiple content events, received ${content.length}`);
  const finish = this.events.findIndex(event => event.type === 'finish');
  assert.ok(finish > this.events.findLastIndex(event => event.type === 'content'));
});

Then('the assembled answer contains the numbers 1 through 20 in order', function () {
  assert.equal(this.answer.trim().split(/\s+/).join(' '),
    Array.from({ length: 20 }, (_, index) => index + 1).join(' '));
});

Then('the stream ends successfully', function () {
  assert.equal(this.finishReason, 'stop');
  assert.equal(this.done, true, 'Missing [DONE] marker');
});

After(function ({ result }) {
  this.attach(JSON.stringify({
    behavior: 'Default agent streams a complete useful answer',
    status: result.status, client_platform: process.platform, os_release: os.release(),
    endpoint: this.base?.origin, model: process.env.UAR_MODEL,
    answer: this.answer, events: this.events, duration_ms: this.durationMs,
  }, null, 2), 'application/json');
});
