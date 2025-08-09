use chrono::{DateTime, Timelike, Utc};
use embassy_time::{Duration, Instant};

pub struct GlobalTime {
    datetime: DateTime<Utc>,
    instant: Instant, // local chip time when datetime was received
}

pub struct GlobalInstant {
    datetime: DateTime<Utc>,
    elapsed: Duration,
}

impl GlobalTime {
    pub fn at(datetime: DateTime<Utc>) -> Self {
        Self {
            instant: Instant::now(),
            datetime,
        }
    }

    pub fn now(&self) -> GlobalInstant {
        GlobalInstant {
            datetime: self.datetime,
            elapsed: self.instant.elapsed(),
        }
    }
}

impl GlobalInstant {
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
