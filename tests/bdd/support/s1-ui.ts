import type { Locator, Page } from '@playwright/test';
import { splitQualifiedModel } from './api';
import type { S1Context } from './s1';
import { expect, openFreshThread, sendMessageAndWait, startNewConversation, switchAgentViaUI } from './world';

/**
 * UI access for the S1 live-model-choice steps. Every selector the console
 * (writer W6) and the picker (writer W5) must provide lives here, so a change of
 * element id is a one-file edit. Every id below was placed by W5 (pickers) or W6
 * (console) and checked against their source in Round 3 of the W3 handoff.
 */
export const TID = {
  // REQUIRED
  livePair: 'system-default-live-pair',
  confirmation: 'system-default-confirmation',
  error: 'system-default-error',
  receipt: 'answer-receipt',
  receiptModel: 'answer-receipt-model',
  receiptOrigin: 'answer-receipt-origin',
  receiptSince: 'answer-receipt-since',
  effective: 'session-config-effective',
  clearPreview: 'session-config-clear-preview',
  // ADDITIONAL
  pickerTrigger: 'system-default-picker-trigger',
  confirmButton: 'system-default-confirm',
  pinnedAgent: 'system-default-pinned-agent',
  followCount: 'system-default-follow-count',
  sessionModelTrigger: 'session-config-model-trigger',
  sessionClear: 'session-config-clear',
  clearConfirm: 'session-config-clear-confirm',
  clearCancel: 'session-config-clear-cancel',
  sessionRequested: 'session-config-requested',
  sessionEffectiveModel: 'session-config-effective-model',
  sessionEffectiveOrigin: 'session-config-effective-origin',
  modelOption: 'model-option',
} as const;

const CONVERSATION = 'section[aria-label="Conversation"]';
const THREAD_LIST = '[aria-label="Thread list"]';
const REPLY_PATTERN = /bdd-reply-[A-Za-z0-9._-]+/g;
/** Root of one assistant turn in the transcript (`enhanced-thread.tsx` AssistantMessage). */
const ASSISTANT_TURN = '[data-role="assistant"]';
/** How long a receipt may take to render under the answer to the message just sent. */
const RECEIPT_TIMEOUT_MS = 15_000;
const RECEIPT_POLL_MS = 100;

/**
 * The message most recently sent in a scenario and the assistant turns the transcript held
 * just before it was sent, so its receipt is read from its own answer and never from an
 * earlier turn. Kept here, keyed by the scenario's context, so `S1Context` stays unchanged.
 */
interface SentMessage {
  readonly message: string;
  readonly assistantIdsBefore: ReadonlySet<string>;
}

const lastSent = new WeakMap<S1Context, SentMessage>();

// ---------------------------------------------------------------------------
// Transcript
// ---------------------------------------------------------------------------

/** The conversation section of the chat page (the transcript and its receipts). */
export function conversationSection(page: Page): Locator {
  return page.locator(CONVERSATION);
}

async function conversationText(page: Page): Promise<string> {
  return page.locator(CONVERSATION).innerText();
}

function repliesIn(text: string): string[] {
  return (text.match(REPLY_PATTERN) ?? []).map((reply) => reply.replace(/[._-]+$/, ''));
}

/** The newest `bdd-reply-...` text in the conversation transcript, or null. */
export async function newestReply(page: Page): Promise<string | null> {
  return repliesIn(await conversationText(page)).at(-1) ?? null;
}

/**
 * Notes how many replies the transcript already holds, and which assistant turns it shows,
 * before a message is sent.
 */
export async function noteRepliesBefore(page: Page, s1: S1Context, message: string): Promise<void> {
  s1.replyCountBefore.set(message, repliesIn(await conversationText(page)).length);
  const ids = (await assistantTurns(page)).map((turn) => turn.id).filter((id): id is string => id !== null);
  lastSent.set(s1, { message, assistantIdsBefore: new Set(ids) });
}

/**
 * Records the reply text the transcript shows for `message`: waits until the
 * transcript holds more `bdd-reply-...` strings than it did at send time, then
 * takes the newest. A later step compares it with the served model's reply text.
 */
