use chrono::{DateTime, Timelike, Utc};
use embassy_time::Instant;

pub struct GlobalTime {
    datetime: DateTime<Utc>,
    instant: Instant, // local chip time when datetime was received
}

impl GlobalTime {
    pub fn now(datetime: DateTime<Utc>) -> Self {
        Self {
            instant: Instant::now(),
            datetime,
        }
    }

    pub fn day_minute(&self) -> u64 {
        ((self.datetime.num_seconds_from_midnight() as u64 + self.instant.elapsed().as_secs()) / 60)
            % (24 * 60)
    }

    pub fn secs_till_minute(&self, minute: u64) -> u64 {
        let current_secs = (self.datetime.num_seconds_from_midnight() as u64
            + self.instant.elapsed().as_secs())
            % (24 * 60 * 60);
        let to_secs = minute * 60;

        if current_secs > to_secs {
            24 * 60 * 60 + to_secs - current_secs
        } else {
            to_secs - current_secs
        }
    }
}
