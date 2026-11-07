import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, test, vi } from "vitest";
import { graphStore } from "@prometheus-ags/prometheus-entity-management";
import { A2uiChunkView } from "@/features/chat/ui/chunks/a2ui-chunk";
import { toChunks } from "@/shared/content/to-chunks";
import { registerA2uiActionEntities } from "@/platform/entities/a2ui-actions";
import { bindA2uiActionPersistence } from "@/platform/entities/a2ui-actions/durability";
import { latestActionReceipt } from "@/platform/entities/a2ui-actions/domain";
import { A2UI_ACTION_ENTITY, A2UI_AUTHORITY_ENTITY, type A2uiActionReceipt } from "@/platform/entities/a2ui-actions/contracts";

import { A2UI_CATALOG_ID, A2UI_PROFILE, A2UI_VERSION } from "@/features/a2ui/a2ui-protocol";
import { MAX_A2UI_SOURCE_BYTES } from "@/features/a2ui/a2ui-rendering-limits";

import { A2uiDisplayBlock } from "./a2ui-artifact-block";

function policySurface(component = "Text") {
  return JSON.stringify([
    {
      version: A2UI_VERSION,
      createSurface: { surfaceId: "policy", catalogId: A2UI_CATALOG_ID },
    },
    {
      version: A2UI_VERSION,
      updateComponents: {
        surfaceId: "policy",
        components: [
          { id: "heading", component: "Text", text: { path: "/title" }, variant: "h2" },
          { id: "summary", component, text: { path: "/summary" }, variant: "body" },
          { id: "root", component: "Column", children: ["heading", "summary"] },
        ],
      },
    },
    {
      version: A2UI_VERSION,
      updateDataModel: {
        surfaceId: "policy",
        path: "/",
        value: {
          title: "Effective run policy",
          summary: "Tools · all · 12 available",
        },
      },
    },
  ]);
}

describe("A2uiDisplayBlock", () => {
  test("renders current-production messages through the canonical UAR surface", () => {
    const source = policySurface();
    render(
      <A2uiDisplayBlock
        artifactType="effective_run_policy"
        title="Effective run policy"
        content={source}
        language="a2ui"
        profile={A2UI_PROFILE}
      />,
    );

    expect(screen.getByRole("heading", { name: "Effective run policy" })).toBeInTheDocument();
    expect(screen.getByText("Tools · all · 12 available")).toBeInTheDocument();
    expect(screen.getByText(`A2UI ${A2UI_VERSION} · rendered · effective_run_policy`)).toBeInTheDocument();
    expect(screen.queryByText(source)).not.toBeInTheDocument();
  });

  test("fails closed and keeps malformed source behind a disclosure", () => {
    render(
      <A2uiDisplayBlock
        artifactType="effective_run_policy"
        title="Effective run policy"
        content='{"effective":true}'
        profile={A2UI_PROFILE}
      />,
    );

    expect(screen.getByRole("alert")).toHaveTextContent("could not be rendered");
    expect(screen.getByText("View A2UI source")).toBeInTheDocument();
    expect(screen.getByLabelText("Effective run policy A2UI source")).toHaveTextContent('{"effective":true}');
  });

  test("rejects components outside the certified UAR catalog", () => {
    render(
      <A2uiDisplayBlock
        artifactType="effective_run_policy"
        title="Effective run policy"
        content={policySurface("Image")}
        profile={A2UI_PROFILE}
      />,
    );

    expect(screen.getByRole("alert")).toHaveTextContent("Unapproved A2UI component: Image");
  });

  test("requires the declared UAR profile instead of fabricating one", () => {
    render(
      <A2uiDisplayBlock
        artifactType="effective_run_policy"
        title="Effective run policy"
        content={policySurface()}
      />,
    );

    expect(screen.getByRole("alert")).toHaveTextContent("missing its required UAR A2UI profile");
  });

  test("rejects artifacts that exceed the synchronous rendering budget", () => {
    render(
      <A2uiDisplayBlock
        artifactType="effective_run_policy"
        title="Effective run policy"
        content={"x".repeat(256 * 1024 + 1)}
        profile={A2UI_PROFILE}
      />,
    );

    expect(screen.getByRole("alert")).toHaveTextContent("exceeds the 256 KiB rendering limit");
  });

  test("bounds oversized source disclosure before profile validation", () => {
    render(
      <A2uiDisplayBlock
        artifactType="effective_run_policy"
        title="Effective run policy"
        content={`untrusted-prefix-${"x".repeat(MAX_A2UI_SOURCE_BYTES + 1)}`}
        validation="invalid"
        validationError="Rejected upstream"
      />,
    );

    const disclosure = screen.getByLabelText("Effective run policy A2UI source");
    expect(disclosure).toHaveTextContent("untrusted-prefix-");
    expect(disclosure).toHaveTextContent("A2UI source truncated at 256 KiB");
    expect(new TextEncoder().encode(disclosure.textContent ?? "").byteLength)
      .toBeLessThanOrEqual(MAX_A2UI_SOURCE_BYTES);
  });

  test("applies the message ceiling to JSONL as well as JSON arrays", () => {
    const jsonl = Array.from({ length: 129 }, (_, index) => JSON.stringify({
      version: A2UI_VERSION,
      deleteSurface: { surfaceId: `surface-${index}` },
    })).join("\n");
    render(
      <A2uiDisplayBlock
        artifactType="effective_run_policy"
        title="Effective run policy"
        content={jsonl}
        profile={A2UI_PROFILE}
      />,
    );

    expect(screen.getByRole("alert")).toHaveTextContent("exceeds the 128-message rendering limit");
  });
});


