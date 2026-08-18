import { createHmac } from 'node:crypto';
import { createBdd } from 'playwright-bdd';
import type { Page } from '@playwright/test';
import {
  expect,
  openFreshThread,
  sendMessageAndWait,
  test,
  waitForDbReady,
} from '../support/world';

const { Given, When, Then } = createBdd(test);
const JWT_SECRET = 'bdd-dev-secret-at-least-32-characters-long';
const completed = new WeakMap<Page, Set<string>>();

type RuntimeReplayWindow = Window & {
  __uarRuntimeReplay?: {
    reset: () => void;
    replayAll: () => void;
    replayApprovalStatus: (status: 'approved' | 'denied' | 'expired') => void;
  };
};

function base64Url(value: string): string {
  return Buffer.from(value).toString('base64url');
}

function signedJwt(subject: string): string {
  const now = Math.floor(Date.now() / 1000);
  const header = base64Url(JSON.stringify({ alg: 'HS256', typ: 'JWT' }));
  const payload = base64Url(JSON.stringify({
    sub: subject,
    name: subject,
    roles: ['admin'],
    tenant_id: `tenant-${subject}`,
    iat: now,
    exp: now + 3600,
  }));
  const signingInput = `${header}.${payload}`;
  const signature = createHmac('sha256', JWT_SECRET).update(signingInput).digest('base64url');
  return `${signingInput}.${signature}`;
}

async function openAdmin(page: Page, section: string): Promise<void> {
  await page.goto(`/admin/${section}`);
  await expect(page.getByTestId(`admin-section-${section}`)).toBeVisible({ timeout: 45_000 });
}

async function replayRuntime(page: Page): Promise<void> {
  await page.waitForFunction(() => Boolean((window as RuntimeReplayWindow).__uarRuntimeReplay));
  await page.evaluate(() => {
    const replay = (window as RuntimeReplayWindow).__uarRuntimeReplay;
    if (!replay) throw new Error('runtime replay helper not installed');
    replay.reset();
    replay.replayAll();
  });
}

async function validateThreads(page: Page): Promise<void> {
  await openFreshThread(page);
  await sendMessageAndWait(page, 'What is 2 plus 2?');
  await expect(page.getByText('2 plus 2 is 4', { exact: false }).last()).toBeVisible();
  await expect(page.getByRole('button', { name: /What is 2 plus 2\?/ }).first()).toBeVisible();
  await page.waitForTimeout(500);
  await page.reload();
  await waitForDbReady(page);
  await page.getByRole('button', { name: /What is 2 plus 2\?/ }).first().click();
  await expect(page.getByText('2 plus 2 is 4', { exact: false }).last()).toBeVisible();
}

async function validateAbout(page: Page): Promise<void> {
  await page.goto('/about');
  await expect(page.getByRole('heading', { name: 'Universal Agent Runtime' })).toBeVisible();
  await expect(page.getByText(/healthy|online|ok/i).first()).toBeVisible({ timeout: 15_000 });
}

async function validateRuntime(page: Page): Promise<void> {
  await openAdmin(page, 'runtime');
  await replayRuntime(page);
  await expect(page.getByText('Live Replay Run')).toBeVisible();
  await expect(page.getByText('Replay tool execution')).toBeVisible();
}

async function validateRuns(page: Page): Promise<void> {
  await openAdmin(page, 'runs');
  await replayRuntime(page);
  await expect(page.getByRole('region', { name: 'Artifacts · 1' })).toContainText('Replay Diagnostics Artifact');
  await expect(page.getByRole('region', { name: 'Tool calls · 1' })).toContainText('provider.health.check');
}

async function validateApprovals(page: Page): Promise<void> {
  await openAdmin(page, 'approvals');
  await replayRuntime(page);
  await expect(page.getByText('1 pending')).toBeVisible();
  await page.evaluate(() => {
    const replay = (window as RuntimeReplayWindow).__uarRuntimeReplay;
    if (!replay) throw new Error('runtime replay helper not installed');
    replay.replayApprovalStatus('denied');
  });
  await expect(page.getByText('0 pending')).toBeVisible();
  await expect(page.getByText('denied')).toBeVisible();
}

