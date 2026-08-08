type ContentUrlKind = "download" | "image" | "video";

const ABSOLUTE_SAFE_PROTOCOL = /^(?:https?:|blob:)/iu;
const EXPLICIT_PROTOCOL = /^[a-z][a-z0-9+.-]*:/iu;
const IMAGE_DATA = /^data:image\/(?:avif|gif|jpeg|png|webp);base64,[a-z0-9+/=\s]+$/iu;
const VIDEO_DATA = /^data:video\/(?:mp4|ogg|webm);base64,[a-z0-9+/=\s]+$/iu;

/** Restricts provider-authored URLs before they reach an executable DOM attribute. */
export function safeContentUrl(value: string | undefined, kind: ContentUrlKind): string {
  const candidate = value?.trim() ?? "";
  if (!candidate || candidate.startsWith("//")) return "";
  if (ABSOLUTE_SAFE_PROTOCOL.test(candidate)) return candidate;
  if (kind === "image" && IMAGE_DATA.test(candidate)) return candidate;
  if (kind === "video" && VIDEO_DATA.test(candidate)) return candidate;
  const hasControlOrSpace = Array.from(candidate).some((character) => character.charCodeAt(0) <= 0x20);
  if (!EXPLICIT_PROTOCOL.test(candidate) && !hasControlOrSpace) return candidate;
  return "";
}
