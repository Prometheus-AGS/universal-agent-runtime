//! Phase acceptance against real provider implementations. No mock database.

use universal_agent_runtime::uar::a2ui::presentations::{
    Presentation, PresentationDraft, PresentationTemplate,
};
use universal_agent_runtime::uar::persistence::{
    PersistenceLayer, presentations::PresentationStoreError,
};

fn draft(title: &str) -> PresentationDraft {
    PresentationDraft {
        title: title.into(),
        description: "Persistent template fixture".into(),
        enabled: true,
        template: PresentationTemplate::default(),
    }
}

async fn catalog_contract(storage: &dyn PersistenceLayer, owner: &str) -> Presentation {
    assert!(storage.list_presentations(owner).await.unwrap().is_empty());
    let record = storage
        .create_presentation(owner, &draft("Initial"))
        .await
        .unwrap();
    assert_eq!(record.revision, 1);
    assert_eq!(record.owner_id, owner);
    assert_eq!(
        storage.get_presentation(owner, &record.id).await.unwrap(),
        Some(record.clone())
    );
    let other = format!("{owner}-other");
    assert!(
        storage
            .get_presentation(&other, &record.id)
            .await
            .unwrap()
            .is_none()
    );
    assert!(storage.list_presentations(&other).await.unwrap().is_empty());
    let foreign_write = storage
        .update_presentation(&other, &record.id, 1, &draft("Foreign"))
        .await
        .unwrap_err();
    assert!(matches!(
        foreign_write.downcast_ref::<PresentationStoreError>(),
        Some(PresentationStoreError::NotFound)
    ));
    assert!(
        storage
            .delete_presentation(&other, &record.id, 1)
            .await
            .is_err()
    );
    assert!(
        storage
            .create_presentation(owner, &draft(" "))
            .await
            .is_err()
    );
    assert!(
        storage
            .create_presentation(" ", &draft("No owner"))
            .await
            .is_err()
    );

    let updated = storage
        .update_presentation(owner, &record.id, 1, &draft("Revision two"))
        .await
        .unwrap();
    assert_eq!(updated.revision, 2);
    assert_eq!(updated.id, record.id);
    assert_eq!(updated.owner_id, record.owner_id);
    assert_eq!(updated.created_at, record.created_at);
    let stale = storage
        .update_presentation(owner, &record.id, 1, &draft("Stale"))
        .await
        .unwrap_err();
    assert!(matches!(
        stale.downcast_ref::<PresentationStoreError>(),
        Some(PresentationStoreError::Conflict)
    ));
    let stale_delete = storage
        .delete_presentation(owner, &record.id, 1)
        .await
        .unwrap_err();
    assert!(matches!(
        stale_delete.downcast_ref::<PresentationStoreError>(),
        Some(PresentationStoreError::Conflict)
    ));
    assert_eq!(
        storage.list_presentations(owner).await.unwrap(),
        vec![updated.clone()]
    );
    updated
}

async fn concurrent_writers_do_not_both_commit(storage: &dyn PersistenceLayer, owner: &str) {
    let record = storage
        .create_presentation(owner, &draft("Race"))
        .await
        .unwrap();
    let left = draft("Left");
    let right = draft("Right");
    let (left_result, right_result) = tokio::join!(
        storage.update_presentation(owner, &record.id, 1, &left),
        storage.update_presentation(owner, &record.id, 1, &right),
    );
    assert_eq!(
        usize::from(left_result.is_ok()) + usize::from(right_result.is_ok()),
        1
    );
    let winner = left_result.or(right_result).unwrap();
    assert_eq!(winner.revision, 2);
    assert_eq!(
        storage.get_presentation(owner, &record.id).await.unwrap(),
        Some(winner)
    );
    storage
        .delete_presentation(owner, &record.id, 2)
        .await
        .unwrap();
}

#[cfg(feature = "in-memory-backend")]
#[tokio::test]
async fn memory_catalog_contract() {
    use universal_agent_runtime::uar::persistence::providers::memory::InMemoryProvider;
    let provider = InMemoryProvider::new();
    let record = catalog_contract(&provider, "memory-owner").await;
    concurrent_writers_do_not_both_commit(&provider, "memory-race-owner").await;
    provider
        .delete_presentation(&record.owner_id, &record.id, record.revision)
        .await
        .unwrap();
    assert!(
        provider
            .list_presentations(&record.owner_id)
            .await
            .unwrap()
            .is_empty()
    );
}

