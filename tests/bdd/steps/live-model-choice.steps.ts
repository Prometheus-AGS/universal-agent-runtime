import type { Page } from '@playwright/test';
import { createBdd } from 'playwright-bdd';
import { expect, switchAgentViaUI, test } from '../support/world';
import {
  S1_PROVIDER_SPECS,
  agentExplicitModel,
  agentTitleOf,
  configureS1Providers,
  createS1Agent,
  directRequest,
  fetchAllRuntimeAgents,
  readGlobalPolicy,
  readRegistryDefaultId,
  readStoredDefaultProvider,
  setProviderEnabled,
  setSystemDefaultViaApi,
  splitQualifiedModel,
} from '../support/api';
import { clockInsideWindow, parseClock } from '../support/s1';
import type { HeldAcknowledgement, S1Context } from '../support/s1';
import {
  TID,
  cancelClearOverride,
  clickConfirmSystemDefault,
  closeSessionSheet,
  confirmClearOverride,
  conversationSection,
  createConversation,
  errorCodeOf,
  finishSend,
  openEffectiveView,
  openSystemDefaultPanel,
  receiptModelsInConversation,
  receiptOfLastMessage,
  selectSystemDefault,
  sendWithoutWaiting,
  setOverride,
  startClearOverride,
  startFreshThread,
  switchToConversation,
  systemDefaultOptionState,
  tryOverride,
} from '../support/s1-ui';
import type { RefusalObservation } from '../support/s1-ui';
import {
  S1_STUBS,
  heldCount,
  releaseAllStubs,
  requestsForMessage,
  resetAllStubs,
  requestsForModel,
  stubBaseUrlFor,
  stubCapabilities,
  stubRequests,
  lastUserMessageOf,
  totalHeld,
  wireModelMatches,
} from '../support/stubs';

const { Given, When, Then } = createBdd(test);

/** `holdWindowMs` of `fixtures/s1-oracle.json`: how long the live route stays held and watched. */
const HOLD_WINDOW_MS = 2_000;
/** `timeComparator.separationSeconds` margin so a window never overlaps its predecessor. */
const NO_WRITE_WINDOW_MS = 3_000;
/**
 * SCN-SYS-007: how long after the live route's acknowledgement the browser is still watched
 * for a forbidden settings write (for example one fired by an effect of the confirmation).
 * Same length as the no-write window of CTL-SYS-001-HELD-500.
 */
const POST_ACK_SETTLE_MS = NO_WRITE_WINDOW_MS;
/** SCN-SYS-016: the runtime's refusal of an override its provider does not offer. */
const OVERRIDE_REFUSAL_STATUS = 422;
const OVERRIDE_REFUSAL_CODE = 'model_not_offered';
/**
 * The live default route's refusals (`src/uar/api/providers.rs` `set_default`): 404 for a
 * provider that does not exist, 409 for one that is disabled. Keyed by the reason the step names.
 */
const LIVE_ROUTE_REFUSAL_STATUS: Readonly<Record<string, number>> = { 'not found': 404, disabled: 409 };
/**
 * The agent the neg-sys-003 control creates with its own model, so a loaded pinned list is
 * observable: the list is only trusted to be empty of the others once it shows this one.
 */
const CONTROL_PINNED_WITNESS = 'BDD Agent Aurora';

/** The latest refused change of a scenario, recorded by the When step that made it. */
const refusals = new WeakMap<S1Context, RefusalObservation>();

/** A refused live-route request sent the way the console would send it. */
function liveRouteRefusal(providerId: string, status: number, body: string, uiText: string | null): RefusalObservation {
  return { target: 'live-default-route', requested: providerId, status, body, code: errorCodeOf(body), uiText, persisted: null };
}

/** A refusal reason in comparable words: lower case, with `model_not_offered` read as "model not offered". */
function reasonWords(body: string): string {
  return body.toLowerCase().replace(/_/g, ' ');
}

interface ReceiptModel {
  provider: string;
  served: string;
}

/** Splits a receipt model label `<served provider>/<served model>` at its first slash. */
function parseReceiptModel(label: string): ReceiptModel {
  const text = label.trim();
  const slash = text.indexOf('/');
  expect(slash, `a "<provider>/<model>" receipt label, got "${text}"`).toBeGreaterThan(0);
  return { provider: text.slice(0, slash), served: text.slice(slash + 1) };
}

/**
 * The served model id with the serving provider's own registry prefix removed. The stub
 * reports the wire model it was asked for, which is either bare or `<provider>/<model>`
 * (the rule `wireModelMatches` applies to the stub log), so a correct build may render
 * `openai/openai/gpt-5.4-mini`. Only that exact prefix is removed; any other text stays and
 * is compared as it is.
 */
