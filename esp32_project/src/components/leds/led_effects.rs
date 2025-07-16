use embassy_time::Instant;

#[derive(Clone, Copy)]
pub struct Color([u8; 3]);

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Color([r, g, b])
    }

    pub fn interpolate(&self, other: Self, value: i64, max: i64) -> Self {
        let mut new_color = self.0.clone();
        new_color.iter_mut().zip(other.0).for_each(|(a, b)| {
            *a = ((*a as i64 * (max - value) + b as i64 * value) / max) as u8
        });
        Color(new_color)
    }

    pub fn rgb(&self) -> [u8; 3] {
        self.0
    }

    pub fn grb(&self) -> [u8; 3] {
        [self.0[1], self.0[0], self.0[2]]
    }
}

pub enum EffectStatus {
    InProgress(Color, u64),
    Finished(Color),
}

impl EffectStatus {
    pub fn color(&self) -> Color {
        match self {
            EffectStatus::InProgress(color, _) => *color,
            EffectStatus::Finished(color) => *color,
        }
    }
}

pub trait Effect {
    fn run(&mut self) -> EffectStatus;
}

pub enum Effects {
    MoveTo(MoveTo),
}

impl Effects {
    pub fn move_to(from: Color, to: Color, duration: u64) -> Self {
        Effects::MoveTo(MoveTo::new(from, to, duration))
    }
}

impl Effect for Effects {
    fn run(&mut self) -> EffectStatus {
        match self {
            Effects::MoveTo(effect) => effect.run(),
        }
    }
}

struct MoveTo {
    from: Color,
    to: Color,
    t0: Instant,
    duration: u64,
}

impl MoveTo{
    fn new(from: Color, to: Color, duration: u64) -> Self {
        Self {
            from,
            to,
            t0: Instant::now(),
            duration,
        }
    }
}

impl Effect for MoveTo {
    fn run(&mut self) -> EffectStatus {
        let dt = self.t0.elapsed().as_millis();
        if dt >= self.duration {
            return EffectStatus::Finished(self.to)
        }

        let current_color = self.from.interpolate(self.to, dt as i64, self.duration as i64);
        EffectStatus::InProgress(current_color, 20)
    }
}
