//! JWKS verification of ES256/ES384 bearer tokens at the A2A gRPC boundary.
//!
//! flint-gate mints ES256 JWTs and publishes its keys at
//! `/.well-known/jwks.json`. These tests serve a JWKS from a wiremock server,
//! point a real `GrpcAgentService` at it through `SecurityConfig::jwks_url`,
//! and observe the production verifier through the gRPC status it returns:
//! `Unauthenticated` when verification fails, any other outcome when the
//! caller was authenticated.
//!
//! The signing keys below are fixed, test-only PKCS#8 (EC) and PKCS#1 (RSA)
//! DER keys generated with openssl for this file.
use std::net::TcpListener as StdTcpListener;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::Engine as _;
use jsonwebtoken::{
    Algorithm, EncodingKey, Header,
    jwk::{Jwk, JwkSet},
};
use serde::Serialize;
use tokio::sync::RwLock;
use tonic::Request;
use universal_agent_runtime::config::{LlmConfig, SecurityConfig};
use universal_agent_runtime::llm::mock_driver::MockLlmDriver;
use universal_agent_runtime::mcp::registry::McpRegistry;
use universal_agent_runtime::session::SessionStore;
use universal_agent_runtime::uar::api::a2a::grpc::GrpcAgentService;
use universal_agent_runtime::uar::api::a2a::grpc::pb::agent_service_client::AgentServiceClient;
use universal_agent_runtime::uar::api::a2a::grpc::pb::{
    GetTaskRequest, Message, Part, SendMessageRequest, part,
};
use universal_agent_runtime::uar::api::a2a::{A2AState, thread_service::A2AThreadService};
use universal_agent_runtime::uar::persistence::{
    PersistenceLayer, providers::surreal::SurrealDbProvider,
};
use universal_agent_runtime::uar::rag::embeddings::{
    EmbeddingBackend, UnavailableEmbeddingBackend,
};
use universal_agent_runtime::uar::runtime::{
    actor::system::ActorCollaboration, manager::RunManager, matching::VectorMatcher,
    skills::SkillRegistry,
};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const ISSUER: &str = "https://gate.know-me.tools";
const AUDIENCE: &str = "uar";

