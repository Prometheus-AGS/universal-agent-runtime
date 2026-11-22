import { APP_BASE_URL, STUB_BASE_URL, STUB_BASE_URL_B, STUB_BASE_URL_C } from './ports';

export async function fetchAllRuntimeAgents(): Promise<Record<string, unknown>[]> {
  const res = await fetch(`${APP_BASE_URL}/api/agents`);
  if (!res.ok) throw new Error(`GET /api/agents failed: ${res.status}`);
  const body = (await res.json()) as { runtime_agents: Record<string, unknown>[] };
  return body.runtime_agents;
}

/** Fetches the first runtime agent from the catalog, used as a clone template for test agents. */
async function fetchTemplateAgent(): Promise<Record<string, unknown>> {
  const agents = await fetchAllRuntimeAgents();
  const template = agents[0];
  if (!template) throw new Error('no runtime agent available to clone as a BDD test-agent template');
  return template;
}

/**
 * Deletes any existing agent whose title matches exactly. The full BDD suite
 * shares one ephemeral SurrealKV store across all 6 scenarios in a run (see
 * playwright.config.ts) — without this, re-running a scenario with the same
 * agent title (or two scenarios sharing a title, e.g. "BDD Agent Aurora" in
 * both chat-agent-switching and chat-model-routing) leaves duplicate
 * same-titled agents behind, and `switchAgentViaUI`'s title-text click
 * becomes ambiguous (picks whichever duplicate sorts first, not necessarily
 * this scenario's own freshly-configured one).
 */
async function deleteAgentsByTitle(title: string): Promise<void> {
  const agents = await fetchAllRuntimeAgents();
  const matches = agents.filter((a) => (a.metadata as Record<string, unknown> | undefined)?.title === title);
  for (const agent of matches) {
    await fetch(`${APP_BASE_URL}/api/agents/${agent.id}`, { method: 'DELETE' });
  }
}

export interface TestAgentOverrides {
  title: string;
  model?: string; // "{provider}/{model}", overrides policy.provider.default
  knowledgeBases?: string[];
  skillIds?: string[];
  tools?: string[];
}

/**
 * Creates a fresh test agent by cloning the first catalog agent and applying
 * overrides, via the real `POST /api/agents` endpoint — avoids depending on
 * whatever knowledge bases / skills the ambient default agent happens to have.
 */
export async function createTestAgent(overrides: TestAgentOverrides): Promise<{ id: string; model: string }> {
  await deleteAgentsByTitle(overrides.title);
  const template = structuredClone(await fetchTemplateAgent());
  const id = crypto.randomUUID();
  template.id = id;
  (template.metadata as Record<string, unknown>).title = overrides.title;

  const policy = template.policy as Record<string, unknown>;
  const provider = (policy.provider as Record<string, unknown>).default as Record<string, unknown>;
  // Always set explicitly — the cloned template's own default (e.g. the
  // seeded default-agent's "gpt-5.2") isn't a model the stub-llm fixtures
  // are keyed on or that this test env's single registered custom provider
  // recognizes, so an agent that didn't ask for a specific model would
  // otherwise get a real "model_not_found" error instead of ever reaching
  // the stub.
  const [providerId, modelId] = (overrides.model ?? 'openai/gpt-5.4-mini').split('/');
  provider.provider = providerId;
  provider.model = modelId;
  policy.skills = { ...(policy.skills as object), prefer: overrides.skillIds ?? [] };
  policy.tools = { ...(policy.tools as object), allow: overrides.tools ?? [] };

  const memory = template.memory as Record<string, unknown>;
  (memory.kb as Record<string, unknown>).enabled = (overrides.knowledgeBases?.length ?? 0) > 0;
  (memory.kb as Record<string, unknown>).knowledge_bases = overrides.knowledgeBases ?? [];

  const res = await fetch(`${APP_BASE_URL}/api/agents`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(template),
  });
  if (!res.ok) throw new Error(`POST /api/agents failed: ${res.status} ${await res.text()}`);
  const created = (await res.json()) as Record<string, unknown>;
  const createdProvider = ((created.policy as Record<string, unknown>).provider as Record<string, unknown>)
    .default as Record<string, unknown>;
  return { id, model: `${createdProvider.provider}/${createdProvider.model}` };
}

