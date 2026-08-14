use crate::AppState;
use axum::{
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{DecodingKey, Validation};
use secrecy::ExposeSecret;

use super::{
    claims::{UserClaims, UserContext},
    jwt::{self, JwtError},
};

fn anonymous_context() -> UserContext {
    UserContext {
        user_id: "anonymous".to_string(),
        claims: UserClaims {
            sub: "anonymous".to_string(),
            name: Some("Anonymous".to_string()),
            roles: Some(vec!["anonymous".to_string()]),
            // far-future expiry for internal placeholder context
            exp: usize::MAX,
        },
    }
}

fn resolve_user_context(
    jwt_required: bool,
    jwt_secret: &str,
    auth_header: Option<&str>,
) -> Result<UserContext, StatusCode> {
    let token = match auth_header {
        Some(header_val) if header_val.starts_with("Bearer ") => {
            &header_val[7..] // Strip "Bearer "
        }
        _ => {
            return if jwt_required {
                Err(StatusCode::UNAUTHORIZED)
            } else {
                Ok(anonymous_context())
            };
        }
    };

    // Decode & validate token
    let key = DecodingKey::from_secret(jwt_secret.as_bytes());
    let validation = Validation::default();

    match jwt::decode::<UserClaims>(token, &key, &validation) {
        Ok(token_data) => {
            let claims = token_data.claims;
            Ok(UserContext {
                user_id: claims.sub.clone(),
                claims,
            })
        }
        Err(JwtError::ProviderConflict) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        Err(JwtError::Token(_)) => {
            if jwt_required {
                Err(StatusCode::UNAUTHORIZED)
            } else {
                Ok(anonymous_context())
            }
        }
    }
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Health probe endpoints must always be reachable without credentials
    // so that Kubernetes liveness and readiness probes pass.
    let path = request.uri().path();
    if matches!(path, "/health" | "/healthz" | "/readyz" | "/metrics") {
        return Ok(next.run(request).await);
    }

    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    // Try JWT first
    let mut context = resolve_user_context(
        false,
        state.config.security.jwt_secret.expose_secret(),
        auth_header,
    )?;

    // If still anonymous, try X-API-Key header
    if context.user_id == "anonymous" {
        if let Some(api_key) = request
            .headers()
            .get("x-api-key")
            .and_then(|v| v.to_str().ok())
        {
            if let Some(api_key_service) = &state.api_key_service {
                match api_key_service.validate_key(api_key).await {
                    Ok(Some(claims)) => {
                        context = UserContext {
                            user_id: claims.sub.clone(),
                            claims,
                        };
                    }
                    Ok(None) => {
                        // Invalid key — if JWT is required, reject; otherwise stay anonymous
                        if state.config.security.jwt_required {
                            return Err(StatusCode::UNAUTHORIZED);
                        }
                    }
                    Err(_) => {
                        if state.config.security.jwt_required {
                            return Err(StatusCode::UNAUTHORIZED);
                        }
                    }
                }
            }
        } else if state.config.security.jwt_required {
            // No auth at all and JWT required
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    request.extensions_mut().insert(context);
    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{EncodingKey, Header};

    #[test]
    fn test_resolve_user_context_anonymous_when_jwt_disabled_and_no_header() {
        let ctx = resolve_user_context(false, "secret", None).expect("expected context");
        assert_eq!(ctx.user_id, "anonymous");
    }

    #[test]
    fn test_resolve_user_context_anonymous_when_jwt_disabled_and_invalid_header() {
        let ctx = resolve_user_context(false, "secret", Some("Bearer invalid.token"))
            .expect("expected anonymous fallback context");
        assert_eq!(ctx.user_id, "anonymous");
    }

    #[test]
    fn test_resolve_user_context_unauthorized_when_jwt_required_and_no_header() {
        let err = resolve_user_context(true, "secret", None).expect_err("expected unauthorized");
        assert_eq!(err, StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_resolve_user_context_valid_token() {
        let claims = UserClaims {
            sub: "user-123".to_string(),
            name: Some("Test User".to_string()),
            roles: Some(vec!["user".to_string()]),
            exp: usize::MAX,
        };
        let token = jwt::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret("secret".as_bytes()),
        )
        .expect("token encode should succeed");
        let header_value = format!("Bearer {token}");

        let ctx =
            resolve_user_context(true, "secret", Some(&header_value)).expect("expected context");
        assert_eq!(ctx.user_id, "user-123");
    }

    #[test]
    fn test_resolve_user_context_rejects_token_signed_with_wrong_secret() {
        let claims = UserClaims {
            sub: "user-123".to_string(),
            name: Some("Test User".to_string()),
            roles: Some(vec!["user".to_string()]),
            exp: usize::MAX,
        };
        let token = jwt::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret("wrong-secret".as_bytes()),
        )
        .expect("token encode should succeed");
        let header_value = format!("Bearer {token}");

        let err = resolve_user_context(true, "secret", Some(&header_value))
            .expect_err("expected invalid signature to be rejected");
        assert_eq!(err, StatusCode::UNAUTHORIZED);
    }
}
