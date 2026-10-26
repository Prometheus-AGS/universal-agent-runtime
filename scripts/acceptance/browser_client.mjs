/** Deferred installed-UI client. No runner, package installation or build. */
import fs from 'node:fs/promises';
import path from 'node:path';
import crypto from 'node:crypto';
import { isDeepStrictEqual } from 'node:util';

const hash = (bytes) => `sha256:${crypto.createHash('sha256').update(bytes).digest('hex')}`;
const check = (value, message) => { if (!value) throw new Error(message); };
const blocked = (message) => Object.assign(new Error(message), { blocked: true });
let browser;
let context;
let output;
let artifactDir;
let receipt = { client: 'browser', status: 'FAIL', turns: [], artifacts: [] };

async function main() {
  const input = JSON.parse(process.env.UAR_ACCEPTANCE_CLIENT_INPUT ?? '{}');
  output = process.env.UAR_ACCEPTANCE_CLIENT_OUTPUT;
  check(output && path.isAbsolute(output), 'exclusive output path required');
  const base = new URL(input.base_url);
  if (!['http:', 'https:'].includes(base.protocol) || base.username || base.password) throw blocked('supported installed origin required');
  check(input.mode === 'chat' || input.mode === 'a2ui', 'unknown browser profile');
  check(Number.isInteger(input.timeout_seconds) && input.timeout_seconds > 0 && input.timeout_seconds <= 600, 'finite timeout required');
  const timeout = input.timeout_seconds * 1000;
  artifactDir = input.artifact_dir;
  check(path.isAbsolute(artifactDir), 'absolute owned artifact directory required');
  await fs.mkdir(artifactDir, { mode: 0o700 });
  const headers = {};
  for (const [name, reference] of Object.entries(input.headers_env ?? {})) {
    if (!process.env[reference]) throw blocked('explicit owner header environment unavailable');
    headers[name] = process.env[reference];
  }
  if (!Object.keys(headers).some((name) => name.toLowerCase() === 'authorization')) throw blocked('explicit frozen owner authorization required');
  let binary;
  try { binary = await fs.readFile(input.browser_executable); } catch { throw blocked('frozen installed browser binary unavailable'); }
  if (hash(binary) !== input.browser_sha256) throw blocked('installed browser binary differs from frozen artifact');
  let chromium;
  try { ({ chromium } = await import('@playwright/test')); } catch { throw blocked('installed pinned Playwright package unavailable'); }
  browser = await chromium.launch({ executablePath: input.browser_executable, headless: true });
  context = await browser.newContext({ extraHTTPHeaders: headers, viewport: { width: 1280, height: 900 },
    recordVideo: { dir: artifactDir, size: { width: 1280, height: 900 } }, reducedMotion: 'reduce' });
  // Headers never leave the declared installed origin; ambient browser profiles
  // and storage-state imports are deliberately absent.
  await context.route('**/*', async (route) => {
    const target = new URL(route.request().url());
    if (target.origin !== base.origin) await route.abort('blockedbyclient');
    else await route.continue();
  });
  const page = await context.newPage();
  page.setDefaultTimeout(timeout);
  const requests = [];
  const completed = [];
  const assets = [];
  const tasks = [];
  const pageErrors = [];
  page.on('pageerror', () => pageErrors.push('pageerror'));
  page.on('request', (request) => {
    const url = new URL(request.url());
    if (!url.pathname.startsWith('/api/')) return;
    let data = {};
    try { data = request.postDataJSON() ?? {}; } catch { /* Body is not JSON. */ }
    requests.push({ method: request.method(), path: url.pathname, query: url.search,
      session_id: request.headers()['x-uar-session-id'] ?? data.session_id ?? null,
      agent_id: data.agent_id ?? null, model: data.model ?? null,
      fresh_input_present: typeof input.fresh_input === 'string' && JSON.stringify(data).includes(input.fresh_input),
      action_name: data.name ?? null, surface_id: data.surfaceId ?? null });
  });
  page.on('response', (response) => {
    const url = new URL(response.url());
    if (url.origin === base.origin && url.pathname.startsWith('/assets/')) {
      tasks.push((async () => {
        const bytes = await response.body();
        const identity = hash(bytes);
        assets.push({ path: url.pathname, sha256: identity, matched: input.static_assets?.[url.pathname] === identity });
      })());
    }
    if (url.origin !== base.origin || !(/\/api\/chat\/completion$/.test(url.pathname)
        || /\/api\/uar\/runs\/[^/]+\/stream$/.test(url.pathname)
        || /\/a2ui\/actions$/.test(url.pathname))) return;
    tasks.push((async () => {
      let phase = 'body';
      try {
        const body = await response.text();
        phase = 'parsing';
        check(Buffer.byteLength(body) <= input.max_bytes, 'stream observation exceeded byte bound');
        if (/\/a2ui\/actions$/.test(url.pathname)) {
          const value = JSON.parse(body);
          completed.push({ kind: 'action', status: response.status(), run_id: value.run_id,
            continuation_run_id: value.continuation_run_id, outcome: value.status });
          return;
        }
        const events = body.split(/\r?\n\r?\n/).flatMap((block) => {
          const data = block.split(/\r?\n/).filter((line) => line.startsWith('data:')).map((line) => line.slice(5).trimStart()).join('\n');
          if (!data || data === '[DONE]') return [];
          try { return [JSON.parse(data)]; } catch { throw new Error('malformed mounted stream'); }
        });
        const starts = events.filter((event) => event.type === 'RUN_STARTED');
        const finishes = events.filter((event) => event.type === 'RUN_FINISHED');
        const errors = events.filter((event) => ['RUN_ERROR', 'RUN_CANCELLED'].includes(event.type));
        const selection = events.filter((event) => event.type === 'CUSTOM' && event.name === 'uar.agent.selected').map((event) => event.value);
        completed.push({ kind: 'stream', status: response.status(), path: url.pathname,
          session_id: response.request().headers()['x-uar-session-id'] ?? null,
          run_id: starts[0]?.runId ?? finishes[0]?.runId,
          starts: starts.length, finishes: finishes.length, errors: errors.length,
          selections: selection, tool_names: events.filter((event) => event.type === 'TOOL_CALL_START').map((event) => event.toolCallName), content: events.filter((event) => event.type === 'TEXT_MESSAGE_CONTENT').map((event) => event.delta).join('') });
      } catch { completed.push({ kind: phase === 'body' ? 'transport-interruption' : 'observation-failure', path: url.pathname }); }
    })());
  });
  await page.addInitScript(() => {
    window.__uarAcceptanceSelections = [];
    document.addEventListener('DOMContentLoaded', () => {
      const observe = () => {
        for (const element of document.querySelectorAll('[data-agent-selection]')) {
          const id = element.getAttribute('data-agent-selection');
          if (!window.__uarAcceptanceSelections.some((row) => row.id === id)) {
            const stop = document.querySelector('button[aria-label="Stop generating"]');
            window.__uarAcceptanceSelections.push({ id, while_running: Boolean(stop && stop.getClientRects().length),
              label: element.getAttribute('aria-label'), text: element.textContent,
              focus_inside: element.contains(document.activeElement), at: performance.now() });
          }
        }
      };
      new MutationObserver(observe).observe(document.documentElement, { childList: true, subtree: true, attributes: true });
      observe();
    });
  });
  await page.goto(new URL('/threads', base).href, { waitUntil: 'domcontentloaded' });
  check(!(await page.getByText('No Model Configured', { exact: true }).isVisible()), 'installed UI has no configured model');
  await page.getByRole('button', { name: 'New thread', exact: true }).click();
  await page.getByRole('button', { name: 'Select agent', exact: true }).click();
  await page.getByPlaceholder('Search agents...').fill(input.agent_title);
  await page.getByRole('option').filter({ hasText: input.agent_title }).click();
  await page.getByRole('button', { name: 'Session configuration', exact: true }).click();
  await page.getByRole('dialog', { name: 'Session Configuration', exact: true }).getByRole('combobox').filter({ hasText: /^(Agent:|Agent default|Select model override)/ }).click();
  await page.getByPlaceholder('Search models...').fill(input.model.split('/').slice(1).join('/'));
  await page.locator('[data-slot="command-group"]').filter({ has: page.getByText(input.provider_title, { exact: true }) }).getByRole('option').filter({ hasText: input.model.split('/').slice(1).join('/') }).click();
  await page.getByRole('button', { name: 'Save Configuration', exact: true }).click();
  await page.keyboard.press('Escape');

  const screenshot = async (name) => {
    const target = path.join(artifactDir, `${name}.png`);
    await page.screenshot({ path: target, fullPage: true });
  };
  const send = async (text, expected, index) => {
    const prior = completed.length;
    await page.getByRole('textbox', { name: 'Message input', exact: true }).fill(text);
    const responseStarted = page.waitForResponse((response) => response.request().method() === 'POST'
      && new URL(response.url()).pathname === '/api/chat/completion');
    const sendButton = page.getByRole('button', { name: 'Send message', exact: true });
    await sendButton.focus();
    check(await sendButton.evaluate((element) => element === document.activeElement), 'send control is not keyboard focusable');
    await sendButton.press('Enter');
    await responseStarted;
    if (input.selected_agent_id && index === 0) {
      const notice = page.locator(`[data-agent-selection="${input.selected_agent_id}"]`);
      await notice.waitFor({ state: 'visible' });
      check(await page.getByRole('button', { name: 'Stop generating', exact: true }).isVisible(), 'selection was not visible while answer was ongoing');
      check((await notice.innerText()).includes(input.selected_agent_title), 'rendered selection title differs');
      await screenshot('selection-during-answer');
      if (input.reconnect) {
        const dropped = page.waitForEvent('requestfailed', { predicate: (request) =>
          /\/api\/chat\/completion$|\/api\/uar\/runs\/[^/]+\/stream/.test(new URL(request.url()).pathname) });
        await context.setOffline(true);
        try { await dropped; } finally { await context.setOffline(false); }
      }
    }
    await page.getByRole('button', { name: 'Send message', exact: true }).waitFor({ state: 'visible' });
    await Promise.all(tasks);
    const streams = completed.slice(prior).filter((row) => row.kind === 'stream');
    const runIds = [...new Set(streams.map((row) => row.run_id).filter(Boolean))];
    check(runIds.length === 1 && streams.every((row) => row.errors === 0) && streams.some((row) => row.finishes === 1 && row.status === 200), 'UI turn lacks one successful actual native run');
    const streamContent = streams.map((row) => row.content).join('');
    const message = page.locator('[data-slot="aui_assistant-message-root"][data-role="assistant"]').last();
    for (const value of Object.values(expected)) await message.filter({ hasText: String(value) }).waitFor({ state: 'visible' });
    const renderedText = await message.locator('[data-slot="aui_assistant-message-content"]').innerText();
    // An interrupted Response does not expose its partial body through Playwright.
    // The installed UI retains those chunks; grade its actual rendered answer.
    const observedAnswer = input.reconnect ? renderedText : streamContent;
    const answerStart = observedAnswer.indexOf('{');
    const answerEnd = observedAnswer.lastIndexOf('}');
    check(answerStart >= 0 && answerEnd >= answerStart
      && isDeepStrictEqual(JSON.parse(observedAnswer.slice(answerStart, answerEnd + 1)), expected), 'genuine UI answer failed fresh fact/private prompt predicate');
    const content = observedAnswer.slice(answerStart, answerEnd + 1);
    for (const value of Object.values(expected)) check(renderedText.includes(String(value)), 'useful result was not actually rendered');
    if (input.selected_agent_id) {
      const selections = streams.flatMap((row) => row.selections);
      check(selections.length === 1 && selections[0].agent.id === input.selected_agent_id
        && selections[0].agent.title === input.selected_agent_title && selections[0].selectionMethod === 'intent'
        && selections[0].selectedThreadId && /^[0-9a-f]{64}$/.test(selections[0].rosterRevision), 'visible selection lacks actual correlated backend event');
      receipt.selection = selections[0];
    }
    receipt.turns.push({ run_id: runIds[0], session_id: streams.find((row) => row.session_id)?.session_id
      ?? requests.filter((row) => row.path === '/api/chat/completion').at(-1)?.session_id,
      content, rendered: true, answer_source: input.reconnect ? 'rendered-ui-after-reconnect' : 'correlated-stream-and-ui',
      protocol_terminal: 'success' });
    await screenshot(`answer-${index}`);
  };

  if (input.mode === 'chat') {
    await send(input.turns[0], input.expected[0], 0);
    const firstSelectionObservations = await page.evaluate(() => window.__uarAcceptanceSelections);
    await page.reload({ waitUntil: 'domcontentloaded' });
    const retained = page.locator('[data-slot="aui_assistant-message-root"][data-role="assistant"]').last();
    for (const value of Object.values(input.expected[0])) {
      await retained.filter({ hasText: String(value) }).waitFor({ state: 'visible' });
      check((await retained.innerText()).includes(String(value)), 'saved history disappeared on installed UI reload');
    }
    receipt.saved_history_reload = true;
    await send(input.turns[1], input.expected[1], 1);
    check(receipt.turns[0].session_id && receipt.turns[0].session_id === receipt.turns[1].session_id, 'UI reload/follow-up changed canonical session');
    if (input.reconnect) check(requests.filter((row) => row.method === 'GET' && /\/runs\/[^/]+\/stream$/.test(row.path)).some((row) => row.query.includes('last_event_id=')), 'real browser network loss did not use mounted stream reconnect');
    if (input.selected_agent_id) {
      receipt.selection_observations = [...firstSelectionObservations, ...await page.evaluate(() => window.__uarAcceptanceSelections)];
      for (const theme of ['dark', 'light', 'high-contrast']) {
        for (let attempt = 0; attempt < 4; attempt += 1) {
          if (await page.locator('html').evaluate((node, value) => node.classList.contains(value), theme)) break;
          await page.getByRole('button', { name: /^(Dark mode|Light mode|High contrast mode|System theme)$/ }).click();
        }
        check(await page.locator('html').evaluate((node, value) => node.classList.contains(value), theme), 'theme control failed');
        await page.setViewportSize({ width: 320, height: 800 });
        const notice = page.locator(`[data-agent-selection="${input.selected_agent_id}"]`).last();
        const geometry = await notice.evaluate((element) => ({ width: element.clientWidth, scroll: element.scrollWidth,
          height: element.clientHeight, text: element.textContent, label: element.getAttribute('aria-label'),
          page_width: document.documentElement.clientWidth, page_scroll: document.documentElement.scrollWidth }));
        check(geometry.width > 0 && geometry.height > 0 && geometry.scroll <= geometry.width + 1
          && geometry.page_scroll <= geometry.page_width + 1 && geometry.text.includes(input.selected_agent_title)
          && geometry.label === 'Agent selection', 'long-name selection is clipped, overflows or lacks accessible label');
        receipt.themes ??= [];
        receipt.themes.push({ theme, geometry });
        await screenshot(`selection-${theme}-320`);
      }
    }
  } else {
    await page.getByRole('textbox', { name: 'Message input', exact: true }).fill(input.turns[0]);
    await page.getByRole('button', { name: 'Send message', exact: true }).click();
    const surface = page.getByRole('region', { name: 'A2UI display artifact', exact: true });
    await surface.locator('[data-a2ui-surface-state="ready"]').waitFor({ state: 'visible' });
    check((await surface.innerText()).includes(input.surface_heading), 'genuine generated useful A2UI heading missing');
    await page.getByRole('button', { name: 'Send message', exact: true }).waitFor({ state: 'visible' });
    await surface.getByRole('textbox', { name: 'Parcel number', exact: true }).fill(input.fresh_input);
    await screenshot('a2ui-before-action');
    const actionResponse = page.waitForResponse((response) => response.request().method() === 'POST'
      && /\/api\/uar\/runs\/[^/]+\/a2ui\/actions$/.test(new URL(response.url()).pathname));
    const continuationStarted = page.waitForResponse((response) => response.request().method() === 'GET'
      && /\/api\/uar\/runs\/[^/]+\/stream$/.test(new URL(response.url()).pathname));
    await surface.getByRole('button', { name: 'Calculate parcel', exact: true }).click();
    const response = await actionResponse;
    check(response.status() === 202, 'standard A2UI action was not admitted');
    const ack = await response.json();
    check(ack.status === 'continued' && ack.run_id && ack.continuation_run_id && ack.run_id !== ack.continuation_run_id, 'standard action lacks actual continuation identity');
    await continuationStarted;
    await page.getByRole('button', { name: 'Send message', exact: true }).waitFor({ state: 'visible' });
    await Promise.all(tasks);
    check(requests.some((row) => /\/a2ui\/actions$/.test(row.path) && row.fresh_input_present && row.action_name === 'calculate_parcel'), 'actual action did not carry freshly entered user data');
    const continuation = completed.filter((row) => row.kind === 'stream' && row.run_id === ack.continuation_run_id);
    check(continuation.length > 0 && continuation.every((row) => row.errors === 0)
      && continuation.some((row) => row.finishes === 1), 'UI did not consume successful genuine continuation stream');
    const continuedText = continuation.map((row) => row.content).join('');
    check(isDeepStrictEqual(JSON.parse(continuedText), input.expected[0]), 'actual continuation ignored fresh input or saved private prompt');
    check(completed.some((row) => row.kind === 'stream' && row.run_id === ack.run_id && row.finishes === 1 && row.errors === 0 && row.tool_names.includes('a2ui_render')), 'surface did not come from real agent tool invocation');
    const last = page.locator('[data-slot="aui_assistant-message-root"][data-role="assistant"]').last();
    const rendered = await last.innerText();
    for (const value of Object.values(input.expected[0])) check(rendered.includes(String(value)), 'continued genuine model result was not rendered');
    receipt.action = { source_run_id: ack.run_id, continuation_run_id: ack.continuation_run_id,
      action: 'calculate_parcel', fresh_input_sent: true, rendered_result: rendered };
    receipt.turns = [{ run_id: ack.run_id, session_id: requests.find((row) => row.path === '/api/chat/completion')?.session_id,
      protocol_terminal: 'success', rendered: true }];
    await screenshot('a2ui-after-continuation');
  }
  await Promise.all(tasks);
  check(!completed.some((row) => row.kind === 'observation-failure'), 'mounted stream observation failed');
  if (!input.reconnect) check(!completed.some((row) => row.kind === 'transport-interruption'), 'unexpected UI transport failure');
  check(assets.some((row) => /\.js$/.test(row.path)) && assets.every((row) => row.matched), 'loaded UI assets differ from frozen shipped static files');
  check(pageErrors.length === 0, 'installed browser emitted page errors');
  receipt.static_assets = assets;
  receipt.requests = requests;
  receipt.page_errors = pageErrors.length;
  receipt.status = 'PASS';
  receipt.mode = input.mode;
  receipt.ui_url = new URL('/threads', base).href;
}

try { await main(); } catch (error) {
  receipt.status = error.blocked ? 'BLOCKED' : 'FAIL';
  // Never copy arbitrary page, HTTP, console or exception text into evidence.
  receipt.error = 'installed UI client did not complete its required behavior';
  receipt.error_type = error.blocked ? 'prerequisite' : 'predicate_or_transport';
} finally {
  if (context) await context.close().catch(() => {});
  if (browser) await browser.close().catch(() => {});
  if (artifactDir) {
    for (const name of await fs.readdir(artifactDir).catch(() => [])) {
      const file = path.join(artifactDir, name);
      if (/\.(png|webm)$/.test(name)) {
        const bytes = await fs.readFile(file);
        receipt.artifacts.push({ path: file, sha256: hash(bytes), bytes: bytes.length, kind: name.endsWith('.webm') ? 'video' : 'screenshot' });
      }
    }
  }
  if (output) await fs.writeFile(output, JSON.stringify(receipt), { mode: 0o600, flag: 'wx' });
}
process.exitCode = receipt.status === 'PASS' ? 0 : 2;
