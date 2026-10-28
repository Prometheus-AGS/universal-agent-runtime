/**
 * Given a running UAR with a real provider/model configured,
 * when a user prompts the default agent,
 * then it returns the correct answer.
 *
 * Run: node scripts/behaviors/prompt.mjs
 * Optional environment: UAR_BASE_URL, UAR_MODEL, UAR_RESULT_FILE.
 * Requires Node.js 20 or later. No packages or test runner required.
 */
import assert from 'node:assert/strict';
import { randomInt, randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import os from 'node:os';

const timestamp = new Date().toISOString();
const platform = { darwin: 'macos', linux: 'linux', win32: 'windows' }[process.platform] ?? process.platform;
const resultFile = resolve(process.env.UAR_RESULT_FILE ??
  `.prometheus/verification/${timestamp.replaceAll(':', '-')}-prompt-${platform}.json`);
const left = randomInt(10, 50);
const right = randomInt(10, 50);
const prompt = `What is ${left} + ${right}? Reply with only the integer answer.`;
const result = {
  behavior: 'A user can prompt the configured default agent',
  given: 'A running UAR with a genuine provider/model configured',
  when: prompt,
  then: `The default agent responds with ${left + right}`,
  timestamp,
  client_platform: platform,
  os_release: os.release(),
  node_version: process.version,
  agent_id: 'default-agent',
  status: 'FAILED',
};
const started = performance.now();

try {
  const base = new URL(process.env.UAR_BASE_URL ?? 'http://127.0.0.1:3000');
  assert.ok(['http:', 'https:'].includes(base.protocol), 'UAR_BASE_URL must use HTTP or HTTPS');
  assert.ok(!base.username && !base.password, 'UAR_BASE_URL must not contain URL credentials');
  const endpoint = new URL('/v1/chat/completions', base);
  result.endpoint = endpoint.href;
  const headers = { 'Content-Type': 'application/json' };
  const request = {
    agent_id: result.agent_id,
    session_id: randomUUID(),
    stream: false,
    memory_enabled: false,
    max_completion_tokens: 512,
    messages: [{ role: 'user', content: prompt }],
  };
  if (process.env.UAR_MODEL) request.model = process.env.UAR_MODEL;
  const response = await fetch(endpoint, {
    method: 'POST', headers, body: JSON.stringify(request),
    redirect: 'error', signal: AbortSignal.timeout(120_000),
  });
  result.http_status = response.status;
  if (response.status !== 200) result.server_error = (await response.text()).slice(0, 2000);
  assert.equal(response.status, 200, `Expected HTTP 200, received ${response.status}`);
  const body = await response.json();
  assert.equal(body.object, 'chat.completion', 'Expected a UAR chat completion');
  result.model = body.model;
  if (process.env.UAR_MODEL) assert.equal(body.model, process.env.UAR_MODEL, 'Unexpected configured model');
  const answer = body.choices?.[0]?.message?.content;
  result.answer = answer;
  assert.equal(typeof answer, 'string', 'Expected a text completion');
  assert.equal(answer.trim(), String(left + right), 'The agent did not return the correct answer');
  result.status = 'VERIFIED';
} catch (error) {
  result.error = error instanceof Error ? error.message : String(error);
  process.exitCode = 1;
} finally {
  result.duration_ms = Math.round(performance.now() - started);
  await mkdir(dirname(resultFile), { recursive: true });
  await writeFile(resultFile, `${JSON.stringify(result, null, 2)}\n`, { flag: 'wx', mode: 0o600 });
  console.log(JSON.stringify({ ...result, result_file: resultFile }, null, 2));
}
