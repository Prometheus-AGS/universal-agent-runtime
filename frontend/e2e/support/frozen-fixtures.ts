/**
 * Frozen oracles for SCN-IDN-001 and SCN-PICK-002, plus the combined provider
 * configuration the real-server Playwright config hands to the runtime.
 *
 * The four fixture files next to this module are byte-for-byte copies of the
 * frozen originals under openspec/changes/uiux-model-identity-graph-ownership/fixtures
 * and openspec/changes/uiux-schema-controls-and-pickers/fixtures. They live here
 * because the boundary baseline run copies only frontend/e2e/ and the Playwright
 * config into a pre-change worktree, where the openspec fixture paths do not
 * exist. The SHA-256 digests below are the ones pinned in the adopted contract
 * (scenarioMatrix SCN-IDN-001 and SCN-PICK-002); a copy that drifts is refused
 * before it can act as an oracle.
 *
 * Node built-ins only: the Playwright config imports this module.
 */
import crypto from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const supportDir = path.dirname(fileURLToPath(import.meta.url));
export const FIXTURE_DIR = path.join(supportDir, "fixtures");
const repoRoot = path.resolve(supportDir, "..", "..", "..");

export const FROZEN_SHA256 = {
  "catalog-identity.json": "e63d3424bdc3f5ac1d8a19e7341c43e3bca1f420759fe1986f24bc78ac3b6f62",
  "collision-config.yaml": "7512d6e53fd47fed35574d58725bce6e751c559b4121e87fb3071a5a4ee36e25",
  "picker-oracle.json": "a65d167d5f1c4339b3f77efb63b38eca6b2c33cebd4ae470e699f22aace7d495",
  "picker-config.yaml": "2a8df0cce979a0d1b33e8ac7fb290e4609632805ec438ac2d7973c272c7ecbcf",
} as const;

export type FrozenFixtureName = keyof typeof FROZEN_SHA256;

/** Digest of catalog/provider_catalog.json named by both oracles (`catalogSnapshot.sha256`). */
export const CATALOG_SNAPSHOT_SHA256 =
  "ea7e74b3ba7d335e5710aec713e88972aa18825512f2c16030b9aeab95694276";
export const CATALOG_SNAPSHOT_PATH = path.join(repoRoot, "catalog", "provider_catalog.json");

function sha256(bytes: Buffer): string {
  return crypto.createHash("sha256").update(bytes).digest("hex");
}

/** Reads a frozen fixture and refuses it when its digest differs from the contract's. */
export function readFrozenFixture(name: FrozenFixtureName): string {
  const bytes = fs.readFileSync(path.join(FIXTURE_DIR, name));
  const actual = sha256(bytes);
  if (actual !== FROZEN_SHA256[name]) {
    throw new Error(
      `Frozen fixture ${name} has sha256 ${actual}; the adopted contract pins ${FROZEN_SHA256[name]}. ` +
        "Refusing to use a drifted oracle.",
    );
  }
  return bytes.toString("utf8");
}

/** One row of the frozen identity oracle. Optional fields are asserted only when present. */
export interface FrozenModelRow {
  id: string;
  provider_id: string;
  model_id: string;
  context: number;
  tool_call: boolean;
  reasoning?: boolean;
  vision: boolean;
  cost_input?: number;
  cost_output?: number;
}

export interface CatalogIdentityOracle {
  catalogSnapshot: { sha256: string; providers: number };
  expectedFromCatalog: {
    providerModelPairs: number;
    distinctBareModelIds: number;
    bareIdsListedByTwoOrMoreProviders: number;
    modelIdsContainingSlash: number;
  };
  sampleRows: FrozenModelRow[];
  sharedBareIdSamples: Record<string, string[]>;
  customProviderCollisionConfig: {
    customProviderModels: FrozenModelRow[];
    extraPairs: number;
  };
}

export interface PickerOracle {
  catalogSnapshot: { sha256: string };
  provider: { id: string; display_name: string; catalogModelCount: number };
  searchCases: Record<string, number>;
  firstTenSortedIds: string[];
  lastTenSortedIds: string[];
  thresholds: {
    virtualizeAboveOptions: number;
    maxLongTaskMs: number;
    maxOptionElementsInDomWhenVirtualized: number;
  };
}

export function loadIdentityOracle(): CatalogIdentityOracle {
  return JSON.parse(readFrozenFixture("catalog-identity.json")) as CatalogIdentityOracle;
}

export function loadPickerOracle(): PickerOracle {
  return JSON.parse(readFrozenFixture("picker-oracle.json")) as PickerOracle;
}

export interface CatalogSnapshotModel {
  id: string;
  name?: string;
}

export interface CatalogSnapshotProvider {
  id: string;
  models?: CatalogSnapshotModel[];
}

/** The committed catalog snapshot, refused unless its digest is the one the oracles name. */
export function loadCatalogSnapshot(): CatalogSnapshotProvider[] {
  const bytes = fs.readFileSync(CATALOG_SNAPSHOT_PATH);
  const actual = sha256(bytes);
  if (actual !== CATALOG_SNAPSHOT_SHA256) {
    throw new Error(
      `catalog/provider_catalog.json has sha256 ${actual}; the oracles were frozen against ${CATALOG_SNAPSHOT_SHA256}.`,
    );
  }
  return JSON.parse(bytes.toString("utf8")) as CatalogSnapshotProvider[];
}

/**
 * Writes the one provider config the real-server runtime loads (`CONFIG_FILE`,
 * src/config.rs:22). It is the frozen collision config (idn-gw-a, idn-gw-b)
 * followed by the frozen picker config's provider entries (nano-gpt); neither
 * source is edited. Both oracles describe a `providers:` list under the same
 * top-level key, so the two lists join into one.
 */
export function writeCombinedProviderConfig(): string {
  const collision = readFrozenFixture("collision-config.yaml");
  const picker = readFrozenFixture("picker-config.yaml");
  const header = /^providers:[ \t]*\r?\n/m;
  const pickerHeader = header.exec(picker);
  if (!pickerHeader || !header.test(collision)) {
    throw new Error("Both frozen provider configs must start their list with `providers:`.");
  }
  const pickerEntries = picker.slice(pickerHeader.index + pickerHeader[0].length);
  const combined =
    `# Generated by frontend/e2e/support/frozen-fixtures.ts from the frozen\n` +
    `# collision-config.yaml (${FROZEN_SHA256["collision-config.yaml"]}) and\n` +
    `# picker-config.yaml (${FROZEN_SHA256["picker-config.yaml"]}). Do not edit.\n` +
    `${collision.replace(/\s*$/, "\n")}${pickerEntries}`;
  const digest = sha256(Buffer.from(combined, "utf8")).slice(0, 16);
  const target = path.join(os.tmpdir(), `uar-real-e2e-providers-${digest}.yaml`);
  fs.writeFileSync(target, combined, { encoding: "utf8", mode: 0o600 });
  return target;
}
