import { InspectorPanel } from "./components/InspectorPanel";
export const A2UI_INSPECTOR_ADDON_ID = "prometheus-ags/a2ui-inspector";
export const A2UI_INSPECTOR_PANEL_ID = `${A2UI_INSPECTOR_ADDON_ID}/panel`;
export const a2uiInspectorAddon = { id: A2UI_INSPECTOR_ADDON_ID, panelId: A2UI_INSPECTOR_PANEL_ID, title: "A2UI Inspector" } as const;
export { InspectorPanel };