/** Deletes an existing knowledge base by exact name, if one exists (name uniqueness is enforced server-side). */
async function deleteKnowledgeBaseByName(name: string): Promise<void> {
  const res = await fetch(`${APP_BASE_URL}/api/knowledge`);
  if (!res.ok) return;
  const list = (await res.json()) as Array<{ id: string; name: string }>;
  const match = list.find((kb) => kb.name === name);
  if (match) await fetch(`${APP_BASE_URL}/api/knowledge/${match.id}`, { method: 'DELETE' });
}

/**
 * Registers a second custom LLM provider pointed at the same stub-llm base
 * URL, distinct from the one custom provider `tests/bdd/playwright.config.ts`
 * registers via `UAR_LLM__BASE_URL`/`UAR_LLM__MODEL` env vars (which only
 * recognizes one model id, `openai/gpt-5.4-mini` — any other model string
 * under that same provider id fails real model-catalog validation with
 * "model_not_found", confirmed empirically). Registering a distinct provider
 * id is how scenario 6 (provider/model routing) gets a second, independently
 * addressable model without touching the env-var-configured default.
 * Idempotent — ignores 409 (already registered) from a prior scenario run
 * sharing this suite invocation's single ephemeral server.
 */
export async function ensureSecondTestProvider(): Promise<void> {
  const res = await fetch(`${APP_BASE_URL}/api/uar/providers`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      id: 'bdd-provider-b',
      display_name: 'BDD Test Provider B',
      base_url: `${STUB_BASE_URL}/v1`,
      default_model: 'claude-5-haiku',
      models: [{ id: 'claude-5-haiku' }],
      enabled: true,
    }),
  });
  if (!res.ok && res.status !== 409) {
    throw new Error(`POST /api/uar/providers failed: ${res.status} ${await res.text()}`);
  }
}

/** Creates a knowledge base and uploads a single text document containing `content`. */
export async function createKnowledgeBaseWithDocument(name: string, content: string): Promise<string> {
  await deleteKnowledgeBaseByName(name);
  const kbRes = await fetch(`${APP_BASE_URL}/api/knowledge`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ name, description: `BDD fixture KB: ${name}` }),
  });
  if (!kbRes.ok) throw new Error(`POST /api/knowledge failed: ${kbRes.status} ${await kbRes.text()}`);
  const kb = (await kbRes.json()) as { id: string };

  const form = new FormData();
  form.append('file', new Blob([content], { type: 'text/plain' }), 'bdd-fixture.txt');
  const uploadRes = await fetch(`${APP_BASE_URL}/api/knowledge/${kb.id}/documents`, {
    method: 'POST',
    body: form,
  });
  if (!uploadRes.ok) throw new Error(`document upload failed: ${uploadRes.status} ${await uploadRes.text()}`);
  const doc = (await uploadRes.json()) as { id: string };

  // Ingestion is async (submitted to a worker pool, 202 Accepted) — poll
  // until it reaches a terminal status before returning, so callers don't
  // race the query against not-yet-indexed content.
  const deadline = Date.now() + 30_000;
  for (;;) {
    const statusRes = await fetch(`${APP_BASE_URL}/api/knowledge/${kb.id}/documents/${doc.id}`);
    if (statusRes.ok) {
      const status = (await statusRes.json()) as { status: string; error_message?: string | null };
      const s = status.status.toLowerCase();
      if (s === 'indexed') break;
      if (s === 'failed') {
        throw new Error(`document ingestion failed: ${status.error_message ?? '(no error_message)'}`);
      }
    }
    if (Date.now() > deadline) throw new Error(`document ingestion did not reach a terminal status within 30s`);
    await new Promise((resolve) => setTimeout(resolve, 250));
  }

  return kb.id;
}

