use core::iter::zip;

use embassy_time::{Duration, Instant};
use esp_println::println;

use super::{Effect, EffectEnum, EffectStatus};
use crate::types::Color;

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

    pub fn millis_till_update(
        &self, 
        current_millis: u64,
    ) -> Option<u64> {
        let current_color = self
            .from
            .interpolate(self.to, current_millis, self.duration);
        let mut next_update: Option<u64> = None;

        for ((&from, &to), &current) in zip(self.from.grb(), self.to.grb()).zip(current_color.grb())
        {
            let sign = (to as i32 - from as i32).signum();
            if sign == 0 || current == to {
                continue;
            }

            let next = current as i32 + sign;

            // The update threshold is different when going up and when going down
            // due to integer division rounding down.
            let target = if sign == 1 {
                next
            } else {
                current as i32
            };


            let mut wait_time = ((target - from as i32) * sign) as u64;
            wait_time *= self.duration;
            wait_time /= ((to as i32 - from as i32) * sign) as u64;

            // Check that at time wait_time we indeed get the next color.
            // If not then the integer division inside interpolate introduces an off by one error.
            let next_at_wait_time = ((from as u64 * (self.duration - wait_time)
                + to as u64 * wait_time)
                / self.duration) as i32;
            if next != next_at_wait_time {
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
        let next_update = self.millis_till_update(dt);

        if let Some(time) = next_update {
            println!("{}", time);
            (
                current_color,
                EffectStatus::InProgress(Duration::from_millis(time)),
            )
        } else {
            (current_color, EffectStatus::Finished)
        }
    }
}