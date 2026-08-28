import type { Meta, StoryObj } from "@storybook/react-vite";
import { uarBasicCatalog } from "../catalog/uar-basic-catalog";
import { renderStorySurface } from "../dev/build-surface";

/**
 * The 9 `uar.a2ui/1` protocol-standard components (`docs/protocols/a2ui-profile.md`),
 * each rendered through a real `MessageProcessor` surface -- see
 * `src/dev/build-surface.tsx`.
 */
const meta: Meta = {
  title: "A2UI/Protocol",
};
export default meta;

type Story = StoryObj;

export const Text: Story = {
  render: () =>
    renderStorySurface(uarBasicCatalog, [
      { id: "root", component: "Text", text: "Order confirmed", variant: "h2" },
    ]),
};

export const Button: Story = {
  render: () =>
    renderStorySurface(uarBasicCatalog, [
      { id: "root", component: "Button", child: "label", action: { event: { name: "confirm", context: {} } } },
      { id: "label", component: "Text", text: "Confirm", variant: "body" },
    ]),
};

export const TextFieldShortText: Story = {
  render: () =>
    renderStorySurface(
      uarBasicCatalog,
      [{ id: "root", component: "TextField", label: "Name", value: { path: "/name" } }],
      { name: "" },
    ),
};

export const TextFieldLongText: Story = {
  render: () =>
    renderStorySurface(uarBasicCatalog, [
      { id: "root", component: "TextField", label: "Notes", variant: "longText" },
    ]),
};

export const CheckBox: Story = {
  render: () =>
    renderStorySurface(
      uarBasicCatalog,
      [{ id: "root", component: "CheckBox", label: "Accept terms", value: { path: "/accepted" } }],
      { accepted: false },
    ),
};

export const ChoicePickerSingleSelect: Story = {
  render: () =>
    renderStorySurface(
      uarBasicCatalog,
      [
        {
          id: "root",
          component: "ChoicePicker",
          label: "Color",
          variant: "mutuallyExclusive",
          options: [
            { label: "Red", value: "red" },
            { label: "Blue", value: "blue" },
          ],
          value: { path: "/color" },
        },
      ],
      { color: [] },
    ),
};

export const ChoicePickerMultiSelect: Story = {
  render: () =>
    renderStorySurface(uarBasicCatalog, [
      {
        id: "root",
        component: "ChoicePicker",
        label: "Toppings",
        variant: "multipleSelection",
        options: [
          { label: "Cheese", value: "cheese" },
          { label: "Olives", value: "olives" },
        ],
        value: [],
      },
    ]),
};

export const Row: Story = {
  render: () =>
    renderStorySurface(uarBasicCatalog, [
      { id: "root", component: "Row", children: ["a", "b"] },
      { id: "a", component: "Text", text: "Left", variant: "body" },
      { id: "b", component: "Text", text: "Right", variant: "body" },
    ]),
};

export const Column: Story = {
  render: () =>
    renderStorySurface(uarBasicCatalog, [
      { id: "root", component: "Column", children: ["a", "b"] },
      { id: "a", component: "Text", text: "Top", variant: "body" },
      { id: "b", component: "Text", text: "Bottom", variant: "body" },
    ]),
};

export const Card: Story = {
  render: () =>
    renderStorySurface(uarBasicCatalog, [
      { id: "root", component: "Card", child: "inner" },
      { id: "inner", component: "Text", text: "Card content", variant: "body" },
    ]),
};

export const Divider: Story = {
  render: () =>
    renderStorySurface(uarBasicCatalog, [{ id: "root", component: "Divider" }]),
};

export const HighContrastLongTranslation: Story = {
  parameters: { viewport: { defaultViewport: "mobile1" } },
  render: () => {
    const surface = renderStorySurface(uarBasicCatalog, [
      { id: "root", component: "Column", children: ["heading", "field", "choice"] },
      { id: "heading", component: "Text", variant: "h2", text: "アクセシビリティ設定を確認してください" },
      { id: "field", component: "TextField", label: "共同作業者に表示する名前", variant: "longText" },
      { id: "choice", component: "ChoicePicker", options: [{ label: "標準", value: "standard" }, { label: "緊急", value: "urgent" }], value: [] },
    ], {}, { theme: "high-contrast", locale: "ja", direction: "rtl" });
    return <div className="max-w-sm">{surface}</div>;
  },
};
