//! Private identity fixtures. Every HTTP assertion uses the standalone binary.
use super::{sidecar_process, stub_llm};
use base64::Engine as _;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, jwk::Jwk};
use reqwest::Method;
use serde_json::{Value, json};
use std::{
    sync::{
        Once,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};
#[path = "bauar_identity_jwks.rs"]
mod jwks;
pub use jwks::{JwksPeer, Reply, after};

pub const ISSUER: &str = "bauar-identity-fixture";
pub const AUDIENCE: &str = "bauar-standalone";
pub const WORKSPACE: &str = "workspace-a";
pub const INPUT: &str = "identity-boundary-content";
const RSA_A: &str = "MIIEpAIBAAKCAQEAyRE6rHuNR0QbHO3H3Kt2pOKGVhQqGZXInOduQNxXzuKlvQTLUTv4l4sggh5/CYYi/cvI+SXVT9kPWSKXxJXBXd/4LkvcPuUakBoAkfh+eiFVMh2VrUyWyj3MFl0HTVF9KwRXLAcwkREiS3npThHRyIxuy0ZMeZfxVL5arMhw1SRELB8HoGfG/AtH89BIE9jDBHZ9dLelK9a184zAf8LwoPLxvJb3Il5nncqPcSfKDDodMFBIMc4lQzDKL5gvmiXLXB1AGLm8KBjfE8s3L5xqi+yUod+j8MtvIj812dkS4QMiRVN/by2h3ZY8LYVGrqZXZTcgn2ujn8uKjXLZVD5TdQIDAQABAoIBAHREk0I0O9DvECKdWUpAmF3mY7oY9PNQiu44Yaf+AoSuyRpRUGTMIgc3u3eivOE8ALX0BmYUO5JtuRNZDpvt4SAwqCnVUinIf6C+eH/wSurCpapSM0BAHp4aOA7igptyOMgMPYBHNA1e9A7jE0dCxKWMl3DSWNyjQTk4zeRGEAEfbNjHrq6YCtjHSZSLmWiG80hnfnYos9hOr5JnLnyS7ZmFE/5P3XVrxLc/tQ5zum0R4cbrgzHiQP5RgfxGJaEi7XcgherCCOgurJSSbYH29Gz8u5fFbS+Yg8s+OiCss3cs1rSgJ9/eHZuzGEdUZVARH6hVMjSuwvqVTFaE8AgtleECgYEA+uLMn4kNqHlJS2A5uAnCkj90ZxEtNm3E8hAxUrhssktY5XSOAPBlxyf5RuRGIImGtUVIr4HuJSa5TX48n3Vdt9MYCprO/iYl6moNRSPt5qowIIOJmIjY2mqPDfDt/zw+fcDD3lmCJrFlzcnh0uea1CohxEbQnL3cypeLt+WbU6kCgYEAzSp19m1ajieFkqgoB0YTpt/OroDx38vvI5unInJlEeOjQ+oIAQdN2wpxBvTrRorMU6P07mFUbt1j+Co6CbNiw+X8HcCaqYLR5clbJOOWNR36PuzOpQLkfK8woupBxzW9B8gZmY8rB1mbJ+/WTPrEJy6YGmIEBkWylQ2VpW8O4O0CgYEApdbvvfFBlwD9YxbrcGz7MeNCFbMz+MucqQntIKoKJ91ImPxvtc0y6e/Rhnv0oyNlaUOwJVu0yNgNG117w0g4t/+Q38mvVC5xV7/cn7x9UMFk6MkqVir3dYGEqIl/OP1grY2Tq9HtB5iyG9L8NIamQOLMyUqqMUILxdthHyFmiGkCgYEAn9+PjpjGMPHxL0gj8Q8VbzsFtou6b1deIRRA2CHmSltltR1gYVTMwXxQeUhPMmgkMqUXzs4/WijgpthY44hK1TaZEKIuoxrS70nJ4WQLf5a9k1065fDsFZD6yGjdGxvwEmlGMZgTwqV7t1I4X0Ilqhav5hcs5apYL7gnPYPeRz0CgYALHCj/Ji8XSsDoF/MhVhnGdIs2P99NNdmo3R2Pv0CuZbDKMU559LJHUvrKS8WkuWRDuKrz1W/EQKApFjDGpdqToZqriUFQzwy7mR3ayIiogzNtHcvbDHx8oFnGY0OFksX/ye0/XGpy2SFxYRwGU98HPYeBvAQQrVjdkzfy7BmXQQ==";
const RSA_B: &str = "MIIEpAIBAAKCAQEAuQQyrIEWpVQtLqrrR8jf2yfwGTM/pL9adokrThQVpMx+PX10clnacNLjB0tYW0/nN6SaD8s2ZBwMq0WmZzhoUfvkl0DafQVdebfK2eo4s9L+ZxMcWV5fnxwHKvEouEK+cyAxp1To+DT9jFI3Gj/rDsBDXrmJ6l+r2syEryDn4Jn5t8i7IwNd57WUIqnWczRhXT/tJ2e92sDsttffH/tDe5d9DXqrVjwVoSqg5W/5ZMMl0Dz6g5lUYxcgkQJ5V6UjIQUBNJkLfhyw5SHY0x4T/Hs+L2sSy9WdGnDI88BmRndSNmt9lGk0Q+68MrkCmKJ/OLOKGS+HHv8A7OmNbReidQIDAQABAoIBAEXq+rVrESJIdcyphcF6fXJGHPuA/P+m2qpp+uYGPAmrx9c39k4Se7TgVTBX/lt/jireduQaEQNzACynZROj4vR8gy3PseHGKcWKOcvxMh1u0nokZDW3rt4jiufk+9TqUCuUkn8gXOwTpm+lUDKIzi0kZjFBX4elQP4uBMRj5IzhLsmEeNdc+QG1I7r7rAZJXdG0MnY008wlkm6ZdHXVsqoiTvGAYTuR/mcX6S+zfNIribX7TlbH2L0rERR0FrzFahkQ3UItIidDUIhzEG0Pp8rwuW43k0BPy4gN2yZRgTg14gbZtF+3QLvLqjY5eokfYdimcbCwJDlfw0PLZU94Vx8CgYEA3+t6mCXOMAr6KHExtcTB3jdQrBdiYJkt5O1xlM+xH9mCWWnd3CrX8Cch8BlwWujI1CbfLAOwGrUctAF5+LAekaQQjBWPIn2LM76EU/xZHphsBZPXfhN/ULvW3wL1B02GKT+yQ0q2mj0xiHq+zMo1/hMB3BHuiMgXLyx3nqcw4csCgYEA04Xj16CG7ABHKTpeEGGSBawSXZz07IlSVp3ZA9S2772BddEMc6vCJa659Z3NfeWOhTmow5FfacPaM9a0KC90bdUYUmvJ5dCA89e3KPTD4pPNxCweE1AZBmCPqGpbEbhVGK+8JTuwueIPluDV9Q1RQKRHDLcCA4yGyJpyPeFIBL8CgYEAuXAH7OySHtNYbBmh80hozSC+HGaZQCpbCZViVLzTkO7OtkGoTGbmwamGv5Ixq/fQKXGvrIG5W8TVanU2j687AZ3/XiOUkBmsKEQEzpDTNTVBcDUJZw26iB+nSLToOw4Gpy5q8LN1GbLHzKDqViq4IBuZlKj9BCXAnX6T6b3IC5UCgYBtd+Rjiqto5ffuCUv3FFfa4aObmQhUhfj75LMUPXjzd9LRI4BbOK/Air2otKNNnYj1v9JsbAbCGN8LZvlTtsN9uAPfW/NgIVkrWR9sbcgWscGS3fYuroxU9ZJDac95yzkXDpPDfTHH8Yt53SA9s0eyuZIfrXK4XXi/xtaK2dVIxwKBgQC4ctED39lE3e68JmkxyXMs8T/azVfM08kEOMtSVli0U5iNKoJ8kqlh1hZ3IwYHYgJKwc683RJ+62fuNm3O5NVsGlhujd/Av01ddizyPmpYIUrgB0U9xEzzN2u8z/EKnBzslLJxtuPXImqDBH2+wwlMLBsyaStIHOJG0e+Kh0fHpA==";

fn crypto() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        jsonwebtoken::crypto::rust_crypto::DEFAULT_PROVIDER
            .install_default()
            .expect("fixture RustCrypto provider")
    });
}

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn claims(subject: &str, tenant: &str) -> Value {
    json!({"sub":subject,"tenant_id":tenant,"roles":["user","fixture:invoke"],
        "iss":ISSUER,"aud":AUDIENCE,"exp":now()+3600,"uar_credential_kind":"issuer"})
}

