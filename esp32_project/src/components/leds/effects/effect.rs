use super::MoveTo;
use super::Color;


pub trait Effect: Into<EffectEnum> {
    fn run(&mut self) -> (Color, EffectStatus);
}

pub enum EffectEnum {
    MoveTo(MoveTo),
}

impl EffectEnum {
    pub fn run(&mut self) -> (Color, EffectStatus) {
        match self {
            EffectEnum::MoveTo(effect) => effect.run(),
        }
    }
}

pub enum EffectStatus {
    InProgress(u64),
    Finished,
}