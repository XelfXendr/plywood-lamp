use chrono::{DateTime, FixedOffset};
use embassy_time::Duration;

use super::{EffectEnum, EffectStatus};
use crate::leds::effects::MoveTo;
use crate::types::ranges::OverlapRanges;
use crate::types::{Color, global_time::GlobalTime};

use super::Effect;

pub struct DaylightCycle {
    on_color: Color,
    current_color: Color,
    time: GlobalTime<FixedOffset>,
    transition_ranges: OverlapRanges<u64, 4>,
    state: CycleState,
    init_effect: Option<MoveTo>,
}

#[derive(Debug)]
enum CycleState {
    Rising(MoveTo),
    On(Duration),
    Falling(MoveTo),
    Off(Duration),
}

impl DaylightCycle {
    pub fn new(
        from_color: Color,
        on_color: Color,
        current_time: DateTime<FixedOffset>,
        transition_ranges: OverlapRanges<u64, 4>,
    ) -> Self {
        let time = GlobalTime::at(current_time);
        let now = time.now();

        let current_minute = now.day_minute();

        let to_minute_idx = transition_ranges.which(current_minute);

        let (move_to_color, state) = match to_minute_idx {
            0 => (
                Color::black(),
                CycleState::Off(now.duration_till_minute(transition_ranges[0])),
            ),
            1 => {
                let color = Color::black().interpolate(
                    on_color,
                    current_minute - transition_ranges[0],
                    transition_ranges[1] - transition_ranges[0],
                );
                let state = CycleState::Rising(MoveTo::new(
                    color,
                    on_color,
                    now.duration_till_minute(transition_ranges[1]),
                ));
                (color, state)
            }
            2 => (
                on_color,
                CycleState::On(now.duration_till_minute(transition_ranges[2])),
            ),
            3 => {
                let color = on_color.interpolate(
                    Color::black(),
                    current_minute - transition_ranges[0],
                    transition_ranges[1] - transition_ranges[0],
                );
                let state = CycleState::Falling(MoveTo::new(
                    color,
                    Color::black(),
                    now.duration_till_minute(transition_ranges[3]),
                ));
                (color, state)
            }
            _ => unreachable!("There are only 4 ranges."),
        };

        Self {
            on_color,
            current_color: from_color,
            time,
            transition_ranges,
            state,
            init_effect: Some(MoveTo::new(
                from_color,
                move_to_color,
                Duration::from_secs(10),
            )),
        }
    }

    fn should_be_state(&self) -> CycleState {
        let now = self.time.now();
        let current_minute = now.day_minute();
        let current_range = self.transition_ranges.which(current_minute);
        let till_next = now.duration_till_minute(self.transition_ranges[current_range]);

        match current_range {
            0 => CycleState::Off(till_next),
            1 => CycleState::Rising(MoveTo::new(self.current_color, self.on_color, till_next)),
            2 => CycleState::On(till_next),
            3 => CycleState::Falling(MoveTo::new(self.current_color, Color::black(), till_next)),
            _ => unreachable!("There are only 4 ranges."),
        }
    }

    fn get_color_status(&mut self) -> (Color, EffectStatus) {
        if let Some(effect) = &mut self.init_effect {
            let (color, status) = effect.step();
            if let EffectStatus::InProgress(_) = status {
                return (color, status);
            }

            self.init_effect = None;
            match &mut self.state {
                CycleState::Rising(effect) => effect.reset_time(),
                CycleState::Falling(effect) => effect.reset_time(),
                _ => {}
            }
        }

        let step = match &mut self.state {
            CycleState::Rising(effect) => Some(effect.step()),
            CycleState::Falling(effect) => Some(effect.step()),
            _ => None,
        };

        if let Some((color, status)) = step
            && let EffectStatus::InProgress(_) = status
        {
            return (color, status);
        }

        // time to update our state
        self.state = self.should_be_state();

        let (color, mut status) = match &mut self.state {
            CycleState::Rising(effect) => effect.step(),
            CycleState::Falling(effect) => effect.step(),
            CycleState::On(duration) => (self.on_color, EffectStatus::InProgress(*duration)),
            CycleState::Off(duration) => (Color::black(), EffectStatus::InProgress(*duration)),
        };

        if let EffectStatus::Finished = status {
            status = EffectStatus::InProgress(Duration::from_secs(1));
        }

        (color, status)
    }
}

impl Into<EffectEnum> for DaylightCycle {
    fn into(self) -> EffectEnum {
        EffectEnum::DaylightCycle(self)
    }
}

impl Effect for DaylightCycle {
    fn step(&mut self) -> (Color, EffectStatus) {
        let (color, status) = self.get_color_status();

        self.current_color = color;

        (color, status)
    }
}