const continuationHarness = vi.hoisted(() => ({ attach: vi.fn(async () => {}) }));
vi.mock("@/stores/chat-stream-store", () => ({ useChatStreamStore: { getState: () => ({ attachContinuation: continuationHarness.attach }) } }));
afterEach(() => { vi.unstubAllGlobals(); bindA2uiActionPersistence(null); });
let publicationSerial = 0;
function publishedButton() {
  return JSON.stringify([
    { version: A2UI_VERSION, createSurface: { surfaceId: "parcel", catalogId: A2UI_CATALOG_ID } },
    { version: A2UI_VERSION, updateComponents: { surfaceId: "parcel", components: [
      { id: "label", component: "Text", text: "Calculate parcel" },
      { id: "submit", component: "Button", child: "label", action: { event: { name: "calculate" } } },
      { id: "root", component: "Column", children: ["submit"] },
    ] } },
  ]);
}
function restoredPublication(runId: string, generation: unknown) {
  const chunks = toChunks([{ type: "artifact", id: "published-artifact", kind: "application/vnd.uar.a2ui+json",
    content: JSON.stringify({ profile: A2UI_PROFILE, component: "a2ui", validation: "valid",
      payload: { content: publishedButton(), metadata: { source_run_id: runId, surface_generations: { parcel: generation } } } }) }],
  { messageId: `message-${runId}`, at: "2026-11-03T00:00:00Z", runId: "different-wire-run", finalized: true });
  const chunk = chunks[0];
  if (chunk.kind !== "a2ui-display") throw new Error("Persisted publication did not restore an A2UI display");
  return chunk;
}
function publicationServer(runId: string, latestGeneration = "G1", loseAcknowledgment = false) {
  registerA2uiActionEntities();
  bindA2uiActionPersistence(async () => {});
  const authority = { id: runId, status: "complete", authoritative: true, recovery_hint: "",
    surfaces: [{ surface_id: "parcel", surface_generation: "G1", source_run_id: runId }] };
  graphStore.getState().replaceEntity(A2UI_AUTHORITY_ENTITY, runId, authority);
  let reads = 0;
  const request = vi.fn<typeof fetch>(async (url, init) => {
    if (init?.method === "POST" && loseAcknowledgment) throw new Error("Action acknowledgment was lost");
    if (init?.method === "POST") return new Response(JSON.stringify({ run_id: runId, continuation_run_id: `${runId}-child`,
      status: "continued", retry_protected: true, interaction_id: JSON.parse(String(init.body)).interaction_id }), { status: 202 });
    if (String(url).endsWith("surface-authority")) return new Response(JSON.stringify({ ...authority,
      surfaces: [{ ...authority.surfaces[0], surface_generation: ++reads === 1 ? "G1" : latestGeneration }] }));
    return new Response(JSON.stringify({ run_id: runId, conversation_id: "owned-thread", status: "done" }));
  });
  vi.stubGlobal("fetch", request);
  return request;
}

