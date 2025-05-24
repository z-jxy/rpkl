use std::time::Duration;

// use crate::utils::macros::_trace;

// work around to avoid requiring unstable feature `duration_constructors` https://github.com/rust-lang/rust/issues/120301

const SECS_PER_MINUTE: u64 = 60;
const MINS_PER_HOUR: u64 = 60;
const HOURS_PER_DAY: u64 = 24;

#[inline]
pub const fn from_days(days: u64) -> Option<Duration> {
    if days > u64::MAX / (SECS_PER_MINUTE * MINS_PER_HOUR * HOURS_PER_DAY) {
        // panic!("overflow in Duration::from_days");
        return None;
    }

    Some(Duration::from_secs(
        days * MINS_PER_HOUR * SECS_PER_MINUTE * HOURS_PER_DAY,
    ))
}

#[inline]
pub const fn from_hours(hours: u64) -> Option<Duration> {
    if hours > u64::MAX / (SECS_PER_MINUTE * MINS_PER_HOUR) {
        // panic!("overflow in Duration::from_hours");
        return None;
    }

    Some(Duration::from_secs(hours * MINS_PER_HOUR * SECS_PER_MINUTE))
}

#[inline]
pub const fn from_mins(mins: u64) -> Option<Duration> {
    if mins > u64::MAX / SECS_PER_MINUTE {
        // panic!("overflow in Duration::from_mins");
        return None;
    }

    Some(Duration::from_secs(mins * SECS_PER_MINUTE))
}
