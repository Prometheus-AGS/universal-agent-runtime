import { render, screen } from "@testing-library/react";
import { StatusChip } from "./status-chip";

describe("StatusChip", () => {
  it("shows the state as text, so colour is never the only signal", () => {
    render(<StatusChip tone="warn">Reconnecting · attempt 3</StatusChip>);
    expect(screen.getByText("Reconnecting · attempt 3")).toBeInTheDocument();
  });

  it("marks the dot as decorative and exposes the tone for styling hooks", () => {
    const { container } = render(<StatusChip tone="ok">Connected</StatusChip>);
    const chip = container.firstElementChild as HTMLElement;
    expect(chip).toHaveAttribute("data-tone", "ok");
    expect(chip.querySelector('[aria-hidden="true"]')).not.toBeNull();
  });

  it("omits the dot when asked and merges caller classes after its own", () => {
    const { container } = render(
      <StatusChip tone="err" dot={false} className="px-4">
        Failed · auth
      </StatusChip>,
    );
    const chip = container.firstElementChild as HTMLElement;
    expect(chip.querySelector('[aria-hidden="true"]')).toBeNull();
    expect(chip.className).toContain("px-4");
    expect(chip.className).not.toContain("px-2.5");
  });

  it("defaults to the neutral tone", () => {
    const { container } = render(<StatusChip>Idle</StatusChip>);
    expect(container.firstElementChild).toHaveAttribute("data-tone", "neutral");
  });
});