/** Creates a skill with a keyword trigger, prompt overlay only (no mcp_config, no preferred_model). */
export async function createTestSkill(opts: {
  name: string;
  keyword: string;
  promptOverlay: string;
}): Promise<string> {
  const res = await fetch(`${APP_BASE_URL}/api/skills`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      name: opts.name,
      description: `BDD fixture skill: ${opts.name}`,
      triggers: { keywords: [opts.keyword], semantic: null },
      prompt_overlay: opts.promptOverlay,
      enabled: true,
    }),
  });
  if (!res.ok) throw new Error(`POST /api/skills failed: ${res.status} ${await res.text()}`);
  const skill = (await res.json()) as { skill_id: string };
  return skill.skill_id;
}

/** Binds a skill to an agent via PUT /api/agents/{agentId}/skills. */
export async function bindSkillToAgent(agentId: string, skillId: string): Promise<void> {
  const res = await fetch(`${APP_BASE_URL}/api/agents/${agentId}/skills`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ skill_ids: [skillId] }),
  });
  if (!res.ok) throw new Error(`PUT /api/agents/${agentId}/skills failed: ${res.status} ${await res.text()}`);
}


// ---------------------------------------------------------------------------
// S1 live model choice (phase uar-uiux-architecture-2026-11). Tests send no
// credentials: they run against the deployment's auth-off test configuration.
// ---------------------------------------------------------------------------

/** One synthetic stub provider of the S1 vocabulary (`fixtures/s1-oracle.json` `providers`). */
export interface S1ProviderSpec {
  id: string;
  displayName: string;
  /** Base URL of the provider's own stub instance. */
  baseUrl: string;
  models: string[];
  defaultModel: string;
  enabled: boolean;
}

/** `bdd-provider-b` is reconfigured over the one `ensureSecondTestProvider` creates (default_model claude-5-haiku). */
export const S1_PROVIDER_SPECS: readonly S1ProviderSpec[] = [
  {
    id: 'bdd-provider-b',
    displayName: 'BDD Test Provider B',
    baseUrl: `${STUB_BASE_URL_B}/v1`,
    models: ['claude-5-sonnet', 'claude-5-haiku'],
    defaultModel: 'claude-5-sonnet',
    enabled: true,
  },
  {
    id: 'bdd-provider-c',
    displayName: 'BDD Test Provider C',
    baseUrl: `${STUB_BASE_URL_C}/v1`,
    models: ['qwen-5-turbo'],
    defaultModel: 'qwen-5-turbo',
    enabled: true,
  },
  {
    // Disabled, so it is never routed to; it shares C's stub, which must log nothing for it.
    id: 'bdd-provider-off',
    displayName: 'BDD Test Provider Off',
    baseUrl: `${STUB_BASE_URL_C}/v1`,
    models: ['off-model-1'],
    defaultModel: 'off-model-1',
    enabled: false,
  },
];

/** Splits a `provider/model` pair at the first slash; model ids may themselves contain slashes. */
export function splitQualifiedModel(qualified: string): { providerId: string; modelId: string } {
  const at = qualified.indexOf('/');
  if (at <= 0 || at === qualified.length - 1) {
    throw new Error(`expected a provider-qualified model "provider/model", got "${qualified}"`);
  }
  return { providerId: qualified.slice(0, at), modelId: qualified.slice(at + 1) };
}

export interface RawResponse {
  status: number;
  text: string;
}