describe("publication identity through restored chunks and canonical controls", () => {
  test("the rendered generation and native source reach the immutable POST unchanged", async () => {
    const runId = `rendered-publication-${++publicationSerial}`;
    const request = publicationServer(runId);
    render(<A2uiChunkView chunk={restoredPublication(runId, "G1")} />);
    const button = await screen.findByRole("button", { name: "Calculate parcel" });
    await waitFor(() => expect(button).toBeEnabled());
    fireEvent.click(button);
    await screen.findByText("Response complete", { exact: true });
    const posts = request.mock.calls.filter((call) => call[1]?.method === "POST");
    expect(posts).toHaveLength(1);
    expect(String(posts[0][0])).toContain(`/runs/${runId}/a2ui/actions`);
    expect(JSON.parse(String(posts[0][1]?.body))).toMatchObject({ surface_generation: "G1", surfaceId: "parcel",
      sourceComponentId: "submit", name: "calculate", interaction_id: latestActionReceipt(runId, "parcel")?.interaction_id });
    expect(latestActionReceipt(runId, "parcel")?.input?.surface_generation).toBe("G1");
  });
  test("a still-rendered G1 button cannot be relabeled by G2 admission authority with the same IDs", async () => {
    const runId = `rendered-publication-${++publicationSerial}`;
    const request = publicationServer(runId, "G2");
    render(<A2uiChunkView chunk={restoredPublication(runId, "G1")} />);
    const button = await screen.findByRole("button", { name: "Calculate parcel" });
    await waitFor(() => expect(button).toBeEnabled());
    fireEvent.click(button);
    await screen.findByText("This surface has changed. Request a fresh surface before acting.", { exact: true });
    expect(button).toBeDisabled();
    expect(request.mock.calls.filter((call) => call[1]?.method === "POST")).toHaveLength(0);
    await waitFor(() => expect(latestActionReceipt(runId, "parcel")).toMatchObject({ surface_generation: "G1", status: "rejected",
      input: { surface_generation: "G1", surfaceId: "parcel", sourceComponentId: "submit", name: "calculate" } }));
  });
  test.each([undefined, "", 42])("missing or malformed publication token %s stays read-only despite matching surface authority", async (generation) => {
    const runId = `rendered-publication-${++publicationSerial}`;
    const request = publicationServer(runId);
    render(<A2uiChunkView chunk={restoredPublication(runId, generation)} />);
    const button = await screen.findByRole("button", { name: "Calculate parcel" });
    expect(button).toBeDisabled();
    expect(screen.getByText("This surface has no publication identity. Request a fresh surface before acting.", { exact: true })).toBeVisible();
    fireEvent.click(button);
    expect(request.mock.calls.filter((call) => call[1]?.method === "POST")).toHaveLength(0);
  });
});

