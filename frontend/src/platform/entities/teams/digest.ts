import type { ImmutableDefinitionRef } from "./contracts";

/**
 * Document digests match src/uar/compiler/collaboration/validation/canonical.rs:
 * `"sha256:" + hex(SHA-256(canonical JSON))`, object keys sorted at every depth,
 * array order kept, the top-level `contentDigest` removed before hashing.
 *
 * Limits: numbers are written the way JSON.stringify writes them, so a source
 * literal such as `1.0` hashes as `1`. Documents the console builds contain
 * only strings, booleans and integers, where both sides agree.
 */

function compareCodePoints(left: string, right: string): number {
  // The runtime orders keys by UTF-8 bytes, which equals code point order.
  let i = 0;
  let j = 0;
  while (i < left.length && j < right.length) {
    const a = left.codePointAt(i) ?? 0;
    const b = right.codePointAt(j) ?? 0;
    if (a !== b) return a < b ? -1 : 1;
    i += a > 0xffff ? 2 : 1;
    j += b > 0xffff ? 2 : 1;
  }
  return left.length - i === right.length - j ? 0 : left.length - i < right.length - j ? -1 : 1;
}

function serialize(value: unknown): string {
  if (value === null) return "null";
  if (Array.isArray(value)) {
    return `[${value.map((item) => (item === undefined ? "null" : serialize(item))).join(",")}]`;
  }
  switch (typeof value) {
    case "string":
    case "boolean":
      return JSON.stringify(value);
    case "number":
      if (!Number.isFinite(value)) throw new TypeError("Canonical JSON cannot encode a non-finite number.");
      return JSON.stringify(value);
    case "object": {
      const record = value as Record<string, unknown>;
      const keys = Object.keys(record)
        .filter((key) => record[key] !== undefined)
        .sort(compareCodePoints);
      return `{${keys.map((key) => `${JSON.stringify(key)}:${serialize(record[key])}`).join(",")}}`;
    }
    default:
      throw new TypeError(`Canonical JSON cannot encode a ${typeof value}.`);
  }
}

/** Sorted-key JSON text, the exact bytes the runtime hashes (and the form the manifest and files must be sent in). */
export function canonicalJson(value: unknown): string {
  return serialize(value);
}

function hex(buffer: ArrayBuffer): string {
  return Array.from(new Uint8Array(buffer), (byte) => byte.toString(16).padStart(2, "0")).join("");
}

/** `"sha256:<hex>"` over the UTF-8 bytes of `text` (or over raw bytes). Needs Web Crypto, so a secure context. */
export async function sha256Digest(input: string | Uint8Array): Promise<string> {
  const subtle = globalThis.crypto?.subtle;
  if (!subtle) throw new Error("SHA-256 needs a secure browser context (https or localhost).");
  // Copied so the digest input is backed by a plain ArrayBuffer, which is what Web Crypto accepts.
  const bytes = new Uint8Array(typeof input === "string" ? new TextEncoder().encode(input) : input);
  return `sha256:${hex(await subtle.digest("SHA-256", bytes))}`;
}

/** Digest of a file's exact bytes, as listed in a manifest's `byteDigest`. */
export function byteDigest(bytes: string | Uint8Array): Promise<string> {
  return sha256Digest(bytes);
}

/** The `contentDigest` a document must carry: canonical JSON with the top-level digest field absent. */
export function contentDigestOf(document: Readonly<Record<string, unknown>>): Promise<string> {
  return sha256Digest(canonicalJson(withoutDigest(document)));
}

function withoutDigest(document: Readonly<Record<string, unknown>>): Record<string, unknown> {
  return Object.fromEntries(Object.entries(document).filter(([key]) => key !== "contentDigest"));
}

/** A copy of `document` carrying a freshly computed `contentDigest`. */
export async function withContentDigest<T extends Readonly<Record<string, unknown>>>(
  document: T,
): Promise<Omit<T, "contentDigest"> & { contentDigest: string }> {
  return { ...withoutDigest(document), contentDigest: await contentDigestOf(document) } as Omit<T, "contentDigest"> & {
    contentDigest: string;
  };
}

/** The `{id, version, digest}` reference to a digested document. */
export function documentReference(document: {
  id: string;
  version: string;
  contentDigest: string;
}): ImmutableDefinitionRef {
  return { id: document.id, version: document.version, digest: document.contentDigest };
}