function servedModelId(receipt: ReceiptModel): string {
  const prefix = `${receipt.provider}/`;
  return receipt.served.startsWith(prefix) ? receipt.served.slice(prefix.length) : receipt.served;
}

function livePath(providerId: string): string {
  return `/api/uar/providers/${providerId}/default`;
}

function isLiveRoutePost(url: string, method: string, providerId: string): boolean {
  return method === 'POST' && new URL(url).pathname === livePath(providerId);
}

function requireHeld(s1: S1Context): HeldAcknowledgement {
  if (!s1.held) throw new Error('no held acknowledgement: run the "while the acknowledgement is held" step first');
  return s1.held;
}

function exactly(text: string): RegExp {
  return new RegExp(`^\\s*${text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}\\s*$`, 'i');
}

/** Model id of a token that is either bare or `provider/model`. */
function modelIdOfToken(token: string): string {
  return token.slice(token.lastIndexOf('/') + 1);
}

async function pause(page: Page, ms: number): Promise<void> {
  await page.waitForTimeout(ms);
}

/** Rejects when `promise` has not settled in `ms`; the timer never outlives the call. */
async function within<T>(promise: Promise<T>, ms: number, what: string): Promise<T> {
  let timer: ReturnType<typeof setTimeout> | undefined;
  const expired = new Promise<never>((_resolve, reject) => {
    timer = setTimeout(() => reject(new Error(`${what} did not happen within ${ms} ms`)), ms);
  });
  try {
    return await Promise.race([promise, expired]);
  } finally {
    clearTimeout(timer);
  }
}

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

Given(
  'the stub providers {string}, {string} and {string} are configured with models {string}, {string} and {string}',
  async ({ s1 }, first: string, second: string, third: string, firstModels: string, secondModels: string, thirdModels: string) => {
    // The step is bound to the frozen S1 vocabulary: each provider has its own stub instance.
    expect([first, second, third]).toEqual(['openai', 'bdd-provider-b', 'bdd-provider-c']);
    const listed = [firstModels, secondModels, thirdModels].map((models) => models.split(',').map((model) => model.trim()));
    expect(listed[0]).toEqual(['gpt-5.4-mini']);
    for (const [index, id] of [second, third].entries()) {
      const spec = S1_PROVIDER_SPECS.find((candidate) => candidate.id === id);
      expect(spec, `${id} is a synthetic S1 provider`).toBeDefined();
      expect(listed[index + 1]).toEqual(spec?.models);
    }
    await s1.begin();
    await configureS1Providers();
    // Verifying the models sent probe requests to the stubs; the scenarios start from empty logs.
    await resetAllStubs();
  },
);

Given('the stub supports {string}', async ({}, kind: string) => {
  for (const stub of S1_STUBS) {
    const kinds = await stubCapabilities(stub.baseUrl);
    expect(kinds, `the stub of ${stub.provider} has no /_stub/capabilities route`).not.toBeNull();
    expect(kinds, `the stub of ${stub.provider} supports ${kind}`).toContain(kind);
  }
});

Given('the system default model is {string}', async ({ s1 }, qualified: string) => {
  const { providerId, modelId } = splitQualifiedModel(qualified);
  const { t0, t1 } = await setSystemDefaultViaApi(providerId, modelId);
  s1.recordWindow(qualified, t0, t1);
});

Given('{int} seconds have passed since the system default was set', async ({ page, s1 }, seconds: number) => {
  const since = s1.latestChangeEnd();
  expect(since, 'the system default was set earlier in this scenario').toBeDefined();
  const remaining = (since ?? 0) + seconds * 1_000 + 250 - Date.now();
  if (remaining > 0) await pause(page, remaining);
});

Given('the provider {string} is disabled', async ({}, providerId: string) => {
  await setProviderEnabled(providerId, false);
});

Given('a conversation {string} with no model override', async ({ page, s1 }, name: string) => {
  await createConversation(page, s1, name, {});
});

Given('a conversation {string} with the model override {string}', async ({ page, s1 }, name: string, model: string) => {
  await createConversation(page, s1, name, { override: model });
});

Given(
  'the conversation {string} and the conversation {string} both use the agent {string}',
  async ({ page, s1 }, first: string, second: string, agent: string) => {
    await createConversation(page, s1, first, { agent });
    await createConversation(page, s1, second, { agent });
  },
);

Given('an agent {string} with the model {string} exists', async ({}, title: string, model: string) => {
  await createS1Agent({ title, model });
});

Given('an agent {string} with the extension model {string} exists', async ({}, title: string, model: string) => {
  await createS1Agent({ title, extensionModel: model });
});

Given('an agent {string} with no model exists', async ({}, title: string) => {
  await createS1Agent({ title });
});

// ---------------------------------------------------------------------------
// Operator actions
// ---------------------------------------------------------------------------

