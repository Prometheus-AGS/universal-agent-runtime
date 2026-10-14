//! Real adapter upgrade collision fixture. Authored in Dot Execute; not run.
//! Every backend stage ends in a joined process before the next connects.
#![cfg(feature = "surreal-backend")]

use std::{process::Command, sync::Arc};
use universal_agent_runtime::uar::{
    domain::knowledge::{DocumentStatus, EmbeddingFingerprint, KbConfig, KnowledgeBase, KnowledgeChunk, KnowledgeDocument},
    persistence::{PersistenceLayer, ingestion_records::{IngestionJob, IngestionPublication, IngestionState},
        knowledge_namespace::KnowledgeNamespace, providers::surreal::SurrealDbProvider},
};

const OWNER: &str = "v1:k:issuer:i:7:fixture:t:1:a:s:4:same";
const OTHER: &str = "v1:k:issuer:i:7:fixture:t:1:b:s:4:same";
const OTHER_ISSUER: &str = "v1:k:issuer:i:8:fixture2:t:1:a:s:4:same";
const CHUNK: uuid::Uuid = uuid::Uuid::from_u128(0x301);

fn kb(owner: &str, id: &str) -> KnowledgeBase {
    let now = chrono::Utc::now().to_rfc3339();
    KnowledgeBase { id: id.into(), owner_id: owner.into(), name: id.into(), description: None,
        config: KbConfig::default(), created_at: now.clone(), updated_at: now }
}
fn document(owner: &str, id: &str) -> KnowledgeDocument {
    let now = chrono::Utc::now().to_rfc3339();
    KnowledgeDocument { id: id.into(), owner_id: owner.into(), kb_id: id.into(), filename: "namespace.txt".into(),
        file_path: None, mime_type: Some("text/plain".into()), chunk_count: 0,
        status: DocumentStatus::Pending, created_at: now.clone(), updated_at: now }
}
fn chunk(owner: &str, id: &str, content: &str) -> KnowledgeChunk {
    KnowledgeChunk { id: CHUNK, owner_id: owner.into(), kb_id: id.into(), document_id: Some(id.into()),
        content: content.into(), metadata: None, embedding: vec![1.0;384], created_at: chrono::Utc::now().to_rfc3339() }
}
fn legacy_job(id: &str) -> IngestionJob {
    // A genuine old serialized record has no provenance marker, even when its
    // subject is byte-for-byte equal to a new verified principal storage key.
    serde_json::from_value(serde_json::json!({"owner_id":OWNER,"document_id":id,"kb_id":id,
        "generation":"old-generation","state":"pending","attempts":0,"next_attempt_at":0,"last_error":null})).unwrap()
}

