//! Integration tests for Knowledge Base functionality.
//!
//! These tests verify the full knowledge base lifecycle including:
//! - KB CRUD operations
//! - Document management
//! - Scoped vector search
//! - Agent-scoped RAG retrieval
//!
//! Requires: `DATABASE_URL` environment variable pointing to a Postgres instance with pgvector.

use serial_test::serial;
use std::sync::Arc;
#[cfg(feature = "postgres-backend")]
use universal_agent_runtime::uar::persistence::providers::postgres::PostgresProvider;
use universal_agent_runtime::uar::{
    defaults::ensure_default_knowledge_base,
    domain::knowledge::{
        DocumentStatus, KbConfig, KnowledgeBase, KnowledgeChunk, KnowledgeDocument,
    },
    persistence::PersistenceLayer,
};
use uuid::Uuid;

const TEST_OWNER: &str = "knowledge-integration-owner";

// =============================================================================
// Test Utilities
// =============================================================================

/// Get the database URL from environment, or skip test if not set.
#[cfg_attr(not(feature = "postgres-backend"), allow(dead_code))]
fn get_database_url() -> Option<String> {
    std::env::var("DATABASE_URL").ok()
}

/// Create a test persistence layer.
async fn setup_persistence() -> Option<Arc<dyn PersistenceLayer>> {
    #[cfg(feature = "postgres-backend")]
    {
        let url = get_database_url()?;
        let provider = PostgresProvider::new(&url).await.ok()?;
        return Some(Arc::new(provider));
    }
    #[cfg(not(feature = "postgres-backend"))]
    None
}

/// Create a test knowledge base with a random name.
fn create_test_kb(suffix: &str) -> KnowledgeBase {
    let now = chrono::Utc::now().to_rfc3339();
    let suffix_id = Uuid::new_v4().to_string();
    let suffix_short = &suffix_id[..8];
    KnowledgeBase {
        id: Uuid::new_v4().to_string(),
        owner_id: TEST_OWNER.to_string(),
        name: format!("test-kb-{suffix}-{suffix_short}"),
        description: Some(format!("Test knowledge base for {suffix}")),
        config: KbConfig::default(),
        created_at: now.clone(),
        updated_at: now,
    }
}

/// Create a test document.
fn create_test_document(kb_id: &str, filename: &str) -> KnowledgeDocument {
    let now = chrono::Utc::now().to_rfc3339();
    KnowledgeDocument {
        id: Uuid::new_v4().to_string(),
        owner_id: TEST_OWNER.to_string(),
        kb_id: kb_id.to_string(),
        filename: filename.to_string(),
        file_path: Some(format!("/data/test/{filename}")),
        mime_type: Some("text/plain".to_string()),
        chunk_count: 0,
        status: DocumentStatus::Pending,
        created_at: now.clone(),
        updated_at: now,
    }
}

/// Create a test knowledge chunk.
fn create_test_chunk(
    kb_id: &str,
    doc_id: Option<&str>,
    content: &str,
    embedding: Vec<f32>,
) -> KnowledgeChunk {
    KnowledgeChunk {
        id: Uuid::new_v4(),
        owner_id: TEST_OWNER.to_string(),
        kb_id: kb_id.to_string(),
        document_id: doc_id.map(String::from),
        content: content.to_string(),
        metadata: Some(serde_json::json!({"test": true})),
        embedding,
        created_at: chrono::Utc::now().to_rfc3339(),
    }
}

fn make_embedding(pattern: &[f32]) -> Vec<f32> {
    pattern.iter().cycle().take(384).copied().collect()
}

// =============================================================================
// Knowledge Base CRUD Tests
// =============================================================================

