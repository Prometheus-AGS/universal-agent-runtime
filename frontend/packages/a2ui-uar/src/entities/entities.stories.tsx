import type { Meta, StoryObj } from "@storybook/react-vite";
import { uarEntityCatalog } from "../catalog/uar-entity-catalog";
import { renderStorySurface } from "../dev/build-surface";

/**
 * The 7 UAR-specific `Entity*` extension components (`urn:uar:a2ui:catalog:1+entities`),
 * each rendered through a real `MessageProcessor` surface.
 */
const meta: Meta = {
  title: "A2UI/Entities",
};
export default meta;

type Story = StoryObj;

export const EntityCard: Story = {
  render: () =>
    renderStorySurface(uarEntityCatalog, [
      {
        id: "root",
        component: "EntityCard",
        entityType: "Order",
        entityId: "order-123",
        title: "Order #123",
        subtitle: "Placed 2026-07-10",
        syncOrigin: "optimistic",
        fields: [
          { label: "Status", value: "Pending" },
          { label: "Total", value: "$42.00" },
        ],
        actions: [{ label: "Cancel", action: { event: { name: "cancelOrder", context: {} } } }],
      },
    ]),
};

export const EntityDiff: Story = {
  render: () =>
    renderStorySurface(uarEntityCatalog, [
      {
        id: "root",
        component: "EntityDiff",
        title: "Order #123 changes",
        changes: [
          { field: "status", before: "Pending", after: "Shipped", operation: "update" },
          { field: "trackingNumber", after: "1Z999AA1", operation: "add" },
        ],
      },
    ]),
};

export const EntityStream: Story = {
  render: () =>
    renderStorySurface(uarEntityCatalog, [
      {
        id: "root",
        component: "EntityStream",
        title: "Live updates",
        status: "streaming",
        items: ["Order created", "Payment confirmed"],
      },
    ]),
};

export const EntityStreamError: Story = {
  render: () =>
    renderStorySurface(uarEntityCatalog, [
      {
        id: "root",
        component: "EntityStream",
        title: "Live updates",
        status: "error",
        message: "Connection lost",
        retry: { event: { name: "retryStream", context: {} } },
      },
    ]),
  parameters: {
    // Same pre-existing --primary/--primary-foreground contrast gap as
    // src/components/ui/ui-primitives.stories.tsx's ButtonPrimary/BadgeDefault
    // (the Retry button uses the shared shadcn Button default variant).
    // Flagged as a follow-up in this change's proposal.md.
    a11y: { test: "off" },
  },
};

export const EntityApproval: Story = {
  render: () =>
    renderStorySurface(uarEntityCatalog, [
      {
        id: "root",
        component: "EntityApproval",
        title: "Approve refund",
        summary: "Refund $42.00 to customer for Order #123",
        changes: ["Refund amount: $42.00", "Reason: Item damaged"],
        approve: { label: "Approve", action: { event: { name: "approve", context: {} } } },
        reject: { label: "Reject", action: { event: { name: "reject", context: {} } } },
      },
    ]),
};

export const EntityToolProvider: Story = {
  render: () =>
    renderStorySurface(uarEntityCatalog, [
      {
        id: "root",
        component: "EntityToolProvider",
        title: "Refund tool",
        description: "Issues a refund via the payments provider",
        status: "available",
        actions: [{ label: "Run", action: { event: { name: "runTool", context: {} } } }],
      },
    ]),
};

export const EntityChat: Story = {
  render: () =>
    renderStorySurface(uarEntityCatalog, [
      {
        id: "root",
        component: "EntityChat",
        title: "Support conversation",
        messages: [
          { role: "user", content: "Where's my order?" },
          { role: "assistant", content: "It shipped yesterday and should arrive Friday." },
        ],
      },
    ]),
};

export const EntityCopilot: Story = {
  render: () =>
    renderStorySurface(uarEntityCatalog, [
      {
        id: "root",
        component: "EntityCopilot",
        title: "Suggested replies",
        suggestions: [{ label: "Track package", action: { event: { name: "track", context: {} } } }],
        dismiss: { event: { name: "dismiss", context: {} } },
      },
    ]),
};