export async function captureReply(page: Page, s1: S1Context, message: string): Promise<void> {
  const before = s1.replyCountBefore.get(message) ?? 0;
  await expect
    .poll(async () => repliesIn(await conversationText(page)).length, {
      message: `the transcript shows a reply to "${message}"`,
      timeout: 15_000,
    })
    .toBeGreaterThan(before);
  const reply = await newestReply(page);
  if (reply === null) throw new Error(`no reply text was found in the transcript for "${message}"`);
  s1.replyShown.set(message, reply);
}

/**
 * Sends a message and waits for the reply, as the shared `I send the message`
 * step does; during an S1 scenario it also records the reply text and, after a
 * conversation's first message, names the conversation's thread.
 */
export async function sendAndCapture(page: Page, s1: S1Context, message: string): Promise<void> {
  if (s1.active) await noteRepliesBefore(page, s1, message);
  await sendMessageAndWait(page, message);
  if (!s1.active) return;
  await finishSend(page, s1, message);
}

/** After a reply has finished: records its text and names the conversation's thread after its first message. */
export async function finishSend(page: Page, s1: S1Context, message: string): Promise<void> {
  await captureReply(page, s1, message);
  await materializeCurrentConversation(page, s1);
}

/** Sends a message and returns as soon as the run has started (the stop control is shown). */
export async function sendWithoutWaiting(page: Page, s1: S1Context, message: string): Promise<void> {
  await noteRepliesBefore(page, s1, message);
  await page.locator('[aria-label="Message input"]').fill(message);
  await page.keyboard.press('Enter');
  await expect(page.locator('[aria-label="Stop generating"]')).toBeVisible({ timeout: 10_000 });
}

// ---------------------------------------------------------------------------
// Conversations
//
// A thread is hidden from the sidebar until its first message is sent, so the
// harness keeps a name -> state handle per conversation and names the thread
// (through the sidebar's Rename dialog) after its first message.
// ---------------------------------------------------------------------------

/** Starts a fresh thread, loading the chat page first when it is not open yet. */
export async function startFreshThread(page: Page): Promise<void> {
  if (page.url().includes('/threads')) await startNewConversation(page);
  else await openFreshThread(page);
}

function threadRow(page: Page, name: string): Locator {
  return page
    .locator(THREAD_LIST)
    .locator('div.group')
    .filter({ has: page.getByText(name, { exact: true }) });
}

async function renameActiveThread(page: Page, name: string): Promise<void> {
  const list = page.locator(THREAD_LIST);
  const row = list.locator('div.group').filter({ has: page.locator('button[aria-current="page"]') }).first();
  await expect(row).toBeVisible({ timeout: 10_000 });
  // The runtime titles a thread after its first reply; give that a moment so it
  // cannot land after the rename. A thread that stays untitled is renamed anyway.
  await expect(row)
    .not.toContainText('New conversation', { timeout: 3_000 })
    .catch(() => undefined);
  await row.hover();
  await row.getByRole('button', { name: /^Actions for / }).click();
  await page.getByRole('menuitem', { name: 'Rename' }).click();
  const dialog = page.getByRole('dialog');
  await dialog.getByLabel('Thread name').fill(name);
  await dialog.getByRole('button', { name: 'Save', exact: true }).click();
  await expect(list.getByText(name, { exact: true })).toBeVisible();
}

/** After a conversation's first message, names its thread so it can be switched to. */
async function materializeCurrentConversation(page: Page, s1: S1Context): Promise<void> {
  const conversation = s1.currentConversationState();
  if (!conversation || conversation.materialized) return;
  await renameActiveThread(page, conversation.name);
  conversation.materialized = true;
}

/** Creates a conversation on a fresh thread and makes it the current one. */
export async function createConversation(
  page: Page,
  s1: S1Context,
  name: string,
  options: { agent?: string; override?: string },
): Promise<void> {
  await startFreshThread(page);
  if (options.agent) await switchAgentViaUI(page, options.agent);
  s1.conversations.set(name, {
    name,
    agent: options.agent ?? null,
    override: null,
    materialized: false,
  });
  s1.currentConversation = name;
  if (options.override) await setOverride(page, s1, options.override);
}

/**
 * Makes a conversation current. A named one is selected in the sidebar. One that
 * has not received its first message has no sidebar row, so its handle is moved
 * onto a fresh thread and its recorded agent and override are applied again.
 */