#[tokio::test]
#[serial]
async fn test_equal_ids_are_partitioned_by_owner_in_postgres() {
    let Some(persistence) = setup_persistence().await else {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    };
    let shared_kb_id = Uuid::new_v4().to_string();
    let shared_doc_id = Uuid::new_v4().to_string();
    let shared_chunk_id = Uuid::new_v4();
    let now = chrono::Utc::now().to_rfc3339();

    for owner in ["postgres-alice", "postgres-bob"] {
        let kb = KnowledgeBase {
            id: shared_kb_id.clone(),
            owner_id: owner.to_string(),
            name: format!("{owner}-{}", Uuid::new_v4()),
            description: None,
            config: KbConfig::default(),
            created_at: now.clone(),
            updated_at: now.clone(),
        };
        let document = KnowledgeDocument {
            id: shared_doc_id.clone(),
            owner_id: owner.to_string(),
            kb_id: shared_kb_id.clone(),
            filename: format!("{owner}.txt"),
            file_path: None,
            mime_type: Some("text/plain".to_string()),
            chunk_count: 1,
            status: DocumentStatus::Indexed,
            created_at: now.clone(),
            updated_at: now.clone(),
        };
        let chunk = KnowledgeChunk {
            id: shared_chunk_id,
            owner_id: owner.to_string(),
            kb_id: shared_kb_id.clone(),
            document_id: Some(shared_doc_id.clone()),
            content: format!("{owner} secret"),
            metadata: None,
            embedding: make_embedding(&[1.0, 0.0, 0.0]),
            created_at: now.clone(),
        };
        persistence.save_knowledge_base(&kb).await.unwrap();
        persistence.save_document(&document).await.unwrap();
        persistence.save_chunk(&chunk).await.unwrap();
    }

    for owner in ["postgres-alice", "postgres-bob"] {
        assert_eq!(
            persistence
                .get_knowledge_base(owner, &shared_kb_id)
                .await
                .unwrap()
                .unwrap()
                .owner_id,
            owner
        );
        assert_eq!(
            persistence
                .get_document(owner, &shared_doc_id)
                .await
                .unwrap()
                .unwrap()
                .owner_id,
            owner
        );
        let matches = persistence
            .search_knowledge(owner, &make_embedding(&[1.0, 0.0, 0.0]), 10, -1.0)
            .await
            .unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].chunk.owner_id, owner);
    }

    let bob_only_kb = KnowledgeBase {
        id: Uuid::new_v4().to_string(),
        owner_id: "postgres-bob".to_string(),
        name: format!("postgres-bob-only-{}", Uuid::new_v4()),
        description: None,
        config: KbConfig::default(),
        created_at: now.clone(),
        updated_at: now.clone(),
    };
    persistence.save_knowledge_base(&bob_only_kb).await.unwrap();
    let cross_owner_document = KnowledgeDocument {
        id: Uuid::new_v4().to_string(),
        owner_id: "postgres-alice".to_string(),
        kb_id: bob_only_kb.id.clone(),
        filename: "must-fail.txt".to_string(),
        file_path: None,
        mime_type: Some("text/plain".to_string()),
        chunk_count: 0,
        status: DocumentStatus::Pending,
        created_at: now.clone(),
        updated_at: now,
    };
    assert!(
        persistence
            .save_document(&cross_owner_document)
            .await
            .is_err(),
        "a document must not reference another owner's KB"
    );

    persistence
        .delete_knowledge_base("postgres-bob", &shared_kb_id)
        .await
        .unwrap();
    assert!(
        persistence
            .get_knowledge_base("postgres-alice", &shared_kb_id)
            .await
            .unwrap()
            .is_some()
    );
    persistence
        .delete_knowledge_base("postgres-alice", &shared_kb_id)
        .await
        .unwrap();
    persistence
        .delete_knowledge_base("postgres-bob", &bob_only_kb.id)
        .await
        .unwrap();
}

#[tokio::test]
#[serial]
async fn test_kb_create_and_retrieve() {
    let Some(persistence) = setup_persistence().await else {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    };

    // Create a knowledge base
    let kb = create_test_kb("crud");
    persistence
        .save_knowledge_base(&kb)
        .await
        .expect("Failed to save KB");

    // Retrieve by ID
    let retrieved = persistence
        .get_knowledge_base(TEST_OWNER, &kb.id)
        .await
        .expect("Failed to get KB")
        .expect("KB not found");

    assert_eq!(retrieved.id, kb.id);
    assert_eq!(retrieved.name, kb.name);
    assert_eq!(retrieved.description, kb.description);

    // Retrieve by name
    let by_name = persistence
        .get_knowledge_base_by_name(TEST_OWNER, &kb.name)
        .await
        .expect("Failed to get KB by name")
        .expect("KB not found by name");

    assert_eq!(by_name.id, kb.id);

    // Cleanup
    persistence
        .delete_knowledge_base(TEST_OWNER, &kb.id)
        .await
        .expect("Failed to delete KB");
}

