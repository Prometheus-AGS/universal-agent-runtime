//! Calendar conversion only. All durable state and admission remain in UAR.

use chrono::{DateTime, Datelike, NaiveDate, NaiveTime, Utc};
use jiff::{Timestamp, tz::{AmbiguousOffset, TimeZone}};
use crate::uar::domain::schedules::{DurableSchedule, ScheduleOccurrence, ScheduleRecurrence, ScheduleSkip};

use super::ScheduleError;

const DAYS_PER_SCAN: usize = 366;

fn wall_time(value: &str) -> Result<NaiveTime, ScheduleError> {
    let bytes = value.as_bytes();
    if bytes.len() != 5 || bytes[2] != b':'
        || ![bytes[0], bytes[1], bytes[3], bytes[4]].iter().all(u8::is_ascii_digit) {
        return Err(ScheduleError::Invalid("time must be HH:mm"));
    }
    NaiveTime::parse_from_str(value, "%H:%M").map_err(|_| ScheduleError::Invalid("time must be HH:mm"))
}

fn zone(name: &str) -> Result<TimeZone, ScheduleError> {
    let zone = TimeZone::get(name).map_err(|_| ScheduleError::Invalid("IANA timezone is invalid or unavailable"))?;
    if zone.iana_name().is_none() {
        return Err(ScheduleError::Invalid("a named IANA timezone is required"));
    }
    Ok(zone)
}

pub(crate) fn validate(recurrence: &ScheduleRecurrence, now: DateTime<Utc>) -> Result<(), ScheduleError> {
    match recurrence {
        ScheduleRecurrence::Once { at } if *at <= now => Err(ScheduleError::Invalid("one-shot time must be in the future")),
        ScheduleRecurrence::Once { .. } => Ok(()),
        ScheduleRecurrence::Daily { time, timezone } => { wall_time(time)?; zone(timezone)?; Ok(()) }
        ScheduleRecurrence::Weekly { weekdays, time, timezone } => {
            wall_time(time)?; zone(timezone)?;
            let unique: std::collections::BTreeSet<_> = weekdays.iter().collect();
            if weekdays.is_empty() || unique.len() != weekdays.len() || weekdays.iter().any(|v| !(1..=7).contains(v)) {
                return Err(ScheduleError::Invalid("weekdays must be unique ISO weekdays 1 through 7"));
            }
            Ok(())
        }
    }
}

fn local_date(zone: &TimeZone, at: DateTime<Utc>) -> Result<NaiveDate, ScheduleError> {
    let stamp = Timestamp::from_second(at.timestamp()).map_err(|_| ScheduleError::Invalid("instant is out of calendar range"))?;
    let local = zone.to_datetime(stamp);
    NaiveDate::from_ymd_opt(i32::from(local.year()), local.month() as u32, local.day() as u32)
        .ok_or(ScheduleError::Invalid("local date is out of calendar range"))
}

pub(crate) fn initial_date(recurrence: &ScheduleRecurrence, now: DateTime<Utc>) -> Result<Option<NaiveDate>, ScheduleError> {
    match recurrence {
        ScheduleRecurrence::Once { .. } => Ok(None),
        ScheduleRecurrence::Daily { timezone, .. } | ScheduleRecurrence::Weekly { timezone, .. } =>
            local_date(&zone(timezone)?, now).map(Some),
    }
}

/// Return the next future business-time boundary that can require a schedule
/// scan. This is advisory wakeup planning only; [`scan`] remains the durable
/// admission authority after the wakeup fires.
pub(crate) fn next_due_after(
    record: &DurableSchedule,
    now: DateTime<Utc>,
) -> Result<Option<DateTime<Utc>>, ScheduleError> {
    if record.cancelled || record.paused || record.guard.is_some() {
        return Ok(None);
    }
    if let Some(pending) = &record.pending {
        return Ok((pending.due_at > now).then_some(pending.due_at));
    }
    match &record.recurrence {
        ScheduleRecurrence::Once { at } => {
            return Ok((!record.once_scanned && *at > now).then_some(*at));
        }
        ScheduleRecurrence::Daily { time, timezone } => {
            let zone = zone(timezone)?;
            let time = wall_time(time)?;
            let mut day = record
                .next_local_date
                .ok_or(ScheduleError::Conflict("calendar cursor is missing"))?;
            for _ in 0..DAYS_PER_SCAN {
                if let Some(at) = resolve(&zone, day, time)? {
                    if at > now && at > record.effective_at {
                        return Ok(Some(at));
                    }
                }
                day = day
                    .succ_opt()
                    .ok_or(ScheduleError::Invalid("calendar cursor exhausted"))?;
            }
            Ok(None)
        }
        ScheduleRecurrence::Weekly {
            weekdays,
            time,
            timezone,
        } => {
            let zone = zone(timezone)?;
            let time = wall_time(time)?;
            let mut day = record
                .next_local_date
                .ok_or(ScheduleError::Conflict("calendar cursor is missing"))?;
            for _ in 0..DAYS_PER_SCAN {
                if weekdays.contains(&(day.weekday().number_from_monday() as u8)) {
                    if let Some(at) = resolve(&zone, day, time)? {
                        if at > now && at > record.effective_at {
                            return Ok(Some(at));
                        }
                    }
                }
                day = day
                    .succ_opt()
                    .ok_or(ScheduleError::Invalid("calendar cursor exhausted"))?;
            }
            Ok(None)
        }
    }
}

