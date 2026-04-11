const UPLOAD_URL = "/api/upload";

export interface UploadApiResponse {
  files: import("@/types").UploadedFileResponse[];
  errors: string[];
}

export async function postUploadAttachment(sessionId: string, file: File): Promise<UploadApiResponse> {
  const form = new FormData();
  form.append("file", file, file.name);
  const res = await fetch(UPLOAD_URL, {
    method: "POST",
    headers: { "X-UAR-Session-ID": sessionId },
    body: form,
  });
  if (!res.ok) {
    const body = await res.text().catch(() => res.statusText);
    throw new Error(body);
  }
  return res.json() as Promise<UploadApiResponse>;
}