#[tokio::test]
#[serial]
async fn test_kb_list() {
    let Some(persistence) = setup_persistence().await else {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    };

    // Create multiple KBs
    let kb1 = create_test_kb("list-1");
    let kb2 = create_test_kb("list-2");

    persistence
        .save_knowledge_base(&kb1)
        .await
        .expect("Failed to save KB1");
    persistence
        .save_knowledge_base(&kb2)
        .await
        .expect("Failed to save KB2");

    // List all
    let all_kbs = persistence
        .list_knowledge_bases(TEST_OWNER)
        .await
        .expect("Failed to list KBs");

    let kb_ids: Vec<&str> = all_kbs.iter().map(|k| k.id.as_str()).collect();
    assert!(kb_ids.contains(&kb1.id.as_str()));
    assert!(kb_ids.contains(&kb2.id.as_str()));

    // Cleanup
    persistence
        .delete_knowledge_base(TEST_OWNER, &kb1.id)
        .await
        .expect("Failed to delete KB1");
    persistence
        .delete_knowledge_base(TEST_OWNER, &kb2.id)
        .await
        .expect("Failed to delete KB2");
}

#[tokio::test]
#[serial]
async fn test_kb_update() {
    let Some(persistence) = setup_persistence().await else {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    };

    // Create a KB
    let mut kb = create_test_kb("update");
    persistence
        .save_knowledge_base(&kb)
        .await
        .expect("Failed to save KB");

    // Update it
    kb.description = Some("Updated description".to_string());
    kb.updated_at = chrono::Utc::now().to_rfc3339();
    persistence
        .save_knowledge_base(&kb)
        .await
        .expect("Failed to update KB");

    // Verify update
    let retrieved = persistence
        .get_knowledge_base(TEST_OWNER, &kb.id)
        .await
        .expect("Failed to retrieve KB")
        .expect("KB not found");

    assert_eq!(
        retrieved.description,
        Some("Updated description".to_string())
    );

    // Cleanup
    persistence
        .delete_knowledge_base(TEST_OWNER, &kb.id)
        .await
        .expect("Failed to delete KB");
}

#[tokio::test]
#[serial]
async fn test_kb_delete_cascade() {
    let Some(persistence) = setup_persistence().await else {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    };

    // Create KB, document, and chunks
    let kb = create_test_kb("cascade");
    persistence
        .save_knowledge_base(&kb)
        .await
        .expect("Failed to save KB");

    let doc = create_test_document(&kb.id, "test.txt");
    persistence
        .save_document(&doc)
        .await
        .expect("Failed to save document");

    let chunk = create_test_chunk(&kb.id, Some(&doc.id), "Test content", vec![0.1; 384]);
    persistence
        .save_chunk(&chunk)
        .await
        .expect("Failed to save chunk");

    // Delete KB - should cascade to documents and chunks
    persistence
        .delete_knowledge_base(TEST_OWNER, &kb.id)
        .await
        .expect("Failed to delete KB");

    // Verify KB is gone
    let kb_result = persistence
        .get_knowledge_base(TEST_OWNER, &kb.id)
        .await
        .expect("Failed to check KB");
    assert!(kb_result.is_none());

    // Verify document is gone
    let doc_result = persistence
        .get_document(TEST_OWNER, &doc.id)
        .await
        .expect("Failed to check document");
    assert!(doc_result.is_none());
}

// =============================================================================
// Document Lifecycle Tests
// =============================================================================