/**
 * The operator's change through the system default picker: save the model, then
 * the live route, waiting for the runtime's acknowledgement. The harness records
 * the clock window around the confirm click and the acknowledgement, and the
 * browser's writes during the change.
 */
async function operatorChange(s1: S1Context, target: string): Promise<void> {
  const { providerId } = splitQualifiedModel(target);
  const operator = await s1.operatorPage();
  await openSystemDefaultPanel(operator);
  s1.changeWritesFrom = s1.writes.length;
  await selectSystemDefault(operator, target);
  const acknowledged = operator.waitForResponse(
    (response) => isLiveRoutePost(response.url(), response.request().method(), providerId),
    { timeout: 20_000 },
  );
  const t0 = Date.now();
  await clickConfirmSystemDefault(operator);
  const response = await acknowledged;
  const t1 = Date.now();
  expect(response.ok(), `POST ${livePath(providerId)} answered ${response.status()}`).toBe(true);
  await expect(operator.getByTestId(TID.confirmation)).toBeVisible({ timeout: 15_000 });
  s1.recordWindow(target, t0, t1);
  s1.changeWritesTo = s1.writes.length;
}

When('the operator changes the system default to {string}', async ({ s1 }, target: string) => {
  await operatorChange(s1, target);
});

When('the operator changes the system default to {string} from another page', async ({ s1 }, target: string) => {
  // The operator always acts on a second page of the browser context, so the
  // chat page keeps its conversation and its in-flight reply.
  await operatorChange(s1, target);
});

When(
  'the operator confirms the system default change to {string} while the acknowledgement is held',
  async ({ s1 }, target: string) => {
    const { providerId } = splitQualifiedModel(target);
    const operator = await s1.operatorPage();
    await openSystemDefaultPanel(operator);

    let markIntercepted: () => void = () => undefined;
    const intercepted = new Promise<void>((resolve) => {
      markIntercepted = resolve;
    });
    const held: HeldAcknowledgement = {
      providerId,
      pair: target,
      t0: 0,
      route: null,
      putStatus: null,
      passThrough: false,
      failedAt: null,
      intercepted,
    };
    s1.held = held;

    // Both are armed before the confirm click.
    operator.on('response', (response) => {
      const request = response.request();
      if (request.method() === 'PUT' && new URL(response.url()).pathname === `/api/uar/providers/${providerId}`) {
        held.putStatus = response.status();
      }
    });
    await operator.route('**/api/uar/providers/*/default', async (route) => {
      const request = route.request();
      const isHeldPost = isLiveRoutePost(request.url(), request.method(), providerId);
      if (!isHeldPost || held.passThrough || held.route !== null) {
        await route.continue();
        return;
      }
      held.route = route;
      markIntercepted();
    });

    s1.changeWritesFrom = s1.writes.length;
    await selectSystemDefault(operator, target);
    held.t0 = Date.now();
    await clickConfirmSystemDefault(operator);

    // Wait for the exact live-route POST to be held and for the PUT before it to
    // have succeeded; never for a response to the held POST.
    await within(intercepted, 20_000, `interception of POST ${livePath(providerId)}`);
    await expect.poll(() => held.putStatus, { message: `PUT /api/uar/providers/${providerId} answered`, timeout: 20_000 }).not.toBeNull();
    expect(held.putStatus ?? 0).toBeGreaterThanOrEqual(200);
    expect(held.putStatus ?? 0).toBeLessThan(300);

    // Keep the POST held for the whole hold window, watching that nothing is confirmed.
    const confirmation = operator.getByTestId(TID.confirmation);
    const deadline = Date.now() + HOLD_WINDOW_MS;
    while (Date.now() < deadline) {
      if (await confirmation.isVisible()) {
        throw new Error('a confirmation appeared while the live route acknowledgement was held');
      }
      await operator.waitForTimeout(100);
    }
  },
);

When('the acknowledgement is released', async ({ s1 }) => {
  const held = requireHeld(s1);
  const route = held.route;
  if (!route) throw new Error('the live route request was not held');
  const operator = await s1.operatorPage();
  const acknowledged = operator.waitForResponse(
    (response) => isLiveRoutePost(response.url(), response.request().method(), held.providerId),
    { timeout: 20_000 },
  );
  held.passThrough = true;
  await route.continue();
  const response = await acknowledged;
  const t1 = Date.now();
  expect(response.ok(), `POST ${livePath(held.providerId)} answered ${response.status()}`).toBe(true);
  s1.recordWindow(held.pair, held.t0, t1);
  s1.changeWritesTo = s1.writes.length;
});

When('the held acknowledgement fails with status {int}', async ({ s1 }, status: number) => {
  const held = requireHeld(s1);
  const route = held.route;
  if (!route) throw new Error('the live route request was not held');
  // Later matching requests pass through to the server, so a retry would change the registry.
  held.passThrough = true;
  await route.fulfill({
    status,
    contentType: 'application/json',
    body: JSON.stringify({ error: 'held acknowledgement failed' }),
  });
  held.failedAt = Date.now();
});

