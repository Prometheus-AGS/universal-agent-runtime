/**
 * useAttachmentManager – manages pending file attachments for a chat message.
 *
 * Usage:
 *   const att = useAttachmentManager();
 *   att.add(files);          // triggers upload to POST /api/upload
 *   att.remove(localId);     // cancel/discard
 *   att.clear();             // after send
 *   att.pending              // current PendingAttachment[]
 *   att.toPayload()          // AttachmentPayload[] ready for the chat request
 */

import { useCallback, useState } from "react";
import type { AttachmentPayload, PendingAttachment } from "@/types";
import { useAttachmentUploadStore } from "@/stores/attachment-upload-store";

export interface AttachmentManager {
    pending: PendingAttachment[];
    add: (files: FileList | File[]) => void;
    remove: (localId: string) => void;
    clear: () => void;
    /** Returns attachment payloads for ready files only. */
    toPayload: () => AttachmentPayload[];
    /** True while any file is still uploading. */
    isUploading: boolean;
}

export function useAttachmentManager(sessionId: string): AttachmentManager {
    const [pending, setPending] = useState<PendingAttachment[]>([]);
    const uploadOne = useAttachmentUploadStore((s) => s.uploadOne);

    const add = useCallback(
        (files: FileList | File[]) => {
            const newEntries: PendingAttachment[] = Array.from(files).map((file) => ({
                localId: crypto.randomUUID(),
                file,
                status: "uploading" as const,
            }));

            setPending((prev) => [...prev, ...newEntries]);

            for (const entry of newEntries) {
                void uploadOne(entry, sessionId, (updated) => {
                    setPending((prev) =>
                        prev.map((p) => (p.localId === updated.localId ? updated : p)),
                    );
                });
            }
        },
        [sessionId, uploadOne],
    );

    const remove = useCallback((localId: string) => {
        setPending((prev) => prev.filter((p) => p.localId !== localId));
    }, []);

    const clear = useCallback(() => {
        setPending([]);
    }, []);

    const toPayload = useCallback((): AttachmentPayload[] => {
        return pending
            .filter((p) => p.status === "ready" && p.uploaded)
            .map((p) => ({
                id: p.uploaded!.id,
                filename: p.uploaded!.filename,
                content_type: p.uploaded!.content_type,
                url: p.uploaded!.url,
                text_content: p.uploaded!.text_content,
            }));
    }, [pending]);

    const isUploading = pending.some((p) => p.status === "uploading");

    return { pending, add, remove, clear, toPayload, isUploading };
}
