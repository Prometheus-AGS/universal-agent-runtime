use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use base64::Engine as _;
use jsonwebtoken::{
    Algorithm, EncodingKey, Header,
    jwk::{Jwk, JwkSet},
};
use serde::Serialize;
use tokio::{net::TcpListener, sync::RwLock, task::JoinHandle};

use super::jwt;

const RSA_PRIVATE_KEY_DER: &str = "MIIEpAIBAAKCAQEAyRE6rHuNR0QbHO3H3Kt2pOKGVhQqGZXInOduQNxXzuKlvQTLUTv4l4sggh5/CYYi/cvI+SXVT9kPWSKXxJXBXd/4LkvcPuUakBoAkfh+eiFVMh2VrUyWyj3MFl0HTVF9KwRXLAcwkREiS3npThHRyIxuy0ZMeZfxVL5arMhw1SRELB8HoGfG/AtH89BIE9jDBHZ9dLelK9a184zAf8LwoPLxvJb3Il5nncqPcSfKDDodMFBIMc4lQzDKL5gvmiXLXB1AGLm8KBjfE8s3L5xqi+yUod+j8MtvIj812dkS4QMiRVN/by2h3ZY8LYVGrqZXZTcgn2ujn8uKjXLZVD5TdQIDAQABAoIBAHREk0I0O9DvECKdWUpAmF3mY7oY9PNQiu44Yaf+AoSuyRpRUGTMIgc3u3eivOE8ALX0BmYUO5JtuRNZDpvt4SAwqCnVUinIf6C+eH/wSurCpapSM0BAHp4aOA7igptyOMgMPYBHNA1e9A7jE0dCxKWMl3DSWNyjQTk4zeRGEAEfbNjHrq6YCtjHSZSLmWiG80hnfnYos9hOr5JnLnyS7ZmFE/5P3XVrxLc/tQ5zum0R4cbrgzHiQP5RgfxGJaEi7XcgherCCOgurJSSbYH29Gz8u5fFbS+Yg8s+OiCss3cs1rSgJ9/eHZuzGEdUZVARH6hVMjSuwvqVTFaE8AgtleECgYEA+uLMn4kNqHlJS2A5uAnCkj90ZxEtNm3E8hAxUrhssktY5XSOAPBlxyf5RuRGIImGtUVIr4HuJSa5TX48n3Vdt9MYCprO/iYl6moNRSPt5qowIIOJmIjY2mqPDfDt/zw+fcDD3lmCJrFlzcnh0uea1CohxEbQnL3cypeLt+WbU6kCgYEAzSp19m1ajieFkqgoB0YTpt/OroDx38vvI5unInJlEeOjQ+oIAQdN2wpxBvTrRorMU6P07mFUbt1j+Co6CbNiw+X8HcCaqYLR5clbJOOWNR36PuzOpQLkfK8woupBxzW9B8gZmY8rB1mbJ+/WTPrEJy6YGmIEBkWylQ2VpW8O4O0CgYEApdbvvfFBlwD9YxbrcGz7MeNCFbMz+MucqQntIKoKJ91ImPxvtc0y6e/Rhnv0oyNlaUOwJVu0yNgNG117w0g4t/+Q38mvVC5xV7/cn7x9UMFk6MkqVir3dYGEqIl/OP1grY2Tq9HtB5iyG9L8NIamQOLMyUqqMUILxdthHyFmiGkCgYEAn9+PjpjGMPHxL0gj8Q8VbzsFtou6b1deIRRA2CHmSltltR1gYVTMwXxQeUhPMmgkMqUXzs4/WijgpthY44hK1TaZEKIuoxrS70nJ4WQLf5a9k1065fDsFZD6yGjdGxvwEmlGMZgTwqV7t1I4X0Ilqhav5hcs5apYL7gnPYPeRz0CgYALHCj/Ji8XSsDoF/MhVhnGdIs2P99NNdmo3R2Pv0CuZbDKMU559LJHUvrKS8WkuWRDuKrz1W/EQKApFjDGpdqToZqriUFQzwy7mR3ayIiogzNtHcvbDHx8oFnGY0OFksX/ye0/XGpy2SFxYRwGU98HPYeBvAQQrVjdkzfy7BmXQQ==";

#[derive(Serialize)]
struct TestClaims<'a> {
    sub: &'a str,
    name: Option<&'a str>,
    roles: Option<Vec<&'a str>>,
    exp: usize,
    iss: &'a str,
    aud: &'a str,
}