When('the operator tries to change the system default to {string}', async ({ s1 }, target: string) => {
  const { providerId } = splitQualifiedModel(target);
  const operator = await s1.operatorPage();
  await openSystemDefaultPanel(operator);
  s1.changeWritesFrom = s1.writes.length;
  const offered = await systemDefaultOptionState(operator, target);
  // The picker must not let the operator choose it; a build that offers it fails here.
  expect(offered.state, `the system default picker must not offer ${target} (it showed: ${offered.text})`).not.toBe('selectable');
  // The harness then sends the request the console would, so the runtime's own refusal
  // and its reason are observed. Only that response can satisfy the refusal step.
  const direct = await directRequest('POST', livePath(providerId));
  refusals.set(s1, liveRouteRefusal(providerId, direct.status, direct.text, offered.text || null));
  s1.changeWritesTo = s1.writes.length;
});

When('a direct request sets the default provider to {string}', async ({ s1 }, providerId: string) => {
  const response = await directRequest('POST', livePath(providerId));
  refusals.set(s1, liveRouteRefusal(providerId, response.status, response.text, null));
});

When(
  'a client writes the setting {string} as {string} through the settings API',
  async ({ s1 }, key: string, value: string) => {
    const current = await directRequest('GET', `/api/uar/settings/${key}`);
    const revision = current.status === 200 ? String((JSON.parse(current.text) as { revision?: string }).revision ?? '') : '';
    const response = await directRequest('PUT', `/api/uar/settings/${key}`, { value, expected_revision: revision });
    s1.refusalTexts = [`HTTP ${response.status} ${response.text}`];
  },
);

When('I use the per-provider Set as default button for {string}', async ({ s1 }, providerId: string) => {
  const operator = await s1.operatorPage();
  await operator.goto('/admin/providers');
  await expect(operator.getByTestId('admin-section-providers')).toBeVisible({ timeout: 45_000 });
  await operator.getByTestId(`provider-row-${providerId}`).click();
  const acknowledged = operator.waitForResponse(
    (response) => isLiveRoutePost(response.url(), response.request().method(), providerId),
    { timeout: 20_000 },
  );
  const t0 = Date.now();
  await operator.getByRole('button', { name: 'Set as default', exact: true }).click();
  const response = await acknowledged;
  expect(response.ok(), `POST ${livePath(providerId)} answered ${response.status()}`).toBe(true);
  const spec = S1_PROVIDER_SPECS.find((candidate) => candidate.id === providerId);
  if (spec) s1.recordWindow(`${providerId}/${spec.defaultModel}`, t0, Date.now());
  await expect(operator.getByTestId(`provider-row-${providerId}`)).toContainText('default');
});

// ---------------------------------------------------------------------------
// Conversation actions
// ---------------------------------------------------------------------------

When('I switch to the conversation {string}', async ({ page, s1 }, name: string) => {
  await switchToConversation(page, s1, name);
});

When('I switch to a fresh conversation using {string}', async ({ page, s1 }, agentTitle: string) => {
  await startFreshThread(page);
  await switchAgentViaUI(page, agentTitle);
  s1.currentConversation = null;
});

When('I clear the model override of this conversation', async ({ page }) => {
  await startClearOverride(page);
});

When('I set the model override of this conversation to {string}', async ({ page, s1 }, model: string) => {
  await setOverride(page, s1, model);
});

When('I try to set the model override of this conversation to {string}', async ({ page, s1 }, model: string) => {
  refusals.set(s1, await tryOverride(page, s1, model));
});

When('I confirm the change', async ({ page, s1 }) => {
  await confirmClearOverride(page, s1);
});

When('I cancel the change', async ({ page }) => {
  await cancelClearOverride(page);
});

When('I send the message {string} without waiting for the reply', async ({ page, s1 }, message: string) => {
  await sendWithoutWaiting(page, s1, message);
});

// ---------------------------------------------------------------------------
// Served-model checks. Each selects the stub requests whose last user message
// equals the message, requires exactly one, and compares the reply text the
// transcript showed (`bdd-reply-<served wire id>`) captured at send time.
// ---------------------------------------------------------------------------

async function assertReplyCameFrom(s1: S1Context, message: string, qualified: string): Promise<void> {
  const { providerId, modelId } = splitQualifiedModel(qualified);
  const hits = await requestsForMessage(message);
  expect(
    hits.map((hit) => `${hit.provider} asked for ${hit.request.model}`),
    `stub requests whose last user message is "${message}"`,
  ).toHaveLength(1);
  expect(hits[0].provider, `the stub that served "${message}"`).toBe(providerId);
  expect(wireModelMatches(hits[0].request.model, modelId), `wire model ${hits[0].request.model} for "${message}"`).toBe(true);
  expect(s1.replyShown.get(message), `reply text shown for "${message}"`).toBe(`bdd-reply-${modelId}`);
}

