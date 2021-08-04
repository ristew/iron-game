use crate::*;

pub const SQRT_3: f32 = 1.73205080757;

#[derive(Debug, Copy, Clone, Default)]
pub struct Point2 {
    pub x: f32,
    pub y: f32,
}

impl Point2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
        }
    }
}

impl std::ops::Add for Point2 {
    type Output = Point2;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl std::ops::AddAssign for Point2 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}
