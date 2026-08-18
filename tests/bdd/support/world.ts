import { test as base } from 'playwright-bdd';
import type { Page } from '@playwright/test';
import { STUB_BASE_URL } from './ports';

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
}>({
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
 */
export async function switchAgentViaUI(page: Page, agentTitle: string): Promise<void> {
  await page.locator('[aria-label="Select agent"]').click();
  await page.locator('[placeholder="Search agents..."]').fill(agentTitle);
  await page.getByText(agentTitle, { exact: true }).first().click();
}
