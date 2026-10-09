//! Admission policy for verified identities and untrusted workspace selectors.
use super::claims::UserContext;
use crate::config::SecurityConfig;

#[derive(Debug, thiserror::Error)]
pub enum AdmissionError {
    #[error("identity policy is unavailable")]
    Configuration,
    #[error("verified remote identity required")]
    Unauthenticated,
    #[error("workspace authority denied")]
    Forbidden,
}

/// Apply operator mappings only after the credential verifier established identity.
/// Local launch identity is deliberately not a remote tenant identity.
pub fn authorize_context(
    config: &SecurityConfig,
    user: Option<&UserContext>,
    workspace: Option<&str>,
) -> Result<(), AdmissionError> {
    config
        .validate_identity_policy()
        .map_err(|_| AdmissionError::Configuration)?;
    if !config.is_remote() {
        return Ok(());
    }
    let user = user.ok_or(AdmissionError::Unauthenticated)?;
    let identity = user
        .verified_identity()
        .ok_or(AdmissionError::Unauthenticated)?;
    if identity.issuer() != config.jwt_issuer.as_deref() {
        return Err(AdmissionError::Unauthenticated);
    }
    let tenant = user
        .tenant_id
        .as_ref()
        .ok_or(AdmissionError::Unauthenticated)?;
    if user.user_id.trim().is_empty() || user.user_id == "anonymous" {
        return Err(AdmissionError::Unauthenticated);
    }
    // In remote mode verify_token requires this exact issuer before constructing
    // UserContext. Request headers/body fields never supply an issuer or tenant.
    let permitted = config.workspace_authorities.iter().any(|entry| {
        Some(entry.issuer.as_str()) == config.jwt_issuer.as_deref()
            && entry.subject == user.user_id
            && entry.tenant_id == tenant.as_str()
            && workspace.is_none_or(|selected| entry.workspace_id == selected)
    });
    if permitted {
        Ok(())
    } else {
        Err(AdmissionError::Forbidden)
    }
}

#[cfg(test)]
mod scenarios {
    // Source scenarios exercise the real verifier and admission policy. They do
    // not substitute for the deferred HTTP/gRPC/A2A delivery acceptance gate.
    use super::*;
    use crate::uar::security::{jwt, verifier::verify_token};
    use jsonwebtoken::{EncodingKey, Header};
    use serde_json::{Value, json};

    fn config() -> SecurityConfig {
        serde_json::from_value(json!({
            "deployment_profile":"remote", "jwt_required":true,
            "jwt_algorithm":"HS256", "jwt_secret":"synthetic-policy-scenario",
            "jwt_issuer":"scenario-issuer", "jwt_audience":"scenario-runtime",
            "jwt_validate_nbf":true, "settings_mutation_auth_required":false,
            "workspace_authorities":[
                {"issuer":"scenario-issuer", "subject":"user-a", "tenant_id":"tenant-a", "workspace_id":"workspace-a"},
                {"issuer":"scenario-issuer", "subject":"user-b", "tenant_id":"tenant-b", "workspace_id":"workspace-b"}
            ]
        })).expect("explicit scenario config")
    }

    fn claims() -> Value {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("scenario clock")
            .as_secs();
        json!({"sub":"user-a", "tenant_id":"tenant-a", "roles":["user"],
            "iss":"scenario-issuer", "aud":"scenario-runtime", "exp":now+3600})
    }

    async fn context(
        config: &SecurityConfig,
        claims: Value,
    ) -> Result<UserContext, super::super::verifier::VerificationError> {
        let token = jwt::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(b"synthetic-policy-scenario"),
        )
        .expect("scenario token");
        let principal = verify_token(config, &token).await?;
        Ok(UserContext {
            host_authority: principal.host_authority,
            authority: principal.authority,
            user_id: principal.subject,
            tenant_id: principal.tenant_id,
            claims: principal.claims,
        })
    }

    #[tokio::test]
    async fn verified_identity_requires_exact_workspace_mapping() {
        let config = config();
        let user = context(&config, claims())
            .await
            .expect("verified scenario principal");
        assert!(authorize_context(&config, Some(&user), Some("workspace-a")).is_ok());
        assert!(matches!(
            authorize_context(&config, Some(&user), Some("workspace-b")),
            Err(AdmissionError::Forbidden)
        ));
        assert!(authorize_context(&config, Some(&user), Some("workspace-a")).is_ok());
        let mut unlisted = claims();
        unlisted["sub"] = json!("unlisted-user");
        let unlisted = context(&config, unlisted)
            .await
            .expect("valid but unmapped identity");
        assert!(matches!(
            authorize_context(&config, Some(&unlisted), None),
            Err(AdmissionError::Forbidden)
        ));
    }

    #[tokio::test]
    async fn incomplete_remote_policy_never_falls_back_to_anonymous() {
        let baseline = config();
        let mut variants = vec![];
        let mut config = baseline.clone();
        config.jwt_required = false;
        variants.push(config);
        let mut config = baseline.clone();
        config.jwt_issuer = None;
        variants.push(config);
        let mut config = baseline.clone();
        config.jwt_audience = None;
        variants.push(config);
        let mut config = baseline.clone();
        config.jwt_algorithm = None;
        variants.push(config);
        let mut config = baseline.clone();
        config.jwt_algorithm = Some(crate::config::JwtAlgorithm::RS256);
        variants.push(config);
        let mut config = baseline.clone();
        config.jwt_validate_nbf = false;
        variants.push(config);
        let mut config = baseline;
        config.workspace_authorities.clear();
        variants.push(config);
        for config in variants {
            assert!(context(&config, claims()).await.is_err());
            assert!(matches!(
                authorize_context(&config, None, None),
                Err(AdmissionError::Configuration)
            ));
        }
    }

    #[tokio::test]
    async fn remote_registered_claims_and_identity_are_required() {
        let config = config();
        for name in ["iss", "aud", "exp", "sub", "tenant_id"] {
            let mut claims = claims();
            claims.as_object_mut().expect("claims object").remove(name);
            assert!(context(&config, claims).await.is_err(), "missing {name}");
        }
        for name in ["iss", "aud"] {
            let mut claims = claims();
            claims[name] = json!("other");
            assert!(context(&config, claims).await.is_err(), "wrong {name}");
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("scenario clock")
            .as_secs();
        let mut future = claims();
        future["nbf"] = json!(now + 300);
        assert!(context(&config, future).await.is_err());
        let mut expired = claims();
        expired["exp"] = json!(now - 300);
        assert!(context(&config, expired).await.is_err());
        let mut leeway = claims();
        leeway["exp"] = json!(now - 30);
        assert!(context(&config, leeway).await.is_ok());
        let wrong_signature = jwt::encode(
            &Header::default(),
            &claims(),
            &EncodingKey::from_secret(b"different-synthetic-secret"),
        )
        .expect("scenario token");
        assert!(verify_token(&config, &wrong_signature).await.is_err());
        let wrong_algorithm = jwt::encode(
            &Header::new(jsonwebtoken::Algorithm::HS384),
            &claims(),
            &EncodingKey::from_secret(b"synthetic-policy-scenario"),
        )
        .expect("scenario token");
        assert!(verify_token(&config, &wrong_algorithm).await.is_err());
        // Existing policy validates nbf when present; it does not require it.
        assert!(context(&config, claims()).await.is_ok());
    }
}
