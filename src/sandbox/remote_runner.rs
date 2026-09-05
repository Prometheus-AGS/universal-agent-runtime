use async_trait::async_trait;

use super::runner::*;
use super::types::*;

#[derive(Debug)]
pub struct RemoteRunner {
    client: reqwest::Client,
    base_url: String,
    auth_token: Option<String>,
}

impl RemoteRunner {
    #[must_use]
    pub fn new(base_url: String, auth_token: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
            auth_token,
        }
    }

    fn build_request(&self, method: &str) -> reqwest::RequestBuilder {
        let url = format!("{}/{method}", self.base_url);
        let mut req = self.client.post(&url);
        if let Some(ref token) = self.auth_token {
            req = req.bearer_auth(token);
        }
        req
    }
}

#[async_trait]
impl SandboxRunner for RemoteRunner {
    fn enforces_isolation(&self) -> bool {
        // The configured remote sandbox service is the trusted enforcement
        // boundary. This client submits SandboxConfig to it and never executes
        // a command locally; this declaration is not remote attestation.
        true
    }

    async fn create(&self, config: SandboxConfig) -> Result<SandboxHandle, SandboxError> {
        let resp = self
            .build_request("create")
            .json(&config)
            .send()
            .await
            .map_err(|e| SandboxError::CreationFailed(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(SandboxError::CreationFailed(format!(
                "HTTP {status}: {body}"
            )));
        }

        #[derive(serde::Deserialize)]
        struct CreateResponse {
            id: String,
        }

        let cr: CreateResponse = resp
            .json()
            .await
            .map_err(|e| SandboxError::CreationFailed(e.to_string()))?;

        Ok(SandboxHandle {
            id: cr.id,
            session_id: None,
        })
    }

    async fn execute(
        &self,
        handle: &SandboxHandle,
        request: ExecutionRequest,
    ) -> Result<ExecutionResult, SandboxError> {
        #[derive(serde::Serialize)]
        struct ExecPayload {
            sandbox_id: String,
            #[serde(flatten)]
            request: ExecutionRequest,
        }

        let resp = self
            .build_request("execute")
            .json(&ExecPayload {
                sandbox_id: handle.id.clone(),
                request,
            })
            .send()
            .await
            .map_err(|e| SandboxError::ExecutionFailed(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(SandboxError::ExecutionFailed(format!(
                "HTTP {status}: {body}"
            )));
        }

        resp.json()
            .await
            .map_err(|e| SandboxError::ExecutionFailed(e.to_string()))
    }

    async fn write_file(
        &self,
        handle: &SandboxHandle,
        path: &str,
        content: &[u8],
    ) -> Result<(), SandboxError> {
        #[derive(serde::Serialize)]
        struct WritePayload {
            sandbox_id: String,
            path: String,
            content_base64: String,
        }

        use base64::Engine as _;
        let encoded = base64::engine::general_purpose::STANDARD.encode(content);

        let resp = self
            .build_request("write_file")
            .json(&WritePayload {
                sandbox_id: handle.id.clone(),
                path: path.to_string(),
                content_base64: encoded,
            })
            .send()
            .await
            .map_err(|e| SandboxError::FileError(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(SandboxError::FileError(format!("HTTP {status}: {body}")));
        }

        Ok(())
    }

    async fn read_file(&self, handle: &SandboxHandle, path: &str) -> Result<Vec<u8>, SandboxError> {
        #[derive(serde::Serialize)]
        struct ReadPayload {
            sandbox_id: String,
            path: String,
        }

        #[derive(serde::Deserialize)]
        struct ReadResponse {
            content_base64: String,
        }

        let resp = self
            .build_request("read_file")
            .json(&ReadPayload {
                sandbox_id: handle.id.clone(),
                path: path.to_string(),
            })
            .send()
            .await
            .map_err(|e| SandboxError::FileError(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(SandboxError::FileError(format!("HTTP {status}: {body}")));
        }

        let rr: ReadResponse = resp
            .json()
            .await
            .map_err(|e| SandboxError::FileError(e.to_string()))?;

        use base64::Engine as _;
        base64::engine::general_purpose::STANDARD
            .decode(&rr.content_base64)
            .map_err(|e| SandboxError::FileError(format!("base64 decode error: {e}")))
    }

    async fn destroy(&self, handle: SandboxHandle) -> Result<(), SandboxError> {
        #[derive(serde::Serialize)]
        struct DestroyPayload {
            sandbox_id: String,
        }

        let resp = self
            .build_request("destroy")
            .json(&DestroyPayload {
                sandbox_id: handle.id,
            })
            .send()
            .await
            .map_err(|e| SandboxError::ExecutionFailed(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(SandboxError::ExecutionFailed(format!(
                "HTTP {status}: {body}"
            )));
        }

        Ok(())
    }

    fn capabilities(&self) -> RunnerCapabilities {
        RunnerCapabilities {
            supports_long_running: true,
            supports_networking: true,
            max_execution_seconds: None,
            runner_type: RunnerType::Remote,
        }
    }
}
