import { expect, test, type APIRequestContext, type Page } from "@playwright/test";

const SESSION_MODEL = "local-openai-proxy/gpt-5.4-mini";
const AGENT_MODEL = "local-openai-proxy/gpt-5.5";
const EXPLICIT_MODEL = "local-openai-proxy/gpt-5.4";

interface ProviderModel {
  id: string;
  enabled?: boolean;
}

interface Provider {
  id: string;
  enabled?: boolean;
  default_model?: string | null;
  models?: ProviderModel[];
}

interface AgentArtifact extends Record<string, unknown> {
  id: string;
  metadata: Record<string, unknown>;
  policy: {
    provider: {
      default: { provider: string; model: string };
      fallbacks: unknown[];
    };
  } & Record<string, unknown>;
}

async function saveSession(
  request: APIRequestContext,
  sessionId: string,
  agentId: string,
  model: string | null,
) {
  const response = await request.post(`/api/uar/sessions/${sessionId}/agent-config`, {
    data: {
      agent_id: agentId,
      model,
      tools: null,
      skills: null,
      knowledge_bases: null,
      mcp_servers: null,
      tool_approval: null,
    },
  });
  expect(response.ok(), await response.text()).toBeTruthy();
}

async function complete(
  request: APIRequestContext,
  sessionId: string,
  marker: string,
  model?: string,
) {
  const response = await request.post("/v1/chat/completions", {
    data: {
      messages: [{ role: "user", content: `Reply with only ${marker}` }],
      session_id: sessionId,
      stream: false,
      memory_enabled: false,
      ...(model ? { model } : {}),
    },
    timeout: 45_000,
  });
  expect(response.ok(), await response.text()).toBeTruthy();
  return response.json() as Promise<{ model: string; choices: Array<{ message: { content: string } }> }>;
}

async function installGraphPublicationCounter(page: Page) {
  await page.evaluate(async () => {
    const href = document.querySelector<HTMLLinkElement>('link[href*="vendor-entities"]')?.href;
    if (!href) throw new Error("installed production bundle has no entity vendor chunk");
    const entityModule = await import(href);
    const graph = Object.values(entityModule).find((candidate) => {
      if (typeof candidate !== "function" && (typeof candidate !== "object" || !candidate)) return false;
      const store = candidate as unknown as {
        getState?: () => { entities?: unknown; lists?: unknown };
        subscribe?: (listener: () => void) => () => void;
      };
      const state = store.getState?.();
      return typeof store.subscribe === "function" && state?.entities && state?.lists;
    }) as unknown as {
      getState: () => { entities: Record<string, Record<string, unknown>> };
      subscribe: (listener: () => void) => () => void;
    } | undefined;
    if (!graph) throw new Error("installed entity graph store export was not found");
    const proof = { count: 0, graph };
    graph.subscribe(() => { proof.count += 1; });
    Object.assign(window, { __uarSessionGraphProof: proof });
  });
}

