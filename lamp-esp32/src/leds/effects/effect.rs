use super::{MoveTo, DaylightCycle};
use crate::types::Color;

pub trait Effect: Into<EffectEnum> {
    fn run(&mut self) -> (Color, EffectStatus);
}

pub enum EffectEnum {
    MoveTo(MoveTo),
    DaylightCycle(DaylightCycle),
}

impl EffectEnum {
    pub fn run(&mut self) -> (Color, EffectStatus) {
        match self {
            EffectEnum::MoveTo(effect) => effect.run(),
            EffectEnum::DaylightCycle(effect) => effect.run(),
        }
    }
}

pub enum EffectStatus {
    InProgress(u64),
    Finished,
}
