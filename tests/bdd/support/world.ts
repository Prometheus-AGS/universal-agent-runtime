import { test as base } from 'playwright-bdd';
import type { Page } from '@playwright/test';
import { STUB_BASE_URL } from './ports';
import { S1Context } from './s1';

/**
 * Every request body the stub LLM has received so far (see
 * `/_stub/requests` in `tests/integration/live/stub_llm.rs`). Used to prove
 * things about the *outgoing* LLM request (system prompt content) that
 * response-routing alone can't show — e.g. RAG-retrieved content actually
 * reaching the system prompt, or a skill's prompt overlay being injected.
 */
export interface StubRequest {
  model: string;
  messages: Array<{ role: string; content: string | null }>;
  tools?: unknown[];
  stream?: boolean;
}

async function fetchStubRequests(): Promise<StubRequest[]> {
  const res = await fetch(`${STUB_BASE_URL}/_stub/requests`);
  if (!res.ok) throw new Error(`stub introspection failed: ${res.status}`);
  const body = (await res.json()) as { requests: StubRequest[] };
  return body.requests;
}

async function resetStubRequests(): Promise<void> {
  await fetch(`${STUB_BASE_URL}/_stub/requests`, { method: 'POST' });
}

/** The system prompt (first `system`-role message) of a captured request, if any. */
export function systemPromptOf(req: StubRequest): string {
  return req.messages.find((m) => m.role === 'system')?.content ?? '';
}

export const test = base.extend<{
  stub: { requests: () => Promise<StubRequest[]>; reset: () => Promise<void> };
  /** S1 live-model-choice state; inert until the S1 Background step calls `begin()`. */
  s1: S1Context;
}>({
  s1: async ({ page, context }, use) => {
    const s1 = new S1Context(page, context);
    await use(s1);
    await s1.dispose();
  },
  stub: async ({}, use) => {
    await resetStubRequests();
    await use({ requests: fetchStubRequests, reset: resetStubRequests });
  },
});

export const { expect } = test;

/**
 * Loads `/threads` and waits for the app shell to be interactive.
 *
 * `tests/e2e/*.spec.ts` waits on a `console` event containing "PGlite
 * database initialized successfully" — that proved racy here: by the time
 * the listener response was checked, the console message had already fired
 * (confirmed via a failed run's page snapshot showing a fully-populated,
 * interactive UI — threads sidebar, agent selector, composer — despite the
 * event-wait timing out). Waiting on a concrete, always-present DOM element
 * (the agent selector trigger) is the same completion signal without the race.
 */
export async function waitForDbReady(page: Page): Promise<void> {
  await page.goto('/threads');
  await page.locator('[aria-label="Select agent"]').waitFor({ state: 'visible', timeout: 15_000 });
}

/** Starts a fresh conversation thread, same button `frontend/e2e/chat-*.spec.ts` already drives. */
export async function startNewConversation(page: Page): Promise<void> {
  const newConv = page.getByRole('button', { name: /New conversation|New thread/ }).first();
  await newConv.waitFor({ state: 'visible', timeout: 15_000 });
  await newConv.click();
  await page.locator('[aria-label="Message input"]').waitFor({ state: 'visible', timeout: 15_000 });
}

/** Loads `/threads`, waits for the local DB, and starts a fresh conversation — the common setup every scenario needs. */
export async function openFreshThread(page: Page): Promise<void> {
  await waitForDbReady(page);
  await startNewConversation(page);
}

/**
 * Sends a chat message via the composer (`ComposerPrimitive.Input`, confirmed
 * `aria-label="Message input"` in `enhanced-thread.tsx`) and waits for the
 * assistant's reply to complete — signaled by the "Stop generating" button
 * (`aria-label="Stop generating"`, visible only while `thread.isRunning`)
 * disappearing again.
 */
export async function sendMessageAndWait(page: Page, message: string): Promise<void> {
  const input = page.locator('[aria-label="Message input"]');
  await input.fill(message);
  await page.keyboard.press('Enter');
  const stopButton = page.locator('[aria-label="Stop generating"]');
  await stopButton.waitFor({ state: 'visible', timeout: 5_000 }).catch(() => {});
  await stopButton.waitFor({ state: 'hidden', timeout: 15_000 });
}

/**
 * Switches the active agent via the real agent-selector UI (`agent-selector.tsx`:
 * trigger `[aria-label="Select agent"]`, `CommandItem` per agent keyed on its
 * title). This is the only mechanism that actually reaches
 * `use-chat-runtime.ts`'s per-turn request (via `AgentConfigContext`) — the
 * session-config-panel's own POST is a documented dead facade (see design.md
 * Findings), so scenario steps must drive this UI, not call an API directly.
 *
 * The click is scoped to the list item (`CommandItem`, `data-slot="command-item"` in
 * `components/ui/command.tsx`): on a fresh thread the trigger itself shows the fallback
 * text "Default Assistant", so an unscoped text match would click the trigger and select
 * nothing. Selecting an item saves the agent through `selectAgentForSession`, which POSTs
 * `/api/uar/sessions/{id}/agent-config`; the switch is complete once that save has answered
 * and the popover has closed.
 */
export async function switchAgentViaUI(page: Page, agentTitle: string): Promise<void> {
  const trigger = page.locator('[aria-label="Select agent"]');
  await trigger.click();
  const search = page.locator('[placeholder="Search agents..."]');
  await search.fill(agentTitle);
  const item = page
    .locator('[data-slot="command-item"]')
    .filter({ has: page.getByText(agentTitle, { exact: true }) })
    .first();
  await expect(item, `the agent list offers "${agentTitle}"`).toBeVisible({ timeout: 15_000 });
  // selectAgentForSession reads the session's config (GET) before saving it (POST). A failed
  // read stops the save, so it is reported by name instead of as a timeout on the POST.
  const read = page.waitForResponse(
    (response) => AGENT_CONFIG_SAVE.test(new URL(response.url()).pathname) && response.request().method() === 'GET',
    { timeout: 15_000 },
  );
  const saved = page.waitForResponse(
    (response) => AGENT_CONFIG_SAVE.test(new URL(response.url()).pathname) && response.request().method() === 'POST',
    { timeout: 15_000 },
  );
  // Neither promise may reject unobserved while the other is awaited.
  read.catch(() => undefined);
  saved.catch(() => undefined);
  await item.click();
  const readResponse = await read;
  if (readResponse.status() >= 400 && readResponse.status() !== 404) {
    throw new Error(
      `selecting the agent "${agentTitle}" failed: GET ${new URL(readResponse.url()).pathname} answered ${readResponse.status()} ${await readResponse.text()}`,
    );
  }
  const response = await saved;
  if (response.status() >= 400) {
    throw new Error(`selecting the agent "${agentTitle}" failed: ${response.status()} ${await response.text()}`);
  }
  await expect(search, 'the agent list closes after a selection').toBeHidden({ timeout: 10_000 });
  await expect(trigger, `the agent selector shows "${agentTitle}"`).toContainText(agentTitle, { timeout: 10_000 });
}

/** The session agent-config save the agent selector issues (`session-configuration-api.ts:209-212`). */
const AGENT_CONFIG_SAVE = /^\/api\/uar\/sessions\/[^/]+\/agent-config$/;