pub fn local_token(claims: &Value) -> String {
    crypto();
    jsonwebtoken::encode(
        &Header::new(Algorithm::HS256),
        claims,
        &EncodingKey::from_secret(sidecar_process::JWT_SECRET.as_bytes()),
    )
    .unwrap()
}

fn rsa_key(rotated: bool) -> EncodingKey {
    let der = base64::engine::general_purpose::STANDARD
        .decode(if rotated { RSA_B } else { RSA_A })
        .unwrap();
    EncodingKey::from_rsa_der(&der)
}

pub fn rsa_token(kid: &str, rotated: bool, claims: &Value) -> String {
    crypto();
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(kid.to_owned());
    jsonwebtoken::encode(&header, claims, &rsa_key(rotated)).unwrap()
}

pub fn jwk(kid: &str, rotated: bool) -> Value {
    crypto();
    let mut key = Jwk::from_encoding_key(&rsa_key(rotated), Algorithm::RS256).unwrap();
    key.common.key_id = Some(kid.to_owned());
    serde_json::to_value(key).unwrap()
}

pub fn exchange_claims(token: &str) -> Value {
    crypto();
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_issuer(&[ISSUER]);
    validation.set_audience(&[AUDIENCE]);
    jsonwebtoken::decode::<Value>(
        token,
        &DecodingKey::from_secret(sidecar_process::JWT_SECRET.as_bytes()),
        &validation,
    )
    .unwrap()
    .claims
}