export async function switchToConversation(page: Page, s1: S1Context, name: string): Promise<void> {
  const conversation = s1.conversations.get(name);
  if (!conversation) throw new Error(`no conversation named "${name}" was created in this scenario`);
  if (s1.currentConversation === name) return;
  if (conversation.materialized) {
    await threadRow(page, name).getByRole('button').first().click();
    await expect(threadRow(page, name).locator('button[aria-current="page"]')).toBeVisible();
  } else {
    await startNewConversation(page);
    if (conversation.agent) await switchAgentViaUI(page, conversation.agent);
    if (conversation.override) await setOverrideOnThread(page, conversation.override);
  }
  s1.currentConversation = name;
}

// ---------------------------------------------------------------------------
// Session Configuration sheet
// ---------------------------------------------------------------------------

function sessionSheet(page: Page): Locator {
  return page.getByRole('dialog').filter({ hasText: 'Session Configuration' });
}

export async function openSessionSheet(page: Page): Promise<Locator> {
  await page.getByRole('button', { name: 'Session configuration', exact: true }).click();
  const sheet = sessionSheet(page);
  await expect(sheet).toBeVisible();
  await expect(sheet.getByRole('button', { name: 'Save Configuration' })).toBeEnabled({ timeout: 15_000 });
  return sheet;
}

export async function closeSessionSheet(page: Page): Promise<void> {
  const sheet = sessionSheet(page);
  if (await sheet.isVisible()) await page.keyboard.press('Escape');
  await expect(sheet).toBeHidden();
}

// The model pickers (W5): with seven or fewer options the trigger is a Base UI
// Select with no search box; with more it is a searchable combobox. In both modes
// the trigger carries the `testId` the call site passed, and every option carries
// `data-testid="model-option"` and `data-model-id="<provider>/<model>"`;
// non-selectable options also carry `aria-disabled="true"` and `data-disabled`.

function modelOption(page: Page, qualified: string): Locator {
  return page.locator(`[data-testid="${TID.modelOption}"][data-model-id="${qualified}"]`);
}

/**
 * Opens a picker from its trigger and narrows it to a model when the picker has
 * a search box. The popup is open once either a search box or an option shows.
 */
async function openPickerFor(page: Page, trigger: Locator, qualified: string): Promise<void> {
  const { modelId } = splitQualifiedModel(qualified);
  await trigger.click();
  const search = page.getByPlaceholder(/search models/i);
  await search.or(page.locator(`[data-testid="${TID.modelOption}"]`).first()).first().waitFor({ state: 'visible', timeout: 15_000 });
  if ((await search.count()) > 0) await search.first().fill(modelId);
}

/** Opens the picker and clicks the option for a provider-qualified model. */
async function chooseModel(page: Page, trigger: Locator, qualified: string): Promise<void> {
  await openPickerFor(page, trigger, qualified);
  const option = modelOption(page, qualified);
  await expect(option, `the picker offers ${qualified}`).toBeVisible({ timeout: 10_000 });
  await expect(option, `${qualified} is selectable`).not.toHaveAttribute('aria-disabled', 'true');
  await option.click();
}

/** What a picker offers for a model: absent, shown but not selectable, or selectable. Closes the picker. */
async function optionState(
  page: Page,
  trigger: Locator,
  qualified: string,
): Promise<{ state: 'absent' | 'blocked' | 'selectable'; text: string }> {
  await openPickerFor(page, trigger, qualified);
  const option = modelOption(page, qualified);
  const present = await option
    .first()
    .waitFor({ state: 'attached', timeout: 3_000 })
    .then(() => true)
    .catch(() => false);
  let result: { state: 'absent' | 'blocked' | 'selectable'; text: string } = { state: 'absent', text: '' };
  if (present) {
    const text = (await option.first().innerText()).trim();
    const blocked =
      (await option.first().getAttribute('aria-disabled')) === 'true' ||
      (await option.first().getAttribute('data-disabled')) !== null;
    result = { state: blocked ? 'blocked' : 'selectable', text };
  }
  await page.keyboard.press('Escape');
  return result;
}

function sessionModelTrigger(sheet: Locator): Locator {
  return sheet.getByTestId(TID.sessionModelTrigger);
}

const AGENT_CONFIG_PATH = /^\/api\/uar\/sessions\/[^/]+\/agent-config$/;
/** The same path, capturing the (encoded) session id. */
const AGENT_CONFIG_SESSION = /^\/api\/uar\/sessions\/([^/]+)\/agent-config$/;

