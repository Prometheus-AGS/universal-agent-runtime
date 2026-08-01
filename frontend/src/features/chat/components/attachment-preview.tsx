/**
 * AttachmentPreviewStrip – horizontal row of file chips shown above the composer
 * when attachments are pending.
 *
 * Each chip shows:
 *  - Image: thumbnail preview (48×48)
 *  - Document: file-type icon + truncated filename
 *  - Upload in-progress: spinner overlay
 *  - Ready: green dot badge
 *  - Error: red border + tooltip with error
 *  - × remove button in top-right corner
 */

import { FC, useEffect, useMemo } from "react";
import { X, FileText, FileImage, File, Loader2 } from "lucide-react";
import type { PendingAttachment } from "@/types";

interface Props {
    attachments: PendingAttachment[];
    onRemove: (localId: string) => void;
}

export const AttachmentPreviewStrip: FC<Props> = ({ attachments, onRemove }) => {
    if (attachments.length === 0) return null;

    return (
        <div className="flex flex-wrap gap-2 px-3 pt-2 pb-1">
            {attachments.map((att) => (
                <AttachmentChip key={att.localId} attachment={att} onRemove={onRemove} />
            ))}
        </div>
    );
};

// ─────────────────────────────────────────────────────────────────────────────
// Single chip
// ─────────────────────────────────────────────────────────────────────────────

interface ChipProps {
    attachment: PendingAttachment;
    onRemove: (localId: string) => void;
}

const AttachmentChip: FC<ChipProps> = ({ attachment, onRemove }) => {
    const { localId, file, status, uploaded, errorMessage } = attachment;
    const isImage = file.type.startsWith("image/");
    const isUploading = status === "uploading";
    const isError = status === "error";

    const mimeLabel = mimeToLabel(file.type);

    return (
        <div
            title={isError ? (errorMessage ?? "Upload failed. Try again or use a smaller file.") : file.name}
            className={[
                "relative flex items-center gap-2 rounded-xl p-1.5 transition-colors",
                "h-14 min-w-[120px] max-w-[180px]",
                isError
                    ? "bg-destructive/10"
                    : "bg-muted",
            ].join(" ")}
        >
            {/* Thumbnail / icon */}
            <div className="h-10 w-10 flex-shrink-0 overflow-hidden rounded-lg bg-muted flex items-center justify-center text-muted-foreground">
                {isImage ? (
                    <ImageThumbnail file={file} uploaded={uploaded} />
                ) : (
                    <DocIcon mimeType={file.type} />
                )}
            </div>

            {/* Label */}
            <div className="flex min-w-0 flex-col">
                <span className="truncate text-xs font-medium leading-tight text-foreground" style={{ maxWidth: "100px" }}>
                    {file.name}
                </span>
                <span className="text-[10px] leading-tight text-muted-foreground">
                    {isUploading ? "Uploading..." : isError ? "Upload failed" : mimeLabel}
                </span>
            </div>

            {/* Upload spinner overlay */}
            {isUploading && (
                <span className="absolute inset-0 flex items-center justify-center rounded-xl bg-background/60">
                    <Loader2 className="h-4 w-4 animate-spin text-primary" />
                </span>
            )}

            {/* Remove button */}
            <button
                className="absolute right-0.5 top-0.5 flex h-4 w-4 items-center justify-center rounded-full bg-muted text-muted-foreground transition hover:bg-destructive hover:text-white"
                onClick={() => onRemove(localId)}
                aria-label={`Remove ${file.name}`}
                type="button"
            >
                <X className="h-2.5 w-2.5" />
            </button>

            {/* Status dot */}
            {!isUploading && (
                <span
                    className={[
                        "absolute bottom-1 right-1 h-2 w-2 rounded-full",
                        isError ? "bg-destructive" : "bg-success",
                    ].join(" ")}
                />
            )}
        </div>
    );
};

// ─────────────────────────────────────────────────────────────────────────────
// Image thumbnail
// ─────────────────────────────────────────────────────────────────────────────

interface ImgProps {
    file: File;
    uploaded?: { url: string };
}

const ImageThumbnail: FC<ImgProps> = ({ file, uploaded }) => {
    // Prefer the server-issued URL once the upload completes.
    const uploadedUrl = uploaded?.url ?? null;

    // Create a local preview URL only while the server URL isn't available yet.
    // useMemo handles creation; the returned cleanup revokes the previous URL.
    const objectUrl = useMemo(() => {
        if (uploadedUrl) return null;
        return URL.createObjectURL(file);
    }, [file, uploadedUrl]);

    // Revoke the object URL when the memo recomputes or the component unmounts.
    useEffect(() => {
        return () => {
            if (objectUrl) URL.revokeObjectURL(objectUrl);
        };
    }, [objectUrl]);

    const src = uploadedUrl ?? objectUrl;

    if (!src) return <FileImage className="h-5 w-5" />;

    return (
        <img
            src={src}
            alt={file.name}
            className="h-full w-full object-cover rounded-lg"
        />
    );
};

// ─────────────────────────────────────────────────────────────────────────────
// Document icon
// ─────────────────────────────────────────────────────────────────────────────

const DocIcon: FC<{ mimeType: string }> = ({ mimeType }) => {
    if (mimeType === "application/pdf")
        return <FileText className="h-5 w-5 text-destructive" />;
    if (
        mimeType === "application/msword" ||
        mimeType.includes("wordprocessingml")
    )
        return <FileText className="h-5 w-5 text-cyan" />;
    if (mimeType.startsWith("text/"))
        return <FileText className="h-5 w-5 text-fg-faint" />;
    return <File className="h-5 w-5" />;
};

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

function mimeToLabel(mimeType: string): string {
    if (mimeType === "application/pdf") return "PDF";
    if (
        mimeType === "application/msword" ||
        mimeType.includes("wordprocessingml")
    )
        return "Word";
    if (mimeType.startsWith("image/"))
        return mimeType.split("/")[1]?.toUpperCase() ?? "Image";
    if (mimeType === "text/plain") return "TXT";
    if (mimeType === "text/markdown") return "MD";
    if (mimeType === "application/json") return "JSON";
    return "File";
}
