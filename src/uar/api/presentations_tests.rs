use super::*;
use crate::uar::a2ui::presentations::PresentationTemplate;
use crate::uar::persistence::providers::memory::InMemoryProvider;
use crate::uar::security::claims::{TenantId, UserClaims};
use axum_test::TestServer;

fn user(tenant: &str) -> UserContext {
    UserContext {
        user_id: "catalog-operator".into(),
        tenant_id: Some(TenantId::for_test(tenant)),
        claims: UserClaims {
            sub: "catalog-operator".into(),
            name: None,
            roles: None,
            tenant_id: Some(tenant.into()),
            uar_instance_id: None,
            exp: usize::MAX,
        },
    }
}

fn server(store: Arc<dyn PersistenceLayer>, principal: Option<UserContext>) -> TestServer {
    let mut router = build_router().with_state(store);
    if let Some(principal) = principal {
        router = router.layer(Extension(principal));
    }
    TestServer::new(router)
}

fn draft() -> PresentationDraft {
    PresentationDraft {
        title: "HTTP report".into(),
        description: "Route regression".into(),
        enabled: true,
        template: PresentationTemplate::default(),
    }
}

#[tokio::test]
async fn anonymous_and_inconsistent_verified_contexts_are_rejected() {
    let store: Arc<dyn PersistenceLayer> = Arc::new(InMemoryProvider::new());
    let anonymous = server(store.clone(), None);
    anonymous
        .get("/")
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    anonymous
        .post("/")
        .json(&draft())
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    anonymous
        .get("/unknown")
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    anonymous
        .put("/unknown")
        .json(&json!({"expected_revision": 1, "content": draft()}))
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    anonymous
        .delete("/unknown?expected_revision=1")
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    let mut inconsistent = user("tenant-a");
    inconsistent.claims.sub = "another-subject".into();
    server(store, Some(inconsistent))
        .get("/")
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn catalog_routes_round_trip_and_reject_stale_edits_and_deletes() {
    let api = server(Arc::new(InMemoryProvider::new()), Some(user("tenant-a")));
    let created = api.post("/").json(&draft()).await;
    created.assert_status(StatusCode::CREATED);
    let record: Presentation = created.json();
    assert_eq!(record.revision, 1);
    let path = format!("/{}", record.id);
    let listed: Value = api.get("/").await.json();
    assert_eq!(listed["owner_id"], record.owner_id);
    assert_eq!(listed["presentations"], json!([record]));
    let mut edited = draft();
    edited.title = "HTTP revision two".into();
    let update = json!({"expected_revision": 1, "content": edited});
    let response = api.put(&path).json(&update).await;
    response.assert_status_ok();
    let updated: Presentation = response.json();
    assert_eq!(updated.revision, 2);
    assert_eq!(updated.content.title, edited.title);
    let mut stale = draft();
    stale.title = "Stale content must not replace revision two".into();
    api.put(&path)
        .json(&json!({"expected_revision": 1, "content": stale}))
        .await
        .assert_status(StatusCode::CONFLICT);
    api.delete(&format!("{path}?expected_revision=1"))
        .await
        .assert_status(StatusCode::CONFLICT);
    assert_eq!(api.get(&path).await.json::<Presentation>(), updated);
    api.delete(&format!("{path}?expected_revision=2"))
        .await
        .assert_status(StatusCode::NO_CONTENT);
    api.get(&path).await.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn same_subject_in_another_tenant_cannot_read_update_or_delete() {
    let store: Arc<dyn PersistenceLayer> = Arc::new(InMemoryProvider::new());
    let owner = server(store.clone(), Some(user("tenant-a")));
    let other = server(store, Some(user("tenant-b")));
    let record: Presentation = owner.post("/").json(&draft()).await.json();
    let path = format!("/{}", record.id);
    let list: Value = other.get("/").await.json();
    assert_ne!(list["owner_id"], record.owner_id);
    assert_eq!(list["presentations"], json!([]));
    other.get(&path).await.assert_status(StatusCode::NOT_FOUND);
    other
        .put(&path)
        .json(&json!({"expected_revision": 1, "content": draft()}))
        .await
        .assert_status(StatusCode::NOT_FOUND);
    other
        .delete(&format!("{path}?expected_revision=1"))
        .await
        .assert_status(StatusCode::NOT_FOUND);
    assert_eq!(owner.get(&path).await.json::<Presentation>(), record);
}

#[tokio::test]
async fn invalid_drafts_and_forged_owner_fields_do_not_create_records() {
    let api = server(Arc::new(InMemoryProvider::new()), Some(user("tenant-a")));
    let mut invalid = draft();
    invalid.title = " ".into();
    api.post("/")
        .json(&invalid)
        .await
        .assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    let mut forged = serde_json::to_value(draft()).unwrap();
    forged["owner_id"] = json!("another-owner");
    api.post("/")
        .json(&forged)
        .await
        .assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    let body: Value = api.get("/").await.json();
    assert_eq!(body["presentations"], json!([]));
}