async function validateProtocols(page: Page): Promise<void> {
  await openAdmin(page, 'protocols');
  await replayRuntime(page);
  await expect(page.getByText('TOOL_CALL_ARGS')).toBeVisible();
  await expect(page.getByText('Replay A2UI Surface')).toBeVisible();
  await expect(page.getByText('gpt-5.4')).toBeVisible();
}

async function validateProviders(page: Page): Promise<void> {
  await openAdmin(page, 'providers');
  await expect(page.getByTestId('providers-heading')).toHaveText(/providers/i);
  const configured = page.getByTestId('provider-row-openai');
  await expect(configured).toBeVisible();
  await configured.click();
  await expect(page.getByText('Configured', { exact: true }).first()).toBeVisible();
}

async function validateCredentials(page: Page): Promise<void> {
  const provider = `bdd-screen-${Date.now()}`;
  await openAdmin(page, 'credentials');
  await page.getByTestId('credentials-add').click();
  await page.getByLabel('Provider ID').fill(provider);
  await page.getByLabel('API Key').fill('screen-secret-1234');
  await page.getByTestId('credentials-save').click();
  const row = page.getByTestId(`credential-row-${provider}`);
  await expect(row).toBeVisible();
  await expect(row).toContainText('1234');
  await page.getByRole('button', { name: `Delete ${provider} key` }).click();
  await page.getByRole('alertdialog').getByRole('button', { name: 'Delete' }).click();
  await expect(row).not.toBeVisible();
}

async function validateModels(page: Page): Promise<void> {
  await openAdmin(page, 'models');
  await page.getByPlaceholder('search models…').fill('gpt-5.4-mini');
  await expect(page.getByText('gpt-5.4-mini', { exact: true }).first()).toBeVisible();
  await expect(page.getByTestId('models-count')).not.toHaveText('0');
}

async function validateSkills(page: Page): Promise<void> {
  const title = `BDD Screen Skill ${Date.now()}`;
  await openAdmin(page, 'skills');
  await page.getByRole('button', { name: 'new skill', exact: true }).first().click();
  await page.getByPlaceholder('Customer Success Coach').fill(title);
  const dialog = page.getByRole('dialog');
  await dialog.evaluate(async (element) => {
    await Promise.all(element.getAnimations({ subtree: true }).map((animation) => animation.finished));
  });
  const created = page.waitForResponse((response) =>
    response.url().endsWith('/api/skills') && response.request().method() === 'POST');
  const createSkill = dialog.getByRole('button', { name: 'Create Skill' });
  await expect(createSkill).toBeEnabled();
  await createSkill.click({ force: true });
  const createResponse = await created;
  if (!createResponse.ok()) {
    throw new Error(`skill creation failed: ${createResponse.status()} ${await createResponse.text()}`);
  }
  await expect(page.getByText(title, { exact: true })).toBeVisible();
  await page.getByRole('button', { name: `Disable ${title}` }).click();
  await expect(page.getByRole('button', { name: `Enable ${title}` })).toBeVisible();
  await page.getByRole('button', { name: `Enable ${title}` }).click();
  await expect(page.getByRole('button', { name: `Disable ${title}` })).toBeVisible();
}

