import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { After, Given, When, Then, Before, setDefaultTimeout } from '@cucumber/cucumber';
import { startDedicatedRuntime } from '../../acceptance/fixtures/dedicated-runtime.mjs';

setDefaultTimeout(240_000);

const COMPILER = '/api/uar/compiler';
const COMPILER_ALIAS = '/api/compiler';

// `local` calls UAR directly on its loopback address with no Authorization
// header, so the opt-in local_loopback_owner makes it the local-host owner.
// `proxied` goes through the fixture's identity proxy, which supplies a
// different owner. The test itself never sends a credential.
async function request(w, via, method, path, payload, expected = [200]) {
  const base = via === 'local' ? w.runtime.runtime : w.runtime.base;
  const res = await fetch(new URL(path, base), {
    method, redirect: 'error', signal: AbortSignal.timeout(120_000),
    headers: { 'Content-Type': 'application/json' },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
  const text = await res.text();
  w.evidence.exchanges.push({ via, method, path, request: payload, status: res.status, response: text.slice(0, 4000) });
  assert.ok(expected.includes(res.status), `${via} ${method} ${path}: ${res.status}: ${text.slice(0, 800)}`);
  if (!text) return null;
  // Refusals can carry a plain-text body; only a JSON body is parsed.
  try { return JSON.parse(text); } catch { return { error: text }; }
}

// The store keeps the document as written; the name is the only unique part.
function specBody(name) {
  return [`# Agent: ${name}`, '', '## Metadata', '```yaml', 'version: "1.0"',
    'description: "Compiler owner isolation probe"', '```', '', '## Identity', '```yaml',
    `name: "${name}"`, 'role: "assistant"', 'persona: "Owner isolation probe"',
    'system_prompt: "Reply briefly."', '```', ''].join('\n');
}

const idsOf = (list) => (Array.isArray(list) ? list.map((record) => record.id) : []);

function begin(w, backend, config) {
  w.started = performance.now();
  w.holder = {};
  w.owned = { local: { specs: [], sessions: [] }, proxied: { specs: [], sessions: [] } };
  w.record = resolve(`.prometheus/verification/compiler-owner-isolation/${randomUUID()}.json`);
  w.evidence = { scenario: w.scenarioName, backend, platform: process.platform,
    authentication_supplied_by_test: false, exchanges: [], fixture: {} };
  // The scenario sends more requests in a burst than the default global rate
  // limit (10/s, burst 20) admits; rate limiting is not under test here.
  const resilience = { requests_per_second: 100, burst_size: 200 };
  return startDedicatedRuntime(w.evidence.fixture, w.holder,
    { backend, label: 'compiler-owner-isolation', config: { ...config, resilience } });
}

async function createRecords(w, via) {
  const name = `owner-iso-${via}-${randomUUID().slice(0, 8)}`;
  const spec = await request(w, via, 'POST', `${COMPILER}/specs`, { content: specBody(name) }, [201]);
  assert.ok(spec?.id, `${via} spec id`);
  w.owned[via].specs.push(spec.id);
  const session = await request(w, via, 'POST', `${COMPILER}/sessions`, undefined, [201]);
  assert.ok(session?.id, `${via} session id`);
  w.owned[via].sessions.push(session.id);
  w[via] = { specId: spec.id, specName: name, sessionId: session.id };
}

async function assertOwnRecordsOnly(w, via, other) {
  const mine = w[via];
  const theirs = w[other];
  const specs = idsOf(await request(w, via, 'GET', `${COMPILER}/specs`));
  assert.ok(specs.includes(mine.specId), `${via} lists its own spec`);
  assert.ok(!specs.includes(theirs.specId), `${via} must not list the ${other} owner's spec`);
  const sessions = idsOf(await request(w, via, 'GET', `${COMPILER}/sessions`));
  assert.ok(sessions.includes(mine.sessionId), `${via} lists its own session`);
  assert.ok(!sessions.includes(theirs.sessionId), `${via} must not list the ${other} owner's session`);
  const spec = await request(w, via, 'GET', `${COMPILER}/specs/${mine.specId}`);
  assert.ok(spec.content.includes(mine.specName), `${via} reads its own spec content`);
  const session = await request(w, via, 'GET', `${COMPILER_ALIAS}/sessions/${mine.sessionId}`);
  assert.equal(session.id, mine.sessionId);
  await request(w, via, 'GET', `${COMPILER}/specs/${theirs.specId}`, undefined, [404]);
  await request(w, via, 'GET', `${COMPILER}/sessions/${theirs.sessionId}`, undefined, [404]);
}

Before(function ({ pickle }) {
  this.scenarioName = pickle.name;
});

Given('a dedicated {word} UAR with local_loopback_owner on', { timeout: 420_000 }, async function (backend) {
  this.runtime = await begin(this, backend, { security: { local_loopback_owner: true } });
});

Given('a dedicated {word} UAR with local_loopback_owner off', { timeout: 420_000 }, async function (backend) {
  // The flag is left at its default (off), as in every existing deployment.
  this.runtime = await begin(this, backend, {});
});

When('the local owner creates a compiler spec and an authoring session', async function () {
  await createRecords(this, 'local');
});

When('the proxied owner creates its own compiler spec and authoring session', async function () {
  await createRecords(this, 'proxied');
});

Then('the local owner lists and reads its own spec and session but not the proxied owner\'s', async function () {
  await assertOwnRecordsOnly(this, 'local', 'proxied');
});

Then('the proxied owner lists and reads its own spec and session but not the local owner\'s', async function () {
  await assertOwnRecordsOnly(this, 'proxied', 'local');
});

Then('each owner gets not found when it compiles, deletes or cancels the other owner\'s records', async function () {
  for (const [via, other] of [['proxied', 'local'], ['local', 'proxied']]) {
    const theirs = this[other];
    // A foreign record answers exactly as an absent one: 404, never 403.
    await request(this, via, 'POST', `${COMPILER}/specs/${theirs.specId}/compile`, undefined, [404]);
    await request(this, via, 'DELETE', `${COMPILER}/specs/${theirs.specId}`, undefined, [404]);
    await request(this, via, 'GET', `${COMPILER_ALIAS}/specs/${theirs.specId}`, undefined, [404]);
    await request(this, via, 'POST', `${COMPILER}/sessions/${theirs.sessionId}/cancel`, undefined, [404]);
    await request(this, via, 'POST', `${COMPILER}/sessions/${theirs.sessionId}/compile`, undefined, [404]);
    await request(this, via, 'GET', `${COMPILER_ALIAS}/sessions/${theirs.sessionId}`, undefined, [404]);
  }
});

Then('the local owner still reads its spec and its session is still active', async function () {
  const spec = await request(this, 'local', 'GET', `${COMPILER}/specs/${this.local.specId}`);
  assert.ok(spec.content.includes(this.local.specName), 'The local spec survived the foreign delete');
  const session = await request(this, 'local', 'GET', `${COMPILER}/sessions/${this.local.sessionId}`);
  assert.equal(session.status, 'active', 'The local session survived the foreign cancel');
  this.evidence.result = 'VERIFIED';
});

Then('creating a compiler spec or an authoring session on the loopback address without credentials is refused', async function () {
  // Without the opt-in the caller is anonymous: no verified owner, so nothing is stored.
  const spec = await request(this, 'local', 'POST', `${COMPILER}/specs`,
    { content: specBody(`owner-iso-anonymous-${randomUUID().slice(0, 8)}`) }, [401]);
  const session = await request(this, 'local', 'POST', `${COMPILER}/sessions`, undefined, [401]);
  this.evidence.refusal = { spec: spec?.code ?? spec?.error, session: session?.code ?? session?.error };
  this.evidence.result = 'VERIFIED';
});

After(async function ({ result }) {
  if (!this.evidence) return;
  const errors = [];
  if (this.runtime?.uarAlive()) {
    for (const via of ['local', 'proxied']) {
      for (const id of this.owned?.[via]?.specs ?? []) {
        try { await request(this, via, 'DELETE', `${COMPILER}/specs/${id}`, undefined, [204, 404]); }
        catch (error) { errors.push(`${via} spec cleanup: ${error}`); }
      }
      for (const id of this.owned?.[via]?.sessions ?? []) {
        try { await request(this, via, 'POST', `${COMPILER}/sessions/${id}/cancel`, undefined, [204, 404]); }
        catch (error) { errors.push(`${via} session cleanup: ${error}`); }
      }
    }
  }
  if (this.holder?.handle) errors.push(...await this.holder.handle.stop());
  this.evidence.cleanup_errors = errors;
  this.evidence.status = errors.length ? 'FAILED' : result.status;
  this.evidence.duration_ms = Math.round(performance.now() - this.started);
  await mkdir(dirname(this.record), { recursive: true });
  await writeFile(this.record, JSON.stringify(this.evidence, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  this.attach(JSON.stringify({ evidence: this.record, status: this.evidence.status }), 'application/json');
  assert.deepEqual(errors, [], 'Disposable cleanup must succeed');
});