/// Resolve one local calendar candidate under the schedule skip/earlier policy.
/// The returned instant is not admission authority.
pub fn resolve_local_time(day: NaiveDate, time: &str, timezone: &str) -> Result<Option<DateTime<Utc>>, ScheduleError> {
    resolve(&zone(timezone)?, day, wall_time(time)?)
}

fn resolve(zone: &TimeZone, day: NaiveDate, time: NaiveTime) -> Result<Option<DateTime<Utc>>, ScheduleError> {
    let civil: jiff::civil::DateTime = day.and_time(time).format("%Y-%m-%dT%H:%M:%S").to_string().parse()
        .map_err(|_| ScheduleError::Invalid("local datetime is out of calendar range"))?;
    let ambiguous = zone.to_ambiguous_timestamp(civil);
    if matches!(ambiguous.offset(), AmbiguousOffset::Gap { .. }) { return Ok(None); }
    let at = ambiguous.earlier().map_err(|_| ScheduleError::Invalid("local datetime cannot be resolved"))?;
    DateTime::from_timestamp(at.as_second(), 0).map(Some)
        .ok_or(ScheduleError::Invalid("instant is out of UTC range"))
}

pub(crate) fn skipped(record: &mut DurableSchedule, reason: &str, value: String) {
    if let Some(last) = record.skipped.last_mut().filter(|s| s.reason == reason && s.revision == record.revision) {
        last.last = value;
        last.count = last.count.saturating_add(1);
    } else {
        record.skipped.push(ScheduleSkip { revision: record.revision, reason: reason.into(),
            first: value.clone(), last: value, count: 1 });
    }
}

fn pending(record: &mut DurableSchedule, at: DateTime<Utc>) {
    if record.pending.as_ref().is_some_and(|p| p.due_at >= at) { return; }
    if let Some(old) = record.pending.take() {
        skipped(record, "coalesced", old.due_at.to_rfc3339());
    }
    let id = format!("{}-{}-{}", record.id, record.revision, at.timestamp());
    record.pending = Some(ScheduleOccurrence { command_id: format!("schedule-{id}"),
        id, revision: record.revision, due_at: at });
}

/// Advance at most one calendar year per pass. A partial catch-up may update
/// pending intent but cannot admit it until the complete due range is scanned.
pub(crate) fn scan(record: &mut DurableSchedule, now: DateTime<Utc>) -> Result<bool, ScheduleError> {
    if record.cancelled { return Ok(true); }
    let recurrence = record.recurrence.clone();
    if let ScheduleRecurrence::Once { at } = recurrence {
        if !record.once_scanned && at <= now {
            pending(record, at); record.once_scanned = true;
        }
        return Ok(true);
    }
    let (time, timezone, weekdays) = match &recurrence {
        ScheduleRecurrence::Daily { time, timezone } => (time, timezone, None),
        ScheduleRecurrence::Weekly { time, timezone, weekdays } => (time, timezone, Some(weekdays)),
        ScheduleRecurrence::Once { .. } => return Ok(true),
    };
    let time = wall_time(time)?;
    let zone = zone(timezone)?;
    let today = local_date(&zone, now)?;
    let effective_local = zone.to_datetime(Timestamp::from_second(record.effective_at.timestamp())
        .map_err(|_| ScheduleError::Invalid("effective instant is out of calendar range"))?);
    let mut day = record.next_local_date.ok_or(ScheduleError::Conflict("calendar cursor is missing"))?;
    for _ in 0..DAYS_PER_SCAN {
        if day > today { return Ok(true); }
        if weekdays.is_none_or(|days| days.contains(&(day.weekday().number_from_monday() as u8))) {
            match resolve(&zone, day, time)? {
                Some(at) if at > now => return Ok(true),
                Some(at) if at > record.effective_at => pending(record, at),
                Some(_) => {},
                None => {
                    let effective_day = local_date(&zone, record.effective_at)?;
                    let effective_time = NaiveTime::from_hms_opt(effective_local.hour() as u32,
                        effective_local.minute() as u32, effective_local.second() as u32)
                        .ok_or(ScheduleError::Invalid("effective wall time is invalid"))?;
                    if day > effective_day || (day == effective_day && time > effective_time) {
                        skipped(record, "dst_gap", day.and_time(time).to_string());
                    }
                },
            }
        }
        day = day.succ_opt().ok_or(ScheduleError::Invalid("calendar cursor exhausted"))?;
        record.next_local_date = Some(day);
    }
    Ok(day > today)
}