Then('the reply to {string} came from {string}', async ({ s1 }, message: string, qualified: string) => {
  await assertReplyCameFrom(s1, message, qualified);
});

Then(
  'the reply to {string} came from {string} with exactly one request',
  async ({ s1 }, message: string, qualified: string) => {
    await assertReplyCameFrom(s1, message, qualified);
  },
);

Then('the reply to {string} did not come from {string}', async ({}, message: string, qualified: string) => {
  const { providerId, modelId } = splitQualifiedModel(qualified);
  const hits = await requestsForMessage(message);
  expect(hits, `stub requests whose last user message is "${message}"`).toHaveLength(1);
  const matches = hits[0].provider === providerId && wireModelMatches(hits[0].request.model, modelId);
  expect(matches, `"${message}" was served by ${hits[0].provider} asking for ${hits[0].request.model}`).toBe(false);
});

Then(
  'the stub of {string} received one request for {string} asking for the model {string}',
  async ({}, providerId: string, message: string, modelId: string) => {
    const logged = (await stubRequests(stubBaseUrlFor(providerId))).filter((request) => lastUserMessageOf(request) === message);
    expect(logged, `requests the stub of ${providerId} logged for "${message}"`).toHaveLength(1);
    expect(wireModelMatches(logged[0].model, modelId), `wire model ${logged[0].model}`).toBe(true);
  },
);

Then('no outgoing request used the model {string}', async ({}, modelId: string) => {
  const hits = await requestsForModel(modelId);
  expect(
    hits.map((hit) => `${hit.provider} asked for ${hit.request.model}`),
    `requests that used ${modelId} on any stub`,
  ).toEqual([]);
});

// ---------------------------------------------------------------------------
// In flight
// ---------------------------------------------------------------------------

Then(
  'the stub for {string} is holding the reply to {string}',
  async ({}, providerId: string, message: string) => {
    const baseUrl = stubBaseUrlFor(providerId);
    await expect
      .poll(() => heldCount(baseUrl), { message: `the stub of ${providerId} holds a reply`, timeout: 20_000 })
      .toBe(1);
    const logged = (await stubRequests(baseUrl)).filter((request) => lastUserMessageOf(request) === message);
    expect(logged, `requests the stub of ${providerId} logged for "${message}"`).toHaveLength(1);
  },
);

Then('the reply to {string} is still held and the stop control is still visible', async ({ page }, message: string) => {
  expect(await totalHeld(), `replies held while "${message}" is generating`).toBe(1);
  await expect(page.locator('[aria-label="Stop generating"]')).toBeVisible();
});

When('the held reply is released', async ({}) => {
  await releaseAllStubs();
});

Then(
  'the reply to {string} finishes with content containing {string}',
  async ({ page, s1 }, message: string, needle: string) => {
    await expect(page.locator('[aria-label="Stop generating"]')).toBeHidden({ timeout: 20_000 });
    await expect(conversationSection(page)).toContainText(needle, { timeout: 15_000 });
    await finishSend(page, s1, message);
  },
);

Then('the stub holds no reply', async ({}) => {
  expect(await totalHeld()).toBe(0);
});

// ---------------------------------------------------------------------------
// Receipts and views
// ---------------------------------------------------------------------------

Then("the answer's receipt names {string} and the origin {string}", async ({ page, s1 }, model: string, origin: string) => {
  const { providerId, modelId } = splitQualifiedModel(model);
  const receipt = await receiptOfLastMessage(page, s1);
  const shown = parseReceiptModel(receipt.model);
  expect(shown.provider, `the serving provider in the receipt "${receipt.model}"`).toBe(providerId);
  expect(servedModelId(shown), `the served model in the receipt "${receipt.model}"`).toBe(modelId);
  expect(receipt.origin, 'the receipt shows an origin').not.toBeNull();
  expect(receipt.origin ?? '', 'the receipt origin').toMatch(exactly(origin));
});

Then("the answer's receipt shows the time the system default became {string}", async ({ page, s1 }, pair: string) => {
  const receipt = await receiptOfLastMessage(page, s1);
  expect(receipt.since, 'the receipt shows since when the system default applies').not.toBeNull();
  const shown = receipt.since ?? '';
  const clock = parseClock(shown);
  expect(clock, `a HH:MM:SS time in the receipt, got "${shown}"`).not.toBeNull();
  const window = s1.latestWindowFor(pair);
  expect(window, `a recorded change window for ${pair}`).toBeDefined();
  if (clock === null || !window) return;
  expect(clockInsideWindow(clock, window), `receipt time ${shown} inside the window of the change to ${pair}`).toBe(true);
  for (const other of s1.windows.filter((candidate) => candidate !== window)) {
    expect(clockInsideWindow(clock, other), `receipt time ${shown} must not lie inside the window of the change to ${other.pair}`).toBe(false);
  }
});

