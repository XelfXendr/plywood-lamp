use embassy_time::Duration;

use super::{MoveTo, DaylightCycle};
use crate::types::Color;

pub trait Effect: Into<EffectEnum> {
    fn step(&mut self) -> (Color, EffectStatus);
}

pub enum EffectEnum {
    MoveTo(MoveTo),
    DaylightCycle(DaylightCycle),
}

impl EffectEnum {
    pub fn step(&mut self) -> (Color, EffectStatus) {
        match self {
            EffectEnum::MoveTo(effect) => effect.step(),
            EffectEnum::DaylightCycle(effect) => effect.step(),
        }
    }
}

pub enum EffectStatus {
    InProgress(Duration),
    Finished,
}
