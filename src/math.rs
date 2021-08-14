use crate::*;

pub const SQRT_3: f32 = 1.73205080757;

#[derive(Debug, Copy, Clone, Default)]
pub struct Point2 {
    pub x: f32,
    pub y: f32,
}

impl From<ggez::mint::Point2<f32>> for Point2 {
    fn from(val: ggez::mint::Point2<f32>) -> Self {
        Self { x: val.x, y: val.y }
    }
}

impl Into<ggez::mint::Point2<f32>> for Point2 {
    fn into(self) -> ggez::mint::Point2<f32> {
        ggez::mint::Point2 {
            x: self.x,
            y: self.y,
        }
    }
}

impl Point2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn zero(&self) -> bool {
        self.x == 0.0 && self.y == 0.0
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

impl std::ops::Mul<f32> for Point2 {
    type Output = Point2;

    fn mul(self, rhs: f32) -> Self::Output {
        Point2::new(rhs * self.x, rhs * self.y)
    }
}
