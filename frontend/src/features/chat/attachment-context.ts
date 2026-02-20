/**
 * AttachmentContext – provides a shared AttachmentManager
 * to the composer and the chat runtime within a thread.
 */

import { createContext, useContext } from "react";
import type { AttachmentManager } from "@/features/chat/use-attachment-manager";

export const AttachmentContext = createContext<AttachmentManager | null>(null);

export function useAttachmentContext(): AttachmentManager | null {
    return useContext(AttachmentContext);
}