/** A request straight to the backend, with no credentials, returning status and body text. */
export async function directRequest(method: string, path: string, body?: unknown): Promise<RawResponse> {
  const res = await fetch(`${APP_BASE_URL}${path}`, {
    method,
    headers: body === undefined ? undefined : { 'Content-Type': 'application/json' },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  return { status: res.status, text: await res.text() };
}

interface ProvidersRead {
  providers: Array<Record<string, unknown>>;
  default_id?: string;
}

async function readProviders(): Promise<ProvidersRead> {
  const res = await fetch(`${APP_BASE_URL}/api/uar/providers`);
  if (!res.ok) throw new Error(`GET /api/uar/providers failed: ${res.status}`);
  return (await res.json()) as ProvidersRead;
}

/** The registry's default provider id, read from `GET /api/uar/providers` (not from the UI). */
export async function readRegistryDefaultId(): Promise<string | null> {
  return (await readProviders()).default_id ?? null;
}

/**
 * Replaces a provider's configuration by merging `overrides` over its current
 * read view. The credential field is dropped from the merged body so the
 * server keeps the stored key instead of overwriting it with a masked value.
 */
async function putProvider(id: string, overrides: Record<string, unknown>): Promise<void> {
  const existing = (await readProviders()).providers.find((provider) => provider.id === id);
  if (!existing) throw new Error(`provider "${id}" is not registered`);
  const body: Record<string, unknown> = { ...existing, ...overrides };
  delete body.api_key;
  const res = await fetch(`${APP_BASE_URL}/api/uar/providers/${id}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });
  if (!res.ok) throw new Error(`PUT /api/uar/providers/${id} failed: ${res.status} ${await res.text()}`);
}

/**
 * A synthetic model entry. The verification probe needs a known context window
 * above its output allowance (the synthetic ids are not in the catalog).
 */
function s1ModelConfig(id: string): Record<string, unknown> {
  return { id, enabled: true, context_window: 128_000, max_output_tokens: 8_192 };
}

/** Creates (idempotent, 409 ignored) and then reconfigures one synthetic S1 provider. */
export async function configureS1Provider(spec: S1ProviderSpec): Promise<void> {
  const created = await fetch(`${APP_BASE_URL}/api/uar/providers`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      id: spec.id,
      display_name: spec.displayName,
      base_url: spec.baseUrl,
      default_model: spec.defaultModel,
      models: spec.models.map(s1ModelConfig),
      enabled: spec.enabled,
    }),
  });
  if (!created.ok && created.status !== 409) {
    throw new Error(`POST /api/uar/providers (${spec.id}) failed: ${created.status} ${await created.text()}`);
  }
  await putProvider(spec.id, {
    display_name: spec.displayName,
    base_url: spec.baseUrl,
    default_model: spec.defaultModel,
    models: spec.models.map(s1ModelConfig),
    enabled: spec.enabled,
  });
}

/**
 * The S1 providers: `openai` (stub A, registered through the app's env) with
 * default model gpt-5.4-mini, `bdd-provider-b` (models claude-5-sonnet and
 * claude-5-haiku, default claude-5-sonnet), `bdd-provider-c` and the disabled
 * `bdd-provider-off`, each pointed at its own stub instance.
 */
export async function configureS1Providers(): Promise<void> {
  await putProvider('openai', {
    default_model: 'gpt-5.4-mini',
    enabled: true,
    models: await openAiModelsWithS1Model(),
  });
  for (const spec of S1_PROVIDER_SPECS) await configureS1Provider(spec);
  await verifyS1Models();
}

/** openai's current models with `gpt-5.4-mini` present, enabled and of known capacity. */
async function openAiModelsWithS1Model(): Promise<Array<Record<string, unknown>>> {
  const existing = (await readProviders()).providers.find((provider) => provider.id === 'openai');
  const models = ((existing?.models as Array<Record<string, unknown>> | undefined) ?? []).slice();
  const index = models.findIndex((model) => model.id === 'gpt-5.4-mini');
  if (index < 0) return [...models, s1ModelConfig('gpt-5.4-mini')];
  const current = models[index];
  models[index] = {
    ...current,
    enabled: true,
    context_window: current.context_window ?? 128_000,
    max_output_tokens: current.max_output_tokens ?? 8_192,
  };
  return models;
}

/**
 * Every model the S1 scenarios serve. Turn admission and override saves require
 * each to be listed, enabled and carry a current verification proof. Models the
 * scenarios expect to be refused (`bdd-provider-off/off-model-1`,
 * `bdd-provider-b/claude-5-retired`) are deliberately absent. `openai/gpt-5.4-mini`
 * is the configured fallback of SCN-SYS-012 and must stay verified.
 */
export const S1_VERIFIED_MODELS = [
  'openai/gpt-5.4-mini',
  'bdd-provider-b/claude-5-sonnet',
  'bdd-provider-b/claude-5-haiku',
  'bdd-provider-c/qwen-5-turbo',
] as const;

/**
 * Produces the verification proof through the API the console uses
 * (`POST /api/uar/providers/{id}/test`, a real one-token probe against the
 * provider's own stub). A proof binds the provider's current configuration, so
 * this runs after every write the setup makes. A later write to anything but
 * default_model makes the proof stale.
 */
export async function verifyModel(providerId: string, modelId: string): Promise<void> {
  const res = await fetch(`${APP_BASE_URL}/api/uar/providers/${providerId}/test`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ model: modelId }),
  });
  if (!res.ok) {
    throw new Error(`POST /api/uar/providers/${providerId}/test (${modelId}) failed: ${res.status} ${await res.text()}`);
  }
}

export async function verifyS1Models(): Promise<void> {
  for (const qualified of S1_VERIFIED_MODELS) {
    const { providerId, modelId } = splitQualifiedModel(qualified);
    await verifyModel(providerId, modelId);
  }
}

export async function setProviderEnabled(providerId: string, enabled: boolean): Promise<void> {
  await putProvider(providerId, { enabled });
}

/**
 * Sets the system default through the existing provider routes, as the
 * baseline-runnable harness step `the system default model is {string}`: the
 * provider's default_model first, then the live default route. Returns the
 * harness-clock window (ms) around the two writes.
 */
export async function setSystemDefaultViaApi(
  providerId: string,
  modelId: string,
): Promise<{ t0: number; t1: number }> {
  const t0 = Date.now();
  // Write only when the default model really changes. A default_model write
  // must not stale the provider's verification proofs.
  const current = (await readProviders()).providers.find((provider) => provider.id === providerId);
  if (current?.default_model !== modelId) await putProvider(providerId, { default_model: modelId });
  const res = await fetch(`${APP_BASE_URL}/api/uar/providers/${providerId}/default`, { method: 'POST' });
  if (!res.ok) throw new Error(`POST /api/uar/providers/${providerId}/default failed: ${res.status} ${await res.text()}`);
  return { t0, t1: Date.now() };
}

/**
 * Puts the shared server back to what the other features expect: every S1
 * provider enabled except the disabled one, `bdd-provider-b` as
 * `ensureSecondTestProvider` creates it (on stub A, model claude-5-haiku), and
 * `openai` as the system default.
 */
export async function restoreS1Baseline(): Promise<void> {
  await putProvider('openai', { default_model: 'gpt-5.4-mini', enabled: true });
  await putProvider('bdd-provider-b', {
    base_url: `${STUB_BASE_URL}/v1`,
    default_model: 'claude-5-haiku',
    models: [{ id: 'claude-5-haiku', enabled: true }],
    enabled: true,
  });
  await putProvider('bdd-provider-c', { enabled: true });
  const res = await fetch(`${APP_BASE_URL}/api/uar/providers/openai/default`, { method: 'POST' });
  if (!res.ok) throw new Error(`POST /api/uar/providers/openai/default failed: ${res.status} ${await res.text()}`);
}

/** The stored `llm.default_provider` row (`saved`), read through the settings API. */
export async function readStoredDefaultProvider(): Promise<string | null> {
  const res = await fetch(`${APP_BASE_URL}/api/uar/settings/llm.default_provider`);
  if (!res.ok) throw new Error(`GET /api/uar/settings/llm.default_provider failed: ${res.status}`);
  const body = (await res.json()) as { saved?: unknown; data?: unknown };
  const saved = body.saved ?? body.data;
  if (typeof saved === 'string') return saved;
  if (saved && typeof saved === 'object' && 'value' in saved) {
    const value = (saved as { value: unknown }).value;
    return typeof value === 'string' ? value : null;
  }
  return null;
}

/** The global run policy (`GET /api/uar/settings/presentation-policy` -> `policy`). */
export async function readGlobalPolicy(): Promise<Record<string, unknown>> {
  const res = await fetch(`${APP_BASE_URL}/api/uar/settings/presentation-policy`);
  if (!res.ok) throw new Error(`GET /api/uar/settings/presentation-policy failed: ${res.status}`);
  return ((await res.json()) as { policy: Record<string, unknown> }).policy;
}

export function agentTitleOf(agent: Record<string, unknown>): string {
  return String((agent.metadata as Record<string, unknown> | undefined)?.title ?? '');
}

/**
 * The model an agent carries of its own, as `provider/model`, or null. Reads the
 * backend's derived `explicit_model` when the agents read exposes it, otherwise
 * the runtime's own rule: a non-empty provider and model on the artifact's
 * `policy.provider.default`, or a model in `extensions["uar.run_policy"]`.
 */
export function agentExplicitModel(agent: Record<string, unknown>): string | null {
  if ('explicit_model' in agent) {
    const projected = agent.explicit_model;
    return typeof projected === 'string' && projected !== '' ? projected : null;
  }
  const extensions = (agent.extensions as Record<string, unknown> | undefined) ?? {};
  const extended = extensions['uar.run_policy'] as { model?: { provider_id?: string; model_id?: string } } | undefined;
  if (extended?.model?.provider_id && extended.model.model_id) {
    return `${extended.model.provider_id}/${extended.model.model_id}`;
  }
  const selection = ((agent.policy as Record<string, unknown> | undefined)?.provider as
    | { default?: { provider?: string; model?: string } }
    | undefined)?.default;
  if (selection?.provider?.trim() && selection.model?.trim()) return `${selection.provider}/${selection.model}`;
  return null;
}

export interface S1AgentSpec {
  title: string;
  /** `provider/model` on the artifact's `policy.provider.default`; omit for an agent with no model. */
  model?: string;
  /** `provider/model` carried only in `extensions["uar.run_policy"]` (the artifact itself has none). */
  extensionModel?: string;
}

/**
 * Creates an agent for the S1 pinned-agent scenarios by cloning the first
 * catalog agent: with `model` the artifact names it, with `extensionModel` only
 * the `uar.run_policy` extension does, with neither the agent has no model of
 * its own (empty provider and model on `policy.provider.default`).
 */
export async function createS1Agent(spec: S1AgentSpec): Promise<void> {
  await deleteAgentsByTitle(spec.title);
  const template = structuredClone(await fetchTemplateAgent());
  template.id = crypto.randomUUID();
  (template.metadata as Record<string, unknown>).title = spec.title;

  const policy = template.policy as Record<string, unknown>;
  const selection = (policy.provider as Record<string, unknown>).default as Record<string, unknown>;
  if (spec.model) {
    const { providerId, modelId } = splitQualifiedModel(spec.model);
    selection.provider = providerId;
    selection.model = modelId;
  } else {
    selection.provider = '';
    selection.model = '';
  }
  policy.skills = { ...(policy.skills as object), prefer: [] };
  policy.tools = { ...(policy.tools as object), allow: [] };

  const memory = template.memory as Record<string, unknown>;
  (memory.kb as Record<string, unknown>).enabled = false;
  (memory.kb as Record<string, unknown>).knowledge_bases = [];

  const extensions = { ...((template.extensions as Record<string, unknown> | undefined) ?? {}) };
  delete extensions['uar.run_policy'];
  if (spec.extensionModel) {
    const { providerId, modelId } = splitQualifiedModel(spec.extensionModel);
    extensions['uar.run_policy'] = { version: 1, model: { provider_id: providerId, model_id: modelId } };
  }
  template.extensions = extensions;

  const res = await fetch(`${APP_BASE_URL}/api/agents`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(template),
  });
  if (!res.ok) throw new Error(`POST /api/agents (${spec.title}) failed: ${res.status} ${await res.text()}`);
}

/**
 * Deletes every agent whose title starts with `BDD Agent`: residue of the other
 * features sharing this server. They all carry an explicit model, so left in
 * place they would be listed as pinned and defeat the exact-list assertion.
 */
export async function purgeBddAgents(): Promise<void> {
  const agents = await fetchAllRuntimeAgents();
  for (const agent of agents.filter((candidate) => agentTitleOf(candidate).startsWith('BDD Agent'))) {
    const res = await fetch(`${APP_BASE_URL}/api/agents/${agent.id}`, { method: 'DELETE' });
    if (!res.ok) throw new Error(`DELETE /api/agents/${agent.id} failed: ${res.status}`);
  }
}