#[tokio::test]
#[serial]
async fn test_document_lifecycle() {
    let Some(persistence) = setup_persistence().await else {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    };

    // Create KB
    let kb = create_test_kb("doc-lifecycle");
    persistence
        .save_knowledge_base(&kb)
        .await
        .expect("Failed to save KB");

    // Create document
    let doc = create_test_document(&kb.id, "lifecycle.txt");
    persistence
        .save_document(&doc)
        .await
        .expect("Failed to save document");

    // Verify initial status
    let retrieved = persistence
        .get_document(TEST_OWNER, &doc.id)
        .await
        .expect("Failed to get document")
        .unwrap();
    assert!(matches!(retrieved.status, DocumentStatus::Pending));

    // Update status to Processing
    persistence
        .update_document_status(TEST_OWNER, &doc.id, &DocumentStatus::Processing)
        .await
        .expect("Failed to update status");

    let processing = persistence
        .get_document(TEST_OWNER, &doc.id)
        .await
        .expect("Failed to get document")
        .unwrap();
    assert!(matches!(processing.status, DocumentStatus::Processing));

    // Update status to Indexed
    persistence
        .update_document_status(TEST_OWNER, &doc.id, &DocumentStatus::Indexed)
        .await
        .expect("Failed to update status");

    let indexed = persistence
        .get_document(TEST_OWNER, &doc.id)
        .await
        .expect("Failed to get document")
        .unwrap();
    assert!(matches!(indexed.status, DocumentStatus::Indexed));

    // Test failed status
    let error_status = DocumentStatus::Failed {
        error: "Test error".to_string(),
    };
    persistence
        .update_document_status(TEST_OWNER, &doc.id, &error_status)
        .await
        .expect("Failed to update status");

    let failed = persistence
        .get_document(TEST_OWNER, &doc.id)
        .await
        .expect("Failed to get document")
        .unwrap();
    match failed.status {
        DocumentStatus::Failed { error } => assert_eq!(error, "Test error"),
        _ => panic!("Expected Failed status"),
    }

    // Cleanup
    persistence
        .delete_knowledge_base(TEST_OWNER, &kb.id)
        .await
        .expect("Failed to delete KB");
}

#[tokio::test]
#[serial]
async fn test_document_list() {
    let Some(persistence) = setup_persistence().await else {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    };

    // Create KB
    let kb = create_test_kb("doc-list");
    persistence
        .save_knowledge_base(&kb)
        .await
        .expect("Failed to save KB");

    // Create multiple documents
    let doc1 = create_test_document(&kb.id, "doc1.txt");
    let doc2 = create_test_document(&kb.id, "doc2.txt");
    let doc3 = create_test_document(&kb.id, "doc3.txt");

    persistence
        .save_document(&doc1)
        .await
        .expect("Failed to save doc1");
    persistence
        .save_document(&doc2)
        .await
        .expect("Failed to save doc2");
    persistence
        .save_document(&doc3)
        .await
        .expect("Failed to save doc3");

    // List documents in KB
    let docs = persistence
        .list_documents(TEST_OWNER, &kb.id)
        .await
        .expect("Failed to list documents");
    assert_eq!(docs.len(), 3);

    let doc_ids: Vec<&str> = docs.iter().map(|d| d.id.as_str()).collect();
    assert!(doc_ids.contains(&doc1.id.as_str()));
    assert!(doc_ids.contains(&doc2.id.as_str()));
    assert!(doc_ids.contains(&doc3.id.as_str()));

    // Cleanup
    persistence
        .delete_knowledge_base(TEST_OWNER, &kb.id)
        .await
        .expect("Failed to delete KB");
}

// =============================================================================
// Scoped Search Tests
// =============================================================================

