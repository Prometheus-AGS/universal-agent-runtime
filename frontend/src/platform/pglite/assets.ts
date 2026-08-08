import pgliteDataUrl from "../../../node_modules/@electric-sql/pglite/dist/pglite.data?url";
import pgliteWasmUrl from "../../../node_modules/@electric-sql/pglite/dist/pglite.wasm?url";
import pgliteSeedV3Url from "@/platform/pglite/pglite-seed-v3.tar.gz?url";

let fsBundlePromise: Promise<Blob> | null = null;
let wasmModulePromise: Promise<WebAssembly.Module> | null = null;
let seedPromise: Promise<Blob> | null = null;

const PGLITE_INDEXED_DB_NAME = "/pglite/uar-threads";

function fetchAsset(url: string, label: string): Promise<Blob> {
  return fetch(url).then((response) => {
    if (!response.ok) {
      throw new Error(`Unable to load ${label}: HTTP ${response.status}`);
    }
    return response.blob();
  });
}

export function loadPgliteFsBundle(): Promise<Blob> {
  if (!fsBundlePromise) {
    fsBundlePromise = fetchAsset(pgliteDataUrl, "PGlite bundle");
  }

  return fsBundlePromise;
}

export function loadPgliteWasmModule(): Promise<WebAssembly.Module> {
  if (!wasmModulePromise) {
    wasmModulePromise = WebAssembly.compileStreaming(fetch(pgliteWasmUrl));
  }
  return wasmModulePromise;
}

async function hasExistingDatabase(): Promise<boolean> {
  if (typeof indexedDB.databases !== "function") {
    return true;
  }

  const databases = await indexedDB.databases();
  return databases.some(({ name }) => name === PGLITE_INDEXED_DB_NAME);
}

export async function loadPgliteSeedForFreshDatabase(): Promise<Blob | undefined> {
  if (await hasExistingDatabase()) {
    return undefined;
  }

  if (!seedPromise) {
    seedPromise = fetchAsset(pgliteSeedV3Url, "PGlite schema seed");
  }
  return seedPromise;
}