test("installed Session Configuration is responsive, persistent, bounded, and effective", async ({
  page,
  request,
}, testInfo) => {
  const network: Array<{ method: string; path: string; status?: number }> = [];
  const consoleMessages: Array<{ type: string; text: string }> = [];
  page.on("request", (entry) => {
    const url = new URL(entry.url());
    if (url.origin === "http://127.0.0.1:1906") {
      network.push({ method: entry.method(), path: url.pathname });
    }
  });
  page.on("response", (entry) => {
    const url = new URL(entry.url());
    const record = [...network].reverse().find((candidate) => (
      candidate.method === entry.request().method() && candidate.path === url.pathname
    ));
    if (record) record.status = entry.status();
  });
  page.on("console", (message) => consoleMessages.push({ type: message.type(), text: message.text() }));

  const providersResponse = await request.get("/api/uar/providers");
  expect(providersResponse.ok(), await providersResponse.text()).toBeTruthy();
  const providersPayload = await providersResponse.json() as { providers: Provider[] };
  const providers = providersPayload.providers.filter((provider) => provider.enabled !== false);
  const modelIds = providers.flatMap((provider) => (
    (provider.models ?? [])
      .filter((model) => model.enabled !== false)
      .map((model) => `${provider.id}/${model.id}`)
  ));
  expect(modelIds).toContain(SESSION_MODEL);
  expect(modelIds).toContain(AGENT_MODEL);
  expect(modelIds).toContain(EXPLICIT_MODEL);
  const publicationLimit = providers.length + modelIds.length + 6;

  const agentsResponse = await request.get("/api/agents");
  expect(agentsResponse.ok(), await agentsResponse.text()).toBeTruthy();
  const agentsPayload = await agentsResponse.json() as { runtime_agents: AgentArtifact[] };
  const defaultAgent = agentsPayload.runtime_agents.find((agent) => agent.id === "default-agent");
  expect(defaultAgent).toBeDefined();
  const temporaryAgentId = `session-config-proof-${crypto.randomUUID()}`;
  const temporaryAgent = structuredClone(defaultAgent!);
  temporaryAgent.id = temporaryAgentId;
  temporaryAgent.metadata = { ...temporaryAgent.metadata, title: "Session configuration proof" };
  temporaryAgent.policy = {
    ...temporaryAgent.policy,
    provider: {
      default: { provider: "local-openai-proxy", model: "gpt-5.5" },
      fallbacks: [],
    },
  };
  const createAgent = await request.post("/api/agents", { data: temporaryAgent });
  expect(createAgent.ok(), await createAgent.text()).toBeTruthy();

  try {
    const agentSessionId = crypto.randomUUID();
    await saveSession(request, agentSessionId, temporaryAgentId, null);
    const agentResult = await complete(request, agentSessionId, "AGENT_DEFAULT_OK");
    expect(agentResult.model).toBe(AGENT_MODEL);
    expect(agentResult.choices[0]?.message.content).toContain("AGENT_DEFAULT_OK");

    const explicitResult = await complete(
      request,
      agentSessionId,
      "EXPLICIT_TURN_OK",
      EXPLICIT_MODEL,
    );
    expect(explicitResult.model).toBe(EXPLICIT_MODEL);
    expect(explicitResult.choices[0]?.message.content).toContain("EXPLICIT_TURN_OK");

    await page.goto("/threads", { waitUntil: "domcontentloaded" });
    await expect(page).toHaveURL(/^http:\/\/127\.0\.0\.1:1906\/threads/);
    await page.getByRole("button", { name: "New thread" }).click();
    const configButton = page.getByRole("button", { name: "Session configuration" });
    await expect(configButton).toBeEnabled();
    await installGraphPublicationCounter(page);

    const openedAt = Date.now();
    await configButton.click();
    await expect(page.getByRole("heading", { name: "Session Configuration" })).toBeVisible({
      timeout: 2_000,
    });
    const openMilliseconds = Date.now() - openedAt;
    expect(openMilliseconds).toBeLessThanOrEqual(2_000);
    await expect(page.getByRole("button", { name: "Save Configuration" })).toBeEnabled();

    const graphProof = await page.evaluate(() => {
      const proof = (window as unknown as {
        __uarSessionGraphProof: {
          count: number;
          graph: { getState: () => { entities: Record<string, Record<string, unknown>> } };
        };
      }).__uarSessionGraphProof;
      const entities = proof.graph.getState().entities;
      return {
        publications: proof.count,
        configuredProviders: Object.keys(entities.ConfiguredProvider ?? {}).length,
        configuredModels: Object.keys(entities.ConfiguredModel ?? {}).length,
      };
    });
    expect(graphProof.configuredModels).toBe(modelIds.length);
    expect(graphProof.publications).toBeLessThanOrEqual(publicationLimit);

    const modelSelector = page.getByRole("combobox").first();
    await modelSelector.click();
    await page.getByRole("option").filter({ hasText: "gpt-5.4-mini" }).click();
    const saveResponsePromise = page.waitForResponse((response) => (
      response.request().method() === "POST"
      && /\/api\/uar\/sessions\/[^/]+\/agent-config$/.test(new URL(response.url()).pathname)
    ));
    await page.getByRole("button", { name: "Save Configuration" }).click();
    const saveResponse = await saveResponsePromise;
    expect(saveResponse.ok()).toBeTruthy();
    const sessionMatch = new URL(saveResponse.url()).pathname.match(/\/sessions\/([^/]+)\/agent-config$/);
    expect(sessionMatch).not.toBeNull();
    const uiSessionId = decodeURIComponent(sessionMatch![1]);

    const savedResponse = await request.get(`/api/uar/sessions/${uiSessionId}/agent-config`);
    expect(savedResponse.ok(), await savedResponse.text()).toBeTruthy();
    await expect(savedResponse.json()).resolves.toMatchObject({ model: SESSION_MODEL });

    await configButton.click();
    await expect(modelSelector).toContainText("gpt-5.4-mini");
    await modelSelector.click();
    await page.getByRole("option", { name: /^gpt-5\.4 —/ }).click();
    await page.getByRole("button", { name: "Close" }).click();
    await configButton.click();
    await expect(modelSelector).toContainText("gpt-5.4-mini");

    const spacing = [];
    for (const width of [320, 768, 1024, 1440]) {
      await page.setViewportSize({ width, height: 900 });
      const measured = await page.locator('[data-slot="sheet-content"]').evaluate((sheet) => {
        const body = sheet.querySelector<HTMLElement>(".px-4.pb-4");
        if (!body) throw new Error("Session Configuration padded body not found");
        const style = getComputedStyle(body);
        return {
          sheetWidth: sheet.getBoundingClientRect().width,
          paddingLeft: Number.parseFloat(style.paddingLeft),
          paddingRight: Number.parseFloat(style.paddingRight),
          gap: Number.parseFloat(style.rowGap),
        };
      });
      expect(measured.paddingLeft).toBeGreaterThanOrEqual(16);
      expect(measured.paddingRight).toBeGreaterThanOrEqual(16);
      expect(measured.gap).toBeGreaterThanOrEqual(24);
      expect(measured.sheetWidth).toBeLessThanOrEqual(Math.min(width, 400));
      spacing.push({ width, ...measured });
    }
    await page.getByRole("button", { name: "Close" }).click();

    const input = page.getByRole("textbox", { name: "Message input" });
    await input.fill("Reply with only UI_SESSION_OK");
    const uiRequestPromise = page.waitForRequest((entry) => (
      entry.method() === "POST" && new URL(entry.url()).pathname === "/api/chat/completion"
    ));
    const uiResponsePromise = page.waitForResponse((entry) => (
      entry.request().method() === "POST"
      && new URL(entry.url()).pathname === "/api/chat/completion"
    ));
    await page.getByRole("button", { name: "Send message" }).click();
    const [uiRequest, uiResponse] = await Promise.all([uiRequestPromise, uiResponsePromise]);
    expect(uiResponse.ok()).toBeTruthy();
    expect(await uiRequest.headerValue("x-uar-session-id")).toBe(uiSessionId);
    expect(uiRequest.postDataJSON()).not.toHaveProperty("model");
    await uiResponse.finished();
    await expect(page.locator('[data-role="assistant"]').last()).toContainText("UI_SESSION_OK", {
      timeout: 45_000,
    });

    const sessionResult = await complete(request, uiSessionId, "SESSION_POLICY_OK");
    expect(sessionResult.model).toBe(SESSION_MODEL);
    expect(sessionResult.choices[0]?.message.content).toContain("SESSION_POLICY_OK");

    expect(network.some((entry) => entry.path === "/api/models")).toBeFalsy();
    const isMissingSessionConfiguration = (entry: typeof network[number]) => (
      entry.method === "GET"
      && entry.status === 404
      && /^\/api\/uar\/sessions\/[^/]+\/agent-config$/.test(entry.path)
    );
    const expectedMissingConfigurations = network.filter(isMissingSessionConfiguration);
    const unexpectedHttpFailures = network.filter((entry) => (
      (entry.status ?? 0) >= 400 && !isMissingSessionConfiguration(entry)
    ));
    const consoleErrors = consoleMessages.filter((entry) => entry.type === "error");
    expect(unexpectedHttpFailures).toEqual([]);
    expect(expectedMissingConfigurations.length).toBeGreaterThan(0);
    expect(consoleErrors).toHaveLength(expectedMissingConfigurations.length);
    expect(consoleErrors.every((entry) => entry.text.includes("404"))).toBeTruthy();
    await testInfo.attach("session-configuration-functional-proof", {
      body: JSON.stringify({
        source: "installed server-full release at http://127.0.0.1:1906",
        openMilliseconds,
        publicationLimit,
        graphProof,
        spacing,
        models: {
          explicitTurn: explicitResult.model,
          savedSession: sessionResult.model,
          agentDefault: agentResult.model,
        },
        network,
        consoleMessages,
      }, null, 2),
      contentType: "application/json",
    });
  } finally {
    const deleted = await request.delete(`/api/agents/${temporaryAgentId}`);
    expect(deleted.ok(), await deleted.text()).toBeTruthy();
  }
});
