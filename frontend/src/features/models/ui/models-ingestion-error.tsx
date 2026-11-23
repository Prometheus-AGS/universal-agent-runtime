import type { FC } from "react";
import { MODEL_CATALOG_SNAPSHOT_LOAD_ID, useModelCatalogLoadError } from "@/platform/entities";

/**
 * Shows why the last realtime models snapshot was not applied. The only subscription is the
 * snapshot load error, so the rest of the Models page does not rerender when it changes.
 */
export const ModelsIngestionError: FC = () => {
  const error = useModelCatalogLoadError(MODEL_CATALOG_SNAPSHOT_LOAD_ID);
  if (!error) return null;
  return (
    <div
      role="alert"
      data-testid="models-ingestion-error"
      className="bg-[color-mix(in_srgb,var(--color-red)_8%,transparent)] px-6 py-2 text-xs text-[var(--color-red)]"
    >
      <span className="mr-2 font-semibold">ERR-MODELS</span>
      Model catalog update rejected: {error}. Showing the previous catalog.
    </div>
  );
};