pub fn security(jwks: Option<&str>) -> Value {
    let mut policy = json!({"deployment_profile":if jwks.is_some() {"remote"} else {"trusted_local"},
        "jwt_required":true,"jwt_algorithm":if jwks.is_some() {"RS256"} else {"HS256"},
        "jwt_secret":sidecar_process::JWT_SECRET,"jwt_issuer":ISSUER,"jwt_audience":AUDIENCE,
        "jwt_validate_nbf":true,"settings_mutation_auth_required":false,
        "api_key_delegable_roles":["user","fixture:invoke"],
        "api_key_admin_principals":[{"issuer":ISSUER,"subject":"admin-a","tenant_id":"tenant-a"}],
        "trusted_host_principals":[{"issuer":ISSUER,"subject":"host-a","tenant_id":"tenant-a","host_id":"identity-service-host"}],
        "workspace_authorities":[]});
    let mappings = policy["workspace_authorities"].as_array_mut().unwrap();
    for (subject, tenant) in [
        ("owner-a", "tenant-a"),
        ("owner-b", "tenant-a"),
        ("owner-a", "tenant-b"),
        ("admin-a", "tenant-a"),
        ("admin-a", "tenant-b"),
        ("host-a", "tenant-a"),
    ] {
        mappings.push(
            json!({"issuer":ISSUER,"subject":subject,"tenant_id":tenant,"workspace_id":WORKSPACE}),
        );
    }
    if let Some(url) = jwks {
        policy["jwks_url"] = json!(url);
    }
    policy
}