const ROTATED_RSA_PRIVATE_KEY_DER: &str = "MIIEpAIBAAKCAQEAuQQyrIEWpVQtLqrrR8jf2yfwGTM/pL9adokrThQVpMx+PX10clnacNLjB0tYW0/nN6SaD8s2ZBwMq0WmZzhoUfvkl0DafQVdebfK2eo4s9L+ZxMcWV5fnxwHKvEouEK+cyAxp1To+DT9jFI3Gj/rDsBDXrmJ6l+r2syEryDn4Jn5t8i7IwNd57WUIqnWczRhXT/tJ2e92sDsttffH/tDe5d9DXqrVjwVoSqg5W/5ZMMl0Dz6g5lUYxcgkQJ5V6UjIQUBNJkLfhyw5SHY0x4T/Hs+L2sSy9WdGnDI88BmRndSNmt9lGk0Q+68MrkCmKJ/OLOKGS+HHv8A7OmNbReidQIDAQABAoIBAEXq+rVrESJIdcyphcF6fXJGHPuA/P+m2qpp+uYGPAmrx9c39k4Se7TgVTBX/lt/jireduQaEQNzACynZROj4vR8gy3PseHGKcWKOcvxMh1u0nokZDW3rt4jiufk+9TqUCuUkn8gXOwTpm+lUDKIzi0kZjFBX4elQP4uBMRj5IzhLsmEeNdc+QG1I7r7rAZJXdG0MnY008wlkm6ZdHXVsqoiTvGAYTuR/mcX6S+zfNIribX7TlbH2L0rERR0FrzFahkQ3UItIidDUIhzEG0Pp8rwuW43k0BPy4gN2yZRgTg14gbZtF+3QLvLqjY5eokfYdimcbCwJDlfw0PLZU94Vx8CgYEA3+t6mCXOMAr6KHExtcTB3jdQrBdiYJkt5O1xlM+xH9mCWWnd3CrX8Cch8BlwWujI1CbfLAOwGrUctAF5+LAekaQQjBWPIn2LM76EU/xZHphsBZPXfhN/ULvW3wL1B02GKT+yQ0q2mj0xiHq+zMo1/hMB3BHuiMgXLyx3nqcw4csCgYEA04Xj16CG7ABHKTpeEGGSBawSXZz07IlSVp3ZA9S2772BddEMc6vCJa659Z3NfeWOhTmow5FfacPaM9a0KC90bdUYUmvJ5dCA89e3KPTD4pPNxCweE1AZBmCPqGpbEbhVGK+8JTuwueIPluDV9Q1RQKRHDLcCA4yGyJpyPeFIBL8CgYEAuXAH7OySHtNYbBmh80hozSC+HGaZQCpbCZViVLzTkO7OtkGoTGbmwamGv5Ixq/fQKXGvrIG5W8TVanU2j687AZ3/XiOUkBmsKEQEzpDTNTVBcDUJZw26iB+nSLToOw4Gpy5q8LN1GbLHzKDqViq4IBuZlKj9BCXAnX6T6b3IC5UCgYBtd+Rjiqto5ffuCUv3FFfa4aObmQhUhfj75LMUPXjzd9LRI4BbOK/Air2otKNNnYj1v9JsbAbCGN8LZvlTtsN9uAPfW/NgIVkrWR9sbcgWscGS3fYuroxU9ZJDac95yzkXDpPDfTHH8Yt53SA9s0eyuZIfrXK4XXi/xtaK2dVIxwKBgQC4ctED39lE3e68JmkxyXMs8T/azVfM08kEOMtSVli0U5iNKoJ8kqlh1hZ3IwYHYgJKwc683RJ+62fuNm3O5NVsGlhujd/Av01ddizyPmpYIUrgB0U9xEzzN2u8z/EKnBzslLJxtuPXImqDBH2+wwlMLBsyaStIHOJG0e+Kh0fHpA==";

fn rotated_encoding_key() -> EncodingKey {
    let der = base64::engine::general_purpose::STANDARD
        .decode(ROTATED_RSA_PRIVATE_KEY_DER)
        .expect("synthetic rotated key decodes");
    EncodingKey::from_rsa_der(&der)
}

fn encoding_key() -> EncodingKey {
    let der = base64::engine::general_purpose::STANDARD
        .decode(RSA_PRIVATE_KEY_DER)
        .expect("test RSA key must decode");
    EncodingKey::from_rsa_der(&der)
}