Then(
  "the answer's receipt does not show the time the system default became {string}",
  async ({ page, s1 }, pair: string) => {
    const receipt = await receiptOfLastMessage(page, s1);
    expect(receipt.since, 'the receipt shows since when the system default applies').not.toBeNull();
    const shown = receipt.since ?? '';
    const clock = parseClock(shown);
    expect(clock, `a HH:MM:SS time in the receipt, got "${shown}"`).not.toBeNull();
    if (clock === null) return;
    for (const window of s1.windows.filter((candidate) => candidate.pair === pair)) {
      expect(clockInsideWindow(clock, window), `receipt time ${shown} must not lie inside the window of the change to ${pair}`).toBe(false);
    }
  },
);

Then(
  "the answer's receipt names the serving model {string} on {string}",
  async ({ page, s1 }, servedModel: string, providerId: string) => {
    const receipt = await receiptOfLastMessage(page, s1);
    const shown = parseReceiptModel(receipt.model);
    expect(shown.provider, `the serving provider in the receipt "${receipt.model}"`).toBe(providerId);
    // Exact: the provider-reported id, never the requested id (M1 renders the requested one).
    expect(servedModelId(shown), `the served model in the receipt "${receipt.model}"`).toBe(servedModel);
  },
);

Then("the answer's receipt does not name {string} as the serving model", async ({ page, s1 }, modelId: string) => {
  const receipt = await receiptOfLastMessage(page, s1);
  const shown = parseReceiptModel(receipt.model);
  // Strict: the last path segment of the served model, so no prefix form can hide the id.
  expect(modelIdOfToken(shown.served), `the served model in the receipt "${receipt.model}"`).not.toBe(modelId);
});

Then("no answer's receipt names {string}", async ({ page, s1 }, modelId: string) => {
  // The answer to the message just sent must have rendered its receipt first, so this
  // cannot pass merely because no receipt has rendered yet.
  await receiptOfLastMessage(page, s1);
  const fields = await receiptModelsInConversation(page);
  expect(fields.join('\n'), 'receipt model fields in this conversation').not.toContain(modelId);
});

async function expectEffectiveView(
  page: Page,
  s1: S1Context,
  name: string,
  expected: { requested?: string; effective: string; origin: string },
): Promise<void> {
  await closeSessionSheet(page);
  await switchToConversation(page, s1, name);
  // Returns once the block has left its loading state (aria-busy="false").
  const view = await openEffectiveView(page);
  try {
    // Each value is read from its own field, never from the block's whole text: the block
    // also shows the requested model and, with nothing saved, a sentence naming the system default.
    await expect(view.getByTestId(TID.sessionEffectiveModel), 'the effective model field').toHaveText(exactly(expected.effective));
    await expect(view.getByTestId(TID.sessionEffectiveOrigin), 'the effective origin field').toHaveText(exactly(expected.origin));
    if (expected.requested !== undefined) {
      await expect(view.getByTestId(TID.sessionRequested), 'the requested model field').toHaveText(exactly(expected.requested));
    }
  } finally {
    await closeSessionSheet(page);
  }
}

Then(
  'the {string} conversation shows the effective model {string} from {string}',
  async ({ page, s1 }, name: string, effective: string, origin: string) => {
    await expectEffectiveView(page, s1, name, { effective, origin });
  },
);

Then(
  'the {string} conversation shows the requested model {string} and the effective model {string} from {string}',
  async ({ page, s1 }, name: string, requested: string, effective: string, origin: string) => {
    await expectEffectiveView(page, s1, name, { requested, effective, origin });
  },
);

// ---------------------------------------------------------------------------
// Preview and results
// ---------------------------------------------------------------------------

Then(
  'the preview says this conversation will answer with {string} from {string}',
  async ({ page }, model: string, origin: string) => {
    const preview = page.getByTestId(TID.clearPreview);
    await expect(preview).toBeVisible();
    // The preview container is visible while it still says "Checking what this conversation
    // would inherit", so each value is awaited with a retrying assertion on its own field.
    await expect(preview.getByTestId(TID.inheritedModel), 'the inherited model in the clear preview').toHaveText(exactly(model));
    await expect(preview, 'the origin in the clear preview').toContainText(`from ${origin}`);
  },
);

Then('the preview does not name {string}', async ({ page }, model: string) => {
  const preview = page.getByTestId(TID.clearPreview);
  await expect(preview).toBeVisible();
  expect(await preview.innerText(), 'the clear preview').not.toContain(model);
});

