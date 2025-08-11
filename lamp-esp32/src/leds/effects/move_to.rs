use core::iter::zip;

use embassy_time::{Duration, Instant};

use super::{Effect, EffectEnum, EffectStatus};
use crate::types::Color;

#[derive(Debug)]
pub struct MoveTo {
    from: Color,
    to: Color,
    t0: Instant,
    duration: u64,
}

impl Into<EffectEnum> for MoveTo {
    fn into(self) -> EffectEnum {
        EffectEnum::MoveTo(self)
    }
}

impl MoveTo {
    pub fn new(from: Color, to: Color, duration: Duration) -> Self {
        Self {
            from,
            to,
            t0: Instant::now(),
            duration: duration.as_millis(),
        }
    }

    pub fn reset_time(&mut self) {
        self.t0 = Instant::now();
    }

    pub fn millis_till_update(&self, current_millis: u64) -> Option<u64> {
        let current_color = self
            .from
            .interpolate(self.to, current_millis, self.duration);
        let mut next_update: Option<u64> = None;

        for ((&from, &to), &current) in zip(self.from.grb(), self.to.grb()).zip(current_color.grb())
        {
            let from = from as i32;
            let to = to as i32;
            let current = current as i32;

            let sign = (to - from).signum();
            if sign == 0 || current == to {
                continue;
            }

            // The update threshold is different when going up and when going down
            // due to integer division rounding down.
            let target = current + (sign + 1) / 2;

            let mut wait_time = ((target - from) * sign) as u64;
            wait_time *= self.duration;
            wait_time /= ((to - from) * sign) as u64;

            // Check that at time wait_time we indeed get the next color.
            // If not then the integer division inside interpolate introduces an off by one error.
            let next_at_wait_time = ((from as u64 * (self.duration - wait_time)
                + to as u64 * wait_time)
                / self.duration) as i32;
            if current + sign != next_at_wait_time {
                wait_time += 1
            }

            wait_time = wait_time - current_millis;

            if let Some(time) = next_update {
                if time > wait_time {
                    next_update = Some(wait_time);
                }
            } else {
                next_update = Some(wait_time);
            }
        }

        next_update
    }
}

impl Effect for MoveTo {
    fn step(&mut self) -> (Color, EffectStatus) {
        let dt = self.t0.elapsed().as_millis();
        if dt >= self.duration {
            return (self.to, EffectStatus::Finished);
        }

        let current_color = self.from.interpolate(self.to, dt, self.duration);

        // when is the next update?
        let status = self
            .millis_till_update(dt)
            .map(Duration::from_millis)
            .map(EffectStatus::InProgress)
            .unwrap_or(EffectStatus::Finished);

        (current_color, status)
    }
}