const EC_P256_A_PRIVATE_KEY_DER: &str = "MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgR9zk3YYwUp8RUzyv0s1KsbPDxVLW6XGGw7qkJMUsQWahRANCAASRQcdumVgq0frBn+8PHVKLkDSUqV/conOHiPaFoKC2/BbhfYMG+oZNC1l4LicwbEUNACG0vF9K/2alJOvfc4XE";
const EC_P256_B_PRIVATE_KEY_DER: &str = "MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgEj4dgJL8LGyujlbSvgB/UWOvKFA9CJwMaAIDAKPdycqhRANCAATKZAVttEDER07HLjpBCAxLNUMWjZXAcijxtl9VNTcZREe0JKQ5R+G35BUOIWk96BCh19qdsLhCWlY4NQvlRKyX";
const EC_P384_PRIVATE_KEY_DER: &str = "MIG2AgEAMBAGByqGSM49AgEGBSuBBAAiBIGeMIGbAgEBBDDi80HWM9zy9m/4J9LOI4aUmCa2XAINetGdhcw+gRu9z6UhXkCQCOW6S2UutFTqlMahZANiAAQdym3uqMrWSpyQWbZQAm8X5+eQ4yQoPRtapIIKgwD/UpzGD4jBX7RGSRb7VEPMCW+a1fcbGSUWCnaujEUcN1zTZEr41CdRSNRgQKpWF5heQflsHj/IDrKoQBp+ZtpraK4=";
const RSA_PRIVATE_KEY_DER: &str = "MIIEpAIBAAKCAQEA28ltu9t3Np5Fyr9+esSWQzUz2+EmybyGJDA+6EAXwhF27e5L5MZPMFt4O3AZIGFH9I75uISJuGclxTqEmJ4FfXcS8AhzynLc1sltd5KlWn5RhEZeRAhdvWvaSPawot4U437R6LkHE0x6aORi+tWrz0m/Sj6XE5+eFcwCDh8t2IjcYpsTvN43CIxwDdpJfZ4MwxkS9NtybB9Yj9l2c7UrwmcZl2N4pqLsEPGRgqkAwNBY1MgVOZpgUBhRLTnrS+I7W1dPIzd5EiH5/OrJyUzDshLoYO8UJKCXkFvIk9uoVGiqU1yKNHsm2NRdil241trBTQcwbPqei4Rdp0M9El8V2wIDAQABAoIBAALM3zQMiMlOXO9HX1IrHQsAK4f6p2bcmwzs/HAzGNplJJHFfnwMtseT8sU3GWrbMnKAO9hJAAQ0dDu+EiBrqwA9OyWJxgfnTL8D0/w5BxhPEbTQvLS7Mo6OSDqzwe5hS/zWCdCgQuHREKIzfrtZa5X5h5FnmL8sQnRepAwQA6KFndMPmNTdegRYxVdDtpytavkeTadPdN2Rt1xTDD9tXLpWAS523HzXMgjZSwRKy0bSUCtqKo5YEZDBUutxnvqTlD72kS7nE5FaDuBtlZEsWm7wq23j1axOpU71cGtV/vGgYEoZQkYRcvswmMIKustj+Q9iPnEfmvNoanSF9bCWhVECgYEA+9feQIfZE3P1HkT2UvbHj+NK4fU6rN8r0BoPyhc2uXnfGbb61xnlB2HmuZQDoFfnBuPBqRjb3sXUn0uiCnH7NWOPRcqJHx3NCb8TXEZxE4oTblf4jUphmlvgUJMOjmGcVWd3jD6sudFMVYvFCgrq7f8olRd27Kojb63hibSBS3MCgYEA32ocNN3a3buXuMl8rlvoEgSC10l7UG0xyqufMFrGI+O/BsjJ1XRP3ezfWXBj5jMT8DlG0Igt0yVuaPwZvdb43BsYn7yMe4w0+74o+zqHMNUwHFq9orVbnpdH2KWeG5jewLHIDwdd5LV+2YENiuOdhaDcg509xTD6klLHHZWZwfkCgYBZUT7+re9cCdUWLikaVXGDY34sUzfDFcdJH+UXrFH5R/LLAO1HmmRy0NLuYENE+8fw1pfZa/qWsJzu/fjzMWeBkNTAUMt+4KfWXBD2ufjikCbCDKsXGRkykIEmsnEIKDA0zeRFNfk2Ubd74303SZX2YHc5IUBJQTIeKpIBr6XnZwKBgQDNIdKQP/vLh4kBZA8U0NI+aOHx5khRSlFjcz0Q2uf+4AfvpMCdOtRyQiG5L1aqcM+nzA9XPRJGQqIjxwWjpxSMlFyBnk+myM+FLc7XDaA/mB86iZ6BHN/ot6KCK18Gm9A7QYEdO3hcnMDB2JqkoeVqYo7WUbP7sMMBQvwMD+ZDkQKBgQDKflcbbd7Fge+2V3BSprFGVSWZeIr0Tvke3w2mKvjR2WG8OOajFSZ73WQ2K0cD2myef7HNFAvHQX36Z26Dxgz+u+nOSUwUsKdduMdyeWlumqLe+jNF7kxdSJyTnmO3QQ0YFMheMQzuteWzdPxC5aZZwUTYBT0UhwS7RuB3VDdl3w==";

/// A fixed test signing key and the JWS algorithm it signs with.
#[derive(Clone, Copy)]
enum TestKey {
    P256A,
    P256B,
    P384,
    Rsa,
}

impl TestKey {
    fn algorithm(self) -> Algorithm {
        match self {
            Self::P256A | Self::P256B => Algorithm::ES256,
            Self::P384 => Algorithm::ES384,
            Self::Rsa => Algorithm::RS256,
        }
    }

    fn encoding_key(self) -> EncodingKey {
        let encoded = match self {
            Self::P256A => EC_P256_A_PRIVATE_KEY_DER,
            Self::P256B => EC_P256_B_PRIVATE_KEY_DER,
            Self::P384 => EC_P384_PRIVATE_KEY_DER,
            Self::Rsa => RSA_PRIVATE_KEY_DER,
        };
        let der = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .expect("test key decodes");
        match self {
            Self::Rsa => EncodingKey::from_rsa_der(&der),
            _ => EncodingKey::from_ec_der(&der),
        }
    }

    /// The public JWK an issuer would publish for this key.
    fn public_jwk(self, kid: &str) -> Jwk {
        let mut jwk = Jwk::from_encoding_key(&self.encoding_key(), self.algorithm())
            .expect("test key produces a public JWK");
        jwk.common.key_id = Some(kid.to_owned());
        jwk
    }
}

#[derive(Serialize)]
struct TestClaims<'a> {
    sub: &'a str,
    exp: u64,
    iss: &'a str,
    aud: &'a str,
}

fn sign(algorithm: Algorithm, key: &EncodingKey, kid: &str, sub: &str, aud: &str) -> String {
    let mut header = Header::new(algorithm);
    header.kid = Some(kid.to_owned());
    let claims = TestClaims {
        sub,
        exp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock follows the Unix epoch")
            .as_secs()
            + 3600,
        iss: ISSUER,
        aud,
    };
    jsonwebtoken::encode(&header, &claims, key).expect("test token encodes")
}

fn signed(key: TestKey, kid: &str) -> String {
    sign(
        key.algorithm(),
        &key.encoding_key(),
        kid,
        "gate-user",
        AUDIENCE,
    )
}

