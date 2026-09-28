use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use serde_json::Value;

use super::TaskReceipt;

#[derive(Debug, Serialize)]
struct ErrorEnvelope {
    error: ErrorBody,
}
#[derive(Debug, Serialize)]
struct ErrorBody {
    code: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    task_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    admission_id: Option<String>,
}
#[derive(Debug)]
pub(crate) struct ApiError {
    status: StatusCode,
    body: ErrorBody,
}
impl ApiError {
    fn new(
        status: StatusCode,
        code: impl Into<String>,
        message: impl Into<String>,
        task_id: Option<String>,
        admission_id: Option<String>,
    ) -> Self {
        Self {
            status,
            body: ErrorBody {
                code: code.into(),
                message: message.into(),
                task_id,
                admission_id,
            },
        }
    }
    pub(super) fn from_rejected_receipt(status: StatusCode, receipt: &TaskReceipt) -> Self {
        let diagnostic = receipt.diagnostics.first();
        Self::new(
            status,
            diagnostic.map_or("admission_rejected", |item| item.code.as_str()),
            diagnostic.map_or("run admission was rejected", |item| item.message.as_str()),
            Some(receipt.task_id.clone()),
            Some(receipt.admission_id.clone()),
        )
    }
    pub(super) fn bad_request(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, code, message, None, None)
    }
    pub(super) fn unauthorized(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, code, message, None, None)
    }
    pub(super) fn conflict(
        code: &'static str,
        message: impl Into<String>,
        task_id: Option<String>,
        admission_id: Option<String>,
    ) -> Self {
        Self::new(StatusCode::CONFLICT, code, message, task_id, admission_id)
    }
    pub(super) fn not_found(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, code, message, None, None)
    }
    pub(super) fn gone(
        code: &'static str,
        message: impl Into<String>,
        task_id: Option<String>,
        admission_id: Option<String>,
    ) -> Self {
        Self::new(StatusCode::GONE, code, message, task_id, admission_id)
    }
    pub(super) fn unprocessable(
        code: &'static str,
        message: impl Into<String>,
        task_id: Option<String>,
    ) -> Self {
        Self::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            code,
            message,
            task_id,
            None,
        )
    }
}
impl ApiError {
    pub(crate) fn code(&self) -> &str {
        &self.body.code
    }
    pub(crate) fn data(&self) -> Value {
        serde_json::json!({ "uar_code": &self.body.code, "task_id": &self.body.task_id, "admission_id": &self.body.admission_id })
    }
}
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(ErrorEnvelope { error: self.body })).into_response()
    }
}
impl std::fmt::Display for ApiError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.body.code, self.body.message)
    }
}