async function validateAgents(page: Page): Promise<void> {
  const title = `BDD Screen Agent ${Date.now()}`;
  await openAdmin(page, 'agents');
  await page.getByRole('button', { name: 'New agent' }).click();
  await page.locator('#agent-name').fill(title);
  const editor = page.getByRole('dialog');
  await expect(editor).toBeVisible();
  await editor.evaluate(async (element) => {
    await Promise.all(element.getAnimations().map((animation) => animation.finished));
  });
  const create = editor.getByRole('button', { name: 'Create Agent', exact: true });
  await expect(create).toBeEnabled();
  const created = page.waitForResponse((response) =>
    response.url().endsWith('/api/agents') && response.request().method() === 'POST');
  await create.click();
  const createResponse = await created;
  if (!createResponse.ok()) {
    throw new Error(`agent creation failed: ${createResponse.status()} ${await createResponse.text()}`);
  }
  await expect(page.getByText(title, { exact: true })).toBeVisible();
  await page.goto('/threads');
  await waitForDbReady(page);
  await page.locator('[aria-label="Select agent"]').click();
  await page.locator('[placeholder="Search agents..."]').fill(title);
  await expect(page.getByText(title, { exact: true }).first()).toBeVisible();
}

async function validateTools(page: Page): Promise<void> {
  await openAdmin(page, 'tools');
  await page.getByPlaceholder('Search tools...').fill('native_echo');
  await expect(page.getByText('native_echo', { exact: false }).first()).toBeVisible();
}

async function validateAuth(page: Page): Promise<void> {
  const name = `bdd-screen-key-${Date.now()}`;
  await openAdmin(page, 'auth');
  await page.getByRole('button', { name: 'New key' }).click();
  await page.getByLabel('Key name').fill(name);
  await page.getByRole('dialog').getByRole('button', { name: 'Create' }).click();
  await expect(page.getByText(/copy it now/i)).toBeVisible();
  await expect(page.getByText(name, { exact: true })).toBeVisible();
  await page.getByRole('button', { name: `Revoke ${name}` }).click();
  await page.getByRole('alertdialog').getByRole('button', { name: 'Revoke' }).click();
  await expect(page.getByText(name, { exact: true })).not.toBeVisible();
}

async function validateKnowledge(page: Page): Promise<void> {
  const baseName = `BDD Screen KB ${Date.now()}`;
  const filename = 'screen-validation.txt';
  const fact = 'The screen validation marker is HELIOS-2048.';
  await openAdmin(page, 'knowledge');
  await page.getByRole('button', { name: 'New', exact: true }).click();
  await page.getByLabel('Name', { exact: true }).fill(baseName);
  await page.getByLabel('Description', { exact: true }).fill('Screen validation fixture');
  await page.getByRole('button', { name: 'Create', exact: true }).click();
  await page.getByText(baseName, { exact: true }).click();
  await page.getByLabel('Select files to upload to this knowledge base').setInputFiles({
    name: filename,
    mimeType: 'text/plain',
    buffer: Buffer.from(fact),
  });
  const documentRow = page.getByTestId(/^knowledge-document-/).filter({ hasText: filename });
  await expect(documentRow.getByText('indexed', { exact: true })).toBeVisible({ timeout: 30_000 });
  await page.getByPlaceholder('Search by meaning across all documents...').fill('screen validation marker');
  await page.getByRole('button', { name: 'Search', exact: true }).click();
  await expect(page.getByText(fact, { exact: true })).toBeVisible({ timeout: 15_000 });
}

async function validateMemory(page: Page): Promise<void> {
  await openAdmin(page, 'memory');
  await expect(page.getByRole('heading', { name: 'memory browser' })).toBeVisible();
  await page.getByPlaceholder('filter by user id').fill('screen-validator');
  await page.getByRole('button', { name: 'apply' }).click();
  await expect(page.getByText('MEMORY', { exact: true })).not.toBeVisible();
}

async function validateCompiler(page: Page): Promise<void> {
  await openAdmin(page, 'compiler');
  const response = page.waitForResponse((res) =>
    res.url().includes('/api/compiler/sessions') && res.request().method() === 'POST');
  await page.getByRole('button', { name: /new session|create session/ }).first().click();
  const created = await response;
  if (!created.ok()) {
    throw new Error(`compiler session creation failed: ${created.status()} ${await created.text()}`);
  }
  const body = await created.json() as { id?: string };
  expect(body.id).toBeTruthy();
  await expect(page.getByText(body.id!, { exact: false })).toBeVisible();
}

