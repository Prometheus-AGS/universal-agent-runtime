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
import { APP_BASE_URL } from '../support/ports';

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
    scopedMemoryLevels: boolean;
    sameMemory: boolean;
    crossMemoryDenied: boolean;
    conversationPrivate: boolean;
  };
}

const evidence = new WeakMap<Page, CrossScreenEvidence>();

interface McpJsonRpcResponse {
  id?: number;
  result?: {
    protocolVersion?: string;
    content?: Array<{ type?: string; text?: string }>;
  };
  error?: unknown;
}

function parseMcpSseResponse(body: string, requestId: number): McpJsonRpcResponse {
  for (const line of body.split('\n').reverse()) {
    if (!line.startsWith('data:')) continue;
    const candidate = JSON.parse(line.slice('data:'.length).trim()) as McpJsonRpcResponse;
    if (candidate.id === requestId) return candidate;
  }
  throw new Error(`MCP response ${requestId} missing from SSE body: ${body}`);
}

async function openMemoryMcpClient(): Promise<{
  call: (name: string, args: Record<string, unknown>) => Promise<unknown>;
}> {
  const endpoint = `${APP_BASE_URL}/mcp/memory`;
  const baseHeaders = {
    Accept: 'application/json, text/event-stream',
    'Content-Type': 'application/json',
  };
  const initializeId = 1;
  const initialized = await fetch(endpoint, {
    method: 'POST',
    headers: baseHeaders,
    body: JSON.stringify({
      jsonrpc: '2.0',
      id: initializeId,
      method: 'initialize',
      params: {
        protocolVersion: '2025-06-18',
        capabilities: {},
        clientInfo: { name: 'uar-bdd-memory-scopes', version: '1.0.0' },
      },
    }),
  });
  if (!initialized.ok) {
    throw new Error(`memory MCP initialize failed: ${initialized.status} ${await initialized.text()}`);
  }
  const sessionId = initialized.headers.get('mcp-session-id');
  if (!sessionId) throw new Error('memory MCP initialize omitted mcp-session-id');
  const initializeBody = parseMcpSseResponse(await initialized.text(), initializeId);
  if (initializeBody.error) throw new Error(`memory MCP initialize error: ${JSON.stringify(initializeBody.error)}`);
  const protocolVersion = initializeBody.result?.protocolVersion;
  if (!protocolVersion) throw new Error('memory MCP initialize omitted protocolVersion');

  const sessionHeaders = {
    ...baseHeaders,
    'Mcp-Session-Id': sessionId,
    'MCP-Protocol-Version': protocolVersion,
  };
  const notification = await fetch(endpoint, {
    method: 'POST',
    headers: sessionHeaders,
    body: JSON.stringify({ jsonrpc: '2.0', method: 'notifications/initialized' }),
  });
  if (notification.status !== 202) {
    throw new Error(`memory MCP initialized notification failed: ${notification.status} ${await notification.text()}`);
  }

  let nextId = 2;
  return {
    call: async (name, args) => {
      const id = nextId++;
      const response = await fetch(endpoint, {
        method: 'POST',
        headers: sessionHeaders,
        body: JSON.stringify({
          jsonrpc: '2.0',
          id,
          method: 'tools/call',
          params: { name, arguments: args },
        }),
      });
      if (!response.ok) {
        throw new Error(`memory MCP ${name} failed: ${response.status} ${await response.text()}`);
      }
      const rpc = parseMcpSseResponse(await response.text(), id);
      if (rpc.error) throw new Error(`memory MCP ${name} error: ${JSON.stringify(rpc.error)}`);
      const text = rpc.result?.content?.find((item) => item.type === 'text')?.text;
      if (!text) throw new Error(`memory MCP ${name} omitted text content`);
      return JSON.parse(text) as unknown;
    },
  };
}

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
  const assistantContent = page.locator('[data-slot="aui_assistant-message-content"]');
  await openFreshThread(page);
  await switchAgentViaUI(page, 'Default Assistant');
  await sendMessageAndWait(page, 'What is 2 plus 2?');
  await expect(assistantContent.last()).toHaveText(/^2 plus 2 is 4\.$/);

  await startNewConversation(page);
  await switchAgentViaUI(page, 'Orchestrator');
  await sendMessageAndWait(page, 'Review this Rust ownership boundary');
  await expect(assistantContent.last()).toHaveText(
    /^\[rust-reviewer\]\s+The ownership boundary is sound\.$/,
  );
  evidence.set(page, { ...evidence.get(page), exactAnswers: true });
});

