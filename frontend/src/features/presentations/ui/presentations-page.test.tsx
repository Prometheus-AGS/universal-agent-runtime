import { act, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import * as api from "@/platform/entities/presentations/api/presentations-api";
import { presentationActions } from "@/platform/entities/presentations/domain";
import { registerPresentationEntities } from "@/platform/entities/presentations/registration";
import { STARTER_PRESENTATION_SOURCE, presentationTemplateSchema } from "@/platform/entities/presentations/contracts";
import { PresentationsPage } from "./presentations-page";

const owner = "presentation-page-test-owner";

beforeEach(async () => {
  registerPresentationEntities();
  presentationActions.close(true);
  vi.spyOn(api, "fetchPresentations").mockResolvedValue({ owner_id: owner, presentations: [] });
  await presentationActions.reload();
});
afterEach(() => vi.restoreAllMocks());

describe("Presentation catalog navigation", () => {
  test.each([0, 1])("New Presentation entry %i opens a new draft, not a click-event ID", async (index) => {
    const begin = vi.spyOn(presentationActions, "begin");
    render(<PresentationsPage />);
    const buttons = screen.getAllByRole("button", { name: "New Presentation" });
    expect(buttons).toHaveLength(2);
    fireEvent.click(buttons[index]!);
    expect(begin).toHaveBeenCalledWith(undefined);
    expect(await screen.findByRole("textbox", { name: /Title/ })).toHaveValue("");
    expect(screen.getByRole("textbox", { name: /Template source/ })).toHaveValue(STARTER_PRESENTATION_SOURCE);
  });

  test("opening an existing row retains its real template identity", async () => {
    vi.mocked(api.fetchPresentations).mockResolvedValue({ owner_id: owner, presentations: [{
      id: "saved-template", owner_id: owner, revision: 7,
      content: { title: "Saved report", description: "A saved template", enabled: true,
        template: presentationTemplateSchema.parse(JSON.parse(STARTER_PRESENTATION_SOURCE)) },
      created_at: "2026-09-05T00:00:00Z", updated_at: "2026-09-05T00:00:00Z",
    }] });
    await act(() => presentationActions.reload());
    const begin = vi.spyOn(presentationActions, "begin");
    render(<PresentationsPage />);
    fireEvent.click(screen.getByRole("button", { name: /Saved report/ }));
    expect(begin).toHaveBeenCalledWith("saved-template");
    expect(await screen.findByRole("textbox", { name: /Title/ })).toHaveValue("Saved report");
  });
});