async fn seed_surreal(store: &SurrealDbProvider, id: &str) {
    let key = format!("{}:{OWNER}:{id}", OWNER.len());
    let mut base = serde_json::to_value(kb(OWNER, id)).unwrap();
    base.as_object_mut().unwrap().remove("id"); base["logical_id"] = id.into();
    let mut doc = serde_json::to_value(document(OWNER, id)).unwrap();
    doc.as_object_mut().unwrap().remove("id"); doc["logical_id"] = id.into();
    let mut part = serde_json::to_value(chunk(OWNER, id, "legacy-private")).unwrap();
    part.as_object_mut().unwrap().remove("id"); part["logical_id"] = CHUNK.to_string().into();
    store.client().query("BEGIN TRANSACTION; CREATE type::record('knowledge_bases',$key) CONTENT $base; CREATE type::record('knowledge_documents',$key) CONTENT $doc; CREATE type::record('knowledge_chunks',$key) CONTENT $part; CREATE type::record('knowledge_ingestion',$key) CONTENT $job; COMMIT TRANSACTION;")
        .bind(("key",key)).bind(("base",base)).bind(("doc",doc)).bind(("part",part))
        .bind(("job",serde_json::json!({"owner_id":OWNER,"document_id":id,"kb_id":id,"state":"pending","next_attempt_at":0,
            "data":serde_json::to_string(&legacy_job(id)).unwrap(),"source":b"legacy-source".to_vec()})))
        .await.unwrap().check().unwrap();
}
#[cfg(feature = "postgres-backend")]
async fn seed_postgres(store: &universal_agent_runtime::uar::persistence::providers::postgres::PostgresProvider, id: &str) {
    let mut tx = store.get_pool().begin().await.unwrap();
    sqlx::query("INSERT INTO knowledge_bases(owner_id,id,name,config) VALUES($1,$2,$2,$3)")
        .bind(OWNER).bind(id).bind(serde_json::to_value(KbConfig::default()).unwrap()).execute(&mut *tx).await.unwrap();
    sqlx::query("INSERT INTO knowledge_documents(owner_id,id,kb_id,filename,mime_type) VALUES($1,$2,$2,'legacy.txt','text/plain')")
        .bind(OWNER).bind(id).execute(&mut *tx).await.unwrap();
    sqlx::query("INSERT INTO knowledge_chunks(owner_id,id,kb_id,document_id,content,embedding) VALUES($1,$2,$3,$3,'legacy-private',$4)")
        .bind(OWNER).bind(CHUNK).bind(id).bind(pgvector::Vector::from(vec![1.0;384])).execute(&mut *tx).await.unwrap();
    sqlx::query("INSERT INTO knowledge_ingestion(owner_id,document_id,kb_id,state,next_attempt_at,data,source) VALUES($1,$2,$2,'pending',0,$3,$4)")
        .bind(OWNER).bind(id).bind(serde_json::to_value(legacy_job(id)).unwrap()).bind(b"legacy-source".as_slice()).execute(&mut *tx).await.unwrap();
    tx.commit().await.unwrap();
}

async fn exercise(store: Arc<dyn PersistenceLayer>, stage: &str, id: &str) {
    if stage == "seed" {
        assert!(store.get_knowledge_base(OWNER,id).await.unwrap().is_none());
        assert!(store.get_knowledge_base_by_name(OWNER,id).await.unwrap().is_none());
        assert!(store.list_knowledge_bases(OWNER).await.unwrap().is_empty());
        assert!(store.get_document(OWNER,id).await.unwrap().is_none());
        assert!(store.list_documents(OWNER,id).await.unwrap().is_empty());
        assert_eq!(store.count_documents(OWNER,id).await.unwrap(),0);
        assert!(store.search_knowledge(OWNER,&vec![1.0;384],10,0.0).await.unwrap().is_empty());
        assert!(store.get_ingestion_job(OWNER,id).await.unwrap().is_none());
        assert!(store.discover_ingestion(100).await.unwrap().is_empty());
        let legacy = legacy_job(id);
        assert_eq!(legacy.namespace,KnowledgeNamespace::Legacy);
        assert!(store.get_ingestion_source(&legacy).await.is_err());
        assert!(store.transition_ingestion(&legacy,&legacy.processing()).await.is_err());
        store.delete_document(OWNER,id).await.unwrap();
        store.delete_knowledge_base(OWNER,id).await.unwrap();
        // The same logical identifiers can now exist independently.
        store.save_knowledge_base(&kb(OWNER,id)).await.unwrap();
        let job = store.accept_ingestion(&document(OWNER,id),b"principal-source").await.unwrap();
        assert_eq!(job.namespace,KnowledgeNamespace::PrincipalV1);
        let anonymous = format!("{id}-anonymous");
        store.save_knowledge_base(&kb("anonymous",&anonymous)).await.unwrap();
        let anon = store.accept_ingestion(&document("anonymous",&anonymous),b"anonymous-source").await.unwrap();
        assert_eq!(anon.namespace,KnowledgeNamespace::Legacy);
        return;
    }
    let job = store.get_ingestion_job(OWNER,id).await.unwrap().unwrap();
    assert_eq!(job.namespace,KnowledgeNamespace::PrincipalV1);
    assert_eq!(store.get_ingestion_source(&job).await.unwrap().unwrap(),b"principal-source");
    for foreign in [OTHER,OTHER_ISSUER] {
        assert!(store.get_knowledge_base(foreign,id).await.unwrap().is_none());
        assert!(store.get_document(foreign,id).await.unwrap().is_none());
        assert!(store.get_ingestion_job(foreign,id).await.unwrap().is_none());
        assert!(store.search_knowledge_scoped(foreign,&[id],&vec![1.0;384],10,0.0).await.unwrap().is_empty());
        store.delete_knowledge_base(foreign,id).await.unwrap();
    }
    if stage == "publish" {
        let processing = job.processing();
        assert!(store.transition_ingestion(&job,&processing).await.unwrap());
        let publication = IngestionPublication { chunks: vec![chunk(OWNER,id,"principal-visible")],
            embedding: EmbeddingFingerprint { backend:"fixture".into(),model:"namespace".into(),dimension:384 } };
        store.publish_ingestion(&processing,&publication).await.unwrap();
        assert!(store.publish_ingestion(&processing,&publication).await.is_err());
        let anonymous = format!("{id}-anonymous");
        let anon = store.get_ingestion_job("anonymous",&anonymous).await.unwrap().unwrap();
        assert_eq!(anon.namespace,KnowledgeNamespace::Legacy);
        assert_eq!(store.get_ingestion_source(&anon).await.unwrap().unwrap(),b"anonymous-source");
        store.delete_knowledge_base("anonymous",&anonymous).await.unwrap();
        return;
    }
    assert_eq!(job.state,IngestionState::Indexed);
    let matches = store.search_knowledge_scoped(OWNER,&[id],&vec![1.0;384],10,0.0).await.unwrap();
    assert_eq!(matches.len(),1);
    assert_eq!(matches[0].chunk.content,"principal-visible");
    store.delete_knowledge_base(OWNER,id).await.unwrap();
    assert!(store.get_document(OWNER,id).await.unwrap().is_none());
    assert!(store.get_ingestion_job(OWNER,id).await.unwrap().is_none());
    assert!(store.search_knowledge(OWNER,&vec![1.0;384],10,0.0).await.unwrap().is_empty());
}

