import { createHmac } from 'node:crypto';
import { createBdd } from 'playwright-bdd';
import type { Page } from '@playwright/test';
import {
  expect,
  openFreshThread,
  sendMessageAndWait,
  startNewConversation,
  switchAgentViaUI,
  test,
} from '../support/world';

const { When, Then } = createBdd(test);
const JWT_SECRET = 'bdd-dev-secret-at-least-32-characters-long';

interface CrossScreenEvidence {
  exactAnswers?: boolean;
  jwt?: { verifiedStatus: number; anonymousStatus: number };
  isolation?: {
    sameSession: boolean;
    crossSessionDenied: boolean;
    sameKb: boolean;
    crossKbDenied: boolean;
    sameMemory: boolean;
    crossMemoryDenied: boolean;
    conversationPrivate: boolean;
  };
}

const evidence = new WeakMap<Page, CrossScreenEvidence>();

function signedJwt(subject: string, tenantId = 'screen-validation-tenant'): string {
  const now = Math.floor(Date.now() / 1000);
  const encode = (value: unknown) => Buffer.from(JSON.stringify(value)).toString('base64url');
  const header = encode({ alg: 'HS256', typ: 'JWT' });
  const payload = encode({
    sub: subject,
    name: subject,
    roles: ['admin'],
    tenant_id: tenantId,
    iat: now,
    exp: now + 3600,
  });
  const input = `${header}.${payload}`;
  const signature = createHmac('sha256', JWT_SECRET).update(input).digest('base64url');
  return `${input}.${signature}`;
}

When('I ask the default and orchestrator agents their deterministic questions', async ({ page }) => {
  await openFreshThread(page);
  await switchAgentViaUI(page, 'Default Assistant');
  await sendMessageAndWait(page, 'What is 2 plus 2?');
  await expect(page.getByText('2 plus 2 is 4.', { exact: true }).last()).toBeVisible();

  await startNewConversation(page);
  await switchAgentViaUI(page, 'Orchestrator');
  await sendMessageAndWait(page, 'Review this Rust ownership boundary');
  await expect(page.getByText('The ownership boundary is sound.', { exact: true }).last()).toBeVisible();
  await expect(page.getByText('[rust-reviewer]', { exact: true }).last()).toBeVisible();
  evidence.set(page, { ...evidence.get(page), exactAnswers: true });
});

Then('both exact answers are visible and the orchestrator contribution is attributed', async ({ page }) => {
  expect(evidence.get(page)?.exactAnswers).toBe(true);
  await expect(page.getByText('The ownership boundary is sound.', { exact: true }).last()).toBeVisible();
  await expect(page.getByText('[rust-reviewer]', { exact: true }).last()).toBeVisible();
});

When('I compare verified and anonymous credential requests', async ({ page }) => {
  const token = signedJwt('screen-validator');
  await page.goto('/about');
  const result = await page.evaluate(async ({ token }) => {
    const verified = await fetch('/api/uar/credentials', {
      headers: { Authorization: `Bearer ${token}` },
    });
    const anonymous = await fetch('/api/uar/credentials', {
      headers: { Authorization: '' },
    });
    return { verifiedStatus: verified.status, anonymousStatus: anonymous.status };
  }, { token });
  evidence.set(page, { ...evidence.get(page), jwt: result });
  await page.goto('/admin/credentials');
  await expect(page.getByTestId('admin-section-credentials')).toBeVisible();
});

Then('the verified credential request succeeds and the anonymous request is rejected', async ({ page }) => {
  expect(evidence.get(page)?.jwt).toEqual({ verifiedStatus: 200, anonymousStatus: 401 });
  await expect(page.getByTestId('credentials-add')).toBeVisible();
});