/** Sets the current conversation's model override through the sheet; returns the save response. */
async function saveOverrideThroughSheet(
  page: Page,
  qualified: string,
): Promise<{ status: number; text: string }> {
  const sheet = await openSessionSheet(page);
  await chooseModel(page, sessionModelTrigger(sheet), qualified);
  const saved = page.waitForResponse(
    (response) => AGENT_CONFIG_PATH.test(new URL(response.url()).pathname) && response.request().method() === 'POST',
    { timeout: 15_000 },
  );
  await sheet.getByRole('button', { name: 'Save Configuration' }).click();
  const response = await saved;
  return { status: response.status(), text: await response.text() };
}

async function setOverrideOnThread(page: Page, qualified: string): Promise<void> {
  const result = await saveOverrideThroughSheet(page, qualified);
  if (result.status >= 400) throw new Error(`saving the override ${qualified} failed: ${result.status} ${result.text}`);
  await expect(sessionSheet(page)).toBeHidden({ timeout: 10_000 });
}

/** Sets the current conversation's model override and records it on the handle. */
export async function setOverride(page: Page, s1: S1Context, qualified: string): Promise<void> {
  await setOverrideOnThread(page, qualified);
  const conversation = s1.currentConversationState();
  if (conversation) conversation.override = qualified;
}

/**
 * What one refused change produced: the response to the specific refused request and,
 * for a conversation override, the model saved for the conversation before and after it.
 * The picker's visible text is kept for messages only; it never proves a refusal.
 */
export interface RefusalObservation {
  readonly target: 'live-default-route' | 'conversation-override';
  /** The provider-qualified model or provider the refused request asked for. */
  readonly requested: string;
  readonly status: number;
  readonly body: string;
  /** The `code` field of a JSON error body, or null. */
  readonly code: string | null;
  readonly uiText: string | null;
  /** Conversation override only: `model` of `GET .../agent-config` around the refused save. */
  readonly persisted: { readonly before: string | null; readonly after: string | null } | null;
}

/** The `code` field of a JSON error body, or null when the body is not such an object. */
export function errorCodeOf(body: string): string | null {
  try {
    const parsed: unknown = JSON.parse(body);
    if (parsed && typeof parsed === 'object' && 'code' in parsed) {
      const code = (parsed as { code: unknown }).code;
      return typeof code === 'string' ? code : null;
    }
  } catch {
    // A plain-text body has no code.
  }
  return null;
}

function postedModelOf(postData: string | null): string | null {
  if (postData === null) return null;
  try {
    const parsed: unknown = JSON.parse(postData);
    const model = parsed && typeof parsed === 'object' ? (parsed as { model?: unknown }).model : null;
    return typeof model === 'string' ? model : null;
  } catch {
    return null;
  }
}

/**
 * Tries to set an override the provider does not offer. The picker must show the
 * model absent or non-selectable (a build that offers it fails here). The harness
 * then sends the agent-config save the sheet would send, with a minimal body
 * `{agent_id, model}` so the model is the only possible reason for a refusal, and
 * observes that exact request's response through `page.waitForResponse`. It reads
 * the conversation's saved model before and after the save, so a refusal that still
 * persisted the model is caught. Leaves the sheet open, so the previous effective
 * value stays reachable.
 */
