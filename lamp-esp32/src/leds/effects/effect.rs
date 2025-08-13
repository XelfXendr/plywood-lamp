use embassy_time::Duration;

use super::{DaylightCycle, MoveTo};
use crate::types::Color;

pub enum EffectStatus {
    InProgress(Duration),
    Finished,
}

pub enum EffectEnum {
    MoveTo(MoveTo),
    DaylightCycle(DaylightCycle),
}

pub trait Effect: Into<EffectEnum> {
    fn step(&mut self) -> (Color, EffectStatus);
}

impl EffectEnum {
    pub fn step(&mut self) -> (Color, EffectStatus) {
        match self {
            EffectEnum::MoveTo(effect) => effect.step(),
            EffectEnum::DaylightCycle(effect) => effect.step(),
        }
    }
}
