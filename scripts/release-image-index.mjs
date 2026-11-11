import { createHash } from "node:crypto";

const digestPattern = /^sha256:[0-9a-f]{64}$/;
const indexTypes = new Set([
  "application/vnd.oci.image.index.v1+json",
  "application/vnd.docker.distribution.manifest.list.v2+json",
]);
const imageTypes = new Set([
  "application/vnd.oci.image.manifest.v1+json",
  "application/vnd.docker.distribution.manifest.v2+json",
]);

export function releaseImagePlatforms(bytes, digest) {
  if (`sha256:${createHash("sha256").update(bytes).digest("hex")}` !== digest) {
    throw new Error("release image index digest mismatch");
  }
  const index = JSON.parse(bytes);
  if (index.schemaVersion !== 2 || !indexTypes.has(index.mediaType) || !Array.isArray(index.manifests)) {
    throw new Error("release image must be a digest-addressed image index");
  }
  const platforms = new Set();
  for (const descriptor of index.manifests) {
    const { os, architecture, variant } = descriptor.platform ?? {};
    if (!imageTypes.has(descriptor.mediaType) || !digestPattern.test(descriptor.digest)) {
      throw new Error("invalid release image descriptor");
    }
    if (os === "unknown" && architecture === "unknown" &&
        descriptor.annotations?.["vnd.docker.reference.type"] === "attestation-manifest") continue;
    const platform = [os, architecture, variant].filter((part) => part !== undefined);
    if (platform.length < 2 || !os || !architecture || platform.some((part) => typeof part !== "string" || !/^[a-z0-9_]+$/.test(part)) ||
        os === "unknown" || architecture === "unknown") {
      throw new Error("invalid runtime platform in release image index");
    }
    const name = platform.join("/");
    if (platforms.has(name)) throw new Error(`duplicate release image platform: ${name}`);
    platforms.add(name);
  }
  if (!platforms.size) throw new Error("release image index has no runtime platforms");
  return [...platforms].sort();
}
