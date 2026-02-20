//! File Upload API handlers.
//!
//! POST /api/upload  — save files to disk, record in DB, return metadata + URL.
//! GET  /api/attachments/{id} — serve a stored attachment by ID.

use axum::{
    body::Body,
    extract::{Multipart, Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Json, Response},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::AppState;

/// Maximum file size in bytes (50 MB).
const MAX_FILE_SIZE: usize = 50 * 1024 * 1024;
/// Maximum total for all files in a single request (100 MB).
const MAX_TOTAL_SIZE: usize = 100 * 1024 * 1024;
/// Maximum number of files per request.
const MAX_FILES: usize = 10;

/// Allowed MIME type prefixes.
const ALLOWED_MIME_PREFIXES: &[&str] = &[
    "image/",
    "text/",
    "application/pdf",
    "application/msword",
    "application/vnd.openxmlformats-officedocument",
    "application/json",
    "application/xml",
];

// ─────────────────────────────────────────────────────────────────────────────
// Response / error types
// ─────────────────────────────────────────────────────────────────────────────

/// Metadata returned to the client for each successfully uploaded file.
#[derive(Debug, Serialize, Clone)]
pub struct UploadedFile {
    pub id: String,
    pub filename: String,
    pub content_type: String,
    pub size: usize,
    pub is_image: bool,
    /// URL the client can use to retrieve this file: GET /api/attachments/{id}
    pub url: String,
    /// Extracted text content for non-image documents.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_content: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub files: Vec<UploadedFile>,
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct UploadError {
    pub error: String,
    pub code: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /api/upload
// ─────────────────────────────────────────────────────────────────────────────

/// Upload one or more files for attachment to a chat message.
///
/// Accepts `multipart/form-data`.  The optional `X-UAR-Session-ID` header
/// associates attachments with a session for later history retrieval.
pub async fn upload_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, (StatusCode, Json<UploadError>)> {
    let session_id = headers
        .get("X-UAR-Session-ID")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("anonymous")
        .to_string();

    // Resolve the upload directory from config.
    let upload_dir = PathBuf::from(&state.config.file_processing.upload_dir);
    if let Err(e) = tokio::fs::create_dir_all(&upload_dir).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(UploadError {
                error: format!("Cannot create upload directory: {e}"),
                code: "STORAGE_ERROR".to_string(),
            }),
        ));
    }

    let mut uploaded_files = Vec::new();
    let mut errors = Vec::new();
    let mut total_size: usize = 0;
    let mut file_count: usize = 0;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(UploadError {
                error: format!("Failed to read multipart field: {e}"),
                code: "MULTIPART_ERROR".to_string(),
            }),
        )
    })? {
        if file_count >= MAX_FILES {
            errors.push(format!("Maximum file count ({MAX_FILES}) exceeded"));
            break;
        }

        let filename = field
            .file_name()
            .map_or_else(|| format!("file_{}", Uuid::new_v4()), ToString::to_string);

        let content_type = field.content_type().map_or_else(
            || "application/octet-stream".to_string(),
            ToString::to_string,
        );

        // Validate MIME type.
        let is_allowed = ALLOWED_MIME_PREFIXES
            .iter()
            .any(|prefix| content_type.starts_with(prefix));
        if !is_allowed {
            errors.push(format!(
                "File '{filename}' has unsupported type: {content_type}"
            ));
            continue;
        }

        let data = field.bytes().await.map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(UploadError {
                    error: format!("Failed to read file '{filename}': {e}"),
                    code: "READ_ERROR".to_string(),
                }),
            )
        })?;

        let size = data.len();

        if size > MAX_FILE_SIZE {
            let max_mb = MAX_FILE_SIZE / (1024 * 1024);
            let size_mb = size / (1024 * 1024);
            errors.push(format!(
                "File '{filename}' exceeds max size ({size_mb}MB > {max_mb}MB)"
            ));
            continue;
        }

        if total_size + size > MAX_TOTAL_SIZE {
            let max_total_mb = MAX_TOTAL_SIZE / (1024 * 1024);
            errors.push(format!("Total size would exceed limit ({max_total_mb}MB)"));
            break;
        }

        total_size += size;
        file_count += 1;

        let file_id = Uuid::new_v4().to_string();
        let is_image = content_type.starts_with("image/");

        // Derive file extension from filename or content_type.
        let ext = PathBuf::from(&filename)
            .extension()
            .and_then(|e| e.to_str())
            .map(ToString::to_string)
            .unwrap_or_else(|| ext_from_mime(&content_type));
        let stored_name = format!("{file_id}.{ext}");
        let file_path = upload_dir.join(&stored_name);

        // Write bytes to disk.
        match tokio::fs::File::create(&file_path).await {
            Ok(mut f) => {
                if let Err(e) = f.write_all(&data).await {
                    errors.push(format!("Failed to write '{filename}': {e}"));
                    continue;
                }
            }
            Err(e) => {
                errors.push(format!("Failed to create file for '{filename}': {e}"));
                continue;
            }
        }

        // Extract text content for non-image documents.
        let text_content: Option<String> = if is_image {
            None
        } else if content_type.starts_with("text/")
            || content_type == "application/json"
            || content_type == "application/xml"
        {
            String::from_utf8(data.to_vec()).ok()
        } else {
            // PDF / Word / other binary — text extraction not yet implemented.
            // Future: dispatch to FileProcessorFactory here.
            None
        };

        let url = format!("/api/attachments/{file_id}");
        let file_path_str = file_path.to_string_lossy().into_owned();

        // Persist metadata to DB (best-effort; don't fail upload if DB is absent).
        if let Some(persistence) = &state.run_manager.persistence {
            let meta = crate::uar::persistence::AttachmentMeta {
                id: file_id.clone(),
                session_id: session_id.clone(),
                filename: filename.clone(),
                content_type: content_type.clone(),
                file_path: file_path_str,
                file_size: size as i64,
                is_image,
                text_content: text_content.clone(),
                created_at: chrono::Utc::now(),
            };
            if let Err(e) = persistence.insert_attachment(&meta).await {
                tracing::warn!("Failed to persist attachment metadata: {e}");
            }
        }

        tracing::info!(
            file_id = %file_id,
            filename = %filename,
            content_type = %content_type,
            size,
            is_image,
            "Uploaded and stored attachment"
        );

        uploaded_files.push(UploadedFile {
            id: file_id,
            filename,
            content_type,
            size,
            is_image,
            url,
            text_content,
        });
    }

    Ok(Json(UploadResponse {
        files: uploaded_files,
        errors,
    }))
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /api/attachments/{id}
// ─────────────────────────────────────────────────────────────────────────────

