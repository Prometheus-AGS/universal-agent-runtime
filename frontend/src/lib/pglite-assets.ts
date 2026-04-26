import pgliteDataUrl from "../../node_modules/@electric-sql/pglite/dist/pglite.data?url";

let fsBundlePromise: Promise<Blob> | null = null;

export function loadPgliteFsBundle(): Promise<Blob> {
  if (!fsBundlePromise) {
    fsBundlePromise = fetch(pgliteDataUrl).then((response) => {
      if (!response.ok) {
        throw new Error(`Unable to load PGlite bundle: HTTP ${response.status}`);
      }
      return response.blob();
    });
  }

  return fsBundlePromise;
}
