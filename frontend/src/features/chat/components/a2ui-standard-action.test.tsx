import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, test, vi } from "vitest";
import { A2UI_CATALOG_ID, A2UI_PROFILE, A2UI_VERSION } from "@/platform/a2ui/protocol";
import { A2uiDisplayBlock } from "./a2ui-artifact-block";

const intent = vi.hoisted(() => vi.fn<(runId: string, input: unknown) => Promise<void>>(async () => {}));
vi.mock("@/platform/entities", () => ({
  useA2uiActionStatus: () => undefined,
  useA2uiActionContinuation: () => null,
  useA2uiActionIntent: () => intent,
}));
const surface = JSON.stringify([
  { version: A2UI_VERSION, createSurface: { surfaceId: "parcel", catalogId: A2UI_CATALOG_ID } },
  { version: A2UI_VERSION, updateComponents: { surfaceId: "parcel", components: [
    { id: "value", component: "TextField", label: "Parcel number", value: { path: "/parcel" }, variant: "shortText" },
    { id: "label", component: "Text", text: "Calculate parcel" },
    { id: "submit", component: "Button", child: "label", action: { event: { name: "calculate" } } },
    { id: "root", component: "Column", children: ["value", "submit"] },
  ] } },
  { version: A2UI_VERSION, updateDataModel: { surfaceId: "parcel", path: "/", value: { parcel: "" } } },
]);

describe("actual pinned standard renderer action binding", () => {
  test("edits the real field, detaches its live model on click, and suppresses duplicate pending callbacks", async () => {
    let finish!: () => void;
    intent.mockReset().mockImplementationOnce(() => new Promise<void>((resolve) => { finish = resolve; }));
    const view = render(<A2uiDisplayBlock runId="native-source" artifactType="a2ui/surface" title="Parcel" content={surface} profile={A2UI_PROFILE} />);
    fireEvent.change(screen.getByRole("textbox", { name: "Parcel number" }), { target: { value: "137" } });
    fireEvent.click(screen.getByRole("button", { name: "Calculate parcel" }));
    fireEvent.click(screen.getByRole("button", { name: "Calculate parcel" }));
    await waitFor(() => expect(intent).toHaveBeenCalledTimes(1));
    expect(intent.mock.calls[0]).toEqual(["native-source", expect.objectContaining({
      name: "calculate", surfaceId: "parcel", sourceComponentId: "submit",
      a2uiClientDataModel: { parcel: { parcel: "137" } },
    })]);
    fireEvent.change(screen.getByRole("textbox", { name: "Parcel number" }), { target: { value: "999" } });
    expect(intent.mock.calls[0][1]).toMatchObject({ a2uiClientDataModel: { parcel: { parcel: "137" } } });
    await act(async () => finish());
    view.unmount();
  });
  test("an unbound surface cannot send an action", () => {
    intent.mockClear();
    render(<A2uiDisplayBlock artifactType="a2ui/surface" title="Parcel" content={surface} profile={A2UI_PROFILE} />);
    expect(screen.getByRole("status")).toHaveTextContent("Actions unavailable: this surface is not linked to a run.");
    expect(screen.getByRole("button", { name: "Calculate parcel" })).toBeDisabled();
    fireEvent.click(screen.getByRole("button", { name: "Calculate parcel" }));
    expect(intent).not.toHaveBeenCalled();
  });
});
