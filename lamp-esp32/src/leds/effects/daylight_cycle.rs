use chrono::{DateTime, Utc};

use crate::leds::effects::MoveTo;
use crate::types::{Color, GlobalInstant};
use super::{EffectEnum, EffectStatus}; 

use super::Effect;

pub struct DaylightCycle {
    on_color: Color,
    t0: GlobalInstant,
    transition_minutes: [u64; 4],
    transition_effect: MoveTo,
    state: CycleState,
}

enum CycleState {
    Rising,
    On,
    Falling,
    Off,
}

impl DaylightCycle {
    pub fn new(color: Color, current_time: DateTime<Utc>, transition_minutes: [u64; 4]) -> Self {
        Self {
            on_color: color,
            t0: GlobalInstant::now(current_time),
            transition_minutes,
            transition_effect: MoveTo::new(Color::default(), Color::default(), 0),
            state: CycleState::Off,
        }
    }
}

impl Into<EffectEnum> for DaylightCycle {
    fn into(self) -> EffectEnum {
        EffectEnum::DaylightCycle(self)
    }
}

impl Effect for DaylightCycle {
    fn run(&mut self) -> (Color, EffectStatus) {
        todo!()
    }
}