Then('both exact answers are visible and the orchestrator contribution is attributed', async ({ page }) => {
  expect(evidence.get(page)?.exactAnswers).toBe(true);
  await expect(page.locator('[data-slot="aui_assistant-message-content"]').last()).toHaveText(
    /^\[rust-reviewer\]\s+The ownership boundary is sound\.$/,
  );
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

When('two verified subjects address scoped memory and the same session and knowledge identifiers', async ({ page }) => {
  const tokenA = signedJwt('screen-owner-a');
  const tokenB = signedJwt('screen-owner-b');
  const scopeNonce = crypto.randomUUID();
  const globalSecret = `screen-global-memory-${scopeNonce}`;
  const agentSecret = `screen-agent-memory-${scopeNonce}`;
  const userSecret = `screen-user-memory-${scopeNonce}`;
  const scopeAgentId = `screen-scope-agent-${scopeNonce}`;
  const memoryMcp = await openMemoryMcpClient();
  const globalMemory = await memoryMcp.call('memory_add', {
    content: globalSecret,
    scope: 'global',
    categories: ['screen-validation'],
  }) as { content?: string; scope?: string };
  const agentMemory = await memoryMcp.call('memory_add', {
    content: agentSecret,
    scope: 'agent',
    agent_id: scopeAgentId,
    categories: ['screen-validation'],
  }) as { content?: string; scope?: string; agent_id?: string };
  const userMemory = await memoryMcp.call('memory_add', {
    content: userSecret,
    scope: 'user',
    user_id: 'screen-owner-a',
    agent_id: 'default-agent',
    categories: ['screen-validation'],
  }) as { content?: string; scope?: string; user_id?: string; agent_id?: string };
  const allMemories = await memoryMcp.call('memory_list', {}) as Array<Record<string, unknown>>;
  const agentMemories = await memoryMcp.call('memory_list', {
    agent_id: scopeAgentId,
  }) as Array<Record<string, unknown>>;
  const userMemories = await memoryMcp.call('memory_list', {
    user_id: 'screen-owner-a',
  }) as Array<Record<string, unknown>>;
  const scopedMemoryLevels = globalMemory.scope === 'global'
    && globalMemory.content === globalSecret
    && agentMemory.scope === 'agent'
    && agentMemory.agent_id === scopeAgentId
    && userMemory.scope === 'user'
    && userMemory.user_id === 'screen-owner-a'
    && allMemories.some((row) => row.content === globalSecret && row.scope === 'global')
    && agentMemories.some((row) => row.content === agentSecret
      && row.scope === 'agent' && row.agent_id === scopeAgentId)
    && userMemories.some((row) => row.content === userSecret
      && row.scope === 'user' && row.user_id === 'screen-owner-a');
  await page.goto('/about');
  const result = await page.evaluate(async ({ tokenA, tokenB, userSecret, scopedMemoryLevels }) => {
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

    const memoryQuery = new URLSearchParams({ q: userSecret, agent_id: 'default-agent' });
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
      scopedMemoryLevels,
      sameMemory: sameRows?.some((row) => row.content === userSecret
        && row.scope === 'User'
        && row.user_id === 'screen-owner-a'
        && row.agent_id === 'default-agent') === true,
      crossMemoryDenied: crossRows?.every((row) => row.content !== userSecret) === true,
      conversationPrivate: samePolicy.status === 200
        && (samePolicy.body as { memory_enabled?: boolean })?.memory_enabled === false
        && crossPolicy.status === 200
        && crossPolicy.body === null,
    };
  }, { tokenA, tokenB, userSecret, scopedMemoryLevels });

  evidence.set(page, { ...evidence.get(page), isolation: result });
  await page.goto('/admin/memory');
  await expect(page.getByTestId('admin-section-memory')).toBeVisible();
  await page.getByPlaceholder('filter by user id').fill('screen-owner-a');
  await page.getByRole('button', { name: 'apply' }).click();
});

Then('all memory levels resolve and the owner sees private resources while the other subject sees none', async ({ page }) => {
  expect(evidence.get(page)?.isolation).toEqual({
    sameSession: true,
    crossSessionDenied: true,
    sameKb: true,
    crossKbDenied: true,
    scopedMemoryLevels: true,
    sameMemory: true,
    crossMemoryDenied: true,
    conversationPrivate: true,
  });
  await expect(page.getByRole('heading', { name: 'memory browser' })).toBeVisible();
});
