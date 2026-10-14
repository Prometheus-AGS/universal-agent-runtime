//! Calendar integration fixture only; it does not certify durable admission.
//! Deferred until the expanded Dot production implementation boundary.
use chrono::{DateTime, NaiveDate, Utc};
use universal_agent_runtime::uar::runtime::schedules::calendar::resolve_local_time;

#[test]
fn named_zone_calendar_policy_matches_dated_iana_fixtures() {
    let fixtures: serde_json::Value = serde_json::from_str(include_str!("fixtures/dot14_calendar_cases.json")).unwrap();
    for case in fixtures["cases"].as_array().unwrap() {
        let day: NaiveDate = case["localDate"].as_str().unwrap().parse().unwrap();
        let instant = resolve_local_time(day, case["recurrence"]["time"].as_str().unwrap(),
            case["recurrence"]["timezone"].as_str().unwrap()).unwrap();
        let expected: Option<DateTime<Utc>> = case["expectedUtc"].as_str().map(|s| s.parse().unwrap());
        assert_eq!(instant, expected, "{}", case["name"]);
        if let Some(forbidden) = case["forbiddenUtc"].as_str() {
            assert_ne!(instant, Some(forbidden.parse().unwrap()));
        }
    }
    let day = NaiveDate::from_ymd_opt(2027, 1, 1).unwrap();
    for time in ["24:00", "9:00", "09:00:00", "00:60"] {
        assert!(resolve_local_time(day, time, "America/Chicago").is_err());
    }
    assert!(resolve_local_time(day, "09:00", "Not/AZone").is_err());
}