Then(
  'the confirmation shows the requested value {string} and the activation {string}',
  async ({ s1 }, requested: string, activation: string) => {
    const operator = await s1.operatorPage();
    const confirmation = operator.getByTestId(TID.confirmation);
    await expect(confirmation).toBeVisible({ timeout: 15_000 });
    await expect(confirmation).toContainText(requested);
    await expect(confirmation).toContainText(activation);
  },
);

Then(/^no confirmation is shown(?: yet)?$/, async ({ s1 }) => {
  const operator = await s1.operatorPage();
  await expect(operator.getByTestId(TID.confirmation)).toBeHidden();
});

Then('the system default control does not yet show {string} as live', async ({ s1 }, qualified: string) => {
  const operator = await s1.operatorPage();
  await expect(operator.getByTestId(TID.livePair)).not.toContainText(qualified);
});

Then('the system default control shows {string} as live', async ({ s1 }, qualified: string) => {
  const operator = await s1.operatorPage();
  await expect(operator.getByTestId(TID.livePair)).toContainText(qualified, { timeout: 15_000 });
});

Then('the system default control still shows {string} as live', async ({ s1 }, qualified: string) => {
  const operator = await s1.operatorPage();
  await expect(operator.getByTestId(TID.livePair)).toContainText(qualified);
});

Then('an error says the system default was not changed', async ({ s1 }) => {
  const operator = await s1.operatorPage();
  const error = operator.getByTestId(TID.error);
  await expect(error).toBeVisible({ timeout: 10_000 });
  await expect(error).toContainText(/not changed/i);
  // With the error rendered the response has been handled, so a confirmation shown on any
  // settled response would exist by now; "no confirmation is shown" alone ran before that.
  await expect(operator.getByTestId(TID.confirmation), 'no confirmation beside the error').toHaveCount(0);
});

/**
 * Judged on the response to the specific refused request only; the picker's label never
 * counts. An override refusal (SCN-SYS-016) must be the runtime's 422 `model_not_offered`
 * and must leave the conversation's saved model unchanged.
 */
Then('the change is refused with a reason mentioning {string}', async ({ s1 }, needle: string) => {
  const refusal = refusals.get(s1);
  expect(refusal, 'a refused change was recorded earlier in this scenario').toBeDefined();
  if (!refusal) return;
  const shown = `HTTP ${refusal.status} ${refusal.body} (requested ${refusal.requested}; picker showed "${refusal.uiText ?? ''}")`;
  expect(refusal.status, `the refused request answered a client error: ${shown}`).toBeGreaterThanOrEqual(400);
  expect(refusal.status, `the refused request answered a client error: ${shown}`).toBeLessThan(500);
  const liveStatus = refusal.target === 'live-default-route' ? LIVE_ROUTE_REFUSAL_STATUS[needle.toLowerCase()] : undefined;
  if (liveStatus !== undefined) {
    expect(refusal.status, `the live route status for a "${needle}" refusal: ${shown}`).toBe(liveStatus);
  }
  if (refusal.target === 'conversation-override') {
    expect(refusal.status, `the override save status: ${shown}`).toBe(OVERRIDE_REFUSAL_STATUS);
    expect(refusal.code, `the override save error code: ${shown}`).toBe(OVERRIDE_REFUSAL_CODE);
    expect(refusal.persisted, 'the saved model was read around the refused save').not.toBeNull();
    const persisted = refusal.persisted ?? { before: null, after: null };
    expect(persisted.after, 'the model saved for the conversation after the refused save').toBe(persisted.before);
    expect(persisted.after, 'the refused model was not saved').not.toBe(refusal.requested);
  }
  expect(reasonWords(refusal.body), `the reason in the refused response: ${shown}`).toContain(needle.toLowerCase());
});

Then(
  'the change result lists exactly {string} and {string} as pinned',
  async ({ s1 }, first: string, second: string) => {
    const operator = await s1.operatorPage();
    await expect(operator.getByTestId(TID.confirmation)).toBeVisible({ timeout: 15_000 });
    const listed = operator.getByTestId(TID.pinnedAgent);
    await expect(listed).toHaveCount(2, { timeout: 15_000 });
    const texts = await listed.allInnerTexts();
    for (const title of [first, second]) {
      expect(texts.filter((text) => text.includes(title)), `pinned entries naming ${title}`).toHaveLength(1);
    }
  },
);