When('two verified subjects address the same session memory and knowledge identifiers', async ({ page }) => {
  const tokenA = signedJwt('screen-owner-a');
  const tokenB = signedJwt('screen-owner-b');
  await page.goto('/about');
  const result = await page.evaluate(async ({ tokenA, tokenB }) => {
    async function jsonRequest(
      path: string,
      token: string,
      init: RequestInit = {},
    ): Promise<{ status: number; body: unknown }> {
      const response = await fetch(path, {
        ...init,
        headers: {
          Authorization: `Bearer ${token}`,
          ...(init.body ? { 'Content-Type': 'application/json' } : {}),
          ...init.headers,
        },
      });
      const text = await response.text();
      let body: unknown = null;
      if (text) {
        try {
          body = JSON.parse(text);
        } catch {
          body = text;
        }
      }
      return { status: response.status, body };
    }

    const sessionId = crypto.randomUUID();
    const chatCreate = await jsonRequest('/api/chat/completion', tokenA, {
      method: 'POST',
      body: JSON.stringify({
        model: 'openai/gpt-5.4-mini',
        messages: [{ role: 'user', content: 'What is 2 plus 2?' }],
        stream: false,
        memory_enabled: false,
      }),
      headers: { 'X-UAR-Session-ID': sessionId },
    });
    if (chatCreate.status !== 200) {
      throw new Error(`session creation failed: ${JSON.stringify(chatCreate)}`);
    }
    const sameSession = await jsonRequest(`/api/uar/sessions/${sessionId}/context-stats`, tokenA);
    const crossSession = await jsonRequest(`/api/uar/sessions/${sessionId}/context-stats`, tokenB);

    const kbCreate = await jsonRequest('/api/knowledge', tokenA, {
      method: 'POST',
      body: JSON.stringify({ name: `screen-private-${crypto.randomUUID()}`, description: 'owner A only' }),
    });
    const kbId = (kbCreate.body as { id?: string }).id;
    if (!kbId) throw new Error(`knowledge base creation failed: ${JSON.stringify(kbCreate)}`);
    const sameKb = await jsonRequest(`/api/knowledge/${kbId}`, tokenA);
    const crossKb = await jsonRequest(`/api/knowledge/${kbId}`, tokenB);

    const secret = `screen-memory-${crypto.randomUUID()}`;
    const memoryCreate = await jsonRequest('/api/memory', tokenA, {
      method: 'POST',
      body: JSON.stringify({ content: secret, categories: ['screen-validation'], agent_id: 'default-agent' }),
    });
    if (memoryCreate.status !== 200) {
      throw new Error(`memory creation failed: ${JSON.stringify(memoryCreate)}`);
    }
    const memoryQuery = new URLSearchParams({ q: secret, agent_id: 'default-agent' });
    const sameMemory = await jsonRequest(`/api/memory?${memoryQuery}`, tokenA);
    const crossMemory = await jsonRequest(`/api/memory?${memoryQuery}`, tokenB);

    const conversationId = crypto.randomUUID();
    const policySave = await jsonRequest(`/api/uar/conversations/${conversationId}/policy`, tokenA, {
      method: 'PUT',
      body: JSON.stringify({ memory_enabled: false }),
    });
    if (policySave.status !== 200) {
      throw new Error(`conversation policy save failed: ${JSON.stringify(policySave)}`);
    }
    const samePolicy = await jsonRequest(`/api/uar/conversations/${conversationId}/policy`, tokenA);
    const crossPolicy = await jsonRequest(`/api/uar/conversations/${conversationId}/policy`, tokenB);

    const sameRows = sameMemory.status === 200 && Array.isArray(sameMemory.body)
      ? sameMemory.body as Array<Record<string, unknown>>
      : null;
    const crossRows = crossMemory.status === 200 && Array.isArray(crossMemory.body)
      ? crossMemory.body as Array<Record<string, unknown>>
      : null;
    return {
      sameSession: sameSession.status === 200,
      crossSessionDenied: crossSession.status === 404,
      sameKb: sameKb.status === 200,
      crossKbDenied: crossKb.status === 404,
      sameMemory: sameRows?.some((row) => row.content === secret
        && row.user_id === 'screen-owner-a'
        && row.agent_id === 'default-agent') === true,
      crossMemoryDenied: crossRows?.every((row) => row.content !== secret) === true,
      conversationPrivate: samePolicy.status === 200
        && (samePolicy.body as { memory_enabled?: boolean })?.memory_enabled === false
        && crossPolicy.status === 200
        && crossPolicy.body === null,
    };
  }, { tokenA, tokenB });

  evidence.set(page, { ...evidence.get(page), isolation: result });
  await page.goto('/admin/memory');
  await expect(page.getByTestId('admin-section-memory')).toBeVisible();
  await page.getByPlaceholder('filter by user id').fill('screen-owner-a');
  await page.getByRole('button', { name: 'apply' }).click();
});

Then('the owner sees every resource and the other subject sees none', async ({ page }) => {
  expect(evidence.get(page)?.isolation).toEqual({
    sameSession: true,
    crossSessionDenied: true,
    sameKb: true,
    crossKbDenied: true,
    sameMemory: true,
    crossMemoryDenied: true,
    conversationPrivate: true,
  });
  await expect(page.getByRole('heading', { name: 'memory browser' })).toBeVisible();
});
