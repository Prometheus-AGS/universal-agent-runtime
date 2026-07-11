use serde_json::Value;

#[test]
fn shared_agui_golden_fixture_matches_the_profile_schema() {
    let schema: Value = serde_json::from_str(include_str!("../schemas/uar-agui-v1.schema.json"))
        .expect("AG-UI profile schema is JSON");
    let events: Vec<Value> = serde_json::from_str(include_str!("fixtures/agui/uar-agui-v1.json"))
        .expect("AG-UI golden fixture is JSON");
    let validator = jsonschema::validator_for(&schema).expect("AG-UI schema compiles");

    assert!(!events.is_empty());
    for event in &events {
        if let Err(error) = validator.validate(event) {
            panic!("invalid AG-UI golden event {event}: {error}");
        }
    }
    assert!(events.iter().all(|event| event["profile"] == "uar.agui/1"));
    assert!(!events.iter().any(|event| {
        event["type"]
            .as_str()
            .is_some_and(|kind| kind.starts_with("THINKING_"))
    }));
}