#[tokio::test]
async fn principal_knowledge_child() {
    let Ok(stage) = std::env::var("UAR_DOT12_KNOWLEDGE_STAGE") else { return; };
    let id = std::env::var("UAR_DOT12_KNOWLEDGE_ID").unwrap();
    let endpoint = std::env::var("UAR_DOT12_KNOWLEDGE_URL").unwrap();
    #[cfg(feature = "postgres-backend")]
    if std::env::var("UAR_DOT12_KNOWLEDGE_BACKEND").as_deref() == Ok("postgres") {
        use sqlx::Row;
        let store = Arc::new(universal_agent_runtime::uar::persistence::providers::postgres::PostgresProvider::new(&endpoint).await.unwrap());
        if stage == "seed" { seed_postgres(&store,&id).await; }
        exercise(store.clone(),&stage,&id).await;
        let row = sqlx::query("SELECT data,source FROM knowledge_ingestion WHERE owner_id=$1 AND document_id=$2")
            .bind(OWNER).bind(&id).fetch_one(store.get_pool()).await.unwrap();
        assert_eq!(row.get::<Vec<u8>,_>("source"),b"legacy-source");
        assert_eq!(row.get::<serde_json::Value,_>("data"),serde_json::to_value(legacy_job(&id)).unwrap());
        let legacy = sqlx::query("SELECT c.content,d.status,b.name FROM knowledge_chunks c JOIN knowledge_documents d ON d.owner_id=c.owner_id AND d.id=c.document_id JOIN knowledge_bases b ON b.owner_id=c.owner_id AND b.id=c.kb_id WHERE c.owner_id=$1 AND c.kb_id=$2")
            .bind(OWNER).bind(&id).fetch_one(store.get_pool()).await.unwrap();
        assert_eq!(legacy.get::<String,_>("content"),"legacy-private");
        assert_eq!(legacy.get::<String,_>("status"),"pending");
        assert_eq!(legacy.get::<String,_>("name"),id);
        if stage == "cold" {
            sqlx::query("DELETE FROM knowledge_bases WHERE owner_id=$1 AND id=$2").bind(OWNER).bind(&id).execute(store.get_pool()).await.unwrap();
        }
        return;
    }
    let namespace = std::env::var("UAR_DOT12_KNOWLEDGE_NAMESPACE").unwrap();
    let remote = std::env::var("UAR_DOT12_KNOWLEDGE_BACKEND").as_deref() == Ok("surreal-network");
    let username = remote.then(|| std::env::var("UAR_DOT12_SURREAL_USER").ok()).flatten();
    let password = remote.then(|| std::env::var("UAR_DOT12_SURREAL_PASS").ok()).flatten();
    assert_eq!(username.is_some(), password.is_some(), "supply both dedicated SurrealDB root credentials or neither");
    let store = Arc::new(SurrealDbProvider::new(&endpoint,username.as_deref(),password.as_deref(),Some(&namespace),Some("knowledge")).await.unwrap());
    if stage == "seed" { seed_surreal(&store,&id).await; }
    exercise(store.clone(),&stage,&id).await;
    let mut response = store.client().query("SELECT VALUE data FROM knowledge_ingestion WHERE owner_id=$owner AND document_id=$id")
        .bind(("owner",OWNER.to_owned())).bind(("id",id.clone())).await.unwrap().check().unwrap();
    let rows: Vec<String> = response.take(0).unwrap();
    assert_eq!(rows,vec![serde_json::to_string(&legacy_job(&id)).unwrap()]);
    let mut response = store.client().query("SELECT VALUE content FROM knowledge_chunks WHERE owner_id=$owner AND kb_id=$id; SELECT VALUE filename FROM knowledge_documents WHERE owner_id=$owner AND logical_id=$id; SELECT VALUE name FROM knowledge_bases WHERE owner_id=$owner AND logical_id=$id;")
        .bind(("owner",OWNER.to_owned())).bind(("id",id.clone())).await.unwrap().check().unwrap();
    let content: Vec<String> = response.take(0).unwrap();
    let filenames: Vec<String> = response.take(1).unwrap();
    let names: Vec<String> = response.take(2).unwrap();
    assert_eq!(content,vec!["legacy-private"]);
    assert_eq!(filenames,vec!["namespace.txt"]);
    assert_eq!(names,vec![id]);
}