pub(crate) fn jwk(kid: &str) -> Jwk {
    jwt::ensure_rustcrypto_provider().expect("RustCrypto must initialize for test JWK");
    let mut jwk = Jwk::from_encoding_key(&encoding_key(), Algorithm::RS256)
        .expect("test RSA key must produce a JWK");
    jwk.common.key_id = Some(kid.to_owned());
    jwk
}

pub(crate) fn signed_token(kid: &str, issuer: &str, audience: &str) -> String {
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(kid.to_owned());
    let claims = TestClaims {
        sub: "user-123",
        name: Some("Test User"),
        roles: Some(vec!["user"]),
        exp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must follow the Unix epoch")
            .as_secs()
            .saturating_add(3600) as usize,
        iss: issuer,
        aud: audience,
    };
    jwt::encode(&header, &claims, &encoding_key()).expect("test token must encode")
}

#[derive(Clone)]
struct TestServerState {
    keys: Arc<RwLock<JwkSet>>,
    requests: Arc<AtomicUsize>,
    failing: Arc<AtomicBool>,
    delay: Arc<RwLock<std::time::Duration>>,
    active: Arc<AtomicUsize>,
    max_active: Arc<AtomicUsize>,
}

async fn serve_jwks(State(state): State<TestServerState>) -> Response {
    state.requests.fetch_add(1, Ordering::SeqCst);
    let active = state.active.fetch_add(1, Ordering::SeqCst) + 1;
    state.max_active.fetch_max(active, Ordering::SeqCst);
    tokio::time::sleep(*state.delay.read().await).await;
    let response = if state.failing.load(Ordering::SeqCst) {
        StatusCode::SERVICE_UNAVAILABLE.into_response()
    } else {
        Json(state.keys.read().await.clone()).into_response()
    };
    state.active.fetch_sub(1, Ordering::SeqCst);
    response
}

pub(crate) struct TestJwksServer {
    pub(crate) url: String,
    keys: Arc<RwLock<JwkSet>>,
    requests: Arc<AtomicUsize>,
    task: JoinHandle<()>,
    state: TestServerState,
}

impl TestJwksServer {
    pub(crate) async fn start(keys: Vec<Jwk>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("test JWKS listener must bind");
        let address = listener
            .local_addr()
            .expect("test JWKS listener must have an address");
        let state = TestServerState {
            keys: Arc::new(RwLock::new(JwkSet { keys })),
            requests: Arc::new(AtomicUsize::new(0)),
            failing: Arc::new(AtomicBool::new(false)),
            delay: Arc::new(RwLock::new(std::time::Duration::ZERO)),
            active: Arc::new(AtomicUsize::new(0)),
            max_active: Arc::new(AtomicUsize::new(0)),
        };
        let app = Router::new()
            .route("/jwks", get(serve_jwks))
            .with_state(state.clone());
        let task = tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("test JWKS server must run");
        });

        Self {
            url: format!("http://{address}/jwks"),
            keys: state.keys.clone(),
            requests: state.requests.clone(),
            task,
            state,
        }
    }

    pub(crate) async fn replace(&self, keys: Vec<Jwk>) {
        *self.keys.write().await = JwkSet { keys };
    }

    pub(crate) fn fail(&self, failing: bool) {
        self.state.failing.store(failing, Ordering::SeqCst);
    }

    pub(crate) async fn delay(&self, delay: std::time::Duration) {
        *self.state.delay.write().await = delay;
    }

    pub(crate) fn max_active(&self) -> usize {
        self.state.max_active.load(Ordering::SeqCst)
    }

    pub(crate) fn request_count(&self) -> usize {
        self.requests.load(Ordering::SeqCst)
    }
}

impl Drop for TestJwksServer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

pub(crate) fn rotated_jwk(kid: &str) -> Jwk {
    jwt::ensure_rustcrypto_provider().expect("RustCrypto initialized");
    let mut key = Jwk::from_encoding_key(&rotated_encoding_key(), Algorithm::RS256).unwrap();
    key.common.key_id = Some(kid.to_owned());
    key
}

pub(crate) fn rotated_signed_token(kid: &str, issuer: &str, audience: &str) -> String {
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(kid.to_owned());
    let claims = TestClaims {
        sub: "user-123",
        name: None,
        roles: Some(vec!["user"]),
        exp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize
            + 3600,
        iss: issuer,
        aud: audience,
    };
    jwt::encode(&header, &claims, &rotated_encoding_key()).expect("synthetic rotated token")
}
