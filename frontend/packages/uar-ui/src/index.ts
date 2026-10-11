// @prometheus-ags/uar-ui — presentational primitives and composition patterns for
// the UAR operator shell. See docs/design/react-implementation-plan.md §4.
//
// Rules for everything exported here: props in, callbacks out. No stores,
// services, fetch, entity graph or Tauri access; styling only through the app's
// design tokens (frontend/src/shared/theme/tokens.css); Flat 2.0 (no borders,
// rings, shadows, blur or gradients).

export { cn } from "./lib/cn";
export { StatusChip, type StatusChipProps, type StatusTone } from "./primitives/status-chip";
