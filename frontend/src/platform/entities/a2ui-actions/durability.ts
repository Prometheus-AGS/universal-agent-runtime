let persist: (() => Promise<void>) | null = null;
export function bindA2uiActionPersistence(writer: (() => Promise<void>) | null): void { persist = writer; }
export async function persistA2uiActionReceipt(): Promise<void> {
  if (!persist) throw new Error("Action receipt storage is unavailable");
  await persist();
}
