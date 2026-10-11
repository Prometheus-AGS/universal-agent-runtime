import type { Meta, StoryObj } from "@storybook/react-vite";
import { StatusChip } from "./status-chip";

const meta = {
  title: "uar-ui/Primitives/StatusChip",
  component: StatusChip,
  args: { tone: "ok", children: "Connected" },
} satisfies Meta<typeof StatusChip>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Connected: Story = {};

export const AllTones: Story = {
  render: () => (
    <div className="flex flex-wrap gap-2 bg-[var(--color-bg)] p-6">
      <StatusChip tone="neutral">Idle</StatusChip>
      <StatusChip tone="ok">Connected</StatusChip>
      <StatusChip tone="warn">Reconnecting · attempt 3</StatusChip>
      <StatusChip tone="err">Failed · auth</StatusChip>
      <StatusChip tone="live">Streaming</StatusChip>
      <StatusChip tone="accent">Selected</StatusChip>
    </div>
  ),
};
