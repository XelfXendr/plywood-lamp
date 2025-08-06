#[derive(Clone, Copy, Default)]
pub struct Color([u8; 3]);

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Color([g, r, b])
    }

    pub fn interpolate(&self, other: Self, value: u64, max: u64) -> Self {
        let mut new_color = self.0.clone();
        new_color.iter_mut().zip(other.0).for_each(|(a, b)| {
            *a = ((*a as u64 * (max - value) + b as u64 * value) / max) as u8
        });
        Color(new_color)
    }

    pub fn grb(&self) -> &[u8; 3] {
        &self.0
    }
}