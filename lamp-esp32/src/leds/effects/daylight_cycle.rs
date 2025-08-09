use chrono::{DateTime, Utc};

use super::{EffectEnum, EffectStatus};
use crate::leds::effects::MoveTo;
use crate::types::{Color, GlobalTime};

use super::Effect;

pub struct DaylightCycle {
    on_color: Color,
    time: GlobalTime,
    transition_minutes: [u64; 4],
    state: CycleState,
    init_effect: Option<MoveTo>,
}

enum CycleState {
    Rising(MoveTo),
    On,
    Falling(MoveTo),
    Off,
}

impl DaylightCycle {
    pub fn new(
        from_color: Color,
        on_color: Color,
        current_time: DateTime<Utc>,
        transition_minutes: [u64; 4],
    ) -> Self {
        let time = GlobalTime::now(current_time);

        let current_minute = time.day_minute();
        let to_minute = transition_minutes
            .iter()
            .position(|&m| current_minute <= m)
            .unwrap_or(0);

        let (move_to_color, state) = match to_minute {
            0 => (Color::black(), CycleState::Off),
            1 => {
                let color = Color::black().interpolate(
                    on_color,
                    current_minute - transition_minutes[0],
                    transition_minutes[1] - transition_minutes[0],
                );
                let state = CycleState::Rising(MoveTo::new(
                    color,
                    on_color,
                    time.secs_till_minute(transition_minutes[1]) * 1000,
                ));
                (color, state)
            }
            2 => (on_color, CycleState::On),
            3 => {
                let color = on_color.interpolate(
                    Color::black(),
                    current_minute - transition_minutes[0],
                    transition_minutes[1] - transition_minutes[0],
                );
                let state = CycleState::Falling(MoveTo::new(
                    color,
                    Color::black(),
                    time.secs_till_minute(transition_minutes[3]) * 1000,
                ));
                (color, state)
            }
            _ => (Color::black(), CycleState::Off), // This will not happen.
        };

        Self {
            on_color,
            time,
            transition_minutes,
            state,
            init_effect: Some(MoveTo::new(from_color, move_to_color, 10000)),
        }
    }
}

impl Into<EffectEnum> for DaylightCycle {
    fn into(self) -> EffectEnum {
        EffectEnum::DaylightCycle(self)
    }
}

impl Effect for DaylightCycle {
    fn step(&mut self) -> (Color, EffectStatus) {
        if let Some(effect) = &mut self.init_effect {
            let (color, status) = effect.step();

            if let EffectStatus::InProgress(_) = status {
                return (color, status);
            }
        }

        match &mut self.state {
            CycleState::Rising(move_to) => todo!(),
            CycleState::On => todo!(),
            CycleState::Falling(move_to) => todo!(),
            CycleState::Off => todo!(),
        }
    }
}
