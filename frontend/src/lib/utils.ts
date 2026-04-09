import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

/**
 * Convert a raw error (typically an HTTP status code or message) into a
 * user-friendly sentence that explains what went wrong and what to try next.
 */
export function friendlyError(raw: string): string {
  const trimmed = raw.trim();

  // Pure numeric status codes
  if (/^\d{3}$/.test(trimmed)) {
    return STATUS_MESSAGES[trimmed] ?? `Something went wrong (code ${trimmed}). Try again or contact your admin.`;
  }

  // "404" or "Error: 404" or "Fetch failed: 404"
  const codeMatch = trimmed.match(/\b([45]\d{2})\b/);
  if (codeMatch) {
    const friendly = STATUS_MESSAGES[codeMatch[1]];
    if (friendly) return friendly;
  }

  // Network / fetch errors
  if (/failed to fetch|network|econnrefused|timeout/i.test(trimmed)) {
    return "Can\u2019t reach the server. Check your connection and try again.";
  }

  // If it already looks like a sentence (starts with uppercase, has spaces), pass through
  if (/^[A-Z].*\s/.test(trimmed) && trimmed.length > 10) {
    return trimmed;
  }

  return `Something went wrong: ${trimmed}. Try again or contact your admin.`;
}

const STATUS_MESSAGES: Record<string, string> = {
  "400": "The request was invalid. Check your input and try again.",
  "401": "You\u2019re not logged in. Sign in and try again.",
  "403": "You don\u2019t have permission for this action. Contact your admin if you need access.",
  "404": "This item wasn\u2019t found. It may have been deleted or moved.",
  "409": "This conflicts with an existing item. Try a different name.",
  "413": "The file is too large. Try a smaller file.",
  "422": "Some fields are invalid. Check your input and try again.",
  "429": "Too many requests. Wait a moment and try again.",
  "500": "Something went wrong on the server. Try again in a moment.",
  "502": "The server is temporarily unavailable. Try again shortly.",
  "503": "The service is under maintenance. Try again in a few minutes.",
};

export function formatRelativeTime(dateStr: string): string {
  try {
    const d = new Date(dateStr);
    const now = new Date();
    const diffMs = now.getTime() - d.getTime();
    const diffMins = Math.floor(diffMs / 60_000);
    const diffHours = Math.floor(diffMs / 3_600_000);
    const diffDays = Math.floor(diffMs / 86_400_000);

    if (diffMins < 1) return "just now";
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 7) return d.toLocaleDateString("en-US", { weekday: "short" });
    return d.toLocaleDateString("en-US", { month: "short", day: "numeric" });
  } catch {
    return "";
  }
}
