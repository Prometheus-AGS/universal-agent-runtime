import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { InspectorPanel } from "../src/components/InspectorPanel";
import { createInspectorStore } from "../src/stores/inspector-store";
describe("InspectorPanel", () => { it("shows an actionable empty state and freeze control", () => { render(<InspectorPanel store={createInspectorStore()} />); expect(screen.getByText("No surface yet")).toBeInTheDocument(); expect(screen.getByRole("button", { name: "Freeze preview" })).toBeInTheDocument(); }); });