#[tokio::test]
#[serial]
async fn test_scoped_search_filters_by_kb() {
    let Some(persistence) = setup_persistence().await else {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    };

    // Create two KBs
    let kb1 = create_test_kb("scope-1");
    let kb2 = create_test_kb("scope-2");
    persistence
        .save_knowledge_base(&kb1)
        .await
        .expect("Failed to save KB1");
    persistence
        .save_knowledge_base(&kb2)
        .await
        .expect("Failed to save KB2");

    // Create chunks with similar embeddings but in different KBs
    let embedding = vec![0.5f32; 384];

    let chunk1 = create_test_chunk(&kb1.id, None, "Content in KB1", embedding.clone());
    let chunk2 = create_test_chunk(&kb2.id, None, "Content in KB2", embedding.clone());

    persistence
        .save_chunk(&chunk1)
        .await
        .expect("Failed to save chunk1");
    persistence
        .save_chunk(&chunk2)
        .await
        .expect("Failed to save chunk2");

    // Search scoped to KB1 only
    let kb1_results = persistence
        .search_knowledge_scoped(TEST_OWNER, &[&kb1.id], &embedding, 10, 0.0)
        .await
        .expect("Failed to search KB1");

    // Should only find chunk from KB1
    assert!(
        kb1_results.iter().all(|m| m.chunk.kb_id == kb1.id),
        "Scoped search returned results from wrong KB"
    );

    // Search scoped to KB2 only
    let kb2_results = persistence
        .search_knowledge_scoped(TEST_OWNER, &[&kb2.id], &embedding, 10, 0.0)
        .await
        .expect("Failed to search KB2");

    assert!(
        kb2_results.iter().all(|m| m.chunk.kb_id == kb2.id),
        "Scoped search returned results from wrong KB"
    );

    // Search across both KBs
    let both_results = persistence
        .search_knowledge_scoped(TEST_OWNER, &[&kb1.id, &kb2.id], &embedding, 10, 0.0)
        .await
        .expect("Failed to search both KBs");

    assert!(
        both_results.len() >= 2,
        "Expected at least 2 results when searching both KBs"
    );

    // Cleanup
    persistence
        .delete_knowledge_base(TEST_OWNER, &kb1.id)
        .await
        .expect("Failed to delete KB1");
    persistence
        .delete_knowledge_base(TEST_OWNER, &kb2.id)
        .await
        .expect("Failed to delete KB2");
}

// =============================================================================
// Default KB Initialization Tests
// =============================================================================

#[tokio::test]
#[serial]
async fn test_default_kb_initialization() {
    let Some(persistence) = setup_persistence().await else {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    };

    // Ensure no default KB exists (may need to clean up from previous tests)
    if let Ok(Some(existing)) = persistence
        .get_knowledge_base_by_name("anonymous", "default")
        .await
    {
        persistence
            .delete_knowledge_base("anonymous", &existing.id)
            .await
            .ok();
    }

    // First call should create the default KB
    let kb1 = ensure_default_knowledge_base(persistence.as_ref(), None)
        .await
        .expect("Failed to create default KB");

    assert_eq!(kb1.name, "default");
    assert!(kb1.description.is_some());

    // Second call should return the same KB (idempotent)
    let kb2 = ensure_default_knowledge_base(persistence.as_ref(), None)
        .await
        .expect("Failed to get default KB");

    assert_eq!(kb1.id, kb2.id);

    // Cleanup
    persistence
        .delete_knowledge_base("anonymous", &kb1.id)
        .await
        .expect("Failed to delete default KB");
}

// =============================================================================
// Chunk Storage and Search Tests
// =============================================================================

#[tokio::test]
#[serial]
async fn test_chunk_storage_and_global_search() {
    let Some(persistence) = setup_persistence().await else {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    };

    // Create KB
    let kb = create_test_kb("chunks");
    persistence
        .save_knowledge_base(&kb)
        .await
        .expect("Failed to save KB");

    // Create chunks with different embeddings
    let chunks = vec![
        create_test_chunk(
            &kb.id,
            None,
            "The quick brown fox",
            make_embedding(&[0.9, 0.1, 0.0]),
        ),
        create_test_chunk(
            &kb.id,
            None,
            "jumps over the lazy dog",
            make_embedding(&[0.1, 0.9, 0.0]),
        ),
        create_test_chunk(
            &kb.id,
            None,
            "A completely different topic",
            make_embedding(&[0.0, 0.1, 0.9]),
        ),
    ];

    for chunk in &chunks {
        persistence
            .save_chunk(chunk)
            .await
            .expect("Failed to save chunk");
    }

    // Search with embedding similar to first chunk
    let query = make_embedding(&[0.85, 0.15, 0.0]);
    let results = persistence
        .search_knowledge(TEST_OWNER, &query, 10, 0.0) // Low threshold to get all results
        .await
        .expect("Failed to search");

    assert!(!results.is_empty(), "Expected search results");

    // The first result should be the most similar one
    // (Results are sorted by score descending)

    // Cleanup
    persistence
        .delete_knowledge_base(TEST_OWNER, &kb.id)
        .await
        .expect("Failed to delete KB");
}