fn integration_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .thread_stack_size(8 * 1024 * 1024)
        .enable_all()
        .build()
        .expect("JWKS integration runtime builds")
}

struct Harness {
    client: AgentServiceClient<tonic::transport::Channel>,
    _jwks: MockServer,
    _database: tempfile::TempDir,
}

/// Start a gRPC service that verifies against a JWKS built by `keys`.
///
/// `keys` runs only after UAR has verified once: deriving a JWK or signing a
/// token touches jsonwebtoken's process provider, and UAR must be the one
/// that installs it or it reports a provider conflict.
async fn harness(keys: impl FnOnce() -> Vec<Jwk>) -> Harness {
    let jwks = MockServer::start().await;

    let port = StdTcpListener::bind("127.0.0.1:0")
        .expect("free port binds")
        .local_addr()
        .expect("free port has an address")
        .port();
    let addr: std::net::SocketAddr = format!("127.0.0.1:{port}").parse().expect("address");
    let database = tempfile::tempdir().expect("JWKS test database directory");
    let endpoint = format!("surrealkv://{}", database.path().join("jwks.db").display());
    let persistence: Arc<dyn PersistenceLayer> = Arc::new(
        SurrealDbProvider::new(&endpoint, None, None, Some("jwks-test"), Some("jwks-test"))
            .await
            .expect("JWKS test database opens"),
    );
    let embeddings: Arc<dyn EmbeddingBackend> = Arc::new(UnavailableEmbeddingBackend::new(
        384,
        "embeddings are not exercised by JWKS tests",
    ));
    let manager = Arc::new(
        RunManager::new(
            LlmConfig::default(),
            Arc::new(McpRegistry::new_empty()),
            SessionStore::new(),
            Arc::new(RwLock::new(SkillRegistry::default())),
            Arc::new(VectorMatcher::new(embeddings, 0.75)),
            Some(persistence),
        )
        .await
        .with_llm_driver(Arc::new(MockLlmDriver::echo())),
    );
    let state = Arc::new(A2AState {
        threads: Arc::new(A2AThreadService::new(Arc::new(ActorCollaboration::new(
            manager,
        )))),
        security: SecurityConfig {
            jwt_required: false,
            jwt_secret: "unused-when-jwks-is-configured".to_owned().into(),
            jwks_url: Some(format!("{}/.well-known/jwks.json", jwks.uri())),
            jwt_issuer: Some(ISSUER.to_owned()),
            jwt_audience: Some(AUDIENCE.to_owned()),
            jwt_validate_nbf: true,
            settings_mutation_auth_required: true,
            settings_admin_key: None,
        },
        base_url: format!("http://127.0.0.1:{port}"),
    });
    let service = GrpcAgentService::new(state);
    tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(service.into_server())
            .serve(addr)
            .await
            .expect("gRPC test server runs");
    });

    let url = format!("http://127.0.0.1:{port}");
    let mut client = None;
    for _ in 0..50 {
        if let Ok(connected) = AgentServiceClient::connect(url.clone()).await {
            client = Some(connected);
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let mut client = client.expect("gRPC server becomes ready");

    // A malformed bearer goes through UAR's verifier, which installs the
    // process provider before rejecting the token without any key material.
    let status = client
        .task_get(bearer(missing_task(), "not-a-jwt"))
        .await
        .expect_err("a malformed bearer is rejected");
    assert_eq!(status.code(), tonic::Code::Unauthenticated);

    Mock::given(method("GET"))
        .and(path("/.well-known/jwks.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(JwkSet { keys: keys() }))
        .mount(&jwks)
        .await;

    Harness {
        client,
        _jwks: jwks,
        _database: database,
    }
}

fn bearer<T>(message: T, token: &str) -> Request<T> {
    let mut request = Request::new(message);
    request.metadata_mut().insert(
        "authorization",
        format!("Bearer {token}")
            .parse()
            .expect("authorization metadata parses"),
    );
    request
}

fn missing_task() -> GetTaskRequest {
    GetTaskRequest {
        task_id: "does-not-exist".to_owned(),
    }
}

/// gRPC status for a TaskGet of an unknown task: `NotFound` proves the caller
/// authenticated, `Unauthenticated` proves the token was rejected.
async fn probe(harness: &mut Harness, token: &str) -> tonic::Code {
    harness
        .client
        .task_get(bearer(missing_task(), token))
        .await
        .expect_err("an unknown task is never returned")
        .code()
}

#[test]
fn es256_token_from_jwks_authenticates_and_owns_its_task() {
    integration_runtime().block_on(async {
        let mut harness = harness(|| vec![TestKey::P256A.public_jwk("gate-es")]).await;
        let token = signed(TestKey::P256A, "gate-es");

        let sent = harness
            .client
            .message_send(bearer(
                SendMessageRequest {
                    task_id: String::new(),
                    message: Some(Message {
                        role: "user".to_owned(),
                        parts: vec![Part {
                            content: Some(part::Content::Text("hello from gate".to_owned())),
                            content_type: "text/plain".to_owned(),
                        }],
                    }),
                },
                &token,
            ))
            .await
            .expect("an ES256 caller can send a message")
            .into_inner();
        let fetched = harness
            .client
            .task_get(bearer(
                GetTaskRequest {
                    task_id: sent.task_id.clone(),
                },
                &token,
            ))
            .await
            .expect("the same ES256 subject can read its task")
            .into_inner();
        assert_eq!(fetched.task_id, sent.task_id);

        let other_subject = sign(
            Algorithm::ES256,
            &TestKey::P256A.encoding_key(),
            "gate-es",
            "someone-else",
            AUDIENCE,
        );
        let foreign = harness
            .client
            .task_get(bearer(
                GetTaskRequest {
                    task_id: sent.task_id,
                },
                &other_subject,
            ))
            .await
            .expect_err("a different subject cannot read the task");
        assert_eq!(foreign.code(), tonic::Code::NotFound);
    });
}

#[test]
fn es384_token_from_jwks_authenticates() {
    integration_runtime().block_on(async {
        let mut harness = harness(|| vec![TestKey::P384.public_jwk("gate-es384")]).await;
        let token = signed(TestKey::P384, "gate-es384");
        assert_eq!(probe(&mut harness, &token).await, tonic::Code::NotFound);
    });
}

#[test]
fn es256_token_with_wrong_audience_is_unauthenticated() {
    integration_runtime().block_on(async {
        let mut harness = harness(|| vec![TestKey::P256A.public_jwk("gate-es")]).await;
        let token = sign(
            Algorithm::ES256,
            &TestKey::P256A.encoding_key(),
            "gate-es",
            "gate-user",
            "another-service",
        );
        assert_eq!(
            probe(&mut harness, &token).await,
            tonic::Code::Unauthenticated
        );
    });
}

#[test]
fn es256_token_signed_by_unpublished_key_is_unauthenticated() {
    integration_runtime().block_on(async {
        let mut harness = harness(|| vec![TestKey::P256A.public_jwk("gate-es")]).await;
        let token = signed(TestKey::P256B, "gate-es");
        assert_eq!(
            probe(&mut harness, &token).await,
            tonic::Code::Unauthenticated
        );
    });
}

#[test]
fn es256_token_against_rsa_jwk_is_unauthenticated() {
    integration_runtime().block_on(async {
        let mut harness = harness(|| vec![TestKey::Rsa.public_jwk("shared-kid")]).await;
        let token = signed(TestKey::P256A, "shared-kid");
        assert_eq!(
            probe(&mut harness, &token).await,
            tonic::Code::Unauthenticated
        );
    });
}

#[test]
fn rs256_token_against_ec_jwk_is_unauthenticated() {
    integration_runtime().block_on(async {
        let mut harness = harness(|| vec![TestKey::P256A.public_jwk("shared-kid")]).await;
        let token = signed(TestKey::Rsa, "shared-kid");
        assert_eq!(
            probe(&mut harness, &token).await,
            tonic::Code::Unauthenticated
        );
    });
}

#[test]
fn rs256_token_from_jwks_still_authenticates() {
    integration_runtime().block_on(async {
        let mut harness = harness(|| vec![TestKey::Rsa.public_jwk("gate-rs")]).await;
        let token = signed(TestKey::Rsa, "gate-rs");
        assert_eq!(probe(&mut harness, &token).await, tonic::Code::NotFound);
    });
}

#[test]
fn hs256_token_is_unauthenticated_on_the_jwks_path() {
    integration_runtime().block_on(async {
        let mut harness = harness(|| vec![TestKey::P256A.public_jwk("gate-es")]).await;
        let token = sign(
            Algorithm::HS256,
            &EncodingKey::from_secret(b"unused-when-jwks-is-configured"),
            "gate-es",
            "gate-user",
            AUDIENCE,
        );
        assert_eq!(
            probe(&mut harness, &token).await,
            tonic::Code::Unauthenticated
        );
    });
}

#[test]
fn ec_jwk_declaring_a_contradicting_alg_is_not_trusted() {
    integration_runtime().block_on(async {
        let mut harness = harness(|| {
            let mut contradicting = TestKey::P256A.public_jwk("gate-es");
            contradicting.common.key_algorithm = Some(jsonwebtoken::jwk::KeyAlgorithm::ES384);
            vec![contradicting]
        })
        .await;
        let token = signed(TestKey::P256A, "gate-es");
        assert_eq!(
            probe(&mut harness, &token).await,
            tonic::Code::Unauthenticated
        );
    });
}
