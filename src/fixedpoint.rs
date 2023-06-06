use std::{ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign}, fmt::Debug};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FixedPoint(i64);

impl Debug for FixedPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let whole_part = self.0 >> Self::DECIMALS;
        let frac_part = self.0 % (1 << Self::DECIMALS);
        write!(f, "{}.{}", whole_part, frac_part)
    }
}

impl FixedPoint {
    const DECIMALS: usize = 16;
    pub const ZERO: Self = Self(0);

    fn new_raw(raw: i64) -> Self {
        Self(raw)
    }
}

impl Add for FixedPoint {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for FixedPoint {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for FixedPoint {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for FixedPoint {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Mul for FixedPoint {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self((self.0 * rhs.0) >> Self::DECIMALS)
    }
}

impl MulAssign for FixedPoint {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl From<FixedPoint> for f32 {
    fn from(value: FixedPoint) -> Self {
        value.0 as f32 / 2usize.pow(FixedPoint::DECIMALS as _) as f32
    }
}

impl From<f32> for FixedPoint {
    fn from(value: f32) -> Self {
        let v = value * 2usize.pow(FixedPoint::DECIMALS as _) as f32;
        if v > i64::MAX as f32 {
            Self(i64::MAX)
        } else if v < i64::MIN as f32 {
            Self(i64::MIN)
        } else {
            Self(v as i64)
        }
    }
}