Then('the change result does not list {string} as pinned', async ({ s1 }, title: string) => {
  const operator = await s1.operatorPage();
  await expect(operator.getByTestId(TID.confirmation)).toBeVisible({ timeout: 15_000 });
  // The list loads independently of the confirmation, and a list that is loading or failed
  // is empty. Absence only means something once the loaded list shows the agent this control
  // pins on purpose, and the loaded state's follow count.
  await expect(operator.getByTestId(TID.followCount), 'the pinned list has loaded').toBeVisible({ timeout: 15_000 });
  await expect(
    operator.getByTestId(TID.pinnedAgent).filter({ hasText: CONTROL_PINNED_WITNESS }),
    `the loaded pinned list shows ${CONTROL_PINNED_WITNESS}`,
  ).toHaveCount(1, { timeout: 15_000 });
  const texts = await operator.getByTestId(TID.pinnedAgent).allInnerTexts();
  expect(texts.filter((text) => text.includes(title)), `pinned entries naming ${title}`).toEqual([]);
});

Then("the change result's follow count equals the number of other agents", async ({ s1 }) => {
  const operator = await s1.operatorPage();
  const shown = await operator.getByTestId(TID.followCount).innerText();
  const count = /\d+/.exec(shown);
  expect(count, `a number in the follow count "${shown}"`).not.toBeNull();
  const pinned = await operator.getByTestId(TID.pinnedAgent).count();
  const total = (await fetchAllRuntimeAgents()).length;
  expect(Number(count?.[0]), `follow count with ${total} agents and ${pinned} pinned`).toBe(total - pinned);
});

// ---------------------------------------------------------------------------
// Read-backs
// ---------------------------------------------------------------------------

Then('the registry default is still {string}', async ({}, providerId: string) => {
  expect(await readRegistryDefaultId(), 'GET /api/uar/providers default_id').toBe(providerId);
});

Then('the stored default provider is still {string}', async ({}, providerId: string) => {
  expect(await readStoredDefaultProvider(), 'the stored llm.default_provider row').toBe(providerId);
});

Then('the settings API refuses the write with a message pointing to the live default route', async ({ s1 }) => {
  const text = s1.refusalTexts.join('\n');
  expect(text, 'the settings write was refused').toMatch(/HTTP 4\d\d/);
  expect(text).toContain('/api/uar/providers/');
  expect(text).toContain('default');
});

Then('the agent {string} still has no model of its own', async ({}, title: string) => {
  const agent = (await fetchAllRuntimeAgents()).find((candidate) => agentTitleOf(candidate) === title);
  expect(agent, `agent ${title}`).toBeDefined();
  expect(agent ? agentExplicitModel(agent) : 'missing', `model of ${title}`).toBeNull();
});

Then('the global policy still has no model', async ({}) => {
  const policy = await readGlobalPolicy();
  expect(policy.model ?? null, 'run_policy.global model').toBeNull();
});

Then(
  "the console's writes to the server were, in order, {string} and {string}",
  async ({ s1 }, first: string, second: string) => {
    const writes = s1.writesBetween(s1.changeWritesFrom, s1.changeWritesTo).map((write) => `${write.method} ${write.path}`);
    expect(writes, 'the browser writes to /api/uar/** during the change').toEqual([first, second]);
  },
);

Then(
  'the console wrote none of the settings {string}, {string} and {string}',
  async ({ s1 }, first: string, second: string, third: string) => {
    // Watch for a settle window after the acknowledgement, so a forbidden write sent
    // shortly after it (for example by an effect of the confirmation) is recorded too.
    const acknowledgedAt = s1.latestChangeEnd();
    expect(acknowledgedAt, 'the live route was acknowledged earlier in this scenario').toBeDefined();
    const remaining = (acknowledgedAt ?? 0) + POST_ACK_SETTLE_MS - Date.now();
    if (remaining > 0) await (await s1.operatorPage()).waitForTimeout(remaining);
    // The whole write log of the scenario, through the end of the settle window.
    for (const key of [first, second, third]) {
      const offending = s1.writes.filter(
        (write) =>
          write.path.includes(`/settings/${key}`) ||
          (key === 'run_policy.global' && write.path.includes('/settings/presentation-policy')) ||
          (write.body ?? '').includes(key),
      );
      expect(
        offending.map((write) => `${write.method} ${write.path}`),
        `browser writes naming ${key}`,
      ).toEqual([]);
    }
  },
);

Then('the console issued no write to the server after the failed acknowledgement', async ({ s1 }) => {
  const held = requireHeld(s1);
  expect(held.failedAt, 'the held acknowledgement was failed').not.toBeNull();
  const failedAt = held.failedAt ?? 0;
  const operator = await s1.operatorPage();
  const remaining = failedAt + NO_WRITE_WINDOW_MS - Date.now();
  if (remaining > 0) await operator.waitForTimeout(remaining);
  const late = s1.writes.filter((write) => write.at > failedAt).map((write) => `${write.method} ${write.path}`);
  expect(late, `browser writes in the ${NO_WRITE_WINDOW_MS} ms after the failed acknowledgement`).toEqual([]);
  // Still no confirmation once the whole no-write window has passed.
  await expect(operator.getByTestId(TID.confirmation), 'no confirmation after the no-write window').toHaveCount(0);
});