async function validateSettings(page: Page): Promise<void> {
  await openAdmin(page, 'settings');
  await page.getByRole('button', { name: /Resilience/ }).click();
  await page.getByRole('button', { name: 'Reset Defaults', exact: true }).click();
  const field = page.getByText('Requests Per Second', { exact: true }).locator('..').locator('..').locator('input');
  await expect(field).toBeVisible();
  const original = await field.inputValue();
  const replacement = original === '10' ? '11' : '10';
  await field.fill(replacement);
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  await expect(page.getByText('Settings saved', { exact: true })).toBeVisible();
  await page.reload();
  await page.getByRole('button', { name: /Resilience/ }).click();
  await expect(field).toHaveValue(replacement);
  await field.fill(original);
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  await expect(page.getByText('Settings saved', { exact: true })).toBeVisible();
}

async function validateA2ui(page: Page): Promise<void> {
  await openAdmin(page, 'a2ui-testing');
  await expect(page.getByRole('heading', { name: 'A2UI Live Testing' })).toBeVisible();
  const preview = page.getByRole('region', { name: 'A2UI surface preview' });
  await expect(preview).toBeVisible();
  await preview.getByRole('button', { name: 'Preview action' }).click();
  await expect(preview.getByText(/Preview action captured/)).toBeVisible();
}

async function validateMcpHealth(page: Page): Promise<void> {
  await openAdmin(page, 'mcp-health');
  const response = page.waitForResponse((res) => res.url().includes('/api/uar/mcp/health'));
  await page.getByRole('button', { name: 'Refresh' }).click();
  expect((await response).ok()).toBeTruthy();
  await expect(page.getByRole('heading', { name: 'Tool Server Health' })).toBeVisible();
  await expect(page.getByText(/servers · auto-refresh 30s/)).toBeVisible();
}

async function validateCost(page: Page): Promise<void> {
  await openAdmin(page, 'cost');
  await replayRuntime(page);
  await expect(page.getByTestId('cost-run-count')).toHaveText('1');
  await expect(page.getByRole('region', { name: 'Spend summary' })).toContainText('$');
}

const validators: Record<string, (page: Page) => Promise<void>> = {
  '/threads': validateThreads,
  '/about': validateAbout,
  '/admin/runtime': validateRuntime,
  '/admin/runs': validateRuns,
  '/admin/approvals': validateApprovals,
  '/admin/protocols': validateProtocols,
  '/admin/providers': validateProviders,
  '/admin/credentials': validateCredentials,
  '/admin/models': validateModels,
  '/admin/skills': validateSkills,
  '/admin/agents': validateAgents,
  '/admin/tools': validateTools,
  '/admin/auth': validateAuth,
  '/admin/knowledge': validateKnowledge,
  '/admin/memory': validateMemory,
  '/admin/compiler': validateCompiler,
  '/admin/settings': validateSettings,
  '/admin/a2ui-testing': validateA2ui,
  '/admin/mcp-health': validateMcpHealth,
  '/admin/cost': validateCost,
};

Given('a verified browser subject named {string}', async ({ context, page }, subject: string) => {
  await context.setExtraHTTPHeaders({ Authorization: `Bearer ${signedJwt(subject)}` });
  completed.set(page, new Set());
});

When('I exercise the primary function of {string}', async ({ page }, route: string) => {
  const validate = validators[route];
  if (!validate) throw new Error(`no screen validator registered for ${route}`);
  await validate(page);
  completed.get(page)?.add(route);
});

Then('the {string} screen validation is visibly complete', async ({ page }, route: string) => {
  expect(completed.get(page)?.has(route), `screen validator did not complete for ${route}`).toBe(true);
  await expect(page.locator('body')).toBeVisible();
  const slug = route.replace(/^\//, '').replaceAll('/', '-') || 'root';
  await page.screenshot({ path: `tests/bdd/test-results/evidence/${slug}.png`, fullPage: false });
});
