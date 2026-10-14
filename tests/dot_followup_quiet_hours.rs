//! Deferred calendar fixtures; real durable delivery is exercised separately.
use chrono::{DateTime, Utc};
use universal_agent_runtime::uar::{domain::followups::QuietHours, runtime::followups::quiet_hours::release_at};

fn instant(value: &str) -> DateTime<Utc> { value.parse().unwrap() }
fn hours(start: &str, end: &str) -> QuietHours {
    QuietHours { start: start.into(), end: end.into(), timezone: "America/Chicago".into() }
}

#[test]
fn quiet_hours_include_start_exclude_end_and_cross_midnight() {
    let quiet = hours("22:00", "07:00");
    assert_eq!(release_at(&quiet, instant("2027-01-15T03:59:59Z")).unwrap(), None);
    assert_eq!(release_at(&quiet, instant("2027-01-15T04:00:00Z")).unwrap(), Some(instant("2027-01-15T13:00:00Z")));
    assert_eq!(release_at(&quiet, instant("2027-01-15T12:59:59Z")).unwrap(), Some(instant("2027-01-15T13:00:00Z")));
    assert_eq!(release_at(&quiet, instant("2027-01-15T13:00:00Z")).unwrap(), None);
}

#[test]
fn missing_end_releases_at_first_real_instant_outside_window() {
    assert_eq!(release_at(&hours("01:00", "02:30"), instant("2027-03-14T07:30:00Z")).unwrap(),
        Some(instant("2027-03-14T08:00:00Z")));
}

#[test]
fn both_repeated_wall_times_are_quiet_and_choose_their_next_real_end() {
    let quiet = hours("00:30", "01:45");
    assert_eq!(release_at(&quiet, instant("2027-11-07T06:30:00Z")).unwrap(), Some(instant("2027-11-07T06:45:00Z")));
    assert_eq!(release_at(&quiet, instant("2027-11-07T07:30:00Z")).unwrap(), Some(instant("2027-11-07T07:45:00Z")));
    assert_eq!(release_at(&quiet, instant("2027-11-07T06:45:00Z")).unwrap(), None);
    assert_eq!(release_at(&quiet, instant("2027-11-07T07:45:00Z")).unwrap(), None);
}

#[test]
fn invalid_quiet_policy_is_not_silently_disabled() {
    for (start, end) in [("07:00", "07:00"), ("7:00", "08:00"), ("24:00", "08:00"), ("07:00:00", "08:00")] {
        assert!(release_at(&hours(start, end), instant("2027-01-15T12:00:00Z")).is_err());
    }
    let mut quiet = hours("22:00", "07:00"); quiet.timezone = "Not/AZone".into();
    assert!(release_at(&quiet, instant("2027-01-15T12:00:00Z")).is_err());
}
