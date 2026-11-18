import { byteDigest, definitionKey, type PackageManifest } from "@/platform/entities";

/**
 * Local checks on a chosen team package before preflight is requested: identify the manifest,
 * list files it names that are missing, files it does not name, and files whose byte digest differs.
 */

export interface SelectedPackageFile {
  name: string;
  bytes: Uint8Array;
}

export type FileCheckStatus = "ok" | "missing" | "digest" | "extra";

export interface FileReport {
  /** Manifest path for listed files; the chosen file's name for extras. */
  path: string;
  status: FileCheckStatus;
  detail: string;
}

export interface PackageAnalysis {
  manifest: PackageManifest | null;
  manifestText: string | null;
  /** Exact file text keyed by manifest path, for the files that matched. */
  files: Record<string, string>;
  reports: FileReport[];
  /** Problems with the selection as a whole (no manifest, several manifests, unreadable manifest). */
  problems: string[];
  /** Preflight may be requested. Extra files do not block; they are simply not sent. */
  ready: boolean;
}

const decoder = new TextDecoder("utf-8", { ignoreBOM: true });
const decode = (bytes: Uint8Array) => decoder.decode(bytes);

function parseJson(bytes: Uint8Array): unknown {
  try {
    return JSON.parse(decode(bytes));
  } catch {
    return undefined;
  }
}

function isManifestShape(value: unknown): value is PackageManifest {
  if (!value || typeof value !== "object") return false;
  const record = value as Record<string, unknown>;
  return (
    record.kind === "PackageManifest" &&
    Array.isArray(record.files) &&
    record.files.every((file) => {
      const entry = file as Record<string, unknown> | null;
      return !!entry && typeof entry.path === "string" && typeof entry.byteDigest === "string";
    })
  );
}

function isManifestKind(value: unknown): boolean {
  return !!value && typeof value === "object" && (value as Record<string, unknown>).kind === "PackageManifest";
}

const baseName = (path: string) => path.slice(path.lastIndexOf("/") + 1);

/** A chosen file matches a manifest path exactly, or by file name when that name is unambiguous. */
function findChosen(path: string, candidates: readonly SelectedPackageFile[]): SelectedPackageFile | "ambiguous" | null {
  const exact = candidates.find((file) => file.name === path);
  if (exact) return exact;
  const named = candidates.filter((file) => file.name === baseName(path));
  if (named.length === 1) return named[0];
  return named.length > 1 ? "ambiguous" : null;
}

export async function analysePackageSelection(selected: readonly SelectedPackageFile[]): Promise<PackageAnalysis> {
  const empty: PackageAnalysis = { manifest: null, manifestText: null, files: {}, reports: [], problems: [], ready: false };
  if (selected.length === 0) return { ...empty, problems: ["Choose the package manifest and its files."] };

  const manifests = selected.filter((file) => isManifestKind(parseJson(file.bytes)));
  if (manifests.length === 0) return { ...empty, problems: ['None of the chosen files is a manifest (kind "PackageManifest").'] };
  if (manifests.length > 1) {
    return { ...empty, problems: [`More than one manifest was chosen: ${manifests.map((file) => file.name).join(", ")}. Choose one package at a time.`] };
  }
  const manifestFile = manifests[0];
  const parsed = parseJson(manifestFile.bytes);
  if (!isManifestShape(parsed)) {
    return { ...empty, problems: [`${manifestFile.name} does not list its files with a path and byteDigest each.`] };
  }

  const others = selected.filter((file) => file !== manifestFile);
  const used = new Set<SelectedPackageFile>();
  const files: Record<string, string> = {};
  const reports: FileReport[] = [];

  for (const listed of parsed.files) {
    const found = findChosen(listed.path, others);
    if (found === null || found === "ambiguous") {
      const detail = found === "ambiguous"
        ? "Several chosen files share this name, so none can be matched. Choose the files from one folder."
        : "Listed in the manifest but not chosen.";
      reports.push({ path: listed.path, status: "missing", detail });
      continue;
    }
    used.add(found);
    const actual = await byteDigest(found.bytes);
    if (actual !== listed.byteDigest) {
      reports.push({
        path: listed.path,
        status: "digest",
        detail: `The file's digest ${actual} differs from the manifest's ${listed.byteDigest}.`,
      });
      continue;
    }
    files[listed.path] = decode(found.bytes);
    reports.push({ path: listed.path, status: "ok", detail: "Matches the manifest digest." });
  }

  for (const file of others) {
    if (!used.has(file)) {
      reports.push({ path: file.name, status: "extra", detail: "Not listed in the manifest. It will not be sent." });
    }
  }

  const ready = reports.every((report) => report.status === "ok" || report.status === "extra");
  return { manifest: parsed, manifestText: decode(manifestFile.bytes), files, reports, problems: [], ready };
}

/** Keys (`id@version`) of the team definitions a manifest lists, in manifest order. */
export function teamDefinitionKeysOf(manifest: PackageManifest): string[] {
  return manifest.files.filter((file) => file.kind === "TeamDefinition").map((file) => definitionKey(file.definition));
}
