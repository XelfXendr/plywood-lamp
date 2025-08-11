use chrono::{DateTime, TimeZone, Timelike};
use embassy_time::{Duration, Instant};

pub struct GlobalTime<Tz: TimeZone> {
    datetime: DateTime<Tz>,
    instant: Instant, // local chip time when datetime was received
}

pub struct GlobalInstant<Tz: TimeZone> {
    datetime: DateTime<Tz>,
    elapsed: Duration,
}

impl<Tz: TimeZone> GlobalTime<Tz> {
    pub fn at(datetime: DateTime<Tz>) -> Self {
        Self {
            instant: Instant::now(),
            datetime,
        }
    }

    pub fn now(&self) -> GlobalInstant<Tz> {
        GlobalInstant {
            datetime: self.datetime.clone(),
            elapsed: self.instant.elapsed(),
        }
    }
}

impl<Tz: TimeZone> GlobalInstant<Tz> {
    pub fn day_minute(&self) -> u64 {
        ((self.datetime.num_seconds_from_midnight() as u64 + self.elapsed.as_secs()) / 60)
            % (24 * 60)
    }

    pub fn secs_till_minute(&self, minute: u64) -> u64 {
        let current_secs = (self.datetime.num_seconds_from_midnight() as u64
            + self.elapsed.as_secs())
            % (24 * 60 * 60);
        let to_secs = minute * 60;

        if current_secs > to_secs {
            24 * 60 * 60 + to_secs - current_secs
        } else {
            to_secs - current_secs
        }
    }

    pub fn duration_till_minute(&self, minute: u64) -> Duration {
        Duration::from_secs(self.secs_till_minute(minute))
    }
}
