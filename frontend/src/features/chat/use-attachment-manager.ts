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
import type { AttachmentPayload, PendingAttachment, UploadedFileResponse } from "@/types";

const UPLOAD_URL = "/api/upload";

interface UploadApiResponse {
    files: UploadedFileResponse[];
    errors: string[];
}

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

    const add = useCallback(
        (files: FileList | File[]) => {
            const newEntries: PendingAttachment[] = Array.from(files).map((file) => ({
                localId: crypto.randomUUID(),
                file,
                status: "uploading" as const,
            }));

            setPending((prev) => [...prev, ...newEntries]);

            // Fire off upload for each file
            for (const entry of newEntries) {
                void uploadFile(entry, sessionId, (updated) => {
                    setPending((prev) =>
                        prev.map((p) => (p.localId === updated.localId ? updated : p)),
                    );
                });
            }
        },
        [sessionId],
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

async function uploadFile(
    entry: PendingAttachment,
    sessionId: string,
    onUpdate: (updated: PendingAttachment) => void,
): Promise<void> {
    try {
        const form = new FormData();
        form.append("file", entry.file, entry.file.name);

        const res = await fetch(UPLOAD_URL, {
            method: "POST",
            headers: { "X-UAR-Session-ID": sessionId },
            body: form,
        });

        if (!res.ok) {
            const body = await res.text().catch(() => res.statusText);
            onUpdate({ ...entry, status: "error", errorMessage: body });
            return;
        }

        const json = (await res.json()) as UploadApiResponse;

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
        onUpdate({
            ...entry,
            status: "error",
            errorMessage: err instanceof Error ? err.message : "Upload failed",
        });
    }
}