#[cfg(feature = "surreal-backend")]
#[tokio::test]
async fn surreal_catalog_survives_process_restart_and_preserves_revision_contract() {
    use universal_agent_runtime::uar::persistence::providers::surreal::SurrealDbProvider;
    const MODE: &str = "UAR_PRESENTATION_RESTART_TEST_MODE";
    const ENDPOINT: &str = "UAR_PRESENTATION_RESTART_TEST_ENDPOINT";
    const OWNER: &str = "presentation-restart-owner";
    if let Ok(mode) = std::env::var(MODE) {
        let endpoint = std::env::var(ENDPOINT).expect("isolated restart fixture endpoint");
        let provider = SurrealDbProvider::new(
            &endpoint,
            None,
            None,
            Some("presentation-tests"),
            Some("restart"),
        )
        .await
        .expect("open the isolated SurrealKV fixture");
        match mode.as_str() {
            "seed" => {
                catalog_contract(&provider, OWNER).await;
                concurrent_writers_do_not_both_commit(&provider, "presentation-race-owner").await;
            }
            "update" => {
                let rows = provider.list_presentations(OWNER).await.unwrap();
                assert_eq!(rows.len(), 1);
                let record = &rows[0];
                assert_eq!(record.revision, 2);
                assert_eq!(record.content.title, "Revision two");
                let mut content = draft("After restart");
                content.enabled = false;
                let updated = provider
                    .update_presentation(OWNER, &record.id, 2, &content)
                    .await
                    .unwrap();
                assert_eq!(updated.revision, 3);
            }
            "delete" => {
                let rows = provider.list_presentations(OWNER).await.unwrap();
                assert_eq!(rows.len(), 1);
                assert_eq!(rows[0].revision, 3);
                assert_eq!(rows[0].content.title, "After restart");
                assert!(!rows[0].content.enabled);
                provider
                    .delete_presentation(OWNER, &rows[0].id, 3)
                    .await
                    .unwrap();
            }
            "empty" => assert!(provider.list_presentations(OWNER).await.unwrap().is_empty()),
            _ => panic!("unexpected restart fixture mode"),
        }
        return;
    }
    let directory = tempfile::tempdir().unwrap();
    let endpoint = format!(
        "surrealkv://{}",
        directory.path().join("presentations.db").display()
    );
    for mode in ["seed", "update", "delete", "empty"] {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "surreal_catalog_survives_process_restart_and_preserves_revision_contract",
                "--test-threads=1",
            ])
            .env(MODE, mode)
            .env(ENDPOINT, &endpoint)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "Presentation {mode} process failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[cfg(feature = "postgres-backend")]
#[tokio::test]
#[ignore = "requires a dedicated disposable UAR_PRESENTATION_TEST_DATABASE_URL; run explicitly for PostgreSQL acceptance"]
async fn postgres_catalog_contract_and_reconnection() {
    use universal_agent_runtime::uar::persistence::providers::postgres::PostgresProvider;
    let url = std::env::var("UAR_PRESENTATION_TEST_DATABASE_URL")
        .expect("dedicated disposable PostgreSQL database required");
    let owner = format!("presentation-test-{}", uuid::Uuid::new_v4());
    let provider = PostgresProvider::new(&url).await.unwrap();
    let record = catalog_contract(&provider, &owner).await;
    concurrent_writers_do_not_both_commit(&provider, &format!("{owner}-race")).await;
    provider.get_pool().close().await;
    let reopened = PostgresProvider::new(&url).await.unwrap();
    assert_eq!(
        reopened.get_presentation(&owner, &record.id).await.unwrap(),
        Some(record.clone())
    );
    reopened
        .delete_presentation(&owner, &record.id, record.revision)
        .await
        .unwrap();
    assert!(
        reopened
            .list_presentations(&owner)
            .await
            .unwrap()
            .is_empty()
    );
    reopened.get_pool().close().await;
}
