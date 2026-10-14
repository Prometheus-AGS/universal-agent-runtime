//! Quiet membership is evaluated at a real instant, including repeated hours.
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use jiff::{Timestamp, tz::{AmbiguousOffset, TimeZone}};
use crate::uar::domain::followups::QuietHours;
use super::FollowupError;

fn time(value: &str) -> Result<NaiveTime, FollowupError> {
    if value.len() != 5 || value.as_bytes()[2] != b':'
        || !value.bytes().enumerate().all(|(i, b)| i == 2 || b.is_ascii_digit()) {
        return Err(FollowupError::Invalid("quiet time must be HH:mm"));
    }
    NaiveTime::parse_from_str(value, "%H:%M").map_err(|_| FollowupError::Invalid("quiet time must be HH:mm"))
}
fn zone(value: &str) -> Result<TimeZone, FollowupError> {
    let zone = TimeZone::get(value).map_err(|_| FollowupError::Invalid("quiet timezone is invalid or unavailable"))?;
    if zone.iana_name().is_none() { return Err(FollowupError::Invalid("quiet timezone must be an IANA name")); }
    Ok(zone)
}
pub(crate) fn validate(hours: &QuietHours) -> Result<(), FollowupError> {
    if time(&hours.start)? == time(&hours.end)? { return Err(FollowupError::Invalid("quiet start and end must differ")); }
    zone(&hours.timezone)?;
    Ok(())
}
fn inside(zone: &TimeZone, start: NaiveTime, end: NaiveTime, instant: Timestamp) -> Result<bool, FollowupError> {
    let local = zone.to_datetime(instant);
    let time = NaiveTime::from_hms_opt(local.hour() as u32, local.minute() as u32, local.second() as u32)
        .ok_or(FollowupError::Invalid("quiet local time is invalid"))?;
    Ok(if start < end { time >= start && time < end } else { time >= start || time < end })
}

/// Earliest future real instant outside quiet hours, or None when permitted now.
/// Candidate boundaries include both fold ends and timezone transitions, so a
/// nonexistent end releases at the first real instant outside the interval.
pub fn release_at(hours: &QuietHours, now: DateTime<Utc>) -> Result<Option<DateTime<Utc>>, FollowupError> {
    validate(hours)?;
    let start = time(&hours.start)?;
    let end = time(&hours.end)?;
    let zone = zone(&hours.timezone)?;
    let stamp = Timestamp::from_second(now.timestamp()).map_err(|_| FollowupError::Invalid("quiet instant out of range"))?;
    if !inside(&zone, start, end, stamp)? { return Ok(None); }
    let local = zone.to_datetime(stamp);
    let mut day = NaiveDate::from_ymd_opt(local.year() as i32, local.month() as u32, local.day() as u32)
        .ok_or(FollowupError::Invalid("quiet date out of range"))?;
    let mut candidates = Vec::new();
    // Named IANA rules can skip a complete local day. Retain multiple real
    // end-boundaries rather than guessing a 24-hour UTC duration.
    for _ in 0..4 {
        let civil: jiff::civil::DateTime = day.and_time(end).format("%Y-%m-%dT%H:%M:%S").to_string().parse()
            .map_err(|_| FollowupError::Invalid("quiet boundary out of range"))?;
        let ambiguous = zone.to_ambiguous_timestamp(civil);
        if !matches!(ambiguous.offset(), AmbiguousOffset::Gap { .. }) {
            candidates.push(ambiguous.earlier().map_err(|_| FollowupError::Invalid("quiet boundary is invalid"))?);
            candidates.push(ambiguous.later().map_err(|_| FollowupError::Invalid("quiet boundary is invalid"))?);
        }
        day = day.succ_opt().ok_or(FollowupError::Invalid("quiet date exhausted"))?;
    }
    let horizon = candidates.iter().copied().max().ok_or(FollowupError::Invalid("quiet boundary unavailable"))?;
    candidates.extend(zone.following(stamp).take_while(|t| t.timestamp() <= horizon).map(|t| t.timestamp()));
    candidates.sort();
    for candidate in candidates {
        if candidate > stamp && !inside(&zone, start, end, candidate)? {
            return DateTime::from_timestamp(candidate.as_second(), 0).map(Some)
                .ok_or(FollowupError::Invalid("quiet release out of range"));
        }
    }
    Err(FollowupError::Invalid("quiet release unavailable"))
}
