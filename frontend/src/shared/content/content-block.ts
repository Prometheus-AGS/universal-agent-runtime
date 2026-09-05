/** Portable wire and storage contract shared with the native renderers. */
export type ContentBlock =
  | { type: "text"; text: string }
  | { type: "thinking"; text: string }
  | { type: "code"; language: string; code: string }
  | { type: "citation"; source: string; quote: string }
  | { type: "memory"; operation: string; key: string; value: string | null }
  | { type: "toolUse"; id: string; name: string; inputJson: string }
  | { type: "toolResult"; toolUseId: string; outputJson: string; isError: boolean }
  | { type: "skill"; name: string; status: string }
  | { type: "artifact"; id: string; kind: string; content: string; title?: string }
  | {
      type: "image";
      url: string | null;
      dataBase64: string | null;
      mime: string;
      path?: string;
      alt?: string;
      width?: number;
      height?: number;
    }
  | { type: "divider" };

/** Compile-time exhaustiveness guard for discriminated unions. */
export function assertNever(value: never): never {
  throw new Error(`Unhandled content variant: ${JSON.stringify(value)}`);
}