describe("action outcome feedback independent of surface authority", () => {
  test("lost acknowledgment keeps the original uncertain intent visible through authority loss and read-only recovery", async () => {
    const runId = `outcome-feedback-${++publicationSerial}`;
    const request = publicationServer(runId, "G1", true);
    const attachments = continuationHarness.attach.mock.calls.length;
    render(<A2uiChunkView chunk={restoredPublication(runId, "G1")} />);
    const button = await screen.findByRole("button", { name: "Calculate parcel" });
    await waitFor(() => expect(button).toBeEnabled());
    fireEvent.click(button);
    const warning = "Action outcome is unconfirmed. Do not submit again.";
    await screen.findByText(warning, { exact: true });
    const original = latestActionReceipt(runId, "parcel");
    expect(original).toMatchObject({ status: "uncertain", surface_generation: "G1" });
    expect(original?.interaction_id).toBeTruthy();
    act(() => {
      graphStore.getState().replaceEntity(A2UI_AUTHORITY_ENTITY, runId, {
        id: runId, status: "unavailable", authoritative: false, surfaces: [],
        recovery_hint: "Surface history is unavailable. Reload the conversation to recover.",
      });
    });
    expect(screen.getByText(warning, { exact: true })).toHaveAttribute("role", "alert");
    expect(screen.getByText("Surface history is unavailable. Reload the conversation to recover.", { exact: true })).toBeVisible();
    expect(button).toBeDisabled();
    fireEvent.click(button);
    fireEvent.click(screen.getByRole("button", { name: "Recover action status" }));
    await waitFor(() => expect(request.mock.calls.some((call) => String(call[0]).endsWith(`/a2ui/actions/${original?.interaction_id}`))).toBe(true));
    expect(screen.getByText(warning, { exact: true })).toBeVisible();
    expect(button).toBeDisabled();
    expect(latestActionReceipt(runId, "parcel")).toEqual(original);
    expect(request.mock.calls.filter((call) => call[1]?.method === "POST")).toHaveLength(1);
    expect(continuationHarness.attach.mock.calls).toHaveLength(attachments);
  });

  const outcomes = [
    { status: "pending", message: "Sending action…", role: "status" },
    { status: "responding", message: "Agent is responding…", role: "status" },
    { status: "uncertain", message: "Action outcome is unconfirmed. Do not submit again.", role: "alert" },
    { status: "interrupted", message: "The response was interrupted.", role: "alert" },
  ] as const;
  const authorities = [
    { kind: "unavailable", message: "Surface history is unavailable. Reload the conversation to recover." },
    { kind: "stale", message: "This surface has changed. Request a fresh surface before acting." },
    { kind: "missing", message: "Checking surface history…" },
  ] as const;

  for (const outcome of outcomes) {
    test.each(authorities)(`${outcome.status} feedback survives $kind authority without redispatch`, async (authority) => {
      const runId = `outcome-feedback-${++publicationSerial}`;
      const request = publicationServer(runId);
      render(<A2uiChunkView chunk={restoredPublication(runId, "G1")} />);
      const button = await screen.findByRole("button", { name: "Calculate parcel" });
      await waitFor(() => expect(button).toBeEnabled());
      const receipt: A2uiActionReceipt = { id: `${runId}-intent`, interaction_id: `${runId}-intent`,
        source_run_id: runId, surface_id: "parcel", surface_generation: "G1", status: outcome.status,
        continuation_run_id: outcome.status === "responding" || outcome.status === "interrupted" ? `${runId}-child` : null };

      // Seed a known logical outcome after mount so mount-time reconciliation
      // does not replace the pending/responding state before this transition.
      act(() => {
        graphStore.getState().replaceEntity(A2UI_ACTION_ENTITY, receipt.id, receipt);
      });
      expect(screen.getByText(outcome.message, { exact: true })).toHaveAttribute("role", outcome.role);
      expect(screen.queryByText(authority.message, { exact: true })).not.toBeInTheDocument();
      act(() => {
        if (authority.kind === "missing") {
          graphStore.getState().removeEntity(A2UI_AUTHORITY_ENTITY, runId);
        } else {
          graphStore.getState().replaceEntity(A2UI_AUTHORITY_ENTITY, runId, {
            id: runId, status: authority.kind === "unavailable" ? "unavailable" : "complete",
            authoritative: authority.kind === "stale", recovery_hint: authority.kind === "unavailable" ? authority.message : "",
            surfaces: authority.kind === "stale" ? [{ surface_id: "parcel", surface_generation: "G2", source_run_id: runId }] : [],
          });
        }
      });

      expect(screen.getByText(outcome.message, { exact: true })).toBeVisible();
      expect(screen.getByText(outcome.message, { exact: true })).toHaveAttribute("role", outcome.role);
      expect(screen.getByText(authority.message, { exact: true })).toBeVisible();
      expect(screen.getByText(authority.message, { exact: true })).toHaveAttribute("role", "status");
      expect(button).toBeDisabled();
      fireEvent.click(button);
      expect(request.mock.calls.filter((call) => call[1]?.method === "POST")).toHaveLength(0);
      expect(latestActionReceipt(runId, "parcel")).toEqual(receipt);

      fireEvent.click(screen.getByRole("button", { name: "Recover action status" }));
      await waitFor(() => expect(request.mock.calls.some((call) => String(call[0]).endsWith(`/a2ui/actions/${receipt.interaction_id}`))).toBe(true));
      expect(request.mock.calls.filter((call) => call[1]?.method === "POST")).toHaveLength(0);
      expect(latestActionReceipt(runId, "parcel")?.interaction_id).toBe(receipt.interaction_id);
      if (outcome.status === "uncertain") {
        expect(screen.getByText(outcome.message, { exact: true })).toBeVisible();
        expect(button).toBeDisabled();
      }
    });
  }
});
