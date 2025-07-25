use core::iter::zip;

use embassy_time::Instant;
use esp_println::{print, println};

use super::effect::{Effect, EffectEnum, EffectStatus};
use super::Color;


pub struct MoveTo {
    from: Color,
    to: Color,
    t0: Instant,
    duration: u64,
}

impl MoveTo{
    pub fn new(from: Color, to: Color, duration: u64) -> Self {
        Self {
            from,
            to,
            t0: Instant::now(),
            duration,
        }
    }
}

impl Into<EffectEnum> for MoveTo {
    fn into(self) -> EffectEnum {
        EffectEnum::MoveTo(self)
    }
}

impl Effect for MoveTo {
    fn run(&mut self) -> (Color, EffectStatus) {
        let dt = self.t0.elapsed().as_millis();
        if dt >= self.duration {
            return (self.to, EffectStatus::Finished)
        }

        let current_color = self.from.interpolate(self.to, dt, self.duration);

        // when is the next update?
        let mut next_update: Option<u64> = None;
        for ((from, to), current) in zip(self.from.grb(), self.to.grb()).zip(current_color.grb()) {
            let sign = (*to as i32 - *from as i32).signum();
            if sign == 0 {
                continue;
            }

            let next = *current as i32 + sign;

            let mut wait_time = ((next - *from as i32) * sign) as u64;
            wait_time *= self.duration;
            wait_time /= ((*to as i32 - *from as i32) * sign) as u64;
            wait_time += 1; //buffer to avoid attemting an update before it actually happens
            
            wait_time = wait_time - dt;

            if let Some(time) = next_update {
                if time > wait_time {
                    next_update = Some(wait_time);
                }
            } else {
                next_update = Some(wait_time);
            }
        }   

        if let Some(time) = next_update {
            println!("{}", time);
            (current_color, EffectStatus::InProgress(time))
        } else {
            (current_color, EffectStatus::Finished)
        }
    }
}