fn stages(url: &str, backend: &str) {
    let id = uuid::Uuid::new_v4().to_string();
    // The same isolated namespace is reopened by three distinct processes.
    // Never seed canonical-looking legacy rows into an existing application's namespace.
    let namespace = format!("dot12_knowledge_{}", uuid::Uuid::new_v4().simple());
    for stage in ["seed","publish","cold"] {
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact","principal_knowledge_child","--nocapture"])
            .env("UAR_DOT12_KNOWLEDGE_STAGE",stage).env("UAR_DOT12_KNOWLEDGE_ID",&id)
            .env("UAR_DOT12_KNOWLEDGE_URL",url).env("UAR_DOT12_KNOWLEDGE_BACKEND",backend)
            .env("UAR_DOT12_KNOWLEDGE_NAMESPACE",&namespace)
            .output().unwrap();
        assert!(output.status.success(),"{stage}: {}\n{}",String::from_utf8_lossy(&output.stdout),String::from_utf8_lossy(&output.stderr));
    }
}
#[test]
fn principal_knowledge_surreal_kv_cold_namespace_isolation() {
    let directory = tempfile::tempdir().unwrap();
    stages(&format!("surrealkv://{}",directory.path().join("knowledge.db").display()),"surreal");
}
#[test]
fn principal_knowledge_surreal_network_cold_namespace_isolation() {
    let endpoint = std::env::var("UAR_DOT12_SURREAL_URL").expect("dedicated network SurrealDB endpoint required; this scenario cannot be skipped");
    assert!(["ws://", "wss://", "http://", "https://"].iter().any(|prefix| endpoint.starts_with(prefix)),
        "network namespace fixture requires a real server endpoint, never embedded or memory storage");
    stages(&endpoint,"surreal-network");
}
#[test]
#[cfg(feature = "postgres-backend")]
#[ignore = "requires a dedicated empty PostgreSQL database with pgvector"]
fn principal_knowledge_postgres_cold_namespace_isolation() {
    stages(&std::env::var("UAR_DOT12_POSTGRES_URL").expect("dedicated database required"),"postgres");
}
