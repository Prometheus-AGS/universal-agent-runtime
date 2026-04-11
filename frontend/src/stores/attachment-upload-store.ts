import { create } from "zustand";

import { postUploadAttachment } from "@/services/upload-api";
import type { PendingAttachment, UploadedFileResponse } from "@/types";

interface UploadApiJson {
  files?: UploadedFileResponse[];
  errors?: string[];
}

interface AttachmentUploadActions {
  /** Upload one file; invokes onUpdate as status progresses. */
  uploadOne: (
    entry: PendingAttachment,
    sessionId: string,
    onUpdate: (updated: PendingAttachment) => void,
  ) => Promise<void>;
}

export const useAttachmentUploadStore = create<AttachmentUploadActions>(() => ({
  uploadOne: async (entry, sessionId, onUpdate) => {
    try {
      const json = (await postUploadAttachment(sessionId, entry.file)) as UploadApiJson;

      if (json.errors?.length && !json.files?.length) {
        onUpdate({ ...entry, status: "error", errorMessage: json.errors[0] });
        return;
      }

      const uploaded = json.files?.[0];
      if (!uploaded) {
        onUpdate({ ...entry, status: "error", errorMessage: "No file returned" });
        return;
      }

      onUpdate({ ...entry, status: "ready", uploaded });
    } catch (err) {
      const msg = err instanceof Error ? err.message : "Upload failed";
      onUpdate({ ...entry, status: "error", errorMessage: msg });
    }
  },
}));