export async function tryOverride(page: Page, s1: S1Context, qualified: string): Promise<RefusalObservation> {
  const sheet = await openSessionSheet(page);
  const offered = await optionState(page, sessionModelTrigger(sheet), qualified);
  expect(offered.state, `the picker must not offer ${qualified} (it showed: ${offered.text})`).not.toBe('selectable');
  const sessionId = s1.lastSessionId;
  if (!sessionId) throw new Error('no session id was observed, so the override request cannot be sent');
  const saved = page.waitForResponse(
    (response) => {
      const request = response.request();
      const match = AGENT_CONFIG_SESSION.exec(new URL(response.url()).pathname);
      return (
        request.method() === 'POST' &&
        match !== null &&
        decodeURIComponent(match[1] ?? '') === sessionId &&
        postedModelOf(request.postData()) === qualified
      );
    },
    { timeout: 15_000 },
  );
  const persisted = await page.evaluate(
    async ({ id, model }) => {
      const url = `/api/uar/sessions/${encodeURIComponent(id)}/agent-config`;
      const readSaved = async (): Promise<{ status: number; agentId: string | null; model: string | null }> => {
        const response = await fetch(url);
        if (response.status !== 200) return { status: response.status, agentId: null, model: null };
        const config = (await response.json()) as { agent_id?: unknown; model?: unknown };
        return {
          status: response.status,
          agentId: typeof config.agent_id === 'string' ? config.agent_id : null,
          model: typeof config.model === 'string' ? config.model : null,
        };
      };
      const before = await readSaved();
      await fetch(url, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ agent_id: before.agentId ?? 'default-agent', model }),
      });
      const after = await readSaved();
      return { before, after };
    },
    { id: sessionId, model: qualified },
  );
  const response = await saved;
  const body = await response.text();
  for (const [when, read] of [['before', persisted.before], ['after', persisted.after]] as const) {
    // 200 is a saved config and 204 is none; anything else means the saved model is unknown.
    expect([200, 204], `GET agent-config ${when} the refused save answered ${read.status}`).toContain(read.status);
  }
  return {
    target: 'conversation-override',
    requested: qualified,
    status: response.status(),
    body,
    code: errorCodeOf(body),
    uiText: offered.text || null,
    persisted: { before: persisted.before.model, after: persisted.after.model },
  };
}

/** Starts clearing the current conversation's override and returns the open sheet's preview. */
export async function startClearOverride(page: Page): Promise<Locator> {
  const sheet = await openSessionSheet(page);
  await sheet.getByTestId(TID.sessionClear).click();
  const preview = sheet.getByTestId(TID.clearPreview);
  await expect(preview).toBeVisible({ timeout: 10_000 });
  return preview;
}

export async function cancelClearOverride(page: Page): Promise<void> {
  const sheet = sessionSheet(page);
  const preview = sheet.getByTestId(TID.clearPreview);
  await preview.getByTestId(TID.clearCancel).click();
  await closeSessionSheet(page);
}

/** Confirms the clear; saves the sheet when confirming alone did not close it. */
export async function confirmClearOverride(page: Page, s1: S1Context): Promise<void> {
  const sheet = sessionSheet(page);
  const preview = sheet.getByTestId(TID.clearPreview);
  await preview.getByTestId(TID.clearConfirm).click();
  const closed = await sheet
    .waitFor({ state: 'hidden', timeout: 3_000 })
    .then(() => true)
    .catch(() => false);
  if (!closed) {
    await sheet.getByRole('button', { name: 'Save Configuration' }).click();
    await expect(sheet).toBeHidden({ timeout: 15_000 });
  }
  const conversation = s1.currentConversationState();
  if (conversation) conversation.override = null;
}

/**
 * Opens the sheet on the current conversation and returns its effective-model block once
 * the runtime's answer has loaded. The block is visible while it still says "Checking which
 * model answers…" (`aria-busy="true"`), so the wait is for `aria-busy="false"`. The failure
 * state (Retry alert) carries no `aria-busy` and therefore fails here.
 */
export async function openEffectiveView(page: Page): Promise<Locator> {
  const sheet = await openSessionSheet(page);
  const effective = sheet.getByTestId(TID.effective);
  await expect(effective).toBeVisible({ timeout: 10_000 });
  await expect(effective, 'the effective model has loaded').toHaveAttribute('aria-busy', 'false', { timeout: 15_000 });
  return effective;
}

// ---------------------------------------------------------------------------
// Receipts
// ---------------------------------------------------------------------------

/** The text of one rendered receipt's fields; a field the receipt does not render is null. */
export interface ReceiptFields {
  /** `answer-receipt-model`: `<served provider>/<served model>` as the product renders it. */
  readonly model: string;
  readonly origin: string | null;
  readonly since: string | null;
}

interface AssistantTurn {
  readonly id: string | null;
  readonly text: string;
  readonly receipts: readonly ReceiptFields[];
}

/**
 * One atomic snapshot of the assistant turns in the transcript: each turn's message id, its
 * text, and the fields of every receipt rendered inside it. Read in one browser call, so a
 * turn's id, text and receipt always belong together.
 */