pub enum Credential<'a> {
    Bearer(&'a str),
    Key(&'a str),
    None,
}

pub struct Host {
    pub process: sidecar_process::ServerProcess,
    pub workspace: sidecar_process::Workspace,
    pub model: stub_llm::StubLlmServer,
    client: reqwest::Client,
    response_ordinal: AtomicUsize,
}

pub fn config(
    workspace: &sidecar_process::Workspace,
    base: &str,
    policy: Value,
) -> (std::path::PathBuf, u16) {
    let options = sidecar_process::ConfigOptions::new(base);
    let mut config: Value =
        serde_norway::from_str(&sidecar_process::render_config(workspace, &options)).unwrap();
    config["security"] = policy;
    (
        workspace.write_config("identity.yaml", &serde_norway::to_string(&config).unwrap()),
        options.port,
    )
}

impl Host {
    pub async fn start(policy: Value, receiver: Option<&str>) -> Self {
        let mut fixtures = stub_llm::FixtureSet::new();
        for has_tools in [false, true] {
            for has_tool_result in [false, true] {
                fixtures = fixtures.with(
                    stub_llm::RequestFingerprint {
                        model: sidecar_process::STUB_MODEL.to_owned(),
                        last_user_message: INPUT.to_owned(),
                        has_tools,
                        has_tool_result,
                    },
                    stub_llm::FixtureResponse::Content("identity-authorized".to_owned()),
                );
            }
        }
        let model = stub_llm::start_stub_llm(fixtures).await;
        let workspace = sidecar_process::Workspace::new();
        if let Some(url) = receiver {
            std::fs::write(workspace.work().join("mcp.json"), serde_json::to_vec(&json!({
                "mcpServers":{"resource":{"url":url,"grant_policy":{
                    "destination_id":"identity-resource","trusted_hosts":["identity-service-host"],
                    "required_scopes":["fixture:invoke"],"allow_private_http":true}}}
            })).unwrap()).unwrap();
        }
        let (path, port) = config(&workspace, &model.base_url, policy);
        let process = sidecar_process::launch_standalone(
            &workspace,
            &sidecar_process::LaunchOptions::new(path, "identity"),
            port,
        )
        .await;
        Self {
            process,
            workspace,
            model,
            client: reqwest::Client::new(),
            response_ordinal: AtomicUsize::new(0),
        }
    }

    pub fn request(
        &self,
        method: Method,
        path: &str,
        credential: Credential<'_>,
        workspace: Option<&str>,
    ) -> reqwest::RequestBuilder {
        let request = self
            .client
            .request(method, format!("{}{path}", self.process.base_url()));
        let request = match credential {
            Credential::Bearer(token) => request.bearer_auth(token),
            Credential::Key(key) => request.header("x-api-key", key),
            Credential::None => request,
        };
        if let Some(workspace) = workspace {
            request.header("x-uar-workspace-id", workspace)
        } else {
            request
        }
    }

    pub async fn call(
        &self,
        method: Method,
        path: &str,
        credential: Credential<'_>,
        workspace: Option<&str>,
        body: Value,
    ) -> (u16, Value) {
        let endpoint = match path {
            "/api/uar/runs" => "run_admission",
            "/api/uar/auth/keys" => "key_collection",
            "/api/uar/auth/exchange" => "key_exchange",
            path if path
                .strip_prefix("/api/uar/auth/keys/")
                .is_some_and(|id| !id.is_empty() && !id.contains('/')) =>
            {
                "key_record"
            }
            _ => "other",
        };
        let method_label = match method.as_str() {
            "GET" => "GET",
            "POST" => "POST",
            "DELETE" => "DELETE",
            "PUT" => "PUT",
            "PATCH" => "PATCH",
            _ => "OTHER",
        };
        self.response(
            self.request(method, path, credential, workspace)
                .json(&body),
            method_label,
            endpoint,
        )
        .await
    }

    pub async fn exchange_key(&self, key: &str) -> (u16, Value) {
        // Optional<Json<_>> requires no Content-Type when the JSON body is absent.
        self.response(
            self.request(
                Method::POST,
                "/api/uar/auth/exchange",
                Credential::Key(key),
                None,
            ),
            "POST",
            "key_exchange",
        )
        .await
    }

    async fn response(
        &self,
        request: reqwest::RequestBuilder,
        method: &'static str,
        endpoint: &'static str,
    ) -> (u16, Value) {
        // Counts this host's call/exchange helpers; excludes startup probes and SSE.
        let ordinal = self.response_ordinal.fetch_add(1, Ordering::SeqCst) + 1;
        let response = request.send().await.unwrap_or_else(|error| {
            self.transport_failure("send", method, endpoint, ordinal, &error)
        });
        let status = response.status().as_u16();
        let text = response.text().await.unwrap_or_else(|error| {
            self.transport_failure("response-body", method, endpoint, ordinal, &error)
        });
        (
            status,
            serde_json::from_str(&text).unwrap_or_else(|_| json!({"body":text})),
        )
    }

    fn transport_failure(
        &self,
        stage: &str,
        method: &str,
        endpoint: &str,
        ordinal: usize,
        error: &reqwest::Error,
    ) -> ! {
        let pid = sysinfo::Pid::from_u32(self.process.pid());
        let mut system = sysinfo::System::new();
        system.refresh_processes_specifics(
            sysinfo::ProcessesToUpdate::Some(&[pid]),
            true,
            sysinfo::ProcessRefreshKind::nothing(),
        );
        let status = system.process(pid).map(|process| process.status());
        // Never emit captured messages: panic payloads can contain request credentials.
        let output = format!("{}\n{}", self.process.stdout(), self.process.stderr());
        // Fixed markers come from Rust's runtime diagnostics and src/main.rs.
        // These report captured text, not an inferred exit code or root cause.
        let fatal_markers = [
            (
                "stack_overflow",
                output.contains("has overflowed its stack")
                    || output.contains("fatal runtime error: stack overflow"),
            ),
            (
                "fatal_runtime_error",
                output.contains("fatal runtime error:"),
            ),
            ("abort", output.contains("aborting")),
            (
                "panic",
                output.contains("panicked at ") || output.contains("non-unwinding panic"),
            ),
            (
                "startup_error",
                [
                    "Failed to load selected environment file",
                    "Failed to initialize telemetry:",
                    "Failed to load configuration:",
                    "Windows service failed:",
                ]
                .iter()
                .any(|marker| output.contains(marker)),
            ),
            ("server_error", output.contains("Server error:")),
            (
                "allocation_failure",
                output.lines().any(|line| {
                    line.strip_prefix("memory allocation of ")
                        .and_then(|text| text.strip_suffix(" bytes failed"))
                        .is_some_and(|bytes| bytes.parse::<usize>().is_ok())
                }),
            ),
        ];
        let sources: Vec<_> = output
            .lines()
            .filter_map(|line| {
                let (_, location) = line.split_once(" panicked at ")?;
                let location = location.trim_end_matches(':');
                let mut parts = location.rsplitn(3, ':');
                let column = parts.next()?.parse::<u32>().ok()?;
                let row = parts.next()?.parse::<u32>().ok()?;
                let path = parts.next()?;
                let source = path.rsplit_once("/src/").map_or(path, |(_, source)| source);
                if !source.ends_with(".rs")
                    || !source
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || b"/_-.".contains(&byte))
                {
                    return None;
                }
                Some(format!("{source}:{row}:{column}"))
            })
            .collect();
        panic!(
            "identity fixture transport failure; stage={stage}; endpoint={endpoint}; method={method}; response_ordinal={ordinal}; timeout={}; connect={}; child_observed={}; os_status={status:?}; exit_status=unavailable; fatal_markers={fatal_markers:?}; panic_sources={sources:?}",
            error.is_timeout(),
            error.is_connect(),
            status.is_some()
        );
    }

    pub async fn run(
        &self,
        credential: Credential<'_>,
        workspace: Option<&str>,
        resource: Option<Value>,
    ) -> (u16, Value) {
        let mut body = json!({"artifact":sidecar_process::test_agent(None),"input":INPUT});
        if let Some(resource) = resource {
            body["mcp_servers"] = json!([resource]);
        }
        self.call(Method::POST, "/api/uar/runs", credential, workspace, body)
            .await
    }

    pub async fn finish(&self, credential: Credential<'_>, workspace: Option<&str>, run: &str) {
        let mut response = self
            .request(
                Method::GET,
                &format!("/api/uar/runs/{run}/stream"),
                credential,
                workspace,
            )
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success());
        let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
        let mut text = String::new();
        loop {
            let chunk = tokio::time::timeout_at(deadline, response.chunk())
                .await
                .unwrap()
                .unwrap()
                .expect("terminal event");
            text.push_str(&String::from_utf8_lossy(&chunk));
            assert!(!text.contains("event: agui.error"), "run failed: {text}");
            if text.contains("event: agui.done") {
                assert!(
                    text.contains("identity-authorized"),
                    "actual provider content absent: {text}"
                );
                return;
            }
        }
    }

    pub async fn model_calls(&self) -> usize {
        let body: Value = self
            .client
            .get(format!(
                "{}/_stub/requests",
                self.model.base_url.trim_end_matches("/v1")
            ))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        body["requests"].as_array().expect("stub request log").len()
    }
}
