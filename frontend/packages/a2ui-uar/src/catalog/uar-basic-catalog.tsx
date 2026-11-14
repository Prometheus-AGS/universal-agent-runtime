import { Catalog, type ComponentContext } from "@prometheus-ags/a2ui-core/v0_9";
import type { FC } from "react";
import { z } from "zod";
import {
  ButtonApi,
  CardApi,
  CheckBoxApi,
  ChoicePickerApi,
  ColumnApi,
  DividerApi,
  RowApi,
  TextApi,
  TextFieldApi,
} from "@prometheus-ags/a2ui-core/v0_9/basic_catalog";
import * as v1 from "@prometheus-ags/a2ui-core/renderer-v1";
import { UarButton } from "../components/Button";
import { UarCard } from "../components/Card";
import { UarCheckBox } from "../components/CheckBox";
import { UarChoicePicker } from "../components/ChoicePicker";
import { UarColumn } from "../components/Column";
import { UarDivider } from "../components/Divider";
import { UarRow } from "../components/Row";
import { UarText } from "../components/Text";
import { UarTextField } from "../components/TextField";
import { createUarComponentImplementation } from "../react/create-component";
import type { BuildChild, UarComponentImplementation, UarComponentProps } from "../react/types";

/**
 * The UAR-approved catalog id for the `uar.a2ui/1` profile
 * (`docs/protocols/a2ui-profile.md`), matching the id UAR's Rust-side
 * `A2uiRegistry` (`src/uar/a2ui/`) advertises via `createSurface.catalogId`
 * (`urn:uar:a2ui:catalog:1`, per
 * `.kbd-orchestrator/phases/uar-grade-a-upgrade-2026-07/analysis.md`).
 */
export const UAR_A2UI_CATALOG_ID = "urn:uar:a2ui:catalog:1";
export const A2UI_BASIC_CATALOG_V1_ID = "https://a2ui.org/specification/v1_0/catalogs/basic/catalog.json";

/**
 * The 9 protocol-standard components approved by `uar.a2ui/1`
 * (`docs/protocols/a2ui-profile.md`): Text, Button, TextField, CheckBox,
 * ChoicePicker, Row, Column, Card, Divider. This deliberately excludes the
 * rest of `web_core`'s basic_catalog (Image, Icon, Video, AudioPlayer,
 * List, Tabs, Modal, Slider, DateTimeInput) until their URL/content/privacy
 * policies are certified, per the profile doc.
 */
export const uarBasicCatalogComponents: UarComponentImplementation[] = [
  createUarComponentImplementation(TextApi, UarText),
  createUarComponentImplementation(ButtonApi, UarButton),
  createUarComponentImplementation(TextFieldApi, UarTextField),
  createUarComponentImplementation(CheckBoxApi, UarCheckBox),
  createUarComponentImplementation(ChoicePickerApi, UarChoicePicker),
  createUarComponentImplementation(RowApi, UarRow),
  createUarComponentImplementation(ColumnApi, UarColumn),
  createUarComponentImplementation(CardApi, UarCard),
  createUarComponentImplementation(DividerApi, UarDivider),
];

/** The UAR basic catalog: the 9 `uar.a2ui/1` protocol-standard components, rendered with shadcn/ui + react-aria-components. */
export const uarBasicCatalog = new Catalog<UarComponentImplementation>(
  UAR_A2UI_CATALOG_ID,
  "v0.9.1",
  uarBasicCatalogComponents,
);

// The wire boundary validates context directives before the binder sees this
// schema. Literal context objects may have ordinary `path` or `call` keys.
const finalButtonSchema = v1.ButtonApi.schema.extend({
  action: z.strictObject({
    event: z.strictObject({
      name: z.string(),
      userMessage: z.unknown().optional(),
      context: z.record(z.string(), z.unknown()).optional(),
    }),
  }),
});

const UarFinalButton: FC<{ props: UarComponentProps<typeof ButtonApi>; buildChild: BuildChild; context: ComponentContext }> = ({ props, buildChild, context }) => (
  <UarButton props={{ ...props, action: async () => {
    const parsed = finalButtonSchema.shape.action.safeParse(context.componentModel.properties.action);
    if (!parsed.success) return;
    const { name, userMessage, context: fields } = parsed.data.event;
    await context.dispatchAction({ event: {
      name,
      userMessage: context.dataContext.resolveDynamicValue(userMessage),
      context: Object.fromEntries(Object.entries(fields ?? {}).map(([key, value]) => [key, context.dataContext.resolveDynamicValue(value)])),
    } });
  } }} buildChild={buildChild} />
);

const v1Components: UarComponentImplementation[] = [
  createUarComponentImplementation(TextApi, UarText, v1.TextApi.schema),
  createUarComponentImplementation(ButtonApi, UarFinalButton, finalButtonSchema),
  createUarComponentImplementation(TextFieldApi, UarTextField, v1.TextFieldApi.schema),
  createUarComponentImplementation(CheckBoxApi, UarCheckBox, v1.CheckBoxApi.schema),
  createUarComponentImplementation(ChoicePickerApi, UarChoicePicker, v1.ChoicePickerApi.schema),
  createUarComponentImplementation(RowApi, UarRow, v1.RowApi.schema),
  createUarComponentImplementation(ColumnApi, UarColumn, v1.ColumnApi.schema),
  createUarComponentImplementation(CardApi, UarCard, v1.CardApi.schema),
  createUarComponentImplementation(DividerApi, UarDivider, v1.DividerApi.schema),
];

export const uarBasicCatalogV1 = new Catalog<UarComponentImplementation>(
  UAR_A2UI_CATALOG_ID,
  "1.0",
  v1Components,
);
export const a2uiBasicCatalogV1 = new Catalog<UarComponentImplementation>(
  A2UI_BASIC_CATALOG_V1_ID,
  "1.0",
  v1Components,
);
