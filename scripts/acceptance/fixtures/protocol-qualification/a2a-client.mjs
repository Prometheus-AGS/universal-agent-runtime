// Independent official SDK client. Reuse the exact existing workspace pin;
// installation/execution belongs only to the approved whole-set gate.
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { readFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

async function pinnedClient({ url, workspace }) {
  const manifestPath = resolve('frontend/packages/prometheus-entity-management/packages/entity-graph-a2a/package.json');
  const manifest = JSON.parse(await readFile(manifestPath, 'utf8'));
  assert.equal(manifest.dependencies['@a2a-js/sdk'], '1.0.1');
  const require = createRequire(manifestPath);
  const modulePath = require.resolve('@a2a-js/sdk/client');
  let directory = dirname(modulePath), actual;
  while (!actual) {
    try {
      const pkg = JSON.parse(await readFile(resolve(directory, 'package.json'), 'utf8'));
      if (pkg.name === '@a2a-js/sdk') actual = pkg;
    } catch (error) { if (error.code !== 'ENOENT') throw error; }
    const parent = dirname(directory);
    assert.ok(actual || parent !== directory, 'Resolved SDK package identity missing');
    directory = parent;
  }
  assert.equal(actual.version, '1.0.1', 'Installed SDK must match the existing exact pin');
  const { Role, TaskState } = await import(pathToFileURL(require.resolve('@a2a-js/sdk')).href);
  const { ClientFactory, DefaultAgentCardResolver, JsonRpcTransportFactory } = await import(pathToFileURL(modulePath).href);
  const wire = [];
  const fetchImpl = async (input, options = {}) => {
    const headers = new Headers(options.headers ?? (input instanceof Request ? input.headers : undefined));
    for (const name of ['authorization', 'proxy-authorization', 'x-api-key', 'x-uar-admin-key']) {
      assert.equal(headers.has(name), false, 'Functional client never carries authentication');
    }
    headers.set('x-uar-workspace-id', workspace);
    const inherited = options.signal ?? (input instanceof Request ? input.signal : undefined);
    // Keep the official client's abort signal. Aborting an observer is never
    // substituted for the separate SDK CancelTask operation below.
    const timeout = AbortSignal.timeout(120_000);
    const signal = inherited ? AbortSignal.any([inherited, timeout]) : timeout;
    const response = await fetch(input, { ...options, headers, redirect: 'error', signal });
    wire.push({ method: options.method ?? 'GET', status: response.status, content_type: response.headers.get('content-type') });
    return response;
  };
  const client = await new ClientFactory({
    transports: [new JsonRpcTransportFactory({ fetchImpl })],
    preferredTransports: ['JSONRPC'],
    cardResolver: new DefaultAgentCardResolver({ fetchImpl }),
  }).createFromUrl(url);
  assert.equal(client.protocolVersion, '1.0');
  return { client, Role, TaskState, wire, identity: { sdk: '@a2a-js/sdk', sdk_version: actual.version, protocol: '1.0', transport: 'JSONRPC streaming' } };
}

function message(Role, text, messageId) {
  return {
    tenant: '', metadata: {},
    message: { role: Role.ROLE_USER, messageId, taskId: '', contextId: '', metadata: {}, extensions: [], referenceTaskIds: [],
      parts: [{ content: { $case: 'text', value: text }, mediaType: 'text/plain', filename: '', metadata: {} }] },
    configuration: { acceptedOutputModes: ['text/plain'], historyLength: 0, blocking: true },
  };
}

async function expectTaskError(operation, reason, code) {
  let failure;
  try { await operation(); } catch (error) { failure = error; }
  assert.ok(failure, `Official SDK must reject ${reason}`);
  assert.equal(failure.reason, reason);
  assert.equal(failure.envelopeCode, code);
  return { reason: failure.reason, envelope_code: failure.envelopeCode };
}

export async function qualifyA2A({ url, workspace, text, messageId }) {
  const { client, Role, TaskState, wire, identity } = await pinnedClient({ url, workspace });
  const events = [];
  for await (const event of client.sendMessageStream(message(Role, text, messageId))) events.push(event);
  assert.ok(events.length > 1, 'Independent client must observe progress beyond discovery');
  const status = events.filter(event => event.payload?.$case === 'statusUpdate').at(-1)?.payload.value;
  assert.ok(status, 'Independent client must decode a task status update');
  assert.equal(status.status?.state, TaskState.TASK_STATE_COMPLETED);
  // A completed task is an explicit negative control for cancellation. Its
  // typed refusal must not change its task identity or completed result.
  const task = await client.getTask({ tenant: '', id: status.taskId, historyLength: 0 });
  const terminalCancel = await expectTaskError(
    () => client.cancelTask({ tenant: '', id: task.id, metadata: {} }), 'TASK_NOT_CANCELABLE', -32002,
  );
  const after = await client.getTask({ tenant: '', id: task.id, historyLength: 0 });
  assert.equal(after.id, task.id); assert.equal(after.contextId, task.contextId);
  assert.equal(after.status?.state, TaskState.TASK_STATE_COMPLETED);
  return { ...identity, wire, events, status, terminal_cancel_refusal: terminalCancel };
}

export async function qualifyA2ACancellation({ url, workspace, text, messageId, waitForActive }) {
  const { client, Role, TaskState, wire, identity } = await pinnedClient({ url, workspace });
  const evidence = { ...identity, wire, events: [], cancellation: null, original_task: null, errors: [] };
  const observer = new AbortController();
  let canceled = false;
  try {
    const card = await client.getAgentCard();
    assert.equal(card.capabilities?.streaming, true);
    evidence.card = card;
    for await (const event of client.sendMessageStream(message(Role, text, messageId), { signal: observer.signal })) {
      evidence.events.push(event);
      if (event.payload?.$case !== 'task' || canceled) continue;
      const task = event.payload.value;
      assert.ok(task.id && task.contextId, 'Cancellation must use the original independent task/context');
      evidence.original_task = { id: task.id, context_id: task.contextId };
      await waitForActive(task);
      const live = await client.getTask({ tenant: '', id: task.id, historyLength: 0 });
      assert.ok([TaskState.TASK_STATE_SUBMITTED, TaskState.TASK_STATE_WORKING].includes(live.status?.state));
      try {
        evidence.cancellation = await client.cancelTask({ tenant: '', id: task.id, metadata: {} });
      } catch (error) {
        // Preserve actual refusal and uncertainty instead of treating an
        // observer abort, natural completion or pending intent as cancellation.
        evidence.cancellation_refusal = { reason: error.reason, envelope_code: error.envelopeCode };
        evidence.refusal_task_readback = await client.getTask({ tenant: '', id: task.id, historyLength: 0 });
        throw error;
      }
      assert.equal(evidence.cancellation.id, task.id);
      assert.equal(evidence.cancellation.contextId, task.contextId);
      assert.equal(evidence.cancellation.status?.state, TaskState.TASK_STATE_CANCELED);
      assert.notEqual(evidence.cancellation.metadata?.cleanup_unconfirmed, true);
      assert.notEqual(evidence.cancellation.metadata?.execution_unconfirmed, true);
      canceled = true;
    }
    assert.equal(canceled, true, 'Actual official CancelTask must have executed');
    const final = evidence.events.filter(event => event.payload?.$case === 'statusUpdate').at(-1)?.payload.value;
    assert.equal(final?.taskId, evidence.original_task.id);
    assert.equal(final?.contextId, evidence.original_task.context_id);
    assert.equal(final?.status?.state, TaskState.TASK_STATE_CANCELED);
    assert.equal(evidence.events.some(event => event.payload?.$case === 'statusUpdate'
      && event.payload.value.status?.state === TaskState.TASK_STATE_COMPLETED), false, 'The cancelled original task cannot also report completed');
    const readback = await client.getTask({ tenant: '', id: evidence.original_task.id, historyLength: 0 });
    assert.equal(readback.status?.state, TaskState.TASK_STATE_CANCELED);
    assert.notEqual(readback.metadata?.cleanup_unconfirmed, true);
    evidence.readback = readback;
    evidence.errors.push(await expectTaskError(
      () => client.cancelTask({ tenant: '', id: evidence.original_task.id, metadata: {} }), 'TASK_NOT_CANCELABLE', -32002,
    ));
    const missing = `pr476-missing-${messageId}`;
    evidence.errors.push(await expectTaskError(
      () => client.getTask({ tenant: '', id: missing, historyLength: 0 }), 'TASK_NOT_FOUND', -32001,
    ));
    evidence.errors.push(await expectTaskError(
      () => client.cancelTask({ tenant: '', id: missing, metadata: {} }), 'TASK_NOT_FOUND', -32001,
    ));
    return evidence;
  } catch (error) {
    error.qualification = evidence;
    throw error;
  } finally {
    // Local observer cleanup is separate from the asserted protocol operation.
    observer.abort();
  }
}
