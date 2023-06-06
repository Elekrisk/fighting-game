use std::ops::{Add, Sub, Mul};

use crate::fixedpoint::FixedPoint;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Vec2 {
    pub x: FixedPoint,
    pub y: FixedPoint,
}

impl Add for Vec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Vec2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul<FixedPoint> for Vec2 {
    type Output = Self;

    fn mul(self, rhs: FixedPoint) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}
