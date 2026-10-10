//! A knowledge base can set how many chunks an agent retrieves per turn and
//! how similar they must be. The defaults, 3 chunks at 0.7, were hard-coded in
//! chat retrieval, but real answers score 0.58 to 0.67 for short questions
//! against text-embedding-v4, so a question about a fact in the KB retrieved
//! nothing.

use universal_agent_runtime::uar::domain::knowledge::{
    DEFAULT_RETRIEVAL_MIN_SCORE, DEFAULT_RETRIEVAL_TOP_K, KbConfig, retrieval_params,
};

fn config(min_score: Option<f32>, top_k: Option<usize>) -> KbConfig {
    KbConfig {
        retrieval_min_score: min_score,
        retrieval_top_k: top_k,
        ..KbConfig::default()
    }
}

#[test]
fn defaults_are_unchanged_when_nothing_is_set() {
    let kb = KbConfig::default();

    assert_eq!(
        retrieval_params(&[&kb]),
        (DEFAULT_RETRIEVAL_TOP_K, DEFAULT_RETRIEVAL_MIN_SCORE)
    );
    assert_eq!(
        (DEFAULT_RETRIEVAL_TOP_K, DEFAULT_RETRIEVAL_MIN_SCORE),
        (3, 0.7)
    );
}

#[test]
fn no_knowledge_bases_use_the_defaults() {
    assert_eq!(
        retrieval_params(&[]),
        (DEFAULT_RETRIEVAL_TOP_K, DEFAULT_RETRIEVAL_MIN_SCORE)
    );
}

#[test]
fn a_kb_setting_replaces_the_default() {
    let kb = config(Some(0.5), Some(5));

    assert_eq!(retrieval_params(&[&kb]), (5, 0.5));
}

#[test]
fn each_setting_falls_back_on_its_own() {
    assert_eq!(
        retrieval_params(&[&config(Some(0.5), None)]),
        (DEFAULT_RETRIEVAL_TOP_K, 0.5)
    );
    assert_eq!(
        retrieval_params(&[&config(None, Some(6))]),
        (6, DEFAULT_RETRIEVAL_MIN_SCORE)
    );
}

#[test]
fn across_kbs_the_most_permissive_setting_wins() {
    let strict = config(Some(0.8), Some(2));
    let tuned = config(Some(0.5), Some(5));
    let unset = KbConfig::default();

    assert_eq!(retrieval_params(&[&strict, &tuned, &unset]), (5, 0.5));
}

#[test]
fn a_kb_stored_before_these_fields_existed_still_loads() {
    let stored = r#"{
        "embedding_provider": "openai",
        "embedding_model": "text-embedding-v4",
        "chunk_strategy": {"Recursive": {"size": 1000}}
    }"#;

    let kb: KbConfig = serde_json::from_str(stored).unwrap();

    assert_eq!(kb.retrieval_min_score, None);
    assert_eq!(kb.retrieval_top_k, None);
}

#[test]
fn settings_survive_a_round_trip_and_unset_ones_are_omitted() {
    let json = serde_json::to_value(config(Some(0.5), None)).unwrap();

    assert_eq!(json["retrieval_min_score"], 0.5);
    assert!(json.get("retrieval_top_k").is_none());
    let back: KbConfig = serde_json::from_value(json).unwrap();
    assert_eq!(back.retrieval_min_score, Some(0.5));
}