async function assistantTurns(page: Page): Promise<AssistantTurn[]> {
  return conversationSection(page)
    .locator(ASSISTANT_TURN)
    .evaluateAll(
      (nodes, ids) =>
        nodes.map((node) => ({
          id: node.getAttribute('data-message-id'),
          text: node.textContent ?? '',
          receipts: Array.from(node.querySelectorAll(`[data-testid="${ids.receipt}"]`)).map((receipt) => ({
            model: receipt.querySelector(`[data-testid="${ids.model}"]`)?.textContent ?? '',
            origin: receipt.querySelector(`[data-testid="${ids.origin}"]`)?.textContent ?? null,
            since: receipt.querySelector(`[data-testid="${ids.since}"]`)?.textContent ?? null,
          })),
        })),
      { receipt: TID.receipt, model: TID.receiptModel, origin: TID.receiptOrigin, since: TID.receiptSince },
    );
}

/**
 * The answer to the message just sent: the newest assistant turn that was not in the
 * transcript when the message was sent and that shows the reply text captured for it.
 */
function answerTurnOf(turns: readonly AssistantTurn[], sent: SentMessage, reply: string): AssistantTurn | null {
  return turns.filter((turn) => turn.id !== null && !sent.assistantIdsBefore.has(turn.id) && turn.text.includes(reply)).at(-1) ?? null;
}

/**
 * The receipt under the answer to the message most recently sent in this scenario. Waits
 * until that answer's own receipt has rendered (an earlier turn's receipt never stands in
 * for it) and requires exactly one receipt there. Fails when the answer shows no receipt.
 */
export async function receiptOfLastMessage(page: Page, s1: S1Context): Promise<ReceiptFields> {
  const sent = lastSent.get(s1);
  if (!sent) throw new Error('no message was sent in this scenario, so there is no answer receipt to read');
  const reply = s1.replyShown.get(sent.message);
  if (!reply) throw new Error(`the reply to "${sent.message}" was not captured, so its answer cannot be identified`);
  const deadline = Date.now() + RECEIPT_TIMEOUT_MS;
  for (;;) {
    const answer = answerTurnOf(await assistantTurns(page), sent, reply);
    if (answer && answer.receipts.length > 1) {
      throw new Error(`the answer to "${sent.message}" shows ${answer.receipts.length} receipts, expected one`);
    }
    const receipt = answer?.receipts[0];
    if (receipt) return receipt;
    if (Date.now() > deadline) {
      throw new Error(
        answer
          ? `the answer to "${sent.message}" rendered no receipt within ${RECEIPT_TIMEOUT_MS} ms`
          : `no new assistant turn showing "${reply}" for "${sent.message}" within ${RECEIPT_TIMEOUT_MS} ms`,
      );
    }
    await page.waitForTimeout(RECEIPT_POLL_MS);
  }
}

/** The model field of every receipt rendered in the current conversation's transcript. */
export async function receiptModelsInConversation(page: Page): Promise<string[]> {
  return (await assistantTurns(page)).flatMap((turn) => turn.receipts.map((receipt) => receipt.model));
}

/** Whitespace-separated tokens of an element's text. */
export function tokensOf(text: string): string[] {
  return text.split(/\s+/).filter((token) => token !== '');
}

// ---------------------------------------------------------------------------
// System default control (the operator's page)
// ---------------------------------------------------------------------------

export async function openSystemDefaultPanel(operator: Page): Promise<void> {
  if (!operator.url().includes('/admin/providers')) await operator.goto('/admin/providers');
  await expect(operator.getByTestId(TID.livePair)).toBeVisible({ timeout: 45_000 });
}

/** Opens the system default picker and clicks the provider-qualified pair; confirms nothing. */
export async function selectSystemDefault(operator: Page, qualified: string): Promise<void> {
  await chooseModel(operator, operator.getByTestId(TID.pickerTrigger), qualified);
}

/** What the system default picker offers for a pair: absent, not selectable, or selectable. */
export async function systemDefaultOptionState(
  operator: Page,
  qualified: string,
): Promise<{ state: 'absent' | 'blocked' | 'selectable'; text: string }> {
  return optionState(operator, operator.getByTestId(TID.pickerTrigger), qualified);
}

export async function clickConfirmSystemDefault(operator: Page): Promise<void> {
  await operator.getByTestId(TID.confirmButton).click();
}
