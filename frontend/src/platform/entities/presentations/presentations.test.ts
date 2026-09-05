import { afterEach, describe, expect, test, vi } from "vitest";
import { parsePresentationSource, STARTER_PRESENTATION_SOURCE, type Presentation, type PresentationContent } from "./contracts";
import { deletePresentation, fetchPresentations, savePresentation } from "./api/presentations-api";

function template(overrides: Record<string, unknown> = {}) {
  return { ...JSON.parse(STARTER_PRESENTATION_SOURCE) as Record<string, unknown>, ...overrides };
}
function content(): PresentationContent {
  const parsed = parsePresentationSource(STARTER_PRESENTATION_SOURCE);
  if (!parsed.template) throw new Error(parsed.error);
  return { title: "Account summary", description: "A reusable summary", enabled: true, template: parsed.template };
}
function record(overrides: Partial<Presentation> = {}): Presentation {
  return { id: "template-one", owner_id: "owner-one", revision: 1, content: content(),
    created_at: "2026-09-05T00:00:00Z", updated_at: "2026-09-05T00:00:00Z", ...overrides };
}
afterEach(() => vi.unstubAllGlobals());

describe("Presentation authoring boundary", () => {
  test("accepts the production starter template", () => {
    expect(parsePresentationSource(STARTER_PRESENTATION_SOURCE)).toMatchObject({ error: null, template: { components: [{ id: "root" }] } });
  });
  test("reports malformed JSON without producing a preview", () => {
    expect(parsePresentationSource("{" )).toMatchObject({ template: null, error: expect.any(String) });
  });
  test.each([
    { version: "unknown" },
    { catalog_id: "https://untrusted.invalid/catalog" },
    { components: [] },
    { components: [{ id: "not-root", component: "Text", text: "Missing root" }] },
    { components: [{ id: "root", component: "Text", text: "One" }, { id: "root", component: "Text", text: "Duplicate" }] },
    { components: [{ id: "root", component: "Text", text: "One" }, { id: "orphan", component: "Text", text: "Unreachable" }] },
    { components: [{ id: "root", component: "Column", children: ["missing"] }] },
    { components: [{ id: "root", component: "Column", children: ["root"] }] },
    { components: [{ id: "root", component: "Column", children: ["child", "child"] }, { id: "child", component: "Text", text: "Shared" }] },
    { components: [{ id: "root", component: "Text", text: { path: "/constructor/value" } }] },
    { default_data: { ["x".repeat(513)]: "oversized pointer" } },
  ])("rejects invalid or non-renderable source %j", (invalid) => {
    expect(parsePresentationSource(JSON.stringify(template(invalid)))).toMatchObject({ template: null, error: expect.any(String) });
  });
  test("rejects prototype data keys even when nested", () => {
    const invalid: unknown = JSON.parse('{"nested":{"__proto__":{"polluted":true}}}');
    expect(parsePresentationSource(JSON.stringify(template({ default_data: invalid })))).toMatchObject({ template: null, error: expect.stringContaining("Prototype") });
  });
  test("accepts literal text and escaped data-key characters as data", () => {
    const source = template({ default_data: { "a/b~c": "Literal ${message}; no interpolation" } });
    expect(parsePresentationSource(JSON.stringify(source))).toMatchObject({ error: null, template: { default_data: source.default_data } });
  });
  test("preserves the protocol prohibition on executable markup", () => {
    const source = template({ default_data: { message: "<script>alert(1)</script>" } });
    expect(parsePresentationSource(JSON.stringify(source))).toMatchObject({ template: null, error: "Executable HTML or JavaScript is not allowed in A2UI data" });
  });
});

describe("Presentation catalog transport", () => {
  test("loads a verified owner catalog without permitting cached reads", async () => {
    const fetch = vi.fn().mockResolvedValue(Response.json({ owner_id: "owner-one", presentations: [record()] }));
    vi.stubGlobal("fetch", fetch);
    expect(await fetchPresentations()).toMatchObject({ owner_id: "owner-one", presentations: [{ id: "template-one" }] });
    expect(fetch).toHaveBeenCalledWith("/api/uar/presentations", expect.objectContaining({ method: "GET", cache: "no-store" }));
  });
  test("rejects cross-owner catalog rows as a whole", async () => {
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue(Response.json({ owner_id: "owner-one", presentations: [record({ owner_id: "owner-two" })] })));
    await expect(fetchPresentations()).rejects.toMatchObject({ status: 403 });
  });
  test("sends revision-checked updates and preserves exact IDs in the URL", async () => {
    const fetch = vi.fn().mockResolvedValue(Response.json(record({ revision: 2 })));
    vi.stubGlobal("fetch", fetch);
    const draft = content();
    await expect(savePresentation("template/one", 1, draft)).resolves.toMatchObject({ revision: 2 });
    expect(fetch).toHaveBeenCalledWith("/api/uar/presentations/template%2Fone", expect.objectContaining({ method: "PUT", body: JSON.stringify({ expected_revision: 1, content: draft }) }));
  });
  test("creates without a client-selected owner or revision", async () => {
    const fetch = vi.fn().mockResolvedValue(Response.json(record()));
    vi.stubGlobal("fetch", fetch);
    const draft = content();
    await savePresentation(null, null, draft);
    expect(fetch).toHaveBeenCalledWith("/api/uar/presentations", expect.objectContaining({ method: "POST", body: JSON.stringify(draft) }));
  });
  test("delete carries the loaded revision", async () => {
    const fetch = vi.fn().mockResolvedValue(new Response(null, { status: 204 }));
    vi.stubGlobal("fetch", fetch);
    await deletePresentation("template/one", 7);
    expect(fetch).toHaveBeenCalledWith("/api/uar/presentations/template%2Fone?expected_revision=7", expect.objectContaining({ method: "DELETE" }));
  });
  test.each([401, 403, 404, 409, 422])("surfaces a confirmed %s save rejection without pretending success", async (status) => {
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue(Response.json({ error: "Rejected template" }, { status })));
    await expect(savePresentation("template-one", 1, content())).rejects.toMatchObject({ status, uncertain: false });
  });
  test("a disconnected write has an unknown result, not a safe automatic retry", async () => {
    vi.stubGlobal("fetch", vi.fn().mockRejectedValue(new Error("disconnected")));
    await expect(savePresentation("template-one", 1, content())).rejects.toMatchObject({ status: 0, uncertain: true });
  });
  test("unreadable save confirmation remains uncertain", async () => {
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue(Response.json({ unexpected: true })));
    await expect(savePresentation("template-one", 1, content())).rejects.toMatchObject({ status: 200, uncertain: true });
  });
});