/// Serve a stored attachment by ID.
pub async fn serve_attachment_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    // Look up metadata in DB.
    let meta = match &state.run_manager.persistence {
        Some(p) => match p.get_attachment(&id).await {
            Ok(Some(m)) => m,
            Ok(None) => {
                return (StatusCode::NOT_FOUND, "Attachment not found").into_response();
            }
            Err(e) => {
                tracing::error!("DB error fetching attachment {id}: {e}");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        },
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "Persistence not configured",
            )
                .into_response();
        }
    };

    // Open file from disk.
    let file = match tokio::fs::File::open(&meta.file_path).await {
        Ok(f) => f,
        Err(e) => {
            tracing::error!("Could not open attachment file {}: {e}", meta.file_path);
            return StatusCode::NOT_FOUND.into_response();
        }
    };

    // Stream the file body.
    let stream = tokio_util::io::ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let mut resp = Response::new(body);
    let ct = meta
        .content_type
        .parse::<HeaderValue>()
        .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream"));
    resp.headers_mut().insert(header::CONTENT_TYPE, ct);

    // Allow browsers to cache attachments (they're immutable once uploaded).
    resp.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=31536000, immutable"),
    );

    // Suggest a download filename if the client requests it.
    let disposition = format!("inline; filename=\"{}\"", meta.filename);
    if let Ok(v) = HeaderValue::from_str(&disposition) {
        resp.headers_mut().insert(header::CONTENT_DISPOSITION, v);
    }

    resp
}

// ─────────────────────────────────────────────────────────────────────────────
// Legacy re-export types (used by ChatCompletionRequest)
// ─────────────────────────────────────────────────────────────────────────────

/// Attachment reference sent inside a chat completion request.
#[derive(Debug, Deserialize, Clone)]
pub struct AttachmentInput {
    pub id: String,
    pub filename: String,
    pub content_type: String,
    pub url: String,
    /// Pre-extracted text (for documents already processed client-side).
    pub text_content: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn ext_from_mime(content_type: &str) -> String {
    match content_type {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/svg+xml" => "svg",
        "application/pdf" => "pdf",
        "application/msword" => "doc",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => "docx",
        "text/plain" => "txt",
        "text/html" => "html",
        "text/markdown" => "md",
        "application/json" => "json",
        _ => "bin",
    }
    .to_string()
}
